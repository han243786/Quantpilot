# QuantPilot

> ⚠️ **实验性软件声明**  
> 当前版本 (v4.4.0) 仍处于密集开发阶段。尽管已完成多轮全维度诱错审计并进入嵌套状态机第一波收口, 但系统中仍然可能存在未被发现的阻断性缺陷和边界问题。本版本仅适用于实验、研究和离线模拟, **不可用于实盘交易或生产环境**。开发者需要自行精细打磨, 并结合自身使用场景进行充分验证。

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v4.4.0** (嵌套状态机第一波) | [版本历史](./CHANGELOG.md)

## 项目治理体系

QuantPilot 用三份文档构成完整的项目知识体系。**新开发者必须依次阅读这三份文档**才能开始贡献代码。

```
                      ┌─────────────┐
                      │   全量树     │  ← 第一步: 了解项目里有什么
                      │  全局透明    │
                      └──────┬──────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────┴────────┐    │    ┌────────┴────────┐
     │ 你要改哪些文件？  │    │    │ 你要遵守什么规则？ │
     └────────┬────────┘    │    └────────┬────────┘
              │              │              │
              ▼              │              ▼
     ┌──────────────┐       │     ┌──────────────┐
     │ 代码怎么改？   │ ←────┘     │ 流程怎么走？   │
     └──────┬───────┘             └──────┬───────┘
            │                            │
            ▼                            ▼
   ┌────────────────┐          ┌────────────────┐
   │  GP            │          │  超级规范化     │
   │  实现约束       │          │  流程约束       │
   │                │          │                │
   │  "代码写成      │          │  "开发怎么管"   │
   │   什么样"       │          │                │
   └────────────────┘          └────────────────┘
```

### 全量树 —— 全局透明

> **回答**: 项目里有什么？每个文件是干什么的？改一个功能会影响哪些文件？

全量树是项目的"源代码地图"。它把 160+ 前端文件、47 个后端模块、7 个 Rust crate 逐个拆解, 从系统入口一直展开到每个文件的每个关键函数。**开发者在读任何一行代码之前, 先读这棵树, 就能知道去哪找、改什么。**

- 📄 [全量树](./markdown/10-overview/overview-full-feature-tree.md)
- 🎯 读者: 所有开发者 (尤其是新人/Agent)
- 📏 原则: 每个节点和叶子都有说明, 结构跟着代码实际组织走

### General_Policy (GP) —— 实现约束

> **回答**: 代码写成什么样？什么能做？什么绝对不能做？

GP 是项目的"代码宪法"。44 条规则分为架构铁律、代码规范、禁止事项、存储生命周期、前端设计规范、治理约束六大类。每条标注检查方式: 🛡️ 门禁自动检查, 🔍 审计人工核查。**违反 GP 的 PR 不予合并。**

- 📄 [General_Policy](./markdown/General_Policy.md) — 44 条, 23 条阻断级
- 🎯 读者: 写代码的人
- 📏 原则: 每条款标注检查方式, 与门禁脚本/审计流程互锁

### 超级规范化 —— 流程约束

> **回答**: 开发怎么管？什么阶段做什么检查？版本怎么发布？

超级规范化是项目的"开发程序法"。定义了三层门禁流水线 (pre-commit → PR/CI → closeout-release)、AI 并行审计机制、五维度评分标准、MAJOR 演化通道和元流水线自进化规则。**不通过门禁就不能进入下一阶段。**

- 📄 [超级规范化](./markdown/01-principles/principles-super-standardization.md)
- 🎯 读者: 管流程的人 (也要求所有开发者遵守)
- 📏 原则: 阻断规则不可跳过, S0 必须当前修复

### 三者关系

| | 全量树 | GP | 超级规范化 |
|---|---|---|---|
| 管什么 | 全局透明 | 实现约束 | 流程约束 |
| 问什么 | 有什么？在哪？ | 怎么写？不能写什么？ | 怎么管？怎么查？ |
| 类型 | 地图 | 实体法 | 程序法 |
| 违反后果 | 找不到代码 | PR 不予合并 | 不能进入下一阶段 |
| 更新频率 | 每次文件变更 | MAJOR/MINOR 条款变更 | 流程优化时 |

三份文档**互不重复**: 全量树不抄 GP 条款, GP 不列文件路径, 超级规范不解释功能。三者通过标注互锁 — GP 标注 🛡️ 的条款对应超级规范的门禁脚本, 全量树标注 `[GP §x.x]` 指向约束来源。

### 开发者上手路径

```
第一步: 读全量树      → 了解项目全貌, 知道每个文件干什么
第二步: 读 GP         → 了解代码规则, 知道什么能做/不能做
第三步: 读超级规范化   → 了解开发流程, 知道提交前要跑什么门禁
第四步: 读系统架构     → 了解技术细节, 知道数据怎么流转
```

| 文档 | 路径 |
|------|------|
| 全量树 | [./markdown/10-overview/overview-full-feature-tree.md](./markdown/10-overview/overview-full-feature-tree.md) |
| GP (项目总规则) | [./markdown/General_Policy.md](./markdown/General_Policy.md) |
| 超级规范化 | [./markdown/01-principles/principles-super-standardization.md](./markdown/01-principles/principles-super-standardization.md) |
| 系统架构 | [./markdown/10-overview/overview-system-architecture.md](./markdown/10-overview/overview-system-architecture.md) |
| 使用指南 | [./markdown/10-overview/overview-system-architecture.md#十一使用指南](./markdown/10-overview/overview-system-architecture.md#十一使用指南) |

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

## v4.4.0 流程收口状态

| 项 | 状态 | 说明 |
|----|:--:|------|
| S0 登录挂起 | ✅ | `ring::rand::SystemRandom` 缓存 + refresh token 生成移出 DB 锁 |
| P1 凭证 DELETE 405 | ✅ | Axum 0.7 路由参数语法修正为 `:service` |
| P2 测试进程文件锁 | ✅ | `scripts/test.ps1` / `scripts/test.sh` 在测试前停止本仓库运行进程 |
| 三层工作流门禁 | ✅ | pre-commit / CI / closeout-release 三层已统一 |
| 功能演进契约 | ✅ | 新能力必须登记能力边界、回归保护矩阵、兼容性与迁移说明 |
| Rust 格式基线 | ✅ | 全仓 `cargo fmt` 已落地，pre-commit / CI / closeout 均执行 `cargo fmt --check` |
| v4 runtime 入口 | ✅ | 后端 `/api/runtime/v4/run`、CLI `v4-run`、前端 `start_v4_simulation` capability 已接入 |
| 执行端 v4 集成 | ✅ | RunnerPool、部署 API、OKX Market 事件、SSE evidence 和执行端前端面板按 v4.2.0 规划落实 |
| v4 回测 + 多交易对 | ✅ | `/api/runtime/backtest` 可走 `runtime_kind=v4`, 回测工件包含 `v4_artifact`, v4 模板和多交易对 MachineGraph 展开已接入 |
| 嵌套状态机第一波 | ✅ | `MachineState.child_machine`、v4 QS state 内嵌套 machine、父优先 runtime 路由、层级 snapshot 和复杂度预算面板已接入 |
| 版本一致性 | ✅ | Cargo、Tauri、前端 package、lockfile、release manifest、OpenAPI 和启动横幅统一到 `4.4.0` |
| executor warning 债务 | ✅ | 当前预算 0；新增 warning 会失败 |
| 完整 closeout | ✅ | v4.4.0 嵌套状态机规划方案已通过 24/24 closeout 门禁 |

## 更多文档

| 文档 | 路径 |
|------|------|
| 文档索引 (全部文档列表) | `./markdown/README.md` |
| 系统架构与使用手册 | `./markdown/10-overview/overview-system-architecture.md` |
| 当前状态与路线图 | `./markdown/10-overview/overview-current-status-and-roadmap.md` |
| RFC 协议索引 (001-020) | `./markdown/02-protocol/README.md` |
| 实现契约目录 | `./markdown/03-implementation/governance/` |
| 功能演进契约 | `./markdown/03-implementation/governance/implementation-feature-evolution-contract.md` |
| 支持矩阵 | `./markdown/03-implementation/governance/implementation-support-matrix.md` |
| 里程碑归档 (50+ 版本) | `./markdown/06-milestones/` |
| 审计与测试报告 | `./markdown/05-testing/` |
