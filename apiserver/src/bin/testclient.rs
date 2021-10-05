use apiserver::api::UpsertBottlerocketNodeRequest;
use models::node::{BottlerocketNodeState, BottlerocketNodeStatus};

#[tokio::main]
async fn main() {
    let node_req = UpsertBottlerocketNodeRequest::new(
        "Test Node".to_string(),
        "asdfasdfasdf".to_string(),
        BottlerocketNodeStatus::new("0.0.1".to_owned(), vec![], BottlerocketNodeState::default()),
    );

    let request_body = serde_json::to_string(&node_req).unwrap();
    println!("Sending request with body {}", &request_body);

    let client = reqwest::Client::new();
    client
        .get("http://127.0.0.1:8080/ping")
        .send()
        .await
        .unwrap();

    let response = client
        .post("http://127.0.0.1:8080/bottlerocket-node-resource")
        .json(&node_req)
        .send()
        .await
        .unwrap();

    println!("{}", response.text().await.unwrap());
}
