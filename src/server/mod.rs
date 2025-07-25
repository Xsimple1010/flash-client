use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::{
    AppState,
    server::{
        download_exe::{download_dep, download_exe},
        ping::ping,
    },
};

mod download_exe;
mod ping;

pub async fn init_server(state: AppState) {
    let flash_port = std::env::var("FLASH_CLIENT_PORT").unwrap_or_else(|_| "4090".to_string());

    println!("server listener on: http://0.0.0.0:{}", flash_port);

    let app = Router::new()
        .route("/exe", post(download_exe))
        .route("/dep", post(download_dep))
        .route("/ping", get(ping))
        .with_state(state)
        .layer(DefaultBodyLimit::max(600 * 1024 * 1024)); // Limite de 600MB por upload

    let addr = format!("0.0.0.0:{}", flash_port);

    let tcp_listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}
