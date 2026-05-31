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
