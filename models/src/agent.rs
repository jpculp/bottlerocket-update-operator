use crate::brupop_labels;
use crate::constants::{
    AGENT, AGENT_NAME, APISERVER_SERVICE_NAME, APP_COMPONENT, APP_MANAGED_BY, APP_PART_OF, BRUPOP,
    BRUPOP_INTERFACE_VERSION, LABEL_BRUPOP_INTERFACE_NAME, LABEL_COMPONENT, NAMESPACE,
    TLS_KEY_MOUNT_PATH,
};
use k8s_openapi::api::apps::v1::{DaemonSet, DaemonSetSpec};
use k8s_openapi::api::core::v1::{
    Affinity, Container, EnvVar, EnvVarSource, HostPathVolumeSource, LocalObjectReference,
    NodeAffinity, NodeSelector, NodeSelectorRequirement, NodeSelectorTerm, ObjectFieldSelector,
    PodSpec, PodTemplateSpec, ProjectedVolumeSource, ResourceRequirements, SELinuxOptions,
    SecretVolumeSource, SecurityContext, ServiceAccount, ServiceAccountTokenProjection, Volume,
    VolumeMount, VolumeProjection,
};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::api::ObjectMeta;
use maplit::btreemap;

const BRUPOP_AGENT_SERVICE_ACCOUNT: &str = "brupop-agent-service-account";
const BRUPOP_AGENT_CLUSTER_ROLE: &str = "brupop-agent-role";
const BRUPOP_APISERVER_CLIENT_CERT_SECRET_NAME: &str = "brupop-apiserver-client-certificate";

pub const TOKEN_PROJECTION_MOUNT_PATH: &str = "/var/run/secrets/tokens/";
pub const AGENT_TOKEN_PATH: &str = "bottlerocket-agent-service-account-token";

/// Defines the brupop-agent service account
pub fn agent_service_account() -> ServiceAccount {
    ServiceAccount {
        metadata: ObjectMeta {
            labels: Some(brupop_labels!(AGENT)),
            name: Some(BRUPOP_AGENT_SERVICE_ACCOUNT.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            annotations: Some(btreemap! {
                "kubernetes.io/service-account.name".to_string() => BRUPOP_AGENT_SERVICE_ACCOUNT.to_string()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Defines the brupop-agent cluster role
pub fn agent_cluster_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            labels: Some(brupop_labels!(AGENT)),
            name: Some(BRUPOP_AGENT_CLUSTER_ROLE.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
            PolicyRule {
                api_groups: Some(vec!["".to_string()]),
                resources: Some(vec!["nodes".to_string()]),
                verbs: vec!["get", "list", "watch"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                ..Default::default()
            },
            PolicyRule {
                api_groups: Some(vec!["brupop.bottlerocket.aws".to_string()]),
                resources: Some(vec![
                    "bottlerocketshadows".to_string(),
                    "bottlerocketshadows/status".to_string(),
                ]),
                verbs: vec!["get", "list", "watch"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Defines the brupop-agent cluster role binding
pub fn agent_cluster_role_binding() -> ClusterRoleBinding {
    ClusterRoleBinding {
        metadata: ObjectMeta {
            labels: Some(brupop_labels!(AGENT)),
            name: Some("brupop-agent-role-binding".to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: BRUPOP_AGENT_CLUSTER_ROLE.to_string(),
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: BRUPOP_AGENT_SERVICE_ACCOUNT.to_string(),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        }]),
    }
}

/// Defines the brupop-agent DaemonSet
pub fn agent_daemonset(
    agent_image: String,
    image_pull_secret: Option<String>,
    exclude_from_lb_wait_time: u64,
    apiserver_service_port: String,
) -> DaemonSet {
    let image_pull_secrets =
        image_pull_secret.map(|secret| vec![LocalObjectReference { name: Some(secret) }]);

    DaemonSet {
        metadata: ObjectMeta {
            labels: Some(brupop_labels!(AGENT)),
            name: Some(AGENT_NAME.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        spec: Some(DaemonSetSpec {
            selector: LabelSelector {
                match_labels: Some(btreemap! { LABEL_COMPONENT.to_string() => AGENT.to_string()}),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(btreemap! {
                        LABEL_COMPONENT.to_string() => AGENT.to_string(),
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
                                                key: LABEL_BRUPOP_INTERFACE_NAME.to_string(),
                                                operator: "In".to_string(),
                                                values: Some(vec![
                                                    BRUPOP_INTERFACE_VERSION.to_string()
                                                ]),
                                            },
                                            NodeSelectorRequirement {
                                                key: "kubernetes.io/arch".to_string(),
                                                operator: "In".to_string(),
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
                        ..Default::default()
                    }),
                    containers: vec![Container {
                        image: Some(agent_image),
                        name: BRUPOP.to_string(),
                        image_pull_policy: None,
                        command: Some(vec!["./agent".to_string()]),
                        env: Some(vec![
                            EnvVar {
                                name: "MY_NODE_NAME".to_string(),
                                value_from: Some(EnvVarSource {
                                    field_ref: Some(ObjectFieldSelector {
                                        field_path: "spec.nodeName".to_string(),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "EXCLUDE_FROM_LB_WAIT_TIME_IN_SEC".to_string(),
                                value: Some(exclude_from_lb_wait_time.to_string()),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "APISERVER_SERVICE_PORT".to_string(),
                                value: Some(apiserver_service_port),
                                ..Default::default()
                            },
                        ]),
                        resources: Some(ResourceRequirements {
                            requests: Some(btreemap! {
                                "memory".to_string() => Quantity("8Mi".to_string()),
                                "cpu".to_string() => Quantity("5m".to_string()),
                            }),
                            limits: Some(btreemap! {
                                "memory".to_string() => Quantity("50Mi".to_string()),
                                "cpu".to_string() => Quantity("10m".to_string()),
                            }),
                        }),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                name: "bottlerocket-api-socket".to_string(),
                                mount_path: "/run/api.sock".to_string(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "bottlerocket-apiclient".to_string(),
                                mount_path: "/bin/apiclient".to_string(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "bottlerocket-agent-service-account-token".to_string(),
                                mount_path: TOKEN_PROJECTION_MOUNT_PATH.to_string(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "bottlerocket-tls-keys".to_string(),
                                mount_path: TLS_KEY_MOUNT_PATH.to_string(),
                                ..Default::default()
                            },
                        ]),
                        security_context: Some(SecurityContext {
                            se_linux_options: Some(SELinuxOptions {
                                role: Some("system_r".to_string()),
                                type_: Some("super_t".to_string()),
                                user: Some("system_u".to_string()),
                                level: Some("s0".to_string()),
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    service_account_name: Some(BRUPOP_AGENT_SERVICE_ACCOUNT.to_string()),
                    image_pull_secrets,
                    volumes: Some(vec![
                        Volume {
                            name: "bottlerocket-api-socket".to_string(),
                            host_path: Some(HostPathVolumeSource {
                                path: "/run/api.sock".to_string(),
                                type_: Some("Socket".to_string()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "bottlerocket-apiclient".to_string(),
                            host_path: Some(HostPathVolumeSource {
                                path: "/bin/apiclient".to_string(),
                                type_: Some("File".to_string()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "bottlerocket-agent-service-account-token".to_string(),
                            projected: Some(ProjectedVolumeSource {
                                sources: Some(vec![VolumeProjection {
                                    service_account_token: Some(ServiceAccountTokenProjection {
                                        path: AGENT_TOKEN_PATH.to_string(),
                                        audience: Some(APISERVER_SERVICE_NAME.to_string()),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }]),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "bottlerocket-tls-keys".to_string(),
                            secret: Some(SecretVolumeSource {
                                secret_name: Some(
                                    BRUPOP_APISERVER_CLIENT_CERT_SECRET_NAME.to_string(),
                                ),
                                optional: Some(false),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}
