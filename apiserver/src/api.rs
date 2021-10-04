use actix_web::{
    get, middleware, post,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use kube::api::Api;
use serde_json::json;
use snafu::ResultExt;

use crate::error::{self, Result};
use crate::k8s;
use crate::models::{
    constants,
    node::{BottlerocketNode, BottlerocketNodeSpec, BottlerocketNodeStatus},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBottlerocketNodeRequest {
    pub node_name: String,
    pub node_status: BottlerocketNodeStatus,
}

impl UpsertBottlerocketNodeRequest {
    pub fn new(node_name: String, node_status: BottlerocketNodeStatus) -> Self {
        UpsertBottlerocketNodeRequest {
            node_name,
            node_status,
        }
    }
}

#[derive(Clone)]
pub struct APIServerSettings {
    pub k8s_client: kube::client::Client,
}

impl APIServerSettings {
    pub fn new(k8s_client: kube::client::Client) -> Self {
        APIServerSettings { k8s_client }
    }
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
    return Ok(HttpResponse::Ok().body(format!("{}", json!(&br_node))));

    let api: Api<BottlerocketNode> = Api::namespaced(k8s_client.clone(), constants::NAMESPACE);

    k8s::create_or_update(&api, &br_node, "BottlerocketNode Custom Resource").await?;

    Ok(HttpResponse::Ok().body(format!("{}", json!(&br_node))))
}

#[derive(Clone)]
pub struct APIServer {
    settings: APIServerSettings,
}

impl APIServer {
    pub fn new(settings: APIServerSettings) -> Self {
        APIServer { settings }
    }

    pub async fn run_server(self) -> Result<()> {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default().exclude("/ping"))
                .app_data(Data::new(self.settings.clone()))
                .service(upsert_bottlerocket_node_resource)
                .service(health_check)
        })
        .bind("127.0.0.1:8080")
        .context(error::HttpServerError)?
        .run()
        .await
        .context(error::HttpServerError)
    }
}
