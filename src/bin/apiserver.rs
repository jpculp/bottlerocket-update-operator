use actix_web::{
    middleware, post, put, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

#[derive(Debug, Clone)]
struct APIServerSettings {}

#[post("/bottlerocket-node-resource")]
async fn create_bottlerocket_node_resource(
    _req: HttpRequest,
    _settings: web::Data<APIServerSettings>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[put("/bottlerocket-node-resource/{node_id}")]
async fn update_bottlerocket_node_resource(
    _req: HttpRequest,
    web::Path(node_id): web::Path<String>,
    _settings: web::Data<APIServerSettings>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("Updating node with id {}", node_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        let api_server_settings = APIServerSettings {};
        App::new()
            .wrap(middleware::Logger::default())
            .data(api_server_settings)
            .service(create_bottlerocket_node_resource)
            .service(update_bottlerocket_node_resource)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
