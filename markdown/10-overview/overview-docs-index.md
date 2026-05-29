# QuantPilot 文档索引

> v4.7.0 代码基线 / v4.15.0 三矩阵完全接管 / v4.16.0 模块化抽离规划 | 最后更新 2026-05-28

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
