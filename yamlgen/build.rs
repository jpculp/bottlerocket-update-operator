/*!

The custom resource definitions are modeled as Rust structs. Here we generate
the corresponding k8s yaml files.

!*/

use brupop::models::node::BottlerocketNode;
use kube::CustomResourceExt;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const YAMLGEN_DIR: &str = env!("CARGO_MANIFEST_DIR");
const HEADER: &str = "# This file is generated. Do not edit.\n";

fn main() {
    // Re-run this build script if the model changes.
    println!("cargo:rerun-if-changed=../client/src/model");
    println!("cargo:rerun-if-changed=../client/src/system");
    // Re-run the yaml generation if these variables change
    println!("cargo:rerun-if-env-changed=BRUPOP_CONTROLLER_IMAGE");
    println!("cargo:rerun-if-env-changed=BRUPOP_CONTROLLER_IMAGE_PULL_SECRET");

    dotenv::dotenv().ok();

    let path = PathBuf::from(YAMLGEN_DIR)
        .join("deploy")
        .join("bottlerocket-node-crd.yaml");
    let mut bottlerocket_node_crd = File::create(&path).unwrap();

    // testsys-crd related K8S manifest
    bottlerocket_node_crd.write_all(HEADER.as_bytes()).unwrap();
    serde_yaml::to_writer(&bottlerocket_node_crd, &BottlerocketNode::crd()).unwrap();
}
