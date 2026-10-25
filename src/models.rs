use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        NodeId(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub username: String,
    pub platform: SocialPlatform,
    pub authority_score: f64,
    pub cluster_id: usize,
    pub followers_count: u64,
    pub historical_win_rate: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SocialPlatform {
    Telegram,
    TwitterX,
    Farcaster,
    Discord,
    OnChainDEX,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub id: String,
    pub author_id: NodeId,
    pub platform: SocialPlatform,
    pub text: String,
    pub token_symbols: Vec<String>,
    pub contract_addresses: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<String>,
    pub repost_of: Option<String>,
    pub raw_payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContagionMetrics {
    pub r0_reproduction_rate: f64,
    pub infection_velocity: f64,
    pub acceleration: f64,
    pub infected_nodes_count: usize,
    pub active_clusters_count: usize,
    pub cross_cluster_bridge_hits: usize,
    pub percolation_threshold_exceeded: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalSeverity {
    EarlySeed,
    ExponentialBreakout,
    ParabolicBlowoff,
    ExhaustionFading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContagionSignal {
    pub id: String,
    pub ticker: String,
    pub contract_address: Option<String>,
    pub chain: Option<String>,
    pub severity: SignalSeverity,
    pub confidence_score: f64,
    pub metrics: ContagionMetrics,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    pub dexscreener_url: Option<String>,
    pub genesis_authors: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexTokenData {
    pub chain_id: String,
    pub dex_id: String,
    pub pair_address: String,
    pub base_token_symbol: String,
    pub base_token_address: String,
    pub price_usd: Option<f64>,
    pub volume_h24: Option<f64>,
    pub market_cap: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub url: String,
}
