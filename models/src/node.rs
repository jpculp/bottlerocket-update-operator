use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
