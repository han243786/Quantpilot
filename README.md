# QuantPilot

> ⚠️ **实验性软件声明**  
> 当前版本 (v4.0.0) 仍处于密集开发阶段。尽管已完成多轮全维度诱错审计并进入状态机化架构收口, 但系统中仍然可能存在未被发现的阻断性缺陷和边界问题。本版本仅适用于实验、研究和离线模拟, **不可用于实盘交易或生产环境**。开发者需要自行精细打磨, 并结合自身使用场景进行充分验证。

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v4.0.0** (状态机化架构 + Risk Plane + ExecutionMachine 能力来源 + 开发者学习流水线) | [系统架构](./markdown/10-overview/overview-system-architecture.md) | [使用指南](./markdown/10-overview/overview-system-architecture.md#十一使用指南) | [General_Policy](./markdown/General_Policy.md) | [超级规范化](./markdown/01-principles/principles-super-standardization.md) | [版本历史](./CHANGELOG.md)

## 产品边界

- **运行时模式**: paper (纸面交易), backtest (回测), live (OKX testnet 模拟盘)
- **执行端**: 独立进程 (:3001), 策略部署/启动/停止/热调参
- **已验证交易所**: `binance`, `okx`
- **已验证交易对**: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- **桌面应用**: Tauri v2 自绘标题栏 Windows 桌面应用 (`start.bat`)
- **前端**: Adobe 暗色面板设计系统、图编辑器、策略工作区、回测详情/对比、研究控制台、Toast 通知
- **QuantScript**: 语法解析 → HIR → lowering → Core IR 完整编译管道, 策略脚本一站式编辑
- **插件**: 18 种指标全部有 evaluator 实现, 零 stub
- **安全**: AES-256-GCM 凭证保险库, bcrypt(12轮) 用户认证, JWT + 刷新令牌轮换+重放检测, 进程间加密通道
- **告警**: 10 条默认规则, 自动恢复 (resolve_condition), 去重

### 已验证的全部指标 (18 种)

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
| `QUANTPILOT_DEV` | DEV 模式 (跳过认证+限速, 缩短TTL) | `false` |
| `QUANTPILOT_STORAGE_ROOT` | 存储根目录 | `storage` |
| `QUANTPILOT_EXECUTOR_URL` | 执行端地址 | `http://127.0.0.1:3001` |
| `QUANTPILOT_EXECUTOR_INSECURE` | 跳过执行端 API 守卫 | `false` |
| `QUANTPILOT_JWT_SECRET` | JWT 密钥 (留空自动生成) | (随机) |
| `QUANTPILOT_API_KEY` | API 密钥 | (无) |
| `QUANTPILOT_MARKET_PUBLIC_KEY` | 插件市场 Ed25519 公钥 | (测试向量) |
| `QUANTPILOT_RATE_LIMIT_RPS` | 全局限速 (请求/秒) | `100` |
| `QUANTPILOT_LOG_FORMAT` | 日志格式 (compact/json) | `compact` |
| `QUANTPILOT_TRUSTED_PROXY` | 反向代理模式 | `false` |

详见 `.env.example`。

## 执行端

```powershell
# 启动执行端 (端口 3001)
cargo run --bin executor
```

## CI / 质量门禁

```powershell
# 一键收口
.\tools\run-closeout-gates.bat

# 单项门禁
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-i18n.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-version-consistency.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-feature-evolution.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-learning-closeout.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-cleanup-boundary.ps1
cargo fmt --check
cargo check --workspace
.\scripts\test.ps1 test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-clippy-warning-budget.ps1 -MaxWarnings 58
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-executor-warning-budget.ps1 -MaxWarnings 0
cd frontend; npm run build
cd frontend; npm run test
cd frontend; npm run test:e2e
cd frontend; npm audit --audit-level=moderate
cd ..\frontend-executor; npm run build
cd ..
cargo check --bin executor
.\scripts\test.ps1 test --bin executor
.\scripts\scenario-smoke.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-clean-worktree.ps1
```

常规 `test:e2e` 只包含阻断级用户路径。视觉响应式审查和性能采样属于 closeout/review 层级，按需显式执行：

```bash
cd frontend; npm run test:e2e:visual-review
cd frontend; npm run test:perf:first-screen
cd frontend; npm run test:perf:react-flow
```

### Pre-commit hook

`scripts/pre-commit` 在 `git commit` 时自动执行 UTF-8 检查、`cargo fmt --check`、`cargo check`、`cargo test --no-run`、`vite build`、`vitest run`。

## v4.0.0 流程收口状态

| 项 | 状态 | 说明 |
|----|:--:|------|
| S0 登录挂起 | ✅ | `ring::rand::SystemRandom` 缓存 + refresh token 生成移出 DB 锁 |
| P1 凭证 DELETE 405 | ✅ | Axum 0.7 路由参数语法修正为 `:service` |
| P2 测试进程文件锁 | ✅ | `scripts/test.ps1` / `scripts/test.sh` 在测试前停止本仓库运行进程 |
| 三层工作流门禁 | ✅ | pre-commit / CI / closeout-release 三层已统一 |
| 功能演进契约 | ✅ | 新能力必须登记能力边界、回归保护矩阵、兼容性与迁移说明 |
| Rust 格式基线 | ✅ | 全仓 `cargo fmt` 已落地，pre-commit / CI / closeout 均执行 `cargo fmt --check` |
| 版本一致性 | ✅ | Cargo、Tauri、前端 package、lockfile 和关键文档统一到 `4.0.0` |
| executor warning 债务 | ✅ | 当前预算 0；新增 warning 会失败 |
| 完整 closeout | ⏳ | v4.0.0 按 `04-Codex执行规范.md` 执行 V1-V10 全量收口 |

## 文档入口

| 文档 | 路径 |
|------|------|
| 文档索引 | `./markdown/README.md` |
| 当前状态与路线图 | `./markdown/10-overview/overview-current-status-and-roadmap.md` |
| 超级规范化 | `./markdown/01-principles/principles-super-standardization.md` |
| RFC 索引 (001-020) | `./markdown/02-protocol/README.md` |
| 编译链合约 | `./markdown/03-implementation/governance/implementation-compile-chain-contract.md` |
| 功能演进契约 | `./markdown/03-implementation/governance/implementation-feature-evolution-contract.md` |
| 支持矩阵 | `./markdown/03-implementation/governance/implementation-support-matrix.md` |
| API 参考 | `./markdown/04-guides/guide-api-reference.md` |
| v1.0.0 规划 | `./markdown/06-milestones/v1.0.0/01-规划方案.md` |
| v1.0.3 规划 | `./markdown/06-milestones/v1.0.3/01-规划方案.md` |
| 四回合审计报告 | `./markdown/05-testing/` |
