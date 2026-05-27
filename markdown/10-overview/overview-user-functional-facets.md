# QuantPilot 用户侧功能切面总览

> 版本：v4.11.0
> 定位：单机量化策略配置、仿真、回测与受控执行工具
> 第一功能切面：v4 策略配置系统

---

## 0. 阅读口径

本文档不按页面清单描述 QuantPilot。页面、编辑器、执行端、AI 治理、快照与设置都只是支撑层；v4 版本真正优先的用户价值是：

> 用户可以把策略配置成一份可编译、可审计、可仿真、可回放、可治理的运行契约。

因此，本文所有功能都按以下四类状态标注：

| 状态 | 含义 |
| --- | --- |
| 当前已实现 | 本地代码或里程碑文档已有明确实现证据，且用户可以通过现有界面、接口或运行链路触达。 |
| 当前可文档化 | 能力边界、数据结构、约束或证据已经存在，可以对用户说明，但还不是完整的一等交互体验。 |
| 后续里程碑 | 符合 v4 策略配置水准，但当前 v4.11 不应宣称已经完成。 |
| 明确不做 | 已经固化为产品边界，不进入后续常规里程碑。 |

核查来源以本地代码与本地文档为准，外部研究报告只作为问题意识和结构启发，不作为实现证据。

---

## 1. 产品边界

| 事项 | 状态 | 用户侧说明 |
| --- | --- | --- |
| 单机交易工具定位 | 当前已实现 | 产品按本地桌面/单机策略工具推进，不按云端多租户 SaaS 设计。 |
| 账户系统、注销、密码找回、2FA、RBAC | 明确不做 | 登录、刷新令牌、凭证隔离只服务本地会话与本机安全边界，不构成完整账户系统。 |
| 策略中心搜索、筛选、排序 | 明确不做 | 单机策略数量按可滚动范围处理，不建设策略资产管理平台。 |
| 真实资金自动交易宣称 | 明确不做 | 当前不宣称真实资金自动交易能力；执行能力必须显式展示模拟盘、演示盘与真实提交边界。 |
| 第三方插件市场 | 明确不做 | 插件能力边界按本地安全治理处理，不建设公开市场。 |
| 研究级回测平台 | 明确不做 | 当前提供基础回放、v4 证据产物和仿真执行证据，不宣称机构级回测准确性。 |

---

## 2. 第一功能切面：v4 策略配置系统

### 2.1 目标水准

v4 策略配置系统的目标不是让用户填几个参数，而是让策略配置具备四个性质：

| 性质 | 用户价值 |
| --- | --- |
| 可编译 | 配置可以进入 QuantScript、Core IR、运行时或回测链路，而不是停留在表单草稿。 |
| 可审计 | 状态机、事件、转移、记忆、风控与执行能力都有静态检查和能力边界。 |
| 可回放 | 仿真、回测和运行证据可以回看机器轨迹、风控决策、执行能力来源与失败原因。 |
| 可治理 | AI 提案、沙箱验证、审批、激活、回滚、快照都有受控路径，不能绕过用户确认。 |

### 2.2 配置域对比

| 配置域 | 目标功能 | v4.11 现状 | 状态 | 本地证据口径 |
| --- | --- | --- | --- | --- |
| 市场与数据配置 | 用户能声明交易所、标的、K 线/报价来源、数据新鲜度、回放范围。 | 前端能力矩阵限定 `binance`、`okx` 与 `BTCUSDT`、`ETHUSDT`、`SOLUSDT`；内置 K 线与报价模块；执行端 K 线支持周期与数量控制。 | 当前已实现 | `frontend/src/capabilities/supportMatrix.js`、`frontend-executor/src/components/KlineChart.jsx`。 |
| 观察与信号配置 | 用户能配置策略观察对象、指标、观测状态、触发事件。 | 18 类 Strategy IR 指标已在支持矩阵中固化；v4 QS 支持 `machine`、`state`、`state_group`、`on event`、`memory` 等结构。 | 当前已实现 | `implementation-support-matrix.md`、`quantscript/src/v4_static_audit.rs`。 |
| 决策状态机配置 | 用户能把策略表达为事件驱动状态机，而不是线性脚本。 | v4 里程碑定义顶层 DAG 加节点内状态机；Core IR 有 v4 machine、transition、memory、risk plane 校验；QS 静态审计可生成 v4 runtime handoff。 | 当前已实现 | `markdown/06-milestones/v4.0.0/01-规划方案.md`、`qrpc_core_ir/src/v4.rs`、`quantscript/src/v4_static_audit.rs`。 |
| 转移守卫与复杂度配置 | 用户能约束 transition guard、嵌套深度、状态覆盖与复杂度预算。 | 支持 `v4.transition.guard` 模块键；v4 仿真要求 QS 静态审计通过；嵌套状态机处于 beta，最大深度 2，并输出复杂度/层级证据。 | 当前可文档化 | `frontend/src/capabilities/supportMatrix.js`、`quantscript/src/v4_static_audit.rs`。 |
| 风控配置 | 用户能独立配置 Risk Plane，并确认执行请求必须经过风控截获。 | v4 Core IR 与 runtime 均有 Risk Plane 结构；v4 回测产物输出 `risk_plane_decisions`；里程碑要求不得绕过 Risk Plane。 | 当前已实现 | `qrpc_core_ir/src/v4.rs`、`src/runtime/backtest.rs`、v4.0 里程碑。 |
| 执行配置 | 用户能区分本地仿真、交易所演示盘、真实提交能力，避免静默降级。 | 主应用运行模式仅 `paper`；v4 运行使用 `PaperSimulated`；执行端支持 `PaperSimulated` 和 OKX demo 边界的 `PaperActual`；`live_execution_allowed=false`。 | 当前已实现 | `frontend/src/capabilities/supportMatrix.js`、`src/runtime/run.rs`、`src-executor/executor_state.rs`、`src-executor/main.rs`。 |
| 回测与证据配置 | 用户能从同一策略配置产生 v4 回测证据、机器轨迹、风控决策与执行能力来源。 | `/api/runtime/backtest` 支持 `runtime_kind=v4`；v4 backtest artifact 包含 machine trajectory、risk plane decisions、execution capability sources。 | 当前已实现 | `src/runtime/backtest.rs`、`qrpc_core_ir/src/v4.rs`、v4.3 里程碑。 |
| 快照与版本配置 | 用户能保存可校验的策略版本与运行边界，恢复时检测篡改。 | 快照服务使用 canonical JSON SHA-256 摘要校验关键字段；恢复时重新计算摘要并拒绝不匹配快照。 | 当前已实现 | `src/snapshot_service.rs`。 |
| AI 提案配置 | 用户能让 AI 基于证据提出参数、守卫、阈值或超时优化，但不能直接改策略。 | 能力边界为 `ai_write_policy=proposal_only`；AI proposal 已绑定配置域、before/after digest、evidence anchors、沙箱验证和审批阻断。 | 当前已实现 | `frontend/src/capabilities/supportMatrix.js`、`src/runtime/mutation.rs`、`src/sandbox_verification.rs`、v4.7/v4.11 里程碑。 |
| 一等图形化 v4 配置台 | 用户能在一个配置台内看到每个配置域的当前值、支持级别、证据、风险和待办。 | 策略工作区已接入 v4 配置台，展示配置域导航、单域来源/诊断、证据锚点、AI 提案绑定、本地 artifact diff、artifact 导出和运行边界；版本历史已展示正式版本间配置契约 diff，并支持显式选择左右 v4 backtest 证据进行有限 evidence diff。 | 当前已实现 | 后续补域级编辑和更深证据钻取。 |
| 规范化 v4 配置契约导出 | 用户能导出完整、稳定、可签名、可 diff 的 v4 策略配置契约。 | 配置台已可导出当前 v4 strategy config artifact JSON；图版本 compare 已附带 `strategy_config_diff`，并可返回显式绑定的 `strategy_config_evidence_diff`。 | 当前已实现 | 后续补签名包和审计包装。 |

### 2.3 第一切面的用户主流程

```text
选择模板/图节点/QS
  -> 配置市场、观察、状态机、风控、执行边界
  -> 编译与 v4 静态审计
  -> 能力矩阵判定
  -> PaperSimulated 仿真或 v4 回测
  -> 生成机器轨迹、风控决策、执行能力证据
  -> AI 只提出绑定配置域的 proposal
  -> 沙箱验证、审批、激活或回滚
  -> 快照保存与摘要校验
```

| 环节 | 用户要得到什么 | 当前状态 |
| --- | --- | --- |
| 配置入口 | 从策略图、模板或 QS 进入策略配置。 | 当前已实现 |
| 静态审计 | 在运行前发现非法状态机、非法 memory、非法 transition 与禁用能力。 | 当前已实现 |
| 能力边界 | 明确知道当前能仿真、能回测、能演示盘提交，还是不支持。 | 当前已实现 |
| 证据闭环 | 每次运行或回测可以看到机器轨迹、风控决策和执行来源。 | 当前已实现 |
| AI 治理 | AI 不替用户直接改策略，只提交可审查 proposal。 | 当前已实现 |
| 配置总览 | 一屏汇总当前策略配置完整度、风险、证据、AI 提案绑定与本地 artifact 差异；版本历史可比较正式版本之间的配置契约差异，并可在用户显式选择 A/B v4 backtest 后展示证据差异。 | 当前已实现 |
| 契约导出 | 导出单一 v4 strategy config artifact，供 diff、签名、审计和复现。 | 当前已实现 |

---

## 3. 目标功能与当前能力矩阵

| 用户侧能力 | 当前已实现 | 当前可文档化 | 后续里程碑 | 明确不做 |
| --- | --- | --- | --- | --- |
| v4 策略配置作为第一功能切面 | 已有 v4 QS、Core IR、runtime、backtest、evidence、AI proposal 链路；策略工作区已接入统一 v4 配置台、配置域深钻、artifact 导出、正式版本间配置契约 diff、显式 v4 evidence diff、capability hash freshness 和 Risk Plane 静态契约校验。 | 可以把策略解释为运行契约，而不是页面集合。 | 域级编辑、证据深钻和签名包。 | 不是普通策略列表产品。 |
| v4 状态机策略表达 | QS 静态审计、Core IR 校验、runtime handoff 已存在。 | 可说明 machine/state/event/memory/transition/risk plane 模型。 | 图形化状态机编辑器与 guard builder。 | 不允许动态不可审计顶层图。 |
| 策略图与模块化配置 | 内置数据、意图、代理、风控、执行、运行控制、v4 参数/守卫模块。 | 可说明 16 个前端模块键和 18 类指标边界。 | 模块级配置完整度评分。 | 不建设无限制第三方模块市场。 |
| 编译、运行、回测 | 编译、PaperSimulated、v4 backtest artifact、基础 replay 已存在；正式版本比较可显式绑定两个 v4 backtest 证据并展示 machine trajectory、Risk Plane、execution capability 与摘要指标差异。 | 可说明基础回测不是研究级回测。 | 更深的证据钻取和可解释报告。 | 不宣称研究级回测平台。 |
| 执行端 | 执行端支持 PaperSimulated 与 OKX demo 边界 PaperActual，具备订单、资产、K 线、v4 证据面板和启动前 strategy_config_preflight 阻断。 | 可说明 demo 与模拟边界。 | 更完整的部署包审计视图。 | 不宣称真实资金自动交易。 |
| AI 策略优化 | proposal_only、配置域绑定、sandbox verification、approval、rollback 路径存在。 | 可说明 AI 只能提案，不能直接改 QS 或越过审批。 | 更细的 proposal diff 与风险解释。 | 不做 AI 自动改写并直接执行策略。 |
| 快照与恢复 | 快照 canonical digest 校验已存在。 | 可说明当前是 SHA-256 摘要校验，不等同 Ed25519 法务级签名。 | 统一 v4 contract hash、签名和 diff 视图。 | 不把快照包装成托管云备份。 |
| 新用户上手 | 教程、空状态、设置、404、命令面板等 v4.10 UX 收尾已补齐或已有实现。 | 可说明它们服务第一功能切面。 | 按 v4 配置流程重排 onboarding。 | 不做营销式首页。 |
| 账户管理 | 本地会话和凭证隔离保留。 | 可说明不是云账户系统。 | 无。 | 注销、密码找回、2FA、RBAC、账户资料页。 |
| 策略中心搜索筛选 | 无。 | 可说明单机策略数按可滚动范围处理。 | 无。 | 搜索、筛选、排序作为产品功能不做。 |

---

## 4. 支撑功能切面

### 4.1 策略研究工作区

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| 策略中心与策略详情 | 当前已实现 | 作为进入某份 v4 策略配置的入口，而不是主要价值本身。 |
| 策略工作区五大区 | 当前已实现 | 构建、研究、监控、源码、策略中心分别承载配置、验证和证据查看。 |
| 模板库、版本历史、协作审计、参数扫描等 workspace surface | 当前已实现 | 支撑配置复用、参数实验和治理追踪。 |
| 搜索、筛选、排序 | 明确不做 | 单机边界内不把策略中心做成资产管理系统。 |

### 4.2 QuantScript 与策略图

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| QuantScript 编写与编译 | 当前已实现 | 提供策略配置的文本表达和编译入口。 |
| v4 QS 静态审计 | 当前已实现 | 在运行前约束 machine、state、memory、transition 和禁用能力。 |
| 策略图模块 | 当前已实现 | 为非纯文本用户提供配置结构入口。 |
| 一等 v4 状态机可视化编辑 | 后续里程碑 | 当前有结构能力，但缺少完整可视化配置体验。 |

### 4.3 回测、仿真与运行证据

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| v4 PaperSimulated runtime | 当前已实现 | 用同一策略配置进行受限仿真。 |
| v4 backtest artifact | 当前已实现 | 把运行结果转成可回放、可审计证据。 |
| Risk Plane evidence | 当前已实现 | 用户能确认风控是否截获、拒绝或放行执行请求。 |
| 正式版本证据差异 | 当前已实现 | 用户显式选择同一策略的两个 v4 backtest 后，可以看到机器轨迹、Risk Plane、执行能力来源和摘要指标差异。 |
| 研究级撮合、滑点、市场微结构准确性 | 明确不做 | 当前只做基础 replay 与模拟证据，不做机构级回测承诺。 |

### 4.4 AI 提案与治理

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| AI proposal generation | 当前已实现 | AI 基于运行证据提出配置改动建议。 |
| Sandbox verification | 当前已实现 | 提案进入沙箱验证，不直接进入执行。 |
| Approval、activation、rollback | 当前已实现 | 用户保留最终控制权。 |
| AI 直接修改 QS 并执行 | 明确不做 | 违反 proposal_only 边界。 |

### 4.5 执行端

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| PaperSimulated | 当前已实现 | 本地仿真，不连接真实提交。 |
| PaperActual | 当前已实现 | OKX demo 边界的演示盘提交，不等同真实资金。 |
| strategy_config_preflight 启动阻断 | 当前已实现 | v4 策略启动前必须通过后端生成的 preflight，执行端不自行推断 capability。 |
| K 线、订单、资产、策略参数、v4 证据面板 | 当前已实现 | 执行端显示配置运行后的市场、执行和证据结果。 |
| 真实资金自动交易对外可用 | 明确不做 | 当前不作为 v4 能力宣称，用户侧统一使用 PaperSimulated / PaperActual。 |

### 4.6 安全、凭证与快照

| 能力 | 状态 | 与第一切面的关系 |
| --- | --- | --- |
| 凭证隔离与本地会话 | 当前已实现 | 支撑交易所 demo 或本地运行边界。 |
| Snapshot digest 校验 | 当前已实现 | 支撑配置恢复时的完整性检查。 |
| 统一 v4 配置签名包 | 后续里程碑 | 把 QS、Core IR、capability、runtime boundary 和 evidence anchor 汇成一个可签名契约。 |
| 云账户与多租户安全体系 | 明确不做 | 不符合单机工具定位。 |

---

## 5. 后续里程碑优先级

后续版本应优先围绕第一功能切面继续收束，而不是横向扩页面。

| 优先级 | 里程碑方向 | 交付口径 |
| --- | --- | --- |
| P0 | v4 策略配置台 | 已在策略工作区一屏呈现配置完整度、配置域深钻、运行边界、证据锚点、AI 提案绑定、本地 artifact diff、正式版本间配置契约 diff 和显式 v4 evidence diff；后续补域级编辑和证据深钻。 |
| P0 | v4 strategy config artifact | 导出单一、稳定、可 diff、可摘要校验的用户级配置契约。 |
| P1 | 状态机可视化配置 | 图形化编辑 state、state_group、event、transition guard、memory 和复杂度预算。 |
| P1 | 配置差异到证据差异 | 已支持正式版本比较中显式绑定 A/B v4 backtest 证据，查看机器轨迹、风控决策、执行能力来源和摘要指标差异；后续补更深 drilldown。 |
| P1 | AI 提案域绑定 | AI proposal 必须明确作用于哪个配置域、修改前后值、沙箱结果和风险。 |
| P2 | 执行前配置核验 | 执行端已消费后端 strategy_config_preflight 并在 v4 启动前阻断不合格配置；后续补更完整的部署包审计视图。 |

不建议把下一波里程碑投入账户、策略搜索、营销首页或泛化页面扩张；这些方向不会提高 v4 策略配置水准。

---

## 6. 当前可对用户宣称的能力

| 可宣称能力 | 推荐表述 |
| --- | --- |
| v4 策略配置 | 支持以状态机、事件、记忆、风控平面和执行能力边界组织策略配置。 |
| v4 静态审计 | 支持在运行前检查 v4 QS 结构、状态机深度、memory 类型和 transition 约束。 |
| 纸面运行时 | 支持 PaperSimulated 本地仿真，并在执行端区分 OKX demo 边界的 PaperActual。 |
| 基础回测 | 支持 v4 runtime kind 的基础回放和 v4 artifact 证据输出。 |
| 版本证据差异 | 支持在正式版本比较中显式选择两个 v4 backtest 证据，查看机器轨迹、Risk Plane、执行能力来源和摘要指标差异。 |
| AI 治理 | AI 只能生成 proposal，需经过沙箱验证、审批、激活或回滚。 |
| 快照完整性 | 支持基于 canonical JSON SHA-256 摘要的快照完整性校验。 |

## 7. 当前不应宣称的能力

| 不应宣称 | 原因 |
| --- | --- |
| 完整账户系统 | 已固化为 unsupported。 |
| 真实资金自动交易 | 当前能力边界明确不允许宣传真实资金自动交易。 |
| 研究级回测平台 | 当前是基础 replay、仿真和证据产物。 |
| AI 自动改写并执行策略 | 与 proposal_only 冲突。 |
| 第三方插件市场 | 已作为产品边界排除。 |
| 策略资产管理平台 | 搜索、筛选、排序已明确不做。 |

---

## 8. 当前功能规模

| 项目 | 数量/边界 |
| --- | --- |
| 前端主要路由 | 策略、策略详情、回测详情、回测对比、QuantScript、审批、告警、快照、Runbook、Chaos、设置、404。 |
| 工作区 surfaces | 10 个：dashboard、code、diagnostics、research、monitor、source、template_library、version_history、collaboration_audit、parameter_sweep。 |
| 命令面板命令 | 11 个：导航、保存、编译、运行、回测等。 |
| 能力动作 | 15 个，包含教程、凭证、保存、编译、仿真、v4 仿真、回测、参数扫描、停止、重置和导出。 |
| 前端模块键 | 16 个，包含数据、意图、代理、风控、执行、运行控制、v4 参数与 v4 transition guard。 |
| Strategy IR 指标 | 18 类。 |
| 主应用 runtime mode | `paper`。 |
| 执行端模式 | `PaperSimulated`、OKX demo 边界 `PaperActual`。 |
| 当前市场边界 | `binance`、`okx`；`BTCUSDT`、`ETHUSDT`、`SOLUSDT`。 |

---

## 9. 与其他文档的关系

| 文档 | 关系 |
| --- | --- |
| `markdown/03-implementation/governance/implementation-support-matrix.md` | 本文的能力边界和 unsupported 决策应与支持矩阵保持一致。 |
| `markdown/06-milestones/v4.0.0/01-规划方案.md` | v4 状态机、Risk Plane、执行能力来源和 PaperSimulated 基线来源。 |
| `markdown/06-milestones/v4.3.0/01-规划方案0.md` | v4 backtest artifact、多标的与模板能力来源。 |
| `markdown/06-milestones/v4.7.0/01-规划方案1.md` | AI proposal、sandbox verification、trajectory 对比和审批治理来源。 |
| `markdown/06-milestones/v4.10.0/01-规划方案2.md` | v4.10 UX 收尾、产品边界和不做事项来源。 |

本文后续更新原则：先更新 v4 策略配置系统的目标、当前能力和边界，再更新页面、导航或局部交互说明。
