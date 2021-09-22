use brupop::controller::BrupopController;

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut controller = BrupopController::new();
    controller.run().await;
}
