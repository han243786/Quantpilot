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
**最新状态补充**: BE-001BZ-01 `backend.runtime.routes` 第六轮父叶残余判断已完成并设置 `stop_split: true`；route aggregate 当前只聚合 run/backtest/event_stream/evidence/mutation/experiment/report_ops 七个 route child。下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断，不能从 route aggregate 内继续细拆。AI proposal、approval、`AppState`、锁顺序、schema、frontend caller 和发布过渡均未迁移。
**状态**: v4.16 BE-001H-03 `runtime.run.v4_handoff` 已完成单叶 closeout，当前不继续细拆；BE-001I-03 `runtime.run.session_start` 已完成单叶 closeout，当前不继续细拆；BE-001J-05 `runtime.run.record_store` 已完成抽离与单叶 closeout，当前不继续细拆；BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，当前不继续细拆；BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，当前不继续细拆；BE-001M-04 `runtime.backtest` 已完成 route facade 抽离与单叶 closeout，route facade 本身停止细分；BE-001N-04 `runtime.backtest.execution_start` 已完成第一轮物理抽离与单叶 closeout；BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout 并设置 `stop_split: true`；BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`；BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`；BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`；BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断；BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`；BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`；BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout 并设置 `stop_split: false`；BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`；BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断；BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout并设置 `stop_split: false`，BE-001AH-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout并设置 `stop_split: true`，下一步只能进入 BE-001AI-01 父叶残余判断。runtime route aggregate 已迁入 `src/backend/runtime/routes.rs`，run route group 已迁入 `src/backend/runtime/routes/run.rs`，backtest route group 已迁入 `src/backend/runtime/routes/backtest.rs`；`/api/runtime/v4/run` handler 已迁入 `src/runtime/run/v4_handoff.rs`，legacy `/api/runtime/test-run` handler 已迁入 `src/runtime/run/session_start.rs`，run record list/detail/save/discard handler 已迁入 `src/runtime/run/record_store.rs`，replay/status handler 已迁入 `src/runtime/run/replay_status.rs`，SSE handler 已迁入 `src/runtime/event_stream.rs`，backtest 创建路径 handler/helper 已迁入 `src/runtime/backtest/execution_start.rs`，v4 projection helper 已迁入 `src/runtime/backtest/v4_projection.rs`，v4 request resolution helper 已迁入 `src/runtime/backtest/v4_request_resolution.rs`，v4 runtime execution helper 已迁入 `src/runtime/backtest/v4_runtime_execution.rs`，legacy dispatch helper 已迁入 `src/runtime/backtest/legacy_dispatch.rs`，backtest record store handler 已迁入 `src/runtime/backtest/record_store.rs`，backtest replay handler 已迁入 `src/runtime/backtest/replay.rs`，backtest experiment sweep handler/helper 已迁入 `src/runtime/backtest/experiment_sweep.rs`，parameter_grid helper 已迁入 `src/runtime/backtest/parameter_grid.rs`，start_orchestration handler 已迁入 `src/runtime/backtest/start_orchestration.rs`，record_lifecycle handler 已迁入 `src/runtime/backtest/record_lifecycle.rs`，transition_lifecycle handler/helper 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`，boundary_safety helper 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`，backtest artifact/compare/persistence 仍保留原 owner，state/shared helper 仍保留在 `src/runtime/`。
**最新状态补充（BE-001CA-01）**: BE-001CA-01 已完成 `backend.runtime` 父叶残余判断。`backend.runtime.routes` 已设置 `stop_split: true`，但 `src/runtime/mod.rs` 仍持有 report/evidence/ops handler 与 helper 残余，因此 `backend.runtime` 保持 `stop_split: false`。下一步只能进入 BE-001CB-01 `runtime.report_ops` 单子叶等价基线，不得直接创建 child 文件、迁移 handler 或启动发布过渡。
**最新状态补充（BE-001CB-01）**: BE-001CB-01 已建立 `runtime.report_ops` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，runtime report / v1 ops report handler 仍在 `src/runtime/mod.rs`。下一步只能进入 BE-001CB-02 抽离方案，不得迁移 handler、扩大 v1 ops/report endpoint 测试缺口或启动发布过渡。
**最新状态补充（BE-001CB-02）**: BE-001CB-02 已建立 `runtime.report_ops` 抽离方案。当前 `no code movement`，下一步只能进入 BE-001CB-03 实际抽离；允许迁移清单仅限 report helper 与十个 public handler，不得处理 `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。
**最新状态补充（BE-001CB-03）**: BE-001CB-03 已完成 `runtime.report_ops` 实际抽离。`src/runtime/report_ops.rs` 已创建并承接十个 public handler 与四个 private helper；`src/runtime/mod.rs` 只保留受控 re-export。下一步只能进入 BE-001CB-04 单叶 closeout，不得跳过 closeout 处理 `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。
**最新状态补充（BE-001CB-04）**: BE-001CB-04 已完成 `runtime.report_ops` 单叶 closeout。第一轮抽离等价成立，但 `runtime.report_ops stop_split: false`；下一步只能进入 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线，不得直接创建 child 文件、迁移 handler、处理 v1 ops/report endpoints 或启动 release transition。
**最新状态补充（BE-001CC-01）**: BE-001CC-01 已建立 `runtime.report_ops.runtime_report` 单子叶等价基线。当前 `no code movement`，runtime_report planned child 文件尚未创建，runtime report 四个 public handler 与四个 private helper 仍在 `src/runtime/report_ops.rs`。下一步只能进入 BE-001CC-02 抽离方案。
**最新状态补充（BE-001CC-02）**: BE-001CC-02 已建立 `runtime.report_ops.runtime_report` 抽离方案。当前 `no code movement`，下一步只能进入 BE-001CC-03 实际抽离；允许迁移清单仅限四个 runtime report public handler 与四个 private helper。
**最新状态补充（BE-001CC-03）**: BE-001CC-03 已完成 `runtime.report_ops.runtime_report` 实际抽离。`src/runtime/report_ops/runtime_report.rs` 已创建并承接四个 public handler 与四个 private helper；父级 `src/runtime/report_ops.rs` 只保留受控 re-export。下一步只能进入 BE-001CC-04 单叶 closeout。
**最新状态补充（BE-001CC-04）**: BE-001CC-04 已完成 `runtime.report_ops.runtime_report` 单叶 closeout。该 child 等价成立并设置 `runtime.report_ops.runtime_report stop_split: true`；父级 `runtime.report_ops stop_split: false`，下一步只能进入 BE-001CD-01 父叶残余判断。
**最新状态补充（BE-001CD-01）**: BE-001CD-01 已完成 `runtime.report_ops` 父叶残余判断。父级仍保留 v1 report endpoints 与 merge/generation/storage health endpoints，因此 `runtime.report_ops stop_split: false`；下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。
**最新状态补充（BE-001CE-01）**: BE-001CE-01 已建立 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，三个 v1 report handler 仍在 `src/runtime/report_ops.rs`；下一步只能进入 BE-001CE-02 抽离方案。
**最新状态补充（BE-001CE-02）**: BE-001CE-02 已建立 `runtime.report_ops.v1_report_endpoints` test-first 抽离方案。当前 `no code movement`，下一步只能进入 BE-001CE-03 endpoint smoke 补测，不能创建 child module 或迁移 handler。
**最新状态补充（BE-001CE-03）**: BE-001CE-03 已完成 `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测。`tests/api_v1_reports.rs` 已创建并覆盖三条 `/api/v1/reports/*` 基础 JSON contract；当前仍未创建 child module、未迁移 handler，下一步只能进入 BE-001CE-04 实际抽离。
**最新状态补充（BE-001CE-04）**: BE-001CE-04 已完成 `runtime.report_ops.v1_report_endpoints` 实际抽离。`src/runtime/report_ops/v1_report_endpoints.rs` 已创建并承接三个 v1 report handler；父级只保留受控 re-export，下一步只能进入 BE-001CE-05 单叶 closeout。
**最新状态补充（BE-001CE-05）**: BE-001CE-05 已完成 `runtime.report_ops.v1_report_endpoints` 单叶 closeout。该 child 设置 `stop_split: true`；父级 `runtime.report_ops stop_split: false`，下一步只能进入 BE-001CF-01 父叶残余判断。
**最新状态补充（BE-001CF-01）**: BE-001CF-01 已完成 `runtime.report_ops` 父叶残余判断。`runtime_report` 与 `v1_report_endpoints` 两个 child 均已 closeout，但父级仍保留 merge/generation/storage health handler，因此 `runtime.report_ops stop_split: false`；下一步只能进入 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线。
**最新状态补充（BE-001CG-01）**: BE-001CG-01 已建立 `runtime.report_ops.merge_generation_health` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`；下一步只能进入 BE-001CG-02 抽离方案。
**最新状态补充（BE-001CG-02）**: BE-001CG-02 已建立 `runtime.report_ops.merge_generation_health` test-first 抽离方案。当前 `no code movement`，下一步只能进入 BE-001CG-03 endpoint smoke 补测；BE-001CG-04 才允许创建 planned child 并迁移三个 handler。
**最新状态补充（BE-001CG-03）**: BE-001CG-03 已完成 `runtime.report_ops.merge_generation_health` endpoint smoke 补测。`tests/api_v1_ops_health.rs` 已创建并覆盖三条 v1 support/health endpoint 的基础 JSON contract；planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CG-04 实际抽离。
**最新状态补充（BE-001CG-04）**: BE-001CG-04 已完成 `runtime.report_ops.merge_generation_health` 实际抽离。`src/runtime/report_ops/merge_generation_health.rs` 已创建并承接 `list_merge_records`、`list_config_generations`、`get_storage_health`；父级只保留受控 re-export，下一步只能进入 BE-001CG-05 单叶 closeout。
**最新状态补充（BE-001CG-05）**: BE-001CG-05 已完成 `runtime.report_ops.merge_generation_health` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。
**最新状态补充（BE-001CH-01）**: BE-001CH-01 已完成 `runtime.report_ops` 第二轮父叶残余判断。三个 report_ops child 均已 closeout，父级只保留受控 re-export，因此设置 `runtime.report_ops stop_split: true`；下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断。
**最新状态补充（BE-001CI-01）**: BE-001CI-01 已完成 `backend.runtime` 第二轮父叶残余判断。`backend.runtime.routes` 与 `runtime.report_ops` 均已 closeout，但 `src/runtime/mod.rs` 仍直接持有 evidence health handler 残余，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
**第二轮父叶残余判断（BE-001CI-01）**: `backend.runtime.routes stop_split: true` 与 `runtime.report_ops stop_split: true` 已成立，但 `src/runtime/mod.rs` 仍直接持有 `get_runtime_evidence_health`、`cleanup_runtime_evidence` 与 `runtime_report_status_counts`。本轮不迁移 handler、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、shared helpers 或 release transition guard；`backend.runtime stop_split: false`，下一步固定为 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
**最新状态补充（BE-001CJ-01）**: BE-001CJ-01 已建立 `runtime.evidence_health` 单子叶等价基线。当前 `no code movement`，planned child 名称 `evidence_health` 尚未落地，两个 evidence handler 与 `runtime_report_status_counts` 仍在 `src/runtime/mod.rs`；下一步只能进入 BE-001CJ-02 抽离方案。
**最新状态补充（BE-001CJ-02）**: BE-001CJ-02 已建立 `runtime.evidence_health` 抽离方案。当前 `no code movement`，下一步 BE-001CJ-03 才允许落地 `evidence_health` child，并且迁移清单仅限 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence`。
**最新状态补充（BE-001CJ-03）**: BE-001CJ-03 已完成 `runtime.evidence_health` 实际抽离。`src/runtime/evidence_health.rs` 已创建并承接两个 evidence handler 与 `runtime_report_status_counts`，父级只保留受控 re-export；下一步只能进入 BE-001CJ-04 单叶 closeout。
**最新状态补充（BE-001CJ-04）**: BE-001CJ-04 已完成 `runtime.evidence_health` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断，不得从本叶继续细拆 health / cleanup 微叶。
**最新状态补充（BE-001CK-01）**: BE-001CK-01 已完成 `backend.runtime` 第三轮父叶残余判断。`backend.runtime.routes`、`runtime.report_ops` 与 `runtime.evidence_health` 均已 closeout，但 src/runtime/mutation.rs (retired drained include) 仍持有 mutation shared governance helper 残余，且父级仍有 query/guard/response support 残余，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。
**最新状态补充（BE-001CL-01）**: BE-001CL-01 已建立 `runtime.mutation.shared_governance` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，9 个 shared governance helper 仍在 src/runtime/mutation.rs (retired drained include)；下一步只能进入 BE-001CL-02 抽离方案。
**最新状态补充（BE-001CL-02）**: BE-001CL-02 已建立 `runtime.mutation.shared_governance` 抽离方案。当前 `no code movement`，planned child 文件尚未创建，9 个 shared governance helper 仍在 src/runtime/mutation.rs (retired drained include)；下一步只能进入 BE-001CL-03 实际抽离。
**最新状态补充（BE-001CL-03）**: BE-001CL-03 已完成 `runtime.mutation.shared_governance` 实际抽离。`src/runtime/mutation/shared_governance.rs` 已创建并迁入 9 个 shared governance helper，父级只保留受控 child 声明与 caller-facing plain import；下一步只能进入 BE-001CL-04 单叶 closeout。
**最新状态补充（BE-001CL-04）**: BE-001CL-04 已完成 `runtime.mutation.shared_governance` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001CM-01 `backend.runtime` 第四轮父叶残余判断，不得从本叶继续细拆 validation / event contract / governance projection 微叶。
**最新状态补充（BE-001CM-01）**: BE-001CM-01 已完成 `backend.runtime` 第四轮父叶残余判断。`runtime.mutation.shared_governance` 已 closeout，但父级仍有 query DTO、run guard、response support 与 experiment limit 残余，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CN-01 `runtime.query_support` 单子叶等价基线。
**最新状态补充（BE-001CN-01）**: BE-001CN-01 已建立 `runtime.query_support` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，7 个 query DTO 与 filter / replay normalization helper 仍在 `src/runtime/mod.rs`、src/runtime/run.rs (retired drained include)、src/runtime/mutation.rs (retired drained include)；下一步只能进入 BE-001CN-02 抽离方案。
**最新状态补充（BE-001CN-02）**: BE-001CN-02 已建立 `runtime.query_support` 抽离方案。当前 `no code movement`，下一步 BE-001CN-03 才允许创建 planned child 文件并迁移 7 个 query DTO、`clean_optional_filter` 与 `normalized_replay_options`。
**最新状态补充（BE-001CN-03）**: BE-001CN-03 已完成 `runtime.query_support` 实际抽离。`src/runtime/query_support.rs` 已创建并迁入 7 个 Query DTO、`clean_optional_filter` 与 `normalized_replay_options`；下一步只能进入 BE-001CN-04 单叶 closeout。
**最新状态补充（BE-001CN-04）**: BE-001CN-04 已完成 `runtime.query_support` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断，不得从本叶继续细拆 replay/mutation/report query 或 normalization 微叶。
**最新状态补充（BE-001CO-01）**: BE-001CO-01 已完成 `backend.runtime` 第五轮父叶残余判断。`runtime.query_support` 已 closeout，但父级仍有 response support、run guard、experiment limit 与 parent include residual，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CP-01 `runtime.response_support` 单子叶等价基线。
**最新状态补充（BE-001CP-01）**: BE-001CP-01 已建立 `runtime.response_support` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，`DiscardRuntimeArtifactResponse` 仍在 `src/runtime/mod.rs`，`MergeRecordsResponse` 与 `MergeRecordEntry` 仍在 src/runtime/run.rs (retired drained include)；下一步只能进入 BE-001CP-02 抽离方案。
**最新状态补充（BE-001CP-02）**: BE-001CP-02 已建立 `runtime.response_support` 抽离方案。当前 `no code movement`，下一步 BE-001CP-03 才允许创建 response_support child 并迁移 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 与 `MergeRecordEntry`。
**最新状态补充（BE-001CP-03）**: BE-001CP-03 已完成 `runtime.response_support` 实际抽离。`src/runtime/response_support.rs` 已创建并迁入 3 个 response DTO，父级只保留 plain import，src/runtime/run.rs (retired drained include) 已降为 drained include 注释；下一步只能进入 BE-001CP-04 单叶 closeout。
**最新状态补充（BE-001CP-04）**: BE-001CP-04 已完成 `runtime.response_support` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断，不得从 response_support 继续细拆。
**最新状态补充（BE-001CQ-01）**: BE-001CQ-01 已完成 `backend.runtime` 第六轮父叶残余判断。`runtime.response_support` 已 closeout，但父级仍有 run guard、experiment limit 与 drained parent include residual，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CR-01 `runtime.run_guard` 单子叶等价基线。
**最新状态补充（BE-001CR-01）**: BE-001CR-01 已建立 `runtime.run_guard` 单子叶等价基线。当前 `no code movement`，planned child 文件尚未创建，`RunInProgressGuard` 仍在 `src/runtime/mod.rs`；下一步只能进入 BE-001CR-02 抽离方案。
**最新状态补充（BE-001CR-02）**: BE-001CR-02 已建立 `runtime.run_guard` 抽离方案。当前 `no code movement`，方案选择不单独开 test-first 批次，planned child 文件尚未创建；下一步只能进入 BE-001CR-03 实际抽离。
**最新状态补充（BE-001CR-03）**: BE-001CR-03 已完成 `runtime.run_guard` 实际抽离。`src/runtime/run_guard.rs` 已创建并承接 `RunInProgressGuard` 与 Drop impl；下一步只能进入 BE-001CR-04 单叶 closeout。
**最新状态补充（BE-001CR-04）**: BE-001CR-04 已完成 `runtime.run_guard` 单叶 closeout，并设置 `runtime.run_guard stop_split: true`。下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。
**最新状态补充（BE-001CS-01）**: BE-001CS-01 已完成 `backend.runtime` 第七轮父叶残余判断。`runtime.run_guard stop_split: true` 已成立，但父级仍有 `MAX_EXPERIMENT_VARIANTS` 与 `include!("run.rs")` / `include!("mutation.rs")` / `include!("backtest.rs")` drained parent include residual，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线。
**最新状态补充（BE-001CT-01）**: BE-001CT-01 已建立 `runtime.experiment_limit` 单子叶等价基线。当前 `no code movement`，只冻结 `MAX_EXPERIMENT_VARIANTS = 27`、`src/runtime/backtest/parameter_grid.rs` 唯一调用方、`variant_count` guard、bad_request 输出与 planned child 门禁；下一步只能进入 BE-001CT-02 抽离方案，不得直接迁移常量、删除 parent include 或启动发布过渡。
**最新状态补充（BE-001CT-02）**: BE-001CT-02 已建立 `runtime.experiment_limit` test-first 抽离方案。当前 `no code movement`，下一批只补 `tests/api_experiments.rs` 超限负测；实际迁移必须等 BE-001CT-03 通过并提交后进入 BE-001CT-04，不得直接创建 planned child、删除 parent include 或启动发布过渡。
**最新状态补充（BE-001CT-03）**: BE-001CT-03 已完成 `runtime.experiment_limit` endpoint smoke 补测。`tests/api_experiments.rs` 已新增 36 个变体超过 27 上限的 bad_request 负测；当前仍未迁移 `MAX_EXPERIMENT_VARIANTS`，下一步只能进入 BE-001CT-04 实际抽离。
**最新状态补充（BE-001CT-04）**: BE-001CT-04 已完成 `runtime.experiment_limit` 实际抽离。`src/runtime/experiment_limit.rs` 已创建并承接 `MAX_EXPERIMENT_VARIANTS = 27`；父级只保留 `mod experiment_limit` 与 plain import，下一步只能进入 BE-001CT-05 单叶 closeout。
**最新状态补充（BE-001CT-05）**: BE-001CT-05 已完成 `runtime.experiment_limit` 单叶 closeout，并设置 `runtime.experiment_limit stop_split: true`。下一步只能进入 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断，不得从 experiment_limit 继续细拆。
**最新状态补充（BE-001CU-01）**: BE-001CU-01 已完成 `backend.runtime` 第八轮父叶残余判断。`runtime.experiment_limit stop_split: true` 已成立，父级真实残余只剩 `include!("run.rs")`、`include!("mutation.rs")` 与 `include!("backtest.rs")` drained parent include cleanup，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线。
**最新状态补充（BE-001CV-01）**: BE-001CV-01 已建立 `runtime.parent_include_cleanup` 单子叶等价基线。当前 `no code movement`，只冻结三条 drained `include!(...)`、三个 drained 文件、public 出口影响面和等价验证门禁；下一步只能进入 BE-001CV-02 抽离方案。
**最新状态补充（BE-001CV-02）**: BE-001CV-02 已建立 `runtime.parent_include_cleanup` 抽离方案。当前 `no code movement`，BE-001CV-03 只允许删除三条 drained `include!(...)` 与三个 drained 文件，不得处理 `backend.runtime` 父叶 closeout、schema/frontend/state/persistence owner 或 release transition。
**最新状态补充（BE-001CV-03）**: BE-001CV-03 已完成 `runtime.parent_include_cleanup` 实际 cleanup。`src/runtime/mod.rs` 中三条 drained `include!(...)` 已删除，三个 drained 文件已删除；public handler owner、route facade、schema/frontend/state/persistence owner、lock order 与 release transition guard 均未处理。下一步只能进入 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断。
**最新状态补充（BE-001CW-01）**: BE-001CW-01 已完成 `backend.runtime` 第九轮父叶残余判断。`src/runtime/mod.rs` 已无行为体和 drained include，但仍作为 parent import bridge 被大量 child `use super::*` 依赖，因此 `backend.runtime stop_split: false`；下一步只能进入 BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线。
**最新状态补充（BE-001CX-01）**: BE-001CX-01 已建立 `runtime.parent_import_bridge` 单子叶等价基线。当前 `no code movement`，冻结 46 个 runtime 文件对 parent import bridge 的 `use super::*` / `super::` 依赖面；下一步只能进入 BE-001CX-02 抽离方案，不能直接批量改写 Rust import 或启动 release transition。
**最新状态补充（BE-001CX-02）**: BE-001CX-02 已建立 `runtime.parent_import_bridge` 抽离方案。当前 `no code movement`，后续采用 staged explicit import pass；下一步只能进入 BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离，且首批只处理 `query_support` 与 `response_support`。
**最新状态补充（BE-001CX-03）**: BE-001CX-03 已完成 `runtime.root_support_import_pilot` 实际抽离。`src/runtime/query_support.rs` 与 `src/runtime/response_support.rs` 已从 `use super::*` 收敛为显式 import，runtime parent bridge 依赖文件数从 46 降为 44；下一步只能进入 BE-001CX-04 单叶 closeout。
**最新状态补充（BE-001CX-04）**: BE-001CX-04 已完成 `runtime.root_support_import_pilot` 单叶 closeout。`runtime.root_support_import_pilot stop_split: true`，不继续拆 `query_support` / `response_support` 微叶；下一步只能进入 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线。
**最新状态补充（BE-001CY-01）**: BE-001CY-01 已建立 `runtime.root_entry_import_pass` 单子叶等价基线。候选范围为 `event_stream`、`evidence_health`、`report_ops`、`src/runtime/run_guard.rs` 与 `src/runtime/mod.rs`，其中 `src/runtime/run_guard.rs` 的 `use super::*` 为 test-only super import；下一步只能进入 BE-001CY-02 抽离方案。
**最新状态补充（BE-001CY-02）**: BE-001CY-02 已建立 `runtime.root_entry_import_pass` 抽离方案。BE-001CY-03 只允许把 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` 的 parent wildcard import 改为显式 import；`src/runtime/report_ops.rs` 将另起 `runtime.report_ops_import_pass`，test-only `src/runtime/run_guard.rs` 与 `src/runtime/mod.rs` 父桥均不得混批处理。
**最新状态补充（BE-001CY-03）**: BE-001CY-03 已完成 `runtime.root_entry_import_pass` 实际抽离。`src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` 已从 `use super::*` 收敛为显式 import，runtime parent bridge 依赖文件数从 44 降为 42；下一步只能进入 BE-001CY-04 单叶 closeout。
**最新状态补充（BE-001CY-04）**: BE-001CY-04 已完成 `runtime.root_entry_import_pass` 单叶 closeout。`runtime.root_entry_import_pass stop_split: true`，不继续拆 `event_stream` / `evidence_health` 微叶；下一步只能进入 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线。
**最新状态补充（BE-001CZ-01）**: BE-001CZ-01 已建立 `runtime.report_ops_import_pass` 单子叶等价基线。候选范围为 `src/runtime/report_ops.rs`、`src/runtime/report_ops/runtime_report.rs`、`src/runtime/report_ops/v1_report_endpoints.rs` 与 `src/runtime/report_ops/merge_generation_health.rs`，且 parent facade 存在 transitive parent surface risk；下一步只能进入 BE-001CZ-02 抽离方案。
**最新状态补充（BE-001CZ-02）**: BE-001CZ-02 已建立 `runtime.report_ops_import_pass` 抽离方案。下一步 BE-001CZ-03 只允许同批处理 report_ops four-file pocket，不得混入 `src/runtime/mod.rs`、test-only `src/runtime/run_guard.rs`、run/backtest/mutation 子树或 release transition。
**最新状态补充（BE-001CZ-03）**: BE-001CZ-03 已完成 `runtime.report_ops_import_pass` 实际抽离。`src/runtime/report_ops.rs` 与 3 个 report_ops child 已从 `use super::*` 收敛为显式 import，runtime parent bridge 依赖文件数从 42 降为 38；下一步只能进入 BE-001CZ-04 单叶 closeout。
**最新状态补充（BE-001CZ-04）**: BE-001CZ-04 已完成 `runtime.report_ops_import_pass` 单叶 closeout。`runtime.report_ops_import_pass stop_split: true`，report_ops import pass 不继续拆微叶；下一步只能进入 BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断。
**最新状态补充（BE-001DA-01）**: BE-001DA-01 已完成 `runtime.parent_import_bridge` 父叶残余判断。`runtime.parent_import_bridge stop_split: false`，当前剩余 38 个 parent bridge 依赖文件；下一步只能进入 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DB-01）**: BE-001DB-01 已建立 `runtime.run_import_pass` 单子叶等价基线。候选范围固定为 4 个 `src/runtime/run/**` child；下一步只能进入 BE-001DB-02 抽离方案。
**最新状态补充（BE-001DB-02）**: BE-001DB-02 已建立 `runtime.run_import_pass` 抽离方案。BE-001DB-03 只允许同批改写 4 个 run child 的 explicit import；不得混入 root parent bridge、backtest/mutation 子树或 release transition。
**最新状态补充（BE-001DB-03）**: BE-001DB-03 已完成 `runtime.run_import_pass` 实际抽离。4 个 run child 已从 `use super::*` 收敛为显式 import，runtime parent bridge 依赖文件数从 38 降为 34；下一步只能进入 BE-001DB-04 单叶 closeout。
**最新状态补充（BE-001DB-04）**: BE-001DB-04 已完成 `runtime.run_import_pass` 单叶 closeout。`runtime.run_import_pass stop_split: true`，run import pass 不继续拆微叶；下一步只能进入 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断。
**最新状态补充（BE-001DC-01）**: BE-001DC-01 已完成 `runtime.parent_import_bridge` 父叶残余判断。`runtime.parent_import_bridge stop_split: false`，当前剩余 34 个 parent bridge 依赖文件，分布为 root 1、run 0、backtest 11、mutation 21、test-only 1；下一步只能进入 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DD-01）**: BE-001DD-01 已建立 `runtime.backtest_import_pass` 单子叶等价基线。冻结 11 个 `src/runtime/backtest/**` 残余文件、public/internal 方法和父级输入面；下一步只能进入 BE-001DD-02 抽离方案。
**最新状态补充（BE-001DD-02）**: BE-001DD-02 已建立 `runtime.backtest_import_pass` 抽离方案。`runtime.backtest_import_pass stop_split: false`，不采用 11 文件整批 import rewrite；下一步只能进入 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DE-01）**: BE-001DE-01 已建立 `runtime.backtest.record_store_import_pass` 单子叶等价基线。冻结 `src/runtime/backtest/record_store.rs` 的 4 个 public 方法、`use super::*` 残余和预期显式输入面；下一步只能进入 BE-001DE-02 抽离方案。
**最新状态补充（BE-001DE-02）**: BE-001DE-02 已建立 `runtime.backtest.record_store_import_pass` 抽离方案。BE-001DE-03 只允许改写 `src/runtime/backtest/record_store.rs` 顶部 import；不得混入 replay、experiment、execution_start、root bridge 或 release transition。
**最新状态补充（BE-001DE-03）**: BE-001DE-03 已完成 `runtime.backtest.record_store_import_pass` 实际抽离。`src/runtime/backtest/record_store.rs` 已删除 `use super::*` 并改为显式 import，runtime parent bridge 依赖文件数从 34 降为 33；下一步只能进入 BE-001DE-04 单叶 closeout。
**最新状态补充（BE-001DE-04）**: BE-001DE-04 已完成 `runtime.backtest.record_store_import_pass` 单叶 closeout。该 import pocket 设置 `stop_split: true`，不继续拆微叶；下一步只能进入 BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断。
**最新状态补充（BE-001DF-01）**: BE-001DF-01 已完成 `runtime.backtest_import_pass` 父叶残余判断。`runtime.backtest_import_pass stop_split: false`，当前剩余分布为 root 1、run 0、backtest 10、mutation 21、test-only 1；下一步只能进入 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DG-01）**: BE-001DG-01 已建立 `runtime.backtest.replay_import_pass` 单子叶等价基线。冻结 `src/runtime/backtest/replay.rs` 的 1 个 public 方法、`use super::*` 残余和预期显式输入面；下一步只能进入 BE-001DG-02 抽离方案。
**最新状态补充（BE-001DG-02）**: BE-001DG-02 已建立 `runtime.backtest.replay_import_pass` 抽离方案。BE-001DG-03 只允许改写 `src/runtime/backtest/replay.rs` 顶部 import；不得混入 experiment、execution_start、root bridge、mutation 或 release transition。
**最新状态补充（BE-001DG-03）**: BE-001DG-03 已完成 `runtime.backtest.replay_import_pass` 实际抽离。`src/runtime/backtest/replay.rs` 已删除 `use super::*` 并改为显式 import，runtime parent bridge 依赖文件数从 33 降为 32；下一步只能进入 BE-001DG-04 单叶 closeout。
**最新状态补充（BE-001DG-04）**: BE-001DG-04 已完成 `runtime.backtest.replay_import_pass` 单叶 closeout。该 import pocket 设置 `stop_split: true`，不继续拆 replay 微叶；下一步只能进入 BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断。
**最新状态补充（BE-001DH-01）**: BE-001DH-01 已完成 `runtime.backtest_import_pass` 第二轮父叶残余判断。父叶保持 `stop_split: false`，当前剩余分布为 root 1 / run 0 / backtest 9 / mutation 21 / test-only 1 / total 32；下一步只能进入 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DI-01）**: BE-001DI-01 已建立 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。冻结 `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/record_lifecycle.rs`、`src/runtime/backtest/start_orchestration.rs` 四文件 pocket；下一步只能进入 BE-001DI-02 抽离方案。
**最新状态补充（BE-001DI-02）**: BE-001DI-02 已建立 `runtime.backtest.experiment_sweep_import_pass` 抽离方案。BE-001DI-03 只允许改写四文件 import 和必要 visibility；预期 runtime parent bridge 依赖文件数从 32 降为 28。
**最新状态补充（BE-001DI-03）**: BE-001DI-03 已完成 `runtime.backtest.experiment_sweep_import_pass` 实际抽离。四文件 parent import 已收敛，runtime parent bridge 依赖文件数从 32 降为 28；下一步只能进入 BE-001DI-04 单叶 closeout。
**最新状态补充（BE-001DI-04）**: BE-001DI-04 已完成 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout。设置 `runtime.backtest.experiment_sweep_import_pass stop_split: true`，旧的三叶暂停目标取消；下一步只能进入 BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断。
**最新状态补充（BE-001DJ-01）**: BE-001DJ-01 已完成 `runtime.backtest_import_pass` 第三轮父叶残余判断。父叶保持 `runtime.backtest_import_pass stop_split: false`，当前剩余分布为 root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28；下一步只能进入 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DK-01）**: BE-001DK-01 已建立 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。冻结 execution_start 五文件 import pocket、白箱方法和等价风险；下一步只能进入 BE-001DK-02 抽离方案。
**最新状态补充（BE-001DK-02）**: BE-001DK-02 已建立 `runtime.backtest.execution_start_import_pass` 抽离方案。BE-001DK-03 只允许改写五文件 import 和必要 visibility；预期 runtime parent bridge 依赖文件数从 28 降为 23。
**最新状态补充（BE-001DK-03）**: BE-001DK-03 已完成 `runtime.backtest.execution_start_import_pass` 实际抽离。五文件 parent wildcard / super import 已清除，runtime parent bridge 依赖文件数从 28 降为 23，backtest residual 为 0；下一步只能进入 BE-001DK-04 单叶 closeout。
**最新状态补充（BE-001DK-04）**: BE-001DK-04 已完成 `runtime.backtest.execution_start_import_pass` 单叶 closeout。设置 `runtime.backtest.execution_start_import_pass stop_split: true`，当前 parent bridge 剩余 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23；旧的三叶暂停目标保持取消，下一步只能进入 BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断。
**最新状态补充（BE-001DL-01）**: BE-001DL-01 已完成 `runtime.backtest_import_pass` 第四轮父叶残余判断。设置 `runtime.backtest_import_pass stop_split: true`，当前 parent bridge 剩余 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23；下一步只能进入 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断。
**最新状态补充（BE-001DM-01）**: BE-001DM-01 已完成 `runtime.parent_import_bridge` 父叶残余判断。父叶保持 `runtime.parent_import_bridge stop_split: false`，当前 parent bridge 剩余 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23；下一步只能进入 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DN-01）**: BE-001DN-01 已建立 `runtime.mutation_import_pass` 单子叶等价基线。冻结 21 个 mutation parent bridge 文件、AI proposal / parameter mutation / shared governance 白箱输入面和后续拆分候选；下一步只能进入 BE-001DN-02 抽离方案。
**最新状态补充（BE-001DN-02）**: BE-001DN-02 已建立 `runtime.mutation_import_pass` 抽离方案。设置 `runtime.mutation_import_pass stop_split: false`，拒绝 21 文件整批 rewrite；下一步只能进入 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DO-01）**: BE-001DO-01 已建立 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。冻结 `src/runtime/mutation/shared_governance.rs` 的 9 个 helper、当前 `use super::*` 和预期显式 import 输入面；下一步只能进入 BE-001DO-02 抽离方案。
**最新状态补充（BE-001DO-02）**: BE-001DO-02 已建立 `runtime.mutation.shared_governance_import_pass` 抽离方案。BE-001DO-03 只允许单文件改写 `src/runtime/mutation/shared_governance.rs` 顶部 import；下一步只能进入实际抽离记录。
**最新状态补充（BE-001DO-03）**: BE-001DO-03 已完成 `runtime.mutation.shared_governance_import_pass` 实际抽离。`src/runtime/mutation/shared_governance.rs` 已删除 `use super::*` 并改为显式 import，parent bridge 剩余降为 total 22；下一步只能进入 BE-001DO-04 单叶 closeout。
**最新状态补充（BE-001DO-04）**: BE-001DO-04 已完成 `runtime.mutation.shared_governance_import_pass` 单叶 closeout。设置 `runtime.mutation.shared_governance_import_pass stop_split: true`，下一步只能进入 BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断。
**最新状态补充（BE-001DP-01）**: BE-001DP-01 已完成 `runtime.mutation_import_pass` 父叶残余判断。父叶保持 `runtime.mutation_import_pass stop_split: false`，当前 parent bridge 剩余 total 22、mutation 20；下一步只能进入 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DQ-01）**: BE-001DQ-01 已建立 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。冻结 10 个 parameter mutation residual 文件、5 个 public handler、8 个内部 helper；旧的三叶暂停目标继续取消，下一步只能进入 BE-001DQ-02 抽离方案。
**最新状态补充（BE-001DQ-02）**: BE-001DQ-02 已建立 `runtime.mutation.parameter_mutation_import_pass` 抽离方案。设置 `runtime.mutation.parameter_mutation_import_pass stop_split: false`，拒绝 10 文件整批 rewrite；下一步只能进入 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DR-01）**: BE-001DR-01 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。冻结 `src/runtime/mutation/parameter_mutation/record_query.rs` 的 list/detail 读路径输入面；下一步只能进入 BE-001DR-02 抽离方案。
**最新状态补充（BE-001DR-02）**: BE-001DR-02 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案。下一步只允许单文件改写 `src/runtime/mutation/parameter_mutation/record_query.rs` 顶部 import，不触碰 parent facade 或 lifecycle sibling。
**最新状态补充（BE-001DR-03）**: BE-001DR-03 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/record_query.rs` 已删除 `use super::*` 并改为显式 import；parent bridge 剩余 total 21、mutation 19，下一步只能进入 BE-001DR-04 单叶 closeout。
**最新状态补充（BE-001DR-04）**: BE-001DR-04 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout。设置 `runtime.mutation.parameter_mutation.record_query_import_pass stop_split: true`，旧三叶暂停目标保持取消；下一步只能进入 BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
**最新状态补充（BE-001DS-01）**: BE-001DS-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation_import_pass stop_split: false`，当前 parameter_mutation residual 为 9 文件；下一步只能进入 BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DT-01）**: BE-001DT-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线。冻结 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 的 proposal 创建输入面；下一步只能进入 BE-001DT-02 抽离方案。
**最新状态补充（BE-001DT-02）**: BE-001DT-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案。下一步只允许单文件改写 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 顶部 import，不触碰 lifecycle sibling、parent facade、AI proposal、root bridge 或 release transition。
**最新状态补充（BE-001DT-03）**: BE-001DT-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/proposal_creation.rs` 已删除 `use super::*` 并改为显式 import；parent bridge 剩余 total 20、mutation 18，下一步只能进入 BE-001DT-04 单叶 closeout。
**最新状态补充（BE-001DT-04）**: BE-001DT-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout。设置 `runtime.mutation.parameter_mutation.proposal_creation_import_pass stop_split: true`，旧三叶暂停目标保持取消；下一步只能进入 BE-001DU-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
**最新状态补充（BE-001DU-01）**: BE-001DU-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation_import_pass stop_split: false`，当前 parameter_mutation residual 为 8 文件；下一步只能进入 BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DV-01）**: BE-001DV-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线。冻结 transition lifecycle 7 文件、activation / rollback public handler、boundary 父级白箱 helper 与内部 helper 输入面；下一步只能进入 BE-001DV-02 抽离方案。
**最新状态补充（BE-001DV-02）**: BE-001DV-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，拒绝 7 文件同批 rewrite；下一步只能进入 BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线。
**最新状态补充（BE-001DW-01）**: BE-001DW-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线。冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 的三个 helper、当前 `use super::*` residual 与预期显式 import 输入面；下一步只能进入 BE-001DW-02 抽离方案。
**最新状态补充（BE-001DW-02）**: BE-001DW-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案。下一步只允许 BE-001DW-03 单文件改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 顶部 import；不改函数体、可见性、facade、activation / rollback sibling、AI proposal、root bridge 或 release transition。旧三叶暂停目标继续取消。
**最新状态补充（BE-001DW-03）**: BE-001DW-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 已移除 `use super::*` 并改为显式 import；residual 降为 total 19 / mutation 17 / parameter_mutation 7 / transition_lifecycle 6。下一步只能进入 BE-001DW-04 单叶 closeout。
**最新状态补充（BE-001DW-04）**: BE-001DW-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout。设置 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass stop_split: true`，本 import pocket 不继续拆 validation / resolution / safe-window 微叶；下一步只能进入 BE-001DX-01 父叶残余判断。
**最新状态补充（BE-001DX-01）**: BE-001DX-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，当前 transition_lifecycle residual 为 6 文件；下一步只能进入 BE-001DY-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线。
**真实文件**:
- `src/backend/runtime.rs`
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/mod.rs`
- `src/runtime/experiment_limit.rs`
- `src/runtime/query_support.rs`
- `src/runtime/response_support.rs`
- `src/runtime/run_guard.rs`
- `src/runtime/mutation/shared_governance.rs`
- `src/runtime/evidence_health.rs`
- `src/runtime/report_ops.rs`
- `src/runtime/report_ops/runtime_report.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`
- `src/runtime/report_ops/merge_generation_health.rs`
- `src/runtime/event_stream.rs`
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run/record_store.rs`
- `src/runtime/run/replay_status.rs`
- `src/runtime/backtest/execution_start.rs`
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
- `src/runtime_persistence.rs`
- `src/runtime_event_projection.rs`
- `src/runtime_validation.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_diagnostics.rs`
- `src/backtest_compare.rs`
- `src/backtest_artifacts.rs`
- `markdown/06-milestones/v4.16.0/58-runtime.run.session_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/59-runtime.run.session_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/60-runtime.run.session_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md`
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md`
- `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md`
- `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md`
- `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md`
- `markdown/06-milestones/v4.16.0/70-runtime.event_stream单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/71-runtime.event_stream抽离方案.md`
- `markdown/06-milestones/v4.16.0/72-runtime.event_stream抽离记录.md`
- `markdown/06-milestones/v4.16.0/73-runtime.event_stream单叶closeout.md`
- `markdown/06-milestones/v4.16.0/74-runtime.backtest单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/75-runtime.backtest抽离方案.md`
- `markdown/06-milestones/v4.16.0/76-runtime.backtest抽离记录.md`
- `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md`
- `markdown/06-milestones/v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/79-runtime.backtest.execution_start抽离方案.md`
- `markdown/06-milestones/v4.16.0/80-runtime.backtest.execution_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md`
- `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`
- `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`
- `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`
- `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md`
- `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`
- `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md`
- `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md`
- `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/100-runtime.backtest.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/101-runtime.backtest.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/102-runtime.backtest.record_store单叶closeout.md`
- `markdown/06-milestones/v4.16.0/103-runtime.backtest.replay单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/104-runtime.backtest.replay抽离方案.md`
- `markdown/06-milestones/v4.16.0/105-runtime.backtest.replay抽离记录.md`
- `markdown/06-milestones/v4.16.0/106-runtime.backtest.replay单叶closeout.md`
- `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md`
- `markdown/06-milestones/v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md`
- `markdown/06-milestones/v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md`
- `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`
- `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md`
- `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`
- `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`
- `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md`
- `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`
- `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/127-backend.runtime.routes父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/251-backend.runtime父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/252-runtime.report_ops单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/253-runtime.report_ops抽离方案.md`
- `markdown/06-milestones/v4.16.0/254-runtime.report_ops抽离记录.md`
- `markdown/06-milestones/v4.16.0/255-runtime.report_ops单叶closeout.md`
- `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md`
- `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md`
- `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md`
- `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md`
- `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md`
- `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md`
- `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md`
- `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md`
- `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md`
- `markdown/06-milestones/v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/273-backend.runtime第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/274-runtime.evidence_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/275-runtime.evidence_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/276-runtime.evidence_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/277-runtime.evidence_health单叶closeout.md`
- `markdown/06-milestones/v4.16.0/278-backend.runtime第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/279-runtime.mutation.shared_governance单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/280-runtime.mutation.shared_governance抽离方案.md`
- `markdown/06-milestones/v4.16.0/281-runtime.mutation.shared_governance抽离记录.md`
- `markdown/06-milestones/v4.16.0/282-runtime.mutation.shared_governance单叶closeout.md`
- `markdown/06-milestones/v4.16.0/283-backend.runtime第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/284-runtime.query_support单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/285-runtime.query_support抽离方案.md`
- `markdown/06-milestones/v4.16.0/286-runtime.query_support抽离记录.md`
- `markdown/06-milestones/v4.16.0/287-runtime.query_support单叶closeout.md`
- `markdown/06-milestones/v4.16.0/288-backend.runtime第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/289-runtime.response_support单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/290-runtime.response_support抽离方案.md`
- `markdown/06-milestones/v4.16.0/291-runtime.response_support抽离记录.md`
- `markdown/06-milestones/v4.16.0/292-runtime.response_support单叶closeout.md`
- `markdown/06-milestones/v4.16.0/293-backend.runtime第六轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/294-runtime.run_guard单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/295-runtime.run_guard抽离方案.md`
- `markdown/06-milestones/v4.16.0/296-runtime.run_guard抽离记录.md`
- `markdown/06-milestones/v4.16.0/297-runtime.run_guard单叶closeout.md`
- `markdown/06-milestones/v4.16.0/298-backend.runtime第七轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/299-runtime.experiment_limit单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/300-runtime.experiment_limit抽离方案.md`
- `markdown/06-milestones/v4.16.0/301-runtime.experiment_limit补测记录.md`
- `markdown/06-milestones/v4.16.0/302-runtime.experiment_limit抽离记录.md`
- `markdown/06-milestones/v4.16.0/303-runtime.experiment_limit单叶closeout.md`
- `markdown/06-milestones/v4.16.0/304-backend.runtime第八轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/305-runtime.parent_include_cleanup单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/306-runtime.parent_include_cleanup抽离方案.md`
- `markdown/06-milestones/v4.16.0/307-runtime.parent_include_cleanup清理记录.md`
- `markdown/06-milestones/v4.16.0/308-backend.runtime第九轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/309-runtime.parent_import_bridge单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/310-runtime.parent_import_bridge抽离方案.md`
- `markdown/06-milestones/v4.16.0/311-runtime.root_support_import_pilot抽离记录.md`
- `markdown/06-milestones/v4.16.0/312-runtime.root_support_import_pilot单叶closeout.md`
- `markdown/06-milestones/v4.16.0/313-runtime.root_entry_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/314-runtime.root_entry_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/315-runtime.root_entry_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/316-runtime.root_entry_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/317-runtime.report_ops_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/318-runtime.report_ops_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/319-runtime.report_ops_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/320-runtime.report_ops_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/321-runtime.parent_import_bridge父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/322-runtime.run_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/323-runtime.run_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/324-runtime.run_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/325-runtime.run_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/326-runtime.parent_import_bridge父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/327-runtime.backtest_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/328-runtime.backtest_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/329-runtime.backtest.record_store_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/330-runtime.backtest.record_store_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/331-runtime.backtest.record_store_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/332-runtime.backtest.record_store_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/333-runtime.backtest_import_pass父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/334-runtime.backtest.replay_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/335-runtime.backtest.replay_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/336-runtime.backtest.replay_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/337-runtime.backtest.replay_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/338-runtime.backtest_import_pass第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/340-runtime.backtest.experiment_sweep_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/341-runtime.backtest.experiment_sweep_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/343-runtime.backtest_import_pass第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/344-runtime.backtest.execution_start_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/345-runtime.backtest.execution_start_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/346-runtime.backtest.execution_start_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/347-runtime.backtest.execution_start_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/348-runtime.backtest_import_pass第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/349-runtime.parent_import_bridge父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/350-runtime.mutation_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/351-runtime.mutation_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/353-runtime.mutation.shared_governance_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/354-runtime.mutation.shared_governance_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/355-runtime.mutation.shared_governance_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/356-runtime.mutation_import_pass父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/358-runtime.mutation.parameter_mutation_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/361-runtime.mutation.parameter_mutation.record_query_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/362-runtime.mutation.parameter_mutation.record_query_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/366-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/367-runtime.mutation.parameter_mutation.proposal_creation_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/368-runtime.mutation.parameter_mutation_import_pass第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md`
- `markdown/06-milestones/v4.16.0/373-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离记录.md`
- `markdown/06-milestones/v4.16.0/374-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单叶closeout.md`
- `markdown/06-milestones/v4.16.0/375-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md`

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
`backend.runtime.routes`、`backend.runtime.routes.run`、`backend.runtime.routes.backtest`、`backend.runtime.routes.mutation`、`runtime.backtest.execution_start`、`runtime_persistence`、`runtime_validation`、`runtime_event_projection`、`backtest_artifacts`。

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
**最新状态补充**: BE-001BZ-01 已完成 `backend.runtime.routes` 第六轮父叶残余判断。当前 `backend.runtime.routes` 通过 `backend.runtime.routes.run`、`backend.runtime.routes.backtest`、`backend.runtime.routes.event_stream`、`backend.runtime.routes.evidence`、`backend.runtime.routes.mutation`、`backend.runtime.routes.experiment` 与 `backend.runtime.routes.report_ops` 委托七个 route child；父叶不再直接持有 route registration，并设置 `stop_split: true`。下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断。
**状态**: v4.16 BE-001G-03 `backend.runtime.routes.run` closeout 已完成，BE-001I-03 已完成其下一个 handler sibling `runtime.run.session_start` 单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断；BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`；BE-001AD-01 已完成 `backend.runtime.routes` 父叶残余判断，确认父叶仍保持 `stop_split: false`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout并设置 `stop_split: false`，BE-001AH-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout，下一步只能进入 BE-001AI-01 父叶残余判断。当前拥有 runtime route aggregate 列表，并通过 `backend.runtime.routes.run`、`backend.runtime.routes.backtest`、`backend.runtime.routes.evidence`、`backend.runtime.routes.mutation` 与 `backend.runtime.routes.experiment` 委托 route child；父级仍直接拥有 event_stream、report_ops 和 ops routes，不拥有 runtime state owner、artifact schema、compare owner 或 persistence owner。
**真实文件**:
- `src/backend/runtime.rs`
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backend/runtime/routes/evidence.rs`
- `src/backend/runtime/routes/event_stream.rs`
- `src/backend/runtime/routes/experiment.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/event_stream.rs`
- `src/runtime/run/record_store.rs`
- `src/runtime/run/replay_status.rs`
- `src/runtime/mod.rs`
- `src/runtime/run/session_start.rs`
- `src/backtest_compare.rs`
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
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md`
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md`
- `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md`
- `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md`
- `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md`
- `markdown/06-milestones/v4.16.0/70-runtime.event_stream单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/71-runtime.event_stream抽离方案.md`
- `markdown/06-milestones/v4.16.0/72-runtime.event_stream抽离记录.md`
- `markdown/06-milestones/v4.16.0/73-runtime.event_stream单叶closeout.md`
- `markdown/06-milestones/v4.16.0/74-runtime.backtest单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/75-runtime.backtest抽离方案.md`
- `markdown/06-milestones/v4.16.0/76-runtime.backtest抽离记录.md`
- `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/127-backend.runtime.routes父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/128-backend.runtime.routes.mutation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/129-backend.runtime.routes.mutation抽离方案.md`
- `markdown/06-milestones/v4.16.0/130-backend.runtime.routes.mutation抽离记录.md`
- `markdown/06-milestones/v4.16.0/131-backend.runtime.routes.mutation单叶closeout.md`
- `markdown/06-milestones/v4.16.0/230-backend.runtime.routes第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/231-backend.runtime.routes.experiment单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/232-backend.runtime.routes.experiment抽离方案.md`
- `markdown/06-milestones/v4.16.0/233-backend.runtime.routes.experiment抽离记录.md`
- `markdown/06-milestones/v4.16.0/234-backend.runtime.routes.experiment单叶closeout.md`
- `markdown/06-milestones/v4.16.0/235-backend.runtime.routes第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/236-backend.runtime.routes.evidence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/237-backend.runtime.routes.evidence抽离方案.md`
- `markdown/06-milestones/v4.16.0/238-backend.runtime.routes.evidence抽离记录.md`
- `markdown/06-milestones/v4.16.0/239-backend.runtime.routes.evidence单叶closeout.md`
- `markdown/06-milestones/v4.16.0/240-backend.runtime.routes第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/241-backend.runtime.routes.event_stream单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/242-backend.runtime.routes.event_stream抽离方案.md`
- `markdown/06-milestones/v4.16.0/243-backend.runtime.routes.event_stream抽离记录.md`
- `markdown/06-milestones/v4.16.0/244-backend.runtime.routes.event_stream单叶closeout.md`
- `markdown/06-milestones/v4.16.0/245-backend.runtime.routes第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/246-backend.runtime.routes.report_ops单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/247-backend.runtime.routes.report_ops抽离方案.md`
- `markdown/06-milestones/v4.16.0/248-backend.runtime.routes.report_ops抽离记录.md`
- `markdown/06-milestones/v4.16.0/249-backend.runtime.routes.report_ops单叶closeout.md`
- `markdown/06-milestones/v4.16.0/250-backend.runtime.routes第六轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/251-backend.runtime父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/252-runtime.report_ops单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/253-runtime.report_ops抽离方案.md`
- `markdown/06-milestones/v4.16.0/254-runtime.report_ops抽离记录.md`
- `markdown/06-milestones/v4.16.0/255-runtime.report_ops单叶closeout.md`
- `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md`
- `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md`
- `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md`
- `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md`
- `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md`
- `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md`
- `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md`
- `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md`
- `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md`
- `markdown/06-milestones/v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md`

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
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 必须经 run/backtest/event_stream/evidence/mutation/experiment 子 facade 委托，不得迁移 runtime handler |
| `backend.runtime.routes.run::register_routes` | Axum Router | run routes | `backend.runtime.routes` | 不得接管 event stream |
| `backend.runtime.routes.backtest::register_routes` | Axum Router | backtest routes | `backend.runtime.routes` | 不得接管 handler、artifact、compare 或 persistence owner |
| `backend.runtime.routes.event_stream::register_routes` | Axum Router | event stream route | `backend.runtime.routes` | 不得迁移 SSE handler 或改变 frame contract |
| `backend.runtime.routes.evidence::register_routes` | Axum Router | evidence routes | `backend.runtime.routes` | 不得接管 handler、AppState、schema 或 frontend caller |
| `backend.runtime.routes.mutation::register_routes` | Axum Router | mutation / AI proposal / approval routes | `backend.runtime.routes` | 不得接管 handler、AppState、锁顺序、schema 或 frontend caller |
| `backend.runtime.routes.experiment::register_routes` | Axum Router | experiment routes | `backend.runtime.routes` | 不得接管 handler、AppState、schema 或 frontend caller |
| `backend.runtime.routes.report_ops::register_runtime_report_routes` | Axum Router | runtime report routes | `backend.runtime.routes` | 不得接管 handler、AppState、schema 或 frontend caller |
| `backend.runtime.routes.report_ops::register_ops_routes` | Axum Router | v1 report ops routes | `backend.runtime.routes` | 不得接管 storage lifecycle、state owner 或 runtime persistence owner |
| `src/runtime/* pub(crate) handler` | HTTP request | concrete runtime response | `backend.runtime.routes` | 不得改变 `/api/runtime/*` 语义 |
| `/api/runtime/test-run` | run request | run record | frontend、tests | 不得迁移 state owner |
| `/api/runtime/v4/run` | v4 graph/run request | v4 run record | frontend、tests | 不得绕过 governance/evidence |
| `/api/runtime/backtest` | backtest request | backtest artifact | frontend、tests | 不得改 artifact schema |
| `/api/runtime/runs/:run_id/events` | run id | SSE stream | frontend、tests | 不得改变 SSE frame |

**父级通信规则**:
`backend.runtime.routes` 只能经 `backend.runtime` 和 `backend.interface_boundary` 暴露 runtime routes；不得横向直接改 `backend.graph_compile`、`backend.storage_security`、`executor` 或 frontend state。

**允许调用的子模块**:
`backend.runtime.routes.run`、`backend.runtime.routes.backtest`、`backend.runtime.routes.event_stream`、`backend.runtime.routes.evidence`、`backend.runtime.routes.mutation`、`backend.runtime.routes.experiment`、`backend.runtime.routes.report_ops`、`src/runtime/mod.rs`、src/runtime/run.rs (retired drained include)、src/runtime/backtest.rs (retired drained include)、src/runtime/mutation.rs (retired drained include)、`src/backtest_compare.rs` 中的 `pub(crate)` route targets。真实 run/backtest/mutation/report/experiment/evidence/event_stream 子域仍留在 `src/runtime/`，后续若继续拆分必须另起单子叶等价基线。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_sse`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**父叶残余判断**:
BE-001BZ-01 已完成 `backend.runtime.routes` 第六轮父叶残余判断；`backend.runtime.routes.run`、`backend.runtime.routes.backtest`、`backend.runtime.routes.event_stream`、`backend.runtime.routes.evidence`、`backend.runtime.routes.mutation`、`backend.runtime.routes.experiment` 与 `backend.runtime.routes.report_ops` 七个 route child 均由父级委托，且 route child 当前均已 closeout。父叶不再直接持有 route registration，并设置 `stop_split: true`。下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断，不得迁移 handler、schema、state owner、frontend caller、runtime persistence owner 或 release transition guard。

**细分价值判断**:
`backend.runtime.routes.experiment` 已完成 BE-001BS-04 单叶 closeout并设置 `stop_split: true`；BE-001BU-04 已确认 `backend.runtime.routes.evidence` 设置 `stop_split: true`；BE-001BW-04 已确认 `backend.runtime.routes.event_stream` 设置 `stop_split: true`；BE-001BY-04 已确认 `backend.runtime.routes.report_ops` 设置 `stop_split: true`；BE-001BZ-01 已确认 route aggregate 本身也设置 `stop_split: true`。继续拆 route aggregate 只会拆委托顺序，不会形成新的稳定 owner。

**幻觉检查点**:
AI 声称 `backend.runtime.routes` 已推进至 BE-001BZ-01 时，必须说明 route aggregate 只完成父叶收口并设置 `stop_split: true`，handler/AppState/schema/frontend caller/runtime persistence owner 均未改变，下一步仍需 `backend.runtime` 父叶残余判断。不得宣称发布过渡已启动、整理或重构已经完成。

### 5.1.1.A `backend.runtime.routes.experiment`

**层级路径**: `root.backend.runtime.routes.experiment`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001BS-04 单叶 closeout 已完成；`src/backend/runtime/routes/experiment.rs` 已创建并承接五个 experiment route registration，父级通过 `experiment::register_routes(router)` 委托并保持 reports -> experiment -> ops 相对 route order。handler、state、schema、frontend caller、runtime persistence owner 和 release transition guard 均未改变。本 route child 设置 `stop_split: true`；BE-001BT-01 已回到父叶并选择 `backend.runtime.routes.evidence` 作为下一候选。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/experiment.rs`
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/mod.rs`
- `tests/api_experiments.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/230-backend.runtime.routes第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/231-backend.runtime.routes.experiment单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/232-backend.runtime.routes.experiment抽离方案.md`
- `markdown/06-milestones/v4.16.0/233-backend.runtime.routes.experiment抽离记录.md`
- `markdown/06-milestones/v4.16.0/234-backend.runtime.routes.experiment单叶closeout.md`

**职责**:
承载 experiment route group 的 route facade 白箱边界，冻结 create/list/detail/save/discard routes 与 handler owner。本节点只承接 route registration，不拥有 experiment handler、state owner、schema owner、artifact schema、compare owner、runtime persistence owner 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| experiment request | frontend、tests | `FrontendExperimentRequest` | 不改变 graph_json、runtime_config、backtest_options 或 parameter_grid |
| experiment id | path param | string id | 不改变 detail/save/discard scoped lookup |
| pagination query | list route | `PaginationQuery` | 不改变 pagination response |
| `AppState` | backend app state | shared state | 不迁移 experiments、backtests、store dir 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| experiment detail | frontend、tests | `ExperimentDetailResponse` | 不改变 create/detail/save response shape |
| experiment list | frontend、tests | `PaginatedResponse<ExperimentListItem>` | 不改变 sorting、saved flag、variant_count 或 best_backtest_id |
| discard response | frontend、tests | `DiscardRuntimeArtifactResponse` | 不改变 preview-only discard 与 saved conflict |

**route owner 基线**:
| route | method | handler | 禁止事项 |
| --- | --- | --- | --- |
| `/api/runtime/experiments/backtest-sweep` | POST | `start_backtest_experiment` | 不得迁移 handler 或改变 variant execution |
| `/api/runtime/experiments` | GET | `list_experiments` | 不得改变 list pagination / sorting |
| `/api/runtime/experiments/:experiment_id/save` | POST | `save_experiment_record` | 不得改变 variant persistence |
| `/api/runtime/experiments/:experiment_id` | GET | `get_experiment_detail` | 不得绕过 scoped lookup |
| `/api/runtime/experiments/:experiment_id` | DELETE | `discard_experiment_record` | 不得改变 preview-only guard |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 只能委托 experiment route child，不得迁移 handler owner |
| `backend.runtime.routes.experiment::register_routes` | Axum Router | experiment routes | `backend.runtime.routes` | 不得注册 evidence/report_ops/event_stream routes |
| `start_backtest_experiment` | user id、`AppState`、request | experiment detail | route child | 不得迁移 handler 或改变 parameter grid |
| `list_experiments` | `AppState`、pagination | paginated list | route child | 不得改变 sorting |
| `get_experiment_detail` | user id、experiment id | experiment detail | route child | 不得绕过 scoped lookup |
| `save_experiment_record` | user id、experiment id | experiment detail | route child | 不得改变 transient-to-persistent flow |
| `discard_experiment_record` | user id、experiment id | discard response | route child | 不得改变 saved conflict |

**父级通信规则**:
`backend.runtime.routes.experiment` 只能经父级 `backend.runtime.routes` 暴露 experiment routes；不得横向接管 evidence、report_ops、event_stream、backtest compare、artifact schema、frontend caller 或 executor。handler owner 仍是 `runtime.backtest.experiment_sweep` 子树，状态 owner 仍是 `AppState`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_experiments`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已设置 `stop_split: true`。继续拆成 sweep/list/save/detail/discard 微 facade 不会形成新的稳定 owner，只会增加父级接线和治理碎片。

**下一步**:
本节点不再继续细拆。全局递归队列已经回到父叶，并进入 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线；不得从本节点继续细拆 experiment route child、直接迁移 evidence/report_ops/event_stream、修改 `AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

**幻觉检查点**:
AI 声称 `backend.runtime.routes.experiment` 已推进至 BE-001BS-04 时，必须说明只完成 route facade closeout，并设置 `stop_split: true`；handler 与 state/persistence owner 均未改变。不得宣称 `backend.runtime.routes` 父叶完成、experiment handler 已迁移、发布过渡已启动、整理或重构已经完成。

### 5.1.1.B `backend.runtime.routes.evidence`

**层级路径**: `root.backend.runtime.routes.evidence`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001BU-04 单叶 closeout 已完成；`src/backend/runtime/routes/evidence.rs` 已创建并承接 health / cleanup route registration，父级通过 `evidence::register_routes(router)` 委托并保持 event_stream -> evidence -> mutation 顺序。handler owner 仍在 `src/runtime/mod.rs`，`AppState`、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。本节点设置 `stop_split: true`，下一步只能回到 BE-001BV-01 `backend.runtime.routes` 父叶残余判断。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/evidence.rs`
- `src/runtime/mod.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `src/lib.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/235-backend.runtime.routes第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/236-backend.runtime.routes.evidence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/237-backend.runtime.routes.evidence抽离方案.md`
- `markdown/06-milestones/v4.16.0/238-backend.runtime.routes.evidence抽离记录.md`
- `markdown/06-milestones/v4.16.0/239-backend.runtime.routes.evidence单叶closeout.md`

**职责**:
承载 evidence health / cleanup route group 的 route facade 白箱边界，冻结两条 route 的 path、method、handler owner、状态读取、持久化 cleanup helper 与回归证据。本节点只拥有 route registration，不拥有 evidence handler、schema owner、state owner、runtime persistence owner 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| health request | frontend、tests | GET `/api/runtime/evidence/health` | 不改变 response shape 或 status |
| cleanup request | frontend、tests | `RuntimeEvidenceCleanupRequest` | 不改变 `max_age_ms` fallback |
| `AppState` | backend app state | shared state | 不迁移 `report_store_dir` 或 `evidence_metrics` |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| evidence health | frontend、tests | `RuntimeEvidenceHealthResponse` | 不改变 metrics、persisted count、status counts 或 cleanup policy |
| cleanup response | frontend、tests | `RuntimeEvidenceCleanupResponse` | 不改变 removed transient outputs 或 retained report count |

**route owner 基线**:
| route | method | handler | 禁止事项 |
| --- | --- | --- | --- |
| `/api/runtime/evidence/health` | GET | `get_runtime_evidence_health` | 不得迁移 metrics owner 或 report status counting |
| `/api/runtime/evidence/cleanup` | POST | `cleanup_runtime_evidence` | 不得迁移 transient cleanup implementation |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 只能委托 evidence route child，不得迁移 handler owner |
| `backend.runtime.routes.evidence::register_routes` | Axum Router | evidence routes | `backend.runtime.routes` | 不得注册 report_ops/event_stream routes |
| `get_runtime_evidence_health` | `AppState` | evidence health response | route aggregate | 不得迁移 report store / metrics owner |
| `cleanup_runtime_evidence` | `AppState`、cleanup request | cleanup response | route aggregate | 不得迁移 cleanup helper 或 clock helper |

**父级通信规则**:
`backend.runtime.routes.evidence` 只能经父级 `backend.runtime.routes` 暴露 evidence routes；不得横向接管 report_ops、event_stream、runtime report generation、frontend caller 或 executor。handler owner 仍是 `src/runtime/mod.rs`，状态 owner 仍是 `AppState`，persistence helper owner 仍是 `src/runtime_persistence.rs`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**下一步**:
本节点不再继续细拆。全局递归队列回到 BE-001BV-01 `backend.runtime.routes` 父叶残余判断；不得从 evidence route child 继续拆 health/cleanup 微 facade，不得迁移 evidence handler、修改 `AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

**幻觉检查点**:
AI 声称 `backend.runtime.routes.evidence` 已推进至 BE-001BU-04 时，必须说明只完成 route facade closeout 并设置 `stop_split: true`，handler 与 state/persistence owner 均未改变。不得宣称 cleanup implementation 已迁移、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

### 5.1.1.C `backend.runtime.routes.event_stream`

**层级路径**: `root.backend.runtime.routes.event_stream`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001BW-04 单叶 closeout 已完成；`src/backend/runtime/routes/event_stream.rs` 只承接 `/api/runtime/runs/:run_id/events` GET route registration，继续拆成更小 route facade 不会形成新的稳定 owner。handler owner 仍在 `src/runtime/event_stream.rs`，`AppState`、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。本节点 `stop_split: true`，下一步只能回到 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/event_stream.rs`
- `src/runtime/event_stream.rs`
- `src/runtime/mod.rs`
- `src/runtime/run/record_store.rs`
- `tests/api_sse.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/240-backend.runtime.routes第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/241-backend.runtime.routes.event_stream单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/242-backend.runtime.routes.event_stream抽离方案.md`
- `markdown/06-milestones/v4.16.0/243-backend.runtime.routes.event_stream抽离记录.md`
- `markdown/06-milestones/v4.16.0/244-backend.runtime.routes.event_stream单叶closeout.md`

**职责**:
承载 runtime run SSE route 的 route facade 白箱边界，冻结 route path、method、handler owner、SSE frame contract、keepalive contract 与测试证据。本节点只规划 route registration facade，不拥有 SSE handler、run record state owner、schema owner、runtime persistence owner 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| run event stream request | frontend、tests | GET `/api/runtime/runs/:run_id/events` | 不改变 path、method 或 auth extractor |
| run id | path param | string `run_id` | 不改变 scoped run record lookup |
| `AppState` | backend app state | shared state | 不迁移 run record store 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| SSE stream | frontend SSE panel、tests | `Sse<impl Stream<Item = Result<Event, Infallible>>>` | 不改变 content-type、event name、frame order、payload shape 或 keepalive |
| `run_started` frame | frontend、tests | JSON SSE event | 不改变 `run_id`、`graph_id`、`compile_id`、`status` |
| `runtime_event` frames | frontend、tests | JSON SSE event | 不改变 event envelope 或 sequence semantics |
| `account` frame | frontend、tests | JSON SSE event | 不改变 account snapshot |
| `run_completed` frame | frontend、tests | JSON SSE event | 不改变 `event_count` |

**route owner 基线**:
| route | method | handler | 当前 owner | 禁止事项 |
| --- | --- | --- | --- | --- |
| `/api/runtime/runs/:run_id/events` | GET | `runtime_handlers::stream_run_events` | `src/backend/runtime/routes/event_stream.rs` | 不得改变 route order 或 handler owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 只能委托 event stream route child，不得迁移 handler owner |
| `backend.runtime.routes.event_stream::register_routes` | Axum Router | event stream route | `backend.runtime.routes` | 不得注册 report_ops/evidence routes |
| `stream_run_events` | `auth::UserId`、`State<AppState>`、`Path(run_id)` | SSE stream | route aggregate | 不得改变 SSE frame、keepalive 或 run record lookup |
| `load_run_record_from_state` | `AppState`、user id、run id | run record | `stream_run_events` | 不得迁移 state owner 或绕过 scoped lookup |
| `json_sse_event` | event name、payload | SSE event | `stream_run_events` | 不得改变 JSON SSE payload contract |

**父级通信规则**:
`backend.runtime.routes.event_stream` 只能经父级 `backend.runtime.routes` 暴露 event stream route；不得横向接管 report_ops、evidence、runtime report generation、frontend caller 或 executor。handler owner 仍是 `src/runtime/event_stream.rs`，状态 owner 仍是 `AppState` 与 run record store。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_sse`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**下一步**:
本节点不再继续细拆。全局递归队列回到 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断；不得从 event_stream route child 继续拆 SSE 微 facade，不得处理 report_ops，不得迁移 `stream_run_events` handler，不得改变 SSE frame/keepalive、`AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

**幻觉检查点**:
AI 声称 `backend.runtime.routes.event_stream` 已推进至 BE-001BW-04 时，必须说明只完成 route facade closeout 并设置 `stop_split: true`，handler 与 state/persistence owner 均未改变。不得宣称 report_ops 已处理、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

### 5.1.1.D `backend.runtime.routes.report_ops`

**层级路径**: `root.backend.runtime.routes.report_ops`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001BY-04 单叶 closeout 已完成；`src/backend/runtime/routes/report_ops.rs` 承接 runtime reports、merge records、runtime generations、storage health、ops/audit/research reports 的 route registration，继续拆 runtime_reports / v1_ops 微 facade 不会形成新的稳定 owner。handler owner 仍在 `src/runtime/mod.rs`，`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未改变。本节点 `stop_split: true`，下一步只能回到 BE-001BZ-01 `backend.runtime.routes` 第六轮父叶残余判断。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/runtime/mod.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `src/storage_lifecycle.rs`
- `tests/api_run.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_backtest.rs`
- `tests/api_mutation.rs`
- `markdown/05-testing/手动全量实机测试检查单.md`
- `markdown/06-milestones/v4.16.0/245-backend.runtime.routes第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/246-backend.runtime.routes.report_ops单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/247-backend.runtime.routes.report_ops抽离方案.md`
- `markdown/06-milestones/v4.16.0/248-backend.runtime.routes.report_ops抽离记录.md`
- `markdown/06-milestones/v4.16.0/249-backend.runtime.routes.report_ops单叶closeout.md`

**职责**:
承载 report_ops route group 的 route facade 白箱边界，冻结 runtime report create/list/detail/export、merge records、runtime generations、storage health、ops daily、audit weekly 与 research monthly routes。本节点只规划 route registration facade，不拥有 report handler、schema owner、state owner、runtime persistence owner、storage lifecycle owner 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| runtime report request | frontend、tests | `CreateRuntimeReportRequest` / pagination / report id | 不改变 source kind、pagination、detail/export lookup |
| ops report query | frontend、manual smoke | `OpsDailyQuery` | 不改变 date fallback |
| audit report query | frontend、manual smoke | `AuditWeeklyQuery` | 不改变 week fallback |
| research report query | frontend、manual smoke | `ResearchMonthlyQuery` | 不改变 month fallback |
| `AppState` | backend app state | shared state | 不迁移 state owner、store dirs 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| runtime evidence report | frontend、tests | `RuntimeEvidenceReportRecord` / paginated response / artifact | 不改变 lifecycle status、source changed materialization、artifact schema |
| merge records | frontend、manual smoke | `MergeRecordsResponse` | 不改变 conflict/suppressed summary |
| generation history | frontend、manual smoke | JSON | 不改变 current_generation/history shape |
| storage health | frontend、manual smoke | JSON | 不改变 storage layer shape |
| ops/audit/research reports | frontend、manual smoke | typed report JSON | 不改变 schema owner |

**route owner 基线**:
| route | method | handler | 当前 owner |
| --- | --- | --- | --- |
| `/api/runtime/reports` | GET | `runtime_handlers::list_runtime_reports` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports` | POST | `runtime_handlers::create_runtime_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports/:report_id` | GET | `runtime_handlers::get_runtime_report_detail` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports/:report_id/export` | GET | `runtime_handlers::export_runtime_report_artifact` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/merge/records` | GET | `runtime_handlers::list_merge_records` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/runtime/generations` | GET | `runtime_handlers::list_config_generations` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/storage/health` | GET | `runtime_handlers::get_storage_health` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/ops/daily` | GET | `runtime_handlers::get_ops_daily_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/audit/weekly` | GET | `runtime_handlers::get_audit_weekly_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/research/monthly` | GET | `runtime_handlers::get_research_monthly_report` | `src/backend/runtime/routes/report_ops.rs` |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 只能委托 report_ops route child，不得迁移 handler owner |
| `report_ops::register_runtime_report_routes` | Axum Router | runtime report routes | `backend.runtime.routes` | 只承接 `/api/runtime/reports*` routes，不得注册 v1 ops 或 experiment routes |
| `report_ops::register_ops_routes` | Axum Router | v1 ops routes | `backend.runtime.routes` | 只承接 `/api/v1/*` report ops routes，不得注册 runtime report 或 experiment routes |
| `create_runtime_report` | user id、`AppState`、request | report record | route child | 不得迁移 report persistence helper |
| `list_runtime_reports` | user id、`AppState`、pagination | paginated reports | route child | 不得改变 sorting/pagination |
| `get_runtime_report_detail` | user id、`AppState`、report id | report record | route child | 不得绕过 report store lookup |
| `export_runtime_report_artifact` | user id、`AppState`、report id | report artifact | route child | 不得改变 artifact schema |
| `list_merge_records` | user id、`AppState` | merge records | route child | 不得迁移 run state owner |
| `list_config_generations` | `AppState` | generation history | route child | 不得迁移 config generation owner |
| `get_storage_health` | `AppState` | storage health | route child | 不得迁移 storage lifecycle owner |
| `get_ops_daily_report` | user id、`AppState`、query | ops report | route child | 不得迁移 report schema owner |
| `get_audit_weekly_report` | user id、`AppState`、query | audit report | route child | 不得迁移 report schema owner |
| `get_research_monthly_report` | user id、`AppState`、query | research report | route child | 不得迁移 report schema owner |

**父级通信规则**:
`backend.runtime.routes.report_ops` 只能经父级 `backend.runtime.routes` 暴露 report_ops routes；不得横向接管 run/backtest/event_stream/evidence/mutation/experiment route child、frontend caller、runtime persistence owner 或 executor。handler owner 仍是 `src/runtime/mod.rs`，状态 owner 仍是 `AppState`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_mutation`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**下一步**:
本节点不再继续细拆。全局递归队列回到 BE-001BZ-01 `backend.runtime.routes` 第六轮父叶残余判断；不得从 report_ops route child 继续拆 runtime_reports / v1_ops 微 facade，不得迁移 report handler、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

**幻觉检查点**:
AI 声称 `backend.runtime.routes.report_ops` 已推进至 BE-001BY-04 时，必须说明只完成 route facade closeout 并设置 `stop_split: true`，handler 与 state/persistence owner 均未改变。不得宣称 `backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

### 5.1.1.1 `backend.runtime.routes.mutation`

**层级路径**: `root.backend.runtime.routes.mutation`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001AE-04 单叶 closeout 已完成；`src/backend/runtime/routes/mutation.rs` 承接 mutation / AI proposal / approval route group，并由 `src/backend/runtime/routes.rs` 父级委托，route facade 等价且设置 `stop_split: true`。BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout，BE-001AN-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout；`src/runtime/mod.rs` facade、`AppState`、`approval_records -> ai_proposals` 锁顺序、schema、frontend caller 和发布过渡均未改变。下一步只能进入 BE-001AO-01 父叶残余判断。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime.rs`
- `tests/api_mutation.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/127-backend.runtime.routes父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/128-backend.runtime.routes.mutation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/129-backend.runtime.routes.mutation抽离方案.md`
- `markdown/06-milestones/v4.16.0/130-backend.runtime.routes.mutation抽离记录.md`
- `markdown/06-milestones/v4.16.0/131-backend.runtime.routes.mutation单叶closeout.md`
- `markdown/06-milestones/v4.16.0/132-runtime.mutation.parameter_mutation单子叶等价基线.md`

**职责**:
承载 runtime mutation、AI proposal 和 approval route group 的 route facade 白箱边界，冻结 path/method、handler owner、AppState owner、approval lock order 和测试证据；本节点只拥有 route registration，不拥有实际 handler 实现。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| mutation request | frontend、API caller、tests | JSON request | 不改变参数版本、capability context、safe window、no-op rejection 或 rollback 语义 |
| AI proposal request | frontend、AI proposal caller、tests | JSON request | 必须保留 static check、strategy config domain binding 和 capability gate |
| approval/proposal id | path param | string id | 不改变 scoped lookup、claim/approve/reject target 或 rejection reason |
| `AppState` | backend app state | shared state | 不迁移 mutation ledger、approval records、AI proposals 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| mutation list/detail/proposal | frontend、tests | JSON response | 不改变 response schema、status、audit events 或 rollback metadata |
| AI proposal list/detail/record | frontend、tests | JSON response | 不改变 static check failure、candidate audit 或 key event |
| approval list/detail/action response | frontend、tests | JSON response | 不改变 approval state transition、claim owner 或 rejection reason |

**route owner 基线**:
| route | method | handler | 禁止事项 |
| --- | --- | --- | --- |
| `/api/runtime/mutations` | GET | `list_runtime_parameter_mutations` | 不得改变排序、filtering 或 response schema |
| `/api/runtime/mutations` | POST | `create_runtime_parameter_mutation` | 不得绕过 capability / safe window / audit |
| `/api/runtime/mutations/:proposal_id` | GET | `get_runtime_parameter_mutation_detail` | 不得绕过 scoped lookup |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` | 不得改变 ledger-backed activation |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` | 不得改变 rollback target |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` | 不得改变 audit projection |
| `/api/runtime/ai-proposals` | POST | `create_runtime_ai_proposal` | 不得绕过 static check 或 capability gate |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` | 不得改变 candidate diagnostics |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` | 不得改变 approval visibility |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` | 不得改变 approval state |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` | 不得改变 approval lock order |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` | 不得丢失 rejection reason |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` | 不得改变 reviewer claim semantics |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 只允许委托 mutation route child，不得迁移 handler owner |
| `backend.runtime.routes.mutation::register_routes` | Axum Router | mutation / AI proposal / approval route group | `backend.runtime.routes` | 不得注册 report/evidence/experiment/ops routes |
| `create_runtime_parameter_mutation` | mutation request | mutation proposal response | route aggregate | 不得改变 capability / safe window / audit |
| `list_runtime_parameter_mutations` | query | mutation list | route aggregate | 不得改变排序或 filtering |
| `get_runtime_parameter_mutation_detail` | proposal id | mutation detail | route aggregate | 不得绕过 scoped lookup |
| `activate_runtime_parameter_mutation` | proposal id | activation response | route aggregate | 不得改变 ledger-backed activation |
| `rollback_runtime_parameter_mutation` | proposal id | rollback response | route aggregate | 不得改变 rollback target |
| `create_runtime_ai_proposal` | AI proposal request | AI proposal response | route aggregate | 不得绕过 static check 或 capability gate |
| `list_runtime_ai_proposals` | query | AI proposal list | route aggregate | 不得改变 audit projection |
| `get_runtime_ai_proposal_detail` | proposal id | AI proposal detail | route aggregate | 不得改变 candidate diagnostics |
| `list_runtime_approvals` | query | approval list | route aggregate | 不得改变 approval visibility |
| `get_runtime_approval_detail` | approval id | approval detail | route aggregate | 不得改变 approval state |
| `approve_ai_proposal` | proposal id | approval action response | route aggregate | 不得改变 approval lock order |
| `reject_ai_proposal` | proposal id | rejection response | route aggregate | 不得丢失 rejection reason |
| `claim_ai_proposal_review` | proposal id | claim response | route aggregate | 不得改变 reviewer claim semantics |

**父级通信规则**:
`backend.runtime.routes.mutation` 只能经父级 `backend.runtime.routes` 暴露 mutation / AI proposal / approval routes；不得横向接管 report、evidence、experiment、ops、strategy_config、frontend caller 或 executor。handler owner 仍是 src/runtime/mutation.rs (retired drained include)，状态 owner 仍是 `AppState`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**状态与锁**:
`approval_records -> ai_proposals` lock order 必须保持不变；mutation ledger、approval records、AI proposals、capability context、sandbox/static check 和 audit projection 均保留原 owner。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已完成等价基线、抽离方案、route facade 最小物理抽离与单叶 closeout，route facade 设置 `stop_split: true`。继续把 facade 拆成 mutation routes、AI proposal routes 和 approval routes 会增加父级导入面，但不会形成新的稳定 owner；后续递归已转入 src/runtime/mutation.rs (retired drained include) handler 域，`runtime.mutation.parameter_mutation` 已完成 BE-001AF-04 单叶 closeout，BE-001AN-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout，下一步只能进入 BE-001AO-01 父叶残余判断。`runtime.mutation.ai_proposal`、`runtime.mutation.approval_review` 值得后续排队，`runtime.mutation.shared_persistence_governance` 暂缓。

**幻觉检查点**:
AI 声称 `backend.runtime.routes.mutation` 已完成 BE-001AF-04 时，必须说明 route facade 已 closeout 并设置 `stop_split: true`，`runtime.mutation.parameter_mutation` 只完成单叶 closeout 且设置 `stop_split: false`；AppState、`approval_records -> ai_proposals` 锁顺序、schema、frontend caller 和发布过渡均未改变。不得宣称 approval/AI proposal 状态 owner 已迁移、`backend.runtime.routes` 父叶完成、整理或重构已经完成。

### 5.1.1.2 `runtime.mutation.parameter_mutation`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation`
**父模块**: `backend.runtime`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AT-01 父叶残余判断已完成；`transition_lifecycle` 已 closeout 并设置 `stop_split: true`，但 proposal creation handler、record id helper、list/detail handler 仍为 parent-owned implementation residual，因此本叶保持 `stop_split: false`。五个 parameter mutation public handler 与本叶私有 helper 已迁入 `src/runtime/mutation/parameter_mutation.rs`，transition lifecycle handler/helper 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`，其六个 child 均已 closeout。父级 `src/runtime/mod.rs` 通过 `pub(crate) use mutation_parameter_mutation` 保持 route facade 调用面。下一步只能进入 BE-001AU-01 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线。AI proposal、approval review、AppState、schema、frontend caller、锁顺序、shared helper 和发布过渡连接未改变。
**最新状态补充**: BE-001AO-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第四轮父叶残余判断；`transition_lifecycle` 父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AP-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线。不得直接移动 shared lifecycle/persistence helper、rollback id、AppState、schema、frontend caller 或启动发布过渡。
**最新状态补充**: BE-001AP-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AP-02 抽离方案。
**最新状态补充**: BE-001AP-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AP-03 实际抽离。
**最新状态补充**: BE-001AP-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 实际抽离；`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 已迁入 child，下一步只能进入 BE-001AP-04 单叶 closeout。
**最新状态补充**: BE-001AP-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AQ-01 `transition_lifecycle` 第五轮父叶残余判断。
**最新状态补充**: BE-001AQ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AR-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线。
**最新状态补充**: BE-001AR-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AR-02 抽离方案。
**最新状态补充**: BE-001AR-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AR-03 实际抽离。
**最新状态补充**: BE-001AR-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 实际抽离；`runtime_parameter_mutation_rollback_record_id` 已迁入 child，下一步只能进入 BE-001AR-04 单叶 closeout。
**最新状态补充**: BE-001AR-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AS-01 `transition_lifecycle` 第六轮父叶残余判断。
**最新状态补充**: BE-001AS-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**最新状态补充**: BE-001AT-01 已完成 `runtime.mutation.parameter_mutation` 父叶残余判断；本叶仍保持 `stop_split: false`，下一步只能进入 BE-001AU-01 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线。
**最新状态补充**: BE-001AU-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AU-02 抽离方案。
**最新状态补充**: BE-001AU-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AU-03 实际抽离。
**最新状态补充**: BE-001AU-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 实际抽离；`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 child，下一步只能进入 BE-001AU-04 单叶 closeout。
**最新状态补充**: BE-001AU-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**最新状态补充**: BE-001AV-01 已完成 `runtime.mutation.parameter_mutation` 第二轮父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AW-01 `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线。
**最新状态补充**: BE-001AW-01 已建立 `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AW-02 抽离方案。
**最新状态补充**: BE-001AW-02 已建立 `runtime.mutation.parameter_mutation.record_query` 抽离方案；当前仍为 `no code movement`，下一步只能进入 BE-001AW-03 实际抽离。
**最新状态补充**: BE-001AW-03 已完成 `runtime.mutation.parameter_mutation.record_query` 实际抽离；`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 已迁入 child，下一步只能进入 BE-001AW-04 单叶 closeout。
**最新状态补充**: BE-001AW-04 已完成 `runtime.mutation.parameter_mutation.record_query` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AX-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**最新状态补充**: BE-001AX-01 已完成 `runtime.mutation.parameter_mutation` 第三轮父叶残余判断并设置父叶 `stop_split: true`；下一步只能进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。
**真实文件**:
- `src/runtime/mutation/parameter_mutation.rs`
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/runtime_persistence.rs`
- `src/lib.rs`
- `src/frontend_api_types.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_event_projection.rs`
- `tests/api_mutation.rs`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/utils/runtimeMutation.js`
- `frontend/src/components/RuntimeMutationPanel.jsx`
- `markdown/06-milestones/v4.16.0/132-runtime.mutation.parameter_mutation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/133-runtime.mutation.parameter_mutation抽离方案.md`
- `markdown/06-milestones/v4.16.0/134-runtime.mutation.parameter_mutation抽离记录.md`
- `markdown/06-milestones/v4.16.0/135-runtime.mutation.parameter_mutation单叶closeout.md`
- `markdown/06-milestones/v4.16.0/136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/170-runtime.mutation.parameter_mutation父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md`
- `markdown/06-milestones/v4.16.0/173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md`
- `markdown/06-milestones/v4.16.0/174-runtime.mutation.parameter_mutation.proposal_creation单叶closeout.md`
- `markdown/06-milestones/v4.16.0/175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/177-runtime.mutation.parameter_mutation.record_query抽离方案.md`
- `markdown/06-milestones/v4.16.0/178-runtime.mutation.parameter_mutation.record_query抽离记录.md`
- `markdown/06-milestones/v4.16.0/179-runtime.mutation.parameter_mutation.record_query单叶closeout.md`
- `markdown/06-milestones/v4.16.0/180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md`

**职责**:
承载 runtime parameter mutation lifecycle handler 白箱边界，冻结 proposal create/list/detail、activation、rollback、safe window、parameter version canonicalization、event contract、run record append 和 persisted mutation record 的等价证据。本节点不拥有 AI proposal、approval review、schema 定义、AppState、runtime persistence、frontend caller 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| mutation create request | `backend.runtime.routes.mutation` | `RuntimeParameterMutationRequest` | 不改变 target、patch、parameter version、safe window semantics |
| mutation transition request | route caller / tests | proposal id + activate/rollback body | 不改变 lifecycle order 或 rollback target |
| `AppState` | `backend.app_state_wiring` | shared state | 不迁移状态 owner 或锁顺序 |
| persisted runtime records | `src/runtime_persistence.rs` | mutation record / run record | 不改变 file layout、audit payload 或 scoped lookup |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| mutation response | frontend / tests | `RuntimeParameterMutationRecord` JSON | 不改变 response schema |
| lifecycle event | run evidence | `FrontendRuntimeEvent` | 不改变 activation/rollback event contract |
| mutation ledger record | runtime persistence | persisted mutation record | 不改变 id、status、governance 或 parameter version |
| run record append | runtime persistence | existing run events update | 不迁移 run record persistence owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_parameter_mutation` | `AppState`、request | mutation record | `backend.runtime.routes.mutation` | 不得接管 AI proposal 或 approval |
| `list_runtime_parameter_mutations` | `AppState`、query | mutation list | route facade | 不得改变 filtering / ordering |
| `get_runtime_parameter_mutation_detail` | proposal id | mutation detail | route facade | 不得改变 not found semantics |
| `activate_runtime_parameter_mutation` | proposal id、activation body | activated record | route facade | 不得迁移 snapshot/config generation owner |
| `rollback_runtime_parameter_mutation` | proposal id、rollback body | rolled back record | route facade | 不得改变 rollback event contract |

**子模块私有 helper**:
`validate_runtime_parameter_mutation_boundary`；`resolve_runtime_parameter_mutation_boundary`；`evaluate_runtime_parameter_mutation_safe_window`；`runtime_parameter_mutation_record_id`；`runtime_parameter_mutation_rollback_record_id`；`mutation_lifecycle_entry`；`persist_runtime_parameter_mutation_transition`；`auto_snapshot_on_activation`。

**父级 shared helper**:
`canonical_runtime_parameter_version`；`validate_runtime_parameter_mutation_target`；`runtime_mode_from_events`；`status_contract_value`；`mutation_event_contract`；`build_runtime_parameter_mutation_event`；`append_parameter_mutation_events_to_run`；`runtime_parameter_mutation_governance`；`governance_with_parameter_version`。

**抽离结果**:
| 项 | BE-001AF-03 结果 | 约束 |
| --- | --- | --- |
| 目标子模块 | `src/runtime/mutation/parameter_mutation.rs` | 已创建，承接五个 public handler 和本叶私有 helper |
| 父级声明 | `#[path = "mutation/parameter_mutation.rs"] mod mutation_parameter_mutation;` | 已落在 `src/runtime/mod.rs`，位于 `include!("mutation.rs")` 之前 |
| 父级出口 | `pub(crate) use mutation_parameter_mutation` | 已只 re-export 五个 parameter mutation public handler |
| route facade | `src/backend/runtime/routes/mutation.rs` | 不改 route、不改 handler 调用名 |
| query owner | `RuntimeParameterMutationListQuery` | 继续留在 `src/runtime/mod.rs` |

**BE-001AF-03 已迁移**:
`create_runtime_parameter_mutation`；`list_runtime_parameter_mutations`；`get_runtime_parameter_mutation_detail`；`activate_runtime_parameter_mutation`；`rollback_runtime_parameter_mutation`；`validate_runtime_parameter_mutation_boundary`；`resolve_runtime_parameter_mutation_boundary`；`evaluate_runtime_parameter_mutation_safe_window`；`runtime_parameter_mutation_record_id`；`runtime_parameter_mutation_rollback_record_id`；`mutation_lifecycle_entry`；`persist_runtime_parameter_mutation_transition`；`auto_snapshot_on_activation`。
**BE-001AP-03 追加迁移**:
`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 已从 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 进一步迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`，父级通过 path-attributed child 与 helper import 维持 sibling 调用面。
**BE-001AR-03 追加迁移**:
`runtime_parameter_mutation_rollback_record_id` 已从 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 进一步迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`，父级通过 path-attributed child 与 helper import 维持 sibling 调用面。
**BE-001AT-01 父叶残余判断结果**:
`runtime.mutation.parameter_mutation` 父叶残余判断已完成。`transition_lifecycle` 已 closeout 并设置 `stop_split: true`，但 `runtime_parameter_mutation_record_id`、`create_runtime_parameter_mutation`、`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 仍为 parent-owned implementation residual，因此父叶保持 `stop_split: false`。下一步只能进入 BE-001AU-01 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线。

**BE-001AU-01 proposal_creation 基线结果**:
`runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线已建立。当前仅冻结 `create_runtime_parameter_mutation`、`runtime_parameter_mutation_record_id`、`RuntimeParameterMutationRecord` 构造字段、record id digest contract 与调用顺序；仍为 `no code movement`，目标文件尚未创建。下一步只能进入 BE-001AU-02 抽离方案。

**BE-001AU-02 proposal_creation 抽离方案结果**:
`runtime.mutation.parameter_mutation.proposal_creation` 抽离方案已建立。当前仍为 `no code movement`；方案只固定 BE-001AU-03 的目标文件、父级 path attribute、handler re-export、`use super::*`、迁移清单、非目标和回退点。下一步只能进入 BE-001AU-03 实际抽离，不得迁移 list/detail、回改 `transition_lifecycle`、改变 AppState/schema/frontend caller 或启动发布过渡。

**BE-001AU-03 proposal_creation 抽离结果**:
`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 `src/runtime/mutation/parameter_mutation/proposal_creation.rs`。父级 `src/runtime/mutation/parameter_mutation.rs` 通过 `#[path = "parameter_mutation/proposal_creation.rs"] mod proposal_creation;` 与 `pub(crate) use proposal_creation::create_runtime_parameter_mutation;` 维持原 handler 出口；`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 仍留在父级。

**BE-001AU-04 proposal_creation closeout 结果**:
`runtime.mutation.parameter_mutation.proposal_creation` 已完成单叶 closeout 并设置 `stop_split: true`。本叶只有一个 public handler，`runtime_parameter_mutation_record_id` 只服务该 handler，继续拆 record builder、event append 或 persistence wrapper 不会形成稳定 owner。下一步只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。

**BE-001AV-01 父叶残余判断结果**:
`transition_lifecycle` 与 `proposal_creation` 均已 closeout 并设置 `stop_split: true`，但 `list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 仍为 parent-owned implementation residual，因此父叶保持 `stop_split: false`。下一候选固定为 `runtime.mutation.parameter_mutation.record_query`，BE-001AW-01 只允许建立该子叶等价基线。

**BE-001AW-01 record_query 基线结果**:
`runtime.mutation.parameter_mutation.record_query` 单子叶等价基线已建立。当前仅冻结 list/detail 查询流的输入输出、排序、filtering、scoped lookup、in-memory 优先级、persistence fallback 和 pagination 语义；仍为 `no code movement`，目标文件尚未创建。下一步只能进入 BE-001AW-02 抽离方案。

**BE-001AW-02 record_query 抽离方案结果**:
`runtime.mutation.parameter_mutation.record_query` 抽离方案已建立。当前仍为 `no code movement`；方案只固定 BE-001AW-03 的目标 Rust 文件、父级 path attribute、双 handler re-export、`use super::*`、迁移清单、非目标和回退点。下一步只能进入 BE-001AW-03 实际抽离，不得迁移 create/activate/rollback、AI proposal、approval review、AppState、schema、frontend caller 或启动发布过渡。

**BE-001AW-03 record_query 抽离结果**:
`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 已迁入 `src/runtime/mutation/parameter_mutation/record_query.rs`。父级 `src/runtime/mutation/parameter_mutation.rs` 通过 `#[path = "parameter_mutation/record_query.rs"] mod record_query;` 与 `pub(crate) use record_query::{get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations};` 维持原 handler 出口。

**BE-001AW-04 record_query closeout 结果**:
`runtime.mutation.parameter_mutation.record_query` 单叶 closeout 已完成并设置 `stop_split: true`。list/detail 属于同一个 mutation record read model；继续拆成 list/detail 微文件不会形成新的稳定 owner。下一步只能进入 BE-001AX-01 父叶残余判断。

**BE-001AX-01 父叶残余判断结果**:
`transition_lifecycle`、`proposal_creation` 与 `record_query` 均已 closeout 并设置 `stop_split: true`；`runtime.mutation.parameter_mutation` 父叶只剩 facade / child declaration / re-export / controlled import，因此父叶设置 `stop_split: true`。下一步只能回到 mutation handler sibling 队列，进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。

**BE-001AF-03 必须保留父级 shared helper**:
`canonical_runtime_parameter_version`；`validate_runtime_parameter_mutation_target`；`runtime_parameter_mutation_governance`；`governance_with_parameter_version`；`append_parameter_mutation_events_to_run`；`build_runtime_parameter_mutation_event`；`mutation_event_contract`；`status_contract_value`；`runtime_mode_from_events`。

**路由 owner 基线**:
| Route | Handler | 当前 owner |
| --- | --- | --- |
| `POST /api/runtime/mutations` | `create_runtime_parameter_mutation` | `src/runtime/mutation/parameter_mutation/proposal_creation.rs` via `src/runtime/mutation/parameter_mutation.rs` / `src/runtime/mod.rs` |
| `GET /api/runtime/mutations` | `list_runtime_parameter_mutations` | `src/runtime/mutation/parameter_mutation/record_query.rs` via `src/runtime/mutation/parameter_mutation.rs` / `src/runtime/mod.rs` |
| `GET /api/runtime/mutations/:proposal_id` | `get_runtime_parameter_mutation_detail` | `src/runtime/mutation/parameter_mutation/record_query.rs` via `src/runtime/mutation/parameter_mutation.rs` / `src/runtime/mod.rs` |
| `POST /api/runtime/mutations/:proposal_id/activate` | `activate_runtime_parameter_mutation` | `src/runtime/mutation/parameter_mutation.rs` via `src/runtime/mod.rs` |
| `POST /api/runtime/mutations/:proposal_id/rollback` | `rollback_runtime_parameter_mutation` | `src/runtime/mutation/parameter_mutation.rs` via `src/runtime/mod.rs` |

**父级通信规则**:
`runtime.mutation.parameter_mutation` 只能经 `backend.runtime.routes.mutation` 暴露 HTTP route，并经父级 runtime facade 维持兼容出口。不得横向接管 AI proposal、approval review、report、evidence、experiment、ops、strategy_config、executor 或 frontend caller。状态 owner 仍是 `AppState`，schema owner 仍是 `src/frontend_api_types.rs`，persistence owner 仍是 `src/runtime_persistence.rs`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**已决策点**:
| 决策 | 结论 | 原因 |
| --- | --- | --- |
| 目标文件路径 | BE-001AF-03 使用 src/runtime/mutation/parameter_mutation.rs | 子模块声明由 `src/runtime/mod.rs` 承担，避开 `include!("mutation.rs")` 路径歧义 |
| `auto_snapshot_on_activation` 是否随 activation handler 移动 | 随 activation handler 移动 | 该 helper 只被 activation 调用；snapshot/config generation owner 不迁移 |
| `append_parameter_mutation_events_to_run` 是否作为本叶私有 helper | 暂留父级 shared helper | AI proposal 也复用 run event append，不能在本批私有化 |
| shared persistence/governance helper 是否另起节点 | 后续批次再判断 | 避免混入 AI proposal/approval 状态流 |

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**细分价值判断**:
BE-001AX-01 已确认 `runtime.mutation.parameter_mutation` 父叶停止细拆，`stop_split: true`。父叶只承担 child declaration、re-export 和 controlled import；继续拆 facade 不会形成新的稳定 owner。下一步只能回到 mutation sibling 队列进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。

**幻觉检查点**:
AI 声称 `runtime.mutation.parameter_mutation` 已推进至 BE-001AX-01 时，必须说明本父叶已设置 `stop_split: true`，但 mutation handler sibling 队列尚未完成，下一步只能进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。不得宣称 AI proposal 已抽离、approval review 已拆分、AppState/schema/frontend caller 已迁移、发布过渡已启动、整理或重构已经完成。

### 5.1.1.2.1 `runtime.mutation.parameter_mutation.transition_lifecycle`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle`
**父模块**: `runtime.mutation.parameter_mutation`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AS-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断并设置父叶 `stop_split: true`；`boundary_safety`、`activation_flow`、`rollback_flow`、`activation_snapshot_side_effect`、`transition_record_persistence` 与 `rollback_record_identity` 均已 closeout 并设置 `stop_split: true`。本父叶只保留 facade / re-export / wrapper / child imports。父级通过 `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;`、`pub(crate) use transition_lifecycle::{activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation};` 和 `use transition_lifecycle::validate_runtime_parameter_mutation_boundary;` 维持 handler 与 boundary validation 出口。下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断，不得混入 proposal create/list/detail、AI proposal、approval、AppState、schema、frontend caller 或发布过渡连接。
**最新状态补充**: BE-001AO-01 已完成第四轮父叶残余判断；本父叶仍保持 `stop_split: false`，因为 `mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition` 和 `runtime_parameter_mutation_rollback_record_id` 仍为 parent-owned residual。下一步只能进入 BE-001AP-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线，先冻结 lifecycle entry 与 transition persistence，不得直接迁移 rollback id 或启动 release transition guard。
**最新状态补充**: BE-001AP-01 已建立 `transition_record_persistence` 单子叶等价基线；`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 仍留在 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`，目标文件尚未创建。下一步只能进入 BE-001AP-02 抽离方案。
**最新状态补充**: BE-001AP-02 已建立 `transition_record_persistence` 抽离方案；目标 child、父级 path attribute、helper import、`pub(super)` visibility 和回退点已固定。下一步只能进入 BE-001AP-03 实际抽离。
**最新状态补充**: BE-001AP-03 已完成 `transition_record_persistence` 实际抽离；`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`，下一步只能进入 BE-001AP-04 单叶 closeout。
**最新状态补充**: BE-001AP-04 已完成 `transition_record_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AQ-01 第五轮父叶残余判断。
**最新状态补充**: BE-001AQ-01 已完成第五轮父叶残余判断；本父叶仍保持 `stop_split: false`，因为 `runtime_parameter_mutation_rollback_record_id` 仍为 parent-owned residual。下一步只能进入 BE-001AR-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线。
**最新状态补充**: BE-001AR-01 已建立 `rollback_record_identity` 单子叶等价基线；`runtime_parameter_mutation_rollback_record_id` 仍留在父级，目标文件尚未创建。下一步只能进入 BE-001AR-02 抽离方案。
**最新状态补充**: BE-001AR-02 已建立 `rollback_record_identity` 抽离方案；目标 child、父级 path attribute、helper import、`pub(super)` visibility 和回退点已固定。下一步只能进入 BE-001AR-03 实际抽离。
**最新状态补充**: BE-001AR-03 已完成 `rollback_record_identity` 实际抽离；`runtime_parameter_mutation_rollback_record_id` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`，父级保留受控 import。下一步只能进入 BE-001AR-04 单叶 closeout。
**最新状态补充**: BE-001AR-04 已完成 `rollback_record_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AS-01 父叶残余判断。
**最新状态补充**: BE-001AS-01 已完成本父叶残余判断；父叶设置 `stop_split: true`，下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**真实文件**:
- `src/runtime/mutation/parameter_mutation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/runtime_persistence.rs`
- `tests/api_mutation.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/138-runtime.mutation.parameter_mutation.transition_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `markdown/06-milestones/v4.16.0/142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md`
- `markdown/06-milestones/v4.16.0/143-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单叶closeout.md`
- `markdown/06-milestones/v4.16.0/144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md`
- `markdown/06-milestones/v4.16.0/147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md`
- `markdown/06-milestones/v4.16.0/148-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单叶closeout.md`
- `markdown/06-milestones/v4.16.0/149-runtime.mutation.parameter_mutation.transition_lifecycle第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md`
- `markdown/06-milestones/v4.16.0/152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md`
- `markdown/06-milestones/v4.16.0/153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md`
- `markdown/06-milestones/v4.16.0/154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md`
- `markdown/06-milestones/v4.16.0/157-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离记录.md`
- `markdown/06-milestones/v4.16.0/158-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单叶closeout.md`
- `markdown/06-milestones/v4.16.0/159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md`
- `markdown/06-milestones/v4.16.0/162-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离记录.md`
- `markdown/06-milestones/v4.16.0/163-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单叶closeout.md`
- `markdown/06-milestones/v4.16.0/164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md`
- `markdown/06-milestones/v4.16.0/167-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离记录.md`
- `markdown/06-milestones/v4.16.0/168-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单叶closeout.md`
- `markdown/06-milestones/v4.16.0/169-runtime.mutation.parameter_mutation.transition_lifecycle第六轮父叶残余判断.md`

**职责**:
承载 runtime parameter mutation transition lifecycle 白箱边界，冻结已有 proposal 从 activation 或 rollback request 进入状态转移、safe window 拒绝、transition record 持久化、run record append 和 activation auto snapshot side effect 的等价证据。本节点不拥有 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| activation request | `backend.runtime.routes.mutation` | proposal id + `ActivateRuntimeParameterMutationRequest` | 不改变 `Proposed`、`SafeWindowDenied`、`ActivationScheduled`、`Activated`、`ActivationFailed` 语义 |
| rollback request | `backend.runtime.routes.mutation` | proposal id + `RollbackRuntimeParameterMutationRequest` | 不改变 rollback target、rollback record id 或 rollback event contract |
| runtime mode / run records | `AppState` / runtime persistence | current run + events | 不改变 safe window reason code 或 run event append |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| transition mutation record | route caller / frontend | `RuntimeParameterMutationRecord` | 不改变 response schema |
| lifecycle event | run evidence | `FrontendRuntimeEvent` | 不改变 event kind、event type 或 status contract |
| optional activation snapshot | config generation / snapshot store | existing snapshot side effect | 不迁移 snapshot owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `activate_runtime_parameter_mutation` | `AppState`、proposal id、`ActivateRuntimeParameterMutationRequest` | activated / scheduled / denied mutation record | `backend.runtime.routes.mutation` | 不得迁移 AppState、schema、snapshot owner 或 release transition guard |
| `rollback_runtime_parameter_mutation` | `AppState`、proposal id、`RollbackRuntimeParameterMutationRequest` | rollback / scheduled mutation record | `backend.runtime.routes.mutation` | 不得改变 rollback target 或 record id 语义 |

**关键 helper 基线**:
`validate_runtime_parameter_mutation_boundary`；`resolve_runtime_parameter_mutation_boundary`；`evaluate_runtime_parameter_mutation_safe_window`；`mutation_lifecycle_entry`；`persist_runtime_parameter_mutation_transition`；`runtime_parameter_mutation_rollback_record_id`；`auto_snapshot_on_activation`。

**父级 shared helper**:
`build_runtime_parameter_mutation_event`；`append_parameter_mutation_events_to_run`；`governance_with_parameter_version`；`runtime_parameter_mutation_governance`；`mutation_event_contract`；`status_contract_value`；`runtime_mode_from_events`。

**HTTP route 基线**:
| Route | Method | Handler |
| --- | --- | --- |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` |

**状态机基线**:
`SafeWindowDenied`、`ActivationScheduled`、`Activated`、`ActivationFailed`、`RollbackScheduled`、`RolledBack`、`RollbackFailed` 必须保持当前 transition order、metric side effect、run event append 和 response schema。

**排除边界**:
不得迁移 `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail`、`runtime_parameter_mutation_record_id`、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。BE-001AG-04 已完成 transition lifecycle 单叶 closeout，并判定本叶继续细拆。

**BE-001AG-03 抽离结果**:
`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 已创建并迁移 `activate_runtime_parameter_mutation`、`rollback_runtime_parameter_mutation`、`validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary`、`evaluate_runtime_parameter_mutation_safe_window`、`runtime_parameter_mutation_rollback_record_id`、`mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition` 和 `auto_snapshot_on_activation`。其中 `validate_runtime_parameter_mutation_boundary` 为 `pub(super)`，供父级 `create_runtime_parameter_mutation` 继续复用；其余 transition helper 保持 private。父级 `src/runtime/mutation/parameter_mutation.rs` 保留 `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail` 和 `runtime_parameter_mutation_record_id`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**幻觉检查点**:
**BE-001AG-04 closeout 结果**:
本叶实际抽离等价成立，但 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 仍有 boundary/safe-window、activation、rollback 和 snapshot side effect 四类责任，设置 `stop_split: false`。下一步只能进入 BE-001AH-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单子叶等价基线，先冻结 `validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary` 和 `evaluate_runtime_parameter_mutation_safe_window` 的输入输出。

**BE-001AH-01 boundary_safety 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 已建立单子叶等价基线，冻结 `validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary` 和 `evaluate_runtime_parameter_mutation_safe_window`。当前 `no code movement`，下一步只能进入 BE-001AH-02 抽离方案。

**BE-001AH-02 boundary_safety 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 已建立抽离方案，固定目标文件 src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs、父级 `mod boundary_safety`、delegating validation wrapper、helper visibility 和回退点。当前 `no code movement`，下一步只能进入 BE-001AH-03 实际抽离。

**BE-001AH-03 boundary_safety 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`。父级使用 `#[path = "transition_lifecycle/boundary_safety.rs"] mod boundary_safety;`、helper import 和 delegating validation wrapper 保持上层调用面。下一步只能进入 BE-001AH-04 单叶 closeout。

**BE-001AH-04 boundary_safety closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout 已完成。该叶只包含三个强相关纯策略 helper，继续细拆会增加父级 import 和测试定位成本，因此设置 `stop_split: true`。下一步只能进入 BE-001AI-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断。

**BE-001AI-01 父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断已完成。`boundary_safety` 已完成 closeout 并设置 `stop_split: true`；父叶仍拥有 `activation_flow`、`rollback_flow` 和 `activation_snapshot_side_effect` 等稳定残余候选，因此父叶保持 `stop_split: false`。下一步只能进入 BE-001AJ-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单子叶等价基线。

**BE-001AJ-01 activation_flow 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单子叶等价基线已建立。当前只冻结 `activate_runtime_parameter_mutation` 的输入输出、状态机分支、event append、metrics、transition persistence 和 `auto_snapshot_on_activation` 调用时机；代码未移动，目标文件未创建。下一步只能进入 BE-001AJ-02 抽离方案。

**BE-001AJ-02 activation_flow 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 抽离方案已建立。目标文件固定为 src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs，父级将使用 path-attributed child 和 `pub(crate) use activation_flow::activate_runtime_parameter_mutation` 保持上层调用面。当前 `no code movement`，下一步只能进入 BE-001AJ-03 实际抽离。

**BE-001AJ-03 activation_flow 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`。父级使用 `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;` 与 `pub(crate) use activation_flow::activate_runtime_parameter_mutation` 保持上层调用面。下一步只能进入 BE-001AJ-04 单叶 closeout。

**BE-001AJ-04 activation_flow closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单叶 closeout 已完成。该叶只承接 `activate_runtime_parameter_mutation` 一个稳定 public handler，内部分支属于同一 activation transaction 状态机，继续细拆不会形成新 owner，因此设置 `stop_split: true`。下一步只能进入 BE-001AK-01 `transition_lifecycle` 父叶残余判断。

**BE-001AK-01 transition_lifecycle 第二轮父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 第二轮父叶残余判断已完成。`boundary_safety` 与 `activation_flow` 均已 closeout，但父叶仍直接拥有 `rollback_runtime_parameter_mutation` 和 activation snapshot side effect，因此保持 `stop_split: false`。下一步只能进入 BE-001AL-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单子叶等价基线。

**BE-001AL-01 rollback_flow 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单子叶等价基线已建立。当前只冻结 `rollback_runtime_parameter_mutation` 的输入输出、activated-only gate、ledger lookup、safe-window 分支、RollbackScheduled / RolledBack / RollbackFailed 状态机、run event append、metrics 和 transition persistence；代码未移动，目标文件未创建。下一步只能进入 BE-001AL-02 抽离方案。

**BE-001AL-02 rollback_flow 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 抽离方案已建立。目标文件固定为 transition lifecycle 下的 rollback_flow child，父级将使用 path-attributed child 和 `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation` 保持上层调用面。当前 `no code movement`，下一步只能进入 BE-001AL-03 实际抽离。

**BE-001AL-03 rollback_flow 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`。父级使用 `#[path = "transition_lifecycle/rollback_flow.rs"] mod rollback_flow;` 与 `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;` 保持上层调用面。rollback id、lifecycle entry、transition persistence 和 activation snapshot helper 仍留在 `transition_lifecycle` 父级。下一步只能进入 BE-001AL-04 单叶 closeout。

**BE-001AL-04 rollback_flow closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单叶 closeout 已完成。该叶只承接 `rollback_runtime_parameter_mutation` 一个稳定 public handler，内部 branch 属于同一 rollback transaction 状态机，继续细拆不会形成新 owner，因此设置 `stop_split: true`。下一步只能进入 BE-001AM-01 `transition_lifecycle` 父叶残余判断。

**BE-001AM-01 transition_lifecycle 第三轮父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 第三轮父叶残余判断已完成。`boundary_safety`、`activation_flow` 与 `rollback_flow` 均已 closeout 并设置 `stop_split: true`，但父叶仍直接拥有 `auto_snapshot_on_activation`、shared lifecycle/persistence helper 和 rollback id helper，因此保持 `stop_split: false`。下一步只能进入 BE-001AN-01 `activation_snapshot_side_effect` 单子叶等价基线。

**BE-001AN-01 activation_snapshot_side_effect 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单子叶等价基线已建立。当前只冻结 `auto_snapshot_on_activation` 的 config generation、history truncation、snapshot id、payload/signature、`DeploymentSignatureSnapshot`、`atomic_write_json` 和 in-memory `state.snapshots` insert；代码未移动，目标文件未创建。下一步只能进入 BE-001AN-02 抽离方案。

**BE-001AN-02 activation_snapshot_side_effect 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 抽离方案已建立。目标 child、父级 path attribute、helper import、`pub(super)` 可见性和回退点已固定；当前 `no code movement`，下一步只能进入 BE-001AN-03 实际抽离。

**BE-001AN-03 activation_snapshot_side_effect 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`。父级使用 `#[path = "transition_lifecycle/activation_snapshot_side_effect.rs"] mod activation_snapshot_side_effect;` 与 `use activation_snapshot_side_effect::auto_snapshot_on_activation;` 保持 activation_flow 受控调用面。下一步只能进入 BE-001AN-04 单叶 closeout。

**BE-001AN-04 activation_snapshot_side_effect closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout 已完成。该叶只承接 `auto_snapshot_on_activation` 一个 activation after-effect helper，内部步骤属于同一条 snapshot side effect 链，继续细拆不会形成新 owner，因此设置 `stop_split: true`。下一步只能进入 BE-001AO-01 `transition_lifecycle` 父叶残余判断。

**BE-001AO-01 transition_lifecycle 第四轮父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 第四轮父叶残余判断已完成。`boundary_safety`、`activation_flow`、`rollback_flow` 与 `activation_snapshot_side_effect` 均已 closeout 并设置 `stop_split: true`，但父叶仍直接拥有 `mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition` 和 `runtime_parameter_mutation_rollback_record_id`，因此保持 `stop_split: false`。下一步只能进入 BE-001AP-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线。

**BE-001AP-01 transition_record_persistence 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线已建立。当前只冻结 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 的输入输出、调用点、lifecycle entry 字段来源、persistence error 传播和 `state.parameter_mutations` 写入语义；代码未移动，目标文件未创建。下一步只能进入 BE-001AP-02 抽离方案。

**BE-001AP-02 transition_record_persistence 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案已建立。目标 child、父级 path attribute、helper import、`pub(super)` visibility、迁移清单和回退点已固定；当前 `no code movement`，下一步只能进入 BE-001AP-03 实际抽离。

**BE-001AP-03 transition_record_persistence 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 已实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 已创建并迁移 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition`；父级新增 path-attributed child 和 helper import，`runtime_parameter_mutation_rollback_record_id` 仍留在父级。下一步只能进入 BE-001AP-04 单叶 closeout。

**BE-001AP-04 transition_record_persistence closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单叶 closeout 已完成并设置 `stop_split: true`。本叶只承接 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 两个 tightly-coupled helper，继续细拆不会形成新的稳定 owner；下一步只能进入 BE-001AQ-01 `transition_lifecycle` 第五轮父叶残余判断。

**BE-001AQ-01 transition_lifecycle 第五轮父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断已完成。五个已抽子叶均已 closeout 并设置 `stop_split: true`，但父叶仍直接拥有 `runtime_parameter_mutation_rollback_record_id`，因此保持 `stop_split: false`。下一步只能进入 BE-001AR-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线。

**BE-001AR-01 rollback_record_identity 基线结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线已建立。当前只冻结 `runtime_parameter_mutation_rollback_record_id` 的输入、digest input、`canonical_json_sha256_digest`、`parameter_rollback_` prefix、`digest[..12]` 和 error mapping；代码未移动，目标文件未创建。下一步只能进入 BE-001AR-02 抽离方案。

**BE-001AR-02 rollback_record_identity 抽离方案结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案已建立。目标 child、父级 path attribute、helper import、`pub(super)` visibility、迁移清单和回退点已固定；当前 `no code movement`，下一步只能进入 BE-001AR-03 实际抽离。

**BE-001AR-03 rollback_record_identity 抽离结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 实际抽离已完成。`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 已创建并承接 `runtime_parameter_mutation_rollback_record_id`，父级通过 `#[path = "transition_lifecycle/rollback_record_identity.rs"] mod rollback_record_identity;` 与 `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;` 维持 sibling 调用面。下一步只能进入 BE-001AR-04 单叶 closeout。

**BE-001AR-04 rollback_record_identity closeout 结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单叶 closeout 已完成并设置 `stop_split: true`。本叶只承接 `runtime_parameter_mutation_rollback_record_id` 这个 deterministic id helper，继续细拆不会形成新的稳定 owner。下一步只能进入 BE-001AS-01 `transition_lifecycle` 第六轮父叶残余判断。

**BE-001AS-01 transition_lifecycle 第六轮父叶残余判断结果**:
`runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断已完成并设置 `stop_split: true`。六个子叶均已 closeout；父叶只保留 path-attributed child declarations、handler re-export、child helper imports 与 `validate_runtime_parameter_mutation_boundary` delegating wrapper。下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断。

AI 声称 `runtime.mutation.parameter_mutation.transition_lifecycle` 已推进至 BE-001AS-01 时，必须说明父叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.parameter_mutation` 父叶尚未完成，下一步只能进入 BE-001AT-01 父叶残余判断。不得宣称 parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

### 5.1.1.2.1.1 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**状态**: v4.16 BE-001AH-04 单叶 closeout 已完成，设置 `stop_split: true`；该回流已由 BE-001AI-01 父叶残余判断承接。不得越过父级 `transition_lifecycle` 直接连接 route facade、AI proposal、approval review、frontend caller 或发布过渡连接。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `src/backend/runtime/routes/mutation.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md`
- `markdown/06-milestones/v4.16.0/142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md`
- `markdown/06-milestones/v4.16.0/143-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单叶closeout.md`

**实际目标文件**: `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`。该文件由 BE-001AH-03 创建并承接三个 helper。

**职责**:
冻结 boundary validation、boundary resolution 和 safe window evaluation 这组三个纯策略 helper 的输入输出。

**关键方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `RuntimeParameterMutationBoundary` | `Ok(())` 或 error | `create_runtime_parameter_mutation`、transition lifecycle | 不得放宽 `immediate` 或非法 boundary |
| `resolve_runtime_parameter_mutation_boundary` | boundary、current sequence no | resolved boundary | activation / rollback | 不得改变 `next_cycle_start` = current + 2 |
| `evaluate_runtime_parameter_mutation_safe_window` | optional safe window snapshot | safe window state | activation / rollback | 不得改变 reason code 优先级 |

**父子通信规则**:
BE-001AH-04 已完成单叶 closeout，并已由 BE-001AI-01 回流到 `transition_lifecycle` 父叶残余判断。`boundary_safety` 只能经 `transition_lifecycle` 父级受控调用；父级保留 delegating validation wrapper，上层 `src/runtime/mutation/parameter_mutation.rs` 不得直接依赖本叶，route facade、AI proposal、approval review、frontend caller 或发布过渡连接也不得直接依赖本叶。

**细分价值判断**:
本叶值得抽离。它副作用低、输入输出稳定，并同时服务 create、activation 和 rollback；优先抽离可先把策略边界从长事务 handler 中拆出。

**幻觉检查点**:
AI 声称 `boundary_safety` 已完成 BE-001AH-04 时，必须说明当前已完成单叶 closeout 并设置 `stop_split: true`，未改变 schema/state/frontend caller，未启动发布过渡；该回流已由 BE-001AI-01 承接，后续不能继续拆本叶。

### 5.1.1.2.1.2 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AJ-04 单叶 closeout 已完成，`stop_split: true`；`activate_runtime_parameter_mutation` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`，父级通过 path-attributed child 和 handler re-export 保持调用面。下一步只能进入 BE-001AK-01 父叶残余判断，不得迁移 rollback flow、snapshot helper body、route facade、schema/frontend caller、AI proposal、approval review、AppState 或发布过渡连接。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `src/backend/runtime/routes/mutation.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md`
- `markdown/06-milestones/v4.16.0/147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md`
- `markdown/06-milestones/v4.16.0/148-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单叶closeout.md`

**职责**:
冻结 activation transaction flow 的白箱边界: capability guard、proposal record load、source run load、actor resolution、boundary resolution、safe-window application、ActivationScheduled / Activated / ActivationFailed lifecycle、run event append、activation metrics、transition persistence 和 `auto_snapshot_on_activation` 调用时机。
**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| user | auth middleware | `auth::UserId` | 只用于 scoped source run / mutation owner |
| state | app state | `AppState` | 只复用既有 store、metrics、runs、snapshot owner |
| proposal id | route path | String | 不改变 route path |
| activation request | frontend/tests | `ActivateRuntimeParameterMutationRequest` | 必须经过 capability guard、boundary 和 safe-window 判断 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| activation response | route facade/frontend/tests | `RuntimeParameterMutationRecord` | 不改变 response schema |
| mutation lifecycle event | source run | governed event append | 不改变 sequence 或 lifecycle reason |
| active parameter version | source run governance | optional version write | 仅 activated 分支写 proposed version |
| transition persistence | mutation store/cache | existing helper call | 不迁移 persistence owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `activate_runtime_parameter_mutation` | `UserId`、`AppState`、proposal id、activation request | mutation record 或 error | `runtime.mutation.parameter_mutation` re-export / route facade | 不得混入 rollback flow 或直接依赖 route facade |

**父子通信规则**:
`activation_flow` 只能作为 `transition_lifecycle` 的 child 被父级管理。实际抽离后，route facade 和 `src/runtime/mutation/parameter_mutation.rs` 仍只能经父级 `transition_lifecycle` 的受控出口调用，不得让 AI proposal、approval review、frontend caller 或发布过渡连接直接依赖本叶。
**允许调用的子模块**:
已 closeout 的 `boundary_safety` helper 只能通过父级受控依赖参与 activation boundary / safe-window 判断；`auto_snapshot_on_activation` helper body 暂留父级或后续独立候选，不在 BE-001AJ-01 迁移。
**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。
**细分价值判断**:
本叶已完成实际抽离与单叶 closeout，因为它拥有独立 public handler 和完整 activation 状态机证据，但继续细拆内部分支不会形成稳定 owner；本叶设置 `stop_split: true`，后续只能回到父叶残余判断。
**幻觉检查点**:
AI 声称 `activation_flow` 已完成 BE-001AJ-04 时，必须说明当前已完成单叶 closeout 并设置 `stop_split: true`，rollback_flow 和 snapshot helper body 未迁移，发布过渡未启动。下一步只能进入 BE-001AK-01 父叶残余判断。

### 5.1.1.2.1.3 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AL-04 单叶 closeout 已完成，设置 `stop_split: true`。`rollback_runtime_parameter_mutation` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`，父级通过 path-attributed child 与 re-export 保持上层调用面。下一步只能回到 BE-001AM-01 父叶残余判断；不得直接迁移 rollback helper、snapshot helper body、route facade、schema/frontend caller、AI proposal、approval review、AppState 或发布过渡连接。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `src/backend/runtime/routes/mutation.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md`
- `markdown/06-milestones/v4.16.0/152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md`
- `markdown/06-milestones/v4.16.0/153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md`

**实际目标文件**: `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`。BE-001AL-03 已创建该文件并承接 rollback public handler。

**职责**:
冻结 rollback transaction flow 的白箱边界: capability guard、activated-only gate、rollback attempt metric、source run load、target parameter version fallback、ledger lookup、rollback no-op protection、boundary resolution、rollback record id、governance projection、safe-window denial、RollbackScheduled / RolledBack / RollbackFailed lifecycle、run event append、rollback metrics 和 transition persistence。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| user | auth middleware | `auth::UserId` | 只用于 scoped source run / mutation owner |
| state | app state | `AppState` | 只复用既有 store、metrics 和 runs |
| proposal id | route path | String | 必须指向已 `Activated` proposal |
| rollback request | frontend/tests | `RollbackRuntimeParameterMutationRequest` | 必须经过 capability guard、ledger、boundary 和 safe-window 判断 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| rollback response | route facade/frontend/tests | `RuntimeParameterMutationRecord` | 不改变 response schema |
| mutation lifecycle event | source run | governed event append | 不改变 sequence 或 lifecycle reason |
| active parameter version | source run governance | optional version write | 仅 `RolledBack` 分支写 rollback target |
| transition persistence | mutation store/cache | existing helper call | 不迁移 persistence owner |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `rollback_runtime_parameter_mutation` | `UserId`、`AppState`、proposal id、rollback request | mutation record 或 error | `runtime.mutation.parameter_mutation` re-export / route facade | 不得混入 activation flow 或直接依赖 route facade |

**父子通信规则**:
`rollback_flow` 只能作为 `transition_lifecycle` 的 child 被父级管理。实际抽离后，route facade 和 `src/runtime/mutation/parameter_mutation.rs` 仍只能经父级 `transition_lifecycle` 的受控出口调用，不得让 AI proposal、approval review、frontend caller 或发布过渡连接直接依赖本叶。

**允许调用的子模块**:
已 closeout 的 `boundary_safety` helper 只能通过父级受控依赖参与 rollback boundary / safe-window 判断；`activation_flow` 不参与 rollback 子叶；`auto_snapshot_on_activation` helper body 暂留父级或后续独立候选，不在 BE-001AL-01 迁移。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**细分价值判断**:
本叶已完成实际抽离与单叶 closeout，设置 `stop_split: true`。它拥有独立 public handler 和完整 rollback 状态机证据；继续拆 ledger lookup、safe-window、scheduled、rolled_back 或 failed branch 只会增加父级 import 与测试定位成本，不会形成新的稳定 owner。rollback id、lifecycle entry 或 transition persistence helper 暂留父级并交给 BE-001AM-01 父叶残余判断。

**幻觉检查点**:
AI 声称 `rollback_flow` 已完成 BE-001AL-04 时，必须说明当前已完成单叶 closeout 并设置 `stop_split: true`，发布过渡未启动。下一步只能回到 BE-001AM-01 父叶残余判断。

### 5.1.1.2.1.4 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**状态**: v4.16 BE-001AN-04 单叶 closeout 已完成，设置 `stop_split: true`。`auto_snapshot_on_activation` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`；父级通过 path-attributed child 和 helper import 保持 `activation_flow` 受控调用面。下一步只能回到 BE-001AO-01 父叶残余判断；不得直接迁移 shared lifecycle/persistence helper、迁移 rollback helper、schema/frontend caller、AI proposal、approval review、AppState 或发布过渡连接。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `src/backend/runtime/routes/mutation.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md`
- `markdown/06-milestones/v4.16.0/157-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离记录.md`
- `markdown/06-milestones/v4.16.0/158-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单叶closeout.md`

**职责**:
冻结 activation 后自动快照副作用的白箱边界: config generation 递增、generation history truncation、snapshot id、deployment signature snapshot payload/signature、`DeploymentSignatureSnapshot`、`canonical_json_sha256_digest`、`atomic_write_json`、`safe_eprintln!` fallback、in-memory `state.snapshots` insert 和 metric baseline read。

**关键候选方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `auto_snapshot_on_activation` | `AppState`、`auth::UserId`、`RuntimeParameterMutationRecord` | config generation / snapshot file / in-memory snapshot side effect | `activation_flow` via parent helper | 不得迁移 snapshot owner、AppState、schema、frontend caller 或 release transition guard |

**父子通信规则**:
`activation_snapshot_side_effect` 只能作为 `transition_lifecycle` 的 child 被父级管理。BE-001AN-03 已按方案实际抽离，`activation_flow` 仍只能经父级 `transition_lifecycle` 的受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller 或发布过渡连接直接依赖本叶。

**细分价值判断**:
本叶已完成 BE-001AN-04 closeout 并设置 `stop_split: true`。它是独立副作用域，但当前只拥有 `auto_snapshot_on_activation` 一个稳定 helper；内部 config generation、snapshot build、atomic write 和 memory insert 是同一条 activation after-effect 链。继续细拆只会制造微文件，不形成新的稳定 owner。

**幻觉检查点**:
AI 声称 `activation_snapshot_side_effect` 已完成 BE-001AN-04 时，必须说明当前已 closeout 并设置 `stop_split: true`，下一步只能回到 BE-001AO-01 父叶残余判断。不得宣称 shared helper 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

### 5.1.1.2.1.5 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**状态**: v4.16 BE-001AP-04 单叶 closeout 已完成，`stop_split: true`。`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`；本叶不再继续细拆，下一步只能回流到 BE-001AQ-01 父叶残余判断。
**最新状态补充**: BE-001AP-04 closeout 已完成；目标 child、父级 path attribute、helper import 与 `pub(super)` visibility 已落地并验证通过。下一步只能进入 BE-001AQ-01 父叶残余判断。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/frontend_api_types.rs`
- `src/runtime_persistence.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md`
- `markdown/06-milestones/v4.16.0/162-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离记录.md`
- `markdown/06-milestones/v4.16.0/163-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单叶closeout.md`

**实际目标模块名**: `transition_record_persistence`，实际 Rust 文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`。

**职责**:
冻结 transition lifecycle entry 构造与 transition record persistence 白箱边界: `RuntimeParameterMutationLifecycleEntry` 字段来源、`mutation_event_contract(status)` reason code、`persist_runtime_parameter_mutation_record` 写入、`io_error` 传播、`state.parameter_mutations` in-memory index 和 `auth::scoped_key` key 语义。

**关键方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `mutation_lifecycle_entry` | `RuntimeParameterMutationStatus`、`FrontendRuntimeEvent`、sequence no、message | `RuntimeParameterMutationLifecycleEntry` | `activation_flow`、`rollback_flow` via parent helper | 不得改变 reason code、event id、sequence no、occurred_at_ms 或 message |
| `persist_runtime_parameter_mutation_transition` | `AppState`、`auth::UserId`、`RuntimeParameterMutationRecord` | persisted record + in-memory mutation index | `activation_flow`、`rollback_flow` via parent helper | 不得改变 persistence error propagation、lock owner 或 scoped key |

**父子通信规则**:
`transition_record_persistence` 只能作为 `transition_lifecycle` 的 child 被父级管理。BE-001AP-03 实际抽离后，activation / rollback 子叶仍经父级受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。ASCII guard: `release transition guard`。

**细分价值判断**:
本叶已完成抽离与单叶 closeout，设置 `stop_split: true`。它同时服务 activation 与 rollback 两条 public handler 流，拥有稳定输入输出和可复用 persistence 语义；继续拆 lifecycle builder、persistence writer 或 memory-index writer 不会形成新的稳定 owner。rollback id helper 仍不混入本叶，避免把单一 rollback-only id generation 与 shared transition record persistence 绑死。

**幻觉检查点**:
AI 声称 `transition_record_persistence` 已完成 BE-001AP-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，rollback id 仍留在父级，下一步只能回到 BE-001AQ-01 父叶残余判断。不得宣称 rollback id 已迁移、AppState/schema/frontend caller 已改变、发布过渡已启动或父叶已完成。

### 5.1.1.2.1.6 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity`
**父模块**: `runtime.mutation.parameter_mutation.transition_lifecycle`
**状态**: v4.16 BE-001AR-04 单叶 closeout 已完成，设置 `stop_split: true`。`runtime_parameter_mutation_rollback_record_id` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`，父级 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 通过 path-attributed child 与 helper import 保持调用面。下一步只能回到 BE-001AS-01 父叶残余判断。
**最新状态补充**: BE-001AR-02 抽离方案已建立；目标 child、父级 path attribute、helper import、`pub(super)` visibility 和回退点已固定，但 Rust 目标文件尚未创建。下一步只能进入 BE-001AR-03 实际抽离。
**最新状态补充**: BE-001AR-03 实际抽离已完成；目标 Rust 文件已创建，helper 已迁移，但本叶尚未 closeout。下一步只能进入 BE-001AR-04 单叶 closeout。
**最新状态补充**: BE-001AR-04 单叶 closeout 已完成；本叶设置 `stop_split: true`，后续不能继续拆本叶，下一步只能回到 BE-001AS-01 父叶残余判断。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md`
- `markdown/06-milestones/v4.16.0/167-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离记录.md`
- `markdown/06-milestones/v4.16.0/168-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单叶closeout.md`

**实际目标模块名**: `rollback_record_identity`，实际文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`。
**实际父级声明**: `#[path = "transition_lifecycle/rollback_record_identity.rs"] mod rollback_record_identity;`
**实际父级导入**: `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;`
**实际 child visibility**: `pub(super) fn runtime_parameter_mutation_rollback_record_id(...)`

**职责**:
冻结 rollback record deterministic identity 白箱边界: digest input、`canonical_json_sha256_digest`、`internal_error` mapping、`parameter_rollback_` prefix、`created_at_ms` 和 `digest[..12]` output segment。

**关键候选方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `runtime_parameter_mutation_rollback_record_id` | `source_id`、`rollback_of`、`RuntimeParameterMutationTarget`、`created_at_ms`、`source_event_count`、`proposed_parameter_version` | `Result<String, (StatusCode, String)>` rollback proposal id | `rollback_flow` via parent helper | 不得改变 digest input、prefix、slice length 或 error mapping |

**父子通信规则**:
`rollback_record_identity` 只能作为 `transition_lifecycle` 的 child 被父级管理。BE-001AR-03 已实际抽离；`rollback_flow` 仍经父级受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。ASCII guard: `release transition guard`。

**细分价值判断**:
本叶已完成单叶 closeout，并设置 `stop_split: true`。它只拥有 `runtime_parameter_mutation_rollback_record_id` 一个 deterministic id helper；继续拆 digest input、digest executor 或 id formatter 不会形成新的稳定 owner，反而会增加父级 import 与 visibility 成本。

**幻觉检查点**:
AI 声称 `rollback_record_identity` 已完成 BE-001AR-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`；helper 已迁入 child，但 `rollback_flow` 未直接依赖 child，下一步只能回到 BE-001AS-01 父叶残余判断。不得宣称 rollback_flow 已回改、transition_lifecycle 父叶完成、AppState/schema/frontend caller 已改变或发布过渡已启动。

### 5.1.1.2.2 `runtime.mutation.parameter_mutation.proposal_creation`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.proposal_creation`
**父模块**: `runtime.mutation.parameter_mutation`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AU-04 单叶 closeout 已完成；实际抽离等价成立，本叶设置 `stop_split: true`。下一步只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**真实文件**:
- `src/runtime/mutation/parameter_mutation.rs`
- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `tests/api_mutation.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/170-runtime.mutation.parameter_mutation父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md`
- `markdown/06-milestones/v4.16.0/173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md`
- `markdown/06-milestones/v4.16.0/174-runtime.mutation.parameter_mutation.proposal_creation单叶closeout.md`

**职责**:
冻结 parameter mutation proposal creation 白箱边界: capability guard、source run load、parameter version canonicalization、noop rejection、proposal id generation、governance build、proposal event append、persistence write、metrics update 与 in-memory index insert。本节点不拥有 list/detail 查询、activation/rollback transition lifecycle、AI proposal、approval review、AppState、schema、frontend caller 或发布过渡连接。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_parameter_mutation` | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeParameterMutationRequest>` | `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>` | `runtime.mutation.parameter_mutation` / route facade | 不得迁移 list/detail、AI proposal 或 approval |

**关键 helper**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `runtime_parameter_mutation_record_id` | `CreateRuntimeParameterMutationRequest`、`created_at_ms`、`source_event_count`、`proposed_parameter_version` | `Result<String, (StatusCode, String)>` proposal id | `create_runtime_parameter_mutation` | 不得改变 digest input、`parameter_mutation_` prefix 或 `digest[..12]` |

**BE-001AU-02 抽离方案**:
后续 BE-001AU-03 只允许创建 proposal_creation child，并只移动 `create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id`。父级必须保留 route facade 出口和 list/detail handler；child 必须通过 `use super::*` 复用父级白箱输入，不得新增横向依赖。

**BE-001AU-03 抽离结果**:
`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 child。父级通过 `#[path = "parameter_mutation/proposal_creation.rs"] mod proposal_creation;`、`pub(crate) use proposal_creation::create_runtime_parameter_mutation;` 维持 route facade 调用面；child 通过 `use super::*` 复用父级白箱输入。

**BE-001AU-04 closeout 结果**:
本叶设置 `stop_split: true`。它只拥有一个 public handler 和一个私有 deterministic id helper；继续细拆会把单一 proposal transaction 切成 record builder、event append、persistence wrapper 等弱边界，不符合当前递归收益。

**父子通信规则**:
BE-001AU-04 已完成单叶 closeout。`proposal_creation` 只能经 `runtime.mutation.parameter_mutation` 父级受控调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。ASCII guard: `release transition guard`。

**细分价值判断**:
本叶不继续细拆，设置 `stop_split: true`。`create_runtime_parameter_mutation` 是单一 proposal transaction，`runtime_parameter_mutation_record_id` 只服务该 transaction；继续拆 record build、event append、persistence 或 metrics wrapper 会增加父级 import 与 visibility 成本，但不会形成稳定 owner。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**幻觉检查点**:
AI 声称 `proposal_creation` 已完成 BE-001AU-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`；下一步只能回到 BE-001AV-01 父叶残余判断。不得宣称 list/detail 已迁移、AppState/schema/frontend caller 已改变、发布过渡已启动或 `runtime.mutation.parameter_mutation` 父叶已经完成。

### 5.1.1.2.3 `runtime.mutation.parameter_mutation.record_query`

**层级路径**: `root.backend.runtime.mutation.parameter_mutation.record_query`
**父模块**: `runtime.mutation.parameter_mutation`
**路由入口**: `backend.runtime.routes.mutation`
**状态**: v4.16 BE-001AW-04 单叶 closeout 已完成，设置 `stop_split: true`。`list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 已迁入 `src/runtime/mutation/parameter_mutation/record_query.rs`。下一步只能回到 BE-001AX-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
**真实文件**:
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/parameter_mutation.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/177-runtime.mutation.parameter_mutation.record_query抽离方案.md`
- `markdown/06-milestones/v4.16.0/178-runtime.mutation.parameter_mutation.record_query抽离记录.md`
- `markdown/06-milestones/v4.16.0/179-runtime.mutation.parameter_mutation.record_query单叶closeout.md`

**职责**:
冻结 parameter mutation read/query 白箱边界: mutation records list、source kind filtering、source id filtering、created_at / proposal_id ordering、pagination、scoped in-memory detail lookup 和 persistence fallback。本节点不拥有 proposal creation、activation/rollback transition lifecycle、AI proposal、approval review、AppState、schema、frontend caller 或发布过渡连接。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_runtime_parameter_mutations` | `State<AppState>`、`Query<RuntimeParameterMutationListQuery>` | `Result<Json<PaginatedResponse<RuntimeParameterMutationRecord>>, (StatusCode, String)>` | route facade | 不得改变 filtering / ordering / pagination |
| `get_runtime_parameter_mutation_detail` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>` | route facade | 不得绕过 scoped in-memory lookup 或 persistence fallback |

**关键 helper / owner**:
`list_runtime_parameter_mutation_records`；`load_runtime_parameter_mutation_record`；`clean_optional_filter`；`PaginationQuery`；`paginate`；`auth::scoped_key`；`state.parameter_mutations`；`mutation_store_dir`。

**父子通信规则**:
BE-001AW-03 已完成实际抽离。`record_query` 只能经 `runtime.mutation.parameter_mutation` 父级受控调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。ASCII guard: `release transition guard`。

**BE-001AW-02 抽离方案**:
下一步只允许创建计划 child，并在父级新增 path-attributed child 声明与双 handler re-export；child 必须通过 `use super::*;` 使用父级白箱输入。只允许迁移 `list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail`，保持 filtering、ordering、pagination、scoped in-memory lookup 和 persistence fallback 等价。

**BE-001AW-03 抽离结果**:
目标 child 已创建并承接 list/detail handler。父级只保留 `#[path = "parameter_mutation/record_query.rs"] mod record_query;` 与 `pub(crate) use record_query::{get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations};` 兼容出口。route facade、AppState、schema、frontend caller 和 persistence owner 均未改变。

**BE-001AW-04 closeout 结果**:
本叶设置 `stop_split: true`。list/detail 是同一个 mutation record read model，继续拆分不会形成新的稳定 owner；后续只能回到父叶残余判断。

**细分价值判断**:
本叶已完成实际抽离与单叶 closeout，设置 `stop_split: true`。list/detail 是同一 mutation record read model，一起触达 persistence list/load、in-memory cache、filtering、sorting 和 pagination；继续拆分会增加微文件和父级 re-export 面，不形成新的稳定 owner。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**幻觉检查点**:
AI 声称 `record_query` 已完成 BE-001AW-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.parameter_mutation` 父叶尚未完成。不得宣称 AppState/schema/frontend caller 已改变、发布过渡已启动或父叶已经完成。

### 5.1.1.3 `runtime.mutation.ai_proposal`

**层级路径**: `root.backend.runtime.mutation.ai_proposal`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断已完成并设置父叶 `stop_split: true`。`runtime.mutation.ai_proposal.static_check`、`runtime.mutation.ai_proposal.source_governance_identity`、`runtime.mutation.ai_proposal.event_lifecycle`、`runtime.mutation.ai_proposal.record_query`、`runtime.mutation.ai_proposal.approval_review`、`runtime.mutation.ai_proposal.approval_persistence`、`runtime.mutation.ai_proposal.sandbox_trigger`、`runtime.mutation.ai_proposal.status_transition` 与 `runtime.mutation.ai_proposal.proposal_creation` 均已完成 closeout；父叶生产代码只保留 path-attributed child、handler re-export 和受控 helper 连接。下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。`AppState`、schema owner、frontend caller、runtime persistence owner、route facade 和 release transition guard 均未改变。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
- `src/runtime/mutation/ai_proposal/event_lifecycle.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_persistence.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/mutation/ai_proposal/sandbox_trigger.rs`
- `src/runtime/mutation/ai_proposal/status_transition.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/frontend_api_types.rs`
- `src/runtime_persistence.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/181-runtime.mutation.ai_proposal单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/182-runtime.mutation.ai_proposal抽离方案.md`
- `markdown/06-milestones/v4.16.0/183-runtime.mutation.ai_proposal抽离记录.md`
- `markdown/06-milestones/v4.16.0/184-runtime.mutation.ai_proposal单叶closeout.md`
- `markdown/06-milestones/v4.16.0/185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/186-runtime.mutation.ai_proposal.static_check抽离方案.md`
- `markdown/06-milestones/v4.16.0/187-runtime.mutation.ai_proposal.static_check抽离记录.md`
- `markdown/06-milestones/v4.16.0/188-runtime.mutation.ai_proposal.static_check单叶closeout.md`
- `markdown/06-milestones/v4.16.0/189-runtime.mutation.ai_proposal父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md`
- `markdown/06-milestones/v4.16.0/192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md`
- `markdown/06-milestones/v4.16.0/193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`
- `markdown/06-milestones/v4.16.0/194-runtime.mutation.ai_proposal第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/199-runtime.mutation.ai_proposal第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/201-runtime.mutation.ai_proposal.record_query抽离方案.md`
- `markdown/06-milestones/v4.16.0/202-runtime.mutation.ai_proposal.record_query抽离记录.md`
- `markdown/06-milestones/v4.16.0/203-runtime.mutation.ai_proposal.record_query单叶closeout.md`
- `markdown/06-milestones/v4.16.0/204-runtime.mutation.ai_proposal第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/206-runtime.mutation.ai_proposal.approval_review抽离方案.md`
- `markdown/06-milestones/v4.16.0/207-runtime.mutation.ai_proposal.approval_review抽离记录.md`
- `markdown/06-milestones/v4.16.0/208-runtime.mutation.ai_proposal.approval_review单叶closeout.md`
- `markdown/06-milestones/v4.16.0/209-runtime.mutation.ai_proposal第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md`
- `markdown/06-milestones/v4.16.0/212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md`
- `markdown/06-milestones/v4.16.0/213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md`
- `markdown/06-milestones/v4.16.0/214-runtime.mutation.ai_proposal第六轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md`
- `markdown/06-milestones/v4.16.0/217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md`
- `markdown/06-milestones/v4.16.0/218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md`
- `markdown/06-milestones/v4.16.0/219-runtime.mutation.ai_proposal第七轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/221-runtime.mutation.ai_proposal.status_transition抽离方案.md`
- `markdown/06-milestones/v4.16.0/222-runtime.mutation.ai_proposal.status_transition抽离记录.md`
- `markdown/06-milestones/v4.16.0/223-runtime.mutation.ai_proposal.status_transition单叶closeout.md`
- `markdown/06-milestones/v4.16.0/224-runtime.mutation.ai_proposal第八轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md`
- `markdown/06-milestones/v4.16.0/227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md`
- `markdown/06-milestones/v4.16.0/228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md`
- `markdown/06-milestones/v4.16.0/229-runtime.mutation.ai_proposal第九轮父叶残余判断.md`

**职责**:
承载 runtime AI proposal 与 approval review handler 域的白箱边界，冻结候选生成、静态检查、proposal 查询、approval 查询、approve/reject/claim、sandbox gate、approval persistence、状态迁移和 evidence event contract。本节点不拥有 parameter mutation、report、evidence、experiment、ops、frontend caller、schema owner、AppState owner 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| AI proposal candidate | `backend.runtime.routes.mutation` | `CreateRuntimeAiProposalRequest` | 保持 source_kind/source_id、target、model、prompt_hash、evidence_hash、reason、config_domain_binding |
| proposal list query | API caller | `RuntimeAiProposalListQuery` | 保持 source_kind/source_id/status filtering 与倒序排序 |
| approval action | approve/reject/claim route | `ApprovalActionRequest` + proposal id | 保持 actor_id、comment、review state guard 和 reviewer lifecycle |
| source context | run/backtest owner | run/backtest record + governance snapshot | 保持 event_count、sequence cursor 和 governance copy |
| sandbox report | sandbox verification owner | proposal sandbox report | approve 前保持 report existence 与 passed gate |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| proposal record | frontend / tests | `RuntimeAiProposalRecord` | 不改变 status、static_check、source_evidence、governance、config_domain_binding、lifecycle |
| approval record | frontend / tests | `RuntimeApprovalRecord` | 不改变 review_state、reviewers、sandbox_report_url、lifecycle |
| runtime event | run/backtest evidence | `FrontendRuntimeEvent` | 不改变 AIProposalCreated / StaticCheckPassed / StaticCheckFailed / Denied / Approved contract |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_ai_proposal` | user id、`AppState`、AI proposal request | AI proposal record | `backend.runtime.routes.mutation` | 不得绕过 static check、capability、approval record 或 sandbox trigger |
| `list_runtime_ai_proposals` | list query | AI proposal records | `backend.runtime.routes.mutation` | 不得改变 filtering 或 sorting |
| `get_runtime_ai_proposal_detail` | proposal id | AI proposal record | `backend.runtime.routes.mutation` | 不得绕过 scoped memory lookup 或 disk fallback |
| `list_runtime_approvals` | user id、query | approval records | `backend.runtime.routes.mutation` | 不得改变 scoped visibility 或 review_state filtering |
| `get_runtime_approval_detail` | approval id | approval record | `backend.runtime.routes.mutation` | 不得绕过 memory-first lookup 或 disk fallback |
| `approve_ai_proposal` | proposal id、approval action | approval record | `backend.runtime.routes.mutation` | 不得改变 sandbox gate、reviewer quorum、approval lifecycle 或 proposal approved transition |
| `reject_ai_proposal` | proposal id、approval action | approval record | `backend.runtime.routes.mutation` | 不得丢失 rejection reason 或 proposal denied transition |
| `claim_ai_proposal_review` | proposal id、approval action | approval record | `backend.runtime.routes.mutation` | 不得改变 pending-only claim guard 或 reviewer assignment |

**关键 helper 基线**:
`static_check` child 已承接 `validate_hash_identity`、`validate_ai_model_identity`、`ai_proposal_static_check_result`、`validate_ai_proposal_config_domain_binding` 等 static check helper。`source_governance_identity` child 已承接 `RuntimeAiProposalSourceContext`、`load_runtime_ai_proposal_source_context`、`runtime_ai_proposal_governance` 和 `runtime_ai_proposal_record_id`。`event_lifecycle` child 已承接 `ai_proposal_event_contract`、`build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry` 与 `persist_runtime_ai_proposal_transition`。`record_query` child 已承接 `load_runtime_ai_proposal_for_user`、`list_runtime_ai_proposals` 与 `get_runtime_ai_proposal_detail`。`approval_review` child 已承接 `list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal` 与 `claim_ai_proposal_review`。`approval_persistence` child 已承接 `persist_approval` 与 `load_approval_from_disk`。`sandbox_trigger` child 已承接 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 与 `spawn_ai_proposal_sandbox_verification`。`status_transition` child 已承接 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`。`proposal_creation` 已承接 `create_runtime_ai_proposal`。九个 child 均已 closeout 且 `stop_split: true`；父级生产代码只保留 path-attributed child、受控 helper import 与 public handler re-export。

**HTTP route 基线**:
| Route | Method | Handler |
| --- | --- | --- |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` |
| `/api/runtime/ai-proposals` | POST | `create_runtime_ai_proposal` |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` |

**状态与锁**:
`AppState` 继续拥有 `ai_proposals` 与 `approval_records`。approval action 路径必须保持既有 `approval_records -> ai_proposals` lock order 和写入语义；sandbox verification background task、JoinHandle 监控和 `sandbox_report_url` 回写语义不得改变。

**父级通信规则**:
`runtime.mutation.ai_proposal` 只能经父级 `backend.runtime.routes.mutation` 暴露 HTTP route；不得横向接管 `runtime.mutation.parameter_mutation`、report、evidence、experiment、ops、strategy_config、frontend caller 或 executor。状态 owner 仍是 `AppState`，schema owner 仍是 `src/frontend_api_types.rs`，共享 persistence owner 仍是 `src/runtime_persistence.rs`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**细分价值判断**:
本父叶已推进至 BE-001BQ-01，并设置 `stop_split: true`。`static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition` 与 `proposal_creation` 均已 closeout 且 `stop_split: true`。父叶生产职责仅剩白箱网络接线层；下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal` 已推进至 BE-001BQ-01 时，必须说明父叶已 closeout 并设置 `stop_split: true`，但 `backend.runtime.routes` 父叶尚未完成残余判断。不得宣称 AppState/schema/frontend caller 已改变、release transition 已启动、整理或重构已经完成。

### 5.1.1.3.1 `runtime.mutation.ai_proposal.static_check`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.static_check`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001AZ-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/static_check.rs` 承接 static check helper 与对应单测，设置 `stop_split: true`。继续细拆 hash identity、model identity、config domain binding 或 v4 artifact analysis 只会增加 helper 接线，不会形成新的稳定 owner。父级 `runtime.mutation.ai_proposal` 仍需进入 BE-001BA-01 残余判断；approval review、record query、AppState、schema owner、frontend caller、route facade 和 release transition guard 均未改变。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `markdown/06-milestones/v4.16.0/185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/186-runtime.mutation.ai_proposal.static_check抽离方案.md`
- `markdown/06-milestones/v4.16.0/187-runtime.mutation.ai_proposal.static_check抽离记录.md`
- `markdown/06-milestones/v4.16.0/188-runtime.mutation.ai_proposal.static_check单叶closeout.md`

**职责**:
承载 AI proposal candidate 的静态校验白箱边界，冻结 hash identity、model identity、source evidence、noop version、reason、config domain binding、v4 backtest source requirement、non-v4 run source requirement 和 v4 artifact analysis。本节点不拥有 create transaction、approval review、record query、source governance、event lifecycle、sandbox trigger、AppState、schema owner、frontend caller 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| AI proposal request | `runtime.mutation.ai_proposal.create_runtime_ai_proposal` | `CreateRuntimeAiProposalRequest` | 保持 source_kind、target、reason、model、config_domain_binding |
| old/new parameter version | parent shared helper | `String` | 保持 canonical digest 比较 |
| source event count | source context | `usize` | 0 必须产生 `missing_source_evidence` |
| checked timestamp | parent create flow | `u64` | 保持原时间来源 |
| v4 backtest artifact | artifact owner | `V4BacktestArtifact` | 只读 analysis summary |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| static check result | proposal record | `RuntimeAiProposalStaticCheckResult` | 不改变 status、reason_code、message、checked_at_ms、details |
| binding validation details | static check result | `RuntimeAiProposalStaticCheckDetail` | 不改变 detail code / target / message |
| v4 artifact analysis | future evidence / tests | `serde_json::Value` | 不改变 `analysis_version`、counts、risk ratio、fill rate |

**关键 helper 基线**:
`validate_hash_identity`；`is_valid_hash_identity`；`validate_ai_model_identity`；`ai_proposal_static_check_result`；`is_v4_ai_proposal_target`；`expected_config_domain_for_target`；`validate_ai_proposal_config_domain_binding`；`analyze_v4_backtest_artifact_for_ai`。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `ai_proposal_static_check_result` | `CreateRuntimeAiProposalRequest`、old/new parameter version、source event count、checked timestamp | `RuntimeAiProposalStaticCheckResult` | `runtime.mutation.ai_proposal.create_runtime_ai_proposal` via `pub(super)` | 不得成为 route handler 或横向服务 |
| `validate_ai_model_identity` | `RuntimeAiModelIdentity` | `Result<(), (StatusCode, String)>` | `runtime.mutation.ai_proposal.create_runtime_ai_proposal` via `pub(super)` | 不得绕过 provider/model/model_version 必填 |
| `validate_hash_identity` | prompt/evidence hash、target、label | `Result<(), (StatusCode, String)>` | `runtime.mutation.ai_proposal.create_runtime_ai_proposal` via `pub(super)` | 不得把 hash 校验开放给 sibling |
| `analyze_v4_backtest_artifact_for_ai` | `V4BacktestArtifact` | `serde_json::Value` | 当前 child 内部单测 / future evidence analysis | 不得接管 artifact schema owner |

**父级通信规则**:
`runtime.mutation.ai_proposal.static_check` 只能被父级 `runtime.mutation.ai_proposal` 调用。父级当前只通过 `ai_proposal_static_check_result`、`validate_ai_model_identity`、`validate_hash_identity` 三个 `pub(super)` helper 受控连接 child；其余 helper 保持 child private。不得横向接管 approval_review、record_query、source_governance_identity、event_lifecycle、sandbox_trigger、parameter mutation、AppState、schema owner、frontend caller 或 runtime persistence owner。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**细分价值判断**:
本批已完成第一轮实际抽离与单叶 closeout，本叶设置 `stop_split: true`。hash identity、model identity、config domain binding 与 v4 artifact analysis 均为同一个 static check owner 的内部 helper，继续细拆不会产生新 owner。后续必须进入 BE-001BA-01 父叶残余判断。当前不得迁移 approval gate、record_query、approval_review、sandbox_trigger 或 create handler。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.static_check` 已推进至 BE-001AZ-04 时，必须说明当前只完成 static_check 单叶 closeout 并设置 `stop_split: true`。不得宣称 `runtime.mutation.ai_proposal` 父级已完成、approval review 已拆分、record query 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动、整理或重构已经完成。

### 5.1.1.3.2 `runtime.mutation.ai_proposal.source_governance_identity`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.source_governance_identity`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BB-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/source_governance_identity.rs` 承接 source context、governance projection 与 proposal record identity helper，设置 `stop_split: true`。继续细拆 source loader、governance projection、record id digest 或 context struct 只会增加父子 helper 接线，不会形成新的稳定 owner。event lifecycle、record query、approval review、approval persistence、sandbox trigger、AppState、schema owner、frontend caller、route facade 和 release transition guard 均未改变。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `markdown/06-milestones/v4.16.0/189-runtime.mutation.ai_proposal父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md`
- `markdown/06-milestones/v4.16.0/192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md`
- `markdown/06-milestones/v4.16.0/193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`

**职责**:
承载 AI proposal create flow 的 source context、governance projection 和 proposal record identity 等价基线。该节点不拥有 static check、event lifecycle、record query、approval review、approval persistence、sandbox trigger、AppState、schema owner、frontend caller 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| source selector | `create_runtime_ai_proposal` | `RuntimeEvidenceSourceKind` + `source_id` | `Run` 与 `Backtest` 分支语义不变 |
| state/user | parent handler | `AppState` + `auth::UserId` | 只读 scoped run/backtest source |
| source governance | run/backtest record | `RuntimeGovernanceSnapshot` | 保持 capability / deployment / permission boundary |
| record identity input | create flow | request、timestamp、source_event_count、proposed parameter version | digest 输入不变 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| source context | `create_runtime_ai_proposal` | `RuntimeAiProposalSourceContext` | `graph_id`、`event_count`、`current_sequence_no`、`governance` 不变 |
| proposal governance | proposal record | `RuntimeAiProposalGovernance` | 字段映射不变 |
| proposal id | proposal record | `ai_proposal_{created_at_ms}_{digest[..12]}` | prefix、digest 输入、12 位截断不变 |

**关键 helper 基线**:
`RuntimeAiProposalSourceContext`；`load_runtime_ai_proposal_source_context`；`runtime_ai_proposal_governance`；`runtime_ai_proposal_record_id`。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `load_runtime_ai_proposal_source_context` | `AppState`、`auth::UserId`、`RuntimeEvidenceSourceKind`、`source_id` | `RuntimeAiProposalSourceContext` | 当前 `create_runtime_ai_proposal` | 不得接管 run/backtest persistence owner |
| `runtime_ai_proposal_governance` | source governance、old/proposed parameter version | `RuntimeAiProposalGovernance` | 当前 `create_runtime_ai_proposal` | 不得改变 permission boundary 字段映射 |
| `runtime_ai_proposal_record_id` | request、timestamp、event count、proposed version | `Result<String, (StatusCode, String)>` | 当前 `create_runtime_ai_proposal` | 不得改变 digest input 或 id prefix |

**父级通信规则**:
`runtime.mutation.ai_proposal.source_governance_identity` 只能被父级 `runtime.mutation.ai_proposal` 调用。父级当前只通过 `load_runtime_ai_proposal_source_context`、`runtime_ai_proposal_governance` 和 `runtime_ai_proposal_record_id` 三个 `pub(super)` helper 受控连接 child；`RuntimeAiProposalSourceContext` 与字段保持 `pub(super)` 以供父级 create flow 读取，不得把 source context 暴露给 sibling。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**细分价值判断**:
本批已完成单叶 closeout 并设置 `stop_split: true`。source loader、governance projection、record id digest 与 context struct 均为同一个 create-flow source identity owner 的内部 helper，继续细拆不会产生新 owner。后续必须进入 BE-001BC-01 父叶残余判断；不得继续细拆 source_governance_identity，也不得迁移 event lifecycle / approval review / record query / sandbox trigger 或启动 release transition。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.source_governance_identity` 已推进至 BE-001BB-04 时，必须说明当前只完成本叶 closeout 并设置 `stop_split: true`。不得宣称 `runtime.mutation.ai_proposal` 父级已完成、approval review 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动、整理或重构已经完成。

### 5.1.1.3.3 `runtime.mutation.ai_proposal.event_lifecycle`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.event_lifecycle`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BD-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/event_lifecycle.rs` 承接 event contract、runtime event builder、lifecycle entry 与 proposal transition persistence helper，并设置 `stop_split: true`。继续细拆 event contract、runtime event builder、lifecycle entry 或 proposal transition persistence 只会增加父子 helper 接线，不会形成新的稳定 owner。record query、approval review、approval persistence、sandbox trigger、status_transition、AppState、schema owner、frontend caller、route facade 和 release transition guard 均未改变。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
- `src/runtime/mutation/ai_proposal/event_lifecycle.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/194-runtime.mutation.ai_proposal第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`

**职责**:
承载 AI proposal / approval 事务中的 event contract、event payload、lifecycle entry 与 proposal transition persistence 等价基线。该节点不拥有 record query、approval review、approval persistence、sandbox trigger、AppState、schema owner、frontend caller 或发布过渡连接。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| proposal status | create / approval transition | `RuntimeAiProposalStatus` | event type / reason code 映射不变 |
| proposal record | parent handler | `RuntimeAiProposalRecord` | event payload 字段不变 |
| event timestamp | parent handler | `u64` | event id 与 occurred_at_ms 语义不变 |
| sequence number / message | parent handler | `u64` / string | lifecycle entry 字段映射不变 |
| state / user / record | parent handler | `AppState` / `auth::UserId` / record | persistence 与 scoped in-memory update 不变 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| event contract pair | event builder / lifecycle entry | `(&'static str, &'static str)` | event type / reason code 不变 |
| runtime event | proposal source evidence | `FrontendRuntimeEvent` | payload schema、severity、summary、default envelope 不变 |
| lifecycle entry | proposal record lifecycle | `RuntimeAiProposalLifecycleEntry` | status、event_id、sequence_no、reason_code 不变 |
| persisted proposal record | disk / `state.ai_proposals` | runtime AI proposal record | scoped key 与 disk write 不变 |

**关键 helper 基线**:
`ai_proposal_event_contract`；`build_runtime_ai_proposal_event`；`ai_proposal_lifecycle_entry`；`persist_runtime_ai_proposal_transition`。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `ai_proposal_event_contract` | `RuntimeAiProposalStatus` | event type / reason code pair | parent event builder / lifecycle entry | 不得改变 status 映射 |
| `build_runtime_ai_proposal_event` | proposal record、status、timestamp | `FrontendRuntimeEvent` | parent create / approval transition | 不得接管 schema owner |
| `ai_proposal_lifecycle_entry` | status、event、sequence、message | `RuntimeAiProposalLifecycleEntry` | parent create / approval transition | 不得改变 sequence 或 reason code |
| `persist_runtime_ai_proposal_transition` | `AppState`、user、record | `Result<(), (StatusCode, String)>` | parent create / approval transition | 不得迁移 AppState owner 或 persistence owner |

**父级通信规则**:
本 child 只能被父级 `runtime.mutation.ai_proposal` 调用。父级当前只通过 `build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry` 与 `persist_runtime_ai_proposal_transition` 三个 `pub(super)` helper 受控连接 child；`ai_proposal_event_contract` 保持 child private，不得把 event builder、lifecycle entry 或 transition persistence 暴露给 sibling。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**细分价值判断**:
本批已完成单叶 closeout 并设置 `stop_split: true`。event contract、runtime event builder、lifecycle entry 与 proposal transition persistence 都服务同一条 AI proposal 状态投影链路；继续细拆不会产生新 owner。后续必须进入 BE-001BE-01 父叶残余判断；不得继续细拆 event_lifecycle，也不得迁移 record_query / approval_review / approval_persistence / sandbox_trigger 或启动 release transition。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.event_lifecycle` 已推进至 BE-001BD-04 时，必须说明当前只完成本叶 closeout 并设置 `stop_split: true`。不得宣称 record_query、approval_review、approval_persistence、sandbox_trigger、AppState/schema/frontend caller 已改变、release transition 已启动、整理或重构已经完成。

### 5.1.1.3.4 `runtime.mutation.ai_proposal.record_query`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.record_query`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BF-04 单叶 closeout 已完成并设置 `stop_split: true`。`src/runtime/mutation/ai_proposal/record_query.rs` 承接 proposal list/detail/read-through loader。继续拆成 list/detail/loader 微文件不会形成稳定 owner，只会增加父子接线面。下一步只能进入 BE-001BG-01 `runtime.mutation.ai_proposal` 父叶残余判断，不得混入 approval review 或 status transition。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime_persistence.rs`
- `src/runtime/mod.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/199-runtime.mutation.ai_proposal第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/201-runtime.mutation.ai_proposal.record_query抽离方案.md`
- `markdown/06-milestones/v4.16.0/202-runtime.mutation.ai_proposal.record_query抽离记录.md`
- `markdown/06-milestones/v4.16.0/203-runtime.mutation.ai_proposal.record_query单叶closeout.md`

**职责**:
承载 AI proposal record read model 的白箱基线，冻结 list/detail/read-through loader 对 proposal record 的读取、过滤、排序、scoped memory 优先和 disk fallback 语义。本节点不拥有 create transaction、approval review、approval persistence、sandbox trigger、status transition、AppState、schema owner、frontend caller、route facade 或 release transition guard。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| list query | `backend.runtime.routes.mutation` | `RuntimeAiProposalListQuery` | `source_kind`、`source_id`、`status` 均为可选过滤条件 |
| detail request | route caller | `auth::UserId` + proposal id | 必须先按 `auth::scoped_key` 查 memory，再 fallback disk |
| loader request | parent approval/status flow | `AppState`、`auth::UserId`、proposal id | 不改变 scoped visibility 或 error mapping |
| proposal store dir | `AppState.ai_proposal_store_dir` | filesystem path | 只调用 persistence helper，不迁移 runtime persistence owner |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| proposal list | frontend / tests | `Vec<RuntimeAiProposalRecord>` | response shape 不变，不新增 pagination |
| proposal detail | frontend / tests | `RuntimeAiProposalRecord` | memory-first lookup 与 disk fallback 不变 |
| loader record | approval/status/sandbox parent flow | `RuntimeAiProposalRecord` | 不改变 not found 或 IO error mapping |

**关键 public 方法与受控 helper**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_runtime_ai_proposals` | `RuntimeAiProposalListQuery` | proposal list JSON | `backend.runtime.routes.mutation` | 不得改变 filtering、sorting 或 response shape |
| `get_runtime_ai_proposal_detail` | user id、proposal id | proposal detail JSON | `backend.runtime.routes.mutation` | 不得绕过 scoped memory lookup 或 disk fallback |
| `load_runtime_ai_proposal_for_user` | `AppState`、user id、proposal id | proposal record | parent approval/status flow | 不得开放给 sibling 或改变 `auth::scoped_key` |
| `clean_optional_filter` | optional string | trimmed optional filter | list handler | 不得改变 empty string drop 语义 |
| `list_runtime_ai_proposal_records` | store dir | persisted records | record_query via persistence owner | 不得迁移 persistence owner |
| `load_runtime_ai_proposal_record` | store dir、proposal id | persisted record | record_query via persistence owner | 不得迁移 persistence owner |

**父级通信规则**:
`runtime.mutation.ai_proposal.record_query` 后续如物理抽离，只能被父级 `runtime.mutation.ai_proposal` 调用并由父级维持 route-facing handler 出口；不得横向连接 approval_review、approval_persistence、sandbox_trigger、status_transition、parameter_mutation、AppState、schema owner、frontend caller 或 runtime persistence owner。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**回归保护**:
本基线批次只跑治理门禁：`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。后续实际抽离必须补跑 `cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`。

**细分价值判断**:
BE-001BF-04 已完成单叶 closeout，并设置 `stop_split: true`。list/detail/loader 共同构成 AI proposal record read model；继续拆成 list/detail/loader 微文件不会形成新的稳定 owner。父叶 BE-001BG-01 已完成残余判断，后续只能进入 `approval_review`。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.record_query` 已推进至 BE-001BF-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，后续 approval_review 已另行进入 BE-001BH 流程。不得宣称 AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

### 5.1.1.3.5 `runtime.mutation.ai_proposal.approval_review`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.approval_review`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BH-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/approval_review.rs` 承接 approval list/detail/approve/reject/claim 五个 handler，并设置 `stop_split: true`。继续拆 query/action 或 approve/reject/claim 微叶不会形成新的稳定 owner，只会增加父子接线面。下一步只能进入 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/204-runtime.mutation.ai_proposal第四轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/206-runtime.mutation.ai_proposal.approval_review抽离方案.md`
- `markdown/06-milestones/v4.16.0/207-runtime.mutation.ai_proposal.approval_review抽离记录.md`
- `markdown/06-milestones/v4.16.0/208-runtime.mutation.ai_proposal.approval_review单叶closeout.md`

**目标文件落地**:
approval_review child file 已在 BE-001BH-03 实际抽离中创建。父级 `src/runtime/mutation/ai_proposal.rs` 通过 `#[path = "ai_proposal/approval_review.rs"] mod approval_review;` 和 `pub(crate) use approval_review::{...};` 维持原 route-facing 调用面。

**职责**:
承载 AI proposal approval review handler 域的白箱基线，冻结 approval 查询、审批通过、审批拒绝、审批认领、reviewer lifecycle、quorum、review_state 和 proposal status side effect。本节点不拥有 approval persistence、sandbox trigger、status transition、AppState、schema owner、frontend caller、route facade 或 release transition guard。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_runtime_approvals` | user id、approval list query | approval list JSON | `backend.runtime.routes.mutation` | 不得改变 scoped prefix、review_state filtering、sorting 或 response shape |
| `get_runtime_approval_detail` | user id、approval id | approval detail JSON | `backend.runtime.routes.mutation` | 不得改变 memory-first lookup 或 disk fallback |
| `approve_ai_proposal` | proposal id、approval action | approval record JSON | `backend.runtime.routes.mutation` | 不得改变 sandbox gate、quorum、lifecycle、status side effect 或 `approval_records -> ai_proposals` 锁顺序 |
| `reject_ai_proposal` | proposal id、approval action | approval record JSON | `backend.runtime.routes.mutation` | 不得改变 pending/under_review guard、comment fallback、lifecycle 或 Denied side effect |
| `claim_ai_proposal_review` | proposal id、approval action | approval record JSON | `backend.runtime.routes.mutation` | 不得改变 pending-only guard、assigned 去重或 claim lifecycle |

**父级通信规则**:
`runtime.mutation.ai_proposal.approval_review` 只能被父级 `runtime.mutation.ai_proposal` 连接，并由父级维持 route-facing handler 出口。approval review 经父级受控调用 `load_runtime_ai_proposal_for_user`、`ensure_ai_proposal_can_be_approved`、`ai_proposal_approved_status`、`update_ai_proposal_status`、`persist_approval` 和 `load_approval_from_disk`；不得横向接管 approval_persistence、sandbox_trigger、status_transition、record_query、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

**回归保护**:
本实际抽离批次必须跑：`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**细分价值判断**:
BE-001BH-04 已完成单叶 closeout，并设置 `stop_split: true`。approval query/action 五个 handler 形成同一 route-facing owner；继续拆 query/action 或 approve/reject/claim 微叶不会形成新的独立状态、独立锁、独立 schema 或独立验证证据。approval persistence、sandbox trigger 和 status transition 仍应作为父叶后续残余单独判断。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.approval_review` 已推进至 BE-001BH-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 approval persistence、sandbox trigger、status transition、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition 已改变。

### 5.1.1.3.6 `runtime.mutation.ai_proposal.approval_persistence`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.approval_persistence`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BJ-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/approval_persistence.rs` 承接 `persist_approval` 与 `load_approval_from_disk`，设置 `stop_split: true`。继续拆 read/write 微叶不会形成新的稳定 owner。父叶已进入 BE-001BK-01 残余判断，下一步只能进入 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/approval_persistence.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime_persistence.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/209-runtime.mutation.ai_proposal第五轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md`
- `markdown/06-milestones/v4.16.0/212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md`
- `markdown/06-milestones/v4.16.0/213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md`

**职责**:
冻结 approval record persistence 白箱边界: `RuntimeApprovalRecord` 写入、磁盘 fallback 读取、`approval_store_dir`、`FsPath`、`fs::create_dir_all`、`atomic_write_json`、`fs::read`、`serde_json::from_slice`、`json_bad_request` / `not_found` 与 `internal_error(anyhow::anyhow)` 错误映射。本节点不拥有 approval review handler、sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

**关键 public/helper 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `persist_approval` | `&FsPath`、`&RuntimeApprovalRecord` | `std::io::Result<()>` | `create_runtime_ai_proposal`、`approval_review`、sandbox background task | 不得绕过 `fs::create_dir_all` 或 `atomic_write_json` |
| `load_approval_from_disk` | `&FsPath`、`approval_id: &str` | `Result<RuntimeApprovalRecord, (StatusCode, String)>` | `approval_review` detail fallback | 不得改变 `not_found`、`serde_json::from_slice` 或 `internal_error` 语义 |

**父级通信规则**:
`runtime.mutation.ai_proposal.approval_persistence` 后续只能由父级 `runtime.mutation.ai_proposal` 连接。`approval_review` 必须继续经父级 `use super::*` 受控调用 `persist_approval` 与 `load_approval_from_disk`，不得横向 import sibling，也不得接管 sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

**回归保护**:
本基线批次只跑治理门禁: `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。后续实际抽离必须补跑 `cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`。

**抽离方案**:
BE-001BJ-03 只允许迁移 `persist_approval` 与 `load_approval_from_disk` 两个 helper。父级只允许新增 path-attributed child、`mod approval_persistence` 和私有 helper import；child 固定 `use super::*`，函数固定 `pub(super) async fn`。不允许改 route facade、`src/runtime/mod.rs` re-export、AppState、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

**抽离结果**:
BE-001BJ-03 已完成实际抽离。目标 child 已创建，父级只保留 `#[path = "ai_proposal/approval_persistence.rs"] mod approval_persistence;` 与 `use approval_persistence::{load_approval_from_disk, persist_approval};`。`approval_review` 仍经父级 `use super::*` 访问 helper，没有横向 import sibling。

**细分价值判断**:
BE-001BJ-04 已完成单叶 closeout，设置 `stop_split: true`。`persist_approval` 与 `load_approval_from_disk` 围绕同一 approval record store path、JSON 文件命名和错误映射形成低副作用 persistence owner；继续拆成 read/write 微叶不会形成新的稳定状态、锁、schema、route facade 或 runtime persistence owner。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.approval_persistence` 已推进至 BE-001BJ-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 sandbox_trigger/status_transition 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构完成。

### 5.1.1.3.7 `runtime.mutation.ai_proposal.sandbox_trigger`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.sandbox_trigger`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BL-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 已承接 approve 前 sandbox gate、sandbox report fallback、create path background task、retry / panic guard、`sandbox_report_url` 回写、失败 lifecycle 和 approval persistence 副作用，并设置 `stop_split: true`。下一步只能进入 BE-001BM-01 `runtime.mutation.ai_proposal` 第七轮父叶残余判断。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/sandbox_trigger.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/mutation/ai_proposal/approval_persistence.rs`
- `src/runtime_persistence.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/214-runtime.mutation.ai_proposal第六轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md`
- `markdown/06-milestones/v4.16.0/217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md`
- `markdown/06-milestones/v4.16.0/218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md`

**职责**:
冻结 AI proposal sandbox trigger 白箱边界: `load_sandbox_report_for_proposal` 的 memory-first / disk fallback、`ensure_ai_proposal_can_be_approved` 的 config binding / static check / sandbox required / sandbox verdict gate、`create_runtime_ai_proposal` 内部 background sandbox verification task 的 `RequestSandboxVerificationRequest`、`run_sandbox_verification`、`JoinHandle` monitoring、`catch_unwind`、retry、`sandbox_report_url` 回写和 `RuntimeApprovalLifecycleEntry` failure side effect。本节点不拥有 `status_transition`、proposal create orchestration、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

**关键 public/helper 方法**:
| 方法 / 入口 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `load_sandbox_report_for_proposal` | `&AppState`、proposal id | `Result<SandboxVerificationReport, (StatusCode, String)>` | `ensure_ai_proposal_can_be_approved` | 不得跳过 `state.sandbox_reports` memory-first lookup 或 `sandbox_report_store_dir` fallback |
| `ensure_ai_proposal_can_be_approved` | `&AppState`、`&RuntimeAiProposalRecord` | `Result<(), (StatusCode, String)>` | `approval_review` via parent `use super::*` | 不得改变 `strategy_config_ai_binding_required`、`ai_proposal_static_check_required`、`ai_proposal_sandbox_required` 或 `ai_proposal_sandbox_failed` gate |
| `spawn_ai_proposal_sandbox_verification` | `AppState` clone、proposal id、approval record | approval side effect | `create_runtime_ai_proposal` | 不得改变 `RequestSandboxVerificationRequest`、`run_sandbox_verification`、3 次 retry、`catch_unwind`、`sandbox_report_url` 或 failed lifecycle |

**父级通信规则**:
`runtime.mutation.ai_proposal.sandbox_trigger` 只能由父级 `runtime.mutation.ai_proposal` 连接。`approval_review` 必须继续经父级 `use super::*` 受控调用 `ensure_ai_proposal_can_be_approved`，不得横向 import sibling。sandbox task 只能经父级受控调用 `persist_approval`，不得横向 import `approval_persistence` sibling。ASCII guard: `release transition guard`。

**回归保护**:
BE-001BL-03 实际抽离批次已跑: `cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。BE-001BL-04 closeout 为 `no code movement`，提交前跑治理门禁。

**抽离方案**:
BE-001BL-03 只允许创建 sandbox_trigger 目标文件，迁移 `load_sandbox_report_for_proposal` 与 `ensure_ai_proposal_can_be_approved`，并将 create flow 内联 background task 提取为 `spawn_ai_proposal_sandbox_verification`。父级只允许新增 `mod sandbox_trigger` 和受控 helper import；`FutureExt` 随 `catch_unwind` 使用点迁入 child。父级 `create_runtime_ai_proposal` 只允许以 `spawn_ai_proposal_sandbox_verification(state.clone(), proposal_id.clone())` 替代原内联 task，不得迁移 status_transition 或整个 create orchestration。

**抽离结果**:
BE-001BL-03 已完成实际抽离。目标 child 已创建，父级只保留 `#[path = "ai_proposal/sandbox_trigger.rs"] mod sandbox_trigger;`、`use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification};` 与 create flow helper 调用。child 固定 `use super::*` 和 `use futures_util::FutureExt;`；`load_sandbox_report_for_proposal` 保持 child private，两个父级入口保持 `pub(super)`。

**closeout 结果**:
BE-001BL-04 已完成单叶 closeout，设置 `stop_split: true`。sandbox report fallback、approve gate、background verification task、retry / panic guard、report URL 回写与 failed lifecycle 共同形成 external sandbox evidence owner；继续拆成 report_loader / approval_gate / background_task 微叶不会形成新的稳定状态、锁、schema、route facade 或 runtime persistence owner。

**细分价值判断**:
BE-001BL-04 已完成单叶 closeout，`stop_split: true`。本叶不再继续细拆；后续必须回到父叶 BE-001BM-01 残余判断，评估 `status_transition` 与 proposal create orchestration。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.sandbox_trigger` 已推进至 BE-001BL-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 status_transition、proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition 已改变。

### 5.1.1.3.8 `runtime.mutation.ai_proposal.status_transition`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.status_transition`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BN-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/status_transition.rs` 承接 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status`、状态机迁移矩阵、`state.ai_proposals` 写入副作用、非法转换日志和父子通信规则，并设置 `stop_split: true`。下一步只能进入 BE-001BO-01 `runtime.mutation.ai_proposal` 第八轮父叶残余判断。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/mutation/ai_proposal/status_transition.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/219-runtime.mutation.ai_proposal第七轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/221-runtime.mutation.ai_proposal.status_transition抽离方案.md`
- `markdown/06-milestones/v4.16.0/222-runtime.mutation.ai_proposal.status_transition抽离记录.md`
- `markdown/06-milestones/v4.16.0/223-runtime.mutation.ai_proposal.status_transition单叶closeout.md`

**职责**:
冻结 AI proposal status transition 白箱边界: `ai_proposal_approved_status` 的 Approved projection、`is_valid_ai_proposal_transition` 的 Submitted / StaticCheckPassed 迁移矩阵、`update_ai_proposal_status` 的 `state.ai_proposals` 写锁、`auth::scoped_key` lookup、非法转换 `safe_eprintln!` 和 `updated_at_ms` side effect。本节点不拥有 proposal create orchestration、approval review handler、sandbox_trigger、approval_persistence、record_query、event_lifecycle、static_check、source_governance_identity、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

**输入输出基线**:
| helper | 输入 | 输出 / 副作用 | 约束 |
| --- | --- | --- | --- |
| `ai_proposal_approved_status` | 无 | `RuntimeAiProposalStatus::Approved` | 不得改回 `StaticCheckPassed` |
| `is_valid_ai_proposal_transition` | current status、next status | `bool` | 保持 `(Submitted, StaticCheckPassed | StaticCheckFailed)` 与 `(StaticCheckPassed, Approved | Denied | Expired)` |
| `update_ai_proposal_status` | `&AppState`、`&auth::UserId`、proposal id、next status | 更新 scoped proposal record | 保持 no-op missing record、非法转换阻断、`updated_at_ms = current_time_ms()` |

**关键 public/helper 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `ai_proposal_approved_status` | 无 | `RuntimeAiProposalStatus::Approved` | `approval_review::approve_ai_proposal` via parent `use super::*` | 不得开放为 route handler |
| `is_valid_ai_proposal_transition` | current status、next status | `bool` | `update_ai_proposal_status` | 后续抽离时默认 child private，不得横向调用 |
| `update_ai_proposal_status` | `AppState`、user id、proposal id、next status | proposal status side effect | `approval_review::approve_ai_proposal` / `reject_ai_proposal` via parent `use super::*` | 不得改变 scoped key、合法迁移矩阵或 missing no-op |

**父级通信规则**:
`runtime.mutation.ai_proposal.status_transition` 只能由父级 `runtime.mutation.ai_proposal` 连接。`approval_review` 继续经父级 `use super::*` 受控调用 `ai_proposal_approved_status` 与 `update_ai_proposal_status`，不得横向 import sibling。status_transition 不得横向 import `approval_review`、`sandbox_trigger`、`approval_persistence`、`record_query`、`event_lifecycle`、`static_check` 或 `source_governance_identity` sibling。ASCII guard: `release transition guard`。

**回归保护**:
本实际抽离批次必须跑: `cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**抽离方案**:
BE-001BN-03 只允许创建 status_transition 目标文件，迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`。父级只允许新增 `mod status_transition` 和受控 helper import；`is_valid_ai_proposal_transition` 保持 child private。父级与 `approval_review` 调用面只允许经 `use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};` 与 `use super::*` 维持，不得迁移 proposal create orchestration。

**抽离结果**:
BE-001BN-03 已创建 `src/runtime/mutation/ai_proposal/status_transition.rs`，并迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`。父级 `src/runtime/mutation/ai_proposal.rs` 只保留 `#[path = "ai_proposal/status_transition.rs"] mod status_transition;` 与 `use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};`；`is_valid_ai_proposal_transition` 保持 child private。proposal create orchestration 未迁移。

**细分价值判断**:
BE-001BN-04 已完成单叶 closeout，`stop_split: true`。approved projection、transition guard 与 scoped status side effect 共同构成同一 AI proposal status machine helper owner；继续拆成 approved_projection / transition_guard / status_writer 微叶不会形成新的稳定状态 owner、锁 owner、schema owner、route facade 或 runtime persistence owner，只会增加父子接线与治理挂载面。下一步只能进入 BE-001BO-01 父叶残余判断，不得迁移 proposal create orchestration。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.status_transition` 已推进至 BE-001BN-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition 已改变。

### 5.1.1.3.9 `runtime.mutation.ai_proposal.proposal_creation`

**层级路径**: `root.backend.runtime.mutation.ai_proposal.proposal_creation`
**父模块**: `runtime.mutation.ai_proposal`
**状态**: v4.16 BE-001BP-04 单叶 closeout 已完成；`src/runtime/mutation/ai_proposal/proposal_creation.rs` 承接 `create_runtime_ai_proposal`，并设置 `stop_split: true`。父级 `src/runtime/mutation/ai_proposal.rs` 通过 path-attributed child 与 handler re-export 维持 route-facing 调用面。下一步只能进入 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断，不得迁移其它 child、改变 AppState/schema/frontend caller 或启动 release transition guard。

**真实文件**:
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
- `src/runtime/mutation/ai_proposal/event_lifecycle.rs`
- `src/runtime/mutation/ai_proposal/approval_persistence.rs`
- `src/runtime/mutation/ai_proposal/sandbox_trigger.rs`
- `src/runtime/mutation/ai_proposal/status_transition.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/frontend_api_types.rs`
- `src/runtime_persistence.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/224-runtime.mutation.ai_proposal第八轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md`
- `markdown/06-milestones/v4.16.0/227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md`
- `markdown/06-milestones/v4.16.0/228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md`

**职责**:
冻结 AI proposal create orchestration 白箱边界: `CreateRuntimeAiProposalRequest` 输入校验、`validate_runtime_capability_guard`、`proposal_only` policy、`validate_runtime_parameter_mutation_target`、`validate_ai_model_identity`、`validate_hash_identity`、`normalize_actor_identity`、source context 读取、parameter version canonicalization、static check、proposal id/governance、`RuntimeAiProposalSourceEvidence`、`RuntimeAiProposalRecord`、event/lifecycle append、StaticCheckPassed / StaticCheckFailed 分支、`RuntimeApprovalRecord`、`RuntimeApprovalLifecycleEntry`、`APPROVAL_CREATED`、`persist_approval`、`approval_records -> ai_proposals` 锁顺序、`persist_runtime_ai_proposal_transition` 与 `spawn_ai_proposal_sandbox_verification`。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| create request | `backend.runtime.routes.mutation` | `CreateRuntimeAiProposalRequest` | 不改变 source_kind/source_id、target、old_value/new_value、model、prompt_hash、evidence_hash、reason、config_domain_binding |
| user id | auth middleware | `auth::UserId` | 仅用于 scoped key、actor 绑定和 proposal/approval visibility |
| state | `AppState` | shared app state | 不迁移 AppState owner、schema owner、frontend caller、route facade 或 runtime persistence owner |
| source context | run/backtest owner | existing runtime/backtest evidence | 只能经 `load_runtime_ai_proposal_source_context` 读取 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| proposal record | frontend/tests | `RuntimeAiProposalRecord` | response shape、status、governance、source_evidence、lifecycle 不变 |
| approval record side effect | approval owner | `RuntimeApprovalRecord` | 仅 StaticCheckPassed 分支自动创建，`APPROVAL_CREATED` lifecycle 不变 |
| proposal transition | persistence/state | `StaticCheckPassed` / `StaticCheckFailed` | `persist_runtime_ai_proposal_transition` 与 transition event 不变 |
| sandbox task | sandbox verification owner | background side effect | 只经 `spawn_ai_proposal_sandbox_verification` 串联 |

**关键 public/helper 方法**:
| 方法 | 输入 | 输出 / 副作用 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_ai_proposal` | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeAiProposalRequest>` | `Json<RuntimeAiProposalRecord>` 与 proposal/approval side effect | `backend.runtime.routes.mutation` | 不得改变 route-facing signature、response shape、锁顺序或状态分支 |
| `ai_proposal_static_check_result` | request、parameter versions、source event count | static check result | create handler via parent | 不得改变 StaticCheckPassed / StaticCheckFailed |
| `runtime_ai_proposal_record_id` | request、created timestamp、versions、static check | proposal id | create handler via parent | 不得改变 digest contract |
| `runtime_ai_proposal_governance` | source context、model、hash evidence | governance metadata | create handler via parent | 不得丢失 governance evidence |
| `build_runtime_ai_proposal_event` / `ai_proposal_lifecycle_entry` | proposal context | runtime event / lifecycle entry | create handler via parent | 不得改变 event contract |
| `append_parameter_mutation_events_to_run` | run id、events | run evidence side effect | create handler | 不得绕过 run evidence append |
| `persist_approval` | approval store dir、record | disk side effect | create handler via parent | 不得绕过 approval persistence |
| `persist_runtime_ai_proposal_transition` | proposal/status/event | transition persistence | create handler via parent | 不得跳过 transition persistence |
| `spawn_ai_proposal_sandbox_verification` | state clone、proposal id、approval record | sandbox background side effect | create handler via parent | 不得改变 retry / failure lifecycle |

**父级通信规则**:
`runtime.mutation.ai_proposal.proposal_creation` 已物理抽离，只能由父级 `runtime.mutation.ai_proposal` 连接；child 固定 `use super::*`，不得横向 import `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition` sibling。父级继续拥有 route-facing re-export，`src/runtime/mod.rs` 与 `src/backend/runtime/routes/mutation.rs` 调用面不变。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

**抽离方案**:
BE-001BP-03 已创建 proposal_creation child，并只迁移 `create_runtime_ai_proposal`。父级只新增 path-attributed child 声明和 `pub(crate) use proposal_creation::create_runtime_ai_proposal;` handler re-export；child 固定 `use super::*` 复用父级已受控 helper。不得迁移 `list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`、approval review handler、approval persistence helper、status transition helper、sandbox trigger helper、AppState、schema owner、frontend caller、route facade 或 runtime persistence owner。

**抽离结果**:
BE-001BP-03 已创建 `src/runtime/mutation/ai_proposal/proposal_creation.rs`，并迁移 `create_runtime_ai_proposal`。父级通过 `#[path = "ai_proposal/proposal_creation.rs"] mod proposal_creation;` 与 `pub(crate) use proposal_creation::create_runtime_ai_proposal;` 维持 route-facing 调用面；child 固定 `use super::*`。`record_query`、`approval_review`、`approval_persistence`、`status_transition` 与 `sandbox_trigger` 均未迁移或横向连接。

**closeout 结果**:
BE-001BP-04 已完成单叶 closeout，并设置 `stop_split: true`。本叶只承接 `create_runtime_ai_proposal` create transaction；继续拆 approval record construction、lifecycle append、transition persistence 或 sandbox trigger call 不会形成稳定 owner，反而会扩大父子接线面和锁顺序风险。下一步只能进入 BE-001BQ-01 父叶残余判断。

**回归保护**:
BE-001BP-04 为 `no code movement` closeout，提交前只跑治理门禁: `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**细分价值判断**:
BE-001BP-04 已完成单叶 closeout，并设置 `stop_split: true`。当前只能进入 BE-001BQ-01 父叶残余判断；不得继续细拆本叶、迁移 record_query、approval_review、approval_persistence、status_transition、sandbox_trigger、改变 `AppState` / schema owner / frontend caller 或启动 release transition guard。

**幻觉检查点**:
AI 声称 `runtime.mutation.ai_proposal.proposal_creation` 已推进至 BE-001BP-04 时，必须说明本叶已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成残余判断。不得宣称 AppState/schema/frontend caller 已改变、route facade 已改变、runtime persistence owner 已迁移、release transition 已启动或 Rust backend 重构已完成。

### 5.1.2 `backend.runtime.routes.run`

**层级路径**: `root.backend.runtime.routes.run`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001H-03 `runtime.run.v4_handoff` 已完成单叶 closeout 并停止内部细分；BE-001I-03 `runtime.run.session_start` 已完成单叶 closeout 并停止内部细分；BE-001J-05 `runtime.run.record_store` 已完成抽离与单叶 closeout 并停止内部细分；BE-001K-04 `runtime.run.replay_status` 已完成抽离与单叶 closeout 并停止内部细分。当前只拥有 run route group facade，不拥有 state owner、event stream 或 persistence owner；route facade 本身停止细分，handler 层已完成当前四个 run handler sibling 的递归收口。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run/record_store.rs`
- `src/runtime/run/replay_status.rs`
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
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md`
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md`
- `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md`
- `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md`
- `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md`

**职责**:
承载 run/v4 run/list/detail/save/replay/status route group facade，固定 `backend.runtime.routes -> backend.runtime.routes.run -> src/runtime/* pub(crate) handler` 的兼容桥和等价证据。

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
`src/runtime/run/replay_status.rs` 中的 replay/status targets、`src/runtime/event_stream.rs` 中的 SSE route target、`src/runtime/run/v4_handoff.rs` 中的 v4 handoff target、`src/runtime/run/session_start.rs` 中的 legacy session start target、`src/runtime/run/record_store.rs` 中的 run record target，以及既有 persistence / event projection helper 调用边界。state owner 继续保留在 `AppState`。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_sse`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
`backend.runtime.routes.run` 这个 route facade 不继续细分；真实 handler owner 已从 `runtime.run.v4_handoff`、`runtime.run.session_start`、`runtime.run.record_store` 和 `runtime.run.replay_status` 完成四片 closeout。后续不得继续细拆这些已 closeout 叶子；`runtime.event_stream` 仍是父级 route 子叶候选，不属于本 facade。

**幻觉检查点**:
AI 声称 runtime run routes 已迁移时，必须说明 run route group facade、`runtime.run.v4_handoff` handler 子模块、`runtime.run.session_start` handler 子模块与 `runtime.run.record_store` handler 子模块是不同动作；不得宣称 src/runtime/run.rs (retired drained include) 全部 handler、state owner、event stream 或 persistence 已迁移。AI 声称本子叶完成时，还必须说明 route facade 停止细分不等于 run handler 全部完成。

### 5.1.3 `runtime.run.v4_handoff`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.v4_handoff`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001H-03 单叶 closeout 已完成，当前停止内部细分。`/api/runtime/v4/run` handler、request/response type、graph resolution、initial event、handoff projection 与 simulated capability matrix 已迁入 `src/runtime/run/v4_handoff.rs`；父级 `runtime` 保留受控 re-export。
**真实文件**:
- `src/runtime/run/v4_handoff.rs`
- `src/runtime/mod.rs`
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
仅允许使用既有 `qrpc_core_ir`、`qrpc_runtime`、`quantscript` static audit / handoff / v4 paper simulated runtime；`runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` 对 src/runtime/backtest.rs (retired drained include) 的复用只能经父级 `runtime` 受控出口，不得形成 sibling 直连。

**细分价值判断**:
本叶不继续细拆。request/response schema、source/graph resolution、initial event、handoff projection 都服务同一条 v4 handoff route；simulated capability matrix 若未来独立，应另起父级共享节点，不能在本叶内部横向拆出。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 `runtime.run.v4_handoff` 已抽离时，必须指出只完成 v4 handoff handler 子模块抽离；legacy run/session、record_store、replay_status 和 SSE 属于不同 sibling。不得宣称 provider 真连接、record store、SSE、persistence 或发布版本过渡已完成。

### 5.1.4 `runtime.run.session_start`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.session_start`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001I-03 单叶 closeout 已完成，当前停止内部细分。legacy `/api/runtime/test-run` 的 `start_test_run` 已迁入 `src/runtime/run/session_start.rs`，父级 `runtime` 保留受控 re-export；record/replay/status、SSE、state owner 和 persistence 仍不迁移。
**真实文件**:
- `src/runtime/run/session_start.rs`
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
**状态**: v4.16 BE-001J-05 抽离与单叶 closeout 已完成，当前不继续细拆。`list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` 已迁入 `src/runtime/run/record_store.rs`；persistence、audit、response mapping、frontend route 和 AppState owner 均保留原位。
**真实文件**:
- `src/runtime/run/record_store.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/collaboration.rs`
- `markdown/06-milestones/v4.16.0/61-runtime.run.record_store单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md`
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md`

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

**真实 route 边界**:
| route | handler | 说明 |
| --- | --- | --- |
| `GET /api/runtime/runs` | `list_runs` | list saved manifests，分页后返回 `RunListItem` |
| `GET /api/runtime/runs/:run_id` | `get_run_detail` | current runtime 优先，manifest fallback |
| `POST /api/runtime/runs/:run_id/save` | `save_run_record` | 保存 run manifest，actor 存在时写 audit |
| `DELETE /api/runtime/runs/:run_id` | `discard_run_record` | 只删除 transient in-memory record；没有 `/discard` 后缀 |

**抽离记录结论**:
四个 handler 已迁入 `src/runtime/run/record_store.rs`，再由 `src/runtime/mod.rs` 通过私有子模块和 `pub(crate) use` 保持 `crate::runtime::*` 兼容出口。`src/backend/runtime/routes/run.rs` 不改 route，`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs` 和 AppState owner 不迁移。

**父级通信规则**:
`runtime.run.record_store` 只能经父级 `runtime` 和 `backend.runtime.routes.run` 暴露 run record routes；不得横向直接接管 `runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest、mutation、executor 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 `runtime_persistence`、`runtime_response_mapping`、`collaboration` audit helper 和 AppState 字段。`state.runs`、`run_store_dir`、`audit_store_dir` 和 persistence owner 继续保留原位，本基线不新建 storage/security owner。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已完成单叶 closeout，当前不继续细拆。list/detail/save/discard 已形成可维护 handler 叶子；persistence、audit、response projection 和 path sanitize 仍是共享 helper owner，不在本叶内私有化。`runtime.run.replay_status` 后续已完成抽离与 closeout，当前默认回到父级 `runtime.event_stream` 候选。

**幻觉检查点**:
AI 声称 `runtime.run.record_store` 已完成时，必须说明只完成四个 record store handler 子模块抽离与单叶 closeout；discard 真实 route 是 `DELETE /api/runtime/runs/:run_id`；replay/status、SSE、state owner、shared helper owner、persistence owner、frontend route 和发布版本过渡均未完成。不得宣称 runtime run handler 全部完成。

### 5.1.6 `runtime.run.replay_status`

**层级路径**: `root.backend.runtime.routes.run.runtime.run.replay_status`
**父模块**: `backend.runtime.routes.run`
**状态**: v4.16 BE-001K-04 抽离与单叶 closeout 已完成，当前不继续细拆。`get_run_replay` 与 `get_run_status` 已迁入 `src/runtime/run/replay_status.rs`；`runtime.event_stream`、response mapping、schema、metrics、state owner 和 persistence owner 均保留原位。
**真实文件**:
- `src/runtime/run/replay_status.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/run.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `src/lib.rs`
- `tests/api_run.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md`
- `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md`
- `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md`

**职责**:
承载 run replay/status handler 子模块的等价基线，固定 replay window、cursor/filter、status projection、replay metrics 和 SSE 排除边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_run_replay` | `auth::UserId`、`AppState`、`run_id`、`RuntimeReplayQuery` | `Json<RuntimeReplayResponse>` | `GET /api/runtime/runs/:run_id/replay` | 不得迁移 SSE、response mapping owner、schema owner 或 state owner |
| `get_run_status` | `auth::UserId`、`AppState`、`run_id` | `Json<RunStatusResponse>` | `GET /api/runtime/runs/:run_id/status` | 不得接管 record store、session start、v4 handoff 或 persistence |
| `normalized_replay_options` | `RuntimeReplayQuery` | `RuntimeReplayOptions` | `get_run_replay`、backtest replay | 不得私有化到 run replay leaf |
| `run_replay_response_from_record` | `RunRecord`、`RuntimeReplayOptions` | `RuntimeReplayResponse` | `get_run_replay` | 不得从 `runtime_response_mapping` 迁出 |
| `run_status_response_from_record` | `RunRecord` | `RunStatusResponse` | `get_run_status` | 不得从 `runtime_response_mapping` 迁出 |

**允许调用的子模块**:
仅允许使用既有 `load_run_record_from_state`、`normalized_replay_options`、`run_replay_response_from_record`、`run_status_response_from_record`、`json_bad_request` 和 `state.evidence_metrics.record_replay_page`。`RuntimeReplayQuery`、`RuntimeReplayResponse`、`RunStatusResponse`、AppState owner 和 metrics owner 保留原位。

**父级通信规则**:
`runtime.run.replay_status` 只能经父级 `runtime` 和 `backend.runtime.routes.run` 暴露 replay/status routes；不得横向直接接管 `runtime.event_stream`、`runtime.run.record_store`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest replay、mutation、executor 或 frontend state。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已完成单叶 closeout，当前不继续细拆。`get_run_replay` 和 `get_run_status` 已形成可维护 handler 叶子；query options、response mapping、schema、metrics、record lookup、state 和 persistence 仍是共享 helper owner，不在本叶内私有化。下一步应回到父级 `backend.runtime.routes` sibling 队列，默认先为 `runtime.event_stream` 建立等价基线。

**幻觉检查点**:
AI 声称 `runtime.run.replay_status` 已完成时，必须说明只完成 replay/status 两个 handler 子模块抽离与单叶 closeout；`stream_run_events`、response mapping、schema、metrics、state owner、persistence owner、frontend route 和发布版本过渡均未完成。不得宣称 runtime run handler 全部完成，也不得把 `runtime.event_stream` 说成本叶的一部分。

### 5.1.7 `runtime.event_stream`

**层级路径**: `root.backend.runtime.routes.runtime.event_stream`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001L-04 抽离与单叶 closeout 已完成，当前不继续细拆。`stream_run_events` 已迁入 `src/runtime/event_stream.rs`；`/api/runtime/runs/:run_id/events` 仍由 `src/backend/runtime/routes.rs` 父级 aggregate 直接注册；run replay/status、record store、backtest、mutation、report、state owner、persistence owner 和 frontend caller 均保留原位。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/runtime/event_stream.rs`
- `src/runtime/mod.rs`
- `tests/api_sse.rs`
- `markdown/06-milestones/v4.16.0/70-runtime.event_stream单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/71-runtime.event_stream抽离方案.md`
- `markdown/06-milestones/v4.16.0/72-runtime.event_stream抽离记录.md`
- `markdown/06-milestones/v4.16.0/73-runtime.event_stream单叶closeout.md`

**职责**:
承载 run event stream SSE route 的等价基线，固定 `run_started`、`runtime_event`、`account`、`run_completed` frame order、keep-alive、record lookup 和父级 route owner。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `UserId` | auth middleware | scoped user id | 只用于 scoped run lookup，不迁移 auth owner |
| `AppState` | `backend.app_state_wiring` | shared app state | 不迁移 AppState owner 或锁顺序 |
| `run_id` | path param | string | lookup 语义必须与 detail/replay/status 一致 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `run_started` | frontend SSE panel、tests | SSE event + JSON data | 不改 `run_id`、`graph_id`、`compile_id`、`status` |
| `runtime_event` | frontend SSE panel、tests | SSE event + runtime event JSON | 不改 event order 或 envelope |
| `account` | frontend SSE panel、tests | SSE event + account JSON | 不改 account payload |
| `run_completed` | frontend SSE panel、tests | SSE event + JSON data | 不改 `event_count` 语义 |
| keep-alive | frontend SSE client | SSE keepalive | 5 秒 interval 和 `keepalive` 文本保持不变 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `stream_run_events` | `UserId`、`AppState`、`run_id` | Axum `Sse` stream | `GET /api/runtime/runs/:run_id/events` | 不得混入 replay/status、record store、mutation 或 frontend state |
| `load_run_record_from_state` | `AppState`、`UserId`、`run_id` | `RunRecord` | `stream_run_events` | 不得改变 current runtime 优先与 manifest fallback |
| `json_sse_event` | event name、JSON payload | Axum `Event` | `stream_run_events` | 不得改变 frame envelope 或 event name |
| `KeepAlive::new` | interval/text | SSE keep-alive | Axum SSE | 不得改变 keepalive interval 或文本 |

**父级通信规则**:
`runtime.event_stream` 只能经父级 `backend.runtime.routes` 暴露 `/api/runtime/runs/:run_id/events`；不得横向直接接管 `backend.runtime.routes.run`、`runtime.run.replay_status`、record store、mutation、backtest、executor 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 `load_run_record_from_state`、`json_sse_event`、Axum `Sse` / `Event` / `KeepAlive`、`sleep` 和 `Duration`。`state.runs`、`run_store_dir`、persistence owner、event projection owner 和 frontend caller 继续保留原位。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_sse`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已完成单叶 closeout，当前不继续细拆。`stream_run_events` 已形成可维护 SSE handler 叶子；record lookup、json event builder、delay、keep-alive、state、persistence 和 frontend caller 仍是共享 owner，不在本叶内私有化。下一步应回到父级 `backend.runtime.routes` sibling 队列，默认先为 `runtime.backtest` 建立等价基线。

**幻觉检查点**:
AI 声称 `runtime.event_stream` 已完成时，必须说明只完成 SSE handler 抽离与单叶 closeout；route facade、shared helper、state owner、persistence owner、frontend caller、backtest、mutation、report 和发布版本过渡均未完成。不得宣称 runtime route aggregate 全部完成，也不得把 `runtime.backtest` 说成本叶的一部分。

### 5.1.8 `runtime.backtest`

**层级路径**: `root.backend.runtime.routes.runtime.backtest`
**父模块**: `backend.runtime.routes`
**状态**: v4.16 BE-001M-04 route facade 抽离与单叶 closeout 已完成，route facade 本身停止细分；BE-001N-04 已将 `runtime.backtest.execution_start` 创建路径 handler/helper 迁入 `src/runtime/backtest/execution_start.rs` 并完成单叶 closeout；BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout 并设置 `stop_split: true`；BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`；BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`；BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`；BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断；BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`；BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`；BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`；BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断；BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。backtest start/list/detail/save/discard/replay/compare routes 已由 `src/backend/runtime/routes/backtest.rs` 注册并经 `src/backend/runtime/routes.rs` 父级 aggregate 接入；record store handler 已迁入 `src/runtime/backtest/record_store.rs`，replay handler 已迁入 `src/runtime/backtest/replay.rs`，experiment、artifact schema、compare owner、state owner、persistence owner 和 frontend caller 均保留原位。
**真实文件**:
- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/runtime/backtest/execution_start.rs`
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
- `src/backtest_compare.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `src/runtime/mod.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/74-runtime.backtest单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/75-runtime.backtest抽离方案.md`
- `markdown/06-milestones/v4.16.0/76-runtime.backtest抽离记录.md`
- `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md`
- `markdown/06-milestones/v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/79-runtime.backtest.execution_start抽离方案.md`
- `markdown/06-milestones/v4.16.0/80-runtime.backtest.execution_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md`
- `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`
- `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`
- `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`
- `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md`
- `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`
- `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md`
- `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md`
- `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/100-runtime.backtest.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/101-runtime.backtest.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/102-runtime.backtest.record_store单叶closeout.md`
- `markdown/06-milestones/v4.16.0/103-runtime.backtest.replay单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/104-runtime.backtest.replay抽离方案.md`
- `markdown/06-milestones/v4.16.0/105-runtime.backtest.replay抽离记录.md`
- `markdown/06-milestones/v4.16.0/106-runtime.backtest.replay单叶closeout.md`
- `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md`
- `markdown/06-milestones/v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md`
- `markdown/06-milestones/v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md`
- `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`
- `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md`
- `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`
- `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`
- `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md`
- `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`
- `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md`

**职责**:
承载 backtest route group 的 route facade 与等价基线，固定 backtest run/list/detail/save/discard/replay/compare、artifact views、transient spill、persistence lookup、v4 backtest evidence 和父级 route owner。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `FrontendRunRequest` | frontend、tests、local API caller | JSON request | 不改变 backtest options、runtime kind、graph/source 解析 |
| `BacktestCompareRequest` | compare route | JSON request | 必须恰好两个 `backtest_id`，仍通过 scoped lookup |
| `RuntimeReplayQuery` | replay route query | pagination/filter query | 与 run replay 共用 options，不私有化到 backtest |
| `UserId` / `AppState` | auth middleware、backend app state | scoped user / shared state | 不迁移 `backtests`、store dirs、transient dirs 或锁顺序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestRunResponse` | frontend、tests | JSON response | 保留 backtest id、output、spec、governance 与 artifact views |
| `BacktestDetailResponse` | detail panel、tests | JSON response | 保留 artifact governance、diagnostics source 和 detail schema |
| `BacktestCompareResponse` | compare panel、tests | JSON response | 保留左右 backtest id、metrics、equity/trade/assumption compare |
| `RuntimeReplayResponse` | replay panel、tests | JSON response | 保留 `kind=backtest`、record id、cursor/filter 和 event order |
| artifact bundle | filesystem、frontend artifact viewer | manifest + event log + metrics + trade ledger + equity curve | 不改 digest、governance rebuild 或 transient spill 语义 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_backtest_run` | `UserId`、`AppState`、`FrontendRunRequest` | `BacktestRunResponse` | `POST /api/runtime/backtest` | 不得混入 experiment/report/mutation 或 frontend state |
| `list_backtests` | `AppState`、pagination query | paginated backtest list | `GET /api/runtime/backtests` | 不得改变排序、分页或 saved-only 语义 |
| `get_backtest_detail` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `GET /api/runtime/backtests/:backtest_id` | 不得绕过 scoped lookup 或 artifact normalization |
| `save_backtest_record` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `POST /api/runtime/backtests/:backtest_id/save` | 不得绕过 persistence/audit/governance |
| `discard_backtest_record` | `UserId`、`AppState`、`backtest_id` | discard response | `DELETE /api/runtime/backtests/:backtest_id` | 不得删除正式保存记录 |
| `get_backtest_replay` | `UserId`、`AppState`、`backtest_id`、`RuntimeReplayQuery` | `RuntimeReplayResponse` | `GET /api/runtime/backtests/:backtest_id/replay` | 不得私有化 replay query/options/schema |
| `compare_backtests` | `UserId`、`AppState`、`BacktestCompareRequest` | `BacktestCompareResponse` | `POST /api/runtime/backtests/compare` | 不得迁移 compare core/narrative owner |

**真实 route 边界**:
| route | handler | 说明 |
| --- | --- | --- |
| `POST /api/runtime/backtest` | `start_backtest_run` | 创建 deterministic / historical / v4 backtest record 与 artifact views |
| `GET /api/runtime/backtests` | `list_backtests` | 列出已保存 backtest records |
| `POST /api/runtime/backtests/compare` | `compare_backtests` | 比较两个 backtest artifacts |
| `POST /api/runtime/backtests/:backtest_id/save` | `save_backtest_record` | 保存 transient 或 in-memory backtest record |
| `GET /api/runtime/backtests/:backtest_id` | `get_backtest_detail` | 读取 backtest detail 与 artifact views |
| `DELETE /api/runtime/backtests/:backtest_id` | `discard_backtest_record` | 删除 transient record，不删除正式保存记录 |
| `GET /api/runtime/backtests/:backtest_id/replay` | `get_backtest_replay` | 返回 backtest replay timeline |

**父级通信规则**:
`runtime.backtest` 只能经父级 `backend.runtime.routes` 与 route facade `backend.runtime.routes.backtest` 暴露 backtest routes；不得横向直接接管 `backend.runtime.routes.run`、`runtime.event_stream`、experiment/report/mutation、executor、storage security 或 frontend state。

**允许调用的子模块**:
仅允许使用既有 `src/backtest_artifacts.rs`、`src/backtest_compare.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、v4 backtest helper 和 AppState 字段。`state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、artifact schema、compare core/narrative、persistence owner 和 frontend caller 继续保留原位。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**抽离记录**:
BE-001M-03 已新建 `src/backend/runtime/routes/backtest.rs` 并迁入 backtest route registration。`src/backend/runtime/routes.rs` 只新增 backtest 子 route facade 注册，并继续保留 event stream、evidence、mutation、report、experiment、approval 和 ops routes。src/runtime/backtest.rs (retired drained include)、`src/backtest_compare.rs`、artifact、persistence、schema、state 和 frontend owner 不迁移。

**单叶 closeout**:
BE-001M-04 已确认 route facade 等价并停止 route facade 内部细分。BE-001N-04 已为 `runtime.backtest.execution_start` 完成第一轮物理抽离与单叶 closeout；BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout 并设置 `stop_split: true`；BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`；BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`；BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`；BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断；BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`；BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`；BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout 并设置 `stop_split: false`；BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`；BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成第二轮父叶残余判断；BE-001AA-01 已建立 `record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断并设置父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`。下一批若继续只能进入 BE-001AD-01 `backend.runtime.routes` 父叶残余判断，不能直接越级迁移 route facade、execution_start、persistence、mapping、schema、state 或共享 owner。

**父叶残余判断**:
BE-001AC-01 已确认 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`runtime.backtest.experiment_sweep` 均已完成当前递归范围内的 closeout，`runtime.backtest` 父叶当前设置 `stop_split: true`。src/runtime/backtest.rs (retired drained include) 仍是 drained parent include placeholder，本批不删除；`src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 和 `AppState` 均保留原 owner。下一候选固定为 BE-001AD-01 `backend.runtime.routes` 父叶残余判断。

**细分价值判断**:
route facade 本身已 `stop_split: true`，因为继续拆只会制造无意义微文件。`runtime.backtest.execution_start` 已完成第一轮物理抽离、内部四个子叶 closeout 和父叶残余判断；`runtime.backtest.record_store`、`runtime.backtest.replay` 和 `runtime.backtest.experiment_sweep` 均已完成当前递归范围内 closeout。BE-001AC-01 已将 `runtime.backtest` 父叶设置为 `stop_split: true`；不得从本父叶继续细拆或直接移动 artifact schema、compare owner、persistence owner、response mapping owner、state owner、frontend caller 或 drained parent include cleanup。

**幻觉检查点**:
AI 声称 `runtime.backtest` 已完成 BE-001AC-01 时，必须说明 `execution_start`、`record_store`、`replay` 与 `experiment_sweep` 均已完成当前递归范围内 closeout，父叶已设置 `stop_split: true`，且本批为 `no code movement`。不得宣称 `backend.runtime.routes` 上层完成、src/runtime/backtest.rs (retired drained include) drained parent include 已删除、compare/artifact schema/persistence/response mapping/frontend caller 已迁移、发布过渡已启动、整理或重构已经完成。

### 5.1.9 `runtime.backtest.execution_start`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.execution_start`
**父模块**: `runtime.backtest`
**状态**: v4.16 BE-001N-04 单叶 closeout 已完成。`start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` 和 v4 helper 已迁入 `src/runtime/backtest/execution_start.rs`；父级 `runtime` 通过 re-export 暴露 `start_backtest_run`，并通过内部桥保留 `execute_backtest_request` 给 experiment sweep 复用。本叶等价成立，但不设置 `stop_split: true`；BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout 并设置 `stop_split: true`；BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`；BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`；BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`；BE-001S-01 已完成父叶残余判断。record store、replay、experiment、artifact schema、compare owner、persistence owner、schema owner、state owner、frontend caller 和发布过渡均未迁移。
**递归状态补充**: BE-001S-01 已确认 `runtime.backtest.execution_start` 当前不再私拆 record/state/persistence 边界；下一步回到 `runtime.backtest` 上层队列，为 `runtime.backtest.record_store` 建立单子叶等价基线。
**真实文件**:
- `src/backend/runtime/routes/backtest.rs`
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/backtest/v4_runtime_execution.rs`
- `src/runtime/backtest/legacy_dispatch.rs`
- `src/runtime/mod.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/79-runtime.backtest.execution_start抽离方案.md`
- `markdown/06-milestones/v4.16.0/80-runtime.backtest.execution_start抽离记录.md`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md`
- `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`
- `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`
- `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`
- `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md`
- `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`
- `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md`
- `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md`
- `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md`

**职责**:
固定 backtest 创建路径的白箱边界，包括 `start_backtest_run`、legacy `execute_backtest_request`、v4 `execute_v4_backtest_request`、v4 request resolution 子模块调用、artifact view 构建调用、governance event envelope 校验和 transient/in-memory record 写入。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `FrontendRunRequest` | `POST /api/runtime/backtest` | JSON request | 必须保留 capability guard、runtime config capability guard、graph_json 和 execution assumption override 校验 |
| `graph_json` | request body | graph JSON / v4 machine graph | legacy path 走 QS compile；v4 path 走 v4 graph resolution |
| `AppState` / `UserId` | backend runtime | shared state / scoped user | 不迁移 store dir、lock order 或 scoped key 语义 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestRunResponse` | frontend、tests | JSON response | 不改变 response schema |
| transient `BacktestRecord` | transient spill 或 `state.backtests` | governed record | 不改变 spill threshold、governance、artifact views 或 scoped key |
| runtime events | artifact/event viewer | governed event envelope | 不改变 envelope、stage、severity 或 module_key 语义 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_backtest_run` | `UserId`、`AppState`、`FrontendRunRequest` | `BacktestRunResponse` | `backend.runtime.routes.backtest` | 不得混入 record/replay/experiment |
| `execute_backtest_request` | `AppState`、`UserId`、`FrontendRunRequest`、optional suffix | `BacktestRecord` | `start_backtest_run`、experiment sweep helper | 不得改变 legacy sandbox、event envelope、artifact 或 spill 语义 |
| `execute_v4_backtest_request` | `AppState`、`UserId`、`FrontendRunRequest`、graph JSON、optional suffix | `BacktestRecord` | `execute_backtest_request` | 不得改变 v4 graph/symbol/event resolution 或 deterministic replay |

**父级通信规则**:
`runtime.backtest.execution_start` 只能经父级 `runtime.backtest` 和 `backend.runtime.routes.backtest` 暴露创建路径；不得横向直接接管 record store、replay、experiment、compare、artifact schema、persistence、state 或 frontend caller。

**允许调用的子模块**:
仅允许继续使用既有 `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_runtime_execution.rs` 和 `src/runtime/backtest/legacy_dispatch.rs`。共享 owner 保持原位，不在本叶私有化。父级 `runtime` 是唯一兼容桥，禁止 sibling 横向直连。

**回归保护**:
`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**细分价值判断**:
本节点已完成单叶 closeout，判定不设置 `stop_split: true`。BE-001O-04 已确认 `runtime.backtest.execution_start.v4_projection` 等价并设置 `stop_split: true`；BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`；BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`；BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`；BE-001S-01 已完成父叶残余判断。上层 `runtime.backtest.record_store` 已在 BE-001T-04 完成 closeout，`runtime.backtest.replay` 已在 BE-001U-04 完成 closeout，`runtime.backtest.experiment_sweep` 已在 BE-001V-04 完成单叶 closeout 并设置 `stop_split: false`，`runtime.backtest.experiment_sweep.parameter_grid` 已在 BE-001W-04 完成单叶 closeout 并设置 `stop_split: true`，`runtime.backtest.experiment_sweep.start_orchestration` 已在 BE-001Y-04 完成单叶 closeout 并设置 `stop_split: true`，BE-001Z-01 已完成第二轮父叶残余判断，BE-001AA-01 已建立 `record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；下一批若继续只能进入 BE-001AD-01 `backend.runtime.routes` 父叶残余判断。不得在本父叶内私拆 record finalize、state write、persistence、schema 或 frontend。

**幻觉检查点**:
AI 声称 `runtime.backtest.execution_start` 已完成父叶残余判断时，必须说明只完成创建路径 handler/helper 的抽离、四个内部子叶 closeout 和 `no code movement` 的残余判断；下一候选回到 `runtime.backtest.record_store`。不得宣称 record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.10 `runtime.backtest.execution_start.v4_projection`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection`
**父模块**: `runtime.backtest.execution_start`
**状态**: v4.16 BE-001O-04 单叶 closeout 已完成。projection helper 与现有两个单元测试已迁入 `src/runtime/backtest/v4_projection.rs`，等价成立，并设置 `stop_split: true`；下一候选回到父叶 `runtime.backtest.execution_start.v4_request_resolution`，不得直接移动 request resolution、record write、artifact schema、response schema、state、persistence 或 frontend caller。
**真实文件**:
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md`
- `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`

**职责**:
只承载 v4 backtest artifact projection 白箱边界，将 `V4BacktestArtifact`、equity curve 与 final snapshot 投影为 `BacktestOutput`、`BacktestEquityPoint`、`PortfolioState` 和 `FrontendRuntimeEvent`。本节点不拥有 v4 request resolution、record write、artifact schema、response schema、state lock、persistence 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `V4BacktestArtifact` | `execute_v4_backtest_request` | v4 backtest artifact | 不改变 schema、trajectory、risk decision 或 final snapshot 语义 |
| `equity_curve` | `v4_equity_curve_from_artifact` | `Vec<BacktestEquityPoint>` | 空 artifact 返回空数组，不补造 zero point |
| `backtest_id` | parent execution path | `&str` | 只用于 frontend event id / trace id 前缀 |
| `final_snapshot` | v4 artifact | JSON value | 只读取 simulated execution portfolio 和 asset curve 字段 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestOutput` | artifact views、response mapping | qrpc core output | 不改变 `mode = v4_backtest`、summary、portfolio 或 artifact embedding |
| `BacktestEquityPoint` | output、frontend projection | equity point vector | 空 artifact 必须保持空数组 |
| `PortfolioState` | `BacktestOutput.final_portfolio` | qrpc core portfolio | 不改变 cash、net/gross notional 和 timestamp 映射 |
| `FrontendRuntimeEvent` | artifact view / frontend event stream view | governed frontend event | 不改变 event type、severity、payload projection 或 sort order |

**关键 helper**:
| helper | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `build_v4_backtest_output` | `V4BacktestArtifact`、equity curve | `BacktestOutput` | `execute_v4_backtest_request` | 不得改变 summary、trade count、step count 或 artifact embedding |
| `v4_win_rate_from_equity_curve` | equity curve | win rate | `build_v4_backtest_output` | 不得把 flat step 或非有限值算作方向步 |
| `v4_equity_curve_from_artifact` | `V4BacktestArtifact` | equity curve | `execute_v4_backtest_request` | 不得为空 artifact 伪造 zero point |
| `v4_portfolio_from_artifact` | `V4BacktestArtifact` | `PortfolioState` | `build_v4_backtest_output` | 不得改变安全默认值和 ended timestamp |
| `frontend_events_from_v4_backtest_artifact` | artifact、backtest id | frontend events | artifact view builder | 不得改变 projection 字段、trace id 或排序 |
| `v4_frontend_event` | event fields | `FrontendRuntimeEvent` | frontend event projection | 不得改变 `RuntimeEventEnvelope::default()` |

**父级通信规则**:
`runtime.backtest.execution_start.v4_projection` 只能由父级 `runtime.backtest.execution_start` 调用，且只能作为父模块内部 helper 使用。不得让 record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前没有更低层子模块，且本叶已设置 `stop_split: true`。`src/runtime/backtest/v4_projection.rs` 只能被父级 `src/runtime/backtest/execution_start.rs` 私有调用；若发现 request resolution 或 schema owner 需要拆分，必须回到父叶另起基线。

**抽离方案**:
BE-001O-03 已按 BE-001O-02 方案移动 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 三个父级调用 helper，以及 `v4_win_rate_from_equity_curve`、`v4_portfolio_from_artifact`、`v4_frontend_event` 三个子模块私有 helper 和现有两个单元测试。父级只私有导入三个入口 helper，不新增 public API。

**单叶 closeout**:
BE-001O-04 已确认 `runtime.backtest.execution_start.v4_projection` 等价成立，并设置 `stop_split: true`。本叶没有 state、IO、锁、route、persistence、schema owner 或外部 API；继续拆成 output projection / frontend event projection 只会增加父级导入面，不会减少耦合。下一候选回到父叶 `runtime.backtest.execution_start.v4_request_resolution`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 `runtime.backtest.execution_start.v4_projection` 已 closeout 时，必须说明只完成 projection helper 与现有两个单元测试的等价 closeout，并设置 `stop_split: true`。不得宣称 `execute_v4_backtest_request`、request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.11 `runtime.backtest.execution_start.v4_request_resolution`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution`
**父模块**: `runtime.backtest.execution_start`
**状态**: v4.16 BE-001P-04 单叶 closeout 已完成。`is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` 已迁入 `src/runtime/backtest/v4_request_resolution.rs`，父级 `src/runtime/backtest/execution_start.rs` 只私有导入四个入口 helper；本叶等价成立并设置 `stop_split: true`。replay/runtime execution、projection、record write、artifact schema、response schema、state、persistence 和 frontend caller 均未迁移。
**真实文件**:
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`
- `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`
- `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`

**职责**:
只承载 v4 backtest 创建路径进入 replay/runtime 前的 request resolution 白箱边界，包括 v4 path detection、v4 machine graph resolution、symbol resolution 和 replay market event type resolution。本节点不拥有 v4 replay bars/ticks、runtime execution、projection、record write、artifact views、response mapping、state lock、persistence 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `FrontendRunRequest` | `execute_backtest_request` / `execute_v4_backtest_request` | request body | 不改变 `runtime_kind`、symbols、backtest options 或 capability guard 语义 |
| `graph_json` | request body | `serde_json::Value` | 不改变 v4 graph pointer 优先级或 fallback 顺序 |
| `V4MachineGraphContract` | graph JSON、formal QS handoff、core IR bridge | v4 machine graph | 必须继续执行 static contract validation |
| `MachineEventCatalog` | v4 machine graph | event catalog | 缺失或无可 replay event 时保持 `v4_event_catalog_missing` |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| v4 path 判定 | `execute_backtest_request` | bool | 不改变 runtime kind、v4 graph、formal source 任一入口判定 |
| graph | `execute_v4_backtest_request` | `V4MachineGraphContract` | 不改变错误 code、validation code 或 fallback bridge 语义 |
| symbols | `execute_v4_backtest_request` | `Vec<String>` | 不改变 request symbols 优先级和 normalize 行为 |
| event type | `execute_v4_backtest_request` | `String` | 不改变 market data event 选择优先级 |

**关键 helper**:
| helper | 当前职责 | 基线约束 |
| --- | --- | --- |
| `is_v4_backtest_request` | 判断是否走 v4 backtest path | 不改变 `runtime_kind = v4`、v4 graph pointers、formal source `v4_strategy` 判定 |
| `resolve_v4_backtest_graph` | 解析或桥接 v4 machine graph | 不改变 pointer 顺序、formal QS handoff、core IR bridge 或 `ERR_QSC_CONTRACT_INVALID` |
| `resolve_v4_backtest_symbols` | 解析 v4 replay symbols | 不改变 request symbols 优先级、metadata fallback 和默认 normalize |
| `resolve_v4_backtest_market_event_type` | 选择 replay event type | 不改变 market data、`bar`/`price` 优先级和 `v4_event_catalog_missing` 错误 |

**父级通信规则**:
`runtime.backtest.execution_start.v4_request_resolution` 只能由父级 `runtime.backtest.execution_start` 调用，且只能作为父模块内部 helper 使用。不得让 projection、record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前物理子模块为 `src/runtime/backtest/v4_request_resolution.rs`。它只能被父级 `src/runtime/backtest/execution_start.rs` 私有调用；若发现 projection、record write 或 schema owner 需要拆分，必须暂停并另起基线。

**等价基线**:
BE-001P-01 已冻结 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`、错误 code、fallback bridge 和回归证据；当前 `no code movement`。

**抽离方案**:
BE-001P-02 已锁定下一批只移动 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 和 `resolve_v4_backtest_market_event_type` 到父级私有 v4_request_resolution 子模块。父级只私有导入四个入口 helper，不新增 public API，不改变错误 code、fallback bridge、replay/runtime execution、projection、record write 或 schema owner。

**抽离记录**:
BE-001P-03 已按方案新建 `src/runtime/backtest/v4_request_resolution.rs`，并迁入四个 request resolution helper。父级 `src/runtime/backtest/execution_start.rs` 只保留 `#[path = "v4_request_resolution.rs"] mod v4_request_resolution;` 和四个 helper 的私有导入；不新增 public API，不改变错误 code、fallback bridge、replay/runtime execution、projection、record write 或 schema owner。

**单叶 closeout**:
BE-001P-04 已确认四个 request resolution helper 等价，并设置 `stop_split: true`。本叶不继续拆成 detection、graph、symbols 或 event type 微文件；下一批若继续，必须回到父叶 `runtime.backtest.execution_start`，先为 `runtime.backtest.execution_start.v4_runtime_execution` 建立单子叶等价基线。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 `runtime.backtest.execution_start.v4_request_resolution` 已 closeout 时，必须说明只完成四个 request resolution helper 的等价 closeout，并设置 `stop_split: true`。不得宣称 `execute_v4_backtest_request`、replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.12 `runtime.backtest.execution_start.v4_runtime_execution`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution`
**父模块**: `runtime.backtest.execution_start`
**状态**: v4.16 BE-001Q-04 单叶 closeout 已完成，等价成立，并设置 `stop_split: true`。`run_v4_backtest_runtime_execution` 已迁入 `src/runtime/backtest/v4_runtime_execution.rs`，当前只移动 deterministic bars/ticks、blocking runtime replay 和 `V4BacktestArtifact` 输出 helper；`execute_v4_backtest_request`、expanded graph、request resolution、projection、record write、artifact schema、response schema、state、persistence 和 frontend caller 均未迁移。
**真实文件**:
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/backtest/v4_runtime_execution.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md`
- `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md`
- `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`
- `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`
- `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md`
- `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`

**职责**:
只承载 v4 backtest 创建路径中 request resolution 之后、projection/record write 之前的 deterministic replay/runtime execution 白箱边界。第一轮物理抽离已承载 deterministic bars/ticks、blocking runtime execution 和 `V4BacktestArtifact` 输出；symbol-expanded graph 暂留父级，避免本子叶横向调用 request resolution sibling。本节点不拥有 request resolution、projection、record write、artifact views、response mapping、state lock、persistence 或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `expanded_graph` | `execute_v4_backtest_request` 父级 | `V4MachineGraphContract` | 不改变 symbol expansion failure mapping；本批暂留父级生成 |
| `symbols` | `v4_request_resolution` | `Vec<String>` | 不改变 symbol normalize 和 fallback 语义 |
| `event_type` | `v4_request_resolution` | `String` | 不改变 market event selection 语义 |
| `now_ms` | `execute_v4_backtest_request` | timestamp | 不改变 deterministic replay 时间锚点 |
| `tick_replay` | `execute_v4_backtest_request` 父级 | bool | request body 中 `tick_replay` 大小写不敏感；默认 false |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| deterministic bars | v4 runtime replay | bar replay inputs | 不改变 symbols、event type 或 timestamp 语义 |
| deterministic ticks | v4 runtime replay | `Vec<V4BacktestTickInput>` | tick replay 时按 bars 顺序生成，sequence 从 0 开始 |
| `V4BacktestArtifact` | `v4_projection` / artifact views | v4 backtest artifact | 不改变 trajectory、risk decisions、execution capability source 或 final snapshot 语义 |

**关键 public 方法**:
| 方法/调用 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `run_v4_backtest_runtime_execution` | expanded graph、symbols、event type、now_ms、tick_replay | `V4BacktestArtifact` | `execute_v4_backtest_request` | 只能为 `pub(super)` 父级私有入口 |
| `qrpc_runtime::expand_v4_graph_for_symbols` | graph、symbols | expanded graph | `execute_v4_backtest_request` | 本批保留父级，不得私有化 qrpc runtime owner |
| `qrpc_runtime::build_v4_deterministic_replay_bars` | symbols、now_ms、event_type | deterministic bars | `run_v4_backtest_runtime_execution` | 不得改变 replay 输入顺序 |
| `qrpc_runtime::V4BacktestTickInput` | bar projection fields | deterministic ticks | `run_v4_backtest_runtime_execution` tick replay branch | 不得改变 sequence、price、size 或 event type 映射 |
| `tokio::task::spawn_blocking` | blocking replay closure | `V4BacktestArtifact` | `run_v4_backtest_runtime_execution` | 不得改变 async blocking 边界 |
| `V4PaperSimulatedRuntime::new_for_backtest` | expanded graph、runtime matrix、capabilities | runtime | spawn blocking closure | 不得改变 `runtime_simulated_v4_matrix("paper-local")` 或 `ExecutionCapabilityKind::Market` |
| `run_backtest_ticks` | ticks | `V4BacktestArtifact` | tick replay branch | 只在 `tick_replay` 模式调用 |
| `run_backtest_bars` | bars | `V4BacktestArtifact` | bar replay branch | 非 tick replay 默认路径 |

**父级通信规则**:
`runtime.backtest.execution_start.v4_runtime_execution` 只能由父级 `runtime.backtest.execution_start` 调用，且只能作为父模块内部 helper 候选。不得让 request resolution、projection、record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前物理子模块为 `src/runtime/backtest/v4_runtime_execution.rs`，只暴露 `run_v4_backtest_runtime_execution` 这个 `pub(super)` 父级私有入口；不得改变 `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs` 或 `AppState` owner。

**等价基线**:
BE-001Q-01 已冻结 `expand_v4_graph_for_symbols`、`build_v4_deterministic_replay_bars`、`V4BacktestTickInput`、`spawn_blocking`、`V4PaperSimulatedRuntime::new_for_backtest`、`run_backtest_ticks`、`run_backtest_bars`、`V4BacktestArtifact` 输出和回归证据；BE-001Q-02 已建立抽离方案；BE-001Q-03 已迁移 deterministic bars/ticks 与 blocking runtime replay helper；BE-001Q-04 已确认等价并设置 `stop_split: true`。下一批若继续必须回到父叶 `runtime.backtest.execution_start` 另起候选基线。

**单叶 closeout**:
BE-001Q-04 已确认 `run_v4_backtest_runtime_execution` 等价成立，并设置 `stop_split: true`。本叶没有 state、IO、锁、route、persistence、schema owner 或外部 API；继续拆成 replay input / blocking execution / artifact output 只会增加父级导入面，不会减少耦合。`expand_v4_graph_for_symbols` 保留在父级是当前正确边界，因为 event type resolution 仍依赖 expanded graph。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_run`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 `runtime.backtest.execution_start.v4_runtime_execution` 已 closeout 时，必须说明只完成 deterministic bars/ticks、blocking runtime replay 和 `run_v4_backtest_runtime_execution` helper 的等价 closeout，并设置 `stop_split: true`；`expand_v4_graph_for_symbols` 仍保留在父级 `execute_v4_backtest_request` 内。不得宣称 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.13 `runtime.backtest.execution_start.legacy_dispatch`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch`
**父模块**: `runtime.backtest.execution_start`
**状态**: v4.16 BE-001R-04 单叶 closeout 已完成，并设置 `stop_split: true`。legacy non-v4 path 的 QS compile、execution assumption override、compile artifact bundle、blocking FastBacktestSandbox replay 和轻量输出结构已迁入 `src/runtime/backtest/legacy_dispatch.rs`；父级仍保留 validation、v4 bridge、actor/collaboration、id、governance、event envelope、record assembly、artifact views、transient spill、state write、audit log、schema owner、persistence owner、frontend caller 和发布过渡边界。
**真实文件**:
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/backtest/legacy_dispatch.rs`
- `src/runtime/backtest/v4_runtime_execution.rs`
- `src/runtime/backtest/v4_request_resolution.rs`
- `src/runtime/backtest/v4_projection.rs`
- `src/backtest_artifacts.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime_persistence.rs`
- `src/frontend_api_types.rs`
- `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`
- `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md`
- `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md`

**职责**:
作为 `runtime.backtest.execution_start` 父叶的 legacy non-v4 dispatch 子叶，承载 compile preparation 与 sandbox replay 两段式父级私有 helper。它覆盖 `compile_runtime_protocol_via_qs`、`apply_backtest_execution_assumption_overrides`、`compile_runtime_protocol_config`、`resolved_backtest_execution_assumptions`、`resolved_execution_assumption_sources`、`build_compile_artifact_bundle`、`FrontendBacktestReplaySource`、`FastBacktestSandbox`、`DeterministicTestMode::replay_defaults`、`BACKTEST_DETERMINISTIC_SEED`、`tokio::task::spawn_blocking`、latency override、`sandbox.start` 和 `sandbox.run_backtest`；不承载 record assembly。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `graph_json` | 父级 `execute_backtest_request` | JSON graph | legacy path 继续走 QS compile；v4 path 仍由父级 bridge 分流 |
| `request` | route / experiment caller | `FrontendRunRequest` | 不改变 runtime_config、backtest_options、actor 或 metadata 语义 |
| `plan` | `prepare_legacy_backtest_dispatch` | `LegacyBacktestDispatchPlan` | 只携带 compiled protocol 和 execution assumption snapshots |
| `now_ms` | 父级 timestamp | u64 | 不改变 replay defaults、compile artifact 或 id time anchor |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `compiled` | parent record/spec/artifact assembly | compiled runtime protocol | 不改变 protocol name、config hash、compiled config 或 core IR |
| `artifacts` | parent `BacktestRecord.artifacts` | compile artifact bundle | 不改变 artifact source kind 或 metadata |
| `backtest` | parent record/event projection | sandbox backtest output | 不改变 portfolio、summary、trade count 或 replay semantics |
| resolved assumptions | parent `build_backtest_spec` | execution assumption snapshot | 不改变 latency override 或 source attribution |

**关键 public 方法**:
| 方法/调用 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `compile_runtime_protocol_via_qs` | graph JSON | QS protocol | legacy path | 不得绕过 QS compile path |
| `apply_backtest_execution_assumption_overrides` | protocol、request overrides | runtime protocol | legacy path | 不得改变 override precedence |
| `compile_runtime_protocol_config` | runtime protocol | compiled config | legacy path | 不得改变 protocol name/config hash |
| `prepare_legacy_backtest_dispatch` | graph JSON、request | `LegacyBacktestDispatchPlan` | 父级 `execute_backtest_request` | 只做 compile/assumption preparation |
| `run_legacy_backtest_dispatch` | plan、request、now_ms | `LegacyBacktestDispatchOutput` | 父级 `execute_backtest_request` | 不得迁移 record assembly |
| `build_compile_artifact_bundle` | metadata、compiled | artifact bundle | legacy path | 不得迁移 artifact schema owner |
| `FastBacktestSandbox::with_replay_from_core_ir` | core IR、now_ms | sandbox | HistoricalReplay | 不得吞掉本地市场数据缺失错误 |
| `FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode` | core IR、test mode | sandbox | DeterministicMock | 不得改变 deterministic seed |
| `tokio::task::spawn_blocking` | legacy replay closure | backtest output | legacy path | 不得改变 blocking boundary |

**父子通信规则**:
`runtime.backtest.execution_start.legacy_dispatch` 只能由父级 `runtime.backtest.execution_start` 调用，且只能作为父模块内部 helper 候选。不得让 v4 request resolution、v4 projection、v4 runtime execution、record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前物理子模块为 `src/runtime/backtest/legacy_dispatch.rs`，只暴露 `prepare_legacy_backtest_dispatch`、`run_legacy_backtest_dispatch`、`LegacyBacktestDispatchPlan` 和 `LegacyBacktestDispatchOutput` 这些 `pub(super)` 父级私有入口；不得被 sibling 横向调用。

**等价基线**:
BE-001R-01 已冻结 legacy compile/sandbox dispatch 的输入输出、排除项和验证计划。当前不得宣称 helper 已抽离，不得宣称 `execute_backtest_request` 已整理，也不得迁移 record write、artifact views、transient spill、`state.backtests`、persistence、schema owner 或 frontend caller。
**抽离方案**:
BE-001R-02 已限定下一批 BE-001R-03 只允许迁移 legacy compile/sandbox dispatch 最小 helper。允许迁移 `compile_runtime_protocol_via_qs`、`apply_backtest_execution_assumption_overrides`、`compile_runtime_protocol_config`、`resolved_backtest_execution_assumptions`、`resolved_execution_assumption_sources`、`build_compile_artifact_bundle`、`FrontendBacktestReplaySource`、`FastBacktestSandbox`、`DeterministicTestMode::replay_defaults`、`BACKTEST_DETERMINISTIC_SEED`、`tokio::task::spawn_blocking`、latency override、`sandbox.start` 和 `sandbox.run_backtest` 所属连续段；必须保留 parent record assembly、artifact views、spill、state write 和 audit log。
**抽离记录**:
BE-001R-03 已按方案新建 `src/runtime/backtest/legacy_dispatch.rs`，并迁入 legacy compile/assumption/artifact/sandbox replay 两段式 helper。父级 `src/runtime/backtest/execution_start.rs` 只保留 path module 与 `pub(super)` 私有导入，不新增 public API，不改变 record assembly、artifact views、spill、state write、audit log 或发布过渡边界。
**单叶 closeout**:
BE-001R-04 已确认 legacy dispatch helper 等价成立，并设置 `stop_split: true`。本叶没有 state、IO 持久化、锁、route、schema owner、frontend caller 或外部 API；继续拆成 compile preparation / artifact bundle / sandbox replay 微叶只会扩大父级导入面，不会减少真实耦合。

**后续递归队列**:
BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，下一步回到 `runtime.backtest` 上层队列，默认候选为 `runtime.backtest.record_store`。若要动 record write、artifact schema、state owner、persistence owner 或 frontend caller，必须另起提案并回到适配性校验。

**回归保护**:
`cargo fmt --check`，`cargo check -p quantpilot`，`cargo test --no-run`，`cargo test -p quantpilot --test api_backtest`，`cargo test -p quantpilot --test api_evidence_contract`，`cargo test -p quantpilot --test api_run`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。
**幻觉检查点**:
AI 声称 `runtime.backtest.execution_start.legacy_dispatch` 已 closeout 时，必须说明当前只完成 legacy compile/assumption/artifact/sandbox replay helper 的等价 closeout，并设置 `stop_split: true`。不得宣称 record write/persistence/state/frontend owner 已迁移、发布过渡已启动、`runtime.backtest.execution_start` 已整体停止细分，或整理/重构已经完成。

### 5.1.14 `runtime.backtest.record_store`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.record_store`
**父模块**: `runtime.backtest`
**状态**: v4.16 BE-001T-04 单叶 closeout 已完成并设置 `stop_split: true`。`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 已迁入 `src/runtime/backtest/record_store.rs` 并确认等价；本叶不继续细拆，`runtime.backtest.replay` 已由 BE-001U-04 完成 closeout。后续不能从 record_store 混入 experiment、compare、shared helper owner、state owner、persistence owner、artifact/transient owner、response mapping、frontend caller 或发布过渡。
**真实文件**:
- `src/backend/runtime/routes/backtest.rs`
- `src/runtime/backtest/record_store.rs`
- `src/runtime/mod.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/backtest_artifacts.rs`
- `src/collaboration.rs`
- `src/frontend_api_types.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md`
- `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/100-runtime.backtest.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/101-runtime.backtest.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/102-runtime.backtest.record_store单叶closeout.md`

**职责**:
固定 backtest record store handler 边界，包括已保存 backtest 列表、detail lookup、transient/in-memory record 保存、artifact view materialization、transient cleanup、保存审计和未保存 record 丢弃。它不承载 backtest 创建、replay window、experiment sweep、compare logic、artifact schema 设计或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `AppState` | backend app state | shared state | 只读取既有 `backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir` |
| `UserId` + `backtest_id` | auth middleware、path param | scoped user / string id | detail/save/discard 必须继续使用 scoped key 或安全路径段 |
| `PaginationQuery` | `/api/runtime/backtests` | pagination query | 不改变分页语义或 created_at 倒序排序 |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `PaginatedResponse<BacktestListItem>` | frontend、tests | JSON response | 不改 list schema、filter metadata 或 execution assumption tag |
| `BacktestDetailResponse` | frontend、tests | JSON response | 不改 governance、artifact views、account、events 或 diagnostics source |
| persisted artifact directory | filesystem | manifest/event log/metrics/trades/equity | 必须继续由 `persist_backtest_record` 和 `backtest_artifacts` owner 写入 |
| transient cleanup | filesystem | transient store mutation | `save_backtest_record` 和 `discard_backtest_record` 继续调用 `delete_transient_backtest_record` |
| graph audit entry | audit store | `GraphAuditAction::BacktestCreated` | 只在 record actor 存在时写入 |
| discard response | frontend、tests | `DiscardRuntimeArtifactResponse` | 已保存记录必须 conflict；不存在记录必须 not found |

**关键 public 方法**:
| 方法/调用 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_backtests` | `AppState`、`PaginationQuery` | `PaginatedResponse<BacktestListItem>` | `GET /api/runtime/backtests` | 不得列出 transient/in-memory 未保存 record |
| `get_backtest_detail` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `GET /api/runtime/backtests/:backtest_id` | 不得绕过 scoped lookup 或 governance normalization |
| `save_backtest_record` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `POST /api/runtime/backtests/:backtest_id/save` | 不得绕过 persistence、artifact views、transient cleanup 或 audit |
| `discard_backtest_record` | `UserId`、`AppState`、`backtest_id` | `DiscardRuntimeArtifactResponse` | `DELETE /api/runtime/backtests/:backtest_id` | 不得删除正式保存记录 |
| `load_backtest_record_from_state` | `AppState`、`UserId`、`backtest_id` | `BacktestRecord` | detail/save/replay/compare/report | 不得改变 memory -> artifact dir -> transient fallback 顺序 |
| `list_backtest_records` | `backtest_store_dir` | `Vec<BacktestRecord>` | list handler | 不得读取 promotion work dir |
| `persist_backtest_record` | store dir、record | artifact views | save handler | 不得迁移 artifact schema owner |
| `delete_transient_backtest_record` | transient dir、id | filesystem cleanup | save/discard handler | 不得改变 transient retention/quota owner |
| `backtest_list_item_from_record` / `backtest_detail_response_from_record` | record | API response | list/detail/save | 不得改 response schema |

**父级通信规则**:
`runtime.backtest.record_store` 只能经父级 `runtime.backtest` 和 route facade `backend.runtime.routes.backtest` 暴露 record store API。不得让 `runtime.backtest.execution_start`、`runtime.backtest.replay`、`runtime.backtest.experiment_sweep`、`backtest_compare`、persistence owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前物理子模块为 `src/runtime/backtest/record_store.rs`，通过父级 `src/runtime/mod.rs` re-export 暴露四个 route handler。它只允许引用既有 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs` 和 `src/frontend_api_types.rs`。`AppState.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir` 保持原 owner。

**等价基线**:
BE-001T-01 已冻结 backtest list/detail/save/discard、transient/persistent record、artifact view、audit 和排除边界。该基线批次为 `no code movement`，不得迁移 replay、experiment、compare、artifact schema owner、state owner、persistence owner 或 frontend caller。

**抽离方案**:
BE-001T-02 已限定下一批 BE-001T-03 只迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 四个 handler 到 record_store 子模块文件。父级 `src/runtime/mod.rs` 只做受控私有子模块接入和 re-export，`src/backend/runtime/routes/backtest.rs` route facade 保持不变；`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` 和 AppState/store dir owner 均不迁移。

**抽离记录**:
BE-001T-03 已按方案新建 `src/runtime/backtest/record_store.rs`，并迁入 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 四个 handler。父级 `src/runtime/mod.rs` 保留 `crate::runtime::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}` 兼容出口；`src/backend/runtime/routes/backtest.rs` 未改 route path、method 或 handler 调用名。

**单叶 closeout**:
BE-001T-04 已确认四个 handler 等价，并设置 `stop_split: true`。继续拆成 list/detail/save/discard 微文件会增加父级 re-export 和导入面，但不会改善 owner 清晰度；persistence、artifact/transient、audit、response mapping 和 AppState 均继续保留共享 owner。

**后续递归队列**:
`runtime.backtest.record_store` 后续队列已由 BE-001U-04 `runtime.backtest.replay` closeout 承接并收口。不得继续细拆 `runtime.backtest.record_store`，也不得直接迁移 shared helper、persistence、audit、artifact/transient、response mapping、frontend route 或发布过渡连接。

**回归保护**:
`cargo fmt --check`，`cargo check -p quantpilot`，`cargo test --no-run`，`cargo test -p quantpilot --test api_backtest`，`cargo test -p quantpilot --test api_evidence_contract`，`cargo test -p quantpilot --test api_run`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。
**幻觉检查点**:
AI 声称 `runtime.backtest.record_store` 已完成时，必须说明只完成 backtest record list/detail/save/discard handler 子模块的抽离与 closeout，并设置 `stop_split: true`；src/runtime/backtest.rs (retired drained include) 仍拥有 replay、experiment 和其他 sibling，state owner、shared helper owner、persistence owner、artifact/transient owner、frontend route、发布版本过渡、整理和重构均未完成。不得宣称 backtest handler 全部完成。

### 5.1.15 `runtime.backtest.replay`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.replay`
**父模块**: `runtime.backtest`
**状态**: v4.16 BE-001U-04 单叶 closeout 已完成并设置 `stop_split: true`。`get_backtest_replay` 已迁入 `src/runtime/backtest/replay.rs` 并确认等价；route facade、record lookup、query normalization、response mapping、schema、metrics、state/persistence、artifact schema、frontend caller 和发布过渡边界均保持不变。后续 sibling 已由 BE-001AA-01 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线承接，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。
**真实文件**:
- `src/backend/runtime/routes/backtest.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/mod.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/backtest_artifacts.rs`
- `src/frontend_api_types.rs`
- `src/lib.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`
- `markdown/06-milestones/v4.16.0/103-runtime.backtest.replay单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/104-runtime.backtest.replay抽离方案.md`
- `markdown/06-milestones/v4.16.0/105-runtime.backtest.replay抽离记录.md`
- `markdown/06-milestones/v4.16.0/106-runtime.backtest.replay单叶closeout.md`
- `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md`

**职责**:
固定 backtest replay handler 边界，包括 replay route、用户作用域 backtest record lookup、`RuntimeReplayQuery` normalization、artifact event log 优先的 replay response mapping、cursor/filter/checkpoint/timeline 输出和 replay metrics。它不承载 record list/detail/save/discard、backtest 创建、experiment sweep、compare logic、artifact schema 设计或 frontend caller。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `UserId` + `backtest_id` | auth middleware、path param | scoped user / string id | 必须继续经 `load_backtest_record_from_state` 读取用户作用域 record |
| `RuntimeReplayQuery` | query string | cursor/filter query | 保持 `cursor`、`checkpoint`、`sequence_cursor`、`limit`、`stage`、`severity`、`retention_class`、`module_key`、`key_only` 语义 |
| `AppState` | backend app state | shared state | 只读取 record 与 `evidence_metrics`，不迁移 AppState owner |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `RuntimeReplayResponse` | frontend、tests | JSON response | 不改 `kind`、`record_id`、`graph_id`、events、timeline、checkpoints、filters、cursor 或 account schema |
| bad cursor error | frontend、tests | `bad_replay_cursor` | 越界 cursor / sequence_cursor 仍由 response mapping 错误映射 |
| replay metrics | `RuntimeEvidenceMetrics` | latency counter | 成功 replay page 后继续调用 `record_replay_page` |

**关键 public 方法**:
| 方法/调用 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_backtest_replay` | `UserId`、`AppState`、`backtest_id`、`RuntimeReplayQuery` | `RuntimeReplayResponse` | `GET /api/runtime/backtests/:backtest_id/replay` | 不得改变 route、cursor/filter 语义或 metrics 记录 |
| `load_backtest_record_from_state` | `AppState`、`UserId`、`backtest_id` | `BacktestRecord` | replay/detail/save/compare/report | 不得改变 memory -> artifact directory -> transient fallback 顺序 |
| `normalized_replay_options` | `RuntimeReplayQuery` | `RuntimeReplayOptions` | replay handlers | 不得改变 default limit、max limit 或 checkpoint/cursor 优先级 |
| `backtest_replay_response_from_record` | `BacktestRecord`、`RuntimeReplayOptions` | `RuntimeReplayResponse` | backtest replay handler | 不得改变 artifact event log 优先级或 response schema |
| `runtime_replay_response` | record kind、events、options | `RuntimeReplayResponse` | run/backtest replay mapping | 不得私有化到 backtest replay 叶子 |
| `record_replay_page` | latency ms | metrics mutation | replay handler | 不得迁移 metrics owner |

**父级通信规则**:
`runtime.backtest.replay` 只能经父级 `runtime.backtest` 和 route facade `backend.runtime.routes.backtest` 暴露 replay API。不得让 `runtime.backtest.record_store`、`runtime.backtest.execution_start`、`runtime.backtest.experiment_sweep`、`backtest_compare`、response mapping owner、schema owner、state/persistence owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**允许调用的子模块**:
当前物理子模块为 `src/runtime/backtest/replay.rs`，只承载 `get_backtest_replay`；父级 `src/runtime/mod.rs` 通过 `#[path = "backtest/replay.rs"] mod backtest_replay;` 和 `pub(crate) use backtest_replay::get_backtest_replay;` 暴露兼容入口。它只允许引用既有 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/frontend_api_types.rs` 和 `src/lib.rs` metrics owner。`RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint` 和 `RuntimeReplayEventItem` 保持原 schema owner。

**等价基线**:
BE-001U-01 已冻结 replay route、record lookup、query normalization、artifact event log 优先级、cursor/filter/checkpoint/timeline response mapping、bad cursor error 和 metrics。该基线本身不代表 handler 已迁移；BE-001U-03 之后才允许声明 replay 物理文件已存在，且不得迁移 record_store、execution_start、experiment、compare、artifact schema owner、state owner、persistence owner、frontend caller 或发布过渡。

**抽离方案**:
BE-001U-02 已限定下一批 BE-001U-03 只迁移 `get_backtest_replay` 到 planned replay module file。父级 `src/runtime/mod.rs` 只做受控私有子模块接入和 re-export，`src/backend/runtime/routes/backtest.rs` route facade 保持不变；`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs`、AppState/store dir owner 和 artifact schema owner 均不迁移。

**抽离记录**:
BE-001U-03 已将 `get_backtest_replay` 迁入 `src/runtime/backtest/replay.rs`。src/runtime/backtest.rs (retired drained include) 继续承载 experiment sweep 和后续 sibling；`src/runtime/mod.rs` 只新增 `backtest_replay` 私有模块和 re-export；`src/backend/runtime/routes/backtest.rs` 未改动。record lookup、query normalization、response mapping、schema、metrics、state/persistence、artifact schema、frontend caller 和发布过渡均不迁移。

**单叶 closeout**:
BE-001U-04 已确认 `get_backtest_replay` 等价，并设置 `stop_split: true`。本叶不继续拆成 query adapter、record lookup、response projection、metrics hook、bad cursor adapter 或 timeline filter；这些 owner 分别保留在共享 query/options、persistence、response mapping、metrics 和 schema 边界。

**后续递归队列**:
后续 sibling 队列已由 BE-001AA-01 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线承接，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。不得继续细拆 replay，也不得顺手迁移 query normalization、response mapping、schema、metrics、record lookup、state/persistence、artifact schema、frontend route 或发布过渡连接。

**回归保护**:
`cargo fmt --check`，`cargo check -p quantpilot`，`cargo test --no-run`，`cargo test -p quantpilot --test api_backtest`，`cargo test -p quantpilot --test api_evidence_contract`，`cargo test -p quantpilot --test api_run`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`，`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。
**幻觉检查点**:
AI 声称 `runtime.backtest.replay` 已完成时，必须说明只完成 `get_backtest_replay` handler 子模块抽离与 closeout，并设置 `stop_split: true`。不得宣称 query normalization、response mapping、schema、metrics、record lookup、record_store、execution_start、experiment、compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.16 `runtime.backtest.experiment_sweep`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.experiment_sweep`
**父模块**: `runtime.backtest`
**状态**: v4.16 BE-001V-04 单叶 closeout 已完成。5 个 experiment handler 已迁入 `src/runtime/backtest/experiment_sweep.rs` 并确认等价，3 个参数网格 helper 已在 BE-001W-04 完成抽离与单叶 closeout；BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，确认 `parameter_grid` 关闭并设置 `stop_split: true`，但父叶仍保持 `stop_split: false`；BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成第二轮父叶残余判断；BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`。BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。src/runtime/backtest.rs (retired drained include) 仅保留 drained parent include 注释，route 真实 owner 仍是 `src/backend/runtime/routes.rs`。

**真实文件**:
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/parameter_grid.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `src/lib.rs`

**治理文档**:
- `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md`
- `markdown/06-milestones/v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md`
- `markdown/06-milestones/v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md`
- `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`
- `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md`
- `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`
- `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`
- `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md`
- `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`
- `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md`

**职责**:
承载 backtest experiment sweep 的创建、列表、详情、保存和丢弃 API 边界。该叶只拥有 experiment handler 编排，不拥有 route aggregate、backtest execution_start 实现、record_store、replay、compare、artifact schema、state owner、persistence owner、response mapping owner、schema owner、frontend caller 或发布过渡连接。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_backtest_experiment` | `UserId`、`AppState`、`FrontendExperimentRequest` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得绕过 capability/config/execution assumption guard |
| `list_experiments` | `AppState`、`PaginationQuery` | `PaginatedResponse<ExperimentListItem>` | `backend.runtime.routes` | 不得改变 created_at 倒序或分页语义 |
| `get_experiment_detail` | `UserId`、`AppState`、`experiment_id` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得绕过 scoped experiment lookup |
| `save_experiment_record` | `UserId`、`AppState`、`experiment_id` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得跳过 variant backtest 持久化、transient 清理或 audit |
| `discard_experiment_record` | `UserId`、`AppState`、`experiment_id` | `DiscardRuntimeArtifactResponse` | `backend.runtime.routes` | 不得允许 saved experiment discard |
| `build_experiment_overrides` | `FrontendExperimentRequest`、`RuntimeProtocolCoreConfig` | `Vec<FrontendExecutionAssumptionOverrides>` | `start_backtest_experiment` | 不得改变 empty grid、负数校验、去重、base fallback 或 `MAX_EXPERIMENT_VARIANTS` |
| `execute_backtest_request` | `AppState`、`UserId`、`FrontendRunRequest`、optional suffix | `BacktestRecord` | `start_backtest_experiment` | 只能作为父级 runtime 内部复用桥，不得扩大为 sibling 横向连接 |

**白箱输入输出**:
| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `FrontendExperimentRequest` | frontend runtime API | 必须保留 `experiment_name`、`actor`、`capability_context`、`runtime_config`、`graph_json`、`runtime_targets`、`backtest_options`、`parameter_grid` |
| 输入 | `FrontendExecutionAssumptionSweepGrid` | experiment request | fee/slippage 不得为负；空轴回退 base；空网格报错；variant 总数受限 |
| 输入 | `execute_backtest_request` | `runtime.backtest.execution_start` | 只能经父级 runtime 内部桥调用 |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 保留 definition、variants、saved 状态和 execution assumptions tag |
| 输出 | `ExperimentListItem` | frontend/tests | 保留 sweep axes、best variant 和分页语义 |
| 输出 | `DiscardRuntimeArtifactResponse` | frontend/tests | 只允许未保存 experiment discard |

**父级通信规则**:
`runtime.backtest.experiment_sweep` 只能经父级 `runtime` 兼容出口和 `backend.runtime.routes` 暴露 experiment API；当前 route 真实 owner 仍是 `backend.runtime.routes`，不得在基线阶段擅自搬到 `backend.runtime.routes.backtest`。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_experiments`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**父叶残余判断**:
BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断。`runtime.backtest.experiment_sweep.parameter_grid` 已关闭并设置 `stop_split: true`；父叶仍保持 `stop_split: false`，因为 `start_backtest_experiment` 仍是创建路径高风险编排段，集中接触 capability/config guard、QS compile、variant request assembly、`execute_backtest_request` 复用桥、preview persistence 和 response assembly。

BE-001Z-01 已完成第二轮父叶残余判断。`runtime.backtest.experiment_sweep.parameter_grid` 与 `runtime.backtest.experiment_sweep.start_orchestration` 均已关闭并设置 `stop_split: true`；父叶仍保持 `stop_split: false`，因为 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 共同形成 experiment record lifecycle 边界，集中接触 scoped lookup、created_at 倒序分页、variant backtest persistence、transient cleanup、state cache、audit 和 response mapping。下一候选固定为 `runtime.backtest.experiment_sweep.record_lifecycle`，默认 BE-001AA-01。

BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线。当前 `no code movement`，只冻结 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 的白箱输入输出、排序分页、scoped lookup、saved conflict、variant backtest persistence、transient cleanup、state cache、audit 和 response mapping 排除边界。

BE-001AA-02 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案。下一批只能进入 BE-001AA-03 实际抽离记录，按计划目标 src/runtime/backtest/record_lifecycle.rs 迁移四个 lifecycle handler，并通过父级 `mod record_lifecycle;` 与受控 `pub(crate) use record_lifecycle::{...};` 保持兼容出口。

**单子叶抽离记录**:
BE-001Y-03 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 实际抽离。`start_backtest_experiment` 已迁入 `src/runtime/backtest/start_orchestration.rs`，父级 `experiment_sweep` 通过 `mod start_orchestration;` 和 `pub(crate) use start_orchestration::start_backtest_experiment;` 保持兼容出口。

BE-001AA-03 已完成 `runtime.backtest.experiment_sweep.record_lifecycle` 实际抽离。`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 已迁入 `src/runtime/backtest/record_lifecycle.rs`，父级 `experiment_sweep` 通过 `mod record_lifecycle;` 和 `pub(crate) use record_lifecycle::{...};` 保持兼容出口。

**单叶 closeout**:
BE-001Y-04 已确认 `runtime.backtest.experiment_sweep.start_orchestration` 等价，并设置 `stop_split: true`。本子叶不继续拆成 guard pipeline、protocol resolution、variant request assembly、variant execution bridge、summary projection 或 preview persistence adapter；这些拆分只会增加微文件和父级导入面，不会形成新的 owner。

BE-001AA-04 已确认 `runtime.backtest.experiment_sweep.record_lifecycle` 等价，并设置 `stop_split: true`。本子叶不继续拆成 list/detail/save/discard、read/write、save transition、discard transition、audit adapter 或 persistence adapter；persistence、response mapping、AppState cache、graph audit、path sanitize、schema、frontend caller 和发布过渡连接继续保留外部 owner。

**第三轮父叶残余判断**:
BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断。`parameter_grid`、`start_orchestration`、`record_lifecycle` 三个子叶均已 closeout 并设置 `stop_split: true`；父叶自身当前也设置 `stop_split: true`。该回流已由 BE-001AC-01 承接并完成 `runtime.backtest` 父叶残余判断，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。

**后续队列**:
BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断，并设置父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`。该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案；不得顺手细拆 save/discard、删除 drained parent include、迁移 route facade、execution_start、record_store、replay、compare、state/persistence、response mapping、schema、frontend caller 或发布过渡连接，也不得宣称 `backend.runtime.routes` 上层完成。

**幻觉检查点**:
AI 声称 `runtime.backtest.experiment_sweep` 已完成 BE-001AB-01 时，必须说明 `parameter_grid`、`start_orchestration` 与 `record_lifecycle` 均已完成各自单叶 closeout，父叶也设置 `stop_split: true`，但 BE-001AD-01 已确认 `backend.runtime.routes` 父叶仍保持 `stop_split: false`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。不得宣称 route facade、record_store、replay、compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.17 `runtime.backtest.experiment_sweep.parameter_grid`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid`
**父模块**: `runtime.backtest.experiment_sweep`
**状态**: v4.16 BE-001W-04 单叶 closeout 已完成并设置 `stop_split: true`。3 个参数网格 helper 已迁入 `src/runtime/backtest/parameter_grid.rs` 并确认等价；父级 `experiment_sweep` 只保留 handler 编排和 `pub(super)` 调用。BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001Z-01 已完成父叶残余判断；BE-001AA-01 已建立 `record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。

**真实文件**:
- `src/runtime/backtest/parameter_grid.rs`
- `src/runtime/backtest/experiment_sweep.rs`
- `src/frontend_api_types.rs`
- `tests/api_experiments.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`
- `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md`
- `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`
- `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`

**职责**:
只负责把 `FrontendExperimentRequest.parameter_grid` 和 `RuntimeProtocolCoreConfig` 解析成 `Vec<FrontendExecutionAssumptionOverrides>`。本节点不拥有 experiment route、handler orchestration、variant backtest execution、persistence、response mapping、schema owner、state owner、audit、frontend caller 或发布过渡连接。

**关键 helper**:
| helper | 输入 | 输出 | 禁止事项 |
| --- | --- | --- | --- |
| `normalize_experiment_float_axis` | `values: &[f64]`、`base: f64`、`field: &str` | `Result<Vec<f64>, (StatusCode, String)>` | 不得改变负数错误、base fallback 或去重顺序 |
| `normalize_experiment_latency_axis` | `values: &[u64]`、`base: u64` | `Vec<u64>` | 不得改成 signed/float latency 或排序去重 |
| `build_experiment_overrides` | `FrontendExperimentRequest`、`RuntimeProtocolCoreConfig` | `Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)>` | 不得改变 empty grid、`MAX_EXPERIMENT_VARIANTS`、三层循环顺序或 `Some` 输出 |

**抽离方案**:
BE-001W-03 已把 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` 迁入 `src/runtime/backtest/parameter_grid.rs`；其中 `build_experiment_overrides` 只以 `pub(super)` 暴露给父级 `runtime.backtest.experiment_sweep`，子模块使用 `use super::*` 复用父级上下文。`MAX_EXPERIMENT_VARIANTS`、schema、route、handler orchestration、execution_start 复用桥、persistence、response mapping、state、audit、frontend caller 和发布过渡连接均未迁移。

**单叶 closeout**:
BE-001W-04 已确认参数网格 helper 等价，并设置 `stop_split: true`。本叶不继续拆成 float axis、latency axis、variant expansion、error adapter、limit policy 或 schema 子叶；这些拆分只会增加微文件和父级导入面，不会形成新的 owner。`MAX_EXPERIMENT_VARIANTS`、schema、route、handler orchestration、execution_start 复用桥、persistence、response mapping、state、audit、frontend caller 和发布过渡连接继续保留外部 owner。

**白箱输入输出**:
| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `FrontendExecutionAssumptionSweepGrid` | `FrontendExperimentRequest.parameter_grid` | fee/slippage 可拒绝负数，latency 为 `u64` |
| 输入 | base assumptions | `resolved_backtest_execution_assumptions` | 空轴回退 base，latency 缺失回退 0 |
| 输出 | `Vec<FrontendExecutionAssumptionOverrides>` | `start_backtest_experiment` | fee 外层、slippage 中层、latency 内层展开 |
| 输出 | `bad_request` | route caller | empty grid、负数、variant 超限语义不变 |

**父级通信规则**:
`runtime.backtest.experiment_sweep.parameter_grid` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用。不得让 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_experiments`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**后续队列**:
本叶已完成 BE-001W-04 closeout，不继续细拆，并已交回父叶完成 BE-001X-01 与 BE-001Z-01 残余判断；BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`；BE-001AA-01 已建立 `record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`。BE-001AB-01 已完成第三轮父叶残余判断；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案；不得直接修改 schema、改 `MAX_EXPERIMENT_VARIANTS`、删除 drained parent include、继续细拆 axis normalization/variant expansion 或宣称发布过渡。

**幻觉检查点**:
AI 声称 `runtime.backtest.experiment_sweep.parameter_grid` 已完成时，必须说明只完成 3 个 helper 的抽离与 closeout，并设置 `stop_split: true`；后续 BE-001AB-01 已完成父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`，但 `backend.runtime.routes` 上层仍未完成。不得宣称 route aggregate、execution_start、schema、state、persistence、response mapping、audit、frontend caller、发布版本过渡、整理和重构均已完成。

### 5.1.18 `runtime.backtest.experiment_sweep.start_orchestration`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration`
**父模块**: `runtime.backtest.experiment_sweep`
**状态**: v4.16 BE-001Y-04 单叶 closeout 已完成并设置 `stop_split: true`。`start_backtest_experiment` 已迁入 `src/runtime/backtest/start_orchestration.rs` 并确认等价；BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断；BE-001AA-01 已建立 `record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AD-01 已承接上层父叶残余判断，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。

**真实文件**:
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/parameter_grid.rs`
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `tests/api_experiments.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`
- `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md`
- `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`

**职责**:
冻结 experiment sweep 创建路径的 handler orchestration 边界，包括 capability/config/execution assumption guard、`graph_json` 必填、QS compile、base execution assumptions、parameter grid 调用、variant `FrontendRunRequest` 组装、`execute_backtest_request` 复用桥、variant summary 投影、preview `ExperimentRecord` 组装、preview persistence 和 detail response mapping。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_backtest_experiment` | `UserId`、`AppState`、`FrontendExperimentRequest` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得绕过 guard、重写 parameter grid、横向直连 execution_start 内部 helper 或迁移 record lifecycle |

**白箱输入输出**:
| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `FrontendExperimentRequest` | frontend runtime API | 保留 actor、capability_context、runtime_config、graph_json、runtime_targets、backtest_options、parameter_grid |
| 输入 | `build_experiment_overrides` | `runtime.backtest.experiment_sweep.parameter_grid` | 只作为父级私有 helper 调用 |
| 输入 | `execute_backtest_request` | `runtime.backtest.execution_start` 复用桥 | 只能经父级 runtime 内部桥调用 |
| 输出 | `ExperimentRecord` | experiment store/cache | `saved=false`，包含 definition、variants、actor |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 继续通过 `experiment_detail_response_from_record` 生成 |

**等价冻结项**:
| 行为 | 当前语义 | 不得改变 |
| --- | --- | --- |
| guard order | capability guard -> runtime config guard -> execution assumption guard -> graph_json check | 不得后置或吞掉 details |
| QS compile | `compile_runtime_protocol_via_qs(graph_json)` 在 grid 前执行 | 不得跳过 protocol |
| replay source | 缺失时回退 `FrontendBacktestReplaySource::HistoricalReplay` | 不得改默认 |
| variant request | 每个 override 组装完整 `FrontendRunRequest` | 不得丢 actor/capability/runtime targets/options |
| execution bridge | 每个 variant 调用 `execute_backtest_request`，suffix 为 `{experiment_id}_v{n}` | 不得横向直连 execution_start 内部 helper |
| summary/tag | 优先 artifact metrics，缺失时回退 record summary | 不得只读 request |
| preview persistence | `persist_experiment_record` 后写 `state.experiments` scoped cache | 不得只写内存或只写文件 |

**父级通信规则**:
`runtime.backtest.experiment_sweep.start_orchestration` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_experiments`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**单叶 closeout**:
BE-001Y-04 已确认本叶等价，并设置 `stop_split: true`。本叶不继续拆成 guard pipeline、protocol resolution、variant request assembly、variant execution bridge、summary projection 或 preview persistence adapter；这些内部步骤都依赖同一条创建编排顺序，不形成新的 owner。

**后续队列**:
BE-001AA-04 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout 已完成并设置 `stop_split: true`；BE-001AB-01 已完成父级第三轮父叶残余判断。BE-001AD-01 已承接上层父叶残余判断，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案，不得把父叶残余判断混入本子叶，也不得回改 start_orchestration。

**幻觉检查点**:
AI 声称 `runtime.backtest.experiment_sweep.start_orchestration` 已 closeout 时，必须说明只完成 `start_backtest_experiment` 的抽离与等价 closeout，并设置 `stop_split: true`；record_lifecycle 已由后续 BE-001AA-04 closeout，父叶已完成 BE-001AB-01 第三轮残余判断，但 BE-001AD-01 已确认 `backend.runtime.routes` 父叶仍保持 `stop_split: false`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。不得宣称 route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成，也不得宣称 `backend.runtime.routes` 上层完成。

### 5.1.19 `runtime.backtest.experiment_sweep.record_lifecycle`

**层级路径**: `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle`
**父模块**: `runtime.backtest.experiment_sweep`
**状态**: v4.16 BE-001AA-01 单子叶等价基线已建立，BE-001AA-02 抽离方案已建立，BE-001AA-03 实际抽离已完成。`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 已迁入 `src/runtime/backtest/record_lifecycle.rs`；BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。

**真实文件**:
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/parameter_grid.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/backtest/execution_start.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `src/backtest_artifacts.rs`
- `tests/api_experiments.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`
- `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`
- `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md`
- `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md`

**职责**:
冻结 experiment sweep 的 record lifecycle 读写边界，包括 experiment list/detail、save、discard、variant backtest 固化、transient cleanup、state cache、audit 和 response mapping 排除边界。本节点不拥有 route registration、parameter grid、start orchestration、execution_start、record_store、replay、compare、persistence owner、response mapping owner、schema owner、state owner、frontend caller 或发布过渡连接。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_experiments` | `AppState`、`PaginationQuery` | `PaginatedResponse<ExperimentListItem>` | `backend.runtime.routes` | 不得改变 created_at 倒序、分页顺序或 list response schema |
| `get_experiment_detail` | `auth::UserId`、`AppState`、`experiment_id` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得绕过 scoped experiment lookup |
| `save_experiment_record` | `auth::UserId`、`AppState`、`experiment_id` | `ExperimentDetailResponse` | `backend.runtime.routes` | 不得跳过 variant backtest persistence、transient cleanup、state cache 或 audit |
| `discard_experiment_record` | `auth::UserId`、`AppState`、`experiment_id` | `DiscardRuntimeArtifactResponse` | `backend.runtime.routes` | 不得允许 saved experiment discard 或误删已保存 variant backtest |

**白箱输入输出**:
| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `PaginationQuery` | list route | 先排序后分页，继续复用 `paginate` |
| 输入 | `auth::UserId` | auth middleware | detail/save/discard 必须 scoped lookup |
| 输入 | `experiment_id` | route path | 用于 record lookup、safe path cleanup 和 response id |
| 输出 | `ExperimentListItem` page | frontend/tests | 继续由 `experiment_list_item_from_record` 投影 |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 继续由 `experiment_detail_response_from_record` 投影 |
| 输出 | persisted variant backtests | `src/runtime_persistence.rs` | save 时每个 variant 先固化再清理 transient |
| 输出 | experiment cache/file | `AppState` / experiment store | save 写 `saved=true`，discard 清未保存 record |
| 输出 | graph audit | audit store | actor 存在时 save 写 `ExperimentCreated` |
| 输出 | discard response | frontend/tests | `discarded_kind` 固定为 `experiment` |

**等价冻结项**:
| 行为 | 当前语义 | 不得改变 |
| --- | --- | --- |
| list order | `list_experiment_records` 后映射 list item，再按 `created_at_ms` 倒序 | 不得先分页再排序 |
| scoped detail | `load_experiment_record_from_state(&state,&user_id,&experiment_id)` | 不得跨用户读取或直接扫文件绕过 scope |
| save variant persistence | 每个 variant 先加载 backtest record，再 `persist_backtest_record` 到正式目录 | 不得在任一 variant 失败后继续写 saved experiment |
| transient cleanup | save/discard 都必须清理 transient backtest file/cache | 不得遗漏 transient cleanup |
| saved conflict | saved experiment discard 返回 `StatusCode::CONFLICT` | 不得允许丢弃已保存 experiment |
| safe path | discard 使用 `sanitize_storage_path_segment` 拼接 experiment file | 不得恢复未清洗路径 |
| response mapping | list/detail 只用 response mapping owner | 不得在本叶私造 schema |
| audit | actor 存在时 save 写 graph audit，失败必须冒泡 | 不得吞掉 audit 失败 |

**父级通信规则**:
`runtime.backtest.experiment_sweep.record_lifecycle` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_experiments`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_evidence_contract`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`git diff --check`。

**抽离方案**:
BE-001AA-02 已锁定下一批只移动 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 四个 handler 到 planned record_lifecycle child file。父级只新增 `mod record_lifecycle;` 和受控 `pub(crate) use record_lifecycle::{...};`，子文件先用 `use super::*;` 复用父级上下文；不得迁移 route registration、parameter_grid、start_orchestration、schema、state、persistence、response mapping、audit、frontend caller 或发布过渡连接。

**抽离记录**:
BE-001AA-03 已按方案新建 `src/runtime/backtest/record_lifecycle.rs`，并迁入 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 四个 handler。父级 `src/runtime/backtest/experiment_sweep.rs` 只保留 `mod record_lifecycle;` 和受控 `pub(crate) use record_lifecycle::{...};`，`src/runtime/mod.rs` 与 route registration 兼容出口不变。

**单叶 closeout**:
BE-001AA-04 已确认四个 lifecycle handler 等价，并设置 `stop_split: true`。本叶不继续拆成 list/detail/save/discard、read/write、save transition、discard transition、audit adapter 或 persistence adapter；这些拆分会扩大父级导入面，但不会形成新的稳定 owner。persistence、response mapping、AppState cache、graph audit、path sanitize、schema、frontend caller 和发布过渡连接继续保留外部 owner。

**后续队列**:
BE-001AA-04 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout 已完成，并设置 `stop_split: true`。BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断，该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案；不得直接细拆 save/discard，也不得混入 parameter_grid、start_orchestration、route registration、schema、persistence owner、response mapping owner、audit owner、frontend caller 或发布过渡连接。

**幻觉检查点**:
AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已抽离时，必须说明本批只移动四个 lifecycle handler 到 `src/runtime/backtest/record_lifecycle.rs`，并通过父级 `pub(crate) use` 保持兼容出口。不得宣称 record lifecycle 已 closeout、`stop_split: true` 已设置、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已 closeout 时，必须说明只完成四个 lifecycle handler 的抽离与等价 closeout，并设置 `stop_split: true`；`runtime.backtest.experiment_sweep` 父叶已完成 BE-001AB-01 第三轮父叶残余判断，但 BE-001AD-01 已确认 `backend.runtime.routes` 父叶仍保持 `stop_split: false`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`，BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。不得宣称 route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

### 5.1.20 `runtime.report_ops`

**层级路径**: `root.backend.runtime.runtime.report_ops`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CH-01 第二轮父叶残余判断已完成，`runtime.report_ops stop_split: true`。下一步只能回到 BE-001CI-01 `backend.runtime` 父叶残余判断。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/report_ops.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`
- `src/runtime/report_ops/merge_generation_health.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/frontend_api_types.rs`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/components/RuntimeReportPanel.jsx`
- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_v1_reports.rs`
- `tests/api_v1_ops_health.rs`
- `markdown/06-milestones/v4.16.0/252-runtime.report_ops单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/253-runtime.report_ops抽离方案.md`
- `markdown/06-milestones/v4.16.0/254-runtime.report_ops抽离记录.md`
- `markdown/06-milestones/v4.16.0/255-runtime.report_ops单叶closeout.md`
- `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md`
- `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md`
- `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md`
- `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md`
- `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md`
- `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md`
- `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md`
- `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md`
- `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md`
- `markdown/06-milestones/v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md`

**职责**:
承载 runtime report create/list/detail/export 与 v1 merge records、runtime generations、storage health、ops daily、audit weekly、research monthly report handler 的白箱边界。当前不拥有 route registration、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_report` | `CreateRuntimeReportRequest` | `RuntimeEvidenceReportRecord` | `backend.runtime.routes.report_ops` | 不得改变 source metadata/idempotent 语义 |
| `list_runtime_reports` | `PaginationQuery` | `PaginatedResponse<RuntimeEvidenceReportRecord>` | `backend.runtime.routes.report_ops` | 不得改变 materialize、排序或分页语义 |
| `get_runtime_report_detail` | report id | `RuntimeEvidenceReportRecord` | `backend.runtime.routes.report_ops` | 不得跳过 source changed 检查 |
| `export_runtime_report_artifact` | report id | `RuntimeEvidenceReportArtifact` | `backend.runtime.routes.report_ops` | 不得私造 artifact schema |
| `list_merge_records` | user scope | `MergeRecordsResponse` | `backend.runtime.routes.report_ops` | 不得改变 merge event scan totals |
| `list_config_generations` | `AppState` generation | JSON | `backend.runtime.routes.report_ops` | 不得迁移 generation state owner |
| `get_storage_health` | storage dirs | JSON | `backend.runtime.routes.report_ops` | 不得迁移 storage lifecycle owner |
| `get_ops_daily_report` | `OpsDailyQuery` | `OpsDailyReport` | `backend.runtime.routes.report_ops` | 不得改变 metrics/alert/storage projection |
| `get_audit_weekly_report` | `AuditWeeklyQuery` | `AuditWeeklyReport` | `backend.runtime.routes.report_ops` | 不得改变 approval/proposal/mutation counts |
| `get_research_monthly_report` | `ResearchMonthlyQuery` | `ResearchMonthlyReport` | `backend.runtime.routes.report_ops` | 不得改变 strategy/capacity/cost projection |

**父级通信规则**:
`runtime.report_ops` 只能经父级 `backend.runtime` 暴露给 `backend.runtime.routes.report_ops` route facade。后续若实际抽离，只能由 `src/runtime/mod.rs` 增加受控 `mod report_ops` 与 `pub(crate) use report_ops::{...}` re-export；不得让 route facade、frontend caller、persistence owner、schema owner 或 storage lifecycle owner 横向直连子叶。

**允许迁移清单**:
BE-001CB-03 若被 BE-001CB-02 允许，第一轮只可迁移 `create_runtime_report`、`report_source_metadata_matches`、`source_changed_report`、`current_report_for_saved_source`、`materialize_runtime_report_record`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`。

**抽离记录**:
BE-001CB-03 已按方案新建 `src/runtime/report_ops.rs`，并迁入 `create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` 以及四个 report helper。父级 `src/runtime/mod.rs` 只保留 `mod report_ops` 与受控 `pub(crate) use report_ops::{...}`，route facade 未改。

**单叶 closeout**:
BE-001CB-04 已确认 `runtime.report_ops` 第一轮抽离等价成立，但该叶不是终叶。`src/runtime/report_ops.rs` 仍同时承载 runtime report lifecycle/materialization、v1 merge/generation/storage health、ops daily、audit weekly、research monthly 等职责，因此 `runtime.report_ops stop_split: false`。下一步只能进入 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线，不得直接创建 child 文件、迁移 handler、处理 v1 ops/report endpoints 或启动 release transition。

BE-001CC-04 已确认 child `runtime.report_ops.runtime_report` 等价成立并设置 `stop_split: true`。父级仍保留 `list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`，因此 `runtime.report_ops stop_split: false`；下一步只能进入 BE-001CD-01 父叶残余判断。

**父叶残余判断**:
BE-001CD-01 已确认 `runtime.report_ops` 父级仍具备继续细拆价值。下一候选固定为 `runtime.report_ops.v1_report_endpoints`，只覆盖 `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`；`list_merge_records`、`list_config_generations`、`get_storage_health` 后续再作为 `runtime.report_ops.merge_generation_health` 候选处理。

**单子叶等价基线**:
BE-001CE-01 已冻结 `runtime.report_ops.v1_report_endpoints` 的三个 `/api/v1/reports/*` handler、query/response、状态读取面、父级 re-export 和专门测试缺口。当前 `no code movement`，planned child 文件尚未创建。

**抽离方案**:
BE-001CE-02 已选择 test-first。下一批 BE-001CE-03 只补 `api_v1_reports` endpoint smoke，不创建 child module、不迁移 handler；BE-001CE-04 才允许实际抽离。

**endpoint smoke 补测**:
BE-001CE-03 已新增 `tests/api_v1_reports.rs`，覆盖 `/api/v1/reports/ops/daily`、`/api/v1/reports/audit/weekly`、`/api/v1/reports/research/monthly` 的最小 JSON contract。断言范围为 HTTP 200、`report_type`、`generated_at` 与 ops/audit/research 关键字段；child module 尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`。

**抽离记录**:
BE-001CE-04 已创建 `src/runtime/report_ops/v1_report_endpoints.rs`，并迁入 `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`。父级 `src/runtime/report_ops.rs` 只新增 `mod v1_report_endpoints` 与受控 `pub(crate) use v1_report_endpoints::{...}`；`src/runtime/mod.rs` 与 `src/backend/runtime/routes/report_ops.rs` 未改。

**单叶 closeout**:
BE-001CE-05 已确认 `runtime.report_ops.v1_report_endpoints` 作为 v1 report projection surface 已稳定，并设置 `runtime.report_ops.v1_report_endpoints stop_split: true`。继续拆成 ops/audit/research 微叶不会形成新的稳定 owner；父级 `runtime.report_ops stop_split: false`，下一步只能进入 BE-001CF-01 父叶残余判断。

**父叶残余判断**:
BE-001CF-01 已确认父级仍保留 `list_merge_records`、`list_config_generations`、`get_storage_health` 三个 public handler，因此 `runtime.report_ops stop_split: false`。下一候选固定为 `runtime.report_ops.merge_generation_health`，只能先进入 BE-001CG-01 单子叶等价基线；不得直接创建 child 文件或迁移 handler。

**单子叶等价基线**:
BE-001CG-01 已冻结 `runtime.report_ops.merge_generation_health` 的三条 v1 support/health endpoint、public handler、状态读取面、父级 re-export、测试缺口和禁止迁移边界。当前 `no code movement`，planned child 文件尚未创建，下一步只能进入 BE-001CG-02 抽离方案。

**抽离方案**:
BE-001CG-02 已选择 test-first。下一批 BE-001CG-03 只补计划测试文件 `api_v1_ops_health` endpoint smoke，不创建 child module、不迁移 handler；BE-001CG-04 才允许创建计划 child 文件 `merge_generation_health` 并迁移 `list_merge_records`、`list_config_generations`、`get_storage_health`。

**endpoint smoke 补测**:
BE-001CG-03 已新增 `tests/api_v1_ops_health.rs`，覆盖 `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health` 的最小 JSON contract。断言范围为 HTTP 200、`records`、`total_conflicts`、`total_suppressed`、`current_generation`、`history`、`total_storage_mb`、`layers`、`hot_layer_usage_ratio`、`disk_watermark_ratio`、`archive_enabled` 与 `runs` layer；child module 尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`。下一步只能进入 BE-001CG-04 实际抽离。

**抽离记录**:
BE-001CG-04 已创建 `src/runtime/report_ops/merge_generation_health.rs`，并迁入 `list_merge_records`、`list_config_generations`、`get_storage_health`。父级 `src/runtime/report_ops.rs` 只新增 `mod merge_generation_health` 与受控 `pub(crate) use merge_generation_health::{get_storage_health, list_config_generations, list_merge_records};`；`src/runtime/mod.rs` 与 `src/backend/runtime/routes/report_ops.rs` 未改。下一步只能进入 BE-001CG-05 单叶 closeout。

**单叶 closeout**:
BE-001CG-05 已确认 `runtime.report_ops.merge_generation_health` 作为 v1 support/health projection surface 已稳定，并设置 `runtime.report_ops.merge_generation_health stop_split: true`。继续拆成 merge_records/config_generations/storage_health 微叶不会形成新的稳定 owner；下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。

**第二轮父叶残余判断**:
BE-001CH-01 已确认 `runtime.report_ops.runtime_report`、`runtime.report_ops.v1_report_endpoints` 与 `runtime.report_ops.merge_generation_health` 三个 child 均已 closeout。父级 `src/runtime/report_ops.rs` 已不直接持有 public handler，只承担 child module 声明与受控 re-export，因此设置 `runtime.report_ops stop_split: true`。下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断。

**明确排除**:
`get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts`、query structs/shared helper、`AppState`、`runtime_persistence`、`runtime_response_mapping`、`frontend_api_types`、frontend caller、storage lifecycle owner 和 release transition guard 均不属于本叶第一轮迁移。`runtime.evidence_health` 应作为 sibling 另起基线。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test --no-run`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_evidence_contract`；`cargo test -p quantpilot --test api_v1_reports`；`cargo test -p quantpilot --test api_v1_ops_health`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`；`git diff --check`。

**幻觉检查点**:
AI 声称 `runtime.report_ops` 已完成 BE-001CC-04 时，必须说明 `runtime.report_ops.runtime_report` 已完成 `no code movement` closeout 并设置 `stop_split: true`，父级 `runtime.report_ops stop_split: false`，下一步只能进入 BE-001CD-01 父叶残余判断，v1 ops/report endpoints 仍有测试缺口，且 `runtime.evidence_health` 未并入本叶。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CD-01 时，必须说明本批次是 `no code movement` 父叶残余判断，`runtime.report_ops stop_split: false`，下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线，v1 ops/report endpoints 测试缺口仍需显式继承，且 `runtime.evidence_health` 未并入本叶。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CE-01 时，必须说明本批次是 `no code movement` 等价基线，planned child 文件尚未创建，三个 v1 report handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CE-02 抽离方案。不得宣称测试缺口已补齐或发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CE-02 时，必须说明本批次是 `no code movement` 抽离方案，方案选择 test-first，下一步只能进入 BE-001CE-03 endpoint smoke 补测，child module 尚未创建且 handler 未迁移。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CE-03 时，必须说明本批次只新增 endpoint smoke 测试，`tests/api_v1_reports.rs` 已覆盖三条 `/api/v1/reports/*` 基础 JSON contract，child module 尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CE-04 实际抽离。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CE-04 时，必须说明 `src/runtime/report_ops/v1_report_endpoints.rs` 已创建并迁入三个 v1 report handler，父级只保留受控 re-export，`src/runtime/mod.rs` 和 route facade 未改变，下一步只能进入 BE-001CE-05 单叶 closeout。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CE-05 时，必须说明本批次是 `no code movement` closeout，`runtime.report_ops.v1_report_endpoints stop_split: true`，父级 `runtime.report_ops stop_split: false`，下一步只能进入 BE-001CF-01 父叶残余判断。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CF-01 时，必须说明本批次是 `no code movement` 父叶残余判断，`runtime_report` 与 `v1_report_endpoints` 两个 child 均已 closeout，父级仍保留 `list_merge_records`、`list_config_generations`、`get_storage_health`，`runtime.report_ops stop_split: false`，下一步只能进入 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CG-01 时，必须说明本批次是 `no code movement` 单子叶等价基线，planned child 文件尚未创建，`list_merge_records`、`list_config_generations`、`get_storage_health` 仍在 `src/runtime/report_ops.rs`，三条 endpoint 专门自动化 smoke 缺口已登记，下一步只能进入 BE-001CG-02 抽离方案。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CG-02 时，必须说明本批次是 `no code movement` test-first 抽离方案，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CG-03 endpoint smoke 补测，BE-001CG-04 才允许实际迁移。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CG-03 时，必须说明本批次只新增 endpoint smoke 测试，`tests/api_v1_ops_health.rs` 已覆盖三条 v1 support/health endpoint 的基础 JSON contract，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CG-04 实际抽离。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CG-04 时，必须说明 `src/runtime/report_ops/merge_generation_health.rs` 已创建，三个 v1 support/health handler 已迁入 child，父级只保留受控 re-export，`src/runtime/mod.rs` 与 route facade 未改，下一步只能进入 BE-001CG-05 单叶 closeout。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CG-05 时，必须说明本批次是 `no code movement` closeout，`runtime.report_ops.merge_generation_health stop_split: true`，三个 report_ops child 均已 closeout，但父级还必须进入 BE-001CH-01 残余判断后才能宣称父叶收口。不得宣称发布过渡已启动。

AI 声称 `runtime.report_ops` 已完成 BE-001CH-01 时，必须说明本批次是 `no code movement` 父叶残余判断，三个 report_ops child 均已 closeout，父级只保留 child module 声明与受控 re-export，`runtime.report_ops stop_split: true`，下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断。不得宣称发布过渡已启动。

### 5.1.20.1 `runtime.report_ops.runtime_report`

**层级路径**: `root.backend.runtime.runtime.report_ops.runtime_report`
**父模块**: `runtime.report_ops`
**状态**: v4.16 BE-001CC-04 单叶 closeout 已完成。`src/runtime/report_ops/runtime_report.rs` 已创建并承接四个 public handler 与四个 private helper；父级 `src/runtime/report_ops.rs` 通过 `mod runtime_report` 与受控 `pub(crate) use runtime_report::{...}` 保持兼容出口。`runtime.report_ops.runtime_report stop_split: true`。
**真实文件**:
- `src/runtime/report_ops.rs`
- `src/runtime/report_ops/runtime_report.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_mutation.rs`
- `frontend/src/components/RuntimeReportPanel.test.jsx`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md`
- `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md`
- `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md`

**职责**:
- `create_runtime_report`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`
- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`

**白箱输入**:
- `auth::UserId`
- `State<AppState>`
- `Json<CreateRuntimeReportRequest>`
- `Query<PaginationQuery>`
- `Path<String>`

**白箱输出**:
- `Json<RuntimeEvidenceReportRecord>`
- `Json<PaginatedResponse<RuntimeEvidenceReportRecord>>`
- `Json<RuntimeEvidenceReportArtifact>`
- `(StatusCode, String)` error tuple

**父级通信规则**:
`runtime.report_ops.runtime_report` 后续只能经 `runtime.report_ops` 父级暴露给 `src/runtime/mod.rs`，再由 `backend.runtime.routes.report_ops` route facade 调用。开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner 或 storage lifecycle owner 横向直连该子叶。

**允许迁移清单**:
BE-001CC-02 只能规划 `create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`report_source_metadata_matches`、`source_changed_report`、`current_report_for_saved_source`、`materialize_runtime_report_record` 迁入 runtime_report planned child 文件。

**抽离方案**:
BE-001CC-02 已固定 BE-001CC-03 的最小迁移方式: 只新建 runtime_report planned child 文件，将四个 runtime report public handler 与四个 private helper 迁入 child，并由 `src/runtime/report_ops.rs` 新增受控 `mod runtime_report` 与 `pub(crate) use runtime_report::{...}`。`src/runtime/mod.rs` 的既有 report_ops re-export 清单、route facade、v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、state/persistence owner、storage lifecycle owner、`AppState` 与 release transition guard 均不得迁移。

**抽离记录**:
BE-001CC-03 已按方案创建 `src/runtime/report_ops/runtime_report.rs`，并迁入 `create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`report_source_metadata_matches`、`source_changed_report`、`current_report_for_saved_source`、`materialize_runtime_report_record`。父级 `src/runtime/report_ops.rs` 只新增 `mod runtime_report` 与受控 `pub(crate) use runtime_report::{...}`，`src/runtime/mod.rs` 和 route facade 未改。

**单叶 closeout**:
BE-001CC-04 已确认本子叶等价成立并设置 `runtime.report_ops.runtime_report stop_split: true`。`create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact` 与四个 materialization/source helper 属于同一 runtime report lifecycle 白箱，继续拆成 create/read/export/helper 微叶不会形成新的稳定 owner。

**明确排除**:
`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`、v1 ops/report endpoints、`get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts`、`runtime.evidence_health`、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 与 release transition guard 均不属于本子叶。

**幻觉检查点**:
AI 声称 `runtime.report_ops.runtime_report` 已完成 BE-001CC-04 时，必须说明本批次是 `no code movement` closeout，`runtime.report_ops.runtime_report stop_split: true`，父级 `runtime.report_ops stop_split: false` 且下一步只能进入 BE-001CD-01 父叶残余判断。不得宣称 v1 ops/report endpoints、`runtime.evidence_health` 或 release transition 已处理。

### 5.1.20.2 `runtime.report_ops.v1_report_endpoints`

**层级路径**: `root.backend.runtime.runtime.report_ops.v1_report_endpoints`
**父模块**: `runtime.report_ops`
**状态**: v4.16 BE-001CE-05 单叶 closeout 已完成。`src/runtime/report_ops/v1_report_endpoints.rs` 承接三个目标 handler，`runtime.report_ops.v1_report_endpoints stop_split: true`；上层已进入 BE-001CF-01 父叶残余判断。
**真实文件**:
- `src/runtime/report_ops.rs`
- `src/runtime/report_ops/runtime_report.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/frontend_api_types.rs`
- `tests/api_v1_reports.rs`
- `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md`
- `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md`
- `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md`
- `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md`

**职责**:
冻结 `/api/v1/reports/*` 三个 v1 report projection handler 的白箱边界，不接管 runtime report lifecycle、merge/generation/storage health endpoints、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

**关键 public 方法**:
| 方法 | Endpoint | 输入 | 输出 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_ops_daily_report` | `/api/v1/reports/ops/daily` | `auth::UserId`、`AppState`、`OpsDailyQuery` | `OpsDailyReport` | 不得改变 runs/alerts/evidence metrics projection |
| `get_audit_weekly_report` | `/api/v1/reports/audit/weekly` | `auth::UserId`、`AppState`、`AuditWeeklyQuery` | `AuditWeeklyReport` | 不得改变 approval/proposal/mutation/hotswap counts |
| `get_research_monthly_report` | `/api/v1/reports/research/monthly` | `auth::UserId`、`AppState`、`ResearchMonthlyQuery` | `ResearchMonthlyReport` | 不得改变 backtest performance 与 proposal effectiveness projection |

**父级通信规则**:
`runtime.report_ops.v1_report_endpoints` 后续只能经 `runtime.report_ops` 父级暴露给 `src/runtime/mod.rs`，再由 `backend.runtime.routes.report_ops` route facade 调用。开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner 或 storage lifecycle owner 横向直连该子叶。

**允许迁移清单**:
BE-001CE-04 已由 test-first smoke 放行并只迁移 `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`。父级仅增加 `mod v1_report_endpoints` 与受控 `pub(crate) use v1_report_endpoints::{...}`。

**抽离方案**:
BE-001CE-02 已把下一步改为 test-first。BE-001CE-03 只允许新增 `api_v1_reports` endpoint smoke，用于覆盖 `/api/v1/reports/ops/daily`、`/api/v1/reports/audit/weekly`、`/api/v1/reports/research/monthly` 的基础 JSON contract；BE-001CE-04 才允许迁移 handler。

**endpoint smoke 补测**:
BE-001CE-03 已新增 `tests/api_v1_reports.rs`。该测试通过 `common::test_app("api_v1_reports_smoke")` 访问三条 v1 report endpoint，并断言 `report_type`、`generated_at`、`summary`、`data_health`、`runtime_health`、`total_approvals`、`notable_incidents`、`strategy_performance`、`ai_proposal_effectiveness` 等最小 contract。该批次未创建 child module、未迁移 handler。

**抽离记录**:
BE-001CE-04 已创建 `src/runtime/report_ops/v1_report_endpoints.rs`，并将 `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` 迁入 child。父级 `src/runtime/report_ops.rs` 只保留 `mod v1_report_endpoints` 与受控 `pub(crate) use v1_report_endpoints::{...}`，`src/runtime/mod.rs` 和 route facade 未改。

**单叶 closeout**:
BE-001CE-05 已确认本叶不继续拆分，并设置 `runtime.report_ops.v1_report_endpoints stop_split: true`。三个 handler 共同服务 `/api/v1/reports/*` projection surface，调用方与父级出口一致，继续拆成 ops/audit/research 微叶会增加治理登记成本但不会产生更稳定的 owner。

**明确排除**:
`create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`runtime.report_ops.runtime_report`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`runtime.report_ops.merge_generation_health`、`get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts`、`runtime.evidence_health`、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 与 release transition guard 均不属于本子叶。

**测试缺口**:
BE-001CE-01 已显式冻结 `/api/v1/reports/ops/daily`、`/api/v1/reports/audit/weekly`、`/api/v1/reports/research/monthly` 专门测试缺口。BE-001CE-02 必须决定先补最小 endpoint smoke，或先纯物理抽离并继承 broad regression 风险。

BE-001CE-03 已用 `tests/api_v1_reports.rs` 补齐最小 endpoint smoke；BE-001CE-04 实际抽离已继续跑 `cargo test -p quantpilot --test api_v1_reports`。

**幻觉检查点**:
AI 声称 `runtime.report_ops.v1_report_endpoints` 已完成 BE-001CE-01 时，必须说明当前 `no code movement`，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，测试缺口尚未补齐，下一步只能进入 BE-001CE-02 抽离方案。不得宣称 merge/generation/storage health、`runtime.evidence_health` 或 release transition 已处理。

AI 声称 `runtime.report_ops.v1_report_endpoints` 已完成 BE-001CE-02 时，必须说明当前 `no code movement`，方案选择 test-first，下一步只能进入 BE-001CE-03 endpoint smoke 补测。不得宣称 child module 已创建、handler 已迁移或测试缺口已完全收口。

AI 声称 `runtime.report_ops.v1_report_endpoints` 已完成 BE-001CE-03 时，必须说明本批次只新增 endpoint smoke 测试，`tests/api_v1_reports.rs` 已创建并覆盖三条 `/api/v1/reports/*` endpoint 的基础 JSON contract；child module 尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CE-04 实际抽离。不得宣称 merge/generation/storage health、`runtime.evidence_health` 或 release transition 已处理。

AI 声称 `runtime.report_ops.v1_report_endpoints` 已完成 BE-001CE-04 时，必须说明 `src/runtime/report_ops/v1_report_endpoints.rs` 已创建，三个 handler 已迁入 child，父级只保留受控 re-export，`src/runtime/mod.rs` 与 route facade 未改，下一步只能进入 BE-001CE-05 单叶 closeout。不得宣称 merge/generation/storage health、`runtime.evidence_health` 或 release transition 已处理。

AI 声称 `runtime.report_ops.v1_report_endpoints` 已完成 BE-001CE-05 时，必须说明本批次是 `no code movement` closeout，`runtime.report_ops.v1_report_endpoints stop_split: true`，下一步只能进入 BE-001CF-01 `runtime.report_ops` 父叶残余判断。不得继续从本叶细拆 ops/audit/research 微叶，不得宣称 release transition 已处理。

### 5.1.20.3 `runtime.report_ops.merge_generation_health`

**层级路径**: `root.backend.runtime.runtime.report_ops.merge_generation_health`
**父模块**: `runtime.report_ops`
**状态**: v4.16 BE-001CG-05 单叶 closeout 已完成，`runtime.report_ops.merge_generation_health stop_split: true`。下一步只能回到父级进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。
**真实文件**:
- `src/runtime/report_ops.rs`
- `src/runtime/report_ops/merge_generation_health.rs`
- `src/runtime/report_ops/runtime_report.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/storage_lifecycle.rs`
- `tests/api_v1_ops_health.rs`
- `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md`
- `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md`

**职责**:
冻结 v1 support/health endpoints 的白箱边界，只覆盖 merge records、runtime generation history 和 storage health projection。当前不接管 runtime report lifecycle、v1 `/api/v1/reports/*` endpoints、runtime evidence health、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、run state owner、config generation owner、`AppState` 或 release transition guard。

**关键 public 方法**:
| 方法 | Endpoint | 输入 | 状态读取 / 依赖 | 输出 | 禁止事项 |
| --- | --- | --- | --- | --- | --- |
| `list_merge_records` | `/api/v1/merge/records` | `auth::UserId`、`State<AppState>` | `auth::scoped_key`、`state.runs.read()`、`merge_engine` event payload | `MergeRecordsResponse` | 不得迁移 run state owner 或 event schema owner |
| `list_config_generations` | `/api/v1/runtime/generations` | `State<AppState>` | `state.config_generation`、`state.config_generation_history` | JSON generation history | 不得迁移 config generation owner 或锁顺序 |
| `get_storage_health` | `/api/v1/storage/health` | `State<AppState>` | store dirs、`storage_lifecycle::dir_size_bytes` | JSON storage health | 不得迁移 storage lifecycle owner 或目录 owner |

**父级通信规则**:
`runtime.report_ops.merge_generation_health` 后续只能经 `runtime.report_ops` 父级暴露给 `src/runtime/mod.rs`，再由 `backend.runtime.routes.report_ops` route facade 调用。开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、run state owner、config generation owner 或 `AppState` 横向直连该子叶。

**允许迁移清单**:
BE-001CG-01 不允许迁移。BE-001CG-02 已选择 test-first，仍不允许迁移。BE-001CG-03 只允许新增计划测试文件 `api_v1_ops_health`，不创建 child module、不迁移 handler。BE-001CG-04 才允许迁移 `list_merge_records`、`list_config_generations`、`get_storage_health`。

**测试缺口**:
BE-001CG-03 已用 `tests/api_v1_ops_health.rs` 补齐 `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health` 的专门自动化 smoke。该测试通过 `common::test_app("api_v1_ops_health_smoke")` 访问三条 endpoint，并断言 `records`、`total_conflicts`、`total_suppressed`、`current_generation`、`history`、`total_storage_mb`、`layers`、`hot_layer_usage_ratio`、`disk_watermark_ratio`、`archive_enabled` 与 `runs` layer。该批次未创建 child module、未迁移 handler。

**抽离方案**:
BE-001CG-02 已固定实际抽离目标为计划 child 文件 `merge_generation_health`。BE-001CG-04 只能在 BE-001CG-03 通过后创建该文件，并由 `src/runtime/report_ops.rs` 增加 `mod merge_generation_health` 与受控 `pub(crate) use merge_generation_health::{get_storage_health, list_config_generations, list_merge_records};`。`src/runtime/mod.rs` 与 route facade 保持不变。

**抽离记录**:
BE-001CG-04 已创建 `src/runtime/report_ops/merge_generation_health.rs`，并迁入 `list_merge_records`、`list_config_generations`、`get_storage_health`。父级 `src/runtime/report_ops.rs` 只保留 `mod merge_generation_health` 与受控 `pub(crate) use merge_generation_health::{get_storage_health, list_config_generations, list_merge_records};`；`src/runtime/mod.rs`、route facade、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 与 release transition guard 均未迁移。

**单叶 closeout**:
BE-001CG-05 已确认本叶内部不再继续细拆。`list_merge_records`、`list_config_generations`、`get_storage_health` 共同构成 v1 support/health projection surface；继续拆成 merge_records/config_generations/storage_health 微叶只会增加 re-export 与治理登记成本，不会形成独立状态机、持久化 owner、schema owner 或 release transition guard。因此设置 `runtime.report_ops.merge_generation_health stop_split: true`。

**明确排除**:
`runtime.report_ops.runtime_report`、`runtime.report_ops.v1_report_endpoints`、`create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`、`get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts`、`runtime.evidence_health`、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 与 release transition guard 均不属于本子叶。

**幻觉检查点**:
AI 声称 `runtime.report_ops.merge_generation_health` 已完成 BE-001CG-01 时，必须说明当前 `no code movement`，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，三条 endpoint 自动化 smoke 缺口已登记，下一步只能进入 BE-001CG-02 抽离方案。不得宣称 handler 已迁移或发布过渡已启动。

AI 声称 `runtime.report_ops.merge_generation_health` 已完成 BE-001CG-02 时，必须说明当前 `no code movement`，方案选择 test-first，planned child 文件尚未创建，下一步只能进入 BE-001CG-03 endpoint smoke 补测。不得宣称 handler 已迁移或发布过渡已启动。

AI 声称 `runtime.report_ops.merge_generation_health` 已完成 BE-001CG-03 时，必须说明本批次只新增 endpoint smoke 测试，`tests/api_v1_ops_health.rs` 已覆盖三条 v1 support/health endpoint 的基础 JSON contract，planned child 文件尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`，下一步只能进入 BE-001CG-04 实际抽离。不得宣称 handler 已迁移或发布过渡已启动。

AI 声称 `runtime.report_ops.merge_generation_health` 已完成 BE-001CG-04 时，必须说明 `src/runtime/report_ops/merge_generation_health.rs` 已创建，`list_merge_records`、`list_config_generations`、`get_storage_health` 已从 `src/runtime/report_ops.rs` 迁入 child，父级只保留受控 re-export，`src/runtime/mod.rs` 与 route facade 未改，下一步只能进入 BE-001CG-05 单叶 closeout。不得宣称 `runtime.evidence_health`、handler 之外 owner 或发布过渡已处理。

AI 声称 `runtime.report_ops.merge_generation_health` 已完成 BE-001CG-05 时，必须说明本批次是 `no code movement` closeout，`runtime.report_ops.merge_generation_health stop_split: true`，三个 v1 support/health handler 仍在 `src/runtime/report_ops/merge_generation_health.rs`，下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。不得宣称 `runtime.evidence_health`、handler 之外 owner 或发布过渡已处理。

### 5.1.21 `runtime.evidence_health`

**层级路径**: `root.backend.runtime.runtime.evidence_health`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CJ-04 单叶 closeout 已完成，`runtime.evidence_health stop_split: true`。下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/evidence_health.rs`
- `src/backend/runtime/routes/evidence.rs`
- `src/frontend_api_types.rs`
- `src/runtime_persistence.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_mutation.rs`
- `markdown/06-milestones/v4.16.0/273-backend.runtime第二轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/274-runtime.evidence_health单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/275-runtime.evidence_health抽离方案.md`
- `markdown/06-milestones/v4.16.0/276-runtime.evidence_health抽离记录.md`
- `markdown/06-milestones/v4.16.0/277-runtime.evidence_health单叶closeout.md`

**职责**:
冻结 runtime evidence health / cleanup handler 的白箱边界，只覆盖 `/api/runtime/evidence/health` 与 `/api/runtime/evidence/cleanup` 的 handler 逻辑、report status count helper、metrics snapshot、cleanup policy projection 和 transient cleanup response。当前不拥有 route registration、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、metrics owner、`AppState` 或 release transition guard。

**路由与 handler 基线**:
| route | method | handler | 当前 handler 文件 | route facade |
| --- | --- | --- | --- | --- |
| `/api/runtime/evidence/health` | GET | `get_runtime_evidence_health` | `src/runtime/evidence_health.rs` | `src/backend/runtime/routes/evidence.rs` |
| `/api/runtime/evidence/cleanup` | POST | `cleanup_runtime_evidence` | `src/runtime/evidence_health.rs` | `src/backend/runtime/routes/evidence.rs` |

**关键 public / helper 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_runtime_evidence_health` | `State<AppState>` | `RuntimeEvidenceHealthResponse` | `backend.runtime.routes.evidence` | 不得迁移 report store、metrics、cleanup policy 或 schema owner |
| `cleanup_runtime_evidence` | `State<AppState>`、`RuntimeEvidenceCleanupRequest` | `RuntimeEvidenceCleanupResponse` | `backend.runtime.routes.evidence` | 不得迁移 transient cleanup implementation、clock helper 或 persistence owner |
| `runtime_report_status_counts` | `&[RuntimeEvidenceReportRecord]` | `RuntimeEvidenceReportStatusCounts` | `get_runtime_evidence_health` | 不得改变 status mapping 或 report lifecycle enum owner |

**父级通信规则**:
后续若实际抽离，只能由 `src/runtime/mod.rs` 通过受控 re-export 暴露给 `backend.runtime.routes.evidence`。开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、metrics owner 或 `AppState` 横向直连该子叶。

**现有等价证据**:
`tests/api_evidence_contract.rs::runtime_evidence_health_tracks_metrics_and_cleanup_preserves_reports` 覆盖 health/cleanup；`tests/api_evidence_contract.rs::runtime_evidence_contract_snapshot_matches_fixture` 覆盖 evidence contract snapshot；`tests/api_mutation.rs` 对 health endpoint 提供 mutation metrics 联动回归。

**抽离方案**:
BE-001CJ-02 已固定 BE-001CJ-03 的最小迁移方式: 只落地 `evidence_health` child，并迁移 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence` 三个函数。父级 `src/runtime/mod.rs` 只增加 child module 声明与受控 re-export；route facade、schema owner、runtime persistence owner、metrics owner、`AppState` 和 release transition guard 均不得迁移。

**抽离记录**:
BE-001CJ-03 已创建 `src/runtime/evidence_health.rs`，并将 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence` 从 `src/runtime/mod.rs` 迁入 child。父级 `src/runtime/mod.rs` 只保留 `mod evidence_health` 与受控 `pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};`；`src/backend/runtime/routes/evidence.rs`、schema owner、runtime persistence owner、metrics owner、`AppState` 和 release transition guard 均未迁移。

**单叶 closeout**:
BE-001CJ-04 已确认本叶内部不再继续细拆。health 与 cleanup 共享 evidence support surface、`RuntimeEvidence*` schema、report store 读取、cleanup policy 与 persistence helper 边界；继续拆成 health / cleanup 微叶不会形成独立状态机、schema owner、runtime persistence owner、metrics owner 或 release transition guard。因此设置 `runtime.evidence_health stop_split: true`。

**明确排除**:
`backend.runtime.routes.evidence`、`runtime.report_ops`、`RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse`、`runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`list_runtime_report_records`、`current_time_ms`、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、metrics owner、shared helpers 和 release transition guard 均不属于本批迁移。

**下一步**:
BE-001CK-01 必须回到 `backend.runtime` 父叶，判断 `runtime.report_ops` 与 `runtime.evidence_health` 均 closeout 后是否仍存在其他值得继续抽离的 handler / helper 残余。

**幻觉检查点**:
AI 声称 `runtime.evidence_health` 已完成 BE-001CJ-01 时，必须说明当前是 `no code movement` 单子叶等价基线，planned child 文件尚未创建，两个 public handler 与 `runtime_report_status_counts` 仍在 `src/runtime/mod.rs`，下一步只能进入 BE-001CJ-02 抽离方案。不得宣称 handler 已抽离、schema/runtime persistence/metrics owner 已迁移、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.evidence_health` 已完成 BE-001CJ-02 时，必须说明当前仍是 `no code movement` 抽离方案，允许迁移清单只有 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence`，下一步只能进入 BE-001CJ-03 实际抽离。不得宣称 child 已落地或 handler 已迁移。

AI 声称 `runtime.evidence_health` 已完成 BE-001CJ-03 时，必须说明 `src/runtime/evidence_health.rs` 已创建，两个 public handler 与 `runtime_report_status_counts` 已迁入 child，父级只保留 `mod evidence_health` 与受控 re-export，下一步只能进入 BE-001CJ-04 单叶 closeout。不得宣称本叶已 closeout、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.evidence_health` 已完成 BE-001CJ-04 时，必须说明本批次是 `no code movement` closeout，`runtime.evidence_health stop_split: true`，不继续拆 health / cleanup 微叶，下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。不得宣称发布过渡已启动或 Rust 重构完成。

AI 声称 `backend.runtime` 已完成 BE-001CK-01 时，必须说明本批次是 `no code movement` 父叶残余判断，`backend.runtime stop_split: false`，下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。不得宣称 parent support 已整体抽离、planned child 文件已创建、helper 已迁移、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.mutation.shared_governance` 已完成 BE-001CL-01 时，必须说明本批次是 `no code movement` 单子叶等价基线，9 个 shared governance helper 仍在 src/runtime/mutation.rs (retired drained include)，下一步只能进入 BE-001CL-02 抽离方案。不得宣称 helper 已迁移、query DTO/run guard 已处理、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.mutation.shared_governance` 已完成 BE-001CL-02 时，必须说明本批次是 `no code movement` 抽离方案，planned child 文件尚未创建，9 个 shared governance helper 仍在 src/runtime/mutation.rs (retired drained include)，下一步只能进入 BE-001CL-03 实际抽离。不得宣称 helper 已迁移、query DTO/run guard 已处理、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.mutation.shared_governance` 已完成 BE-001CL-03 时，必须说明 `src/runtime/mutation/shared_governance.rs` 已创建，9 个 shared governance helper 已迁入 child，父级只保留 caller-facing plain import，下一步只能进入 BE-001CL-04 单叶 closeout。不得宣称 query DTO/run guard 已处理、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.query_support` 已完成 BE-001CN-03 时，必须说明 `src/runtime/query_support.rs` 已创建，7 个 Query DTO、`clean_optional_filter` 与 `normalized_replay_options` 已迁入 child，DTO 类型本体保持 `pub(crate)`，字段/helper 为 `pub(super)`，父级只保留 plain import，下一步只能进入 BE-001CN-04 单叶 closeout。不得宣称 response support、run guard、parent include 删除、发布过渡已启动或 Rust 重构完成。

### 5.1.22 `runtime.mutation.shared_governance`

**层级路径**: `root.backend.runtime.runtime.mutation.shared_governance`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CL-03 实际抽离已完成。`src/runtime/mutation/shared_governance.rs` 已创建并迁入 9 个 shared governance helper；下一步只能进入 BE-001CL-04 单叶 closeout。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/mutation/shared_governance.rs`
- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`
- `tests/api_mutation.rs`
- `tests/api_ai_proposal.rs`
- `markdown/06-milestones/v4.16.0/279-runtime.mutation.shared_governance单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/280-runtime.mutation.shared_governance抽离方案.md`
- `markdown/06-milestones/v4.16.0/281-runtime.mutation.shared_governance抽离记录.md`

**职责**:
冻结 runtime mutation shared governance helper 的白箱边界，只覆盖 parameter mutation 与 AI proposal 共享的 target validation、canonical parameter version、event contract、event append 和 governance projection。当前不拥有 query DTO、run guard、response support、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

**关键 public / helper 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `canonical_runtime_parameter_version` | mutation target、request payload | canonical parameter version | parameter mutation / AI proposal create path | 不得改变 digest input 或 prefix |
| `validate_runtime_parameter_mutation_target` | mutation target | validation result | parameter mutation / AI proposal create path | 不得放宽 module key、node id、path 或 capability gate |
| `runtime_mode_from_events` | runtime events | runtime mode | event append helper | 不得改变 default mode |
| `status_contract_value` | mutation status | status contract string | event builder | 不得改 contract spelling |
| `mutation_event_contract` | mutation status | event type / reason code | event builder、transition persistence | 不得改变 event type 或 reason code |
| `build_runtime_parameter_mutation_event` | mutation record、status、event time | frontend runtime event | mutation lifecycle callers | 不得改变 payload fields 或 severity mapping |
| `append_parameter_mutation_events_to_run` | state、user、run id、events、optional parameter version | persisted / in-memory event append result | mutation lifecycle callers | 不得改变 sequence、mode、persistence condition 或 lock owner |
| `runtime_parameter_mutation_governance` | source governance、old/proposed version | mutation governance | proposal / rollback callers | 不得改变 governance mapping |
| `governance_with_parameter_version` | governance snapshot、parameter version | updated governance snapshot | proposal / activation / rollback callers | 不得改变其他 governance fields |

**父级通信规则**:
BE-001CL-03 只能由 `src/runtime/mod.rs` 创建 child 声明并用 plain parent import 回填 helper surface；调用方继续通过 `use super::*` 访问父级受控 helper。开发者未明确进入发布版本过渡前，不得让 parameter mutation child、AI proposal child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、`AppState` 或任何 sibling 横向直连该 planned child。

**抽离方案**:
BE-001CL-02 已固定 BE-001CL-03 的最小迁移方式: 只创建 planned child 文件，只迁移 9 个 shared governance helper，helper visibility 固定为 `pub(super)`，父级只补 child 声明与 plain import。`include!("mutation.rs")` 暂时保留，因为 `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery` 与后续 query/guard/response support 残余需要另起父叶判断。

**抽离记录**:
BE-001CL-03 已创建 `src/runtime/mutation/shared_governance.rs`，并将 `canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`runtime_mode_from_events`、`status_contract_value`、`mutation_event_contract`、`build_runtime_parameter_mutation_event`、`append_parameter_mutation_events_to_run`、`runtime_parameter_mutation_governance`、`governance_with_parameter_version` 从 src/runtime/mutation.rs (retired drained include) 迁入 child。`src/runtime/mod.rs` 只保留 `#[path = "mutation/shared_governance.rs"] mod mutation_shared_governance;` 与 caller-facing plain import；`runtime_mode_from_events`、`status_contract_value` 留作 child-internal helper，不回填父级 import，以保持 warning-free。

**明确排除**:
`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`RunInProgressGuard`、`DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均不属于本子叶。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称 `runtime.mutation.shared_governance` 已完成 BE-001CL-02 时，必须说明本批次是 `no code movement` 抽离方案，planned child 文件尚未创建，9 个 helper 仍在 src/runtime/mutation.rs (retired drained include)，下一步只能进入 BE-001CL-03 实际抽离。不得宣称 helper 已迁移、`backend.runtime` 已完成、query DTO/run guard 已处理、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.mutation.shared_governance` 已完成 BE-001CL-03 时，必须说明 `src/runtime/mutation/shared_governance.rs` 已创建，9 个 helper 已迁入 child，src/runtime/mutation.rs (retired drained include) 仍保留 `OpsDailyQuery`、`AuditWeeklyQuery` 与 `ResearchMonthlyQuery`，下一步只能进入 BE-001CL-04 单叶 closeout。不得宣称 `backend.runtime` 已完成、query DTO/run guard 已处理、发布过渡已启动或 Rust 重构完成。

### 5.1.23 `runtime.query_support`

**层级路径**: `root.backend.runtime.runtime.query_support`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CN-04 单叶 closeout 已完成。`runtime.query_support stop_split: true`；下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/query_support.rs`
- `src/runtime/run/replay_status.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`
- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_mutation.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_v1_reports.rs`
- `markdown/06-milestones/v4.16.0/284-runtime.query_support单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/285-runtime.query_support抽离方案.md`
- `markdown/06-milestones/v4.16.0/286-runtime.query_support抽离记录.md`
- `markdown/06-milestones/v4.16.0/287-runtime.query_support单叶closeout.md`

**职责**:
冻结 runtime Query DTO、filter normalization 与 replay option normalization 的白箱边界。当前不拥有 response support、run guard、experiment limit、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

**关键 public / helper 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `RuntimeReplayQuery` | HTTP query | replay query DTO | run/backtest replay handlers | 不得改变 cursor / checkpoint precedence、limit、filters 或 key_only |
| `RuntimeParameterMutationListQuery` | HTTP query | parameter mutation list DTO | parameter mutation list handler | 不得改变 source / pagination semantics |
| `RuntimeAiProposalListQuery` | HTTP query | AI proposal list DTO | AI proposal list handler | 不得改变 source / status filter semantics |
| `RuntimeApprovalListQuery` | HTTP query | approval list DTO | approval list handler | 不得改变 `review_state` default 或 filter semantics |
| `OpsDailyQuery` | HTTP query | ops daily DTO | ops daily report handler | 不得改变 optional date behavior |
| `AuditWeeklyQuery` | HTTP query | audit weekly DTO | audit weekly report handler | 不得改变 optional week_start behavior |
| `ResearchMonthlyQuery` | HTTP query | research monthly DTO | research monthly report handler | 不得改变 optional month behavior |
| `clean_optional_filter` | optional string | optional trimmed string | mutation/proposal query filters、replay filters | 不得改变 trim / empty-filter behavior |
| `normalized_replay_options` | `RuntimeReplayQuery` | `RuntimeReplayOptions` | run/backtest replay handlers | 不得改变 default page size、max page size、cursor precedence、sequence cursor 或 filters |

**父级通信规则**:
`runtime.query_support` 只能经 `src/runtime/mod.rs` 受控 query surface 供 runtime child callers 使用。父级只允许 `mod query_support;` 与普通 `use query_support::{...};`，不得使用 `pub(crate) use query_support::{...};`。开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连本 child。

**抽离记录**:
BE-001CN-03 已创建 `src/runtime/query_support.rs`，并从 `src/runtime/mod.rs`、src/runtime/run.rs (retired drained include)、src/runtime/mutation.rs (retired drained include) 迁入 7 个 Query DTO、`clean_optional_filter`、`normalized_replay_options`、`DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE`。DTO 类型本体保持 `pub(crate)` 以满足 `pub(crate)` handler 签名，字段统一为 `pub(super)`，两个 helper 为 `pub(super)`。`MAX_EXPERIMENT_VARIANTS` 仍留在父级。

**明确排除**:
`DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均不属于本子叶。src/runtime/mutation.rs (retired drained include) 当前为 drained include，后续只能由父叶残余判断决定是否处理。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_v1_reports`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`。

**幻觉检查点**:
AI 声称 `runtime.query_support` 已完成 BE-001CN-03 时，必须说明 `src/runtime/query_support.rs` 已创建，7 个 Query DTO 与两个 normalization helper 已迁入 child，DTO 类型本体为 `pub(crate)`，字段/helper 为 `pub(super)`，调用方文件未改动且仍通过 `use super::*`，`MAX_EXPERIMENT_VARIANTS` 与 response/run guard 未迁移，下一步只能进入 BE-001CN-04 单叶 closeout。不得宣称 `backend.runtime` 已完成、parent include 已删除、发布过渡已启动或 Rust 重构完成。

AI 声称 `runtime.query_support` 已完成 BE-001CN-04 时，必须说明本批次是 `no code movement` closeout，`runtime.query_support stop_split: true`，不继续拆 replay/mutation/report query 或 normalization 微叶，下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。不得宣称 run guard、response support、parent include 删除、发布过渡已启动或 Rust 重构完成。

### 5.1.24 `runtime.response_support`

**层级路径**: `root.backend.runtime.runtime.response_support`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CP-04 单叶 closeout 已完成。`runtime.response_support stop_split: true`；下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/response_support.rs`
- `src/runtime/run/record_store.rs`
- `src/runtime/backtest/record_store.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/report_ops/merge_generation_health.rs`
- `markdown/06-milestones/v4.16.0/289-runtime.response_support单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/290-runtime.response_support抽离方案.md`
- `markdown/06-milestones/v4.16.0/291-runtime.response_support抽离记录.md`
- `markdown/06-milestones/v4.16.0/292-runtime.response_support单叶closeout.md`

**职责**:
冻结 runtime response DTO 的白箱边界。当前 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 与 `MergeRecordEntry` 已从父级迁入 `src/runtime/response_support.rs`；不拥有 run guard、experiment limit、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。
**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| discard outcome | run/backtest discard handlers | artifact kind / id | 不改变 discard response field semantics |
| merge record projection | `runtime.report_ops.merge_generation_health` | record entries / totals | 不改变 `/api/v1/merge/records` response contract |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | run/backtest discard endpoints | JSON response DTO | type 后续若迁移须保持 handler signature 所需 `pub(crate)` visibility |
| `MergeRecordsResponse` | merge records endpoint | JSON response DTO | type 后续若迁移须保持 handler signature 所需 `pub(crate)` visibility |
| `MergeRecordEntry` | `MergeRecordsResponse.records` | JSON response item | field visibility 优先收敛为 `pub(super)` |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | discarded kind/id | discard JSON | `src/runtime/run/record_store.rs`、`src/runtime/backtest/record_store.rs`、`src/runtime/backtest/record_lifecycle.rs` | 不得改字段名、字段类型或 response schema |
| `MergeRecordsResponse` | merge records / totals | merge records JSON | `src/runtime/report_ops/merge_generation_health.rs` | 不得改 records/totals 语义 |
| `MergeRecordEntry` | merge record projection | merge record item | `src/runtime/report_ops/merge_generation_health.rs` | 不得改 file/status/conflict/suppressed/path 表达 |

**父级通信规则**:
`runtime.response_support` 只能经 `src/runtime/mod.rs` controlled response surface 服务 sibling child callers。父级只允许 `mod response_support;` 与普通 `use response_support::{...};`，不得用 `pub(crate) use` 扩大对外 surface。开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连本 child。
**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_v1_reports`；`cargo test -p quantpilot --test api_v1_ops_health`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。
**幻觉检查点**:
AI 声称 BE-001CP-01 完成时，必须说明本批次是 `no code movement` 基线，`response_support` planned child 尚未创建，response DTO 仍在 `src/runtime/mod.rs` 与 src/runtime/run.rs (retired drained include)，下一步只能进入 BE-001CP-02 抽离方案。不得宣称 response DTO 已抽离、run guard 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CP-02 完成时，必须说明本批次仍是 `no code movement` 抽离方案，下一步 BE-001CP-03 才允许创建 planned child 并迁移 3 个 response DTO；父级只能使用 `mod response_support` 与 plain `use response_support::{...};`，src/runtime/run.rs (retired drained include) 迁移后只能降为 drained include 注释。不得宣称 response DTO 已迁移、parent include 已删除、run guard 已处理、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CP-03 完成时，必须说明 `src/runtime/response_support.rs` 已创建，`DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 与 `MergeRecordEntry` 已迁入 child，父级只保留 `mod response_support;` 与 plain `use response_support::{...};`，src/runtime/run.rs (retired drained include) 只剩 drained include 注释但 `include!("run.rs")` 仍保留，下一步只能进入 BE-001CP-04 单叶 closeout。不得宣称 run guard、experiment limit、parent include 删除、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CP-04 完成时，必须说明本批次是 `no code movement` closeout，`runtime.response_support stop_split: true`，不继续拆 discard response / merge records response 微叶，`src/runtime/response_support.rs` 仍承接 3 个 response DTO，下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。不得宣称 run guard、experiment limit、parent include 删除、发布过渡已启动或 Rust 重构完成。

### 5.1.25 `runtime.run_guard`

**层级路径**: `root.backend.runtime.runtime.run_guard`
**父模块**: `backend.runtime`
**状态**: v4.16 BE-001CR-04 单叶 closeout 已完成。`runtime.run_guard stop_split: true`；下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。
**真实文件**:
- `src/runtime/mod.rs`
- `src/runtime/run_guard.rs`
- `src/runtime/run/session_start.rs`
- `src/runtime/run/v4_handoff.rs`
- `markdown/06-milestones/v4.16.0/293-backend.runtime第六轮父叶残余判断.md`
- `markdown/06-milestones/v4.16.0/294-runtime.run_guard单子叶等价基线.md`
- `markdown/06-milestones/v4.16.0/295-runtime.run_guard抽离方案.md`
- `markdown/06-milestones/v4.16.0/296-runtime.run_guard抽离记录.md`
- `markdown/06-milestones/v4.16.0/297-runtime.run_guard单叶closeout.md`

**职责**:
承接 `RunInProgressGuard` 的白箱边界。当前 child 只拥有 run guard owner、`AtomicBool` 引用、Drop `store(false, Ordering::Release)` 复位语义和 child-local unit smoke；调用方 `swap(true, Ordering::AcqRel)` 进入检查仍保留在 `runtime.run.session_start` 与 `runtime.run.v4_handoff`。本叶不拥有 experiment limit、parent include cleanup、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` owner、lock order 或 release transition guard。

BE-001CR-04 已确认本叶不继续拆成 enter check、drop reset 或 unit smoke 微叶；继续拆分不会形成新的稳定 owner，只会扩大父子接线面。

**输入**:
| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `run_in_progress` | `AppState` | `AtomicBool` | owner 不迁移，不改变 `AppState` 字段 |
| run start attempt | `start_test_run`、`start_v4_runtime_run` | handler call | 进入前保持 `swap(true, Ordering::AcqRel)` |

**输出**:
| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| conflict rejection | run callers | HTTP 409 | busy 时不构造 guard |
| Drop reset | `AppState.run_in_progress` | `store(false, Ordering::Release)` | 成功进入后所有返回路径必须复位 |

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `RunInProgressGuard` | `&AtomicBool` | RAII guard | `src/runtime/run/session_start.rs`、`src/runtime/run/v4_handoff.rs` | 不得改构造时机、Drop 复位或 ordering |
| `Drop for RunInProgressGuard` | guard lifetime end | Release reset | Rust drop path | 不得改成手动复位或横向连接 |

**父级通信规则**:
`runtime.run_guard` 后续若实际抽离，只能经 `src/runtime/mod.rs` controlled run guard surface 服务 run child callers。父级方案只能引入 planned child 和 plain import，不得用 `pub(crate) use` 扩大对外 surface。开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连 planned child。

BE-001CR-02 固定 BE-001CR-03 的父级声明为 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`。方案选择不单独开 test-first 批次，但允许实际抽离时在 child 内新增最小 unit smoke；不得把 `swap(true, Ordering::AcqRel)` 移入 guard，不得统一 legacy/v4 busy response。

BE-001CR-03 已按该方案落地。父级 `src/runtime/mod.rs` 只保留 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`，两个 run child 调用方继续通过 `use super::*` 访问父级受控 surface，没有 direct child import。

**回归保护**:
`cargo fmt --check`；`cargo check -p quantpilot`；`cargo test -p quantpilot --test api_run`；`cargo test -p quantpilot --test api_backtest`；`cargo test -p quantpilot --test api_mutation`；`cargo test -p quantpilot --test api_ai_proposal`；`cargo test -p quantpilot --test api_v1_reports`；`cargo test -p quantpilot --test api_v1_ops_health`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`；`powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`。

**幻觉检查点**:
AI 声称 BE-001CR-01 完成时，必须说明本批次是 `no code movement` 基线，planned run guard child 尚未创建，`RunInProgressGuard` 仍在 `src/runtime/mod.rs`，两个调用方仍为 `src/runtime/run/session_start.rs` 与 `src/runtime/run/v4_handoff.rs`，下一步只能进入 BE-001CR-02 抽离方案。不得宣称 run guard 已抽离、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CR-02 完成时，必须说明本批次是 `no code movement` 抽离方案，planned run guard child 尚未创建，BE-001CR-03 的迁移清单仅限 `RunInProgressGuard` 与 Drop impl，父级只允许 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`，下一步只能进入 BE-001CR-03 实际抽离。不得宣称 run guard 已迁移、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CR-03 完成时，必须说明 `src/runtime/run_guard.rs` 已创建，`RunInProgressGuard` 与 Drop impl 已迁入 child，父级只保留 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`，两个调用方没有新增 direct child import，下一步只能进入 BE-001CR-04 单叶 closeout。不得宣称 experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

AI 声称 BE-001CR-04 完成时，必须说明本批次是 `no code movement` closeout，`runtime.run_guard stop_split: true`，本叶不继续细拆，下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。不得宣称 experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

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
- `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md`
- `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md`
- `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md`
- `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md`

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
| `markdown/06-milestones/v4.16.0/62-runtime.run.record_store真实边界梳理.md` runtime run record store true boundary | `runtime.run.record_store` | 真实 route method、frontend 调用、shared helper owner 和最小迁移边界 | BE-001J 真实边界梳理 | 不得引入 `/discard` route 或私有化 shared helper |
| `markdown/06-milestones/v4.16.0/63-runtime.run.record_store抽离方案.md` runtime run record store extraction plan | `runtime.run.record_store` | 四个 handler 最小迁移方案、父级 re-export 和 shared helper 保留边界 | BE-001J 抽离方案 | 不得迁移 replay/status/SSE、state owner、persistence owner 或 frontend route |
| `markdown/06-milestones/v4.16.0/64-runtime.run.record_store抽离记录.md` runtime run record store extraction record | `runtime.run.record_store` | 四个 handler 迁入 `src/runtime/run/record_store.rs`，父级保留兼容出口 | BE-001J 抽离记录 | 不得宣称 replay/status/SSE、state owner、persistence owner、shared helper owner 或 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/65-runtime.run.record_store单叶closeout.md` runtime run record store closeout | `runtime.run.record_store` | 单叶整理、等价证据和停止内部细分判断 | BE-001J 单叶 closeout | 不得继续细拆本叶或宣称 replay/status/SSE、state owner、persistence owner 已完成 |
| `markdown/06-milestones/v4.16.0/66-runtime.run.replay_status单子叶等价基线.md` runtime run replay status baseline | `runtime.run.replay_status` | run replay/status handler 层等价基线 | BE-001K 单子叶基线 | 不得迁移 SSE、response mapping、schema、metrics、state owner 或 persistence owner |
| `markdown/06-milestones/v4.16.0/67-runtime.run.replay_status抽离方案.md` runtime run replay status extraction plan | `runtime.run.replay_status` | 两个 handler 最小迁移方案、父级 re-export 和 SSE 排除边界 | BE-001K 抽离方案 | 不得迁移 SSE、response mapping、schema、metrics、state owner 或 persistence owner |
| `markdown/06-milestones/v4.16.0/68-runtime.run.replay_status抽离记录.md` runtime run replay status extraction record | `runtime.run.replay_status` | 两个 handler 迁入 `src/runtime/run/replay_status.rs`，父级保留兼容出口 | BE-001K 抽离记录 | 不得宣称 SSE、response mapping、schema、metrics、state owner、persistence owner 或 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/69-runtime.run.replay_status单叶closeout.md` runtime run replay status closeout | `runtime.run.replay_status` | 单叶整理、等价证据和停止内部细分判断 | BE-001K 单叶 closeout | 不得继续细拆本叶或宣称 SSE、response mapping、schema、metrics、state owner、persistence owner 已完成 |
| `markdown/06-milestones/v4.16.0/70-runtime.event_stream单子叶等价基线.md` runtime event stream baseline | `runtime.event_stream` | SSE route、frame order、keep-alive 和父级 route owner 等价基线 | BE-001L 单子叶基线 | 不得迁移 `stream_run_events`、state owner、persistence owner 或 frontend caller |
| `markdown/06-milestones/v4.16.0/71-runtime.event_stream抽离方案.md` runtime event stream extraction plan | `runtime.event_stream` | `stream_run_events` 最小迁移方案、父级 route owner 和 shared helper 保留边界 | BE-001L 抽离方案 | 不得宣称 SSE 已迁移、route facade 已迁移、state/persistence/frontend 已迁移或发布过渡启动 |
| `markdown/06-milestones/v4.16.0/72-runtime.event_stream抽离记录.md` runtime event stream extraction record | `runtime.event_stream` | `stream_run_events` 迁入 `src/runtime/event_stream.rs`，父级保留兼容出口 | BE-001L 抽离记录 | 不得宣称 route facade、shared helper、state/persistence/frontend 或本叶 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/73-runtime.event_stream单叶closeout.md` runtime event stream closeout | `runtime.event_stream` | 单叶整理、等价证据和停止内部细分判断 | BE-001L 单叶 closeout | 不得继续细拆本叶或宣称 backtest、mutation、report、state/persistence/frontend 已完成 |
| `markdown/06-milestones/v4.16.0/74-runtime.backtest单子叶等价基线.md` runtime backtest baseline | `runtime.backtest` | backtest route group、handler、artifact/compare/replay/persistence owner 边界 | BE-001M 单子叶基线 | 不得移动 handler、artifact/schema、compare、state/persistence 或 frontend owner |
| `markdown/06-milestones/v4.16.0/75-runtime.backtest抽离方案.md` runtime backtest extraction plan | `runtime.backtest` | 下一批只抽离 backtest route facade | BE-001M 抽离方案 | 不得宣称 route facade、handler、artifact、compare、persistence、schema 或 frontend owner 已迁移 |
| `markdown/06-milestones/v4.16.0/76-runtime.backtest抽离记录.md` runtime backtest extraction record | `runtime.backtest` | backtest route registration 迁入 `src/backend/runtime/routes/backtest.rs` | BE-001M route facade 抽离 | 不得宣称 handler、artifact、compare、persistence、schema、state 或 frontend owner 已迁移 |
| `markdown/06-milestones/v4.16.0/77-runtime.backtest单叶closeout.md` runtime backtest closeout | `runtime.backtest` | route facade closeout、handler 域继续细分判断 | BE-001M 单叶 closeout | 不得宣称 handler、artifact、compare、persistence、schema、state 或 frontend owner 已迁移 |
| `markdown/06-milestones/v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md` runtime backtest execution start baseline | `runtime.backtest.execution_start` | backtest 创建路径、legacy/v4 execution helper 和 transient spill 边界 | BE-001N 单子叶基线 | 不得迁移代码或混入 record/replay/experiment/artifact/compare/persistence/state/frontend owner |
| `markdown/06-milestones/v4.16.0/79-runtime.backtest.execution_start抽离方案.md` runtime backtest execution start extraction plan | `runtime.backtest.execution_start` | 下一批只移动 backtest 创建路径 handler/helper，并保留 experiment 复用桥 | BE-001N 抽离方案 | 不得宣称 handler/helper 已迁移或混入 record/replay/experiment/artifact/compare/persistence/state/frontend owner |
| `markdown/06-milestones/v4.16.0/80-runtime.backtest.execution_start抽离记录.md` runtime backtest execution start extraction record | `runtime.backtest.execution_start` | 创建路径 handler/helper 迁入 `src/runtime/backtest/execution_start.rs`，父级保留 re-export 与 experiment 复用桥 | BE-001N 抽离记录 | 不得宣称 record/replay/experiment/artifact/compare/persistence/state/frontend owner 已迁移 |
| `markdown/06-milestones/v4.16.0/81-runtime.backtest.execution_start单叶closeout.md` runtime backtest execution start closeout | `runtime.backtest.execution_start` | 创建路径 handler/helper 等价 closeout，下一候选为 `runtime.backtest.execution_start.v4_projection` | BE-001N 单叶 closeout | 不得宣称 stop_split、record/replay/experiment/shared owner 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md` runtime backtest execution start v4 projection baseline | `runtime.backtest.execution_start.v4_projection` | 冻结 v4 artifact projection helper、输入输出、测试证据和排除项 | BE-001O 单子叶等价基线 | `no code movement`；不得宣称 request resolution、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md` runtime backtest execution start v4 projection extraction plan | `runtime.backtest.execution_start.v4_projection` | 下一批只移动 v4 projection helper 与现有单元测试，父级私有导入三个入口 helper | BE-001O 抽离方案 | `no code movement`；不得宣称 helper、request resolution、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md` runtime backtest execution start v4 projection extraction record | `runtime.backtest.execution_start.v4_projection` | projection helper 与现有单元测试迁入 `src/runtime/backtest/v4_projection.rs` | BE-001O 抽离记录 | 不得宣称 request resolution、record write、schema/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md` runtime backtest execution start v4 projection closeout | `runtime.backtest.execution_start.v4_projection` | projection 子模块等价 closeout 并设置 `stop_split: true` | BE-001O 单叶 closeout | 不得宣称 request resolution、record write、schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md` runtime backtest execution start v4 request resolution baseline | `runtime.backtest.execution_start.v4_request_resolution` | 冻结 v4 request detection、graph/symbol/event resolution 和错误 code | BE-001P 单子叶等价基线 | `no code movement`；不得宣称 helper、projection、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md` runtime backtest execution start v4 request resolution extraction plan | `runtime.backtest.execution_start.v4_request_resolution` | 下一批只移动四个 request resolution helper | BE-001P 抽离方案 | `no code movement`；不得宣称 helper、projection、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md` runtime backtest execution start v4 request resolution extraction record | `runtime.backtest.execution_start.v4_request_resolution` | 四个 request resolution helper 迁入 `src/runtime/backtest/v4_request_resolution.rs` | BE-001P 抽离记录 | 不得宣称 replay/runtime execution、projection、record write、schema/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md` runtime backtest execution start v4 request resolution closeout | `runtime.backtest.execution_start.v4_request_resolution` | 四个 request resolution helper 等价 closeout 并设置 `stop_split: true` | BE-001P 单叶 closeout | 不得宣称 `execute_v4_backtest_request`、replay/runtime execution、projection、record write、schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md` runtime backtest execution start v4 runtime execution baseline | `runtime.backtest.execution_start.v4_runtime_execution` | 冻结 deterministic replay、v4 runtime execution 和 `V4BacktestArtifact` 输出 | BE-001Q 单子叶等价基线 | `no code movement`；不得宣称 runtime execution helper、request resolution、projection、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md` runtime backtest execution start v4 runtime execution extraction plan | `runtime.backtest.execution_start.v4_runtime_execution` | 限定下一批只迁移 deterministic runtime execution 最小 helper | BE-001Q 抽离方案 | `no code movement`；不得宣称 helper、request resolution、projection、record write、schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md` runtime backtest execution start v4 runtime execution extraction record | `runtime.backtest.execution_start.v4_runtime_execution` | deterministic bars/ticks 与 blocking runtime replay 迁入 `src/runtime/backtest/v4_runtime_execution.rs` | BE-001Q 抽离记录 | 不得宣称 expanded graph、request resolution、projection、record write、schema/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md` runtime backtest execution start v4 runtime execution closeout | `runtime.backtest.execution_start.v4_runtime_execution` | `run_v4_backtest_runtime_execution` 等价 closeout 并设置 `stop_split: true` | BE-001Q 单叶 closeout | 不得宣称 expanded graph、request resolution、projection、record write、schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md` runtime backtest execution start legacy dispatch baseline | `runtime.backtest.execution_start.legacy_dispatch` | legacy compile/sandbox dispatch 等价基线，当前 `no code movement` | BE-001R 单子叶基线 | 不得宣称 legacy helper 已抽离、record write/persistence/state/frontend owner 已迁移或发布过渡已启动 |
| `markdown/06-milestones/v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md` runtime backtest execution start legacy dispatch extraction plan | `runtime.backtest.execution_start.legacy_dispatch` | 下一批只允许迁移 legacy compile/sandbox dispatch 最小 helper | BE-001R 抽离方案 | `no code movement`；不得宣称 helper 已迁移或 record write/persistence/state/frontend owner 已迁移 |
| `markdown/06-milestones/v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md` runtime backtest execution start legacy dispatch extraction record | `runtime.backtest.execution_start.legacy_dispatch` | legacy compile/assumption/artifact/sandbox replay 迁入 `src/runtime/backtest/legacy_dispatch.rs` | BE-001R 抽离记录 | 不得宣称 record assembly、artifact views、schema/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md` runtime backtest execution start legacy dispatch closeout | `runtime.backtest.execution_start.legacy_dispatch` | legacy dispatch helper 等价 closeout 并设置 `stop_split: true` | BE-001R 单叶 closeout | 不得宣称 record write、schema/state/persistence/frontend caller、父叶整体完成或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md` runtime backtest execution start parent residual decision | `runtime.backtest.execution_start` | 父叶残余判断完成，下一候选回到 `runtime.backtest.record_store` | BE-001S 父叶残余判断 | `no code movement`；不得宣称 record store、replay、experiment、schema/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md` runtime backtest record store baseline | `runtime.backtest.record_store` | backtest list/detail/save/discard、transient/persistent record、artifact view 与 audit 等价基线 | BE-001T 单子叶基线 | `no code movement`；不得宣称 handler 已迁移或 replay/experiment/compare/schema/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/100-runtime.backtest.record_store抽离方案.md` runtime backtest record store extraction plan | `runtime.backtest.record_store` | 抽离方案，下一批只允许四个 handler 最小迁移 | BE-001T 抽离方案 | `no code movement`；不得宣称 handler 已迁移或 replay/experiment/compare/shared owner/schema/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/101-runtime.backtest.record_store抽离记录.md` runtime backtest record store extraction record | `runtime.backtest.record_store` | 四个 handler 迁入 `src/runtime/backtest/record_store.rs` | BE-001T 抽离记录 | 不得宣称 replay/experiment/compare/shared owner/schema/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/102-runtime.backtest.record_store单叶closeout.md` runtime backtest record store closeout | `runtime.backtest.record_store` | 单叶整理、等价证据和停止内部细分判断 | BE-001T 单叶 closeout | 不得继续细拆本叶或宣称 replay/experiment/compare/shared owner/schema/state/persistence/frontend caller、发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/103-runtime.backtest.replay单子叶等价基线.md` runtime backtest replay baseline | `runtime.backtest.replay` | backtest replay route、query normalization、response mapping 和 metrics 等价基线 | BE-001U 单子叶基线 | `no code movement`；不得宣称 handler 已迁移或 record_store/execution_start/experiment/compare/schema/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/104-runtime.backtest.replay抽离方案.md` runtime backtest replay extraction plan | `runtime.backtest.replay` | 抽离方案，下一批只允许迁移 `get_backtest_replay` | BE-001U 抽离方案 | `no code movement`；不得宣称 handler 已迁移或 query/mapping/schema/metrics/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/105-runtime.backtest.replay抽离记录.md` runtime backtest replay extraction record | `runtime.backtest.replay` | `get_backtest_replay` 迁入 `src/runtime/backtest/replay.rs` | BE-001U 抽离记录 | 不得宣称 query/mapping/schema/metrics/state/persistence/frontend caller、closeout 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/106-runtime.backtest.replay单叶closeout.md` runtime backtest replay closeout | `runtime.backtest.replay` | 单叶整理、等价证据和停止内部细分判断 | BE-001U 单叶 closeout | 不得继续细拆本叶或宣称 experiment/compare/schema/state/persistence/frontend caller、发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md` runtime backtest experiment sweep baseline | `runtime.backtest.experiment_sweep` | experiment routes、参数网格、复用桥、persistence、save/discard lifecycle 与 audit 等价基线 | BE-001V 单子叶基线 | `no code movement`；不得宣称 handler、route facade、record_store/replay/compare/schema/state/persistence/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md` runtime backtest experiment sweep extraction plan | `runtime.backtest.experiment_sweep` | 抽离方案，下一批只允许迁移 experiment handler/helper | BE-001V 抽离方案 | `no code movement`；不得宣称 handler 已迁移或 route facade/execution_start/persistence/mapping/schema/state/frontend caller 已迁移 |
| `markdown/06-milestones/v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md` runtime backtest experiment sweep extraction record | `runtime.backtest.experiment_sweep` | 抽离记录，5 个 handler 和 3 个 helper 已迁入 `src/runtime/backtest/experiment_sweep.rs` | BE-001V 抽离记录 | 不得宣称 route facade、execution_start、persistence、mapping、schema、state、audit、frontend caller、发布过渡或 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md` runtime backtest experiment sweep closeout | `runtime.backtest.experiment_sweep` | 单叶整理、等价证据和继续细分判断，登记 `stop_split: false` 与下一候选 `runtime.backtest.experiment_sweep.parameter_grid` | BE-001V 单叶 closeout | 不得宣称 parameter_grid 已抽离、route facade、execution_start、persistence、mapping、schema、state、audit、frontend caller、发布过渡或整理重构已完成 |
| `markdown/06-milestones/v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md` runtime backtest experiment sweep parameter grid baseline | `runtime.backtest.experiment_sweep.parameter_grid` | 参数网格校验、轴归一化、base fallback、去重、variant count 和展开顺序等价基线 | BE-001W 单子叶基线 | `no code movement`；不得宣称 helper 已抽离、schema 已修改、`MAX_EXPERIMENT_VARIANTS` 已调整或 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md` runtime backtest experiment sweep parameter grid extraction plan | `runtime.backtest.experiment_sweep.parameter_grid` | 抽离方案，下一批只允许迁移 3 个 helper 到父级私有子模块 | BE-001W 抽离方案 | `no code movement`；不得宣称 helper 已迁移、schema/constant/route/shared owner 或 closeout 已完成 |
| `markdown/06-milestones/v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md` runtime backtest experiment sweep parameter grid extraction record | `runtime.backtest.experiment_sweep.parameter_grid` | 抽离记录，3 个 helper 已迁入 `src/runtime/backtest/parameter_grid.rs` | BE-001W 抽离记录 | 不得宣称 parameter_grid 已 closeout、schema/constant/route/shared owner 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md` runtime backtest experiment sweep parameter grid closeout | `runtime.backtest.experiment_sweep.parameter_grid` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001W 单叶 closeout | 不得继续细拆 parameter_grid 或宣称 experiment_sweep 父叶最终完成、schema/constant/route/shared owner 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md` runtime backtest experiment sweep parent residual decision | `runtime.backtest.experiment_sweep` | 父叶残余判断，确认 `parameter_grid` 关闭但父叶仍 `stop_split: false`，下一候选 `start_orchestration` | BE-001X 父叶残余判断 | `no code movement`；不得宣称 `start_orchestration`、`record_lifecycle`、route/schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md` runtime backtest experiment sweep start orchestration baseline | `runtime.backtest.experiment_sweep.start_orchestration` | 单子叶等价基线，冻结 `start_backtest_experiment` 创建编排、guard、variant request、execution bridge 和 preview persistence | BE-001Y 单子叶基线 | `no code movement`；不得宣称 start handler、record lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md` runtime backtest experiment sweep start orchestration extraction plan | `runtime.backtest.experiment_sweep.start_orchestration` | 抽离方案，限定下一批只迁移 `start_backtest_experiment` 到 planned start_orchestration child file | BE-001Y 抽离方案 | `no code movement`；不得宣称 start handler 已迁移、record lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md` runtime backtest experiment sweep start orchestration extraction record | `runtime.backtest.experiment_sweep.start_orchestration` | 抽离记录，将 `start_backtest_experiment` 迁入 `src/runtime/backtest/start_orchestration.rs` | BE-001Y 抽离记录 | 不得宣称 start orchestration 已 closeout、record lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md` runtime backtest experiment sweep start orchestration closeout | `runtime.backtest.experiment_sweep.start_orchestration` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001Y 单叶 closeout | 不得继续细拆 start_orchestration 或宣称 record_lifecycle、route/schema/state/persistence/frontend caller、发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md` runtime backtest experiment sweep second parent residual decision | `runtime.backtest.experiment_sweep` | 第二轮父叶残余判断，确认 `parameter_grid` 与 `start_orchestration` 均关闭但父叶仍 `stop_split: false`，下一候选 `record_lifecycle` | BE-001Z 父叶残余判断 | `no code movement`；不得宣称 record_lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md` runtime backtest experiment sweep record lifecycle baseline | `runtime.backtest.experiment_sweep.record_lifecycle` | 单子叶等价基线，冻结 list/detail/save/discard record lifecycle 边界 | BE-001AA 单子叶基线 | `no code movement`；不得宣称 record_lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md` runtime backtest experiment sweep record lifecycle extraction plan | `runtime.backtest.experiment_sweep.record_lifecycle` | 抽离方案，限定下一批只迁移四个 lifecycle handler 到 planned child file | BE-001AA 抽离方案 | `no code movement`；不得宣称 record_lifecycle、route/schema/state/persistence/frontend caller 或发布过渡已迁移 |
| `markdown/06-milestones/v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md` runtime backtest experiment sweep record lifecycle extraction record | `runtime.backtest.experiment_sweep.record_lifecycle` | 抽离记录，四个 lifecycle handler 已迁入 `src/runtime/backtest/record_lifecycle.rs` | BE-001AA 抽离记录 | 不得宣称 record_lifecycle 已 closeout、route/schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md` runtime backtest experiment sweep record lifecycle closeout | `runtime.backtest.experiment_sweep.record_lifecycle` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AA 单叶 closeout | 不得继续细拆 record_lifecycle 或宣称 experiment_sweep 父叶最终完成、route/schema/state/persistence/frontend caller、发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md` runtime backtest experiment sweep third parent residual decision | `runtime.backtest.experiment_sweep` | 第三轮父叶残余判断，三个子叶均已 closeout 并设置父叶 `stop_split: true` | BE-001AB 父叶残余判断 | `no code movement`；不得宣称 `backend.runtime.routes` 上层完成、route/schema/state/persistence/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/126-runtime.backtest父叶残余判断.md` runtime backtest parent residual decision | `runtime.backtest` | 父叶残余判断，四个 handler 子叶均已 closeout 并设置父叶 `stop_split: true` | BE-001AC 父叶残余判断 | `no code movement`；不得宣称 `backend.runtime.routes` 上层完成、drained parent include 已删除、compare/artifact/persistence/response/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/127-backend.runtime.routes父叶残余判断.md` backend runtime routes parent residual decision | `backend.runtime.routes` | 父叶残余判断，确认父叶仍 `stop_split: false` 并登记下一候选 `backend.runtime.routes.mutation` | BE-001AD 父叶残余判断 | `no code movement`；不得宣称 mutation route 已抽离、handler/schema/state/frontend caller 或发布过渡已完成 |
| `markdown/06-milestones/v4.16.0/128-backend.runtime.routes.mutation单子叶等价基线.md` backend runtime routes mutation baseline | `backend.runtime.routes.mutation` | 单子叶等价基线，冻结 mutation / AI proposal / approval route group | BE-001AE 单子叶基线 | `no code movement`；不得宣称 route/handler/AppState/lock order/schema/frontend caller 或 release transition 已迁移 |
| `markdown/06-milestones/v4.16.0/129-backend.runtime.routes.mutation抽离方案.md` backend runtime routes mutation extraction plan | `backend.runtime.routes.mutation` | 抽离方案，只规划 route facade 最小迁移 | BE-001AE 抽离方案 | `no code movement`；不得宣称 planned route facade 已创建、handler/AppState/lock order/schema/frontend caller 或 release transition 已迁移 |
| `markdown/06-milestones/v4.16.0/130-backend.runtime.routes.mutation抽离记录.md` backend runtime routes mutation extraction record | `backend.runtime.routes.mutation` | route facade 实际抽离，`src/backend/runtime/routes/mutation.rs` 承接 mutation / AI proposal / approval route group | BE-001AE 抽离记录 | 不得宣称 handler/AppState/lock order/schema/frontend caller 或 release transition 已迁移；单叶 closeout 尚未完成 |
| `markdown/06-milestones/v4.16.0/131-backend.runtime.routes.mutation单叶closeout.md` backend runtime routes mutation closeout | `backend.runtime.routes.mutation` | 单叶 closeout，route facade 等价并设置 `stop_split: true` | BE-001AE 单叶 closeout | 不得宣称 handler/AppState/lock order/schema/frontend caller 或 release transition 已迁移；已由 BE-001AF-01 `runtime.mutation.parameter_mutation` 基线承接 |
| `markdown/06-milestones/v4.16.0/132-runtime.mutation.parameter_mutation单子叶等价基线.md` runtime mutation parameter mutation baseline | `runtime.mutation.parameter_mutation` | 单子叶等价基线，冻结 parameter mutation handler 生命周期 | BE-001AF 单子叶基线 | `no code movement`；不得宣称 handler moved、target file created、AI proposal/approval/shared owner migrated 或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/133-runtime.mutation.parameter_mutation抽离方案.md` runtime mutation parameter mutation extraction plan | `runtime.mutation.parameter_mutation` | 抽离方案，固定目标子模块、父级 re-export 和 shared helper 保留边界 | BE-001AF 抽离方案 | `no code movement`；下一步只能进入 BE-001AF-03 实际抽离，不得宣称 handler moved、target file created 或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/134-runtime.mutation.parameter_mutation抽离记录.md` runtime mutation parameter mutation extraction record | `runtime.mutation.parameter_mutation` | 实际抽离，五个 parameter mutation handler 已迁入 `src/runtime/mutation/parameter_mutation.rs` 并通过父级 re-export 暴露 | BE-001AF 抽离记录 | 下一步只能进入 BE-001AF-04 单叶 closeout；不得宣称 AI proposal/approval/AppState/schema/frontend caller 或 release transition 已迁移 |
| `markdown/06-milestones/v4.16.0/135-runtime.mutation.parameter_mutation单叶closeout.md` runtime mutation parameter mutation closeout | `runtime.mutation.parameter_mutation` | 单叶 closeout，确认等价并设置 `stop_split: false`，登记 transition lifecycle 下一基线 | BE-001AF 单叶 closeout | `no code movement`；下一步只能进入 BE-001AG-01，不得宣称 transition lifecycle 已抽离 |
| `markdown/06-milestones/v4.16.0/136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle baseline | `runtime.mutation.parameter_mutation.transition_lifecycle` | 单子叶等价基线，冻结 activation / rollback lifecycle、safe window、boundary、transition persistence、auto snapshot side effect 和 run event append | BE-001AG 单子叶基线 | `no code movement`；下一步只能进入 BE-001AG-02，不得宣称 transition lifecycle 已抽离、目标文件已创建或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md` runtime mutation parameter mutation transition lifecycle extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle` | 抽离方案，固定目标文件、父级声明、handler re-export、boundary validation 可见性和迁移清单 | BE-001AG 抽离方案 | `no code movement`；下一步只能进入 BE-001AG-03，不得宣称 transition lifecycle 已抽离、目标文件已创建或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/138-runtime.mutation.parameter_mutation.transition_lifecycle抽离记录.md` runtime mutation parameter mutation transition lifecycle extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle` | 实际抽离，activation / rollback handler 和 transition helper 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` | BE-001AG 抽离记录 | 下一步只能进入 BE-001AG-04，不得宣称单叶 closeout、parameter_mutation 父叶完成或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md` runtime mutation parameter mutation transition lifecycle closeout | `runtime.mutation.parameter_mutation.transition_lifecycle` | 单叶 closeout，确认实际抽离等价并设置 `stop_split: false` | BE-001AG 单叶 closeout | 下一步只能进入 BE-001AH-01 `boundary_safety` 等价基线，不得宣称 boundary_safety 已创建或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle boundary safety baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 单子叶等价基线，冻结 boundary validation、boundary resolution 和 safe window evaluation | BE-001AH 单子叶基线 | `no code movement`；下一步只能进入 BE-001AH-02，不得创建 boundary_safety 目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md` runtime mutation parameter mutation transition lifecycle boundary safety extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 抽离方案，固定目标文件、父级 mod、delegating validation wrapper 和 helper visibility | BE-001AH 抽离方案 | `no code movement`；下一步只能进入 BE-001AH-03，不得宣称 boundary_safety 已抽离或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md` runtime mutation parameter mutation transition lifecycle boundary safety extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 实际抽离，boundary validation / boundary resolution / safe-window evaluation 已迁入 child | BE-001AH 抽离记录 | 下一步只能进入 BE-001AH-04 单叶 closeout，不得宣称 boundary_safety 已完成 closeout 或 release transition 已启动 |
| `markdown/06-milestones/v4.16.0/143-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单叶closeout.md` runtime mutation parameter mutation transition lifecycle boundary safety closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AH 单叶 closeout | `no code movement`；下一步只能进入 BE-001AI-01 父叶残余判断，不得继续拆 boundary_safety 或 release transition |
| `markdown/06-milestones/v4.16.0/144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md` runtime mutation parameter mutation transition lifecycle parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 父叶残余判断，确认 `boundary_safety` 停止细拆，父叶保持 `stop_split: false` | BE-001AI 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AJ-01 `activation_flow` 单子叶等价基线，不得直接移动 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle activation flow baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 单子叶等价基线，冻结 activation handler 状态机、event append、metrics、transition persistence 和 snapshot trigger | BE-001AJ 单子叶基线 | `no code movement`；下一步只能进入 BE-001AJ-02 抽离方案，不得创建目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md` runtime mutation parameter mutation transition lifecycle activation flow extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 抽离方案，固定目标文件、path-attributed child、handler re-export 和 helper 保留边界 | BE-001AJ 抽离方案 | `no code movement`；下一步只能进入 BE-001AJ-03 实际抽离，不得迁移 rollback/snapshot body 或 release transition |
| `markdown/06-milestones/v4.16.0/147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md` runtime mutation parameter mutation transition lifecycle activation flow extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 实际抽离，activation public handler 已迁入 child | BE-001AJ 抽离记录 | 下一步只能进入 BE-001AJ-04 单叶 closeout，不得迁移 rollback/snapshot body 或 release transition |
| `markdown/06-milestones/v4.16.0/148-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单叶closeout.md` runtime mutation parameter mutation transition lifecycle activation flow closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AJ 单叶 closeout | `no code movement`；下一步只能进入 BE-001AK-01 父叶残余判断，不得继续拆 activation_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/149-runtime.mutation.parameter_mutation.transition_lifecycle第二轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle second parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 第二轮父叶残余判断，确认下一候选为 rollback_flow | BE-001AK 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AL-01，不得创建 rollback_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle rollback flow baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 单子叶等价基线，冻结 rollback handler 状态机、ledger lookup、event append、metrics 和 transition persistence | BE-001AL 单子叶基线 | `no code movement`；下一步只能进入 BE-001AL-02，不得创建 rollback_flow 目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md` runtime mutation parameter mutation transition lifecycle rollback flow extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 抽离方案，固定目标文件、path-attributed child、handler re-export 和 helper 保留边界 | BE-001AL 抽离方案 | `no code movement`；下一步只能进入 BE-001AL-03 实际抽离，不得迁移 snapshot body 或 release transition |
| `markdown/06-milestones/v4.16.0/152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md` runtime mutation parameter mutation transition lifecycle rollback flow extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 实际抽离，rollback public handler 已迁入 child | BE-001AL 抽离记录 | 下一步只能进入 BE-001AL-04 单叶 closeout，不得迁移 rollback helper、snapshot body 或 release transition |
| `markdown/06-milestones/v4.16.0/153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md` runtime mutation parameter mutation transition lifecycle rollback flow closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AL 单叶 closeout | `no code movement`；下一步只能进入 BE-001AM-01 父叶残余判断，不得继续拆 rollback_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle third parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 第三轮父叶残余判断，确认下一候选为 activation_snapshot_side_effect | BE-001AM 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AN-01，不得创建 side effect 文件或 release transition |
| `markdown/06-milestones/v4.16.0/155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 单子叶等价基线，冻结 activation auto snapshot 的 config generation、history truncation、snapshot id、payload/signature、atomic write 与 in-memory insert | BE-001AN 单子叶基线 | `no code movement`；下一步只能进入 BE-001AN-02，不得创建 side effect 目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 抽离方案，固定目标 child、父级 path attribute、helper import、visibility 与回退点 | BE-001AN 抽离方案 | `no code movement`；下一步只能进入 BE-001AN-03 实际抽离，不得迁移 shared helper 或 release transition |
| `markdown/06-milestones/v4.16.0/157-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离记录.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 实际抽离，activation auto snapshot helper 已迁入 child | BE-001AN 抽离记录 | 下一步只能进入 BE-001AN-04 单叶 closeout，不得迁移 shared helper 或 release transition |
| `markdown/06-milestones/v4.16.0/158-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单叶closeout.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AN 单叶 closeout | `no code movement`；下一步只能进入 BE-001AO-01 父叶残余判断，不得继续拆 activation_snapshot_side_effect 或 release transition |
| `markdown/06-milestones/v4.16.0/159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle fourth parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 第四轮父叶残余判断，确认四个子叶已 closeout 且父叶仍 `stop_split: false` | BE-001AO 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AP-01 `transition_record_persistence` 单子叶等价基线，不得迁移 rollback id 或 release transition |
| `markdown/06-milestones/v4.16.0/160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle transition record persistence baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 单子叶等价基线，冻结 lifecycle entry 与 transition persistence | BE-001AP 单子叶基线 | `no code movement`；下一步只能进入 BE-001AP-02 抽离方案，不得创建目标文件、迁移 rollback id 或 release transition |
| `markdown/06-milestones/v4.16.0/161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md` runtime mutation parameter mutation transition lifecycle transition record persistence extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 抽离方案，固定目标 child、父级声明、helper import 和回退点 | BE-001AP 抽离方案 | `no code movement`；下一步只能进入 BE-001AP-03 实际抽离，不得迁移 rollback id 或 release transition |
| `markdown/06-milestones/v4.16.0/162-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离记录.md` runtime mutation parameter mutation transition lifecycle transition record persistence extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 实际抽离，lifecycle entry 与 transition persistence helper 已迁入 child | BE-001AP 抽离记录 | 下一步只能进入 BE-001AP-04 单叶 closeout，不得迁移 rollback id 或 release transition |
| `markdown/06-milestones/v4.16.0/163-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单叶closeout.md` runtime mutation parameter mutation transition lifecycle transition record persistence closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AP 单叶 closeout | `no code movement`；下一步只能进入 BE-001AQ-01 父叶残余判断，不得继续拆 transition_record_persistence 或 release transition |
| `markdown/06-milestones/v4.16.0/164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle fifth parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 第五轮父叶残余判断，确认五个子叶已 closeout 且父叶仍 `stop_split: false` | BE-001AQ 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AR-01 `rollback_record_identity` 单子叶等价基线，不得迁移 rollback id 或 release transition |
| `markdown/06-milestones/v4.16.0/165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle rollback record identity baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 单子叶等价基线，冻结 rollback id digest contract | BE-001AR 单子叶基线 | `no code movement`；下一步只能进入 BE-001AR-02 抽离方案，不得创建目标文件、回改 rollback_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md` runtime mutation parameter mutation transition lifecycle rollback record identity extraction plan | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 抽离方案，固定目标 child、父级声明、helper import 和回退点 | BE-001AR 抽离方案 | `no code movement`；下一步只能进入 BE-001AR-03 实际抽离，不得回改 rollback_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/167-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离记录.md` runtime mutation parameter mutation transition lifecycle rollback record identity extraction record | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 实际抽离，rollback id helper 已迁入 child | BE-001AR 抽离记录 | 下一步只能进入 BE-001AR-04 单叶 closeout，不得回改 rollback_flow 或 release transition |
| `markdown/06-milestones/v4.16.0/168-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单叶closeout.md` runtime mutation parameter mutation transition lifecycle rollback record identity closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AR 单叶 closeout | `no code movement`；下一步只能进入 BE-001AS-01 父叶残余判断，不得继续拆 rollback_record_identity 或 release transition |
| `markdown/06-milestones/v4.16.0/169-runtime.mutation.parameter_mutation.transition_lifecycle第六轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle sixth parent residual decision | `runtime.mutation.parameter_mutation.transition_lifecycle` | 第六轮父叶残余判断，确认六个子叶已 closeout 且父叶设置 `stop_split: true` | BE-001AS 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断，不得继续拆 transition_lifecycle 或 release transition |
| `markdown/06-milestones/v4.16.0/170-runtime.mutation.parameter_mutation父叶残余判断.md` runtime mutation parameter mutation parent residual decision | `runtime.mutation.parameter_mutation` | 父叶残余判断，确认 proposal creation/list/detail 仍为残余且父叶保持 `stop_split: false` | BE-001AT 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AU-01 `proposal_creation` 单子叶等价基线，不得移动 create handler 或 release transition |
| `markdown/06-milestones/v4.16.0/171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md` runtime mutation parameter mutation proposal creation baseline | `runtime.mutation.parameter_mutation.proposal_creation` | 单子叶等价基线，冻结 create handler 与 record id helper | BE-001AU 单子叶基线 | `no code movement`；下一步只能进入 BE-001AU-02 抽离方案，不得创建目标文件、迁移 list/detail 或 release transition |
| `markdown/06-milestones/v4.16.0/172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md` runtime mutation parameter mutation proposal creation extraction plan | `runtime.mutation.parameter_mutation.proposal_creation` | 抽离方案，固定目标文件、父级声明、handler re-export、迁移清单和回退点 | BE-001AU 抽离方案 | `no code movement`；下一步只能进入 BE-001AU-03 实际抽离，不得迁移 list/detail 或 release transition |
| `markdown/06-milestones/v4.16.0/173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md` runtime mutation parameter mutation proposal creation extraction record | `runtime.mutation.parameter_mutation.proposal_creation` | 实际抽离，create handler 与 record id helper 已迁入 child | BE-001AU 抽离记录 | 下一步只能进入 BE-001AU-04 单叶 closeout，不得迁移 list/detail 或 release transition |
| `markdown/06-milestones/v4.16.0/174-runtime.mutation.parameter_mutation.proposal_creation单叶closeout.md` runtime mutation parameter mutation proposal creation closeout | `runtime.mutation.parameter_mutation.proposal_creation` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AU 单叶 closeout | `no code movement`；下一步只能进入 BE-001AV-01 父叶残余判断，不得继续拆 proposal_creation 或 release transition |
| `markdown/06-milestones/v4.16.0/175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md` runtime mutation parameter mutation second parent residual decision | `runtime.mutation.parameter_mutation` | 父叶残余判断，确认 list/detail 查询流为下一候选且父叶保持 `stop_split: false` | BE-001AV 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AW-01 `record_query` 单子叶等价基线，不得移动 list/detail 或 release transition |
| `markdown/06-milestones/v4.16.0/176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md` runtime mutation parameter mutation record query baseline | `runtime.mutation.parameter_mutation.record_query` | 单子叶等价基线，冻结 list/detail read model | BE-001AW 单子叶基线 | `no code movement`；下一步只能进入 BE-001AW-02 抽离方案，不得创建目标文件、迁移 list/detail 或 release transition |
| `markdown/06-milestones/v4.16.0/177-runtime.mutation.parameter_mutation.record_query抽离方案.md` runtime mutation parameter mutation record query extraction plan | `runtime.mutation.parameter_mutation.record_query` | 抽离方案，固定目标文件、父级声明、双 handler re-export 和迁移清单 | BE-001AW 抽离方案 | `no code movement`；下一步只能进入 BE-001AW-03 实际抽离，不得迁移 create/activate/rollback 或 release transition |
| `markdown/06-milestones/v4.16.0/178-runtime.mutation.parameter_mutation.record_query抽离记录.md` runtime mutation parameter mutation record query extraction record | `runtime.mutation.parameter_mutation.record_query` | 实际抽离，list/detail handler 已迁入 child | BE-001AW 抽离记录 | 下一步只能进入 BE-001AW-04 单叶 closeout，不得迁移 create/activate/rollback 或 release transition |
| `markdown/06-milestones/v4.16.0/179-runtime.mutation.parameter_mutation.record_query单叶closeout.md` runtime mutation parameter mutation record query closeout | `runtime.mutation.parameter_mutation.record_query` | 单叶 closeout，确认等价并设置 `stop_split: true` | BE-001AW 单叶 closeout | `no code movement`；下一步只能进入 BE-001AX-01 父叶残余判断，不得继续拆 record_query 或 release transition |
| `markdown/06-milestones/v4.16.0/180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md` runtime mutation parameter mutation third parent residual decision | `runtime.mutation.parameter_mutation` | 父叶残余判断，确认三个 child 均 closeout 并设置父叶 `stop_split: true` | BE-001AX 父叶残余判断 | `no code movement`；下一步只能进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线，不得移动 AI proposal 或 release transition |
| `markdown/06-milestones/v4.16.0/181-runtime.mutation.ai_proposal单子叶等价基线.md` runtime mutation ai proposal baseline | `runtime.mutation.ai_proposal` | 单子叶等价基线，冻结 AI proposal / approval handler 域 | BE-001AY 单子叶基线 | `no code movement`；下一步只能进入 BE-001AY-02 抽离方案，不得移动 handler/helper、AppState/schema/frontend caller 或 release transition |
| `markdown/06-milestones/v4.16.0/182-runtime.mutation.ai_proposal抽离方案.md` runtime mutation ai proposal extraction plan | `runtime.mutation.ai_proposal` | 抽离方案，固定目标文件、父级声明、public handler re-export 和 shared helper 保留清单 | BE-001AY 抽离方案 | `no code movement`；下一步只能进入 BE-001AY-03 实际抽离，不得迁移 shared helper 或 release transition |
| `markdown/06-milestones/v4.16.0/183-runtime.mutation.ai_proposal抽离记录.md` runtime mutation ai proposal extraction record | `runtime.mutation.ai_proposal` | 实际抽离，AI proposal / approval public handlers 与专属 helper 已迁入 child | BE-001AY 抽离记录 | 下一步只能进入 BE-001AY-04 单叶 closeout，不得继续细拆或 release transition |
| `markdown/06-milestones/v4.16.0/184-runtime.mutation.ai_proposal单叶closeout.md` runtime mutation ai proposal closeout | `runtime.mutation.ai_proposal` | 单叶 closeout，确认抽离等价并设置 `stop_split: false` | BE-001AY 单叶 closeout | `no code movement`；下一步只能进入 BE-001AZ-01 `static_check` 单子叶等价基线，不得直接创建目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md` runtime mutation ai proposal static check baseline | `runtime.mutation.ai_proposal.static_check` | 单子叶等价基线，冻结 validation / analysis helper | BE-001AZ 单子叶基线 | `no code movement`；下一步只能进入 BE-001AZ-02 抽离方案，不得创建 static_check 目标文件或 release transition |
| `markdown/06-milestones/v4.16.0/186-runtime.mutation.ai_proposal.static_check抽离方案.md` runtime mutation ai proposal static check extraction plan | `runtime.mutation.ai_proposal.static_check` | 抽离方案，固定目标文件、helper import、`pub(super)` visibility 和回退点 | BE-001AZ 抽离方案 | `no code movement`；下一步只能进入 BE-001AZ-03 实际抽离，不得迁移 approval gate 或 release transition |
| `markdown/06-milestones/v4.16.0/187-runtime.mutation.ai_proposal.static_check抽离记录.md` runtime mutation ai proposal static check extraction record | `runtime.mutation.ai_proposal.static_check` | 实际抽离，helper 与静态检查单测迁入 child | BE-001AZ 实际抽离 | 下一步只能进入 BE-001AZ-04 单叶 closeout，不得迁移 approval_review、record_query 或 release transition |
| `markdown/06-milestones/v4.16.0/188-runtime.mutation.ai_proposal.static_check单叶closeout.md` runtime mutation ai proposal static check closeout | `runtime.mutation.ai_proposal.static_check` | 单叶 closeout，设置 `stop_split: true` | BE-001AZ 单叶收口 | 下一步只能进入 BE-001BA-01 父叶残余判断，不得继续细拆 static_check 或 release transition |
| `markdown/06-milestones/v4.16.0/189-runtime.mutation.ai_proposal父叶残余判断.md` runtime mutation ai proposal parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `source_governance_identity` | BE-001BA 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BB-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md` runtime mutation ai proposal source governance identity baseline | `runtime.mutation.ai_proposal.source_governance_identity` | 单子叶等价基线，冻结 source context / governance / record id | BE-001BB 单子叶基线 | `no code movement`；下一步只能进入 BE-001BB-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md` runtime mutation ai proposal source governance identity extraction plan | `runtime.mutation.ai_proposal.source_governance_identity` | 抽离方案，冻结目标文件 / child 声明 / helper visibility | BE-001BB 抽离方案 | `no code movement`；下一步只能进入 BE-001BB-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md` runtime mutation ai proposal source governance identity extraction record | `runtime.mutation.ai_proposal.source_governance_identity` | 实际抽离记录，source/governance/id helper 迁入 child | BE-001BB 实际抽离 | 下一步只能进入 BE-001BB-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md` runtime mutation ai proposal source governance identity closeout | `runtime.mutation.ai_proposal.source_governance_identity` | 单叶 closeout，设置 `stop_split: true` | BE-001BB 单叶收口 | 下一步只能进入 BE-001BC-01 父叶残余判断，不得继续细拆 source_governance_identity 或 release transition |
| `markdown/06-milestones/v4.16.0/194-runtime.mutation.ai_proposal第二轮父叶残余判断.md` runtime mutation ai proposal second parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `event_lifecycle` | BE-001BC 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BD-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md` runtime mutation ai proposal event lifecycle baseline | `runtime.mutation.ai_proposal.event_lifecycle` | 单子叶等价基线，冻结 event contract / payload / lifecycle / transition persistence | BE-001BD 单子叶基线 | `no code movement`；下一步只能进入 BE-001BD-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md` runtime mutation ai proposal event lifecycle extraction plan | `runtime.mutation.ai_proposal.event_lifecycle` | 抽离方案，冻结目标文件 / child 声明 / helper visibility | BE-001BD 抽离方案 | `no code movement`；下一步只能进入 BE-001BD-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md` runtime mutation ai proposal event lifecycle extraction record | `runtime.mutation.ai_proposal.event_lifecycle` | 实际抽离记录，event/lifecycle helper 迁入 child | BE-001BD 实际抽离 | 下一步只能进入 BE-001BD-04 单叶 closeout，不得迁移 record_query、approval_review 或 release transition |
| `markdown/06-milestones/v4.16.0/198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md` runtime mutation ai proposal event lifecycle closeout | `runtime.mutation.ai_proposal.event_lifecycle` | 单叶 closeout，设置 `stop_split: true` | BE-001BD 单叶收口 | 下一步只能进入 BE-001BE-01 父叶残余判断，不得继续细拆 event_lifecycle 或 release transition |
| `markdown/06-milestones/v4.16.0/199-runtime.mutation.ai_proposal第三轮父叶残余判断.md` runtime mutation ai proposal third parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `record_query` | BE-001BE 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BF-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md` runtime mutation ai proposal record query baseline | `runtime.mutation.ai_proposal.record_query` | 单子叶等价基线，冻结 proposal list/detail/read-through loader | BE-001BF 单子叶基线 | `no code movement`；下一步只能进入 BE-001BF-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/201-runtime.mutation.ai_proposal.record_query抽离方案.md` runtime mutation ai proposal record query extraction plan | `runtime.mutation.ai_proposal.record_query` | 抽离方案，冻结目标文件、handler re-export 与 loader import | BE-001BF 抽离方案 | `no code movement`；下一步只能进入 BE-001BF-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/202-runtime.mutation.ai_proposal.record_query抽离记录.md` runtime mutation ai proposal record query extraction record | `runtime.mutation.ai_proposal.record_query` | 实际抽离记录，list/detail/read-through loader 迁入 child | BE-001BF 实际抽离 | 下一步只能进入 BE-001BF-04 单叶 closeout，不得迁移 approval_review 或 release transition |
| `markdown/06-milestones/v4.16.0/203-runtime.mutation.ai_proposal.record_query单叶closeout.md` runtime mutation ai proposal record query closeout | `runtime.mutation.ai_proposal.record_query` | 单叶 closeout，设置 `stop_split: true` | BE-001BF 单叶收口 | 下一步只能进入 BE-001BG-01 父叶残余判断，不得继续细拆 record_query 或 release transition |
| `markdown/06-milestones/v4.16.0/204-runtime.mutation.ai_proposal第四轮父叶残余判断.md` runtime mutation ai proposal fourth parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `approval_review` | BE-001BG 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BH-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md` runtime mutation ai proposal approval review baseline | `runtime.mutation.ai_proposal.approval_review` | 单子叶等价基线，冻结 approval list/detail/approve/reject/claim | BE-001BH 单子叶基线 | `no code movement`；下一步只能进入 BE-001BH-02 抽离方案，不得创建 approval_review 文件或 release transition |
| `markdown/06-milestones/v4.16.0/206-runtime.mutation.ai_proposal.approval_review抽离方案.md` runtime mutation ai proposal approval review extraction plan | `runtime.mutation.ai_proposal.approval_review` | 抽离方案，固定目标 child、父级声明和五 handler re-export | BE-001BH 抽离方案 | `no code movement`；下一步只能进入 BE-001BH-03 实际抽离，不得迁移 persistence/sandbox/status helper 或 release transition |
| `markdown/06-milestones/v4.16.0/207-runtime.mutation.ai_proposal.approval_review抽离记录.md` runtime mutation ai proposal approval review extraction record | `runtime.mutation.ai_proposal.approval_review` | 实际抽离记录，approval 五 handler 迁入 child | BE-001BH 实际抽离 | 下一步只能进入 BE-001BH-04 单叶 closeout，不得迁移 persistence/sandbox/status helper 或 release transition |
| `markdown/06-milestones/v4.16.0/208-runtime.mutation.ai_proposal.approval_review单叶closeout.md` runtime mutation ai proposal approval review closeout | `runtime.mutation.ai_proposal.approval_review` | 单叶 closeout，设置 `stop_split: true` | BE-001BH 单叶收口 | 下一步只能进入 BE-001BI-01 父叶残余判断，不得继续细拆 approval_review 或 release transition |
| `markdown/06-milestones/v4.16.0/209-runtime.mutation.ai_proposal第五轮父叶残余判断.md` runtime mutation ai proposal fifth parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `approval_persistence` | BE-001BI 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BJ-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md` runtime mutation ai proposal approval persistence baseline | `runtime.mutation.ai_proposal.approval_persistence` | 单子叶等价基线，冻结 approval record disk read/write helper | BE-001BJ 单子叶基线 | `no code movement`；下一步只能进入 BE-001BJ-02 抽离方案，不得创建 approval_persistence 文件或 release transition |
| `markdown/06-milestones/v4.16.0/211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md` runtime mutation ai proposal approval persistence extraction plan | `runtime.mutation.ai_proposal.approval_persistence` | 抽离方案，固定目标文件、父级声明和 helper import | BE-001BJ 抽离方案 | `no code movement`；下一步只能进入 BE-001BJ-03 实际抽离，不得迁移 sandbox/status helper 或 release transition |
| `markdown/06-milestones/v4.16.0/212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md` runtime mutation ai proposal approval persistence extraction record | `runtime.mutation.ai_proposal.approval_persistence` | 实际抽离记录，两个 persistence helper 迁入 child | BE-001BJ 实际抽离 | 下一步只能进入 BE-001BJ-04 单叶 closeout，不得迁移 sandbox/status helper 或 release transition |
| `markdown/06-milestones/v4.16.0/213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md` runtime mutation ai proposal approval persistence closeout | `runtime.mutation.ai_proposal.approval_persistence` | 单叶 closeout，设置 `stop_split: true` | BE-001BJ 单叶收口 | 下一步只能进入 BE-001BK-01 父叶残余判断，不得继续细拆 approval_persistence 或 release transition |
| `markdown/06-milestones/v4.16.0/214-runtime.mutation.ai_proposal第六轮父叶残余判断.md` runtime mutation ai proposal sixth parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `sandbox_trigger` | BE-001BK 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BL-01 单子叶等价基线，不得直接创建 sandbox_trigger 文件或 release transition |
| `markdown/06-milestones/v4.16.0/215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md` runtime mutation ai proposal sandbox trigger baseline | `runtime.mutation.ai_proposal.sandbox_trigger` | 单子叶等价基线，冻结 sandbox gate / background task / report URL side effect | BE-001BL 单子叶基线 | `no code movement`；下一步只能进入 BE-001BL-02 抽离方案，不得创建 sandbox_trigger 文件或 release transition |
| `markdown/06-milestones/v4.16.0/216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md` runtime mutation ai proposal sandbox trigger extraction plan | `runtime.mutation.ai_proposal.sandbox_trigger` | 抽离方案，固定目标文件、父级声明、helper import 和 background task helper | BE-001BL 抽离方案 | `no code movement`；下一步只能进入 BE-001BL-03 实际抽离，不得迁移 status_transition 或 release transition |
| `markdown/06-milestones/v4.16.0/217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md` runtime mutation ai proposal sandbox trigger extraction record | `runtime.mutation.ai_proposal.sandbox_trigger` | 实际抽离，创建 child 文件并迁移 sandbox gate / background task helper | BE-001BL 实际抽离 | 下一步只能进入 BE-001BL-04 单叶 closeout，不得迁移 status_transition 或 release transition |
| `markdown/06-milestones/v4.16.0/218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md` runtime mutation ai proposal sandbox trigger closeout | `runtime.mutation.ai_proposal.sandbox_trigger` | 单叶 closeout，设置 `stop_split: true` | BE-001BL 单叶收口 | 下一步只能进入 BE-001BM-01 父叶残余判断，不得继续细拆 sandbox_trigger 或 release transition |
| `markdown/06-milestones/v4.16.0/219-runtime.mutation.ai_proposal第七轮父叶残余判断.md` runtime mutation ai proposal seventh parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `status_transition` | BE-001BM 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BN-01 单子叶等价基线，不得直接创建 status_transition 文件或 release transition |
| `markdown/06-milestones/v4.16.0/220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md` runtime mutation ai proposal status transition baseline | `runtime.mutation.ai_proposal.status_transition` | 单子叶等价基线，冻结状态机 helper 边界 | BE-001BN 单子叶基线 | `no code movement`；下一步只能进入 BE-001BN-02 抽离方案，不得创建 status_transition 文件或 release transition |
| `markdown/06-milestones/v4.16.0/221-runtime.mutation.ai_proposal.status_transition抽离方案.md` runtime mutation ai proposal status transition extraction plan | `runtime.mutation.ai_proposal.status_transition` | 抽离方案，固定目标文件、父级声明和 helper import | BE-001BN 抽离方案 | `no code movement`；下一步只能进入 BE-001BN-03 实际抽离，不得迁移 proposal create orchestration 或 release transition |
| `markdown/06-milestones/v4.16.0/222-runtime.mutation.ai_proposal.status_transition抽离记录.md` runtime mutation ai proposal status transition extraction record | `runtime.mutation.ai_proposal.status_transition` | 实际抽离，创建 child 文件并迁移 approved/status helper | BE-001BN 实际抽离 | 下一步只能进入 BE-001BN-04 单叶 closeout，不得迁移 proposal create orchestration 或 release transition |
| `markdown/06-milestones/v4.16.0/223-runtime.mutation.ai_proposal.status_transition单叶closeout.md` runtime mutation ai proposal status transition closeout | `runtime.mutation.ai_proposal.status_transition` | 单叶 closeout，设置 `stop_split: true` | BE-001BN 单叶 closeout | 下一步只能进入 BE-001BO-01 父叶残余判断，不得继续细拆 status_transition 或 release transition |
| `markdown/06-milestones/v4.16.0/224-runtime.mutation.ai_proposal第八轮父叶残余判断.md` runtime mutation ai proposal eighth parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，选择下一候选 `proposal_creation` | BE-001BO 父叶判断 | 父叶保持 `stop_split: false`；下一步只能进入 BE-001BP-01 单子叶等价基线，不得直接创建 proposal_creation 文件或 release transition |
| `markdown/06-milestones/v4.16.0/225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md` runtime mutation ai proposal proposal creation baseline | `runtime.mutation.ai_proposal.proposal_creation` | 单子叶等价基线，冻结 create handler、状态副作用和锁顺序 | BE-001BP 单子叶基线 | `no code movement`；下一步只能进入 BE-001BP-02 抽离方案，不得创建 proposal_creation 文件、迁移 create handler 或 release transition |
| `markdown/06-milestones/v4.16.0/226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md` runtime mutation ai proposal proposal creation extraction plan | `runtime.mutation.ai_proposal.proposal_creation` | 抽离方案，固定目标 child、父级声明、handler re-export、迁移清单和回退点 | BE-001BP 抽离方案 | `no code movement`；下一步只能进入 BE-001BP-03 实际抽离，不得迁移 record_query、approval_review、AppState/schema/frontend caller 或 release transition |
| `markdown/06-milestones/v4.16.0/227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md` runtime mutation ai proposal proposal creation extraction record | `runtime.mutation.ai_proposal.proposal_creation` | 实际抽离，create handler 已迁入 child | BE-001BP 抽离记录 | 下一步只能进入 BE-001BP-04 单叶 closeout，不得迁移 record_query、approval_review、AppState/schema/frontend caller 或 release transition |
| `markdown/06-milestones/v4.16.0/228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md` runtime mutation ai proposal proposal creation closeout | `runtime.mutation.ai_proposal.proposal_creation` | 单叶 closeout，设置 `stop_split: true` | BE-001BP 单叶 closeout | `no code movement`；下一步只能进入 BE-001BQ-01 父叶残余判断，不得继续拆 proposal_creation 或 release transition |
| `markdown/06-milestones/v4.16.0/229-runtime.mutation.ai_proposal第九轮父叶残余判断.md` runtime mutation ai proposal ninth parent residual decision | `runtime.mutation.ai_proposal` | 父叶残余判断，九个子叶均已 closeout 并设置父叶 `stop_split: true` | BE-001BQ 父叶残余判断 | `no code movement`；下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断，不得回改 AI proposal 或 release transition |
| `markdown/06-milestones/v4.16.0/230-backend.runtime.routes第二轮父叶残余判断.md` backend runtime routes second parent residual decision | `backend.runtime.routes` | 父叶残余判断，run/backtest/mutation route child 已关闭但父叶保持 `stop_split: false` | BE-001BR 父叶残余判断 | `no code movement`；下一步只能进入 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线，不得迁移 route handler 或 release transition |
| `markdown/06-milestones/v4.16.0/231-backend.runtime.routes.experiment单子叶等价基线.md` backend runtime routes experiment baseline | `backend.runtime.routes.experiment` | 单子叶等价基线，冻结 experiment route group | BE-001BS 单子叶基线 | `no code movement`；下一步只能进入 BE-001BS-02 抽离方案，不得创建 route child、迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/232-backend.runtime.routes.experiment抽离方案.md` backend runtime routes experiment extraction plan | `backend.runtime.routes.experiment` | 抽离方案，下一批只迁移五个 experiment route registration | BE-001BS 抽离方案 | `no code movement`；下一步只能进入 BE-001BS-03 实际抽离，不得迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/233-backend.runtime.routes.experiment抽离记录.md` backend runtime routes experiment extraction record | `backend.runtime.routes.experiment` | 实际抽离，五个 experiment route registration 已迁入 child | BE-001BS 抽离记录 | 下一步只能进入 BE-001BS-04 单叶 closeout，不得迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/234-backend.runtime.routes.experiment单叶closeout.md` backend runtime routes experiment closeout | `backend.runtime.routes.experiment` | 单叶 closeout，设置 `stop_split: true` | BE-001BS 单叶 closeout | `no code movement`；下一步只能进入 BE-001BT-01 `backend.runtime.routes` 父叶残余判断，不得继续拆 experiment route child 或 release transition |
| `markdown/06-milestones/v4.16.0/235-backend.runtime.routes第三轮父叶残余判断.md` backend runtime routes third parent residual decision | `backend.runtime.routes` | 父叶残余判断，四个 route child 已关闭但父叶保持 `stop_split: false` | BE-001BT 父叶残余判断 | `no code movement`；下一步只能进入 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线，不得迁移 route handler 或 release transition |
| `markdown/06-milestones/v4.16.0/236-backend.runtime.routes.evidence单子叶等价基线.md` backend runtime routes evidence baseline | `backend.runtime.routes.evidence` | 单子叶等价基线，冻结 evidence health / cleanup route group | BE-001BU 单子叶基线 | `no code movement`；下一步只能进入 BE-001BU-02 抽离方案，不得创建 route child、迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/237-backend.runtime.routes.evidence抽离方案.md` backend runtime routes evidence extraction plan | `backend.runtime.routes.evidence` | 抽离方案，固定 route child facade、父级委托顺序和允许迁移清单 | BE-001BU 抽离方案 | `no code movement`；下一步只能进入 BE-001BU-03 实际抽离，不得迁移 handler、schema、AppState、frontend caller、runtime persistence owner 或 release transition |
| `markdown/06-milestones/v4.16.0/238-backend.runtime.routes.evidence抽离记录.md` backend runtime routes evidence extraction record | `backend.runtime.routes.evidence` | 实际抽离，两条 evidence route registration 已迁入 child | BE-001BU 实际抽离 | 下一步只能进入 BE-001BU-04 单叶 closeout，不得迁移 handler、schema、AppState、frontend caller、runtime persistence owner 或 release transition |
| `markdown/06-milestones/v4.16.0/239-backend.runtime.routes.evidence单叶closeout.md` backend runtime routes evidence closeout | `backend.runtime.routes.evidence` | 单叶 closeout，设置 `stop_split: true` | BE-001BU 单叶 closeout | `no code movement`；下一步只能进入 BE-001BV-01 父叶残余判断，不得继续拆 evidence route child 或 release transition |
| `markdown/06-milestones/v4.16.0/240-backend.runtime.routes第四轮父叶残余判断.md` backend runtime routes fourth parent residual decision | `backend.runtime.routes` | 父叶残余判断，五个 route child 已关闭但父叶保持 `stop_split: false` | BE-001BV 父叶残余判断 | `no code movement`；下一步只能进入 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线，不得直接创建 route child 或 release transition |
| `markdown/06-milestones/v4.16.0/241-backend.runtime.routes.event_stream单子叶等价基线.md` backend runtime routes event stream baseline | `backend.runtime.routes.event_stream` | 单子叶等价基线，冻结 SSE route / handler owner / keepalive contract | BE-001BW 单子叶基线 | `no code movement`；下一步只能进入 BE-001BW-02 抽离方案，不得直接创建 route child 或 release transition |
| `markdown/06-milestones/v4.16.0/242-backend.runtime.routes.event_stream抽离方案.md` backend runtime routes event stream extraction plan | `backend.runtime.routes.event_stream` | 抽离方案，固定目标 route child、父级委托顺序和允许迁移清单 | BE-001BW 抽离方案 | `no code movement`；下一步只能进入 BE-001BW-03 实际抽离，不得迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/243-backend.runtime.routes.event_stream抽离记录.md` backend runtime routes event stream extraction record | `backend.runtime.routes.event_stream` | 实际抽离，迁移 run events SSE route registration | BE-001BW 实际抽离 | 创建 `src/backend/runtime/routes/event_stream.rs`；下一步只能进入 BE-001BW-04 单叶 closeout，不得处理 report_ops 或 release transition |
| `markdown/06-milestones/v4.16.0/244-backend.runtime.routes.event_stream单叶closeout.md` backend runtime routes event stream closeout | `backend.runtime.routes.event_stream` | 单叶 closeout，设置 `stop_split: true` | BE-001BW 单叶 closeout | `no code movement`；下一步只能进入 BE-001BX-01 父叶残余判断，不得继续拆 event_stream route child 或 release transition |
| `markdown/06-milestones/v4.16.0/245-backend.runtime.routes第五轮父叶残余判断.md` backend runtime routes fifth parent residual decision | `backend.runtime.routes` | 父叶残余判断，report_ops 为唯一 direct residual | BE-001BX 父叶残余判断 | `no code movement`；下一步只能进入 BE-001BY-01 `backend.runtime.routes.report_ops` 单子叶等价基线，不得直接创建 route child 或 release transition |
| `markdown/06-milestones/v4.16.0/246-backend.runtime.routes.report_ops单子叶等价基线.md` backend runtime routes report ops baseline | `backend.runtime.routes.report_ops` | 单子叶等价基线，冻结 report_ops routes / handler owner / state boundary | BE-001BY 单子叶基线 | `no code movement`；下一步只能进入 BE-001BY-02 抽离方案，不得直接创建 route child 或 release transition |
| `markdown/06-milestones/v4.16.0/247-backend.runtime.routes.report_ops抽离方案.md` backend runtime routes report ops extraction plan | `backend.runtime.routes.report_ops` | 抽离方案，固定两个父级委托入口、route order 和允许迁移清单 | BE-001BY 抽离方案 | `no code movement`；下一步只能进入 BE-001BY-03 实际抽离，不得迁移 handler、state owner、schema owner、runtime persistence owner 或 release transition |
| `markdown/06-milestones/v4.16.0/248-backend.runtime.routes.report_ops抽离记录.md` backend runtime routes report ops extraction record | `backend.runtime.routes.report_ops` | 实际抽离，迁移十条 report_ops route registration | BE-001BY 实际抽离 | 创建 `src/backend/runtime/routes/report_ops.rs`；下一步只能进入 BE-001BY-04 单叶 closeout，不得迁移 handler、state owner、schema owner、runtime persistence owner 或 release transition |
| `markdown/06-milestones/v4.16.0/249-backend.runtime.routes.report_ops单叶closeout.md` backend runtime routes report ops closeout | `backend.runtime.routes.report_ops` | 单叶 closeout，设置 `stop_split: true` | BE-001BY 单叶 closeout | `no code movement`；下一步只能进入 BE-001BZ-01 父叶残余判断，不得继续拆 report_ops route child 或 release transition |
| `markdown/06-milestones/v4.16.0/250-backend.runtime.routes第六轮父叶残余判断.md` backend runtime routes sixth parent residual decision | `backend.runtime.routes` | 父叶残余判断，七个 route child 均已 closeout，route aggregate 设置 `stop_split: true` | BE-001BZ 父叶收口 | `no code movement`；下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断，不得继续拆 route aggregate 或 release transition |
| `markdown/06-milestones/v4.16.0/251-backend.runtime父叶残余判断.md` backend runtime parent residual decision | `backend.runtime` | 父叶残余判断，route aggregate 已关闭但 handler 残余仍在 `src/runtime/mod.rs` | BE-001CA 父叶判断 | `no code movement`；父叶保持 `stop_split: false`；下一步只能进入 BE-001CB-01 `runtime.report_ops` 单子叶等价基线，不得直接迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/252-runtime.report_ops单子叶等价基线.md` runtime report ops baseline | `runtime.report_ops` | 单子叶等价基线，冻结 runtime report / v1 ops report handler 边界 | BE-001CB 单子叶基线 | `no code movement`；下一步只能进入 BE-001CB-02 抽离方案，不得创建 planned child 文件、迁移 handler 或 release transition |
| `markdown/06-milestones/v4.16.0/253-runtime.report_ops抽离方案.md` runtime report ops extraction plan | `runtime.report_ops` | 抽离方案，固定父级 re-export、允许迁移清单、测试缺口和回退点 | BE-001CB 抽离方案 | `no code movement`；下一步只能进入 BE-001CB-03 实际抽离，不得迁移 `runtime.evidence_health`、schema、frontend、persistence、storage lifecycle、`AppState` 或 release transition |
| `markdown/06-milestones/v4.16.0/254-runtime.report_ops抽离记录.md` runtime report ops extraction record | `runtime.report_ops` | 实际抽离，创建 handler child 并迁移十个 public handler 与四个 private helper | BE-001CB 实际抽离 | 创建 `src/runtime/report_ops.rs`；下一步只能进入 BE-001CB-04 单叶 closeout，不得迁移 `runtime.evidence_health`、schema、frontend、persistence、storage lifecycle、`AppState` 或 release transition |
| `markdown/06-milestones/v4.16.0/255-runtime.report_ops单叶closeout.md` runtime report ops closeout | `runtime.report_ops` | 单叶 closeout，确认第一轮抽离等价并设置 `stop_split: false` | BE-001CB 单叶 closeout | `no code movement`；下一步只能进入 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线，不得直接创建 child 文件、迁移 handler、处理 v1 ops/report endpoints 或 release transition |
| `markdown/06-milestones/v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md` runtime report ops runtime report baseline | `runtime.report_ops.runtime_report` | 单子叶等价基线，冻结 runtime report handler/helper 白箱边界 | BE-001CC 单子叶基线 | `no code movement`；下一步只能进入 BE-001CC-02 抽离方案，不得创建 runtime_report planned child 文件、迁移 handler、处理 v1 ops/report endpoints、`runtime.evidence_health` 或 release transition |
| `markdown/06-milestones/v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md` runtime report ops runtime report extraction plan | `runtime.report_ops.runtime_report` | 抽离方案，固定 child module、父级 re-export、验证命令和回退点 | BE-001CC 抽离方案 | `no code movement`；下一步只能进入 BE-001CC-03 实际抽离，迁移清单仅限四个 public handler 与四个 private helper，不得处理 v1 ops/report endpoints、`runtime.evidence_health` 或 release transition |
| `markdown/06-milestones/v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md` runtime report ops runtime report extraction record | `runtime.report_ops.runtime_report` | 实际抽离，创建 child module 并迁移四个 public handler 与四个 private helper | BE-001CC 实际抽离 | 创建 `src/runtime/report_ops/runtime_report.rs`；下一步只能进入 BE-001CC-04 单叶 closeout，不得处理 v1 ops/report endpoints、`runtime.evidence_health`、schema、frontend、persistence、storage lifecycle、`AppState` 或 release transition |
| `markdown/06-milestones/v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md` runtime report ops runtime report closeout | `runtime.report_ops.runtime_report` | 单叶 closeout，确认 child 等价并设置 `stop_split: true` | BE-001CC 单叶 closeout | `no code movement`；下一步只能进入 BE-001CD-01 `runtime.report_ops` 父叶残余判断，不得回改 closed child、处理 v1 ops/report endpoints、`runtime.evidence_health` 或 release transition |
| `markdown/06-milestones/v4.16.0/260-runtime.report_ops父叶残余判断.md` runtime report ops parent residual | `runtime.report_ops` | 父叶残余判断，确认父级保持 `stop_split: false` 并选择 v1 report endpoints 下一候选 | BE-001CD 父叶残余判断 | `no code movement`；下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线，不得创建 child 文件或迁移 handler |
| `markdown/06-milestones/v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md` runtime report ops v1 report endpoints baseline | `runtime.report_ops.v1_report_endpoints` | 单子叶等价基线，冻结三条 `/api/v1/reports/*` handler 与测试缺口 | BE-001CE 单子叶基线 | `no code movement`；下一步只能进入 BE-001CE-02 抽离方案，不得创建 child 文件、迁移 handler 或处理 merge/generation/storage health endpoints |
| `markdown/06-milestones/v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md` runtime report ops v1 report endpoints extraction plan | `runtime.report_ops.v1_report_endpoints` | 抽离方案，选择 test-first 并固定 smoke / extraction 顺序 | BE-001CE 抽离方案 | `no code movement`；下一步只能进入 BE-001CE-03 endpoint smoke 补测，不得创建 child module 或迁移 handler |
| `markdown/06-milestones/v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md` runtime report ops v1 report endpoints smoke | `runtime.report_ops.v1_report_endpoints` | endpoint smoke 补测，新增 `tests/api_v1_reports.rs` 覆盖三条 v1 report JSON contract | BE-001CE 补测记录 | 下一步只能进入 BE-001CE-04 实际抽离；不得创建 child module、迁移 handler 以外的 owner 或处理 merge/generation/storage health endpoints |
| `markdown/06-milestones/v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md` runtime report ops v1 report endpoints extraction record | `runtime.report_ops.v1_report_endpoints` | 实际抽离，创建 child module 并迁移三个 v1 report handler | BE-001CE 抽离记录 | 下一步只能进入 BE-001CE-05 单叶 closeout；不得处理 merge/generation/storage health endpoints、`runtime.evidence_health` 或 release transition |
| `markdown/06-milestones/v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md` runtime report ops v1 report endpoints closeout | `runtime.report_ops.v1_report_endpoints` | 单叶 closeout，确认 child 等价并设置 `stop_split: true` | BE-001CE 单叶 closeout | `no code movement`；下一步只能进入 BE-001CF-01 `runtime.report_ops` 父叶残余判断，不得继续细拆 ops/audit/research 微叶 |
| `markdown/06-milestones/v4.16.0/266-runtime.report_ops父叶残余判断.md` runtime report ops parent residual | `runtime.report_ops` | 父叶残余判断，确认父级保持 `stop_split: false` 并选择 merge/generation/storage health 下一候选 | BE-001CF 父叶残余判断 | `no code movement`；下一步只能进入 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线，不得创建 child 文件或迁移 handler |
| `markdown/06-milestones/v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md` runtime report ops merge generation health baseline | `runtime.report_ops.merge_generation_health` | 单子叶等价基线，冻结 merge/generation/storage health 三个 endpoint handler 白箱边界 | BE-001CG 单子叶基线 | `no code movement`；下一步只能进入 BE-001CG-02 抽离方案，不得创建 child 文件、迁移 handler 或跳过 endpoint smoke 缺口判定 |
| `markdown/06-milestones/v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md` runtime report ops merge generation health extraction plan | `runtime.report_ops.merge_generation_health` | test-first 抽离方案，固定 endpoint smoke、planned child 文件、父级 re-export 和允许迁移清单 | BE-001CG 抽离方案 | `no code movement`；下一步只能进入 BE-001CG-03 endpoint smoke 补测，BE-001CG-04 才允许实际迁移 |
| `markdown/06-milestones/v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md` runtime report ops merge generation health smoke | `runtime.report_ops.merge_generation_health` | endpoint smoke 补测，新增 `tests/api_v1_ops_health.rs` 覆盖三条 v1 support/health JSON contract | BE-001CG 补测记录 | 下一步只能进入 BE-001CG-04 实际抽离；不得创建 child module、迁移 handler 以外的 owner 或处理 `runtime.evidence_health` |
| `markdown/06-milestones/v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md` runtime report ops merge generation health extraction record | `runtime.report_ops.merge_generation_health` | 实际抽离，创建 child module 并迁移三个 v1 support/health handler | BE-001CG 抽离记录 | 下一步只能进入 BE-001CG-05 单叶 closeout；不得处理 `runtime.evidence_health`、schema owner、frontend caller 或 release transition |
| `markdown/06-milestones/v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md` runtime report ops merge generation health closeout | `runtime.report_ops.merge_generation_health` | 单叶 closeout，确认 child 等价并设置 `stop_split: true` | BE-001CG 单叶 closeout | `no code movement`；下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断，不得继续细拆三条 support/health 微叶 |
| `markdown/06-milestones/v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md` runtime report ops second parent residual | `runtime.report_ops` | 第二轮父叶残余判断，确认三个 child 均 closeout 并设置父叶 `stop_split: true` | BE-001CH 父叶残余判断 | `no code movement`；下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/273-backend.runtime第二轮父叶残余判断.md` backend runtime second parent residual | `backend.runtime` | 第二轮父叶残余判断，确认 routes/report_ops 已 closeout 但 evidence health handler 仍残留 | BE-001CI 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/274-runtime.evidence_health单子叶等价基线.md` runtime evidence health baseline | `runtime.evidence_health` | 单子叶等价基线，冻结 evidence health / cleanup handler 与 status count helper 白箱边界 | BE-001CJ 单子叶基线 | `no code movement`；下一步只能进入 BE-001CJ-02 抽离方案，不得创建 child 文件或迁移 handler |
| `markdown/06-milestones/v4.16.0/275-runtime.evidence_health抽离方案.md` runtime evidence health extraction plan | `runtime.evidence_health` | 抽离方案，固定 child、父级 re-export、允许迁移清单和回退点 | BE-001CJ 抽离方案 | `no code movement`；下一步只能进入 BE-001CJ-03 实际抽离，迁移清单仅限三个函数 |
| `markdown/06-milestones/v4.16.0/276-runtime.evidence_health抽离记录.md` runtime evidence health extraction record | `runtime.evidence_health` | 实际抽离，创建 child module 并迁移两个 handler 与 status count helper | BE-001CJ 抽离记录 | 下一步只能进入 BE-001CJ-04 单叶 closeout；不得迁移 route/schema/persistence/metrics/state owner |
| `markdown/06-milestones/v4.16.0/277-runtime.evidence_health单叶closeout.md` runtime evidence health closeout | `runtime.evidence_health` | 单叶 closeout，确认不继续拆 health / cleanup 微叶并设置 `stop_split: true` | BE-001CJ 单叶 closeout | `no code movement`；下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/278-backend.runtime第三轮父叶残余判断.md` backend runtime third parent residual | `backend.runtime` | 第三轮父叶残余判断，确认 mutation shared governance 与 query/guard/response support 残余仍存在 | BE-001CK 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/279-runtime.mutation.shared_governance单子叶等价基线.md` runtime mutation shared governance baseline | `runtime.mutation.shared_governance` | 单子叶等价基线，冻结 9 个 mutation shared governance helper 与调用方边界 | BE-001CL 单子叶基线 | `no code movement`；下一步只能进入 BE-001CL-02 抽离方案，不得创建 planned child 文件或迁移 helper |
| `markdown/06-milestones/v4.16.0/280-runtime.mutation.shared_governance抽离方案.md` runtime mutation shared governance extraction plan | `runtime.mutation.shared_governance` | 抽离方案，固定 planned child、父级声明、plain import、helper visibility 和迁移清单 | BE-001CL 抽离方案 | `no code movement`；下一步只能进入 BE-001CL-03 实际抽离，不得处理 query DTO/run guard 或 release transition |
| `markdown/06-milestones/v4.16.0/281-runtime.mutation.shared_governance抽离记录.md` runtime mutation shared governance extraction record | `runtime.mutation.shared_governance` | 实际抽离，创建 child module 并迁移 9 个 shared governance helper | BE-001CL 抽离记录 | 下一步只能进入 BE-001CL-04 单叶 closeout；不得处理 query DTO/run guard 或 release transition |
| `markdown/06-milestones/v4.16.0/282-runtime.mutation.shared_governance单叶closeout.md` runtime mutation shared governance closeout | `runtime.mutation.shared_governance` | 单叶 closeout，确认不继续拆 validation / event contract / governance projection 微叶并设置 `stop_split: true` | BE-001CL 单叶 closeout | `no code movement`；下一步只能进入 BE-001CM-01 `backend.runtime` 第四轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/283-backend.runtime第四轮父叶残余判断.md` backend runtime fourth parent residual | `backend.runtime` | 第四轮父叶残余判断，确认 query DTO / run guard / response support / experiment limit 残余仍存在 | BE-001CM 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CN-01 `runtime.query_support` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/284-runtime.query_support单子叶等价基线.md` runtime query support baseline | `runtime.query_support` | 单子叶等价基线，冻结 query DTO、filter normalization、replay option normalization 与 field visibility | BE-001CN 单子叶基线 | `no code movement`；下一步只能进入 BE-001CN-02 抽离方案，不得创建 planned child 文件或迁移 DTO/helper |
| `markdown/06-milestones/v4.16.0/285-runtime.query_support抽离方案.md` runtime query support extraction plan | `runtime.query_support` | 抽离方案，固定 planned child、父级声明、plain import、DTO `pub(crate)` shell、field/helper `pub(super)` visibility 和迁移清单 | BE-001CN 抽离方案 | `no code movement`；下一步只能进入 BE-001CN-03 实际抽离，不得处理 response support、run guard 或 release transition |
| `markdown/06-milestones/v4.16.0/286-runtime.query_support抽离记录.md` runtime query support extraction record | `runtime.query_support` | 实际抽离，创建 query_support child 并迁入 7 个 Query DTO 与两个 normalization helper | BE-001CN 抽离记录 | 下一步只能进入 BE-001CN-04 单叶 closeout；不得处理 response support、run guard 或 release transition |
| `markdown/06-milestones/v4.16.0/287-runtime.query_support单叶closeout.md` runtime query support closeout | `runtime.query_support` | 单叶 closeout，确认不继续细拆 query / normalization 微叶 | BE-001CN 单叶 closeout | `runtime.query_support stop_split: true`；下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/288-backend.runtime第五轮父叶残余判断.md` backend runtime fifth parent residual | `backend.runtime` | 第五轮父叶残余判断，确认 response support / run guard / experiment limit / parent include residual 仍存在 | BE-001CO 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CP-01 `runtime.response_support` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/289-runtime.response_support单子叶等价基线.md` runtime response support baseline | `runtime.response_support` | 单子叶等价基线，冻结 response DTO owner、调用方、visibility 与硬门禁 | BE-001CP 单子叶基线 | `no code movement`；下一步只能进入 BE-001CP-02 抽离方案，不得创建 planned child 文件或迁移 response DTO |
| `markdown/06-milestones/v4.16.0/290-runtime.response_support抽离方案.md` runtime response support extraction plan | `runtime.response_support` | 抽离方案，固定 planned child、plain import、3 个 DTO 迁移清单、drained include 规则与回退点 | BE-001CP 抽离方案 | `no code movement`；下一步只能进入 BE-001CP-03 实际抽离，不得迁移 run guard、experiment limit 或 parent include cleanup |
| `markdown/06-milestones/v4.16.0/291-runtime.response_support抽离记录.md` runtime response support extraction record | `runtime.response_support` | 实际抽离，创建 response_support child 并迁入 3 个 response DTO | BE-001CP 抽离记录 | 下一步只能进入 BE-001CP-04 单叶 closeout；不得处理 run guard、experiment limit 或 release transition |
| `markdown/06-milestones/v4.16.0/292-runtime.response_support单叶closeout.md` runtime response support closeout | `runtime.response_support` | 单叶 closeout，确认不继续拆 discard / merge response 微叶 | BE-001CP 单叶 closeout | `runtime.response_support stop_split: true`；下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/293-backend.runtime第六轮父叶残余判断.md` backend runtime sixth parent residual | `backend.runtime` | 第六轮父叶残余判断，确认 run guard / experiment limit / parent include residual 仍存在 | BE-001CQ 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CR-01 `runtime.run_guard` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/294-runtime.run_guard单子叶等价基线.md` runtime run guard baseline | `runtime.run_guard` | 单子叶等价基线，冻结 run guard 并发复位语义、调用方和 planned child 门禁 | BE-001CR 单子叶基线 | `no code movement`；下一步只能进入 BE-001CR-02 抽离方案，不得创建 planned child 文件或迁移 guard |
| `markdown/06-milestones/v4.16.0/295-runtime.run_guard抽离方案.md` runtime run guard extraction plan | `runtime.run_guard` | 抽离方案，固定 planned child、父级声明、plain import、`pub(super)` visibility、test-first 判定和回退点 | BE-001CR 抽离方案 | `no code movement`；下一步只能进入 BE-001CR-03 实际抽离，不得迁移 experiment limit 或删除 parent include |
| `markdown/06-milestones/v4.16.0/296-runtime.run_guard抽离记录.md` runtime run guard extraction record | `runtime.run_guard` | 实际抽离，创建 run_guard child 并迁入 RAII guard 与 Drop impl | BE-001CR 抽离记录 | 下一步只能进入 BE-001CR-04 单叶 closeout；不得处理 experiment limit、parent include cleanup 或 release transition |
| `markdown/06-milestones/v4.16.0/297-runtime.run_guard单叶closeout.md` runtime run guard closeout | `runtime.run_guard` | 单叶 closeout，确认 run guard 不继续拆分 | BE-001CR 单叶 closeout | `runtime.run_guard stop_split: true`；下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/298-backend.runtime第七轮父叶残余判断.md` backend runtime seventh parent residual | `backend.runtime` | 第七轮父叶残余判断，确认 experiment limit 与 drained parent include residual 仍存在 | BE-001CS 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/299-runtime.experiment_limit单子叶等价基线.md` runtime experiment limit baseline | `runtime.experiment_limit` | 单子叶等价基线，冻结 experiment variant limit 常量、调用方与 planned child 门禁 | BE-001CT 单子叶基线 | `no code movement`；下一步只能进入 BE-001CT-02 抽离方案，不得迁移常量、删除 parent include 或 release transition |
| `markdown/06-milestones/v4.16.0/300-runtime.experiment_limit抽离方案.md` runtime experiment limit extraction plan | `runtime.experiment_limit` | test-first 抽离方案，下一批先补 experiment variant limit 超限负测 | BE-001CT 抽离方案 | `no code movement`；下一步只能进入 BE-001CT-03 endpoint smoke 补测，不得创建 planned child 或 release transition |
| `markdown/06-milestones/v4.16.0/301-runtime.experiment_limit补测记录.md` runtime experiment limit smoke record | `runtime.experiment_limit` | endpoint smoke 补测，覆盖 36 个变体超过 27 上限的 bad_request | BE-001CT 补测记录 | `test-only`；下一步只能进入 BE-001CT-04 实际抽离，不得删除 parent include 或 release transition |
| `markdown/06-milestones/v4.16.0/302-runtime.experiment_limit抽离记录.md` runtime experiment limit extraction record | `runtime.experiment_limit` | 实际抽离，创建 experiment_limit child 并迁移 `MAX_EXPERIMENT_VARIANTS` | BE-001CT 抽离记录 | 下一步只能进入 BE-001CT-05 单叶 closeout，不得删除 parent include 或 release transition |
| `markdown/06-milestones/v4.16.0/303-runtime.experiment_limit单叶closeout.md` runtime experiment limit closeout | `runtime.experiment_limit` | 单叶 closeout，确认 experiment limit 不继续拆分 | BE-001CT 单叶 closeout | `runtime.experiment_limit stop_split: true`；下一步只能进入 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断 |
| `markdown/06-milestones/v4.16.0/304-backend.runtime第八轮父叶残余判断.md` backend runtime eighth parent residual | `backend.runtime` | 第八轮父叶残余判断，确认真实残余只剩 drained parent include cleanup | BE-001CU 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/305-runtime.parent_include_cleanup单子叶等价基线.md` runtime parent include cleanup baseline | `runtime.parent_include_cleanup` | 单子叶等价基线，冻结 drained include cleanup 删除边界和 public 出口影响面 | BE-001CV 单子叶基线 | `no code movement`；下一步只能进入 BE-001CV-02 抽离方案，不得直接删除 include 或 drained 文件 |
| `markdown/06-milestones/v4.16.0/306-runtime.parent_include_cleanup抽离方案.md` runtime parent include cleanup extraction plan | `runtime.parent_include_cleanup` | 抽离方案，限定三条 include 删除与三个 drained 文件删除 | BE-001CV 抽离方案 | `no code movement`；下一步只能进入 BE-001CV-03 实际 cleanup，不得处理父叶 closeout 或 release transition |
| `markdown/06-milestones/v4.16.0/307-runtime.parent_include_cleanup清理记录.md` runtime parent include cleanup actual cleanup | `runtime.parent_include_cleanup` | 实际 cleanup，删除三条 drained include 与三个 drained 文件 | BE-001CV 实际 cleanup | 下一步只能进入 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断，不得宣称父级完成或启动 release transition |
| `markdown/06-milestones/v4.16.0/308-backend.runtime第九轮父叶残余判断.md` backend runtime ninth parent residual | `backend.runtime` | 第九轮父叶残余判断，确认真实残余转为 parent import bridge | BE-001CW 父叶残余判断 | `no code movement`；`backend.runtime stop_split: false`；下一步只能进入 BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/309-runtime.parent_import_bridge单子叶等价基线.md` runtime parent import bridge baseline | `runtime.parent_import_bridge` | 单子叶等价基线，冻结 46 文件 parent import bridge 依赖面 | BE-001CX 单子叶基线 | `no code movement`；下一步只能进入 BE-001CX-02 抽离方案，不得直接批量改写 Rust import 或启动 release transition |
| `markdown/06-milestones/v4.16.0/310-runtime.parent_import_bridge抽离方案.md` runtime parent import bridge extraction plan | `runtime.parent_import_bridge` | 抽离方案，固定 staged explicit import pass 与首批 root support pilot | BE-001CX 抽离方案 | `no code movement`；下一步只能进入 BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离 |
| `markdown/06-milestones/v4.16.0/311-runtime.root_support_import_pilot抽离记录.md` runtime root support import pilot extraction | `runtime.root_support_import_pilot` | 实际抽离，改写 query_support 与 response_support parent wildcard import | BE-001CX 实际抽离 | 依赖文件数从 46 降为 44；下一步只能进入 BE-001CX-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/312-runtime.root_support_import_pilot单叶closeout.md` runtime root support import pilot closeout | `runtime.root_support_import_pilot` | 单叶 closeout，确认 root support import pilot 不继续细拆 | BE-001CX closeout | `runtime.root_support_import_pilot stop_split: true`；下一步只能进入 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/313-runtime.root_entry_import_pass单子叶等价基线.md` runtime root entry import pass baseline | `runtime.root_entry_import_pass` | 单子叶等价基线，冻结 root entry 候选与 test-only super import 判定 | BE-001CY 单子叶基线 | `no code movement`；下一步只能进入 BE-001CY-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/314-runtime.root_entry_import_pass抽离方案.md` runtime root entry import pass extraction plan | `runtime.root_entry_import_pass` | 抽离方案，固定 two-handler root entry pilot 与 BE-001CY-03 文件边界 | BE-001CY 抽离方案 | `no code movement`；下一步只能进入 BE-001CY-03 实际抽离，只处理 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` |
| `markdown/06-milestones/v4.16.0/315-runtime.root_entry_import_pass抽离记录.md` runtime root entry import pass extraction | `runtime.root_entry_import_pass` | 实际抽离，改写 event_stream 与 evidence_health parent wildcard import | BE-001CY 实际抽离 | 依赖文件数从 44 降为 42；下一步只能进入 BE-001CY-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/316-runtime.root_entry_import_pass单叶closeout.md` runtime root entry import pass closeout | `runtime.root_entry_import_pass` | 单叶 closeout，确认 root entry import pass 不继续细拆 | BE-001CY closeout | `runtime.root_entry_import_pass stop_split: true`；下一步只能进入 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/317-runtime.report_ops_import_pass单子叶等价基线.md` runtime report ops import pass baseline | `runtime.report_ops_import_pass` | 单子叶等价基线，冻结 report_ops facade 与 3 child 的 transitive parent surface risk | BE-001CZ 单子叶基线 | `no code movement`；下一步只能进入 BE-001CZ-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/318-runtime.report_ops_import_pass抽离方案.md` runtime report ops import pass extraction plan | `runtime.report_ops_import_pass` | 抽离方案，固定 report_ops four-file pocket 同批处理 | BE-001CZ 抽离方案 | `no code movement`；下一步只能进入 BE-001CZ-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/319-runtime.report_ops_import_pass抽离记录.md` runtime report ops import pass extraction | `runtime.report_ops_import_pass` | 实际抽离，改写 report_ops four-file pocket parent wildcard import | BE-001CZ 实际抽离 | 依赖文件数从 42 降为 38；下一步只能进入 BE-001CZ-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/320-runtime.report_ops_import_pass单叶closeout.md` runtime report ops import pass closeout | `runtime.report_ops_import_pass` | 单叶 closeout，确认 report_ops import pass 不继续细拆 | BE-001CZ closeout | `runtime.report_ops_import_pass stop_split: true`；下一步只能进入 BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/321-runtime.parent_import_bridge父叶残余判断.md` runtime parent import bridge residual judgement | `runtime.parent_import_bridge` | 父叶残余判断，确认剩余 38 个 parent bridge 依赖文件 | BE-001DA 父叶残余判断 | `runtime.parent_import_bridge stop_split: false`；下一步只能进入 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/322-runtime.run_import_pass单子叶等价基线.md` runtime run import pass baseline | `runtime.run_import_pass` | 单子叶等价基线，冻结 4 个 run child 的 import 收敛边界 | BE-001DB 单子叶基线 | `no code movement`；下一步只能进入 BE-001DB-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/323-runtime.run_import_pass抽离方案.md` runtime run import pass extraction plan | `runtime.run_import_pass` | 抽离方案，固定 4 个 run child 同批 explicit import rewrite | BE-001DB 抽离方案 | `no code movement`；下一步只能进入 BE-001DB-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/324-runtime.run_import_pass抽离记录.md` runtime run import pass extraction | `runtime.run_import_pass` | 实际抽离，改写 4 个 run child parent wildcard import | BE-001DB 实际抽离 | 依赖文件数从 38 降为 34；下一步只能进入 BE-001DB-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/325-runtime.run_import_pass单叶closeout.md` runtime run import pass closeout | `runtime.run_import_pass` | 单叶 closeout，确认 run import pass 不继续细拆 | BE-001DB closeout | `runtime.run_import_pass stop_split: true`；下一步只能进入 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/326-runtime.parent_import_bridge父叶残余判断.md` runtime parent import bridge residual judgement after run | `runtime.parent_import_bridge` | 父叶残余判断，确认剩余 34 个 parent bridge 依赖文件 | BE-001DC 父叶残余判断 | `runtime.parent_import_bridge stop_split: false`；下一步只能进入 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/327-runtime.backtest_import_pass单子叶等价基线.md` runtime backtest import pass baseline | `runtime.backtest_import_pass` | 单子叶等价基线，冻结 11 个 backtest 残余文件 | BE-001DD 单子叶基线 | `no code movement`；下一步只能进入 BE-001DD-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/328-runtime.backtest_import_pass抽离方案.md` runtime backtest import pass plan | `runtime.backtest_import_pass` | 抽离方案，拒绝 11 文件整批并拆 pocket | BE-001DD 抽离方案 | `runtime.backtest_import_pass stop_split: false`；下一步只能进入 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/329-runtime.backtest.record_store_import_pass单子叶等价基线.md` runtime backtest record store import pass baseline | `runtime.backtest.record_store_import_pass` | 单子叶等价基线，冻结 `src/runtime/backtest/record_store.rs` import 输入面 | BE-001DE 单子叶基线 | `no code movement`；下一步只能进入 BE-001DE-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/330-runtime.backtest.record_store_import_pass抽离方案.md` runtime backtest record store import pass plan | `runtime.backtest.record_store_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001DE 抽离方案 | `no code movement`；下一步只能进入 BE-001DE-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/331-runtime.backtest.record_store_import_pass抽离记录.md` runtime backtest record store import pass extraction | `runtime.backtest.record_store_import_pass` | 实际抽离，改写 `src/runtime/backtest/record_store.rs` parent wildcard import | BE-001DE 实际抽离 | 依赖文件数从 34 降为 33；下一步只能进入 BE-001DE-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/332-runtime.backtest.record_store_import_pass单叶closeout.md` runtime backtest record store import pass closeout | `runtime.backtest.record_store_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001DE 单叶 closeout | parent bridge 仍剩 33；下一步只能进入 BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/333-runtime.backtest_import_pass父叶残余判断.md` runtime backtest import pass residual judgment | `runtime.backtest_import_pass` | 父叶残余判断，确认 backtest 剩余 10 个 import 依赖文件 | BE-001DF 父叶残余判断 | `runtime.backtest_import_pass stop_split: false`；下一步只能进入 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/334-runtime.backtest.replay_import_pass单子叶等价基线.md` runtime backtest replay import pass baseline | `runtime.backtest.replay_import_pass` | 单子叶等价基线，冻结 `src/runtime/backtest/replay.rs` import 输入面 | BE-001DG 单子叶基线 | `no code movement`；下一步只能进入 BE-001DG-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/335-runtime.backtest.replay_import_pass抽离方案.md` runtime backtest replay import pass plan | `runtime.backtest.replay_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001DG 抽离方案 | `no code movement`；下一步只能进入 BE-001DG-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/336-runtime.backtest.replay_import_pass抽离记录.md` runtime backtest replay import pass extraction | `runtime.backtest.replay_import_pass` | 实际抽离，改写 `src/runtime/backtest/replay.rs` parent wildcard import | BE-001DG 实际抽离 | 依赖文件数从 33 降为 32；下一步只能进入 BE-001DG-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/337-runtime.backtest.replay_import_pass单叶closeout.md` runtime backtest replay import pass closeout | `runtime.backtest.replay_import_pass` | 单叶 closeout，确认 replay import pocket 不继续拆微叶 | BE-001DG 单叶 closeout | `runtime.backtest.replay_import_pass stop_split: true`；下一步只能进入 BE-001DH-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/338-runtime.backtest_import_pass第二轮父叶残余判断.md` runtime backtest import pass second residual judgment | `runtime.backtest_import_pass` | 第二轮父叶残余判断，确认 backtest 剩余 9 个 import 依赖文件 | BE-001DH 父叶残余判断 | `runtime.backtest_import_pass stop_split: false`；下一步只能进入 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md` runtime backtest experiment sweep import pass baseline | `runtime.backtest.experiment_sweep_import_pass` | 单子叶等价基线，冻结 experiment sweep 四文件 import pocket | BE-001DI 单子叶基线 | `no code movement`；下一步只能进入 BE-001DI-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/340-runtime.backtest.experiment_sweep_import_pass抽离方案.md` runtime backtest experiment sweep import pass plan | `runtime.backtest.experiment_sweep_import_pass` | 抽离方案，固定四文件 import rewrite | BE-001DI 抽离方案 | `no code movement`；下一步只能进入 BE-001DI-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/341-runtime.backtest.experiment_sweep_import_pass抽离记录.md` runtime backtest experiment sweep import pass extraction | `runtime.backtest.experiment_sweep_import_pass` | 实际抽离，四文件 parent import 收敛 | BE-001DI 实际抽离 | 依赖文件数从 32 降为 28；下一步只能进入 BE-001DI-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md` runtime backtest experiment sweep import pass closeout | `runtime.backtest.experiment_sweep_import_pass` | 单叶 closeout，确认四文件 pocket 不继续拆微叶 | BE-001DI 单叶 closeout | `runtime.backtest.experiment_sweep_import_pass stop_split: true`；下一步只能进入 BE-001DJ-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/343-runtime.backtest_import_pass第三轮父叶残余判断.md` runtime backtest import pass third residual judgment | `runtime.backtest_import_pass` | 第三轮父叶残余判断，确认 backtest 剩余 5 个 execution_start 组 import 依赖文件 | BE-001DJ 父叶残余判断 | `runtime.backtest_import_pass stop_split: false`；下一步只能进入 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/344-runtime.backtest.execution_start_import_pass单子叶等价基线.md` runtime backtest execution start import pass baseline | `runtime.backtest.execution_start_import_pass` | 单子叶等价基线，冻结 execution_start 五文件 import pocket | BE-001DK 单子叶基线 | `no code movement`；下一步只能进入 BE-001DK-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/345-runtime.backtest.execution_start_import_pass抽离方案.md` runtime backtest execution start import pass plan | `runtime.backtest.execution_start_import_pass` | 抽离方案，固定五文件 import rewrite | BE-001DK 抽离方案 | `no code movement`；下一步只能进入 BE-001DK-03 实际抽离 |
| `markdown/06-milestones/v4.16.0/346-runtime.backtest.execution_start_import_pass抽离记录.md` runtime backtest execution start import pass extraction | `runtime.backtest.execution_start_import_pass` | 实际抽离，五文件 parent import 收敛 | BE-001DK 实际抽离 | 依赖文件数从 28 降为 23；backtest residual 为 0；下一步只能进入 BE-001DK-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/347-runtime.backtest.execution_start_import_pass单叶closeout.md` runtime backtest execution start import pass closeout | `runtime.backtest.execution_start_import_pass` | 单叶 closeout，确认五文件 pocket 不继续拆微叶 | BE-001DK 单叶 closeout | `runtime.backtest.execution_start_import_pass stop_split: true`；下一步只能进入 BE-001DL-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/348-runtime.backtest_import_pass第四轮父叶残余判断.md` runtime backtest import pass fourth residual judgment | `runtime.backtest_import_pass` | 第四轮父叶残余判断，确认 backtest residual 清零 | BE-001DL 父叶残余判断 | `runtime.backtest_import_pass stop_split: true`；下一步只能进入 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/349-runtime.parent_import_bridge父叶残余判断.md` runtime parent import bridge residual judgment | `runtime.parent_import_bridge` | 父叶残余判断，选择 mutation import pass | BE-001DM 父叶残余判断 | `runtime.parent_import_bridge stop_split: false`；下一步只能进入 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/350-runtime.mutation_import_pass单子叶等价基线.md` runtime mutation import pass baseline | `runtime.mutation_import_pass` | 单子叶等价基线，冻结 21 个 mutation parent bridge 文件 | BE-001DN 单子叶基线 | `no code movement`；下一步只能进入 BE-001DN-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/351-runtime.mutation_import_pass抽离方案.md` runtime mutation import pass plan | `runtime.mutation_import_pass` | 抽离方案，选择 shared_governance import pass | BE-001DN 抽离方案 | `runtime.mutation_import_pass stop_split: false`；下一步只能进入 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md` runtime mutation shared governance import pass baseline | `runtime.mutation.shared_governance_import_pass` | 单子叶等价基线，冻结 shared_governance import 输入面 | BE-001DO 单子叶基线 | `no code movement`；下一步只能进入 BE-001DO-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/353-runtime.mutation.shared_governance_import_pass抽离方案.md` runtime mutation shared governance import pass plan | `runtime.mutation.shared_governance_import_pass` | 抽离方案，固定单文件 explicit import rewrite | BE-001DO 抽离方案 | `no code movement`；下一步只能进入 BE-001DO-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/354-runtime.mutation.shared_governance_import_pass抽离记录.md` runtime mutation shared governance import pass record | `runtime.mutation.shared_governance_import_pass` | 实际抽离，改写 shared_governance.rs parent wildcard import | BE-001DO 抽离记录 | `actual_parent_import_bridge_23_to_22`；下一步只能进入 BE-001DO-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/355-runtime.mutation.shared_governance_import_pass单叶closeout.md` runtime mutation shared governance import pass closeout | `runtime.mutation.shared_governance_import_pass` | 单叶 closeout，确认不继续拆微叶 | BE-001DO 单叶 closeout | `runtime.mutation.shared_governance_import_pass stop_split: true`；下一步只能进入 BE-001DP-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/356-runtime.mutation_import_pass父叶残余判断.md` runtime mutation import pass residual judgment | `runtime.mutation_import_pass` | 父叶残余判断，选择 parameter_mutation import pass | BE-001DP 父叶残余判断 | `runtime.mutation_import_pass stop_split: false`；下一步只能进入 BE-001DQ-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md` runtime mutation parameter mutation import pass baseline | `runtime.mutation.parameter_mutation_import_pass` | 单子叶等价基线，冻结 10 个 parameter mutation residual 文件 | BE-001DQ 单子叶基线 | `no code movement`；下一步只能进入 BE-001DQ-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/358-runtime.mutation.parameter_mutation_import_pass抽离方案.md` runtime mutation parameter mutation import pass plan | `runtime.mutation.parameter_mutation_import_pass` | 抽离方案，拒绝 10 文件整批 rewrite 并选择 record_query import pass | BE-001DQ 抽离方案 | `runtime.mutation.parameter_mutation_import_pass stop_split: false`；下一步只能进入 BE-001DR-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md` runtime mutation parameter mutation record query import pass baseline | `runtime.mutation.parameter_mutation.record_query_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/record_query.rs` 读路径输入面 | BE-001DR 单子叶基线 | `no code movement`；下一步只能进入 BE-001DR-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md` runtime mutation parameter mutation record query import pass plan | `runtime.mutation.parameter_mutation.record_query_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001DR 抽离方案 | `single_file_record_query_import_rewrite`；下一步只能进入 BE-001DR-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/361-runtime.mutation.parameter_mutation.record_query_import_pass抽离记录.md` runtime mutation parameter mutation record query import pass record | `runtime.mutation.parameter_mutation.record_query_import_pass` | 实际抽离，改写 record_query parent wildcard import | BE-001DR 抽离记录 | `actual_parent_import_bridge_22_to_21`；下一步只能进入 BE-001DR-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/362-runtime.mutation.parameter_mutation.record_query_import_pass单叶closeout.md` runtime mutation parameter mutation record query import pass closeout | `runtime.mutation.parameter_mutation.record_query_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001DR 单叶 closeout | `record_query_import_pass_closeout_complete`；下一步只能进入 BE-001DS-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md` runtime mutation parameter mutation import pass parent residual judgment | `runtime.mutation.parameter_mutation_import_pass` | 父叶残余判断，保持 `stop_split: false` | BE-001DS 父叶判断 | `parameter_mutation_parent_residual_judgment_complete`；下一步只能进入 BE-001DT-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md` runtime mutation parameter mutation proposal creation import pass baseline | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 输入面 | BE-001DT 单子叶基线 | `proposal_creation_import_pass baseline_frozen`；下一步只能进入 BE-001DT-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md` runtime mutation parameter mutation proposal creation import pass plan | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001DT 抽离方案 | `single_file_proposal_creation_import_rewrite`；下一步只能进入 BE-001DT-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/366-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离记录.md` runtime mutation parameter mutation proposal creation import pass record | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 实际抽离，改写 proposal_creation parent wildcard import | BE-001DT 抽离记录 | `actual_parent_import_bridge_21_to_20`；下一步只能进入 BE-001DT-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/367-runtime.mutation.parameter_mutation.proposal_creation_import_pass单叶closeout.md` runtime mutation parameter mutation proposal creation import pass closeout | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001DT 单叶 closeout | `proposal_creation_import_pass_closeout_complete`；下一步只能进入 BE-001DU-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/368-runtime.mutation.parameter_mutation_import_pass第二轮父叶残余判断.md` runtime mutation parameter mutation import pass parent residual judgment round 2 | `runtime.mutation.parameter_mutation_import_pass` | 第二轮父叶残余判断，保持 `stop_split: false` | BE-001DU 父叶判断 | `transition_lifecycle_import_pass_selected`；下一步只能进入 BE-001DV-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 单子叶等价基线，冻结 7 文件 lifecycle 输入面 | BE-001DV 单子叶基线 | `transition_lifecycle_import_pass baseline_frozen`；下一步只能进入 BE-001DV-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 抽离方案，拒绝 7 文件同批 rewrite 并选择 boundary_safety import pass | BE-001DV 抽离方案 | `boundary_safety_import_pass_selected`；下一步只能进入 BE-001DW-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle boundary safety import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 输入面 | BE-001DW 单子叶基线 | `boundary_safety_import_pass baseline_frozen`；下一步只能进入 BE-001DW-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle boundary safety import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 抽离方案，固定 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` 单文件 import rewrite | BE-001DW 抽离方案 | `boundary_safety_import_pass plan_frozen`；下一步只能进入 BE-001DW-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/373-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle boundary safety import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` parent wildcard import | BE-001DW 抽离记录 | `boundary_safety_import_pass extraction_complete`；下一步只能进入 BE-001DW-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/374-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle boundary safety import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` | 单叶 closeout，设置 `stop_split: true` 并回到父叶残余判断 | BE-001DW 单叶 closeout | `boundary_safety_import_pass_closeout_complete`；下一步只能进入 BE-001DX-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/375-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 rollback_record_identity import pass | BE-001DX 父叶判断 | `rollback_record_identity_import_pass_selected`；下一步只能进入 BE-001DY-01 单子叶等价基线 |

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
| `markdown/06-milestones/v4.16.0/376-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle rollback record identity import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 输入面 | BE-001DY 单子叶基线 | `rollback_record_identity_import_pass baseline_frozen`；下一步只能进入 BE-001DY-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/377-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle rollback record identity import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | 抽离方案，固定 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 单文件 import rewrite | BE-001DY 抽离方案 | `single_file_rollback_record_identity_import_rewrite`；下一步只能进入 BE-001DY-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/378-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle rollback record identity import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` parent wildcard import | BE-001DY 抽离记录 | `rollback_record_identity_import_pass extraction_complete`；下一步只能进入 BE-001DY-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/379-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle rollback record identity import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001DY 单叶 closeout | `rollback_record_identity_import_pass_closeout_complete`；下一步只能进入 BE-001DZ-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/380-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 transition_record_persistence import pass | BE-001DZ 父叶判断 | `transition_record_persistence_import_pass_selected`；下一步只能进入 BE-001EA-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/381-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle transition record persistence import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 输入面 | BE-001EA 单子叶基线 | `transition_record_persistence_import_pass baseline_frozen`；下一步只能进入 BE-001EA-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/382-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle transition record persistence import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 抽离方案，固定 BE-001EA-03 只能改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 顶部 import | BE-001EA 抽离方案 | `transition_record_persistence_import_pass plan_frozen`；下一步只能进入 BE-001EA-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/383-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle transition record persistence import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` parent wildcard import | BE-001EA 抽离记录 | `transition_record_persistence_import_pass extraction_complete`；下一步只能进入 BE-001EA-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/384-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle transition record persistence import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EA 单叶 closeout | `transition_record_persistence_import_pass_closeout_complete`；下一步只能进入 BE-001EB-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/385-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第三轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass third parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 activation_snapshot_side_effect import pass | BE-001EB 父叶判断 | `activation_snapshot_side_effect_import_pass_selected`；下一步只能进入 BE-001EC-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/386-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 输入面 | BE-001EC 单子叶基线 | `activation_snapshot_side_effect_import_pass baseline_frozen`；下一步只能进入 BE-001EC-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/387-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 抽离方案，固定 BE-001EC-03 只能改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 顶部 import | BE-001EC 抽离方案 | `activation_snapshot_side_effect_import_pass plan_frozen`；下一步只能进入 BE-001EC-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/388-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` parent wildcard import | BE-001EC 抽离记录 | `activation_snapshot_side_effect_import_pass extraction_complete`；下一步只能进入 BE-001EC-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/389-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle activation snapshot side effect import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EC 单叶 closeout | `activation_snapshot_side_effect_import_pass_closeout_complete`；下一步只能进入 BE-001ED-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/390-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第四轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass fourth parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 activation_flow import pass | BE-001ED 父叶判断 | `activation_flow_import_pass_selected`；下一步只能进入 BE-001EE-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/391-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle activation flow import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 输入面 | BE-001EE 单子叶基线 | `activation_flow_import_pass baseline_frozen`；下一步只能进入 BE-001EE-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/392-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle activation flow import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 抽离方案，固定 BE-001EE-03 只能改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 顶部 import | BE-001EE 抽离方案 | `activation_flow_import_pass plan_frozen`；下一步只能进入 BE-001EE-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/393-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle activation flow import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` parent wildcard import | BE-001EE 抽离记录 | `activation_flow_import_pass extraction_complete`；下一步只能进入 BE-001EE-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/394-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle activation flow import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EE 单叶 closeout | `activation_flow_import_pass_closeout_complete`；下一步只能进入 BE-001EF-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/395-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第五轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass fifth parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 rollback_flow import pass | BE-001EF 父叶判断 | `rollback_flow_import_pass_selected`；下一步只能进入 BE-001EG-01 单子叶等价基线 |
| `markdown/06-milestones/v4.16.0/396-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle rollback flow import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 输入面 | BE-001EG 单子叶基线 | `rollback_flow_import_pass baseline_frozen`；下一步只能进入 BE-001EG-02 抽离方案 |
| `markdown/06-milestones/v4.16.0/397-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle rollback flow import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 抽离方案，固定 BE-001EG-03 只能改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 顶部 import | BE-001EG 抽离方案 | `rollback_flow_import_pass plan_frozen`；下一步只能进入 BE-001EG-03 实际抽离记录 |
| `markdown/06-milestones/v4.16.0/398-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle rollback flow import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` parent wildcard import | BE-001EG 抽离记录 | `rollback_flow_import_pass extraction_complete`；下一步只能进入 BE-001EG-04 单叶 closeout |
| `markdown/06-milestones/v4.16.0/399-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle rollback flow import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EG 单叶 closeout | `rollback_flow_import_pass_closeout_complete`；下一步只能进入 BE-001EH-01 父叶残余判断 |
| `markdown/06-milestones/v4.16.0/400-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第六轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass sixth parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，保持 `stop_split: false` 并选择 parent_facade import pass | BE-001EH 父叶判断 | `transition_lifecycle_parent_facade_import_pass_selected`；下一步只能进入 BE-001EI-01 单子叶等价基线 |

**最新状态补充（BE-001DY-01）**: BE-001DY-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`，helper 为 `runtime_parameter_mutation_rollback_record_id`；下一步只能进入 BE-001DY-02 抽离方案。旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。
**最新状态补充（BE-001DY-02）**: BE-001DY-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离方案。下一步只允许 BE-001DY-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 顶部 import，不改函数体、可见性、parent facade、rollback flow 或 sibling。
**最新状态补充（BE-001DY-03）**: BE-001DY-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 已移除 `use super::*`，residual 降为 total 18 / mutation 16 / parameter_mutation 6 / transition_lifecycle 5；下一步只能进入 BE-001DY-04 单叶 closeout。
**最新状态补充（BE-001DY-04）**: BE-001DY-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass stop_split: true`。下一步只能进入 BE-001DZ-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。
**最新状态补充（BE-001DZ-01）**: BE-001DZ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，当前 transition_lifecycle residual 为 5 文件；下一步只能进入 BE-001EA-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线。
**最新状态补充（BE-001EA-01）**: BE-001EA-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`，helper 为 `mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition`；下一步只能进入 BE-001EA-02 抽离方案。
**最新状态补充（BE-001EA-02）**: BE-001EA-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案。下一步只允许 BE-001EA-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 顶部 import，不改函数体、可见性、parent facade、activation flow、rollback flow 或 sibling。
**最新状态补充（BE-001EA-03）**: BE-001EA-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` 已移除 `use super::*`，residual 降为 total 17 / mutation 15 / parameter_mutation 5 / transition_lifecycle 4；下一步只能进入 BE-001EA-04 单叶 closeout。
**最新状态补充（BE-001EA-04）**: BE-001EA-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass stop_split: true`。下一步只能进入 BE-001EB-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。
**最新状态补充（BE-001EB-01）**: BE-001EB-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第三轮父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，当前 transition_lifecycle residual 为 4 文件；下一步只能进入 BE-001EC-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线。
**最新状态补充（BE-001EC-01）**: BE-001EC-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`，helper 为 `auto_snapshot_on_activation`；下一步只能进入 BE-001EC-02 抽离方案。
**最新状态补充（BE-001EC-02）**: BE-001EC-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案。下一步只允许 BE-001EC-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 顶部 import，不改函数体、可见性、parent facade、activation flow、rollback flow、snapshot persistence 或 sibling。
**最新状态补充（BE-001EC-03）**: BE-001EC-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs` 已移除 `use super::*`，residual 降为 total 16 / mutation 14 / parameter_mutation 4 / transition_lifecycle 3；下一步只能进入 BE-001EC-04 单叶 closeout。
**最新状态补充（BE-001EC-04）**: BE-001EC-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass stop_split: true`。下一步只能进入 BE-001ED-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。
**最新状态补充（BE-001ED-01）**: BE-001ED-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，当前 transition_lifecycle residual 为 3 文件；下一步只能进入 BE-001EE-01 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线。
**最新状态补充（BE-001EE-01）**: BE-001EE-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`，handler 为 `activate_runtime_parameter_mutation`；下一步只能进入 BE-001EE-02 抽离方案。
**最新状态补充（BE-001EE-02）**: BE-001EE-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案。下一步只允许 BE-001EE-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 顶部 import，不改函数体、可见性、parent facade、rollback flow、snapshot side effect 或 sibling。
**最新状态补充（BE-001EE-03）**: BE-001EE-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 已移除 `use super::*` 并改为显式输入面，函数体、可见性、parent facade、rollback flow、snapshot side effect 与 sibling 均未改；下一步只能进入 BE-001EE-04 单叶 closeout。
**最新状态补充（BE-001EE-04）**: BE-001EE-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单叶 closeout。该 import pocket 设置 `stop_split: true`，不继续拆 safe-window denied / scheduled / activated / failed 微叶；下一步只能进入 BE-001EF-01 `transition_lifecycle_import_pass` 父叶残余判断。
**最新状态补充（BE-001EF-01）**: BE-001EF-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第五轮父叶残余判断。当前 residual 为 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 与 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`；父叶保持 `stop_split: false`，下一步只能进入 BE-001EG-01 `rollback_flow_import_pass` 单子叶等价基线。
**最新状态补充（BE-001EG-01）**: BE-001EG-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`，handler 为 `rollback_runtime_parameter_mutation`；下一步只能进入 BE-001EG-02 抽离方案。
**最新状态补充（BE-001EG-02）**: BE-001EG-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离方案。下一步只允许 BE-001EG-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 顶部 import，不改函数体、可见性、parent facade、activation flow、snapshot side effect 或 sibling。
**最新状态补充（BE-001EG-03）**: BE-001EG-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 已移除 `use super::*` 并改为显式输入面，函数体、可见性、parent facade、activation flow、snapshot side effect 与 sibling 均未改；下一步只能进入 BE-001EG-04 单叶 closeout。
**最新状态补充（BE-001EG-04）**: BE-001EG-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass stop_split: true`。下一步只能进入 BE-001EH-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。
**最新状态补充（BE-001EH-01）**: BE-001EH-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断。父叶保持 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: false`，当前 transition_lifecycle residual 为 1 文件；下一步只能进入 BE-001EI-01 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线。
**最新状态补充（BE-001EI-01）**: BE-001EI-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线。当前 `no code movement`，冻结文件为 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`，parent facade 仍保留 `use super::*`；下一步只能进入 BE-001EI-02 抽离方案。

| `markdown/06-milestones/v4.16.0/401-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass单子叶等价基线.md` runtime mutation parameter mutation transition lifecycle parent facade import pass baseline | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` parent facade 输入面 | BE-001EI 单子叶基线 | `parent_facade_import_pass baseline_frozen`；下一步只能进入 BE-001EI-02 抽离方案 |
**最新状态补充（BE-001EI-02）**: BE-001EI-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案。下一步只允许 BE-001EI-03 改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 顶部 import，不改 child module declaration、re-export、helper import、wrapper、sibling 或 release transition。

| `markdown/06-milestones/v4.16.0/402-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离方案.md` runtime mutation parameter mutation transition lifecycle parent facade import pass plan | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 抽离方案，固定 BE-001EI-03 只能改写 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 顶部 import | BE-001EI 抽离方案 | `parent_facade_import_pass plan_frozen`；下一步只能进入 BE-001EI-03 实际抽离记录 |
**最新状态补充（BE-001EI-03）**: BE-001EI-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 已移除 `use super::*` 并改为显式输入面；函数体、可见性、child module declaration、re-export、helper import 与 sibling 均未改。下一步只能进入 BE-001EI-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/403-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass抽离记录.md` runtime mutation parameter mutation transition lifecycle parent facade import pass extraction | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` parent wildcard import | BE-001EI 抽离记录 | `parent_facade_import_pass extraction_complete`；下一步只能进入 BE-001EI-04 单叶 closeout |
**最新状态补充（BE-001EI-04）**: BE-001EI-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass stop_split: true`。下一步只能进入 BE-001EJ-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。

| `markdown/06-milestones/v4.16.0/404-runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass单叶closeout.md` runtime mutation parameter mutation transition lifecycle parent facade import pass closeout | `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EI 单叶 closeout | `parent_facade_import_pass_closeout_complete`；下一步只能进入 BE-001EJ-01 父叶残余判断 |
**最新状态补充（BE-001EJ-01）**: BE-001EJ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第七轮父叶残余判断，并设置 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass stop_split: true`。当前上层 residual 为 `src/runtime/mutation/parameter_mutation.rs`；下一步只能进入 BE-001EK-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。

| `markdown/06-milestones/v4.16.0/405-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass第七轮父叶残余判断.md` runtime mutation parameter mutation transition lifecycle import pass seventh parent residual judgment | `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` | 父叶残余判断，设置 `stop_split: true` | BE-001EJ 父叶判断 | `transition_lifecycle_import_pass seventh_parent_residual_judgment`；下一步只能进入 BE-001EK-01 父叶残余判断 |
**最新状态补充（BE-001EK-01）**: BE-001EK-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第三轮父叶残余判断。当前唯一 `parameter_mutation` residual 为 `src/runtime/mutation/parameter_mutation.rs`，父叶保持 `runtime.mutation.parameter_mutation_import_pass stop_split: false`；下一步只能进入 BE-001EL-01 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/406-runtime.mutation.parameter_mutation_import_pass第三轮父叶残余判断.md` runtime mutation parameter mutation import pass third parent residual judgment | `runtime.mutation.parameter_mutation_import_pass` | 父叶残余判断，选择 parent facade import pass | BE-001EK 父叶判断 | `parameter_mutation_import_pass third_parent_residual_judgment`；下一步只能进入 BE-001EL-01 单子叶等价基线 |
**最新状态补充（BE-001EL-01）**: BE-001EL-01 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/parameter_mutation.rs` 的 child module declaration、public handler re-export、private helper alias 与预期空显式输入面；下一步只能进入 BE-001EL-02 抽离方案。

| `markdown/06-milestones/v4.16.0/407-runtime.mutation.parameter_mutation.parent_facade_import_pass单子叶等价基线.md` runtime mutation parameter mutation parent facade import pass baseline | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 单子叶等价基线，冻结 `src/runtime/mutation/parameter_mutation.rs` parent facade 输入面 | BE-001EL 单子叶基线 | `parent_facade_import_pass baseline_frozen`；下一步只能进入 BE-001EL-02 抽离方案 |
**最新状态补充（BE-001EL-02）**: BE-001EL-02 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案。下一步只允许 BE-001EL-03 删除 `src/runtime/mutation/parameter_mutation.rs` 顶部 `use super::*`，不新增替代 import，不改 child module declaration、public re-export、private helper alias、sibling 或 release transition。

| `markdown/06-milestones/v4.16.0/408-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离方案.md` runtime mutation parameter mutation parent facade import pass plan | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 抽离方案，固定 BE-001EL-03 单文件 import rewrite | BE-001EL 抽离方案 | `parent_facade_import_pass plan_frozen`；下一步只能进入 BE-001EL-03 实际抽离记录 |
**最新状态补充（BE-001EL-03）**: BE-001EL-03 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 实际抽离。`src/runtime/mutation/parameter_mutation.rs` 已移除 `use super::*` 并改为显式 `use super::mutation_event_contract;`；child module declaration、public re-export、private helper alias 与 child files 均未改。下一步只能进入 BE-001EL-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/409-runtime.mutation.parameter_mutation.parent_facade_import_pass抽离记录.md` runtime mutation parameter mutation parent facade import pass extraction | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 实际抽离，清理 `src/runtime/mutation/parameter_mutation.rs` parent wildcard import | BE-001EL 抽离记录 | `parent_facade_import_pass extraction_complete`；下一步只能进入 BE-001EL-04 单叶 closeout |
**最新状态补充（BE-001EL-04）**: BE-001EL-04 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout，并设置 `runtime.mutation.parameter_mutation.parent_facade_import_pass stop_split: true`。下一步只能进入 BE-001EM-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。

| `markdown/06-milestones/v4.16.0/410-runtime.mutation.parameter_mutation.parent_facade_import_pass单叶closeout.md` runtime mutation parameter mutation parent facade import pass closeout | `runtime.mutation.parameter_mutation.parent_facade_import_pass` | 单叶 closeout，设置 `stop_split: true` | BE-001EL 单叶 closeout | `parent_facade_import_pass_closeout_complete`；下一步只能进入 BE-001EM-01 父叶残余判断 |
**最新状态补充（BE-001EM-01）**: BE-001EM-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第四轮父叶残余判断，并设置 `runtime.mutation.parameter_mutation_import_pass stop_split: true`。下一步只能进入 BE-001EN-01 `runtime.mutation_import_pass` 父叶残余判断。

| `markdown/06-milestones/v4.16.0/411-runtime.mutation.parameter_mutation_import_pass第四轮父叶残余判断.md` runtime mutation parameter mutation import pass fourth parent residual judgment | `runtime.mutation.parameter_mutation_import_pass` | 父叶残余判断，设置 `stop_split: true` | BE-001EM 父叶收口 | `parameter_mutation_import_pass fourth_parent_residual_judgment`；下一步只能进入 BE-001EN-01 父叶残余判断 |
**最新状态补充（BE-001EN-01）**: BE-001EN-01 已完成 `runtime.mutation_import_pass` 第二轮父叶残余判断。当前 mutation residual 为 10 个 ai proposal 文件，父叶保持 `runtime.mutation_import_pass stop_split: false`；下一步只能进入 BE-001EO-01 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/412-runtime.mutation_import_pass第二轮父叶残余判断.md` runtime mutation import pass second parent residual judgment | `runtime.mutation_import_pass` | 父叶残余判断，选择 ai_proposal import pass | BE-001EN 父叶重判 | `runtime.mutation_import_pass second_parent_residual_judgment`；下一步只能进入 BE-001EO-01 单子叶等价基线 |
**最新状态补充（BE-001EO-01）**: BE-001EO-01 已建立 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 10 个 ai proposal residual 文件、8 个 route-facing handler 与 18 个内部 helper；下一步只能进入 BE-001EO-02 抽离方案。

| `markdown/06-milestones/v4.16.0/413-runtime.mutation.ai_proposal_import_pass单子叶等价基线.md` runtime mutation ai proposal import pass baseline | `runtime.mutation.ai_proposal_import_pass` | 单子叶等价基线，冻结 ai proposal import 输入面 | BE-001EO 单子叶基线 | `ai_proposal_import_pass baseline_frozen`；下一步只能进入 BE-001EO-02 抽离方案 |
**最新状态补充（BE-001EO-02）**: BE-001EO-02 已建立 `runtime.mutation.ai_proposal_import_pass` 抽离方案。父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`，拒绝 10 文件整批 rewrite；下一步只能进入 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/414-runtime.mutation.ai_proposal_import_pass抽离方案.md` runtime mutation ai proposal import pass plan | `runtime.mutation.ai_proposal_import_pass` | 抽离方案，选择 record_query import pass | BE-001EO 抽离方案 | `reject_ai_proposal_bulk_rewrite_10_files`；下一步只能进入 BE-001EP-01 单子叶等价基线 |
**最新状态补充（BE-001EP-01）**: BE-001EP-01 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/record_query.rs` 的 3 个函数与隐式输入面；下一步只能进入 BE-001EP-02 抽离方案。

| `markdown/06-milestones/v4.16.0/415-runtime.mutation.ai_proposal.record_query_import_pass单子叶等价基线.md` runtime mutation ai proposal record query import pass baseline | `runtime.mutation.ai_proposal.record_query_import_pass` | 单子叶等价基线，冻结 record_query 输入面 | BE-001EP 单子叶基线 | `runtime.mutation.ai_proposal.record_query_import_pass baseline_frozen`；下一步只能进入 BE-001EP-02 抽离方案 |
**最新状态补充（BE-001EP-02）**: BE-001EP-02 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案。当前 `no code movement`，固定 BE-001EP-03 只能改写 `src/runtime/mutation/ai_proposal/record_query.rs` 顶部 import；下一步只能进入 BE-001EP-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/416-runtime.mutation.ai_proposal.record_query_import_pass抽离方案.md` runtime mutation ai proposal record query import pass plan | `runtime.mutation.ai_proposal.record_query_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001EP 抽离方案 | `record_query_import_pass plan_frozen`；下一步只能进入 BE-001EP-03 实际抽离记录 |
**最新状态补充（BE-001EP-03）**: BE-001EP-03 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/record_query.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001EP-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/417-runtime.mutation.ai_proposal.record_query_import_pass抽离记录.md` runtime mutation ai proposal record query import pass extraction | `runtime.mutation.ai_proposal.record_query_import_pass` | 实际抽离，record_query import 显式化 | BE-001EP 实际抽离 | `record_query_import_pass extraction_done`；下一步只能进入 BE-001EP-04 单叶 closeout |
**最新状态补充（BE-001EP-04）**: BE-001EP-04 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.record_query_import_pass stop_split: true`；下一步只能进入 BE-001EQ-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/418-runtime.mutation.ai_proposal.record_query_import_pass单叶closeout.md` runtime mutation ai proposal record query import pass closeout | `runtime.mutation.ai_proposal.record_query_import_pass` | 单叶 closeout，设置 stop_split true | BE-001EP 单叶收口 | `runtime.mutation.ai_proposal.record_query_import_pass closeout_done`；下一步只能进入 BE-001EQ-01 父叶残余判断 |
**最新状态补充（BE-001EQ-01）**: BE-001EQ-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第三轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/419-runtime.mutation.ai_proposal_import_pass第三轮父叶残余判断.md` runtime mutation ai proposal import pass third parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 source_governance_identity import pass | BE-001EQ 父叶重判 | `runtime.mutation.ai_proposal_import_pass third_parent_residual_judgment`；下一步只能进入 BE-001ER-01 单子叶等价基线 |
**最新状态补充（BE-001ER-01）**: BE-001ER-01 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 的 source context、governance projection、deterministic record id 与隐式输入面；下一步只能进入 BE-001ER-02 抽离方案。

| `markdown/06-milestones/v4.16.0/420-runtime.mutation.ai_proposal.source_governance_identity_import_pass单子叶等价基线.md` runtime mutation ai proposal source governance identity import pass baseline | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 单子叶等价基线，冻结 source governance 输入面 | BE-001ER 单子叶基线 | `runtime.mutation.ai_proposal.source_governance_identity_import_pass baseline_frozen`；下一步只能进入 BE-001ER-02 抽离方案 |
**最新状态补充（BE-001ER-02）**: BE-001ER-02 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案。当前 `no code movement`，固定 BE-001ER-03 只能改写 `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 顶部 import；下一步只能进入 BE-001ER-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/421-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离方案.md` runtime mutation ai proposal source governance identity import pass plan | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001ER 抽离方案 | `source_governance_identity_import_pass plan_frozen`；下一步只能进入 BE-001ER-03 实际抽离记录 |
**最新状态补充（BE-001ER-03）**: BE-001ER-03 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/source_governance_identity.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001ER-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/422-runtime.mutation.ai_proposal.source_governance_identity_import_pass抽离记录.md` runtime mutation ai proposal source governance identity import pass extraction | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 实际抽离，source governance import 显式化 | BE-001ER 实际抽离 | `source_governance_identity_import_pass extraction_done`；下一步只能进入 BE-001ER-04 单叶 closeout |
**最新状态补充（BE-001ER-04）**: BE-001ER-04 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.source_governance_identity_import_pass stop_split: true`；下一步只能进入 BE-001ES-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/423-runtime.mutation.ai_proposal.source_governance_identity_import_pass单叶closeout.md` runtime mutation ai proposal source governance identity import pass closeout | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 单叶 closeout，设置 stop_split true | BE-001ER 单叶收口 | `runtime.mutation.ai_proposal.source_governance_identity_import_pass closeout_done`；下一步只能进入 BE-001ES-01 父叶残余判断 |
**最新状态补充（BE-001ES-01）**: BE-001ES-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/424-runtime.mutation.ai_proposal_import_pass第四轮父叶残余判断.md` runtime mutation ai proposal import pass fourth parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 static_check import pass | BE-001ES 父叶重判 | `runtime.mutation.ai_proposal_import_pass fourth_parent_residual_judgment`；下一步只能进入 BE-001ET-01 单子叶等价基线 |
**最新状态补充（BE-001ET-01）**: BE-001ET-01 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/static_check.rs` 的静态校验输入面；下一步只能进入 BE-001ET-02 抽离方案。

| `markdown/06-milestones/v4.16.0/425-runtime.mutation.ai_proposal.static_check_import_pass单子叶等价基线.md` runtime mutation ai proposal static check import pass baseline | `runtime.mutation.ai_proposal.static_check_import_pass` | 单子叶等价基线，冻结 static check 输入面 | BE-001ET 单子叶基线 | `runtime.mutation.ai_proposal.static_check_import_pass baseline_frozen`；下一步只能进入 BE-001ET-02 抽离方案 |
**最新状态补充（BE-001ET-02）**: BE-001ET-02 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案。当前 `no code movement`，BE-001ET-03 只能改写 `src/runtime/mutation/ai_proposal/static_check.rs` 顶部 import；下一步只能进入 BE-001ET-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/426-runtime.mutation.ai_proposal.static_check_import_pass抽离方案.md` runtime mutation ai proposal static check import pass plan | `runtime.mutation.ai_proposal.static_check_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001ET 抽离方案 | `static_check_import_pass plan_frozen`；下一步只能进入 BE-001ET-03 实际抽离记录 |
**最新状态补充（BE-001ET-03）**: BE-001ET-03 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/static_check.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001ET-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/427-runtime.mutation.ai_proposal.static_check_import_pass抽离记录.md` runtime mutation ai proposal static check import pass extraction | `runtime.mutation.ai_proposal.static_check_import_pass` | 实际抽离，static_check import 显式化 | BE-001ET 实际抽离 | `static_check_import_pass extraction_done`；下一步只能进入 BE-001ET-04 单叶 closeout |
**最新状态补充（BE-001ET-04）**: BE-001ET-04 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.static_check_import_pass stop_split: true`；下一步只能进入 BE-001EU-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/428-runtime.mutation.ai_proposal.static_check_import_pass单叶closeout.md` runtime mutation ai proposal static check import pass closeout | `runtime.mutation.ai_proposal.static_check_import_pass` | 单叶 closeout，设置 stop_split true | BE-001ET 单叶收口 | `runtime.mutation.ai_proposal.static_check_import_pass closeout_done`；下一步只能进入 BE-001EU-01 父叶残余判断 |
**最新状态补充（BE-001EU-01）**: BE-001EU-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/429-runtime.mutation.ai_proposal_import_pass第五轮父叶残余判断.md` runtime mutation ai proposal import pass fifth parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 event_lifecycle import pass | BE-001EU 父叶重判 | `runtime.mutation.ai_proposal_import_pass fifth_parent_residual_judgment`；下一步只能进入 BE-001EV-01 单子叶等价基线 |
**最新状态补充（BE-001EV-01）**: BE-001EV-01 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/event_lifecycle.rs` 的事件 contract、payload、lifecycle entry 与 transition persistence 边界；下一步只能进入 BE-001EV-02 抽离方案。

| `markdown/06-milestones/v4.16.0/430-runtime.mutation.ai_proposal.event_lifecycle_import_pass单子叶等价基线.md` runtime mutation ai proposal event lifecycle import pass baseline | `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | 单子叶等价基线，冻结 event lifecycle 输入面 | BE-001EV 单子叶基线 | `runtime.mutation.ai_proposal.event_lifecycle_import_pass baseline_frozen`；下一步只能进入 BE-001EV-02 抽离方案 |
**最新状态补充（BE-001EV-02）**: BE-001EV-02 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案。当前 `no code movement`，BE-001EV-03 只能改写 `src/runtime/mutation/ai_proposal/event_lifecycle.rs` 顶部 import；下一步只能进入 BE-001EV-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/431-runtime.mutation.ai_proposal.event_lifecycle_import_pass抽离方案.md` runtime mutation ai proposal event lifecycle import pass plan | `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001EV 抽离方案 | `event_lifecycle_import_pass plan_frozen`；下一步只能进入 BE-001EV-03 实际抽离记录 |
**最新状态补充（BE-001EV-03）**: BE-001EV-03 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/event_lifecycle.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001EV-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/432-runtime.mutation.ai_proposal.event_lifecycle_import_pass抽离记录.md` runtime mutation ai proposal event lifecycle import pass extraction | `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | 实际抽离，event_lifecycle import 显式化 | BE-001EV 实际抽离 | `event_lifecycle_import_pass extraction_done`；下一步只能进入 BE-001EV-04 单叶 closeout |
**最新状态补充（BE-001EV-04）**: BE-001EV-04 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.event_lifecycle_import_pass stop_split: true`；下一步只能进入 BE-001EW-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/433-runtime.mutation.ai_proposal.event_lifecycle_import_pass单叶closeout.md` runtime mutation ai proposal event lifecycle import pass closeout | `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | 单叶 closeout，设置 stop_split true | BE-001EV 单叶收口 | `runtime.mutation.ai_proposal.event_lifecycle_import_pass closeout_done`；下一步只能进入 BE-001EW-01 父叶残余判断 |
**最新状态补充（BE-001EW-01）**: BE-001EW-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001EX-01 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/434-runtime.mutation.ai_proposal_import_pass第六轮父叶残余判断.md` runtime mutation ai proposal import pass sixth parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 approval_persistence import pass | BE-001EW 父叶重判 | `runtime.mutation.ai_proposal_import_pass sixth_parent_residual_judgment`；下一步只能进入 BE-001EX-01 单子叶等价基线 |
**最新状态补充（BE-001EX-01）**: BE-001EX-01 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/approval_persistence.rs` 的审批持久化输入面、atomic write、not_found 映射与 decode error 映射；下一步只能进入 BE-001EX-02 抽离方案。

| `markdown/06-milestones/v4.16.0/435-runtime.mutation.ai_proposal.approval_persistence_import_pass单子叶等价基线.md` runtime mutation ai proposal approval persistence import pass baseline | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 单子叶等价基线，冻结 approval persistence 输入面 | BE-001EX 单子叶基线 | `runtime.mutation.ai_proposal.approval_persistence_import_pass baseline_frozen`；下一步只能进入 BE-001EX-02 抽离方案 |
**最新状态补充（BE-001EX-02）**: BE-001EX-02 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案。当前 `no code movement`，BE-001EX-03 只能改写 `src/runtime/mutation/ai_proposal/approval_persistence.rs` 顶部 import；下一步只能进入 BE-001EX-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/436-runtime.mutation.ai_proposal.approval_persistence_import_pass抽离方案.md` runtime mutation ai proposal approval persistence import pass plan | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001EX 抽离方案 | `approval_persistence_import_pass plan_frozen`；下一步只能进入 BE-001EX-03 实际抽离记录 |
**最新状态补充（BE-001EX-03）**: BE-001EX-03 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/approval_persistence.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001EX-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/437-runtime.mutation.ai_proposal.approval_persistence_import_pass抽离记录.md` runtime mutation ai proposal approval persistence import pass extraction | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 实际抽离，approval_persistence import 显式化 | BE-001EX 实际抽离 | `approval_persistence_import_pass extraction_done`；下一步只能进入 BE-001EX-04 单叶 closeout |
**最新状态补充（BE-001EX-04）**: BE-001EX-04 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.approval_persistence_import_pass stop_split: true`；下一步只能进入 BE-001EY-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/438-runtime.mutation.ai_proposal.approval_persistence_import_pass单叶closeout.md` runtime mutation ai proposal approval persistence import pass closeout | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 单叶 closeout，设置 stop_split true | BE-001EX 单叶收口 | `runtime.mutation.ai_proposal.approval_persistence_import_pass closeout_done`；下一步只能进入 BE-001EY-01 父叶残余判断 |
**最新状态补充（BE-001EY-01）**: BE-001EY-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001EZ-01 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线。

| `markdown/06-milestones/v4.16.0/439-runtime.mutation.ai_proposal_import_pass第七轮父叶残余判断.md` runtime mutation ai proposal import pass seventh parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 status_transition import pass | BE-001EY 父叶重判 | `status_transition_import_pass_selected`；下一步只能进入 BE-001EZ-01 单子叶等价基线 |
**最新状态补充（BE-001EZ-01）**: BE-001EZ-01 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal/status_transition.rs` 的 Approved 映射、状态转换表、状态写入顺序和 invalid transition 日志；下一步只能进入 BE-001EZ-02 抽离方案。

| `markdown/06-milestones/v4.16.0/440-runtime.mutation.ai_proposal.status_transition_import_pass单子叶等价基线.md` runtime mutation ai proposal status transition import pass baseline | `runtime.mutation.ai_proposal.status_transition_import_pass` | 单子叶等价基线，冻结 status transition 输入面 | BE-001EZ 单子叶基线 | `runtime.mutation.ai_proposal.status_transition_import_pass baseline_frozen`；下一步只能进入 BE-001EZ-02 抽离方案 |
**最新状态补充（BE-001EZ-02）**: BE-001EZ-02 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案。当前 `no code movement`，BE-001EZ-03 只能改写 `src/runtime/mutation/ai_proposal/status_transition.rs` 顶部 import；下一步只能进入 BE-001EZ-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/441-runtime.mutation.ai_proposal.status_transition_import_pass抽离方案.md` runtime mutation ai proposal status transition import pass plan | `runtime.mutation.ai_proposal.status_transition_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001EZ 抽离方案 | `status_transition_import_pass plan_frozen`；下一步只能进入 BE-001EZ-03 实际抽离记录 |
**最新状态补充（BE-001EZ-03）**: BE-001EZ-03 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/status_transition.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001EZ-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/442-runtime.mutation.ai_proposal.status_transition_import_pass抽离记录.md` runtime mutation ai proposal status transition import pass extraction | `runtime.mutation.ai_proposal.status_transition_import_pass` | 实际抽离，status_transition import 显式化 | BE-001EZ 实际抽离 | `status_transition_import_pass extraction_done`；下一步只能进入 BE-001EZ-04 单叶 closeout |
**最新状态补充（BE-001EZ-04）**: BE-001EZ-04 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.status_transition_import_pass stop_split: true`；下一步只能进入 BE-001FA-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/443-runtime.mutation.ai_proposal.status_transition_import_pass单叶closeout.md` runtime mutation ai proposal status transition import pass closeout | `runtime.mutation.ai_proposal.status_transition_import_pass` | 单叶 closeout，设置 stop_split true | BE-001EZ 单叶收口 | `runtime.mutation.ai_proposal.status_transition_import_pass closeout_done`；下一步只能进入 BE-001FA-01 父叶残余判断 |

**最新状态补充（BE-001FA-01）**: BE-001FA-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001FB-01 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 等价基线。

| `markdown/06-milestones/v4.16.0/444-runtime.mutation.ai_proposal_import_pass第八轮父叶残余判断.md` runtime mutation ai proposal import pass eighth parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 sandbox_trigger import pass | BE-001FA 父叶重判 | `sandbox_trigger_import_pass_selected`；下一步只能进入 BE-001FB-01 等价基线 |

**最新状态补充（BE-001FB-01）**: BE-001FB-01 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线。当前 `no code movement`，冻结 sandbox gate、async retry side effect、approval lifecycle 与 persistence order；下一步只能进入 BE-001FB-02 抽离方案。

| `markdown/06-milestones/v4.16.0/445-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单子叶等价基线.md` runtime mutation ai proposal sandbox trigger import pass baseline | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 单子叶等价基线，冻结 sandbox trigger 输入面 | BE-001FB 单子叶基线 | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass baseline_frozen`；下一步只能进入 BE-001FB-02 抽离方案 |

**最新状态补充（BE-001FB-02）**: BE-001FB-02 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案。当前 `no code movement`，BE-001FB-03 只能改写 `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 顶部 import；下一步只能进入 BE-001FB-03 实际抽离记录。

| `markdown/06-milestones/v4.16.0/446-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离方案.md` runtime mutation ai proposal sandbox trigger import pass plan | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 抽离方案，固定单文件 import rewrite | BE-001FB 抽离方案 | `sandbox_trigger_import_pass plan_frozen`；下一步只能进入 BE-001FB-03 实际抽离记录 |

**最新状态补充（BE-001FB-03）**: BE-001FB-03 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 实际抽离。`src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 已移除 parent wildcard import 并改为显式输入面；下一步只能进入 BE-001FB-04 单叶 closeout。

| `markdown/06-milestones/v4.16.0/447-runtime.mutation.ai_proposal.sandbox_trigger_import_pass抽离记录.md` runtime mutation ai proposal sandbox trigger import pass extraction | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 实际抽离，sandbox_trigger import 显式化 | BE-001FB 实际抽离 | `sandbox_trigger_import_pass extraction_done`；下一步只能进入 BE-001FB-04 单叶 closeout |

**最新状态补充（BE-001FB-04）**: BE-001FB-04 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout。当前 `no code movement`，设置 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass stop_split: true`；下一步只能进入 BE-001FC-01 父叶残余判断。

| `markdown/06-milestones/v4.16.0/448-runtime.mutation.ai_proposal.sandbox_trigger_import_pass单叶closeout.md` runtime mutation ai proposal sandbox trigger import pass closeout | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | 单叶 closeout，设置 stop_split true | BE-001FB 单叶收口 | `runtime.mutation.ai_proposal.sandbox_trigger_import_pass closeout_done`；下一步只能进入 BE-001FC-01 父叶残余判断 |

**最新状态补充（BE-001FC-01）**: BE-001FC-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断。当前 `no code movement`，父叶保持 `runtime.mutation.ai_proposal_import_pass stop_split: false`；下一步只能进入 BE-001FD-01 `runtime.mutation.ai_proposal.approval_review_import_pass` 等价基线。

| `markdown/06-milestones/v4.16.0/449-runtime.mutation.ai_proposal_import_pass第九轮父叶残余判断.md` runtime mutation ai proposal import pass ninth parent residual judgment | `runtime.mutation.ai_proposal_import_pass` | 父叶残余判断，选择 approval_review import pass | BE-001FC 父叶重判 | `approval_review_import_pass_selected`；下一步只能进入 BE-001FD-01 等价基线 |
