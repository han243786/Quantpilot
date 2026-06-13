# QuantPilot 能力治理

## 目的

本文档是与能力相关规则的 P1 治理层。它建立在 P0 支持矩阵之上，将当前 beta 边界转化为可审计的维护策略。

在以下情况下使用本文档：

- 修改 `/api/capabilities`
- 修改前端模块暴露
- 修改能力驱动的 UI 操作可用性
- 修改工作区界面的可见性或其真实数据源分类
- 修改关于已支持或未支持能力的面向用户措辞
- 决定某项能力应保持可见、保持锁定还是彻底移除

本文档不扩展产品范围。它仅治理现有能力边界如何被分类、归属、审查和退役。

机器可读配套文件：

- [frontend/src/capabilities/capabilityGovernance.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/capabilityGovernance.js)
- [implementation-capability-governance-registry.generated.md](./implementation-capability-governance-registry.generated.md)

## 真实数据源链

能力治理遵循以下链条：

1. 后端 `/api/capabilities`
2. [implementation-support-matrix.md](./implementation-support-matrix.md)
3. [运行时治理合约](../runtime/implementation-runtime-governance-contract.md)
4. 前端支持矩阵和能力门禁
5. README、UI 提示、测试和验收检查

如果这些层不一致，后端 `/api/capabilities` 是权威源，其他层必须更新。工作区入口以 `workspace.surfaces` 为真源，工具栏和卡片操作以 `ui_actions.actions` 为真源。运行时标识字段、事件信封、部署修订版和权限边界执行由运行时治理合约拥有。

## 能力类别

每项能力必须恰好属于以下类别之一。

### `supported`（已支持）

- 在声明 beta 边界内描述为当前已支持是安全的。
- 必须拥有后端支持、前端暴露规则和回归测试覆盖。
- 可以在 README 和 UI 中作为正常的已支持路径出现。

当前示例：

- `paper` 运行模式
- `builtin.execution.paper`
- `binance`、`okx`
- `BTCUSDT`、`ETHUSDT`、`SOLUSDT`
- 当前 K 线驱动的意图模块
- 版本历史与协作/审计工作区界面

### `restricted`（受限）

- 存在并且在编译或运行时路径中可能可用，但仅在清晰限定的边界内。
- 必须始终携带边界说明。
- 不得被宣传为比实际实现范围更广的平台支持。

当前示例：

- 受限的 `Custom` Strategy IR 表达式路径
- Strategy IR 语义预检
- 存在时的正式 QuantScript 降级
- 仅作为有限 beta 编译/运行时行为存在的价差相关降级路径

### `trace_only`（仅追踪）

- 为 beta 兼容性、内部实验或过渡状态存在于代码或工件中。
- 可以在代码、fixture 或工件结构中保持可见。
- 不得用作已支持产品范围的证据。

当前示例：

- 存在于 beta 编译路径中的套利相关模块键
- 为能力响应连续性保留的旧兼容性字段

### `disallowed_claim`（禁止声明）

- 绝不允许作为正面支持声明出现在面向用户的材料中。
- 测试和措辞门禁应捕获这些声明。

当前示例：

- 研究级回测支持
- 实盘交易支持
- 真实套利代理支持
- 第三方插件市场支持

## 能力注册表和负责人

所有权按角色分配，而非按个人姓名。每个能力系列必须有一个主要负责人角色。

| 能力系列 | 类别 | 负责人角色 | 审查责任 |
|---|---|---|---|
| 运行模式 | supported / restricted | 后端运行时负责人 | 后端合约、编译/运行时检查 |
| 执行模块 | supported / restricted | 后端运行时负责人 | 执行语义、能力响应 |
| 交易所和交易对 | supported / restricted | 后端市场数据负责人 | 市场边界、fixture、措辞 |
| Strategy IR 指标类型 | supported / restricted | 后端编译负责人 | 降级边界、诊断 |
| 前端模块暴露 | supported / trace_only | 前端编辑器负责人 | 侧边栏暴露、禁用原因、用户体验 |
| 能力驱动的 UI 操作 | supported / restricted | 前端编辑器负责人 | 操作门禁、原因文本、E2E |
| 工作区界面 | supported / restricted | 前端编辑器负责人 | 工作区暴露、后端路由诚实性、收口审计 |
| 公开措辞 | 所有类别 | 文档和 QA 负责人 | README、markdown、UI 文案、文本门禁 |

## 变更策略

对能力的任何变更必须在同一批次中更新所有受影响的层。

### 必需更新检查清单

- 后端能力响应或兼容性 fixture
- 前端支持矩阵或能力门禁逻辑
- 证明可见性、禁用状态或操作路由的测试
- 当支持声明或限制发生变化时的面向用户措辞
- 支持矩阵文档

### 必需审查问题

- 此变更是扩展、收缩还是仅澄清当前边界？
- 前端是否仍然避免虚假入口点？
- 可见的工作区卡片是否针对正确的可见性源进行分类，而非被静默视为能力驱动？
- 安全回退是否仍然比正常模式更严格？
- README 和 UI 措辞是否仍然与后端真实情况匹配？
- E2E 是否证明用户无法通过正常交互到达被阻止的后端路径？

## 退役和收敛策略

能力生命周期有意保持保守。

### 何时将 `trace_only` 移至 `restricted`

- 该能力具有真实的后端合约
- 边界说明是显式的
- 测试证明该路径是有意受限而非偶然

### 何时将 `restricted` 移至 `supported`

- 后端和前端语义稳定
- 措辞可以描述该能力而无需实质性地改变用户期望的附加说明
- 存在合约层和 UI 层的回归覆盖

### 何时从面向用户界面中移除能力

- 后端不再将其返回为可用
- 该能力无法被辩护为真实的已支持或受限路径
- 保持其可见会产生虚假入口点或支持声明

## 漂移预防规则

- 在未更新支持矩阵的情况下，绝不添加新的前端模块卡片。
- 在未决定其类别和负责人角色的情况下，绝不添加新的能力响应字段。
- 在未分类其是由能力驱动、仅本地还是由持久化驱动的情况下，绝不添加新的可见工作区界面。
- 在未对照 `allowed_claim` 白名单和 `disallowed_claim` 集合进行检查的情况下，绝不添加新的正面支持声明。
- 绝不单独依赖代码存在性作为产品支持的证据。

## 治理证据

以下工件算作治理证据：

- 支持矩阵文档更新
- 能力 fixture 更新
- 前端能力测试
- Playwright 能力路径测试
- 面向用户措辞门禁结果

## 参考

- [implementation-support-matrix.md](./implementation-support-matrix.md)
- [implementation-compile-chain-contract.md](./implementation-compile-chain-contract.md)
- [运行时治理合约](../runtime/implementation-runtime-governance-contract.md)
- [首次发布就绪状态](../../09-archive/planning-retired/implementation-first-release-readiness.md)
- [overview-current-status-and-roadmap.md](../../10-overview/overview-current-status-and-roadmap.md)
