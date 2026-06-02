# QuantPilot 文档索引

> v4.7.0 代码基线 / v4.15.0 三矩阵完全接管 / v4.16.0 模块化抽离规划 | 最后更新 2026-05-30

## 从这里开始

| 文档 | 路径 | 说明 |
|------|------|------|
| 项目 README | `../README.md` | 产品边界、快速启动、质量门禁 |
| 当前状态 | `./overview-current-status-and-roadmap.md` | 版本路线、仓库状态 |
| 三矩阵治理 | `../00-matrix-governance/README.md` | 流程矩阵、规范矩阵、引导矩阵、模块树和提案流程 |
| 支持矩阵 | `../03-implementation/governance/implementation-support-matrix.md` | supported / restricted / unsupported 产品边界 |
| General_Policy | `../General_Policy.md` | §1-§8 全部开发规范 |
| 超级规范化 | `../01-principles/principles-super-standardization.md` | 五条流水线的执行标准 |

## 三矩阵治理 (`00-matrix-governance/`)

| 文档 | 说明 |
|------|------|
| `README.md` | 三矩阵治理总入口 |
| `process-matrix.md` | 流程矩阵 |
| `standard-matrix.md` | 规范矩阵 |
| `guidance-matrix.md` | 引导矩阵 |
| `module-tree.md` | 模块树白箱网络 |
| `proposal-flow.md` | 提案状态机、三档执行判定表和模板 |
| `proposal-examples.md` | 轻量、标准、重型三档提案样例 |
| `release-transition-protocol.md` | 发布过渡期连接协议 |
| `landing-roadmap.md` | v4.12.0 至 v4.16.0 治理落地与模块化抽离路线 |
| `recursive-speed-protocol.md` | v4.16+ 递归模块化高速执行协议 |
| `recursive-state.json` | 当前递归状态游标 |

## 架构原则 (`01-principles/`)

| 文档 | 说明 |
|------|------|
| `principles-quantpilot-design.md` | 系统设计哲学 |
| `principles-data-and-intent-layer.md` | 数据层与意图层设计 |
| `principles-super-standardization.md` | 超级规范化 — 开发/检查/审计/优化流水线 |

## 协议规范 (`02-protocol/`)

| 文档 | 说明 |
|------|------|
| `README.md` | RFC 索引 + 状态矩阵 |
| `RFC-001` ~ `RFC-020` | 运行时协议全链路 (数据→意图→代理→风险→执行→事件) |

## 实现规范 (`03-implementation/`)

| 子目录 | 说明 |
|--------|------|
| `governance/` | 能力治理、编译链合约、QS 保留界面、支持矩阵、插件存储标准 |
| `runtime/` | 沙盒、回测、持久化、测试模块、存储生命周期 |
| `frontend/` | 前端实施方案 |
| `quantscript/` | QS 设计笔记、执行假设、风险配置 |

## 操作指南 (`04-guides/`)

| 文档 | 说明 |
|------|------|
| `guide-api-reference.md` | API 参考 |
| `guide-formal-quantscript-syntax.md` | 正式 QuantScript 语法 |
| `guide-quantscript-trunk-baseline.md` | QuantScript 主干基线 |
| `guide-paper-to-strategy-development.md` | Paper 运行到策略开发 |
| `guide-strategy-template-library.md` | 策略模板库 |
| `guide-user-guide-zh.md` | 中文用户指南 |
| `guide-user-guide-en.md` | English user guide |

## 测试 (`05-testing/`)

| 文档 | 说明 |
|------|------|
| `全量审计报告-2026-05-10.md` | 最新审计报告 |
| `测试自动化脚本化方案.md` | 测试自动化方案 |

## 里程碑 (`06-milestones/`)

| 版本 | 状态 |
|------|:--:|
| v4.16.0 | 推进中: 模块化抽离第一波；system 已完成当前范围 closeout，backend 已完成 run/backtest 当前递归链路与 `backend.runtime.routes.mutation` route facade closeout；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout，设置 `stop_split: false`；BE-001AJ-02 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 抽离方案，下一步只能进入 BE-001AJ-03 实际抽离 |
| v4.16.0 / BE-001R-01 | 新增: `runtime.backtest.execution_start.legacy_dispatch` 单子叶等价基线已建立，冻结 legacy compile/sandbox dispatch，当前 `no code movement` |
| v4.16.0 / BE-001R-02 | 新增: `runtime.backtest.execution_start.legacy_dispatch` 抽离方案已建立，下一批只允许 legacy compile/sandbox dispatch 最小 helper |
| v4.16.0 / BE-001R-03 | 新增: `runtime.backtest.execution_start.legacy_dispatch` 第一轮物理抽离已完成，legacy compile/sandbox dispatch 已迁入 `src/runtime/backtest/legacy_dispatch.rs` |
| v4.16.0 / BE-001R-04 | 新增: `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 已完成，确认等价并设置 `stop_split: true` |
| v4.16.0 / BE-001S-01 | 新增: `runtime.backtest.execution_start` 父叶残余判断已完成，下一候选回到 `runtime.backtest.record_store` |
| v4.16.0 / BE-001T-01 | 新增: `runtime.backtest.record_store` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001T-02 | 新增: `runtime.backtest.record_store` 抽离方案已建立，下一批只允许四个 handler 最小迁移，当前 `no code movement` |
| v4.16.0 / BE-001T-03 | 新增: `runtime.backtest.record_store` 第一轮物理抽离已完成，四个 handler 已迁入 `src/runtime/backtest/record_store.rs` |
| v4.16.0 / BE-001T-04 | 新增: `runtime.backtest.record_store` 单叶 closeout 已完成，确认等价并设置 `stop_split: true` |
| v4.16.0 / BE-001U-01 | 新增: `runtime.backtest.replay` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001U-02 | 新增: `runtime.backtest.replay` 抽离方案已建立，下一批只允许迁移 `get_backtest_replay` |
| v4.16.0 / BE-001U-03 | 新增: `runtime.backtest.replay` 抽离记录已建立，`get_backtest_replay` 已迁入 `src/runtime/backtest/replay.rs` |
| v4.16.0 / BE-001U-04 | 新增: `runtime.backtest.replay` 单叶 closeout 已完成，当前设置 `stop_split: true` |
| v4.16.0 / BE-001V-01 | 新增: `runtime.backtest.experiment_sweep` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001V-02 | 新增: `runtime.backtest.experiment_sweep` 抽离方案已建立，下一批只允许迁移 experiment handler/helper |
| v4.16.0 / BE-001V-03 | 新增: `runtime.backtest.experiment_sweep` 抽离记录已建立，experiment handler/helper 已迁入 `src/runtime/backtest/experiment_sweep.rs` |
| v4.16.0 / BE-001V-04 | 新增: `runtime.backtest.experiment_sweep` 单叶 closeout 已完成，`stop_split: false`，下一候选为 `parameter_grid` |
| v4.16.0 / BE-001W-01 | 新增: `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001W-02 | 新增: `runtime.backtest.experiment_sweep.parameter_grid` 抽离方案已建立，下一批只允许迁移 3 个 helper，当前 `no code movement` |
| v4.16.0 / BE-001W-03 | 新增: `runtime.backtest.experiment_sweep.parameter_grid` 抽离记录已建立，3 个 helper 已迁入 `src/runtime/backtest/parameter_grid.rs` |
| v4.16.0 / BE-001W-04 | 新增: `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 已完成，`stop_split: true`，下一步回到 `experiment_sweep` 父叶残余判断 |
| v4.16.0 / BE-001X-01 | 新增: `runtime.backtest.experiment_sweep` 父叶残余判断已完成，下一候选为 `start_orchestration` |
| v4.16.0 / BE-001Y-01 | 新增: `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001Y-02 | 新增: `runtime.backtest.experiment_sweep.start_orchestration` 抽离方案已建立，下一批只允许迁移 `start_backtest_experiment`，当前 `no code movement` |
| v4.16.0 / BE-001Y-03 | 新增: `runtime.backtest.experiment_sweep.start_orchestration` 抽离记录已建立，`start_backtest_experiment` 已迁入 `src/runtime/backtest/start_orchestration.rs` |
| v4.16.0 / BE-001Y-04 | 新增: `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 已完成，`stop_split: true`，下一步回到 `experiment_sweep` 父叶残余判断 |
| v4.16.0 / BE-001Z-01 | 新增: `runtime.backtest.experiment_sweep` 第二轮父叶残余判断已完成，下一候选为 `record_lifecycle` |
| v4.16.0 / BE-001AA-01 | 新增: `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001AA-02 | 新增: `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案已建立，下一批只允许迁移四个 lifecycle handler |
| v4.16.0 / BE-001AA-03 | 新增: `runtime.backtest.experiment_sweep.record_lifecycle` 抽离记录已建立，四个 lifecycle handler 已迁入 `src/runtime/backtest/record_lifecycle.rs` |
| v4.16.0 / BE-001AA-04 | 新增: `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout 已完成，`stop_split: true`，下一步回到 `experiment_sweep` 父叶残余判断 |
| v4.16.0 / BE-001AB-01 | 新增: `runtime.backtest.experiment_sweep` 第三轮父叶残余判断已完成，三子叶均 `stop_split: true`，父叶也设置 `stop_split: true`，下一步回到 `runtime.backtest` 父叶残余判断 |
| v4.16.0 / BE-001AC-01 | 新增: `runtime.backtest` 父叶残余判断已完成，当前 handler 域设置 `stop_split: true`，下一步回到 `backend.runtime.routes` 父叶残余判断 |
| v4.16.0 / BE-001AD-01 | 新增: `backend.runtime.routes` 父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 `backend.runtime.routes.mutation` 单子叶等价基线 |
| v4.16.0 / BE-001AE-01 | 新增: `backend.runtime.routes.mutation` 单子叶等价基线已建立，冻结 mutation / AI proposal / approval route group，当前 `no code movement` |
| v4.16.0 / BE-001AE-02 | 新增: `backend.runtime.routes.mutation` 抽离方案已建立，下一步只允许 route facade 最小物理抽离 |
| v4.16.0 / BE-001AE-03 | 新增: `backend.runtime.routes.mutation` route facade 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001AE-04 | 新增: `backend.runtime.routes.mutation` 单叶 closeout 已完成，route facade 设置 `stop_split: true`，已由 BE-001AF-01 `runtime.mutation.parameter_mutation` 基线承接 |
| v4.16.0 / BE-001AF-01 | 新增: `runtime.mutation.parameter_mutation` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AF-02 抽离方案 |
| v4.16.0 / BE-001AF-02 | 新增: `runtime.mutation.parameter_mutation` 抽离方案已建立，当前仍是 `no code movement`，下一步进入 BE-001AF-03 实际抽离 |
| v4.16.0 / BE-001AF-03 | 新增: `runtime.mutation.parameter_mutation` 实际抽离已完成，五个 parameter mutation handler 迁入 `src/runtime/mutation/parameter_mutation.rs`，下一步进入 BE-001AF-04 单叶 closeout |
| v4.16.0 / BE-001AF-04 | 新增: `runtime.mutation.parameter_mutation` 单叶 closeout 已完成，设置 `stop_split: false`，下一步进入 BE-001AG-01 `transition_lifecycle` 单子叶等价基线 |
| v4.16.0 / BE-001AG-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AG-02 抽离方案 |
| v4.16.0 / BE-001AG-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AG-03 实际抽离 |
| v4.16.0 / BE-001AG-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 实际抽离已完成，下一步进入 BE-001AG-04 单叶 closeout |
| v4.16.0 / BE-001AG-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 单叶 closeout 已完成，设置 `stop_split: false`，下一步进入 BE-001AH-01 `boundary_safety` 单子叶等价基线 |
| v4.16.0 / BE-001AH-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AH-02 抽离方案 |
| v4.16.0 / BE-001AH-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AH-03 实际抽离 |
| v4.16.0 / BE-001AH-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 实际抽离已完成，下一步进入 BE-001AH-04 单叶 closeout |
| v4.16.0 / BE-001AH-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单叶 closeout 已完成，设置 `stop_split: true`，下一步进入 BE-001AI-01 父叶残余判断 |
| v4.16.0 / BE-001AI-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 BE-001AJ-01 `activation_flow` 单子叶等价基线 |
| v4.16.0 / BE-001AJ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AJ-02 抽离方案 |
| v4.16.0 / BE-001AJ-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AJ-03 实际抽离 |
| v4.16.0 / BE-001AJ-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 实际抽离已完成，`activate_runtime_parameter_mutation` 已迁入 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` |
| v4.16.0 / BE-001AJ-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单叶 closeout 已完成，`stop_split: true`，下一步回到 `transition_lifecycle` 父叶残余判断 |
| v4.16.0 / BE-001AK-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 第二轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 `rollback_flow` 单子叶等价基线 |
| v4.16.0 / BE-001AL-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AL-02 抽离方案 |
| v4.16.0 / BE-001AL-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AL-03 实际抽离 |
| v4.16.0 / BE-001AL-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 实际抽离已完成，`rollback_runtime_parameter_mutation` 已迁入 child |
| v4.16.0 / BE-001AL-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单叶 closeout 已完成，`stop_split: true`，下一步回到 `transition_lifecycle` 父叶残余判断 |
| v4.16.0 / BE-001AM-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 第三轮父叶残余判断已完成，下一候选为 `activation_snapshot_side_effect` |
| v4.16.0 / BE-001AN-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单子叶等价基线已建立，当前 `no code movement` |
| v4.16.0 / BE-001AN-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 抽离方案已建立，下一步进入 BE-001AN-03 实际抽离 |
| v4.16.0 / BE-001AN-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 实际抽离已完成，下一步进入 BE-001AN-04 单叶 closeout |
| v4.16.0 / BE-001AN-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout 已完成，`stop_split: true` |
| v4.16.0 / BE-001AO-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 第四轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 BE-001AP-01 `transition_record_persistence` 单子叶等价基线 |
| v4.16.0 / BE-001AP-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AP-02 抽离方案 |
| v4.16.0 / BE-001AP-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AP-03 实际抽离 |
| v4.16.0 / BE-001AP-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 实际抽离已完成，下一步进入 BE-001AP-04 单叶 closeout |
| v4.16.0 / BE-001AP-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单叶 closeout 已完成，设置 `stop_split: true`，下一步进入 BE-001AQ-01 父叶残余判断 |
| v4.16.0 / BE-001AQ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 BE-001AR-01 `rollback_record_identity` 单子叶等价基线 |
| v4.16.0 / BE-001AR-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AR-02 抽离方案 |
| v4.16.0 / BE-001AR-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AR-03 实际抽离 |
| v4.16.0 / BE-001AR-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 实际抽离已完成，下一步进入 BE-001AR-04 单叶 closeout |
| v4.16.0 / BE-001AR-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001AS-01 父叶残余判断 |
| v4.16.0 / BE-001AS-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断已完成，父叶设置 `stop_split: true`，下一步进入 BE-001AT-01 |
| v4.16.0 / BE-001AT-01 | 新增: `runtime.mutation.parameter_mutation` 父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 BE-001AU-01 `proposal_creation` |
| v4.16.0 / BE-001AU-01 | 新增: `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AU-02 抽离方案 |
| v4.16.0 / BE-001AU-02 | 新增: `runtime.mutation.parameter_mutation.proposal_creation` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AU-03 实际抽离 |
| v4.16.0 / BE-001AU-03 | 新增: `runtime.mutation.parameter_mutation.proposal_creation` 实际抽离已完成，下一步进入 BE-001AU-04 单叶 closeout |
| v4.16.0 / BE-001AU-04 | 新增: `runtime.mutation.parameter_mutation.proposal_creation` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001AV-01 父叶残余判断 |
| v4.16.0 / BE-001AV-01 | 新增: `runtime.mutation.parameter_mutation` 第二轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 BE-001AW-01 `record_query` |
| v4.16.0 / BE-001AW-01 | 新增: `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AW-02 抽离方案 |
| v4.16.0 / BE-001AW-02 | 新增: `runtime.mutation.parameter_mutation.record_query` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AW-03 实际抽离 |
| v4.16.0 / BE-001AW-03 | 新增: `runtime.mutation.parameter_mutation.record_query` 实际抽离已完成，list/detail handler 已迁入 child，下一步进入 BE-001AW-04 单叶 closeout |
| v4.16.0 / BE-001AW-04 | 新增: `runtime.mutation.parameter_mutation.record_query` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001AX-01 父叶残余判断 |
| v4.16.0 / BE-001AX-01 | 新增: `runtime.mutation.parameter_mutation` 第三轮父叶残余判断已完成，父叶设置 `stop_split: true`，下一步进入 BE-001AY-01 `runtime.mutation.ai_proposal` |
| v4.16.0 / BE-001AY-01 | 新增: `runtime.mutation.ai_proposal` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AY-02 抽离方案 |
| v4.16.0 / BE-001AY-02 | 新增: `runtime.mutation.ai_proposal` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AY-03 实际抽离 |
| v4.16.0 / BE-001AY-03 | 新增: `runtime.mutation.ai_proposal` 实际抽离已完成，AI proposal / approval handlers 已迁入 child，下一步进入 BE-001AY-04 单叶 closeout |
| v4.16.0 / BE-001AY-04 | 新增: `runtime.mutation.ai_proposal` 单叶 closeout 已完成，`stop_split: false`，下一步进入 BE-001AZ-01 `static_check` |
| v4.16.0 / BE-001AZ-01 | 新增: `runtime.mutation.ai_proposal.static_check` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001AZ-02 抽离方案 |
| v4.16.0 / BE-001AZ-02 | 新增: `runtime.mutation.ai_proposal.static_check` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001AZ-03 实际抽离 |
| v4.16.0 / BE-001AZ-03 | 新增: `runtime.mutation.ai_proposal.static_check` 实际抽离已完成，helper 与静态检查单测迁入 child，下一步进入 BE-001AZ-04 单叶 closeout |
| v4.16.0 / BE-001AZ-04 | 新增: `runtime.mutation.ai_proposal.static_check` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001BA-01 父叶残余判断 |
| v4.16.0 / BE-001BA-01 | 新增: `runtime.mutation.ai_proposal` 父叶残余判断已完成，父叶 `stop_split: false`，下一候选为 `source_governance_identity` |
| v4.16.0 / BE-001BB-01 | 新增: `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BB-02 抽离方案 |
| v4.16.0 / BE-001BB-02 | 新增: `runtime.mutation.ai_proposal.source_governance_identity` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BB-03 实际抽离 |
| v4.16.0 / BE-001BB-03 | 新增: `runtime.mutation.ai_proposal.source_governance_identity` 实际抽离已完成，source/governance/id helper 迁入 child，下一步进入 BE-001BB-04 单叶 closeout |
| v4.16.0 / BE-001BB-04 | 新增: `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001BC-01 父叶残余判断 |
| v4.16.0 / BE-001BC-01 | 新增: `runtime.mutation.ai_proposal` 第二轮父叶残余判断已完成，父叶 `stop_split: false`，下一候选为 `event_lifecycle` |
| v4.16.0 / BE-001BD-01 | 新增: `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BD-02 抽离方案 |
| v4.16.0 / BE-001BD-02 | 新增: `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BD-03 实际抽离 |
| v4.16.0 / BE-001BD-03 | 新增: `runtime.mutation.ai_proposal.event_lifecycle` 实际抽离已完成，下一步进入 BE-001BD-04 单叶 closeout |
| v4.16.0 / BE-001BD-04 | 新增: `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout 已完成并设置 `stop_split: true`，下一步进入 BE-001BE-01 父叶残余判断 |
| v4.16.0 / BE-001BE-01 | 新增: `runtime.mutation.ai_proposal` 第三轮父叶残余判断已完成，下一步进入 BE-001BF-01 `record_query` 单子叶等价基线 |
| v4.16.0 / BE-001BF-01 | 新增: `runtime.mutation.ai_proposal.record_query` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BF-02 抽离方案 |
| v4.16.0 / BE-001BF-02 | 新增: `runtime.mutation.ai_proposal.record_query` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BF-03 实际抽离 |
| v4.16.0 / BE-001BF-03 | 新增: `runtime.mutation.ai_proposal.record_query` 实际抽离已完成，下一步进入 BE-001BF-04 单叶 closeout |
| v4.16.0 / BE-001BF-04 | 新增: `runtime.mutation.ai_proposal.record_query` 单叶 closeout 已完成并设置 `stop_split: true`，下一步进入 BE-001BG-01 父叶残余判断 |
| v4.16.0 / BE-001BG-01 | 新增: `runtime.mutation.ai_proposal` 第四轮父叶残余判断已完成，下一步进入 BE-001BH-01 `approval_review` 单子叶等价基线 |
| v4.16.0 / BE-001BH-01 | 新增: `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BH-02 抽离方案 |
| v4.16.0 / BE-001BH-02 | 新增: `runtime.mutation.ai_proposal.approval_review` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BH-03 实际抽离 |
| v4.16.0 / BE-001BH-03 | 新增: `runtime.mutation.ai_proposal.approval_review` 实际抽离已完成，五个 approval handler 已迁入 child，下一步进入 BE-001BH-04 单叶 closeout |
| v4.16.0 / BE-001BH-04 | 新增: `runtime.mutation.ai_proposal.approval_review` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001BI-01 父叶残余判断 |
| v4.16.0 / BE-001BI-01 | 新增: `runtime.mutation.ai_proposal` 第五轮父叶残余判断已完成，下一候选为 `approval_persistence` |
| v4.16.0 / BE-001BJ-01 | 新增: `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BJ-02 抽离方案 |
| v4.16.0 / BE-001BJ-02 | 新增: `runtime.mutation.ai_proposal.approval_persistence` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BJ-03 实际抽离 |
| v4.16.0 / BE-001BJ-03 | 新增: `runtime.mutation.ai_proposal.approval_persistence` 实际抽离已完成，两个 persistence helper 已迁入 child，下一步进入 BE-001BJ-04 单叶 closeout |
| v4.16.0 / BE-001BJ-04 | 新增: `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001BK-01 父叶残余判断 |
| v4.16.0 / BE-001BK-01 | 新增: `runtime.mutation.ai_proposal` 第六轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一候选为 `sandbox_trigger` |
| v4.16.0 / BE-001BL-01 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BL-02 抽离方案 |
| v4.16.0 / BE-001BL-02 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BL-03 实际抽离 |
| v4.16.0 / BE-001BL-03 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger` 实际抽离已完成，下一步进入 BE-001BL-04 单叶 closeout |
| v4.16.0 / BE-001BL-04 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout 已完成，设置 `stop_split: true`，下一步进入 BE-001BM-01 父叶残余判断 |
| v4.16.0 / BE-001BM-01 | 新增: `runtime.mutation.ai_proposal` 第七轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一候选为 `status_transition` |
| v4.16.0 / BE-001BN-01 | 新增: `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BN-02 抽离方案 |
| v4.16.0 / BE-001BN-02 | 新增: `runtime.mutation.ai_proposal.status_transition` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BN-03 实际抽离 |
| v4.16.0 / BE-001BN-03 | 新增: `runtime.mutation.ai_proposal.status_transition` 实际抽离已完成，三个状态 helper 已迁入 child，下一步进入 BE-001BN-04 单叶 closeout |
| v4.16.0 / BE-001BN-04 | 新增: `runtime.mutation.ai_proposal.status_transition` 单叶 closeout 已完成，`stop_split: true`，下一步进入 BE-001BO-01 父叶残余判断 |
| v4.16.0 / BE-001BO-01 | 新增: `runtime.mutation.ai_proposal` 第八轮父叶残余判断已完成，下一候选为 `proposal_creation` |
| v4.16.0 / BE-001BP-01 | 新增: `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线已建立，当前 `no code movement`，下一步进入 BE-001BP-02 抽离方案 |
| v4.16.0 / BE-001BP-02 | 新增: `runtime.mutation.ai_proposal.proposal_creation` 抽离方案已建立，当前 `no code movement`，下一步进入 BE-001BP-03 实际抽离 |
| v4.16.0 / BE-001BP-03 | 新增: `runtime.mutation.ai_proposal.proposal_creation` 实际抽离已完成，下一步进入 BE-001BP-04 单叶 closeout |
| v4.16.0 / BE-001BP-04 | 新增: `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout 已完成，设置 `stop_split: true` |
| v4.16.0 / BE-001BQ-01 | 新增: `runtime.mutation.ai_proposal` 父叶残余判断已完成，设置 `stop_split: true` |
| v4.16.0 / BE-001BR-01 | 新增: `backend.runtime.routes` 第二轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 `backend.runtime.routes.experiment` 单子叶等价基线 |
| v4.16.0 / BE-001BS-01 | 新增: `backend.runtime.routes.experiment` 单子叶等价基线已建立，当前 `no code movement`，下一步进入抽离方案 |
| v4.16.0 / BE-001BS-02 | 新增: `backend.runtime.routes.experiment` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001BS-03 | 新增: `backend.runtime.routes.experiment` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001BS-04 | 新增: `backend.runtime.routes.experiment` 单叶 closeout 已完成并设置 `stop_split: true`，下一步回到 `backend.runtime.routes` 父叶残余判断 |
| v4.16.0 / BE-001BT-01 | 新增: `backend.runtime.routes` 第三轮父叶残余判断已完成，下一步进入 `backend.runtime.routes.evidence` 单子叶等价基线 |
| v4.16.0 / BE-001BU-01 | 新增: `backend.runtime.routes.evidence` 单子叶等价基线已建立，当前 `no code movement`，下一步进入抽离方案 |
| v4.16.0 / BE-001BU-02 | 新增: `backend.runtime.routes.evidence` 抽离方案已建立，当前 `no code movement`，下一步进入实际抽离 |
| v4.16.0 / BE-001BU-03 | 新增: `backend.runtime.routes.evidence` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001BU-04 | 新增: `backend.runtime.routes.evidence` 单叶 closeout 已完成并设置 `stop_split: true`，下一步回到父叶残余判断 |
| v4.16.0 / BE-001BV-01 | 新增: `backend.runtime.routes` 第四轮父叶残余判断已完成，下一步进入 `backend.runtime.routes.event_stream` 单子叶等价基线 |
| v4.16.0 / BE-001BW-01 | 新增: `backend.runtime.routes.event_stream` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001BW-02 | 新增: `backend.runtime.routes.event_stream` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001BW-03 | 新增: `backend.runtime.routes.event_stream` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001BW-04 | 新增: `backend.runtime.routes.event_stream` 单叶 closeout 已完成，下一步回到父叶残余判断 |
| v4.16.0 / BE-001BX-01 | 新增: `backend.runtime.routes` 第五轮父叶残余判断已完成，下一步进入 `backend.runtime.routes.report_ops` 单子叶等价基线 |
| v4.16.0 / BE-001BY-01 | 新增: `backend.runtime.routes.report_ops` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001BY-02 | 新增: `backend.runtime.routes.report_ops` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001BY-03 | 新增: `backend.runtime.routes.report_ops` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001BY-04 | 新增: `backend.runtime.routes.report_ops` 单叶 closeout 已完成，下一步进入父叶残余判断 |
| v4.16.0 / BE-001BZ-01 | 新增: `backend.runtime.routes` 第六轮父叶残余判断已完成，route aggregate 设置 `stop_split: true` |
| v4.16.0 / BE-001CA-01 | 新增: `backend.runtime` 父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 `runtime.report_ops` 单子叶等价基线 |
| v4.16.0 / BE-001CB-01 | 新增: `runtime.report_ops` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CB-02 | 新增: `runtime.report_ops` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CB-03 | 新增: `runtime.report_ops` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CB-04 | 新增: `runtime.report_ops` 单叶 closeout 已完成，设置 `stop_split: false` 并锁定下一步 `runtime.report_ops.runtime_report` 基线 |
| v4.16.0 / BE-001CC-01 | 新增: `runtime.report_ops.runtime_report` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CC-02 | 新增: `runtime.report_ops.runtime_report` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CC-03 | 新增: `runtime.report_ops.runtime_report` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CC-04 | 新增: `runtime.report_ops.runtime_report` 单叶 closeout 已完成，设置 `stop_split: true`，下一步进入父叶残余判断 |
| v4.16.0 / BE-001CD-01 | 新增: `runtime.report_ops` 父叶残余判断已完成，下一步进入 `v1_report_endpoints` 基线 |
| v4.16.0 / BE-001CE-01 | 新增: `runtime.report_ops.v1_report_endpoints` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CE-02 | 新增: `runtime.report_ops.v1_report_endpoints` test-first 抽离方案已建立，下一步先补 endpoint smoke |
| v4.16.0 / BE-001CE-03 | 新增: `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测已完成，下一步进入实际抽离 |
| v4.16.0 / BE-001CE-04 | 新增: `runtime.report_ops.v1_report_endpoints` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CE-05 | 新增: `runtime.report_ops.v1_report_endpoints` 单叶 closeout 已完成，下一步进入父叶残余判断 |
| v4.16.0 / BE-001CF-01 | 新增: `runtime.report_ops` 父叶残余判断已完成，下一步进入 `merge_generation_health` 基线 |
| v4.16.0 / BE-001CG-01 | 新增: `runtime.report_ops.merge_generation_health` 单子叶等价基线已建立，当前 `no code movement`，下一步进入抽离方案 |
| v4.16.0 / BE-001CG-02 | 新增: `runtime.report_ops.merge_generation_health` test-first 抽离方案已建立，下一步进入 endpoint smoke 补测 |
| v4.16.0 / BE-001CG-03 | 新增: `runtime.report_ops.merge_generation_health` endpoint smoke 补测已完成，下一步进入实际抽离 |
| v4.16.0 / BE-001CG-04 | 新增: `runtime.report_ops.merge_generation_health` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CG-05 | 新增: `runtime.report_ops.merge_generation_health` 单叶 closeout 已完成，下一步进入父叶残余判断 |
| v4.16.0 / BE-001CH-01 | 新增: `runtime.report_ops` 第二轮父叶残余判断已完成，父叶设置 `stop_split: true` |
| v4.16.0 / BE-001CI-01 | 新增: `backend.runtime` 第二轮父叶残余判断已完成，下一步进入 `runtime.evidence_health` 单子叶等价基线 |
| v4.16.0 / BE-001CJ-01 | 新增: `runtime.evidence_health` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CJ-02 | 新增: `runtime.evidence_health` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CJ-03 | 新增: `runtime.evidence_health` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CJ-04 | 新增: `runtime.evidence_health` 单叶 closeout 已完成，下一步进入 `backend.runtime` 第三轮父叶残余判断 |
| v4.16.0 / BE-001CK-01 | 新增: `backend.runtime` 第三轮父叶残余判断已完成，下一步进入 `runtime.mutation.shared_governance` 单子叶等价基线 |
| v4.16.0 / BE-001CL-01 | 新增: `runtime.mutation.shared_governance` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CL-02 | 新增: `runtime.mutation.shared_governance` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CL-03 | 新增: `runtime.mutation.shared_governance` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CL-04 | 新增: `runtime.mutation.shared_governance` 单叶 closeout 已完成，设置 `stop_split: true`，下一步进入 `backend.runtime` 第四轮父叶残余判断 |
| v4.16.0 / BE-001CM-01 | 新增: `backend.runtime` 第四轮父叶残余判断已完成，父叶保持 `stop_split: false`，下一步进入 `runtime.query_support` 单子叶等价基线 |
| v4.16.0 / BE-001CN-01 | 新增: `runtime.query_support` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CN-02 | 新增: `runtime.query_support` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CN-03 | 新增: `runtime.query_support` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CN-04 | 新增: `runtime.query_support` 单叶 closeout 已完成，下一步进入 `backend.runtime` 第五轮父叶残余判断 |
| v4.16.0 / BE-001CO-01 | 新增: `backend.runtime` 第五轮父叶残余判断已完成，下一步进入 `runtime.response_support` 单子叶等价基线 |
| v4.16.0 / BE-001CP-01 | 新增: `runtime.response_support` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CP-02 | 新增: `runtime.response_support` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CP-03 | 新增: `runtime.response_support` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CP-04 | 新增: `runtime.response_support` 单叶 closeout 已完成，下一步进入 `backend.runtime` 第六轮父叶残余判断 |
| v4.16.0 / BE-001CQ-01 | 新增: `backend.runtime` 第六轮父叶残余判断已完成，下一步进入 `runtime.run_guard` 单子叶等价基线 |
| v4.16.0 / BE-001CR-01 | 新增: `runtime.run_guard` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CR-02 | 新增: `runtime.run_guard` 抽离方案已建立，下一步进入实际抽离 |
| v4.16.0 / BE-001CR-03 | 新增: `runtime.run_guard` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CR-04 | 新增: `runtime.run_guard` 单叶 closeout 已完成，下一步进入 `backend.runtime` 第七轮父叶残余判断 |
| v4.16.0 / BE-001CS-01 | 新增: `backend.runtime` 第七轮父叶残余判断已完成，下一步进入 `runtime.experiment_limit` 单子叶等价基线 |
| v4.16.0 / BE-001CT-01 | 新增: `runtime.experiment_limit` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CT-02 | 新增: `runtime.experiment_limit` test-first 抽离方案已建立，下一步进入 endpoint smoke 补测 |
| v4.16.0 / BE-001CT-03 | 新增: `runtime.experiment_limit` endpoint smoke 补测已完成，下一步进入实际抽离 |
| v4.16.0 / BE-001CT-04 | 新增: `runtime.experiment_limit` 实际抽离已完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001CT-05 | 新增: `runtime.experiment_limit` 单叶 closeout 已完成，下一步进入 `backend.runtime` 第八轮父叶残余判断 |
| v4.16.0 / BE-001CU-01 | 新增: `backend.runtime` 第八轮父叶残余判断已完成，下一步进入 `runtime.parent_include_cleanup` 单子叶等价基线 |
| v4.16.0 / BE-001CV-01 | 新增: `runtime.parent_include_cleanup` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001CV-02 | 新增: `runtime.parent_include_cleanup` 抽离方案已建立，下一步进入实际 cleanup |
| v4.16.0 / BE-001CV-03 | 新增: `runtime.parent_include_cleanup` 实际 cleanup 已完成，下一步进入 `backend.runtime` 第九轮父叶残余判断 |
| v4.16.0 / BE-001CW-01 | 新增: `backend.runtime` 第九轮父叶残余判断已完成，下一步进入 `runtime.parent_import_bridge` 单子叶等价基线 |
| v4.16.0 / BE-001CX-01 | 新增: `runtime.parent_import_bridge` 单子叶等价基线已建立，冻结 46 文件 parent import bridge 依赖面 |
| v4.16.0 / BE-001CX-02 | 新增: `runtime.parent_import_bridge` 抽离方案已建立，固定 staged explicit import pass 与首批 root support pilot |
| v4.16.0 / BE-001CX-03 | 新增: `runtime.root_support_import_pilot` 抽离记录已建立，`query_support` 与 `response_support` parent wildcard import 已收敛 |
| v4.16.0 / BE-001CX-04 | 新增: `runtime.root_support_import_pilot` 单叶 closeout 已完成，`stop_split: true` 并转入 root entry import pass |
| v4.16.0 / BE-001CY-01 | 新增: `runtime.root_entry_import_pass` 单子叶等价基线已建立，冻结 root entry 候选与 test-only super import 判定 |
| v4.16.0 / BE-001CY-02 | 新增: `runtime.root_entry_import_pass` 抽离方案已建立，固定 two-handler root entry pilot 与 BE-001CY-03 文件边界 |
| v4.16.0 / BE-001CY-03 | 新增: `runtime.root_entry_import_pass` 抽离记录已建立，`event_stream` 与 `evidence_health` parent wildcard import 已收敛 |
| v4.16.0 / BE-001CY-04 | 新增: `runtime.root_entry_import_pass` 单叶 closeout 已完成，`stop_split: true` 并转入 report_ops import pass |
| v4.16.0 / BE-001CZ-01 | 新增: `runtime.report_ops_import_pass` 单子叶等价基线已建立，冻结 report_ops facade 与 3 child import 转运风险 |
| v4.16.0 / BE-001CZ-02 | 新增: `runtime.report_ops_import_pass` 抽离方案已建立，固定 report_ops four-file pocket 同批处理 |
| v4.16.0 / BE-001CZ-03 | 新增: `runtime.report_ops_import_pass` 抽离记录已建立，report_ops four-file pocket parent wildcard import 已收敛 |
| v4.16.0 / BE-001CZ-04 | 新增: `runtime.report_ops_import_pass` 单叶 closeout 已建立，设置 `stop_split: true` 并回到 parent import bridge 残余判断 |
| v4.16.0 / BE-001DA-01 | 新增: `runtime.parent_import_bridge` 父叶残余判断已建立，选择 `runtime.run_import_pass` 为下一候选 |
| v4.16.0 / BE-001DB-01 | 新增: `runtime.run_import_pass` 单子叶等价基线已建立，冻结 4 个 run child import 边界 |
| v4.16.0 / BE-001DB-02 | 新增: `runtime.run_import_pass` 抽离方案已建立，固定 4 个 run child 同批 explicit import rewrite |
| v4.16.0 / BE-001DB-03 | 新增: `runtime.run_import_pass` 抽离记录已建立，4 个 run child parent wildcard import 已收敛 |
| v4.16.0 / BE-001DB-04 | 新增: `runtime.run_import_pass` 单叶 closeout 已建立，设置 `stop_split: true` 并回到 parent import bridge 残余判断 |
| v4.16.0 / BE-001DC-01 | 新增: `runtime.parent_import_bridge` 父叶残余判断已建立，选择 `runtime.backtest_import_pass` 为下一候选 |
| v4.16.0 / BE-001DD-01 | 新增: `runtime.backtest_import_pass` 单子叶等价基线已建立，冻结 11 个 backtest import 残余文件 |
| v4.16.0 / BE-001DD-02 | 新增: `runtime.backtest_import_pass` 抽离方案已建立，选择 `runtime.backtest.record_store_import_pass` 为下一候选 |
| v4.16.0 / BE-001DE-01 | 新增: `runtime.backtest.record_store_import_pass` 单子叶等价基线已建立，冻结 `record_store.rs` import 输入面 |
| v4.16.0 / BE-001DE-02 | 新增: `runtime.backtest.record_store_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DE-03 | 新增: `runtime.backtest.record_store_import_pass` 抽离记录已建立，`record_store.rs` parent wildcard import 已收敛 |
| v4.16.0 / BE-001DE-04 | 新增: `runtime.backtest.record_store_import_pass` 单叶 closeout 已建立，设置 `stop_split: true` 并回到父叶残余判断 |
| v4.16.0 / BE-001DF-01 | 新增: `runtime.backtest_import_pass` 父叶残余判断已建立，选择 `runtime.backtest.replay_import_pass` 为下一候选 |
| v4.16.0 / BE-001DG-01 | 新增: `runtime.backtest.replay_import_pass` 单子叶等价基线已建立，冻结 `replay.rs` import 输入面 |
| v4.16.0 / BE-001DG-02 | 新增: `runtime.backtest.replay_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DG-03 | 新增: `runtime.backtest.replay_import_pass` 抽离记录已建立，`replay.rs` parent wildcard import 已收敛 |
| v4.16.0 / BE-001DG-04 | 新增: `runtime.backtest.replay_import_pass` 单叶 closeout 已建立，确认 `stop_split: true` |
| v4.16.0 / BE-001DH-01 | 新增: `runtime.backtest_import_pass` 第二轮父叶残余判断已建立，选择 experiment sweep import pass |
| v4.16.0 / BE-001DI-01 | 新增: `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线已建立，冻结四文件 pocket |
| v4.16.0 / BE-001DI-02 | 新增: `runtime.backtest.experiment_sweep_import_pass` 抽离方案已建立，固定四文件 import rewrite |
| v4.16.0 / BE-001DI-03 | 新增: `runtime.backtest.experiment_sweep_import_pass` 抽离记录已建立，四文件 parent import 已收敛 |
| v4.16.0 / BE-001DI-04 | 新增: `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout 已建立，回到父叶残余判断 |
| v4.16.0 / BE-001DJ-01 | 新增: `runtime.backtest_import_pass` 第三轮父叶残余判断已建立，锁定 execution_start import pass |
| v4.16.0 / BE-001DK-01 | 新增: `runtime.backtest.execution_start_import_pass` 单子叶等价基线已建立，冻结五文件 pocket |
| v4.16.0 / BE-001DK-02 | 新增: `runtime.backtest.execution_start_import_pass` 抽离方案已建立，固定五文件 import rewrite |
| v4.16.0 / BE-001DK-03 | 新增: `runtime.backtest.execution_start_import_pass` 抽离记录已建立，backtest import residual 清零 |
| v4.16.0 / BE-001DK-04 | 新增: `runtime.backtest.execution_start_import_pass` 单叶 closeout 已建立，确认 `stop_split: true` |
| v4.16.0 / BE-001DL-01 | 新增: `runtime.backtest_import_pass` 第四轮父叶残余判断已建立，确认 `stop_split: true` |
| v4.16.0 / BE-001DM-01 | 新增: `runtime.parent_import_bridge` 父叶残余判断已建立，锁定 mutation import pass |
| v4.16.0 / BE-001DN-01 | 新增: `runtime.mutation_import_pass` 单子叶等价基线已建立，冻结 21 个 mutation parent bridge 文件 |
| v4.16.0 / BE-001DN-02 | 新增: `runtime.mutation_import_pass` 抽离方案已建立，锁定 shared_governance import pass |
| v4.16.0 / BE-001DO-01 | 新增: `runtime.mutation.shared_governance_import_pass` 单子叶等价基线已建立，冻结 shared_governance import 输入面 |
| v4.16.0 / BE-001DO-02 | 新增: `runtime.mutation.shared_governance_import_pass` 抽离方案已建立，固定单文件 explicit import rewrite |
| v4.16.0 / BE-001DO-03 | 新增: `runtime.mutation.shared_governance_import_pass` 抽离记录已建立，shared_governance parent import 已收敛 |
| v4.16.0 / BE-001DO-04 | 新增: `runtime.mutation.shared_governance_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001DP-01 | 新增: `runtime.mutation_import_pass` 父叶残余判断已建立，选择 parameter_mutation import pass |
| v4.16.0 / BE-001DQ-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线已建立，冻结 10 个 residual 文件 |
| v4.16.0 / BE-001DQ-02 | 新增: `runtime.mutation.parameter_mutation_import_pass` 抽离方案已建立，选择 record_query import pass |
| v4.16.0 / BE-001DR-01 | 新增: `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001DR-02 | 新增: `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DR-03 | 新增: `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离记录已建立，record_query parent import 已收敛 |
| v4.16.0 / BE-001DR-04 | 新增: `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001DS-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断已建立，选择 proposal_creation import pass |
| v4.16.0 / BE-001DT-01 | 新增: `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001DT-02 | 新增: `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DT-03 | 新增: `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离记录已建立，proposal_creation parent import 已收敛 |
| v4.16.0 / BE-001DT-04 | 新增: `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001DU-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断已建立，选择 transition_lifecycle import pass |
| v4.16.0 / BE-001DV-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001DV-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案已建立，选择 boundary_safety import pass |
| v4.16.0 / BE-001DW-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001DW-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DW-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离记录已建立，boundary_safety parent import 已收敛 |
| v4.16.0 / BE-001DW-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001DX-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断已建立，选择 rollback_record_identity import pass |
| v4.15.0 | 当前治理基线: 三矩阵完全接管 closeout |
| v4.14.0 | ✅ 治理门禁自动化 |
| v4.13.0 | ✅ 模块树白箱扩面 |
| v4.12.0 | ✅ 三矩阵治理入口启用 |
| v4.11.0 | 推进中: 策略配置系统一等化 |
| v4.10.0 | ✅ UX 收口与产品边界固化 |
| v4.9.0 | ✅ 产品功能完整度 + 插件执行安全 |
| v4.8.x | ✅ UX/i18n/API 契约收敛 |
| v0.5.2 | ✅ 16/16 |
| v0.5.1 | ✅ 15/15 |
| v0.5.0 | ✅ |
| v0.4.x ~ v0.3.x | ✅ 详见各版本目录 |

## 归档 (`09-archive/`)

已退役的设计笔记、追踪文档、历史审计报告。
| v4.16.0 / BE-001DY-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线已建立，下一步进入 BE-001DY-02 抽离方案 |
| v4.16.0 / BE-001DY-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离方案已建立，下一步进入 BE-001DY-03 实际抽离记录 |
| v4.16.0 / BE-001DY-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离记录已建立，下一步进入 BE-001DY-04 单叶 closeout |
| v4.16.0 / BE-001DY-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单叶 closeout 已建立，下一步进入 BE-001DZ-01 父叶残余判断 |
| v4.16.0 / BE-001DZ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断已建立，选择 transition_record_persistence import pass |
| v4.16.0 / BE-001EA-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EA-02 抽离方案 |
| v4.16.0 / BE-001EA-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案已建立，下一步进入 BE-001EA-03 实际抽离记录 |
| v4.16.0 / BE-001EA-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离记录已建立，下一步进入 BE-001EA-04 单叶 closeout |
| v4.16.0 / BE-001EA-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EB-01 父叶残余判断 |
| v4.16.0 / BE-001EB-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第三轮父叶残余判断已建立，下一步进入 BE-001EC-01 单子叶等价基线 |
| v4.16.0 / BE-001EC-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EC-02 抽离方案 |
| v4.16.0 / BE-001EC-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案已建立，下一步进入 BE-001EC-03 实际抽离记录 |
| v4.16.0 / BE-001EC-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离记录已建立，下一步进入 BE-001EC-04 单叶 closeout |
| v4.16.0 / BE-001EC-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单叶 closeout 已建立，下一步进入 BE-001ED-01 父叶残余判断 |
| v4.16.0 / BE-001ED-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断已建立，下一步进入 BE-001EE-01 单子叶等价基线 |
| v4.16.0 / BE-001EE-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EE-02 抽离方案 |
| v4.16.0 / BE-001EE-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案已建立，下一步进入 BE-001EE-03 实际抽离记录 |
| v4.16.0 / BE-001EE-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离记录已建立，下一步进入 BE-001EE-04 单叶 closeout |
| v4.16.0 / BE-001EE-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EF-01 父叶残余判断 |
| v4.16.0 / BE-001EF-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第五轮父叶残余判断已建立，下一步进入 BE-001EG-01 单子叶等价基线 |
| v4.16.0 / BE-001EG-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EG-02 抽离方案 |
| v4.16.0 / BE-001EG-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离方案已建立，下一步进入 BE-001EG-03 实际抽离记录 |
| v4.16.0 / BE-001EG-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离记录已建立，下一步进入 BE-001EG-04 单叶 closeout |
| v4.16.0 / BE-001EG-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EH-01 父叶残余判断 |
| v4.16.0 / BE-001EH-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断已建立，下一步进入 BE-001EI-01 单子叶等价基线 |
| v4.16.0 / BE-001EI-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EI-02 抽离方案 |
| v4.16.0 / BE-001EI-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案已建立，下一步进入 BE-001EI-03 实际抽离记录 |
| v4.16.0 / BE-001EI-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离记录已建立，下一步进入 BE-001EI-04 单叶 closeout |
| v4.16.0 / BE-001EI-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EJ-01 父叶残余判断 |
| v4.16.0 / BE-001EJ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第七轮父叶残余判断已建立，下一步进入 BE-001EK-01 父叶残余判断 |
| v4.16.0 / BE-001EK-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 第三轮父叶残余判断已建立，下一步进入 BE-001EL-01 单子叶等价基线 |
| v4.16.0 / BE-001EL-01 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EL-02 抽离方案 |
| v4.16.0 / BE-001EL-02 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案已建立，下一步进入 BE-001EL-03 实际抽离记录 |
| v4.16.0 / BE-001EL-03 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离记录已建立，下一步进入 BE-001EL-04 单叶 closeout |
| v4.16.0 / BE-001EL-04 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EM-01 父叶残余判断 |
| v4.16.0 / BE-001EM-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 第四轮父叶残余判断已建立，下一步进入 BE-001EN-01 父叶残余判断 |
| v4.16.0 / BE-001EN-01 | 新增: `runtime.mutation_import_pass` 第二轮父叶残余判断已建立，下一步进入 BE-001EO-01 单子叶等价基线 |
| v4.16.0 / BE-001EO-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EO-02 抽离方案 |
| v4.16.0 / BE-001EO-02 | 新增: `runtime.mutation.ai_proposal_import_pass` 抽离方案已建立，下一步进入 BE-001EP-01 单子叶等价基线 |
| v4.16.0 / BE-001EP-01 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EP-02 抽离方案 |
| v4.16.0 / BE-001EP-02 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案已建立，下一步进入 BE-001EP-03 实际抽离记录 |
| v4.16.0 / BE-001EP-03 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 抽离记录已建立，下一步进入 BE-001EP-04 单叶 closeout |
| v4.16.0 / BE-001EP-04 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EQ-01 父叶残余判断 |
| v4.16.0 / BE-001EQ-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第三轮父叶残余判断已建立，下一步进入 BE-001ER-01 单子叶等价基线 |
| v4.16.0 / BE-001ER-01 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线已建立，下一步进入 BE-001ER-02 抽离方案 |
| v4.16.0 / BE-001ER-02 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案已建立，下一步进入 BE-001ER-03 实际抽离记录 |
| v4.16.0 / BE-001ER-03 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离记录已建立，下一步进入 BE-001ER-04 单叶 closeout |
| v4.16.0 / BE-001ER-04 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout 已建立，下一步进入 BE-001ES-01 父叶残余判断 |
| v4.16.0 / BE-001ES-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断已建立，下一步进入 BE-001ET-01 static_check import pass 基线 |
| v4.16.0 / BE-001ET-01 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线已建立，下一步进入 BE-001ET-02 抽离方案 |
| v4.16.0 / BE-001ET-02 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案已建立，下一步进入 BE-001ET-03 实际抽离记录 |
| v4.16.0 / BE-001ET-03 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 抽离记录已建立，下一步进入 BE-001ET-04 单叶 closeout |
| v4.16.0 / BE-001ET-04 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EU-01 父叶残余判断 |
| v4.16.0 / BE-001EU-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断已建立，下一步进入 BE-001EV-01 event_lifecycle import pass 基线 |
| v4.16.0 / BE-001EV-01 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EV-02 抽离方案 |
| v4.16.0 / BE-001EV-02 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案已建立，下一步进入 BE-001EV-03 实际抽离记录 |
| v4.16.0 / BE-001EV-03 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离记录已建立，下一步进入 BE-001EV-04 单叶 closeout |
| v4.16.0 / BE-001EV-04 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EW-01 父叶残余判断 |
| v4.16.0 / BE-001EW-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断已建立，下一步进入 BE-001EX-01 approval_persistence import pass 基线 |
| v4.16.0 / BE-001EX-01 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EX-02 抽离方案 |
| v4.16.0 / BE-001EX-02 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案已建立，下一步进入 BE-001EX-03 实际抽离记录 |
| v4.16.0 / BE-001EX-03 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离记录已建立，下一步进入 BE-001EX-04 单叶 closeout |
| v4.16.0 / BE-001EX-04 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout 已建立，下一步进入 BE-001EY-01 父叶残余判断 |
| v4.16.0 / BE-001EY-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断已建立，下一步进入 BE-001EZ-01 status_transition import pass 基线 |
| v4.16.0 / BE-001EZ-01 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线已建立，下一步进入 BE-001EZ-02 抽离方案 |
| v4.16.0 / BE-001EZ-02 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案已建立，下一步进入 BE-001EZ-03 实际抽离记录 |
| v4.16.0 / BE-001EZ-03 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离记录已建立，下一步进入 BE-001EZ-04 单叶 closeout |
| v4.16.0 / BE-001EZ-04 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout 已建立，下一步进入 BE-001FA-01 父叶残余判断 |
| v4.16.0 / BE-001FA-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断已建立，下一步进入 BE-001FB-01 sandbox_trigger 等价基线 |
| v4.16.0 / BE-001FB-01 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线已建立，下一步进入 BE-001FB-02 抽离方案 |
| v4.16.0 / BE-001FB-02 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案已建立，下一步进入 BE-001FB-03 实际抽离记录 |
| v4.16.0 / BE-001FB-03 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离记录已建立，下一步进入 BE-001FB-04 单叶 closeout |
| v4.16.0 / BE-001FB-04 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout 已建立，下一步进入 BE-001FC-01 父叶残余判断 |
| v4.16.0 / BE-001FC-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断已建立，下一步进入 BE-001FD-01 approval_review 等价基线 |
| v4.16.0 / BE-001FD-01 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线已建立，下一步进入 BE-001FD-02 approval_review 抽离方案 |
| v4.16.0 / BE-001FD-02 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案已建立，限制下一步为单文件 import rewrite |
| v4.16.0 / BE-001FD-03 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离记录已建立，下一步进入单叶 closeout |
| v4.16.0 / BE-001FD-04 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout 已建立，下一步回到父叶残余判断 |
| v4.16.0 / BE-001FE-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断已建立，下一步进入 proposal_creation import pass 等价基线 |
| v4.16.0 / BE-001FF-01 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FF-02 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FF-03 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离记录已建立，下一步进入单叶 closeout |
| v4.16.0 / BE-001FF-04 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout 已建立，下一步回父叶残余判断 |
| v4.16.0 / BE-001FG-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十一轮父叶残余判断已建立，下一步进入 parent facade 等价基线 |
| v4.16.0 / BE-001FH-01 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FH-02 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FH-03 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离记录已建立，下一步进入单叶 closeout |
| v4.16.0 / BE-001FH-04 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout 已建立，下一步回父叶残余判断 |
| v4.16.0 / BE-001FI-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十二轮父叶残余判断已建立，下一步回上层父叶 |
| v4.16.0 / BE-001FJ-01 | 新增: `runtime.mutation_import_pass` 第三轮父叶残余判断已建立，下一步回 root parent bridge |
| v4.16.0 / BE-001FK-01 | 新增: `runtime.parent_import_bridge` 第四轮父叶残余判断已建立，下一步进入 root parent facade 基线 |
| v4.16.0 / BE-001FL-01 | 新增: `runtime.root_parent_facade_import_pass` 单子叶等价基线已建立，下一步进入 root import 抽离方案 |
| v4.16.0 / BE-001FL-02 | 新增: `runtime.root_parent_facade_import_pass` 抽离方案已建立，下一步进入单文件实际抽离 |
| v4.16.0 / BE-001FL-03 | 新增: `runtime.root_parent_facade_import_pass` 抽离记录已建立，下一步进入单叶 closeout |
| v4.16.0 / BE-001FL-04 | 新增: `runtime.root_parent_facade_import_pass` 单叶 closeout 已建立，下一步回到 parent bridge 父叶判断 |
| v4.16.0 / BE-001FM-01 | 新增: `runtime.parent_import_bridge` 第五轮父叶残余判断已建立，下一步进入 backend.runtime 父叶判断 |
| v4.16.0 / BE-001FN-01 | 新增: `backend.runtime` 第十轮父叶残余判断已建立，下一步进入 backend 父叶判断 |
| v4.16.0 / BE-001FO-01 | 新增: `backend` 父叶残余判断已建立，下一步进入 backend.graph_compile 父叶判断 |
| v4.16.0 / BE-001FP-01 | 新增: `backend.graph_compile` 父叶残余判断已建立，下一步进入 quantscript_graph 等价基线 |
| v4.16.0 / BE-001FQ-01 | 新增: `backend.graph_compile.quantscript_graph` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FQ-02 | 新增: `backend.graph_compile.quantscript_graph` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FQ-03 | 新增: `backend.graph_compile.quantscript_graph` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FQ-04 | 新增: `backend.graph_compile.quantscript_graph` 单叶 closeout 完成，下一步进入 `graph_to_qs_generation` 等价基线 |
| v4.16.0 / BE-001FR-01 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FR-02 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FR-03 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FR-04 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单叶 closeout 已完成，下一步回父叶残余判断 |
| v4.16.0 / BE-001FS-01 | 新增: `backend.graph_compile.quantscript_graph` 父叶残余判断已完成，下一步进入 `formal_module_conversion` 等价基线 |
| v4.16.0 / BE-001FT-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FT-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FT-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FT-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 单叶 closeout 完成，下一步进入父叶残余判断 |
| v4.16.0 / BE-001FU-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 父叶残余判断已完成，下一步进入 `intent_lowering` 等价基线 |
| v4.16.0 / BE-001FV-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FV-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FV-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FV-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单叶 closeout 完成，下一步进入父叶残余判断 |
| v4.16.0 / BE-001FW-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，下一步进入 `spread_observer_lowering` 等价基线 |
| v4.16.0 / BE-001FX-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FX-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FX-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FX-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单叶 closeout 完成，下一步回到父叶残余判断 |
| v4.16.0 / BE-001FY-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，下一步进入 `macd_lowering` 等价基线 |
| v4.16.0 / BE-001FZ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001FZ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001FZ-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001FZ-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单叶 closeout 完成，下一步回到父叶残余判断 |
| v4.16.0 / BE-001GA-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，下一步进入 `double_ma_lowering` 等价基线 |
| v4.16.0 / BE-001GB-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001GB-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001GB-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001GB-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单叶 closeout 完成，下一步回到父叶残余判断 |
| v4.16.0 / BE-001GC-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，下一步进入 `rsi_lowering` 等价基线 |
| v4.16.0 / BE-001GD-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单子叶等价基线已建立，下一步进入抽离方案 |
| v4.16.0 / BE-001GD-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 抽离方案已建立，下一步进入实际抽离记录 |
| v4.16.0 / BE-001GD-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 实际抽离完成，下一步进入单叶 closeout |
| v4.16.0 / BE-001GD-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单叶 closeout 完成，下一步回到父叶残余判断 |
| v4.16.0 / GOV-LEAF-SPLIT-GATE | 新增: 递归叶子细分判定硬规则固化；后续单叶 closeout / 父叶残余判断必须包含 `leaf_split_decision_gate` 判定 |
| v4.16.0 / BE-001GE-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects ma_deviation_lowering |
| v4.16.0 / BE-001GF-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GF-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GG-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects momentum_lowering |
| v4.16.0 / BE-001GH-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GH-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GI-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects zscore_lowering |
| v4.16.0 / BE-001GJ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GJ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GK-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects shared_intent_context |
| v4.16.0 / BE-001GL-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context baseline and extraction plan frozen |
| v4.16.0 / BE-001GL-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context actual extraction and closeout complete |
| v4.16.0 / BE-001GM-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects unsupported_intent_failure |
| v4.16.0 / BE-001GN-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure equivalence baseline and extraction plan |
| v4.16.0 / BE-001GN-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure actual extraction and closeout complete |
| v4.16.0 / BE-001GO-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent closeout sets stop_split true |
| v4.16.0 / BE-001GP-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects data_source_lowering |
| v4.16.0 / BE-001GQ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering equivalence baseline and extraction plan |
| v4.16.0 / BE-001GQ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GR-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects profile_lowering |
| v4.16.0 / BE-001GS-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering equivalence baseline and extraction plan |
| v4.16.0 / BE-001GS-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GT-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects input_shape_validation |
| v4.16.0 / BE-001GU-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001GU-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation actual extraction and closeout complete |
| v4.16.0 / BE-001GV-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects terminal_parse |
| v4.16.0 / BE-001GW-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse equivalence baseline and extraction plan |
| v4.16.0 / BE-001GW-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse actual extraction and closeout complete |
| v4.16.0 / BE-001GX-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects unsupported_node_logging |
| v4.16.0 / BE-001GY-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging equivalence baseline and extraction plan |
| v4.16.0 / BE-001GY-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging actual extraction and closeout complete |
| v4.16.0 / BE-001GZ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent closeout sets stop_split true |
| v4.16.0 / BE-001HA-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects strategy_graph_parser |
| v4.16.0 / BE-001HB-01 | 新增: `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser equivalence baseline and extraction plan |
| v4.16.0 / BE-001HB-02 | 新增: `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser actual extraction and closeout complete |
| v4.16.0 / BE-001HC-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects artifact_target_projection |
| v4.16.0 / BE-001HD-01 | 新增: `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001HD-02 | 新增: `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection actual extraction and closeout complete |
| v4.16.0 / BE-001HE-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects route_surface |
| v4.16.0 / BE-001HF-01 | 新增: `backend.graph_compile.quantscript_graph.route_surface` route_surface equivalence baseline and extraction plan |
| v4.16.0 / BE-001HF-02 | 新增: `backend.graph_compile.quantscript_graph.route_surface` route_surface actual extraction and closeout complete |
| v4.16.0 / BE-001HG-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent closeout sets stop_split true |
| v4.16.0 / BE-001HH-01 | 新增: `backend.graph_compile` backend.graph_compile parent residual judgment selects compile |
| v4.16.0 / BE-001HI-01 | 新增: `backend.graph_compile.compile` backend.graph_compile.compile equivalence baseline and extraction plan |
| v4.16.0 / BE-001HI-02 | 新增: `backend.graph_compile.compile` backend.graph_compile.compile actual extraction and closeout complete |
| v4.16.0 / BE-001HJ-01 | 新增: `backend.graph_compile` backend.graph_compile parent residual judgment selects graph |
| v4.16.0 / BE-001HK-01 | 新增: `backend.graph_compile.graph` backend.graph_compile.graph equivalence baseline and extraction plan |
| v4.16.0 / BE-001HK-02 | 新增: `backend.graph_compile.graph` backend.graph_compile.graph actual extraction and closeout complete |
| v4.16.0 / BE-001HL-01 | 新增: `backend.graph_compile` backend.graph_compile parent closeout sets stop_split true |
| v4.16.0 / BE-001HM-01 | 新增: `backend` backend parent residual judgment selects capability |
| v4.16.0 / BE-001HN-01 | 新增: `backend.capability` backend.capability equivalence baseline and extraction plan |
| v4.16.0 / BE-001HN-02 | 新增: `backend.capability` backend.capability actual extraction and closeout complete |
| v4.16.0 / BE-001HO-01 | 新增: `backend` backend parent residual judgment selects strategy_config |
| v4.16.0 / BE-001HP-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects artifact |
| v4.16.0 / BE-001HQ-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact equivalence baseline and extraction plan |
| v4.16.0 / BE-001HQ-02 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact route owner extraction complete |
| v4.16.0 / BE-001HR-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects schema_model |
| v4.16.0 / BE-001HS-01 | 新增: `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan |
| v4.16.0 / BE-001HS-02 | 新增: `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model actual extraction complete |
| v4.16.0 / BE-001HT-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects domain_projection |
| v4.16.0 / BE-001HU-01 | 新增: `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001HU-02 | 新增: `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection actual extraction complete |
| v4.16.0 / BE-001HV-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects builder_core |
| v4.16.0 / BE-001HW-01 | 新增: `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan |
| v4.16.0 / BE-001HW-02 | 新增: `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core actual extraction complete |
| v4.16.0 / BE-001HX-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent closeout sets stop_split true |
| v4.16.0 / BE-001HY-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects preflight |
| v4.16.0 / BE-001HZ-01 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight equivalence baseline and extraction plan |
| v4.16.0 / BE-001HZ-02 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight actual extraction complete |
| v4.16.0 / BE-001IA-01 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight single leaf closeout sets stop_split true |
| v4.16.0 / BE-001IB-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects diff |
| v4.16.0 / BE-001IC-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001IC-02 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff actual extraction complete |
| v4.16.0 / BE-001ID-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001IE-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects artifact_diff |
| v4.16.0 / BE-001IF-01 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001IF-02 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff actual extraction complete |
| v4.16.0 / BE-001IG-01 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff single leaf closeout sets stop_split true |
| v4.16.0 / BE-001IH-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects evidence_diff |
| v4.16.0 / BE-001II-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001II-02 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff actual extraction complete |
| v4.16.0 / BE-001IJ-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001IK-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory |
| v4.16.0 / BE-001IL-01 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan |
| v4.16.0 / BE-001IL-02 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete |
| v4.16.0 / BE-001IM-01 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory single leaf closeout stops further split |
| v4.16.0 / BE-001IN-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane |
| v4.16.0 / BE-001IO-01 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan |
| v4.16.0 / BE-001IO-02 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane actual extraction complete |
| v4.16.0 / BE-001IP-01 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane single leaf closeout stops further split |
| v4.16.0 / BE-001IQ-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability |
| v4.16.0 / BE-001IR-01 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan |
| v4.16.0 / BE-001IR-02 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability actual extraction complete |
| v4.16.0 / BE-001IS-01 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability single leaf closeout stops further split |
| v4.16.0 / BE-001IT-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics |
| v4.16.0 / BE-001IU-01 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan |
| v4.16.0 / BE-001IU-02 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics actual extraction complete |
| v4.16.0 / BE-001IV-01 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics single leaf closeout stops further split |
| v4.16.0 / BE-001IW-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent closeout retains report assembly and shared helpers |
| v4.16.0 / BE-001IX-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent closeout keeps facade and child mediation |
| v4.16.0 / BE-001IY-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects ai_proposal_binding |
| v4.16.0 / BE-001IZ-01 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-op route pocket baseline and plan |
| v4.16.0 / BE-001IZ-02 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-code extraction closeout complete |
| v4.16.0 / BE-001JA-01 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding single leaf closeout stops further split |
| v4.16.0 / BE-001JB-01 | 新增: `backend.strategy_config` backend.strategy_config parent closeout keeps route aggregation facade |
| v4.16.0 / BE-001JC-01 | 新增: `backend` backend parent residual judgment selects storage_security safety baseline |
| v4.16.0 / BE-001JD-01 | 新增: `backend.storage_security` backend.storage_security safety equivalence baseline and extraction plan |
| v4.16.0 / BE-001JD-02 | 新增: `backend.storage_security` backend.storage_security facade extraction closeout keeps sensitive semantics paused |
| v4.16.0 / BE-001JE-01 | 新增: `backend.storage_security` backend.storage_security single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JF-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_api |
