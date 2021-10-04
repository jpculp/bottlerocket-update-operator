/// Helper macro to avoid retyping the base domain-like name of our system when creating further
/// string constants from it. When given no parameters, this returns the base domain-like name of
/// the system. When given a string literal parameter it adds `/parameter` to the end.
macro_rules! brupop {
    () => {
        "brupop.bottlerocket.aws"
    };
    ($s:literal) => {
        concat!(brupop!(), "/", $s)
    };
}

pub const API_VERSION: &str = brupop!("v1");
pub const NAMESPACE: &str = "brupop-bottlerocket-aws";
pub const BRUPOP: &str = brupop!();

// Label keys
pub const LABEL_COMPONENT: &str = brupop!("component");

// Standard tags https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/
pub const APP_NAME: &str = "app.kubernetes.io/name";
pub const APP_INSTANCE: &str = "app.kubernetes.io/instance";
pub const APP_COMPONENT: &str = "app.kubernetes.io/component";
pub const APP_PART_OF: &str = "app.kubernetes.io/part-of";
pub const APP_MANAGED_BY: &str = "app.kubernetes.io/managed-by";
pub const APP_CREATED_BY: &str = "app.kubernetes.io/created-by";
