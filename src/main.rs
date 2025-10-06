use actix_web::{App, HttpServer};
use matrix_api::{api, config::Config};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use log::{info, error};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = match Config::from_file("config.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config.toml: {}", e);
            eprintln!("Please ensure config.toml exists and has the correct format.");
            std::process::exit(1);
        }
    };
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    let state = api::ApiState { sessions, config };
    info!("Starting Matrix API server on {}:{}", config.server.host, config.server.port);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .configure(api::config)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run();

    info!("Matrix API server started successfully");
    server.await
}