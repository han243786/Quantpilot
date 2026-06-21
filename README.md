# QuantPilot

> 单人本地使用的专业量化策略研究、回测、模拟执行与运行治理桌面工具。
> 本 README 只负责入口导航、当前状态和能力边界说明；事实真源以 `governance-next/`、QPCursor、全量树、模块树、后端 capability、测试门禁和 closeout evidence 为准。

## 当前状态

| 维度 | 当前口径 |
| --- | --- |
| 代码发布基线 | v4.7.0 |
| 产品边界口径 | v4.10.0 UX 收口与产品边界固化 |
| 当前治理入口 | `governance-next/` |
| 日常开发主控 | QPCursor + 全量树 + 模块树 |
| 架构推进线 | v4.16.0 模块化抽离与递归治理推进 |
| 产品定位 | 单人本地桌面量化工具 |
| 真实资金自动交易 | 当前不作为可用能力承诺 |
| SaaS / 多租户 / 团队账号 | unsupported |

代码版本、产品边界和治理推进线不是同一件事。`v4.7.0` 是代码发布基线；`v4.10.0` 固化了单机工具、账户裁剪和策略中心不做搜索筛选等产品边界；`v4.15-v4.16` 是治理和模块化抽离推进线。

## QuantPilot 是什么

QuantPilot 是本地运行的量化策略工作台，目标是把策略从想法推进到可编译、可审计、可回放、可治理的运行契约。

核心能力包括：

- 用策略图或 QuantScript 构建策略。
- 通过统一编译链生成可验证的运行表示。
- 运行 paper 模拟、历史回测和参数实验。
- 用 v4 状态机、Risk Plane、执行能力来源和事件证据解释运行结果。
- 在执行端观察订单、资产、K 线、策略状态和运行证据。
- 通过 AI proposal、沙箱验证、审批、激活和回滚管理运行时变更。
- 明确区分 supported、deferred 和 unsupported 能力，不把未支持能力包装成可用功能。

QuantPilot 不按 SaaS、多租户、多用户后台或团队账号系统设计。

## 先读什么

新开发者、Agent 或维护者进入仓库时，默认按以下顺序定位：

1. `governance-next/README.md`: 当前权威治理入口。

2. `governance-next/05-authoritative-operating-model.md`: 当前运行模型、旧治理关系和证据边界。

3. `governance-next/01-qpcursor-protocol.md`: QPCursor 接管坐标和工作游标规则。

4. `governance-next/02-governance-heat-trigger.md`: 判断本次变更属于 G0-G5 哪个治理热度。

5. `governance-next/03-local-invariants.md`: 绑定模块、切面、接口、状态和边界不变量。

6. `markdown/10-overview/overview-state-machine-productization-vision.md`: v4/v5 状态机产品化推进循环的北极星文档。

7. `markdown/10-overview/overview-full-feature-tree.md`: 全量树，物理文件地图，回答项目里有什么、文件在哪、改动会影响什么。

8. `markdown/00-matrix-governance/module-tree.md`: 模块树，逻辑白箱网络，回答模块输入、输出、关键 public 方法、父子通信和回归保护。

维护提醒：README 不能替代全量树和模块树。若全量树或模块树为空、失真或未同步，先修复事实树，再更新 README。

## 用户主流程

```text
创建或选择策略
  -> 构建策略图或编写 QuantScript
  -> 编译与静态审计
  -> 回测 / 模拟运行 / 参数扫掠
  -> 分析证据、指标与运行历史
  -> 进入执行端做 PaperSimulated / PaperActual 观察和控制
  -> 通过 AI proposal 与审批链管理受控变更
```

## 当前能力概览

| 能力域 | 当前能力 |
| --- | --- |
| 策略中心 | 全量策略列表、模板库、近期运行、近期回测、对比队列、检查器、新手入口 |
| 策略工作区 | 总览、构建、诊断、研究回测、运行监控、源码、模板、版本历史、协作审计、参数扫掠 |
| 图编辑器 | React Flow 画布、节点池、模块侧栏、连线、属性面板、版本历史、导入导出 |
| QuantScript | 轻量代码编辑、草稿自动保存、运行测试、Tab 缩进、粘贴保护 |
| 编译链 | graph -> QS -> parse -> HIR -> lower -> Core IR |
| 回测 | 历史回放、回测详情、回测对比、12 项指标、事件流、v4 artifact 摘要 |
| v4 运行证据 | 状态机轨迹、Risk Plane 决策、执行能力来源、复杂度预算、tick replay 证据 |
| AI 治理 | AI proposal、沙箱验证、L1/L2/L3 审批、签名快照、运行时变更 |
| 执行端 | PaperSimulated、OKX demo 边界 PaperActual、订单、资产、K 线、策略图、紧急停止 |
| 安全 | OKX 凭证管理、AES-256-GCM、PBKDF2、本地 JWT 会话、进程间加密通道、日志脱敏 |
| 全局交互 | Ctrl/Cmd+K 命令面板、Toast、错误边界、教程、离线检测、配额提示、Tauri 桌面壳 |

## 产品边界

### Supported

- 单机桌面工具定位。
- 本地策略管理。
- 图形化策略构建。
- QuantScript 编写、静态审计与测试。
- 统一编译链。
- PaperSimulated 本地仿真。
- OKX demo 边界的 PaperActual 演示盘提交。
- 基础历史回测和 v4 backtest artifact。
- 参数扫掠。
- AI proposal 与审批治理。
- 签名快照和运行证据链。
- 告警、Runbook、Chaos 实验入口。
- OKX 凭证管理。
- 中英双语与 auto/dark/light 主题。

### Deferred

- 更深的证据 drilldown。
- 更完整的部署包审计视图。
- v4 strategy config 签名包。
- 图形化状态机编辑器与 guard builder。
- 更多 provider 和资产类别扩展。

### Unsupported

- 真实资金自动交易对外可用。
- 研究级回测平台承诺。
- 注销、密码找回、2FA / TOTP / WebAuthn。
- RBAC、管理员用户管理 UI、用户资料页。
- SaaS 多租户账号系统。
- 策略中心搜索、筛选、排序、分页。
- 第三方插件市场。
- QuantScript 任意主机代码执行。
- 绕过 Risk Plane 的真实下单。
- 未声明 provider 能力的静默降级。

## 系统拓扑

```text
用户桌面
  |
  |-- Tauri 桌面壳
  |     |
  |     `-- 前端 React SPA (:5173 / frontend/dist)
  |            |
  |            `-- 后端 Axum 服务 (:3000)
  |                   |
  |                   |-- 编译链
  |                   |-- 图存储
  |                   |-- 运行时
  |                   |-- 回测
  |                   |-- 能力声明
  |                   |-- 凭证与本地会话
  |                   |-- 告警 / 快照 / 审批 / Runbook / Chaos
  |                   |
  |                   `-- qrpc_session 加密通道
  |                          |
  |                          `-- 执行端 Axum 服务 (:3001)
  |                                 |
  |                                 |-- RunnerPool
  |                                 |-- OKX WebSocket 行情
  |                                 |-- OKX demo REST 回执
  |                                 |-- 执行端凭证保险库
  |                                 `-- frontend-executor SPA
```

## 仓库阅读法

README 告诉你入口，不承载完整知识。实际开发必须回到全量树、模块树和 `governance-next/`。

| 问题 | 事实来源 |
| --- | --- |
| 项目里有哪些文件 | `markdown/10-overview/overview-full-feature-tree.md` |
| 某个功能在哪个文件 | 全量树对应根节点 |
| 新增、删除、重命名文件怎么登记 | 全量树维护规则 |
| 模块输入和输出是什么 | `markdown/00-matrix-governance/module-tree.md` |
| 哪些 public 方法跨模块暴露 | 模块树白箱节点 |
| 当前任务怎么接管 | `governance-next/README.md` |
| 是否需要升级治理强度 | `governance-next/02-governance-heat-trigger.md` |
| 哪些局部不变量不能破坏 | `governance-next/03-local-invariants.md` |
| 当前状态机产品化愿景 | `markdown/10-overview/overview-state-machine-productization-vision.md` |
| 旧三矩阵兼容档案 | `markdown/00-matrix-governance/README.md` |

## 快速启动

### 一键启动

```bat
.\start.bat
```

`start.bat` 会编译后端、启动后端服务，并进入 Tauri 桌面开发流程。

### 分开启动

```powershell
# 后端
cargo run

# 主前端
cd frontend
npm install
npm run dev

# 执行端
cargo run --bin executor

# 执行端前端
cd frontend-executor
npm install
npm run dev
```

## 环境变量

详见 `.env.example`。常用变量如下：

| 变量 | 用途 | 默认值 |
| --- | --- | --- |
| `QUANTPILOT_DEV` | DEV 模式，跳过认证和限速、缩短 TTL | `false` |
| `QUANTPILOT_STORAGE_ROOT` | 存储根目录 | `storage` |
| `QUANTPILOT_EXECUTOR_URL` | 执行端地址 | `http://127.0.0.1:3001` |
| `QUANTPILOT_EXECUTOR_INSECURE` | 跳过执行端 API 守卫，仅限开发 | `false` |
| `QUANTPILOT_JWT_SECRET` | JWT 密钥，留空自动生成 | 随机 |
| `QUANTPILOT_API_KEY` | API 密钥 | 无 |
| `QUANTPILOT_MARKET_PUBLIC_KEY` | 插件市场 Ed25519 公钥 | 测试向量 |
| `QUANTPILOT_RATE_LIMIT_RPS` | 全局请求限速 | `100` |
| `QUANTPILOT_LOG_FORMAT` | 日志格式，`compact` 或 `json` | `compact` |
| `QUANTPILOT_TRUSTED_PROXY` | 反向代理模式 | `false` |

## 常用开发命令

### Rust

```powershell
cargo fmt --check
cargo check --workspace
.\scripts\test.ps1 test --workspace
cargo check --bin executor
.\scripts\test.ps1 test --bin executor
```

### 前端

```powershell
cd frontend
npm run build
npm run test
npm run test:e2e
npm audit --audit-level=moderate
```

### 执行端前端

```powershell
cd frontend-executor
npm run build
```

### Closeout

```powershell
.\tools\run-closeout-gates.bat
```

## 关键门禁

| 门禁 | 命令 |
| --- | --- |
| UTF-8 | `powershell tools/check-utf8.ps1` |
| 用户文案 | `powershell tools/check-user-facing-text.ps1` |
| 能力治理 | `powershell tools/check-capability-governance.ps1` |
| 能力栈 | `powershell tools/check-capability-stack.ps1` |
| i18n | `powershell tools/check-i18n.ps1` |
| 版本一致性 | `powershell tools/check-version-consistency.ps1` |
| 功能演进 | `powershell tools/check-feature-evolution.ps1` |
| 新治理兼容门禁 | `powershell tools/check-matrix-governance.ps1` |
| 学习流水线 | `powershell tools/check-learning-closeout.ps1` |
| 全量树 | `powershell tools/check-full-feature-tree.ps1` |
| 干净工作区 | `powershell tools/check-clean-worktree.ps1` |
| QS 场景 smoke | `powershell scripts/scenario-smoke.ps1` |

## README 不承载什么

| 内容 | 应放位置 |
| --- | --- |
| 全文件清单 | 全量树 |
| 模块输入、输出和 public 方法 | 模块树 |
| 长版本流水账 | milestone / roadmap / topology ledger |
| closeout 证据 | milestone closeout |
| GP 条款全文 | `markdown/General_Policy.md` |
| 流程规则全文 | `governance-next/` 或 `principles-super-standardization.md` |
| 用户功能完整目录 | `markdown/10-overview/overview-user-functional-facets.md` |
| API 细节 | `contracts/openapi/root.yaml` |
| 当前递归推进游标 | QPCursor / recursive state |
| 临时修复记录 | 对应 milestone 或 closeout evidence |

README 的职责只有三个：

1. 说明 QuantPilot 是什么。
2. 说明当前状态和边界。
3. 把读者送到正确的事实真源。

## 更多文档

| 文档 | 路径 |
| --- | --- |
| 文档索引 | `markdown/README.md` |
| 当前状态与路线图 | `markdown/10-overview/overview-current-status-and-roadmap.md` |
| 用户功能切面 | `markdown/10-overview/overview-user-functional-facets.md` |
| 系统架构 | `markdown/10-overview/overview-system-architecture.md` |
| 详细文档索引 | `markdown/10-overview/overview-docs-index.md` |
| 支持矩阵 | `markdown/03-implementation/governance/implementation-support-matrix.md` |
| RFC 协议索引 | `markdown/02-protocol/README.md` |
| 实现契约目录 | `markdown/03-implementation/governance/` |
| 里程碑归档 | `markdown/06-milestones/` |
| 审计与测试报告 | `markdown/05-testing/` |

## 维护提醒

修改 README 前先确认：

- `governance-next/README.md` 仍是权威入口。
- 全量树和模块树非空，且与当前文件和模块事实一致。
- README 没有把 unsupported / deferred 写成 supported。
- README 没有混淆代码版本、产品能力版本和治理推进线。
- README 没有替代全量树或模块树。
- README 新增的用户可见能力已经进入后端 capability、OpenAPI、支持矩阵和测试证据。

## 许可与风险提示

QuantPilot 仍处于密集开发和治理推进阶段。当前能力适合本地研究、策略验证、paper 模拟和执行链路观察，不应被理解为真实资金自动交易承诺。任何真实资金路径都必须单独经过 Risk Plane、凭证保险库、执行能力来源、审计证据和发布门禁。
