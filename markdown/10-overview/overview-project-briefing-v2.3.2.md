# QuantPilot v2.3.2 项目全量简报

> 简报日期: 2026-05-18 | 代码版本: v2.3.2 | 基于 4 个 Agent 全量并行读取

---

## 一、项目身份

| 维度 | 内容 |
|------|------|
| **名称** | QuantPilot |
| **定位** | 单机量化策略研发沙盒 |
| **当前版本** | v2.3.2 |
| **仓库地址** | `D:\rust-js-pr\QuantPilot\quantpilot` |
| **许可证** | 待定 (LICENSE 为占位符) |
| **代码总量** | ~178,000 行 (Rust 76K + JS/JSX 44K + CSS 8K + 测试 20K + 文档 29K) |

### 明确的"是"与"不是"

| 是 | 不是 |
|----|------|
| 策略设计、编译验证、Paper 运行、历史回测 | 交易执行平台 |
| 本地桌面应用 (Tauri v2) | 行情终端 |
| 18 种技术指标全部有 evaluator 实现 | 策略托管/SaaS 服务 |
| 策略对比分析工具 | 社交跟单平台 |

---

## 二、技术栈总览

```
桌面壳:  Tauri v2 (自绘标题栏, WebView2)
前端:    React 18 + Vite 6 + Zustand 4 + React Flow 12
         设计系统: Adobe 暗色面板 (--ad-* CSS 令牌 ~50个)
         测试: Vitest (92文件/269测试) + Playwright (E2E)
后端:    Rust Axum 0.7 + Tokio
         5 子 crate: qrpc_core_ir → qrpc_core → qrpc_compiler → qrpc_runtime → quantscript
存储:    文件系统分级 (Permanent/Temporary/Transient, 500MB 全局配额)
加密:    AES-256-GCM (ring crate), PBKDF2 密钥派生, Zeroizing on drop
部署:    Docker (三阶段构建) + NSIS Windows 安装器 + GitHub Actions CI
```

---

## 三、架构分层 (自底向上)

### 3.1 编译链 (唯一核心路径)

```
策略图 JSON / QuantScript 源码 / StrategyIR JSON
              │
              ▼
    quantscript crate: Tokenize → Parse → AST → HIR
              │
              ▼
    qrpc_core_ir: CoreStrategyIr (data_bindings + indicators + signal_rules
                   + agent_policies + risk_policies + execution)
              │
              ▼
    qrpc_compiler: RuntimeProtocolCoreConfig → CompiledRuntimeProtocol
                   (图循环检测 + 自动风险检查器 + 冲突检测)
              │
              ▼
    qrpc_runtime: RuntimeCoordinator (6 阶段主链)
```

### 3.2 运行时主链 (不可绕过)

```
DataCollection → IntentComputation → AgentDecision → RiskCheck → ExecutionPlan → FillEngine
     │                │                  │              │            │             │
     │                │                  │              │            │             └── 订单撮合
     │                │                  │              │            └── 计划订单
     │                │                  │              └── 5 层风控 + 17 种拒绝原因
     │                │                  └── 3 种策略 (加权/再平衡/跨交易所套利)
     │                └── 18 种指标 evaluator (零 stub)
     └── OKX V5 + Binance + 模拟数据 + 历史缓存
```

### 3.3 前端路由表 (11 个页面)

| 路由 | 页面 | 职责 |
|------|------|------|
| `/strategies` | StrategyHubPage | 策略目录 |
| `/strategies/:id` | StrategyWorkspacePage | 策略工作台 (画布/研究/源码/仪表盘) |
| `/strategies/:id/backtests` | StrategyBacktestsPage | 回测历史 |
| `/backtests/:id` | BacktestDetailPage | 回测详情 |
| `/backtests/compare` | BacktestComparePage | 多回测对比 |
| `/approvals` | ApprovalPage | AI 提案审批 |
| `/alerts` | AlertsPage | 告警面板 |
| `/snapshots` | SnapshotsPage | 快照管理 |
| `/runbook` | RunbookPage | 故障手册 |
| `/chaos` | ChaosPage | 混沌实验 |
| `/quantscript` | QuantScriptEditor | QS 源码编辑器 |

---

## 四、核心数字指标

| 指标 | 数值 |
|------|------:|
| Rust 子 crate | 5 |
| Rust 源文件 | ~357 |
| Rust 代码行 | ~76,438 |
| JS/JSX 代码行 | ~44,134 |
| CSS 代码行 | ~8,344 |
| 测试代码行 | ~19,951 |
| 文档行 | ~29,347 |
| 活跃 RFC | 20 (19 ✅ 已落地) |
| 技术指标种类 | 18 (全部有 evaluator) |
| 支持交易所 | 2 (binance, okx) |
| 支持交易对 | 3 (BTCUSDT, ETHUSDT, SOLUSDT) |
| 前端测试文件 | 92 |
| 前端测试用例 | 269 |
| 后端测试用例 | ~115 |
| API 路由 | 42+ |
| CI 门禁步数 | 13 |
| PluginKind | 5 (Data/Intent/Agent/Risk/Execution) |
| RiskReasonCode | 17 种拒绝原因 |
| 存储配额 | 500 MB |

---

## 五、版本演进轨迹

| 版本 | 核心交付 |
|------|---------|
| v0.1.0 | 私有基线 (Paper 运行时 / 图编辑器 / QS 编译管道) |
| v0.2.0 - v0.4.3 | TestRunner / 信号扩展 / UI 简洁化 / 安全审计 |
| v0.5.x | Adobe 前端重构 / 38 项全量审计 |
| v1.0.0 | 插件化架构 / 超级规范化 / 重型策略 |
| v1.0.x - v1.4.x | 边界防御 / 15+ 轮诱错审计 / 技术债清零 |
| v2.0.0 | OKX 实盘 / 多用户 / 插件市场 / 打包 |
| v2.1.x | 97 项 P1-P3 清零 (断路器/备份/NaN 防御) |
| v2.2.x | 架构重构 (Coordinator 拆分 / tracing / i18n) |
| v2.3.x | 错误国际化 41 码 / TLS / JWT 刷新 / ISP 拆分 |

---

## 六、关键架构决策

1. **QS 唯一编译路径** — 所有策略必须经过 `QS 源码 → parse → HIR → lower → Core IR`，禁止绕过
2. **双周期运行时** — 慢周期 (K 线驱动: MA/RSI/MACD) + 快速周期 (报价驱动: QuoteObserve)
3. **代际屏障激活** — 配置变更使用代际编号而非挂钟时间，确保确定性回放
4. **内容寻址签名快照** — SHA-256 规范 JSON 摘要，防篡改参数验证
5. **ISP 拆分 (v2.3.2)** — ExecutionModuleProvider 拆为 ExecutionPlanner + ExecutionSubmitter
6. **三级存储生命周期** — Permanent / Temporary (7d) / Transient (1h)，500MB 硬上限
7. **确定性回放** — Box-Muller 确定性正态分布 + 固定种子模拟 + 代际屏障
8. **插件双层模型** — Atom (最小可组合单元) + Suite (纯打包层)
9. **五维度审计** — 功能进度 / 仓库稳定 / 发布就绪 / 用户友好 / 系统稳定
10. **十角色诱错** — 7 用户角色 + 3 内部角色，S0 场景必须全量通过

---

## 七、当前版本状态

### v2.3.2 (2026-05-18)

**本次变更:**
- 集成测试编译错误修复 (credential_api crate→super + Serialize)
- 冗余启动脚本和构建产物清理
- 文档同步 (README/CHANGELOG/里程碑/概览)

**Git 状态:** `b7d2d5d` (最新提交)

### 能力边界

| 维度 | 已支持 | 未宣称 |
|------|--------|--------|
| 运行模式 | paper, backtest | live trading |
| 交易所 | binance, okx | 其他交易所 |
| 交易对 | BTCUSDT, ETHUSDT, SOLUSDT | 其他 |
| 指标 | 18 种全实现 | 研究级回测语义 |
| 订单类型 | Market, Limit, StopLoss, TakeProfit | 复杂衍生品 |
| 插件 | 本地注册表 | 第三方市场 |

---

## 八、项目文件结构速览

```
quantpilot/
├── src/                    # 主二进制 crate (33K 行) — API/服务/状态
├── src-tauri/              # Tauri 桌面壳
├── quantscript/            # QuantScript DSL 编译器 (17K 行)
├── qrpc_runtime/           # 运行时引擎 (20K 行) — 沙盒/回测/执行
├── qrpc_core/              # 核心域类型 (3K 行) — 事件/订单/组合
├── qrpc_compiler/          # 协议编译器 (3K 行)
├── qrpc_core_ir/           # 核心 IR 类型 (0.6K 行)
├── frontend/src/           # React SPA (44K 行)
│   ├── components/         # UI 组件 (~25 个)
│   ├── store/              # Zustand 状态管理 (~25 切片)
│   ├── hooks/              # 业务逻辑 hooks (~22 个)
│   ├── pages/              # 11 个页面
│   ├── modules/            # 14 个内置模块
│   ├── graph/              # 图编译引擎
│   ├── i18n/               # 国际化 (zh-CN/en-US)
│   └── api/                # API 客户端
├── markdown/               # 文档 (250+ 文件)
│   ├── 01-principles/      # 架构设计哲学
│   ├── 02-protocol/        # RFC 001-020 协议规范
│   ├── 03-implementation/  # 行为契约
│   ├── 04-guides/          # 用户操作指南
│   ├── 05-testing/         # 审计报告/诱错矩阵
│   ├── 06-milestones/      # 版本规划 (60+ 文件)
│   ├── 08-research/        # 研究文档
│   ├── 09-archive/         # 已退役文档
│   ├── 10-overview/        # 系统架构/状态索引
│   └── General_Policy.md   # 项目总规则 (8 章核心规范)
├── tests/                  # 集成测试 (5K 行, 12 文件)
│   └── scenarios/          # QS 场景文件 (26 个 .qs)
├── tools/                  # DevOps 工具链 (21 文件)
│   ├── check-*.ps1         # 静态分析门禁 (UTF-8/文本/能力/i18n)
│   ├── run-*.js            # 测试编排
│   └── build_package.js    # 发布打包
├── config/                 # 示例配置与 JSON Schema
├── contracts/              # OpenAPI/AsyncAPI/Spectral 规范
├── plugins/                # 插件目录 (builtin + installed)
├── storage/                # 运行时数据 (Permanent/Temporary/Transient)
├── scripts/pre-commit      # Git pre-commit 钩子
├── packaging/windows/      # NSIS 安装器脚本
└── .github/workflows/      # CI/CD (ci.yml / release.yml / scenario-test.yml)
```

---

## 九、GP 核心开发指引

> 以下基于 `General_Policy.md` (8章) 和 `principles-super-standardization.md` (9章) 提取核心开发约束。

### 9.1 架构铁律 (4 条不可违背)

| # | 规则 | 违反后果 |
|---|------|---------|
| §1.1 | QS 是唯一策略定义路径 — 所有策略必须经 `图→QS源码→parse→HIR→lower→Core IR` | 禁止合并 |
| §1.2 | 新增功能跨三层验证 — QS解析 + CoreIR + 运行时 + 前端 + 端到端 | 禁止合并 |
| §1.3 | 编译路径不可绕过 — 禁止直接构造 RuntimeProtocolCoreConfig | 禁止合并 |
| §1.4 | 数据流单向 — QS源码→graph JSON→前端可视化, 保存时不可覆盖原始QS | 合并前修复 |

### 9.2 代码规范 (5 条)

- **§2.1** 所有 `bail!`/`anyhow!`/`Err()` 用户可读文本必须是中文
- **§2.2** 测试断言使用中文子串匹配错误消息
- **§2.3** 新 indicator/evaluator 必须有单元测试
- **§2.4** 新 TestAction 必须有 .qs 集成场景
- **§2.5** 前端字符串用 `t()` 包裹 (国际化准备)

### 9.3 禁止事项 (5 条)

- **§5.1** 禁止硬编码魔数 — 必须从配置/参数读取
- **§5.2** 禁止静默忽略参数 — 必须使用或报错
- **§5.3** 禁止 stub evaluator — 必须有完整计算实现
- **§5.4** 禁止在图编辑器中绕过 QS 编译
- **§5.5** 禁止跳过端到端验证

### 9.4 前端设计规范 (8 条)

- 配色饱和度限制 — 仅 Adobe 蓝 `#1473e6`，低饱和状态色，所有颜色通过 `--ad-*` 令牌
- 圆角 ≤ 6px，背景纯色 `#0d0d0d`，禁止渐变/毛玻璃
- 图标必须是 SVG 组件，禁止 Unicode emoji
- 所有字符串用 `t()` 包裹，`data-testid` 必须设置

### 9.5 存储生命周期 (5 条)

| 级别 | TTL | 示例 | 每目录上限 |
|------|-----|------|:---:|
| Permanent | 无上限 | graphs/, audit/, .credentials | 豁免 |
| Temporary | 7天 (DEV 1天) | runs/, backtests/, experiments/ | 200MB |
| Transient | 1小时 (DEV 10分钟) | snapshots/, alerts/, chaos/ | 50MB |

全局硬上限: 500MB

### 9.6 五条流水线

```
设计 → 开发 → 检查 → 审计 → 优化 → (元流水线自审计)
```

**检查流水线 10 项门禁:**

| # | 门禁 | 阻断级别 |
|---|------|:---:|
| 1 | UTF-8 编码检查 | 阻断 |
| 2 | 面向用户文本检查 | 阻断 |
| 3 | 能力治理快照检查 | 阻断 |
| 4 | cargo check --workspace | 阻断 |
| 5 | cargo test --workspace | 阻断 |
| 6 | cargo clippy -- -D warnings | 高 |
| 7 | npm run build | 阻断 |
| 8 | vitest run | 阻断 |
| 9 | Playwright E2E | 阻断 |
| 10 | npm audit (moderate+) | 阻断 |

### 9.7 审计标准

**五维度评分:**
1. 功能开发进度 (1-10)
2. 仓库稳定程度 (1-10)
3. 发布就绪度 (1-10)
4. 用户友好程度 (1-10)
5. 系统整体稳定性 (1-10)

**GP 合规矩阵:** §1-§8 逐条核查 ✅/❌

**十角色诱错:** 7 用户角色 + 3 内部角色，共 38 S0 场景必须全量通过

**自由维度诱错:** 5 维度 (逻辑契约/并发竞态/边界数值/序列化持久化/API错误) 可多轮反复执行

### 9.8 PR 提交前 10 项检查单

| # | 检查项 |
|---|--------|
| 1 | 错误消息是否全中文 |
| 2 | 测试是否全通过 (`cargo test --workspace`) |
| 3 | 前端是否可构建 (`npx vite build`) |
| 4 | indicator 是否有 evaluator (无 "not yet implemented") |
| 5 | 新功能是否有 .qs 场景 (`ls tests/scenarios/`) |
| 6 | capability 变更是否更新了固件 |
| 7 | 参数是否被静默忽略 (搜索 `_: `) |
| 8 | 文档是否放到正确目录 (对照分层表) |
| 9 | storage 写入是否声明了生命周期 |
| 10 | storage 是否超过配额 (< 500MB) |

---

## 十、开发工作流入口

### 启动开发环境

```bat
.\start.bat
```

### 一键全部门禁

```powershell
.\tools\run-closeout-gates.bat
```

### 关键文档索引

| 文档 | 路径 | 用途 |
|------|------|------|
| 项目总规则 | `markdown/General_Policy.md` | 代码规范/禁止事项/存储/前端设计 |
| 超级规范化 | `markdown/01-principles/principles-super-standardization.md` | 五条流水线/审计/门禁 |
| 系统架构 | `markdown/10-overview/overview-system-architecture.md` | 完整架构参考手册 |
| 协议规范 | `markdown/02-protocol/RFC-*.md` | 20 个 RFC 数据结构定义 |
| QS 语法 | `markdown/04-guides/guide-formal-quantscript-syntax.md` | QuantScript 语法参考 |
| 十角色诱错 | `markdown/09-archive/testing-retired/十角色全量诱错测试矩阵.md` | 诱错测试矩阵 |

---

## 十一、建议关注点

1. **测试覆盖** — 前端 E2E 测试当前需要后端运行 (60 失败因服务器未启动)，建议增加 mock server 模式
2. **node_modules** — 根目录有残留 `.vite` 缓存，已加入 .gitignore
3. **storagebacktests/** — 空目录，可能是历史遗留
4. **LICENSE** — 当前为占位符，建议项目所有者正式确定
5. **插件市场** — 客户端代码已就绪，但仅用于本地元数据，未接入真实市场
6. **实盘交易** — 代码中有 `live_execution.rs` (OKX V5 签名/限速)，但 `/api/capabilities` 明确声明 `live_execution_allowed: false`
