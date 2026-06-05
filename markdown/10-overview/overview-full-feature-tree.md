# QuantPilot 全量树 v4.7.0

> 本文档是 QuantPilot 的"源代码地图"——开发者通过这棵树可以彻底了解项目的每一个模块、每一个文件、每一个功能。
> 与 GP (实现约束) 和超级规范化 (流程约束) 配合使用: GP 管代码写成什么样, 超级规范化管开发怎么管, 全量树管项目里有什么。

---

## 0. 使用说明与边界

### 0.1 本文档回答什么

| 问题 | 如何回答 |
|------|---------|
| 项目里有什么？ | 7 个根节点覆盖全部 active 源文件 |
| 某个功能在哪？ | 从功能反查代码路径 (见附录 C: GP 约束快速索引) |
| 改一个文件会影响什么？ | 每个文件节点标注了"什么时候改这里"和所属子系统 |
| 新增文件要改哪？ | 参见 §0.4 维护规则 |
| 这个文件为什么存在？ | 每个子系统有"职责"说明, 每个文件有"做什么"一行说明 |

### 0.2 与 GP / 超级规范化的分工

| | 全量树 | GP | 超级规范化 |
|---|---|---|---|
| 管什么 | 全局透明 | 实现约束 | 流程约束 |
| 类型 | 地图 | 实体法 | 程序法 |
| 读者 | 所有开发者 | 写代码的人 | 管流程的人 |
| 更新触发 | 文件变更时 | 条款变更时 | 流程优化时 |

**互不重复**: 全量树不抄 GP 条款全文, 只在相关节点标注 `[GP §x.x]` 和一句影响说明。超级规范化不在全量树中重复解释, 只在根6/根7说明流程入口。

### 0.3 覆盖范围

**必须覆盖的 active 文件**:
- `src/**/*.rs`
- `src-executor/**/*.rs`
- `qrpc_*/src/**/*.rs`
- `quantscript/src/**/*.rs`
- `src-tauri/**/*.rs`
- `frontend/src/**/*.{js,jsx}`
- `frontend-executor/src/**/*.{js,jsx}`
- `tools/**/*.{ps1,bat,js,rs}`
- `scripts/**/*.{ps1,bat,js}`
- `contracts/**/*.yaml`
- `config/**/*.{yaml,json}`
- `release/*.yaml`
- `tests/**/*.rs`
- `frontend/tests/**/*.{js,jsx}`
- 根目录活配置: `Cargo.toml`, `start.bat`, `start.ps1`, `package.json` 等

**不要求逐文件展开**:
- `target/`, `node_modules/`, `frontend/dist/`, `storage/`, `tmp/`
- `markdown/learning/` (GP §1.11: 本地忽略)
- 历史里程碑归档 (`markdown/06-milestones/vX.Y.Z/`) 按版本目录摘要
- `frontend/test-results/`, `markdown/测试/test-reports/` (生成物)

### 0.4 维护规则 (阻断级)

1. **代码变更必须同步全量树** — 新增/删除/重命名 active 文件时, 必须更新对应树节点和附录 E。

2. **树结构跟代码结构走** — 不创造代码里不存在的抽象层级; 抽象分组必须能落到真实路径。

3. **每个 active 文件至少一行说明** — 说明它做什么、属于哪个子系统、什么时候需要改它。

4. **GP 标注挂在功能节点上** — 全量树不复制 GP 条款全文, 只在相关节点标注受哪些 GP 条款约束, 格式: `[GP §x.x]: 约束说明`。

5. **新增能力标注引入版本** — 新增功能节点使用 `🆕 vX.Y.Z` 标注。

6. **路径必须 repo-relative 完整路径** — 使用 `src/runtime/run/session_start.rs`, 禁止 `runtime/run/session_start.rs` 这种上下文相对路径。

7. **行数不手写为事实来源** — 不使用具体行数。若保留统计数字, 必须来自脚本输出; 否则只写功能描述。

8. **禁止占位符** — 文档中不得出现待处理标记、工程占位标记或错误版本号。

### 0.5 节点标注规范

**子系统节点** (固定格式):

```
### X.Y 子系统名

**职责**: 一句话说明这个子系统做什么。
**入口文件**: 关键公共入口和跨层边界。
**关键数据流**: 数据在这个子系统中的流转路径。
**主要约束**: 适用的 GP 条款和约束说明。
**验证命令**: 如何验证这个子系统正常工作。
```

**文件叶子节点** (固定格式):

```
- `path/to/file.ext` — 做什么; 什么时候改这里。
```

---

## 根1: 系统入口与进程拓扑

**一句话**: 系统由 4 个可独立运行的进程/服务器组成, 通过 `start.bat` 一键编排启动。

```
用户桌面
  │
  └─ start.bat (编排脚本)
       │
       ├── Step 1: cargo build --bin quantpilot
       ├── Step 2: 启动后端 quantpilot.exe → 监听 :3000
       └── Step 3: cargo tauri dev → Tauri 壳 → 等待后端就绪 → 打开桌面窗口
                                                      │
                                                    WebView2 加载前端 → 连接 :3000
```

### 1.1 编排脚本: `start.bat`

**文件**: `start.bat` (根目录)

开发环境一键启动脚本。做的事:
- 设置 `QUANTPILOT_DEV=true` — 跳过认证和限速, 缩短 TTL
- 杀掉旧进程 (quantpilot.exe on :5173, quantpilot-tauri.exe)
- Step 1: `cargo build --bin quantpilot` — 编译后端
- Step 2: 后台启动后端, 轮询 `:3000 LISTENING` (最多 30 次 x 2s)
- Step 3: `cd src-tauri && cargo tauri dev` — 启动 Tauri 桌面壳

**替代启动方式** (`start.ps1`): PowerShell 版本, 同样逻辑。

### 1.2 Tauri 桌面壳

**文件**: `src-tauri/src/main.rs`

Tauri v2 桌面壳入口。做的事:
- 等待后端 `127.0.0.1:3000` 就绪 (最多 30s, 每秒尝试一次 TCP 连接)
- 构建 Tauri 应用: 自绘标题栏 (`decorations: false`), 窗口 1400×900, 最小 960×600
- 开发模式下自动打开 WebView2 DevTools

**配置文件**: `src-tauri/tauri.conf.json`
- `productName`: "QuantPilot"
- `identifier`: "com.quantpilot.app"
- CSP (内容安全策略): 只允许 `'self'` + `localhost:5173` + `127.0.0.1:3000`
- NSIS 安装器: `currentUser` 模式

**前端目录**: `src-tauri/` 的 `build.frontendDist` 指向 `../frontend/dist`

### 1.3 后端服务 (:3000)

**文件**: `src/main.rs` → `src/system/entry/backend_process.rs` → `src/lib.rs` → `src/backend/mod.rs` → `src/app_router.rs`

```
src/main.rs                         →  tokio::main, 调用 quantpilot::run_server()
src/system/entry/backend_process.rs →  system.entry.backend_process, 承载 run_server()
                                       run_api_server() 构建 Axum Router, 绑定 :3000
src/lib.rs                          →  crate root 兼容 re-export, 加载 system/backend 模块
src/backend/mod.rs                  →  backend 父模块, 汇总 9 个叶子 facade
src/backend/interface_boundary.rs   →  backend.interface_boundary, route owner 父级 facade
src/app_router.rs                   →  build_app_router(), 经 backend.interface_boundary 注册 HTTP 路由
```

后端是单进程 Axum 0.7 HTTP 服务器, 使用 tokio 多线程运行时。所有功能通过模块化组织在 `src/` 下的 Rust 文件中。

**启动逻辑** (`src/system/entry/backend_process.rs`):
- 加载 `.env` (dotenvy)
- 初始化 tracing-subscriber (日志格式 compact/json)
- 初始化凭证保险库 (`CredentialVault`)
- 初始化存储生命周期
- 调用 `build_app_router()` 构建路由
- `axum::serve` 绑定 `127.0.0.1:3000`

**测试入口**: `src/tests_backend.rs` — 集成测试入口, 使用 `tower::ServiceExt` 发送 HTTP 请求。

**详细模块展开**: 见 [根2: 后端服务](#根2-后端服务-3000)

### 1.4 执行端 (:3001)

**文件**: `src-executor/main.rs` → 独立 binary `cargo run --bin executor`

```
Cargo.toml [[bin]] name="executor", path="src-executor/main.rs"
```

执行端是**独立进程**, 与后端分离运行。职责:
- 策略部署/启动/停止/热调参
- OKX WebSocket 行情连接
- `PaperSimulated` 实时模拟盘: 本地模拟成交 (Live Runner)
- `PaperActual` OKX 模拟盘: provider submit/query/cancel 回执路径, 固定 demo flag=1 与 `x-simulated-trading: 1`
- v4 策略启动前消费后端生成的 `strategy_config_preflight`, 不在执行端重新推断 capability
- 独立凭证保险库 (v2, PBKDF2 100 万轮)

**启动逻辑** (`src-executor/main.rs`):
- 初始化 `qrpc_session` 会话密钥 (进程间加密通道)
- 构建 Axum Router, 绑定 `127.0.0.1:3001`
- SSE 端点 `/api/executor/events` — 向后端推送策略状态变更

**详细模块展开**: 见 [根3: 执行端](#根3-执行端-3001)

### 1.5 前端开发服务器 (:5173)

**文件**: `frontend/package.json` → `npm run dev` → Vite

```
Vite 6 dev server → http://localhost:5173
  │
  ├── 代理 /api → http://127.0.0.1:3000 (后端)
  └── HMR (热模块替换) 实时刷新
```

开发时前端独立运行, 通过 Vite proxy 转发 API 请求到后端。生产构建 (`npm run build`) 产出静态文件到 `frontend/dist/`, Tauri 壳直接加载。

**详细模块展开**: 见 [根5: 前端 React SPA](#根5-前端-react-spa)

### 1.6 进程间通信

```
┌─────────────────────────────────────────────────────┐
│                   Tauri 壳 (WebView2)                │
│  ┌──────────────────┐    HTTP     ┌────────────────┐│
│  │  前端 React SPA   │ ←────────→ │ 后端 :3000      ││
│  │  localhost:5173   │  /api/*    │  Axum HTTP      ││
│  └──────────────────┘            │  47 模块        ││
│                                   └───────┬────────┘│
│                                           │          │
│                                    qrpc_session     │
│                                    (加密通道)        │
│                                           │          │
│                                   ┌───────┴────────┐│
│                                   │ 执行端 :3001     ││
│                                   │ 独立进程         ││
│                                   │ OKX 模拟盘      ││
│                                   └────────────────┘│
└─────────────────────────────────────────────────────┘
```

- **前端 ↔ 后端**: HTTP REST API (`/api/*`), SSE 事件流
- **后端 ↔ 执行端**: `qrpc_session` crate 提供 AES-256-GCM + HMAC-SHA256 加密通道
- **执行端 ↔ OKX**: WebSocket (行情) + REST (OKX 模拟盘 provider 回执)

v4 provider 范围: v4 只确保 OKX 单一 provider 切面; 美股、港股、A股、贵金属、大宗商品、期货、期权和其他主流 provider 适配统一延后到 v5。v5 只接受支持 WebSocket 全双工或等效实时订单/行情事件回执的 provider。

---

## 根2: 后端服务 (:3000)

**一句话**: Axum 0.7 HTTP 服务器, Rust 模块按功能分为 backend 九叶 facade 和既有 handler 子系统, 是整个 QuantPilot 的核心。

### 2.-1 backend 九叶模块壳

**父模块文件**:
- `src/backend/mod.rs`
- `src/backend/interface_boundary.rs`
- `src/backend/capability.rs`
- `src/backend/strategy_config.rs`
- `src/backend/runtime.rs`
- `src/backend/graph_compile.rs`
- `src/backend/storage_security.rs`
- `src/backend/ops_governance.rs`
- `src/backend/app_state_wiring.rs`
- `src/backend/test_support.rs`
- `src/backend/interface_boundary/app_state_bridge.rs`
- `src/backend/interface_boundary/capability_bridge.rs`
- `src/backend/interface_boundary/graph_compile_bridge.rs`
- `src/backend/interface_boundary/ops_governance_bridge.rs`
- `src/backend/interface_boundary/runtime_bridge.rs`
- `src/backend/interface_boundary/storage_security_bridge.rs`
- `src/backend/interface_boundary/strategy_config_bridge.rs`
- `src/backend/interface_boundary/test_support_bridge.rs`
- `src/backend/capability/snapshot.rs`
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/backend/runtime/routes/run.rs`
- `src/backend/graph_compile/compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs`
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs`
- `src/backend/storage_security/credential_vault/implementation/secret_pattern_extraction.rs`
- `src/backend/storage_security/credential_vault/implementation/type_surface.rs`
- `src/backend/storage_security/credential_vault/implementation/tests.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs`
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/atomic_save_commit.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/load_restore_entry.rs`
- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/chaos.rs`
- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/runbook.rs`
- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/snapshots.rs`
- `src/backend/app_state_wiring/health_route.rs`
- `src/backend/app_state_wiring/state_factory.rs`
- `src/backend/test_support/scenario.rs`
- `src/backend/strategy_config/artifact.rs`
- `src/backend/strategy_config/artifact/builder_core.rs`
- `src/backend/strategy_config/artifact/domain_projection.rs`
- `src/backend/strategy_config/artifact/schema_model.rs`
- `src/backend/strategy_config/preflight.rs`
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/diff/artifact_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`
- `src/backend/strategy_config/diff/evidence_diff/risk_plane.rs`
- `src/backend/strategy_config/diff/evidence_diff/execution_capability.rs`
- `src/backend/strategy_config/diff/evidence_diff/metrics.rs`
- `src/backend/strategy_config/ai_proposal_binding.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/backtest/v4_runtime_execution.rs`
- `src/runtime/backtest/legacy_dispatch.rs`
- `src/runtime/backtest/record_store.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/parameter_grid.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/shared_governance.rs`
- `src/runtime/query_support.rs`
- `src/runtime/response_support.rs`

**抽离口径**: v4.16 BE-001B 已建立 9 个叶子 facade，BE-001C 已完成九叶逐叶 closeout，BE-001D 已启动 `backend.strategy_config` L3 模块壳抽离，BE-001E 已完成其余八叶薄壳抽离，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`，BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`，BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout 并设置 `stop_split: false`，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`，BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`，BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断并设置父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout，BE-001AH-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout 并设置 `stop_split: true`，BE-001AJ-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单叶 closeout 并设置 `stop_split: true`，BE-001AK-01 已完成 `transition_lifecycle` 第二轮父叶残余判断，BE-001AL-04 已完成 `rollback_flow` 单叶 closeout 并设置 `stop_split: true`，BE-001AM-01 已完成 `transition_lifecycle` 第三轮父叶残余判断，BE-001AN-04 已完成 `activation_snapshot_side_effect` 单叶 closeout 并设置 `stop_split: true`，下一步进入 BE-001AO-01 父叶残余判断。`src/app_router.rs` 通过 `backend.interface_boundary` 进入各叶子；state owner、response schema 和 artifact schema 仍按各模块白箱边界保留。

**抽离补充**: BE-001AD-01 已确认 `backend.runtime.routes` 父叶残余判断完成但父叶保持 `stop_split: false`；BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout；BE-001AN-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout，下一步只能进入 BE-001AO-01 父叶残余判断。

**细分判断**: `backend.interface_boundary`、`backend.capability`、`backend.app_state_wiring`、`backend.test_support` 本阶段停止细分；`backend.strategy_config`、`backend.runtime`、`backend.graph_compile`、`backend.storage_security`、`backend.ops_governance` 值得进入下一轮 L3 等价基线，其中 `backend.storage_security` 必须先过安全决策暂停。

### 2.0 路由总表

**文件**: `src/app_router.rs`

`build_app_router()` 函数定义了全部 HTTP 端点。路由按功能分组:

| 路由前缀 | 功能域 | 关键模块 |
|---------|--------|---------|
| `/api/graph/*` | 策略图 CRUD | `src/backend/graph_compile/graph.rs` |
| `/api/runtime/compile` | 编译 | `src/backend/graph_compile/compile.rs` |
| `/api/v1/strategy-config/*` | v4 策略配置契约 / preflight / diff / evidence diff helper | `strategy_config_api.rs` |
| `/api/runtime/run` | 纸面运行 | `src/backend/runtime/routes/run.rs`、`src/runtime/event_stream.rs`、`src/runtime/run/session_start.rs`、`src/runtime/run/v4_handoff.rs`、`src/runtime/run/record_store.rs`、`src/runtime/run/replay_status.rs` |
| `/api/runtime/backtest/*` | 回测 | `src/backend/runtime/routes/backtest.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_runtime_execution.rs`、`src/runtime/backtest/legacy_dispatch.rs`、`src/runtime/backtest/record_store.rs`、`src/runtime/backtest/replay.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/backtest_compare.rs` |
| `/api/runtime/experiments/*` | 实验/参数扫描 | `src/backend/runtime/routes.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/record_lifecycle.rs` |
| `/api/runtime/mutations/*`、`/api/runtime/ai-proposals/*`、`/api/v1/ai/*` | 运行时变更 / AI proposal / approval | `src/backend/runtime/routes/mutation.rs`、`src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/shared_governance.rs` |
| `/api/auth/*` | 本地会话认证 | `auth/mod.rs` |
| `/api/credentials/*` | 凭证管理 | `credential_api.rs` |
| `/api/v1/alerts/*` | 告警 | `alert_engine.rs` |
| `/api/v1/snapshots/*` | 快照 | `snapshot_service.rs` |
| `/api/v1/approvals/*` | 审批 | `runtime/mutation.rs` |
| `/api/v1/chaos/*` | 混沌实验 | `chaos_experiment.rs` |
| `/api/v1/runbook/*` | 运行手册 | `runbook.rs` |
| `/api/capabilities` | 能力声明 | `src/backend/capability/snapshot.rs` |
| `/api/quantscript/formal/*` | QS 正式编译 | `src/backend/graph_compile/compile.rs` |
| `/api/collaboration/*` | 协作 | `collaboration.rs` |
| `/api/hotswap/*` | 模块热替换 | `src/backend/ops_governance/hotswap/handlers.rs` |
| `/api/migration/*` | 数据迁移 | `migration_sender.rs` |
| SPA fallback | 前端静态文件 | `dist/index.html` |

[GP §4.4]: 新增路由必须在 `app_router.rs` 注册, SPA fallback 不可删除。

### 2.1 编译系统

**职责**: 将策略图 (graph JSON) 或 QuantScript 源码编译为可执行的 Core IR。

**数据流**: `graph JSON → QS 源码 → parse → HIR → lower → Core IR → sandbox`

```
src/backend/graph_compile/compile.rs — 编译入口, /api/runtime/compile
  ├── compile_runtime_protocol_via_qs()  — QS 路径编译 (主路径)
  ├── compile_runtime_protocol()         — graph 直接编译 (内部路径)
  └── 编译缓存: LRU 50 条, key=(graph_hash, compile_options_hash)

compile_diagnostics.rs          — 编译诊断生成
  └── 结构化诊断: 错误码 + 文件位置 + 修复建议

compile_artifact_builders.rs    — 编译产物构建
  └── 组装 RuntimeProtocolCoreConfig, StrategyPackage, MigrationPackage

backend/graph_compile/quantscript_graph.rs — graph JSON → QS 源码
  └── generate_quantscript_from_graph_value() — 前端图编辑器的唯一合法出口

graph_version_compare.rs        — 图版本对比
  └── 比较两个图版本的差异, 用于版本历史；图版本 compare 响应会附带 `strategy_config_diff` 配置契约差异，并在显式绑定左右 v4 backtest 时附带 `strategy_config_evidence_diff`

error_codes.rs                  — 全局错误码注册表
  └── QSxxxx, QPQSxxxx 诊断码定义

formal_quantscript_authoring_types.rs — QS 正式编写类型定义
```

[GP §1.1]: QS 是唯一策略定义路径 — 前端 graph 编辑器不能产出独立编译路径
[GP §1.3]: 编译路径不可绕过 — 所有 compile_runtime_protocol_config 必须来自 compile_runtime_protocol_via_qs
[GP §5.4]: 禁止在图编辑器中绕过 QS 编译

### 2.2 图存储系统

**职责**: 策略图的 CRUD、版本管理、持久化。

```
src/backend/graph_compile/graph.rs — 图 CRUD API
  ├── POST   /api/graph/save           — 保存策略图
  ├── GET    /api/graph/load/:id       — 加载策略图
  ├── GET    /api/graph/list           — 列出全部策略
  ├── DELETE /api/graph/:id            — 删除策略图
  └── GET    /api/graph/versions/:id   — 版本历史
  └── bundle commit: staging 文件齐全校验 + 多文件失败回滚 🆕 v4.8.0

[存储]: graphs/ 目录 (Permanent 级)
  └── 原子写入: write(.tmp) → fsync → rename → fsync(parent)
```

[GP §1.4]: 数据流单向 — QS 源码 → graph JSON → 前端可视化, 保存时不覆盖原始 QS
[GP §7.1]: 图存储属于 Permanent 级, 不可被启动清理删除

### 2.3 运行时系统

**职责**: Paper 运行、策略调度、事件流、诊断。

```
runtime/mod.rs                  — 运行时主模块
  ├── Paper 运行: 启动/停止/暂停/恢复
  ├── 事件流: SSE 推送 (EventStream)
  └── 策略生命周期管理

runtime/run.rs                  — 单次运行执行
  └── 沙盒内执行策略, 产出事件序列

runtime/mutation.rs             — 运行时变更 (审批 + AI 提案)
  ├── AI 提案: 策略参数/结构自动优化建议
  ├── 审批工作流: L1/L2/L3 三级
  ├── 沙箱验证: run_sandbox_verification()
  └── 锁顺序: approval_records → ai_proposals (防死锁)

runtime/backtest.rs             — 回测引擎
  ├── 历史回放/确定性 Mock
  ├── 12 项指标: 夏普/索提诺/卡尔玛/最大回撤/胜率/盈亏比/...
  └── 回测工件: 事件序列 + 权益曲线 + 成交记录

runtime_persistence.rs          — 运行时持久化
  └── 运行记录/回测工件/实验数据的读写, 大文件读取 100MB 上限 🆕 v4.8.0

runtime_diagnostics.rs          — 运行时诊断
  └── 运行时健康检查、性能诊断

runtime_event_projection.rs     — 运行时事件投影 (v4)
  └── 将 v4 runtime 事件映射为前端可消费的格式

runtime_response_mapping.rs     — 运行时响应映射
  └── 后端内部类型 → 前端 API 响应类型

runtime_validation.rs           — 运行时验证
  └── graph_id, compile_id, 参数合法性校验

sandbox_verification.rs         — 沙箱验证兼容桥
src/backend/ops_governance/sandbox/handlers.rs — 沙箱验证实现
src/backend/ops_governance/sandbox/comparison_metrics.rs — 沙箱 backtest comparison metrics 子叶
src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs — 沙箱 backtest projection 子叶
src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs — 沙箱 v4 replay-shape comparison 子叶
src/backend/ops_governance/sandbox/metrics_evaluation.rs — 沙箱 metric diff/verdict/warnings 评价子叶
src/backend/ops_governance/sandbox/report_api.rs — 沙箱验证 report API
src/backend/ops_governance/sandbox/verification_run.rs — 沙箱验证 runner
src/backend/ops_governance/sandbox/verification_run/proposal_gate.rs — 沙箱验证 proposal eligibility gate 子叶
src/backend/ops_governance/sandbox/verification_run/replay_window.rs — 沙箱验证 replay window shape 子叶
src/backend/ops_governance/sandbox/verification_run/report_assembly.rs — 沙箱验证 report DTO assembly 子叶
src/backend/ops_governance/sandbox/verification_run/report_commit.rs — 沙箱验证 report 持久化提交子叶
  └── AI 提案独立回放验证, catch_unwind + 3 重试

frontend_runtime_mapping.rs     — 前端运行时映射
frontend_api_types.rs           — 前端 API 类型定义
```

[GP §9.1]: 沙箱验证 — AI 提案必须通过独立沙箱回放, CandidateUnderperforms 阻断
[GP §9.2]: 签名快照 — SHA-256 5 项指纹, 恢复前验签, 原子写入
[GP §9.3]: 告警引擎 — 10 条规则全部有 resolve_condition, 双重去重
[GP §9.4]: 审批工作流 — 过期自动 Expired, 状态联动, 锁顺序反序死锁

### 2.4 回测系统

**职责**: 回测执行、结果分析、回测对比。

```
backtest_artifacts.rs           — 回测工件管理
  └── 工件创建、读取、列表

backtest_compare.rs             — 回测对比入口
  └── 多回测并行对比 API

backtest_compare_core.rs        — 回测对比核心逻辑
  └── 指标对比、权益曲线叠加、相关性分析

backtest_compare_narrative.rs   — 回测对比叙述生成
  └── 将对比数据转化为人类可读的中文分析文本

backtest_compare_types.rs       — 回测对比类型定义
```

**回测 12 项指标** (来自 `qrpc_runtime/src/backtest_metrics.rs`):
夏普比率、索提诺比率、卡尔玛比率、最大回撤、胜率、盈亏比、总收益率、年化收益率、波动率、下行波动率、平均持仓时间、成交数

### 2.5 安全系统

**职责**: 本地会话认证、凭证加密存储、速率限制、中间件；不代表完整账户系统。

```
auth/mod.rs                     — 本地会话认证
  ├── POST /api/auth/register  — 注册 (bcrypt 12 轮)
  ├── POST /api/auth/login     — 登录 (JWT HS256, 24h 过期)
  ├── POST /api/auth/refresh   — 刷新令牌轮换 + 重放检测
  ├── bcrypt verify → tokio::spawn_blocking + 30s timeout (不阻塞工作线程) 🆕 v4.8.0
  └── 注册限流: 6 次/分钟/IP

auth_middleware.rs              — 认证中间件
  └── JWT 验证, 注入用户上下文

credential_vault.rs             — 凭证保险库
  ├── AES-256-GCM 加密 (ring crate)
  ├── 原子写入: write(.tmp) → fsync → rename → fsync(parent) → .bak 回滚
  ├── 内存清零: Zeroizing (密钥/凭证 Drop 时)
  └── PBKDF2 ≥600,000 轮 (测试端)

credential_api.rs               — 凭证管理 API
  ├── POST /api/credentials/set
  ├── GET  /api/credentials/list
  └── DELETE /api/credentials/:service

rate_limiter.rs                 — 速率限制
  └── 全局限速: 默认 100 请求/秒 (QUANTPILOT_RATE_LIMIT_RPS)

middleware.rs                   — 通用中间件
safe_log.rs                     — 安全日志
  └── 日志输出前清除 secret/key/sign/passphrase 字段
```

[GP §2.6]: 凭证保险库安全 — AES-256-GCM 禁止降级, Zeroizing, 原子写入
[GP §2.7]: 实时执行安全 — HMAC-SHA256 签名, 每日 ≤100 单/≤$1000, 错误清洗
[GP §2.8]: 本地会话认证安全 — bcrypt ≥12 轮, JWT HS256, 刷新令牌轮换, 重放检测 410 GONE; 账户系统扩展为 unsupported

### 2.6 运维系统

**职责**: 告警、快照、审批、混沌实验、运行手册。

```
alert_engine.rs                 — 告警引擎
  ├── 10 条默认告警规则 (不可删除, 仅可追加)
  ├── resolve_condition 自动恢复
  ├── 双重去重: INSERT OR IGNORE + 内存 HashSet<AlertFingerprint>
  └── 三阶段无锁恢复: 内存更新 → 写锁释放 → 磁盘 I/O

snapshot_service.rs             — 快照服务
  ├── SHA-256 签名: (capability_hash + strategy_version + parameter_version
  │                   + core_ir_digest + event_slice_bounds + created_at_ms)
  ├── 恢复前验证签名完整性
  └── 原子写入 + fsync

chaos_experiment.rs             — 混沌实验
  └── 实验定义、执行、报告

runbook.rs                      — 运行手册
  └── 运维操作手册定义与执行

collaboration.rs                — 协作
  └── 多人协作相关功能

backup.rs                       — 备份
  └── 数据备份与恢复
```

### 2.7 能力与诊断系统

**职责**: capability 声明、API 错误格式、测试场景、CLI 支持。

```
src/backend/capability/snapshot.rs — 能力声明 API
  ├── GET /api/capabilities     — 后端能力真源
  └── CapabilityResponse: workspace.surfaces + ui_actions.actions
      + runtime.modes + market_data + strategy_ir + permission_boundary

api_errors.rs                   — API 错误格式
  ├── json_bad_request()        — 400 错误 (用户侧)
  ├── internal_error()          — 500 错误 (服务器侧)
  └── 统一 JSON 格式: {"error":"...","message":"...","details":[...]}

api_test_scenario.rs            — 测试场景 API
  └── 自动化测试场景定义与执行

test_runner.rs                  — 测试运行器
  └── 测试用例调度

cli_support.rs                  — CLI 支持
  └── 命令行工具辅助函数

migration_sender.rs             — 数据迁移
  └── 跨版本数据迁移逻辑

src/backend/ops_governance/hotswap/handlers.rs — 模块热替换
  └── 运行时模块替换 API
```

[GP §1.12]: 前端能力入口必须以后端 `/api/capabilities` 为唯一真源

---

## 根3: 执行端 (:3001)

**一句话**: 独立进程, 负责策略部署/启停/热调参, 对接 OKX 行情、实时模拟盘本地成交与 OKX 模拟盘 provider 回执。

**编译与启动**:
```
Cargo.toml [[bin]] name="executor", path="src-executor/main.rs"
cargo run --bin executor    →    监听 127.0.0.1:3001
```

### 3.1 入口与状态管理

```
src-executor/main.rs             — 执行端入口 (tokio::main)
  ├── 初始化 qrpc_session (进程间加密通道)
  ├── 构建 Axum Router → 绑定 :3001
  ├── SSE 端点 /api/executor/events — 向后端推送策略状态变更
  └── REST 端点: deploy / start / stop / status / params / okx-demo submit-query-cancel

src-executor/executor_state.rs   — 执行端核心状态
  ├── ExecutorState: 全局执行端状态
  ├── ExecutionMode: PaperSimulated / PaperActual
  ├── StrategyStatus: 策略生命周期
  └── TriggerEvent: 触发事件定义
```

### 3.2 实时运行

```
src-executor/live_runner.rs      — 实时运行器 (RunnerPool v3/v4 双 runner); 改策略执行逻辑时改这里
  ├── 策略启动后激活 Runner
  ├── 行情事件 → 策略求值 → 模拟成交
  └── RunnerPool 管理多策略并发

src-executor/ws_client.rs        — OKX WebSocket 客户端; 改 WS 连接/订阅时改这里
  ├── 每交易所独立 WS 连接
  ├── 行情订阅: ticker / kline / orderbook
  └── 自动重连

src-executor/okx_rest.rs         — OKX REST API; 改下单/撤单逻辑时改这里
  ├── OKX 模拟盘 submit / query / cancel
  ├── demo flag=1 + x-simulated-trading=1
  └── HMAC-SHA256 签名与 provider 回执解析

src-executor/kline_buffer.rs     — K线缓冲池; 改 K 线缓存策略时改这里
  ├── KlinePool: MAX_SYMBOLS=100, LRU 淘汰
  └── KLINE_POOL_CAPACITY 命名常量
```

### 3.3 安全与审计

```
src-executor/credential_vault_v2.rs — 凭证保险库 v2 (执行端专用)
  ├── PBKDF2 ≥1,000,000 轮 (比测试端更严格)
  ├── 独立随机机器密钥 (.executor-machine-key)
  ├── Zeroizing<String> (CredentialEntry)
  └── 原子写入 + .bak 回滚

src-executor/api_guard.rs        — API 守卫
  └── 执行端 API 认证/授权

src-executor/audit_log.rs        — 审计日志
  └── 操作审计记录

src-executor/migration_api.rs    — 迁移 API
  └── 执行端数据迁移接口
```

[GP §2.6]: 凭证保险库 — 执行端 PBKDF2 ≥1M 轮, Zeroizing, 独立机器密钥
[GP §2.7]: 实时执行安全 — OKX HMAC-SHA256, 错误清洗, 速率限制

### 3.4 执行端前端 (frontend-executor/)

**文件**: `frontend-executor/` — 执行端的独立前端 SPA, Vite + React, 端口 5174。

```
frontend-executor/
  ├── package.json            — 项目配置 (quantpilot-executor)
  ├── vite.config.js          — Vite 构建配置
  ├── index.html              — HTML 入口 (<title>QuantPilot 实时执行端 v4.0.0</title>)
  └── src/
      ├── main.jsx            — React 入口, 挂载 ExecutorApp
      ├── ExecutorApp.jsx     — 执行端 App Shell: 多策略标签页 + 状态轮询 + 模式切换
      ├── i18n.js             — 执行端轻量 i18n provider, zh-CN/en-US
      ├── design-system.css   — 执行端专用设计系统 (暗色/亮色面板) 🆕 v4.10.0
      └── components/
          ├── ExecutorTopBar.jsx      — 顶部工具栏: 模式切换 (Paper/Live) + 策略选择
          ├── StrategyGraphPanel.jsx  — 策略图面板: React Flow 只读预览
          ├── KlineChart.jsx          — K线图表: lightweight-charts 实时行情 + bars 预设
          ├── OrderPanel.jsx          — 订单面板: 挂单/成交/历史显示
          ├── AssetPanel.jsx          — 资产面板: 持仓/余额/权益曲线
          └── StrategyParamsPanel.jsx — 策略参数面板: 热调参 pending→commit/rollback
```

**与后端的关系**: 执行端前端通过 `/api/executor/*` 与执行端 (:3001) 通信, 不经过主后端 (:3000)。

---

## 根4: Cargo 工作区

**一句话**: 7 个 crate 构成 Rust 类型系统、编译管道、运行时和工具链。

```
Cargo.toml [workspace]
  ├── qrpc_core_ir     — 类型定义层 (含 v4.1.0 全量类型, v4.rs)
  ├── qrpc_core         — 核心抽象
  ├── qrpc_compiler     — 编译层
  ├── qrpc_runtime      — 运行时层 (含 v4_runtime + compat)
  ├── qrpc_session      — 进程间加密
  ├── quantscript       — QS 语言 (含 v4_static_audit)
  └── src-tauri         — Tauri 桌面壳
```

### 4.1 qrpc_core_ir — 类型定义层

**文件**: `qrpc_core_ir/src/lib.rs` + `qrpc_core_ir/src/v4.rs`

整个项目最底层的类型基石。定义了所有跨 crate 共享的数据结构。

**v3 类型** (lib.rs):
- `CoreStrategyIr` — 策略中间表示
- `RuntimeProtocolCoreConfig` — 运行时协议核心配置
- 20 种 RFC 协议类型 (001-020)

**v4 类型** (`v4.rs`):
- 47+ struct/enum, 40+ 验证函数
- 三大 Machine 模板: `ObservationMachine` / `DecisionMachine` / `ExecutionMachine`
- 事件模型: `MachineEventCatalog`, `MachineEventTypeSpec`, 5 事件域
- 事件拒绝: `v4.runtime.event_rejected` 记录 payload/type/guard/memory 写入 fail-closed 证据 🆕 v4.1.0
- 执行端接入: v4 runner 消费 OKX Market 事件并输出 `v4RuntimeMemorySnapshot` SSE 证据 🆕 v4.2.0
- v4 回测: `/api/runtime/backtest` 可按 `runtime_kind=v4` 执行 deterministic bar replay, 生成 machine trajectory / Risk Plane / Execution capability artifact 🆕 v4.3.0
- 嵌套状态机: `MachineState.child_machine` 支持同模板二级 child machine, 静态契约拒绝三级嵌套、重复 id 与模板漂移 🆕 v4.7.0
- Risk Plane: `MachineGraphRiskPlane`, priority ≥9000
- 24 种执行能力: `ExecutionCapabilityKind`, `CapabilitySupportSource`
- 运行时模式: 用户侧公开命名统一为 `PaperActual` / `PaperSimulated`；历史内部 enum 需经兼容层归一化，不直出给用户
- QS 类型系统: 23 标量类型 + 5 复合类型
- 静态契约束: `V4StaticContractBundle` 聚合 10 子契约
- 编译期能力报告: `V4CompileTimeCapabilityReport`
- 兼容桥: `bridge_core_ir_to_v4_machine_graph()`
- 插件治理: `PluginGovernanceContract`, `PluginManifestSpec`
- 复现契约: `ReproducibilityContract` (8 项证据)
- 复杂度预算: `ComplexityBudgetContract` (10 项上限, 含嵌套深度与事件处理路径)
- 学习流水线: `DeveloperLearningPipelineContract`
- 版本 Manifest: `V4VersionManifest`

### 4.2 qrpc_core — 核心抽象

**文件**: `qrpc_core/src/`

运行时协议核心抽象层:
- `qrpc_core/src/lib.rs` — crate 入口, 核心 trait 定义
- `qrpc_core/src/error.rs` — 核心错误类型; 改跨 crate 错误定义时改这里
- `qrpc_core/src/plugin.rs` — 插件 trait 定义; 改插件接口时改这里
- `qrpc_core/src/strategy_ir.rs` — 策略 IR 核心类型; 改策略中间表示时改这里

### 4.3 qrpc_compiler — 编译层

**文件**: `qrpc_compiler/src/lib.rs`

将 Core IR 编译为可执行指令。负责:
- 策略图 → Core IR 转换
- 优化 pass
- 代码生成

### 4.4 qrpc_runtime — 运行时层

**文件**: `qrpc_runtime/src/`

```
qrpc_runtime/src/
  ├── lib.rs                  — 运行时入口
  ├── v4_runtime.rs           — v4.0.0 PaperSimulated 运行时 🆕 v4.0.0
  ├── v4_runtime_types.rs     — v4 runtime 类型/初始化辅助 🆕 v4.8.0
  ├── v4_simulated_execution.rs — v4 模拟撮合/订单/持仓/资产曲线 🆕 v4.8.0
  ├── v4_runtime_tests.rs     — v4 runtime 单元测试模块 🆕 v4.8.0
  │   ├── V4PaperSimulatedRuntime     — 核心结构体; 改 v4 运行时行为时改这里
  │   ├── submit_event()              — 事件提交 → process_event()
  │   ├── advance_time()              — 静默检测
  │   ├── pull_machine()              — 缓存返回/恢复
  │   ├── complete_recovery()         — 恢复完成
  │   ├── update_simulated_market_price() — 模拟行情
  │   ├── evaluate_risk_plane_for_execution() — Risk Plane 门禁
  │   ├── evaluate_execution_capabilities_for_execution() — 能力检查
  │   ├── V4SimulatedExecutionRuntimeState — 模拟撮合引擎
  │   └── memory_snapshot()           — 内存快照, v4.7.0 起包含 active child machine 层级和 complexity_metrics
  ├── compat.rs               — 模块热替换兼容性检查 🆕 v4.0.0
  │   ├── CompatibilityChecker        — 7 维度检查 (identity/version/schema/capability/ABI)
  │   └── validate_module_surface()   — 模块表面验证; 改模块热替换规则时改这里
  ├── core_ir_evaluator.rs    — Core IR 求值器 (18 种指标 evaluator 全实现, 零 stub)
  │                           改指标计算逻辑或新增指标 evaluator 时改这里
  ├── backtest_metrics.rs     — 回测指标计算 (12 项); 改回测指标公式时改这里
  ├── sandbox/                — 沙盒模块
  │   ├── mod.rs              — 沙盒调度器; 改沙盒执行流程时改这里
  │   ├── replay.rs           — 确定性回放; 改回放机制时改这里
  │   └── timeline.rs         — 时间线管理; 改事件时序时改这里
  ├── data_module.rs          — 数据模块; 改市场数据接入时改这里
  ├── intent_module.rs        — 意图模块; 改意图生成逻辑时改这里
  ├── agent_module.rs         — 代理模块; 改代理决策时改这里
  ├── execution_module.rs     — 执行模块; 改执行路径时改这里
  ├── fill_engine.rs          — 模拟成交引擎; 改成交逻辑时改这里
  ├── live_execution.rs       — 实时执行 (OKX); 改 OKX 对接时改这里
  ├── circuit_breaker.rs      — 熔断器; 改异常保护逻辑时改这里
  ├── config_tracker.rs       — 配置追踪; 改配置变更检测时改这里
  ├── hotswap.rs              — 模块热替换; 改运行时模块替换时改这里
  ├── merge.rs                — 合并逻辑; 改多策略合并时改这里
  ├── merge_coordinator.rs    — 合并协调; 改合并调度时改这里
  ├── reconcile.rs            — 对账; 改运行结果对账时改这里
  ├── runtime_state.rs        — 运行时状态; 改状态管理时改这里
  ├── risk_checker.rs         — 风控检查; 改风控规则时改这里
  ├── risk_monitor.rs         — 风控监控; 改风控监控逻辑时改这里
  ├── slippage.rs             — 滑点计算; 改滑点模型时改这里
  ├── plugin_market.rs        — 插件市场 (Ed25519 签名验证); 改插件验证时改这里
  ├── plugin_runtime_registry.rs — 插件注册表和安全策略检查; 改插件加载时改这里
  └── plugin_sandbox.rs       — 插件沙盒, execute_checked 接入安全门禁; 改插件隔离时改这里
```

### 4.5 qrpc_session — 进程间加密

**文件**: `qrpc_session/src/lib.rs`

后端 ↔ 执行端之间的加密通道:
- AES-256-GCM 加密
- HMAC-SHA256 完整性验证
- 临时密钥交换

### 4.6 quantscript — QS 语言编译器

**文件**: `quantscript/src/`

```
quantscript/src/
  ├── lib.rs                  — crate 入口
  ├── script.rs               — QS 词法/语法解析; 改 QS 语法时改这里
  ├── hir.rs                  — 高级中间表示 (HIR); 改 QS 语义模型时改这里
  ├── analysis.rs             — 语义分析; 改类型检查/符号解析时改这里
  ├── resolve.rs              — 符号解析, 新语法可解析, 未知函数拒绝; 改 QS 函数注册时改这里
  ├── types.rs                — QS 类型系统; 改 QS 类型定义时改这里
  ├── evaluator.rs            — 表达式求值; 改 QS 表达式语义时改这里
  ├── diagnostics.rs          — QS 诊断码 (QSxxxx); 新增错误/警告时改这里
  ├── test_plan.rs            — 测试计划生成; 改测试场景生成逻辑时改这里
  ├── v4_static_audit.rs      — v4 状态机静态审计 🆕 v4.0.0
  │   ├── audit_v4_quant_script_static()    — 审计入口: parse → analyze → report
  │   ├── parse_v4_static_document()        — 解析 v4 QS 脚本 (v4_strategy/machine/state/transition)
  │   ├── derive_event_catalog()            — 从 transition 和 edge 自动推导事件目录
  │   ├── build_compile_time_capability_report() — 编译期能力报告
  │   ├── build_v4_qs_runtime_handoff()     — 运行时交接: 验证审计通过 + 模式=PaperSimulated
  │   └── 30 个诊断码: QSV4000-QSV4300
  └── lowering/               — Lowering 降级: HIR → Core IR
      ├── mod.rs              — lowering 模块入口
      ├── orchestrator.rs     — 降级编排器: 协调全部 lowering pass; 改降级流程时改这里
      ├── bindings.rs         — 绑定降级: HIR binding → Core IR data_binding
      ├── binding_sources.rs  — 绑定来源处理
      ├── context.rs          — 降级上下文: 符号表/类型环境
      ├── diagnostics.rs      — lowering 诊断: 降级过程中的错误和警告
      ├── fallback.rs         — 降级回退: 无法降级时的 fallback 策略
      ├── helper_env.rs       — 辅助环境: lowering 辅助函数
      ├── intents.rs          — 意图降级: HIR intent → Core IR intent
      ├── semantic.rs         — 语义降级: 语义分析结果 → Core IR
      ├── shared.rs           — 共享工具: lowering 阶段共用函数
      ├── source_recovery.rs  — 源码恢复: 从 Core IR 反推 QS 源码位置
      └── universe.rs         — 交易对展开: 多交易对策略展开
```

**QS 编译管道**: `QS 源码 → parse → HIR → semantic analysis → type check → lowering → Core IR`

[GP §1.2]: 新功能跨三层验证 — QS parse, Core IR 枚举, runtime evaluator

### 4.7 src-tauri — Tauri 桌面壳

**文件**: `src-tauri/`

- `src/main.rs` — Tauri 壳入口, 等待后端 :3000 就绪
- `src-tauri/tauri.conf.json` — Tauri 配置 (窗口/CSP/打包)
- `Cargo.toml` — 依赖 `tauri` v2
- `icons/` — 应用图标
- 开发/构建脚本: `dev.bat`, `build.bat`

---

## 根5: 前端 React SPA

**一句话**: React 18 + Vite 6 + Zustand 4 + React Flow 12, Adobe 暗色面板设计系统, 12 个用户路由 + 404, 160+ 文件。

### 5.1 路由与页面 (12 路由 + 404)

**路由定义**: `frontend/src/router.js`
**路由挂载**: `frontend/src/App.jsx`

```
/                                           → StrategyHubPage (策略中心)
/strategies/:strategyId                      → StrategyWorkspacePage (策略工作区)
/strategies/:strategyId/backtests            → StrategyBacktestsPage (回测列表)
/backtests/:backtestId?strategy=:id          → BacktestDetailPage (回测详情)
/backtests/compare?ids=...&strategy=:id      → BacktestComparePage (回测对比)
/approvals                                   → ApprovalPanel (审批面板)
/alerts                                      → AlertsPage (告警页面)
/snapshots                                   → SnapshotsPage (快照页面)
/runbook                                     → RunbookPage (运行手册)
/chaos                                       → ChaosPage (混沌工程)
/settings                                    → SettingsPage (设置)
/quantscript                                 → QuantScriptEditor (QS 编辑器)
unknown                                      → NotFoundPage (404)
```

**全局 UI**:
- `frontend/src/components/LeftSidebar.jsx` — 左侧导航栏
- `frontend/src/components/TopToolbar.jsx` — 顶部工具栏 (含 capability 同步状态、v4 模拟运行入口、策略包导入/导出) 🆕 v4.9.0
- `frontend/src/components/CommandPalette.jsx` — ⌘K 命令面板 (页面跳转 + 保存/编译/运行/回测命令)
- `frontend/src/components/TutorialOverlay.jsx` — 教程覆盖层
- `frontend/src/components/ToastContainer.jsx` — Toast 通知容器
- `frontend/src/components/ErrorBoundary.jsx` — 每个路由独立的错误边界与结构化回退 UI

### 5.2 策略工作区 (StrategyWorkspacePage)

**文件**: `StrategyWorkspacePage.jsx` — 策略工作区主容器

工作区有 10 个表面 (workspace surfaces), 由后端 `/api/capabilities` 能力声明驱动:

```
工作区表面 (Workspace Surfaces):
  ├── dashboard           — 仪表盘: 编译状态/运行状态/最近回测/快速操作
  ├── code                — 代码视图: QS 编辑器 + 编译面板
  ├── research            — 研究控制台: 回测/运行/事件流
  ├── monitor             — 监控面板: 运行时状态/诊断 (v4)
  ├── source              — 源码视图: 图 JSON 原始数据
  ├── template_library    — 模板库: 策略模板浏览/加载/应用
  ├── version_history     — 版本历史: 图版本对比/回滚/配置契约 diff
  ├── collaboration_audit — 协作审计
  └── parameter_sweep     — 参数扫描 (v4)
```

[GP §8.11]: 工作区表面入口必须由后端 capability projection 驱动
[GP §8.10]: 工作区职责分区稳定 — 导航/控制/主对象/inspector/时间线

### 5.3 策略图编辑器

**文件**: `StrategyCanvas.jsx` — React Flow 画布

```
图编辑器:
  ├── 6 类节点: data / intent / agent / risk / execution / fill
  ├── 节点拖拽、连线、参数配置
  ├── 小地图: StrategyCanvasMiniMap.jsx
  ├── 视口管理: strategyCanvasViewport.js
  ├── 焦点管理: strategyCanvasFocus.js
  └── 模块侧栏: ModuleSidebar.jsx
```

**节点配置面板** (`PropertyPanel.jsx`):
- 属性编辑、编译摘要、策略 IR 检查
- 属性选择器: `propertyPanelSelectors.js`
- 属性动作: `usePropertyPanelActions.js`

**18 种内置指标模块** (`builtinModules.js`):
MA Cross, RSI, MACD, Momentum, Spread, ZScore, Custom, QuoteObserve, ATR, Bollinger Bands, OBV, CMF, ADX, Stochastic, CCI, Parabolic SAR, Keltner Channel, Donchian Channel

### 5.4 回测系统 (前端)

```
BacktestDetailPage.jsx       — 回测详情: 权益曲线/成交记录/指标面板
BacktestComparePage.jsx      — 回测对比: 多回测叠加/相关性
BacktestAnalysisLayout.jsx   — 回测分析布局
backtestAnalysisShared.jsx   — 回测分析共享逻辑
StrategyBacktestsPage.jsx    — 策略回测列表
```

**可视化组件**:
- `frontend/src/components/DrawdownChart.jsx` — 回撤曲线
- `frontend/src/components/MonthlyReturnsHeatmap.jsx` — 月收益热力图
- `frontend/src/components/AssetCandlesPanel.jsx` — K 线面板

### 5.5 事件流与运行时展示

```
EventStreamPanel.jsx         — 事件流面板 (核心)
  ├── 运行时事件实时展示
  ├── 回测历史事件回放
  ├── 事件详情/诊断
  └── 运行时工件操作

EventReplaySection.jsx       — 事件回放区
RunHistorySection.jsx        — 运行历史
BacktestHistorySection.jsx   — 回测历史
EvidenceSummaryCards.jsx     — 证据摘要卡片
GovernedTimelinePanel.jsx    — 治理时间线
V4RuntimeEvidencePanel.jsx   — v4 状态机证据面板, 递归展示嵌套 machine
ComplexityBudgetPanel.jsx    — v4 复杂度预算面板, 展示状态/transition/memory/嵌套深度/事件路径使用率
```

### 5.6 运维面板

```
ApprovalPanel.jsx            — 审批面板 (L1/L2/L3)
RuntimeMutationPanel.jsx     — 运行时变更面板 (AI 提案)
RuntimeDiagnosticsPanel.jsx  — 运行时诊断面板
RuntimeReportPanel.jsx       — 运行时报告面板
StrategyDiagnosticsPanel.jsx — 策略诊断面板
StrategyParamsPanel.jsx      — 策略参数面板 (热调参)
StrategyRunsPanel.jsx        — 策略运行面板
StrategyBacktestsPanel.jsx   — 策略回测面板
StrategyResearchConsole.jsx  — 策略研究控制台
DeployButton.jsx             — 部署按钮 (策略→执行端)
CredentialInput.jsx          — 凭证输入组件
```

### 5.7 状态管理 (Zustand)

**文件**: `frontend/src/store/graphStore.js` — 主 store

```
graphStore 模块体系 (40+ 模块):
  ├── graphStore.js                     — 主 store (Zustand)
  ├── graphStoreCompileState.js         — 编译状态
  ├── graphStoreCompileActions.js       — 编译动作
  ├── graphStoreCompileApi.js           — 编译 API 调用
  ├── graphStoreCompileFlow.js          — 编译流程
  ├── graphStoreCompileHelpers.js       — 编译辅助
  ├── graphStoreCompileOutcomeMapping.js — 编译结果映射
  ├── graphStoreCompileOutcomeProjection.js — 编译结果投影
  ├── graphStoreCompileProtocolFlow.js  — 协议编译流程
  ├── graphStoreCompileProtocolMapping.js — 协议编译映射
  ├── graphStoreEditorActions.js        — 编辑器动作
  ├── graphStoreHelpers.js              — 通用辅助
  ├── graphStorePersistenceActions.js   — 持久化动作 + 策略包导入
  ├── graphStorePersistenceHelpers.js   — 持久化辅助
  ├── graphStoreRuntimeActions.js       — 运行时动作
  ├── graphStoreRuntimeHelpers.js       — 运行时辅助
  ├── graphStoreRuntimeTransport.js     — 运行时传输 (SSE)
  ├── graphStoreRuntimeSessionState.js  — 运行时会话状态
  ├── graphStoreRuntimeSelectionState.js — 运行时选择状态
  ├── graphStoreRuntimeHistoryActions.js — 运行时历史动作
  ├── graphStoreRuntimeHistoryApi.js    — 运行时历史 API
  ├── graphStoreRuntimeHistoryFlow.js   — 运行时历史流程
  ├── graphStoreRuntimeHistoryProjection.js — 运行时历史投影
  └── graphStoreRuntimeHistoryState.js  — 运行时历史状态
```

**测试覆盖**: graphStore 有 20+ 测试文件, 覆盖编译/编辑器动作/运行时/持久化/模板/版本历史/回退

### 5.8 能力投影层

**文件**: `frontend/src/capabilities/`

```
capabilityProjection.js      — 能力投影核心
  ├── projectWorkspaceSurfaces()  — 工作区表面投影
  ├── projectUiActions()          — UI 动作投影 (含 capability 同步阻塞)
  ├── projectCapabilityView()     — 顶层组合
  └── projectEntry()              — 投影原子: {visible, enabled, status, reason, source}

supportMatrix.js              — 支持矩阵
  ├── WORKSPACE_SURFACE_MAP  — 10 个工作区表面定义
  ├── CAPABILITY_ACTION_MAP  — 15 个能力动作定义
  ├── EXPECTED_PERMISSION_BOUNDARY — 6 项权限约束
  ├── normalizeUiActionStatus()    — 能力状态标准化
  ├── getCapabilityActionBlockReason() — 阻塞原因判断
  └── isCapabilitySyncBlocked()    — 同步阻塞检测

capabilityGovernance.js       — 能力治理
  ├── 4 个 CAPABILITY_CLASSES: supported / restricted / trace_only / disallowed_claim
  ├── 66 supported / 6 restricted / 1 trace_only / 4 disallowed_claim
  └── positiveClaimAudit — 用户声明文本门控
```

[GP §1.12]: 前端 static list 只允许骨架, 能力边界真源只能是后端 CapabilityResponse

### 5.9 Hooks 层 (24 个自定义 Hook)

```
useStrategyWorkspaceSharedModel.js   — 工作区共享数据模型
useStrategyWorkspaceUiState.js       — 工作区 UI 状态
useStrategyWorkspacePageData.js      — 工作区页面数据
useStrategyHubBodyData.js            — 策略中心主体数据
useStrategyHubInspectorData.js       — 策略中心检查器数据
useStrategyHubRosterData.js          — 策略中心列表数据
useStrategyDirectoryModel.js         — 策略目录模型
useStrategyResearchModel.js          — 研究模型
useStrategyResearchActions.js        — 研究动作
useStrategyResearchUiState.js        — 研究 UI 状态
usePropertyPanelModel.js             — 属性面板模型
usePropertyPanelActions.js           — 属性面板动作
useWorkspaceActionBarActions.js      — 工作区动作栏动作
useWorkspaceActionBarModel.js        — 工作区动作栏模型
workspaceActionBarShared.js          — 工作区动作栏共享
workspaceActionSelectors.js          — 工作区动作选择器
propertyPanelSelectors.js            — 属性面板选择器
propertyPanelShared.js               — 属性面板共享
strategyResearchSelectors.js         — 研究选择器
useNotification.js                   — 通知
useOrderAnimation.js                 — 订单动画
usePanelResize.js                    — 面板缩放
useTutorial.js                       — 教程; 首次访问使用 `qp.tutorial.seen`, 可由策略中心入口重开 🆕 v4.10.0
```

### 5.10 工具函数层 (utils/)

```
api.js                              — API 调用封装
compileContract.js                  — 编译契约
errorMessages.js / errorText.js    — 错误消息/文本
actionFailure.js                    — 操作失败处理
configureFieldPriority.js           — 字段优先级配置
repairPathInsights.js               — 路径修复洞察
runtimeAiProposal.js                — AI 提案
runtimeApproval.js                  — 审批
runtimeDiagnosticsProjection.js     — 诊断投影
runtimeEvidenceSummary.js           — 证据摘要
runtimeExplanation.js               — 运行时解释
runtimeGovernance.js                — 运行时治理
runtimeMutation.js                  — 运行时变更
runtimeStatus.js                    — 运行时状态
runtimeTimeline.js                  — 运行时时间线
v4RuntimeEvidence.js                — v4 运行时证据投影, 递归 normalize child machine 并输出 complexity_metrics
strategyHubCompareQueueActions.js   — 策略中心对比队列
strategyHubFormatters.js            — 策略中心格式化
strategyHubInspectorActions.js      — 策略中心检查器动作
strategyHubInspectorProjection.js   — 策略中心检查器投影
strategyHubRecentBacktestsActions.js — 策略中心最近回测
strategyHubRecentRunsView.js        — 策略中心最近运行
strategyHubRosterProjection.js      — 策略中心列表投影
strategyHubRosterRowActions.js      — 策略中心行动作
strategyHubStrategyIdentity.js      — 策略中心策略身份
strategyWorkspaceIssueQueue.js      — 工作区问题队列
workspaceContextLabels.js           — 工作区上下文标签
```

### 5.11 国际化 (i18n)

```
frontend/src/i18n/
  ├── index.js              — i18n 入口 (useI18n hook)
  ├── locales/zh-CN.js      — 中文翻译
  └── locales/en-US.js      — 英文翻译
```

[GP §2.5]: 前端字符串使用 `t()` 包裹
[GP §2.1]: 错误消息必须是中文

### 5.12 设计系统

```
frontend/src/
  ├── styles.css            — 全局样式 + :root CSS 变量
  ├── styles-responsive-panels.css — 响应式面板、减少动效和教程覆盖层样式 🆕 v4.10.0
  ├── shared.css            — 共享样式
  └── design-system.css     — Adobe 暗色面板设计系统
      └── --ad-* CSS 令牌 (~50 个变量): 颜色/间距/圆角/字号/阴影
```

[GP §8.1-§8.8]: 前端设计规范 — Adobe 暗色面板, 响应式, 空状态引导

---

## 根6: 工具链与质量门禁

**一句话**: 三层流水线 (pre-commit → PR/CI → closeout-release), 10+ 门禁脚本, 26 项 closeout gates。

### 6.1 Pre-commit Hook

**文件**: `scripts/pre-commit`

`git commit` 时自动执行 staged-file 智能分流:
```
powershell tools/run-smart-pre-commit.ps1
```

docs-only 默认只跑 diff / UTF-8 / full-feature-tree / matrix governance；rust-only 默认跑 cargo fmt 与 cargo check；frontend-only 默认跑 build 与 vitest；tooling 改动额外检查 hook sync。任何一步失败 → 提交被拒。

### 6.2 closeout 基础门禁 (22 项)

| # | 检查项 | 命令/工具 |
|---|--------|---------|
| 1 | UTF-8 编码 | `tools/check-utf8.ps1` |
| 2 | 面向用户文本 | `tools/check-user-facing-text.ps1` |
| 3 | 能力治理 | `tools/check-capability-governance.ps1` |
| 4 | i18n 覆盖 | `tools/check-i18n.ps1` |
| 5 | 版本一致性 | `tools/check-version-consistency.ps1` |
| 6 | 功能演进 | `tools/check-feature-evolution.ps1` |
| 7 | 三矩阵治理 | `tools/check-matrix-governance.ps1` |
| 8 | 学习流水线 closeout | `tools/check-learning-closeout.ps1` |
| 9 | Pre-commit 同步 | `tools/check-pre-commit-hook.ps1` |
| 10 | 清理边界 | `tools/check-cleanup-boundary.ps1` |
| 11 | Rust 格式 | `cargo fmt --check` |
| 12 | Rust 编译 | `cargo check --workspace` |
| 13 | Rust 测试 | `scripts/test.ps1 test --workspace` |
| 14 | Clippy budget | `tools/check-clippy-warning-budget.ps1 -MaxWarnings 58` |
| 15 | 执行端 warning | `tools/check-executor-warning-budget.ps1 -MaxWarnings 0` |
| 16 | 前端构建 | `npm run build` (frontend) |
| 17 | 前端测试 | `npm run test` (frontend) |
| 18 | E2E | `npm run test:e2e` (frontend) |
| 19 | npm 审计 | `npm audit --audit-level=moderate` |
| 20 | 执行端前端 | `npm run build` (frontend-executor) |
| 21 | 执行端编译 | `cargo check --bin executor` |
| 22 | 执行端测试 | `scripts/test.ps1 test --bin executor` |

### 6.3 Closeout/Release 门禁 (额外 4 项)

在 PR/CI 基础上增加:
```
#23 QS 场景 smoke       → scripts/scenario-smoke.ps1
#24 干净工作区           → tools/check-clean-worktree.ps1
#25 全量树完整性         → tools/check-full-feature-tree.ps1
#26 能力栈一致性         → tools/check-capability-stack.ps1
```

### 6.4 一键收口

```powershell
.\tools\run-closeout-gates.bat      # 26 项全量
```

### 6.5 测试脚本

```
scripts/test.ps1 / test.sh          — 测试运行 (自动停止旧进程)
scripts/scenario-smoke.ps1          — QS 场景测试
```

### 6.6 CI/CD

**文件**: `.github/workflows/ci.yml`

Windows runner, tag 触发 Release, 构建/打包/生成 SHA256SUMS.

### 6.7 其他脚本

```
tools/export-capability-fixture.ps1 — 能力 fixture 导出
```

[超级规范化 §2.1-§2.3]: 三层门禁流水线定义
[超级规范化 §8.1]: 阻断规则 — 任何阻断级门禁未通过禁止进入下一阶段

---

## 根7: 治理文档体系

**一句话**: 原则 → 协议 → 契约 → 指南 → 测试审计 → 里程碑归档, 全部中文。

### 7.0 三矩阵治理层 (markdown/00-matrix-governance/)

三矩阵治理层是 v4.12.0 起新增的开发治理控制面，位于旧 GP、超级规范化和全量树之上。

```
README.md                         — 三矩阵治理总入口
process-matrix.md                 — 流程矩阵: 提案、校验、优化、实现、验证、收口
standard-matrix.md                — 规范矩阵: 硬规则、禁止项、父子通信、新旧冲突、并发锁、AI 幻觉发现
guidance-matrix.md                — 引导矩阵: 全量树 + 模块树定位流程
module-tree.md                    — 模块树: 白箱网络、输入输出、关键 public 方法、父子通信边界
proposal-flow.md                  — 提案状态机、三档执行判定表、提案模板
proposal-examples.md              — 轻量、标准、重型三档提案样例
release-transition-protocol.md    — 发布过渡期连接协议
landing-roadmap.md                — v4.12.0 至 v4.16.0 治理落地与模块化抽离路线
recursive-speed-protocol.md       — v4.16+ 递归高速执行协议
recursive-state.json              — 当前递归状态游标
```

自动化门禁: `tools/check-matrix-governance.ps1` 校验三矩阵入口、提案模板、模块树漂移、里程碑索引、发布过渡协议和递归高速执行协议。
提案样例库: `markdown/00-matrix-governance/proposal-examples.md` 提供轻量、标准、重型三档最小样例。
递归高速协议: `markdown/00-matrix-governance/recursive-speed-protocol.md` 固化智能门禁、两段式、同构批处理、同父级子叶并行、治理生成器和状态游标规则。
递归状态游标: `markdown/00-matrix-governance/recursive-state.json` 记录当前递归 parent、phase、closed children、open residuals 和一次性提示黑名单。

治理接管路线:

| 里程碑 | 焦点 |
| --- | --- |
| v4.12.0 | 三矩阵治理入口启用 |
| v4.13.0 | 模块树白箱扩面 |
| v4.14.0 | 治理门禁自动化 |
| v4.15.0 | 三矩阵完全接管 closeout |
| v4.16.0 | 模块化抽离第一波: 后端抽离、前端抽离、E2E 整理延后、测试资产汰换登记 |

### 7.1 原则层 (markdown/01-principles/)

```
principles-super-standardization.md    — 超级规范化
  ├── 三层门禁流水线
  ├── AI 并行审计 (自由维度诱错)
  ├── 五维度评分 + GP 合规矩阵
  ├── 元流水线 (自进化)
  ├── §7.7 MAJOR 演化通道 (8 Phase)
  ├── §7.8 前端后端能力真源通道 (5 阶段)
  └── §8.9 v4 状态机化演化防偏规则

General_Policy.md (父级)              — GP 项目总规则
  ├── §1.1-§1.12: 12 条架构铁律
  ├── §2.1-§2.8: 8 条代码规范
  ├── §3.1-§3.3: 3 条文档规范
  ├── §4.1-§4.4: 4 条变更管理
  ├── §5.1-§5.6: 6 条禁止事项
  ├── §7.1-§7.5: 5 条存储生命周期
  ├── §8.1-§8.11: 11 条前端设计规范
  ├── §9.1-§9.4: 4 条治理系统约束
  └── §10.1-§10.5: 功能覆盖矩阵 + 回归保护

principles-data-and-intent-layer.md    — 数据与意图层原则
principles-quantpilot-design.md        — QuantPilot 设计原则
```

### 7.2 协议层 (markdown/02-protocol/)

20 份 RFC, 19 份已落地:

```
RFC-001  data-request-protocol            — 数据请求协议
RFC-002  normalized-market-data-protocol   — 标准化市场数据协议
RFC-003  runtime-state-protocol            — 运行时状态协议
RFC-004  agent-protocol                    — 代理协议
RFC-005  intent-protocol                   — 意图协议
RFC-006  intent-generator-protocol         — 意图生成器协议
RFC-007  portfolio-protocol                — 投资组合协议
RFC-008  risk-protocol                     — 风控协议
RFC-009  risk-decision-protocol            — 风控决策协议
RFC-010  allocation-protocol               — 分配协议
RFC-011  execution-plan-protocol           — 执行计划协议
RFC-012  order-protocol                    — 订单协议
RFC-013  execution-feedback-protocol       — 执行反馈协议
RFC-014  runtime-mode-protocol             — 运行时模式协议
RFC-015  runtime-event-protocol            — 运行时事件协议
RFC-016  capability-discovery-protocol     — 能力发现协议
RFC-017  backtest-artifact-protocol        — 回测工件协议
RFC-018  backtest-input-protocol           — 回测输入协议
RFC-019  backtest-output-artifact-protocol — 回测输出工件协议
```

### 7.3 实现契约层 (markdown/03-implementation/)

```
governance/
  ├── implementation-v4-machine-and-venue-contract.md — v4 状态机静态契约
  ├── implementation-developer-learning-pipeline.md   — 学习流水线契约
  ├── implementation-compile-chain-contract.md        — 编译链契约
  ├── implementation-quantscript-retained-surface-contract.md — QS 保留面契约
  ├── implementation-feature-evolution-contract.md    — 功能演进契约
  ├── implementation-capability-governance.md         — 能力治理
  ├── implementation-capability-governance-registry.generated.md — 能力注册表 (自动生成)
  ├── implementation-support-matrix.md                — 支持矩阵
  ├── implementation-artifact-governance.md           — 工件治理
  └── implementation-plugin-storage-standard.md       — 插件存储标准

runtime/
  ├── implementation-persistence-replay-contract.md   — 持久化回放契约
  ├── implementation-runtime-governance-contract.md   — 运行时治理契约
  ├── implementation-runtime-evidence-contract.md     — 运行时证据契约
  ├── implementation-runtime-mutation-contract.md     — 运行时变更契约
  ├── implementation-runtime-ai-approval-contract.md  — AI 审批契约
  ├── implementation-runtime-backtest-explanation-contract.md — 回测解释契约
  ├── implementation-runtime-artifact-retention.md    — 工件保留
  ├── implementation-testing-module.md                — 测试模块
  ├── implementation-test-mode.md                     — 测试模式
  ├── implementation-trading-sandbox.md               — 交易沙盒
  └── implementation-storage-lifecycle.md             — 存储生命周期

quantscript/
  ├── guide-backtest-execution-assumptions-minimal-contract.md
  ├── guide-execution-profile-minimal-contract.md
  ├── guide-risk-profile-minimal-contract.md
  ├── quantscript-checklist.md
  └── quantscript-resolve-lowering-boundary.md

frontend/
  └── README.md
```

### 7.4 指南层 (markdown/04-guides/)

```
guide-formal-quantscript-syntax.md           — QS 正式语法指南
guide-quantscript-trunk-baseline.md          — QS 主干基线
guide-paper-to-strategy-development.md       — 从论文到策略开发
guide-strategy-template-library.md           — 策略模板库使用
guide-user-guide-zh.md                       — 中文用户指南
guide-user-guide-en.md                       — English user guide
```

### 7.5 测试与审计 (markdown/05-testing/)

```
全量审计报告.md
测试报告-latest.md
测试自动化脚本化方案.md
实机场景化测试指南.md
手动全量实机测试检查单.md
自由维度诱错审计-v3.7.1-第1轮.md
自由维度诱错审计-v4.0.0-第1轮.md           — v4 审计报告
自由维度诱错审计-v4.7.0-第1轮.md           — 嵌套状态机 5 维诱错审计
Claude产品UX功能完整度审计核查-v4.7.0-2026-05-26.md — Claude 产品/UX/功能完整度发现的代码侧核查与里程碑分流
meta-pipeline-log.md                         — 元流水线日志
```

### 7.6 里程碑归档 (markdown/06-milestones/)

50+ 版本目录, 从 `v0.2.0` 到 `v4.16.0`, 每个含 `01-规划方案.md` + `02-综合优化清单.md` (或等效文档) + `03-closeout.md` / `02-closeout.md` / `02-落地记录.md` (或等效文档)。

活跃归档:
- `markdown/06-milestones/v4.7.0/02-closeout.md` — v4.7.0 嵌套状态机第一波 closeout 归档
- `markdown/06-milestones/v4.8.0/01-规划方案.md` — v4.8.0 双执行切面 + P2 质量收敛规划
- `markdown/06-milestones/v4.8.0/02-综合优化清单.md` — v4.8.0 W0-W4 优化清单
- `markdown/06-milestones/v4.8.1/01-规划方案.md` — v4.8.1 API 契约与部署治理超级规范化规划
- `markdown/06-milestones/v4.8.1/02-综合优化清单.md` — v4.8.1 P1/P2/P3 优化清单
- `markdown/06-milestones/v4.8.1/03-落地记录.md` — v4.8.1 API 契约落地记录
- `markdown/06-milestones/v4.8.2/01-规划方案.md` — v4.8.2 产品/UX/i18n 收敛规划
- `markdown/06-milestones/v4.8.2/02-综合优化清单.md` — v4.8.2 UX 与 i18n 优化清单
- `markdown/06-milestones/v4.8.2/03-落地记录.md` — v4.8.2 UX/i18n 落地记录
- `markdown/06-milestones/v4.9.0/01-规划方案.md` — v4.9.0 产品功能完整度与插件执行安全规划
- `markdown/06-milestones/v4.9.0/02-综合优化清单.md` — v4.9.0 功能完整度与安全优化清单
- `markdown/06-milestones/v4.9.0/03-落地记录.md` — v4.9.0 功能完整度落地记录
- `markdown/06-milestones/v4.10.0/01-规划方案2.md` — v4.10.0 UX 收口与产品边界固化规划
- `markdown/06-milestones/v4.10.0/02-落地记录.md` — v4.10.0 UX 收口与产品边界固化落地记录
- `markdown/06-milestones/v4.11.0/01-规划方案.md` — v4.11.0 策略配置系统一等化规划
- `markdown/06-milestones/v4.11.0/02-策略配置系统端到端设计.md` — v4.11.0 端到端设计
- `markdown/06-milestones/v4.11.0/03-推进约束与防偏移检查单.md` — v4.11.0 推进约束
- `markdown/06-milestones/v4.12.0/01-规划方案.md` — v4.12.0 三矩阵治理入口启用规划
- `markdown/06-milestones/v4.12.0/02-落地记录.md` — v4.12.0 三矩阵治理入口启用记录
- `markdown/06-milestones/v4.13.0/01-规划方案.md` — v4.13.0 模块树白箱扩面规划
- `markdown/06-milestones/v4.13.0/02-落地记录.md` — v4.13.0 模块树白箱扩面记录
- `markdown/06-milestones/v4.14.0/01-规划方案.md` — v4.14.0 治理门禁自动化规划
- `markdown/06-milestones/v4.14.0/02-落地记录.md` — v4.14.0 治理门禁自动化记录
- `markdown/06-milestones/v4.15.0/01-规划方案.md` — v4.15.0 三矩阵完全接管 closeout 规划
- `markdown/06-milestones/v4.15.0/02-治理closeout.md` — v4.15.0 三矩阵完全接管治理 closeout
- `markdown/06-milestones/v4.16.0/01-规划方案.md` — v4.16.0 模块化抽离第一波规划，覆盖后端抽离、前端抽离、E2E 整理延后和测试资产汰换登记
- `markdown/06-milestones/v4.16.0/02-落地记录.md` — v4.16.0 抽离控制面落地记录
- `markdown/06-milestones/v4.16.0/03-后端抽离登记.md` — v4.16.0 后端抽离候选登记
- `markdown/06-milestones/v4.16.0/04-前端抽离登记.md` — v4.16.0 前端抽离候选登记
- `markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md` — v4.16.0 E2E 延后与测试资产汰换登记
- `markdown/06-milestones/v4.16.0/06-后端接口边界首批抽离方案.md` — v4.16.0 BE-001 后端接口边界首批抽离方案
- `markdown/06-milestones/v4.16.0/07-顶层大模块统计.md` — v4.16.0 顶层大模块统计，确认 6 个逻辑顶层和首批 backend 大模块
- `markdown/06-milestones/v4.16.0/08-system大模块分层统计.md` — v4.16.0 system 大模块分层统计，确认 3 层和 10 个叶子模块
- `markdown/06-milestones/v4.16.0/09-system.entry首批抽离记录.md` — v4.16.0 system 试水抽离记录，确认 `system.entry.backend_process` 第一刀和兼容桥
- `markdown/06-milestones/v4.16.0/10-system抽离完成记录.md` — v4.16.0 system 抽离完成记录，确认 `run_api_server` 与启动期 helper 已迁入 `system.entry.backend_process`
- `markdown/06-milestones/v4.16.0/11-system抽离经验回填.md` — v4.16.0 system 抽离经验回填，固化 public/内部实现分类、owner 复核和未迁移边界准则
- `markdown/06-milestones/v4.16.0/12-system十叶模块等价基线.md` — v4.16.0 system 10 叶模块等价基线，标定 S1-S10 的功能等价证据、继续抽离状态和暂停点
- `markdown/06-milestones/v4.16.0/13-递归模块化全局根流程.md` — v4.16.0 递归模块化全局根流程，定义顶层模块、叶子抽离、叶子整理、细分价值判断和全量模块树收束
- `markdown/06-milestones/v4.16.0/14-system.entry.launch_scripts单叶closeout.md` — v4.16.0 S1 启动脚本单叶 closeout，确认 `start.bat` / `start.ps1` 等价并停止继续细分
- `markdown/06-milestones/v4.16.0/15-system.desktop_shell.tauri_config单叶closeout.md` — v4.16.0 S4 Tauri config 单叶 closeout，确认 CSP、窗口配置和 capability allowlist 等价并停止继续细分
- `markdown/06-milestones/v4.16.0/16-system.runtime_profile.config_examples单叶closeout.md` — v4.16.0 S10 配置样例单叶 closeout，确认环境变量、runtime protocol 和 strategy_ir schema/example 等价并停止继续细分
- `markdown/06-milestones/v4.16.0/17-system.desktop_shell.tauri_runtime-readiness等价检查.md` — v4.16.0 S3 Tauri runtime readiness 等价检查，确认 3000 wait、30 秒超时和 Tauri Builder 启动顺序等价
- `markdown/06-milestones/v4.16.0/18-system.desktop_shell.tauri_runtime单叶closeout.md` — v4.16.0 S3 Tauri runtime 单叶 closeout，确认桌面启动 smoke、主窗口生命周期和关闭路径等价并停止继续细分
- `markdown/06-milestones/v4.16.0/19-system.build_delivery.desktop_build_scripts单叶closeout.md` — v4.16.0 S7 desktop build/dev scripts 单叶 closeout，确认 `build.rs`、`build.bat`、`dev.bat` 等价并停止继续细分
- `markdown/06-milestones/v4.16.0/20-system.entry.backend_process单叶closeout.md` — v4.16.0 S2 backend process 单叶 closeout，正式收束启动进程边界并保持 API owner 外置
- `markdown/06-milestones/v4.16.0/21-system.desktop_shell.assets_schema单叶closeout.md` — v4.16.0 S5 assets/schema 单叶 closeout，确认桌面图标和 Tauri generated schema 等价并停止继续细分
- `markdown/06-milestones/v4.16.0/22-system.build_delivery.container_proxy单叶closeout.md` — v4.16.0 S8 container/proxy 静态单叶 closeout，登记 Dockerfile、compose 和 nginx proxy 边界
- `markdown/06-milestones/v4.16.0/23-system.build_delivery.S6-S9暂停决策记录.md` — v4.16.0 S6 workspace manifest 与 S9 CI/release 暂停决策记录，明确不算 closeout 完成
- `markdown/06-milestones/v4.16.0/24-system顶层阶段性closeout.md` — v4.16.0 `root.system` 顶层阶段性 closeout，收束当前允许范围；S1-S10 已完成 closeout 或静态 closeout
- `markdown/06-milestones/v4.16.0/25-system.build_delivery.S6-S9恢复提案与适配性校验.md` — v4.16.0 S6/S9 恢复提案与适配性校验，确认只做文档级 closeout，不改 manifest/workflow/release 语义
- `markdown/06-milestones/v4.16.0/26-system.build_delivery.workspace_manifest单叶closeout.md` — v4.16.0 S6 workspace manifest 单叶 closeout，确认 Cargo workspace/package manifest 与 lockfile 边界
- `markdown/06-milestones/v4.16.0/27-system.build_delivery.ci_release单叶closeout.md` — v4.16.0 S9 CI/release 单叶 closeout，确认 workflow、packaging 和 release manifest 边界
- `markdown/06-milestones/v4.16.0/28-backend大模块分层统计.md` — v4.16.0 backend 顶层分层统计，确认 `root.backend` 的 3 层网络和 9 个 L2 叶子候选
- `markdown/06-milestones/v4.16.0/29-backend.interface_boundary等价基线.md` — v4.16.0 BE-001A `backend.interface_boundary` 等价基线，锁定 route owner、public 入口和未迁移边界
- `markdown/06-milestones/v4.16.0/30-backend九叶模块壳抽离记录.md` — v4.16.0 BE-001B backend 九叶模块壳抽离记录，建立 `src/backend/` 父模块和 9 个叶子 facade
- `markdown/06-milestones/v4.16.0/31-backend.interface_boundary单叶closeout.md` — v4.16.0 BE-001C-01 `backend.interface_boundary` 单叶 closeout，确认父级 route facade 不继续拆分
- `markdown/06-milestones/v4.16.0/32-backend.capability单叶closeout.md` — v4.16.0 BE-001C-02 `backend.capability` 单叶 closeout，确认 capability 真源边界不继续拆分
- `markdown/06-milestones/v4.16.0/33-backend.strategy_config单叶closeout.md` — v4.16.0 BE-001C-03 `backend.strategy_config` 单叶 closeout，登记 artifact/preflight/diff/AI proposal L3 候选
- `markdown/06-milestones/v4.16.0/34-backend.runtime单叶closeout.md` — v4.16.0 BE-001C-04 `backend.runtime` 单叶 closeout，登记 run/backtest/mutation/evidence/persistence L3 候选
- `markdown/06-milestones/v4.16.0/35-backend.graph_compile单叶closeout.md` — v4.16.0 BE-001C-05 `backend.graph_compile` 单叶 closeout，登记 graph/QS/compile/diagnostics L3 候选
- `markdown/06-milestones/v4.16.0/36-backend.storage_security单叶closeout.md` — v4.16.0 BE-001C-06 `backend.storage_security` 单叶 closeout，登记安全 L3 候选和安全决策暂停点
- `markdown/06-milestones/v4.16.0/37-backend.ops_governance单叶closeout.md` — v4.16.0 BE-001C-07 `backend.ops_governance` 单叶 closeout，登记 sandbox/alerts/snapshots/runbook/chaos/hotswap L3 候选
- `markdown/06-milestones/v4.16.0/38-backend.app_state_wiring单叶closeout.md` — v4.16.0 BE-001C-08 `backend.app_state_wiring` 单叶 closeout，确认 AppState wiring 不继续拆分
- `markdown/06-milestones/v4.16.0/39-backend.test_support单叶closeout.md` — v4.16.0 BE-001C-09 `backend.test_support` 单叶 closeout，确认测试资产汰换前不继续拆分
- `markdown/06-milestones/v4.16.0/40-backend.strategy_config_L3模块壳抽离记录.md` — v4.16.0 BE-001D `backend.strategy_config` L3 模块壳抽离，建立 artifact/preflight/diff/AI proposal binding 子叶 facade
- `markdown/06-milestones/v4.16.0/41-backend其余八叶模块壳抽离记录.md` — v4.16.0 BE-001E backend 其余八叶薄壳抽离，建立 interface/capability/runtime/graph/storage/ops/state/test 子 facade
- `markdown/06-milestones/v4.16.0/42-backend.interface_boundary子叶抽离完成记录.md` — v4.16.0 BE-001E-01 `backend.interface_boundary` 子叶抽离完成记录，确认 8 个 bridge facade 等价
- `markdown/06-milestones/v4.16.0/43-backend.capability子叶抽离完成记录.md` — v4.16.0 BE-001E-02 `backend.capability` 子叶抽离完成记录，确认 capability snapshot facade 等价
- `markdown/06-milestones/v4.16.0/44-backend.runtime子叶抽离完成记录.md` — v4.16.0 BE-001E-03 `backend.runtime` 子叶抽离完成记录，确认 runtime routes facade 等价
- `markdown/06-milestones/v4.16.0/45-backend.graph_compile子叶抽离完成记录.md` — v4.16.0 BE-001E-04 `backend.graph_compile` 子叶抽离完成记录，确认 compile/graph/QS route facade 等价
- `markdown/06-milestones/v4.16.0/46-backend.storage_security子叶抽离完成记录.md` — v4.16.0 BE-001E-05 `backend.storage_security` 子叶抽离完成记录，确认 credential API/vault facade 等价且安全暂停保留
- `markdown/06-milestones/v4.16.0/47-backend.ops_governance子叶抽离完成记录.md` — v4.16.0 BE-001E-06 `backend.ops_governance` 子叶抽离完成记录，确认 ops route facade 等价
- `markdown/06-milestones/v4.16.0/48-backend.app_state_wiring子叶抽离完成记录.md` — v4.16.0 BE-001E-07 `backend.app_state_wiring` 子叶抽离完成记录，确认 health/state factory facade 等价
- `markdown/06-milestones/v4.16.0/49-backend.test_support子叶抽离完成记录.md` — v4.16.0 BE-001E-08 `backend.test_support` 子叶抽离完成记录，确认 test scenario facade 等价
- `markdown/06-milestones/v4.16.0/50-backend.runtime.routes单子叶等价基线.md` — v4.16.0 BE-001F-01 `backend.runtime.routes` 单子叶等价基线，固定 runtime route aggregate facade 的真实 owner 和回归证据
- `markdown/06-milestones/v4.16.0/51-backend.runtime.routes抽离记录.md` — v4.16.0 BE-001F-02 `backend.runtime.routes` 抽离记录，接管 runtime route aggregate 列表并保留 handler/state owner 原位
- `markdown/06-milestones/v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md` — v4.16.0 BE-001G-01 `backend.runtime.routes.run` 单子叶等价基线，固定 run route group 和 event stream 排除边界
- `markdown/06-milestones/v4.16.0/53-backend.runtime.routes.run抽离记录.md` — v4.16.0 BE-001G-02 `backend.runtime.routes.run` 抽离记录，接管 run route group 并保留 handler/state owner 原位
- `markdown/06-milestones/v4.16.0/54-backend.runtime.routes.run单叶closeout.md` — v4.16.0 BE-001G-03 `backend.runtime.routes.run` 单叶 closeout，收束 route facade 并判断 run handler 层值得另起基线
- `markdown/06-milestones/v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md` — v4.16.0 BE-001H-01 `runtime.run.v4_handoff` 单子叶等价基线，固定 `/api/runtime/v4/run` handler 层边界和 `api_run` 证据
- `markdown/06-milestones/v4.16.0/56-runtime.run.v4_handoff抽离记录.md` — v4.16.0 BE-001H-02 `runtime.run.v4_handoff` 抽离记录，将 v4 handoff handler/type/helper 迁入 `src/runtime/run/v4_handoff.rs`
- `markdown/06-milestones/v4.16.0/57-runtime.run.v4_handoff单叶closeout.md` — v4.16.0 BE-001H-03 `runtime.run.v4_handoff` 单叶 closeout，确认本叶停止内部细分
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md` — v4.16.0 BE-001I-01 `runtime.run.session_start` 单子叶等价基线，固定 legacy `/api/runtime/test-run` handler 边界和 `api_run` 证据
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md` — v4.16.0 BE-001I-02 `runtime.run.session_start` 抽离记录，将 `start_test_run` 迁入 `src/runtime/run/session_start.rs`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md` — v4.16.0 BE-001I-03 `runtime.run.session_start` 单叶 closeout，确认本叶停止内部细分
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md` — v4.16.0 BE-001J-01 `runtime.run.record_store` 单子叶等价基线，固定 run record list/detail/save/discard 与 persistence/audit 边界
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md` — v4.16.0 BE-001J-02 `runtime.run.record_store` 真实边界梳理，校正 route method、frontend 调用和 shared helper owner
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md` — v4.16.0 BE-001J-03 `runtime.run.record_store` 抽离方案，锁定四个 handler 的最小移动方案和 shared helper 保留边界
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md` — v4.16.0 BE-001J-04 `runtime.run.record_store` 抽离记录，将四个 handler 迁入 `src/runtime/run/record_store.rs`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md` — v4.16.0 BE-001J-05 `runtime.run.record_store` 单叶 closeout，确认本叶等价并停止内部细拆
- `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md` — v4.16.0 BE-001K-01 `runtime.run.replay_status` 单子叶等价基线，固定 replay/status 与 SSE 排除边界
- `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md` — v4.16.0 BE-001K-02 `runtime.run.replay_status` 抽离方案，锁定两个 handler 的最小移动方案和 owner 保留边界
- `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md` — v4.16.0 BE-001K-03 `runtime.run.replay_status` 抽离记录，将两个 handler 迁入 `src/runtime/run/replay_status.rs`
- `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md` — v4.16.0 BE-001K-04 `runtime.run.replay_status` 单叶 closeout，确认本叶等价并停止内部细拆
- `markdown/06-milestones/v4.16.0/70-runtime.event_stream单子叶等价基线.md` — v4.16.0 BE-001L-01 `runtime.event_stream` 单子叶等价基线，固定 SSE route、frame order 和 keep-alive
- `markdown/06-milestones/v4.16.0/71-runtime.event_stream抽离方案.md` — v4.16.0 BE-001L-02 `runtime.event_stream` 抽离方案，锁定 `stream_run_events` 最小迁移和 route/shared owner 保留边界
- `markdown/06-milestones/v4.16.0/72-runtime.event_stream抽离记录.md` — v4.16.0 BE-001L-03 `runtime.event_stream` 抽离记录，将 `stream_run_events` 迁入 `src/runtime/event_stream.rs`
- `markdown/06-milestones/v4.16.0/73-runtime.event_stream单叶closeout.md` — v4.16.0 BE-001L-04 `runtime.event_stream` 单叶 closeout，确认本叶等价并停止内部细拆
- `markdown/06-milestones/v4.16.0/74-runtime.backtest单子叶等价基线.md` — v4.16.0 BE-001M-01 `runtime.backtest` 单子叶等价基线，冻结 backtest route group、artifact/compare/replay/persistence owner；当前不移动代码
- `markdown/06-milestones/v4.16.0/75-runtime.backtest抽离方案.md` — v4.16.0 BE-001M-02 `runtime.backtest` 抽离方案，锁定下一批只抽离 backtest route facade
- `markdown/06-milestones/v4.16.0/76-runtime.backtest抽离记录.md` - v4.16.0 BE-001M-03 `runtime.backtest` 抽离记录，将 backtest route registration 迁入 `src/backend/runtime/routes/backtest.rs`
- `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md` - v4.16.0 BE-001M-04 `runtime.backtest` 单叶 closeout，确认 route facade 等价并进入 handler 域细分判断
- `markdown/06-milestones/v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md` - v4.16.0 BE-001N-01 `runtime.backtest.execution_start` 单子叶等价基线，冻结 backtest 创建路径且当前不移动代码
- `markdown/06-milestones/v4.16.0/79-runtime.backtest.execution_start抽离方案.md` - v4.16.0 BE-001N-02 `runtime.backtest.execution_start` 抽离方案，锁定下一批只移动 backtest 创建路径 handler/helper 并保留 experiment 复用桥
- `markdown/06-milestones/v4.16.0/80-runtime.backtest.execution_start抽离记录.md` - v4.16.0 BE-001N-03 `runtime.backtest.execution_start` 抽离记录，将 backtest 创建路径 handler/helper 迁入 `src/runtime/backtest/execution_start.rs`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md` - v4.16.0 BE-001N-04 `runtime.backtest.execution_start` 单叶 closeout，确认等价并将下一候选固定为 `runtime.backtest.execution_start.v4_projection`
- `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md` - v4.16.0 BE-001O-01 `runtime.backtest.execution_start.v4_projection` 单子叶等价基线，冻结 v4 projection helper 输入输出、测试证据和禁止横向连接边界
- `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md` - v4.16.0 BE-001O-02 `runtime.backtest.execution_start.v4_projection` 抽离方案，锁定下一批只移动 projection helper 与现有单元测试
- `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md` - v4.16.0 BE-001O-03 `runtime.backtest.execution_start.v4_projection` 抽离记录，将 projection helper 与现有单元测试迁入 `src/runtime/backtest/v4_projection.rs`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md` - v4.16.0 BE-001O-04 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md` - v4.16.0 BE-001P-01 `runtime.backtest.execution_start.v4_request_resolution` 单子叶等价基线，冻结 v4 请求识别与 graph/symbol/event resolution
- `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md` - v4.16.0 BE-001P-02 `runtime.backtest.execution_start.v4_request_resolution` 抽离方案，锁定下一批只移动四个 request resolution helper
- `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md` - v4.16.0 BE-001P-03 `runtime.backtest.execution_start.v4_request_resolution` 抽离记录，将四个 request resolution helper 迁入 `src/runtime/backtest/v4_request_resolution.rs`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md` - v4.16.0 BE-001P-04 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md` - v4.16.0 BE-001Q-01 `runtime.backtest.execution_start.v4_runtime_execution` 单子叶等价基线，冻结 deterministic replay、v4 runtime execution 和 artifact output
- `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md` - v4.16.0 BE-001Q-02 `runtime.backtest.execution_start.v4_runtime_execution` 抽离方案，限定下一批只迁移 deterministic runtime execution 最小 helper
- `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md` - v4.16.0 BE-001Q-03 `runtime.backtest.execution_start.v4_runtime_execution` 抽离记录，将 deterministic bars/ticks 与 blocking runtime replay 迁入 `src/runtime/backtest/v4_runtime_execution.rs`
- `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md` - v4.16.0 BE-001Q-04 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md` - v4.16.0 BE-001R-01 `runtime.backtest.execution_start.legacy_dispatch` 单子叶等价基线，冻结 legacy compile/sandbox dispatch 且当前不移动代码
- `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md` - v4.16.0 BE-001R-02 `runtime.backtest.execution_start.legacy_dispatch` 抽离方案，限定下一批只迁移 legacy compile/sandbox dispatch 最小 helper
- `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md` - v4.16.0 BE-001R-03 `runtime.backtest.execution_start.legacy_dispatch` 抽离记录，将 legacy compile/sandbox dispatch 迁入 `src/runtime/backtest/legacy_dispatch.rs`
- `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md` - v4.16.0 BE-001R-04 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md` - v4.16.0 BE-001S-01 `runtime.backtest.execution_start` 父叶残余判断，确认回到 `runtime.backtest.record_store` 上层队列
- `markdown/06-milestones/v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md` - v4.16.0 BE-001T-01 `runtime.backtest.record_store` 单子叶等价基线，冻结 backtest list/detail/save/discard 边界
- `markdown/06-milestones/v4.16.0/100-runtime.backtest.record_store抽离方案.md` - v4.16.0 BE-001T-02 `runtime.backtest.record_store` 抽离方案，限定下一批只迁移四个 handler 并保留 shared owner
- `markdown/06-milestones/v4.16.0/101-runtime.backtest.record_store抽离记录.md` - v4.16.0 BE-001T-03 `runtime.backtest.record_store` 抽离记录，将四个 handler 迁入 `src/runtime/backtest/record_store.rs`
- `markdown/06-milestones/v4.16.0/102-runtime.backtest.record_store单叶closeout.md` - v4.16.0 BE-001T-04 `runtime.backtest.record_store` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/103-runtime.backtest.replay单子叶等价基线.md` - v4.16.0 BE-001U-01 `runtime.backtest.replay` 单子叶等价基线，冻结 replay route、query、response mapping 和 metrics 边界
- `markdown/06-milestones/v4.16.0/104-runtime.backtest.replay抽离方案.md` - v4.16.0 BE-001U-02 `runtime.backtest.replay` 抽离方案，限定下一批只迁移 `get_backtest_replay`
- `markdown/06-milestones/v4.16.0/105-runtime.backtest.replay抽离记录.md` - v4.16.0 BE-001U-03 `runtime.backtest.replay` 抽离记录，将 `get_backtest_replay` 迁入 `src/runtime/backtest/replay.rs`
- `markdown/06-milestones/v4.16.0/106-runtime.backtest.replay单叶closeout.md` - v4.16.0 BE-001U-04 `runtime.backtest.replay` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md` - v4.16.0 BE-001V-01 `runtime.backtest.experiment_sweep` 单子叶等价基线，冻结 experiment routes、参数网格、复用桥和 lifecycle 边界
- `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md` - v4.16.0 BE-001V-02 `runtime.backtest.experiment_sweep` 抽离方案，限定下一批只迁移 experiment handler/helper
- `markdown/06-milestones/v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md` - v4.16.0 BE-001V-03 `runtime.backtest.experiment_sweep` 抽离记录，将 experiment handler/helper 迁入 `src/runtime/backtest/experiment_sweep.rs`
- `markdown/06-milestones/v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md` - v4.16.0 BE-001V-04 `runtime.backtest.experiment_sweep` 单叶 closeout，确认等价并登记 `parameter_grid` 下一候选
- `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md` - v4.16.0 BE-001W-01 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线，冻结参数网格 helper 边界
- `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md` - v4.16.0 BE-001W-02 `runtime.backtest.experiment_sweep.parameter_grid` 抽离方案，限定下一批只迁移 3 个参数网格 helper
- `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md` - v4.16.0 BE-001W-03 `runtime.backtest.experiment_sweep.parameter_grid` 抽离记录，将 3 个 helper 迁入 `src/runtime/backtest/parameter_grid.rs`
- `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md` - v4.16.0 BE-001W-04 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md` - v4.16.0 BE-001X-01 `runtime.backtest.experiment_sweep` 父叶残余判断，下一候选为 `start_orchestration`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md` - v4.16.0 BE-001Y-01 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线，当前 `no code movement`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md` - v4.16.0 BE-001Y-02 `runtime.backtest.experiment_sweep.start_orchestration` 抽离方案，限定下一批只迁移 `start_backtest_experiment`
- `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md` - v4.16.0 BE-001Y-03 `runtime.backtest.experiment_sweep.start_orchestration` 抽离记录，将 `start_backtest_experiment` 迁入 `src/runtime/backtest/start_orchestration.rs`
- `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md` - v4.16.0 BE-001Y-04 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md` - v4.16.0 BE-001Z-01 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，下一候选为 `record_lifecycle`
- `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md` - v4.16.0 BE-001AA-01 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，当前 `no code movement`
- `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md` - v4.16.0 BE-001AA-02 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案，限定下一批迁移四个 lifecycle handler
- `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md` - v4.16.0 BE-001AA-03 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离记录，将四个 lifecycle handler 迁入 `src/runtime/backtest/record_lifecycle.rs`
- `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md` - v4.16.0 BE-001AA-04 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout，确认等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md` - v4.16.0 BE-001AB-01 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断，三子叶均已 closeout，父叶设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md` - v4.16.0 BE-001AC-01 `runtime.backtest` 父叶残余判断，当前 handler 域设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/127-backend.runtime.routes父叶残余判断.md` - v4.16.0 BE-001AD-01 `backend.runtime.routes` 父叶残余判断，父叶保持 `stop_split: false` 并登记下一候选 `backend.runtime.routes.mutation`
- `markdown/06-milestones/v4.16.0/128-backend.runtime.routes.mutation单子叶等价基线.md` - v4.16.0 BE-001AE-01 `backend.runtime.routes.mutation` 单子叶等价基线，冻结 mutation / AI proposal / approval route group
- `markdown/06-milestones/v4.16.0/129-backend.runtime.routes.mutation抽离方案.md` - v4.16.0 BE-001AE-02 `backend.runtime.routes.mutation` 抽离方案，只规划 route facade 最小迁移
- `markdown/06-milestones/v4.16.0/130-backend.runtime.routes.mutation抽离记录.md` - v4.16.0 BE-001AE-03 `backend.runtime.routes.mutation` route facade 实际抽离记录
- `markdown/06-milestones/v4.16.0/131-backend.runtime.routes.mutation单叶closeout.md` - v4.16.0 BE-001AE-04 `backend.runtime.routes.mutation` 单叶 closeout，route facade 等价并设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/132-runtime.mutation.parameter_mutation单子叶等价基线.md` - v4.16.0 BE-001AF-01 `runtime.mutation.parameter_mutation` 单子叶等价基线，冻结参数变更 handler 生命周期
- `markdown/06-milestones/v4.16.0/133-runtime.mutation.parameter_mutation抽离方案.md` - v4.16.0 BE-001AF-02 `runtime.mutation.parameter_mutation` 抽离方案，固定目标子模块、父级 re-export 和 shared helper 保留边界
- `markdown/06-milestones/v4.16.0/134-runtime.mutation.parameter_mutation抽离记录.md` - v4.16.0 BE-001AF-03 `runtime.mutation.parameter_mutation` 抽离记录，五个 parameter mutation handler 迁入子模块并保留父级兼容出口
- `markdown/06-milestones/v4.16.0/135-runtime.mutation.parameter_mutation单叶closeout.md` - v4.16.0 BE-001AF-04 `runtime.mutation.parameter_mutation` 单叶 closeout，设置 `stop_split: false` 并登记 transition lifecycle 下一基线
- `markdown/06-milestones/v4.16.0/136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md` - v4.16.0 BE-001AG-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线，冻结 activation / rollback lifecycle
- `markdown/06-milestones/v4.16.0/137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md` - v4.16.0 BE-001AG-02 `runtime.mutation.parameter_mutation.transition_lifecycle` 抽离方案，固定目标文件与迁移清单
- `markdown/06-milestones/v4.16.0/138-runtime.mutation.parameter_mutation.transition_lifecycle抽离记录.md` - v4.16.0 BE-001AG-03 `runtime.mutation.parameter_mutation.transition_lifecycle` 抽离记录，activation / rollback handler 迁入子模块
- `markdown/06-milestones/v4.16.0/139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md` - v4.16.0 BE-001AG-04 `runtime.mutation.parameter_mutation.transition_lifecycle` 单叶 closeout，设置 `stop_split: false`
- `markdown/06-milestones/v4.16.0/140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md` - v4.16.0 BE-001AH-01 `boundary_safety` 单子叶等价基线，冻结 boundary / safe window 纯策略
- `markdown/06-milestones/v4.16.0/141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md` - v4.16.0 BE-001AH-02 `boundary_safety` 抽离方案，固定目标文件和 wrapper 方式
- `markdown/06-milestones/v4.16.0/142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md` - v4.16.0 BE-001AH-03 `boundary_safety` 抽离记录，迁移 boundary / safe-window helper
- `markdown/06-milestones/v4.16.0/143-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单叶closeout.md` - v4.16.0 BE-001AH-04 `boundary_safety` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md` - v4.16.0 BE-001AI-01 `transition_lifecycle` 父叶残余判断，父叶保持 `stop_split: false`
- `markdown/06-milestones/v4.16.0/145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md` - v4.16.0 BE-001AJ-01 `activation_flow` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md` - v4.16.0 BE-001AJ-02 `activation_flow` 抽离方案
- `markdown/06-milestones/v4.16.0/147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md` - v4.16.0 BE-001AJ-03 `activation_flow` 抽离记录，将 `activate_runtime_parameter_mutation` 迁入 child
- `markdown/06-milestones/v4.16.0/148-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单叶closeout.md` - v4.16.0 BE-001AJ-04 `activation_flow` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/149-runtime.mutation.parameter_mutation.transition_lifecycle第二轮父叶残余判断.md` - v4.16.0 BE-001AK-01 `transition_lifecycle` 第二轮父叶残余判断，下一候选为 `rollback_flow`
- `markdown/06-milestones/v4.16.0/150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md` - v4.16.0 BE-001AL-01 `rollback_flow` 单子叶等价基线，冻结 rollback transaction 状态机
- `markdown/06-milestones/v4.16.0/151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md` - v4.16.0 BE-001AL-02 `rollback_flow` 抽离方案，固定目标文件与父级 re-export
- `markdown/06-milestones/v4.16.0/152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md` - v4.16.0 BE-001AL-03 `rollback_flow` 抽离记录，将 `rollback_runtime_parameter_mutation` 迁入 child
- `markdown/06-milestones/v4.16.0/153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md` - v4.16.0 BE-001AL-04 `rollback_flow` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md` - v4.16.0 BE-001AM-01 `transition_lifecycle` 第三轮父叶残余判断，下一候选为 `activation_snapshot_side_effect`
- `markdown/06-milestones/v4.16.0/155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md` - v4.16.0 BE-001AN-01 `activation_snapshot_side_effect` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md` - v4.16.0 BE-001AN-02 `activation_snapshot_side_effect` 抽离方案
- `markdown/06-milestones/v4.16.0/157-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离记录.md` - v4.16.0 BE-001AN-03 `activation_snapshot_side_effect` 抽离记录，将 `auto_snapshot_on_activation` 迁入 child
- `markdown/06-milestones/v4.16.0/158-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单叶closeout.md` - v4.16.0 BE-001AN-04 `activation_snapshot_side_effect` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md` - v4.16.0 BE-001AO-01 `transition_lifecycle` 第四轮父叶残余判断，父叶保持 `stop_split: false`，下一候选为 `transition_record_persistence`
- `markdown/06-milestones/v4.16.0/160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md` - v4.16.0 BE-001AP-01 `transition_record_persistence` 单子叶等价基线，冻结 lifecycle entry 与 transition persistence
- `markdown/06-milestones/v4.16.0/161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md` - v4.16.0 BE-001AP-02 `transition_record_persistence` 抽离方案，固定目标文件、父级声明和回退点
- `markdown/06-milestones/v4.16.0/162-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离记录.md` - v4.16.0 BE-001AP-03 `transition_record_persistence` 实际抽离，迁移 lifecycle entry 与 transition persistence helper
- `markdown/06-milestones/v4.16.0/163-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单叶closeout.md` - v4.16.0 BE-001AP-04 `transition_record_persistence` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md` - v4.16.0 BE-001AQ-01 `transition_lifecycle` 第五轮父叶残余判断，父叶保持 `stop_split: false`，下一候选为 `rollback_record_identity`
- `markdown/06-milestones/v4.16.0/165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md` - v4.16.0 BE-001AR-01 `rollback_record_identity` 单子叶等价基线，冻结 rollback id digest contract
- `markdown/06-milestones/v4.16.0/166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md` - v4.16.0 BE-001AR-02 `rollback_record_identity` 抽离方案，固定目标文件、父级声明和回退点
- `markdown/06-milestones/v4.16.0/167-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离记录.md` - v4.16.0 BE-001AR-03 `rollback_record_identity` 实际抽离，迁移 rollback id helper
- `markdown/06-milestones/v4.16.0/168-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单叶closeout.md` - v4.16.0 BE-001AR-04 `rollback_record_identity` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/169-runtime.mutation.parameter_mutation.transition_lifecycle第六轮父叶残余判断.md` - v4.16.0 BE-001AS-01 `transition_lifecycle` 第六轮父叶残余判断，父叶设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/170-runtime.mutation.parameter_mutation父叶残余判断.md` - v4.16.0 BE-001AT-01 `parameter_mutation` 父叶残余判断，下一候选为 `proposal_creation`
- `markdown/06-milestones/v4.16.0/171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md` - v4.16.0 BE-001AU-01 `proposal_creation` 单子叶等价基线，冻结 create handler 与 record id helper
- `markdown/06-milestones/v4.16.0/172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md` - v4.16.0 BE-001AU-02 `proposal_creation` 抽离方案，固定目标文件、父级声明和 handler re-export
- `markdown/06-milestones/v4.16.0/173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md` - v4.16.0 BE-001AU-03 `proposal_creation` 实际抽离，迁移 create handler 与 record id helper
- `markdown/06-milestones/v4.16.0/174-runtime.mutation.parameter_mutation.proposal_creation单叶closeout.md` - v4.16.0 BE-001AU-04 `proposal_creation` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md` - v4.16.0 BE-001AV-01 `parameter_mutation` 第二轮父叶残余判断，下一候选为 `record_query`
- `markdown/06-milestones/v4.16.0/176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md` - v4.16.0 BE-001AW-01 `record_query` 单子叶等价基线，冻结 list/detail 查询流
- `markdown/06-milestones/v4.16.0/177-runtime.mutation.parameter_mutation.record_query抽离方案.md` - v4.16.0 BE-001AW-02 `record_query` 抽离方案，固定目标文件、父级声明和双 handler re-export
- `markdown/06-milestones/v4.16.0/178-runtime.mutation.parameter_mutation.record_query抽离记录.md` - v4.16.0 BE-001AW-03 `record_query` 实际抽离，迁移 list/detail handler
- `markdown/06-milestones/v4.16.0/179-runtime.mutation.parameter_mutation.record_query单叶closeout.md` - v4.16.0 BE-001AW-04 `record_query` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md` - v4.16.0 BE-001AX-01 `parameter_mutation` 第三轮父叶残余判断，父叶设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/181-runtime.mutation.ai_proposal单子叶等价基线.md` - v4.16.0 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线，冻结 AI proposal / approval handler 域
- `markdown/06-milestones/v4.16.0/182-runtime.mutation.ai_proposal抽离方案.md` - v4.16.0 BE-001AY-02 `runtime.mutation.ai_proposal` 抽离方案，固定目标文件与迁移清单
- `markdown/06-milestones/v4.16.0/183-runtime.mutation.ai_proposal抽离记录.md` - v4.16.0 BE-001AY-03 `runtime.mutation.ai_proposal` 实际抽离，迁移 AI proposal / approval handler
- `markdown/06-milestones/v4.16.0/184-runtime.mutation.ai_proposal单叶closeout.md` - v4.16.0 BE-001AY-04 `runtime.mutation.ai_proposal` 单叶 closeout，设置 `stop_split: false`
- `markdown/06-milestones/v4.16.0/185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md` - v4.16.0 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线，冻结 validation / analysis helper
- `markdown/06-milestones/v4.16.0/186-runtime.mutation.ai_proposal.static_check抽离方案.md` - v4.16.0 BE-001AZ-02 `runtime.mutation.ai_proposal.static_check` 抽离方案，固定目标文件和 helper visibility
- `markdown/06-milestones/v4.16.0/187-runtime.mutation.ai_proposal.static_check抽离记录.md` - v4.16.0 BE-001AZ-03 `runtime.mutation.ai_proposal.static_check` 实际抽离记录，helper 与静态检查单测迁入 child
- `markdown/06-milestones/v4.16.0/188-runtime.mutation.ai_proposal.static_check单叶closeout.md` - v4.16.0 BE-001AZ-04 `runtime.mutation.ai_proposal.static_check` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/189-runtime.mutation.ai_proposal父叶残余判断.md` - v4.16.0 BE-001BA-01 `runtime.mutation.ai_proposal` 父叶残余判断，下一候选为 `source_governance_identity`
- `markdown/06-milestones/v4.16.0/190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md` - v4.16.0 BE-001BB-01 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md` - v4.16.0 BE-001BB-02 `runtime.mutation.ai_proposal.source_governance_identity` 抽离方案
- `markdown/06-milestones/v4.16.0/192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md` - v4.16.0 BE-001BB-03 `runtime.mutation.ai_proposal.source_governance_identity` 实际抽离记录
- `markdown/06-milestones/v4.16.0/193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md` - v4.16.0 BE-001BB-04 `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout，设置 `stop_split: true`
- `markdown/06-milestones/v4.16.0/194-runtime.mutation.ai_proposal第二轮父叶残余判断.md` - v4.16.0 BE-001BC-01 `runtime.mutation.ai_proposal` 第二轮父叶残余判断，下一候选为 `event_lifecycle`
- `markdown/06-milestones/v4.16.0/195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md` - v4.16.0 BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md` - v4.16.0 BE-001BD-02 `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案
- `markdown/06-milestones/v4.16.0/197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md` - v4.16.0 BE-001BD-03 `runtime.mutation.ai_proposal.event_lifecycle` 抽离记录
- `markdown/06-milestones/v4.16.0/198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md` - v4.16.0 BE-001BD-04 `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout
- `markdown/06-milestones/v4.16.0/199-runtime.mutation.ai_proposal第三轮父叶残余判断.md` - v4.16.0 BE-001BE-01 `runtime.mutation.ai_proposal` 第三轮父叶残余判断
- `markdown/06-milestones/v4.16.0/200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md` - v4.16.0 BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/201-runtime.mutation.ai_proposal.record_query抽离方案.md` - v4.16.0 BE-001BF-02 `runtime.mutation.ai_proposal.record_query` 抽离方案
- `markdown/06-milestones/v4.16.0/202-runtime.mutation.ai_proposal.record_query抽离记录.md` - v4.16.0 BE-001BF-03 `runtime.mutation.ai_proposal.record_query` 抽离记录
- `markdown/06-milestones/v4.16.0/203-runtime.mutation.ai_proposal.record_query单叶closeout.md` - v4.16.0 BE-001BF-04 `runtime.mutation.ai_proposal.record_query` 单叶 closeout
- `markdown/06-milestones/v4.16.0/204-runtime.mutation.ai_proposal第四轮父叶残余判断.md` - v4.16.0 BE-001BG-01 `runtime.mutation.ai_proposal` 第四轮父叶残余判断，下一候选为 `approval_review`
- `markdown/06-milestones/v4.16.0/205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md` - v4.16.0 BE-001BH-01 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/206-runtime.mutation.ai_proposal.approval_review抽离方案.md` - v4.16.0 BE-001BH-02 `runtime.mutation.ai_proposal.approval_review` 抽离方案
- `markdown/06-milestones/v4.16.0/207-runtime.mutation.ai_proposal.approval_review抽离记录.md` - v4.16.0 BE-001BH-03 `runtime.mutation.ai_proposal.approval_review` 抽离记录
- `markdown/06-milestones/v4.16.0/208-runtime.mutation.ai_proposal.approval_review单叶closeout.md` - v4.16.0 BE-001BH-04 `runtime.mutation.ai_proposal.approval_review` 单叶 closeout
- `markdown/06-milestones/v4.16.0/209-runtime.mutation.ai_proposal第五轮父叶残余判断.md` - v4.16.0 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断，下一候选为 `approval_persistence`
- `markdown/06-milestones/v4.16.0/210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md` - v4.16.0 BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md` - v4.16.0 BE-001BJ-02 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案
- `markdown/06-milestones/v4.16.0/212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md` - v4.16.0 BE-001BJ-03 `runtime.mutation.ai_proposal.approval_persistence` 抽离记录
- `markdown/06-milestones/v4.16.0/213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md` - v4.16.0 BE-001BJ-04 `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout
- `markdown/06-milestones/v4.16.0/214-runtime.mutation.ai_proposal第六轮父叶残余判断.md` - v4.16.0 BE-001BK-01 `runtime.mutation.ai_proposal` 第六轮父叶残余判断，下一候选为 `sandbox_trigger`
- `markdown/06-milestones/v4.16.0/215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md` - v4.16.0 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md` - v4.16.0 BE-001BL-02 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案
- `markdown/06-milestones/v4.16.0/217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md` - v4.16.0 BE-001BL-03 `runtime.mutation.ai_proposal.sandbox_trigger` 实际抽离记录
- `markdown/06-milestones/v4.16.0/218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md` - v4.16.0 BE-001BL-04 `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout
- `markdown/06-milestones/v4.16.0/219-runtime.mutation.ai_proposal第七轮父叶残余判断.md` - v4.16.0 BE-001BM-01 `runtime.mutation.ai_proposal` 第七轮父叶残余判断，下一候选为 `status_transition`
- `markdown/06-milestones/v4.16.0/220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md` - v4.16.0 BE-001BN-01 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/221-runtime.mutation.ai_proposal.status_transition抽离方案.md` - v4.16.0 BE-001BN-02 `runtime.mutation.ai_proposal.status_transition` 抽离方案
- `markdown/06-milestones/v4.16.0/222-runtime.mutation.ai_proposal.status_transition抽离记录.md` - v4.16.0 BE-001BN-03 `runtime.mutation.ai_proposal.status_transition` 实际抽离记录
- `markdown/06-milestones/v4.16.0/223-runtime.mutation.ai_proposal.status_transition单叶closeout.md` - v4.16.0 BE-001BN-04 `runtime.mutation.ai_proposal.status_transition` 单叶 closeout
- `markdown/06-milestones/v4.16.0/224-runtime.mutation.ai_proposal第八轮父叶残余判断.md` - v4.16.0 BE-001BO-01 `runtime.mutation.ai_proposal` 第八轮父叶残余判断，下一候选为 `proposal_creation`
- `markdown/06-milestones/v4.16.0/225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md` - v4.16.0 BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md` - v4.16.0 BE-001BP-02 `runtime.mutation.ai_proposal.proposal_creation` 抽离方案
- `markdown/06-milestones/v4.16.0/227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md` - v4.16.0 BE-001BP-03 `runtime.mutation.ai_proposal.proposal_creation` 实际抽离记录
- `markdown/06-milestones/v4.16.0/228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md` - v4.16.0 BE-001BP-04 `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout
- `markdown/06-milestones/v4.16.0/229-runtime.mutation.ai_proposal第九轮父叶残余判断.md` - v4.16.0 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断
- `markdown/06-milestones/v4.16.0/230-backend.runtime.routes第二轮父叶残余判断.md` - v4.16.0 BE-001BR-01 `backend.runtime.routes` 第二轮父叶残余判断
- `markdown/06-milestones/v4.16.0/231-backend.runtime.routes.experiment单子叶等价基线.md` - v4.16.0 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/232-backend.runtime.routes.experiment抽离方案.md` - v4.16.0 BE-001BS-02 `backend.runtime.routes.experiment` 抽离方案
- `markdown/06-milestones/v4.16.0/233-backend.runtime.routes.experiment抽离记录.md` - v4.16.0 BE-001BS-03 `backend.runtime.routes.experiment` 实际抽离记录
- `markdown/06-milestones/v4.16.0/234-backend.runtime.routes.experiment单叶closeout.md` - v4.16.0 BE-001BS-04 `backend.runtime.routes.experiment` 单叶 closeout
- `markdown/06-milestones/v4.16.0/235-backend.runtime.routes第三轮父叶残余判断.md` - v4.16.0 BE-001BT-01 `backend.runtime.routes` 第三轮父叶残余判断
- `markdown/06-milestones/v4.16.0/236-backend.runtime.routes.evidence单子叶等价基线.md` - v4.16.0 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/237-backend.runtime.routes.evidence抽离方案.md` - v4.16.0 BE-001BU-02 `backend.runtime.routes.evidence` 抽离方案
- `markdown/06-milestones/v4.16.0/238-backend.runtime.routes.evidence抽离记录.md` - v4.16.0 BE-001BU-03 `backend.runtime.routes.evidence` 实际抽离记录
- `markdown/06-milestones/v4.16.0/239-backend.runtime.routes.evidence单叶closeout.md` - v4.16.0 BE-001BU-04 `backend.runtime.routes.evidence` 单叶 closeout
- `markdown/06-milestones/v4.16.0/240-backend.runtime.routes第四轮父叶残余判断.md` - v4.16.0 BE-001BV-01 `backend.runtime.routes` 第四轮父叶残余判断
- `markdown/06-milestones/v4.16.0/241-backend.runtime.routes.event_stream单子叶等价基线.md` - v4.16.0 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/242-backend.runtime.routes.event_stream抽离方案.md` - v4.16.0 BE-001BW-02 `backend.runtime.routes.event_stream` 抽离方案
- `markdown/06-milestones/v4.16.0/243-backend.runtime.routes.event_stream抽离记录.md` - v4.16.0 BE-001BW-03 `backend.runtime.routes.event_stream` 实际抽离记录
- `markdown/06-milestones/v4.16.0/244-backend.runtime.routes.event_stream单叶closeout.md` - v4.16.0 BE-001BW-04 `backend.runtime.routes.event_stream` 单叶 closeout
- `markdown/06-milestones/v4.16.0/245-backend.runtime.routes第五轮父叶残余判断.md` - v4.16.0 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断
- `markdown/06-milestones/v4.16.0/246-backend.runtime.routes.report_ops单子叶等价基线.md` - v4.16.0 BE-001BY-01 `backend.runtime.routes.report_ops` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/247-backend.runtime.routes.report_ops抽离方案.md` - v4.16.0 BE-001BY-02 `backend.runtime.routes.report_ops` 抽离方案
- `markdown/06-milestones/v4.16.0/248-backend.runtime.routes.report_ops抽离记录.md` - v4.16.0 BE-001BY-03 `backend.runtime.routes.report_ops` 实际抽离记录
- `markdown/06-milestones/v4.16.0/249-backend.runtime.routes.report_ops单叶closeout.md` - v4.16.0 BE-001BY-04 `backend.runtime.routes.report_ops` 单叶 closeout
- `markdown/06-milestones/v4.16.0/250-backend.runtime.routes第六轮父叶残余判断.md` - v4.16.0 BE-001BZ-01 `backend.runtime.routes` 第六轮父叶残余判断
- `markdown/06-milestones/v4.16.0/251-backend.runtime父叶残余判断.md` - v4.16.0 BE-001CA-01 `backend.runtime` 父叶残余判断
- `markdown/06-milestones/v4.16.0/252-runtime.report_ops单子叶等价基线.md` - v4.16.0 BE-001CB-01 `runtime.report_ops` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/253-runtime.report_ops抽离方案.md` - v4.16.0 BE-001CB-02 `runtime.report_ops` 抽离方案
- `markdown/06-milestones/v4.16.0/254-runtime.report_ops抽离记录.md` - v4.16.0 BE-001CB-03 `runtime.report_ops` 实际抽离记录
- `markdown/06-milestones/v4.16.0/255-runtime.report_ops单叶closeout.md` - v4.16.0 BE-001CB-04 `runtime.report_ops` 单叶 closeout
- `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md` - v4.16.0 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md` - v4.16.0 BE-001CC-02 `runtime.report_ops.runtime_report` 抽离方案
- `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md` - v4.16.0 BE-001CC-03 `runtime.report_ops.runtime_report` 实际抽离记录
- `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md` - v4.16.0 BE-001CC-04 `runtime.report_ops.runtime_report` 单叶 closeout
- `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md` - v4.16.0 BE-001CD-01 `runtime.report_ops` 父叶残余判断
- `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md` - v4.16.0 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md` - v4.16.0 BE-001CE-02 `runtime.report_ops.v1_report_endpoints` test-first 抽离方案
- `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md` - v4.16.0 BE-001CE-03 `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测记录
- `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md` - v4.16.0 BE-001CE-04 `runtime.report_ops.v1_report_endpoints` 实际抽离记录
- `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md` - v4.16.0 BE-001CE-05 `runtime.report_ops.v1_report_endpoints` 单叶 closeout
- `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md` - v4.16.0 BE-001CF-01 `runtime.report_ops` 父叶残余判断
- `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md` - v4.16.0 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md` - v4.16.0 BE-001CG-02 `runtime.report_ops.merge_generation_health` test-first 抽离方案
- `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md` - v4.16.0 BE-001CG-03 `runtime.report_ops.merge_generation_health` endpoint smoke 补测记录
- `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md` - v4.16.0 BE-001CG-04 `runtime.report_ops.merge_generation_health` 实际抽离记录
- `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md` - v4.16.0 BE-001CG-05 `runtime.report_ops.merge_generation_health` 单叶 closeout
- `markdown/06-milestones/v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md` - v4.16.0 BE-001CH-01 `runtime.report_ops` 第二轮父叶残余判断
- `markdown/06-milestones/v4.16.0/273-backend.runtime第二轮父叶残余判断.md` - v4.16.0 BE-001CI-01 `backend.runtime` 第二轮父叶残余判断
- `markdown/06-milestones/v4.16.0/274-runtime.evidence_health单子叶等价基线.md` - v4.16.0 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/275-runtime.evidence_health抽离方案.md` - v4.16.0 BE-001CJ-02 `runtime.evidence_health` 抽离方案
- `markdown/06-milestones/v4.16.0/276-runtime.evidence_health抽离记录.md` - v4.16.0 BE-001CJ-03 `runtime.evidence_health` 实际抽离
- `markdown/06-milestones/v4.16.0/277-runtime.evidence_health单叶closeout.md` - v4.16.0 BE-001CJ-04 `runtime.evidence_health` 单叶 closeout
- `markdown/06-milestones/v4.16.0/278-backend.runtime第三轮父叶残余判断.md` - v4.16.0 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断
- `markdown/06-milestones/v4.16.0/279-runtime.mutation.shared_governance单子叶等价基线.md` - v4.16.0 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/280-runtime.mutation.shared_governance抽离方案.md` - v4.16.0 BE-001CL-02 `runtime.mutation.shared_governance` 抽离方案
- `markdown/06-milestones/v4.16.0/281-runtime.mutation.shared_governance抽离记录.md` - v4.16.0 BE-001CL-03 `runtime.mutation.shared_governance` 实际抽离
- `markdown/06-milestones/v4.16.0/282-runtime.mutation.shared_governance单叶closeout.md` - v4.16.0 BE-001CL-04 `runtime.mutation.shared_governance` 单叶 closeout
- `markdown/06-milestones/v4.16.0/283-backend.runtime第四轮父叶残余判断.md` - v4.16.0 BE-001CM-01 `backend.runtime` 第四轮父叶残余判断
- `markdown/06-milestones/v4.16.0/284-runtime.query_support单子叶等价基线.md` - v4.16.0 BE-001CN-01 `runtime.query_support` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/285-runtime.query_support抽离方案.md` - v4.16.0 BE-001CN-02 `runtime.query_support` 抽离方案
- `markdown/06-milestones/v4.16.0/286-runtime.query_support抽离记录.md` - v4.16.0 BE-001CN-03 `runtime.query_support` 实际抽离
- `markdown/06-milestones/v4.16.0/287-runtime.query_support单叶closeout.md` - v4.16.0 BE-001CN-04 `runtime.query_support` 单叶 closeout
- `markdown/06-milestones/v4.16.0/288-backend.runtime第五轮父叶残余判断.md` - v4.16.0 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断
- `markdown/06-milestones/v4.16.0/289-runtime.response_support单子叶等价基线.md` - v4.16.0 BE-001CP-01 `runtime.response_support` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/290-runtime.response_support抽离方案.md` - v4.16.0 BE-001CP-02 `runtime.response_support` 抽离方案
- `markdown/06-milestones/v4.16.0/291-runtime.response_support抽离记录.md` - v4.16.0 BE-001CP-03 `runtime.response_support` 实际抽离
- `markdown/06-milestones/v4.16.0/292-runtime.response_support单叶closeout.md` - v4.16.0 BE-001CP-04 `runtime.response_support` 单叶 closeout
- `markdown/06-milestones/v4.16.0/293-backend.runtime第六轮父叶残余判断.md` - v4.16.0 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断
- `markdown/06-milestones/v4.16.0/294-runtime.run_guard单子叶等价基线.md` - v4.16.0 BE-001CR-01 `runtime.run_guard` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/295-runtime.run_guard抽离方案.md` - v4.16.0 BE-001CR-02 `runtime.run_guard` 抽离方案
- `markdown/06-milestones/v4.16.0/296-runtime.run_guard抽离记录.md` - v4.16.0 BE-001CR-03 `runtime.run_guard` 实际抽离
- `markdown/06-milestones/v4.16.0/297-runtime.run_guard单叶closeout.md` - v4.16.0 BE-001CR-04 `runtime.run_guard` 单叶 closeout
- `markdown/06-milestones/v4.16.0/298-backend.runtime第七轮父叶残余判断.md` - v4.16.0 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断
- `markdown/06-milestones/v4.16.0/299-runtime.experiment_limit单子叶等价基线.md` - v4.16.0 BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/300-runtime.experiment_limit抽离方案.md` - v4.16.0 BE-001CT-02 `runtime.experiment_limit` test-first 抽离方案
- `markdown/06-milestones/v4.16.0/301-runtime.experiment_limit补测记录.md` - v4.16.0 BE-001CT-03 `runtime.experiment_limit` endpoint smoke 补测记录
- `markdown/06-milestones/v4.16.0/302-runtime.experiment_limit抽离记录.md` - v4.16.0 BE-001CT-04 `runtime.experiment_limit` 实际抽离记录
- `markdown/06-milestones/v4.16.0/303-runtime.experiment_limit单叶closeout.md` - v4.16.0 BE-001CT-05 `runtime.experiment_limit` 单叶 closeout
- `markdown/06-milestones/v4.16.0/304-backend.runtime第八轮父叶残余判断.md` - v4.16.0 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断
- `markdown/06-milestones/v4.16.0/305-runtime.parent_include_cleanup单子叶等价基线.md` - v4.16.0 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/306-runtime.parent_include_cleanup抽离方案.md` - v4.16.0 BE-001CV-02 `runtime.parent_include_cleanup` 抽离方案
- `markdown/06-milestones/v4.16.0/307-runtime.parent_include_cleanup清理记录.md` - v4.16.0 BE-001CV-03 `runtime.parent_include_cleanup` 实际 cleanup
- `markdown/06-milestones/v4.16.0/308-backend.runtime第九轮父叶残余判断.md` - v4.16.0 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断
- `markdown/06-milestones/v4.16.0/309-runtime.parent_import_bridge单子叶等价基线.md` - v4.16.0 BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/310-runtime.parent_import_bridge抽离方案.md` - v4.16.0 BE-001CX-02 `runtime.parent_import_bridge` 抽离方案
- `markdown/06-milestones/v4.16.0/311-runtime.root_support_import_pilot抽离记录.md` - v4.16.0 BE-001CX-03 `runtime.root_support_import_pilot` 抽离记录
- `markdown/06-milestones/v4.16.0/312-runtime.root_support_import_pilot单叶closeout.md` - v4.16.0 BE-001CX-04 `runtime.root_support_import_pilot` 单叶 closeout
- `markdown/06-milestones/v4.16.0/313-runtime.root_entry_import_pass单子叶等价基线.md` - v4.16.0 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/314-runtime.root_entry_import_pass抽离方案.md` - v4.16.0 BE-001CY-02 `runtime.root_entry_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/315-runtime.root_entry_import_pass抽离记录.md` - v4.16.0 BE-001CY-03 `runtime.root_entry_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/316-runtime.root_entry_import_pass单叶closeout.md` - v4.16.0 BE-001CY-04 `runtime.root_entry_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/317-runtime.report_ops_import_pass单子叶等价基线.md` - v4.16.0 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/318-runtime.report_ops_import_pass抽离方案.md` - v4.16.0 BE-001CZ-02 `runtime.report_ops_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/319-runtime.report_ops_import_pass抽离记录.md` - v4.16.0 BE-001CZ-03 `runtime.report_ops_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/320-runtime.report_ops_import_pass单叶closeout.md` - v4.16.0 BE-001CZ-04 `runtime.report_ops_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/321-runtime.parent_import_bridge父叶残余判断.md` - v4.16.0 BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断
- `markdown/06-milestones/v4.16.0/322-runtime.run_import_pass单子叶等价基线.md` - v4.16.0 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/323-runtime.run_import_pass抽离方案.md` - v4.16.0 BE-001DB-02 `runtime.run_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/324-runtime.run_import_pass抽离记录.md` - v4.16.0 BE-001DB-03 `runtime.run_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/325-runtime.run_import_pass单叶closeout.md` - v4.16.0 BE-001DB-04 `runtime.run_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/326-runtime.parent_import_bridge父叶残余判断.md` - v4.16.0 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断
- `markdown/06-milestones/v4.16.0/327-runtime.backtest_import_pass单子叶等价基线.md` - v4.16.0 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/328-runtime.backtest_import_pass抽离方案.md` - v4.16.0 BE-001DD-02 `runtime.backtest_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/329-runtime.backtest.record_store_import_pass单子叶等价基线.md` - v4.16.0 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/330-runtime.backtest.record_store_import_pass抽离方案.md` - v4.16.0 BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/331-runtime.backtest.record_store_import_pass抽离记录.md` - v4.16.0 BE-001DE-03 `runtime.backtest.record_store_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/332-runtime.backtest.record_store_import_pass单叶closeout.md` - v4.16.0 BE-001DE-04 `runtime.backtest.record_store_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/333-runtime.backtest_import_pass父叶残余判断.md` - v4.16.0 BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断
- `markdown/06-milestones/v4.16.0/334-runtime.backtest.replay_import_pass单子叶等价基线.md` - v4.16.0 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/335-runtime.backtest.replay_import_pass抽离方案.md` - v4.16.0 BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/336-runtime.backtest.replay_import_pass抽离记录.md` - v4.16.0 BE-001DG-03 `runtime.backtest.replay_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/337-runtime.backtest.replay_import_pass单叶closeout.md` - v4.16.0 BE-001DG-04 `runtime.backtest.replay_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/338-runtime.backtest_import_pass第二轮父叶残余判断.md` - v4.16.0 BE-001DH-01 `runtime.backtest_import_pass` 第二轮父叶残余判断
- `markdown/06-milestones/v4.16.0/339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md` - v4.16.0 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/340-runtime.backtest.experiment_sweep_import_pass抽离方案.md` - v4.16.0 BE-001DI-02 `runtime.backtest.experiment_sweep_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/341-runtime.backtest.experiment_sweep_import_pass抽离记录.md` - v4.16.0 BE-001DI-03 `runtime.backtest.experiment_sweep_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md` - v4.16.0 BE-001DI-04 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/343-runtime.backtest_import_pass第三轮父叶残余判断.md` - v4.16.0 BE-001DJ-01 `runtime.backtest_import_pass` 第三轮父叶残余判断
- `markdown/06-milestones/v4.16.0/344-runtime.backtest.execution_start_import_pass单子叶等价基线.md` - v4.16.0 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/345-runtime.backtest.execution_start_import_pass抽离方案.md` - v4.16.0 BE-001DK-02 `runtime.backtest.execution_start_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/346-runtime.backtest.execution_start_import_pass抽离记录.md` - v4.16.0 BE-001DK-03 `runtime.backtest.execution_start_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/347-runtime.backtest.execution_start_import_pass单叶closeout.md` - v4.16.0 BE-001DK-04 `runtime.backtest.execution_start_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/348-runtime.backtest_import_pass第四轮父叶残余判断.md` - v4.16.0 BE-001DL-01 `runtime.backtest_import_pass` 第四轮父叶残余判断
- `markdown/06-milestones/v4.16.0/349-runtime.parent_import_bridge父叶残余判断.md` - v4.16.0 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断
- `markdown/06-milestones/v4.16.0/350-runtime.mutation_import_pass单子叶等价基线.md` - v4.16.0 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/351-runtime.mutation_import_pass抽离方案.md` - v4.16.0 BE-001DN-02 `runtime.mutation_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md` - v4.16.0 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/353-runtime.mutation.shared_governance_import_pass抽离方案.md` - v4.16.0 BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/354-runtime.mutation.shared_governance_import_pass抽离记录.md` - v4.16.0 BE-001DO-03 `runtime.mutation.shared_governance_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/355-runtime.mutation.shared_governance_import_pass单叶closeout.md` - v4.16.0 BE-001DO-04 `runtime.mutation.shared_governance_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/356-runtime.mutation_import_pass父叶残余判断.md` - v4.16.0 BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断
- `markdown/06-milestones/v4.16.0/357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md` - v4.16.0 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/358-runtime.mutation.parameter_mutation_import_pass抽离方案.md` - v4.16.0 BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md` - v4.16.0 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md` - v4.16.0 BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/361-runtime.mutation.parameter_mutation.record_query_import_pass抽离记录.md` - v4.16.0 BE-001DR-03 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/362-runtime.mutation.parameter_mutation.record_query_import_pass单叶closeout.md` - v4.16.0 BE-001DR-04 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md` - v4.16.0 BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断
- `markdown/06-milestones/v4.16.0/364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md` - v4.16.0 BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md` - v4.16.0 BE-001DT-02 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/366-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离记录.md` - v4.16.0 BE-001DT-03 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/367-runtime.mutation.parameter_mutation.proposal_creation_import_pass单叶closeout.md` - v4.16.0 BE-001DT-04 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/368-runtime.mutation.parameter_mutation_import_pass第二轮父叶残余判断.md` - v4.16.0 BE-001DU-01 `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断
- `markdown/06-milestones/v4.16.0/369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md` - v4.16.0 BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md` - v4.16.0 BE-001DV-02 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md` - v4.16.0 BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线
- `markdown/06-milestones/v4.16.0/372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md` - v4.16.0 BE-001DW-02 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案
- `markdown/06-milestones/v4.16.0/373-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离记录.md` - v4.16.0 BE-001DW-03 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离记录
- `markdown/06-milestones/v4.16.0/374-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单叶closeout.md` - v4.16.0 BE-001DW-04 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout
- `markdown/06-milestones/v4.16.0/375-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md` - v4.16.0 BE-001DX-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断
- `src/backend/runtime/routes/evidence.rs` - backend runtime evidence route child，承接 evidence health / cleanup route registration
- `src/backend/runtime/routes/event_stream.rs` - backend runtime event stream route child，承接 run events SSE route registration
- `src/backend/runtime/routes/experiment.rs` - backend runtime experiment route child，承接 experiment route registration
- `src/backend/runtime/routes/report_ops.rs` - backend runtime report ops route child，承接 runtime reports、merge records、runtime generations、storage health、ops/audit/research report route registration
- `src/runtime/mutation/ai_proposal.rs` - runtime AI proposal child，承接 AI proposal / approval public handler 与专属 helper
- `src/runtime/mutation/shared_governance.rs` - runtime mutation shared governance child，承接 9 个 mutation shared governance helper
- `src/runtime/mutation/ai_proposal/proposal_creation.rs` - runtime AI proposal proposal creation child，承接 `create_runtime_ai_proposal`
- `src/runtime/mutation/ai_proposal/approval_persistence.rs` - runtime AI proposal approval persistence child，承接 approval record disk read/write helper
- `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` - runtime AI proposal sandbox trigger child，承接 sandbox approve gate 与 background sandbox verification helper
- `src/runtime/mutation/ai_proposal/status_transition.rs` - runtime AI proposal status transition child，承接 approved projection、状态迁移矩阵和 scoped status side effect
- `src/runtime/mutation/ai_proposal/static_check.rs` - runtime AI proposal static check child，承接 hash/model/static check/config binding/v4 analysis helper 与静态检查单测
- `src/runtime/mutation/ai_proposal/source_governance_identity.rs` - runtime AI proposal source governance identity child，承接 source context、governance projection 与 record identity helper
- `src/runtime/mutation/ai_proposal/event_lifecycle.rs` - runtime AI proposal event lifecycle child，承接 event contract、runtime event builder、lifecycle entry 与 proposal transition persistence helper
- `src/runtime/mutation/ai_proposal/record_query.rs` - runtime AI proposal record query child，承接 proposal list/detail/read-through loader
- `src/runtime/mutation/ai_proposal/approval_review.rs` - runtime AI proposal approval review child，承接 approval list/detail/approve/reject/claim handler
- `src/runtime/report_ops/v1_report_endpoints.rs` - runtime report ops v1 report endpoint child，承接 ops/audit/research 三个 v1 report handler
- `src/runtime/report_ops/merge_generation_health.rs` - runtime report ops merge/generation/storage health child，承接三条 v1 support/health handler
- `src/runtime/mutation/parameter_mutation/record_query.rs` - runtime parameter mutation record query child，承接 list/detail read model handler
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` - transition lifecycle entry / persistence child
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` - transition lifecycle rollback id identity child

当前治理基线: `v4.15.0/` — 三矩阵完全接管，后续常态维护模块树、全量树和治理 gate。
当前架构规划: `v4.16.0/` — 面向十万行级重大工程，只启用模块化抽离控制；system 抽离经验已回填为后续抽离准则，S1-S10 closeout 或静态 closeout 已完成，`root.system` 顶层阶段性 closeout 已刷新，递归模块化流程已明确；backend 已进入 R5，BE-001B `src/backend/` 九叶模块壳已落位，BE-001C 九叶逐叶 closeout 已完成，BE-001D `backend.strategy_config` L3 模块壳已落位，BE-001E 其余八叶薄壳已落位且 `42-49` 已完成逐叶完成记录，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `v4_runtime_execution` 单叶 closeout，BE-001R-04 已完成 `legacy_dispatch` 单叶 closeout，BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`，BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`，BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断并设置父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout，BE-001AH-04 已完成 `boundary_safety` 单叶 closeout，BE-001AJ-04 已完成 `activation_flow` 单叶 closeout，BE-001AK-01 已完成 `transition_lifecycle` 第二轮父叶残余判断，BE-001AL-04 已完成 `rollback_flow` 单叶 closeout，BE-001AM-01 已完成 `transition_lifecycle` 第三轮父叶残余判断，BE-001AN-04 已完成 `activation_snapshot_side_effect` 单叶 closeout，下一步进入 BE-001AO-01 父叶残余判断，前端抽离和 E2E 整理延后，测试资产汰换登记已建立。
当前架构补充: BE-001AC-01 已确认 `runtime.backtest` 当前 handler 域设置 `stop_split: true`，drained parent include、compare/artifact schema/persistence/response mapping/frontend caller 仍保留原 owner。
当前最新递归点: 该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout；BE-001AN-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout 并设置 `stop_split: true`，下一步只能进入 BE-001AO-01 父叶残余判断。
当前最新递归点补充: BE-001AO-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第四轮父叶残余判断；父叶仍为 `stop_split: false`，下一步只能进入 BE-001AP-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线。
当前最新递归点补充: BE-001AP-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线；下一步只能进入 BE-001AP-02 抽离方案，目标文件尚未创建。
当前最新递归点补充: BE-001AP-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案；下一步只能进入 BE-001AP-03 实际抽离。
当前最新递归点补充: BE-001AP-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 实际抽离；下一步只能进入 BE-001AP-04 单叶 closeout。
当前最新递归点补充: BE-001AP-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AQ-01 父叶残余判断。
当前最新递归点补充: BE-001AQ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断；父叶仍为 `stop_split: false`，下一步只能进入 BE-001AR-01 `rollback_record_identity` 单子叶等价基线。
当前最新递归点补充: BE-001AR-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线；下一步只能进入 BE-001AR-02 抽离方案，目标文件尚未创建。
当前最新递归点补充: BE-001AR-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案；下一步只能进入 BE-001AR-03 实际抽离。
当前最新递归点补充: BE-001AR-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 实际抽离；下一步只能进入 BE-001AR-04 单叶 closeout。
当前最新递归点补充: BE-001AR-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AS-01 父叶残余判断。
当前最新递归点补充: BE-001AS-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
当前最新递归点补充: BE-001AT-01 已完成 `runtime.mutation.parameter_mutation` 父叶残余判断；父叶仍为 `stop_split: false`，下一步只能进入 BE-001AU-01 `proposal_creation` 单子叶等价基线。
当前最新递归点补充: BE-001AU-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线；下一步只能进入 BE-001AU-02 抽离方案，目标文件尚未创建。
当前最新递归点补充: BE-001AU-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 抽离方案；下一步只能进入 BE-001AU-03 实际抽离，目标文件尚未创建。
当前最新递归点补充: BE-001AU-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 实际抽离；下一步只能进入 BE-001AU-04 单叶 closeout。
当前最新递归点补充: BE-001AU-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AV-01 父叶残余判断。
当前最新递归点补充: BE-001AV-01 已完成 `runtime.mutation.parameter_mutation` 第二轮父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AW-01 `record_query` 单子叶等价基线。
当前最新递归点补充: BE-001AW-01 已建立 `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线；下一步只能进入 BE-001AW-02 抽离方案，目标文件尚未创建。
当前最新递归点补充: BE-001AW-02 已建立 `runtime.mutation.parameter_mutation.record_query` 抽离方案；当前仍为 `no code movement`，下一步只能进入 BE-001AW-03 实际抽离，目标文件尚未创建。
当前最新递归点补充: BE-001AW-03 已完成 `runtime.mutation.parameter_mutation.record_query` 实际抽离；list/detail handler 已迁入 child，下一步只能进入 BE-001AW-04 单叶 closeout。
当前最新递归点补充: BE-001AW-04 已完成 `runtime.mutation.parameter_mutation.record_query` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AX-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
当前最新递归点补充: BE-001AX-01 已完成 `runtime.mutation.parameter_mutation` 第三轮父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。
当前最新递归点补充: BE-001AY-01 已建立 `runtime.mutation.ai_proposal` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AY-02 抽离方案。
当前最新递归点补充: BE-001AY-02 已建立 `runtime.mutation.ai_proposal` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AY-03 实际抽离。
当前最新递归点补充: BE-001AY-03 已完成 `runtime.mutation.ai_proposal` 实际抽离；AI proposal / approval handlers 已迁入 child，下一步只能进入 BE-001AY-04 单叶 closeout。
当前最新递归点补充: BE-001AY-04 已完成 `runtime.mutation.ai_proposal` 单叶 closeout 并设置 `stop_split: false`；下一步只能进入 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。
当前最新递归点补充: BE-001AZ-01 已建立 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AZ-02 抽离方案。
当前最新递归点补充: BE-001AZ-02 已建立 `runtime.mutation.ai_proposal.static_check` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AZ-03 实际抽离。
当前最新递归点补充: BE-001AZ-03 已完成 `runtime.mutation.ai_proposal.static_check` 实际抽离；helper 与静态检查单测已迁入 child，下一步只能进入 BE-001AZ-04 单叶 closeout。
当前最新递归点补充: BE-001AZ-04 已完成 `runtime.mutation.ai_proposal.static_check` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BA-01 `runtime.mutation.ai_proposal` 父叶残余判断。
当前最新递归点补充: BE-001BA-01 已完成 `runtime.mutation.ai_proposal` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BB-01 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线。
当前最新递归点补充: BE-001BB-01 已建立 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BB-02 抽离方案。
当前最新递归点补充: BE-001BB-02 已建立 `runtime.mutation.ai_proposal.source_governance_identity` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BB-03 实际抽离。
当前最新递归点补充: BE-001BB-03 已完成 `runtime.mutation.ai_proposal.source_governance_identity` 实际抽离；source/governance/id helper 已迁入 child，下一步只能进入 BE-001BB-04 单叶 closeout。
当前最新递归点补充: BE-001BB-04 已完成 `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BC-01 `runtime.mutation.ai_proposal` 父叶残余判断。
当前最新递归点补充: BE-001BC-01 已完成 `runtime.mutation.ai_proposal` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线。
当前最新递归点补充: BE-001BD-01 已建立 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BD-02 抽离方案。
当前最新递归点补充: BE-001BD-02 已建立 `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BD-03 实际抽离。
当前最新递归点补充: BE-001BD-03 已完成 `runtime.mutation.ai_proposal.event_lifecycle` 实际抽离；event/lifecycle helper 已迁入 child，下一步只能进入 BE-001BD-04 单叶 closeout。
当前最新递归点补充: BE-001BD-04 已完成 `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BE-01 `runtime.mutation.ai_proposal` 父叶残余判断。
当前最新递归点补充: BE-001BE-01 已完成 `runtime.mutation.ai_proposal` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线。
当前最新递归点补充: BE-001BF-01 已建立 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BF-02 抽离方案。
当前最新递归点补充: BE-001BF-02 已建立 `runtime.mutation.ai_proposal.record_query` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BF-03 实际抽离。
当前最新递归点补充: BE-001BF-03 已完成 `runtime.mutation.ai_proposal.record_query` 实际抽离；list/detail/read-through loader 已迁入 child，下一步只能进入 BE-001BF-04 单叶 closeout。
当前最新递归点补充: BE-001BF-04 已完成 `runtime.mutation.ai_proposal.record_query` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BG-01 父叶残余判断。
当前最新递归点补充: BE-001BG-01 已完成 `runtime.mutation.ai_proposal` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BH-01 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线。
当前最新递归点补充: BE-001BH-01 已建立 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BH-02 抽离方案。
当前最新递归点补充: BE-001BH-02 已建立 `runtime.mutation.ai_proposal.approval_review` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BH-03 实际抽离。
当前最新递归点补充: BE-001BH-03 已完成 `runtime.mutation.ai_proposal.approval_review` 实际抽离；approval list/detail/approve/reject/claim 五个 handler 已迁入 child，下一步只能进入 BE-001BH-04 单叶 closeout。
当前最新递归点补充: BE-001BH-04 已完成 `runtime.mutation.ai_proposal.approval_review` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断。
当前最新递归点补充: BE-001BI-01 已完成 `runtime.mutation.ai_proposal` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线。
当前最新递归点补充: BE-001BJ-01 已建立 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BJ-02 抽离方案。
当前最新递归点补充: BE-001BJ-02 已建立 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BJ-03 实际抽离。
当前最新递归点补充: BE-001BJ-03 已完成 `runtime.mutation.ai_proposal.approval_persistence` 实际抽离；两个 persistence helper 已迁入 child，下一步只能进入 BE-001BJ-04 单叶 closeout。
当前最新递归点补充: BE-001BJ-04 已完成 `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BK-01 父叶残余判断。
当前最新递归点补充: BE-001BK-01 已完成 `runtime.mutation.ai_proposal` 第六轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线。
当前最新递归点补充: BE-001BL-01 已建立 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BL-02 抽离方案。
当前最新递归点补充: BE-001BL-02 已建立 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BL-03 实际抽离。
当前最新递归点补充: BE-001BL-03 已完成 `runtime.mutation.ai_proposal.sandbox_trigger` 实际抽离；下一步只能进入 BE-001BL-04 单叶 closeout。
当前最新递归点补充: BE-001BL-04 已完成 `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BM-01 父叶残余判断。
当前最新递归点补充: BE-001BM-01 已完成 `runtime.mutation.ai_proposal` 第七轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BN-01 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线。
当前最新递归点补充: BE-001BN-01 已建立 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BN-02 抽离方案。
当前最新递归点补充: BE-001BN-02 已建立 `runtime.mutation.ai_proposal.status_transition` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BN-03 实际抽离。
当前最新递归点补充: BE-001BN-03 已完成 `runtime.mutation.ai_proposal.status_transition` 实际抽离；三个状态 helper 已迁入 child，下一步只能进入 BE-001BN-04 单叶 closeout。
当前最新递归点补充: BE-001BN-04 已完成 `runtime.mutation.ai_proposal.status_transition` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BO-01 父叶残余判断。
当前最新递归点补充: BE-001BO-01 已完成 `runtime.mutation.ai_proposal` 第八轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线。
当前最新递归点补充: BE-001BP-01 已建立 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BP-02 抽离方案。
当前最新递归点补充: BE-001BP-02 已建立 `runtime.mutation.ai_proposal.proposal_creation` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BP-03 实际抽离。
当前最新递归点补充: BE-001BP-03 已完成 `runtime.mutation.ai_proposal.proposal_creation` 实际抽离；`create_runtime_ai_proposal` 已迁入 `src/runtime/mutation/ai_proposal/proposal_creation.rs`，下一步只能进入 BE-001BP-04 单叶 closeout。
当前最新递归点补充: BE-001BP-04 已完成 `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断。
当前最新递归点补充: BE-001BQ-01 已完成 `runtime.mutation.ai_proposal` 父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。
当前最新递归点补充: BE-001BR-01 已完成 `backend.runtime.routes` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线。
当前最新递归点补充: BE-001BS-01 已建立 `backend.runtime.routes.experiment` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BS-02 抽离方案。
当前最新递归点补充: BE-001BS-02 已建立 `backend.runtime.routes.experiment` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BS-03 实际抽离。
当前最新递归点补充: BE-001BS-03 已完成 `backend.runtime.routes.experiment` 实际抽离；`src/backend/runtime/routes/experiment.rs` 已创建，下一步只能进入 BE-001BS-04 单叶 closeout。
当前最新递归点补充: BE-001BS-04 已完成 `backend.runtime.routes.experiment` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BT-01 `backend.runtime.routes` 父叶残余判断。
当前最新递归点补充: BE-001BT-01 已完成 `backend.runtime.routes` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线。
当前最新递归点补充: BE-001BU-01 已建立 `backend.runtime.routes.evidence` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BU-02 抽离方案。
当前最新递归点补充: BE-001BU-02 已建立 `backend.runtime.routes.evidence` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BU-03 实际抽离。
当前最新递归点补充: BE-001BU-03 已完成 `backend.runtime.routes.evidence` 实际抽离；`src/backend/runtime/routes/evidence.rs` 已创建，下一步只能进入 BE-001BU-04 单叶 closeout。
当前最新递归点补充: BE-001BU-04 已完成 `backend.runtime.routes.evidence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BV-01 `backend.runtime.routes` 父叶残余判断。
当前最新递归点补充: BE-001BV-01 已完成 `backend.runtime.routes` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线。
当前最新递归点补充: BE-001BW-01 已建立 `backend.runtime.routes.event_stream` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BW-02 抽离方案。
当前最新递归点补充: BE-001BW-02 已建立 `backend.runtime.routes.event_stream` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BW-03 实际抽离。
当前最新递归点补充: BE-001BW-03 已完成 `backend.runtime.routes.event_stream` 实际抽离；`src/backend/runtime/routes/event_stream.rs` 已创建，下一步只能进入 BE-001BW-04 单叶 closeout。
当前最新递归点补充: BE-001BW-04 已完成 `backend.runtime.routes.event_stream` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断。
当前最新递归点补充: BE-001BX-01 已完成 `backend.runtime.routes` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BY-01 `backend.runtime.routes.report_ops` 单子叶等价基线。
当前最新递归点补充: BE-001BY-01 已建立 `backend.runtime.routes.report_ops` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BY-02 抽离方案。
当前最新递归点补充: BE-001BY-02 已建立 `backend.runtime.routes.report_ops` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BY-03 实际抽离。
当前最新递归点补充: BE-001BY-03 已完成 `backend.runtime.routes.report_ops` 实际抽离；下一步只能进入 BE-001BY-04 单叶 closeout。
当前最新递归点补充: BE-001BY-04 已完成 `backend.runtime.routes.report_ops` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BZ-01 父叶残余判断。
当前最新递归点补充: BE-001BZ-01 已完成 `backend.runtime.routes` 第六轮父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断。
当前最新递归点补充: BE-001CA-01 已完成 `backend.runtime` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001CB-01 `runtime.report_ops` 单子叶等价基线。
当前最新递归点补充: BE-001CB-01 已建立 `runtime.report_ops` 单子叶等价基线；下一步只能进入 BE-001CB-02 抽离方案。
当前最新递归点补充: BE-001CB-02 已建立 `runtime.report_ops` 抽离方案；下一步只能进入 BE-001CB-03 实际抽离。
当前最新递归点补充: BE-001CB-03 已完成 `runtime.report_ops` 实际抽离；下一步只能进入 BE-001CB-04 单叶 closeout。
当前最新递归点补充: BE-001CB-04 已完成 `runtime.report_ops` 单叶 closeout；该叶设置 `stop_split: false`，下一步只能进入 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线。
当前最新递归点补充: BE-001CC-01 已建立 `runtime.report_ops.runtime_report` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CC-02 抽离方案。
当前最新递归点补充: BE-001CC-02 已建立 `runtime.report_ops.runtime_report` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CC-03 实际抽离。
当前最新递归点补充: BE-001CC-03 已完成 `runtime.report_ops.runtime_report` 实际抽离；下一步只能进入 BE-001CC-04 单叶 closeout。
当前最新递归点补充: BE-001CC-04 已完成 `runtime.report_ops.runtime_report` 单叶 closeout 并设置 `stop_split: true`；父级 `runtime.report_ops` 仍为 `stop_split: false`，下一步只能进入 BE-001CD-01 父叶残余判断。
当前最新递归点补充: BE-001CD-01 已完成 `runtime.report_ops` 父叶残余判断；父级保持 `stop_split: false`，下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。
当前最新递归点补充: BE-001CE-01 已建立 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CE-02 抽离方案。
当前最新递归点补充: BE-001CE-02 已建立 `runtime.report_ops.v1_report_endpoints` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CE-03 endpoint smoke 补测。
当前最新递归点补充: BE-001CE-03 已完成 `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测；新增 `tests/api_v1_reports.rs` 覆盖三条 `/api/v1/reports/*` 基础 JSON contract，下一步只能进入 BE-001CE-04 实际抽离。
当前最新递归点补充: BE-001CE-04 已完成 `runtime.report_ops.v1_report_endpoints` 实际抽离；`src/runtime/report_ops/v1_report_endpoints.rs` 已创建并承接三个 v1 report handler，下一步只能进入 BE-001CE-05 单叶 closeout。
当前最新递归点补充: BE-001CE-05 已完成 `runtime.report_ops.v1_report_endpoints` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CF-01 `runtime.report_ops` 父叶残余判断。
当前最新递归点补充: BE-001CF-01 已完成 `runtime.report_ops` 父叶残余判断；父级仍保留 `list_merge_records`、`list_config_generations`、`get_storage_health`，因此 `stop_split: false`，下一步只能进入 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线。
当前最新递归点补充: BE-001CG-01 已建立 `runtime.report_ops.merge_generation_health` 单子叶等价基线；当前 `no code movement`，planned child 文件尚未创建，下一步只能进入 BE-001CG-02 抽离方案。
当前最新递归点补充: BE-001CG-02 已建立 `runtime.report_ops.merge_generation_health` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CG-03 endpoint smoke 补测。
当前最新递归点补充: BE-001CG-03 已完成 `runtime.report_ops.merge_generation_health` endpoint smoke 补测；新增 `tests/api_v1_ops_health.rs` 覆盖三条 v1 support/health endpoint 最小 JSON contract，下一步只能进入 BE-001CG-04 实际抽离。
当前最新递归点补充: BE-001CG-04 已完成 `runtime.report_ops.merge_generation_health` 实际抽离；新增 `src/runtime/report_ops/merge_generation_health.rs` 承接三条 v1 support/health handler，下一步只能进入 BE-001CG-05 单叶 closeout。
当前最新递归点补充: BE-001CG-05 已完成 `runtime.report_ops.merge_generation_health` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。
当前最新递归点补充: BE-001CH-01 已完成 `runtime.report_ops` 第二轮父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断。
当前最新递归点补充: BE-001CI-01 已完成 `backend.runtime` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
当前最新递归点补充: BE-001CJ-01 已建立 `runtime.evidence_health` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CJ-02 抽离方案。
当前最新递归点补充: BE-001CJ-02 已建立 `runtime.evidence_health` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CJ-03 实际抽离。
当前最新递归点补充: BE-001CJ-03 已完成 `runtime.evidence_health` 实际抽离；下一步只能进入 BE-001CJ-04 单叶 closeout。
当前最新递归点补充: BE-001CJ-04 已完成 `runtime.evidence_health` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。
当前最新递归点补充: BE-001CK-01 已完成 `backend.runtime` 第三轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。
当前最新递归点补充: BE-001CL-01 已建立 `runtime.mutation.shared_governance` 单子叶等价基线；下一步只能进入 BE-001CL-02 抽离方案。
当前最新递归点补充: BE-001CL-02 已建立 `runtime.mutation.shared_governance` 抽离方案；下一步只能进入 BE-001CL-03 实际抽离。
当前最新递归点补充: BE-001CL-03 已完成 `runtime.mutation.shared_governance` 实际抽离；下一步只能进入 BE-001CL-04 单叶 closeout。
当前最新递归点补充: BE-001CL-04 已完成 `runtime.mutation.shared_governance` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CM-01 `backend.runtime` 第四轮父叶残余判断。
当前最新递归点补充: BE-001CM-01 已完成 `backend.runtime` 第四轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CN-01 `runtime.query_support` 单子叶等价基线。
当前最新递归点补充: BE-001CN-01 已建立 `runtime.query_support` 单子叶等价基线；下一步只能进入 BE-001CN-02 抽离方案。
当前最新递归点补充: BE-001CN-02 已建立 `runtime.query_support` 抽离方案；下一步只能进入 BE-001CN-03 实际抽离。
当前最新递归点补充: BE-001CN-03 已完成 `runtime.query_support` 实际抽离；下一步只能进入 BE-001CN-04 单叶 closeout。
当前最新递归点补充: BE-001CN-04 已完成 `runtime.query_support` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。
当前最新递归点补充: BE-001CO-01 已完成 `backend.runtime` 第五轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CP-01 `runtime.response_support` 单子叶等价基线。
当前最新递归点补充: BE-001CP-01 已建立 `runtime.response_support` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CP-02 抽离方案。
当前最新递归点补充: BE-001CP-02 已建立 `runtime.response_support` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CP-03 实际抽离。
当前最新递归点补充: BE-001CP-03 已完成 `runtime.response_support` 实际抽离；`src/runtime/response_support.rs` 已创建，下一步只能进入 BE-001CP-04 单叶 closeout。
当前最新递归点补充: BE-001CP-04 已完成 `runtime.response_support` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。
当前最新递归点补充: BE-001CQ-01 已完成 `backend.runtime` 第六轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CR-01 `runtime.run_guard` 单子叶等价基线。
当前最新递归点补充: BE-001CR-01 已建立 `runtime.run_guard` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CR-02 抽离方案。
当前最新递归点补充: BE-001CR-02 已建立 `runtime.run_guard` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CR-03 实际抽离。
当前最新递归点补充: BE-001CR-03 已完成 `runtime.run_guard` 实际抽离；下一步只能进入 BE-001CR-04 单叶 closeout。
当前最新递归点补充: BE-001CR-04 已完成 `runtime.run_guard` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。
当前最新递归点补充: BE-001CS-01 已完成 `backend.runtime` 第七轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线。
当前最新递归点补充: BE-001CT-01 已建立 `runtime.experiment_limit` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CT-02 抽离方案。
当前最新递归点补充: BE-001CT-02 已建立 `runtime.experiment_limit` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CT-03 endpoint smoke 补测。
当前最新递归点补充: BE-001CT-03 已完成 `runtime.experiment_limit` endpoint smoke 补测；下一步只能进入 BE-001CT-04 实际抽离。
当前最新递归点补充: BE-001CT-04 已完成 `runtime.experiment_limit` 实际抽离；下一步只能进入 BE-001CT-05 单叶 closeout。
当前最新递归点补充: BE-001CT-05 已完成 `runtime.experiment_limit` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断。
当前最新递归点补充: BE-001CU-01 已完成 `backend.runtime` 第八轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线。
当前最新递归点补充: BE-001CV-01 已建立 `runtime.parent_include_cleanup` 单子叶等价基线；下一步只能进入 BE-001CV-02 抽离方案。
当前最新递归点补充: BE-001CV-02 已建立 `runtime.parent_include_cleanup` 抽离方案；下一步只能进入 BE-001CV-03 实际 cleanup。
当前最新递归点补充: BE-001CV-03 已完成 `runtime.parent_include_cleanup` 实际 cleanup；下一步只能进入 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断。
当前最新递归点补充: BE-001CW-01 已完成 `backend.runtime` 第九轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线。
当前最新递归点补充: BE-001CX-01 已建立 `runtime.parent_import_bridge` 单子叶等价基线；下一步只能进入 BE-001CX-02 抽离方案，不能直接批量改写 Rust import。
当前最新递归点补充: BE-001CX-02 已建立 `runtime.parent_import_bridge` 抽离方案；下一步只能进入 BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离。
当前最新递归点补充: BE-001CX-03 已完成 `runtime.root_support_import_pilot` 实际抽离；下一步只能进入 BE-001CX-04 单叶 closeout。
当前最新递归点补充: BE-001CX-04 已完成 `runtime.root_support_import_pilot` 单叶 closeout；下一步只能进入 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001CY-01 已建立 `runtime.root_entry_import_pass` 单子叶等价基线；下一步只能进入 BE-001CY-02 抽离方案。
当前最新递归点补充: BE-001CY-02 已建立 `runtime.root_entry_import_pass` 抽离方案；下一步只能进入 BE-001CY-03 实际抽离，且只处理 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs`。
当前最新递归点补充: BE-001CY-03 已完成 `runtime.root_entry_import_pass` 实际抽离；下一步只能进入 BE-001CY-04 单叶 closeout。
当前最新递归点补充: BE-001CY-04 已完成 `runtime.root_entry_import_pass` 单叶 closeout；下一步只能进入 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001CZ-01 已建立 `runtime.report_ops_import_pass` 单子叶等价基线；下一步只能进入 BE-001CZ-02 抽离方案。
当前最新递归点补充: BE-001CZ-02 已建立 `runtime.report_ops_import_pass` 抽离方案；下一步只能进入 BE-001CZ-03 实际抽离。
当前最新递归点补充: BE-001CZ-03 已完成 `runtime.report_ops_import_pass` 实际抽离；下一步只能进入 BE-001CZ-04 单叶 closeout。
当前最新递归点补充: BE-001CZ-04 已完成 `runtime.report_ops_import_pass` 单叶 closeout；下一步只能进入 BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断。
当前最新递归点补充: BE-001DA-01 已完成 `runtime.parent_import_bridge` 父叶残余判断；下一步只能进入 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DB-01 已建立 `runtime.run_import_pass` 单子叶等价基线；下一步只能进入 BE-001DB-02 抽离方案。
当前最新递归点补充: BE-001DB-02 已建立 `runtime.run_import_pass` 抽离方案；下一步只能进入 BE-001DB-03 实际抽离。
当前最新递归点补充: BE-001DB-03 已完成 `runtime.run_import_pass` 实际抽离；下一步只能进入 BE-001DB-04 单叶 closeout。
当前最新递归点补充: BE-001DB-04 已完成 `runtime.run_import_pass` 单叶 closeout；下一步只能进入 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断。
当前最新递归点补充: BE-001DC-01 已完成 `runtime.parent_import_bridge` 父叶残余判断；下一步只能进入 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DD-01 已建立 `runtime.backtest_import_pass` 单子叶等价基线；下一步只能进入 BE-001DD-02 抽离方案。
当前最新递归点补充: BE-001DD-02 已建立 `runtime.backtest_import_pass` 抽离方案；下一步只能进入 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DE-01 已建立 `runtime.backtest.record_store_import_pass` 单子叶等价基线；下一步只能进入 BE-001DE-02 抽离方案。
当前最新递归点补充: BE-001DE-02 已建立 `runtime.backtest.record_store_import_pass` 抽离方案；下一步只能进入 BE-001DE-03 实际抽离。
当前最新递归点补充: BE-001DE-03 已完成 `runtime.backtest.record_store_import_pass` 实际抽离；下一步只能进入 BE-001DE-04 单叶 closeout。
当前最新递归点补充: BE-001DE-04 已完成 `runtime.backtest.record_store_import_pass` 单叶 closeout；下一步只能进入 BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DF-01 已完成 `runtime.backtest_import_pass` 父叶残余判断；下一步只能进入 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DG-01 已建立 `runtime.backtest.replay_import_pass` 单子叶等价基线；下一步只能进入 BE-001DG-02 抽离方案。
当前最新递归点补充: BE-001DG-02 已建立 `runtime.backtest.replay_import_pass` 抽离方案；下一步只能进入 BE-001DG-03 实际抽离。
当前最新递归点补充: BE-001DG-03 已完成 `runtime.backtest.replay_import_pass` 实际抽离；下一步只能进入 BE-001DG-04 单叶 closeout。
当前最新递归点补充: BE-001DG-04 已完成 `runtime.backtest.replay_import_pass` 单叶 closeout；下一步只能进入 BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DH-01 已完成 `runtime.backtest_import_pass` 第二轮父叶残余判断；下一步只能进入 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DI-01 已建立 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线；下一步只能进入 BE-001DI-02 抽离方案。
当前最新递归点补充: BE-001DI-02 已建立 `runtime.backtest.experiment_sweep_import_pass` 抽离方案；下一步只能进入 BE-001DI-03 实际抽离。
当前最新递归点补充: BE-001DI-03 已完成 `runtime.backtest.experiment_sweep_import_pass` 实际抽离；下一步只能进入 BE-001DI-04 单叶 closeout。
当前最新递归点补充: BE-001DI-04 已完成 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout；旧的三叶暂停目标取消，下一步只能进入 BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DJ-01 已完成 `runtime.backtest_import_pass` 第三轮父叶残余判断；下一步只能进入 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DK-01 已建立 `runtime.backtest.execution_start_import_pass` 单子叶等价基线；下一步只能进入 BE-001DK-02 抽离方案。
当前最新递归点补充: BE-001DK-02 已建立 `runtime.backtest.execution_start_import_pass` 抽离方案；下一步只能进入 BE-001DK-03 实际抽离。
当前最新递归点补充: BE-001DK-03 已完成 `runtime.backtest.execution_start_import_pass` 实际抽离；下一步只能进入 BE-001DK-04 单叶 closeout。
当前最新递归点补充: BE-001DK-04 已完成 `runtime.backtest.execution_start_import_pass` 单叶 closeout；旧的三叶暂停目标保持取消，下一步只能进入 BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DL-01 已完成 `runtime.backtest_import_pass` 第四轮父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断。
当前最新递归点补充: BE-001DM-01 已完成 `runtime.parent_import_bridge` 父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DN-01 已建立 `runtime.mutation_import_pass` 单子叶等价基线；下一步只能进入 BE-001DN-02 抽离方案。
当前最新递归点补充: BE-001DN-02 已建立 `runtime.mutation_import_pass` 抽离方案；下一步只能进入 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DO-01 已建立 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线；下一步只能进入 BE-001DO-02 抽离方案。
当前最新递归点补充: BE-001DO-02 已建立 `runtime.mutation.shared_governance_import_pass` 抽离方案；下一步只能进入 BE-001DO-03 实际抽离记录。
当前最新递归点补充: BE-001DO-03 已完成 `runtime.mutation.shared_governance_import_pass` 实际抽离；下一步只能进入 BE-001DO-04 单叶 closeout。
当前最新递归点补充: BE-001DO-04 已完成 `runtime.mutation.shared_governance_import_pass` 单叶 closeout；下一步只能进入 BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DP-01 已完成 `runtime.mutation_import_pass` 父叶残余判断；下一步只能进入 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DQ-01 已建立 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线；下一步只能进入 BE-001DQ-02 抽离方案。
当前最新递归点补充: BE-001DQ-02 已建立 `runtime.mutation.parameter_mutation_import_pass` 抽离方案；下一步只能进入 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DR-01 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线；下一步只能进入 BE-001DR-02 抽离方案。
当前最新递归点补充: BE-001DR-02 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案；下一步只能进入 BE-001DR-03 实际抽离记录。
当前最新递归点补充: BE-001DR-03 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 实际抽离；下一步只能进入 BE-001DR-04 单叶 closeout。
当前最新递归点补充: BE-001DR-04 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout；设置 `stop_split: true`，旧三叶暂停目标保持取消，下一步只能进入 BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DS-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DT-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线；下一步只能进入 BE-001DT-02 抽离方案。
当前最新递归点补充: BE-001DT-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案；下一步只能进入 BE-001DT-03 实际抽离记录。
当前最新递归点补充: BE-001DT-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 实际抽离；下一步只能进入 BE-001DT-04 单叶 closeout。
当前最新递归点补充: BE-001DT-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout；设置 `stop_split: true`，旧三叶暂停目标保持取消，下一步只能进入 BE-001DU-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
当前最新递归点补充: BE-001DU-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DV-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线；下一步只能进入 BE-001DV-02 抽离方案。
当前最新递归点补充: BE-001DV-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案；拒绝 7 文件同批 rewrite，下一步只能进入 BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线。
当前最新递归点补充: BE-001DW-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线；下一步只能进入 BE-001DW-02 抽离方案。
当前最新递归点补充: BE-001DW-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案；下一步只能进入 BE-001DW-03 单文件 import rewrite，旧三叶暂停目标保持取消。
当前最新递归点补充: BE-001DW-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 实际抽离；下一步只能进入 BE-001DW-04 单叶 closeout。
当前最新递归点补充: BE-001DW-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001DX-01 父叶残余判断。
当前最新递归点补充: BE-001DX-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断；下一步只能进入 BE-001DY-01 `rollback_record_identity_import_pass` 单子叶等价基线。

### 7.7 总览 (markdown/10-overview/)

```
overview-system-architecture.md              — 系统架构总览
overview-current-status-and-roadmap.md       — 当前状态与路线图
overview-full-feature-tree.md                — 本文档 (全量树)
```

### 7.8 研究层 (markdown/08-research/)

```
deep-research-report.md                      — 深度研究报告
research-backtest-artifact-protocol.md       — 回测工件协议研究
research-frontend-backend-e2e-plan.md        — 前后端 E2E 计划
research-quantscript-typed-hir-diagnostics.md — QS 类型化 HIR 诊断
research-spread-custom-plugin-sequencing.md  — Spread 自定义插件排序
```

### 7.9 归档层 (markdown/09-archive/)

退役的实现文档、规划文档、追踪清单的历史归档。

---

## 附录 A: 项目数据流全景

```
用户操作 (React 前端 :5173)
  │  拖拽节点、连线、配置参数、编写 QS
  ▼
策略图 (graph JSON) / QuantScript 源码
  │  POST /api/runtime/compile
  ▼
编译管道 (backend.graph_compile.compile → quantscript)
  │  graph → QS 源码 → parse → HIR → lower → Core IR
  ▼
运行时 (runtime/mod.rs → qrpc_runtime)
  ├── Paper 运行 → 事件流 → SSE 推送 → EventStreamPanel
  ├── 回测 → 历史数据回放 → 12 项指标 → BacktestDetailPage
  └── 执行端 (:3001) → OKX 模拟盘 → provider 回执 / 实时模拟成交
       │
       └── SSE /api/executor/events → 后端 → 前端监控面板
```

## 附录 B: 存储目录结构

```
storage/
  ├── graphs/         (Permanent)  策略图和 QS 源码
  ├── runs/           (Temporary)  Paper 运行记录
  ├── backtests/      (Temporary)  回测工件
  ├── experiments/    (Temporary)  实验记录
  ├── snapshots/      (Transient)  快照
  ├── alerts/         (Transient)  告警
  ├── chaos/          (Transient)  混沌实验报告
  ├── audit/          (Permanent)  审计日志
  └── .credentials    (Permanent)  AES-256-GCM 加密凭证
```

[GP §7.1-§7.5]: 存储生命周期 — 三级分类 (Permanent/Temporary/Transient), 500MB 配额

## 附录 C: GP 约束快速索引

| GP 条款 | 主题 | 约束对象 |
|---------|------|---------|
| §1.1 | QS 唯一策略定义路径 | 编译系统 (根2.1) |
| §1.2 | 跨三层验证 | 编译 + Core IR + 运行时 (根2.1, 根4.1, 根4.4) |
| §1.3 | 编译路径不可绕过 | 编译系统 (根2.1) |
| §1.4 | 数据流单向 | 图存储 (根2.2) |
| §1.5 | 功能演进先登记 | 全项目 |
| §1.6 | 顶层 DAG + 状态机边界 | v4 runtime (根4.4) |
| §1.7 | QS 状态机 DSL 边界 | v4 静态审计 (根4.6) |
| §1.8 | 事件驱动迁移 | v4 runtime (根4.4) |
| §1.9 | Risk Plane 不可绕过 | v4 runtime (根4.4) |
| §1.10 | Execution 能力来源 | VenueCapabilityMatrix (根4.1) |
| §1.11 | 学习流水线边界 | 治理文档 (根7.3) |
| §1.12 | 前端以后端 capability 为真源 | 能力投影层 (根5.8) |
| §2.6 | 凭证保险库安全 | 安全系统 (根2.5, 根3.3) |
| §2.7 | 实时执行安全 | 执行端 (根3) |
| §2.8 | 本地会话认证安全 | 安全系统 (根2.5) |
| §5.5 | 端到端验证 | 工具链 (根6) |
| §5.6 | 禁止格式漂移 | 工具链 (根6) |
| §7.1-7.5 | 存储生命周期 | 存储系统 |
| §8.9-8.11 | 前端设计规范 | 前端 (根5) |
| §9.1-9.4 | 治理系统 | 运行时 (根2.3) |
| §10.1-10.5 | 功能覆盖 + 回归保护 | 全项目 |

## 附录 D: 超级规范化约束快速索引

| 超级规范化章节 | 主题 | 约束对象 |
|--------------|------|---------|
| 第二章 | 三层门禁流水线 | 工具链 (根6) |
| 第三章 | AI 并行审计 | 质量流程 |
| 第四章 | 发布前检查单 | Release 流程 |
| 第五章 | Closeout 审计 (五维度评分 + GP 矩阵) | 里程碑流程 |
| 第六章 | 优化流水线 | 持续改进 |
| 第七章 | 元流水线 (自进化) | 流程自身 |
| §7.7 | MAJOR 演化通道 (8 Phase) | MAJOR 版本 |
| §7.8 | 前端后端能力真源通道 | 能力系统 |
| §8.1 | 阻断规则 | 全项目 |
| §8.5 | 自由维度诱错审计常态化 | 质量流程 |
| §8.8 | 功能演进防回退 | 功能演进 |
| §8.9 | v4 状态机化演化防偏规则 | v4 实现 |

---

## 附录 E: 全文件覆盖清单

> 每个 active 源文件至少一行说明。按目录分组。
> 由 `tools/check-full-feature-tree.ps1` 自动校验覆盖率。

### E.1 根目录配置

- `Cargo.toml` — Rust workspace 定义, 7 个 crate + 1 个 binary; 改依赖/版本时改这里
- `start.bat` — Windows 一键启动脚本 (编译+后端+Tauri); 改启动流程时改这里
- `start.ps1` — PowerShell 启动脚本
- `Dockerfile` — Docker 镜像构建; 改容器化部署时改这里
- `docker-compose.yml` — Docker 编排
- `nginx.conf` — Nginx 反向代理配置
- `.env.example` — 环境变量模板; 新增环境变量时改这里
- `.gitignore` — Git 忽略规则
- `.gitattributes` — Git 属性

### E.2 后端: `src/`

- `src/main.rs` — 二进制入口, 调用 `quantpilot::run_server()`
- `src/lib.rs` — 核心库入口, 全部模块声明 + `run_server` 兼容 re-export; 启动实现已归入 system 模块
- `src/system/mod.rs` — system 父模块入口; 改 system 顶层模块导出时改这里
- `src/system/entry/mod.rs` — system.entry 二级域入口; 改启动域子模块导出时改这里
- `src/system/entry/backend_process.rs` — 后端进程启动 public 入口, `run_server()`、`run_api_server()`、CLI 分发、环境和日志初始化、启动期中间件和后台任务; 改进程启动边界时改这里
- `src/app_router.rs` — 路由构建, `build_app_router()` 定义全部 HTTP 端点; 新增 API 时改这里
- `src/alert_engine.rs` — 告警引擎, 10 条默认规则; 改告警规则/去重/恢复/404 语义时改这里
- `src/api_errors.rs` — API 错误格式, `json_bad_request()`/`json_not_found()`/`internal_error()`; 改错误响应格式或 error_code 时改这里
- `src/api_test_scenario.rs` — 测试场景 API; 新增自动化测试场景时改这里
- `src/app_runtime_helpers.rs` — 应用状态工厂, `new_app_state()`; 改存储路径/应用初始化时改这里
- `src/auth/mod.rs` — 本地会话认证 (注册/登录/刷新/JWT/bcrypt); 改本地会话边界时改这里, 不扩展为完整账户系统
- `src/auth_middleware.rs` — 认证中间件, JWT 验证; 改认证拦截时改这里
- `src/backtest_artifacts.rs` — 回测工件管理; 改回测工件格式/存储或 v4 artifact 持久化时改这里 🆕 v4.3.0
- `src/backtest_compare.rs` — 回测对比入口 API; 改对比功能时改这里
- `src/backtest_compare_core.rs` — 回测对比核心; 改对比算法时改这里
- `src/backtest_compare_narrative.rs` — 回测对比中文叙述生成; 改分析文案时改这里
- `src/backtest_compare_types.rs` — 回测对比类型定义; 改对比数据结构时改这里
- `src/backup.rs` — 数据备份与恢复; 改备份策略时改这里
- `src/backend/capability/snapshot.rs` — 能力声明 API (`GET /api/capabilities`); 新增能力声明时改这里
- `src/capability_api.rs` — root capability API compatibility shim; real implementation lives in `src/backend/capability/snapshot.rs`
- `src/chaos_experiment.rs` — 混沌实验; 改混沌测试定义/执行时改这里
- `src/cli_support.rs` — CLI 辅助; 改命令行参数或 `v4-run` 时改这里 🆕 v4.1.0
- `src/collaboration.rs` — 协作功能; 改多人协作时改这里
- `src/backend/graph_compile/compile.rs` — 编译入口, `/api/runtime/compile`; 改编译流程或 v4 诊断码映射时改这里
- `src/compile_api.rs` — root compile API compatibility marker; real implementation lives in `src/backend/graph_compile/compile.rs`
- `src/compile_artifact_builders.rs` — 编译产物组装; 改策略包/迁移包结构时改这里
- `src/compile_diagnostics.rs` — 编译诊断; 改编译错误/警告格式时改这里
- `src/backend/storage_security/credential_api_handler_implementation.rs` — 凭证管理 API handler implementation (set/list/delete); 改凭证 CRUD 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs` — 凭证管理 API delete mutation child; 改 DELETE /api/credentials/:service 的 validation/delete/audit/response 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/delete_commit.rs` — 凭证管理 API delete mutation commit child; 改 DELETE /api/credentials/:service 的 vault delete/error/audit/success response 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/service_path_validation.rs` — 凭证管理 API delete mutation service path validation child; 改 DELETE service label 验证时改这里
- `src/backend/storage_security/credential_api_handler_implementation/key_scope.rs` — 凭证管理 API shared key-scope child; 改 `{user_id}:{service}` credential key format 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/list_projection.rs` — 凭证管理 API list projection child; 改 GET /api/credentials 的 scoped list projection 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs` — 凭证管理 API set mutation child; 改 POST /api/credentials 的 validation/storage/audit/response 时改这里
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/service_and_fields_validation.rs` — 凭证管理 API set mutation validation child; 改 POST /api/credentials 的 service/fields 输入验证时改这里
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/storage_commit.rs` — 凭证管理 API set mutation storage commit child; 改 POST /api/credentials 的 vault set/audit/success response 时改这里
- `src/backend/storage_security/credential_vault/implementation.rs` — 凭证保险库实现 parent owner, 保留 public API facade、secret pattern extraction 和 type/tests; 改 public surface 或 parent-owned type 时改这里
- `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs` — credential vault AES-GCM codec child; 改 nonce/tag、version framing、AAD、encrypt/decrypt 分支时改这里
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs` — credential vault machine-key cache/init and key derivation child; 改 machine key 文件、cache、PBKDF2/SHA-256 派生时改这里
- `src/backend/storage_security/credential_vault/implementation/secret_pattern_extraction.rs` — credential vault safe-log pattern extraction child; 改 `extract_secret_patterns` traversal、Zeroizing clone wrapping 或 `len() >= 4` threshold 时改这里
- `src/backend/storage_security/credential_vault/implementation/type_surface.rs` — credential vault shared type/public facade surface child; 改 `SecretString` serde/drop zeroize、`VaultData.entries` shape、`CredentialFields` alias、`CredentialVault` field layout 或 `storage_root` fallback 时改这里
- `src/backend/storage_security/credential_vault/implementation/tests.rs` — credential vault implementation-local test harness child; 改 vault unit test setup、temp storage fixture、serialized test guard 或 implementation-local assertions 时改这里
- `src/backend/storage_security/credential_vault/implementation/service_crud/mod.rs` — credential vault service CRUD parent child; 改 parent mediation、get/list read projection 或 CRUD facade helper 时改这里
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_mutation_commit.rs` — credential vault service mutation child; 改 set/delete、empty-field validation、missing delete error 或 mutation save handoff 时改这里
- `src/backend/storage_security/credential_vault/implementation/service_crud/service_read_projection.rs` — credential vault service read projection child; 改 get/list、missing read、Zeroizing clone wrapping 或 service key listing 时改这里
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs` — credential vault persistence/restore parent child; 改 load/save 父级委托时改这里
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/atomic_save_commit.rs` — credential vault atomic save commit child; 改 save tmp/bak rollback、fsync、backup cleanup 或 Unix/Windows permission hardening 时改这里
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/load_restore_entry.rs` — credential vault load/restore entry child; 改 storage-root load、`.bak` restore、existing encrypted read/decode、fresh vault creation 或 initial encrypted write 时改这里
- `src/credential_vault.rs` — credential vault root compatibility shim; real implementation lives in `src/backend/storage_security/credential_vault/implementation.rs`
- `src/error_codes.rs` — 全局错误码注册表; 新增诊断码或 API error_code 时改这里
- `src/formal_quantscript_authoring_types.rs` — QS 正式编写类型; 改 QS 编写 API 类型时改这里
- `src/frontend_api_types.rs` — 前端 API 类型定义; 改前后端接口类型时改这里
- `src/frontend_runtime_mapping.rs` — 前端运行时映射; 改后端→前端数据映射时改这里
- `src/backend/graph_compile/graph.rs` — 图 CRUD API (save/load/list/delete/versions); 改图存储 API 时改这里
- `src/graph_api.rs` — root graph API compatibility shim for tests; real implementation lives in `src/backend/graph_compile/graph.rs`
- `src/backend/graph_compile/quantscript_graph.rs` — QS graph route/parser/artifact 父叶, `generate_quantscript_from_graph_value()` 由 child re-export
- `src/backend/graph_compile/quantscript_graph/artifact_target_projection.rs` — QS graph artifact and runtime target projection child; parent mediates graph-to-QS generator reuse
- `src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs` — graph JSON → QS 源码, `generate_quantscript_from_graph_value()`; 改图→QS 转换时改这里
- `src/backend/graph_compile/quantscript_graph/route_surface.rs` — QS graph route facade child, owns load/parse HTTP handlers
- `src/backend/graph_compile/quantscript_graph/strategy_graph_parser.rs` — strategy_graph source parser child, owns source-to-imported-graph parsing before artifact attachment
- `src/graph_version_compare.rs` — 图版本对比; 改版本 diff 算法、配置契约 diff 或 evidence diff 响应挂接时改这里
- `src/backend/ops_governance/hotswap/handlers.rs` — 模块热替换 API; 改热替换接口时改这里
- `src/middleware.rs` — 通用中间件; 改请求处理管道时改这里
- `src/migration_sender.rs` — 数据迁移; 改跨版本迁移时改这里
- `src/rate_limiter.rs` — 速率限制; 改限速策略时改这里
- `src/runbook.rs` — 运行手册; 改运维操作定义时改这里
- `src/runtime/mod.rs` — 运行时主模块, Paper 运行/v4 run 路由与 `event_stream` / `run_v4_handoff` 等父级 re-export; 改运行时 API 聚合时改这里 🆕 v4.1.0
- `src/runtime/query_support.rs` — runtime Query DTO、filter normalization 与 replay option normalization child; 改 runtime query parsing 或 replay option mapping 时改这里 🆕 v4.16.0
- `src/runtime/response_support.rs` — runtime response DTO child; 改 discard response 或 merge records response DTO 时改这里 🆕 v4.16.0
- `src/runtime/run_guard.rs` — runtime run-in-progress RAII guard child; 改 run 并发复位语义或 guard Drop reset 时改这里 🆕 v4.16.0
- `src/runtime/experiment_limit.rs` — runtime experiment variant limit child; 改 experiment sweep 变体上限时改这里 🆕 v4.16.0
- `src/runtime/evidence_health.rs` — runtime evidence health / cleanup handler 与 report status counts helper child; 改 evidence health API 等价时改这里 🆕 v4.16.0
- `src/runtime/report_ops.rs` — runtime report / v1 ops report handler child; 改 report ops handler 等价时改这里 🆕 v4.16.0
- `src/runtime/report_ops/runtime_report.rs` — runtime report create/list/detail/export handler 与 materialization helper child; 改 runtime report API 等价时改这里 🆕 v4.16.0
- `src/runtime/event_stream.rs` — run event stream SSE handler、frame order、delay 和 keep-alive; 改运行事件流 SSE 时改这里 🆕 v4.16.0
- `src/runtime/run/session_start.rs` — legacy `POST /api/runtime/test-run` handler、capability guard 调用、QS compile、sandbox session、event envelope 和 in-memory run record 写入; 改 session start 时改这里 🆕 v4.16.0
- `src/runtime/run/v4_handoff.rs` — `POST /api/runtime/v4/run` handler、request/response、graph resolution、handoff projection 和 simulated capability matrix; 改 v4 handoff run 时改这里 🆕 v4.16.0
- `src/runtime/run/record_store.rs` — run record list/detail/save/discard handler、manifest save/discard 编排和 graph audit 调用; 改 run record API handler 时改这里 🆕 v4.16.0
- `src/runtime/run/replay_status.rs` — run replay/status handler、replay cursor/options 编排、status projection 调用和 replay metrics 触发; 改 run replay/status API handler 时改这里 🆕 v4.16.0
- `src/runtime/backtest/execution_start.rs` — backtest 创建路径 handler、legacy/v4 execution helper 和 transient record 写入; 改 backtest start 执行入口时改这里 🆕 v4.16.0
- `src/runtime/backtest/legacy_dispatch.rs` — legacy backtest compile/assumption/artifact/sandbox replay 父级私有 helper; 改 legacy non-v4 backtest dispatch 时改这里 🆕 v4.16.0
- `src/runtime/backtest/record_store.rs` — backtest record list/detail/save/discard handler; 改 backtest record API handler 时改这里 🆕 v4.16.0
- `src/runtime/backtest/v4_projection.rs` — v4 backtest artifact projection helper 与单元测试; 改 `V4BacktestArtifact -> BacktestOutput / FrontendRuntimeEvent` 投影时改这里 🆕 v4.16.0
- `src/runtime/backtest/v4_request_resolution.rs` — v4 backtest request detection、graph resolution、symbol resolution 和 event type resolution helper; 改 v4 backtest 请求解析时改这里 🆕 v4.16.0
- `src/runtime/backtest/v4_runtime_execution.rs` — v4 backtest deterministic bars/ticks、blocking runtime replay 和 `V4BacktestArtifact` 输出 helper; 改 v4 backtest runtime execution 时改这里 🆕 v4.16.0
- `src/runtime/mutation/parameter_mutation.rs` — 运行时参数变更 create/list/detail 与 transition child re-export; 改 runtime parameter mutation proposal record 时改这里 🆕 v4.16.0
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` — 运行时参数变更 activation / rollback lifecycle、boundary/safe window、transition persistence 和 activation snapshot side effect; 改 runtime parameter mutation transition 时改这里 🆕 v4.16.0
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` — 运行时参数变更 activation public handler、activation 状态机、run event append 和 activation metrics; 改 runtime parameter mutation activation flow 时改这里 🆕 v4.16.0
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` — 运行时参数变更 boundary validation / resolution 与 safe-window evaluation; 改 runtime parameter mutation boundary safety 时改这里 🆕 v4.16.0
- `src/runtime/mutation/shared_governance.rs` — 运行时 mutation shared governance helper child; 改 parameter mutation / AI proposal 共享 target validation、event contract 或 governance projection 时改这里 🆕 v4.16.0
- `src/runtime_diagnostics.rs` — 运行时诊断; 改健康检查/性能诊断时改这里
- `src/runtime_event_projection.rs` — v4 运行时事件投影; 改 v4 事件→前端映射时改这里
- `src/runtime_persistence.rs` — 运行时持久化; 改运行记录/回测工件读写时改这里
- `src/runtime_response_mapping.rs` — 运行时响应映射; 改后端→前端响应格式时改这里
- `src/runtime_validation.rs` — 运行时验证; 改参数校验时改这里
- `src/safe_log.rs` — 安全日志, 输出前清除 secret/key; 改日志脱敏时改这里
- `src/sandbox_verification.rs` — 沙箱验证兼容桥; 保持 runtime mutation 既有 sandbox runner 和 disk loader 调用
- `src/backend/ops_governance/sandbox/handlers.rs` — 沙箱验证, AI 提案回放, v4 artifact replay-shape 对比, 提供 proposal sandbox report 读取给审批阻断; 改验证逻辑或 CandidateUnderperforms 判定时改这里
- `src/backend/ops_governance/sandbox/comparison_metrics.rs` — sandbox backtest comparison metrics and v4 replay-shape helper child
- `src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs` — sandbox backtest projection child
- `src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs` — sandbox v4 replay-shape comparison child
- `src/backend/ops_governance/sandbox/metrics_evaluation.rs` — sandbox metric diff, verdict, and warning evaluation child
- `src/backend/ops_governance/sandbox/report_api.rs` — sandbox report API route registrar and GET/POST handlers
- `src/backend/ops_governance/sandbox/verification_run.rs` — reusable sandbox verification runner and report persistence side effects
- `src/backend/ops_governance/sandbox/verification_run/proposal_gate.rs` — sandbox verification proposal eligibility gate child
- `src/backend/ops_governance/sandbox/verification_run/replay_window.rs` — sandbox verification replay window shape child
- `src/backend/ops_governance/sandbox/verification_run/report_assembly.rs` — sandbox verification report DTO assembly child
- `src/backend/ops_governance/sandbox/verification_run/report_commit.rs` — sandbox report persistence commit child
- `src/snapshot_service.rs` — 快照服务, SHA-256 签名; 改快照/验签时改这里
- `src/strategy_config_api.rs` — v4 策略配置 artifact、preflight、artifact diff、正式版本配置契约 diff 和显式 v4 backtest evidence diff API; 改策略配置契约、PaperSimulated/PaperActual 边界、capability freshness、Risk Plane 静态契约、配置域状态或证据差异口径时改这里 🆕 v4.11.0
- `src/storage_lifecycle.rs` — 存储生命周期, 三级分类 (Permanent/Temporary/Transient); 改存储策略时改这里
- `src/test_runner.rs` — 测试运行器; 改测试调度时改这里
- `src/tests_backend.rs` — 后端集成测试入口; 新增集成测试时改这里
- `src/bin/gen_screenshots.rs` — 截图生成工具; 改截图生成逻辑时改这里

### E.3 执行端: `src-executor/`

- `src-executor/main.rs` — 执行端入口, Axum Router :3001, 含 OKX demo provider submit/query/cancel 路由、PaperActual 启动前 demo 凭证校验与 v4 strategy_config_preflight 启动阻断; 改执行端启动或 provider 回执入口时改这里 🆕 v4.11.0
- `src-executor/executor_state.rs` — 执行端核心状态, ExecutionMode/StrategyStatus; 改状态机时改这里
- `src-executor/live_runner.rs` — 实时运行器, RunnerPool v3/v4 双 runner、OKX Market→v4 event 转换、v4 evidence 广播; 改策略执行循环时改这里 🆕 v4.2.0
- `src-executor/ws_client.rs` — OKX WebSocket 客户端; 改 WS 连接/行情订阅时改这里
- `src-executor/okx_rest.rs` — OKX REST API, 固定 OKX 模拟盘 demo profile、submit/query/cancel 回执和签名; 改 OKX provider 适配时改这里 🆕 v4.8.0
- `src-executor/kline_buffer.rs` — K线缓冲池; 改 K 线缓存策略时改这里
- `src-executor/credential_vault_v2.rs` — 凭证保险库 v2, PBKDF2 1M 轮; 改执行端加密时改这里
- `src-executor/api_guard.rs` — API 守卫; 改执行端认证时改这里
- `src-executor/audit_log.rs` — 审计日志; 改操作审计时改这里
- `src-executor/migration_api.rs` — 迁移 API; 改执行端数据迁移时改这里

### E.4 Cargo 工作区 crates

**qrpc_core**:
- `qrpc_core/Cargo.toml` — qrpc_core 包配置
- `qrpc_core/src/lib.rs` — 核心 trait 定义
- `qrpc_core/src/error.rs` — 核心错误类型; 改跨 crate 错误时改这里
- `qrpc_core/src/plugin.rs` — 插件 trait; 改插件接口时改这里
- `qrpc_core/src/strategy_ir.rs` — 策略 IR 核心类型; 改策略中间表示时改这里

**qrpc_core_ir**:
- `qrpc_core_ir/Cargo.toml` — qrpc_core_ir 包配置
- `qrpc_core_ir/src/lib.rs` — v3 类型定义 + v4 模块导出
- `qrpc_core_ir/src/v4.rs` — v4 parent facade and residual contract families (runtime/venue/type-system validators, compat bridge, complexity budget); 改 v4 parent wiring or residual v4 contract families 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/backtest_artifact_contract.rs` — v4 backtest artifact DTO family (`V4BacktestArtifact`, microstructure metrics, machine trajectory, risk-plane decision, execution capability source records); 改 v4 artifact schema DTO 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_contract.rs` — v4 machine schema/taxonomy facade (`V4MachineContract`, states, groups, transitions, memory fields, machine policy enums); 改 v4 machine schema or taxonomy 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_contract/static_validation.rs` — v4 machine static validation behavior (`V4MachineContract::validate_static_contract`, state/group/transition/memory/child-machine/policy conflict checks); 改 v4 machine validation behavior or error strings 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract.rs` — v4 machine graph schema, event catalog schema, graph/event/risk-plane validation, and parent-only traversal helpers for complexity metrics; 改 v4 graph schema or graph validation behavior 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract/event_catalog.rs` — v4 graph event catalog schema and local event catalog validation; 改 v4 event catalog DTOs or catalog-local validation 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract/static_validation.rs` — v4 graph static validation behavior (`V4MachineGraphContract::validate_static_contract`, DAG/event/risk-plane checks, event party helper); 改 v4 graph validation behavior or error strings 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract/static_validation/event_usage_validation.rs` — v4 graph event usage validation behavior (`V4MachineGraphContract::validate_event_catalog`); 改 graph event catalog usage, emitter, consumer, or edge-party invariants 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract/static_validation/event_usage_validation/event_party_validation.rs` — v4 graph event party validation behavior; 改 transition/action/edge emitter or consumer permission invariants 时改这里 🆕 v4.16.0
- `qrpc_core_ir/src/v4/machine_graph_contract/static_validation/risk_plane_validation.rs` — v4 graph risk-plane validation behavior (`V4MachineGraphContract::validate_risk_plane`); 改 execution graph risk-plane invariants or error strings 时改这里 🆕 v4.16.0

**qrpc_compiler**:
- `qrpc_compiler/Cargo.toml` — qrpc_compiler 包配置
- `qrpc_compiler/src/lib.rs` — 编译层入口; 改 Core IR 编译逻辑时改这里

**qrpc_runtime**:
- `qrpc_runtime/Cargo.toml` — qrpc_runtime 包配置
- `qrpc_runtime/src/lib.rs` — 运行时入口
- `qrpc_runtime/src/v4_runtime.rs` — v4 runtime 主事件循环; 改 PaperSimulated/PaperActual 边界、tick replay、Market 事件注入、多交易对 machine 展开、嵌套 machine 路由/snapshot 或 v4 运行时行为时改这里 🆕 v4.7.0
- `qrpc_runtime/src/v4_runtime_types.rs` — v4 runtime 类型定义与初始化辅助; 改 runtime snapshot/input/output 类型或 machine 初始化时改这里 🆕 v4.8.0
- `qrpc_runtime/src/v4_simulated_execution.rs` — v4 模拟撮合引擎; 改 OCO/trailing/GTD/amend、订单/成交/持仓/资产曲线时改这里 🆕 v4.8.0
- `qrpc_runtime/src/v4_runtime_tests.rs` — v4 runtime 单元测试模块; 改 v4 runtime 行为测试时改这里 🆕 v4.8.0
- `qrpc_runtime/src/compat.rs` — 模块热替换兼容性检查; 改热替换规则时改这里
- `qrpc_runtime/src/core_ir_evaluator.rs` — Core IR 求值器, 18 种指标 evaluator; 改指标计算时改这里
- `qrpc_runtime/src/backtest_metrics.rs` — 回测 12 项指标; 改回测公式或 equity_curve 诊断时改这里
- `qrpc_runtime/src/sandbox/mod.rs` — 沙盒调度器; 改沙盒流程时改这里
- `qrpc_runtime/src/sandbox/replay.rs` — 确定性回放; 改 v3 timeline 或 v4 deterministic bar replay 机制时改这里 🆕 v4.3.0
- `qrpc_runtime/src/sandbox/timeline.rs` — 时间线管理; 改事件时序时改这里
- `qrpc_runtime/src/data_module.rs` — 数据模块; 改市场数据接入时改这里
- `qrpc_runtime/src/intent_module.rs` — 意图模块; 改意图生成时改这里
- `qrpc_runtime/src/agent_module.rs` — 代理模块; 改代理决策时改这里
- `qrpc_runtime/src/execution_module.rs` — 执行模块; 改执行路径时改这里
- `qrpc_runtime/src/fill_engine.rs` — 模拟成交引擎; 改成交逻辑时改这里
- `qrpc_runtime/src/live_execution.rs` — 实时执行 (OKX); 改 OKX 对接时改这里
- `qrpc_runtime/src/circuit_breaker.rs` — 熔断器; 改异常保护时改这里
- `qrpc_runtime/src/config_tracker.rs` — 配置追踪; 改配置变更检测时改这里
- `qrpc_runtime/src/hotswap.rs` — 模块热替换; 改运行时替换时改这里
- `qrpc_runtime/src/merge.rs` — 合并逻辑; 改多策略合并时改这里
- `qrpc_runtime/src/merge_coordinator.rs` — 合并协调; 改合并调度时改这里
- `qrpc_runtime/src/reconcile.rs` — 对账; 改运行结果对账时改这里
- `qrpc_runtime/src/runtime_state.rs` — 运行时状态; 改状态管理时改这里
- `qrpc_runtime/src/risk_checker.rs` — 风控检查; 改风控规则时改这里
- `qrpc_runtime/src/risk_monitor.rs` — 风控监控; 改风控监控时改这里
- `qrpc_runtime/src/slippage.rs` — 滑点计算; 改滑点模型时改这里
- `qrpc_runtime/src/plugin_market.rs` — 插件市场, Ed25519 签名; 改插件验证时改这里
- `qrpc_runtime/src/plugin_runtime_registry.rs` — 插件注册表和安全策略检查; 改插件加载或 action 安全门禁时改这里
- `qrpc_runtime/src/plugin_sandbox.rs` — 插件沙盒, 提供 `execute_checked` 接入注册表安全策略; 改插件隔离时改这里 🆕 v4.9.0

**qrpc_session**:
- `qrpc_session/Cargo.toml` — qrpc_session 包配置
- `qrpc_session/src/lib.rs` — 进程间加密 (AES-256-GCM + HMAC-SHA256); 改进程通信安全时改这里

**quantscript**:
- `quantscript/Cargo.toml` — quantscript 包配置
- `quantscript/src/lib.rs` — crate 入口
- `quantscript/src/script.rs` — QS 词法/语法解析; 改 QS 语法时改这里
- `quantscript/src/hir.rs` — 高级中间表示; 改 QS 语义模型时改这里
- `quantscript/src/analysis.rs` — 语义分析; 改类型检查时改这里
- `quantscript/src/resolve.rs` — 符号解析; 改 QS 函数注册时改这里
- `quantscript/src/types.rs` — QS 类型系统; 改类型定义时改这里
- `quantscript/src/evaluator.rs` — 表达式求值; 改表达式语义时改这里
- `quantscript/src/diagnostics.rs` — QS 诊断码; 新增诊断时改这里
- `quantscript/src/test_plan.rs` — 测试计划生成; 改场景生成时改这里
- `quantscript/src/v4_static_audit.rs` — v4 状态机静态审计; 改 v4 QS 审计、state 内嵌套 machine、memory `QsTypeRef` 解析或 QSV 诊断码时改这里 🆕 v4.7.0
- `quantscript/src/lowering/mod.rs` — lowering 模块入口
- `quantscript/src/lowering/orchestrator.rs` — 降级编排器; 改降级流程时改这里
- `quantscript/src/lowering/bindings.rs` — 绑定降级; 改 HIR→Core IR 绑定时改这里
- `quantscript/src/lowering/binding_sources.rs` — 绑定来源处理
- `quantscript/src/lowering/context.rs` — 降级上下文, 符号表/类型环境
- `quantscript/src/lowering/diagnostics.rs` — lowering 诊断
- `quantscript/src/lowering/fallback.rs` — 降级回退策略
- `quantscript/src/lowering/helper_env.rs` — 辅助环境
- `quantscript/src/lowering/intents.rs` — 意图降级; 改 HIR intent→Core IR 时改这里
- `quantscript/src/lowering/semantic.rs` — 语义降级; 改语义→Core IR 时改这里
- `quantscript/src/lowering/shared.rs` — 共享工具
- `quantscript/src/lowering/source_recovery.rs` — 源码恢复, Core IR→QS 源码位置
- `quantscript/src/lowering/universe.rs` — 交易对展开; 改多交易对策略时改这里

**src-tauri**:
- `src-tauri/Cargo.toml` — Tauri 包配置
- `src-tauri/src/main.rs` — Tauri 壳入口, 等待后端 :3000; 改桌面壳行为时改这里
- `src-tauri/tauri.conf.json` — Tauri 配置, 窗口/CSP/打包; 改桌面配置时改这里
- `src-tauri/build.rs` — Tauri 构建脚本
- `src-tauri/build.bat` — 前端构建脚本
- `src-tauri/dev.bat` — 前端开发脚本

### E.5 前端: `frontend/src/`

**入口**:
- `frontend/src/main.jsx` — React 入口, 挂载 App
- `frontend/src/App.jsx` — App Shell, 路由匹配/全局状态/教程/命令面板/Toast; 改全局 UI 时改这里
- `frontend/src/router.js` — 路由定义, 12 用户路由 + 404 + `navigateTo()`/`parseRoute()`; 新增路由时改这里
- `frontend/src/router.test.js` — 路由单元测试
- `frontend/index.html` — HTML 入口
- `frontend/package.json` — 前端包配置, 依赖/脚本; 改依赖时改这里
- `frontend/vite.config.js` — Vite 构建配置
- `frontend/vitest.config.js` — Vitest 测试配置
- `frontend/playwright.config.js` — Playwright E2E 配置
- `frontend/playwright.ci.config.js` — CI E2E 配置
- `frontend/playwright.perf.config.js` — 性能测试配置
- `frontend/playwright.visual.config.js` — 视觉回归配置
- `frontend/playwright.real.config.js` — 真实环境 E2E 配置

**设计系统**:
- `frontend/src/styles.css` — 全局样式, `:root` CSS 变量; v4.10.0 后主文件已拆分瘦身
- `frontend/src/styles-responsive-panels.css` — 响应式面板、减少动效和教程覆盖层样式 🆕 v4.10.0
- `frontend/src/shared.css` — 共享样式
- `frontend/src/design-system.css` — Adobe 暗色面板设计系统, `--ad-*` CSS 令牌

**API**:
- `frontend/src/api/client.js` — HTTP 客户端封装; 改 API 调用方式时改这里

**能力投影层**:
- `frontend/src/capabilities/capabilityProjection.js` — 能力投影核心, 3 函数流水线; 改能力→UI 映射时改这里
- `frontend/src/capabilities/capabilityProjection.test.js` — 能力投影测试
- `frontend/src/capabilities/supportMatrix.js` — 支持矩阵, 9 surface + 14 action + 权限边界; 新增能力入口时改这里
- `frontend/src/capabilities/supportMatrix.test.js` — 支持矩阵测试
- `frontend/src/capabilities/capabilityGovernance.js` — 能力治理, 4 类 74 项; 改能力分类时改这里
- `frontend/src/capabilities/capabilityGovernance.test.js` — 能力治理测试

**状态管理**:
- `frontend/src/store/graphStore.js` — 主 Zustand store; 改全局状态时改这里
- `frontend/src/store/graphStoreCompileState.js` — 编译状态
- `frontend/src/store/graphStoreCompileActions.js` — 编译动作
- `frontend/src/store/graphStoreCompileApi.js` — 编译 API 调用
- `frontend/src/store/graphStoreCompileFlow.js` — 编译流程
- `frontend/src/store/graphStoreCompileHelpers.js` — 编译辅助
- `frontend/src/store/graphStoreCompileOutcomeMapping.js` — 编译结果映射
- `frontend/src/store/graphStoreCompileOutcomeProjection.js` — 编译结果投影
- `frontend/src/store/graphStoreCompileOutcomeProjection.test.js` — 编译结果投影测试
- `frontend/src/store/graphStoreCompileProtocolFlow.js` — 协议编译流程
- `frontend/src/store/graphStoreCompileProtocolMapping.js` — 协议编译映射
- `frontend/src/store/graphStoreEditorActions.js` — 编辑器动作
- `frontend/src/store/graphStoreHelpers.js` — 通用辅助
- `frontend/src/store/graphStorePersistenceActions.js` — 持久化动作, 含策略包导入入口
- `frontend/src/store/graphStorePersistenceHelpers.js` — 持久化辅助
- `frontend/src/store/graphStorePersistenceConsistency.test.js` — 持久化一致性测试
- `frontend/src/store/graphStoreRuntimeActions.js` — 运行时动作
- `frontend/src/store/graphStoreRuntimeHelpers.js` — 运行时辅助
- `frontend/src/store/graphStoreRuntimeTransport.js` — 运行时 SSE 传输
- `frontend/src/store/graphStoreRuntimeTransport.test.js` — SSE 传输测试
- `frontend/src/store/graphStoreRuntimeSessionActions.js` — 运行时会话动作, 含 v4 QS 模拟启动与 v4 backtest runtime_kind/symbols 透传 🆕 v4.3.0
- `frontend/src/store/graphStoreRuntimeSessionState.js` — 运行时会话状态, 含 v4 memory snapshot/handoff 清理 🆕 v4.1.0
- `frontend/src/store/graphStoreRuntimeSelectionState.js` — 运行时选择状态
- `frontend/src/store/graphStoreRuntimeSelectionState.test.js` — 运行时选择测试
- `frontend/src/store/graphStoreRuntimeHistoryActions.js` — 运行时历史动作
- `frontend/src/store/graphStoreRuntimeHistoryApi.js` — 运行时历史 API
- `frontend/src/store/graphStoreRuntimeHistoryFlow.js` — 运行时历史流程
- `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js` — 运行时历史流程测试
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js` — 运行时历史投影
- `frontend/src/store/graphStoreRuntimeHistoryState.js` — 运行时历史状态
- 以及 `graphStore.*.test.js` 测试文件 (backtestArtifacts, capabilities, detailLoadErrors, diagnostics, editorActions, export, recentNodes, runtimeActionLock, runtimeErrors, saveGraphRollback, startupRecovery, strategyIrCompile, strategyIrDraft, templates, versionHistory)

**图编辑**:
- `frontend/src/graph/compileGraph.js` — 图编译逻辑; 改编译流程时改这里
- `frontend/src/graph/compileGraph.diagnostics.test.js` — 编译诊断测试
- `frontend/src/graph/compileGraph.multiSymbol.test.js` — 多交易对编译测试
- `frontend/src/graph/createGraph.js` — 图创建; 改图初始化时改这里
- `frontend/src/graph/createNode.js` — 节点创建; 改节点初始化时改这里
- `frontend/src/graph/quantscript.js` — QS 集成; 改 QS 前端交互时改这里
- `frontend/src/graph/quantscript.test.js` — QS 测试
- `frontend/src/graph/spread.test.js` — Spread 测试
- `frontend/src/graph/validation.js` — 图验证; 改验证规则时改这里

**国际化**:
- `frontend/src/i18n/index.js` — i18n 入口, `useI18n` hook; 改翻译机制时改这里
- `frontend/src/i18n/i18n.test.js` — i18n 测试
- `frontend/src/i18n/locales/zh-CN.js` — 中文翻译; 新增/修改中文文本时改这里
- `frontend/src/i18n/locales/en-US.js` — 英文翻译; 新增/修改英文文本时改这里

**测试 fixtures**:
- `frontend/src/test/setup.js` — 测试环境初始化
- `frontend/src/test/testBridge.js` — 测试桥接, mock API
- `frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json` — 能力 API mock
- `frontend/src/test/fixtures/capabilities/capabilityFallbacks.js` — 能力降级 mock
- `frontend/src/test/fixtures/runtime/*.js` — 运行时 mock (backtestSuccess, buildValidatedSampleGraph, capabilityRejections, editorBootstrap, runSuccess)

**模板**:
- `frontend/src/templates/strategyTemplates.js` — 策略模板库; 改 v3/v4 starter graph、v4 MachineGraph metadata 或模板入口时改这里 🆕 v4.3.0
- `frontend/src/templates/strategyTemplates.test.js` — 策略模板边界测试; 改 v4 `market.data` / `default_venue_id` / provider 去耦合断言时改这里 🆕 v4.8.0

**模块**:
- `frontend/src/modules/builtinModules.js` — 18 种内置指标模块定义; 新增指标模块时改这里
- `frontend/src/modules/moduleRegistry.js` — 模块注册表; 改模块加载时改这里
- `frontend/src/modules/moduleRegistry.test.js` — 模块注册表测试

**Node**:
- `frontend/src/nodes/BaseNodeCard.jsx` — 节点卡片基础组件; 改节点外观时改这里
- `frontend/src/nodes/BaseNodeCard.test.jsx` — 节点卡片测试
- `frontend/src/nodes/nodeCardPresentation.js` — 节点卡片展示逻辑; 改节点颜色/布局时改这里
- `frontend/src/nodes/nodeCardPresentation.test.js` — 节点展示测试

**教程**:
- `frontend/src/data/tutorialSteps.js` — 教程步骤定义; 改教程内容时改这里

**E2E 测试**:
- `frontend/tests/e2e/editor-capabilities-smoke.spec.js` — 编辑器能力冒烟
- `frontend/tests/e2e/evidence-contract-walkthrough.spec.js` — 证据契约遍历
- `frontend/tests/e2e/perf-first-screen-review.spec.js` — 首屏性能审查
- `frontend/tests/e2e/perf-react-flow-mount-review.spec.js` — React Flow 挂载性能
- `frontend/tests/e2e/run-backtest.spec.js` — 回测 E2E
- `frontend/tests/e2e/run-simulation.spec.js` — 模拟运行 E2E
- `frontend/tests/e2e/runtime-mutation-walkthrough.spec.js` — 运行时变更遍历
- `frontend/tests/e2e/scenario-test-v2.spec.js` — 场景测试 v2
- `frontend/tests/e2e/v4-runtime-contracts.spec.js` — v4 runtime 契约 E2E
- `frontend/tests/e2e/visual-regression.spec.js` — 视觉回归
- `frontend/tests/e2e/visual-responsive-review.spec.js` — 响应式审查
- `frontend/tests/e2e/support/*.js` — E2E 辅助 (apiHarness, workspaceBootstrapMocks, workspaceGraphFixture, analysisReviewFixtures)

**精确路径补充索引**:
- `frontend/src/components/ApprovalPanel.jsx` — 审批面板组件; 改审批展示时改这里
- `frontend/src/components/AssetCandlesPanel.test.jsx` — K 线面板测试
- `frontend/src/components/BacktestHistorySection.jsx` — 回测历史组件; 改历史展示时改这里
- `frontend/src/components/CompilePanel.integration.test.jsx` — 编译面板集成测试
- `frontend/src/components/CredentialInput.jsx` — 凭证输入组件; 改凭证表单时改这里
- `frontend/src/components/DeployButton.jsx` — 部署按钮; 改部署入口时改这里
- `frontend/src/components/DiagnosticsPanel.jsx` — 诊断面板; 改诊断展示时改这里
- `frontend/src/components/DiagnosticsPanel.test.jsx` — 诊断面板测试
- `frontend/src/components/EventReplaySection.jsx` — 事件回放组件; 改回放 UI 时改这里
- `frontend/src/components/EventReplaySection.test.jsx` — 事件回放测试
- `frontend/src/components/EventStreamPanel.backtestArtifacts.test.jsx` — 事件流回测工件测试
- `frontend/src/components/EventStreamPanel.backtestHistory.test.jsx` — 事件流回测历史测试
- `frontend/src/components/EventStreamPanel.dataQuality.test.jsx` — 事件流数据质量测试
- `frontend/src/components/EventStreamPanel.executionExplanation.test.jsx` — 事件流执行解释测试
- `frontend/src/components/EventStreamPanel.historyExplanation.test.jsx` — 事件流历史解释测试
- `frontend/src/components/EventStreamPanel.jsx` — 事件流面板; 改运行事件展示时改这里
- `frontend/src/components/EventStreamPanel.layout.test.jsx` — 事件流布局测试
- `frontend/src/components/EventStreamPanel.nodeFocus.test.jsx` — 事件流节点聚焦测试
- `frontend/src/components/EventStreamPanel.refreshFeedback.test.jsx` — 事件流刷新反馈测试
- `frontend/src/components/EventStreamPanel.runtimeArtifactActions.test.jsx` — 事件流运行工件动作测试
- `frontend/src/components/EvidenceSummaryCards.jsx` — 证据摘要卡片; 改证据概览时改这里
- `frontend/src/components/GovernedTimelinePanel.jsx` — 治理时间线面板; 改治理时间线时改这里
- `frontend/src/components/GovernedTimelinePanel.test.jsx` — 治理时间线测试
- `frontend/src/components/Icons.jsx` — 前端共享图标; 改本地 icon 时改这里
- `frontend/src/components/LeftSidebar.test.jsx` — 左侧导航测试
- `frontend/src/components/ModuleSidebar.jsx` — 模块侧栏; 改模块列表/拖拽入口时改这里
- `frontend/src/components/ModuleSidebar.test.jsx` — 模块侧栏测试
- `frontend/src/components/PropertyPanel.compileSummary.test.jsx` — 属性面板编译摘要测试
- `frontend/src/components/PropertyPanel.jsx` — 属性面板; 改节点/边配置面板时改这里
- `frontend/src/components/PropertyPanel.layout.test.jsx` — 属性面板布局测试
- `frontend/src/components/PropertyPanel.strategyIr.test.jsx` — 属性面板 Strategy IR 测试
- `frontend/src/components/propertyPanelViews.jsx` — 属性面板视图拆分; 改面板子视图时改这里
- `frontend/src/components/RunHistorySection.jsx` — 运行历史组件; 改运行历史时改这里
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx` — 运行时诊断面板; 改运行诊断时改这里
- `frontend/src/components/RuntimeDiagnosticsPanel.test.jsx` — 运行时诊断测试
- `frontend/src/components/RuntimeMutationPanel.jsx` — 运行时变更面板; 改 AI 提案/审批入口时改这里
- `frontend/src/components/RuntimeMutationPanel.test.jsx` — 运行时变更测试
- `frontend/src/components/RuntimeReportPanel.jsx` — 运行报告面板; 改报告展示时改这里
- `frontend/src/components/RuntimeReportPanel.test.jsx` — 运行报告测试
- `frontend/src/components/StrategyBacktestsPanel.jsx` — 策略回测面板; 改回测入口时改这里
- `frontend/src/components/StrategyCanvas.focus.test.jsx` — 策略画布焦点测试
- `frontend/src/components/StrategyCanvas.interaction.test.jsx` — 策略画布交互测试
- `frontend/src/components/StrategyCanvas.jsx` — 策略画布; 改 React Flow 画布时改这里
- `frontend/src/components/strategyCanvasFocus.js` — 画布焦点工具; 改焦点规则时改这里
- `frontend/src/components/strategyCanvasFocus.test.js` — 画布焦点工具测试
- `frontend/src/components/StrategyCanvasMiniMap.jsx` — 策略画布小地图; 改小地图时改这里
- `frontend/src/components/strategyCanvasViewport.js` — 画布视口工具; 改缩放/定位时改这里
- `frontend/src/components/strategyCanvasViewport.test.js` — 画布视口工具测试
- `frontend/src/components/StrategyCodePanel.authoringView.test.jsx` — 策略代码面板编写视图测试
- `frontend/src/components/StrategyCodePanel.jsx` — 策略代码面板; 改 QS 编写/预览时改这里
- `frontend/src/components/StrategyDiagnosticsPanel.jsx` — 策略诊断面板; 改策略诊断时改这里
- `frontend/src/components/StrategyEventsPanel.jsx` — 策略事件面板; 改事件摘要时改这里
- `frontend/src/components/StrategyParamsPanel.jsx` — 策略参数面板; 改参数编辑时改这里
- `frontend/src/components/StrategyResearchConsole.jsx` — 策略研究控制台; 改研究工作流时改这里
- `frontend/src/components/StrategyResearchConsole.test.jsx` — 策略研究控制台测试
- `frontend/src/components/StrategyRunsPanel.jsx` — 策略运行面板; 改运行入口时改这里
- `frontend/src/components/TopToolbar.capabilities.test.jsx` — 顶栏 capability 测试
- `frontend/src/components/TopToolbar.exportFailure.test.jsx` — 顶栏导出失败测试
- `frontend/src/components/TopToolbar.failureNotices.test.jsx` — 顶栏失败提示测试
- `frontend/src/components/TopToolbar.formalSourceMode.test.jsx` — 顶栏正式源模式测试
- `frontend/src/components/TopToolbar.persistenceFailure.test.jsx` — 顶栏持久化失败测试
- `frontend/src/components/ComplexityBudgetPanel.jsx` — v4 复杂度预算面板; 改嵌套状态机预算展示时改这里 🆕 v4.7.0
- `frontend/src/components/V4RuntimeEvidencePanel.jsx` — v4 运行证据面板; 改 v4 evidence UI 或嵌套 machine 层级展示时改这里
- `frontend/src/components/V4RuntimeEvidencePanel.test.jsx` — v4 运行证据面板测试
- `frontend/src/hooks/propertyPanelSelectors.js` — 属性面板 selector; 改面板数据投影时改这里
- `frontend/src/hooks/propertyPanelShared.js` — 属性面板共享逻辑
- `frontend/src/hooks/strategyResearchSelectors.js` — 研究页 selector; 改研究数据选择时改这里
- `frontend/src/hooks/strategyResearchSelectors.test.js` — 研究 selector 测试
- `frontend/src/hooks/useNotification.js` — 通知 hook; 改 Toast 状态时改这里
- `frontend/src/hooks/useOrderAnimation.js` — 订单动画 hook
- `frontend/src/hooks/usePanelResize.js` — 面板 resize hook
- `frontend/src/hooks/usePropertyPanelActions.js` — 属性面板动作 hook
- `frontend/src/hooks/usePropertyPanelModel.js` — 属性面板模型 hook
- `frontend/src/hooks/useStrategyDirectoryModel.js` — 策略目录模型 hook; 策略中心保持全量可滚动列表, 不做搜索/筛选/排序 🆕 v4.10.0
- `frontend/src/hooks/useStrategyHubBodyData.js` — 策略中心主体数据 hook
- `frontend/src/hooks/useStrategyHubInspectorData.js` — 策略中心 inspector 数据 hook
- `frontend/src/hooks/useStrategyHubRosterData.js` — 策略中心列表数据 hook
- `frontend/src/hooks/useStrategyResearchActions.js` — 策略研究动作 hook
- `frontend/src/hooks/useStrategyResearchModel.js` — 策略研究模型 hook
- `frontend/src/hooks/useStrategyResearchUiState.js` — 策略研究 UI 状态 hook
- `frontend/src/hooks/useStrategyWorkspacePageData.js` — 工作区页面数据 hook
- `frontend/src/hooks/useStrategyWorkspaceSharedModel.js` — 工作区共享模型 hook
- `frontend/src/hooks/useStrategyWorkspaceUiState.js` — 工作区 UI 状态 hook
- `frontend/src/hooks/useTutorial.js` — 教程 hook, 首次访问自动触发与可见入口事件 🆕 v4.10.0
- `frontend/src/hooks/useWorkspaceActionBarActions.js` — 工作区动作栏动作 hook
- `frontend/src/hooks/useWorkspaceActionBarModel.js` — 工作区动作栏模型 hook
- `frontend/src/hooks/workspaceActionBarShared.js` — 工作区动作栏共享工具
- `frontend/src/hooks/workspaceActionSelectors.js` — 工作区动作 selector
- `frontend/src/pages/AlertsPage.jsx` — 告警页; 改告警 UI 时改这里
- `frontend/src/pages/backtest-analysis.css` — 回测分析样式
- `frontend/src/pages/BacktestAnalysisLayout.jsx` — 回测分析布局
- `frontend/src/pages/backtestAnalysisShared.jsx` — 回测分析共享组件
- `frontend/src/pages/backtestAnalysisShared.test.jsx` — 回测分析共享测试
- `frontend/src/pages/BacktestComparePage.jsx` — 回测对比页
- `frontend/src/pages/BacktestComparePage.test.jsx` — 回测对比页测试
- `frontend/src/pages/BacktestDetailPage.jsx` — 回测详情页, 展示 v4 backtest artifact/evidence/tick 与 microstructure 指标; 改回测分析入口时改这里 🆕 v4.7.0
- `frontend/src/pages/BacktestDetailPage.test.jsx` — 回测详情页测试
- `frontend/src/pages/ChaosPage.jsx` — 混沌实验页
- `frontend/src/pages/EditorPage.jsx` — 编辑器页
- `frontend/src/pages/NotFoundPage.jsx` — 404 页面, 未知路由返回策略中心或设置入口 🆕 v4.8.2
- `frontend/src/pages/QuantScriptEditor.jsx` — QuantScript 编辑页, 含草稿持久化、humanized error 和粘贴超限 Toast
- `frontend/src/pages/RunbookPage.jsx` — 运行手册页
- `frontend/src/pages/SettingsPage.jsx` — 设置页, 管理语言、auto/dark/light 主题和 QS 草稿策略; 不含账户功能 🆕 v4.10.0
- `frontend/src/pages/SnapshotsPage.jsx` — 快照页
- `frontend/src/pages/StrategyBacktestsPage.jsx` — 策略回测页
- `frontend/src/pages/StrategyBacktestsPage.test.jsx` — 策略回测页测试
- `frontend/src/pages/strategy-hub.css` — 策略中心样式
- `frontend/src/pages/StrategyHubActivityPanelsSection.jsx` — 策略中心活动面板区
- `frontend/src/pages/StrategyHubBacktestActivityCard.jsx` — 策略中心回测活动卡片
- `frontend/src/pages/StrategyHubBodySection.jsx` — 策略中心主体区
- `frontend/src/pages/StrategyHubCompareQueueSection.jsx` — 策略中心对比队列区
- `frontend/src/pages/StrategyHubHeroSection.jsx` — 策略中心头部区, 含新手指引可见入口; 不含搜索/筛选入口 🆕 v4.10.0
- `frontend/src/pages/StrategyHubInlineNote.jsx` — 策略中心内联提示
- `frontend/src/pages/StrategyHubInspectorOverviewSection.jsx` — 策略中心 inspector 概览
- `frontend/src/pages/StrategyHubInspectorSection.jsx` — 策略中心 inspector 区
- `frontend/src/pages/StrategyHubPage.jsx` — 策略中心页
- `frontend/src/pages/StrategyHubPage.test.jsx` — 策略中心页测试
- `frontend/src/pages/StrategyHubPanelFallbacks.jsx` — 策略中心 fallback 面板
- `frontend/src/pages/StrategyHubRecentBacktestsSection.jsx` — 最近回测区
- `frontend/src/pages/StrategyHubRecentRunItem.jsx` — 最近运行项
- `frontend/src/pages/StrategyHubRecentRunsSection.jsx` — 最近运行区
- `frontend/src/pages/StrategyHubRosterDirectorySection.jsx` — 策略目录区
- `frontend/src/pages/StrategyHubRosterRowActions.jsx` — 策略行操作
- `frontend/src/pages/StrategyHubRosterSection.jsx` — 策略列表区
- `frontend/src/pages/StrategyHubRosterTableRow.jsx` — 策略表格行
- `frontend/src/pages/StrategyHubRosterTableSection.jsx` — 策略表格区
- `frontend/src/pages/StrategyHubRosterTableSection.test.jsx` — 策略表格区测试
- `frontend/src/pages/StrategyHubRosterToolbar.jsx` — 策略列表工具栏
- `frontend/src/pages/StrategyHubRunActivityCard.jsx` — 运行活动卡片
- `frontend/src/pages/StrategyHubSectionFallbacks.jsx` — 策略中心区块 fallback
- `frontend/src/pages/StrategyHubSharedComponents.jsx` — 策略中心共享组件
- `frontend/src/pages/StrategyHubTemplateLibrarySection.jsx` — 模板库区
- `frontend/src/pages/StrategyHubTemplateLibrarySection.test.jsx` — 模板库测试
- `frontend/src/pages/strategy-workspace.css` — 策略工作区样式
- `frontend/src/pages/StrategyWorkspaceCodeTab.jsx` — 工作区代码页签
- `frontend/src/pages/StrategyWorkspaceCollaborationCard.jsx` — 工作区协作卡片
- `frontend/src/pages/StrategyWorkspaceCollaborationCard.test.jsx` — 工作区协作卡片测试
- `frontend/src/pages/StrategyConfigCockpit.jsx` — v4 策略配置台, 展示 preflight、配置域导航、单域来源/诊断、runtime boundary、真实资金未开放边界、证据锚点、AI 提案绑定、当前 artifact JSON 导出和本地 artifact diff 🆕 v4.11.0
- `frontend/src/pages/StrategyConfigCockpit.test.jsx` — v4 策略配置台测试, 锁定 PaperSimulated 请求、真实资金未开放文案、配置域深钻、artifact 导出、证据锚点和 AI 提案绑定展示 🆕 v4.11.0
- `frontend/src/pages/StrategyWorkspaceDashboard.jsx` — 工作区仪表盘
- `frontend/src/pages/StrategyWorkspaceDebugTab.jsx` — 工作区调试页签
- `frontend/src/pages/StrategyWorkspaceDiagnosticsTab.jsx` — 工作区诊断页签
- `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx` — 工作区实验卡片
- `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx` — 工作区实验卡片测试
- `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx` — 工作区问题队列卡片
- `frontend/src/pages/StrategyWorkspaceMonitorTab.jsx` — 工作区监控页签
- `frontend/src/pages/StrategyWorkspaceOverviewTab.jsx` — 工作区概览页签
- `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx` — 工作区代码模式测试
- `frontend/src/pages/StrategyWorkspacePage.jsx` — 策略工作区页
- `frontend/src/pages/StrategyWorkspacePageSections.jsx` — 工作区分区组件
- `frontend/src/pages/StrategyWorkspacePanelFallbacks.jsx` — 工作区面板 fallback
- `frontend/src/pages/StrategyWorkspaceResearchTab.jsx` — 工作区研究页签
- `frontend/src/pages/StrategyWorkspaceSourceTab.jsx` — 工作区源码页签
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx` — 工作区版本历史卡片, 展示图版本 diff、正式版本间配置契约 diff 与显式 v4 backtest evidence diff
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx` — 工作区版本历史测试, 覆盖正式版本配置 diff 与显式证据 diff
- `frontend/src/store/graphStore.backtestArtifacts.test.js` — graphStore 回测工件测试
- `frontend/src/store/graphStore.capabilities.test.js` — graphStore capability 测试
- `frontend/src/store/graphStore.detailLoadErrors.test.js` — graphStore 详情加载错误测试
- `frontend/src/store/graphStore.diagnostics.test.js` — graphStore 诊断测试
- `frontend/src/store/graphStore.editorActions.test.js` — graphStore 编辑动作测试
- `frontend/src/store/graphStore.export.test.js` — graphStore 导出测试
- `frontend/src/store/graphStore.recentNodes.test.js` — graphStore 最近节点测试
- `frontend/src/store/graphStore.runtimeActionLock.test.js` — graphStore 运行时动作锁测试
- `frontend/src/store/graphStore.runtimeErrors.test.js` — graphStore 运行时错误测试
- `frontend/src/store/graphStore.saveGraphRollback.test.js` — graphStore 保存回滚测试
- `frontend/src/store/graphStore.startupRecovery.test.js` — graphStore 启动恢复测试
- `frontend/src/store/graphStore.strategyIrCompile.test.js` — graphStore Strategy IR 编译测试
- `frontend/src/store/graphStore.strategyIrDraft.test.js` — graphStore Strategy IR 草稿测试
- `frontend/src/store/graphStore.templates.test.js` — graphStore 模板测试
- `frontend/src/store/graphStore.versionHistory.test.js` — graphStore 版本历史测试
- `frontend/src/test/fixtures/runtime/backtestSuccess.js` — 回测成功 fixture
- `frontend/src/test/fixtures/runtime/buildValidatedSampleGraph.js` — 已验证样例图 fixture
- `frontend/src/test/fixtures/runtime/capabilityRejections.js` — capability 拒绝 fixture
- `frontend/src/test/fixtures/runtime/editorBootstrap.js` — 编辑器启动 fixture
- `frontend/src/test/fixtures/runtime/runSuccess.js` — 运行成功 fixture
- `frontend/src/utils/actionFailure.js` — 动作失败工具
- `frontend/src/utils/actionFailure.test.js` — 动作失败工具测试
- `frontend/src/utils/api.js` — 前端 API 辅助
- `frontend/src/utils/compileContract.js` — 编译契约工具
- `frontend/src/utils/configureFieldPriority.js` — 字段优先级工具
- `frontend/src/utils/configureFieldPriority.test.js` — 字段优先级测试
- `frontend/src/utils/errorMessages.js` — 错误消息工具
- `frontend/src/utils/errorText.js` — 错误文本工具
- `frontend/src/utils/errorText.test.js` — 错误文本测试
- `frontend/src/utils/repairPathInsights.js` — 修复路径洞察工具
- `frontend/src/utils/runtimeAiProposal.js` — 运行时 AI 提案工具, 保留 strategy config domain binding 投影
- `frontend/src/utils/runtimeAiProposal.test.js` — 运行时 AI 提案测试
- `frontend/src/utils/runtimeApproval.js` — 运行时审批工具
- `frontend/src/utils/runtimeDiagnosticsProjection.js` — 运行时诊断投影
- `frontend/src/utils/runtimeDiagnosticsProjection.test.js` — 运行时诊断投影测试
- `frontend/src/utils/runtimeEvidenceSummary.js` — 运行证据摘要工具
- `frontend/src/utils/runtimeEvidenceSummary.test.js` — 运行证据摘要测试
- `frontend/src/utils/runtimeExplanation.js` — 运行解释工具
- `frontend/src/utils/runtimeExplanation.test.js` — 运行解释测试
- `frontend/src/utils/runtimeGovernance.js` — 运行治理工具
- `frontend/src/utils/runtimeGovernance.test.js` — 运行治理测试
- `frontend/src/utils/runtimeMutation.js` — 运行时变更工具
- `frontend/src/utils/runtimeMutation.test.js` — 运行时变更测试
- `frontend/src/utils/runtimeStatus.js` — 运行状态工具
- `frontend/src/utils/runtimeTimeline.js` — 运行时间线工具
- `frontend/src/utils/runtimeTimeline.test.js` — 运行时间线测试
- `frontend/src/utils/strategyHubCompareQueueActions.js` — 策略中心对比队列动作
- `frontend/src/utils/strategyHubCompareQueueActions.test.js` — 策略中心对比队列动作测试
- `frontend/src/utils/strategyHubFormatters.js` — 策略中心格式化工具
- `frontend/src/utils/strategyHubInspectorActions.js` — 策略中心 inspector 动作
- `frontend/src/utils/strategyHubInspectorActions.test.js` — 策略中心 inspector 动作测试
- `frontend/src/utils/strategyHubInspectorProjection.js` — 策略中心 inspector 投影
- `frontend/src/utils/strategyHubInspectorProjection.test.js` — 策略中心 inspector 投影测试
- `frontend/src/utils/strategyHubRecentBacktestsActions.js` — 最近回测动作
- `frontend/src/utils/strategyHubRecentBacktestsActions.test.js` — 最近回测动作测试
- `frontend/src/utils/strategyHubRecentRunsView.js` — 最近运行视图工具
- `frontend/src/utils/strategyHubRecentRunsView.test.js` — 最近运行视图测试
- `frontend/src/utils/strategyHubRosterProjection.js` — 策略列表投影
- `frontend/src/utils/strategyHubRosterProjection.test.js` — 策略列表投影测试
- `frontend/src/utils/strategyHubRosterRowActions.js` — 策略列表行动作
- `frontend/src/utils/strategyHubRosterRowActions.test.js` — 策略列表行动作测试
- `frontend/src/utils/strategyHubStrategyIdentity.js` — 策略身份工具
- `frontend/src/utils/strategyWorkspaceIssueQueue.js` — 工作区问题队列工具
- `frontend/src/utils/strategyWorkspaceIssueQueue.test.js` — 工作区问题队列测试
- `frontend/src/utils/v4RuntimeEvidence.js` — v4 runtime evidence 投影工具; 改层级 machine 或 complexity_metrics 前端投影时改这里
- `frontend/src/utils/v4RuntimeEvidence.test.js` — v4 runtime evidence 测试
- `frontend/src/utils/workspaceContextLabels.js` — 工作区上下文标签工具
- `frontend/tests/e2e/support/analysisReviewFixtures.js` — 分析审查 E2E fixture
- `frontend/tests/e2e/support/apiHarness.js` — E2E API harness
- `frontend/tests/e2e/support/workspaceBootstrapMocks.js` — 工作区启动 mock
- `frontend/tests/e2e/support/workspaceGraphFixture.js` — 工作区图 fixture
- `tests/fixtures/runtime/evidence_contract_snapshot.json` — runtime evidence 契约快照 fixture
- `tests/fixtures/runtime/minimal_runtime_request.json` — 最小运行请求 fixture
- `tests/fixtures/runtime/mutation_contract_snapshot.json` — runtime mutation 契约快照 fixture

### E.6 执行端前端: `frontend-executor/src/`

- `frontend-executor/src/main.jsx` — React 入口
- `frontend-executor/src/ExecutorApp.jsx` — 执行端 App Shell, 多策略标签页/状态轮询/模式切换和启动阻断错误展示
- `frontend-executor/src/i18n.js` — 执行端轻量 i18n provider, 支持 `zh-CN` / `en-US` 🆕 v4.8.2
- `frontend-executor/src/design-system.css` — 执行端设计系统, 支持 auto/dark/light 主题令牌 🆕 v4.10.0
- `frontend-executor/src/components/ExecutorTopBar.jsx` — 顶部工具栏, PaperSimulated / OKX 模拟盘模式切换、策略选择、v3-v4 runtime 标识 🆕 v4.8.0
- `frontend-executor/src/components/V4EvidencePanel.jsx` — v4 状态机证据面板, 展示 machine/Risk Plane/Execution/模拟订单/双模拟盘 provider 边界摘要 🆕 v4.8.0
- `frontend-executor/src/components/StrategyGraphPanel.jsx` — 策略图只读预览, 已接入执行端 i18n 🆕 v4.10.0
- `frontend-executor/src/components/KlineChart.jsx` — K线图表, lightweight-charts, 含 bars 预设和数据源说明
- `frontend-executor/src/components/OrderPanel.jsx` — 订单面板
- `frontend-executor/src/components/AssetPanel.jsx` — 资产面板, 持仓/余额/权益曲线
- `frontend-executor/src/components/StrategyParamsPanel.jsx` — 热调参面板
- `frontend-executor/index.html` — HTML 入口
- `frontend-executor/package.json` — 包配置
- `frontend-executor/vite.config.js` — Vite 配置

### E.7 测试: `tests/`

- `tests/common/mod.rs` — 测试公共模块
- `tests/common/re_exports.rs` — 测试重导出
- `tests/api_ai_proposal.rs` — AI 提案 API 测试
- `tests/api_auth.rs` — 认证 API 错误响应测试
- `tests/api_backtest.rs` — 回测 API 测试
- `tests/api_collaboration.rs` — 协作 API 测试
- `tests/api_evidence_contract.rs` — 证据契约 API 测试
- `tests/api_experiments.rs` — 实验 API 测试
- `tests/api_graph_versions.rs` — 图版本 API 测试
- `tests/api_mutation.rs` — 变更 API 测试
- `tests/api_v1_reports.rs` — v1 report endpoint smoke 测试
- `tests/api_v1_ops_health.rs` — v1 merge/generation/storage health endpoint smoke 测试
- `tests/api_run.rs` — 运行 API 测试
- `tests/api_sse.rs` — SSE API 测试
- `tests/quantscript_real_strategy_authoring.rs` — QS 真实策略编写测试
- `tests/quantscript_universe_strategy.rs` — QS 多交易对策略测试
- `tests/report_qs_strategy.rs` — QS 策略报告测试
- `tests/test_deploy.json` — 部署测试配置
- `tests/fixtures/runtime/*.json` — 运行时测试 fixtures

### E.8 工具与脚本

**门禁脚本** (`tools/check-*.ps1`):
- `tools/check-utf8.ps1` — UTF-8 编码检查
- `tools/check-user-facing-text.ps1` — 面向用户文本检查
- `tools/check-capability-governance.ps1` — 能力治理检查
- `tools/check-capability-stack.ps1` — 能力栈一致性检查, 校验 schema hash、模块 key、fixture 和元流水线 DryRun
- `tools/check-i18n.ps1` — i18n 覆盖检查
- `tools/check-version-consistency.ps1` — 版本一致性检查, 覆盖 release manifest/OpenAPI/启动横幅 🆕 v4.1.0
- `tools/check-openapi-route-diff.ps1` — OpenAPI path 与 Rust route 基线 diff, 可用 `-FailOnDiff` 阻断 🆕 v4.8.1
- `tools/check-feature-evolution.ps1` — 功能演进检查
- `tools/check-matrix-governance.ps1` — 三矩阵治理检查, 校验治理入口、提案模板、模块树漂移和发布过渡协议 🆕 v4.14.0
- `tools/check-learning-closeout.ps1` — 学习流水线 closeout
- `tools/check-pre-commit-hook.ps1` — Pre-commit hook 同步检查
- `tools/check-cleanup-boundary.ps1` — 清理边界检查
- `tools/check-clippy-warning-budget.ps1` — Clippy warning 预算
- `tools/check-executor-warning-budget.ps1` — 执行端 warning 预算
- `tools/check-clean-worktree.ps1` — 干净工作区检查
- `tools/check-gates-smoke.ps1` — 门禁冒烟
- `tools/check-full-feature-tree.ps1` — 全量树校验 🆕 v4.0.0
- `tools/run-smart-pre-commit.ps1` — staged-file 智能 pre-commit 分流
- `tools/update-recursive-governance.ps1` — 递归治理 skeleton 与索引生成器

**其他工具**:
- `tools/run-closeout-gates.bat` — 一键 26 项 closeout
- `tools/export-capability-fixture.ps1` — 能力 fixture 导出
- `tools/cleanup-artifacts.ps1` — 工件清理
- `tools/track-gate-metrics.ps1` — 门禁指标追踪, 支持 closeout 前 DryRun 和 NDJSON 记录
- `tools/build_package.js` — 打包脚本
- `tools/check-qs.js` — QS 检查
- `tools/destructive_test.js` — 破坏性测试
- `tools/gen_screenshot_data.js` — 截图数据生成
- `tools/generate-test-report.js` — 测试报告生成
- `tools/run-all-scenarios.js` — 全场景运行
- `tools/run-scenario.js` — 单场景运行
- `tools/testnet_long_test.js` — 测试网长时间测试

**脚本** (`scripts/`):
- `scripts/test.ps1` — Rust 测试 PowerShell 脚本
- `scripts/scenario-smoke.ps1` — QS 场景冒烟

### E.9 合约与配置

- `contracts/openapi/root.yaml` — OpenAPI 根定义, 含 `/api/runtime/v4/run`、`/api/runtime/backtest` 与 OKX demo provider submit/query/cancel 契约 🆕 v4.8.0
- `contracts/asyncapi/runtime-events.yaml` — AsyncAPI 运行时事件
- `contracts/governance/.spectral.yaml` — API 治理规则
- `config/runtime_protocol.example.yaml` — 运行时协议示例
- `config/strategy_ir.v0.example.json` — 策略 IR 示例
- `config/strategy_ir.v0.schema.json` — 策略 IR JSON Schema
- `release/release-manifest.yaml` — 发布清单
- `plugins/builtin/data/kline/plugin.json` — K线插件清单

---

## 附录 F: 全量树维护检查命令

### F.1 校验全量树完整性

```powershell
# 全量树校验 (路径存在性 / 文件覆盖率 / 占位符 / 版本号)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
```

### F.2 检查新增文件是否已在全量树中

```powershell
# 列出未被全量树覆盖的 active 文件
# (check-full-feature-tree.ps1 第5步自动输出)
```

### F.3 手动检查某个功能是否可反查

```bash
# 示例: 从"凭证保险库"反查代码路径
grep -n "凭证保险库" markdown/10-overview/overview-full-feature-tree.md

# 示例: 从文件反查其在全量树中的说明
grep -n "credential_vault" markdown/10-overview/overview-full-feature-tree.md
```

### F.4 新增文件后的维护流程

1. 确定文件属于哪个根节点/子系统
2. 在对应章节的文件列表中新增一行: `- path/to/file.ext — 做什么; 什么时候改这里`
3. 如文件引入新功能, 在附录 E 对应分组中新增条目
4. 运行 `check-full-feature-tree.ps1` 确认通过
5. 若文件在排除列表中, 更新 `tools/full-feature-tree-excludes.txt`

### F.5 删除/重命名文件后的维护流程

1. 在全量树中搜索旧文件名/路径
2. 更新或删除对应节点
3. 若重命名, 同时更新附录 E 中的条目
4. 运行 `check-full-feature-tree.ps1` 确认通过
- `markdown/06-milestones/v4.16.0/376-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass单子叶等价基线.md` - v4.16.0 BE-001DY-01 `rollback_record_identity_import_pass` 单子叶等价基线，冻结 rollback id import pass 输入面
当前最新递归点补充: BE-001DY-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线；下一步只能进入 BE-001DY-02 抽离方案。
- `markdown/06-milestones/v4.16.0/377-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass抽离方案.md` - v4.16.0 BE-001DY-02 `rollback_record_identity_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001DY-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离方案；下一步只能进入 BE-001DY-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/378-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass抽离记录.md` - v4.16.0 BE-001DY-03 `rollback_record_identity_import_pass` 抽离记录，移除 rollback_record_identity parent wildcard import
当前最新递归点补充: BE-001DY-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 实际抽离；下一步只能进入 BE-001DY-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/379-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass单叶closeout.md` - v4.16.0 BE-001DY-04 `rollback_record_identity_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001DY-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单叶 closeout；下一步只能进入 BE-001DZ-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/380-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md` - v4.16.0 BE-001DZ-01 `transition_lifecycle_import_pass` 父叶残余判断，选择 transition_record_persistence import pass
当前最新递归点补充: BE-001DZ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断；下一步只能进入 BE-001EA-01 `transition_record_persistence_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/381-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单子叶等价基线.md` - v4.16.0 BE-001EA-01 `transition_record_persistence_import_pass` 单子叶等价基线，冻结 lifecycle persistence 输入面
当前最新递归点补充: BE-001EA-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线；下一步只能进入 BE-001EA-02 抽离方案。
- `markdown/06-milestones/v4.16.0/382-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass抽离方案.md` - v4.16.0 BE-001EA-02 `transition_record_persistence_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001EA-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案；下一步只能进入 BE-001EA-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/383-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass抽离记录.md` - v4.16.0 BE-001EA-03 `transition_record_persistence_import_pass` 抽离记录，移除 transition_record_persistence parent wildcard import
当前最新递归点补充: BE-001EA-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 实际抽离；下一步只能进入 BE-001EA-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/384-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单叶closeout.md` - v4.16.0 BE-001EA-04 `transition_record_persistence_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001EA-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单叶 closeout；下一步只能进入 BE-001EB-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/385-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第三轮父叶残余判断.md` - v4.16.0 BE-001EB-01 `transition_lifecycle_import_pass` 第三轮父叶残余判断，选择 activation_snapshot_side_effect import pass
当前最新递归点补充: BE-001EB-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第三轮父叶残余判断；下一步只能进入 BE-001EC-01 `activation_snapshot_side_effect_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/386-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单子叶等价基线.md` - v4.16.0 BE-001EC-01 `activation_snapshot_side_effect_import_pass` 单子叶等价基线，冻结 activation snapshot side-effect 输入面
当前最新递归点补充: BE-001EC-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线；下一步只能进入 BE-001EC-02 抽离方案。
- `markdown/06-milestones/v4.16.0/387-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass抽离方案.md` - v4.16.0 BE-001EC-02 `activation_snapshot_side_effect_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001EC-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案；下一步只能进入 BE-001EC-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/388-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass抽离记录.md` - v4.16.0 BE-001EC-03 `activation_snapshot_side_effect_import_pass` 抽离记录，移除 activation_snapshot_side_effect parent wildcard import
当前最新递归点补充: BE-001EC-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 实际抽离；下一步只能进入 BE-001EC-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/389-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单叶closeout.md` - v4.16.0 BE-001EC-04 `activation_snapshot_side_effect_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001EC-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单叶 closeout；下一步只能进入 BE-001ED-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/390-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第四轮父叶残余判断.md` - v4.16.0 BE-001ED-01 `transition_lifecycle_import_pass` 第四轮父叶残余判断，选择 activation_flow import pass
当前最新递归点补充: BE-001ED-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断；下一步只能进入 BE-001EE-01 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/391-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass单子叶等价基线.md` - v4.16.0 BE-001EE-01 `activation_flow_import_pass` 单子叶等价基线，冻结 activation flow 输入面
当前最新递归点补充: BE-001EE-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线；下一步只能进入 BE-001EE-02 抽离方案。
- `markdown/06-milestones/v4.16.0/392-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离方案.md` - v4.16.0 BE-001EE-02 `activation_flow_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001EE-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案；下一步只能进入 BE-001EE-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/393-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离记录.md` - v4.16.0 BE-001EE-03 `activation_flow_import_pass` 抽离记录，移除 activation_flow parent wildcard import
当前最新递归点补充: BE-001EE-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 实际抽离；下一步只能进入 BE-001EE-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/394-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass单叶closeout.md` - v4.16.0 BE-001EE-04 `activation_flow_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001EE-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单叶 closeout；下一步只能进入 BE-001EF-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/395-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第五轮父叶残余判断.md` - v4.16.0 BE-001EF-01 `transition_lifecycle_import_pass` 第五轮父叶残余判断，选择 rollback_flow import pass
当前最新递归点补充: BE-001EF-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第五轮父叶残余判断；下一步只能进入 BE-001EG-01 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/396-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单子叶等价基线.md` - v4.16.0 BE-001EG-01 `rollback_flow_import_pass` 单子叶等价基线，冻结 rollback flow 输入面
当前最新递归点补充: BE-001EG-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线；下一步只能进入 BE-001EG-02 抽离方案。
- `markdown/06-milestones/v4.16.0/397-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离方案.md` - v4.16.0 BE-001EG-02 `rollback_flow_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001EG-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离方案；下一步只能进入 BE-001EG-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/398-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离记录.md` - v4.16.0 BE-001EG-03 `rollback_flow_import_pass` 抽离记录，移除 rollback_flow parent wildcard import
当前最新递归点补充: BE-001EG-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 实际抽离；下一步只能进入 BE-001EG-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/399-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单叶closeout.md` - v4.16.0 BE-001EG-04 `rollback_flow_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001EG-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单叶 closeout；下一步只能进入 BE-001EH-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/400-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第六轮父叶残余判断.md` - v4.16.0 BE-001EH-01 `transition_lifecycle_import_pass` 第六轮父叶残余判断，选择 parent_facade import pass
当前最新递归点补充: BE-001EH-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断；下一步只能进入 BE-001EI-01 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/401-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass单子叶等价基线.md` - v4.16.0 BE-001EI-01 `parent_facade_import_pass` 单子叶等价基线，冻结 parent facade 输入面
当前最新递归点补充: BE-001EI-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线；下一步只能进入 BE-001EI-02 抽离方案。
- `markdown/06-milestones/v4.16.0/402-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离方案.md` - v4.16.0 BE-001EI-02 `parent_facade_import_pass` 抽离方案，固定单文件 import rewrite
当前最新递归点补充: BE-001EI-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案；下一步只能进入 BE-001EI-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/403-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离记录.md` - v4.16.0 BE-001EI-03 `parent_facade_import_pass` 抽离记录，清理 transition_lifecycle parent wildcard import
当前最新递归点补充: BE-001EI-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 实际抽离；下一步只能进入 BE-001EI-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/404-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass单叶closeout.md` - v4.16.0 BE-001EI-04 `parent_facade_import_pass` 单叶 closeout，设置 stop_split true
当前最新递归点补充: BE-001EI-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout；下一步只能进入 BE-001EJ-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/405-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第七轮父叶残余判断.md` - v4.16.0 BE-001EJ-01 `transition_lifecycle_import_pass` 第七轮父叶残余判断，设置 stop_split true
当前最新递归点补充: BE-001EJ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第七轮父叶残余判断；下一步只能进入 BE-001EK-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/406-runtime.mutation.parameter_mutation_import_pass第三轮父叶残余判断.md` - v4.16.0 BE-001EK-01 `parameter_mutation_import_pass` 第三轮父叶残余判断，选择 parent_facade import pass
当前最新递归点补充: BE-001EK-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第三轮父叶残余判断；下一步只能进入 BE-001EL-01 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/407-runtime.mutation.parameter_mutation.parent_facade_import_pass单子叶等价基线.md` - v4.16.0 BE-001EL-01 `parameter_mutation.parent_facade_import_pass` 单子叶等价基线，冻结 parent facade 输入面
递归边界补充: BE-001EL-01 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线；下一步只能进入 BE-001EL-02 抽离方案。
- `markdown/06-milestones/v4.16.0/408-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离方案.md` - v4.16.0 BE-001EL-02 `parameter_mutation.parent_facade_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001EL-02 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案；下一步只能进入 BE-001EL-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/409-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离记录.md` - v4.16.0 BE-001EL-03 `parameter_mutation.parent_facade_import_pass` 抽离记录，清理 parameter_mutation parent wildcard import
递归边界补充: BE-001EL-03 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 实际抽离；下一步只能进入 BE-001EL-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/410-runtime.mutation.parameter_mutation.parent_facade_import_pass单叶closeout.md` - v4.16.0 BE-001EL-04 `parameter_mutation.parent_facade_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001EL-04 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout；下一步只能进入 BE-001EM-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/411-runtime.mutation.parameter_mutation_import_pass第四轮父叶残余判断.md` - v4.16.0 BE-001EM-01 `parameter_mutation_import_pass` 第四轮父叶残余判断，设置 stop_split true
递归边界补充: BE-001EM-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断；下一步只能进入 BE-001EN-01 `runtime.mutation_import_pass` 父叶残余判断。
- `markdown/06-milestones/v4.16.0/412-runtime.mutation_import_pass第二轮父叶残余判断.md` - v4.16.0 BE-001EN-01 `runtime.mutation_import_pass` 第二轮父叶残余判断，选择 ai_proposal import pass
递归边界补充: BE-001EN-01 已完成 `runtime.mutation_import_pass` 父叶残余判断；下一步只能进入 BE-001EO-01 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/413-runtime.mutation.ai_proposal_import_pass单子叶等价基线.md` - v4.16.0 BE-001EO-01 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线，冻结 ai proposal 输入面
递归边界补充: BE-001EO-01 已建立 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线；下一步只能进入 BE-001EO-02 抽离方案。
- `markdown/06-milestones/v4.16.0/414-runtime.mutation.ai_proposal_import_pass抽离方案.md` - v4.16.0 BE-001EO-02 `runtime.mutation.ai_proposal_import_pass` 抽离方案，选择 record_query import pass
递归边界补充: BE-001EO-02 已建立 `runtime.mutation.ai_proposal_import_pass` 抽离方案；下一步只能进入 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/415-runtime.mutation.ai_proposal.record_query_import_pass单子叶等价基线.md` - v4.16.0 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线，冻结 record_query 输入面
递归边界补充: BE-001EP-01 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线；下一步只能进入 BE-001EP-02 抽离方案。
- `markdown/06-milestones/v4.16.0/416-runtime.mutation.ai_proposal.record_query_import_pass抽离方案.md` - v4.16.0 BE-001EP-02 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001EP-02 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案；下一步只能进入 BE-001EP-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/417-runtime.mutation.ai_proposal.record_query_import_pass抽离记录.md` - v4.16.0 BE-001EP-03 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离记录，record_query import 已显式化
递归边界补充: BE-001EP-03 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 实际抽离；下一步只能进入 BE-001EP-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/418-runtime.mutation.ai_proposal.record_query_import_pass单叶closeout.md` - v4.16.0 BE-001EP-04 `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001EP-04 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout；下一步只能进入 BE-001EQ-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/419-runtime.mutation.ai_proposal_import_pass第三轮父叶残余判断.md` - v4.16.0 BE-001EQ-01 `runtime.mutation.ai_proposal_import_pass` 第三轮父叶残余判断，选择 source_governance_identity import pass
递归边界补充: BE-001EQ-01 已完成 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断；下一步只能进入 BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/420-runtime.mutation.ai_proposal.source_governance_identity_import_pass单子叶等价基线.md` - v4.16.0 BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线，冻结 source governance 输入面
递归边界补充: BE-001ER-01 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线；下一步只能进入 BE-001ER-02 抽离方案。
- `markdown/06-milestones/v4.16.0/421-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离方案.md` - v4.16.0 BE-001ER-02 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001ER-02 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案；下一步只能进入 BE-001ER-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/422-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离记录.md` - v4.16.0 BE-001ER-03 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离记录，source_governance_identity import 已显式化
递归边界补充: BE-001ER-03 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 实际抽离；下一步只能进入 BE-001ER-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/423-runtime.mutation.ai_proposal.source_governance_identity_import_pass单叶closeout.md` - v4.16.0 BE-001ER-04 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001ER-04 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout；下一步只能进入 BE-001ES-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/424-runtime.mutation.ai_proposal_import_pass第四轮父叶残余判断.md` - v4.16.0 BE-001ES-01 `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断，选择 static_check import pass
递归边界补充: BE-001ES-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断；下一步只能进入 BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/425-runtime.mutation.ai_proposal.static_check_import_pass单子叶等价基线.md` - v4.16.0 BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线，冻结 static check 输入面
递归边界补充: BE-001ET-01 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线；下一步只能进入 BE-001ET-02 抽离方案。
- `markdown/06-milestones/v4.16.0/426-runtime.mutation.ai_proposal.static_check_import_pass抽离方案.md` - v4.16.0 BE-001ET-02 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001ET-02 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案；下一步只能进入 BE-001ET-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/427-runtime.mutation.ai_proposal.static_check_import_pass抽离记录.md` - v4.16.0 BE-001ET-03 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离记录，static_check import 已显式化
递归边界补充: BE-001ET-03 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 实际抽离；下一步只能进入 BE-001ET-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/428-runtime.mutation.ai_proposal.static_check_import_pass单叶closeout.md` - v4.16.0 BE-001ET-04 `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001ET-04 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout；下一步只能进入 BE-001EU-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/429-runtime.mutation.ai_proposal_import_pass第五轮父叶残余判断.md` - v4.16.0 BE-001EU-01 `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断，选择 event_lifecycle import pass
递归边界补充: BE-001EU-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断；下一步只能进入 BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/430-runtime.mutation.ai_proposal.event_lifecycle_import_pass单子叶等价基线.md` - v4.16.0 BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线，冻结 event lifecycle 输入面
递归边界补充: BE-001EV-01 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线；下一步只能进入 BE-001EV-02 抽离方案。
- `markdown/06-milestones/v4.16.0/431-runtime.mutation.ai_proposal.event_lifecycle_import_pass抽离方案.md` - v4.16.0 BE-001EV-02 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001EV-02 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案；下一步只能进入 BE-001EV-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/432-runtime.mutation.ai_proposal.event_lifecycle_import_pass抽离记录.md` - v4.16.0 BE-001EV-03 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离记录，event_lifecycle import 已显式化
递归边界补充: BE-001EV-03 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 实际抽离；下一步只能进入 BE-001EV-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/433-runtime.mutation.ai_proposal.event_lifecycle_import_pass单叶closeout.md` - v4.16.0 BE-001EV-04 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001EV-04 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout；下一步只能进入 BE-001EW-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/434-runtime.mutation.ai_proposal_import_pass第六轮父叶残余判断.md` - v4.16.0 BE-001EW-01 `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断，选择 approval_persistence import pass
递归边界补充: BE-001EW-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断；下一步只能进入 BE-001EX-01 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/435-runtime.mutation.ai_proposal.approval_persistence_import_pass单子叶等价基线.md` - v4.16.0 BE-001EX-01 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线，冻结 approval persistence 输入面
递归边界补充: BE-001EX-01 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线；下一步只能进入 BE-001EX-02 抽离方案。
- `markdown/06-milestones/v4.16.0/436-runtime.mutation.ai_proposal.approval_persistence_import_pass抽离方案.md` - v4.16.0 BE-001EX-02 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001EX-02 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案；下一步只能进入 BE-001EX-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/437-runtime.mutation.ai_proposal.approval_persistence_import_pass抽离记录.md` - v4.16.0 BE-001EX-03 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离记录，approval_persistence import 已显式化
递归边界补充: BE-001EX-03 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 实际抽离；下一步只能进入 BE-001EX-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/438-runtime.mutation.ai_proposal.approval_persistence_import_pass单叶closeout.md` - v4.16.0 BE-001EX-04 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001EX-04 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout；下一步只能进入 BE-001EY-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/439-runtime.mutation.ai_proposal_import_pass第七轮父叶残余判断.md` - v4.16.0 BE-001EY-01 `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断，选择 status_transition import pass
递归边界补充: BE-001EY-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断；下一步只能进入 BE-001EZ-01 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/440-runtime.mutation.ai_proposal.status_transition_import_pass单子叶等价基线.md` - v4.16.0 BE-001EZ-01 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线，冻结 status transition 输入面
递归边界补充: BE-001EZ-01 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线；下一步只能进入 BE-001EZ-02 抽离方案。
- `markdown/06-milestones/v4.16.0/441-runtime.mutation.ai_proposal.status_transition_import_pass抽离方案.md` - v4.16.0 BE-001EZ-02 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001EZ-02 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案；下一步只能进入 BE-001EZ-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/442-runtime.mutation.ai_proposal.status_transition_import_pass抽离记录.md` - v4.16.0 BE-001EZ-03 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离记录，status_transition import 已显式化
递归边界补充: BE-001EZ-03 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 实际抽离；下一步只能进入 BE-001EZ-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/443-runtime.mutation.ai_proposal.status_transition_import_pass单叶closeout.md` - v4.16.0 BE-001EZ-04 `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001EZ-04 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout；下一步只能进入 BE-001FA-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/444-runtime.mutation.ai_proposal_import_pass第八轮父叶残余判断.md` - v4.16.0 BE-001FA-01 `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断，选择 sandbox_trigger import pass
递归边界补充: BE-001FA-01 已建立第八轮父叶残余判断；下一步只能进入 BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 等价基线。
- `markdown/06-milestones/v4.16.0/445-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单子叶等价基线.md` - v4.16.0 BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线，冻结 sandbox gate 与 async retry side effect
递归边界补充: BE-001FB-01 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 等价基线；下一步只能进入 BE-001FB-02 抽离方案。
- `markdown/06-milestones/v4.16.0/446-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离方案.md` - v4.16.0 BE-001FB-02 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001FB-02 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案；下一步只能进入 BE-001FB-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/447-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离记录.md` - v4.16.0 BE-001FB-03 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离记录，sandbox_trigger import 已显式化
递归边界补充: BE-001FB-03 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 实际抽离；下一步只能进入 BE-001FB-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/448-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单叶closeout.md` - v4.16.0 BE-001FB-04 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001FB-04 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout；下一步只能进入 BE-001FC-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/449-runtime.mutation.ai_proposal_import_pass第九轮父叶残余判断.md` - v4.16.0 BE-001FC-01 `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断，选择 approval_review import pass
递归边界补充: BE-001FC-01 已建立第九轮父叶残余判断；下一步只能进入 BE-001FD-01 `runtime.mutation.ai_proposal.approval_review_import_pass` 等价基线。
递归边界补充: BE-001FD-01 已建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线；下一步只能进入 BE-001FD-02 抽离方案，不得直接改写函数体、锁顺序、approval lifecycle、proposal status transition、route facade 或发布过渡。
递归边界补充: BE-001FD-02 已建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案；BE-001FD-03 只允许单文件 import rewrite，不得迁移 handler owner、route facade、state/persistence/schema/frontend owner 或启动 release transition。
递归边界补充: BE-001FD-03 已完成 `runtime.mutation.ai_proposal.approval_review_import_pass` 单文件 import rewrite；`approval_review.rs` 已显式化输入面，但 proposal_creation、ai proposal parent facade、mutation import 与 root parent bridge 仍未完成。
递归边界补充: BE-001FD-04 已完成 `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout；本叶不继续拆 approval filter / lock / lifecycle 微叶，下一步回到 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断。
递归边界补充: BE-001FE-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断；父叶继续 `stop_split: false`，下一步只能进入 BE-001FF-01 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 等价基线。
- `markdown/06-milestones/v4.16.0/454-runtime.mutation.ai_proposal_import_pass第十轮父叶残余判断.md` - v4.16.0 BE-001FE-01 `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断，选择 proposal_creation import pass
递归边界补充: BE-001FF-01 已建立 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线；下一步只能进入 BE-001FF-02 抽离方案，不得直接改写函数体、自动审批、事件生命周期、persist order、sandbox trigger、parent facade 或发布过渡。
- `markdown/06-milestones/v4.16.0/455-runtime.mutation.ai_proposal.proposal_creation_import_pass单子叶等价基线.md` - v4.16.0 BE-001FF-01 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线，冻结 create handler import 输入面
递归边界补充: BE-001FF-02 已建立 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案；BE-001FF-03 只允许单文件 import rewrite，不得迁移 handler owner、route facade、state/persistence/schema/frontend owner 或启动 release transition。
- `markdown/06-milestones/v4.16.0/456-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离方案.md` - v4.16.0 BE-001FF-02 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001FF-03 已完成 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 实际抽离；下一步只能进入 BE-001FF-04 单叶 closeout，不得跳到 parent facade 或 root bridge 完成声明。
- `markdown/06-milestones/v4.16.0/457-runtime.mutation.ai_proposal.proposal_creation_import_pass抽离记录.md` - v4.16.0 BE-001FF-03 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离记录，`proposal_creation.rs` import 显式化
递归边界补充: BE-001FF-04 已完成 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout；设置 `stop_split: true`，下一步只能回到父叶 residual 判断。
- `markdown/06-milestones/v4.16.0/458-runtime.mutation.ai_proposal.proposal_creation_import_pass单叶closeout.md` - v4.16.0 BE-001FF-04 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout，停止继续细拆 create handler import pocket
递归边界补充: BE-001FG-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第十一轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001FH-01 parent facade 等价基线。
- `markdown/06-milestones/v4.16.0/459-runtime.mutation.ai_proposal_import_pass第十一轮父叶残余判断.md` - v4.16.0 BE-001FG-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断，选择 parent facade import pass
递归边界补充: BE-001FH-01 已建立 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线；下一步只能进入 BE-001FH-02 抽离方案，不得直接改写 Rust import。
- `markdown/06-milestones/v4.16.0/460-runtime.mutation.ai_proposal.parent_facade_import_pass单子叶等价基线.md` - v4.16.0 BE-001FH-01 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线，冻结 parent facade 输入面
递归边界补充: BE-001FH-02 已建立 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案；下一步只能进入 BE-001FH-03 实际抽离记录，不得越过单文件 import rewrite。
- `markdown/06-milestones/v4.16.0/461-runtime.mutation.ai_proposal.parent_facade_import_pass抽离方案.md` - v4.16.0 BE-001FH-02 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案，固定单文件 import rewrite
递归边界补充: BE-001FH-03 已完成 `runtime.mutation.ai_proposal.parent_facade_import_pass` 实际抽离；下一步只能进入 BE-001FH-04 单叶 closeout，不得直接声明 ai proposal import pass 完成。
- `markdown/06-milestones/v4.16.0/462-runtime.mutation.ai_proposal.parent_facade_import_pass抽离记录.md` - v4.16.0 BE-001FH-03 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离记录，parent facade import 已显式化
递归边界补充: BE-001FH-04 已完成 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能回到 BE-001FI-01 父叶 residual 判断。
- `markdown/06-milestones/v4.16.0/463-runtime.mutation.ai_proposal.parent_facade_import_pass单叶closeout.md` - v4.16.0 BE-001FH-04 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout，停止继续细拆 parent facade import pocket
递归边界补充: BE-001FI-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第十二轮父叶残余判断；父叶设置 `stop_split: true`，下一步只能进入 BE-001FJ-01 `runtime.mutation_import_pass` 父叶 residual 判断。
- `markdown/06-milestones/v4.16.0/464-runtime.mutation.ai_proposal_import_pass第十二轮父叶残余判断.md` - v4.16.0 BE-001FI-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断，ai proposal import pass 收口
递归边界补充: BE-001FJ-01 已完成 `runtime.mutation_import_pass` 第三轮父叶残余判断；父叶设置 `stop_split: true`，下一步只能进入 BE-001FK-01 `runtime.parent_import_bridge` 父叶 residual 判断。
- `markdown/06-milestones/v4.16.0/465-runtime.mutation_import_pass第三轮父叶残余判断.md` - v4.16.0 BE-001FJ-01 `runtime.mutation_import_pass` 父叶残余判断，mutation import pass 收口
递归边界补充: BE-001FK-01 已完成 `runtime.parent_import_bridge` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/466-runtime.parent_import_bridge第四轮父叶残余判断.md` - v4.16.0 BE-001FK-01 `runtime.parent_import_bridge` 父叶残余判断，选择 root parent facade import pass
递归边界补充: BE-001FL-01 已建立 `runtime.root_parent_facade_import_pass` 单子叶等价基线；当前 `no code movement`，`src/runtime/mod.rs` 尚未改写，下一步只能进入 BE-001FL-02 抽离方案。
- `markdown/06-milestones/v4.16.0/467-runtime.root_parent_facade_import_pass单子叶等价基线.md` - v4.16.0 BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线，冻结 root facade 输入面
递归边界补充: BE-001FL-02 已建立 `runtime.root_parent_facade_import_pass` 抽离方案；下一步只能进入 BE-001FL-03 单文件 root import cleanup。
- `markdown/06-milestones/v4.16.0/468-runtime.root_parent_facade_import_pass抽离方案.md` - v4.16.0 BE-001FL-02 `runtime.root_parent_facade_import_pass` 抽离方案，固定 single file import cleanup
递归边界补充: BE-001FL-03 已完成 `runtime.root_parent_facade_import_pass` 实际抽离；`src/runtime/mod.rs` root import residual 已清除，下一步只能进入 BE-001FL-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/469-runtime.root_parent_facade_import_pass抽离记录.md` - v4.16.0 BE-001FL-03 `runtime.root_parent_facade_import_pass` 抽离记录，root import residual 清除
递归边界补充: BE-001FL-04 已完成 `runtime.root_parent_facade_import_pass` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001FM-01 `runtime.parent_import_bridge` 父叶 residual 判断。
- `markdown/06-milestones/v4.16.0/470-runtime.root_parent_facade_import_pass单叶closeout.md` - v4.16.0 BE-001FL-04 `runtime.root_parent_facade_import_pass` 单叶 closeout，root facade import pocket 收口
递归边界补充: BE-001FM-01 已完成 `runtime.parent_import_bridge` 第五轮父叶残余判断；生产级 parent wildcard residual 为 0，下一步只能进入 BE-001FN-01 `backend.runtime` 父叶判断。
- `markdown/06-milestones/v4.16.0/471-runtime.parent_import_bridge第五轮父叶残余判断.md` - v4.16.0 BE-001FM-01 `runtime.parent_import_bridge` 父叶残余判断，生产级 parent bridge 收口
递归边界补充: BE-001FN-01 已完成 `backend.runtime` 第十轮父叶残余判断；`backend.runtime stop_split: true`，下一步只能进入 BE-001FO-01 `backend` 父叶判断。
- `markdown/06-milestones/v4.16.0/472-backend.runtime第十轮父叶残余判断.md` - v4.16.0 BE-001FN-01 `backend.runtime` 父叶残余判断，runtime 顶层父叶收口
递归边界补充: BE-001FO-01 已完成 `backend` 父叶残余判断；`backend stop_split: false`，下一步只能进入 BE-001FP-01 `backend.graph_compile` 父叶判断。
- `markdown/06-milestones/v4.16.0/473-backend父叶残余判断.md` - v4.16.0 BE-001FO-01 `backend` 父叶残余判断，选择 backend.graph_compile
递归边界补充: BE-001FP-01 已完成 `backend.graph_compile` 父叶残余判断；`backend.graph_compile stop_split: false`，下一步只能进入 BE-001FQ-01 `backend.graph_compile.quantscript_graph` 等价基线。
- `markdown/06-milestones/v4.16.0/474-backend.graph_compile父叶残余判断.md` - v4.16.0 BE-001FP-01 `backend.graph_compile` 父叶残余判断，选择 quantscript_graph
递归边界补充: BE-001FQ-01 已建立 `backend.graph_compile.quantscript_graph` 单子叶等价基线；下一步只能进入 BE-001FQ-02 抽离方案。
- `markdown/06-milestones/v4.16.0/475-backend.graph_compile.quantscript_graph单子叶等价基线.md` - v4.16.0 BE-001FQ-01 `backend.graph_compile.quantscript_graph` 等价基线，冻结 QS graph route/helper 面
递归边界补充: BE-001FQ-02 已建立 `backend.graph_compile.quantscript_graph` 抽离方案；下一步只能进入 BE-001FQ-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/476-backend.graph_compile.quantscript_graph抽离方案.md` - v4.16.0 BE-001FQ-02 `backend.graph_compile.quantscript_graph` 抽离方案，固定 planned move 与 root parent re-export
递归边界补充: BE-001FQ-03 已完成 `backend.graph_compile.quantscript_graph` 实际抽离；下一步只能进入 BE-001FQ-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/477-backend.graph_compile.quantscript_graph抽离记录.md` - v4.16.0 BE-001FQ-03 `backend.graph_compile.quantscript_graph` 抽离记录，真实 owner 迁入 backend child
递归边界补充: BE-001FQ-04 已完成 `backend.graph_compile.quantscript_graph` 单叶 closeout；本叶保持 `stop_split: false`，下一步只能进入 BE-001FR-01 `graph_to_qs_generation` 等价基线。
- `markdown/06-milestones/v4.16.0/478-backend.graph_compile.quantscript_graph单叶closeout.md` - v4.16.0 BE-001FQ-04 `backend.graph_compile.quantscript_graph` 单叶 closeout，选择 graph_to_qs_generation
递归边界补充: BE-001FR-01 已建立 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线；下一步只能进入 BE-001FR-02 抽离方案。
- `markdown/06-milestones/v4.16.0/479-backend.graph_compile.quantscript_graph.graph_to_qs_generation单子叶等价基线.md` - v4.16.0 BE-001FR-01 `graph_to_qs_generation` 等价基线，冻结 graph-to-QS generator
递归边界补充: BE-001FR-02 已建立 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 抽离方案；下一步只能进入 BE-001FR-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/480-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离方案.md` - v4.16.0 BE-001FR-02 `graph_to_qs_generation` 抽离方案，固定 planned child
递归边界补充: BE-001FR-03 已完成 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 实际抽离；下一步只能进入 BE-001FR-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/481-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离记录.md` - v4.16.0 BE-001FR-03 `graph_to_qs_generation` 抽离记录，child file 承接 generator
递归边界补充: BE-001FR-04 已完成 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单叶 closeout；下一步只能进入 BE-001FS-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/482-backend.graph_compile.quantscript_graph.graph_to_qs_generation单叶closeout.md` - v4.16.0 BE-001FR-04 `graph_to_qs_generation` 单叶 closeout，设置 stop_split true
递归边界补充: BE-001FS-01 已完成 `backend.graph_compile.quantscript_graph` 父叶残余判断；下一步只能进入 BE-001FT-01 formal module conversion 等价基线。
- `markdown/06-milestones/v4.16.0/483-backend.graph_compile.quantscript_graph父叶残余判断.md` - v4.16.0 BE-001FS-01 `quantscript_graph` 父叶残余判断，选择 formal_module_conversion
递归边界补充: BE-001FT-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线；下一步只能进入 BE-001FT-02 抽离方案，不得直接创建 child file 或移动 `convert_graph_json_to_script_module`。
- `markdown/06-milestones/v4.16.0/484-backend.graph_compile.quantscript_graph.formal_module_conversion单子叶等价基线.md` - v4.16.0 BE-001FT-01 `formal_module_conversion` 等价基线，冻结 graph JSON 到 `ScriptModule` conversion 语义
递归边界补充: BE-001FT-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion` 抽离方案；下一步只能进入 BE-001FT-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/485-backend.graph_compile.quantscript_graph.formal_module_conversion抽离方案.md` - v4.16.0 BE-001FT-02 `formal_module_conversion` 抽离方案，固定 planned child 与单函数迁移清单
递归边界补充: BE-001FT-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 实际抽离；下一步只能进入 BE-001FT-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/486-backend.graph_compile.quantscript_graph.formal_module_conversion抽离记录.md` - v4.16.0 BE-001FT-03 `formal_module_conversion` 抽离记录，child file 承接 formal conversion
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs` - backend graph compile quantscript graph formal conversion child，承接 `convert_graph_json_to_script_module`
递归边界补充: BE-001FT-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单叶 closeout；下一步只能进入 BE-001FU-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/487-backend.graph_compile.quantscript_graph.formal_module_conversion单叶closeout.md` - v4.16.0 BE-001FT-04 `formal_module_conversion` 单叶 closeout，确认等价并保持继续细拆
递归边界补充: BE-001FU-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 父叶残余判断；下一步只能进入 BE-001FV-01 `intent_lowering` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/488-backend.graph_compile.quantscript_graph.formal_module_conversion父叶残余判断.md` - v4.16.0 BE-001FU-01 `formal_module_conversion` 父叶残余判断，选择 `intent_lowering`
递归边界补充: BE-001FV-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单子叶等价基线；下一步只能进入 BE-001FV-02 抽离方案，不得直接创建 child file 或移动 intent 分支。
- `markdown/06-milestones/v4.16.0/489-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单子叶等价基线.md` - v4.16.0 BE-001FV-01 `intent_lowering` 等价基线，冻结 built-in intent lowering 语义
递归边界补充: BE-001FV-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 抽离方案；下一步只能进入 BE-001FV-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/490-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离方案.md` - v4.16.0 BE-001FV-02 `intent_lowering` 抽离方案，固定 planned child 与 helper signature
递归边界补充: BE-001FV-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 实际抽离；下一步只能进入 BE-001FV-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/491-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离记录.md` - v4.16.0 BE-001FV-03 `intent_lowering` 抽离记录，child file 承接 intent block
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` - backend graph compile formal conversion intent lowering child，承接 built-in intent branch lowering
递归边界补充: BE-001FV-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单叶 closeout；下一步只能进入 BE-001FW-01 父叶残余判断。
- `markdown/06-milestones/v4.16.0/492-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单叶closeout.md` - v4.16.0 BE-001FV-04 `intent_lowering` 单叶 closeout，确认等价并保持继续细拆
递归边界补充: BE-001FW-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；下一步只能进入 BE-001FX-01 `spread_observer_lowering` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/493-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001FW-01 `intent_lowering` 父叶残余判断，选择 `spread_observer_lowering`
递归边界补充: BE-001FX-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单子叶等价基线；下一步只能进入 BE-001FX-02 抽离方案。
- `markdown/06-milestones/v4.16.0/494-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单子叶等价基线.md` - v4.16.0 BE-001FX-01 `spread_observer_lowering` 等价基线，冻结 spread observer branch 语义
递归边界补充: BE-001FX-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 抽离方案；下一步只能进入 BE-001FX-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/495-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离方案.md` - v4.16.0 BE-001FX-02 `spread_observer_lowering` 抽离方案，固定 planned child 与 helper signature
递归边界补充: BE-001FX-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 实际抽离；下一步只能进入 BE-001FX-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/496-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离记录.md` - v4.16.0 BE-001FX-03 `spread_observer_lowering` 实际抽离记录
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs` - spread observer intent lowering child helper
递归边界补充: BE-001FX-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单叶 closeout；下一步只能进入 BE-001FY-01 `intent_lowering` 父叶残余判断。
- `markdown/06-milestones/v4.16.0/497-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单叶closeout.md` - v4.16.0 BE-001FX-04 `spread_observer_lowering` 单叶 closeout
递归边界补充: BE-001FY-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；下一步只能进入 BE-001FZ-01 `macd_lowering` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/498-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001FY-01 `intent_lowering` 父叶残余判断，选择 `macd_lowering`
递归边界补充: BE-001FZ-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单子叶等价基线；下一步只能进入 BE-001FZ-02 抽离方案。
- `markdown/06-milestones/v4.16.0/499-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering单子叶等价基线.md` - v4.16.0 BE-001FZ-01 `macd_lowering` 等价基线，冻结 MACD branch 语义
递归边界补充: BE-001FZ-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 抽离方案；下一步只能进入 BE-001FZ-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/500-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离方案.md` - v4.16.0 BE-001FZ-02 `macd_lowering` 抽离方案，固定 planned child 与 helper signature
递归边界补充: BE-001FZ-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 实际抽离；下一步只能进入 BE-001FZ-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/501-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离记录.md` - v4.16.0 BE-001FZ-03 `macd_lowering` 实际抽离记录
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs` - MACD intent lowering child helper
递归边界补充: BE-001FZ-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单叶 closeout；`macd_lowering stop_split: true`，下一步只能进入 BE-001GA-01 `intent_lowering` 父叶残余判断。
- `markdown/06-milestones/v4.16.0/502-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering单叶closeout.md` - v4.16.0 BE-001FZ-04 `macd_lowering` 单叶 closeout
递归边界补充: BE-001GA-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；`intent_lowering stop_split: false`，下一步只能进入 BE-001GB-01 `double_ma_lowering` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/503-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GA-01 `intent_lowering` 父叶残余判断，选择 `double_ma_lowering`
递归边界补充: BE-001GB-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001GB-02 抽离方案。
- `markdown/06-milestones/v4.16.0/504-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering单子叶等价基线.md` - v4.16.0 BE-001GB-01 `double_ma_lowering` 单子叶等价基线
递归边界补充: BE-001GB-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 抽离方案；下一步只能进入 BE-001GB-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/505-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering抽离方案.md` - v4.16.0 BE-001GB-02 `double_ma_lowering` 抽离方案
递归边界补充: BE-001GB-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 实际抽离；下一步只能进入 BE-001GB-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/506-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering抽离记录.md` - v4.16.0 BE-001GB-03 `double_ma_lowering` 实际抽离记录
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/double_ma_lowering.rs` - double MA intent lowering child helper
递归边界补充: BE-001GB-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单叶 closeout；`double_ma_lowering stop_split: true`，下一步只能进入 BE-001GC-01 `intent_lowering` 父叶残余判断。
- `markdown/06-milestones/v4.16.0/507-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering单叶closeout.md` - v4.16.0 BE-001GB-04 `double_ma_lowering` 单叶 closeout
递归边界补充: BE-001GC-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；`intent_lowering stop_split: false`，下一步只能进入 BE-001GD-01 `rsi_lowering` 单子叶等价基线。
- `markdown/06-milestones/v4.16.0/508-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GC-01 `intent_lowering` 父叶残余判断，选择 `rsi_lowering`
递归边界补充: BE-001GD-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001GD-02 抽离方案。
- `markdown/06-milestones/v4.16.0/509-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering单子叶等价基线.md` - v4.16.0 BE-001GD-01 `rsi_lowering` 单子叶等价基线
递归边界补充: BE-001GD-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 抽离方案；下一步只能进入 BE-001GD-03 实际抽离记录。
- `markdown/06-milestones/v4.16.0/510-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering抽离方案.md` - v4.16.0 BE-001GD-02 `rsi_lowering` 抽离方案
递归边界补充: BE-001GD-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 实际抽离；下一步只能进入 BE-001GD-04 单叶 closeout。
- `markdown/06-milestones/v4.16.0/511-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering抽离记录.md` - v4.16.0 BE-001GD-03 `rsi_lowering` 实际抽离记录
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs` - RSI intent lowering child helper
递归边界补充: BE-001GD-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单叶 closeout；`rsi_lowering stop_split: true`，下一步只能进入 BE-001GE-01 `intent_lowering` 父叶残余判断。
- `markdown/06-milestones/v4.16.0/512-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering单叶closeout.md` - v4.16.0 BE-001GD-04 `rsi_lowering` 单叶 closeout
递归治理补充: GOV-LEAF-SPLIT-GATE 已固化 `leaf_split_decision_gate`；后续新增单叶 closeout / 父叶残余判断必须显式触发叶子细分判定。
- `markdown/06-milestones/v4.16.0/513-递归叶子细分判定硬规则固化.md` - v4.16.0 GOV-LEAF-SPLIT-GATE 递归叶子细分判定硬规则
递归边界补充: BE-001GE-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects ma_deviation_lowering；下一步: BE-001GF-01 ma_deviation_lowering baseline_plan。
- `markdown/06-milestones/v4.16.0/514-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GE-01 intent_lowering parent residual judgment selects ma_deviation_lowering
递归边界补充: BE-001GF-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering baseline and extraction plan frozen；下一步: BE-001GF-02 ma_deviation_lowering extract_closeout。
- `markdown/06-milestones/v4.16.0/515-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering等价基线与抽离方案.md` - v4.16.0 BE-001GF-01 ma_deviation_lowering baseline and extraction plan frozen
递归边界补充: BE-001GF-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering actual extraction and closeout complete；下一步: BE-001GG-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/516-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering抽离与closeout.md` - v4.16.0 BE-001GF-02 ma_deviation_lowering actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/ma_deviation_lowering.rs` - MA deviation built-in intent lowering child
递归边界补充: BE-001GF-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering actual extraction and closeout complete；下一步: BE-001GG-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/516-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering抽离与closeout.md` - v4.16.0 BE-001GF-02 ma_deviation_lowering actual extraction and closeout complete
递归边界补充: BE-001GG-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects momentum_lowering；下一步: BE-001GH-01 momentum_lowering baseline_plan。
- `markdown/06-milestones/v4.16.0/517-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GG-01 intent_lowering parent residual judgment selects momentum_lowering
递归边界补充: BE-001GH-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering baseline and extraction plan frozen；下一步: BE-001GH-02 momentum_lowering extract_closeout。
- `markdown/06-milestones/v4.16.0/518-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering等价基线与抽离方案.md` - v4.16.0 BE-001GH-01 momentum_lowering baseline and extraction plan frozen
递归边界补充: BE-001GH-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering actual extraction and closeout complete；下一步: BE-001GI-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/519-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering抽离与closeout.md` - v4.16.0 BE-001GH-02 momentum_lowering actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/momentum_lowering.rs` - Momentum built-in intent lowering child
递归边界补充: BE-001GH-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering actual extraction and closeout complete；下一步: BE-001GI-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/519-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering抽离与closeout.md` - v4.16.0 BE-001GH-02 momentum_lowering actual extraction and closeout complete
递归边界补充: BE-001GI-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects zscore_lowering；下一步: BE-001GJ-01 zscore_lowering baseline_plan。
- `markdown/06-milestones/v4.16.0/520-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GI-01 intent_lowering parent residual judgment selects zscore_lowering
递归边界补充: BE-001GJ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering baseline and extraction plan frozen；下一步: BE-001GJ-02 zscore_lowering extract_closeout。
- `markdown/06-milestones/v4.16.0/521-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering等价基线与抽离方案.md` - v4.16.0 BE-001GJ-01 zscore_lowering baseline and extraction plan frozen
递归边界补充: BE-001GJ-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering actual extraction and closeout complete；下一步: BE-001GK-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/522-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering抽离与closeout.md` - v4.16.0 BE-001GJ-02 zscore_lowering actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/zscore_lowering.rs` - Zscore built-in intent lowering child
递归边界补充: BE-001GJ-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering actual extraction and closeout complete；下一步: BE-001GK-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/522-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering抽离与closeout.md` - v4.16.0 BE-001GJ-02 zscore_lowering actual extraction and closeout complete
递归边界补充: BE-001GK-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects shared_intent_context；下一步: BE-001GL-01 shared_intent_context baseline_plan。
- `markdown/06-milestones/v4.16.0/523-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` - v4.16.0 BE-001GK-01 intent_lowering parent residual judgment selects shared_intent_context
递归边界补充: BE-001GL-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context baseline and extraction plan frozen；下一步: BE-001GL-02 shared_intent_context extract_closeout。
- `markdown/06-milestones/v4.16.0/524-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context等价基线与抽离方案.md` - v4.16.0 BE-001GL-01 shared_intent_context baseline and extraction plan frozen
递归边界补充: BE-001GL-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context actual extraction and closeout complete；下一步: BE-001GM-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/525-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context抽离与closeout.md` - v4.16.0 BE-001GL-02 shared_intent_context actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/shared_intent_context.rs` - Shared intent lowering context child
递归边界补充: BE-001GL-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context actual extraction and closeout complete；下一步: BE-001GM-01 intent_lowering parent residual judgment。
- `markdown/06-milestones/v4.16.0/525-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context抽离与closeout.md` - v4.16.0 BE-001GL-02 shared_intent_context actual extraction and closeout complete
递归边界补充: BE-001GM-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects unsupported_intent_failure；下一步: BE-001GN-01 unsupported_intent_failure baseline_plan。
- `markdown/06-milestones/v4.16.0/526-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.parent_residual_judgment.md` - v4.16.0 BE-001GM-01 intent_lowering parent residual judgment selects unsupported_intent_failure
递归边界补充: BE-001GN-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure equivalence baseline and extraction plan；下一步: BE-001GN-02 unsupported_intent_failure extract_closeout。
- `markdown/06-milestones/v4.16.0/527-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure.baseline_plan.md` - v4.16.0 BE-001GN-01 unsupported_intent_failure equivalence baseline and extraction plan
递归边界补充: BE-001GN-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure actual extraction and closeout complete；下一步: BE-001GO-01 intent_lowering parent residual closeout。
- `markdown/06-milestones/v4.16.0/528-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure.extract_closeout.md` - v4.16.0 BE-001GN-02 unsupported_intent_failure actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/unsupported_intent_failure.rs` - Unsupported intent failure child helper, owns supported intent display string and hard bail diagnostic
递归边界补充: BE-001GO-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent closeout sets stop_split true；下一步: BE-001GP-01 formal_module_conversion parent residual judgment。
- `markdown/06-milestones/v4.16.0/529-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.parent_closeout.md` - v4.16.0 BE-001GO-01 intent_lowering parent closeout sets stop_split true
递归边界补充: BE-001GP-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects data_source_lowering；下一步: BE-001GQ-01 data_source_lowering baseline_plan。
- `markdown/06-milestones/v4.16.0/530-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_residual_judgment.md` - v4.16.0 BE-001GP-01 formal_module_conversion parent residual judgment selects data_source_lowering
递归边界补充: BE-001GQ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering equivalence baseline and extraction plan；下一步: BE-001GQ-02 data_source_lowering extract_closeout。
- `markdown/06-milestones/v4.16.0/531-backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering.baseline_plan.md` - v4.16.0 BE-001GQ-01 data_source_lowering equivalence baseline and extraction plan
递归边界补充: BE-001GQ-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering actual extraction and closeout complete；下一步: BE-001GR-01 formal_module_conversion parent residual judgment。
- `markdown/06-milestones/v4.16.0/532-backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering.extract_closeout.md` - v4.16.0 BE-001GQ-02 data_source_lowering actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/data_source_lowering.rs` - Data source lowering child, owns data node fetch argument construction and QS fetch line rendering
递归边界补充: BE-001GR-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects profile_lowering；下一步: BE-001GS-01 profile_lowering baseline_plan。
- `markdown/06-milestones/v4.16.0/533-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_residual_judgment.profile_lowering.md` - v4.16.0 BE-001GR-01 formal_module_conversion parent residual judgment selects profile_lowering
递归边界补充: BE-001GS-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering equivalence baseline and extraction plan；下一步: BE-001GS-02 profile_lowering extract_closeout。
- `markdown/06-milestones/v4.16.0/534-backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering.baseline_plan.md` - v4.16.0 BE-001GS-01 profile_lowering equivalence baseline and extraction plan
递归边界补充: BE-001GS-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering actual extraction and closeout complete；下一步: BE-001GT-01 formal_module_conversion parent residual judgment。
- `markdown/06-milestones/v4.16.0/535-backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering.extract_closeout.md` - v4.16.0 BE-001GS-02 profile_lowering actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/profile_lowering.rs` - Profile lowering child, owns risk/execution formal profile line rendering
递归边界补充: BE-001GT-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects input_shape_validation；下一步: BE-001GU-01 input_shape_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/536-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_residual_judgment.input_shape_validation.md` - v4.16.0 BE-001GT-01 formal_module_conversion parent residual judgment selects input_shape_validation
递归边界补充: BE-001GU-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation equivalence baseline and extraction plan；下一步: BE-001GU-02 input_shape_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/537-backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation.baseline_plan.md` - v4.16.0 BE-001GU-01 input_shape_validation equivalence baseline and extraction plan
递归边界补充: BE-001GU-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation actual extraction and closeout complete；下一步: BE-001GV-01 formal_module_conversion parent residual judgment。
- `markdown/06-milestones/v4.16.0/538-backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation.extract_closeout.md` - v4.16.0 BE-001GU-02 input_shape_validation actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/input_shape_validation.rs` - Input shape validation child, owns graph nodes/edges required-array checks
递归边界补充: BE-001GV-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects terminal_parse；下一步: BE-001GW-01 terminal_parse baseline_plan。
- `markdown/06-milestones/v4.16.0/539-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_residual_judgment.terminal_parse.md` - v4.16.0 BE-001GV-01 formal_module_conversion parent residual judgment selects terminal_parse
递归边界补充: BE-001GW-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse equivalence baseline and extraction plan；下一步: BE-001GW-02 terminal_parse extract_closeout。
- `markdown/06-milestones/v4.16.0/540-backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse.baseline_plan.md` - v4.16.0 BE-001GW-01 terminal_parse equivalence baseline and extraction plan
递归边界补充: BE-001GW-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse actual extraction and closeout complete；下一步: BE-001GX-01 formal_module_conversion parent residual judgment。
- `markdown/06-milestones/v4.16.0/541-backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse.extract_closeout.md` - v4.16.0 BE-001GW-02 terminal_parse actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/terminal_parse.rs` - Terminal parse child, owns closing brace, QS line join, and parse_quant_script_module invocation
递归边界补充: BE-001GX-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects unsupported_node_logging；下一步: BE-001GY-01 unsupported_node_logging baseline_plan。
- `markdown/06-milestones/v4.16.0/542-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_residual_judgment.unsupported_node_logging.md` - v4.16.0 BE-001GX-01 formal_module_conversion parent residual judgment selects unsupported_node_logging
递归边界补充: BE-001GY-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging equivalence baseline and extraction plan；下一步: BE-001GY-02 unsupported_node_logging extract_closeout。
- `markdown/06-milestones/v4.16.0/543-backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging.baseline_plan.md` - v4.16.0 BE-001GY-01 unsupported_node_logging equivalence baseline and extraction plan
递归边界补充: BE-001GY-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging actual extraction and closeout complete；下一步: BE-001GZ-01 formal_module_conversion parent closeout。
- `markdown/06-milestones/v4.16.0/544-backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging.extract_closeout.md` - v4.16.0 BE-001GY-02 unsupported_node_logging actual extraction and closeout complete
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/unsupported_node_logging.rs` - Unsupported node logging child, owns known-node no-op classification and unknown-node safe log message
递归边界补充: BE-001GZ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent closeout sets stop_split true；下一步: BE-001HA-01 quantscript_graph parent residual judgment。
- `markdown/06-milestones/v4.16.0/545-backend.graph_compile.quantscript_graph.formal_module_conversion.parent_closeout.md` - v4.16.0 BE-001GZ-01 formal_module_conversion parent closeout sets stop_split true
递归边界补充: BE-001HA-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects strategy_graph_parser；下一步: BE-001HB-01 strategy_graph_parser baseline_plan。
- `markdown/06-milestones/v4.16.0/546-backend.graph_compile.quantscript_graph.parent_residual_judgment.strategy_graph_parser.md` - v4.16.0 BE-001HA-01 quantscript_graph parent residual judgment selects strategy_graph_parser
递归边界补充: BE-001HB-01 `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser equivalence baseline and extraction plan；下一步: BE-001HB-02 strategy_graph_parser extract_closeout。
- `markdown/06-milestones/v4.16.0/547-backend.graph_compile.quantscript_graph.strategy_graph_parser.baseline_plan.md` - v4.16.0 BE-001HB-01 strategy_graph_parser equivalence baseline and extraction plan
递归边界补充: BE-001HB-02 `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser actual extraction and closeout complete；下一步: BE-001HC-01 quantscript_graph parent residual judgment。
- `markdown/06-milestones/v4.16.0/548-backend.graph_compile.quantscript_graph.strategy_graph_parser.extract_closeout.md` - v4.16.0 BE-001HB-02 strategy_graph_parser actual extraction and closeout complete
递归边界补充: BE-001HC-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects artifact_target_projection；下一步: BE-001HD-01 artifact_target_projection baseline_plan。
- `markdown/06-milestones/v4.16.0/549-backend.graph_compile.quantscript_graph.parent_residual_judgment.artifact_target_projection.md` - v4.16.0 BE-001HC-01 quantscript_graph parent residual judgment selects artifact_target_projection
递归边界补充: BE-001HD-01 `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection equivalence baseline and extraction plan；下一步: BE-001HD-02 artifact_target_projection extract_closeout。
- `markdown/06-milestones/v4.16.0/550-backend.graph_compile.quantscript_graph.artifact_target_projection.baseline_plan.md` - v4.16.0 BE-001HD-01 artifact_target_projection equivalence baseline and extraction plan
递归边界补充: BE-001HD-02 `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection actual extraction and closeout complete；下一步: BE-001HE-01 quantscript_graph parent residual judgment。
- `markdown/06-milestones/v4.16.0/551-backend.graph_compile.quantscript_graph.artifact_target_projection.extract_closeout.md` - v4.16.0 BE-001HD-02 artifact_target_projection actual extraction and closeout complete
递归边界补充: BE-001HE-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects route_surface；下一步: BE-001HF-01 route_surface baseline_plan。
- `markdown/06-milestones/v4.16.0/552-backend.graph_compile.quantscript_graph.parent_residual_judgment.route_surface.md` - v4.16.0 BE-001HE-01 quantscript_graph parent residual judgment selects route_surface
递归边界补充: BE-001HF-01 `backend.graph_compile.quantscript_graph.route_surface` route_surface equivalence baseline and extraction plan；下一步: BE-001HF-02 route_surface extract_closeout。
- `markdown/06-milestones/v4.16.0/553-backend.graph_compile.quantscript_graph.route_surface.baseline_plan.md` - v4.16.0 BE-001HF-01 route_surface equivalence baseline and extraction plan
递归边界补充: BE-001HF-02 `backend.graph_compile.quantscript_graph.route_surface` route_surface actual extraction and closeout complete；下一步: BE-001HG-01 quantscript_graph parent closeout。
- `markdown/06-milestones/v4.16.0/554-backend.graph_compile.quantscript_graph.route_surface.extract_closeout.md` - v4.16.0 BE-001HF-02 route_surface actual extraction and closeout complete
递归边界补充: BE-001HG-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent closeout sets stop_split true；下一步: BE-001HH-01 backend.graph_compile parent residual judgment。
- `markdown/06-milestones/v4.16.0/555-backend.graph_compile.quantscript_graph.parent_closeout.md` - v4.16.0 BE-001HG-01 quantscript_graph parent closeout sets stop_split true
递归边界补充: BE-001HH-01 `backend.graph_compile` backend.graph_compile parent residual judgment selects compile；下一步: BE-001HI-01 backend.graph_compile.compile baseline_plan。
- `markdown/06-milestones/v4.16.0/556-backend.graph_compile.parent_residual_judgment.md` - v4.16.0 BE-001HH-01 backend.graph_compile parent residual judgment selects compile
递归边界补充: BE-001HI-01 `backend.graph_compile.compile` backend.graph_compile.compile equivalence baseline and extraction plan；下一步: BE-001HI-02 backend.graph_compile.compile extract_closeout。
- `markdown/06-milestones/v4.16.0/557-backend.graph_compile.compile.baseline_plan.md` - v4.16.0 BE-001HI-01 backend.graph_compile.compile equivalence baseline and extraction plan
递归边界补充: BE-001HI-02 `backend.graph_compile.compile` backend.graph_compile.compile actual extraction and closeout complete；下一步: BE-001HJ-01 backend.graph_compile parent residual judgment。
- `markdown/06-milestones/v4.16.0/558-backend.graph_compile.compile.extract_closeout.md` - v4.16.0 BE-001HI-02 backend.graph_compile.compile actual extraction and closeout complete
递归边界补充: BE-001HJ-01 `backend.graph_compile` backend.graph_compile parent residual judgment selects graph；下一步: BE-001HK-01 backend.graph_compile.graph baseline_plan。
- `markdown/06-milestones/v4.16.0/559-backend.graph_compile.parent_residual_judgment.graph.md` - v4.16.0 BE-001HJ-01 backend.graph_compile parent residual judgment selects graph
递归边界补充: BE-001HK-01 `backend.graph_compile.graph` backend.graph_compile.graph equivalence baseline and extraction plan；下一步: BE-001HK-02 backend.graph_compile.graph extract_closeout。
- `markdown/06-milestones/v4.16.0/560-backend.graph_compile.graph.baseline_plan.md` - v4.16.0 BE-001HK-01 backend.graph_compile.graph equivalence baseline and extraction plan
递归边界补充: BE-001HK-02 `backend.graph_compile.graph` backend.graph_compile.graph actual extraction and closeout complete；下一步: BE-001HL-01 backend.graph_compile parent closeout。
- `markdown/06-milestones/v4.16.0/561-backend.graph_compile.graph.extract_closeout.md` - v4.16.0 BE-001HK-02 backend.graph_compile.graph actual extraction and closeout complete
递归边界补充: BE-001HL-01 `backend.graph_compile` backend.graph_compile parent closeout sets stop_split true；下一步: BE-001HM-01 backend parent residual judgment。
- `markdown/06-milestones/v4.16.0/562-backend.graph_compile.parent_closeout.md` - v4.16.0 BE-001HL-01 backend.graph_compile parent closeout sets stop_split true
递归边界补充: BE-001HM-01 `backend` backend parent residual judgment selects capability；下一步: BE-001HN-01 backend.capability baseline_plan。
- `markdown/06-milestones/v4.16.0/563-backend.parent_residual_judgment.capability.md` - v4.16.0 BE-001HM-01 backend parent residual judgment selects capability
递归边界补充: BE-001HN-01 `backend.capability` backend.capability equivalence baseline and extraction plan；下一步: BE-001HN-02 backend.capability extract_closeout。
- `markdown/06-milestones/v4.16.0/564-backend.capability.baseline_plan.md` - v4.16.0 BE-001HN-01 backend.capability equivalence baseline and extraction plan
递归边界补充: BE-001HN-02 `backend.capability` backend.capability actual extraction and closeout complete；下一步: BE-001HO-01 backend parent residual judgment。
- `markdown/06-milestones/v4.16.0/565-backend.capability.extract_closeout.md` - v4.16.0 BE-001HN-02 backend.capability actual extraction and closeout complete
递归边界补充: BE-001HO-01 `backend` backend parent residual judgment selects strategy_config；下一步: BE-001HP-01 backend.strategy_config parent residual judgment。
- `markdown/06-milestones/v4.16.0/566-backend.parent_residual_judgment.strategy_config.md` - v4.16.0 BE-001HO-01 backend parent residual judgment selects strategy_config
递归边界补充: BE-001HP-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects artifact；下一步: BE-001HQ-01 backend.strategy_config.artifact baseline_plan。
- `markdown/06-milestones/v4.16.0/567-backend.strategy_config.parent_residual_judgment.artifact.md` - v4.16.0 BE-001HP-01 backend.strategy_config parent residual judgment selects artifact
递归边界补充: BE-001HQ-01 `backend.strategy_config.artifact` backend.strategy_config.artifact equivalence baseline and extraction plan；下一步: BE-001HQ-02 backend.strategy_config.artifact extract_closeout。
- `markdown/06-milestones/v4.16.0/568-backend.strategy_config.artifact.baseline_plan.md` - v4.16.0 BE-001HQ-01 backend.strategy_config.artifact equivalence baseline and extraction plan
递归边界补充: BE-001HQ-02 `backend.strategy_config.artifact` backend.strategy_config.artifact route owner extraction complete；下一步: BE-001HR-01 backend.strategy_config.artifact parent residual judgment。
- `markdown/06-milestones/v4.16.0/569-backend.strategy_config.artifact.extract_closeout.md` - v4.16.0 BE-001HQ-02 backend.strategy_config.artifact route owner extraction complete
递归边界补充: BE-001HR-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects schema_model；下一步: BE-001HS-01 backend.strategy_config.artifact.schema_model baseline_plan。
- `markdown/06-milestones/v4.16.0/570-backend.strategy_config.artifact.parent_residual_judgment.schema_model.md` - v4.16.0 BE-001HR-01 backend.strategy_config.artifact parent residual judgment selects schema_model
递归边界补充: BE-001HS-01 `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan；下一步: BE-001HS-02 backend.strategy_config.artifact.schema_model extract_closeout。
- `markdown/06-milestones/v4.16.0/571-backend.strategy_config.artifact.schema_model.baseline_plan.md` - v4.16.0 BE-001HS-01 backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan
递归边界补充: BE-001HS-02 `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model actual extraction complete；下一步: BE-001HT-01 backend.strategy_config.artifact parent residual judgment。
- `markdown/06-milestones/v4.16.0/572-backend.strategy_config.artifact.schema_model.extract_closeout.md` - v4.16.0 BE-001HS-02 backend.strategy_config.artifact.schema_model actual extraction complete
递归边界补充: BE-001HT-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects domain_projection；下一步: BE-001HU-01 backend.strategy_config.artifact.domain_projection baseline_plan。
- `markdown/06-milestones/v4.16.0/573-backend.strategy_config.artifact.parent_residual_judgment.domain_projection.md` - v4.16.0 BE-001HT-01 backend.strategy_config.artifact parent residual judgment selects domain_projection
递归边界补充: BE-001HU-01 `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan；下一步: BE-001HU-02 backend.strategy_config.artifact.domain_projection extract_closeout。
- `markdown/06-milestones/v4.16.0/574-backend.strategy_config.artifact.domain_projection.baseline_plan.md` - v4.16.0 BE-001HU-01 backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan
递归边界补充: BE-001HU-02 `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection actual extraction complete；下一步: BE-001HV-01 backend.strategy_config.artifact parent residual judgment。
- `markdown/06-milestones/v4.16.0/575-backend.strategy_config.artifact.domain_projection.extract_closeout.md` - v4.16.0 BE-001HU-02 backend.strategy_config.artifact.domain_projection actual extraction complete
递归边界补充: BE-001HV-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects builder_core；下一步: BE-001HW-01 backend.strategy_config.artifact.builder_core baseline_plan。
- `markdown/06-milestones/v4.16.0/576-backend.strategy_config.artifact.parent_residual_judgment.builder_core.md` - v4.16.0 BE-001HV-01 backend.strategy_config.artifact parent residual judgment selects builder_core
递归边界补充: BE-001HW-01 `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan；下一步: BE-001HW-02 backend.strategy_config.artifact.builder_core extract_closeout。
- `markdown/06-milestones/v4.16.0/577-backend.strategy_config.artifact.builder_core.baseline_plan.md` - v4.16.0 BE-001HW-01 backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan
递归边界补充: BE-001HW-02 `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core actual extraction complete；下一步: BE-001HX-01 backend.strategy_config.artifact parent closeout。
- `markdown/06-milestones/v4.16.0/578-backend.strategy_config.artifact.builder_core.extract_closeout.md` - v4.16.0 BE-001HW-02 backend.strategy_config.artifact.builder_core actual extraction complete
递归边界补充: BE-001HX-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent closeout sets stop_split true；下一步: BE-001HY-01 backend.strategy_config parent residual judgment。
- `markdown/06-milestones/v4.16.0/579-backend.strategy_config.artifact.parent_closeout.md` - v4.16.0 BE-001HX-01 backend.strategy_config.artifact parent closeout sets stop_split true
递归边界补充: BE-001HY-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects preflight；下一步: BE-001HZ-01 backend.strategy_config.preflight baseline_plan。
- `markdown/06-milestones/v4.16.0/580-backend.strategy_config.parent_residual_judgment.preflight.md` - v4.16.0 BE-001HY-01 backend.strategy_config parent residual judgment selects preflight
递归边界补充: BE-001HZ-01 `backend.strategy_config.preflight` backend.strategy_config.preflight equivalence baseline and extraction plan；下一步: BE-001HZ-02 backend.strategy_config.preflight extract_closeout。
- `markdown/06-milestones/v4.16.0/581-backend.strategy_config.preflight.baseline_plan.md` - v4.16.0 BE-001HZ-01 backend.strategy_config.preflight equivalence baseline and extraction plan
递归边界补充: BE-001HZ-02 `backend.strategy_config.preflight` backend.strategy_config.preflight actual extraction complete；下一步: BE-001IA-01 backend.strategy_config.preflight single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/582-backend.strategy_config.preflight.extract_closeout.md` - v4.16.0 BE-001HZ-02 backend.strategy_config.preflight actual extraction complete
递归边界补充: BE-001IA-01 `backend.strategy_config.preflight` backend.strategy_config.preflight single leaf closeout sets stop_split true；下一步: BE-001IB-01 backend.strategy_config parent residual judgment。
- `markdown/06-milestones/v4.16.0/583-backend.strategy_config.preflight.single_leaf_closeout.md` - v4.16.0 BE-001IA-01 backend.strategy_config.preflight single leaf closeout sets stop_split true
递归边界补充: BE-001IB-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects diff；下一步: BE-001IC-01 backend.strategy_config.diff baseline_plan。
- `markdown/06-milestones/v4.16.0/584-backend.strategy_config.parent_residual_judgment.diff.md` - v4.16.0 BE-001IB-01 backend.strategy_config parent residual judgment selects diff
递归边界补充: BE-001IC-01 `backend.strategy_config.diff` backend.strategy_config.diff equivalence baseline and extraction plan；下一步: BE-001IC-02 backend.strategy_config.diff extract_closeout。
- `markdown/06-milestones/v4.16.0/585-backend.strategy_config.diff.baseline_plan.md` - v4.16.0 BE-001IC-01 backend.strategy_config.diff equivalence baseline and extraction plan
递归边界补充: BE-001IC-02 `backend.strategy_config.diff` backend.strategy_config.diff actual extraction complete；下一步: BE-001ID-01 backend.strategy_config.diff single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/586-backend.strategy_config.diff.extract_closeout.md` - v4.16.0 BE-001IC-02 backend.strategy_config.diff actual extraction complete
递归边界补充: BE-001ID-01 `backend.strategy_config.diff` backend.strategy_config.diff single leaf closeout keeps stop_split false；下一步: BE-001IE-01 backend.strategy_config.diff parent residual judgment。
- `markdown/06-milestones/v4.16.0/587-backend.strategy_config.diff.single_leaf_closeout.md` - v4.16.0 BE-001ID-01 backend.strategy_config.diff single leaf closeout keeps stop_split false
递归边界补充: BE-001IE-01 `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects artifact_diff；下一步: BE-001IF-01 backend.strategy_config.diff.artifact_diff baseline_plan。
- `markdown/06-milestones/v4.16.0/588-backend.strategy_config.diff.parent_residual_judgment.artifact_diff.md` - v4.16.0 BE-001IE-01 backend.strategy_config.diff parent residual judgment selects artifact_diff
递归边界补充: BE-001IF-01 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan；下一步: BE-001IF-02 backend.strategy_config.diff.artifact_diff extract_closeout。
- `markdown/06-milestones/v4.16.0/589-backend.strategy_config.diff.artifact_diff.baseline_plan.md` - v4.16.0 BE-001IF-01 backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan
递归边界补充: BE-001IF-02 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff actual extraction complete；下一步: BE-001IG-01 backend.strategy_config.diff.artifact_diff single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/590-backend.strategy_config.diff.artifact_diff.extract_closeout.md` - v4.16.0 BE-001IF-02 backend.strategy_config.diff.artifact_diff actual extraction complete
递归边界补充: BE-001IG-01 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff single leaf closeout sets stop_split true；下一步: BE-001IH-01 backend.strategy_config.diff parent residual judgment。
- `markdown/06-milestones/v4.16.0/591-backend.strategy_config.diff.artifact_diff.single_leaf_closeout.md` - v4.16.0 BE-001IG-01 backend.strategy_config.diff.artifact_diff single leaf closeout sets stop_split true
递归边界补充: BE-001IH-01 `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects evidence_diff；下一步: BE-001II-01 backend.strategy_config.diff.evidence_diff baseline_plan。
- `markdown/06-milestones/v4.16.0/592-backend.strategy_config.diff.parent_residual_judgment.evidence_diff.md` - v4.16.0 BE-001IH-01 backend.strategy_config.diff parent residual judgment selects evidence_diff
递归边界补充: BE-001II-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan；下一步: BE-001II-02 backend.strategy_config.diff.evidence_diff extract_closeout。
- `markdown/06-milestones/v4.16.0/593-backend.strategy_config.diff.evidence_diff.baseline_plan.md` - v4.16.0 BE-001II-01 backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan
递归边界补充: BE-001II-02 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff actual extraction complete；下一步: BE-001IJ-01 backend.strategy_config.diff.evidence_diff single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/594-backend.strategy_config.diff.evidence_diff.extract_closeout.md` - v4.16.0 BE-001II-02 backend.strategy_config.diff.evidence_diff actual extraction complete
递归边界补充: BE-001IJ-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff single leaf closeout keeps stop_split false；下一步: BE-001IK-01 backend.strategy_config.diff.evidence_diff parent residual judgment。
- `markdown/06-milestones/v4.16.0/595-backend.strategy_config.diff.evidence_diff.single_leaf_closeout.md` - v4.16.0 BE-001IJ-01 backend.strategy_config.diff.evidence_diff single leaf closeout keeps stop_split false
递归边界补充: BE-001IK-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory；下一步: BE-001IL-01 backend.strategy_config.diff.evidence_diff.machine_trajectory baseline_plan。
- `markdown/06-milestones/v4.16.0/596-backend.strategy_config.diff.evidence_diff.parent_residual_judgment.machine_trajectory.md` - v4.16.0 BE-001IK-01 backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory
递归边界补充: BE-001IL-01 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan；下一步: BE-001IL-02 backend.strategy_config.diff.evidence_diff.machine_trajectory extract_closeout。
- `markdown/06-milestones/v4.16.0/597-backend.strategy_config.diff.evidence_diff.machine_trajectory.baseline_plan.md` - v4.16.0 BE-001IL-01 backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan
递归边界补充: BE-001IL-02 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete；下一步: BE-001IM-01 backend.strategy_config.diff.evidence_diff.machine_trajectory single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/598-backend.strategy_config.diff.evidence_diff.machine_trajectory.extract_closeout.md` - v4.16.0 BE-001IL-02 backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete
递归边界补充: BE-001IM-01 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory single leaf closeout stops further split；下一步: BE-001IN-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/599-backend.strategy_config.diff.evidence_diff.machine_trajectory.single_leaf_closeout.md` - v4.16.0 BE-001IM-01 backend.strategy_config.diff.evidence_diff.machine_trajectory single leaf closeout stops further split
递归边界补充: BE-001IN-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane；下一步: BE-001IO-01 backend.strategy_config.diff.evidence_diff.risk_plane baseline_plan。
- `markdown/06-milestones/v4.16.0/600-backend.strategy_config.diff.evidence_diff.parent_residual_judgment.risk_plane.md` - v4.16.0 BE-001IN-01 backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane
递归边界补充: BE-001IO-01 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan；下一步: BE-001IO-02 backend.strategy_config.diff.evidence_diff.risk_plane extract_closeout。
- `markdown/06-milestones/v4.16.0/601-backend.strategy_config.diff.evidence_diff.risk_plane.baseline_plan.md` - v4.16.0 BE-001IO-01 backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan
递归边界补充: BE-001IO-02 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane actual extraction complete；下一步: BE-001IP-01 backend.strategy_config.diff.evidence_diff.risk_plane single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/602-backend.strategy_config.diff.evidence_diff.risk_plane.extract_closeout.md` - v4.16.0 BE-001IO-02 backend.strategy_config.diff.evidence_diff.risk_plane actual extraction complete
递归边界补充: BE-001IP-01 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane single leaf closeout stops further split；下一步: BE-001IQ-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/603-backend.strategy_config.diff.evidence_diff.risk_plane.single_leaf_closeout.md` - v4.16.0 BE-001IP-01 backend.strategy_config.diff.evidence_diff.risk_plane single leaf closeout stops further split
递归边界补充: BE-001IQ-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability；下一步: BE-001IR-01 backend.strategy_config.diff.evidence_diff.execution_capability baseline_plan。
- `markdown/06-milestones/v4.16.0/604-backend.strategy_config.diff.evidence_diff.parent_residual_judgment.execution_capability.md` - v4.16.0 BE-001IQ-01 backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability
递归边界补充: BE-001IR-01 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan；下一步: BE-001IR-02 backend.strategy_config.diff.evidence_diff.execution_capability extract_closeout。
- `markdown/06-milestones/v4.16.0/605-backend.strategy_config.diff.evidence_diff.execution_capability.baseline_plan.md` - v4.16.0 BE-001IR-01 backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan
递归边界补充: BE-001IR-02 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability actual extraction complete；下一步: BE-001IS-01 backend.strategy_config.diff.evidence_diff.execution_capability single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/606-backend.strategy_config.diff.evidence_diff.execution_capability.extract_closeout.md` - v4.16.0 BE-001IR-02 backend.strategy_config.diff.evidence_diff.execution_capability actual extraction complete
递归边界补充: BE-001IS-01 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability single leaf closeout stops further split；下一步: BE-001IT-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/607-backend.strategy_config.diff.evidence_diff.execution_capability.single_leaf_closeout.md` - v4.16.0 BE-001IS-01 backend.strategy_config.diff.evidence_diff.execution_capability single leaf closeout stops further split
递归边界补充: BE-001IT-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics；下一步: BE-001IU-01 backend.strategy_config.diff.evidence_diff.metrics baseline_plan。
- `markdown/06-milestones/v4.16.0/608-backend.strategy_config.diff.evidence_diff.parent_residual_judgment.metrics.md` - v4.16.0 BE-001IT-01 backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics
递归边界补充: BE-001IU-01 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan；下一步: BE-001IU-02 backend.strategy_config.diff.evidence_diff.metrics extract_closeout。
- `markdown/06-milestones/v4.16.0/609-backend.strategy_config.diff.evidence_diff.metrics.baseline_plan.md` - v4.16.0 BE-001IU-01 backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan
递归边界补充: BE-001IU-02 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics actual extraction complete；下一步: BE-001IV-01 backend.strategy_config.diff.evidence_diff.metrics single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/610-backend.strategy_config.diff.evidence_diff.metrics.extract_closeout.md` - v4.16.0 BE-001IU-02 backend.strategy_config.diff.evidence_diff.metrics actual extraction complete
递归边界补充: BE-001IV-01 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics single leaf closeout stops further split；下一步: BE-001IW-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/611-backend.strategy_config.diff.evidence_diff.metrics.single_leaf_closeout.md` - v4.16.0 BE-001IV-01 backend.strategy_config.diff.evidence_diff.metrics single leaf closeout stops further split
递归边界补充: BE-001IW-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent closeout retains report assembly and shared helpers；下一步: BE-001IX-01 backend.strategy_config.diff parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/612-backend.strategy_config.diff.evidence_diff.parent_closeout.md` - v4.16.0 BE-001IW-01 backend.strategy_config.diff.evidence_diff parent closeout retains report assembly and shared helpers
递归边界补充: BE-001IX-01 `backend.strategy_config.diff` backend.strategy_config.diff parent closeout keeps facade and child mediation；下一步: BE-001IY-01 backend.strategy_config parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/613-backend.strategy_config.diff.parent_closeout.md` - v4.16.0 BE-001IX-01 backend.strategy_config.diff parent closeout keeps facade and child mediation
递归边界补充: BE-001IY-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects ai_proposal_binding；下一步: BE-001IZ-01 backend.strategy_config.ai_proposal_binding baseline_plan。
- `markdown/06-milestones/v4.16.0/614-backend.strategy_config.parent_residual_judgment.ai_proposal_binding.md` - v4.16.0 BE-001IY-01 backend.strategy_config parent residual judgment selects ai_proposal_binding
递归边界补充: BE-001IZ-01 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-op route pocket baseline and plan；下一步: BE-001IZ-02 backend.strategy_config.ai_proposal_binding extract_closeout。
- `markdown/06-milestones/v4.16.0/615-backend.strategy_config.ai_proposal_binding.baseline_plan.md` - v4.16.0 BE-001IZ-01 backend.strategy_config.ai_proposal_binding no-op route pocket baseline and plan
递归边界补充: BE-001IZ-02 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-code extraction closeout complete；下一步: BE-001JA-01 backend.strategy_config.ai_proposal_binding single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/616-backend.strategy_config.ai_proposal_binding.extract_closeout.md` - v4.16.0 BE-001IZ-02 backend.strategy_config.ai_proposal_binding no-code extraction closeout complete
递归边界补充: BE-001JA-01 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding single leaf closeout stops further split；下一步: BE-001JB-01 backend.strategy_config parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/617-backend.strategy_config.ai_proposal_binding.single_leaf_closeout.md` - v4.16.0 BE-001JA-01 backend.strategy_config.ai_proposal_binding single leaf closeout stops further split
递归边界补充: BE-001JB-01 `backend.strategy_config` backend.strategy_config parent closeout keeps route aggregation facade；下一步: BE-001JC-01 backend parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/618-backend.strategy_config.parent_closeout.md` - v4.16.0 BE-001JB-01 backend.strategy_config parent closeout keeps route aggregation facade
递归边界补充: BE-001JC-01 `backend` backend parent residual judgment selects storage_security safety baseline；下一步: BE-001JD-01 backend.storage_security baseline_plan。
- `markdown/06-milestones/v4.16.0/619-backend.parent_residual_judgment.storage_security.md` - v4.16.0 BE-001JC-01 backend parent residual judgment selects storage_security safety baseline
递归边界补充: BE-001JD-01 `backend.storage_security` backend.storage_security safety equivalence baseline and extraction plan；下一步: BE-001JD-02 backend.storage_security extract_closeout。
- `markdown/06-milestones/v4.16.0/620-backend.storage_security.safety_baseline_plan.md` - v4.16.0 BE-001JD-01 backend.storage_security safety equivalence baseline and extraction plan
递归边界补充: BE-001JD-02 `backend.storage_security` backend.storage_security facade extraction closeout keeps sensitive semantics paused；下一步: BE-001JE-01 backend.storage_security single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/621-backend.storage_security.extract_closeout.md` - v4.16.0 BE-001JD-02 backend.storage_security facade extraction closeout keeps sensitive semantics paused
递归边界补充: BE-001JE-01 `backend.storage_security` backend.storage_security single leaf closeout keeps stop_split false；下一步: BE-001JF-01 backend.storage_security parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/622-backend.storage_security.single_leaf_closeout.md` - v4.16.0 BE-001JE-01 backend.storage_security single leaf closeout keeps stop_split false
递归边界补充: BE-001JF-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_api；下一步: BE-001JG-01 backend.storage_security.credential_api baseline_plan。
- `markdown/06-milestones/v4.16.0/623-backend.storage_security.parent_residual_judgment.credential_api.md` - v4.16.0 BE-001JF-01 backend.storage_security parent residual judgment selects credential_api
递归边界补充: BE-001JG-01 `backend.storage_security.credential_api` backend.storage_security.credential_api route facade baseline and plan；下一步: BE-001JG-02 backend.storage_security.credential_api extract_closeout。
- `markdown/06-milestones/v4.16.0/624-backend.storage_security.credential_api.baseline_plan.md` - v4.16.0 BE-001JG-01 backend.storage_security.credential_api route facade baseline and plan
递归边界补充: BE-001JG-02 `backend.storage_security.credential_api` backend.storage_security.credential_api facade extraction closeout complete；下一步: BE-001JH-01 backend.storage_security.credential_api single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/625-backend.storage_security.credential_api.extract_closeout.md` - v4.16.0 BE-001JG-02 backend.storage_security.credential_api facade extraction closeout complete
递归边界补充: BE-001JH-01 `backend.storage_security.credential_api` backend.storage_security.credential_api single leaf closeout stops further facade split；下一步: BE-001JI-01 backend.storage_security parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/626-backend.storage_security.credential_api.single_leaf_closeout.md` - v4.16.0 BE-001JH-01 backend.storage_security.credential_api single leaf closeout stops further facade split
递归边界补充: BE-001JI-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault；下一步: BE-001JJ-01 backend.storage_security.credential_vault baseline_plan。
- `markdown/06-milestones/v4.16.0/627-backend.storage_security.parent_residual_judgment.credential_vault.md` - v4.16.0 BE-001JI-01 backend.storage_security parent residual judgment selects credential_vault
递归边界补充: BE-001JJ-01 `backend.storage_security.credential_vault` backend.storage_security.credential_vault re-export facade baseline and plan；下一步: BE-001JJ-02 backend.storage_security.credential_vault extract_closeout。
- `markdown/06-milestones/v4.16.0/628-backend.storage_security.credential_vault.baseline_plan.md` - v4.16.0 BE-001JJ-01 backend.storage_security.credential_vault re-export facade baseline and plan
递归边界补充: BE-001JJ-02 `backend.storage_security.credential_vault` backend.storage_security.credential_vault facade extraction closeout complete；下一步: BE-001JK-01 backend.storage_security.credential_vault single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/629-backend.storage_security.credential_vault.extract_closeout.md` - v4.16.0 BE-001JJ-02 backend.storage_security.credential_vault facade extraction closeout complete
递归边界补充: BE-001JK-01 `backend.storage_security.credential_vault` backend.storage_security.credential_vault single leaf closeout stops further facade split；下一步: BE-001JL-01 backend.storage_security parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/630-backend.storage_security.credential_vault.single_leaf_closeout.md` - v4.16.0 BE-001JK-01 backend.storage_security.credential_vault single leaf closeout stops further facade split
递归边界补充: BE-001JL-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault_implementation；下一步: BE-001JM-01 backend.storage_security.credential_vault_implementation baseline_plan。
- `markdown/06-milestones/v4.16.0/631-backend.storage_security.parent_residual_judgment.credential_vault_implementation.md` - v4.16.0 BE-001JL-01 backend.storage_security parent residual judgment selects credential_vault_implementation
递归边界补充: BE-001JM-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation safety baseline and extraction plan；下一步: BE-001JM-02 backend.storage_security.credential_vault_implementation extract_closeout。
- `markdown/06-milestones/v4.16.0/632-backend.storage_security.credential_vault_implementation.baseline_plan.md` - v4.16.0 BE-001JM-01 backend.storage_security.credential_vault_implementation safety baseline and extraction plan
递归边界补充: BE-001JM-02 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation actual extraction complete；下一步: BE-001JN-01 backend.storage_security.credential_vault_implementation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/633-backend.storage_security.credential_vault_implementation.extract_closeout.md` - v4.16.0 BE-001JM-02 backend.storage_security.credential_vault_implementation actual extraction complete
递归边界补充: BE-001JN-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation single leaf closeout keeps stop_split false；下一步: BE-001JO-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/634-backend.storage_security.credential_vault_implementation.single_leaf_closeout.md` - v4.16.0 BE-001JN-01 backend.storage_security.credential_vault_implementation single leaf closeout keeps stop_split false
递归边界补充: BE-001JO-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management；下一步: BE-001JP-01 backend.storage_security.credential_vault_implementation.machine_key_management baseline_plan。
- `markdown/06-milestones/v4.16.0/635-backend.storage_security.credential_vault_implementation.parent_residual_judgment.machine_key_management.md` - v4.16.0 BE-001JO-01 backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management
递归边界补充: BE-001JP-01 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan；下一步: BE-001JP-02 backend.storage_security.credential_vault_implementation.machine_key_management extract_closeout。
- `markdown/06-milestones/v4.16.0/636-backend.storage_security.credential_vault_implementation.machine_key_management.baseline_plan.md` - v4.16.0 BE-001JP-01 backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan
递归边界补充: BE-001JP-02 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management actual extraction complete；下一步: BE-001JP-03 backend.storage_security.credential_vault_implementation.machine_key_management single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/637-backend.storage_security.credential_vault_implementation.machine_key_management.extract_closeout.md` - v4.16.0 BE-001JP-02 backend.storage_security.credential_vault_implementation.machine_key_management actual extraction complete
递归边界补充: BE-001JP-03 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management single leaf closeout stops further split；下一步: BE-001JQ-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/638-backend.storage_security.credential_vault_implementation.machine_key_management.single_leaf_closeout.md` - v4.16.0 BE-001JP-03 backend.storage_security.credential_vault_implementation.machine_key_management single leaf closeout stops further split
递归边界补充: BE-001JQ-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec；下一步: BE-001JR-01 backend.storage_security.credential_vault_implementation.crypto_codec baseline_plan。
- `markdown/06-milestones/v4.16.0/639-backend.storage_security.credential_vault_implementation.parent_residual_judgment.crypto_codec.md` - v4.16.0 BE-001JQ-01 backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec
递归边界补充: BE-001JR-01 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan；下一步: BE-001JR-02 backend.storage_security.credential_vault_implementation.crypto_codec extract_closeout。
- `markdown/06-milestones/v4.16.0/640-backend.storage_security.credential_vault_implementation.crypto_codec.baseline_plan.md` - v4.16.0 BE-001JR-01 backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan
递归边界补充: BE-001JR-02 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec actual extraction complete；下一步: BE-001JR-03 backend.storage_security.credential_vault_implementation.crypto_codec single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/641-backend.storage_security.credential_vault_implementation.crypto_codec.extract_closeout.md` - v4.16.0 BE-001JR-02 backend.storage_security.credential_vault_implementation.crypto_codec actual extraction complete
递归边界补充: BE-001JR-03 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec single leaf closeout stops further split；下一步: BE-001JS-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/642-backend.storage_security.credential_vault_implementation.crypto_codec.single_leaf_closeout.md` - v4.16.0 BE-001JR-03 backend.storage_security.credential_vault_implementation.crypto_codec single leaf closeout stops further split
递归边界补充: BE-001JS-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects vault_persistence_restore；下一步: BE-001JT-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore baseline_plan。
- `markdown/06-milestones/v4.16.0/643-backend.storage_security.credential_vault_implementation.parent_residual_judgment.vault_persistence_restore.md` - v4.16.0 BE-001JS-01 backend.storage_security.credential_vault_implementation parent residual judgment selects vault_persistence_restore
递归边界补充: BE-001JT-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan；下一步: BE-001JT-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore extract_closeout。
- `markdown/06-milestones/v4.16.0/644-backend.storage_security.credential_vault_implementation.vault_persistence_restore.baseline_plan.md` - v4.16.0 BE-001JT-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan
递归边界补充: BE-001JT-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore actual extraction complete；下一步: BE-001JT-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/645-backend.storage_security.credential_vault_implementation.vault_persistence_restore.extract_closeout.md` - v4.16.0 BE-001JT-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore actual extraction complete
递归边界补充: BE-001JT-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore single leaf closeout keeps stop_split false；下一步: BE-001JU-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/646-backend.storage_security.credential_vault_implementation.vault_persistence_restore.single_leaf_closeout.md` - v4.16.0 BE-001JT-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore single leaf closeout keeps stop_split false
递归边界补充: BE-001JU-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects load_restore_entry；下一步: BE-001JV-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry baseline_plan。
- `markdown/06-milestones/v4.16.0/647-backend.storage_security.credential_vault_implementation.vault_persistence_restore.parent_residual_judgment.load_restore_entry.md` - v4.16.0 BE-001JU-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects load_restore_entry
递归边界补充: BE-001JV-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan；下一步: BE-001JV-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry extract_closeout。
- `markdown/06-milestones/v4.16.0/648-backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry.baseline_plan.md` - v4.16.0 BE-001JV-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan
递归边界补充: BE-001JV-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry actual extraction complete；下一步: BE-001JV-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/649-backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry.extract_closeout.md` - v4.16.0 BE-001JV-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry actual extraction complete
递归边界补充: BE-001JV-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single leaf closeout stops further split；下一步: BE-001JW-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/650-backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry.single_leaf_closeout.md` - v4.16.0 BE-001JV-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single leaf closeout stops further split
递归边界补充: BE-001JW-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects atomic_save_commit；下一步: BE-001JX-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit baseline_plan。
- `markdown/06-milestones/v4.16.0/651-backend.storage_security.credential_vault_implementation.vault_persistence_restore.parent_residual_judgment.atomic_save_commit.md` - v4.16.0 BE-001JW-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects atomic_save_commit
递归边界补充: BE-001JX-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan；下一步: BE-001JX-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit extract_closeout。
- `markdown/06-milestones/v4.16.0/652-backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit.baseline_plan.md` - v4.16.0 BE-001JX-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan
递归边界补充: BE-001JX-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit actual extraction complete；下一步: BE-001JX-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/653-backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit.extract_closeout.md` - v4.16.0 BE-001JX-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit actual extraction complete
递归边界补充: BE-001JX-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single leaf closeout stops further split；下一步: BE-001JY-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_closeout。
- `markdown/06-milestones/v4.16.0/654-backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit.single_leaf_closeout.md` - v4.16.0 BE-001JX-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single leaf closeout stops further split
递归边界补充: BE-001JY-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent closeout stops persistence split；下一步: BE-001JZ-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/655-backend.storage_security.credential_vault_implementation.vault_persistence_restore.parent_closeout.md` - v4.16.0 BE-001JY-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent closeout stops persistence split
递归边界补充: BE-001JZ-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects service_crud；下一步: BE-001KA-01 backend.storage_security.credential_vault_implementation.service_crud baseline_plan。
- `markdown/06-milestones/v4.16.0/656-backend.storage_security.credential_vault_implementation.parent_residual_judgment.service_crud.md` - v4.16.0 BE-001JZ-01 backend.storage_security.credential_vault_implementation parent residual judgment selects service_crud
递归边界补充: BE-001KA-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan；下一步: BE-001KA-02 backend.storage_security.credential_vault_implementation.service_crud extract_closeout。
- `markdown/06-milestones/v4.16.0/657-backend.storage_security.credential_vault_implementation.service_crud.baseline_plan.md` - v4.16.0 BE-001KA-01 backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan
递归边界补充: BE-001KA-02 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud actual extraction complete；下一步: BE-001KA-03 backend.storage_security.credential_vault_implementation.service_crud single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/658-backend.storage_security.credential_vault_implementation.service_crud.extract_closeout.md` - v4.16.0 BE-001KA-02 backend.storage_security.credential_vault_implementation.service_crud actual extraction complete
递归边界补充: BE-001KA-03 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud single leaf closeout keeps stop_split false；下一步: BE-001KB-01 backend.storage_security.credential_vault_implementation.service_crud parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/659-backend.storage_security.credential_vault_implementation.service_crud.single_leaf_closeout.md` - v4.16.0 BE-001KA-03 backend.storage_security.credential_vault_implementation.service_crud single leaf closeout keeps stop_split false
递归边界补充: BE-001KB-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_mutation_commit；下一步: BE-001KC-01 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit baseline_plan。
- `markdown/06-milestones/v4.16.0/660-backend.storage_security.credential_vault_implementation.service_crud.parent_residual_judgment.service_mutation_commit.md` - v4.16.0 BE-001KB-01 backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_mutation_commit
递归边界补充: BE-001KC-01 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan；下一步: BE-001KC-02 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit extract_closeout。
- `markdown/06-milestones/v4.16.0/661-backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit.baseline_plan.md` - v4.16.0 BE-001KC-01 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan
递归边界补充: BE-001KC-02 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit actual extraction complete；下一步: BE-001KC-03 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/662-backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit.extract_closeout.md` - v4.16.0 BE-001KC-02 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit actual extraction complete
递归边界补充: BE-001KC-03 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single leaf closeout stops further split；下一步: BE-001KD-01 backend.storage_security.credential_vault_implementation.service_crud parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/663-backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit.single_leaf_closeout.md` - v4.16.0 BE-001KC-03 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single leaf closeout stops further split
递归边界补充: BE-001KD-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection；下一步: BE-001KE-01 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection baseline_plan。
- `markdown/06-milestones/v4.16.0/664-backend.storage_security.credential_vault_implementation.service_crud.parent_residual_judgment.service_read_projection.md` - v4.16.0 BE-001KD-01 backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection
递归边界补充: BE-001KE-01 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan；下一步: BE-001KE-02 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection extract_closeout。
- `markdown/06-milestones/v4.16.0/665-backend.storage_security.credential_vault_implementation.service_crud.service_read_projection.baseline_plan.md` - v4.16.0 BE-001KE-01 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan
递归边界补充: BE-001KE-02 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection actual extraction complete；下一步: BE-001KE-03 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/666-backend.storage_security.credential_vault_implementation.service_crud.service_read_projection.extract_closeout.md` - v4.16.0 BE-001KE-02 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection actual extraction complete
递归边界补充: BE-001KE-03 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split；下一步: BE-001KF-01 backend.storage_security.credential_vault_implementation.service_crud parent_closeout。
- `markdown/06-milestones/v4.16.0/667-backend.storage_security.credential_vault_implementation.service_crud.service_read_projection.single_leaf_closeout.md` - v4.16.0 BE-001KE-03 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split
递归边界补充: BE-001KF-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent closeout stops CRUD split；下一步: BE-001KG-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/668-backend.storage_security.credential_vault_implementation.service_crud.parent_closeout.md` - v4.16.0 BE-001KF-01 backend.storage_security.credential_vault_implementation.service_crud parent closeout stops CRUD split
递归边界补充: BE-001KG-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects secret_pattern_extraction；下一步: BE-001KH-01 backend.storage_security.credential_vault_implementation.secret_pattern_extraction baseline_plan。
- `markdown/06-milestones/v4.16.0/669-backend.storage_security.credential_vault_implementation.parent_residual_judgment.secret_pattern_extraction.md` - v4.16.0 BE-001KG-01 backend.storage_security.credential_vault_implementation parent residual judgment selects secret_pattern_extraction
递归边界补充: BE-001KH-01 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan；下一步: BE-001KH-02 backend.storage_security.credential_vault_implementation.secret_pattern_extraction extract_closeout。
- `markdown/06-milestones/v4.16.0/670-backend.storage_security.credential_vault_implementation.secret_pattern_extraction.baseline_plan.md` - v4.16.0 BE-001KH-01 backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan
递归边界补充: BE-001KH-02 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction actual extraction complete；下一步: BE-001KH-03 backend.storage_security.credential_vault_implementation.secret_pattern_extraction single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/671-backend.storage_security.credential_vault_implementation.secret_pattern_extraction.extract_closeout.md` - v4.16.0 BE-001KH-02 backend.storage_security.credential_vault_implementation.secret_pattern_extraction actual extraction complete
递归边界补充: BE-001KH-03 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction single leaf closeout stops further split；下一步: BE-001KI-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/672-backend.storage_security.credential_vault_implementation.secret_pattern_extraction.single_leaf_closeout.md` - v4.16.0 BE-001KH-03 backend.storage_security.credential_vault_implementation.secret_pattern_extraction single leaf closeout stops further split
递归边界补充: BE-001KI-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects type_surface；下一步: BE-001KJ-01 backend.storage_security.credential_vault_implementation.type_surface baseline_plan。
- `markdown/06-milestones/v4.16.0/673-backend.storage_security.credential_vault_implementation.parent_residual_judgment.type_surface.md` - v4.16.0 BE-001KI-01 backend.storage_security.credential_vault_implementation parent residual judgment selects type_surface
递归边界补充: BE-001KJ-01 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan；下一步: BE-001KJ-02 backend.storage_security.credential_vault_implementation.type_surface extract_closeout。
- `markdown/06-milestones/v4.16.0/674-backend.storage_security.credential_vault_implementation.type_surface.baseline_plan.md` - v4.16.0 BE-001KJ-01 backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan
递归边界补充: BE-001KJ-02 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface actual extraction complete；下一步: BE-001KJ-03 backend.storage_security.credential_vault_implementation.type_surface single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/675-backend.storage_security.credential_vault_implementation.type_surface.extract_closeout.md` - v4.16.0 BE-001KJ-02 backend.storage_security.credential_vault_implementation.type_surface actual extraction complete
递归边界补充: BE-001KJ-03 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface single leaf closeout stops further split；下一步: BE-001KK-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/676-backend.storage_security.credential_vault_implementation.type_surface.single_leaf_closeout.md` - v4.16.0 BE-001KJ-03 backend.storage_security.credential_vault_implementation.type_surface single leaf closeout stops further split
递归边界补充: BE-001KK-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects implementation_test_harness；下一步: BE-001KL-01 backend.storage_security.credential_vault_implementation.implementation_test_harness baseline_plan。
- `markdown/06-milestones/v4.16.0/677-backend.storage_security.credential_vault_implementation.parent_residual_judgment.implementation_test_harness.md` - v4.16.0 BE-001KK-01 backend.storage_security.credential_vault_implementation parent residual judgment selects implementation_test_harness
递归边界补充: BE-001KL-01 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan；下一步: BE-001KL-02 backend.storage_security.credential_vault_implementation.implementation_test_harness extract_closeout。
- `markdown/06-milestones/v4.16.0/678-backend.storage_security.credential_vault_implementation.implementation_test_harness.baseline_plan.md` - v4.16.0 BE-001KL-01 backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan
递归边界补充: BE-001KL-02 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness actual extraction complete；下一步: BE-001KL-03 backend.storage_security.credential_vault_implementation.implementation_test_harness single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/679-backend.storage_security.credential_vault_implementation.implementation_test_harness.extract_closeout.md` - v4.16.0 BE-001KL-02 backend.storage_security.credential_vault_implementation.implementation_test_harness actual extraction complete
递归边界补充: BE-001KL-03 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness single leaf closeout stops further split；下一步: BE-001KM-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/680-backend.storage_security.credential_vault_implementation.implementation_test_harness.single_leaf_closeout.md` - v4.16.0 BE-001KL-03 backend.storage_security.credential_vault_implementation.implementation_test_harness single leaf closeout stops further split
递归边界补充: BE-001KM-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment closes implementation parent；下一步: BE-001KN-01 backend.storage_security parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/681-backend.storage_security.credential_vault_implementation.parent_residual_judgment.stop_split.md` - v4.16.0 BE-001KM-01 backend.storage_security.credential_vault_implementation parent residual judgment closes implementation parent
递归边界补充: BE-001KN-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_api_handler_implementation；下一步: BE-001KO-01 backend.storage_security.credential_api_handler_implementation baseline_plan。
- `markdown/06-milestones/v4.16.0/682-backend.storage_security.parent_residual_judgment.credential_api_handler_implementation.md` - v4.16.0 BE-001KN-01 backend.storage_security parent residual judgment selects credential_api_handler_implementation
递归边界补充: BE-001KO-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan；下一步: BE-001KO-02 backend.storage_security.credential_api_handler_implementation extract_closeout。
- `markdown/06-milestones/v4.16.0/683-backend.storage_security.credential_api_handler_implementation.baseline_plan.md` - v4.16.0 BE-001KO-01 backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan
递归边界补充: BE-001KO-02 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation actual extraction complete；下一步: BE-001KO-03 backend.storage_security.credential_api_handler_implementation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/684-backend.storage_security.credential_api_handler_implementation.extract_closeout.md` - v4.16.0 BE-001KO-02 backend.storage_security.credential_api_handler_implementation actual extraction complete
递归边界补充: BE-001KO-03 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation single leaf closeout continues split；下一步: BE-001KP-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/685-backend.storage_security.credential_api_handler_implementation.single_leaf_closeout.md` - v4.16.0 BE-001KO-03 backend.storage_security.credential_api_handler_implementation single leaf closeout continues split
递归边界补充: BE-001KP-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection；下一步: BE-001KQ-01 backend.storage_security.credential_api_handler_implementation.list_projection baseline_plan。
- `markdown/06-milestones/v4.16.0/686-backend.storage_security.credential_api_handler_implementation.parent_residual_judgment.list_projection.md` - v4.16.0 BE-001KP-01 backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection
递归边界补充: BE-001KQ-01 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan；下一步: BE-001KQ-02 backend.storage_security.credential_api_handler_implementation.list_projection extract_closeout。
- `markdown/06-milestones/v4.16.0/687-backend.storage_security.credential_api_handler_implementation.list_projection.baseline_plan.md` - v4.16.0 BE-001KQ-01 backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan
递归边界补充: BE-001KQ-02 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete；下一步: BE-001KQ-03 backend.storage_security.credential_api_handler_implementation.list_projection single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/688-backend.storage_security.credential_api_handler_implementation.list_projection.extract_closeout.md` - v4.16.0 BE-001KQ-02 backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete
递归边界补充: BE-001KQ-03 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection single leaf closeout stops further split；下一步: BE-001KR-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/689-backend.storage_security.credential_api_handler_implementation.list_projection.single_leaf_closeout.md` - v4.16.0 BE-001KQ-03 backend.storage_security.credential_api_handler_implementation.list_projection single leaf closeout stops further split
递归边界补充: BE-001KR-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope；下一步: BE-001KS-01 backend.storage_security.credential_api_handler_implementation.key_scope baseline_plan。
- `markdown/06-milestones/v4.16.0/690-backend.storage_security.credential_api_handler_implementation.parent_residual_judgment.key_scope.md` - v4.16.0 BE-001KR-01 backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope
递归边界补充: BE-001KS-01 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan；下一步: BE-001KS-02 backend.storage_security.credential_api_handler_implementation.key_scope extract_closeout。
- `markdown/06-milestones/v4.16.0/691-backend.storage_security.credential_api_handler_implementation.key_scope.baseline_plan.md` - v4.16.0 BE-001KS-01 backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan
递归边界补充: BE-001KS-02 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete；下一步: BE-001KS-03 backend.storage_security.credential_api_handler_implementation.key_scope single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/692-backend.storage_security.credential_api_handler_implementation.key_scope.extract_closeout.md` - v4.16.0 BE-001KS-02 backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete
递归边界补充: BE-001KS-03 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope single leaf closeout stops further split；下一步: BE-001KT-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/693-backend.storage_security.credential_api_handler_implementation.key_scope.single_leaf_closeout.md` - v4.16.0 BE-001KS-03 backend.storage_security.credential_api_handler_implementation.key_scope single leaf closeout stops further split
递归边界补充: BE-001KT-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation；下一步: BE-001KU-01 backend.storage_security.credential_api_handler_implementation.set_mutation baseline_plan。
- `markdown/06-milestones/v4.16.0/694-backend.storage_security.credential_api_handler_implementation.parent_residual_judgment.set_mutation.md` - v4.16.0 BE-001KT-01 backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation
递归边界补充: BE-001KU-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan；下一步: BE-001KU-02 backend.storage_security.credential_api_handler_implementation.set_mutation extract_closeout。
- `markdown/06-milestones/v4.16.0/695-backend.storage_security.credential_api_handler_implementation.set_mutation.baseline_plan.md` - v4.16.0 BE-001KU-01 backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan
递归边界补充: BE-001KU-02 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete；下一步: BE-001KU-03 backend.storage_security.credential_api_handler_implementation.set_mutation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/696-backend.storage_security.credential_api_handler_implementation.set_mutation.extract_closeout.md` - v4.16.0 BE-001KU-02 backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete
递归边界补充: BE-001KU-03 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation single leaf closeout continues split；下一步: BE-001KV-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/697-backend.storage_security.credential_api_handler_implementation.set_mutation.single_leaf_closeout.md` - v4.16.0 BE-001KU-03 backend.storage_security.credential_api_handler_implementation.set_mutation single leaf closeout continues split
递归边界补充: BE-001KV-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation；下一步: BE-001KW-01 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/698-backend.storage_security.credential_api_handler_implementation.set_mutation.parent_residual_judgment.service_and_fields_validation.md` - v4.16.0 BE-001KV-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation
递归边界补充: BE-001KW-01 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan；下一步: BE-001KW-02 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/699-backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation.baseline_plan.md` - v4.16.0 BE-001KW-01 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan
递归边界补充: BE-001KW-02 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete；下一步: BE-001KW-03 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/700-backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation.extract_closeout.md` - v4.16.0 BE-001KW-02 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete
递归边界补充: BE-001KW-03 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single leaf closeout stops further split；下一步: BE-001KX-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/701-backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation.single_leaf_closeout.md` - v4.16.0 BE-001KW-03 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single leaf closeout stops further split
递归边界补充: BE-001KX-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects storage_commit；下一步: BE-001KY-01 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit baseline_plan。
- `markdown/06-milestones/v4.16.0/702-backend.storage_security.credential_api_handler_implementation.set_mutation.parent_residual_judgment.storage_commit.md` - v4.16.0 BE-001KX-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects storage_commit
递归边界补充: BE-001KY-01 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan；下一步: BE-001KY-02 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit extract_closeout。
- `markdown/06-milestones/v4.16.0/703-backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit.baseline_plan.md` - v4.16.0 BE-001KY-01 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan
递归边界补充: BE-001KY-02 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete；下一步: BE-001KY-03 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/704-backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit.extract_closeout.md` - v4.16.0 BE-001KY-02 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete
递归边界补充: BE-001KY-03 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single leaf closeout stops further split；下一步: BE-001KZ-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/705-backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit.single_leaf_closeout.md` - v4.16.0 BE-001KY-03 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single leaf closeout stops further split
递归边界补充: BE-001KZ-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment closes parent；下一步: BE-001LA-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/706-backend.storage_security.credential_api_handler_implementation.set_mutation.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001KZ-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment closes parent
递归边界补充: BE-001LA-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation；下一步: BE-001LB-01 backend.storage_security.credential_api_handler_implementation.delete_mutation baseline_plan。
- `markdown/06-milestones/v4.16.0/707-backend.storage_security.credential_api_handler_implementation.parent_residual_judgment.delete_mutation.md` - v4.16.0 BE-001LA-01 backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation
递归边界补充: BE-001LB-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan；下一步: BE-001LB-02 backend.storage_security.credential_api_handler_implementation.delete_mutation extract_closeout。
- `markdown/06-milestones/v4.16.0/708-backend.storage_security.credential_api_handler_implementation.delete_mutation.baseline_plan.md` - v4.16.0 BE-001LB-01 backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan
递归边界补充: BE-001LB-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete；下一步: BE-001LB-03 backend.storage_security.credential_api_handler_implementation.delete_mutation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/709-backend.storage_security.credential_api_handler_implementation.delete_mutation.extract_closeout.md` - v4.16.0 BE-001LB-02 backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete
递归边界补充: BE-001LB-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation single leaf closeout continues split；下一步: BE-001LC-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/710-backend.storage_security.credential_api_handler_implementation.delete_mutation.single_leaf_closeout.md` - v4.16.0 BE-001LB-03 backend.storage_security.credential_api_handler_implementation.delete_mutation single leaf closeout continues split
递归边界补充: BE-001LC-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation；下一步: BE-001LD-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/711-backend.storage_security.credential_api_handler_implementation.delete_mutation.parent_residual_judgment.service_path_validation.md` - v4.16.0 BE-001LC-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation
递归边界补充: BE-001LD-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan；下一步: BE-001LD-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/712-backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation.baseline_plan.md` - v4.16.0 BE-001LD-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan
递归边界补充: BE-001LD-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete；下一步: BE-001LD-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/713-backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation.extract_closeout.md` - v4.16.0 BE-001LD-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete
递归边界补充: BE-001LD-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single leaf closeout stops further split；下一步: BE-001LE-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/714-backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation.single_leaf_closeout.md` - v4.16.0 BE-001LD-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single leaf closeout stops further split
递归边界补充: BE-001LE-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects delete_commit；下一步: BE-001LF-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit baseline_plan。
- `markdown/06-milestones/v4.16.0/715-backend.storage_security.credential_api_handler_implementation.delete_mutation.parent_residual_judgment.delete_commit.md` - v4.16.0 BE-001LE-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects delete_commit
递归边界补充: BE-001LF-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan；下一步: BE-001LF-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit extract_closeout。
- `markdown/06-milestones/v4.16.0/716-backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit.baseline_plan.md` - v4.16.0 BE-001LF-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan
递归边界补充: BE-001LF-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete；下一步: BE-001LF-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/717-backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit.extract_closeout.md` - v4.16.0 BE-001LF-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete
递归边界补充: BE-001LF-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single leaf closeout stops further split；下一步: BE-001LG-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/718-backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit.single_leaf_closeout.md` - v4.16.0 BE-001LF-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single leaf closeout stops further split
递归边界补充: BE-001LG-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment closes parent；下一步: BE-001LH-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/719-backend.storage_security.credential_api_handler_implementation.delete_mutation.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001LG-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment closes parent
递归边界补充: BE-001LH-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment closes parent；下一步: BE-001LI-01 backend.storage_security parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/720-backend.storage_security.credential_api_handler_implementation.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001LH-01 backend.storage_security.credential_api_handler_implementation parent residual judgment closes parent
递归边界补充: BE-001LI-01 `backend.storage_security` backend.storage_security parent residual judgment closes parent；下一步: BE-001LJ-01 backend parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/721-backend.storage_security.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001LI-01 backend.storage_security parent residual judgment closes parent
递归边界补充: BE-001LJ-01 `backend` backend parent residual judgment selects ops_governance；下一步: BE-001LK-01 backend.ops_governance baseline_plan。
- `markdown/06-milestones/v4.16.0/722-backend.parent_residual_judgment.ops_governance.md` - v4.16.0 BE-001LJ-01 backend parent residual judgment selects ops_governance
递归边界补充: BE-001LK-01 `backend.ops_governance` backend.ops_governance equivalence baseline and extraction plan；下一步: BE-001LK-02 backend.ops_governance extract_closeout。
- `markdown/06-milestones/v4.16.0/723-backend.ops_governance.baseline_plan.md` - v4.16.0 BE-001LK-01 backend.ops_governance equivalence baseline and extraction plan
递归边界补充: BE-001LK-02 `backend.ops_governance` backend.ops_governance facade extraction closeout；下一步: BE-001LK-03 backend.ops_governance single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/724-backend.ops_governance.extract_closeout.md` - v4.16.0 BE-001LK-02 backend.ops_governance facade extraction closeout
递归边界补充: BE-001LK-03 `backend.ops_governance` backend.ops_governance single leaf closeout continues split；下一步: BE-001LL-01 backend.ops_governance parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/725-backend.ops_governance.single_leaf_closeout.md` - v4.16.0 BE-001LK-03 backend.ops_governance single leaf closeout continues split
递归边界补充: BE-001LL-01 `backend.ops_governance` backend.ops_governance parent residual judgment selects hotswap；下一步: BE-001LM-01 backend.ops_governance.hotswap baseline_plan。
- `markdown/06-milestones/v4.16.0/726-backend.ops_governance.parent_residual_judgment.hotswap.md` - v4.16.0 BE-001LL-01 backend.ops_governance parent residual judgment selects hotswap
递归边界补充: BE-001LM-01 `backend.ops_governance.hotswap` backend.ops_governance.hotswap equivalence baseline and extraction plan；下一步: BE-001LM-02 backend.ops_governance.hotswap extract_closeout。
- `markdown/06-milestones/v4.16.0/727-backend.ops_governance.hotswap.baseline_plan.md` - v4.16.0 BE-001LM-01 backend.ops_governance.hotswap equivalence baseline and extraction plan
递归边界补充: BE-001LM-02 `backend.ops_governance.hotswap` backend.ops_governance.hotswap actual extraction complete；下一步: BE-001LM-03 backend.ops_governance.hotswap single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/728-backend.ops_governance.hotswap.extract_closeout.md` - v4.16.0 BE-001LM-02 backend.ops_governance.hotswap actual extraction complete
递归边界补充: BE-001LM-03 `backend.ops_governance.hotswap` backend.ops_governance.hotswap single leaf closeout stops further split；下一步: BE-001LN-01 backend.ops_governance parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/729-backend.ops_governance.hotswap.single_leaf_closeout.md` - v4.16.0 BE-001LM-03 backend.ops_governance.hotswap single leaf closeout stops further split
递归边界补充: BE-001LN-01 `backend.ops_governance` backend.ops_governance parent residual judgment selects sandbox；下一步: BE-001LO-01 backend.ops_governance.sandbox baseline_plan。
- `markdown/06-milestones/v4.16.0/730-backend.ops_governance.parent_residual_judgment.sandbox.md` - v4.16.0 BE-001LN-01 backend.ops_governance parent residual judgment selects sandbox
递归边界补充: BE-001LO-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox equivalence baseline and extraction plan；下一步: BE-001LO-02 backend.ops_governance.sandbox extract_closeout。
- `markdown/06-milestones/v4.16.0/731-backend.ops_governance.sandbox.baseline_plan.md` - v4.16.0 BE-001LO-01 backend.ops_governance.sandbox equivalence baseline and extraction plan
递归边界补充: BE-001LO-02 `backend.ops_governance.sandbox` backend.ops_governance.sandbox actual extraction complete；下一步: BE-001LO-03 backend.ops_governance.sandbox single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/732-backend.ops_governance.sandbox.extract_closeout.md` - v4.16.0 BE-001LO-02 backend.ops_governance.sandbox actual extraction complete
递归边界补充: BE-001LO-03 `backend.ops_governance.sandbox` backend.ops_governance.sandbox single leaf closeout continues split；下一步: BE-001LP-01 backend.ops_governance.sandbox parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/733-backend.ops_governance.sandbox.single_leaf_closeout.md` - v4.16.0 BE-001LO-03 backend.ops_governance.sandbox single leaf closeout continues split
递归边界补充: BE-001LP-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects report_api；下一步: BE-001LQ-01 backend.ops_governance.sandbox.report_api baseline_plan。
- `markdown/06-milestones/v4.16.0/734-backend.ops_governance.sandbox.parent_residual_judgment.report_api.md` - v4.16.0 BE-001LP-01 backend.ops_governance.sandbox parent residual judgment selects report_api
递归边界补充: BE-001LQ-01 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan；下一步: BE-001LQ-02 backend.ops_governance.sandbox.report_api extract_closeout。
- `markdown/06-milestones/v4.16.0/735-backend.ops_governance.sandbox.report_api.baseline_plan.md` - v4.16.0 BE-001LQ-01 backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan
递归边界补充: BE-001LQ-02 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api actual extraction complete；下一步: BE-001LQ-03 backend.ops_governance.sandbox.report_api single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/736-backend.ops_governance.sandbox.report_api.extract_closeout.md` - v4.16.0 BE-001LQ-02 backend.ops_governance.sandbox.report_api actual extraction complete
递归边界补充: BE-001LQ-03 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api single leaf closeout stops further split；下一步: BE-001LR-01 backend.ops_governance.sandbox parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/737-backend.ops_governance.sandbox.report_api.single_leaf_closeout.md` - v4.16.0 BE-001LQ-03 backend.ops_governance.sandbox.report_api single leaf closeout stops further split
递归边界补充: BE-001LR-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects verification_run；下一步: BE-001LS-01 backend.ops_governance.sandbox.verification_run baseline_plan。
- `markdown/06-milestones/v4.16.0/738-backend.ops_governance.sandbox.parent_residual_judgment.verification_run.md` - v4.16.0 BE-001LR-01 backend.ops_governance.sandbox parent residual judgment selects verification_run
递归边界补充: BE-001LS-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan；下一步: BE-001LS-02 backend.ops_governance.sandbox.verification_run extract_closeout。
- `markdown/06-milestones/v4.16.0/739-backend.ops_governance.sandbox.verification_run.baseline_plan.md` - v4.16.0 BE-001LS-01 backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan
递归边界补充: BE-001LS-02 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run actual extraction complete；下一步: BE-001LS-03 backend.ops_governance.sandbox.verification_run single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/740-backend.ops_governance.sandbox.verification_run.extract_closeout.md` - v4.16.0 BE-001LS-02 backend.ops_governance.sandbox.verification_run actual extraction complete
递归边界补充: BE-001LS-03 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run single leaf closeout continues split；下一步: BE-001LT-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/741-backend.ops_governance.sandbox.verification_run.single_leaf_closeout.md` - v4.16.0 BE-001LS-03 backend.ops_governance.sandbox.verification_run single leaf closeout continues split
递归边界补充: BE-001LT-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit；下一步: BE-001LU-01 backend.ops_governance.sandbox.verification_run.report_commit baseline_plan。
- `markdown/06-milestones/v4.16.0/742-backend.ops_governance.sandbox.verification_run.parent_residual_judgment.report_commit.md` - v4.16.0 BE-001LT-01 backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit
递归边界补充: BE-001LU-01 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan；下一步: BE-001LU-02 backend.ops_governance.sandbox.verification_run.report_commit extract_closeout。
- `markdown/06-milestones/v4.16.0/743-backend.ops_governance.sandbox.verification_run.report_commit.baseline_plan.md` - v4.16.0 BE-001LU-01 backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan
递归边界补充: BE-001LU-02 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete；下一步: BE-001LU-03 backend.ops_governance.sandbox.verification_run.report_commit single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/744-backend.ops_governance.sandbox.verification_run.report_commit.extract_closeout.md` - v4.16.0 BE-001LU-02 backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete
递归边界补充: BE-001LU-03 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit single leaf closeout stops further split；下一步: BE-001LV-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/745-backend.ops_governance.sandbox.verification_run.report_commit.single_leaf_closeout.md` - v4.16.0 BE-001LU-03 backend.ops_governance.sandbox.verification_run.report_commit single leaf closeout stops further split
递归边界补充: BE-001LV-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate；下一步: BE-001LW-01 backend.ops_governance.sandbox.verification_run.proposal_gate baseline_plan。
- `markdown/06-milestones/v4.16.0/746-backend.ops_governance.sandbox.verification_run.parent_residual_judgment.proposal_gate.md` - v4.16.0 BE-001LV-01 backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate
递归边界补充: BE-001LW-01 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan；下一步: BE-001LW-02 backend.ops_governance.sandbox.verification_run.proposal_gate extract_closeout。
- `markdown/06-milestones/v4.16.0/747-backend.ops_governance.sandbox.verification_run.proposal_gate.baseline_plan.md` - v4.16.0 BE-001LW-01 backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan
递归边界补充: BE-001LW-02 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete；下一步: BE-001LW-03 backend.ops_governance.sandbox.verification_run.proposal_gate single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/748-backend.ops_governance.sandbox.verification_run.proposal_gate.extract_closeout.md` - v4.16.0 BE-001LW-02 backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete
递归边界补充: BE-001LW-03 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate single leaf closeout stops further split；下一步: BE-001LX-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/749-backend.ops_governance.sandbox.verification_run.proposal_gate.single_leaf_closeout.md` - v4.16.0 BE-001LW-03 backend.ops_governance.sandbox.verification_run.proposal_gate single leaf closeout stops further split
递归边界补充: BE-001LX-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window；下一步: BE-001LY-01 backend.ops_governance.sandbox.verification_run.replay_window baseline_plan。
- `markdown/06-milestones/v4.16.0/750-backend.ops_governance.sandbox.verification_run.parent_residual_judgment.replay_window.md` - v4.16.0 BE-001LX-01 backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window
递归边界补充: BE-001LY-01 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan；下一步: BE-001LY-02 backend.ops_governance.sandbox.verification_run.replay_window extract_closeout。
- `markdown/06-milestones/v4.16.0/751-backend.ops_governance.sandbox.verification_run.replay_window.baseline_plan.md` - v4.16.0 BE-001LY-01 backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan
递归边界补充: BE-001LY-02 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete；下一步: BE-001LY-03 backend.ops_governance.sandbox.verification_run.replay_window single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/752-backend.ops_governance.sandbox.verification_run.replay_window.extract_closeout.md` - v4.16.0 BE-001LY-02 backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete
递归边界补充: BE-001LY-03 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout stops further split；下一步: BE-001LZ-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/753-backend.ops_governance.sandbox.verification_run.replay_window.single_leaf_closeout.md` - v4.16.0 BE-001LY-03 backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout stops further split
递归边界补充: BE-001LZ-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly；下一步: BE-001MA-01 backend.ops_governance.sandbox.verification_run.report_assembly baseline_plan。
- `markdown/06-milestones/v4.16.0/754-backend.ops_governance.sandbox.verification_run.parent_residual_judgment.report_assembly.md` - v4.16.0 BE-001LZ-01 backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly
递归边界补充: BE-001MA-01 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan；下一步: BE-001MA-02 backend.ops_governance.sandbox.verification_run.report_assembly extract_closeout。
- `markdown/06-milestones/v4.16.0/755-backend.ops_governance.sandbox.verification_run.report_assembly.baseline_plan.md` - v4.16.0 BE-001MA-01 backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan
递归边界补充: BE-001MA-02 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete；下一步: BE-001MA-03 backend.ops_governance.sandbox.verification_run.report_assembly single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/756-backend.ops_governance.sandbox.verification_run.report_assembly.extract_closeout.md` - v4.16.0 BE-001MA-02 backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete
递归边界补充: BE-001MA-03 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout stops further split；下一步: BE-001MB-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/757-backend.ops_governance.sandbox.verification_run.report_assembly.single_leaf_closeout.md` - v4.16.0 BE-001MA-03 backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout stops further split
递归边界补充: BE-001MB-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment closes parent；下一步: BE-001MC-01 backend.ops_governance.sandbox parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/758-backend.ops_governance.sandbox.verification_run.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001MB-01 backend.ops_governance.sandbox.verification_run parent residual judgment closes parent
递归边界补充: BE-001MC-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation；下一步: BE-001MD-01 backend.ops_governance.sandbox.metrics_evaluation baseline_plan。
- `markdown/06-milestones/v4.16.0/759-backend.ops_governance.sandbox.parent_residual_judgment.metrics_evaluation.md` - v4.16.0 BE-001MC-01 backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation
递归边界补充: BE-001MD-01 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan；下一步: BE-001MD-02 backend.ops_governance.sandbox.metrics_evaluation extract_closeout。
- `markdown/06-milestones/v4.16.0/760-backend.ops_governance.sandbox.metrics_evaluation.baseline_plan.md` - v4.16.0 BE-001MD-01 backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan
递归边界补充: BE-001MD-02 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation actual extraction complete；下一步: BE-001MD-03 backend.ops_governance.sandbox.metrics_evaluation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/761-backend.ops_governance.sandbox.metrics_evaluation.extract_closeout.md` - v4.16.0 BE-001MD-02 backend.ops_governance.sandbox.metrics_evaluation actual extraction complete
递归边界补充: BE-001MD-03 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation single leaf closeout stops further split；下一步: BE-001ME-01 backend.ops_governance.sandbox parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/762-backend.ops_governance.sandbox.metrics_evaluation.single_leaf_closeout.md` - v4.16.0 BE-001MD-03 backend.ops_governance.sandbox.metrics_evaluation single leaf closeout stops further split
递归边界补充: BE-001ME-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects comparison_metrics；下一步: BE-001MF-01 backend.ops_governance.sandbox.comparison_metrics baseline_plan。
- `markdown/06-milestones/v4.16.0/763-backend.ops_governance.sandbox.parent_residual_judgment.comparison_metrics.md` - v4.16.0 BE-001ME-01 backend.ops_governance.sandbox parent residual judgment selects comparison_metrics
递归边界补充: BE-001MF-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan；下一步: BE-001MF-02 backend.ops_governance.sandbox.comparison_metrics extract_closeout。
- `markdown/06-milestones/v4.16.0/764-backend.ops_governance.sandbox.comparison_metrics.baseline_plan.md` - v4.16.0 BE-001MF-01 backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan
递归边界补充: BE-001MF-02 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics actual extraction complete；下一步: BE-001MF-03 backend.ops_governance.sandbox.comparison_metrics single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/765-backend.ops_governance.sandbox.comparison_metrics.extract_closeout.md` - v4.16.0 BE-001MF-02 backend.ops_governance.sandbox.comparison_metrics actual extraction complete
递归边界补充: BE-001MF-03 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split；下一步: BE-001MG-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/766-backend.ops_governance.sandbox.comparison_metrics.single_leaf_closeout.md` - v4.16.0 BE-001MF-03 backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split
递归边界补充: BE-001MG-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape；下一步: BE-001MH-01 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape baseline_plan。
- `markdown/06-milestones/v4.16.0/767-backend.ops_governance.sandbox.comparison_metrics.parent_residual_judgment.v4_replay_shape.md` - v4.16.0 BE-001MG-01 backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape
递归边界补充: BE-001MH-01 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan；下一步: BE-001MH-02 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape extract_closeout。
- `markdown/06-milestones/v4.16.0/768-backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape.baseline_plan.md` - v4.16.0 BE-001MH-01 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan
递归边界补充: BE-001MH-02 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete；下一步: BE-001MH-03 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/769-backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape.extract_closeout.md` - v4.16.0 BE-001MH-02 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete
递归边界补充: BE-001MH-03 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout stops further split；下一步: BE-001MI-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/770-backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape.single_leaf_closeout.md` - v4.16.0 BE-001MH-03 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout stops further split
递归边界补充: BE-001MI-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection；下一步: BE-001MJ-01 backend.ops_governance.sandbox.comparison_metrics.backtest_projection baseline_plan。
- `markdown/06-milestones/v4.16.0/771-backend.ops_governance.sandbox.comparison_metrics.parent_residual_judgment.backtest_projection.md` - v4.16.0 BE-001MI-01 backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection
递归边界补充: BE-001MJ-01 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan；下一步: BE-001MJ-02 backend.ops_governance.sandbox.comparison_metrics.backtest_projection extract_closeout。
- `markdown/06-milestones/v4.16.0/772-backend.ops_governance.sandbox.comparison_metrics.backtest_projection.baseline_plan.md` - v4.16.0 BE-001MJ-01 backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan
递归边界补充: BE-001MJ-02 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete；下一步: BE-001MJ-03 backend.ops_governance.sandbox.comparison_metrics.backtest_projection single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/773-backend.ops_governance.sandbox.comparison_metrics.backtest_projection.extract_closeout.md` - v4.16.0 BE-001MJ-02 backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete
递归边界补充: BE-001MJ-03 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout stops further split；下一步: BE-001MK-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- `markdown/06-milestones/v4.16.0/774-backend.ops_governance.sandbox.comparison_metrics.backtest_projection.single_leaf_closeout.md` - v4.16.0 BE-001MJ-03 backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout stops further split
Recursive boundary supplement: BE-001MK-01 `backend.ops_governance.sandbox.comparison_metrics` parent residual judgment closes parent; next step: BE-001ML-01 backend.ops_governance.sandbox parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/775-backend.ops_governance.sandbox.comparison_metrics.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001MK-01 backend.ops_governance.sandbox.comparison_metrics parent residual judgment closes parent
Recursive boundary supplement: BE-001ML-01 `backend.ops_governance.sandbox` parent residual judgment selects proposal_loader; next step: BE-001MM-01 backend.ops_governance.sandbox.proposal_loader baseline_plan.
- `markdown/06-milestones/v4.16.0/776-backend.ops_governance.sandbox.parent_residual_judgment.proposal_loader.md` - v4.16.0 BE-001ML-01 backend.ops_governance.sandbox parent residual judgment selects proposal_loader
Recursive boundary supplement: BE-001MM-01 `backend.ops_governance.sandbox.proposal_loader` equivalence baseline and extraction plan; next step: BE-001MM-02 backend.ops_governance.sandbox.proposal_loader extract_closeout.
- `markdown/06-milestones/v4.16.0/777-backend.ops_governance.sandbox.proposal_loader.baseline_plan.md` - v4.16.0 BE-001MM-01 backend.ops_governance.sandbox.proposal_loader equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MM-02 `backend.ops_governance.sandbox.proposal_loader` actual extraction complete; next step: BE-001MM-03 backend.ops_governance.sandbox.proposal_loader single_leaf_closeout.
- `src/backend/ops_governance/sandbox/proposal_loader.rs` - backend.ops_governance.sandbox.proposal_loader implementation
- `markdown/06-milestones/v4.16.0/778-backend.ops_governance.sandbox.proposal_loader.extract_closeout.md` - v4.16.0 BE-001MM-02 backend.ops_governance.sandbox.proposal_loader actual extraction complete
Recursive boundary supplement: BE-001MM-03 `backend.ops_governance.sandbox.proposal_loader` single leaf closeout stops further split; next step: BE-001MN-01 backend.ops_governance.sandbox parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/779-backend.ops_governance.sandbox.proposal_loader.single_leaf_closeout.md` - v4.16.0 BE-001MM-03 backend.ops_governance.sandbox.proposal_loader single leaf closeout stops further split
Recursive boundary supplement: BE-001MN-01 `backend.ops_governance.sandbox` parent residual judgment selects report_disk_loader; next step: BE-001MO-01 backend.ops_governance.sandbox.report_disk_loader baseline_plan.
- `markdown/06-milestones/v4.16.0/780-backend.ops_governance.sandbox.parent_residual_judgment.report_disk_loader.md` - v4.16.0 BE-001MN-01 backend.ops_governance.sandbox parent residual judgment selects report_disk_loader
Recursive boundary supplement: BE-001MO-01 `backend.ops_governance.sandbox.report_disk_loader` equivalence baseline and extraction plan; next step: BE-001MO-02 backend.ops_governance.sandbox.report_disk_loader extract_closeout.
- `markdown/06-milestones/v4.16.0/781-backend.ops_governance.sandbox.report_disk_loader.baseline_plan.md` - v4.16.0 BE-001MO-01 backend.ops_governance.sandbox.report_disk_loader equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MO-02 `backend.ops_governance.sandbox.report_disk_loader` actual extraction complete; next step: BE-001MO-03 backend.ops_governance.sandbox.report_disk_loader single_leaf_closeout.
- `src/backend/ops_governance/sandbox/report_disk_loader.rs` - backend.ops_governance.sandbox.report_disk_loader implementation
- `markdown/06-milestones/v4.16.0/782-backend.ops_governance.sandbox.report_disk_loader.extract_closeout.md` - v4.16.0 BE-001MO-02 backend.ops_governance.sandbox.report_disk_loader actual extraction complete
Recursive boundary supplement: BE-001MO-03 `backend.ops_governance.sandbox.report_disk_loader` single leaf closeout stops further split; next step: BE-001MP-01 backend.ops_governance.sandbox parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/783-backend.ops_governance.sandbox.report_disk_loader.single_leaf_closeout.md` - v4.16.0 BE-001MO-03 backend.ops_governance.sandbox.report_disk_loader single leaf closeout stops further split
Recursive boundary supplement: BE-001MP-01 `backend.ops_governance.sandbox` parent residual judgment closes parent; next step: BE-001MQ-01 backend.ops_governance parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/784-backend.ops_governance.sandbox.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001MP-01 backend.ops_governance.sandbox parent residual judgment closes parent
Recursive boundary supplement: BE-001MQ-01 `backend.ops_governance` parent residual judgment selects alerts; next step: BE-001MR-01 backend.ops_governance.alerts baseline_plan.
- `markdown/06-milestones/v4.16.0/785-backend.ops_governance.parent_residual_judgment.alerts.md` - v4.16.0 BE-001MQ-01 backend.ops_governance parent residual judgment selects alerts
Recursive boundary supplement: BE-001MR-01 `backend.ops_governance.alerts` equivalence baseline and extraction plan; next step: BE-001MR-02 backend.ops_governance.alerts extract_closeout.
- `markdown/06-milestones/v4.16.0/786-backend.ops_governance.alerts.baseline_plan.md` - v4.16.0 BE-001MR-01 backend.ops_governance.alerts equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MR-02 `backend.ops_governance.alerts` actual extraction complete; next step: BE-001MR-03 backend.ops_governance.alerts single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers.rs` - backend.ops_governance.alerts handler implementation
- `markdown/06-milestones/v4.16.0/787-backend.ops_governance.alerts.extract_closeout.md` - v4.16.0 BE-001MR-02 backend.ops_governance.alerts actual extraction complete
Recursive boundary supplement: BE-001MR-03 `backend.ops_governance.alerts` single leaf closeout continues split; next step: BE-001MS-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/788-backend.ops_governance.alerts.single_leaf_closeout.md` - v4.16.0 BE-001MR-03 backend.ops_governance.alerts single leaf closeout continues split
Recursive boundary supplement: BE-001MS-01 `backend.ops_governance.alerts` parent residual judgment selects rule_catalog; next step: BE-001MT-01 backend.ops_governance.alerts.rule_catalog baseline_plan.
- `markdown/06-milestones/v4.16.0/789-backend.ops_governance.alerts.parent_residual_judgment.rule_catalog.md` - v4.16.0 BE-001MS-01 backend.ops_governance.alerts parent residual judgment selects rule_catalog
Recursive boundary supplement: BE-001MT-01 `backend.ops_governance.alerts.rule_catalog` equivalence baseline and extraction plan; next step: BE-001MT-02 backend.ops_governance.alerts.rule_catalog extract_closeout.
- `markdown/06-milestones/v4.16.0/790-backend.ops_governance.alerts.rule_catalog.baseline_plan.md` - v4.16.0 BE-001MT-01 backend.ops_governance.alerts.rule_catalog equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MT-02 `backend.ops_governance.alerts.rule_catalog` actual extraction complete; next step: BE-001MT-03 backend.ops_governance.alerts.rule_catalog single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/rule_catalog.rs` - backend.ops_governance.alerts.rule_catalog implementation
- `markdown/06-milestones/v4.16.0/791-backend.ops_governance.alerts.rule_catalog.extract_closeout.md` - v4.16.0 BE-001MT-02 backend.ops_governance.alerts.rule_catalog actual extraction complete
Recursive boundary supplement: BE-001MT-03 `backend.ops_governance.alerts.rule_catalog` single leaf closeout stops further split; next step: BE-001MU-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/792-backend.ops_governance.alerts.rule_catalog.single_leaf_closeout.md` - v4.16.0 BE-001MT-03 backend.ops_governance.alerts.rule_catalog single leaf closeout stops further split
Recursive boundary supplement: BE-001MU-01 `backend.ops_governance.alerts` parent residual judgment selects acknowledge_flow; next step: BE-001MV-01 backend.ops_governance.alerts.acknowledge_flow baseline_plan.
- `markdown/06-milestones/v4.16.0/793-backend.ops_governance.alerts.parent_residual_judgment.acknowledge_flow.md` - v4.16.0 BE-001MU-01 backend.ops_governance.alerts parent residual judgment selects acknowledge_flow
Recursive boundary supplement: BE-001MV-01 `backend.ops_governance.alerts.acknowledge_flow` equivalence baseline and extraction plan; next step: BE-001MV-02 backend.ops_governance.alerts.acknowledge_flow extract_closeout.
- `markdown/06-milestones/v4.16.0/794-backend.ops_governance.alerts.acknowledge_flow.baseline_plan.md` - v4.16.0 BE-001MV-01 backend.ops_governance.alerts.acknowledge_flow equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MV-02 `backend.ops_governance.alerts.acknowledge_flow` actual extraction complete; next step: BE-001MV-03 backend.ops_governance.alerts.acknowledge_flow single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/acknowledge_flow.rs` - backend.ops_governance.alerts.acknowledge_flow implementation
- `markdown/06-milestones/v4.16.0/795-backend.ops_governance.alerts.acknowledge_flow.extract_closeout.md` - v4.16.0 BE-001MV-02 backend.ops_governance.alerts.acknowledge_flow actual extraction complete
Recursive boundary supplement: BE-001MV-03 `backend.ops_governance.alerts.acknowledge_flow` single leaf closeout stops further split; next step: BE-001MW-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/796-backend.ops_governance.alerts.acknowledge_flow.single_leaf_closeout.md` - v4.16.0 BE-001MV-03 backend.ops_governance.alerts.acknowledge_flow single leaf closeout stops further split
Recursive boundary supplement: BE-001MW-01 `backend.ops_governance.alerts` parent residual judgment selects trigger_engine; next step: BE-001MX-01 backend.ops_governance.alerts.trigger_engine baseline_plan.
- `markdown/06-milestones/v4.16.0/797-backend.ops_governance.alerts.parent_residual_judgment.trigger_engine.md` - v4.16.0 BE-001MW-01 backend.ops_governance.alerts parent residual judgment selects trigger_engine
Recursive boundary supplement: BE-001MX-01 `backend.ops_governance.alerts.trigger_engine` equivalence baseline and extraction plan; next step: BE-001MX-02 backend.ops_governance.alerts.trigger_engine extract_closeout.
- `markdown/06-milestones/v4.16.0/798-backend.ops_governance.alerts.trigger_engine.baseline_plan.md` - v4.16.0 BE-001MX-01 backend.ops_governance.alerts.trigger_engine equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MX-02 `backend.ops_governance.alerts.trigger_engine` actual extraction complete; next step: BE-001MX-03 backend.ops_governance.alerts.trigger_engine single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/trigger_engine.rs` - backend.ops_governance.alerts.trigger_engine implementation
- `markdown/06-milestones/v4.16.0/799-backend.ops_governance.alerts.trigger_engine.extract_closeout.md` - v4.16.0 BE-001MX-02 backend.ops_governance.alerts.trigger_engine actual extraction complete
Recursive boundary supplement: BE-001MX-03 `backend.ops_governance.alerts.trigger_engine` single leaf closeout stops further split; next step: BE-001MY-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/800-backend.ops_governance.alerts.trigger_engine.single_leaf_closeout.md` - v4.16.0 BE-001MX-03 backend.ops_governance.alerts.trigger_engine single leaf closeout stops further split
Recursive boundary supplement: BE-001MY-01 `backend.ops_governance.alerts` parent residual judgment selects predicate_checks; next step: BE-001MZ-01 backend.ops_governance.alerts.predicate_checks baseline_plan.
- `markdown/06-milestones/v4.16.0/801-backend.ops_governance.alerts.parent_residual_judgment.predicate_checks.md` - v4.16.0 BE-001MY-01 backend.ops_governance.alerts parent residual judgment selects predicate_checks
Recursive boundary supplement: BE-001MZ-01 `backend.ops_governance.alerts.predicate_checks` equivalence baseline and extraction plan; next step: BE-001MZ-02 backend.ops_governance.alerts.predicate_checks extract_closeout.
- `markdown/06-milestones/v4.16.0/802-backend.ops_governance.alerts.predicate_checks.baseline_plan.md` - v4.16.0 BE-001MZ-01 backend.ops_governance.alerts.predicate_checks equivalence baseline and extraction plan
Recursive boundary supplement: BE-001MZ-02 `backend.ops_governance.alerts.predicate_checks` actual extraction complete; next step: BE-001MZ-03 backend.ops_governance.alerts.predicate_checks single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/predicate_checks.rs` - backend.ops_governance.alerts.predicate_checks implementation
- `markdown/06-milestones/v4.16.0/803-backend.ops_governance.alerts.predicate_checks.extract_closeout.md` - v4.16.0 BE-001MZ-02 backend.ops_governance.alerts.predicate_checks actual extraction complete
Recursive boundary supplement: BE-001MZ-03 `backend.ops_governance.alerts.predicate_checks` single leaf closeout stops further split; next step: BE-001NA-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/804-backend.ops_governance.alerts.predicate_checks.single_leaf_closeout.md` - v4.16.0 BE-001MZ-03 backend.ops_governance.alerts.predicate_checks single leaf closeout stops further split
Recursive boundary supplement: BE-001NA-01 `backend.ops_governance.alerts` parent residual judgment selects persistence; next step: BE-001NA-02 backend.ops_governance.alerts.persistence baseline_plan.
- `markdown/06-milestones/v4.16.0/805-backend.ops_governance.alerts.parent_residual_judgment.persistence.md` - v4.16.0 BE-001NA-01 backend.ops_governance.alerts parent residual judgment selects persistence
Recursive boundary supplement: BE-001NA-02 `backend.ops_governance.alerts.persistence` equivalence baseline and extraction plan; next step: BE-001NA-03 backend.ops_governance.alerts.persistence extract_closeout.
- `markdown/06-milestones/v4.16.0/806-backend.ops_governance.alerts.persistence.baseline_plan.md` - v4.16.0 BE-001NA-02 backend.ops_governance.alerts.persistence equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NA-03 `backend.ops_governance.alerts.persistence` actual extraction complete; next step: BE-001NA-04 backend.ops_governance.alerts.persistence single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/persistence.rs` - backend.ops_governance.alerts.persistence implementation
- `markdown/06-milestones/v4.16.0/807-backend.ops_governance.alerts.persistence.extract_closeout.md` - v4.16.0 BE-001NA-03 backend.ops_governance.alerts.persistence actual extraction complete
Recursive boundary supplement: BE-001NA-04 `backend.ops_governance.alerts.persistence` single leaf closeout stops further split; next step: BE-001NB-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/808-backend.ops_governance.alerts.persistence.single_leaf_closeout.md` - v4.16.0 BE-001NA-04 backend.ops_governance.alerts.persistence single leaf closeout stops further split
Recursive boundary supplement: BE-001NB-01 `backend.ops_governance.alerts` parent residual judgment selects startup_initialization; next step: BE-001NB-02 backend.ops_governance.alerts.startup_initialization baseline_plan.
- `markdown/06-milestones/v4.16.0/809-backend.ops_governance.alerts.parent_residual_judgment.startup_initialization.md` - v4.16.0 BE-001NB-01 backend.ops_governance.alerts parent residual judgment selects startup_initialization
Recursive boundary supplement: BE-001NB-02 `backend.ops_governance.alerts.startup_initialization` equivalence baseline and extraction plan; next step: BE-001NB-03 backend.ops_governance.alerts.startup_initialization extract_closeout.
- `markdown/06-milestones/v4.16.0/810-backend.ops_governance.alerts.startup_initialization.baseline_plan.md` - v4.16.0 BE-001NB-02 backend.ops_governance.alerts.startup_initialization equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NB-03 `backend.ops_governance.alerts.startup_initialization` actual extraction complete; next step: BE-001NB-04 backend.ops_governance.alerts.startup_initialization single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/startup_initialization.rs` - backend.ops_governance.alerts.startup_initialization implementation
- `markdown/06-milestones/v4.16.0/811-backend.ops_governance.alerts.startup_initialization.extract_closeout.md` - v4.16.0 BE-001NB-03 backend.ops_governance.alerts.startup_initialization actual extraction complete
Recursive boundary supplement: BE-001NB-04 `backend.ops_governance.alerts.startup_initialization` single leaf closeout stops further split; next step: BE-001NC-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/812-backend.ops_governance.alerts.startup_initialization.single_leaf_closeout.md` - v4.16.0 BE-001NB-04 backend.ops_governance.alerts.startup_initialization single leaf closeout stops further split
Recursive boundary supplement: BE-001NC-01 `backend.ops_governance.alerts` parent residual judgment selects read_routes; next step: BE-001NC-02 backend.ops_governance.alerts.read_routes baseline_plan.
- `markdown/06-milestones/v4.16.0/813-backend.ops_governance.alerts.parent_residual_judgment.read_routes.md` - v4.16.0 BE-001NC-01 backend.ops_governance.alerts parent residual judgment selects read_routes
Recursive boundary supplement: BE-001NC-02 `backend.ops_governance.alerts.read_routes` equivalence baseline and extraction plan; next step: BE-001NC-03 backend.ops_governance.alerts.read_routes extract_closeout.
- `markdown/06-milestones/v4.16.0/814-backend.ops_governance.alerts.read_routes.baseline_plan.md` - v4.16.0 BE-001NC-02 backend.ops_governance.alerts.read_routes equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NC-03 `backend.ops_governance.alerts.read_routes` actual extraction complete; next step: BE-001NC-04 backend.ops_governance.alerts.read_routes single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/read_routes.rs` - backend.ops_governance.alerts.read_routes implementation
- `markdown/06-milestones/v4.16.0/815-backend.ops_governance.alerts.read_routes.extract_closeout.md` - v4.16.0 BE-001NC-03 backend.ops_governance.alerts.read_routes actual extraction complete
Recursive boundary supplement: BE-001NC-04 `backend.ops_governance.alerts.read_routes` single leaf closeout stops further split; next step: BE-001ND-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/816-backend.ops_governance.alerts.read_routes.single_leaf_closeout.md` - v4.16.0 BE-001NC-04 backend.ops_governance.alerts.read_routes single leaf closeout stops further split
Recursive boundary supplement: BE-001ND-01 `backend.ops_governance.alerts.route_facade` static closeout and recovery_bridge selection; next step: BE-001NE-01 backend.ops_governance.alerts.recovery_bridge baseline_plan.
- `markdown/06-milestones/v4.16.0/817-backend.ops_governance.alerts.parent_residual_judgment.route_facade.md` - v4.16.0 BE-001ND-01 backend.ops_governance.alerts route_facade static closeout and recovery_bridge selection
Recursive boundary supplement: BE-001NE-01 `backend.ops_governance.alerts.recovery_bridge` equivalence baseline and extraction plan; next step: BE-001NE-02 backend.ops_governance.alerts.recovery_bridge extract_closeout.
- `markdown/06-milestones/v4.16.0/818-backend.ops_governance.alerts.recovery_bridge.baseline_plan.md` - v4.16.0 BE-001NE-01 backend.ops_governance.alerts.recovery_bridge equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NE-02 `backend.ops_governance.alerts.recovery_bridge` actual extraction complete; next step: BE-001NE-03 backend.ops_governance.alerts.recovery_bridge single_leaf_closeout.
- `src/backend/ops_governance/alerts/handlers/recovery_bridge.rs` - backend.ops_governance.alerts.recovery_bridge implementation
- `markdown/06-milestones/v4.16.0/819-backend.ops_governance.alerts.recovery_bridge.extract_closeout.md` - v4.16.0 BE-001NE-02 backend.ops_governance.alerts.recovery_bridge actual extraction complete
Recursive boundary supplement: BE-001NE-03 `backend.ops_governance.alerts.recovery_bridge` single leaf closeout stops further split; next step: BE-001NF-01 backend.ops_governance.alerts parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/820-backend.ops_governance.alerts.recovery_bridge.single_leaf_closeout.md` - v4.16.0 BE-001NE-03 backend.ops_governance.alerts.recovery_bridge single leaf closeout stops further split
Recursive boundary supplement: BE-001NF-01 `backend.ops_governance.alerts` parent residual judgment closes parent; next step: BE-001NG-01 backend.ops_governance parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/821-backend.ops_governance.alerts.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001NF-01 backend.ops_governance.alerts parent residual judgment closes parent
Recursive boundary supplement: BE-001NG-01 `backend.ops_governance` parent residual judgment selects snapshots; next step: BE-001NH-01 backend.ops_governance.snapshots baseline_plan.
- `markdown/06-milestones/v4.16.0/822-backend.ops_governance.parent_residual_judgment.snapshots.md` - v4.16.0 BE-001NG-01 backend.ops_governance parent residual judgment selects snapshots
Recursive boundary supplement: BE-001NH-01 `backend.ops_governance.snapshots` equivalence baseline and extraction plan; next step: BE-001NH-02 backend.ops_governance.snapshots extract_closeout.
- `markdown/06-milestones/v4.16.0/823-backend.ops_governance.snapshots.baseline_plan.md` - v4.16.0 BE-001NH-01 backend.ops_governance.snapshots equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NH-02 `backend.ops_governance.snapshots` actual extraction complete; next step: BE-001NH-03 backend.ops_governance.snapshots single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers.rs` - backend.ops_governance.snapshots handler implementation
- `markdown/06-milestones/v4.16.0/824-backend.ops_governance.snapshots.extract_closeout.md` - v4.16.0 BE-001NH-02 backend.ops_governance.snapshots actual extraction complete
Recursive boundary supplement: BE-001NH-03 `backend.ops_governance.snapshots` single leaf closeout continues split; next step: BE-001NI-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/825-backend.ops_governance.snapshots.single_leaf_closeout.md` - v4.16.0 BE-001NH-03 backend.ops_governance.snapshots single leaf closeout continues split
Recursive boundary supplement: BE-001NI-01 `backend.ops_governance.snapshots` parent residual judgment selects snapshot_id_validation; next step: BE-001NJ-01 backend.ops_governance.snapshots.snapshot_id_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/826-backend.ops_governance.snapshots.parent_residual_judgment.snapshot_id_validation.md` - v4.16.0 BE-001NI-01 backend.ops_governance.snapshots parent residual judgment selects snapshot_id_validation
Recursive boundary supplement: BE-001NJ-01 `backend.ops_governance.snapshots.snapshot_id_validation` equivalence baseline and extraction plan; next step: BE-001NJ-02 backend.ops_governance.snapshots.snapshot_id_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/827-backend.ops_governance.snapshots.snapshot_id_validation.baseline_plan.md` - v4.16.0 BE-001NJ-01 backend.ops_governance.snapshots.snapshot_id_validation equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NJ-02 `backend.ops_governance.snapshots.snapshot_id_validation` actual extraction complete; next step: BE-001NJ-03 backend.ops_governance.snapshots.snapshot_id_validation single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/snapshot_id_validation.rs` - backend.ops_governance.snapshots.snapshot_id_validation implementation
- `markdown/06-milestones/v4.16.0/828-backend.ops_governance.snapshots.snapshot_id_validation.extract_closeout.md` - v4.16.0 BE-001NJ-02 backend.ops_governance.snapshots.snapshot_id_validation actual extraction complete
Recursive boundary supplement: BE-001NJ-03 `backend.ops_governance.snapshots.snapshot_id_validation` single leaf closeout stops further split; next step: BE-001NK-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/829-backend.ops_governance.snapshots.snapshot_id_validation.single_leaf_closeout.md` - v4.16.0 BE-001NJ-03 backend.ops_governance.snapshots.snapshot_id_validation single leaf closeout stops further split
Recursive boundary supplement: BE-001NK-01 `backend.ops_governance.snapshots` parent residual judgment selects create_flow; next step: BE-001NL-01 backend.ops_governance.snapshots.create_flow baseline_plan.
- `markdown/06-milestones/v4.16.0/830-backend.ops_governance.snapshots.parent_residual_judgment.create_flow.md` - v4.16.0 BE-001NK-01 backend.ops_governance.snapshots parent residual judgment selects create_flow
Recursive boundary supplement: BE-001NL-01 `backend.ops_governance.snapshots.create_flow` equivalence baseline and extraction plan; next step: BE-001NL-02 backend.ops_governance.snapshots.create_flow extract_closeout.
- `markdown/06-milestones/v4.16.0/831-backend.ops_governance.snapshots.create_flow.baseline_plan.md` - v4.16.0 BE-001NL-01 backend.ops_governance.snapshots.create_flow equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NL-02 `backend.ops_governance.snapshots.create_flow` actual extraction complete; next step: BE-001NL-03 backend.ops_governance.snapshots.create_flow single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/create_flow.rs` - backend.ops_governance.snapshots.create_flow implementation
- `markdown/06-milestones/v4.16.0/832-backend.ops_governance.snapshots.create_flow.extract_closeout.md` - v4.16.0 BE-001NL-02 backend.ops_governance.snapshots.create_flow actual extraction complete
Recursive boundary supplement: BE-001NL-03 `backend.ops_governance.snapshots.create_flow` single leaf closeout stops further split; next step: BE-001NM-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/833-backend.ops_governance.snapshots.create_flow.single_leaf_closeout.md` - v4.16.0 BE-001NL-03 backend.ops_governance.snapshots.create_flow single leaf closeout stops further split
Recursive boundary supplement: BE-001NM-01 `backend.ops_governance.snapshots` parent residual judgment selects read_routes; next step: BE-001NN-01 backend.ops_governance.snapshots.read_routes baseline_plan.
- `markdown/06-milestones/v4.16.0/834-backend.ops_governance.snapshots.parent_residual_judgment.read_routes.md` - v4.16.0 BE-001NM-01 backend.ops_governance.snapshots parent residual judgment selects read_routes
Recursive boundary supplement: BE-001NN-01 `backend.ops_governance.snapshots.read_routes` equivalence baseline and extraction plan; next step: BE-001NN-02 backend.ops_governance.snapshots.read_routes extract_closeout.
- `markdown/06-milestones/v4.16.0/835-backend.ops_governance.snapshots.read_routes.baseline_plan.md` - v4.16.0 BE-001NN-01 backend.ops_governance.snapshots.read_routes equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NN-02 `backend.ops_governance.snapshots.read_routes` actual extraction complete; next step: BE-001NN-03 backend.ops_governance.snapshots.read_routes single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/read_routes.rs` - backend.ops_governance.snapshots.read_routes implementation
- `markdown/06-milestones/v4.16.0/836-backend.ops_governance.snapshots.read_routes.extract_closeout.md` - v4.16.0 BE-001NN-02 backend.ops_governance.snapshots.read_routes actual extraction complete
Recursive boundary supplement: BE-001NN-03 `backend.ops_governance.snapshots.read_routes` single leaf closeout stops further split; next step: BE-001NO-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/837-backend.ops_governance.snapshots.read_routes.single_leaf_closeout.md` - v4.16.0 BE-001NN-03 backend.ops_governance.snapshots.read_routes single leaf closeout stops further split
Recursive boundary supplement: BE-001NO-01 `backend.ops_governance.snapshots` parent residual judgment selects restore_flow; next step: BE-001NP-01 backend.ops_governance.snapshots.restore_flow baseline_plan.
- `markdown/06-milestones/v4.16.0/838-backend.ops_governance.snapshots.parent_residual_judgment.restore_flow.md` - v4.16.0 BE-001NO-01 backend.ops_governance.snapshots parent residual judgment selects restore_flow
Recursive boundary supplement: BE-001NP-01 `backend.ops_governance.snapshots.restore_flow` equivalence baseline and extraction plan; next step: BE-001NP-02 backend.ops_governance.snapshots.restore_flow extract_closeout.
- `markdown/06-milestones/v4.16.0/839-backend.ops_governance.snapshots.restore_flow.baseline_plan.md` - v4.16.0 BE-001NP-01 backend.ops_governance.snapshots.restore_flow equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NP-02 `backend.ops_governance.snapshots.restore_flow` actual extraction complete; next step: BE-001NP-03 backend.ops_governance.snapshots.restore_flow single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/restore_flow.rs` - backend.ops_governance.snapshots.restore_flow implementation
- `markdown/06-milestones/v4.16.0/840-backend.ops_governance.snapshots.restore_flow.extract_closeout.md` - v4.16.0 BE-001NP-02 backend.ops_governance.snapshots.restore_flow actual extraction complete
Recursive boundary supplement: BE-001NP-03 `backend.ops_governance.snapshots.restore_flow` single leaf closeout stops further split; next step: BE-001NQ-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/841-backend.ops_governance.snapshots.restore_flow.single_leaf_closeout.md` - v4.16.0 BE-001NP-03 backend.ops_governance.snapshots.restore_flow single leaf closeout stops further split
Recursive boundary supplement: BE-001NQ-01 `backend.ops_governance.snapshots` parent residual judgment selects persistence; next step: BE-001NR-01 backend.ops_governance.snapshots.persistence baseline_plan.
- `markdown/06-milestones/v4.16.0/842-backend.ops_governance.snapshots.parent_residual_judgment.persistence.md` - v4.16.0 BE-001NQ-01 backend.ops_governance.snapshots parent residual judgment selects persistence
Recursive boundary supplement: BE-001NR-01 `backend.ops_governance.snapshots.persistence` equivalence baseline and extraction plan; next step: BE-001NR-02 backend.ops_governance.snapshots.persistence extract_closeout.
- `markdown/06-milestones/v4.16.0/843-backend.ops_governance.snapshots.persistence.baseline_plan.md` - v4.16.0 BE-001NR-01 backend.ops_governance.snapshots.persistence equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NR-02 `backend.ops_governance.snapshots.persistence` actual extraction complete; next step: BE-001NR-03 backend.ops_governance.snapshots.persistence single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/persistence.rs` - backend.ops_governance.snapshots.persistence implementation
- `markdown/06-milestones/v4.16.0/844-backend.ops_governance.snapshots.persistence.extract_closeout.md` - v4.16.0 BE-001NR-02 backend.ops_governance.snapshots.persistence actual extraction complete
Recursive boundary supplement: BE-001NR-03 `backend.ops_governance.snapshots.persistence` single leaf closeout stops further split; next step: BE-001NS-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/845-backend.ops_governance.snapshots.persistence.single_leaf_closeout.md` - v4.16.0 BE-001NR-03 backend.ops_governance.snapshots.persistence single leaf closeout stops further split
Recursive boundary supplement: BE-001NS-01 `backend.ops_governance.snapshots` parent residual judgment selects signature_contract; next step: BE-001NT-01 backend.ops_governance.snapshots.signature_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/846-backend.ops_governance.snapshots.parent_residual_judgment.signature_contract.md` - v4.16.0 BE-001NS-01 backend.ops_governance.snapshots parent residual judgment selects signature_contract
Recursive boundary supplement: BE-001NT-01 `backend.ops_governance.snapshots.signature_contract` equivalence baseline and extraction plan; next step: BE-001NT-02 backend.ops_governance.snapshots.signature_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/847-backend.ops_governance.snapshots.signature_contract.baseline_plan.md` - v4.16.0 BE-001NT-01 backend.ops_governance.snapshots.signature_contract equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NT-02 `backend.ops_governance.snapshots.signature_contract` actual extraction complete; next step: BE-001NT-03 backend.ops_governance.snapshots.signature_contract single_leaf_closeout.
- `src/backend/ops_governance/snapshots/handlers/signature_contract.rs` - backend.ops_governance.snapshots.signature_contract implementation
- `markdown/06-milestones/v4.16.0/848-backend.ops_governance.snapshots.signature_contract.extract_closeout.md` - v4.16.0 BE-001NT-02 backend.ops_governance.snapshots.signature_contract actual extraction complete
Recursive boundary supplement: BE-001NT-03 `backend.ops_governance.snapshots.signature_contract` single leaf closeout stops further split; next step: BE-001NU-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/849-backend.ops_governance.snapshots.signature_contract.single_leaf_closeout.md` - v4.16.0 BE-001NT-03 backend.ops_governance.snapshots.signature_contract single leaf closeout stops further split
Recursive boundary supplement: BE-001NU-01 `backend.ops_governance.snapshots.route_facade` static closeout and parent closeout selection; next step: BE-001NV-01 backend.ops_governance.snapshots parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/850-backend.ops_governance.snapshots.parent_residual_judgment.route_facade.md` - v4.16.0 BE-001NU-01 backend.ops_governance.snapshots route_facade static closeout and parent closeout selection
Recursive boundary supplement: BE-001NV-01 `backend.ops_governance.snapshots` parent residual judgment closes parent; next step: BE-001NW-01 backend.ops_governance parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/851-backend.ops_governance.snapshots.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001NV-01 backend.ops_governance.snapshots parent residual judgment closes parent
Recursive boundary supplement: BE-001NW-01 `backend.ops_governance` parent residual judgment selects runbook; next step: BE-001NX-01 backend.ops_governance.runbook baseline_plan.
- `markdown/06-milestones/v4.16.0/852-backend.ops_governance.parent_residual_judgment.runbook.md` - v4.16.0 BE-001NW-01 backend.ops_governance parent residual judgment selects runbook
Recursive boundary supplement: BE-001NX-01 `backend.ops_governance.runbook` equivalence baseline and extraction plan; next step: BE-001NX-02 backend.ops_governance.runbook extract_closeout.
- `markdown/06-milestones/v4.16.0/853-backend.ops_governance.runbook.baseline_plan.md` - v4.16.0 BE-001NX-01 backend.ops_governance.runbook equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NX-02 `backend.ops_governance.runbook` actual extraction complete; next step: BE-001NX-03 backend.ops_governance.runbook single_leaf_closeout.
- `src/backend/ops_governance/runbook/handlers.rs` - backend.ops_governance.runbook handler implementation
- `markdown/06-milestones/v4.16.0/854-backend.ops_governance.runbook.extract_closeout.md` - v4.16.0 BE-001NX-02 backend.ops_governance.runbook actual extraction complete
Recursive boundary supplement: BE-001NX-03 `backend.ops_governance.runbook` single leaf closeout continues split; next step: BE-001NY-01 backend.ops_governance.runbook parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/855-backend.ops_governance.runbook.single_leaf_closeout.md` - v4.16.0 BE-001NX-03 backend.ops_governance.runbook single leaf closeout continues split
Recursive boundary supplement: BE-001NY-01 `backend.ops_governance.runbook` parent residual judgment selects scenario_catalog; next step: BE-001NZ-01 backend.ops_governance.runbook.scenario_catalog baseline_plan.
- `markdown/06-milestones/v4.16.0/856-backend.ops_governance.runbook.parent_residual_judgment.scenario_catalog.md` - v4.16.0 BE-001NY-01 backend.ops_governance.runbook parent residual judgment selects scenario_catalog
Recursive boundary supplement: BE-001NZ-01 `backend.ops_governance.runbook.scenario_catalog` equivalence baseline and extraction plan; next step: BE-001NZ-02 backend.ops_governance.runbook.scenario_catalog extract_closeout.
- `markdown/06-milestones/v4.16.0/857-backend.ops_governance.runbook.scenario_catalog.baseline_plan.md` - v4.16.0 BE-001NZ-01 backend.ops_governance.runbook.scenario_catalog equivalence baseline and extraction plan
Recursive boundary supplement: BE-001NZ-02 `backend.ops_governance.runbook.scenario_catalog` actual extraction complete; next step: BE-001NZ-03 backend.ops_governance.runbook.scenario_catalog single_leaf_closeout.
- `src/backend/ops_governance/runbook/handlers/scenario_catalog.rs` - backend.ops_governance.runbook.scenario_catalog implementation
- `markdown/06-milestones/v4.16.0/858-backend.ops_governance.runbook.scenario_catalog.extract_closeout.md` - v4.16.0 BE-001NZ-02 backend.ops_governance.runbook.scenario_catalog actual extraction complete
Recursive boundary supplement: BE-001NZ-03 `backend.ops_governance.runbook.scenario_catalog` single leaf closeout; next step: BE-001OA-01 backend.ops_governance.runbook parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/859-backend.ops_governance.runbook.scenario_catalog.single_leaf_closeout.md` - v4.16.0 BE-001NZ-03 backend.ops_governance.runbook.scenario_catalog single leaf closeout
Recursive boundary supplement: BE-001OA-01 `backend.ops_governance.runbook` parent residual judgment selects read_routes; next step: BE-001OB-01 backend.ops_governance.runbook.read_routes baseline_plan.
- `markdown/06-milestones/v4.16.0/860-backend.ops_governance.runbook.parent_residual_judgment.read_routes.md` - v4.16.0 BE-001OA-01 backend.ops_governance.runbook parent residual judgment selects read_routes
Recursive boundary supplement: BE-001OB-01 `backend.ops_governance.runbook.read_routes` equivalence baseline and extraction plan; next step: BE-001OB-02 backend.ops_governance.runbook.read_routes extract_closeout.
- `markdown/06-milestones/v4.16.0/861-backend.ops_governance.runbook.read_routes.baseline_plan.md` - v4.16.0 BE-001OB-01 backend.ops_governance.runbook.read_routes equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OB-02 `backend.ops_governance.runbook.read_routes` actual extraction complete; next step: BE-001OB-03 backend.ops_governance.runbook.read_routes single_leaf_closeout.
- `src/backend/ops_governance/runbook/handlers/read_routes.rs` - backend.ops_governance.runbook.read_routes implementation
- `markdown/06-milestones/v4.16.0/862-backend.ops_governance.runbook.read_routes.extract_closeout.md` - v4.16.0 BE-001OB-02 backend.ops_governance.runbook.read_routes actual extraction complete
Recursive boundary supplement: BE-001OB-03 `backend.ops_governance.runbook.read_routes` single leaf closeout; next step: BE-001OC-01 backend.ops_governance.runbook parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/863-backend.ops_governance.runbook.read_routes.single_leaf_closeout.md` - v4.16.0 BE-001OB-03 backend.ops_governance.runbook.read_routes single leaf closeout
Recursive boundary supplement: BE-001OC-01 `backend.ops_governance.runbook` parent residual judgment selects route_facade; next step: BE-001OD-01 backend.ops_governance.runbook.route_facade baseline_plan.
- `markdown/06-milestones/v4.16.0/864-backend.ops_governance.runbook.parent_residual_judgment.route_facade.md` - v4.16.0 BE-001OC-01 backend.ops_governance.runbook parent residual judgment selects route_facade
Recursive boundary supplement: BE-001OD-01 `backend.ops_governance.runbook.route_facade` equivalence baseline and extraction plan; next step: BE-001OD-02 backend.ops_governance.runbook.route_facade extract_closeout.
- `markdown/06-milestones/v4.16.0/865-backend.ops_governance.runbook.route_facade.baseline_plan.md` - v4.16.0 BE-001OD-01 backend.ops_governance.runbook.route_facade equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OD-02 `backend.ops_governance.runbook.route_facade` actual extraction complete; next step: BE-001OD-03 backend.ops_governance.runbook.route_facade single_leaf_closeout.
- `src/backend/ops_governance/runbook/handlers/route_facade.rs` - backend.ops_governance.runbook.route_facade implementation
- `markdown/06-milestones/v4.16.0/866-backend.ops_governance.runbook.route_facade.extract_closeout.md` - v4.16.0 BE-001OD-02 backend.ops_governance.runbook.route_facade actual extraction complete
Recursive boundary supplement: BE-001OD-03 `backend.ops_governance.runbook.route_facade` single leaf closeout; next step: BE-001OE-01 backend.ops_governance.runbook parent_closeout.
- `markdown/06-milestones/v4.16.0/867-backend.ops_governance.runbook.route_facade.single_leaf_closeout.md` - v4.16.0 BE-001OD-03 backend.ops_governance.runbook.route_facade single leaf closeout
Recursive boundary supplement: BE-001OE-01 `backend.ops_governance.runbook` parent closeout; next step: BE-001OF-01 backend.ops_governance parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/868-backend.ops_governance.runbook.parent_closeout.md` - v4.16.0 BE-001OE-01 backend.ops_governance.runbook parent closeout
Recursive boundary supplement: BE-001OF-01 `backend.ops_governance` parent residual judgment selects chaos; next step: BE-001OG-01 backend.ops_governance.chaos baseline_plan.
- `markdown/06-milestones/v4.16.0/869-backend.ops_governance.parent_residual_judgment.chaos.md` - v4.16.0 BE-001OF-01 backend.ops_governance parent residual judgment selects chaos
Recursive boundary supplement: BE-001OG-01 `backend.ops_governance.chaos` equivalence baseline and extraction plan; next step: BE-001OG-02 backend.ops_governance.chaos extract_closeout.
- `markdown/06-milestones/v4.16.0/870-backend.ops_governance.chaos.baseline_plan.md` - v4.16.0 BE-001OG-01 backend.ops_governance.chaos equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OG-02 `backend.ops_governance.chaos` actual extraction complete; next step: BE-001OG-03 backend.ops_governance.chaos single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers.rs` - backend.ops_governance.chaos handler implementation
- `markdown/06-milestones/v4.16.0/871-backend.ops_governance.chaos.extract_closeout.md` - v4.16.0 BE-001OG-02 backend.ops_governance.chaos actual extraction complete
Recursive boundary supplement: BE-001OG-03 `backend.ops_governance.chaos` single leaf closeout continues split; next step: BE-001OH-01 backend.ops_governance.chaos parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/872-backend.ops_governance.chaos.single_leaf_closeout.md` - v4.16.0 BE-001OG-03 backend.ops_governance.chaos single leaf closeout continues split
Recursive boundary supplement: BE-001OH-01 `backend.ops_governance.chaos` parent residual judgment selects report_persistence; next step: BE-001OI-01 backend.ops_governance.chaos.report_persistence baseline_plan.
- `markdown/06-milestones/v4.16.0/873-backend.ops_governance.chaos.parent_residual_judgment.report_persistence.md` - v4.16.0 BE-001OH-01 backend.ops_governance.chaos parent residual judgment selects report_persistence
Recursive boundary supplement: BE-001OI-01 `backend.ops_governance.chaos.report_persistence` equivalence baseline and extraction plan; next step: BE-001OI-02 backend.ops_governance.chaos.report_persistence extract_closeout.
- `markdown/06-milestones/v4.16.0/874-backend.ops_governance.chaos.report_persistence.baseline_plan.md` - v4.16.0 BE-001OI-01 backend.ops_governance.chaos.report_persistence equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OI-02 `backend.ops_governance.chaos.report_persistence` actual extraction complete; next step: BE-001OI-03 backend.ops_governance.chaos.report_persistence single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/report_persistence.rs` - backend.ops_governance.chaos.report_persistence implementation
- `markdown/06-milestones/v4.16.0/875-backend.ops_governance.chaos.report_persistence.extract_closeout.md` - v4.16.0 BE-001OI-02 backend.ops_governance.chaos.report_persistence actual extraction complete
Recursive boundary supplement: BE-001OI-03 `backend.ops_governance.chaos.report_persistence` single leaf closeout; next step: BE-001OJ-01 backend.ops_governance.chaos parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/876-backend.ops_governance.chaos.report_persistence.single_leaf_closeout.md` - v4.16.0 BE-001OI-03 backend.ops_governance.chaos.report_persistence single leaf closeout
Recursive boundary supplement: BE-001OJ-01 `backend.ops_governance.chaos` parent residual judgment selects experiment_creation; next step: BE-001OK-01 backend.ops_governance.chaos.experiment_creation baseline_plan.
- `markdown/06-milestones/v4.16.0/877-backend.ops_governance.chaos.parent_residual_judgment.experiment_creation.md` - v4.16.0 BE-001OJ-01 backend.ops_governance.chaos parent residual judgment selects experiment_creation
Recursive boundary supplement: BE-001OK-01 `backend.ops_governance.chaos.experiment_creation` equivalence baseline and extraction plan; next step: BE-001OK-02 backend.ops_governance.chaos.experiment_creation extract_closeout.
- `markdown/06-milestones/v4.16.0/878-backend.ops_governance.chaos.experiment_creation.baseline_plan.md` - v4.16.0 BE-001OK-01 backend.ops_governance.chaos.experiment_creation equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OK-02 `backend.ops_governance.chaos.experiment_creation` actual extraction complete; next step: BE-001OK-03 backend.ops_governance.chaos.experiment_creation single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` - backend.ops_governance.chaos.experiment_creation implementation
- `markdown/06-milestones/v4.16.0/879-backend.ops_governance.chaos.experiment_creation.extract_closeout.md` - v4.16.0 BE-001OK-02 backend.ops_governance.chaos.experiment_creation actual extraction complete
Recursive boundary supplement: BE-001OK-03 `backend.ops_governance.chaos.experiment_creation` single leaf closeout continues split; next step: BE-001OL-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/880-backend.ops_governance.chaos.experiment_creation.single_leaf_closeout.md` - v4.16.0 BE-001OK-03 backend.ops_governance.chaos.experiment_creation single leaf closeout continues split
Recursive boundary supplement: BE-001OL-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects perturbation_execution; next step: BE-001OM-01 backend.ops_governance.chaos.experiment_creation.perturbation_execution baseline_plan.
- `markdown/06-milestones/v4.16.0/881-backend.ops_governance.chaos.experiment_creation.parent_residual_judgment.perturbation_execution.md` - v4.16.0 BE-001OL-01 backend.ops_governance.chaos.experiment_creation parent residual judgment selects perturbation_execution
Recursive boundary supplement: BE-001OM-01 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` equivalence baseline and extraction plan; next step: BE-001OM-02 backend.ops_governance.chaos.experiment_creation.perturbation_execution extract_closeout.
- `markdown/06-milestones/v4.16.0/882-backend.ops_governance.chaos.experiment_creation.perturbation_execution.baseline_plan.md` - v4.16.0 BE-001OM-01 backend.ops_governance.chaos.experiment_creation.perturbation_execution equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OM-02 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` actual extraction complete; next step: BE-001OM-03 backend.ops_governance.chaos.experiment_creation.perturbation_execution single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/experiment_creation/perturbation_execution.rs` - backend.ops_governance.chaos.experiment_creation.perturbation_execution implementation
- `markdown/06-milestones/v4.16.0/883-backend.ops_governance.chaos.experiment_creation.perturbation_execution.extract_closeout.md` - v4.16.0 BE-001OM-02 backend.ops_governance.chaos.experiment_creation.perturbation_execution actual extraction complete
Recursive boundary supplement: BE-001OM-03 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` single leaf closeout; next step: BE-001ON-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/884-backend.ops_governance.chaos.experiment_creation.perturbation_execution.single_leaf_closeout.md` - v4.16.0 BE-001OM-03 backend.ops_governance.chaos.experiment_creation.perturbation_execution single leaf closeout
Recursive boundary supplement: BE-001ON-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects report_projection; next step: BE-001OO-01 backend.ops_governance.chaos.experiment_creation.report_projection baseline_plan.
- `markdown/06-milestones/v4.16.0/885-backend.ops_governance.chaos.experiment_creation.parent_residual_judgment.report_projection.md` - v4.16.0 BE-001ON-01 backend.ops_governance.chaos.experiment_creation parent residual judgment selects report_projection
Recursive boundary supplement: BE-001OO-01 `backend.ops_governance.chaos.experiment_creation.report_projection` equivalence baseline and extraction plan; next step: BE-001OO-02 backend.ops_governance.chaos.experiment_creation.report_projection extract_closeout.
- `markdown/06-milestones/v4.16.0/886-backend.ops_governance.chaos.experiment_creation.report_projection.baseline_plan.md` - v4.16.0 BE-001OO-01 backend.ops_governance.chaos.experiment_creation.report_projection equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OO-02 `backend.ops_governance.chaos.experiment_creation.report_projection` actual extraction complete; next step: BE-001OO-03 backend.ops_governance.chaos.experiment_creation.report_projection single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/experiment_creation/report_projection.rs` - backend.ops_governance.chaos.experiment_creation.report_projection implementation
- `markdown/06-milestones/v4.16.0/887-backend.ops_governance.chaos.experiment_creation.report_projection.extract_closeout.md` - v4.16.0 BE-001OO-02 backend.ops_governance.chaos.experiment_creation.report_projection actual extraction complete
Recursive boundary supplement: BE-001OO-03 `backend.ops_governance.chaos.experiment_creation.report_projection` single leaf closeout; next step: BE-001OP-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/888-backend.ops_governance.chaos.experiment_creation.report_projection.single_leaf_closeout.md` - v4.16.0 BE-001OO-03 backend.ops_governance.chaos.experiment_creation.report_projection single leaf closeout
Recursive boundary supplement: BE-001OP-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects memory_commit; next step: BE-001OQ-01 backend.ops_governance.chaos.experiment_creation.memory_commit baseline_plan.
- `markdown/06-milestones/v4.16.0/889-backend.ops_governance.chaos.experiment_creation.parent_residual_judgment.memory_commit.md` - v4.16.0 BE-001OP-01 backend.ops_governance.chaos.experiment_creation parent residual judgment selects memory_commit
Recursive boundary supplement: BE-001OQ-01 `backend.ops_governance.chaos.experiment_creation.memory_commit` equivalence baseline and extraction plan; next step: BE-001OQ-02 backend.ops_governance.chaos.experiment_creation.memory_commit extract_closeout.
- `markdown/06-milestones/v4.16.0/890-backend.ops_governance.chaos.experiment_creation.memory_commit.baseline_plan.md` - v4.16.0 BE-001OQ-01 backend.ops_governance.chaos.experiment_creation.memory_commit equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OQ-02 `backend.ops_governance.chaos.experiment_creation.memory_commit` actual extraction complete; next step: BE-001OQ-03 backend.ops_governance.chaos.experiment_creation.memory_commit single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/experiment_creation/memory_commit.rs` - backend.ops_governance.chaos.experiment_creation.memory_commit implementation
- `markdown/06-milestones/v4.16.0/891-backend.ops_governance.chaos.experiment_creation.memory_commit.extract_closeout.md` - v4.16.0 BE-001OQ-02 backend.ops_governance.chaos.experiment_creation.memory_commit actual extraction complete
Recursive boundary supplement: BE-001OQ-03 `backend.ops_governance.chaos.experiment_creation.memory_commit` single leaf closeout; next step: BE-001OR-01 backend.ops_governance.chaos.experiment_creation parent_closeout.
- `markdown/06-milestones/v4.16.0/892-backend.ops_governance.chaos.experiment_creation.memory_commit.single_leaf_closeout.md` - v4.16.0 BE-001OQ-03 backend.ops_governance.chaos.experiment_creation.memory_commit single leaf closeout
Recursive boundary supplement: BE-001OR-01 `backend.ops_governance.chaos.experiment_creation` parent closeout; next step: BE-001OS-01 backend.ops_governance.chaos parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/893-backend.ops_governance.chaos.experiment_creation.parent_closeout.md` - v4.16.0 BE-001OR-01 backend.ops_governance.chaos.experiment_creation parent closeout
Recursive boundary supplement: BE-001OS-01 `backend.ops_governance.chaos` parent residual judgment selects read_routes; next step: BE-001OT-01 backend.ops_governance.chaos.read_routes baseline_plan.
- `markdown/06-milestones/v4.16.0/894-backend.ops_governance.chaos.parent_residual_judgment.read_routes.md` - v4.16.0 BE-001OS-01 backend.ops_governance.chaos parent residual judgment selects read_routes
Recursive boundary supplement: BE-001OT-01 `backend.ops_governance.chaos.read_routes` equivalence baseline and extraction plan; next step: BE-001OT-02 backend.ops_governance.chaos.read_routes extract_closeout.
- `markdown/06-milestones/v4.16.0/895-backend.ops_governance.chaos.read_routes.baseline_plan.md` - v4.16.0 BE-001OT-01 backend.ops_governance.chaos.read_routes equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OT-02 `backend.ops_governance.chaos.read_routes` actual extraction complete; next step: BE-001OT-03 backend.ops_governance.chaos.read_routes single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/read_routes.rs` - backend.ops_governance.chaos.read_routes implementation
- `markdown/06-milestones/v4.16.0/896-backend.ops_governance.chaos.read_routes.extract_closeout.md` - v4.16.0 BE-001OT-02 backend.ops_governance.chaos.read_routes actual extraction complete
Recursive boundary supplement: BE-001OT-03 `backend.ops_governance.chaos.read_routes` single leaf closeout; next step: BE-001OU-01 backend.ops_governance.chaos parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/897-backend.ops_governance.chaos.read_routes.single_leaf_closeout.md` - v4.16.0 BE-001OT-03 backend.ops_governance.chaos.read_routes single leaf closeout
Recursive boundary supplement: BE-001OU-01 `backend.ops_governance.chaos` parent residual judgment selects route_facade; next step: BE-001OV-01 backend.ops_governance.chaos.route_facade baseline_plan.
- `markdown/06-milestones/v4.16.0/898-backend.ops_governance.chaos.parent_residual_judgment.route_facade.md` - v4.16.0 BE-001OU-01 backend.ops_governance.chaos parent residual judgment selects route_facade
Recursive boundary supplement: BE-001OV-01 `backend.ops_governance.chaos.route_facade` equivalence baseline and extraction plan; next step: BE-001OV-02 backend.ops_governance.chaos.route_facade extract_closeout.
- `markdown/06-milestones/v4.16.0/899-backend.ops_governance.chaos.route_facade.baseline_plan.md` - v4.16.0 BE-001OV-01 backend.ops_governance.chaos.route_facade equivalence baseline and extraction plan
Recursive boundary supplement: BE-001OV-02 `backend.ops_governance.chaos.route_facade` actual extraction complete; next step: BE-001OV-03 backend.ops_governance.chaos.route_facade single_leaf_closeout.
- `src/backend/ops_governance/chaos/handlers/route_facade.rs` - backend.ops_governance.chaos.route_facade implementation
- `markdown/06-milestones/v4.16.0/900-backend.ops_governance.chaos.route_facade.extract_closeout.md` - v4.16.0 BE-001OV-02 backend.ops_governance.chaos.route_facade actual extraction complete
Recursive boundary supplement: BE-001OV-03 `backend.ops_governance.chaos.route_facade` single leaf closeout; next step: BE-001OW-01 backend.ops_governance.chaos parent_closeout.
- `markdown/06-milestones/v4.16.0/901-backend.ops_governance.chaos.route_facade.single_leaf_closeout.md` - v4.16.0 BE-001OV-03 backend.ops_governance.chaos.route_facade single leaf closeout
Recursive boundary supplement: BE-001OW-01 `backend.ops_governance.chaos` parent closeout; next step: BE-001OX-01 backend.ops_governance parent_closeout.
- `markdown/06-milestones/v4.16.0/902-backend.ops_governance.chaos.parent_closeout.md` - v4.16.0 BE-001OW-01 backend.ops_governance.chaos parent closeout
Recursive boundary supplement: BE-001OX-01 `backend.ops_governance` parent closeout; next step: BE-001OY-01 backend parent_residual_judgment selects backend.app_state_wiring.
- `markdown/06-milestones/v4.16.0/903-backend.ops_governance.parent_closeout.md` - v4.16.0 BE-001OX-01 backend.ops_governance parent closeout
Recursive boundary supplement: BE-001OY-01 `backend` parent residual judgment selects backend.app_state_wiring; next step: BE-001OZ-01 backend.app_state_wiring single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/904-backend.parent_residual_judgment.app_state_wiring.md` - v4.16.0 BE-001OY-01 backend parent residual judgment selects backend.app_state_wiring
Recursive boundary supplement: BE-001OZ-01 `backend.app_state_wiring` single leaf closeout; next step: BE-001PA-01 backend parent_residual_judgment selects backend.test_support.
- `markdown/06-milestones/v4.16.0/905-backend.app_state_wiring.single_leaf_closeout.md` - v4.16.0 BE-001OZ-01 backend.app_state_wiring single leaf closeout
Recursive boundary supplement: BE-001PA-01 `backend` parent residual judgment selects backend.test_support; next step: BE-001PB-01 backend.test_support single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/906-backend.parent_residual_judgment.test_support.md` - v4.16.0 BE-001PA-01 backend parent residual judgment selects backend.test_support
Recursive boundary supplement: BE-001PB-01 `backend.test_support` single leaf closeout; next step: BE-001PC-01 backend parent_closeout.
- `markdown/06-milestones/v4.16.0/907-backend.test_support.single_leaf_closeout.md` - v4.16.0 BE-001PB-01 backend.test_support single leaf closeout
Recursive boundary supplement: BE-001PC-01 `backend` parent closeout; next step: BE-001PD-01 root parent_residual_judgment selects root.contracts.
- `markdown/06-milestones/v4.16.0/908-backend.parent_closeout.md` - v4.16.0 BE-001PC-01 backend parent closeout
Recursive boundary supplement: BE-001PD-01 `root` parent residual judgment selects root.contracts; next step: BE-001PE-01 root.contracts baseline_plan.
- `markdown/06-milestones/v4.16.0/909-root.parent_residual_judgment.contracts.md` - v4.16.0 BE-001PD-01 root parent residual judgment selects root.contracts
Recursive boundary supplement: BE-001PE-01 `root.contracts` baseline frozen; next step: BE-001PF-01 root.contracts parent_residual_judgment selects contracts.api_surface.
- `markdown/06-milestones/v4.16.0/910-root.contracts.baseline_plan.md` - v4.16.0 BE-001PE-01 root.contracts baseline plan
Recursive boundary supplement: BE-001PF-01 `root.contracts` parent residual judgment selects contracts.api_surface; next step: BE-001PG-01 root.contracts.api_surface single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/911-root.contracts.parent_residual_judgment.api_surface.md` - v4.16.0 BE-001PF-01 root.contracts parent residual judgment selects contracts.api_surface
Recursive boundary supplement: BE-001PG-01 `root.contracts.api_surface` single leaf closeout sets stop_split false; next step: BE-001PH-01 root.contracts.api_surface parent_residual_judgment selects contracts.api_surface.openapi_http.
- `markdown/06-milestones/v4.16.0/912-root.contracts.api_surface.single_leaf_closeout.md` - v4.16.0 BE-001PG-01 root.contracts.api_surface single leaf closeout
Recursive boundary supplement: BE-001PH-01 `root.contracts.api_surface` parent residual judgment selects contracts.api_surface.openapi_http; next step: BE-001PI-01 root.contracts.api_surface.openapi_http single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/913-root.contracts.api_surface.parent_residual_judgment.openapi_http.md` - v4.16.0 BE-001PH-01 root.contracts.api_surface parent residual judgment selects openapi_http
Recursive boundary supplement: BE-001PI-01 `root.contracts.api_surface.openapi_http` single leaf closeout sets stop_split true; next step: BE-001PJ-01 root.contracts.api_surface parent_residual_judgment selects contracts.api_surface.asyncapi_runtime_events.
- `markdown/06-milestones/v4.16.0/914-root.contracts.api_surface.openapi_http.single_leaf_closeout.md` - v4.16.0 BE-001PI-01 root.contracts.api_surface.openapi_http single leaf closeout
Recursive boundary supplement: BE-001PJ-01 `root.contracts.api_surface` parent residual judgment selects contracts.api_surface.asyncapi_runtime_events; next step: BE-001PK-01 root.contracts.api_surface.asyncapi_runtime_events single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/915-root.contracts.api_surface.parent_residual_judgment.asyncapi_runtime_events.md` - v4.16.0 BE-001PJ-01 root.contracts.api_surface parent residual judgment selects asyncapi_runtime_events
Recursive boundary supplement: BE-001PK-01 `root.contracts.api_surface.asyncapi_runtime_events` single leaf closeout sets stop_split true; next step: BE-001PL-01 root.contracts.api_surface parent_closeout.
- `markdown/06-milestones/v4.16.0/916-root.contracts.api_surface.asyncapi_runtime_events.single_leaf_closeout.md` - v4.16.0 BE-001PK-01 root.contracts.api_surface.asyncapi_runtime_events single leaf closeout
Recursive boundary supplement: BE-001PL-01 `root.contracts.api_surface` parent closeout; next step: BE-001PM-01 root.contracts parent_residual_judgment selects contracts.qrpc_core.
- `markdown/06-milestones/v4.16.0/917-root.contracts.api_surface.parent_closeout.md` - v4.16.0 BE-001PL-01 root.contracts.api_surface parent closeout
Recursive boundary supplement: BE-001PM-01 `root.contracts` parent residual judgment selects contracts.qrpc_core; next step: BE-001PN-01 root.contracts.qrpc_core baseline_plan.
- `markdown/06-milestones/v4.16.0/918-root.contracts.parent_residual_judgment.qrpc_core.md` - v4.16.0 BE-001PM-01 root.contracts parent residual judgment selects qrpc_core
Recursive boundary supplement: BE-001PN-01 `root.contracts.qrpc_core` baseline frozen; next step: BE-001PO-01 root.contracts.qrpc_core parent_residual_judgment selects error_contract.
- `markdown/06-milestones/v4.16.0/919-root.contracts.qrpc_core.baseline_plan.md` - v4.16.0 BE-001PN-01 root.contracts.qrpc_core baseline plan
Recursive boundary supplement: BE-001PO-01 `root.contracts.qrpc_core` parent residual judgment selects error_contract; next step: BE-001PP-01 root.contracts.qrpc_core.error_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/920-root.contracts.qrpc_core.parent_residual_judgment.error_contract.md` - v4.16.0 BE-001PO-01 root.contracts.qrpc_core parent residual judgment selects error_contract
Recursive boundary supplement: BE-001PP-01 `root.contracts.qrpc_core.error_contract` single leaf closeout sets stop_split true; next step: BE-001PQ-01 root.contracts.qrpc_core parent_residual_judgment selects event_envelope_proto.
- `markdown/06-milestones/v4.16.0/921-root.contracts.qrpc_core.error_contract.single_leaf_closeout.md` - v4.16.0 BE-001PP-01 root.contracts.qrpc_core.error_contract single leaf closeout
Recursive boundary supplement: BE-001PQ-01 `root.contracts.qrpc_core` parent residual judgment selects event_envelope_proto; next step: BE-001PR-01 root.contracts.qrpc_core.event_envelope_proto single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/922-root.contracts.qrpc_core.parent_residual_judgment.event_envelope_proto.md` - v4.16.0 BE-001PQ-01 root.contracts.qrpc_core parent residual judgment selects event_envelope_proto
Recursive boundary supplement: BE-001PR-01 `root.contracts.qrpc_core.event_envelope_proto` single leaf closeout sets stop_split true; next step: BE-001PS-01 root.contracts.qrpc_core parent_residual_judgment selects plugin_contract.
- `markdown/06-milestones/v4.16.0/923-root.contracts.qrpc_core.event_envelope_proto.single_leaf_closeout.md` - v4.16.0 BE-001PR-01 root.contracts.qrpc_core.event_envelope_proto single leaf closeout
Recursive boundary supplement: BE-001PS-01 `root.contracts.qrpc_core` parent residual judgment selects plugin_contract; next step: BE-001PT-01 root.contracts.qrpc_core.plugin_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/924-root.contracts.qrpc_core.parent_residual_judgment.plugin_contract.md` - v4.16.0 BE-001PS-01 root.contracts.qrpc_core parent residual judgment selects plugin_contract
Recursive boundary supplement: BE-001PT-01 `root.contracts.qrpc_core.plugin_contract` baseline frozen; child queue is taxonomy_extension, capability_contract, execution_security_dependency, manifest_validation, and registry.
- `markdown/06-milestones/v4.16.0/925-root.contracts.qrpc_core.plugin_contract.baseline_plan.md` - v4.16.0 BE-001PT-01 root.contracts.qrpc_core.plugin_contract baseline plan
Recursive boundary supplement: BE-001PU-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects taxonomy_extension; next step: BE-001PV-01 root.contracts.qrpc_core.plugin_contract.taxonomy_extension baseline_plan.
- `markdown/06-milestones/v4.16.0/926-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.taxonomy_extension.md` - v4.16.0 BE-001PU-01 root.contracts.qrpc_core.plugin_contract parent residual judgment selects taxonomy_extension
Recursive boundary supplement: BE-001PV-01 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` equivalence baseline and extraction plan; next step: BE-001PV-02 root.contracts.qrpc_core.plugin_contract.taxonomy_extension extract_closeout.
- `markdown/06-milestones/v4.16.0/927-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.baseline_plan.md` - v4.16.0 BE-001PV-01 root.contracts.qrpc_core.plugin_contract.taxonomy_extension baseline plan
Recursive boundary supplement: BE-001PV-02 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` actual extraction complete; next step: BE-001PV-03 root.contracts.qrpc_core.plugin_contract.taxonomy_extension single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/928-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.extract_closeout.md` - v4.16.0 BE-001PV-02 root.contracts.qrpc_core.plugin_contract.taxonomy_extension actual extraction closeout
- `qrpc_core/src/plugin/taxonomy_extension.rs` - Extracted qrpc plugin taxonomy and extension-point contract child
Recursive boundary supplement: BE-001PV-03 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` single leaf closeout sets stop_split true; next step: BE-001PW-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects capability_contract.
- `markdown/06-milestones/v4.16.0/929-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.single_leaf_closeout.md` - v4.16.0 BE-001PV-03 root.contracts.qrpc_core.plugin_contract.taxonomy_extension single leaf closeout
Recursive boundary supplement: BE-001PW-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects capability_contract; next step: BE-001PX-01 root.contracts.qrpc_core.plugin_contract.capability_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/930-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.capability_contract.md` - v4.16.0 BE-001PW-01 root.contracts.qrpc_core.plugin_contract parent residual judgment selects capability_contract
Recursive boundary supplement: BE-001PX-01 `root.contracts.qrpc_core.plugin_contract.capability_contract` equivalence baseline and extraction plan; next step: BE-001PX-02 root.contracts.qrpc_core.plugin_contract.capability_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/931-root.contracts.qrpc_core.plugin_contract.capability_contract.baseline_plan.md` - v4.16.0 BE-001PX-01 root.contracts.qrpc_core.plugin_contract.capability_contract baseline plan
Recursive boundary supplement: BE-001PX-02 `root.contracts.qrpc_core.plugin_contract.capability_contract` actual extraction complete; next step: BE-001PX-03 root.contracts.qrpc_core.plugin_contract.capability_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/932-root.contracts.qrpc_core.plugin_contract.capability_contract.extract_closeout.md` - v4.16.0 BE-001PX-02 root.contracts.qrpc_core.plugin_contract.capability_contract actual extraction closeout
- `qrpc_core/src/plugin/capability_contract.rs` - Extracted qrpc plugin capability contract child
Recursive boundary supplement: BE-001PX-03 `root.contracts.qrpc_core.plugin_contract.capability_contract` single leaf closeout sets stop_split true; next step: BE-001PY-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects execution_security_dependency.
- `markdown/06-milestones/v4.16.0/933-root.contracts.qrpc_core.plugin_contract.capability_contract.single_leaf_closeout.md` - v4.16.0 BE-001PX-03 root.contracts.qrpc_core.plugin_contract.capability_contract single leaf closeout
Recursive boundary supplement: BE-001PY-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects execution_security_dependency; next step: BE-001PZ-01 root.contracts.qrpc_core.plugin_contract.execution_security_dependency baseline_plan.
- `markdown/06-milestones/v4.16.0/934-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.execution_security_dependency.md` - v4.16.0 BE-001PY-01 root.contracts.qrpc_core.plugin_contract parent residual judgment selects execution_security_dependency
Recursive boundary supplement: BE-001PZ-01 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` equivalence baseline and extraction plan; next step: BE-001PZ-02 root.contracts.qrpc_core.plugin_contract.execution_security_dependency extract_closeout.
- `markdown/06-milestones/v4.16.0/935-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.baseline_plan.md` - v4.16.0 BE-001PZ-01 root.contracts.qrpc_core.plugin_contract.execution_security_dependency baseline plan
Recursive boundary supplement: BE-001PZ-02 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` actual extraction complete; next step: BE-001PZ-03 root.contracts.qrpc_core.plugin_contract.execution_security_dependency single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/936-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.extract_closeout.md` - v4.16.0 BE-001PZ-02 root.contracts.qrpc_core.plugin_contract.execution_security_dependency actual extraction closeout
- `qrpc_core/src/plugin/execution_security_dependency.rs` - Extracted qrpc plugin execution, security, compatibility, and dependency DTO child
Recursive boundary supplement: BE-001PZ-03 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` single leaf closeout sets stop_split true; next step: BE-001QA-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects manifest_validation.
- `markdown/06-milestones/v4.16.0/937-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.single_leaf_closeout.md` - v4.16.0 BE-001PZ-03 root.contracts.qrpc_core.plugin_contract.execution_security_dependency single leaf closeout
Recursive boundary supplement: BE-001QA-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects manifest_validation; next step: BE-001QB-01 root.contracts.qrpc_core.plugin_contract.manifest_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/938-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.manifest_validation.md` - v4.16.0 BE-001QA-01 root.contracts.qrpc_core.plugin_contract parent residual judgment selects manifest_validation
Recursive boundary supplement: BE-001QB-01 `root.contracts.qrpc_core.plugin_contract.manifest_validation` equivalence baseline and extraction plan; next step: BE-001QB-02 root.contracts.qrpc_core.plugin_contract.manifest_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/939-root.contracts.qrpc_core.plugin_contract.manifest_validation.baseline_plan.md` - v4.16.0 BE-001QB-01 root.contracts.qrpc_core.plugin_contract.manifest_validation baseline plan
Recursive boundary supplement: BE-001QB-02 `root.contracts.qrpc_core.plugin_contract.manifest_validation` actual extraction complete; next step: BE-001QB-03 root.contracts.qrpc_core.plugin_contract.manifest_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/940-root.contracts.qrpc_core.plugin_contract.manifest_validation.extract_closeout.md` - v4.16.0 BE-001QB-02 root.contracts.qrpc_core.plugin_contract.manifest_validation actual extraction closeout
- `qrpc_core/src/plugin/manifest_validation.rs` - Extracted qrpc plugin manifest schema and validation child
Recursive boundary supplement: BE-001QB-03 `root.contracts.qrpc_core.plugin_contract.manifest_validation` single leaf closeout sets stop_split true; next step: BE-001QC-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects registry.
- `markdown/06-milestones/v4.16.0/941-root.contracts.qrpc_core.plugin_contract.manifest_validation.single_leaf_closeout.md` - v4.16.0 BE-001QB-03 root.contracts.qrpc_core.plugin_contract.manifest_validation single leaf closeout
Recursive boundary supplement: BE-001QC-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects registry; next step: BE-001QD-01 root.contracts.qrpc_core.plugin_contract.registry baseline_plan.
- `markdown/06-milestones/v4.16.0/942-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.registry.md` - v4.16.0 BE-001QC-01 root.contracts.qrpc_core.plugin_contract parent residual judgment selects registry
Recursive boundary supplement: BE-001QD-01 `root.contracts.qrpc_core.plugin_contract.registry` equivalence baseline and extraction plan; next step: BE-001QD-02 root.contracts.qrpc_core.plugin_contract.registry extract_closeout.
- `markdown/06-milestones/v4.16.0/943-root.contracts.qrpc_core.plugin_contract.registry.baseline_plan.md` - v4.16.0 BE-001QD-01 root.contracts.qrpc_core.plugin_contract.registry baseline plan
Recursive boundary supplement: BE-001QD-02 `root.contracts.qrpc_core.plugin_contract.registry` actual extraction complete; next step: BE-001QD-03 root.contracts.qrpc_core.plugin_contract.registry single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/944-root.contracts.qrpc_core.plugin_contract.registry.extract_closeout.md` - v4.16.0 BE-001QD-02 root.contracts.qrpc_core.plugin_contract.registry actual extraction closeout
- `qrpc_core/src/plugin/registry.rs` - Extracted qrpc plugin in-memory registry child
Recursive boundary supplement: BE-001QD-03 `root.contracts.qrpc_core.plugin_contract.registry` single leaf closeout sets stop_split true; next step: BE-001QE-01 root.contracts.qrpc_core.plugin_contract parent_closeout.
- `markdown/06-milestones/v4.16.0/945-root.contracts.qrpc_core.plugin_contract.registry.single_leaf_closeout.md` - v4.16.0 BE-001QD-03 root.contracts.qrpc_core.plugin_contract.registry single leaf closeout
Recursive boundary supplement: BE-001QE-01 `root.contracts.qrpc_core.plugin_contract` parent closeout; next step: BE-001QF-01 root.contracts.qrpc_core parent_residual_judgment selects strategy_ir.
- `markdown/06-milestones/v4.16.0/946-root.contracts.qrpc_core.plugin_contract.parent_closeout.md` - v4.16.0 BE-001QE-01 root.contracts.qrpc_core.plugin_contract parent closeout
Recursive boundary supplement: BE-001QF-01 `root.contracts.qrpc_core` parent residual judgment selects strategy_ir; next step: BE-001QG-01 root.contracts.qrpc_core.strategy_ir baseline_plan.
- `markdown/06-milestones/v4.16.0/947-root.contracts.qrpc_core.parent_residual_judgment.strategy_ir.md` - v4.16.0 BE-001QF-01 root.contracts.qrpc_core parent residual judgment selects strategy_ir
Recursive boundary supplement: BE-001QG-01 `root.contracts.qrpc_core.strategy_ir` baseline frozen; child queue is version_unknown_error, metadata_source, signal_indicator, logic_position, risk_contract, data_requirement, execution_contract, gap_unknown_annotation, and root_validation.
- `markdown/06-milestones/v4.16.0/948-root.contracts.qrpc_core.strategy_ir.baseline_plan.md` - v4.16.0 BE-001QG-01 root.contracts.qrpc_core.strategy_ir baseline plan
Recursive boundary supplement: BE-001QH-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects version_unknown_error; next step: BE-001QI-01 root.contracts.qrpc_core.strategy_ir.version_unknown_error baseline_plan.
- `markdown/06-milestones/v4.16.0/949-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.version_unknown_error.md` - v4.16.0 BE-001QH-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects version_unknown_error
Recursive boundary supplement: BE-001QI-01 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` equivalence baseline and extraction plan; next step: BE-001QI-02 root.contracts.qrpc_core.strategy_ir.version_unknown_error extract_closeout.
- `markdown/06-milestones/v4.16.0/950-root.contracts.qrpc_core.strategy_ir.version_unknown_error.baseline_plan.md` - v4.16.0 BE-001QI-01 root.contracts.qrpc_core.strategy_ir.version_unknown_error baseline plan
Recursive boundary supplement: BE-001QI-02 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` actual extraction complete; next step: BE-001QI-03 root.contracts.qrpc_core.strategy_ir.version_unknown_error single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/951-root.contracts.qrpc_core.strategy_ir.version_unknown_error.extract_closeout.md` - v4.16.0 BE-001QI-02 root.contracts.qrpc_core.strategy_ir.version_unknown_error actual extraction closeout
- `qrpc_core/src/strategy_ir/version_unknown_error.rs` - Extracted Strategy IR version, unknown wrapper, and validation error diagnostic child
Recursive boundary supplement: BE-001QI-03 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` single leaf closeout sets stop_split true; next step: BE-001QJ-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects metadata_source.
- `markdown/06-milestones/v4.16.0/952-root.contracts.qrpc_core.strategy_ir.version_unknown_error.single_leaf_closeout.md` - v4.16.0 BE-001QI-03 root.contracts.qrpc_core.strategy_ir.version_unknown_error single leaf closeout
Recursive boundary supplement: BE-001QJ-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects metadata_source; next step: BE-001QK-01 root.contracts.qrpc_core.strategy_ir.metadata_source baseline_plan.
- `markdown/06-milestones/v4.16.0/953-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.metadata_source.md` - v4.16.0 BE-001QJ-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects metadata_source
Recursive boundary supplement: BE-001QK-01 `root.contracts.qrpc_core.strategy_ir.metadata_source` equivalence baseline and extraction plan; next step: BE-001QK-02 root.contracts.qrpc_core.strategy_ir.metadata_source extract_closeout.
- `markdown/06-milestones/v4.16.0/954-root.contracts.qrpc_core.strategy_ir.metadata_source.baseline_plan.md` - v4.16.0 BE-001QK-01 root.contracts.qrpc_core.strategy_ir.metadata_source baseline plan
Recursive boundary supplement: BE-001QK-02 `root.contracts.qrpc_core.strategy_ir.metadata_source` actual extraction complete; next step: BE-001QK-03 root.contracts.qrpc_core.strategy_ir.metadata_source single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/955-root.contracts.qrpc_core.strategy_ir.metadata_source.extract_closeout.md` - v4.16.0 BE-001QK-02 root.contracts.qrpc_core.strategy_ir.metadata_source actual extraction closeout
- `qrpc_core/src/strategy_ir/metadata_source.rs` - Extracted Strategy IR metadata and source DTO child
Recursive boundary supplement: BE-001QK-03 `root.contracts.qrpc_core.strategy_ir.metadata_source` single leaf closeout sets stop_split true; next step: BE-001QL-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects signal_indicator.
- `markdown/06-milestones/v4.16.0/956-root.contracts.qrpc_core.strategy_ir.metadata_source.single_leaf_closeout.md` - v4.16.0 BE-001QK-03 root.contracts.qrpc_core.strategy_ir.metadata_source single leaf closeout
Recursive boundary supplement: BE-001QL-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects signal_indicator; next step: BE-001QM-01 root.contracts.qrpc_core.strategy_ir.signal_indicator baseline_plan.
- `markdown/06-milestones/v4.16.0/957-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.signal_indicator.md` - v4.16.0 BE-001QL-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects signal_indicator
Recursive boundary supplement: BE-001QM-01 `root.contracts.qrpc_core.strategy_ir.signal_indicator` equivalence baseline and extraction plan; next step: BE-001QM-02 root.contracts.qrpc_core.strategy_ir.signal_indicator extract_closeout.
- `markdown/06-milestones/v4.16.0/958-root.contracts.qrpc_core.strategy_ir.signal_indicator.baseline_plan.md` - v4.16.0 BE-001QM-01 root.contracts.qrpc_core.strategy_ir.signal_indicator baseline plan
Recursive boundary supplement: BE-001QM-02 `root.contracts.qrpc_core.strategy_ir.signal_indicator` actual extraction complete; next step: BE-001QM-03 root.contracts.qrpc_core.strategy_ir.signal_indicator single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/959-root.contracts.qrpc_core.strategy_ir.signal_indicator.extract_closeout.md` - v4.16.0 BE-001QM-02 root.contracts.qrpc_core.strategy_ir.signal_indicator actual extraction closeout
- `qrpc_core/src/strategy_ir/signal_indicator.rs` - Extracted Strategy IR signal/indicator DTO and registry child
Recursive boundary supplement: BE-001QM-03 `root.contracts.qrpc_core.strategy_ir.signal_indicator` single leaf closeout sets stop_split true; next step: BE-001QN-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects logic_position.
- `markdown/06-milestones/v4.16.0/960-root.contracts.qrpc_core.strategy_ir.signal_indicator.single_leaf_closeout.md` - v4.16.0 BE-001QM-03 root.contracts.qrpc_core.strategy_ir.signal_indicator single leaf closeout
Recursive boundary supplement: BE-001QN-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects logic_position; next step: BE-001QO-01 root.contracts.qrpc_core.strategy_ir.logic_position baseline_plan.
- `markdown/06-milestones/v4.16.0/961-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.logic_position.md` - v4.16.0 BE-001QN-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects logic_position
Recursive boundary supplement: BE-001QO-01 `root.contracts.qrpc_core.strategy_ir.logic_position` equivalence baseline and extraction plan; next step: BE-001QO-02 root.contracts.qrpc_core.strategy_ir.logic_position extract_closeout.
- `markdown/06-milestones/v4.16.0/962-root.contracts.qrpc_core.strategy_ir.logic_position.baseline_plan.md` - v4.16.0 BE-001QO-01 root.contracts.qrpc_core.strategy_ir.logic_position baseline plan
Recursive boundary supplement: BE-001QO-02 `root.contracts.qrpc_core.strategy_ir.logic_position` actual extraction complete; next step: BE-001QO-03 root.contracts.qrpc_core.strategy_ir.logic_position single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/963-root.contracts.qrpc_core.strategy_ir.logic_position.extract_closeout.md` - v4.16.0 BE-001QO-02 root.contracts.qrpc_core.strategy_ir.logic_position actual extraction closeout
- `qrpc_core/src/strategy_ir/logic_position.rs` - Extracted Strategy IR logic/action/position sizing/rebalance DTO child
Recursive boundary supplement: BE-001QO-03 `root.contracts.qrpc_core.strategy_ir.logic_position` single leaf closeout sets stop_split true; next step: BE-001QP-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects risk_contract.
- `markdown/06-milestones/v4.16.0/964-root.contracts.qrpc_core.strategy_ir.logic_position.single_leaf_closeout.md` - v4.16.0 BE-001QO-03 root.contracts.qrpc_core.strategy_ir.logic_position single leaf closeout
Recursive boundary supplement: BE-001QP-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects risk_contract; next step: BE-001QQ-01 root.contracts.qrpc_core.strategy_ir.risk_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/965-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.risk_contract.md` - v4.16.0 BE-001QP-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects risk_contract
Recursive boundary supplement: BE-001QQ-01 `root.contracts.qrpc_core.strategy_ir.risk_contract` equivalence baseline and extraction plan; next step: BE-001QQ-02 root.contracts.qrpc_core.strategy_ir.risk_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/966-root.contracts.qrpc_core.strategy_ir.risk_contract.baseline_plan.md` - v4.16.0 BE-001QQ-01 root.contracts.qrpc_core.strategy_ir.risk_contract baseline plan
Recursive boundary supplement: BE-001QQ-02 `root.contracts.qrpc_core.strategy_ir.risk_contract` actual extraction complete; next step: BE-001QQ-03 root.contracts.qrpc_core.strategy_ir.risk_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/967-root.contracts.qrpc_core.strategy_ir.risk_contract.extract_closeout.md` - v4.16.0 BE-001QQ-02 root.contracts.qrpc_core.strategy_ir.risk_contract actual extraction closeout
- `qrpc_core/src/strategy_ir/risk_contract.rs` - Extracted Strategy IR risk rule/profile DTO child
Recursive boundary supplement: BE-001QQ-03 `root.contracts.qrpc_core.strategy_ir.risk_contract` single leaf closeout sets stop_split true; next step: BE-001QR-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects data_requirement.
- `markdown/06-milestones/v4.16.0/968-root.contracts.qrpc_core.strategy_ir.risk_contract.single_leaf_closeout.md` - v4.16.0 BE-001QQ-03 root.contracts.qrpc_core.strategy_ir.risk_contract single leaf closeout
Recursive boundary supplement: BE-001QR-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects data_requirement; next step: BE-001QS-01 root.contracts.qrpc_core.strategy_ir.data_requirement baseline_plan.
- `markdown/06-milestones/v4.16.0/969-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.data_requirement.md` - v4.16.0 BE-001QR-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects data_requirement
Recursive boundary supplement: BE-001QS-01 `root.contracts.qrpc_core.strategy_ir.data_requirement` equivalence baseline and extraction plan; next step: BE-001QS-02 root.contracts.qrpc_core.strategy_ir.data_requirement extract_closeout.
- `markdown/06-milestones/v4.16.0/970-root.contracts.qrpc_core.strategy_ir.data_requirement.baseline_plan.md` - v4.16.0 BE-001QS-01 root.contracts.qrpc_core.strategy_ir.data_requirement baseline plan
Recursive boundary supplement: BE-001QS-02 `root.contracts.qrpc_core.strategy_ir.data_requirement` actual extraction complete; next step: BE-001QS-03 root.contracts.qrpc_core.strategy_ir.data_requirement single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/971-root.contracts.qrpc_core.strategy_ir.data_requirement.extract_closeout.md` - v4.16.0 BE-001QS-02 root.contracts.qrpc_core.strategy_ir.data_requirement actual extraction closeout
- `qrpc_core/src/strategy_ir/data_requirement.rs` - Extracted Strategy IR data requirement DTO child
Recursive boundary supplement: BE-001QS-03 `root.contracts.qrpc_core.strategy_ir.data_requirement` single leaf closeout sets stop_split true; next step: BE-001QT-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects execution_contract.
- `markdown/06-milestones/v4.16.0/972-root.contracts.qrpc_core.strategy_ir.data_requirement.single_leaf_closeout.md` - v4.16.0 BE-001QS-03 root.contracts.qrpc_core.strategy_ir.data_requirement single leaf closeout
Recursive boundary supplement: BE-001QT-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects execution_contract; next step: BE-001QU-01 root.contracts.qrpc_core.strategy_ir.execution_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/973-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.execution_contract.md` - v4.16.0 BE-001QT-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects execution_contract
Recursive boundary supplement: BE-001QU-01 `root.contracts.qrpc_core.strategy_ir.execution_contract` equivalence baseline and extraction plan; next step: BE-001QU-02 root.contracts.qrpc_core.strategy_ir.execution_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/974-root.contracts.qrpc_core.strategy_ir.execution_contract.baseline_plan.md` - v4.16.0 BE-001QU-01 root.contracts.qrpc_core.strategy_ir.execution_contract baseline plan
Recursive boundary supplement: BE-001QU-02 `root.contracts.qrpc_core.strategy_ir.execution_contract` actual extraction complete; next step: BE-001QU-03 root.contracts.qrpc_core.strategy_ir.execution_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/975-root.contracts.qrpc_core.strategy_ir.execution_contract.extract_closeout.md` - v4.16.0 BE-001QU-02 root.contracts.qrpc_core.strategy_ir.execution_contract actual extraction closeout
- `qrpc_core/src/strategy_ir/execution_contract.rs` - Extracted Strategy IR execution DTO child
Recursive boundary supplement: BE-001QU-03 `root.contracts.qrpc_core.strategy_ir.execution_contract` single leaf closeout sets stop_split true; next step: BE-001QV-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects gap_unknown_annotation.
- `markdown/06-milestones/v4.16.0/976-root.contracts.qrpc_core.strategy_ir.execution_contract.single_leaf_closeout.md` - v4.16.0 BE-001QU-03 root.contracts.qrpc_core.strategy_ir.execution_contract single leaf closeout
Recursive boundary supplement: BE-001QV-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects gap_unknown_annotation; next step: BE-001QW-01 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation baseline_plan.
- `markdown/06-milestones/v4.16.0/977-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.gap_unknown_annotation.md` - v4.16.0 BE-001QV-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects gap_unknown_annotation
Recursive boundary supplement: BE-001QW-01 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` equivalence baseline and extraction plan; next step: BE-001QW-02 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation extract_closeout.
- `markdown/06-milestones/v4.16.0/978-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.baseline_plan.md` - v4.16.0 BE-001QW-01 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation baseline plan
Recursive boundary supplement: BE-001QW-02 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` actual extraction complete; next step: BE-001QW-03 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/979-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.extract_closeout.md` - v4.16.0 BE-001QW-02 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation actual extraction closeout
- `qrpc_core/src/strategy_ir/gap_unknown_annotation.rs` - Extracted Strategy IR gap annotation and strategy unknown DTO child
Recursive boundary supplement: BE-001QW-03 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` single leaf closeout sets stop_split true; next step: BE-001QX-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects root_validation.
- `markdown/06-milestones/v4.16.0/980-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.single_leaf_closeout.md` - v4.16.0 BE-001QW-03 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation single leaf closeout
Recursive boundary supplement: BE-001QX-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects root_validation; next step: BE-001QY-01 root.contracts.qrpc_core.strategy_ir.root_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/981-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.root_validation.md` - v4.16.0 BE-001QX-01 root.contracts.qrpc_core.strategy_ir parent residual judgment selects root_validation
Recursive boundary supplement: BE-001QY-01 `root.contracts.qrpc_core.strategy_ir.root_validation` equivalence baseline and extraction plan; next step: BE-001QY-02 root.contracts.qrpc_core.strategy_ir.root_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/982-root.contracts.qrpc_core.strategy_ir.root_validation.baseline_plan.md` - v4.16.0 BE-001QY-01 root.contracts.qrpc_core.strategy_ir.root_validation baseline plan
Recursive boundary supplement: BE-001QY-02 `root.contracts.qrpc_core.strategy_ir.root_validation` actual extraction complete; next step: BE-001QY-03 root.contracts.qrpc_core.strategy_ir.root_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/983-root.contracts.qrpc_core.strategy_ir.root_validation.extract_closeout.md` - v4.16.0 BE-001QY-02 root.contracts.qrpc_core.strategy_ir.root_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation.rs` - Extracted Strategy IR root DTO, validation methods, private validation helpers, and local validation tests
Recursive boundary supplement: BE-001QY-03 `root.contracts.qrpc_core.strategy_ir.root_validation` single leaf closeout sets continue_split true; next step: BE-001QZ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects identity_required_validation.
- `markdown/06-milestones/v4.16.0/984-root.contracts.qrpc_core.strategy_ir.root_validation.single_leaf_closeout.md` - v4.16.0 BE-001QY-03 root.contracts.qrpc_core.strategy_ir.root_validation single leaf closeout continues split
Recursive boundary supplement: BE-001QZ-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects identity_required_validation; next step: BE-001RA-01 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/985-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.identity_required_validation.md` - v4.16.0 BE-001QZ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects identity_required_validation
Recursive boundary supplement: BE-001RA-01 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` equivalence baseline and extraction plan; next step: BE-001RA-02 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/986-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.baseline_plan.md` - v4.16.0 BE-001RA-01 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation baseline plan
Recursive boundary supplement: BE-001RA-02 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` actual extraction complete; next step: BE-001RA-03 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/987-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.extract_closeout.md` - v4.16.0 BE-001RA-02 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/identity_required_validation.rs` - Extracted Strategy IR root validation identity, required-field, required-collection, duplicate-id, and unique-id helper child
Recursive boundary supplement: BE-001RA-03 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` single leaf closeout sets stop_split true; next step: BE-001RB-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects signal_logic_validation.
- `markdown/06-milestones/v4.16.0/988-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.single_leaf_closeout.md` - v4.16.0 BE-001RA-03 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation single leaf closeout
Recursive boundary supplement: BE-001RB-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects signal_logic_validation; next step: BE-001RC-01 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/989-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.signal_logic_validation.md` - v4.16.0 BE-001RB-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects signal_logic_validation
Recursive governance supplement: GOV-SAME-PARENT-PARALLEL allows guarded same-parent child parallel waves without changing the active Rust cursor.
- `markdown/06-milestones/v4.16.0/990-governance.same_parent_parallel_children.protocol_update.md` - v4.16.0 guarded same-parent child parallel wave protocol update
Recursive boundary supplement: BE-001RC-01 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` equivalence baseline frozen; next step: BE-001RC-02 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/991-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.baseline_plan.md` - v4.16.0 BE-001RC-01 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation baseline plan
Recursive boundary supplement: BE-001RC-02 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` actual extraction complete; next step: BE-001RC-03 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/992-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.extract_closeout.md` - v4.16.0 BE-001RC-02 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/signal_logic_validation.rs` - Extracted Strategy IR root validation signal/detail, indicator support, logic rule, and logic unknown-marker child
Recursive boundary supplement: BE-001RC-03 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` single leaf closeout sets stop_split true; next step: BE-001RD-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects risk_validation.
- `markdown/06-milestones/v4.16.0/993-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.single_leaf_closeout.md` - v4.16.0 BE-001RC-03 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation single leaf closeout
Recursive boundary supplement: BE-001RD-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects risk_validation; next step: BE-001RE-01 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/994-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.risk_validation.md` - v4.16.0 BE-001RD-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects risk_validation
Recursive boundary supplement: BE-001RE-01 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` equivalence baseline frozen; next step: BE-001RE-02 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/995-root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation.baseline_plan.md` - v4.16.0 BE-001RE-01 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation baseline plan
Recursive boundary supplement: BE-001RE-02 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` actual extraction complete; next step: BE-001RE-03 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/996-root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation.extract_closeout.md` - v4.16.0 BE-001RE-02 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/risk_validation.rs` - Extracted Strategy IR root validation risk unknownable and risk profile validation child
Recursive boundary supplement: BE-001RE-03 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` single leaf closeout sets stop_split true; next step: BE-001RF-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects data_execution_validation.
- `markdown/06-milestones/v4.16.0/997-root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation.single_leaf_closeout.md` - v4.16.0 BE-001RE-03 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation single leaf closeout
Recursive boundary supplement: BE-001RF-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects data_execution_validation; next step: BE-001RG-01 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/998-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.data_execution_validation.md` - v4.16.0 BE-001RF-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects data_execution_validation
Recursive boundary supplement: BE-001RG-01 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` equivalence baseline frozen; next step: BE-001RG-02 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/999-root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation.baseline_plan.md` - v4.16.0 BE-001RG-01 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation baseline plan
Recursive boundary supplement: BE-001RG-02 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` actual extraction complete; next step: BE-001RG-03 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1000-root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation.extract_closeout.md` - v4.16.0 BE-001RG-02 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/data_execution_validation.rs` - Extracted Strategy IR root validation data requirement, execution, and execution profile validation child
Recursive boundary supplement: BE-001RG-03 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` single leaf closeout sets stop_split true; next step: BE-001RH-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects unknown_marker_validation.
- `markdown/06-milestones/v4.16.0/1001-root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation.single_leaf_closeout.md` - v4.16.0 BE-001RG-03 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation single leaf closeout
Recursive boundary supplement: BE-001RH-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects unknown_marker_validation; next step: BE-001RI-01 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation baseline_plan.
- `markdown/06-milestones/v4.16.0/1002-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.unknown_marker_validation.md` - v4.16.0 BE-001RH-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects unknown_marker_validation
Recursive boundary supplement: BE-001RI-01 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` equivalence baseline frozen; next step: BE-001RI-02 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation extract_closeout.
- `markdown/06-milestones/v4.16.0/1003-root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation.baseline_plan.md` - v4.16.0 BE-001RI-01 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation baseline plan
Recursive boundary supplement: BE-001RI-02 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` actual extraction complete; next step: BE-001RI-03 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1004-root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation.extract_closeout.md` - v4.16.0 BE-001RI-02 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/unknown_marker_validation.rs` - Extracted Strategy IR root validation unknown marker path/reason validation and unknownable helper implementation child
Recursive boundary supplement: BE-001RI-03 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` single leaf closeout sets stop_split true; next step: BE-001RJ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/1005-root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation.single_leaf_closeout.md` - v4.16.0 BE-001RI-03 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation single leaf closeout
Recursive boundary supplement: BE-001RJ-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects test_fixture; next step: BE-001RK-01 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture baseline_plan.
- `markdown/06-milestones/v4.16.0/1006-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.test_fixture.md` - v4.16.0 BE-001RJ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent residual judgment selects test_fixture
Recursive boundary supplement: BE-001RK-01 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` equivalence baseline frozen; next step: BE-001RK-02 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture extract_closeout.
- `markdown/06-milestones/v4.16.0/1007-root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture.baseline_plan.md` - v4.16.0 BE-001RK-01 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture baseline plan
Recursive boundary supplement: BE-001RK-02 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` actual extraction complete; next step: BE-001RK-03 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1008-root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture.extract_closeout.md` - v4.16.0 BE-001RK-02 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture actual extraction closeout
- `qrpc_core/src/strategy_ir/root_validation/tests.rs` - Extracted Strategy IR root validation local sample fixture and unit tests
Recursive boundary supplement: BE-001RK-03 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` single leaf closeout sets stop_split true; next step: BE-001RL-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/1009-root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture.single_leaf_closeout.md` - v4.16.0 BE-001RK-03 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture single leaf closeout
Recursive boundary supplement: BE-001RL-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment closes root_validation parent; next step: BE-001RM-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/1010-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001RL-01 root.contracts.qrpc_core.strategy_ir.root_validation parent closeout
Recursive boundary supplement: BE-001RM-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment closes strategy_ir parent; next step: BE-001RN-01 root.contracts.qrpc_core parent_residual_judgment selects protocol_primitives.
- `markdown/06-milestones/v4.16.0/1011-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001RM-01 root.contracts.qrpc_core.strategy_ir parent closeout
Recursive boundary supplement: BE-001RN-01 `root.contracts.qrpc_core` parent residual judgment selects protocol_primitives; next step: BE-001RO-01 root.contracts.qrpc_core.protocol_primitives baseline_plan.
- `markdown/06-milestones/v4.16.0/1012-root.contracts.qrpc_core.parent_residual_judgment.protocol_primitives.md` - v4.16.0 BE-001RN-01 root.contracts.qrpc_core parent residual judgment selects protocol_primitives
Recursive boundary supplement: BE-001RO-01 `root.contracts.qrpc_core.protocol_primitives` equivalence baseline frozen; next step: BE-001RO-02 root.contracts.qrpc_core.protocol_primitives extract_closeout.
- `markdown/06-milestones/v4.16.0/1013-root.contracts.qrpc_core.protocol_primitives.baseline_plan.md` - v4.16.0 BE-001RO-01 root.contracts.qrpc_core.protocol_primitives baseline plan
Recursive boundary supplement: BE-001RO-02 `root.contracts.qrpc_core.protocol_primitives` actual extraction complete; next step: BE-001RO-03 root.contracts.qrpc_core.protocol_primitives single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1014-root.contracts.qrpc_core.protocol_primitives.extract_closeout.md` - v4.16.0 BE-001RO-02 root.contracts.qrpc_core.protocol_primitives actual extraction closeout
- `qrpc_core/src/protocol_primitives.rs` - Extracted qrpc-core primitive constants, enums, symbol serde/parse behavior, display behavior, and defaults
Recursive boundary supplement: BE-001RO-03 `root.contracts.qrpc_core.protocol_primitives` single leaf closeout sets stop_split true; next step: BE-001RP-01 root.contracts.qrpc_core parent_residual_judgment selects runtime_protocol_config.
- `markdown/06-milestones/v4.16.0/1015-root.contracts.qrpc_core.protocol_primitives.single_leaf_closeout.md` - v4.16.0 BE-001RO-03 root.contracts.qrpc_core.protocol_primitives single leaf closeout
Recursive boundary supplement: BE-001RP-01 `root.contracts.qrpc_core` parent residual judgment selects runtime_protocol_config; next step: BE-001RQ-01 root.contracts.qrpc_core.runtime_protocol_config baseline_plan.
- `markdown/06-milestones/v4.16.0/1016-root.contracts.qrpc_core.parent_residual_judgment.runtime_protocol_config.md` - v4.16.0 BE-001RP-01 root.contracts.qrpc_core parent residual judgment selects runtime_protocol_config
Recursive boundary supplement: BE-001RQ-01 `root.contracts.qrpc_core.runtime_protocol_config` equivalence baseline frozen; next step: BE-001RQ-02 root.contracts.qrpc_core.runtime_protocol_config extract_closeout.
- `markdown/06-milestones/v4.16.0/1017-root.contracts.qrpc_core.runtime_protocol_config.baseline_plan.md` - v4.16.0 BE-001RQ-01 root.contracts.qrpc_core.runtime_protocol_config baseline plan
Recursive boundary supplement: BE-001RQ-02 `root.contracts.qrpc_core.runtime_protocol_config` actual extraction complete; next step: BE-001RQ-03 root.contracts.qrpc_core.runtime_protocol_config single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1018-root.contracts.qrpc_core.runtime_protocol_config.extract_closeout.md` - v4.16.0 BE-001RQ-02 root.contracts.qrpc_core.runtime_protocol_config actual extraction closeout
- `qrpc_core/src/runtime_protocol_config.rs` - Extracted qrpc-core runtime config DTOs, universe config DTOs, defaults, and compiled protocol container
Recursive boundary supplement: BE-001RQ-03 `root.contracts.qrpc_core.runtime_protocol_config` single leaf closeout sets stop_split true; next step: BE-001RR-01 root.contracts.qrpc_core parent_residual_judgment selects artifact_specs.
- `markdown/06-milestones/v4.16.0/1019-root.contracts.qrpc_core.runtime_protocol_config.single_leaf_closeout.md` - v4.16.0 BE-001RQ-03 root.contracts.qrpc_core.runtime_protocol_config single leaf closeout
Recursive boundary supplement: BE-001RR-01 `root.contracts.qrpc_core` parent residual judgment selects artifact_specs; next step: BE-001RS-01 root.contracts.qrpc_core.artifact_specs baseline_plan.
- `markdown/06-milestones/v4.16.0/1020-root.contracts.qrpc_core.parent_residual_judgment.artifact_specs.md` - v4.16.0 BE-001RR-01 root.contracts.qrpc_core parent residual judgment selects artifact_specs
Recursive boundary supplement: BE-001RS-01 `root.contracts.qrpc_core.artifact_specs` equivalence baseline frozen; next step: BE-001RS-02 root.contracts.qrpc_core.artifact_specs extract_closeout.
- `markdown/06-milestones/v4.16.0/1021-root.contracts.qrpc_core.artifact_specs.baseline_plan.md` - v4.16.0 BE-001RS-01 root.contracts.qrpc_core.artifact_specs baseline plan
Recursive boundary supplement: BE-001RS-02 `root.contracts.qrpc_core.artifact_specs` actual extraction complete; next step: BE-001RS-03 root.contracts.qrpc_core.artifact_specs single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1022-root.contracts.qrpc_core.artifact_specs.extract_closeout.md` - v4.16.0 BE-001RS-02 root.contracts.qrpc_core.artifact_specs actual extraction closeout
- `qrpc_core/src/artifact_specs.rs` - Extracted qrpc-core canonical digest, run/backtest specs, dataset/execution projections, and artifact bundle contracts
Recursive boundary supplement: BE-001RS-03 `root.contracts.qrpc_core.artifact_specs` single leaf closeout sets continue_split true; next step: BE-001RT-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects canonical_digest.
- `markdown/06-milestones/v4.16.0/1023-root.contracts.qrpc_core.artifact_specs.single_leaf_closeout.md` - v4.16.0 BE-001RS-03 root.contracts.qrpc_core.artifact_specs single leaf closeout continues split
Recursive boundary supplement: BE-001RT-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects canonical_digest; next step: BE-001RU-01 root.contracts.qrpc_core.artifact_specs.canonical_digest baseline_plan.
- `markdown/06-milestones/v4.16.0/1024-root.contracts.qrpc_core.artifact_specs.parent_residual_judgment.canonical_digest.md` - v4.16.0 BE-001RT-01 root.contracts.qrpc_core.artifact_specs parent residual judgment selects canonical_digest
Recursive boundary supplement: BE-001RU-01 `root.contracts.qrpc_core.artifact_specs.canonical_digest` equivalence baseline frozen; next step: BE-001RU-02 root.contracts.qrpc_core.artifact_specs.canonical_digest extract_closeout.
- `markdown/06-milestones/v4.16.0/1025-root.contracts.qrpc_core.artifact_specs.canonical_digest.baseline_plan.md` - v4.16.0 BE-001RU-01 root.contracts.qrpc_core.artifact_specs.canonical_digest baseline plan
Recursive boundary supplement: BE-001RU-02 `root.contracts.qrpc_core.artifact_specs.canonical_digest` actual extraction complete; next step: BE-001RU-03 root.contracts.qrpc_core.artifact_specs.canonical_digest single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1026-root.contracts.qrpc_core.artifact_specs.canonical_digest.extract_closeout.md` - v4.16.0 BE-001RU-02 root.contracts.qrpc_core.artifact_specs.canonical_digest actual extraction closeout
- `qrpc_core/src/artifact_specs/canonical_digest.rs` - Extracted qrpc-core artifact digest algorithm, digest DTO, and canonical JSON SHA-256 helper
Recursive boundary supplement: BE-001RU-03 `root.contracts.qrpc_core.artifact_specs.canonical_digest` single leaf closeout sets stop_split true; next step: BE-001RV-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects run_backtest_specs.
- `markdown/06-milestones/v4.16.0/1027-root.contracts.qrpc_core.artifact_specs.canonical_digest.single_leaf_closeout.md` - v4.16.0 BE-001RU-03 root.contracts.qrpc_core.artifact_specs.canonical_digest single leaf closeout
Recursive boundary supplement: BE-001RV-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects run_backtest_specs; next step: BE-001RW-01 root.contracts.qrpc_core.artifact_specs.run_backtest_specs baseline_plan.
- `markdown/06-milestones/v4.16.0/1028-root.contracts.qrpc_core.artifact_specs.parent_residual_judgment.run_backtest_specs.md` - v4.16.0 BE-001RV-01 root.contracts.qrpc_core.artifact_specs parent residual judgment selects run_backtest_specs
Recursive boundary supplement: BE-001RW-01 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` equivalence baseline frozen; next step: BE-001RW-02 root.contracts.qrpc_core.artifact_specs.run_backtest_specs extract_closeout.
- `markdown/06-milestones/v4.16.0/1029-root.contracts.qrpc_core.artifact_specs.run_backtest_specs.baseline_plan.md` - v4.16.0 BE-001RW-01 root.contracts.qrpc_core.artifact_specs.run_backtest_specs baseline plan
Recursive boundary supplement: BE-001RW-02 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` actual extraction complete; next step: BE-001RW-03 root.contracts.qrpc_core.artifact_specs.run_backtest_specs single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1030-root.contracts.qrpc_core.artifact_specs.run_backtest_specs.extract_closeout.md` - v4.16.0 BE-001RW-02 root.contracts.qrpc_core.artifact_specs.run_backtest_specs actual extraction closeout
- `qrpc_core/src/artifact_specs/run_backtest_specs.rs` - Extracted qrpc-core run/backtest modes, dataset/execution projections, market data snapshot specs, RunSpec, and BacktestSpec
Recursive boundary supplement: BE-001RW-03 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` single leaf closeout sets stop_split true; next step: BE-001RX-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects artifact_bundle_contract.
- `markdown/06-milestones/v4.16.0/1031-root.contracts.qrpc_core.artifact_specs.run_backtest_specs.single_leaf_closeout.md` - v4.16.0 BE-001RW-03 root.contracts.qrpc_core.artifact_specs.run_backtest_specs single leaf closeout
Recursive boundary supplement: BE-001RX-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects artifact_bundle_contract; next step: BE-001RY-01 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/1032-root.contracts.qrpc_core.artifact_specs.parent_residual_judgment.artifact_bundle_contract.md` - v4.16.0 BE-001RX-01 root.contracts.qrpc_core.artifact_specs parent residual judgment selects artifact_bundle_contract
Recursive boundary supplement: BE-001RY-01 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` equivalence baseline frozen; next step: BE-001RY-02 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/1033-root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract.baseline_plan.md` - v4.16.0 BE-001RY-01 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract baseline plan
Recursive boundary supplement: BE-001RY-02 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` actual extraction complete; next step: BE-001RY-03 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1034-root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract.extract_closeout.md` - v4.16.0 BE-001RY-02 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract actual extraction closeout
- `qrpc_core/src/artifact_specs/artifact_bundle_contract.rs` - Extracted qrpc-core strategy/core-IR/compile artifact DTOs and CompileArtifactBundle
Recursive boundary supplement: BE-001RY-03 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` single leaf closeout sets stop_split true; next step: BE-001RZ-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment.
- `markdown/06-milestones/v4.16.0/1035-root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract.single_leaf_closeout.md` - v4.16.0 BE-001RY-03 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract single leaf closeout
Recursive boundary supplement: BE-001RZ-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment closes parent; next step: BE-001SA-01 root.contracts.qrpc_core parent_residual_judgment selects runtime_io_contract.
- `markdown/06-milestones/v4.16.0/1036-root.contracts.qrpc_core.artifact_specs.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001RZ-01 root.contracts.qrpc_core.artifact_specs parent closeout
Recursive boundary supplement: BE-001SA-01 `root.contracts.qrpc_core` parent residual judgment selects runtime_io_contract; next step: BE-001SB-01 root.contracts.qrpc_core.runtime_io_contract baseline_plan.
- `markdown/06-milestones/v4.16.0/1037-root.contracts.qrpc_core.parent_residual_judgment.runtime_io_contract.md` - v4.16.0 BE-001SA-01 root.contracts.qrpc_core parent residual judgment selects runtime_io_contract
Recursive boundary supplement: BE-001SB-01 `root.contracts.qrpc_core.runtime_io_contract` equivalence baseline frozen; next step: BE-001SB-02 root.contracts.qrpc_core.runtime_io_contract extract_closeout.
- `markdown/06-milestones/v4.16.0/1038-root.contracts.qrpc_core.runtime_io_contract.baseline_plan.md` - v4.16.0 BE-001SB-01 root.contracts.qrpc_core.runtime_io_contract baseline plan
Recursive boundary supplement: BE-001SB-02 `root.contracts.qrpc_core.runtime_io_contract` actual extraction complete; next step: BE-001SB-03 root.contracts.qrpc_core.runtime_io_contract single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1039-root.contracts.qrpc_core.runtime_io_contract.extract_closeout.md` - v4.16.0 BE-001SB-02 root.contracts.qrpc_core.runtime_io_contract actual extraction closeout
- `qrpc_core/src/runtime_io_contract.rs` - Extracted qrpc-core runtime input/output DTOs from RawKline through BacktestOutput
Recursive boundary supplement: BE-001SB-03 `root.contracts.qrpc_core.runtime_io_contract` single leaf closeout sets continue_split true; next step: BE-001SC-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects market_data_io.
- `markdown/06-milestones/v4.16.0/1040-root.contracts.qrpc_core.runtime_io_contract.single_leaf_closeout.md` - v4.16.0 BE-001SB-03 root.contracts.qrpc_core.runtime_io_contract single leaf closeout continues split
Recursive boundary supplement: BE-001SC-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects market_data_io; next step: BE-001SD-01 root.contracts.qrpc_core.runtime_io_contract.market_data_io baseline_plan.
- `markdown/06-milestones/v4.16.0/1041-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.market_data_io.md` - v4.16.0 BE-001SC-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects market_data_io
Recursive boundary supplement: BE-001SD-01 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` equivalence baseline frozen; next step: BE-001SD-02 root.contracts.qrpc_core.runtime_io_contract.market_data_io extract_closeout.
- `markdown/06-milestones/v4.16.0/1042-root.contracts.qrpc_core.runtime_io_contract.market_data_io.baseline_plan.md` - v4.16.0 BE-001SD-01 root.contracts.qrpc_core.runtime_io_contract.market_data_io baseline plan
Recursive boundary supplement: BE-001SD-02 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` actual extraction complete; next step: BE-001SD-03 root.contracts.qrpc_core.runtime_io_contract.market_data_io single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1043-root.contracts.qrpc_core.runtime_io_contract.market_data_io.extract_closeout.md` - v4.16.0 BE-001SD-02 root.contracts.qrpc_core.runtime_io_contract.market_data_io actual extraction closeout
- `qrpc_core/src/runtime_io_contract/market_data_io.rs` - Extracted qrpc-core runtime IO raw and normalized market data DTOs
Recursive boundary supplement: BE-001SD-03 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` single leaf closeout sets stop_split true; next step: BE-001SE-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects decision_flow.
- `markdown/06-milestones/v4.16.0/1044-root.contracts.qrpc_core.runtime_io_contract.market_data_io.single_leaf_closeout.md` - v4.16.0 BE-001SD-03 root.contracts.qrpc_core.runtime_io_contract.market_data_io single leaf closeout
Recursive boundary supplement: BE-001SE-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects decision_flow; next step: BE-001SF-01 root.contracts.qrpc_core.runtime_io_contract.decision_flow baseline_plan.
- `markdown/06-milestones/v4.16.0/1045-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.decision_flow.md` - v4.16.0 BE-001SE-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects decision_flow
Recursive boundary supplement: BE-001SF-01 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` equivalence baseline frozen; next step: BE-001SF-02 root.contracts.qrpc_core.runtime_io_contract.decision_flow extract_closeout.
- `markdown/06-milestones/v4.16.0/1046-root.contracts.qrpc_core.runtime_io_contract.decision_flow.baseline_plan.md` - v4.16.0 BE-001SF-01 root.contracts.qrpc_core.runtime_io_contract.decision_flow baseline plan
Recursive boundary supplement: BE-001SF-02 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` actual extraction complete; next step: BE-001SF-03 root.contracts.qrpc_core.runtime_io_contract.decision_flow single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1047-root.contracts.qrpc_core.runtime_io_contract.decision_flow.extract_closeout.md` - v4.16.0 BE-001SF-02 root.contracts.qrpc_core.runtime_io_contract.decision_flow actual extraction closeout
- `qrpc_core/src/runtime_io_contract/decision_flow.rs` - Extracted qrpc-core runtime IO intent/action/target/agent/risk decision DTOs
Recursive boundary supplement: BE-001SF-03 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` single leaf closeout sets stop_split true; next step: BE-001SG-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects execution_io.
- `markdown/06-milestones/v4.16.0/1048-root.contracts.qrpc_core.runtime_io_contract.decision_flow.single_leaf_closeout.md` - v4.16.0 BE-001SF-03 root.contracts.qrpc_core.runtime_io_contract.decision_flow single leaf closeout
Recursive boundary supplement: BE-001SG-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects execution_io; next step: BE-001SH-01 root.contracts.qrpc_core.runtime_io_contract.execution_io baseline_plan.
- `markdown/06-milestones/v4.16.0/1049-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.execution_io.md` - v4.16.0 BE-001SG-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects execution_io
Recursive boundary supplement: BE-001SH-01 `root.contracts.qrpc_core.runtime_io_contract.execution_io` equivalence baseline frozen; next step: BE-001SH-02 root.contracts.qrpc_core.runtime_io_contract.execution_io extract_closeout.
- `markdown/06-milestones/v4.16.0/1050-root.contracts.qrpc_core.runtime_io_contract.execution_io.baseline_plan.md` - v4.16.0 BE-001SH-01 root.contracts.qrpc_core.runtime_io_contract.execution_io baseline plan
Recursive boundary supplement: BE-001SH-02 `root.contracts.qrpc_core.runtime_io_contract.execution_io` actual extraction complete; next step: BE-001SH-03 root.contracts.qrpc_core.runtime_io_contract.execution_io single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1051-root.contracts.qrpc_core.runtime_io_contract.execution_io.extract_closeout.md` - v4.16.0 BE-001SH-02 root.contracts.qrpc_core.runtime_io_contract.execution_io actual extraction closeout
- `qrpc_core/src/runtime_io_contract/execution_io.rs` - Extracted qrpc-core runtime IO simulated order, execution plan, fill report, open order, and fill result DTOs
Recursive boundary supplement: BE-001SH-03 `root.contracts.qrpc_core.runtime_io_contract.execution_io` single leaf closeout sets stop_split true; next step: BE-001SI-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects portfolio_state.
- `markdown/06-milestones/v4.16.0/1052-root.contracts.qrpc_core.runtime_io_contract.execution_io.single_leaf_closeout.md` - v4.16.0 BE-001SH-03 root.contracts.qrpc_core.runtime_io_contract.execution_io single leaf closeout
Recursive boundary supplement: BE-001SI-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects portfolio_state; next step: BE-001SJ-01 root.contracts.qrpc_core.runtime_io_contract.portfolio_state baseline_plan.
- `markdown/06-milestones/v4.16.0/1053-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.portfolio_state.md` - v4.16.0 BE-001SI-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects portfolio_state
Recursive boundary supplement: BE-001SJ-01 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` equivalence baseline frozen; next step: BE-001SJ-02 root.contracts.qrpc_core.runtime_io_contract.portfolio_state extract_closeout.
- `markdown/06-milestones/v4.16.0/1054-root.contracts.qrpc_core.runtime_io_contract.portfolio_state.baseline_plan.md` - v4.16.0 BE-001SJ-01 root.contracts.qrpc_core.runtime_io_contract.portfolio_state baseline plan
Recursive boundary supplement: BE-001SJ-02 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` actual extraction complete; next step: BE-001SJ-03 root.contracts.qrpc_core.runtime_io_contract.portfolio_state single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1055-root.contracts.qrpc_core.runtime_io_contract.portfolio_state.extract_closeout.md` - v4.16.0 BE-001SJ-02 root.contracts.qrpc_core.runtime_io_contract.portfolio_state actual extraction closeout
- `qrpc_core/src/runtime_io_contract/portfolio_state.rs` - Extracted qrpc-core runtime IO position, exchange exposure, portfolio state DTOs, and helper methods
Recursive boundary supplement: BE-001SJ-03 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` single leaf closeout sets stop_split true; next step: BE-001SK-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects runtime_output.
- `markdown/06-milestones/v4.16.0/1056-root.contracts.qrpc_core.runtime_io_contract.portfolio_state.single_leaf_closeout.md` - v4.16.0 BE-001SJ-03 root.contracts.qrpc_core.runtime_io_contract.portfolio_state single leaf closeout
Recursive boundary supplement: BE-001SK-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects runtime_output; next step: BE-001SL-01 root.contracts.qrpc_core.runtime_io_contract.runtime_output baseline_plan.
- `markdown/06-milestones/v4.16.0/1057-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.runtime_output.md` - v4.16.0 BE-001SK-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects runtime_output
Recursive boundary supplement: BE-001SL-01 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` equivalence baseline frozen; next step: BE-001SL-02 root.contracts.qrpc_core.runtime_io_contract.runtime_output extract_closeout.
- `markdown/06-milestones/v4.16.0/1058-root.contracts.qrpc_core.runtime_io_contract.runtime_output.baseline_plan.md` - v4.16.0 BE-001SL-01 root.contracts.qrpc_core.runtime_io_contract.runtime_output baseline plan
Recursive boundary supplement: BE-001SL-02 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` actual extraction complete; next step: BE-001SL-03 root.contracts.qrpc_core.runtime_io_contract.runtime_output single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1059-root.contracts.qrpc_core.runtime_io_contract.runtime_output.extract_closeout.md` - v4.16.0 BE-001SL-02 root.contracts.qrpc_core.runtime_io_contract.runtime_output actual extraction closeout
- `qrpc_core/src/runtime_io_contract/runtime_output.rs` - Extracted qrpc-core runtime IO runtime event, cycle output, and session output DTOs
Recursive boundary supplement: BE-001SL-03 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` single leaf closeout sets stop_split true; next step: BE-001SM-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects backtest_output.
- `markdown/06-milestones/v4.16.0/1060-root.contracts.qrpc_core.runtime_io_contract.runtime_output.single_leaf_closeout.md` - v4.16.0 BE-001SL-03 root.contracts.qrpc_core.runtime_io_contract.runtime_output single leaf closeout
Recursive boundary supplement: BE-001SM-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects backtest_output; next step: BE-001SN-01 root.contracts.qrpc_core.runtime_io_contract.backtest_output baseline_plan.
- `markdown/06-milestones/v4.16.0/1061-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.backtest_output.md` - v4.16.0 BE-001SM-01 root.contracts.qrpc_core.runtime_io_contract parent residual judgment selects backtest_output
Recursive boundary supplement: BE-001SN-01 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` equivalence baseline frozen; next step: BE-001SN-02 root.contracts.qrpc_core.runtime_io_contract.backtest_output extract_closeout.
- `markdown/06-milestones/v4.16.0/1062-root.contracts.qrpc_core.runtime_io_contract.backtest_output.baseline_plan.md` - v4.16.0 BE-001SN-01 root.contracts.qrpc_core.runtime_io_contract.backtest_output baseline plan
Recursive boundary supplement: BE-001SN-02 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` actual extraction complete; next step: BE-001SN-03 root.contracts.qrpc_core.runtime_io_contract.backtest_output single_leaf_closeout.
- `markdown/06-milestones/v4.16.0/1063-root.contracts.qrpc_core.runtime_io_contract.backtest_output.extract_closeout.md` - v4.16.0 BE-001SN-02 root.contracts.qrpc_core.runtime_io_contract.backtest_output actual extraction closeout
- `qrpc_core/src/runtime_io_contract/backtest_output.rs` - Extracted qrpc-core runtime IO final backtest output DTOs and nested metric DTOs
Recursive boundary supplement: BE-001SN-03 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` single leaf closeout sets stop_split true; next step: BE-001SO-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment closes parent.
- `markdown/06-milestones/v4.16.0/1064-root.contracts.qrpc_core.runtime_io_contract.backtest_output.single_leaf_closeout.md` - v4.16.0 BE-001SN-03 root.contracts.qrpc_core.runtime_io_contract.backtest_output single leaf closeout
Recursive boundary supplement: BE-001SO-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment closes parent; next step: BE-001SP-01 root.contracts.qrpc_core parent_residual_judgment selects rfc_execution_contracts.
- `markdown/06-milestones/v4.16.0/1065-root.contracts.qrpc_core.runtime_io_contract.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001SO-01 root.contracts.qrpc_core.runtime_io_contract parent closeout
递归边界补充: BE-001SP-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects rfc_execution_contracts；下一步: BE-001SQ-01 root.contracts.qrpc_core.rfc_execution_contracts baseline_plan。
- `markdown/06-milestones/v4.16.0/1066-root.contracts.qrpc_core.parent_residual_judgment.rfc_execution_contracts.md` - v4.16.0 BE-001SP-01 root.contracts.qrpc_core parent residual judgment selects rfc_execution_contracts
递归边界补充: BE-001SQ-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts equivalence baseline and extraction plan；下一步: BE-001SQ-02 root.contracts.qrpc_core.rfc_execution_contracts extract_closeout。
- `markdown/06-milestones/v4.16.0/1067-root.contracts.qrpc_core.rfc_execution_contracts.baseline_plan.md` - v4.16.0 BE-001SQ-01 root.contracts.qrpc_core.rfc_execution_contracts equivalence baseline and extraction plan
递归边界补充: BE-001SQ-02 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts actual extraction complete；下一步: BE-001SQ-03 root.contracts.qrpc_core.rfc_execution_contracts single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1068-root.contracts.qrpc_core.rfc_execution_contracts.extract_closeout.md` - v4.16.0 BE-001SQ-02 root.contracts.qrpc_core.rfc_execution_contracts actual extraction complete
- `qrpc_core/src/rfc_execution_contracts.rs` - Extracted qrpc-core RFC execution contracts from MarketScope through HandoffSnapshot
递归边界补充: BE-001SQ-03 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts single leaf closeout continues split；下一步: BE-001SR-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects data_request。
- `markdown/06-milestones/v4.16.0/1069-root.contracts.qrpc_core.rfc_execution_contracts.single_leaf_closeout.md` - v4.16.0 BE-001SQ-03 root.contracts.qrpc_core.rfc_execution_contracts single leaf closeout continues split
递归边界补充: BE-001SR-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects data_request；下一步: BE-001SS-01 root.contracts.qrpc_core.rfc_execution_contracts.data_request baseline_plan。
- `markdown/06-milestones/v4.16.0/1070-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.data_request.md` - v4.16.0 BE-001SR-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects data_request
递归边界补充: BE-001SS-01 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request equivalence baseline and extraction plan；下一步: BE-001SS-02 root.contracts.qrpc_core.rfc_execution_contracts.data_request extract_closeout。
- `markdown/06-milestones/v4.16.0/1071-root.contracts.qrpc_core.rfc_execution_contracts.data_request.baseline_plan.md` - v4.16.0 BE-001SS-01 root.contracts.qrpc_core.rfc_execution_contracts.data_request equivalence baseline and extraction plan
递归边界补充: BE-001SS-02 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request actual extraction complete；下一步: BE-001SS-03 root.contracts.qrpc_core.rfc_execution_contracts.data_request single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1072-root.contracts.qrpc_core.rfc_execution_contracts.data_request.extract_closeout.md` - v4.16.0 BE-001SS-02 root.contracts.qrpc_core.rfc_execution_contracts.data_request actual extraction complete
- `qrpc_core/src/rfc_execution_contracts/data_request.rs` - Extracted qrpc-core RFC data request taxonomy and DTO contract
递归边界补充: BE-001SS-03 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request single leaf closeout stops split；下一步: BE-001ST-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects allocation。
- `markdown/06-milestones/v4.16.0/1073-root.contracts.qrpc_core.rfc_execution_contracts.data_request.single_leaf_closeout.md` - v4.16.0 BE-001SS-03 root.contracts.qrpc_core.rfc_execution_contracts.data_request single leaf closeout stops split
递归边界补充: BE-001ST-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects allocation；下一步: BE-001SU-01 root.contracts.qrpc_core.rfc_execution_contracts.allocation baseline_plan。
- `markdown/06-milestones/v4.16.0/1074-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.allocation.md` - v4.16.0 BE-001ST-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects allocation
递归边界补充: BE-001SU-01 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation equivalence baseline and extraction plan；下一步: BE-001SU-02 root.contracts.qrpc_core.rfc_execution_contracts.allocation extract_closeout。
- `markdown/06-milestones/v4.16.0/1075-root.contracts.qrpc_core.rfc_execution_contracts.allocation.baseline_plan.md` - v4.16.0 BE-001SU-01 root.contracts.qrpc_core.rfc_execution_contracts.allocation equivalence baseline and extraction plan
递归边界补充: BE-001SU-02 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation actual extraction complete；下一步: BE-001SU-03 root.contracts.qrpc_core.rfc_execution_contracts.allocation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1076-root.contracts.qrpc_core.rfc_execution_contracts.allocation.extract_closeout.md` - v4.16.0 BE-001SU-02 root.contracts.qrpc_core.rfc_execution_contracts.allocation actual extraction complete
- `qrpc_core/src/rfc_execution_contracts/allocation.rs` - Extracted qrpc-core RFC allocation DTO and projection helper contract
递归边界补充: BE-001SU-03 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation single leaf closeout stops split；下一步: BE-001SV-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects order_contract。
- `markdown/06-milestones/v4.16.0/1077-root.contracts.qrpc_core.rfc_execution_contracts.allocation.single_leaf_closeout.md` - v4.16.0 BE-001SU-03 root.contracts.qrpc_core.rfc_execution_contracts.allocation single leaf closeout stops split
递归边界补充: BE-001SV-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects order_contract；下一步: BE-001SW-01 root.contracts.qrpc_core.rfc_execution_contracts.order_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1078-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.order_contract.md` - v4.16.0 BE-001SV-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects order_contract
递归边界补充: BE-001SW-01 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract equivalence baseline and extraction plan；下一步: BE-001SW-02 root.contracts.qrpc_core.rfc_execution_contracts.order_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1079-root.contracts.qrpc_core.rfc_execution_contracts.order_contract.baseline_plan.md` - v4.16.0 BE-001SW-01 root.contracts.qrpc_core.rfc_execution_contracts.order_contract equivalence baseline and extraction plan
递归边界补充: BE-001SW-02 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract actual extraction complete；下一步: BE-001SW-03 root.contracts.qrpc_core.rfc_execution_contracts.order_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1080-root.contracts.qrpc_core.rfc_execution_contracts.order_contract.extract_closeout.md` - v4.16.0 BE-001SW-02 root.contracts.qrpc_core.rfc_execution_contracts.order_contract actual extraction complete
- `qrpc_core/src/rfc_execution_contracts/order_contract.rs` - Extracted qrpc-core RFC order DTO and transition matrix contract
递归边界补充: BE-001SW-03 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract single leaf closeout stops split；下一步: BE-001SX-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects execution_feedback。
- `markdown/06-milestones/v4.16.0/1081-root.contracts.qrpc_core.rfc_execution_contracts.order_contract.single_leaf_closeout.md` - v4.16.0 BE-001SW-03 root.contracts.qrpc_core.rfc_execution_contracts.order_contract single leaf closeout stops split
递归边界补充: BE-001SX-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects execution_feedback；下一步: BE-001SY-01 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback baseline_plan。
- `markdown/06-milestones/v4.16.0/1082-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.execution_feedback.md` - v4.16.0 BE-001SX-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects execution_feedback
递归边界补充: BE-001SY-01 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback equivalence baseline and extraction plan；下一步: BE-001SY-02 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback extract_closeout。
- `markdown/06-milestones/v4.16.0/1083-root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback.baseline_plan.md` - v4.16.0 BE-001SY-01 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback equivalence baseline and extraction plan
递归边界补充: BE-001SY-02 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback actual extraction complete；下一步: BE-001SY-03 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1084-root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback.extract_closeout.md` - v4.16.0 BE-001SY-02 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback actual extraction complete
- `qrpc_core/src/rfc_execution_contracts/execution_feedback.rs` - Extracted qrpc-core RFC execution feedback DTO contract
递归边界补充: BE-001SY-03 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single leaf closeout stops split；下一步: BE-001SZ-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects handoff_snapshot。
- `markdown/06-milestones/v4.16.0/1085-root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback.single_leaf_closeout.md` - v4.16.0 BE-001SY-03 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single leaf closeout stops split
递归边界补充: BE-001SZ-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects handoff_snapshot；下一步: BE-001TA-01 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot baseline_plan。
- `markdown/06-milestones/v4.16.0/1086-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.handoff_snapshot.md` - v4.16.0 BE-001SZ-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects handoff_snapshot
递归边界补充: BE-001TA-01 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot equivalence baseline and extraction plan；下一步: BE-001TA-02 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot extract_closeout。
- `markdown/06-milestones/v4.16.0/1087-root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot.baseline_plan.md` - v4.16.0 BE-001TA-01 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot equivalence baseline and extraction plan
递归边界补充: BE-001TA-02 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot actual extraction complete；下一步: BE-001TA-03 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1088-root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot.extract_closeout.md` - v4.16.0 BE-001TA-02 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot actual extraction complete
- `qrpc_core/src/rfc_execution_contracts/handoff_snapshot.rs` - Extracted qrpc-core RFC handoff snapshot DTO and validation contract
递归边界补充: BE-001TA-03 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single leaf closeout stops split；下一步: BE-001TB-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment closes parent。
- `markdown/06-milestones/v4.16.0/1089-root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot.single_leaf_closeout.md` - v4.16.0 BE-001TA-03 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single leaf closeout stops split
递归边界补充: BE-001TB-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment closes parent；下一步: BE-001TC-01 root.contracts.qrpc_core parent_residual_judgment closes parent。
- `markdown/06-milestones/v4.16.0/1090-root.contracts.qrpc_core.rfc_execution_contracts.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001TB-01 root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment closes parent
递归边界补充: BE-001TC-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects test_fixture；下一步: BE-001TD-01 root.contracts.qrpc_core.test_fixture baseline_plan。
- `markdown/06-milestones/v4.16.0/1091-root.contracts.qrpc_core.parent_residual_judgment.test_fixture.md` - v4.16.0 BE-001TC-01 root.contracts.qrpc_core parent residual judgment selects test_fixture
递归边界补充: BE-001TD-01 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture equivalence baseline and extraction plan；下一步: BE-001TD-02 root.contracts.qrpc_core.test_fixture extract_closeout。
- `markdown/06-milestones/v4.16.0/1092-root.contracts.qrpc_core.test_fixture.baseline_plan.md` - v4.16.0 BE-001TD-01 root.contracts.qrpc_core.test_fixture equivalence baseline and extraction plan
递归边界补充: BE-001TD-02 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture actual extraction complete；下一步: BE-001TD-03 root.contracts.qrpc_core.test_fixture single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1093-root.contracts.qrpc_core.test_fixture.extract_closeout.md` - v4.16.0 BE-001TD-02 root.contracts.qrpc_core.test_fixture actual extraction complete
- `qrpc_core/src/tests.rs` - Extracted qrpc-core crate-root test fixture and regression tests
递归边界补充: BE-001TD-03 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture single leaf closeout stops split；下一步: BE-001TE-01 root.contracts.qrpc_core parent_residual_judgment closes parent。
- `markdown/06-milestones/v4.16.0/1094-root.contracts.qrpc_core.test_fixture.single_leaf_closeout.md` - v4.16.0 BE-001TD-03 root.contracts.qrpc_core.test_fixture single leaf closeout stops split
递归边界补充: BE-001TE-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment closes parent；下一步: BE-001TF-01 root.contracts parent_residual_judgment selects core_ir。
- `markdown/06-milestones/v4.16.0/1095-root.contracts_qrpc_core.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001TE-01 root.contracts.qrpc_core parent residual judgment closes parent
递归边界补充: BE-001TF-01 `root.contracts` root.contracts parent residual judgment selects core_ir；下一步: BE-001TG-01 root.contracts.core_ir baseline_plan。
- `markdown/06-milestones/v4.16.0/1096-root.contracts.parent_residual_judgment.core_ir.md` - v4.16.0 BE-001TF-01 root.contracts parent residual judgment selects core_ir
递归边界补充: BE-001TG-01 `root.contracts.core_ir` root.contracts.core_ir equivalence baseline and extraction plan；下一步: BE-001TH-01 root.contracts.core_ir parent_residual_judgment selects v1_contract。
- `markdown/06-milestones/v4.16.0/1097-root.contracts.core_ir.baseline_plan.md` - v4.16.0 BE-001TG-01 root.contracts.core_ir equivalence baseline and extraction plan
递归边界补充: BE-001TH-01 `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v1_contract；下一步: BE-001TI-01 root.contracts.core_ir.v1_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1098-root.contracts.core_ir.parent_residual_judgment.v1_contract.md` - v4.16.0 BE-001TH-01 root.contracts.core_ir parent residual judgment selects v1_contract
递归边界补充: BE-001TI-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract equivalence baseline and extraction plan；下一步: BE-001TI-02 root.contracts.core_ir.v1_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1099-root.contracts.core_ir.v1_contract.baseline_plan.md` - v4.16.0 BE-001TI-01 root.contracts.core_ir.v1_contract equivalence baseline and extraction plan
递归边界补充: BE-001TI-02 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract actual extraction complete；下一步: BE-001TI-03 root.contracts.core_ir.v1_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1100-root.contracts.core_ir.v1_contract.extract_closeout.md` - v4.16.0 BE-001TI-02 root.contracts.core_ir.v1_contract actual extraction complete
- `qrpc_core_ir/src/v1.rs` - Extracted qrpc-core-ir v1 Core IR schema, helpers, validation, and tests
递归边界补充: BE-001TI-03 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract single leaf closeout continues split；下一步: BE-001TJ-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects root_graph_contract。
- `markdown/06-milestones/v4.16.0/1101-root.contracts.core_ir.v1_contract.single_leaf_closeout.md` - v4.16.0 BE-001TI-03 root.contracts.core_ir.v1_contract single leaf closeout continues split
递归边界补充: BE-001TJ-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects root_graph_contract；下一步: BE-001TK-01 root.contracts.core_ir.v1_contract.root_graph_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1102-root.contracts.core_ir.v1_contract.parent_residual_judgment.root_graph_contract.md` - v4.16.0 BE-001TJ-01 root.contracts.core_ir.v1_contract parent residual judgment selects root_graph_contract
递归边界补充: BE-001TK-01 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract equivalence baseline and extraction plan；下一步: BE-001TK-02 root.contracts.core_ir.v1_contract.root_graph_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1103-root.contracts.core_ir.v1_contract.root_graph_contract.baseline_plan.md` - v4.16.0 BE-001TK-01 root.contracts.core_ir.v1_contract.root_graph_contract equivalence baseline and extraction plan
递归边界补充: BE-001TK-02 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract actual extraction complete；下一步: BE-001TK-03 root.contracts.core_ir.v1_contract.root_graph_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1104-root.contracts.core_ir.v1_contract.root_graph_contract.extract_closeout.md` - v4.16.0 BE-001TK-02 root.contracts.core_ir.v1_contract.root_graph_contract actual extraction complete
- `qrpc_core_ir/src/v1/root_graph_contract.rs` - Extracted qrpc-core-ir v1 root graph schema, constructor, and DAG validation contract
递归边界补充: BE-001TK-03 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract single leaf closeout stops split；下一步: BE-001TL-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects data_indicator_expression_contract。
- `markdown/06-milestones/v4.16.0/1105-root.contracts.core_ir.v1_contract.root_graph_contract.single_leaf_closeout.md` - v4.16.0 BE-001TK-03 root.contracts.core_ir.v1_contract.root_graph_contract single leaf closeout stops split
递归边界补充: BE-001TL-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects data_indicator_expression_contract；下一步: BE-001TM-01 root.contracts.core_ir.v1_contract.data_indicator_expression_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1106-root.contracts.core_ir.v1_contract.parent_residual_judgment.data_indicator_expression_contract.md` - v4.16.0 BE-001TL-01 root.contracts.core_ir.v1_contract parent residual judgment selects data_indicator_expression_contract
递归边界补充: BE-001TM-01 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract equivalence baseline and extraction plan；下一步: BE-001TM-02 root.contracts.core_ir.v1_contract.data_indicator_expression_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1107-root.contracts.core_ir.v1_contract.data_indicator_expression_contract.baseline_plan.md` - v4.16.0 BE-001TM-01 root.contracts.core_ir.v1_contract.data_indicator_expression_contract equivalence baseline and extraction plan
递归边界补充: BE-001TM-02 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract actual extraction complete；下一步: BE-001TM-03 root.contracts.core_ir.v1_contract.data_indicator_expression_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1108-root.contracts.core_ir.v1_contract.data_indicator_expression_contract.extract_closeout.md` - v4.16.0 BE-001TM-02 root.contracts.core_ir.v1_contract.data_indicator_expression_contract actual extraction complete
- `qrpc_core_ir/src/v1/data_indicator_expression_contract.rs` - Extracted qrpc-core-ir v1 data, indicator, expression DTOs and helper contract
递归边界补充: BE-001TM-03 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract single leaf closeout stops split；下一步: BE-001TN-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects policy_execution_contract。
- `markdown/06-milestones/v4.16.0/1109-root.contracts.core_ir.v1_contract.data_indicator_expression_contract.single_leaf_closeout.md` - v4.16.0 BE-001TM-03 root.contracts.core_ir.v1_contract.data_indicator_expression_contract single leaf closeout stops split
递归边界补充: BE-001TN-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects policy_execution_contract；下一步: BE-001TO-01 root.contracts.core_ir.v1_contract.policy_execution_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1110-root.contracts.core_ir.v1_contract.parent_residual_judgment.policy_execution_contract.md` - v4.16.0 BE-001TN-01 root.contracts.core_ir.v1_contract parent residual judgment selects policy_execution_contract
递归边界补充: BE-001TO-01 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract equivalence baseline and extraction plan；下一步: BE-001TO-02 root.contracts.core_ir.v1_contract.policy_execution_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1111-root.contracts.core_ir.v1_contract.policy_execution_contract.baseline_plan.md` - v4.16.0 BE-001TO-01 root.contracts.core_ir.v1_contract.policy_execution_contract equivalence baseline and extraction plan
递归边界补充: BE-001TO-02 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract actual extraction complete；下一步: BE-001TO-03 root.contracts.core_ir.v1_contract.policy_execution_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1112-root.contracts.core_ir.v1_contract.policy_execution_contract.extract_closeout.md` - v4.16.0 BE-001TO-02 root.contracts.core_ir.v1_contract.policy_execution_contract actual extraction complete
- qrpc_core_ir/src/v1/policy_execution_contract.rs - Extracted qrpc-core-ir v1 signal, agent policy, risk policy, execution DTOs, and default helper contract
递归边界补充: BE-001TO-03 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract single leaf closeout stops split；下一步: BE-001TP-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects test_fixture。
- `markdown/06-milestones/v4.16.0/1113-root.contracts.core_ir.v1_contract.policy_execution_contract.single_leaf_closeout.md` - v4.16.0 BE-001TO-03 root.contracts.core_ir.v1_contract.policy_execution_contract single leaf closeout stops split
递归边界补充: BE-001TP-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects test_fixture；下一步: BE-001TQ-01 root.contracts.core_ir.v1_contract.test_fixture baseline_plan。
- `markdown/06-milestones/v4.16.0/1114-root.contracts.core_ir.v1_contract.parent_residual_judgment.test_fixture.md` - v4.16.0 BE-001TP-01 root.contracts.core_ir.v1_contract parent residual judgment selects test_fixture
递归边界补充: BE-001TQ-01 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture equivalence baseline and extraction plan；下一步: BE-001TQ-02 root.contracts.core_ir.v1_contract.test_fixture extract_closeout。
- `markdown/06-milestones/v4.16.0/1115-root.contracts.core_ir.v1_contract.test_fixture.baseline_plan.md` - v4.16.0 BE-001TQ-01 root.contracts.core_ir.v1_contract.test_fixture equivalence baseline and extraction plan
递归边界补充: BE-001TQ-02 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture actual extraction complete；下一步: BE-001TQ-03 root.contracts.core_ir.v1_contract.test_fixture single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1116-root.contracts.core_ir.v1_contract.test_fixture.extract_closeout.md` - v4.16.0 BE-001TQ-02 root.contracts.core_ir.v1_contract.test_fixture actual extraction complete
- qrpc_core_ir/src/v1/tests.rs - Extracted qrpc-core-ir v1 cfg-test round-trip fixture
递归边界补充: BE-001TQ-03 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture single leaf closeout stops split；下一步: BE-001TR-01 root.contracts.core_ir.v1_contract parent_residual_judgment closes parent。
- `markdown/06-milestones/v4.16.0/1117-root.contracts.core_ir.v1_contract.test_fixture.single_leaf_closeout.md` - v4.16.0 BE-001TQ-03 root.contracts.core_ir.v1_contract.test_fixture single leaf closeout stops split
递归边界补充: BE-001TR-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment closes parent；下一步: BE-001TS-01 root.contracts.core_ir parent_residual_judgment selects v4_contracts。
- `markdown/06-milestones/v4.16.0/1118-root.contracts.core_ir.v1_contract.parent_residual_judgment.close_parent.md` - v4.16.0 BE-001TR-01 root.contracts.core_ir.v1_contract parent residual judgment closes parent
递归边界补充: BE-001TS-01 `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v4_contracts；下一步: BE-001TT-01 root.contracts.core_ir.v4_contracts baseline_plan。
- `markdown/06-milestones/v4.16.0/1119-root.contracts.core_ir.parent_residual_judgment.v4_contracts.md` - v4.16.0 BE-001TS-01 root.contracts.core_ir parent residual judgment selects v4_contracts
递归边界补充: BE-001TT-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts equivalence baseline and split plan；下一步: BE-001TU-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects schema_identity_constants。
- `markdown/06-milestones/v4.16.0/1120-root.contracts.core_ir.v4_contracts.baseline_plan.md` - v4.16.0 BE-001TT-01 root.contracts.core_ir.v4_contracts equivalence baseline and split plan
递归边界补充: BE-001TU-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects schema_identity_constants；下一步: BE-001TV-01 root.contracts.core_ir.v4_contracts.schema_identity_constants baseline_plan。
- `markdown/06-milestones/v4.16.0/1121-root.contracts.core_ir.v4_contracts.parent_residual_judgment.schema_identity_constants.md` - v4.16.0 BE-001TU-01 root.contracts.core_ir.v4_contracts parent residual judgment selects schema_identity_constants
递归边界补充: BE-001TV-01 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants equivalence baseline and extraction plan；下一步: BE-001TV-02 root.contracts.core_ir.v4_contracts.schema_identity_constants extract_closeout。
- `markdown/06-milestones/v4.16.0/1122-root.contracts.core_ir.v4_contracts.schema_identity_constants.baseline_plan.md` - v4.16.0 BE-001TV-01 root.contracts.core_ir.v4_contracts.schema_identity_constants equivalence baseline and extraction plan
递归边界补充: BE-001TV-02 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants actual extraction complete；下一步: BE-001TV-03 root.contracts.core_ir.v4_contracts.schema_identity_constants single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1123-root.contracts.core_ir.v4_contracts.schema_identity_constants.extract_closeout.md` - v4.16.0 BE-001TV-02 root.contracts.core_ir.v4_contracts.schema_identity_constants actual extraction complete
- qrpc_core_ir/src/v4/schema_identity_constants.rs - Extracted qrpc-core-ir v4 schema identity constants, compat identifiers, and guard constants
递归边界补充: BE-001TV-03 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants single leaf closeout stops split；下一步: BE-001TW-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects backtest_artifact_contract。
- `markdown/06-milestones/v4.16.0/1124-root.contracts.core_ir.v4_contracts.schema_identity_constants.single_leaf_closeout.md` - v4.16.0 BE-001TV-03 root.contracts.core_ir.v4_contracts.schema_identity_constants single leaf closeout stops split
递归边界补充: BE-001TW-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects backtest_artifact_contract；下一步: BE-001TX-01 root.contracts.core_ir.v4_contracts.backtest_artifact_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1125-root.contracts.core_ir.v4_contracts.parent_residual_judgment.backtest_artifact_contract.md` - v4.16.0 BE-001TW-01 root.contracts.core_ir.v4_contracts parent residual judgment selects backtest_artifact_contract
递归边界补充: BE-001TX-01 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract equivalence baseline and extraction plan；下一步: BE-001TX-02 root.contracts.core_ir.v4_contracts.backtest_artifact_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1126-root.contracts.core_ir.v4_contracts.backtest_artifact_contract.baseline_plan.md` - v4.16.0 BE-001TX-01 root.contracts.core_ir.v4_contracts.backtest_artifact_contract equivalence baseline and extraction plan
递归边界补充: BE-001TX-02 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract actual extraction complete；下一步: BE-001TX-03 root.contracts.core_ir.v4_contracts.backtest_artifact_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1127-root.contracts.core_ir.v4_contracts.backtest_artifact_contract.extract_closeout.md` - v4.16.0 BE-001TX-02 root.contracts.core_ir.v4_contracts.backtest_artifact_contract actual extraction complete
递归边界补充: BE-001TX-03 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract single leaf closeout stops split；下一步: BE-001TY-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects machine_contract。
- `markdown/06-milestones/v4.16.0/1128-root.contracts.core_ir.v4_contracts.backtest_artifact_contract.single_leaf_closeout.md` - v4.16.0 BE-001TX-03 root.contracts.core_ir.v4_contracts.backtest_artifact_contract single leaf closeout stops split
递归边界补充: BE-001TY-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_contract；下一步: BE-001TZ-01 root.contracts.core_ir.v4_contracts.machine_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1129-root.contracts.core_ir.v4_contracts.parent_residual_judgment.machine_contract.md` - v4.16.0 BE-001TY-01 root.contracts.core_ir.v4_contracts parent residual judgment selects machine_contract
递归边界补充: BE-001TZ-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract equivalence baseline and extraction plan；下一步: BE-001TZ-02 root.contracts.core_ir.v4_contracts.machine_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1130-root.contracts.core_ir.v4_contracts.machine_contract.baseline_plan.md` - v4.16.0 BE-001TZ-01 root.contracts.core_ir.v4_contracts.machine_contract equivalence baseline and extraction plan
递归边界补充: BE-001TZ-02 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract actual extraction complete；下一步: BE-001TZ-03 root.contracts.core_ir.v4_contracts.machine_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1131-root.contracts.core_ir.v4_contracts.machine_contract.extract_closeout.md` - v4.16.0 BE-001TZ-02 root.contracts.core_ir.v4_contracts.machine_contract actual extraction complete
递归边界补充: BE-001TZ-03 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract single leaf closeout continues split；下一步: BE-001UA-01 root.contracts.core_ir.v4_contracts.machine_contract parent_residual_judgment selects static_validation。
- `markdown/06-milestones/v4.16.0/1132-root.contracts.core_ir.v4_contracts.machine_contract.single_leaf_closeout.md` - v4.16.0 BE-001TZ-03 root.contracts.core_ir.v4_contracts.machine_contract single leaf closeout continues split
递归边界补充: BE-001UA-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment selects static_validation；下一步: BE-001UB-01 root.contracts.core_ir.v4_contracts.machine_contract.static_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/1133-root.contracts.core_ir.v4_contracts.machine_contract.parent_residual_judgment.static_validation.md` - v4.16.0 BE-001UA-01 root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment selects static_validation
递归边界补充: BE-001UB-01 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation equivalence baseline and extraction plan；下一步: BE-001UB-02 root.contracts.core_ir.v4_contracts.machine_contract.static_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/1134-root.contracts.core_ir.v4_contracts.machine_contract.static_validation.baseline_plan.md` - v4.16.0 BE-001UB-01 root.contracts.core_ir.v4_contracts.machine_contract.static_validation equivalence baseline and extraction plan
递归边界补充: BE-001UB-02 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation actual extraction complete；下一步: BE-001UB-03 root.contracts.core_ir.v4_contracts.machine_contract.static_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1135-root.contracts.core_ir.v4_contracts.machine_contract.static_validation.extract_closeout.md` - v4.16.0 BE-001UB-02 root.contracts.core_ir.v4_contracts.machine_contract.static_validation actual extraction complete
递归边界补充: BE-001UB-03 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation single leaf closeout stops split；下一步: BE-001UC-01 root.contracts.core_ir.v4_contracts.machine_contract parent_residual_judgment closes parent。
- `markdown/06-milestones/v4.16.0/1136-root.contracts.core_ir.v4_contracts.machine_contract.static_validation.single_leaf_closeout.md` - v4.16.0 BE-001UB-03 root.contracts.core_ir.v4_contracts.machine_contract.static_validation single leaf closeout stops split
递归边界补充: BE-001UC-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment closes parent；下一步: BE-001UD-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects machine_graph_contract。
- `markdown/06-milestones/v4.16.0/1137-root.contracts.core_ir.v4_contracts.machine_contract.parent_residual_judgment.closes_parent.md` - v4.16.0 BE-001UC-01 root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment closes parent
递归边界补充: BE-001UD-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_graph_contract；下一步: BE-001UE-01 root.contracts.core_ir.v4_contracts.machine_graph_contract baseline_plan。
- `markdown/06-milestones/v4.16.0/1138-root.contracts.core_ir.v4_contracts.parent_residual_judgment.machine_graph_contract.md` - v4.16.0 BE-001UD-01 root.contracts.core_ir.v4_contracts parent residual judgment selects machine_graph_contract
递归边界补充: BE-001UE-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract equivalence baseline and extraction plan；下一步: BE-001UE-02 root.contracts.core_ir.v4_contracts.machine_graph_contract extract_closeout。
- `markdown/06-milestones/v4.16.0/1139-root.contracts.core_ir.v4_contracts.machine_graph_contract.baseline_plan.md` - v4.16.0 BE-001UE-01 root.contracts.core_ir.v4_contracts.machine_graph_contract equivalence baseline and extraction plan
递归边界补充: BE-001UE-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract actual extraction complete；下一步: BE-001UE-03 root.contracts.core_ir.v4_contracts.machine_graph_contract single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1140-root.contracts.core_ir.v4_contracts.machine_graph_contract.extract_closeout.md` - v4.16.0 BE-001UE-02 root.contracts.core_ir.v4_contracts.machine_graph_contract actual extraction complete
递归边界补充: BE-001UE-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract single leaf closeout continues split；下一步: BE-001UF-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment selects event_catalog。
- `markdown/06-milestones/v4.16.0/1141-root.contracts.core_ir.v4_contracts.machine_graph_contract.single_leaf_closeout.md` - v4.16.0 BE-001UE-03 root.contracts.core_ir.v4_contracts.machine_graph_contract single leaf closeout continues split
递归边界补充: BE-001UF-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects event_catalog；下一步: BE-001UG-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog baseline_plan。
- `markdown/06-milestones/v4.16.0/1142-root.contracts.core_ir.v4_contracts.machine_graph_contract.parent_residual_judgment.event_catalog.md` - v4.16.0 BE-001UF-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects event_catalog
递归边界补充: BE-001UG-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog equivalence baseline and extraction plan；下一步: BE-001UG-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog extract_closeout。
- `markdown/06-milestones/v4.16.0/1143-root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog.baseline_plan.md` - v4.16.0 BE-001UG-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog equivalence baseline and extraction plan
递归边界补充: BE-001UG-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog actual extraction complete；下一步: BE-001UG-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1144-root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog.extract_closeout.md` - v4.16.0 BE-001UG-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog actual extraction complete
递归边界补充: BE-001UG-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single leaf closeout stops split；下一步: BE-001UH-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment selects graph_static_validation。
- `markdown/06-milestones/v4.16.0/1145-root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog.single_leaf_closeout.md` - v4.16.0 BE-001UG-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single leaf closeout stops split
递归边界补充: BE-001UH-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects graph_static_validation；下一步: BE-001UI-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/1146-root.contracts.core_ir.v4_contracts.machine_graph_contract.parent_residual_judgment.graph_static_validation.md` - v4.16.0 BE-001UH-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects graph_static_validation
递归边界补充: BE-001UI-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation equivalence baseline and extraction plan；下一步: BE-001UI-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/1147-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.baseline_plan.md` - v4.16.0 BE-001UI-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation equivalence baseline and extraction plan
递归边界补充: BE-001UI-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation actual extraction complete；下一步: BE-001UI-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1148-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.extract_closeout.md` - v4.16.0 BE-001UI-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation actual extraction complete
递归边界补充: BE-001UI-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split；下一步: BE-001UJ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects risk_plane_validation。
- `markdown/06-milestones/v4.16.0/1149-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.single_leaf_closeout.md` - v4.16.0 BE-001UI-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split
递归边界补充: BE-001UJ-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects risk_plane_validation；下一步: BE-001UK-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/1150-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.parent_residual_judgment.risk_plane_validation.md` - v4.16.0 BE-001UJ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects risk_plane_validation
递归边界补充: BE-001UK-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation equivalence baseline and extraction plan；下一步: BE-001UK-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/1151-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation.baseline_plan.md` - v4.16.0 BE-001UK-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation equivalence baseline and extraction plan
递归边界补充: BE-001UK-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation actual extraction complete；下一步: BE-001UK-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1152-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation.extract_closeout.md` - v4.16.0 BE-001UK-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation actual extraction complete
递归边界补充: BE-001UK-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single leaf closeout stops split；下一步: BE-001UL-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects event_usage_validation。
- `markdown/06-milestones/v4.16.0/1153-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation.single_leaf_closeout.md` - v4.16.0 BE-001UK-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single leaf closeout stops split
递归边界补充: BE-001UL-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects event_usage_validation；下一步: BE-001UM-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/1154-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.parent_residual_judgment.event_usage_validation.md` - v4.16.0 BE-001UL-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects event_usage_validation
递归边界补充: BE-001UM-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation equivalence baseline and extraction plan；下一步: BE-001UM-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/1155-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.baseline_plan.md` - v4.16.0 BE-001UM-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation equivalence baseline and extraction plan
递归边界补充: BE-001UM-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation actual extraction complete；下一步: BE-001UM-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1156-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.extract_closeout.md` - v4.16.0 BE-001UM-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation actual extraction complete
递归边界补充: BE-001UM-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout continues split；下一步: BE-001UN-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent_residual_judgment selects event_party_validation。
- `markdown/06-milestones/v4.16.0/1157-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.single_leaf_closeout.md` - v4.16.0 BE-001UM-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout continues split
递归边界补充: BE-001UN-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_party_validation；下一步: BE-001UO-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation baseline_plan。
- `markdown/06-milestones/v4.16.0/1158-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.parent_residual_judgment.event_party_validation.md` - v4.16.0 BE-001UN-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_party_validation
递归边界补充: BE-001UO-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation equivalence baseline and extraction plan；下一步: BE-001UO-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation extract_closeout。
- `markdown/06-milestones/v4.16.0/1159-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation.baseline_plan.md` - v4.16.0 BE-001UO-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation equivalence baseline and extraction plan
递归边界补充: BE-001UO-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation actual extraction complete；下一步: BE-001UO-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single_leaf_closeout。
- `markdown/06-milestones/v4.16.0/1160-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation.extract_closeout.md` - v4.16.0 BE-001UO-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation actual extraction complete
递归边界补充: BE-001UO-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single leaf closeout stops split；下一步: BE-001UP-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent_residual_judgment selects event_reference_resolution。
- `markdown/06-milestones/v4.16.0/1161-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation.single_leaf_closeout.md` - v4.16.0 BE-001UO-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single leaf closeout stops split
递归边界补充: BE-001UP-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_reference_resolution；下一步: BE-001UQ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution baseline_plan。
- `markdown/06-milestones/v4.16.0/1162-root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.parent_residual_judgment.event_reference_resolution.md` - v4.16.0 BE-001UP-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_reference_resolution
