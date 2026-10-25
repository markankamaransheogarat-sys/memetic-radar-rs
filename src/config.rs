use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "memetic-radar-rs",
    author = "Markan Kamaransheogarat",
    version = "0.1.0",
    about = "High-Frequency Social Graph Viral Contagion Radar & Memetic Sentiment Arbitrage Engine"
)]
pub struct Config {
    #[arg(long, env = "PORT", default_value_t = 8080)]
    pub port: u16,

    #[arg(
        long,
        env = "TELEGRAM_BOT_TOKEN",
        default_value = "7729508480:AAER27k6f9XScuakIKKRuZ_rU431hfPZe0k"
    )]
    pub telegram_bot_token: String,

    #[arg(long, env = "TELEGRAM_CHAT_ID", default_value = "456429284")]
    pub telegram_chat_id: String,

    #[arg(long, env = "MIN_CONFIDENCE", default_value_t = 0.5)]
    pub min_confidence: f64,

    #[arg(long, env = "ANALYSIS_WINDOW_SECS", default_value_t = 600)]
    pub analysis_window_secs: i64,

    #[arg(long, env = "DEMO_FEED", default_value_t = false)]
    pub demo_feed: bool,
}
