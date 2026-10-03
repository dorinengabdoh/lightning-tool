use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

use crate::api::*;
use crate::node::LightningNodeManager;

#[derive(Clone)]
pub struct AppState {
    pub node: Arc<Mutex<LightningNodeManager>>,
}

async fn serve_index_html() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../public/index.html"))
}

pub async fn start_web_server(port: u16, network: String) -> anyhow::Result<()> {
    let node_manager = Arc::new(Mutex::new(LightningNodeManager::new(&network)));
    let state = AppState { node: node_manager };

    // Endpoints REST HTTP (Axum)
    let api_router = Router::new()
        .route("/invoice/decode", post(handle_decode_invoice))
        .route("/invoice/create", post(handle_create_invoice))
        .route("/invoice/pay", post(handle_pay_invoice))
        .route("/node/open-channel", post(handle_open_channel))
        .route("/node/close-channel", post(handle_close_channel))
        .route("/node/force-close", post(handle_force_close))
        .route("/node/simulate-breach", post(handle_simulate_breach))
        .route("/node/transfer", post(handle_transfer_peer_balance))
        .route("/node/pay-multihop", post(handle_multihop_payment))
        .route("/node/fund", post(handle_fund_wallet))
        .route("/node/status", get(handle_node_status))
        .with_state(state);

    let app = Router::new()
        .nest("/api", api_router)
        .route("/", get(serve_index_html))
        .route("/dashboard", get(serve_index_html))
        .fallback_service(ServeDir::new("public").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive());

    let mut current_port = port;
    let listener = loop {
        let addr = format!("0.0.0.0:{}", current_port);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(_e) if current_port < port + 10 => {
                info!("Port {} in use, trying port {}...", current_port, current_port + 1);
                current_port += 1;
            }
            Err(e) => return Err(e.into()),
        }
    };

    let dashboard_url = format!("http://localhost:{}/dashboard", current_port);

    println!("\n⚡ Lightning SatGate (Design 1) :");
    println!("👉 {}\n", dashboard_url);

    let url_for_browser = dashboard_url.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        #[cfg(target_os = "linux")]
        {
            let res = std::process::Command::new("google-chrome").arg(&url_for_browser).spawn();
            if res.is_err() {
                let res2 = std::process::Command::new("firefox").arg(&url_for_browser).spawn();
                if res2.is_err() {
                    let res3 = std::process::Command::new("x-www-browser").arg(&url_for_browser).spawn();
                    if res3.is_err() {
                        let _ = std::process::Command::new("xdg-open").arg(&url_for_browser).spawn();
                    }
                }
            }
        }
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url_for_browser).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", "start", &url_for_browser]).spawn();
    });

    axum::serve(listener, app).await?;

    Ok(())
}
