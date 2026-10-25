use crate::models::{NodeId, SocialPlatform, SocialPost};
use chrono::Utc;
use regex::Regex;
use std::sync::OnceLock;

static TICKER_RE: OnceLock<Regex> = OnceLock::new();
static EVM_ADDR_RE: OnceLock<Regex> = OnceLock::new();
static SOL_ADDR_RE: OnceLock<Regex> = OnceLock::new();

fn ticker_regex() -> &'static Regex {
    TICKER_RE.get_or_init(|| Regex::new(r"(?i)(?:\$([a-zA-Z0-9]{2,10})\b|\b([A-Z0-9]{3,8})/USDT\b|\b([A-Z0-9]{3,8})/SOL\b)").unwrap())
}

fn evm_regex() -> &'static Regex {
    EVM_ADDR_RE.get_or_init(|| Regex::new(r"\b(0x[a-fA-F0-9]{40})\b").unwrap())
}

fn solana_regex() -> &'static Regex {
    SOL_ADDR_RE.get_or_init(|| Regex::new(r"\b([1-9A-HJ-NP-Za-km-z]{32,44}pump|[1-9A-HJ-NP-Za-km-z]{43,44})\b").unwrap())
}

pub struct MessageParser;

impl MessageParser {
    pub fn parse_post(
        id: &str,
        author: &str,
        text: &str,
        platform: SocialPlatform,
    ) -> SocialPost {
        let mut tickers = Vec::new();
        let mut contracts = Vec::new();

        // Tickers
        for cap in ticker_regex().captures_iter(text) {
            if let Some(t) = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3)) {
                let symbol = t.as_str().to_uppercase();
                // Avoid common false positives
                if !["THE", "FOR", "AND", "ARE", "NOT", "BUY", "ALL", "NEW", "HOLD", "AIR"].contains(&symbol.as_str()) {
                    if !tickers.contains(&symbol) {
                        tickers.push(symbol);
                    }
                }
            }
        }

        // EVM Addresses
        for cap in evm_regex().captures_iter(text) {
            if let Some(a) = cap.get(1) {
                let addr = a.as_str().to_string();
                if !contracts.contains(&addr) {
                    contracts.push(addr);
                }
            }
        }

        // Solana Addresses
        for cap in solana_regex().captures_iter(text) {
            if let Some(a) = cap.get(1) {
                let addr = a.as_str().to_string();
                if !contracts.contains(&addr) {
                    contracts.push(addr);
                }
            }
        }

        SocialPost {
            id: id.to_string(),
            author_id: NodeId::from(author),
            platform,
            text: text.to_string(),
            token_symbols: tickers,
            contract_addresses: contracts,
            timestamp: Utc::now(),
            reply_to: None,
            repost_of: None,
            raw_payload: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tickers_and_contracts() {
        let text = "Massive whale accumulation on $ACT and $GOAT! CA: 0x2260fac5e5542a773aa44fbcfedf7c193bc2c599 or pump 6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfPump";
        let post = MessageParser::parse_post("msg_1", "@crypto_alpha_channel", text, SocialPlatform::Telegram);

        assert!(post.token_symbols.contains(&"ACT".to_string()));
        assert!(post.token_symbols.contains(&"GOAT".to_string()));
        assert_eq!(post.contract_addresses.len(), 2);
    }
}
