use actix_web::{middleware::Logger, App, HttpServer};
pub mod errors;
mod views;
use views::view_factory;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().wrap(Logger::default()).configure(view_factory))
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
