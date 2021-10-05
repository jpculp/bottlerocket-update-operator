use actix_web::{
    get, middleware, post, put,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams};
use serde_json::json;
use snafu::ResultExt;

use crate::error::{self, Result};
use models::{
    constants,
    node::{BottlerocketNode, BottlerocketNodeSpec, BottlerocketNodeStatus},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBottlerocketNodeRequest {
    pub node_name: String,
    pub node_uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBottlerocketNodeRequest {
    pub node_name: String,
    pub node_uid: String,
    pub node_status: BottlerocketNodeStatus,
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
async fn create_bottlerocket_node_resource(
    _req: HttpRequest,
    settings: web::Data<APIServerSettings>,
    create_request: web::Json<CreateBottlerocketNodeRequest>,
) -> Result<impl Responder> {
    let k8s_client = &settings.k8s_client;

    let brn_name = format!("brn-{}", &create_request.node_name);

    let br_node = BottlerocketNode {
        metadata: ObjectMeta {
            name: Some(brn_name),
            owner_references: Some(vec![OwnerReference {
                api_version: "v1".to_string(),
                kind: "BottlerocketNode".to_string(),
                name: create_request.node_name.clone(),
                uid: create_request.node_uid.clone(),
                ..Default::default()
            }]),
            ..Default::default()
        },
        spec: BottlerocketNodeSpec::default(),
        ..Default::default()
    };

    Api::namespaced(k8s_client.clone(), constants::NAMESPACE)
        .create(&PostParams::default(), &br_node)
        .await
        .context(error::BottlerocketNodeCreate {
            node_name: create_request.node_name.clone(),
            node_uid: create_request.node_uid.clone(),
        })?;

    Ok(HttpResponse::Ok().body(format!("{}", json!(&br_node))))
}

#[put("/bottlerocket-node-resource")]
async fn update_bottlerocket_node_resource(
    _req: HttpRequest,
    settings: web::Data<APIServerSettings>,
    update_request: web::Json<UpdateBottlerocketNodeRequest>,
) -> Result<impl Responder> {
    let k8s_client = &settings.k8s_client;

    let br_node_patch = json!({
        "apiVersion": constants::API_VERSION,
        "kind": "BottlerocketNode",
        "status": &update_request.node_status
    });

    let brn_name = format!("brn-{}", &update_request.node_name);

    let api: Api<BottlerocketNode> = Api::namespaced(k8s_client.clone(), constants::NAMESPACE);

    api.patch_status(
        &brn_name,
        &PatchParams::default(),
        &Patch::Merge(&br_node_patch),
    )
    .await
    .context(error::BottlerocketNodeUpdate {
        node_name: update_request.node_name.clone(),
        node_uid: update_request.node_uid.clone(),
    })?;

    Ok(HttpResponse::Ok().body(format!("{}", json!(&update_request.node_status))))
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
                .service(create_bottlerocket_node_resource)
                .service(update_bottlerocket_node_resource)
                .service(health_check)
        })
        .bind("0.0.0.0:8080")
        .context(error::HttpServerError)?
        .run()
        .await
        .context(error::HttpServerError)
    }
}
