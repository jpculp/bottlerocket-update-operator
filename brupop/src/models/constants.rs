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
