# QuantPilot User Guide

> v1.0.7 | English Edition

---

## Quick Start

### Launch

```bat
.\start.bat
```

Or with PowerShell:

```powershell
.\start.ps1
```

The backend starts on port 3000, Vite dev server on port 5173. The Tauri desktop window opens automatically.

### Separate Launch

```powershell
# Backend only
cargo run

# Frontend only
cd frontend && npm install && npm run dev
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `QUANTPILOT_DEV` | Dev mode (skip auth, aggressive cleanup) | `false` |
| `QUANTPILOT_API_KEY` | API authentication key | auto-generated |
| `QUANTPILOT_RATE_LIMIT_RPS` | Rate limit (requests/sec) | `100` |
| `QUANTPILOT_PORT` | Backend listen port | `3000` |

---

## Core Concepts

### Strategy Graph

A QuantPilot strategy is a directed acyclic graph with 5 node types:

```
Data → Intent → Agent → Risk → Execution
```

| Stage | Role | Example |
|-------|------|---------|
| **Data** | Market data source | K-line (OHLCV) from OKX |
| **Intent** | Signal generator | RSI, MACD, Dual MA |
| **Agent** | Decision maker | Weighted signals, Portfolio rebalance |
| **Risk** | Risk management | Position limits, Leverage caps |
| **Execution** | Order placement | Paper execution |

### Verified Indicators (18)

Moving Average (Cross/Deviation), RSI, MACD, Momentum, Z-Score, Spread, Quote Observe, ATR, Bollinger Bands, OBV, CMF, ADX, Stochastic, CCI, Parabolic SAR, Keltner Channel, Donchian Channel.

### Verified Exchanges & Symbols

- Exchanges: Binance (mock data), OKX (live testnet)
- Symbols: BTCUSDT, ETHUSDT, SOLUSDT

### Runtime Modes

- **Paper**: Simulated execution with configurable fee/slippage/latency
- **Testnet**: Real OKX testnet order placement

---

## Building a Strategy

### Using the Graph Editor

1. Drag a **Data** node from the left sidebar (e.g., "K-line Data")
2. Configure the exchange, symbol, and timeframe in the right panel
3. Drag an **Intent** node (e.g., "RSI Intent") and connect it to the data node
4. Continue connecting: Intent → Agent → Risk → Execution
5. Click **Compile** to validate the graph

### Using QuantScript

Create a `.qs` file:

```
fn strategy() {
    let btc = fetch("BTCUSDT", exchange="okx", interval="1h", lookback=100);
    let rsi_signal = rsi(btc, period=14);
    emit Intent(BUY, rsi_signal);
}
```

---

## Running Backtests

1. Compile your strategy graph
2. Click **Run Backtest**
3. Choose replay source: Deterministic Mock or Historical
4. View results: equity curve, trade ledger, metrics

### Experiment Parameter Sweeps

Run multiple backtests with different parameters to find optimal settings:
- Fee (bps), Slippage (bps), Latency (ms)
- Up to 27 variants per experiment

---

## Key Features

### Credential Management

Store OKX API credentials securely with AES-256-GCM encryption:

```bash
quantpilot credential set okx
```

### Alert Engine

10 built-in alert rules for data freshness, risk reject rate, event gaps, storage watermark, etc.

### Runbook

6 fault scenarios with diagnostic steps, recovery procedures, and verification.

### Chaos Experiments

Inject faults to test strategy resilience: data latency, event loss, disk pressure, clock skew.

### Hot-Swap

Replace runtime modules without stopping the trading session, with rollback support.

---

## Quality Gates

```powershell
.\tools\run-closeout-gates.bat

# Individual gates:
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd frontend && npm run build
cd frontend && npm run test
```

---

## API Reference

See [API Error Codes](./guide-api-error-codes.md) for all error codes and remediation steps.
