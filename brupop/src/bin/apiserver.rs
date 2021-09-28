use actix_web::{
    get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use kube::api::Api;
use snafu::ResultExt;

use brupop::error::{self, Result};
use brupop::k8s;
use brupop::models::{
    api::UpsertBottlerocketNodeRequest,
    constants,
    node::{BottlerocketNode, BottlerocketNodeSpec, BottlerocketNodeStatus},
};

#[derive(Clone)]
struct APIServerSettings {
    k8s_client: kube::client::Client,
}

/// Implements a shallow health check for the HTTP service.
#[get("/ping")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[post("/bottlerocket-node-resource")]
async fn upsert_bottlerocket_node_resource(
    _req: HttpRequest,
    settings: web::Data<APIServerSettings>,
    create_request: web::Json<UpsertBottlerocketNodeRequest>,
) -> Result<impl Responder> {
    let k8s_client = &settings.k8s_client;

    let node_spec: BottlerocketNodeSpec = Default::default();
    // TODO add initial node state per below
    // let node_status = create_request.node_status.clone();
    // TODO add OwnerReference to created resource.
    let br_node = BottlerocketNode::new(&create_request.node_name, node_spec);

    let api: Api<BottlerocketNode> = Api::namespaced(k8s_client.clone(), constants::NAMESPACE);

    k8s::create_or_update(&api, br_node, "BottlerocketNode Custom Resource").await?;

    Ok(HttpResponse::Ok().body("Hello, world!"))
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let k8s_client = kube::client::Client::try_default()
        .await
        .context(error::ClientCreate)?;

    HttpServer::new(move || {
        let api_server_settings = APIServerSettings {
            k8s_client: k8s_client.clone(),
        };
        App::new()
            .wrap(middleware::Logger::default().exclude("/ping"))
            .data(api_server_settings)
            .service(upsert_bottlerocket_node_resource)
            .service(health_check)
    })
    .bind("127.0.0.1:8080")
    .context(error::HttpServerError)?
    .run()
    .await
    .context(error::HttpServerError)
}
