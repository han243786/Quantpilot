# 模块树

> 职责: 以白箱网络描述模块的输入、输出、关键 public 方法、父子关系和通信边界。
> 状态: v4.12.0 起建立模板和种子节点。后续重型变更必须逐步补齐受影响模块。

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
**父模块**: `root`
**真实文件**:
- `start.bat`
- `start.ps1`
- `src/main.rs`
- `src/lib.rs`
- `src-tauri/src/main.rs`

**职责**:
编排本地桌面应用、后端服务、前端开发服务和 Tauri 壳的启动边界。

**关键 public 方法**:
| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `run_server` | 环境变量、存储路径、路由构建依赖 | Axum 服务 | `src/main.rs` | 不得绕过 `build_app_router` 注册路由 |

**父级通信规则**:
系统入口只负责启动和编排，不拥有业务能力真源。

**回归保护**:
`cargo check --workspace`；涉及启动脚本时执行对应 PowerShell 或批处理 dry-run。

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

