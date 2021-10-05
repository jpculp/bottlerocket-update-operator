use apiserver::api::{CreateBottlerocketNodeRequest, UpdateBottlerocketNodeRequest};
use models::node::{BottlerocketNodeState, BottlerocketNodeStatus};

#[tokio::main]
async fn main() {
    let server_domain = "52.24.238.246:30000";
    let node_name = "ip-192-168-66-198.us-west-2.compute.internal";
    let node_uid = "623ca9b3-bfc6-4c0f-b5d8-d8bd320a0a2f";

    let node_req = CreateBottlerocketNodeRequest {
        node_name: node_name.to_string(),
        node_uid: node_uid.to_string(),
    };

    let client = reqwest::Client::new();
    client
        .get(format!("http://{}/ping", &server_domain))
        .send()
        .await
        .unwrap();

    let request_body = serde_json::to_string(&node_req).unwrap();
    println!("Sending POST request with body {}", &request_body);
    let _response = client
        .post(format!(
            "http://{}/bottlerocket-node-resource",
            &server_domain
        ))
        .json(&node_req)
        .send()
        .await
        .unwrap();

    let node_req = UpdateBottlerocketNodeRequest {
        node_name: node_name.to_string(),
        node_uid: node_uid.to_string(),
        node_status: BottlerocketNodeStatus::new(
            "1.2.1".to_string(),
            vec!["1.3.0".to_string()],
            BottlerocketNodeState::default(),
        ),
    };

    let request_body = serde_json::to_string(&node_req).unwrap();
    println!("Sending PUT request with body {}", &request_body);

    let response = client
        .put(format!(
            "http://{}/bottlerocket-node-resource",
            &server_domain
        ))
        .json(&node_req)
        .send()
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap());
}
