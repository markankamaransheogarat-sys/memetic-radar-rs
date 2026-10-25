use crate::contagion::ContagionEngine;
use crate::graph::SocialGraphEngine;
use crate::ingest::DexScreenerClient;
use crate::models::{ContagionSignal, SignalSeverity, SocialPost};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct SignalSynthesizer {
    graph: Arc<Mutex<SocialGraphEngine>>,
    contagion: Arc<Mutex<ContagionEngine>>,
    dex: DexScreenerClient,
    min_confidence: f64,
}

impl SignalSynthesizer {
    pub fn new(
        graph: Arc<Mutex<SocialGraphEngine>>,
        contagion: Arc<Mutex<ContagionEngine>>,
        dex: DexScreenerClient,
        min_confidence: f64,
    ) -> Self {
        Self {
            graph,
            contagion,
            dex,
            min_confidence,
        }
    }

    pub async fn process_post(&self, post: &SocialPost) -> Vec<ContagionSignal> {
        let mut signals = Vec::new();
        let author_id = &post.author_id;

        // Check author cluster & authority
        let (cluster_id, authority) = {
            let g = self.graph.lock().await;
            if let Some(node) = g.get_node(author_id) {
                (node.cluster_id, node.authority_score)
            } else {
                (0, 1.0)
            }
        };

        for ticker in &post.token_symbols {
            let contract = post.contract_addresses.first().cloned();
            let is_bridge = {
                let _g = self.graph.lock().await;
                // If author's cluster differs from tracker's dominant cluster
                let c = self.contagion.lock().await;
                if let Some(tracker) = c.get_tracker(ticker) {
                    !tracker.infected_clusters.contains(&cluster_id)
                } else {
                    false
                }
            };

            let (metrics, severity) = {
                let mut c = self.contagion.lock().await;
                c.ingest_mention(ticker, contract.clone(), author_id.clone(), cluster_id, is_bridge)
            };

            // Only trigger signals on meaningful phase transitions
            if severity == SignalSeverity::ExponentialBreakout || severity == SignalSeverity::ParabolicBlowoff {
                let query = contract.as_deref().unwrap_or(ticker);
                let onchain_data = self.dex.fetch_token_data(query).await.unwrap_or(None);

                let (price, mcap, vol, chain, url) = match &onchain_data {
                    Some(data) => (
                        data.price_usd,
                        data.market_cap,
                        data.volume_h24,
                        Some(data.chain_id.clone()),
                        Some(data.url.clone()),
                    ),
                    None => (None, None, None, None, None),
                };

                // Calculate composite confidence score (0.0 to 1.0)
                let r0_factor = (metrics.r0_reproduction_rate / 4.0).min(1.0) * 0.4;
                let authority_factor = (authority / 10.0).min(1.0) * 0.3;
                let bridge_factor = (metrics.cross_cluster_bridge_hits as f64 / 3.0).min(1.0) * 0.3;
                let confidence = (r0_factor + authority_factor + bridge_factor).min(1.0);

                if confidence >= self.min_confidence {
                    let reason = format!(
                        "Phase transition detected: R0={:.2}, velocity={:.1} mentions/min, clusters={}, bridges={}",
                        metrics.r0_reproduction_rate,
                        metrics.infection_velocity,
                        metrics.active_clusters_count,
                        metrics.cross_cluster_bridge_hits
                    );

                    info!("🚨 ALPHA SIGNAL DETECTED: ${} (Confidence: {:.2}) - {}", ticker, confidence, reason);

                    signals.push(ContagionSignal {
                        id: format!("{}-{}", ticker, Utc::now().timestamp()),
                        ticker: ticker.clone(),
                        contract_address: contract,
                        chain,
                        severity,
                        confidence_score: (confidence * 100.0).round() / 100.0,
                        metrics,
                        price_usd: price,
                        market_cap_usd: mcap,
                        volume_24h_usd: vol,
                        dexscreener_url: url,
                        genesis_authors: vec![author_id.0.clone()],
                        created_at: Utc::now(),
                        reason,
                    });
                }
            }
        }

        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::MessageParser;
    use crate::models::{GraphNode, NodeId, SocialPlatform};

    #[tokio::test]
    async fn test_full_pipeline_signal_synthesis() {
        let graph = Arc::new(Mutex::new(SocialGraphEngine::new()));
        let contagion = Arc::new(Mutex::new(ContagionEngine::new(300)));
        let dex = DexScreenerClient::new();

        {
            let mut g = graph.lock().await;
            g.upsert_node(GraphNode {
                id: NodeId::from("alpha_whale"),
                username: "alpha_whale".to_string(),
                platform: SocialPlatform::Telegram,
                authority_score: 9.5,
                cluster_id: 1,
                followers_count: 50_000,
                historical_win_rate: 0.9,
            });
            g.upsert_node(GraphNode {
                id: NodeId::from("retail_caller"),
                username: "retail_caller".to_string(),
                platform: SocialPlatform::TwitterX,
                authority_score: 7.0,
                cluster_id: 2,
                followers_count: 20_000,
                historical_win_rate: 0.7,
            });
        }

        let synthesizer = SignalSynthesizer::new(graph, contagion, dex, 0.4);

        // First post from whale
        let post1 = MessageParser::parse_post(
            "p1",
            "alpha_whale",
            "Insane accumulation on $SOLMEME! 100x incoming.",
            SocialPlatform::Telegram,
        );
        let signals1 = synthesizer.process_post(&post1).await;
        // First mention doesn't break out yet
        assert_eq!(signals1.len(), 0);

        // Follow up mentions jumping across to cluster 2
        for i in 2..=5 {
            let caller = if i % 2 == 0 { "retail_caller" } else { "alpha_whale" };
            let post = MessageParser::parse_post(
                &format!("p{}", i),
                caller,
                "$SOLMEME breaking out on high volume!",
                SocialPlatform::TwitterX,
            );
            let sigs = synthesizer.process_post(&post).await;
            if !sigs.is_empty() {
                assert_eq!(sigs[0].ticker, "SOLMEME");
                assert!(sigs[0].confidence_score >= 0.4);
                return;
            }
        }
    }
}
