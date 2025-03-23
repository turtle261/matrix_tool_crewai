use actix_web::{App, HttpServer};
use matrix_api::{api, config::Config};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_file("config.toml").expect("Failed to load config.toml");
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    let state = api::ApiState { sessions, config };
    println!("Starting server!");

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .configure(api::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}