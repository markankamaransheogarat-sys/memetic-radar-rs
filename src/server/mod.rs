use crate::contagion::ContagionEngine;
use crate::graph::SocialGraphEngine;
use crate::models::{ContagionSignal, SocialPlatform};
use crate::signals::SignalSynthesizer;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub graph: Arc<Mutex<SocialGraphEngine>>,
    pub contagion: Arc<Mutex<ContagionEngine>>,
    pub synthesizer: Arc<SignalSynthesizer>,
    pub signal_tx: broadcast::Sender<ContagionSignal>,
}

#[derive(Debug, Deserialize)]
pub struct IngestPayload {
    pub author: String,
    pub text: String,
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub active_narratives: usize,
    pub graph_nodes: usize,
    pub graph_edges: usize,
}

pub struct ApiServer {
    state: AppState,
    port: u16,
}

impl ApiServer {
    pub fn new(state: AppState, port: u16) -> Self {
        Self { state, port }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/api/v1/feed", post(ingest_handler))
            .route("/ws", get(ws_handler))
            .with_state(self.state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("🛰️ MemeticRadar HFT API Server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let (active_narratives, nodes, edges) = {
        let c = state.contagion.lock().await;
        let g = state.graph.lock().await;
        (c.active_narratives_count(), g.node_count(), g.edge_count())
    };

    Json(HealthResponse {
        status: "ONLINE",
        active_narratives,
        graph_nodes: nodes,
        graph_edges: edges,
    })
}

async fn ingest_handler(
    State(state): State<AppState>,
    Json(payload): Json<IngestPayload>,
) -> Json<serde_json::Value> {
    let platform = match payload.platform.as_deref() {
        Some("twitter") | Some("x") => SocialPlatform::TwitterX,
        Some("farcaster") => SocialPlatform::Farcaster,
        _ => SocialPlatform::Telegram,
    };

    let post = crate::ingest::MessageParser::parse_post(
        &format!("api_{}", chrono::Utc::now().timestamp_millis()),
        &payload.author,
        &payload.text,
        platform,
    );

    let signals = state.synthesizer.process_post(&post).await;
    for signal in &signals {
        let _ = state.signal_tx.send(signal.clone());
    }

    Json(serde_json::json!({
        "status": "PROCESSED",
        "tokens_detected": post.token_symbols,
        "signals_generated": signals.len()
    }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.signal_tx.subscribe();
    info!("New quant client connected to HFT WebSocket stream");

    while let Ok(signal) = rx.recv().await {
        let json_text = match serde_json::to_string(&signal) {
            Ok(t) => t,
            Err(_) => continue,
        };

        if socket.send(Message::Text(json_text)).await.is_err() {
            // Client disconnected
            break;
        }
    }
}
