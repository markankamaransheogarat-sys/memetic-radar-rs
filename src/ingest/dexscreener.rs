use crate::models::DexTokenData;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct DexScreenerClient {
    client: Client,
}

impl Default for DexScreenerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DexScreenerClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("MemeticRadar/1.0 (HFT Research)")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Search DexScreener by ticker or token address
    pub async fn fetch_token_data(&self, query: &str) -> Result<Option<DexTokenData>> {
        let url = format!("https://api.dexscreener.com/latest/dex/search?q={}", query);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to query DexScreener API")?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: Value = resp.json().await.context("Failed to parse DexScreener response")?;
        let pairs = match body.get("pairs").and_then(|p| p.as_array()) {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };

        // Pick pair with highest liquidity
        let best_pair = pairs
            .iter()
            .max_by(|a, b| {
                let liq_a = a
                    .get("liquidity")
                    .and_then(|l| l.get("usd"))
                    .and_then(|u| u.as_f64())
                    .unwrap_or(0.0);
                let liq_b = b
                    .get("liquidity")
                    .and_then(|l| l.get("usd"))
                    .and_then(|u| u.as_f64())
                    .unwrap_or(0.0);
                liq_a.partial_cmp(&liq_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(pair) = best_pair {
            let chain_id = pair.get("chainId").and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
            let dex_id = pair.get("dexId").and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
            let pair_address = pair.get("pairAddress").and_then(|s| s.as_str()).unwrap_or("").to_string();
            let base_token = pair.get("baseToken");
            let symbol = base_token
                .and_then(|b| b.get("symbol"))
                .and_then(|s| s.as_str())
                .unwrap_or(query)
                .to_string();
            let address = base_token
                .and_then(|b| b.get("address"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let price_usd = pair
                .get("priceUsd")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let volume_h24 = pair
                .get("volume")
                .and_then(|v| v.get("h24"))
                .and_then(|u| u.as_f64());

            let market_cap = pair
                .get("fdv")
                .and_then(|f| f.as_f64())
                .or_else(|| pair.get("marketCap").and_then(|m| m.as_f64()));

            let liquidity_usd = pair
                .get("liquidity")
                .and_then(|l| l.get("usd"))
                .and_then(|u| u.as_f64());

            let url = pair
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();

            Ok(Some(DexTokenData {
                chain_id,
                dex_id,
                pair_address,
                base_token_symbol: symbol,
                base_token_address: address,
                price_usd,
                volume_h24,
                market_cap,
                liquidity_usd,
                url,
            }))
        } else {
            Ok(None)
        }
    }
}
