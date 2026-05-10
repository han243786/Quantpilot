# QuantPilot

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v0.5.2** | 下一版本: [v1.0.0](./markdown/06-milestones/v1.0.0/01-规划方案.md) | 全局规则: [General_Policy.md](markdown/General_Policy.md)

## 产品边界

- **运行时模式**: paper (纸面交易), testnet (测试网)
- **交易所**: `binance`, `okx`
- **交易对**: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- **桌面应用**: Tauri v2 自绘标题栏 Windows 桌面应用 (`start.bat`)
- **前端**: Adobe 暗色面板设计系统、图编辑器、策略工作区、回测详情/对比、研究控制台
- **QuantScript**: 语法解析 → HIR → lowering → Core IR 完整编译管道
- **插件**: 18 种指标全部有 evaluator 实现, 零 stub

### 支持的全部指标 (18 种)

| # | 指标 | # | 指标 |
|---|------|---|------|
| 1 | MA Cross | 10 | OBV |
| 2 | MA Deviation | 11 | CMF |
| 3 | RSI | 12 | ADX |
| 4 | MACD | 13 | Stochastic |
| 5 | Momentum | 14 | CCI |
| 6 | ZScore | 15 | Parabolic SAR |
| 7 | Spread | 16 | Keltner Channel |
| 8 | QuoteObserve | 17 | Donchian Channel |
| 9 | ATR | 18 | Bollinger Bands |

## 非宣称能力

- 实盘交易 (live trading)
- 研究级回测语义
- 真正套利平台支持
- 第三方插件市场
- 通过 QuantScript 执行任意主机代码
- 公开 SaaS 服务

## 快速启动

```bat
.\start.bat
```

Tauri 自动启动后端 (端口 3000) + 前端 Vite dev server (端口 5173)。

或分开启动:

```powershell
# 仅后端
cargo run
# 仅前端
cd frontend && npm install && npm run dev
```

## 环境变量

| 变量 | 用途 | 默认值 |
|------|------|--------|
| `VITE_BACKEND_ORIGIN` | Vite dev proxy 后端地址 | `http://127.0.0.1:3000` |
| `VITE_API_BASE_URL` | 浏览器直连 API 地址 | 从当前 origin 派生 `/api` |
| `QUANTPILOT_DEV` | DEV 模式 (缩短 TTL, 强制清理瞬态) | `false` |

## CI / 质量门禁

```powershell
# 一键收口
.\tools\run-closeout-gates.bat

# 单项门禁
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd frontend; npm run build
cd frontend; npm run test
cd frontend; npm run test:e2e
cd frontend; npm audit --audit-level=moderate
```

### Pre-commit hook

`scripts/pre-commit` 在 `git commit` 时自动执行 `cargo check` + `cargo test --no-run` + `vite build` + `vitest run`

## v0.5.2 就绪度

| 维度 | v0.5.2 |
|------|:------:|
| 功能开发进度 | 8/10 |
| 仓库稳定程度 | 8/10 |
| 发布就绪度 | 7/10 |
| 用户友好程度 | 7/10 |
| 系统整体稳定性 | 7/10 |
| **加权** | **7.4/10** |

## 文档入口

| 文档 | 路径 |
|------|------|
| 文档索引 | `./markdown/README.md` |
| 当前状态与路线图 | `./markdown/10-overview/overview-current-status-and-roadmap.md` |
| 超级规范化 | `./markdown/01-principles/principles-super-standardization.md` |
| RFC 索引 (001-020) | `./markdown/02-protocol/README.md` |
| 编译链合约 | `./markdown/03-implementation/governance/implementation-compile-chain-contract.md` |
| 支持矩阵 | `./markdown/03-implementation/governance/implementation-support-matrix.md` |
| API 参考 | `./markdown/04-guides/guide-api-reference.md` |
| v1.0.0 规划 | `./markdown/06-milestones/v1.0.0/01-规划方案.md` |
