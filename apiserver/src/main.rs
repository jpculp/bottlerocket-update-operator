use apiserver::api::{APIServer, APIServerSettings};
use apiserver::error::{self, Result};

use snafu::ResultExt;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let k8s_client = kube::client::Client::try_default()
        .await
        .context(error::ClientCreate)?;

    let settings = APIServerSettings::new(k8s_client);
    APIServer::new(settings).run_server().await
}
