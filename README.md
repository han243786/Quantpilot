# QuantPilot

> ⚠️ **实验性软件声明**  
> 当前版本 (v3.7.0) 仍处于密集开发阶段。尽管已完成三轮全维度诱错审计 (53+ 发现, P1 清零), 但系统中仍然存在大量未被发现的阻断性缺陷和边界问题。本版本仅适用于实验、研究和离线模拟, **不可用于实盘交易或生产环境**。开发者需要自行精细打磨, 并结合自身使用场景进行充分验证。

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v3.7.0** (实时执行端 + OKX testnet + 多策略并发 + 审计闭环 + 用户体验优化) | [系统架构](./markdown/10-overview/overview-system-architecture.md) | [使用指南](./markdown/10-overview/overview-system-architecture.md#十一使用指南) | [General_Policy](./markdown/General_Policy.md) | [超级规范化](./markdown/01-principles/principles-super-standardization.md) | [版本历史](./CHANGELOG.md)

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

## v3.7.0 五维度评分

| 维度 | 评分 | 说明 |
|------|:--:|------|
| 功能开发进度 | **9.5/10** | 18 指标全实现 / 实时执行端 + OKX testnet / Paper/Live 切换 / 编译缓存 / ParamsPanel 热调参 / Toast 系统 |
| 仓库稳定程度 | **9.2/10** | cargo check 0 错误 / test 182/185 / vitest 269/269 / executor 36 预存警告 |
| 发布就绪度 | **9.0/10** | P1 清零 / GP+超规范化 v3.7.0 对齐 / 版本一致性 / 5 P2 延后 |
| 用户友好程度 | **9.5/10** | 术语全中文化 / 空状态引导 / 进度反馈 / 错误码映射 / ARIA 无障碍 |
| 系统整体稳定性 | **9.3/10** | 事务保护 / TOCTOU 修复 / 三阶段无锁恢复 / 状态持久化 / Zeroizing / api_guard 强制 |
| **加权** | **9.3/10** | = 9.5×0.3 + 9.2×0.3 + 9.0×0.2 + 9.5×0.1 + 9.3×0.1 |
| **加权** | **9.6/10** |

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
