use apiserver::api::{CreateBottlerocketNodeRequest, UpdateBottlerocketNodeRequest};
use models::node::{BottlerocketNodeState, BottlerocketNodeStatus};

#[tokio::main]
async fn main() {
    let node_req = CreateBottlerocketNodeRequest {
        node_name: "ip-192-168-23-112.us-west-2.compute.internal".to_string(),
        node_uid: "9df49eac-1d7d-4e25-b78a-608894ce0ce0".to_string(),
    };

    let client = reqwest::Client::new();
    client
        .get("http://52.37.161.91:30000/ping")
        .send()
        .await
        .unwrap();

    let request_body = serde_json::to_string(&node_req).unwrap();
    println!("Sending POST request with body {}", &request_body);
    let response = client
        .post("http://52.37.161.91:30000/bottlerocket-node-resource")
        .json(&node_req)
        .send()
        .await
        .unwrap();

    let node_req = UpdateBottlerocketNodeRequest {
        node_name: "ip-192-168-23-112.us-west-2.compute.internal".to_string(),
        node_uid: "9df49eac-1d7d-4e25-b78a-608894ce0ce0".to_string(),
        node_status: BottlerocketNodeStatus::new(
            "1.2.1".to_string(),
            vec!["1.3.0".to_string()],
            BottlerocketNodeState::default(),
        ),
    };

    let request_body = serde_json::to_string(&node_req).unwrap();
    println!("Sending PUT request with body {}", &request_body);

    let response = client
        .put("http://52.37.161.91:30000/bottlerocket-node-resource")
        .json(&node_req)
        .send()
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap());
}
