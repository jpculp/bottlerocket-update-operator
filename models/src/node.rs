use crate::constants;

use async_trait::async_trait;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum BottlerocketNodeError {
    #[snafu(display(
        "Unable to create BottlerocketNode ({}, {}): {}",
        selector.node_name,
        selector.node_uid,
        source
    ))]
    CreateBottlerocketNode {
        source: Box<dyn std::error::Error>,
        selector: BottlerocketNodeSelector,
    },

    #[snafu(display(
        "Unable to update BottlerocketNode status ({}, {}): {}",
        selector.node_name,
        selector.node_uid,
        source
    ))]
    UpdateBottlerocketNodeStatus {
        source: Box<dyn std::error::Error>,
        selector: BottlerocketNodeSelector,
    },
}

/// BottlerocketNodeState represents a node's state in the update state machine.
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, JsonSchema)]
pub enum BottlerocketNodeState {
    WaitingForUpdate,
    PreparingToUpdate,
    PerformingUpdate,
    RebootingToUpdate,
    MonitoringUpdate,
}

impl Default for BottlerocketNodeState {
    fn default() -> Self {
        BottlerocketNodeState::WaitingForUpdate
    }
}

/// The `BottlerocketNodeSpec` can be used to drive a node through the update state machine. A node
/// linearly drives towards the desired state. The brupop controller updates the spec to specify a node's desired state,
/// and the host agent drives state changes forward and updates the `BottlerocketNodeStatus`.
#[derive(
    Clone, CustomResource, Serialize, Deserialize, Debug, Default, Eq, PartialEq, JsonSchema,
)]
#[kube(
    derive = "Default",
    derive = "PartialEq",
    group = "brupop.bottlerocket.aws",
    kind = "BottlerocketNode",
    namespaced,
    plural = "bottlerocketnodes",
    shortname = "brn",
    singular = "bottlerocketnode",
    status = "BottlerocketNodeStatus",
    version = "v1"
)]
pub struct BottlerocketNodeSpec {
    state: BottlerocketNodeState,
    version: Option<String>,
}

/// `BottlerocketNodeStatus` surfaces the current state of a bottlerocket node. The status is updated by the host agent,
/// while the spec is updated by the brupop controller.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
pub struct BottlerocketNodeStatus {
    current_version: String,
    available_versions: Vec<String>,
    current_state: BottlerocketNodeState,
}

impl BottlerocketNodeStatus {
    pub fn new(
        current_version: String,
        available_versions: Vec<String>,
        current_state: BottlerocketNodeState,
    ) -> Self {
        BottlerocketNodeStatus {
            current_version,
            available_versions,
            current_state,
        }
    }
}

/// Indicates the specific k8s node that BottlerocketNode object is associated with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BottlerocketNodeSelector {
    pub node_name: String,
    pub node_uid: String,
}

fn node_resource_name(node_selector: &BottlerocketNodeSelector) -> String {
    format!("brn-{}", node_selector.node_name)
}

#[async_trait]
/// A trait providing an interface to interact with BottlerocketNode objects. This is provided as a trait
/// in order to allow mocks to be used for testing purposes.
pub trait BottlerocketNodeClient: Clone + Sync + Send + Sized {
    /// Create a BottlerocketNode object for the specified node.
    async fn create_node(
        &self,
        selector: &BottlerocketNodeSelector,
    ) -> Result<BottlerocketNode, BottlerocketNodeError>;
    async fn update_node_status(
        &self,
        selector: &BottlerocketNodeSelector,
        status: &BottlerocketNodeStatus,
    ) -> Result<(), BottlerocketNodeError>;
    async fn update_node_spec(
        &self,
        selector: &BottlerocketNodeSelector,
        spec: &BottlerocketNodeSpec,
    ) -> Result<(), BottlerocketNodeError>;
}

#[derive(Clone)]
/// Concrete implementation of the `BottlerocketNodeClient` trait. This implementation will almost
/// certainly be used in any case that isn't a unit test.
pub struct K8SBottlerocketNodeClient {
    k8s_client: kube::client::Client,
}

impl K8SBottlerocketNodeClient {
    pub fn new(k8s_client: kube::client::Client) -> Self {
        K8SBottlerocketNodeClient { k8s_client }
    }
}

#[async_trait]
impl BottlerocketNodeClient for K8SBottlerocketNodeClient {
    async fn create_node(
        &self,
        selector: &BottlerocketNodeSelector,
    ) -> Result<BottlerocketNode, BottlerocketNodeError> {
        let br_node = BottlerocketNode {
            metadata: ObjectMeta {
                name: Some(node_resource_name(&selector)),
                owner_references: Some(vec![OwnerReference {
                    api_version: "v1".to_string(),
                    kind: "BottlerocketNode".to_string(),
                    name: selector.node_name.clone(),
                    uid: selector.node_uid.clone(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            spec: BottlerocketNodeSpec::default(),
            ..Default::default()
        };

        Api::namespaced(self.k8s_client.clone(), constants::NAMESPACE)
            .create(&PostParams::default(), &br_node)
            .await
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            .context(CreateBottlerocketNode {
                selector: selector.clone(),
            })?;

        Ok(br_node)
    }

    async fn update_node_status(
        &self,
        selector: &BottlerocketNodeSelector,
        status: &BottlerocketNodeStatus,
    ) -> Result<(), BottlerocketNodeError> {
        let br_node_patch = json!({
            "apiVersion": constants::API_VERSION,
            "kind": "BottlerocketNode",
            "status": status
        });

        let api: Api<BottlerocketNode> =
            Api::namespaced(self.k8s_client.clone(), constants::NAMESPACE);

        api.patch_status(
            &node_resource_name(&selector),
            &PatchParams::default(),
            &Patch::Merge(&br_node_patch),
        )
        .await
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
        .context(UpdateBottlerocketNodeStatus {
            selector: selector.clone(),
        })?;

        Ok(())
    }

    async fn update_node_spec(
        &self,
        _selector: &BottlerocketNodeSelector,
        _spec: &BottlerocketNodeSpec,
    ) -> Result<(), BottlerocketNodeError> {
        unimplemented!()
    }
}
