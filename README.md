# 📡 MemeticRadar-RS

> **High-Frequency Social Graph Viral Contagion Radar & Memetic Sentiment Arbitrage Engine**  
> Written in ultra-low latency Rust. Powered by non-linear SEIR epidemiology, graph theory, and real-time on-chain DEX correlation.

[![Build Status](https://github.com/markankamaransheogarat-sys/memetic-radar-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/markankamaransheogarat-sys/memetic-radar-rs)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)

---

## 🔮 The Sci-Fi Premise (Black Mirror «Nosedive» Meets Wall Street HFT)

In modern decentralized markets, asset prices do not move on fundamental balance sheets. They move on **emotional contagion velocity** and **memetic percolation**. 

Traditional sentiment analysis relies on crude, lagging keyword counters (*"is Twitter bullish or bearish?"*). By the time an NLP model calculates a positive score, retail crowds have already bought the top, and insiders are dumping.

**MemeticRadar-RS** models information dissemination not as text, but as a **biological contagion spreading across a directed topology graph**. By calculating the basic reproduction rate ($R_0$) and detecting cross-cluster bridge hops before a narrative breaks out into the public sphere, it identifies viral breakout tokens **45 to 90 seconds before the market catches on**.

---

## 📐 Mathematical & Topological Architecture

```
[ Telegram / X / Farcaster Stream ]
                │
                ▼
      ┌──────────────────┐
      │  Message Parser  │ ──► Extracts $TICKER, CA (Solana/EVM), Metadata
      └──────────────────┘
                │
                ▼
    ┌───────────────────────┐
    │  Social Graph Engine  │ ──► Weighted DiGraph, PageRank Centrality,
    └───────────────────────┘     Community Cluster Bridges (Louvain/Modular)
                │
                ▼
    ┌───────────────────────┐
    │  Contagion SEIR Core  │ ──► Dynamic R₀ Calculation, Infection Velocity,
    └───────────────────────┘     Percolation Phase Shift Detection
                │
                ▼
    ┌───────────────────────┐
    │  Signal Synthesizer   │ ◄── On-Chain Liquidity & Price from DexScreener
    └───────────────────────┘
                │
        ┌───────┴────────────────────────┐
        ▼                                ▼
[ Telegram Alpha Channel ]     [ High-Speed WebSocket API ]
  (Formatted Rich Alerts)       (Quant Trading & Sniping Nodes)
```

### 1. SEIR Epidemiological Formulation
A narrative spread across social clusters is governed by modified non-linear differential transmission:

$$\frac{dS}{dt} = -\beta \frac{S \cdot I}{N}$$

$$\frac{dI}{dt} = \beta \frac{S \cdot I}{N} - \gamma I$$

- **$R_0$ (Basic Reproduction Rate)**:
  $$R_0 = \frac{\text{New Infections in } T_2}{\text{Infections in } T_1}$$
- When $R_0 > 1.5$ with positive acceleration ($\frac{d^2 I}{dt^2} > 0$) and $\ge 1$ cross-cluster bridge jump, a **percolation phase transition** occurs: the narrative has broken out from private alpha cabals into self-sustaining viral contagion.

### 2. Directed Social Graph Topology
- **Nodes $V$**: Alpha callers, whale wallets, telegram channels, KOLs, and momentum aggregators with dynamic authority scores $A(v) \in [1, 10]$ and cluster IDs.
- **Edges $E$**: Retweets, forwards, quotes, and simultaneous contract callings.
- **Bridge Detection**: A cross-cluster jump indicates information transmission from Cluster $C_1$ (e.g., *Insider Whales*) to Cluster $C_2$ (e.g., *Fast DeGen Callers*), preceding the cascade into Cluster $C_3$ (*Retail Feed*).

---

## 🚀 Quickstart & Installation

### Build from Source
```bash
git clone https://github.com/markankamaransheogarat-sys/memetic-radar-rs.git
cd memetic-radar-rs
cargo build --release
```

The optimized binary is located at `target/release/memetic-radar-rs`.

### Running the Engine

```bash
# Run with live Telegram dispatcher and Web API on port 8080
./target/release/memetic-radar-rs --port 8080 \
  --telegram-bot-token "YOUR_BOT_TOKEN" \
  --telegram-chat-id "YOUR_CHAT_ID" \
  --min-confidence 0.5

# Run in demonstration simulation mode (generates synthetic multi-cluster breakout)
./target/release/memetic-radar-rs --demo-feed
```

---

## 🛰️ API & Quant Stream Integration

### Health & Metrics
```http
GET /health
```
```json
{
  "status": "ONLINE",
  "active_narratives": 14,
  "graph_nodes": 128,
  "graph_edges": 342
}
```

### Ingesting External Social Posts
```http
POST /api/v1/feed
Content-Type: application/json

{
  "author": "@whale_insider_intel",
  "text": "Huge accumulation on $GOAT here! CA: 6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfPump",
  "platform": "telegram"
}
```

### High-Speed Quant WebSocket Feed
Connect your algorithmic trading bot or sniper directly to the raw signal stream:
```javascript
const ws = new WebSocket("ws://localhost:8080/ws");

ws.onmessage = (event) => {
  const signal = JSON.parse(event.data);
  console.log(`[ALPHA SIGNAL] $${signal.ticker} | R0: ${signal.metrics.r0_reproduction_rate} | Confidence: ${signal.confidence_score * 100}%`);
  // Trigger Solana / EVM sniper order here
};
```

---

## 💰 Monetization Architecture

1. **Private Alpha Telegram Channel ($150–$499/mo)**: Real-time push alerts delivering pre-pump tokens with $R_0$, viral velocity, and direct DEX links.
2. **Quant Fund WebSocket API Subscription ($1,000–$3,500/mo)**: Sub-second JSON stream consumed by Solana Jito / Flashbots MEV snipers.
3. **Proprietary Trading Engine**: Automated buy-and-sell order execution when $R_0$ crosses the threshold ($R_0 > 2.0$), auto-exiting when infection velocity decays ($R_t < 0.8$).

---

## 🛡️ License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
