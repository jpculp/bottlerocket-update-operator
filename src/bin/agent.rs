use brupop::agent::BrupopAgent;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut agent = BrupopAgent::new();
    agent.ensure_node_custom_resource_exists().await;
    agent.run().await;
}
