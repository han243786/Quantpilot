# QuantPilot

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v1.0.5** | [系统架构](./markdown/10-overview/overview-system-architecture.md) | [使用指南](./markdown/10-overview/overview-system-architecture.md#十一使用指南) | [General_Policy](./markdown/General_Policy.md) | [版本历史](./CHANGELOG.md)

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

## v1.0.5 最终状态

| 维度 | v1.0.5 | 说明 |
|------|:------:|------|
| 功能开发进度 | 9.5/10 | 18 指标全部实现 / 分页全栈 / API 客户端统一 |
| 仓库稳定程度 | 9.5/10 | 六轮审计 138 项发现全部闭环 / 0 已知缺陷 |
| 发布就绪度 | 9.5/10 | S0 19/19 | P1 45/45 | P2 40/40 | 测试 92 文件 269 用例 |
| 用户友好程度 | 9.0/10 | A11y 全页面语义化 / i18n 覆盖 8 页面 / 骨架屏 / 错误恢复 |
| 系统整体稳定性 | 9.0/10 | 统一锁 / SSE 重连修复 / 分页兼容 / safe_log 修复 |
| **加权** | **9.3/10** |

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
| v1.0.3 规划 | `./markdown/06-milestones/v1.0.3/01-规划方案.md` |
| 四回合审计报告 | `./markdown/05-testing/` |
