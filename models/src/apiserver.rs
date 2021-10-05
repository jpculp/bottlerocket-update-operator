use crate::constants::{
    APP_COMPONENT, APP_MANAGED_BY, APP_PART_OF, BRUPOP, LABEL_COMPONENT, NAMESPACE,
};
use k8s_openapi::api::apps::v1::{
    Deployment, DeploymentSpec, DeploymentStrategy, RollingUpdateDeployment,
};
use k8s_openapi::api::core::v1::{
    Affinity, Container, ContainerPort, LocalObjectReference, NodeAffinity, NodeSelector,
    NodeSelectorRequirement, NodeSelectorTerm, PodSpec, PodTemplateSpec, ServiceAccount,
};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::ObjectMeta;
use maplit::btreemap;

const BRUPOP_APISERVER_SERVICE_ACCOUNT: &str = "brupop-controller-service-account";
const BRUPOP_APISERVER_CLUSTER_ROLE: &str = "brupop-controller-role";

/// Defines the brupop-apiserver service account
pub fn apiserver_service_account() -> ServiceAccount {
    ServiceAccount {
        metadata: ObjectMeta {
            name: Some(BRUPOP_APISERVER_SERVICE_ACCOUNT.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            annotations: Some(btreemap! {
                "kubernetes.io/service-account.name".to_string() => BRUPOP_APISERVER_SERVICE_ACCOUNT.to_string()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Defines the brupop-apiserver cluster role
pub fn apiserver_cluster_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(BRUPOP_APISERVER_CLUSTER_ROLE.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
            PolicyRule {
                api_groups: Some(vec![BRUPOP.to_string()]),
                resources: Some(vec![
                    "bottlerocketnodes".to_string(),
                    "bottlerocketnodes/status".to_string(),
                ]),
                verbs: vec!["create", "get", "list", "patch", "update", "watch"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                ..Default::default()
            },
            PolicyRule {
                api_groups: Some(vec!["apps".to_string()]),
                resources: Some(vec!["deployments".to_string()]),
                verbs: vec![
                    "create",
                    "delete",
                    "deletecollection",
                    "get",
                    "list",
                    "patch",
                    "update",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Defines the brupop-apiserver cluster role binding
pub fn apiserver_cluster_role_binding() -> ClusterRoleBinding {
    ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some("brupop-apiserver-role-binding".to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: BRUPOP_APISERVER_CLUSTER_ROLE.to_string(),
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: BRUPOP_APISERVER_SERVICE_ACCOUNT.to_string(),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        }]),
    }
}

/// Defines the brupop-apiserver deployment
pub fn apiserver_deployment(
    apiserver_image: String,
    image_pull_secret: Option<String>,
) -> Deployment {
    let image_pull_secrets =
        image_pull_secret.map(|secret| vec![LocalObjectReference { name: Some(secret) }]);

    Deployment {
        metadata: ObjectMeta {
            labels: Some(
                btreemap! {
                    APP_COMPONENT => "apiserver",
                    APP_MANAGED_BY => "brupop",
                    APP_PART_OF => "brupop",
                    LABEL_COMPONENT => "apiserver",
                }
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ),
            name: Some("brupop-apiserver".to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(3),
            selector: LabelSelector {
                match_labels: Some(
                    btreemap! { LABEL_COMPONENT.to_string() => "apiserver".to_string()},
                ),
                ..Default::default()
            },
            strategy: Some(DeploymentStrategy {
                rolling_update: Some(RollingUpdateDeployment {
                    max_unavailable: Some(IntOrString::String("33%".to_string())),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(btreemap! {
                        LABEL_COMPONENT.to_string() => "apiserver".to_string(),
                    }),
                    namespace: Some(NAMESPACE.to_string()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    affinity: Some(Affinity {
                        node_affinity: Some(NodeAffinity {
                            required_during_scheduling_ignored_during_execution: Some(
                                NodeSelector {
                                    node_selector_terms: vec![NodeSelectorTerm {
                                        match_expressions: Some(vec![
                                            NodeSelectorRequirement {
                                                key: "kubernetes.io/os".to_string(),
                                                operator: "In".to_string(),
                                                values: Some(vec!["linux".to_string()]),
                                            },
                                            NodeSelectorRequirement {
                                                key: "kubernetes.io/arch".to_string(),
                                                operator: "In".to_string(),
                                                // TODO make sure the pod works on arm64 before adding arm64 here.
                                                // https://github.com/bottlerocket-os/bottlerocket-test-system/issues/90
                                                values: Some(vec![
                                                    "amd64".to_string(),
                                                    "arm64".to_string(),
                                                ]),
                                            },
                                        ]),
                                        ..Default::default()
                                    }],
                                },
                            ),
                            ..Default::default()
                        }),
                        // TODO: Potentially add pods we want to avoid here, e.g. update operator agent pod
                        pod_anti_affinity: None,
                        ..Default::default()
                    }),
                    containers: vec![Container {
                        image: Some(apiserver_image),
                        image_pull_policy: None,
                        name: "apiserver".to_string(),
                        ports: Some(vec![ContainerPort {
                            container_port: 8080,
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    image_pull_secrets,
                    service_account_name: Some(BRUPOP_APISERVER_SERVICE_ACCOUNT.to_string()),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}
