use actix_web::{
    get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use brupop::models::api::UpsertBottlerocketNodeRequest;

#[derive(Debug, Clone)]
struct APIServerSettings {}

/// Implements a shallow health check for the HTTP service.
#[get("/ping")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[post("/bottlerocket-node-resource")]
async fn create_or_update_bottlerocket_node_resource(
    _req: HttpRequest,
    _settings: web::Data<APIServerSettings>,
    upsert_request: web::Json<UpsertBottlerocketNodeRequest>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        let api_server_settings = APIServerSettings {};
        App::new()
            .wrap(middleware::Logger::default().exclude("/ping"))
            .data(api_server_settings)
            .service(create_or_update_bottlerocket_node_resource)
            .service(health_check)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
