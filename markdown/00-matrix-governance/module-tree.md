# 模块树

> 职责: 以白箱网络描述模块的输入、输出、关键 public 方法、父子关系和通信边界。
> 状态: v4.16.0 模块化抽离白箱扩面中。后续重型变更必须逐步补齐受影响模块。

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

### 3.0 `system`

**层级路径**: `root.system`
**父模块**: `root`
**子模块**: `system.entry`、`system.desktop_shell`、`system.build_delivery`、`system.runtime_profile`
**状态**: v4.16 顶层阶段性 closeout 完成。S1-S10 已完成 closeout 或静态 closeout；整理、重构、发布验收和 Docker runtime smoke 仍未启动。
**真实文件**:
- `src/system/mod.rs`
- `src/system/entry/mod.rs`
- `src/system/entry/backend_process.rs`
- `src/main.rs`
- `src/lib.rs`
- `src-tauri/src/main.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/build.rs`
- `Dockerfile`
- `docker-compose.yml`
- `nginx.conf`
- `.env.example`

**职责**:
承载系统级启动、进程编排、桌面壳、构建交付、容器代理和运行配置样例的顶层父模块。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `quantpilot::run_server` | CLI / 环境变量 | 后端进程启动 | `src/main.rs`、旧 crate public 入口 | 不得绕过 `system.entry.backend_process` 或后端接口边界 |
| 启动脚本入口 | shell / Windows CMD / PowerShell | 本地桌面开发启动 | 开发者 | 不得拥有业务能力真源 |
| Tauri `main` | 桌面启动 | Tauri runtime | Tauri CLI / 桌面入口 | 不得拥有后端 API 或前端路由语义 |
| build/delivery config | Cargo/Tauri/Docker/compose/nginx 输入 | 构建、dev server、容器和代理配置 | 开发者、构建工具 | 不得主动进入发布版本过渡 |

**父级通信规则**:
`system` 作为顶层父模块，只协调启动、桌面壳、构建交付和运行配置样例。子模块对外必须经对应父域或明确入口通信，不得横向拥有后端业务 API、frontend 业务状态、executor 状态、contracts 真源或 release transition。

**允许调用的子模块**:
`system.entry.launch_scripts`、`system.entry.backend_process`、`system.desktop_shell.tauri_runtime`、`system.desktop_shell.tauri_config`、`system.desktop_shell.assets_schema`、`system.build_delivery.desktop_build_scripts`、`system.build_delivery.container_proxy`、`system.runtime_profile.config_examples`。

**已收束子模块**:
S1-S10 已完成 closeout 或静态 closeout。S6/S9 的历史暂停已由 `markdown/06-milestones/v4.16.0/25-system.build_delivery.S6-S9恢复提案与适配性校验.md` 解除并收束。

**回归保护**:
`cargo check -p quantpilot`；`cargo check -p quantpilot-tauri`；`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`；schema JSON parse；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 system 顶层 closeout 完成时，必须指出这是阶段性 closeout；整理、重构、发布验收和 Docker runtime smoke 均未启动。

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
**状态**: v4.16 S2 单叶 closeout 完成。public 启动入口和 API server 启动实现已迁入 system 模块，旧 crate 入口通过 re-export 兼容；不扩大到 API route owner。
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
AI 声称 S2 已完成时，必须指出完成范围是 `system.entry.backend_process` 启动边界；`build_app_router` 仍属 `backend.interface_boundary`，`new_app_state` 仍属 `app_runtime_helpers`。

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

### 3.1.3 `system.desktop_shell.tauri_runtime`

**层级路径**: `root.system.desktop_shell.tauri_runtime`
**父模块**: `system.desktop_shell`
**状态**: v4.16 S3 单叶 closeout 完成。Tauri runtime 入口、3000 readiness wait、桌面启动 smoke、窗口生命周期和关闭路径已完成白箱登记；未改代码，不继续细分。
**真实文件**:
- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**职责**:
承载 Tauri 桌面 runtime 入口、后端 readiness wait、shell plugin 初始化、debug devtools setup 和 `generate_context` 启动链。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| Tauri `main` | 桌面应用启动 | Tauri runtime 进程 | Tauri CLI / 桌面启动链 | 不得改窗口生命周期、后端启动关系或 Tauri command 权限 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `wait_for_backend` | `127.0.0.1:3000` TCP connect | 后端 readiness 判定 | Tauri `main` | 不得拥有后端 API、AppState 或业务 capability 真源 |
| `TcpStream::connect_timeout` | 1 秒连接超时 | readiness 成功/失败路径 | `wait_for_backend` | 不得替代为未登记业务 API probe |
| `MAX_WAIT_SECS = 30` | 启动等待窗口 | 超时后继续进入 Tauri runtime | `wait_for_backend` | 不得造成永久阻塞 |
| `tauri::Builder::default` | Tauri context | 桌面 runtime | Tauri `main` | 不得混入后端 handler 或前端 route owner |
| `tauri_plugin_shell::init` | Tauri Builder | shell plugin | Tauri runtime | 不得新增未登记权限 |

**父级通信规则**:
`system.desktop_shell.tauri_runtime` 只能通过 `system.desktop_shell` 管理桌面壳 runtime 和 readiness wait；不得直接横向连接 `backend.interface_boundary`、`frontend.*`、runtime state、AppState 或 capability 真源。

**回归保护**:
`cargo check -p quantpilot-tauri`；`cargo build --bin quantpilot`；`cargo tauri dev --no-watch` 桌面启动 smoke；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 S3 完成时，必须指出完成范围是 Tauri runtime 单叶 closeout；本批次没有修改 `src-tauri/src/main.rs`、Tauri config、capability 或启动脚本。

### 3.1.4 `system.desktop_shell.tauri_config`

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

### 3.1.5 `system.desktop_shell.assets_schema`

**层级路径**: `root.system.desktop_shell.assets_schema`
**父模块**: `system.desktop_shell`
**状态**: v4.16 S5 单叶 closeout 完成。桌面图标和 Tauri generated schema 已完成白箱登记，不改资产，不重新生成 schema，不继续细分。
**真实文件**:
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.ico`
- `src-tauri/gen/schemas/acl-manifests.json`
- `src-tauri/gen/schemas/capabilities.json`
- `src-tauri/gen/schemas/desktop-schema.json`
- `src-tauri/gen/schemas/windows-schema.json`
- `src-tauri/tauri.conf.json`

**职责**:
承载桌面壳打包图标和 Tauri generated schema 资产，保证资产路径和生成物消费方式可追踪。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| Tauri icon asset paths | Tauri config | 桌面打包图标 | Tauri CLI / bundler | 不得借资产 closeout 改品牌或窗口配置 |
| Tauri generated schema files | Tauri tooling | ACL/capability/desktop/window schema | Tauri 工具链、文档核查 | 不得手改生成物并当业务 schema 真源 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| icon files | image assets | app icon resources | Tauri bundler | 不得改 `src-tauri/tauri.conf.json` icon path |
| generated schema JSON | Tauri generator | schema artifacts | Tauri tooling | 不得和后端 API response schema 混用 |

**父级通信规则**:
`system.desktop_shell.assets_schema` 只能经 `system.desktop_shell` 提供桌面资产和 generated schema；不得直接横向连接后端 schema、前端 UI 设计系统、Tauri runtime 权限语义或 release packaging。

**回归保护**:
JSON parse；资产存在性检查；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。涉及 schema 重新生成或图标替换时必须补 diff 和人工验收。

**幻觉检查点**:
AI 声称 S5 完成时，必须指出本批次没有改图标、没有重新生成 schema、没有把 generated schema 当业务 schema 真源。

### 3.1.6 `system.runtime_profile.config_examples`

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

### 3.1.7 `system.build_delivery.desktop_build_scripts`

**层级路径**: `root.system.build_delivery.desktop_build_scripts`
**父模块**: `system.build_delivery`
**状态**: v4.16 S7 单叶 closeout 完成。Desktop build/dev scripts 已完成白箱登记，不改脚本语义，不继续细分。
**真实文件**:
- `src-tauri/build.rs`
- `src-tauri/build.bat`
- `src-tauri/dev.bat`
- `src-tauri/tauri.conf.json`
- `frontend/package.json`

**职责**:
承载 Tauri build/dev 前置脚本和 Rust build script，负责把 Tauri CLI 的 build/dev 生命周期连接到前端 build/dev server。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `src-tauri/build.rs` | Cargo build script lifecycle | Tauri build metadata | Cargo / Tauri build | 不得拥有业务构建产物语义 |
| `src-tauri/build.bat` | Tauri `beforeBuildCommand` | `frontend/dist` production build | Tauri CLI | 不得改变 frontend build 命令或产物路径 |
| `src-tauri/dev.bat` | Tauri `beforeDevCommand` | Vite dev server on 5173 | Tauri CLI | 不得改变 dev server 端口或 strictPort 语义 |

**关键内部启动实现**:
| 实现 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `tauri_build::build()` | Tauri build config | build script side effects | `src-tauri/build.rs` | 不得混入 runtime 初始化 |
| `npm run build` | `frontend/package.json` | Vite production bundle | `src-tauri/build.bat` | 不得绕过 frontend build owner |
| `npm run dev -- --strictPort` | `frontend/package.json` | Vite dev server 5173 | `src-tauri/dev.bat` | 不得改端口抢占策略 |
| `beforeBuildCommand` / `beforeDevCommand` | Tauri config | build/dev hook wiring | Tauri CLI | 不得混入 S4 config 变更 |

**父级通信规则**:
`system.build_delivery.desktop_build_scripts` 只能通过 `system.build_delivery` 提供 desktop build/dev 脚本入口，不拥有根启动脚本、Tauri runtime、Tauri config、CI/release、container proxy、后端 API 或前端业务模块。

**回归保护**:
`cargo check -p quantpilot-tauri`；`cmd /c src-tauri\build.bat`；受控 `src-tauri\dev.bat` 5173 smoke；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 S7 完成时，必须指出本批次没有修改 `src-tauri/build.rs`、`src-tauri/build.bat` 或 `src-tauri/dev.bat`，只完成 desktop build/dev scripts 等价 closeout。

### 3.1.8 `system.build_delivery.workspace_manifest`

**层级路径**: `root.system.build_delivery.workspace_manifest`
**父模块**: `system.build_delivery`
**状态**: v4.16 S6 单叶 closeout 完成。workspace manifest、package manifest 和 lockfile 已登记边界，不改依赖、workspace 成员、feature 或 lockfile。
**真实文件**:
- `Cargo.toml`
- `Cargo.lock`
- `src-tauri/Cargo.toml`

**职责**:
承载 Rust workspace/package manifest、crate metadata、依赖版本、feature 和 lockfile 的交付边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `Cargo.toml` workspace manifest | workspace 成员、依赖、profile | Rust workspace 编译图 | Cargo、CI、开发者 | 不得顺手升级依赖或改 workspace member |
| `src-tauri/Cargo.toml` package manifest | Tauri package metadata、依赖、features | desktop crate 编译图 | Cargo、Tauri CLI | 不得混入 Tauri runtime 或 config 语义 |
| `Cargo.lock` lockfile | dependency resolution | 固定依赖版本图 | Cargo、CI | 不得无说明制造大幅漂移 |

**父级通信规则**:
`system.build_delivery.workspace_manifest` 只能经 `system.build_delivery` 管理编译图和依赖边界。它不得直接改变后端 API、Tauri runtime、CI/release workflow、发布版本过渡或业务模块行为。

**回归保护**:
`cargo metadata --format-version 1 --no-deps`；`cargo check -p quantpilot`；`cargo check -p quantpilot-tauri`；lockfile diff 人工核查；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称 S6 已完成时，必须指出这是文档级 closeout，不是依赖升级；本批次未改 `Cargo.toml`、`Cargo.lock` 或 `src-tauri/Cargo.toml`。

### 3.1.9 `system.build_delivery.container_proxy`

**层级路径**: `root.system.build_delivery.container_proxy`
**父模块**: `system.build_delivery`
**状态**: v4.16 S8 静态单叶 closeout 完成。Dockerfile、compose 和 nginx proxy 已登记；Docker runtime smoke 只有在开发者明确决定进入版本发布/发布验收时才执行。
**真实文件**:
- `Dockerfile`
- `docker-compose.yml`
- `nginx.conf`

**职责**:
承载容器镜像构建、compose 本地编排和 nginx TLS 反向代理配置。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| Docker build context | repo source | backend/frontend/runtime image | Docker build | 不得改变桌面默认运行路径 |
| compose `backend` service | image build、env、volume | backend container on 3000 | docker compose | 不得改端口或环境语义 |
| compose `frontend-dev` service | frontend source、backend origin | Vite dev server on 5173 | docker compose dev profile | 不得替代 S7 desktop dev script |
| nginx proxy config | TLS cert、HTTP request | proxy to `quantpilot:3000` | nginx | 不得改后端 handler 或 route 语义 |

**父级通信规则**:
`system.build_delivery.container_proxy` 只能经 `system.build_delivery` 提供容器和代理配置；不得直接拥有启动脚本、桌面壳、后端 API handler、前端路由、CI/release workflow 或发布版本过渡决策。Docker runtime smoke 不由 AI 主动触发，只能由开发者版本发布/发布验收决策或明确 S8 runtime 验收要求触发。

**回归保护**:
Docker/compose static review；发布验收触发的 `docker compose config`；版本发布/发布验收时补 runtime smoke；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 S8 完成时，必须指出这是静态 closeout；当前未进入版本发布/发布验收，未执行 `docker compose config` 或容器启动 smoke。

### 3.1.10 `system.build_delivery.ci_release`

**层级路径**: `root.system.build_delivery.ci_release`
**父模块**: `system.build_delivery`
**状态**: v4.16 S9 单叶 closeout 完成。CI/release workflow、packaging 和 release manifest 已登记边界，不改 workflow、测试矩阵、artifact、release 权限或 packaging 语义。
**真实文件**:
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/scenario-test.yml`
- `packaging/windows/installer.nsi`
- `release/release-manifest.yaml`

**职责**:
承载 GitHub Actions CI、release workflow、scenario test workflow、Windows packaging 和 release manifest 的交付边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `.github/workflows/ci.yml` | push/PR workflow event | CI job result | GitHub Actions | 不得静默删除测试门禁 |
| `.github/workflows/release.yml` | release/tag workflow event | release artifact | GitHub Actions | 不得无 dry-run 改 release 权限或 artifact |
| `.github/workflows/scenario-test.yml` | scenario workflow event | scenario test result | GitHub Actions | 不得和测试资产汰换混成一批 |
| `packaging/windows/installer.nsi` | release packaging inputs | Windows installer script | release workflow | 不得改安装路径或打包语义 |
| `release/release-manifest.yaml` | release metadata | release manifest | release workflow、开发者 | 不得伪造发布状态 |

**父级通信规则**:
`system.build_delivery.ci_release` 只能经 `system.build_delivery` 管理 CI/release 交付边界。它不得直接改变测试资产汰换策略、业务测试语义、发布版本过渡或运行时能力声明。

**回归保护**:
workflow YAML review；pre-commit 本地门禁；release dry-run 方案；测试资产汰换登记；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称 S9 已完成时，必须指出这是文档级 closeout，不是发布验收；本批次未改 `.github/workflows/*.yml`、`packaging/` 或 `release/`。

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
**状态**: v4.16 BE-001E capability snapshot 薄壳已落位。capability 真源边界保持单一 API facade。
**真实文件**:
- `src/backend/capability.rs`
- `src/backend/capability/snapshot.rs`
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
**状态**: v4.16 BE-001D L3 模块壳抽离完成。artifact、preflight、diff、AI proposal binding 四个子叶 facade 已落位；handler/schema 仍保留在 `src/strategy_config_api.rs`。
**真实文件**:
- `src/backend/strategy_config.rs`
- `src/backend/strategy_config/artifact.rs`
- `src/backend/strategy_config/preflight.rs`
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/ai_proposal_binding.rs`
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

### 3.4.1 `backend.strategy_config.artifact`

**层级路径**: `root.backend.strategy_config.artifact`
**父模块**: `backend.strategy_config`
**状态**: v4.16 BE-001D L3 facade 已落位。只拥有 artifact route facade，不拥有 artifact handler/schema。
**真实文件**:
- `src/backend/strategy_config/artifact.rs`
- `src/strategy_config_api.rs`

**职责**:
登记 v4 strategy config artifact 的 L3 子叶边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_strategy_config_artifact_route` | Axum Router | artifact route | `backend.strategy_config::register_routes` | 不得迁移 `StrategyConfigArtifact` schema |
| `/api/v1/strategy-config/artifact` | strategy config request | strategy config artifact | 前端配置台、导出路径 | 不得绕过 QS/Core IR 证据 |

**父级通信规则**:
artifact 子叶只能经 `backend.strategy_config` 注册 route；不得直接横向调用 runtime state 或 graph compile 内部状态。

**回归保护**:
`cargo test -p quantpilot strategy_config`；涉及 API 时运行 `cargo test -p quantpilot --test api_ai_proposal` 和 route diff。

### 3.4.2 `backend.strategy_config.preflight`

**层级路径**: `root.backend.strategy_config.preflight`
**父模块**: `backend.strategy_config`
**状态**: v4.16 BE-001D L3 facade 已落位。只拥有 preflight route facade，不拥有 preflight handler/schema。
**真实文件**:
- `src/backend/strategy_config/preflight.rs`
- `src/strategy_config_api.rs`

**职责**:
登记 strategy config preflight readiness、runtime boundary 和拒绝原因的 L3 子叶边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_strategy_config_preflight_route` | Axum Router | preflight route | `backend.strategy_config::register_routes` | 不得静默降级 unsupported |
| `/api/v1/strategy-config/preflight` | artifact 或策略输入 | readiness report | 前端、执行前核验 | 不得绕过 capability 真源 |

**父级通信规则**:
preflight 子叶只能通过 `backend.strategy_config` 暴露 API；不得直接替代 runtime 或 executor 的执行状态判断。

**回归保护**:
`cargo test -p quantpilot strategy_config`；涉及 capability 时运行 capability governance 检查。

### 3.4.3 `backend.strategy_config.diff`

**层级路径**: `root.backend.strategy_config.diff`
**父模块**: `backend.strategy_config`
**状态**: v4.16 BE-001D L3 facade 已落位。只拥有 diff route facade，不拥有 diff handler/schema。
**真实文件**:
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`

**职责**:
登记 strategy config domain diff 与 evidence diff 的 L3 子叶边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_strategy_config_diff_route` | Axum Router | diff route | `backend.strategy_config::register_routes` | 不得以裸 JSON diff 替代用户语义 |
| `/api/v1/strategy-config/diff` | 左右 artifact | domain diff | 版本历史、配置台 | 不得丢弃 source digest changes |

**父级通信规则**:
diff 子叶只比较 strategy config artifact 和 evidence，不拥有 graph version 或 backtest record 的状态所有权。

**回归保护**:
`cargo test -p quantpilot strategy_config`；涉及 graph version compare 时运行 `cargo test -p quantpilot --test api_graph_versions`。

### 3.4.4 `backend.strategy_config.ai_proposal_binding`

**层级路径**: `root.backend.strategy_config.ai_proposal_binding`
**父模块**: `backend.strategy_config`
**状态**: v4.16 BE-001D L3 facade 已落位。当前是 no-op facade，只登记 AI proposal 配置域绑定边界。
**真实文件**:
- `src/backend/strategy_config/ai_proposal_binding.rs`
- `src/strategy_config_api.rs`
- `src/runtime/mutation.rs`
- `tests/api_ai_proposal.rs`

**职责**:
登记 AI proposal 与 strategy config domain binding 的 L3 子叶边界，实际校验逻辑当前仍保留在 runtime mutation。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_routes` | Axum Router | unchanged router | `backend.strategy_config::register_routes` | 不得伪造不存在的 route |
| `validate_ai_proposal_config_domain_binding` | AI proposal mutation request | static check detail | runtime mutation | 不得在无等价基线时迁出 runtime |

**父级通信规则**:
AI proposal binding 子叶只能记录 strategy config 与 runtime mutation 的契约关系；不得绕过 approval、sandbox 或 mutation ledger。

**回归保护**:
`cargo test -p quantpilot --test api_ai_proposal`；涉及 approval/sandbox 时运行对应 mutation 和 sandbox 测试。

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

### 4.1 `backend`

**层级路径**: `root.backend`
**父模块**: `root`
**状态**: v4.16 BE-001E 九叶子 facade 坐标已落位。`src/backend/` 已建立父模块、9 个叶子 facade、strategy_config L3 facade 和其余八叶薄壳子 facade；真实 handler、state owner、response schema 和 artifact schema 仍保留原位。
**真实文件**:
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
- `src/backend/graph_compile/compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/backend/storage_security/credential_vault.rs`
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
- `src/backend/strategy_config/preflight.rs`
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/ai_proposal_binding.rs`
- `src/app_router.rs`
- `src/app_runtime_helpers.rs`
- `src/lib.rs`
- `src/capability_api.rs`
- `src/strategy_config_api.rs`
- `src/runtime/mod.rs`
- `src/runtime/run.rs`
- `src/runtime/backtest.rs`
- `src/runtime/mutation.rs`
- `src/graph_api.rs`
- `src/graph_quantscript_api.rs`
- `src/compile_api.rs`
- `src/storage_lifecycle.rs`
- `src/credential_vault.rs`
- `src/tests_backend.rs`

**职责**:
承载后端 API、运行、编译、配置、能力真源、存储安全、运维治理、AppState wiring 和后端测试支撑的顶层父模块。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `build_app_router` | `AppState` | Axum Router | system 启动链、测试入口 | 不得绕过 `backend.interface_boundary` |
| `get_capabilities` | capability source | capability snapshot | 前端 capability projection、治理检查 | 不得由前端静态判断替代 |
| `register_runtime_routes` | Axum Router | runtime routes | `build_app_router` | 不得迁移 runtime state owner |
| `register_graph_routes` / `register_graph_quantscript_routes` | Axum Router | graph/QS routes | `build_app_router` | 不得绕过 graph version 和 QS 安全边界 |
| `register_compile_routes` | Axum Router | compile routes | `build_app_router` | 不得把 strategy_ir 当 runtime 真源 |
| `register_strategy_config_routes` | Axum Router | strategy config routes | `build_app_router` | 不得改变 preflight 或 artifact 语义 |

**父级通信规则**:
`backend` 的子叶必须经 `backend.interface_boundary`、明确 API/facade、storage helper 或契约边界通信。子叶不得横向抢 route owner、handler、state owner、response schema、artifact schema 或测试资产归属。

**允许调用的子模块**:
`backend.interface_boundary`、`backend.runtime`、`backend.graph_compile`、`backend.capability`、`backend.strategy_config`、`backend.storage_security`、`backend.ops_governance`、`backend.app_state_wiring`、`backend.test_support`。这些叶子当前是 facade 壳和白箱 closeout 坐标，不代表 handler 已迁移。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_graph_versions`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 backend 已推进时，必须说明当前完成的是 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D strategy_config L3 壳和 BE-001E 其余八叶薄壳；不得宣称 runtime、compile、storage/security、AppState 或测试资产 handler 已迁移完成，也不得宣称 `root.backend` 顶层已经收束。

---

## 5. v4.13 第一波白箱节点

### 5.0 `backend.interface_boundary`

**层级路径**: `root.backend.interface_boundary`
**父模块**: `backend`
**状态**: v4.16 BE-001E 薄壳子 facade 已落位。`src/app_router.rs` 现在通过 `src/backend/interface_boundary.rs` 调用各桥接子 facade；本叶仍作为父级 route facade，真实 handler 仍分布在既有文件中。
**真实文件**:
- `src/backend/interface_boundary.rs`
- `src/backend/interface_boundary/app_state_bridge.rs`
- `src/backend/interface_boundary/capability_bridge.rs`
- `src/backend/interface_boundary/graph_compile_bridge.rs`
- `src/backend/interface_boundary/ops_governance_bridge.rs`
- `src/backend/interface_boundary/runtime_bridge.rs`
- `src/backend/interface_boundary/storage_security_bridge.rs`
- `src/backend/interface_boundary/strategy_config_bridge.rs`
- `src/backend/interface_boundary/test_support_bridge.rs`
- `src/backend/capability.rs`
- `src/backend/strategy_config.rs`
- `src/backend/runtime.rs`
- `src/backend/graph_compile.rs`
- `src/backend/storage_security.rs`
- `src/backend/ops_governance.rs`
- `src/backend/app_state_wiring.rs`
- `src/backend/test_support.rs`
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
先抽一个大模块，再在大模块里抽小模块。BE-001C 已确认 `backend.interface_boundary` 只作为父级 route facade，不继续向下拆；后续业务拆分落到它管理的子叶。

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
| `register_credential_routes` | Axum Router | credential routes | `build_app_router` | 不得绕过凭证保险库 |
| `register_test_scenario_routes` | Axum Router | test scenario routes | `build_app_router` | 不得把测试支撑当生产 owner |

**父级通信规则**:
所有后端接口抽离必须先经过 `backend.interface_boundary` 父级边界。子模块不得直接互相横向改 route、handler、state owner 或 response schema。

**允许调用的子模块**:
`backend.capability`、`backend.strategy_config`、`backend.runtime`、`backend.graph_compile`、`backend.storage_security`、`backend.ops_governance`、`backend.app_state_wiring`、`backend.test_support`。

**禁止横向连接**:
不得让 `backend.runtime` 直接改 `backend.graph_compile` route owner；不得让前端绕过 API 读取后端内部文件；不得让执行端状态直接并入后端接口边界。

**状态与锁**:
BE-001 不迁移状态所有权，不改变 AppState、runtime state、executor state、锁顺序或事务边界。

**回归保护**:
`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_graph_versions`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称后端接口边界已经抽离时，必须指出 BE-001、`build_app_router`、对应 `register_*_routes`、旧 handler 保留方式和回退点。

### 5.1 `backend.runtime`

**层级路径**: `root.backend.runtime`
**父模块**: `backend`
**状态**: v4.16 BE-001H-03 `runtime.run.v4_handoff` 已完成单叶 closeout，当前不继续细拆；BE-001I-03 `runtime.run.session_start` 已完成单叶 closeout，当前不继续细拆；BE-001J-01 `runtime.run.record_store` 已建立单子叶等价基线，当前不移动代码。runtime route aggregate 已迁入 `src/backend/runtime/routes.rs`，run route group 已迁入 `src/backend/runtime/routes/run.rs`；`/api/runtime/v4/run` handler 已迁入 `src/runtime/run/v4_handoff.rs`，legacy `/api/runtime/test-run` handler 已迁入 `src/runtime/run/session_start.rs`，其余 runtime record/replay/status/SSE/state 仍保留在 `src/runtime/`。
**真实文件**:
- `src/backend/runtime.rs`
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/mod.rs`
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run.rs`
- `src/runtime_persistence.rs`
- `src/runtime_event_projection.rs`
- `src/runtime_validation.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_diagnostics.rs`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`
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
runtime 对外必须经过 `backend.interface_boundary` 注册的 HTTP API、事件流或持久化接口；不得由前端直接读取内部文件推断运行状态。

**允许调用的子模块**:
`backend.runtime.routes`、`backend.runtime.routes.run`、`runtime_persistence`、`runtime_validation`、`runtime_event_projection`、`backtest_artifacts`。

**禁止横向连接**:
不得直接调用 `executor.runner` 的内部状态；执行端交互必须经迁移包、执行端 API 或 runtime evidence。

**状态与锁**:
涉及运行记录、事件流、backtest artifact 和 transient spill 时，必须保留状态归属和清理边界。

**回归保护**:
`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；涉及 v4 evidence 时跑 `cargo test -p quantpilot --test api_evidence_contract`。

**幻觉检查点**:
AI 声称 runtime 支持新能力时，必须指出真实路由、record/artifact 字段和测试。

### 5.1.1 `backend.runtime.routes`

**层级路径**: `root.backend.runtime.routes`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001G-03 `backend.runtime.routes.run` closeout 已完成，BE-001I-03 已完成其下一个 handler sibling `runtime.run.session_start` 单叶 closeout，BE-001J-01 已建立 `runtime.run.record_store` 单子叶等价基线。当前拥有 runtime route aggregate 列表，并通过 `backend.runtime.routes.run` 委托 run routes；父级仍直接拥有 event stream route，不拥有 runtime handler、state owner、artifact schema 或 persistence owner。
**真实文件**:
- `src/backend/runtime.rs`
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/mod.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run.rs`
- `src/runtime/backtest.rs`
- `src/runtime/mutation.rs`
- `src/runtime_event_projection.rs`
- `src/runtime_persistence.rs`
- `src/backtest_artifacts.rs`
- `markdown/06-milestones/v4.16.0/50-backend.runtime.routes单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/51-backend.runtime.routes抽离记录.md`
- `markdown/06-milestones/v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/53-backend.runtime.routes.run抽离记录.md`
- `markdown/06-milestones/v4.16.0/54-backend.runtime.routes.run单叶closeout.md`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`

**职责**:
承载 backend runtime route aggregate facade 的白箱坐标，固定 `backend.runtime -> backend.runtime.routes -> src/runtime/* pub(crate) handler` 的兼容桥和等价证据。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| Axum Router | `backend.runtime` | `Router<AppState>` | 不改变 route registration 顺序 |
| AppState | `backend.app_state_wiring` | shared app state | 不迁移 AppState owner 或锁顺序 |
| runtime HTTP request | frontend、tests、local API caller | `/api/runtime/*` request | 不改 path、method、payload 或 error code |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| runtime routes | `backend.interface_boundary` | Axum Router | 由 `backend.runtime.routes` 注册并委托 `src/runtime/*` handler |
| runtime response | frontend、tests | JSON / status code | 不改 response schema |
| runtime event stream | frontend SSE panel、tests | SSE frames | 不改 event envelope 或 replay cursor |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime::register_routes` | Axum Router | runtime routes | `backend.interface_boundary` | 不得绕过 `backend.runtime.routes` |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 不得迁移 runtime handler |
| `backend.runtime.routes.run::register_routes` | Axum Router | run routes | `backend.runtime.routes` | 不得接管 event stream |
| `src/runtime/* pub(crate) handler` | HTTP request | concrete runtime response | `backend.runtime.routes` | 不得改变 `/api/runtime/*` 语义 |
| `/api/runtime/test-run` | run request | run record | frontend、tests | 不得迁移 state owner |
| `/api/runtime/v4/run` | v4 graph/run request | v4 run record | frontend、tests | 不得绕过 governance/evidence |
| `/api/runtime/backtest` | backtest request | backtest artifact | frontend、tests | 不得改 artifact schema |
| `/api/runtime/runs/:run_id/events` | run id | SSE stream | frontend、tests | 不得改变 SSE frame |

**父级通信规则**:
`backend.runtime.routes` 只能经 `backend.runtime` 和 `backend.interface_boundary` 暴露 runtime routes；不得横向直接改 `backend.graph_compile`、`backend.storage_security`、`executor` 或 frontend state。

**允许调用的子模块**:
`backend.runtime.routes.run`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`、`src/backtest_compare.rs` 中的 `pub(crate)` route targets。真实 run/backtest/mutation/report/experiment 子域仍留在 `src/runtime/`，后续若继续拆分必须另起单子叶等价基线。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_sse`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称 runtime routes 已迁移时，必须说明当前只迁移 route aggregate owner；不得宣称 run/backtest/mutation handler、event stream、state owner 或 persistence 已迁移。

### 5.1.2 `backend.runtime.routes.run`

**层级路径**: `root.backend.runtime.routes.run`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001H-03 `runtime.run.v4_handoff` 已完成单叶 closeout 并停止内部细分；BE-001I-03 `runtime.run.session_start` 已完成单叶 closeout 并停止内部细分；BE-001J-01 `runtime.run.record_store` 已建立单子叶等价基线。当前只拥有 run route group facade，不拥有 state owner、event stream 或 persistence owner；route facade 本身停止细分，handler 层继续按 `runtime.run` sibling 队列递归。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run.rs`
- `src/runtime_persistence.rs`
- `src/runtime_event_projection.rs`
- `markdown/06-milestones/v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/53-backend.runtime.routes.run抽离记录.md`
- `markdown/06-milestones/v4.16.0/54-backend.runtime.routes.run单叶closeout.md`
- `markdown/06-milestones/v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/56-runtime.run.v4_handoff抽离记录.md`
- `markdown/06-milestones/v4.16.0/57-runtime.run.v4_handoff单叶closeout.md`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`

**职责**:
承载 run/v4 run/list/detail/save/replay/status route group facade，固定 `backend.runtime.routes -> backend.runtime.routes.run -> src/runtime/run.rs pub(crate) handler` 的兼容桥和等价证据。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| Axum Router | `backend.runtime.routes` | `Router<AppState>` | 不改变 run route path、method 或 handler 类型 |
| run HTTP request | frontend、tests、local API caller | `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs*` request | 不改 payload、path param、response schema 或 error code |
| AppState | `backend.app_state_wiring` | shared app state | 不迁移 AppState owner 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| run routes | `backend.runtime.routes` | Axum Router | 不接管 `/api/runtime/runs/:run_id/events` |
| run response | frontend、tests | JSON / status code | 不改 run record、status 或 replay schema |
| persistence / replay lookup | `src/runtime_persistence.rs`、`src/runtime_event_projection.rs` | existing helper call | 不改 owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes.run::register_routes` | Axum Router | run route group | `backend.runtime.routes` | 不得注册 event stream |
| `start_test_run` | run request | run record | route facade | 不得迁移 state owner |
| `start_v4_runtime_run` | v4 graph/run request | v4 run record | route facade | 不得绕过 validation/evidence |
| `list_runs` / `get_run_detail` | run list/detail request | run record response | route facade | 不得改 persistence projection |
| `save_run_record` / `discard_run_record` | run id | storage mutation response | route facade | 不得改 storage semantics |
| `get_run_replay` / `get_run_status` | run id | replay/status response | route facade | 不得改 event projection |

**父级通信规则**:
`backend.runtime.routes.run` 只能经 `backend.runtime.routes` 暴露 run routes；不得横向直接改 event stream、backtest、mutation、report、experiment、executor 或 frontend state。

**允许调用的子模块**:
`src/runtime/run.rs` 中的 legacy run route targets、`src/runtime/run/v4_handoff.rs` 中的 v4 handoff target，以及既有 persistence / event projection helper 调用边界。state owner 继续保留在 `AppState`。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_sse`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
`backend.runtime.routes.run` 这个 route facade 不继续细分；真实 handler owner 已从 `runtime.run.v4_handoff` 和 `runtime.run.session_start` 完成两片 closeout，并已为 `runtime.run.record_store` 建立单子叶等价基线。后续不得继续细拆 session start；record_store 若推进必须先基于 61 做抽离方案；其余 sibling 候选包括 `runtime.run.replay_status`；`runtime.event_stream` 仍是父级 route 子叶候选，不属于本 facade。

**幻觉检查点**:
AI 声称 runtime run routes 已迁移时，必须说明 run route group facade、`runtime.run.v4_handoff` handler 子模块、`runtime.run.session_start` handler 子模块与 `runtime.run.record_store` 基线是不同动作；不得宣称 `src/runtime/run.rs` 全部 handler、state owner、event stream 或 persistence 已迁移。AI 声称本子叶完成时，还必须说明 route facade 停止细分不等于 run handler 全部完成。

### 5.1.3 `runtime.run.v4_handoff`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.v4_handoff`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001H-03 单叶 closeout 已完成，当前停止内部细分。`/api/runtime/v4/run` handler、request/response type、graph resolution、initial event、handoff projection 与 simulated capability matrix 已迁入 `src/runtime/run/v4_handoff.rs`；父级 `runtime` 保留受控 re-export。
**真实文件**:
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/mod.rs`
- `src/runtime/run.rs`
- `src/runtime/backtest.rs`
- `src/backend/runtime/routes/run.rs`
- `markdown/06-milestones/v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/56-runtime.run.v4_handoff抽离记录.md`
- `markdown/06-milestones/v4.16.0/57-runtime.run.v4_handoff单叶closeout.md`

**职责**:
承载 `/api/runtime/v4/run` 的 v4 QS source / preparsed graph / initial event / handoff report / paper simulated runtime handler 子模块，并通过父级 runtime 出口保持 route 兼容。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `/api/runtime/v4/run` request | `backend.runtime.routes.run` | `V4RuntimeRunRequest` | 必须包含 source 或 graph |
| v4 QS source | frontend、tests | String | 必须经 static audit 与 handoff |
| v4 machine graph | frontend、tests | `V4MachineGraphContract` | 必须经 static contract validation |
| initial event | frontend、tests | `V4RuntimeInputEvent` | 缺省时从 event catalog 派生 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| v4 run response | frontend、tests | `V4RuntimeRunResponse` | 不改 response schema |
| handoff diagnostics | frontend、tests | `V4RuntimeRunDiagnostic` | 不改 error code 语义 |
| paper simulated output | frontend、tests | `V4PaperSimulatedRunOutput` | 不改 capability matrix |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_v4_runtime_run` | `V4RuntimeRunRequest` | `V4RuntimeRunResponse` | `backend.runtime.routes.run` | 不得混入 legacy `start_test_run` |
| `resolve_v4_runtime_run_graph` | source / graph / initial event | graph、handoff、diagnostics、initial event | `start_v4_runtime_run` | 不得绕过 static audit |
| `handoff_initial_event` | handoff、graph、timestamp | `V4RuntimeInputEvent` | `start_v4_runtime_run` | 不得改变 event catalog fallback |
| `v4_runtime_handoff_response` | handoff report | response handoff | `start_v4_runtime_run` | 不得改 response schema |
| `default_v4_payload_value` | payload field、graph id | JSON value | `handoff_initial_event` | 不得改 default payload semantics |
| `runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` | venue id | v4 static contract / capability matrix | v4 handoff path | 不得扩大真实 provider 支持 |

**父级通信规则**:
`runtime.run.v4_handoff` 只能经父级 `runtime` re-export 和 `backend.runtime.routes.run` 暴露 `/api/runtime/v4/run`；不得横向直接改 `runtime.run.session_start`、record store、SSE、backtest、mutation、executor 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 `qrpc_core_ir`、`qrpc_runtime`、`quantscript` static audit / handoff / v4 paper simulated runtime；`runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` 对 `src/runtime/backtest.rs` 的复用只能经父级 `runtime` 受控出口，不得形成 sibling 直连。

**细分价值判断**:
本叶不继续细拆。request/response schema、source/graph resolution、initial event、handoff projection 都服务同一条 v4 handoff route；simulated capability matrix 若未来独立，应另起父级共享节点，不能在本叶内部横向拆出。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 `runtime.run.v4_handoff` 已抽离时，必须指出只完成 v4 handoff handler 子模块抽离；`src/runtime/run.rs` 仍拥有 legacy run/session/record/replay/status sibling。不得宣称 provider 真连接、record store、SSE、persistence 或发布版本过渡已完成。

### 5.1.4 `runtime.run.session_start`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.session_start`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001I-03 单叶 closeout 已完成，当前停止内部细分。legacy `/api/runtime/test-run` 的 `start_test_run` 已迁入 `src/runtime/run/session_start.rs`，父级 `runtime` 保留受控 re-export；record/replay/status、SSE、state owner 和 persistence 仍不迁移。
**真实文件**:
- `src/runtime/run/session_start.rs`
- `src/runtime/run.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime_validation.rs`
- `src/runtime_event_projection.rs`
- `src/runtime_response_mapping.rs`
- `src/compile_api.rs`
- `src/capability_api.rs`
- `src/collaboration.rs`
- `src/graph_quantscript_api.rs`
- `src/frontend_runtime_mapping.rs`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`

**职责**:
承载 legacy `POST /api/runtime/test-run` session start handler 子模块，固定 capability guard、QS compile、runtime session、event envelope、governance snapshot、actor collaboration 和 in-memory `state.runs` 写入边界。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `UserId` | auth middleware | scoped user id | 仅用于 scoped run key，不迁移 auth owner |
| `AppState` | `backend.app_state_wiring` | shared app state | 只使用既有 `run_in_progress`、`runs`、`graph_store_dir` 等字段 |
| `FrontendRunRequest.capability_context` | frontend、tests | runtime capability context | 缺失必须返回 `capability_boundary_violation` 且不创建 run |
| `FrontendRunRequest.runtime_config` | frontend、tests | runtime config | 必须经 `validate_runtime_config_capabilities` |
| `FrontendRunRequest.graph_json` | frontend、tests | graph JSON | 缺失必须按既有 bad request 拒绝 |
| `FrontendRunRequest.runtime_targets` | frontend、tests | runtime target list | 与 graph targets 合并，不改变 event node mapping |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `RunStartResponse` | frontend、tests | run start response | 不改 `run_id`、`graph_id`、`compile_id`、`event_count`、`status` schema |
| in-memory `RunRecord` | `AppState.runs` | scoped run record | 不改 scoped key、governance、actor、events、account、session 写入语义 |
| run guard 状态 | `AppState.run_in_progress` | `RunInProgressGuard` | 不迁移 owner，不改 AcqRel / Release 语义 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_test_run` | `UserId`、`AppState`、`FrontendRunRequest` | `RunStartResponse` | `backend.runtime.routes.run` | 不得混入 record/replay/SSE 迁移 |
| `validate_runtime_capability_guard` | capability context | validation result | `start_test_run` | 不得绕过 capability boundary |
| `validate_runtime_config_capabilities` | runtime config、capability context | validation result | `start_test_run` | 不得放宽 provider 能力 |
| `compile_runtime_protocol_via_qs` / `compile_runtime_protocol_config` | graph/config | compiled runtime protocol | `start_test_run` | 不得绕过 QS compile path |
| `build_compile_runtime_targets_from_graph` / `merge_runtime_targets` | graph targets、request targets | merged runtime targets | `start_test_run` | 不得改变 event node mapping |
| `runtime_governance_snapshot` | compile/runtime context | governance evidence | `start_test_run` | 不得缺失 evidence metadata |
| `collect_frontend_events` / `prepend_capability_snapshot_event` | runtime session events | frontend events | `start_test_run` | 不得改变 event order |
| `attach_runtime_event_envelopes` / `validate_runtime_event_envelopes` | frontend events | governed events | `start_test_run` | 不得绕过 envelope validation |
| `account_summary` / `run_start_response` | run record context | API response | `start_test_run` | 不得改 response schema |
| `normalize_actor_identity` / `collaboration_with_run_actor` | actor context | collaboration metadata | `start_test_run` | 不得迁移 graph audit owner |

**父级通信规则**:
`runtime.run.session_start` 只能经父级 `runtime` 和 `backend.runtime.routes.run` 暴露 `/api/runtime/test-run`；不得横向直接改 `runtime.run.v4_handoff`、`runtime.run.record_store`、`runtime.run.replay_status`、`runtime.event_stream`、backtest、mutation、executor 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 capability validation、QS compile、runtime event projection、response mapping、collaboration、frontend runtime mapping helper。`run_in_progress` 和 `state.runs` owner 继续保留在 `AppState`，本基线不引入新的 persistence owner。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本叶不继续细拆。capability guard、compile path、event projection 和 response mapping 已有真实 owner；本叶只编排 legacy `/api/runtime/test-run` 的 session start 事务。record store、replay/status、SSE、persistence 和 state owner 仍是独立候选，不得在本批混入。

**幻觉检查点**:
AI 声称 `runtime.run.session_start` 已完成时，必须说明只完成 legacy `/api/runtime/test-run` handler 子模块抽离与 closeout；record store、replay/status、SSE、persistence 和 state owner 尚未完成。不得宣称 runtime run handler 全部完成或发布版本过渡已启动。

### 5.1.5 `runtime.run.record_store`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.record_store`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001J-01 单子叶等价基线已建立，当前不移动代码。`list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` 仍保留在 `src/runtime/run.rs`，persistence、audit、response mapping 和 AppState owner 均保留原位。
**真实文件**:
- `src/runtime/run.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/collaboration.rs`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`

**职责**:
承载 run record list/detail/save/discard handler 子模块的等价基线，固定 transient `state.runs`、saved manifest、response projection、安全路径清洗和 graph audit 写入边界。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `AppState` | `backend.app_state_wiring` | shared app state | 只读取既有 `runs`、`run_store_dir`、`audit_store_dir` |
| `UserId` / `run_id` | auth middleware、path param | scoped id / string | detail/save/discard 必须继续使用 scoped run key 或安全路径段 |
| `PaginationQuery` | `/api/runtime/runs` | query | 不改变分页或 created_at 倒序排序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| run list/detail response | frontend、tests | `RunListItem` / `RunDetailResponse` | 不改 response schema |
| saved run manifest | `run_store_dir` | JSON manifest | 不改 bounded read、atomic write、安全路径清洗 |
| graph audit entry | `audit_store_dir` | audit JSON | 不改 `GraphAuditAction::RunCreated` |
| discard response | frontend、tests | `DiscardRuntimeArtifactResponse` | saved record 必须 conflict，transient record 才可 discard |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_runs` | `AppState`、`PaginationQuery` | `PaginatedResponse<RunListItem>` | `backend.runtime.routes.run` | 不得改排序或分页 |
| `get_run_detail` | `UserId`、`AppState`、`run_id` | `RunDetailResponse` | `backend.runtime.routes.run` | 不得绕过 scoped lookup |
| `save_run_record` | `UserId`、`AppState`、`run_id` | `RunDetailResponse` | `backend.runtime.routes.run` | 不得绕过 persistence/audit |
| `discard_run_record` | `UserId`、`AppState`、`run_id` | `DiscardRuntimeArtifactResponse` | `backend.runtime.routes.run` | 不得删除已保存 manifest |
| `load_run_record_from_state` | `AppState`、`UserId`、`run_id` | `RunRecord` | record/replay/mutation callers | 不得改变 in-memory 优先、manifest fallback 顺序 |
| `list_run_records` / `persist_run_record` | run store dir、record | manifest list/write | record store handler | 不得迁移 persistence owner |
| `run_list_item_from_record` / `run_detail_response_from_record` | `RunRecord` | API response | record store handler | 不得改 schema |
| `sanitize_storage_path_segment` | id segment | safe segment | persistence/discard | 不得放宽路径过滤 |
| `persist_graph_audit_entry` / `build_graph_audit_entry` | audit context | audit manifest | save handler | 不得迁移 graph audit owner |

**父级通信规则**:
`runtime.run.record_store` 只能经父级 `runtime` 和 `backend.runtime.routes.run` 暴露 run record routes；不得横向直接接管 `runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest、mutation、executor 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 `runtime_persistence`、`runtime_response_mapping`、`collaboration` audit helper 和 AppState 字段。`state.runs`、`run_store_dir`、`audit_store_dir` 和 persistence owner 继续保留原位，本基线不新建 storage/security owner。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点当前只建立等价基线，不判断内部 helper 是否继续细拆。若后续完成 record_store 抽离和 closeout，再判断 persistence/audit/response projection 是否值得继续拆；当前不得跳过抽离方案直接细拆 helper。

**幻觉检查点**:
AI 声称 `runtime.run.record_store` 已建立基线时，必须说明只是冻结 run record list/detail/save/discard 边界，代码尚未移动；replay/status、SSE、state owner 和 persistence owner 尚未迁移。不得宣称 runtime run handler 全部完成或发布版本过渡已启动。

### 5.2 `backend.graph_compile`

**层级路径**: `root.backend.graph_compile`
**父模块**: `backend`
**状态**: v4.16 BE-001E compile、graph、quantscript graph 薄壳已落位。当前只分出 route facade；真实 handler 和 diagnostics 仍保留原位。
**真实文件**:
- `src/backend/graph_compile.rs`
- `src/backend/graph_compile/compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/backend/graph_compile/quantscript_graph.rs`
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
`cargo test -p quantpilot --test api_graph_versions`；`cargo test -p quantpilot --test quantscript_real_strategy_authoring`；涉及 compile 时跑相关 compile/graph 测试。

**幻觉检查点**:
任何“编译链已支持”的结论必须同时指出 graph route、compile route 和诊断测试。

### 5.3 `backend.storage_security`

**层级路径**: `root.backend.storage_security`
**父模块**: `backend`
**状态**: v4.16 BE-001E credential route 和 vault re-export 薄壳已落位。auth、storage lifecycle、safe log、backup 仍未迁移，后续必须先过安全决策暂停。
**真实文件**:
- `src/backend/storage_security.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/backend/storage_security/credential_vault.rs`
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

### 5.3.1 `backend.ops_governance`

**层级路径**: `root.backend.ops_governance`
**父模块**: `backend`
**状态**: v4.16 BE-001E ops 子 route facade 已落位。sandbox、alerts、snapshots、runbook、chaos、hotswap 分开注册；真实 handler 仍保留在原文件。
**真实文件**:
- `src/backend/ops_governance.rs`
- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/chaos.rs`
- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/runbook.rs`
- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/snapshots.rs`
- `src/alert_engine.rs`
- `src/sandbox_verification.rs`
- `src/snapshot_service.rs`
- `src/runbook.rs`
- `src/chaos_experiment.rs`
- `src/hotswap_api.rs`
- `src/collaboration.rs`
- `src/migration_sender.rs`

**职责**:
承载后端运维治理 route facade，包括告警、沙箱验证、快照、运行手册、混沌实验、hotswap、协作和迁移发送边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_alert_routes` | Axum Router | alert routes | `backend.interface_boundary` | 不得改默认告警规则语义 |
| `register_sandbox_verification_routes` | Axum Router | sandbox routes | `backend.interface_boundary` | 不得跳过沙箱验证证据 |
| `register_snapshot_routes` | Axum Router | snapshot routes | `backend.interface_boundary` | 不得绕过签名校验 |
| `register_runbook_routes` | Axum Router | runbook routes | `backend.interface_boundary` | 不得把操作手册当执行真源 |
| `register_chaos_routes` | Axum Router | chaos routes | `backend.interface_boundary` | 不得默认开启 chaos mode |
| `register_hotswap_routes` | Axum Router | hotswap routes | `backend.interface_boundary` | 不得绕过 hotswap 审计 |

**父级通信规则**:
ops 能力只经 `backend.interface_boundary` 暴露 route facade，不得横向改 runtime state、executor state、storage_security 或 release transition。

**回归保护**:
`cargo check -p quantpilot`；涉及具体 ops route 时运行相应 API test 或人工 route 审核；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

### 5.3.2 `backend.app_state_wiring`

**层级路径**: `root.backend.app_state_wiring`
**父模块**: `backend`
**状态**: v4.16 BE-001E health route 和 state factory 薄壳已落位。`new_app_state` 保持兼容 re-export，不迁移 AppState 字段 owner。
**真实文件**:
- `src/backend/app_state_wiring.rs`
- `src/backend/app_state_wiring/health_route.rs`
- `src/backend/app_state_wiring/state_factory.rs`
- `src/app_runtime_helpers.rs`
- `src/lib.rs`
- `src/system/entry/backend_process.rs`

**职责**:
承载 backend AppState wiring、health route adapter 和启动链与 backend interface 的连接点。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `new_app_state` | storage dirs | AppState | `system.entry.backend_process`、测试入口 | 不得迁移 AppState 字段 owner |
| `health` | AppState | health response | `backend.interface_boundary` | 不得把 health 当业务 capability 真源 |
| `attach_state` | Router + AppState | Axum Router | `build_app_router` | 不得改变 route order 或 state owner |

**父级通信规则**:
AppState wiring 只能连接启动链、router 和现有 AppState 工厂；不得横向修改 runtime、credential、storage 或 executor 状态所有权。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`；涉及 API 状态时运行相关 integration test。

### 5.3.3 `backend.test_support`

**层级路径**: `root.backend.test_support`
**父模块**: `backend`
**状态**: v4.16 BE-001E test scenario 薄壳已落位。测试资产汰换未启动前不删除旧测试程序，旧测试程序和 E2E 整理仍延后。
**真实文件**:
- `src/backend/test_support.rs`
- `src/backend/test_support/scenario.rs`
- `src/api_test_scenario.rs`
- `src/test_runner.rs`
- `src/tests_backend.rs`

**职责**:
承载后端测试支撑入口、test scenario route 和旧测试资产风险窗口登记。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `register_test_scenario_routes` | Axum Router | test scenario routes | `backend.interface_boundary` | 不得把测试 route 当生产能力 |
| `TestRunner::execute` | test runner context | test report | 测试支撑 | 不得替代后端 API 等价证据 |
| `src/tests_backend.rs` integration tests | HTTP requests | assertions | 后端回归 | 不得在无替代证据时删除 |

**父级通信规则**:
测试支撑只证明等价，不拥有生产 handler、state owner 或 response schema。

**回归保护**:
`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_graph_versions`；测试资产汰换时必须引用 `markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md`。

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
- `markdown/06-milestones/v4.16.0/17-system.desktop_shell.tauri_runtime-readiness等价检查.md`
- `markdown/06-milestones/v4.16.0/18-system.desktop_shell.tauri_runtime单叶closeout.md`
- `markdown/06-milestones/v4.16.0/19-system.build_delivery.desktop_build_scripts单叶closeout.md`
- `markdown/06-milestones/v4.16.0/20-system.entry.backend_process单叶closeout.md`
- `markdown/06-milestones/v4.16.0/21-system.desktop_shell.assets_schema单叶closeout.md`
- `markdown/06-milestones/v4.16.0/22-system.build_delivery.container_proxy单叶closeout.md`
- `markdown/06-milestones/v4.16.0/23-system.build_delivery.S6-S9暂停决策记录.md`
- `markdown/06-milestones/v4.16.0/24-system顶层阶段性closeout.md`
- `markdown/06-milestones/v4.16.0/25-system.build_delivery.S6-S9恢复提案与适配性校验.md`
- `markdown/06-milestones/v4.16.0/26-system.build_delivery.workspace_manifest单叶closeout.md`
- `markdown/06-milestones/v4.16.0/27-system.build_delivery.ci_release单叶closeout.md`
- `markdown/06-milestones/v4.16.0/28-backend大模块分层统计.md`
- `markdown/06-milestones/v4.16.0/29-backend.interface_boundary等价基线.md`
- `markdown/06-milestones/v4.16.0/30-backend九叶模块壳抽离记录.md`
- `markdown/06-milestones/v4.16.0/31-backend.interface_boundary单叶closeout.md`
- `markdown/06-milestones/v4.16.0/32-backend.capability单叶closeout.md`
- `markdown/06-milestones/v4.16.0/33-backend.strategy_config单叶closeout.md`
- `markdown/06-milestones/v4.16.0/34-backend.runtime单叶closeout.md`
- `markdown/06-milestones/v4.16.0/35-backend.graph_compile单叶closeout.md`
- `markdown/06-milestones/v4.16.0/36-backend.storage_security单叶closeout.md`
- `markdown/06-milestones/v4.16.0/37-backend.ops_governance单叶closeout.md`
- `markdown/06-milestones/v4.16.0/38-backend.app_state_wiring单叶closeout.md`
- `markdown/06-milestones/v4.16.0/39-backend.test_support单叶closeout.md`
- `markdown/06-milestones/v4.16.0/40-backend.strategy_config_L3模块壳抽离记录.md`
- `markdown/06-milestones/v4.16.0/41-backend其余八叶模块壳抽离记录.md`
- `markdown/06-milestones/v4.16.0/42-backend.interface_boundary子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/43-backend.capability子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/44-backend.runtime子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/45-backend.graph_compile子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/46-backend.storage_security子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/47-backend.ops_governance子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/48-backend.app_state_wiring子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/49-backend.test_support子叶抽离完成记录.md`
- `markdown/06-milestones/v4.16.0/50-backend.runtime.routes单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/51-backend.runtime.routes抽离记录.md`
- `markdown/06-milestones/v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/53-backend.runtime.routes.run抽离记录.md`
- `markdown/06-milestones/v4.16.0/54-backend.runtime.routes.run单叶closeout.md`
- `markdown/06-milestones/v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/56-runtime.run.v4_handoff抽离记录.md`
- `markdown/06-milestones/v4.16.0/57-runtime.run.v4_handoff单叶closeout.md`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`

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
| `markdown/06-milestones/v4.16.0/17-system.desktop_shell.tauri_runtime-readiness等价检查.md` S3 readiness | `system.desktop_shell.tauri_runtime` | Tauri `main`、`wait_for_backend`、3000 readiness 等价证据 | system 单叶 readiness 检查 | 不得把 readiness 检查宣告为完整 S3 closeout |
| `markdown/06-milestones/v4.16.0/18-system.desktop_shell.tauri_runtime单叶closeout.md` S3 closeout | `system.desktop_shell.tauri_runtime` | 桌面启动 smoke、主窗口生命周期、`CloseMainWindow` 退出证据 | system 单叶 closeout | 不得改 Tauri runtime 代码或继续细分 |
| `markdown/06-milestones/v4.16.0/19-system.build_delivery.desktop_build_scripts单叶closeout.md` S7 closeout | `system.build_delivery.desktop_build_scripts` | `src-tauri/build.rs`、`src-tauri/build.bat`、`src-tauri/dev.bat`、5173 dev smoke | system 单叶 closeout | 不得改脚本或混入启动脚本语义 |
| `markdown/06-milestones/v4.16.0/20-system.entry.backend_process单叶closeout.md` S2 closeout | `system.entry.backend_process` | `run_server`、`run_api_server`、兼容入口、未迁移边界 | system 单叶 closeout | 不得扩大到 API route owner |
| `markdown/06-milestones/v4.16.0/21-system.desktop_shell.assets_schema单叶closeout.md` S5 closeout | `system.desktop_shell.assets_schema` | icons、Tauri generated schema、JSON parse 证据 | system 单叶 closeout | 不得把 generated schema 当业务 schema 真源 |
| `markdown/06-milestones/v4.16.0/22-system.build_delivery.container_proxy单叶closeout.md` S8 closeout | `system.build_delivery.container_proxy` | Dockerfile、compose、nginx proxy 静态证据 | system 静态单叶 closeout | 不得宣称 Docker runtime smoke |
| `markdown/06-milestones/v4.16.0/23-system.build_delivery.S6-S9暂停决策记录.md` S6/S9 pause | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` | manifest、workflow、packaging、release 暂停历史边界 | system 暂停历史登记 | 不得把暂停期误判为 closeout 完成 |
| `markdown/06-milestones/v4.16.0/24-system顶层阶段性closeout.md` system top closeout | `root.system` | 10 叶收束、阶段性完成边界 | system 顶层阶段性 closeout | 不得宣称 system 全量最终完成 |
| `markdown/06-milestones/v4.16.0/25-system.build_delivery.S6-S9恢复提案与适配性校验.md` S6/S9 resume | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` | 暂停恢复、适配性校验、分批 closeout 设计 | system 恢复提案 | 不得改真实 manifest/workflow 文件 |
| `markdown/06-milestones/v4.16.0/26-system.build_delivery.workspace_manifest单叶closeout.md` S6 closeout | `system.build_delivery.workspace_manifest` | Cargo workspace/package manifest、lockfile、cargo metadata/check 证据 | system 单叶 closeout | 不得改依赖、feature 或 lockfile |
| `markdown/06-milestones/v4.16.0/27-system.build_delivery.ci_release单叶closeout.md` S9 closeout | `system.build_delivery.ci_release` | CI/release/scenario workflow、packaging、release manifest 证据 | system 单叶 closeout | 不得宣称发布验收完成 |
| `markdown/06-milestones/v4.16.0/28-backend大模块分层统计.md` backend 分层 | `root.backend` | 3 层网络、9 个 L2 叶子候选、后续递归顺序 | backend 抽离批次 | 不得宣称 backend 代码抽离完成 |
| `markdown/06-milestones/v4.16.0/29-backend.interface_boundary等价基线.md` BE-001A baseline | `backend.interface_boundary` | route owner、public/接口入口、保留 handler/state/schema 边界 | backend 接口边界批次 | 不得迁移 handler 或状态所有权 |
| `markdown/06-milestones/v4.16.0/30-backend九叶模块壳抽离记录.md` backend 九叶壳 | `root.backend` 9 叶子 | `src/backend/`、route facade、保留 handler/state/schema 边界 | backend 九叶抽离批次 | 不得宣称叶子整理或 handler 迁移完成 |
| `markdown/06-milestones/v4.16.0/31-backend.interface_boundary单叶closeout.md` interface boundary closeout | `backend.interface_boundary` | 父级 route facade、兼容桥、停止细分判断 | backend 九叶整理 | 不得把父级 facade 拆成目录美化 |
| `markdown/06-milestones/v4.16.0/32-backend.capability单叶closeout.md` capability closeout | `backend.capability` | capability 真源、单一 API facade、停止细分判断 | backend 九叶整理 | 不得由前端静态数组替代 capability 真源 |
| `markdown/06-milestones/v4.16.0/33-backend.strategy_config单叶closeout.md` strategy config closeout | `backend.strategy_config` | artifact/preflight/diff/AI proposal L3 候选 | backend 九叶整理 | 不得迁移 schema 或 capability 语义而不重新提案 |
| `markdown/06-milestones/v4.16.0/34-backend.runtime单叶closeout.md` runtime closeout | `backend.runtime` | run/backtest/mutation/evidence/persistence L3 候选 | backend 九叶整理 | 不得迁移 runtime state owner |
| `markdown/06-milestones/v4.16.0/35-backend.graph_compile单叶closeout.md` graph compile closeout | `backend.graph_compile` | graph/QS/compile/diagnostics L3 候选 | backend 九叶整理 | 不得绕过 graph version 或 compile diagnostics |
| `markdown/06-milestones/v4.16.0/36-backend.storage_security单叶closeout.md` storage security closeout | `backend.storage_security` | credential/storage/auth/safe log L3 候选和安全暂停 | backend 九叶整理 | 不得直接改密钥、认证、quota、原子写或日志清洗语义 |
| `markdown/06-milestones/v4.16.0/37-backend.ops_governance单叶closeout.md` ops governance closeout | `backend.ops_governance` | sandbox/alerts/snapshots/runbook/chaos/hotswap L3 候选 | backend 九叶整理 | 不得横向改 runtime、executor 或 release transition |
| `markdown/06-milestones/v4.16.0/38-backend.app_state_wiring单叶closeout.md` app state wiring closeout | `backend.app_state_wiring` | AppState 工厂、health、attach_state、停止细分判断 | backend 九叶整理 | 不得迁移 AppState 字段 owner 或锁顺序 |
| `markdown/06-milestones/v4.16.0/39-backend.test_support单叶closeout.md` test support closeout | `backend.test_support` | test scenario route、legacy tests、测试资产汰换暂停 | backend 九叶整理 | 不得无替代证据删除旧测试程序 |
| `markdown/06-milestones/v4.16.0/40-backend.strategy_config_L3模块壳抽离记录.md` strategy config L3 shell | `backend.strategy_config` | artifact/preflight/diff/AI proposal binding 子叶 facade | backend L3 抽离 | 不得宣称 handler 或 schema 已迁移 |
| `markdown/06-milestones/v4.16.0/41-backend其余八叶模块壳抽离记录.md` backend eight leaf shell | backend 其余 8 叶 | interface/capability/runtime/graph/storage/ops/state/test 子 facade | backend L3 抽离 | 不得宣称 handler、state、auth/storage 或测试资产已迁移 |
| `markdown/06-milestones/v4.16.0/42-backend.interface_boundary子叶抽离完成记录.md` interface child complete | `backend.interface_boundary` | 8 个 bridge facade、route owner 保留 | BE-001E 逐叶完成 | 不得宣称 route owner、handler、schema 或 AppState 已迁移 |
| `markdown/06-milestones/v4.16.0/43-backend.capability子叶抽离完成记录.md` capability child complete | `backend.capability` | capability snapshot facade、capability 真源保留 | BE-001E 逐叶完成 | 不得以前端静态数组替代 capability 真源 |
| `markdown/06-milestones/v4.16.0/44-backend.runtime子叶抽离完成记录.md` runtime child complete | `backend.runtime` | runtime routes facade、runtime handler/state 保留 | BE-001E 逐叶完成 | 不得宣称 runtime state owner、event stream 或 persistence 已迁移 |
| `markdown/06-milestones/v4.16.0/45-backend.graph_compile子叶抽离完成记录.md` graph compile child complete | `backend.graph_compile` | compile/graph/QS route facade、diagnostics 保留 | BE-001E 逐叶完成 | 不得宣称 compile/graph handler 已迁移 |
| `markdown/06-milestones/v4.16.0/46-backend.storage_security子叶抽离完成记录.md` storage security child complete | `backend.storage_security` | credential API/vault facade、安全暂停保留 | BE-001E 逐叶完成 | 不得迁移 auth、storage、safe log、backup 或密钥语义 |
| `markdown/06-milestones/v4.16.0/47-backend.ops_governance子叶抽离完成记录.md` ops governance child complete | `backend.ops_governance` | sandbox/alerts/snapshots/runbook/chaos/hotswap route facade | BE-001E 逐叶完成 | 不得横向改 runtime、executor 或 release transition |
| `markdown/06-milestones/v4.16.0/48-backend.app_state_wiring子叶抽离完成记录.md` app state child complete | `backend.app_state_wiring` | health/state factory facade、AppState owner 保留 | BE-001E 逐叶完成 | 不得迁移 AppState 字段 owner 或锁顺序 |
| `markdown/06-milestones/v4.16.0/49-backend.test_support子叶抽离完成记录.md` test support child complete | `backend.test_support` | test scenario facade、旧测试程序保留 | BE-001E 逐叶完成 | 不得启动测试资产汰换或删除旧测试 |
| `markdown/06-milestones/v4.16.0/50-backend.runtime.routes单子叶等价基线.md` runtime routes baseline | `backend.runtime.routes` | runtime route aggregate facade、真实 runtime owner 和回归证据 | BE-001F 单子叶基线 | 不得迁移 run/backtest/mutation handler、event stream、state owner 或 persistence |
| `markdown/06-milestones/v4.16.0/51-backend.runtime.routes抽离记录.md` runtime routes extraction | `backend.runtime.routes` | runtime route aggregate 列表迁入 backend facade，handler 保留原位 | BE-001F 单子叶抽离 | 不得宣称 run/backtest/mutation handler、event stream、state owner 或 persistence 已迁移 |
| `markdown/06-milestones/v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md` runtime run routes baseline | `backend.runtime.routes.run` | run route group facade、event stream 排除边界和回归证据 | BE-001G 单子叶基线 | 不得迁移 run handler、state owner、event stream 或 persistence |
| `markdown/06-milestones/v4.16.0/53-backend.runtime.routes.run抽离记录.md` runtime run routes extraction | `backend.runtime.routes.run` | run route group 迁入 backend route child facade，handler 保留原位 | BE-001G 单子叶抽离 | 不得宣称 run handler、event stream、state owner 或 persistence 已迁移 |
| `markdown/06-milestones/v4.16.0/54-backend.runtime.routes.run单叶closeout.md` runtime run routes closeout | `backend.runtime.routes.run` | route facade closeout、handler 层继续细拆判断 | BE-001G 单叶 closeout | 不得把 route facade closeout 宣称为 run handler 完成 |
| `markdown/06-milestones/v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md` runtime run v4 handoff baseline | `runtime.run.v4_handoff` | `/api/runtime/v4/run` handler 层等价基线 | BE-001H 单子叶基线 | 不得移动 handler 或扩大 provider 支持 |
| `markdown/06-milestones/v4.16.0/56-runtime.run.v4_handoff抽离记录.md` runtime run v4 handoff extraction | `runtime.run.v4_handoff` | v4 handoff handler/type/helper 迁入 `src/runtime/run/v4_handoff.rs`，父级保留受控出口 | BE-001H 单子叶抽离 | 不得宣称 run handler 全部完成或 provider 真连接可用 |
| `markdown/06-milestones/v4.16.0/57-runtime.run.v4_handoff单叶closeout.md` runtime run v4 handoff closeout | `runtime.run.v4_handoff` | 单叶整理、等价证据和停止内部细分判断 | BE-001H 单叶 closeout | 不得继续细拆本叶或宣称 run handler 全部完成 |
| `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md` runtime run session start baseline | `runtime.run.session_start` | legacy `/api/runtime/test-run` handler 层等价基线 | BE-001I 单子叶基线 | 不得迁移 `start_test_run`、state owner、record/replay/SSE 或 persistence |
| `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md` runtime run session start extraction | `runtime.run.session_start` | `start_test_run` 迁入 `src/runtime/run/session_start.rs`，父级保留受控出口 | BE-001I 单子叶抽离 | 不得宣称 record/replay/SSE、state owner、persistence 或本叶 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md` runtime run session start closeout | `runtime.run.session_start` | 单叶整理、等价证据和停止内部细分判断 | BE-001I 单叶 closeout | 不得继续细拆本叶或宣称 record/replay/SSE、state owner、persistence 已完成 |
| `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md` runtime run record store baseline | `runtime.run.record_store` | run record list/detail/save/discard handler 层等价基线 | BE-001J 单子叶基线 | 不得迁移 replay/status/SSE、state owner 或 persistence owner |

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
