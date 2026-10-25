use crate::models::{ContagionSignal, SignalSeverity};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::info;

#[derive(Clone)]
pub struct TelegramAlertDispatcher {
    bot_token: String,
    chat_id: String,
    client: Client,
}

impl TelegramAlertDispatcher {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            client: Client::new(),
        }
    }

    pub fn format_alert_html(&self, signal: &ContagionSignal) -> String {
        let severity_badge = match signal.severity {
            SignalSeverity::EarlySeed => "🌱 <b>[EARLY VIRAL SEED]</b>",
            SignalSeverity::ExponentialBreakout => "⚡ <b>[EXPONENTIAL BREAKOUT]</b>",
            SignalSeverity::ParabolicBlowoff => "🔥 <b>[PARABOLIC MANIA]</b>",
            SignalSeverity::ExhaustionFading => "⚠️ <b>[FADING EXHAUSTION]</b>",
        };

        let chain_str = signal.chain.as_deref().unwrap_or("SOL/EVM");
        let mcap_str = signal
            .market_cap_usd
            .map(|m| {
                if m >= 1_000_000.0 {
                    format!("${:.2}M", m / 1_000_000.0)
                } else {
                    format!("${:.1}K", m / 1_000.0)
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        let price_str = signal
            .price_usd
            .map(|p| format!("${:.6}", p))
            .unwrap_or_else(|| "N/A".to_string());

        let dex_link = signal
            .dexscreener_url
            .as_deref()
            .unwrap_or("https://dexscreener.com");

        format!(
            "{severity_badge}\n\n\
            💎 <b>Ticker:</b> <code>${ticker}</code>\n\
            ⛓️ <b>Chain:</b> <code>{chain_str}</code>\n\
            📊 <b>Market Cap:</b> <code>{mcap_str}</code> | <b>Price:</b> <code>{price_str}</code>\n\
            🎯 <b>Signal Confidence:</b> <b>{confidence:.0}%</b>\n\n\
            🧬 <b>Epidemiological Contagion:</b>\n\
            • <b>R₀ (Viral Rate):</b> <code>{r0:.2}</code>\n\
            • <b>Velocity:</b> <code>{velocity:.1} mentions/min</code>\n\
            • <b>Infected Clusters:</b> <code>{clusters}</code> | <b>Cross-Bridges:</b> <code>{bridges}</code>\n\n\
            💡 <i>{reason}</i>\n\n\
            📈 <a href=\"{dex_link}\">Open on DexScreener</a>",
            severity_badge = severity_badge,
            ticker = signal.ticker,
            chain_str = chain_str,
            mcap_str = mcap_str,
            price_str = price_str,
            confidence = signal.confidence_score * 100.0,
            r0 = signal.metrics.r0_reproduction_rate,
            velocity = signal.metrics.infection_velocity,
            clusters = signal.metrics.active_clusters_count,
            bridges = signal.metrics.cross_cluster_bridge_hits,
            reason = signal.reason,
            dex_link = dex_link
        )
    }

    pub async fn dispatch(&self, signal: &ContagionSignal) -> Result<()> {
        let text = self.format_alert_html(signal);
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let body = json!({
            "chat_id": self.chat_id,
            "text": text,
            "parse_mode": "HTML",
            "disable_web_page_preview": false
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to dispatch alert to Telegram API")?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Telegram API error: {}", err_text);
        }

        info!("Telegram viral signal alert delivered for ${}", signal.ticker);
        Ok(())
    }
}
