mod alerts;
mod config;
mod contagion;
mod graph;
mod ingest;
mod models;
mod server;
mod signals;

use alerts::TelegramAlertDispatcher;
use clap::Parser;
use config::Config;
use contagion::ContagionEngine;
use graph::SocialGraphEngine;
use ingest::DexScreenerClient;
use models::{GraphNode, NodeId, SocialPlatform};
use server::{ApiServer, AppState};
use signals::SignalSynthesizer;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,memetic_radar_rs=debug".into()),
        )
        .init();

    dotenvy::dotenv().ok();
    let config = Config::parse();

    info!("🚀 Initializing MemeticRadar-RS v0.1.0");
    info!("📡 Port: {}, Min Confidence: {}", config.port, config.min_confidence);

    let graph = Arc::new(Mutex::new(SocialGraphEngine::new()));
    let contagion = Arc::new(Mutex::new(ContagionEngine::new(config.analysis_window_secs)));
    let dex = DexScreenerClient::new();

    // Pre-populate core graph topology clusters
    {
        let mut g = graph.lock().await;
        seed_alpha_topology(&mut g);
    }

    let synthesizer = Arc::new(SignalSynthesizer::new(
        graph.clone(),
        contagion.clone(),
        dex,
        config.min_confidence,
    ));

    let (signal_tx, _): (broadcast::Sender<models::ContagionSignal>, _) = broadcast::channel(100);

    // Telegram dispatcher worker
    let alert_dispatcher = TelegramAlertDispatcher::new(
        config.telegram_bot_token.clone(),
        config.telegram_chat_id.clone(),
    );

    let mut alert_rx: broadcast::Receiver<models::ContagionSignal> = signal_tx.subscribe();
    tokio::spawn(async move {
        while let Ok(signal) = alert_rx.recv().await {
            info!("⚡ Dispatching Telegram alert for ${}", signal.ticker);
            if let Err(e) = alert_dispatcher.dispatch(&signal).await {
                error!("Failed to dispatch Telegram alert: {:?}", e);
            }
        }
    });

    let app_state = AppState {
        graph: graph.clone(),
        contagion: contagion.clone(),
        synthesizer: synthesizer.clone(),
        signal_tx: signal_tx.clone(),
    };

    // Optional demo feed simulation loop
    if config.demo_feed {
        let synth_clone = synthesizer.clone();
        let tx_clone = signal_tx.clone();
        tokio::spawn(async move {
            run_demo_feed_simulation(synth_clone, tx_clone).await;
        });
    }

    let server = ApiServer::new(app_state, config.port);
    server.run().await?;

    Ok(())
}

fn seed_alpha_topology(graph: &mut SocialGraphEngine) {
    // Cluster 1: Whale & On-Chain Smart Money
    graph.upsert_node(GraphNode {
        id: NodeId::from("@whale_insider_intel"),
        username: "whale_insider_intel".to_string(),
        platform: SocialPlatform::Telegram,
        authority_score: 9.8,
        cluster_id: 1,
        followers_count: 85_000,
        historical_win_rate: 0.88,
    });
    graph.upsert_node(GraphNode {
        id: NodeId::from("@smart_money_sol"),
        username: "smart_money_sol".to_string(),
        platform: SocialPlatform::TwitterX,
        authority_score: 9.2,
        cluster_id: 1,
        followers_count: 140_000,
        historical_win_rate: 0.82,
    });

    // Cluster 2: Fast Alpha Callers
    graph.upsert_node(GraphNode {
        id: NodeId::from("@solana_gems_alerts"),
        username: "solana_gems_alerts".to_string(),
        platform: SocialPlatform::Telegram,
        authority_score: 7.5,
        cluster_id: 2,
        followers_count: 32_000,
        historical_win_rate: 0.70,
    });
    graph.upsert_node(GraphNode {
        id: NodeId::from("@dex_degen_calls"),
        username: "dex_degen_calls".to_string(),
        platform: SocialPlatform::Telegram,
        authority_score: 6.8,
        cluster_id: 2,
        followers_count: 19_000,
        historical_win_rate: 0.65,
    });

    // Cluster 3: Retail Amplifiers
    graph.upsert_node(GraphNode {
        id: NodeId::from("@crypto_trending_feed"),
        username: "crypto_trending_feed".to_string(),
        platform: SocialPlatform::TwitterX,
        authority_score: 5.5,
        cluster_id: 3,
        followers_count: 350_000,
        historical_win_rate: 0.50,
    });

    // Add inter-cluster transmission probabilities (bridges)
    graph.add_transmission(&NodeId::from("@whale_insider_intel"), &NodeId::from("@solana_gems_alerts"), 2.0);
    graph.add_transmission(&NodeId::from("@solana_gems_alerts"), &NodeId::from("@crypto_trending_feed"), 3.5);
}

async fn run_demo_feed_simulation(
    synthesizer: Arc<SignalSynthesizer>,
    signal_tx: broadcast::Sender<models::ContagionSignal>,
) {
    info!("🧪 Starting live demo feed simulation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let simulation_sequence = [
        ("@whale_insider_intel", "Accumulating $GOAT early here. Looks like a monster run coming. CA: 6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfPump"),
        ("@smart_money_sol", "$GOAT on-chain wallet clustering confirms top-tier accumulation."),
        ("@solana_gems_alerts", "BREAKOUT: $GOAT crossing bridge threshold! Huge volume incoming."),
        ("@dex_degen_calls", "Aping $GOAT! CA: 6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfPump to the moon 🚀"),
        ("@crypto_trending_feed", "$GOAT is now #1 trending on Solana DexScreener! Viral contagion confirmed."),
    ];

    for (author, text) in simulation_sequence {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let post = crate::ingest::MessageParser::parse_post(
            &format!("sim_{}", chrono::Utc::now().timestamp_millis()),
            author,
            text,
            SocialPlatform::Telegram,
        );

        let signals = synthesizer.process_post(&post).await;
        for s in signals {
            let _ = signal_tx.send(s);
        }
    }
}
