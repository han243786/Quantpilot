# v4.0.0 CHANGELOG 条目规范

> 约束文档 | Codex 执行: 插入到 CHANGELOG.md 顶部 (第一个 `##` 条目之前)

---

## 必须插入的内容

以下内容必须完整插入到 `CHANGELOG.md` 文件顶部 (在 `## v3.7.1` 之前):

```markdown
## v4.0.0 — 状态机化架构 + 开发者学习流水线 (2026-05-24)

### 架构升级 (MAJOR)

- **顶层 DAG + 节点内状态机双层架构**: 量化基准链路 `data -> intent -> agent -> risk -> execution -> fill`
  保持有向无环, 节点内部升级为标准状态机模板, 复杂度下沉到可治理状态机模块内部。
- **三大 Machine 模板**: `ObservationMachine` (数据/指标/特征), `DecisionMachine` (意图/代理/风控),
  `ExecutionMachine` (执行/订单/成交/资产账本)。旧链路通过兼容桥映射为三大模板默认实例。

### QuantScript v4 状态机 DSL

- 新增状态机语法概念: `machine`, `state`, `state_group`, `memory`, `on event`, `transition`,
  `when`, `do`, `emit`, `cache`, `silence`, `recover`, `priority`, `mode`, `diagnostic`
- 声明式 `transition` 为主, 受控 `action block` 为辅
- `action block` 限制: 只能读写声明输入、事件上下文、局部变量和本节点 typed memory;
  禁止文件/网络/系统API/密钥/真实下单访问; 禁止无限循环/递归/dynamic eval
- 强类型系统: `QsTypeRef`, 类型嵌套深度 ≤8, nullable 支持
- v4 静态审计: parse → analyze → report (编译期拒绝 unsupported 路径, 结构化诊断)

### 事件模型

- 5 事件域: Market (price_tick/bar_*/orderbook_updated), Runtime (strategy_started/paused/silence_*/recovery_*),
  Decision (intent_emitted/weight_changed/risk_decision_changed), Execution (order_*/fee_charged/portfolio_changed),
  Manual (manual_override/manual_cancel/mode_changed)
- 事件字段: event_id, event_type, event_time, source, target_scope, payload, freshness, sequence, replayable
- 调度顺序: 因果顺序 > 安全层级 > DAG 依赖 > 用户自定义优先级 > 稳定兜底排序
- transition 必须绑定明确事件来源; memory 变化/cache 返回/silence/recovery 均形成事件或可回放证据

### Risk Plane 运行时安全平面

- 独立高优先级 Risk Plane (priority ≥9000)
- 所有真实下单路径必须经过 `risk_precheck -> risk_order_check -> risk_postcheck`
- `LiveActual` 模式下 ExecutionMachine 不得直接调用 VenueAdapter
- `emergency_halt` 高于所有 QS 逻辑; `reduce_only` 只允许降低敞口; `freeze_open` 禁止新开仓
- `stale`/`recovering` 数据默认不得扩大真实风险敞口

### ExecutionMachine 能力来源标记

- 每个订单能力标记为 `provider_native` / `runtime_simulated` / `unsupported`
- `runtime_simulated` 执行路径有本地订单/成交/手续费/资产账本/provider detached 证据
- `unsupported` 在编译期或运行前拒绝, 不允许静默降级
- VenueCapabilityMatrix 与 `/api/capabilities`/支持矩阵/前端文案一致

### 四种运行时模式

- `PaperActual` / `PaperSimulated` / `LiveActual` / `LiveSimulated`
- 账户域和成交来源不可混淆; 模式切换需确认 + SSE广播
- PaperSimulated 事件循环独立骨架, 与 v3.7.1 RuntimeCoordinator 隔离

### v4 Core IR 兼容桥

- `compat.rs` (363行): 旧链路 `data/intent/agent/risk/execution/fill` 映射为
  `ObservationMachine` / `DecisionMachine` / `ExecutionMachine` 默认实例
- 旧图/旧 QS/旧运行记录读取边界不受影响

### 前端能力真源通道

- `CapabilityResponse` 后端声明为能力存在性/启用状态/拒绝原因唯一真源
- `capabilityProjection.js` (132行): 前端能力投影层, `normalizeCapabilitySnapshot` → 类型化 view model
- 工作区 tab、工具栏 action、模块面板共用同一 projection; 禁止组件自行维护支持状态
- `safe_fallback` 只保留最小只读或明确禁用入口, 不恢复上一版本完整工作区
- `V4RuntimeEvidencePanel.jsx` (250行): v4 runtime evidence UI 投影面板
- capability projection 测试 (+97行), V4RuntimeEvidencePanel 测试 (+168行)

### 开发者学习流水线

- `implementation-developer-learning-pipeline.md` (176行): 学习流水线契约
- `tools/check-learning-closeout.ps1` (84行): closeout 门禁, 检查结构入口和本地学习记录边界
- 个人学习记录放入 `markdown/learning/`, 不入 Git (`.gitignore` 已配置)
- MAJOR closeout 必须检查 owner 必学核心机制

### 静态契约束

- `implementation-v4-machine-and-venue-contract.md` (986行): v4 状态机与交易场所能力静态契约
- `qrpc_core_ir/src/v4.rs` (5885行): 全部 v4 类型定义、校验、序列化
- `quantscript/src/v4_static_audit.rs` (1196行): QS 静态 parse/analyze/report 入口
- `qrpc_runtime/src/v4_runtime.rs` (2961行): v4 PaperSimulated 事件循环 + Risk Plane + ExecutionMachine

### GP / 超级规范化更新

- GP §1.6-§1.12: 7 条 v4 新增架构铁律 (顶层DAG+状态机边界/QS DSL边界/事件驱动迁移/Risk Plane/
  Execution能力来源/学习流水线边界/前端真源)
- 超级规范化 §7.7: MAJOR 演化通道 8 Phase | §7.8: 前端后端能力真源通道 | §8.9: v4 防偏规则
- GP §10.5: v4 演化回归保护矩阵

### 质量门禁

- `cargo check --workspace` ✅ 0 error
- `cargo fmt --check` ✅ 格式基线通过
- `cargo test --workspace --no-run` ✅ 全部测试可编译
- 23 项 closeout 门禁待全量复跑
```

---

## 验收标准

1. CHANGELOG.md 中 v4.0.0 条目位于 v3.7.1 条目之前
2. 条目日期为 `2026-05-24`
3. 包含全部章节: 架构升级, QuantScript v4, 事件模型, Risk Plane, ExecutionMachine, 运行时模式, 兼容桥, 前端真源, 学习流水线, 静态契约束, GP/超级规范化更新, 质量门禁
4. 不删除或修改任何已有版本的 CHANGELOG 条目
