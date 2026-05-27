# 模块树

> 职责: 以白箱网络描述模块的输入、输出、关键 public 方法、父子关系和通信边界。
> 状态: v4.13.0 第一波白箱扩面中。后续重型变更必须逐步补齐受影响模块。

---

## 1. 模块树原则

1. 模块树是逻辑白箱网络，不是文件树。
2. 每个模块必须能落到真实文件。
3. 关键 public 方法必须登记。
4. 父模块是默认对外协调层。
5. 子模块横向直连默认禁止。
6. 发布态性能边必须登记为例外，不得污染开发态结构。

---

## 2. 白箱节点模板

```markdown
## 模块 ID: `domain.parent.child`

**层级路径**: `root.domain.parent.child`
**父模块**: `domain.parent`
**子模块**: `child_a`, `child_b`
**真实文件**:
- `path/to/file.rs`

**职责**:
一句话说明本模块存在的理由。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |

**父级通信规则**:
本模块对外必须经过哪个父模块、接口、事件或 adapter。

**允许调用的子模块**:
列出允许直接调用的下级模块或方法。

**禁止横向连接**:
列出不得直接调用的兄弟模块或跨域模块。

**状态与锁**:
涉及状态、事务、锁顺序、并发边界时填写。

**回归保护**:
修改本模块必须跑哪些测试或门禁。

**幻觉检查点**:
AI 提到本模块时，必须能指出真实文件、真实方法、真实测试；否则视为未证实。
```

---

## 3. 种子模块树

### 3.1 `system.entry`

**层级路径**: `root.system.entry`
**父模块**: `system`
**真实文件**:
- `start.bat`
- `start.ps1`
- `src/main.rs`
- `src/lib.rs`
- `src/system/mod.rs`
- `src/system/entry/mod.rs`
- `src/system/entry/backend_process.rs`
- `src-tauri/src/main.rs`

**职责**:
编排本地桌面应用、后端服务、前端开发服务和 Tauri 壳的启动边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `run_server` | 环境变量、CLI 参数、路由构建依赖 | Axum 服务或 CLI 输出 | `src/main.rs`、`quantpilot::run_server` | 不得绕过 `build_app_router` 注册路由 |

**父级通信规则**:
系统入口只负责启动和编排，不拥有业务能力真源。

**回归保护**:
`cargo check --workspace`；涉及启动脚本时执行对应 PowerShell 或批处理 dry-run。

### 3.1.1 `system.entry.backend_process`

**层级路径**: `root.system.entry.backend_process`
**父模块**: `system.entry`
**状态**: v4.16 system 抽离完成。public 启动入口和 API server 启动实现已迁入 system 模块，旧 crate 入口通过 re-export 兼容。
**真实文件**:
- `src/system/mod.rs`
- `src/system/entry/mod.rs`
- `src/system/entry/backend_process.rs`
- `src/lib.rs`
- `src/main.rs`

**职责**:
承载后端进程启动 public 入口、环境初始化、tracing 初始化、panic hook、CLI 分发、API server 启动、启动期中间件、后台观察任务、优雅关闭和关闭刷盘。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| CLI 参数 | OS process | `std::env::args()` | 不改变现有 `credential`、`v4-run`、`strategy-ir validate` 语义 |
| 环境变量 | `.env`、shell | `QUANTPILOT_*` | 不改变默认端口或 bind 规则 |
| 启动调用 | `src/main.rs` | `quantpilot::run_server()` | 不改二进制入口 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| API server 启动 | `run_api_server` | async result | 不拥有 API route owner |
| CLI 输出 | stdout/stderr | text | 不改变已有 CLI 输出语义 |
| 兼容 public 入口 | crate root | `pub use ...::run_server` | 不删除 `quantpilot::run_server` |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `run_server` | 环境变量、CLI 参数 | `anyhow::Result<()>` | `src/main.rs`、旧 crate public 入口 | 不得拥有 handler、route schema 或 runtime state |

**关键内部启动实现**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `run_api_server` | 存储目录、环境变量、AppState 工厂、router 构建器 | Axum 服务 | `run_server` | 不得拥有 handler、response schema 或 AppState 字段定义 |

**父级通信规则**:
`system.entry.backend_process` 只能通过 `run_api_server -> backend.interface_boundary -> build_app_router` 进入后端接口边界，不得直接横向改 handler、response schema 或状态所有权。

**回归保护**:
`cargo check -p quantpilot`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 system 已经抽离时，必须指出完成范围是 `system.entry.backend_process` 启动边界；`build_app_router` 仍属 `backend.interface_boundary`，`new_app_state` 仍属 `app_runtime_helpers`。

### 3.1.2 `system.entry.launch_scripts`

**层级路径**: `root.system.entry.launch_scripts`
**父模块**: `system.entry`
**状态**: v4.16 S1 单叶 closeout 完成。启动脚本入口已完成白箱登记，不改脚本语义，不继续细分。
**真实文件**:
- `start.bat`
- `start.ps1`

**职责**:
承载 Windows CMD 与 PowerShell 的本地桌面启动入口，负责设置开发模式、清理旧进程、构建后端、启动后端、等待 3000 端口和进入 Tauri dev。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start.bat` | Windows CMD shell | 本地开发桌面启动流程 | 开发者 | 不得改默认端口、启动顺序或用户调用方式 |
| `start.ps1` | PowerShell shell | 本地开发桌面启动流程 | 开发者 | 不得改默认端口、启动顺序或用户调用方式 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `QUANTPILOT_DEV=true` | shell env | 开发模式环境变量 | 启动脚本 | 不得改变默认开发模式语义 |
| `cargo build --bin quantpilot` | workspace manifest | `target\debug\quantpilot.exe` | 启动脚本 | 不得替代为未登记构建链 |
| `cargo tauri dev` | `src-tauri` workspace | Tauri dev runtime | 启动脚本 | 不得绕过 desktop shell 边界 |

**父级通信规则**:
`system.entry.launch_scripts` 只能通过脚本入口编排 `system.entry.backend_process` 和 `system.desktop_shell`，不得拥有后端 API、runtime state、capability 真源或 Tauri command 权限。

**回归保护**:
`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；脚本语义变化时补本地启动 smoke 或人工验收。

**幻觉检查点**:
AI 声称 S1 完成时，必须指出本批次没有修改 `start.bat` 或 `start.ps1`，只完成启动脚本入口等价 closeout。

### 3.1.3 `system.desktop_shell.tauri_config`

**层级路径**: `root.system.desktop_shell.tauri_config`
**父模块**: `system.desktop_shell`
**状态**: v4.16 S4 单叶 closeout 完成。Tauri config 和 capability allowlist 已完成白箱登记，不改配置语义，不继续细分。
**真实文件**:
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`

**职责**:
承载 Tauri 桌面壳配置、窗口配置、CSP、bundle 配置和 capability allowlist。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `src-tauri/tauri.conf.json` | Tauri CLI/config loader | 桌面应用配置、CSP、bundle 配置 | Tauri runtime/build | 不得放宽 CSP、改 app identifier 或改窗口语义 |
| `src-tauri/capabilities/default.json` | Tauri capability loader | 默认窗口权限 allowlist | Tauri runtime | 不得新增未登记权限 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `devUrl` / `beforeDevCommand` / `beforeBuildCommand` | Tauri build config | dev/build 命令链 | Tauri CLI | 不得绕过 desktop build/dev scripts 叶子 |
| CSP | 本地 dev/API/websocket 连接 | 浏览器安全策略 | Tauri runtime | 不得把 CSP 变更混入无关抽离 |
| capability permissions | default window | Tauri API permission | Tauri runtime | 不得把权限声明当业务 capability 真源 |

**父级通信规则**:
`system.desktop_shell.tauri_config` 只为 `system.desktop_shell` 提供桌面壳配置，不拥有前端 capability projection、后端 API 权限语义、runtime state 或业务 supported/unsupported 声明。

**回归保护**:
JSON parse；`cargo check -p quantpilot-tauri`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；涉及 CSP、窗口或权限变更时补 Tauri 启动 smoke 或人工验收。

**幻觉检查点**:
AI 声称 S4 完成时，必须指出本批次没有修改 `src-tauri/tauri.conf.json` 或 `src-tauri/capabilities/default.json`，只完成 Tauri config 等价 closeout。

### 3.1.4 `system.runtime_profile.config_examples`

**层级路径**: `root.system.runtime_profile.config_examples`
**父模块**: `system.runtime_profile`
**状态**: v4.16 S10 单叶 closeout 完成。运行配置样例和 strategy_ir schema/example 已完成白箱登记，不改样例语义，不继续细分。
**真实文件**:
- `.env.example`
- `config/runtime_protocol.example.yaml`
- `config/strategy_ir.v0.schema.json`
- `config/strategy_ir.v0.example.json`

**职责**:
承载环境变量模板、runtime protocol 示例和 strategy_ir v0 schema/example，用于开发者理解运行配置和协议样例。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `.env.example` | 开发者复制/阅读 | 环境变量模板 | 开发者、文档 | 不得当作真实运行配置 |
| `config/runtime_protocol.example.yaml` | 开发者阅读/示例引用 | runtime protocol 示例 | 开发者、文档 | 不得当作 runtime 行为真源 |
| `config/strategy_ir.v0.schema.json` | schema consumer | strategy_ir v0 schema | 工具、文档 | 不得无契约验证改字段 |
| `config/strategy_ir.v0.example.json` | example consumer | strategy_ir v0 example | 工具、文档 | 不得当作编译器真源 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `QUANTPILOT_*` 示例 | 环境变量键 | 配置模板 | 开发者 | 不得改变默认配置语义 |
| runtime protocol 示例结构 | generators/agents/global_risk/runtime_mode | 协议样例 | 开发者 | 不得冒充真实 runtime state |
| strategy_ir v0 schema/example | JSON schema/example | 合约样例 | 工具、文档 | 不得绕过 contracts owner |

**父级通信规则**:
`system.runtime_profile.config_examples` 只提供配置样例和 schema/example 入口，不拥有 runtime 行为真源、编译器真源、后端 capability 真源或执行端状态。

**回归保护**:
JSON parse；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；涉及 schema 或 runtime protocol 变化时补契约验证。

**幻觉检查点**:
AI 声称 S10 完成时，必须指出本批次没有修改 `.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json` 或 `config/strategy_ir.v0.example.json`，只完成配置样例等价 closeout。

### 3.2 `backend.router`

**层级路径**: `root.backend.router`
**父模块**: `backend`
**真实文件**:
- `src/app_router.rs`

**职责**:
集中注册后端 HTTP 路由和 SPA fallback。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `build_app_router` | 后端共享状态与 handler 模块 | Axum Router | `run_server`、测试入口 | 新增路由不得只写 handler 而不注册 OpenAPI 和测试 |

**父级通信规则**:
路由层只分发请求，不自行创造业务语义。

**回归保护**:
`cargo test -p quantpilot tests_backend`；涉及 API 变更时执行 OpenAPI route diff 检查。

### 3.3 `backend.capability`

**层级路径**: `root.backend.capability`
**父模块**: `backend`
**真实文件**:
- `src/capability_api.rs`
- `frontend/src/capabilities/capabilityProjection.js`
- `frontend/src/capabilities/capabilityGovernance.js`

**职责**:
提供用户可见能力、工作区入口、工具栏 action 和模块暴露的后端真源及前端投影。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `GET /api/capabilities` | 当前后端能力状态 | capability snapshot | 前端 projection、治理检查 | 前端不得用静态数组替代真源 |

**父级通信规则**:
能力判断由后端拥有，前端只做投影、排序、标签和禁用原因展示。

**回归保护**:
`powershell tools/check-capability-governance.ps1`；相关前端 capability 测试。

### 3.4 `backend.strategy_config`

**层级路径**: `root.backend.strategy_config`
**父模块**: `backend`
**真实文件**:
- `src/strategy_config_api.rs`
- `tests/api_ai_proposal.rs`

**职责**:
聚合 v4 策略配置 artifact、preflight、diff、AI proposal 配置域绑定和证据边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `/api/v1/strategy-config/artifact` | 策略输入、capability、编译证据 | v4 strategy config artifact | 前端配置台、导出路径 | 不得绕过 QS 编译路径 |
| `/api/v1/strategy-config/preflight` | artifact 或策略输入 | readiness、runtime boundary、拒绝原因 | 前端、执行端启动前核验 | 不得把 unsupported 静默降级 |
| `/api/v1/strategy-config/diff` | 左右 artifact 或配置草稿 | domain 级差异 | 版本历史、配置台 | 不得以裸 JSON diff 替代用户语义 |

**父级通信规则**:
必须通过后端 API 和 capability 真源对外提供配置状态。

**回归保护**:
`cargo test -p quantpilot strategy_config`；`powershell tools/check-openapi-route-diff.ps1`。

### 3.5 `frontend.workspace`

**层级路径**: `root.frontend.workspace`
**父模块**: `frontend`
**真实文件**:
- `frontend/src/pages/StrategyWorkspacePage.jsx`
- `frontend/src/pages/StrategyConfigCockpit.jsx`
- `frontend/src/hooks/useStrategyWorkspacePageData.js`

**职责**:
承载策略工作区、配置台、源码、诊断、回测、运行监控和版本历史等用户主路径。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `StrategyWorkspacePage` | 路由参数、workspace 数据、capability projection | 工作区界面 | React router | 不得直接决定能力支持 |
| `StrategyConfigCockpit` | strategy config view、preflight、i18n 文案 | 配置域状态和证据展示 | 工作区页面 | 不得显示账户、实盘、研究级回测 CTA |

**父级通信规则**:
工作区入口状态来自 capability projection；页面组件不得维护独立支持判断。

**回归保护**:
`cd frontend && npm run test -- --run src/pages/StrategyConfigCockpit.test.jsx`；涉及路由时跑相关页面测试。

---

## 4. v4.13 父模块分类

| 父模块 | 层级路径 | 职责 | 禁止事项 |
| --- | --- | --- | --- |
| `system` | `root.system` | 启动、进程编排、Tauri 壳和本地运行入口 | 不拥有业务能力真源 |
| `backend` | `root.backend` | 后端 API、编译、运行、持久化、能力真源和治理判断 | 不让前端静态判断替代后端真源 |
| `frontend` | `root.frontend` | 用户工作区、策略中心、配置台、能力投影和运行证据展示 | 不直接创造 supported/restricted/unsupported 结论 |
| `executor` | `root.executor` | 独立执行端状态、runner、迁移包、行情连接、凭证和审计 | 不绕过 preflight、Risk Plane 和执行模式边界 |
| `contracts` | `root.contracts` | OpenAPI、RFC、artifact、QS/Core IR 和事件契约 | 不在无迁移方案时改数据结构 |
| `docs` | `root.docs` | 三矩阵、全量树、GP、超级规范化、里程碑和治理索引 | 不删除旧主干，不让新文档成为孤岛 |

---

## 5. v4.13 第一波白箱节点

### 5.0 `backend.interface_boundary`

**层级路径**: `root.backend.interface_boundary`
**父模块**: `backend`
**状态**: v4.16 BE-001 抽离候选大模块。真实代码仍分布在既有文件中，本节点只登记后端接口边界的父级白箱。
**真实文件**:
- `src/app_router.rs`
- `src/capability_api.rs`
- `src/strategy_config_api.rs`
- `src/runtime/mod.rs`
- `src/graph_api.rs`
- `src/graph_quantscript_api.rs`
- `src/compile_api.rs`

**职责**:
作为后端接口边界的大模块，先管理 router、route registration、API facade、旧 handler 保留和 response schema 冻结。后续小模块抽离必须先挂到本父级边界下，再进入 capability、strategy config、runtime、graph/compile 等子模块。

**抽离策略**:
先抽一个大模块，再在大模块里抽小模块。BE-001 只建立 `backend.interface_boundary` 父级边界，小模块抽离按后续批次逐个推进。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| HTTP request | 前端、测试、CLI | Axum request | 不改变现有 `/api/*` 入口语义 |
| AppState | 后端启动入口 | shared app state | 不迁移状态所有权 |
| route registration | 后端模块 | `Router<AppState>` | 不删除旧 handler |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| Axum Router | `run_server`、测试入口 | Router | `build_app_router` 仍是父入口 |
| API response | 前端、测试 | JSON / SSE / status code | 不改 response schema |
| route owner map | 后续抽离提案 | 文档登记 | 不替代真实代码证据 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `build_app_router` | `AppState` | Axum Router | `run_server`、测试入口 | 不得跳过父级 router |
| `get_capabilities` | backend state | capability snapshot | 前端 capability projection | 不得硬编码替代真源 |
| `register_strategy_config_routes` | Axum Router | strategy config routes | `build_app_router` | 不得改变 preflight 语义 |
| `register_runtime_routes` | Axum Router | runtime routes | `build_app_router` | 不得迁移 runtime 状态所有权 |
| `register_graph_routes` | Axum Router | graph routes | `build_app_router` | 不得绕过版本记录 |
| `register_compile_routes` | Axum Router | compile routes | `build_app_router` | 不得把 strategy_ir 当运行真源 |

**父级通信规则**:
所有后端接口抽离必须先经过 `backend.interface_boundary` 父级边界。子模块不得直接互相横向改 route、handler、state owner 或 response schema。

**允许调用的子模块**:
`backend.capability`、`backend.strategy_config`、`backend.runtime`、`backend.graph_compile`。

**禁止横向连接**:
不得让 `backend.runtime` 直接改 `backend.graph_compile` route owner；不得让前端绕过 API 读取后端内部文件；不得让执行端状态直接并入后端接口边界。

**状态与锁**:
BE-001 不迁移状态所有权，不改变 AppState、runtime state、executor state、锁顺序或事务边界。

**回归保护**:
`cargo test api_run`；`cargo test api_backtest`；`cargo test api_graph_versions`；`cargo test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称后端接口边界已经抽离时，必须指出 BE-001、`build_app_router`、对应 `register_*_routes`、旧 handler 保留方式和回退点。

### 5.1 `backend.runtime`

**层级路径**: `root.backend.runtime`
**父模块**: `backend`
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime_persistence.rs`
- `src/runtime_event_projection.rs`
- `src/runtime_validation.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_diagnostics.rs`
- `src/backtest_artifacts.rs`

**职责**:
承载 runtime run、v4 run、backtest、事件流、持久化记录、AI proposal 审批和运行证据输出。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| 运行请求 | 前端工作区、API 调用 | JSON request | 必须通过 runtime validation |
| v4 machine graph | 编译链或策略图 | MachineGraph / runtime config | 不得绕过 QS/Core IR 约束 |
| backtest request | 研究路径、配置台 | Backtest request | evidence 与 artifact 必须可追踪 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| run record | 前端、持久化层 | runtime record | 必须保留 run id 与事件锚点 |
| backtest artifact | 前端、文件系统 | artifact views | 不得把 transient 记录伪装成正式持久化 |
| runtime event | SSE、证据面板 | structured event | 不得静默丢弃阻断错误 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_runtime_routes` | Axum Router | Runtime routes | `build_app_router` | 不得在路由外新增 runtime API |
| `/api/runtime/backtest` | backtest request | backtest record/artifact | 前端研究路径 | 不得绕过 artifact 生成 |
| `/api/runtime/v4/run` | v4 graph/run request | v4 run record | 前端、测试 | 不得绕过 Risk Plane 与 capability |
| `/api/runtime/runs/:run_id/events` | run id | SSE event stream | 前端运行面板 | 不得输出未结构化事件 |
| `build_backtest_artifact_views` | backtest record | artifact views | runtime persistence | 不得生成无证据锚点摘要 |

**父级通信规则**:
runtime 对外必须经过 `backend.router` 注册的 HTTP API、事件流或持久化接口；不得由前端直接读取内部文件推断运行状态。

**允许调用的子模块**:
`runtime_persistence`、`runtime_validation`、`runtime_event_projection`、`backtest_artifacts`。

**禁止横向连接**:
不得直接调用 `executor.runner` 的内部状态；执行端交互必须经迁移包、执行端 API 或 runtime evidence。

**状态与锁**:
涉及运行记录、事件流、backtest artifact 和 transient spill 时，必须保留状态归属和清理边界。

**回归保护**:
`cargo test api_run`；`cargo test api_backtest`；涉及 v4 evidence 时跑 `cargo test api_evidence_contract`。

**幻觉检查点**:
AI 声称 runtime 支持新能力时，必须指出真实路由、record/artifact 字段和测试。

### 5.2 `backend.graph_compile`

**层级路径**: `root.backend.graph_compile`
**父模块**: `backend`
**真实文件**:
- `src/graph_api.rs`
- `src/graph_quantscript_api.rs`
- `src/graph_version_compare.rs`
- `src/compile_api.rs`
- `src/compile_artifact_builders.rs`
- `src/compile_diagnostics.rs`

**职责**:
管理策略图保存、加载、版本比较、QuantScript 入口、runtime compile 和诊断输出。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| strategy graph | 前端工作区 | graph JSON | 必须保留 graph id 与版本上下文 |
| QuantScript source | QS 编辑器 | text source | 不得直接执行主机代码 |
| compile request | 前端、测试 | runtime compile request | runtime compile 是真实数据源 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| graph version | 持久化层、前端 | graph metadata | 不得覆盖当前草稿而无版本记录 |
| compile summary | 前端、runtime | structured summary | strategy_ir 不取代 runtime compile |
| diagnostics | 前端诊断队列 | structured diagnostics | 不得降级为纯文本错误 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_graph_routes` | Axum Router | graph routes | `build_app_router` | 不得跳过审计与版本路径 |
| `register_graph_quantscript_routes` | Axum Router | QS graph routes | `build_app_router` | 不得允许任意主机代码 |
| `register_compile_routes` | Axum Router | compile routes | `build_app_router` | 不得把 strategy_ir 当运行真源 |
| `/api/runtime/compile` | graph/QS input | compile summary | 前端、测试 | 不得返回无 diagnostics 的失败 |

**父级通信规则**:
graph 和 compile 必须通过后端 API 与编译链契约对外通信；前端只消费 compile summary 和 diagnostics。

**回归保护**:
`cargo test api_graph_versions`；`cargo test quantscript_real_strategy_authoring`；涉及 compile 时跑相关 compile/graph 测试。

**幻觉检查点**:
任何“编译链已支持”的结论必须同时指出 graph route、compile route 和诊断测试。

### 5.3 `backend.storage_security`

**层级路径**: `root.backend.storage_security`
**父模块**: `backend`
**真实文件**:
- `src/storage_lifecycle.rs`
- `src/credential_vault.rs`
- `src/credential_api.rs`
- `src/safe_log.rs`
- `src/auth/mod.rs`
- `src/auth_middleware.rs`

**职责**:
管理存储生命周期、凭证保险库、安全日志清洗、本地会话边界和 API 守卫。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `CredentialVault::load` | 本地存储路径 | vault handle | credential API、CLI | 不得明文落盘 secret |
| `CredentialVault::set_service` | service fields | encrypted record | credential API | 不得把密钥写入日志 |
| `persist_with_ttl` | path、bytes、lifecycle | persisted file | runtime/storage callers | 不得跳过目录同步 |
| `ensure_storage_quota` | storage root、write size | quota result | 写入路径 | 不得失败后继续写 |
| `sanitize_secrets` | log text | redacted text | logging | 不得返回未清洗密钥 |

**父级通信规则**:
存储和凭证能力只能通过后端 API、CLI 命令或明确的 storage helper 使用；业务模块不得私自拼路径写敏感数据。

**状态与锁**:
涉及原子写、目录同步、TTL 清理、quota 检查和密钥清洗顺序。

**回归保护**:
`cargo test credential`；`cargo test storage_lifecycle`；涉及日志时复核 safe log 测试。

**幻觉检查点**:
AI 声称“安全存储已覆盖”时，必须指出 vault、storage lifecycle、日志清洗和测试证据。

### 5.4 `frontend.strategy_hub`

**层级路径**: `root.frontend.strategy_hub`
**父模块**: `frontend`
**真实文件**:
- `frontend/src/pages/StrategyHubPage.jsx`
- `frontend/src/pages/StrategyHubHeroSection.jsx`
- `frontend/src/pages/StrategyHubBodySection.jsx`
- `frontend/src/pages/StrategyHubRosterSection.jsx`
- `frontend/src/pages/StrategyHubInspectorSection.jsx`
- `frontend/src/hooks/useStrategyDirectoryModel.js`
- `frontend/src/hooks/useStrategyHubBodyData.js`
- `frontend/src/hooks/useStrategyHubRosterData.js`
- `frontend/src/hooks/useStrategyHubInspectorData.js`
- `frontend/src/utils/strategyHubRosterProjection.js`
- `frontend/src/utils/strategyHubInspectorProjection.js`

**职责**:
提供策略中心总览、策略清单、活动面板、当前策略驾驶舱和工作区入口。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `StrategyHubPage` | store state、router | 策略中心页面 | React router | 不得替代工作区执行能力判断 |
| `useStrategyDirectoryModel` | graph store、runtime history | hub model | `StrategyHubPage` | 不得把 fallback graph 当真实策略文件 |
| `projectStrategyHubRosterRows` | hub model | roster rows | roster section | 不得显示 unsupported 能力为可用 |
| `projectStrategyHubInspectorOverview` | selected strategy | inspector overview | inspector section | 不得创造后端没有的 evidence |

**父级通信规则**:
策略中心只做总览和入口分流；进入策略细节必须通过 `frontend.workspace` 或后端 API。

**禁止横向连接**:
不得直接写 runtime 状态；不得跳过 workspace/action bar 触发运行。

**回归保护**:
`cd frontend && npm run test -- --run src/pages/StrategyHubPage.test.jsx src/pages/StrategyHubRosterTableSection.test.jsx`。

**幻觉检查点**:
AI 提到策略中心支持新管理能力时，必须指出 projection、页面组件和测试。

### 5.5 `frontend.capability_projection`

**层级路径**: `root.frontend.capability_projection`
**父模块**: `frontend`
**真实文件**:
- `frontend/src/capabilities/capabilityProjection.js`
- `frontend/src/capabilities/capabilityGovernance.js`
- `frontend/src/capabilities/supportMatrix.js`

**职责**:
把后端 capability 真源投影为工作区入口、工具栏 action、支持矩阵和治理展示。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `projectWorkspaceSurfaces` | capabilities | workspace surfaces | 工作区页面 | 不得硬编码替代后端真源 |
| `projectUiActions` | capability snapshot | UI actions | 工具栏、工作区 | 不得展示 unsupported CTA |
| `getCapabilityBoundaryIssues` | capabilities | boundary issues | 治理检查 | 不得吞掉能力漂移 |
| `buildCapabilityContext` | capabilities | capability context | 支持矩阵 | 不得伪造 supported |

**父级通信规则**:
只消费 `/api/capabilities` 与 support matrix，不拥有业务真源。

**回归保护**:
`cd frontend && npm run test -- --run src/capabilities/capabilityProjection.test.js src/capabilities/supportMatrix.test.js src/capabilities/capabilityGovernance.test.js`；`powershell tools/check-capability-governance.ps1`。

**幻觉检查点**:
任何能力状态声明必须能回到后端 capability、support matrix 和治理注册表。

### 5.6 `frontend.runtime_panels`

**层级路径**: `root.frontend.runtime_panels`
**父模块**: `frontend`
**真实文件**:
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/RuntimeMutationPanel.jsx`
- `frontend/src/components/RuntimeReportPanel.jsx`
- `frontend/src/components/V4RuntimeEvidencePanel.jsx`
- `frontend/src/utils/runtimeDiagnosticsProjection.js`
- `frontend/src/utils/runtimeTimeline.js`
- `frontend/src/utils/runtimeMutation.js`
- `frontend/src/utils/runtimeAiProposal.js`
- `frontend/src/utils/v4RuntimeEvidence.js`

**职责**:
展示运行事件、运行诊断、AI proposal、mutation、报告和 v4 runtime evidence。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `EventStreamPanel` | runtime events | event UI | 工作区 | 不得隐藏阻断事件 |
| `RuntimeDiagnosticsPanel` | graph/runtime/selected node | diagnostics UI | 工作区 | 不得自行判断能力支持 |
| `buildRuntimeDiagnosticsProjection` | graph、runtime | diagnostics projection | panels/tests | 不得丢失 source anchor |
| `buildRuntimeTimelineItemsFromEvents` | runtime events | timeline items | event panels | 不得改变事件语义 |
| `buildV4RuntimeEvidenceProjection` | evidence source | v4 evidence view | evidence panel | 不得把缺失 evidence 写成通过 |

**父级通信规则**:
运行面板只投影 runtime store 与后端事件；运行状态变更必须经 workspace action、runtime API 或审批流。

**回归保护**:
`cd frontend && npm run test -- --run src/components/EventStreamPanel.layout.test.jsx src/components/RuntimeDiagnosticsPanel.test.jsx src/components/V4RuntimeEvidencePanel.test.jsx src/utils/runtimeTimeline.test.js`。

**幻觉检查点**:
AI 声称运行证据存在时，必须指出 event source、projection util 和对应面板测试。

### 5.7 `executor.state`

**层级路径**: `root.executor.state`
**父模块**: `executor`
**真实文件**:
- `src-executor/executor_state.rs`
- `src-executor/audit_log.rs`
- `src-executor/api_guard.rs`

**职责**:
管理执行端策略状态、执行模式、SSE lag 计数、持久化状态、审计日志和 API 守卫。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `ExecutorState::load_default_or_new` | storage path/env | shared executor state | executor main | 不得忽略损坏状态恢复 |
| `ExecutorState::register` | active strategy | persisted state | migration API、runner | 不得绕过 persist |
| `ExecutorState::set_mode` | execution mode | current mode | executor API | 不得绕过 mode boundary |
| `AuditLog::append` | audit entry | audit file append | executor API | 不得写入 secret |
| `api_guard_middleware` | HTTP request | guarded request/result | executor router | 不得默认开放危险 API |

**父级通信规则**:
执行端状态只能经 executor API、migration API 和 runner pool 变化；后端不得直接修改执行端文件。

**状态与锁**:
涉及 `RwLock` 状态、原子持久化、审计 append 和 API guard 顺序。

**回归保护**:
`cargo test -p quantpilot --bin executor executor_state`；涉及 API guard 时跑 executor 相关测试。

**幻觉检查点**:
AI 提到执行端模式或状态时，必须指出 `ExecutorState` 方法和 API 路由。

### 5.8 `executor.runner`

**层级路径**: `root.executor.runner`
**父模块**: `executor`
**真实文件**:
- `src-executor/live_runner.rs`
- `src-executor/kline_buffer.rs`
- `src-executor/ws_client.rs`
- `src-executor/okx_rest.rs`
- `src-executor/migration_api.rs`

**职责**:
管理 live/v4 runner、行情事件、K 线缓冲、OKX REST/testnet 边界和策略迁移包加载。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `RunnerPool::register` | active strategy | runner instance | migration API、executor main | 不得跳过 package verification |
| `RunnerPool::broadcast_ws_event` | ws event | runner state update | ws feed | 不得跨 runner 写状态 |
| `V4Runner::from_strategy` | active strategy、broadcast sender | v4 runner | runner pool | 不得缺少 v4 graph evidence |
| `load_strategy` | executor state、strategy package | registered strategy | migration API | 不得忽略签名/编译证明 |
| `place_order_with_profile` | OKX profile/order | REST result | executor order path | 不得绕过 execution mode |

**父级通信规则**:
runner 只接受 executor state、migration package、ws event 和明确 API 命令；不得被后端或前端直接横向调用。

**禁止横向连接**:
不得直接访问 `backend.runtime` 内部状态；性能优化必须通过发布过渡协议登记。

**回归保护**:
`cargo test -p quantpilot --bin executor live_runner`；`cargo test -p quantpilot --bin executor migration_api`。

**幻觉检查点**:
AI 声称执行端已能真实下单时，必须指出 execution mode、OKX profile、Risk Plane 和测试证据。

### 5.9 `docs.matrix_governance`

**层级路径**: `root.docs.matrix_governance`
**父模块**: `docs`
**真实文件**:
- `markdown/00-matrix-governance/README.md`
- `markdown/00-matrix-governance/process-matrix.md`
- `markdown/00-matrix-governance/standard-matrix.md`
- `markdown/00-matrix-governance/guidance-matrix.md`
- `markdown/00-matrix-governance/module-tree.md`
- `markdown/00-matrix-governance/proposal-flow.md`
- `markdown/00-matrix-governance/proposal-examples.md`
- `markdown/00-matrix-governance/release-transition-protocol.md`
- `markdown/00-matrix-governance/landing-roadmap.md`
- `markdown/06-milestones/v4.16.0/01-规划方案.md`
- `markdown/06-milestones/v4.16.0/02-落地记录.md`
- `markdown/06-milestones/v4.16.0/03-后端抽离登记.md`
- `markdown/06-milestones/v4.16.0/04-前端抽离登记.md`
- `markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md`
- `markdown/06-milestones/v4.16.0/06-后端接口边界首批抽离方案.md`
- `markdown/06-milestones/v4.16.0/07-顶层大模块统计.md`
- `markdown/06-milestones/v4.16.0/08-system大模块分层统计.md`
- `markdown/06-milestones/v4.16.0/09-system.entry首批抽离记录.md`
- `markdown/06-milestones/v4.16.0/10-system抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/11-system抽离经验回填.md`
- `markdown/06-milestones/v4.16.0/12-system十叶模块等价基线.md`
- `markdown/06-milestones/v4.16.0/13-递归模块化全局根流程.md`
- `markdown/06-milestones/v4.16.0/14-system.entry.launch_scripts单叶closeout.md`
- `markdown/06-milestones/v4.16.0/15-system.desktop_shell.tauri_config单叶closeout.md`
- `markdown/06-milestones/v4.16.0/16-system.runtime_profile.config_examples单叶closeout.md`

**职责**:
作为三矩阵治理控制面，定义提案、判档、父子通信、引导坐标、模块树和发布过渡协议。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `markdown/00-matrix-governance/proposal-flow.md` 提案模板 | 变更意图 | 提案状态机 | 所有开发者、AI 辅助流程 | 不得跳过适配性校验和方案优化 |
| `markdown/00-matrix-governance/proposal-examples.md` 三档样例 | 新开发者学习 | 轻量/标准/重型样例 | 所有开发者、AI 辅助流程 | 不得把样例当真实提案证据 |
| `markdown/00-matrix-governance/guidance-matrix.md` 引导坐标 | 需求、模块、文件 | 全量树和模块树定位 | 重型变更 | 不得找不到父模块仍继续 |
| `markdown/00-matrix-governance/module-tree.md` 白箱节点 | 模块事实 | 输入输出、public 方法、边界 | 重型变更 | 不得登记虚构模块 |
| `markdown/00-matrix-governance/release-transition-protocol.md` 发布过渡协议 | 开发者显式声明 | 横向连接例外方案 | 发布过渡提案 | AI 不得主动触发 |
| `markdown/06-milestones/v4.16.0/02-落地记录.md` 抽离控制面 | v4.16 工作线 | 落地状态、决策项、禁止事项 | 后续抽离提案 | 不得宣称整理或重构已完成 |
| `markdown/06-milestones/v4.16.0/03-后端抽离登记.md` 后端抽离登记 | 后端候选 | 父模块、public 方法、兼容桥、等价证据 | 后端抽离批次 | 不得切换主 API 或删除旧 handler |
| `markdown/06-milestones/v4.16.0/04-前端抽离登记.md` 前端抽离登记 | 前端候选 | 页面/store 边界、UI 对照、暂停条件 | 前端抽离批次 | 不得借抽离做 UX 重构 |
| `markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md` 测试资产汰换 | 旧测试路径 | 废弃候选、替代证据、风险窗口 | 测试汰换批次 | 不得静默删除测试程序 |
| `markdown/06-milestones/v4.16.0/06-后端接口边界首批抽离方案.md` BE-001 | 开发者决策 | 后端 router/API/facade 边界 | 后端接口抽离批次 | 不得迁移状态所有权 |
| `markdown/06-milestones/v4.16.0/07-顶层大模块统计.md` 顶层统计 | 模块树与 repo 文件 | 顶层大模块、白箱子节点、物理规模 | 后续大模块选择 | 不得把未覆盖缺口伪装成已完成 |
| `markdown/06-milestones/v4.16.0/08-system大模块分层统计.md` system 分层 | `root.system` | 3 层、10 个叶子模块、BE-001 关系 | system 抽离批次 | 不得把启动编排当业务能力真源 |
| `markdown/06-milestones/v4.16.0/09-system.entry首批抽离记录.md` system 试水 | `system.entry.backend_process` | public 启动入口、兼容桥、等价证据 | system 抽离批次 | 不得宣称 system 全量抽离完成 |
| `markdown/06-milestones/v4.16.0/10-system抽离完成记录.md` system 完成 | `system.entry.backend_process` | `run_server`、`run_api_server`、启动期 helper、完成边界 | system 抽离批次 | 不得宣称整理或重构完成 |
| `markdown/06-milestones/v4.16.0/11-system抽离经验回填.md` 抽离经验回填 | 后续抽离候选 | public/内部实现分类、owner 复核、未迁移边界 | 后续抽离批次 | 不得把内部 helper 误写成 public API |
| `markdown/06-milestones/v4.16.0/12-system十叶模块等价基线.md` system 十叶等价基线 | `root.system` 10 叶子 | 等价证据、继续抽离状态、暂停点 | system 后续单叶抽离 | 不得一次性推进 10 叶抽离 |
| `markdown/06-milestones/v4.16.0/13-递归模块化全局根流程.md` 递归模块化流程 | 六大顶层模块 | 顶层模块、叶子抽离、叶子整理、细分判断、全局根 | 全量模块树推进 | 不得无停止条件地继续细分 |
| `markdown/06-milestones/v4.16.0/14-system.entry.launch_scripts单叶closeout.md` S1 closeout | `system.entry.launch_scripts` | `start.bat`、`start.ps1`、启动脚本等价证据 | system 单叶 closeout | 不得改脚本语义 |
| `markdown/06-milestones/v4.16.0/15-system.desktop_shell.tauri_config单叶closeout.md` S4 closeout | `system.desktop_shell.tauri_config` | Tauri config、CSP、capability allowlist 等价证据 | system 单叶 closeout | 不得改 CSP、窗口或权限语义 |
| `markdown/06-milestones/v4.16.0/16-system.runtime_profile.config_examples单叶closeout.md` S10 closeout | `system.runtime_profile.config_examples` | 环境变量、runtime protocol、strategy_ir schema/example 等价证据 | system 单叶 closeout | 不得把样例当 runtime 真源 |

**父级通信规则**:
文档治理变更必须经三矩阵自身判档。改变规则含义时直接重型。

**回归保护**:
`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 引用治理规则时，必须指出具体矩阵、文件和章节；不能只说“按规范”。

### 5.10 `docs.feature_tree`

**层级路径**: `root.docs.feature_tree`
**父模块**: `docs`
**真实文件**:
- `markdown/10-overview/overview-full-feature-tree.md`
- `tools/check-full-feature-tree.ps1`

**职责**:
维护全量树物理文件地图，确保 active 文件、路径引用和文档入口不漂移。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `markdown/10-overview/overview-full-feature-tree.md` active 文件索引 | 仓库文件结构 | 全量树 | 所有变更 | 不得漏掉新增 active 文件 |
| `tools/check-full-feature-tree.ps1` | repo tree | path coverage result | closeout、人工验证 | 不得忽略 explicit path missing |

**父级通信规则**:
全量树回答“项目有什么”；模块通信和 public 方法归 `docs.matrix_governance` 的模块树管理。

**回归保护**:
`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称文件存在或路径有效时，必须能通过全量树或实际文件检查证实。
