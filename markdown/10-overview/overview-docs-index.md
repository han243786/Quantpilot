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
| `recursive-speed-protocol.md` | v4.16+ 递归模块化高速执行协议，含同父级子叶并行 wave |
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
| v4.16.0 / BE-001JG-01 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api route facade baseline and plan |
| v4.16.0 / BE-001JG-02 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api facade extraction closeout complete |
| v4.16.0 / BE-001JH-01 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api single leaf closeout stops further facade split |
| v4.16.0 / BE-001JI-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault |
| v4.16.0 / BE-001JJ-01 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault re-export facade baseline and plan |
| v4.16.0 / BE-001JJ-02 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault facade extraction closeout complete |
| v4.16.0 / BE-001JK-01 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault single leaf closeout stops further facade split |
| v4.16.0 / BE-001JL-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault_implementation |
| v4.16.0 / BE-001JM-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation safety baseline and extraction plan |
| v4.16.0 / BE-001JM-02 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation actual extraction complete |
| v4.16.0 / BE-001JN-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JO-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management |
| v4.16.0 / BE-001JP-01 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan |
| v4.16.0 / BE-001JP-02 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management actual extraction complete |
| v4.16.0 / BE-001JP-03 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management single leaf closeout stops further split |
| v4.16.0 / BE-001JQ-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec |
| v4.16.0 / BE-001JR-01 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan |
| v4.16.0 / BE-001JR-02 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec actual extraction complete |
| v4.16.0 / BE-001JR-03 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec single leaf closeout stops further split |
| v4.16.0 / BE-001JS-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects vault_persistence_restore |
| v4.16.0 / BE-001JT-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan |
| v4.16.0 / BE-001JT-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore actual extraction complete |
| v4.16.0 / BE-001JT-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JU-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects load_restore_entry |
| v4.16.0 / BE-001JV-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan |
| v4.16.0 / BE-001JV-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry actual extraction complete |
| v4.16.0 / BE-001JV-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single leaf closeout stops further split |
| v4.16.0 / BE-001JW-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects atomic_save_commit |
| v4.16.0 / BE-001JX-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001JX-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit actual extraction complete |
| v4.16.0 / BE-001JX-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single leaf closeout stops further split |
| v4.16.0 / BE-001JY-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent closeout stops persistence split |
| v4.16.0 / BE-001JZ-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects service_crud |
| v4.16.0 / BE-001KA-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan |
| v4.16.0 / BE-001KA-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud actual extraction complete |
| v4.16.0 / BE-001KA-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001KB-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_mutation_commit |
| v4.16.0 / BE-001KC-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001KC-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit actual extraction complete |
| v4.16.0 / BE-001KC-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single leaf closeout stops further split |
| v4.16.0 / BE-001KD-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection |
| v4.16.0 / BE-001KE-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001KE-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection actual extraction complete |
| v4.16.0 / BE-001KE-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split |
| v4.16.0 / BE-001KF-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent closeout stops CRUD split |
| v4.16.0 / BE-001KG-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects secret_pattern_extraction |
| v4.16.0 / BE-001KH-01 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan |
| v4.16.0 / BE-001KH-02 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction actual extraction complete |
| v4.16.0 / BE-001KH-03 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction single leaf closeout stops further split |
| v4.16.0 / BE-001KI-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects type_surface |
| v4.16.0 / BE-001KJ-01 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan |
| v4.16.0 / BE-001KJ-02 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface actual extraction complete |
| v4.16.0 / BE-001KJ-03 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface single leaf closeout stops further split |
| v4.16.0 / BE-001KK-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects implementation_test_harness |
| v4.16.0 / BE-001KL-01 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan |
| v4.16.0 / BE-001KL-02 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness actual extraction complete |
| v4.16.0 / BE-001KL-03 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness single leaf closeout stops further split |
| v4.16.0 / BE-001KM-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment closes implementation parent |
| v4.16.0 / BE-001KN-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_api_handler_implementation |
| v4.16.0 / BE-001KO-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan |
| v4.16.0 / BE-001KO-02 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation actual extraction complete |
| v4.16.0 / BE-001KO-03 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation single leaf closeout continues split |
| v4.16.0 / BE-001KP-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection |
| v4.16.0 / BE-001KQ-01 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001KQ-02 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete |
| v4.16.0 / BE-001KQ-03 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection single leaf closeout stops further split |
| v4.16.0 / BE-001KR-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope |
| v4.16.0 / BE-001KS-01 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan |
| v4.16.0 / BE-001KS-02 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete |
| v4.16.0 / BE-001KS-03 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope single leaf closeout stops further split |
| v4.16.0 / BE-001KT-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation |
| v4.16.0 / BE-001KU-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan |
| v4.16.0 / BE-001KU-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete |
| v4.16.0 / BE-001KU-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation single leaf closeout continues split |
| v4.16.0 / BE-001KV-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation |
| v4.16.0 / BE-001KW-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001KW-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete |
| v4.16.0 / BE-001KW-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single leaf closeout stops further split |
| v4.16.0 / BE-001KX-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects storage_commit |
| v4.16.0 / BE-001KY-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001KY-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete |
| v4.16.0 / BE-001KY-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single leaf closeout stops further split |
| v4.16.0 / BE-001KZ-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment closes parent |
| v4.16.0 / BE-001LA-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation |
| v4.16.0 / BE-001LB-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan |
| v4.16.0 / BE-001LB-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete |
| v4.16.0 / BE-001LB-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation single leaf closeout continues split |
| v4.16.0 / BE-001LC-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation |
| v4.16.0 / BE-001LD-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001LD-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete |
| v4.16.0 / BE-001LD-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single leaf closeout stops further split |
| v4.16.0 / BE-001LE-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects delete_commit |
| v4.16.0 / BE-001LF-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001LF-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete |
| v4.16.0 / BE-001LF-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single leaf closeout stops further split |
| v4.16.0 / BE-001LG-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment closes parent |
| v4.16.0 / BE-001LH-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment closes parent |
| v4.16.0 / BE-001LI-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment closes parent |
| v4.16.0 / BE-001LJ-01 | 新增: `backend` backend parent residual judgment selects ops_governance |
| v4.16.0 / BE-001LK-01 | 新增: `backend.ops_governance` backend.ops_governance equivalence baseline and extraction plan |
| v4.16.0 / BE-001LK-02 | 新增: `backend.ops_governance` backend.ops_governance facade extraction closeout |
| v4.16.0 / BE-001LK-03 | 新增: `backend.ops_governance` backend.ops_governance single leaf closeout continues split |
| v4.16.0 / BE-001LL-01 | 新增: `backend.ops_governance` backend.ops_governance parent residual judgment selects hotswap |
| v4.16.0 / BE-001LM-01 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap equivalence baseline and extraction plan |
| v4.16.0 / BE-001LM-02 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap actual extraction complete |
| v4.16.0 / BE-001LM-03 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap single leaf closeout stops further split |
| v4.16.0 / BE-001LN-01 | 新增: `backend.ops_governance` backend.ops_governance parent residual judgment selects sandbox |
| v4.16.0 / BE-001LO-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox equivalence baseline and extraction plan |
| v4.16.0 / BE-001LO-02 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox actual extraction complete |
| v4.16.0 / BE-001LO-03 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox single leaf closeout continues split |
| v4.16.0 / BE-001LP-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects report_api |
| v4.16.0 / BE-001LQ-01 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan |
| v4.16.0 / BE-001LQ-02 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api actual extraction complete |
| v4.16.0 / BE-001LQ-03 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api single leaf closeout stops further split |
| v4.16.0 / BE-001LR-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects verification_run |
| v4.16.0 / BE-001LS-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan |
| v4.16.0 / BE-001LS-02 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run actual extraction complete |
| v4.16.0 / BE-001LS-03 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run single leaf closeout continues split |
| v4.16.0 / BE-001LT-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit |
| v4.16.0 / BE-001LU-01 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001LU-02 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete |
| v4.16.0 / BE-001LU-03 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit single leaf closeout stops further split |
| v4.16.0 / BE-001LV-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate |
| v4.16.0 / BE-001LW-01 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan |
| v4.16.0 / BE-001LW-02 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete |
| v4.16.0 / BE-001LW-03 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate single leaf closeout stops further split |
| v4.16.0 / BE-001LX-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window |
| v4.16.0 / BE-001LY-01 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan |
| v4.16.0 / BE-001LY-02 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete |
| v4.16.0 / BE-001LY-03 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout stops further split |
| v4.16.0 / BE-001LZ-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly |
| v4.16.0 / BE-001MA-01 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan |
| v4.16.0 / BE-001MA-02 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete |
| v4.16.0 / BE-001MA-03 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout stops further split |
| v4.16.0 / BE-001MB-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment closes parent |
| v4.16.0 / BE-001MC-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation |
| v4.16.0 / BE-001MD-01 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan |
| v4.16.0 / BE-001MD-02 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation actual extraction complete |
| v4.16.0 / BE-001MD-03 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation single leaf closeout stops further split |
| v4.16.0 / BE-001ME-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects comparison_metrics |
| v4.16.0 / BE-001MF-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan |
| v4.16.0 / BE-001MF-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics actual extraction complete |
| v4.16.0 / BE-001MF-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split |
| v4.16.0 / BE-001MG-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape |
| v4.16.0 / BE-001MH-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan |
| v4.16.0 / BE-001MH-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete |
| v4.16.0 / BE-001MH-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout stops further split |
| v4.16.0 / BE-001MI-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection |
| v4.16.0 / BE-001MJ-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001MJ-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete |
| v4.16.0 / BE-001MJ-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout stops further split |
| v4.16.0 / BE-001MK-01 | Added: `backend.ops_governance.sandbox.comparison_metrics` parent residual judgment closes parent |
| v4.16.0 / BE-001ML-01 | Added: `backend.ops_governance.sandbox` parent residual judgment selects proposal_loader |
| v4.16.0 / BE-001MM-01 | Added: `backend.ops_governance.sandbox.proposal_loader` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MM-02 | Added: `backend.ops_governance.sandbox.proposal_loader` actual extraction complete |
| v4.16.0 / BE-001MM-03 | Added: `backend.ops_governance.sandbox.proposal_loader` single leaf closeout stops further split |
| v4.16.0 / BE-001MN-01 | Added: `backend.ops_governance.sandbox` parent residual judgment selects report_disk_loader |
| v4.16.0 / BE-001MO-01 | Added: `backend.ops_governance.sandbox.report_disk_loader` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MO-02 | Added: `backend.ops_governance.sandbox.report_disk_loader` actual extraction complete |
| v4.16.0 / BE-001MO-03 | Added: `backend.ops_governance.sandbox.report_disk_loader` single leaf closeout stops further split |
| v4.16.0 / BE-001MP-01 | Added: `backend.ops_governance.sandbox` parent residual judgment closes parent |
| v4.16.0 / BE-001MQ-01 | Added: `backend.ops_governance` parent residual judgment selects alerts |
| v4.16.0 / BE-001MR-01 | Added: `backend.ops_governance.alerts` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MR-02 | Added: `backend.ops_governance.alerts` actual extraction complete |
| v4.16.0 / BE-001MR-03 | Added: `backend.ops_governance.alerts` single leaf closeout continues split |
| v4.16.0 / BE-001MS-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects rule_catalog |
| v4.16.0 / BE-001MT-01 | Added: `backend.ops_governance.alerts.rule_catalog` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MT-02 | Added: `backend.ops_governance.alerts.rule_catalog` actual extraction complete |
| v4.16.0 / BE-001MT-03 | Added: `backend.ops_governance.alerts.rule_catalog` single leaf closeout stops further split |
| v4.16.0 / BE-001MU-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects acknowledge_flow |
| v4.16.0 / BE-001MV-01 | Added: `backend.ops_governance.alerts.acknowledge_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MV-02 | Added: `backend.ops_governance.alerts.acknowledge_flow` actual extraction complete |
| v4.16.0 / BE-001MV-03 | Added: `backend.ops_governance.alerts.acknowledge_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001MW-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects trigger_engine |
| v4.16.0 / BE-001MX-01 | Added: `backend.ops_governance.alerts.trigger_engine` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MX-02 | Added: `backend.ops_governance.alerts.trigger_engine` actual extraction complete |
| v4.16.0 / BE-001MX-03 | Added: `backend.ops_governance.alerts.trigger_engine` single leaf closeout stops further split |
| v4.16.0 / BE-001MY-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects predicate_checks |
| v4.16.0 / BE-001MZ-01 | Added: `backend.ops_governance.alerts.predicate_checks` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MZ-02 | Added: `backend.ops_governance.alerts.predicate_checks` actual extraction complete |
| v4.16.0 / BE-001MZ-03 | Added: `backend.ops_governance.alerts.predicate_checks` single leaf closeout stops further split |
| v4.16.0 / BE-001NA-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects persistence |
| v4.16.0 / BE-001NA-02 | Added: `backend.ops_governance.alerts.persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NA-03 | Added: `backend.ops_governance.alerts.persistence` actual extraction complete |
| v4.16.0 / BE-001NA-04 | Added: `backend.ops_governance.alerts.persistence` single leaf closeout stops further split |
| v4.16.0 / BE-001NB-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects startup_initialization |
| v4.16.0 / BE-001NB-02 | Added: `backend.ops_governance.alerts.startup_initialization` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NB-03 | Added: `backend.ops_governance.alerts.startup_initialization` actual extraction complete |
| v4.16.0 / BE-001NB-04 | Added: `backend.ops_governance.alerts.startup_initialization` single leaf closeout stops further split |
| v4.16.0 / BE-001NC-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects read_routes |
| v4.16.0 / BE-001NC-02 | Added: `backend.ops_governance.alerts.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NC-03 | Added: `backend.ops_governance.alerts.read_routes` actual extraction complete |
| v4.16.0 / BE-001NC-04 | Added: `backend.ops_governance.alerts.read_routes` single leaf closeout stops further split |
| v4.16.0 / BE-001ND-01 | Added: `backend.ops_governance.alerts.route_facade` static closeout and recovery_bridge selection |
| v4.16.0 / BE-001NE-01 | Added: `backend.ops_governance.alerts.recovery_bridge` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NE-02 | Added: `backend.ops_governance.alerts.recovery_bridge` actual extraction complete |
| v4.16.0 / BE-001NE-03 | Added: `backend.ops_governance.alerts.recovery_bridge` single leaf closeout stops further split |
| v4.16.0 / BE-001NF-01 | Added: `backend.ops_governance.alerts` parent residual judgment closes parent |
| v4.16.0 / BE-001NG-01 | Added: `backend.ops_governance` parent residual judgment selects snapshots |
| v4.16.0 / BE-001NH-01 | Added: `backend.ops_governance.snapshots` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NH-02 | Added: `backend.ops_governance.snapshots` actual extraction complete |
| v4.16.0 / BE-001NH-03 | Added: `backend.ops_governance.snapshots` single leaf closeout continues split |
| v4.16.0 / BE-001NI-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects snapshot_id_validation |
| v4.16.0 / BE-001NJ-01 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NJ-02 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` actual extraction complete |
| v4.16.0 / BE-001NJ-03 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` single leaf closeout stops further split |
| v4.16.0 / BE-001NK-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects create_flow |
| v4.16.0 / BE-001NL-01 | Added: `backend.ops_governance.snapshots.create_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NL-02 | Added: `backend.ops_governance.snapshots.create_flow` actual extraction complete |
| v4.16.0 / BE-001NL-03 | Added: `backend.ops_governance.snapshots.create_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001NM-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects read_routes |
| v4.16.0 / BE-001NN-01 | Added: `backend.ops_governance.snapshots.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NN-02 | Added: `backend.ops_governance.snapshots.read_routes` actual extraction complete |
| v4.16.0 / BE-001NN-03 | Added: `backend.ops_governance.snapshots.read_routes` single leaf closeout stops further split |
| v4.16.0 / BE-001NO-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects restore_flow |
| v4.16.0 / BE-001NP-01 | Added: `backend.ops_governance.snapshots.restore_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NP-02 | Added: `backend.ops_governance.snapshots.restore_flow` actual extraction complete |
| v4.16.0 / BE-001NP-03 | Added: `backend.ops_governance.snapshots.restore_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001NQ-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects persistence |
| v4.16.0 / BE-001NR-01 | Added: `backend.ops_governance.snapshots.persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NR-02 | Added: `backend.ops_governance.snapshots.persistence` actual extraction complete |
| v4.16.0 / BE-001NR-03 | Added: `backend.ops_governance.snapshots.persistence` single leaf closeout stops further split |
| v4.16.0 / BE-001NS-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects signature_contract |
| v4.16.0 / BE-001NT-01 | Added: `backend.ops_governance.snapshots.signature_contract` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NT-02 | Added: `backend.ops_governance.snapshots.signature_contract` actual extraction complete |
| v4.16.0 / BE-001NT-03 | Added: `backend.ops_governance.snapshots.signature_contract` single leaf closeout stops further split |
| v4.16.0 / BE-001NU-01 | Added: `backend.ops_governance.snapshots.route_facade` static closeout and parent closeout selection |
| v4.16.0 / BE-001NV-01 | Added: `backend.ops_governance.snapshots` parent residual judgment closes parent |
| v4.16.0 / BE-001NW-01 | Added: `backend.ops_governance` parent residual judgment selects runbook |
| v4.16.0 / BE-001NX-01 | Added: `backend.ops_governance.runbook` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NX-02 | Added: `backend.ops_governance.runbook` actual extraction complete |
| v4.16.0 / BE-001NX-03 | Added: `backend.ops_governance.runbook` single leaf closeout continues split |
| v4.16.0 / BE-001NY-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects scenario_catalog |
| v4.16.0 / BE-001NZ-01 | Added: `backend.ops_governance.runbook.scenario_catalog` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NZ-02 | Added: `backend.ops_governance.runbook.scenario_catalog` actual extraction complete |
| v4.16.0 / BE-001NZ-03 | Added: `backend.ops_governance.runbook.scenario_catalog` single leaf closeout |
| v4.16.0 / BE-001OA-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects read_routes |
| v4.16.0 / BE-001OB-01 | Added: `backend.ops_governance.runbook.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OB-02 | Added: `backend.ops_governance.runbook.read_routes` actual extraction complete |
| v4.16.0 / BE-001OB-03 | Added: `backend.ops_governance.runbook.read_routes` single leaf closeout |
| v4.16.0 / BE-001OC-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects route_facade |
| v4.16.0 / BE-001OD-01 | Added: `backend.ops_governance.runbook.route_facade` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OD-02 | Added: `backend.ops_governance.runbook.route_facade` actual extraction complete |
| v4.16.0 / BE-001OD-03 | Added: `backend.ops_governance.runbook.route_facade` single leaf closeout |
| v4.16.0 / BE-001OE-01 | Added: `backend.ops_governance.runbook` parent closeout |
| v4.16.0 / BE-001OF-01 | Added: `backend.ops_governance` parent residual judgment selects chaos |
| v4.16.0 / BE-001OG-01 | Added: `backend.ops_governance.chaos` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OG-02 | Added: `backend.ops_governance.chaos` actual extraction complete |
| v4.16.0 / BE-001OG-03 | Added: `backend.ops_governance.chaos` single leaf closeout continues split |
| v4.16.0 / BE-001OH-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects report_persistence |
| v4.16.0 / BE-001OI-01 | Added: `backend.ops_governance.chaos.report_persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OI-02 | Added: `backend.ops_governance.chaos.report_persistence` actual extraction complete |
| v4.16.0 / BE-001OI-03 | Added: `backend.ops_governance.chaos.report_persistence` single leaf closeout |
| v4.16.0 / BE-001OJ-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects experiment_creation |
| v4.16.0 / BE-001OK-01 | Added: `backend.ops_governance.chaos.experiment_creation` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OK-02 | Added: `backend.ops_governance.chaos.experiment_creation` actual extraction complete |
| v4.16.0 / BE-001OK-03 | Added: `backend.ops_governance.chaos.experiment_creation` single leaf closeout continues split |
| v4.16.0 / BE-001OL-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects perturbation_execution |
| v4.16.0 / BE-001OM-01 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OM-02 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` actual extraction complete |
| v4.16.0 / BE-001OM-03 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` single leaf closeout |
| v4.16.0 / BE-001ON-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects report_projection |
| v4.16.0 / BE-001OO-01 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OO-02 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` actual extraction complete |
| v4.16.0 / BE-001OO-03 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` single leaf closeout |
| v4.16.0 / BE-001OP-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects memory_commit |
| v4.16.0 / BE-001OQ-01 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OQ-02 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` actual extraction complete |
| v4.16.0 / BE-001OQ-03 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` single leaf closeout |
| v4.16.0 / BE-001OR-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent closeout |
| v4.16.0 / BE-001OS-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects read_routes |
| v4.16.0 / BE-001OT-01 | Added: `backend.ops_governance.chaos.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OT-02 | Added: `backend.ops_governance.chaos.read_routes` actual extraction complete |
| v4.16.0 / BE-001OT-03 | Added: `backend.ops_governance.chaos.read_routes` single leaf closeout |
| v4.16.0 / BE-001OU-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects route_facade |
| v4.16.0 / BE-001OV-01 | Added: `backend.ops_governance.chaos.route_facade` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OV-02 | Added: `backend.ops_governance.chaos.route_facade` actual extraction complete |
| v4.16.0 / BE-001OV-03 | Added: `backend.ops_governance.chaos.route_facade` single leaf closeout |
| v4.16.0 / BE-001OW-01 | Added: `backend.ops_governance.chaos` parent closeout |
| v4.16.0 / BE-001OX-01 | Added: `backend.ops_governance` parent closeout |
| v4.16.0 / BE-001OY-01 | Added: `backend` parent residual judgment selects `backend.app_state_wiring` |
| v4.16.0 / BE-001OZ-01 | Added: `backend.app_state_wiring` single leaf closeout |
| v4.16.0 / BE-001PA-01 | Added: `backend` parent residual judgment selects `backend.test_support` |
| v4.16.0 / BE-001PB-01 | Added: `backend.test_support` single leaf closeout |
| v4.16.0 / BE-001PC-01 | Added: `backend` parent closeout |
| v4.16.0 / BE-001PD-01 | Added: `root` parent residual judgment selects `root.contracts` |
| v4.16.0 / BE-001PE-01 | Added: `root.contracts` baseline plan |
| v4.16.0 / BE-001PF-01 | Added: `root.contracts` parent residual judgment selects `contracts.api_surface` |
| v4.16.0 / BE-001PG-01 | Added: `root.contracts.api_surface` single leaf closeout |
| v4.16.0 / BE-001PH-01 | Added: `root.contracts.api_surface` parent residual judgment selects `contracts.api_surface.openapi_http` |
| v4.16.0 / BE-001PI-01 | Added: `root.contracts.api_surface.openapi_http` single leaf closeout |
| v4.16.0 / BE-001PJ-01 | Added: `root.contracts.api_surface` parent residual judgment selects `contracts.api_surface.asyncapi_runtime_events` |
| v4.16.0 / BE-001PK-01 | Added: `root.contracts.api_surface.asyncapi_runtime_events` single leaf closeout |
| v4.16.0 / BE-001PL-01 | Added: `root.contracts.api_surface` parent closeout |
| v4.16.0 / BE-001PM-01 | Added: `root.contracts` parent residual judgment selects `contracts.qrpc_core` |
| v4.16.0 / BE-001PN-01 | Added: `root.contracts.qrpc_core` baseline plan |
| v4.16.0 / BE-001PO-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.error_contract` |
| v4.16.0 / BE-001PP-01 | Added: `root.contracts.qrpc_core.error_contract` single leaf closeout |
| v4.16.0 / BE-001PQ-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.event_envelope_proto` |
| v4.16.0 / BE-001PR-01 | Added: `root.contracts.qrpc_core.event_envelope_proto` single leaf closeout |
| v4.16.0 / BE-001PS-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.plugin_contract` |
| v4.16.0 / BE-001PT-01 | Added: `root.contracts.qrpc_core.plugin_contract` baseline plan |
| v4.16.0 / BE-001PU-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `taxonomy_extension` |
| v4.16.0 / BE-001PV-01 | Added: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` baseline plan |
| v4.16.0 / BE-001PV-02 | Added: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` actual extraction closeout |
| v4.16.0 / BE-001PV-03 | Added: `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` single leaf closeout |
| v4.16.0 / BE-001PW-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `capability_contract` |
| v4.16.0 / BE-001PX-01 | Added: `root.contracts.qrpc_core.plugin_contract.capability_contract` baseline plan |
| v4.16.0 / BE-001PX-02 | Added: `root.contracts.qrpc_core.plugin_contract.capability_contract` actual extraction closeout |
| v4.16.0 / BE-001PX-03 | Added: `root.contracts.qrpc_core.plugin_contract.capability_contract` single leaf closeout |
| v4.16.0 / BE-001PY-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `execution_security_dependency` |
| v4.16.0 / BE-001PZ-01 | Added: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` baseline plan |
| v4.16.0 / BE-001PZ-02 | Added: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` actual extraction closeout |
| v4.16.0 / BE-001PZ-03 | Added: `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` single leaf closeout |
| v4.16.0 / BE-001QA-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `manifest_validation` |
| v4.16.0 / BE-001QB-01 | Added: `root.contracts.qrpc_core.plugin_contract.manifest_validation` baseline plan |
| v4.16.0 / BE-001QB-02 | Added: `root.contracts.qrpc_core.plugin_contract.manifest_validation` actual extraction closeout |
| v4.16.0 / BE-001QB-03 | Added: `root.contracts.qrpc_core.plugin_contract.manifest_validation` single leaf closeout |
| v4.16.0 / BE-001QC-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `registry` |
| v4.16.0 / BE-001QD-01 | Added: `root.contracts.qrpc_core.plugin_contract.registry` baseline plan |
| v4.16.0 / BE-001QD-02 | Added: `root.contracts.qrpc_core.plugin_contract.registry` actual extraction closeout |
| v4.16.0 / BE-001QD-03 | Added: `root.contracts.qrpc_core.plugin_contract.registry` single leaf closeout |
| v4.16.0 / BE-001QE-01 | Added: `root.contracts.qrpc_core.plugin_contract` parent closeout |
| v4.16.0 / BE-001QF-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `strategy_ir` |
| v4.16.0 / BE-001QG-01 | Added: `root.contracts.qrpc_core.strategy_ir` baseline plan |
| v4.16.0 / BE-001QH-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `version_unknown_error` |
| v4.16.0 / BE-001QI-01 | Added: `root.contracts.qrpc_core.strategy_ir.version_unknown_error` baseline plan |
| v4.16.0 / BE-001QI-02 | Added: `root.contracts.qrpc_core.strategy_ir.version_unknown_error` actual extraction closeout |
| v4.16.0 / BE-001QI-03 | Added: `root.contracts.qrpc_core.strategy_ir.version_unknown_error` single leaf closeout |
| v4.16.0 / BE-001QJ-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `metadata_source` |
| v4.16.0 / BE-001QK-01 | Added: `root.contracts.qrpc_core.strategy_ir.metadata_source` baseline plan |
| v4.16.0 / BE-001QK-02 | Added: `root.contracts.qrpc_core.strategy_ir.metadata_source` actual extraction closeout |
| v4.16.0 / BE-001QK-03 | Added: `root.contracts.qrpc_core.strategy_ir.metadata_source` single leaf closeout |
| v4.16.0 / BE-001QL-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `signal_indicator` |
| v4.16.0 / BE-001QM-01 | Added: `root.contracts.qrpc_core.strategy_ir.signal_indicator` baseline plan |
| v4.16.0 / BE-001QM-02 | Added: `root.contracts.qrpc_core.strategy_ir.signal_indicator` actual extraction closeout |
| v4.16.0 / BE-001QM-03 | Added: `root.contracts.qrpc_core.strategy_ir.signal_indicator` single leaf closeout |
| v4.16.0 / BE-001QN-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `logic_position` |
| v4.16.0 / BE-001QO-01 | Added: `root.contracts.qrpc_core.strategy_ir.logic_position` baseline plan |
| v4.16.0 / BE-001QO-02 | Added: `root.contracts.qrpc_core.strategy_ir.logic_position` actual extraction closeout |
| v4.16.0 / BE-001QO-03 | Added: `root.contracts.qrpc_core.strategy_ir.logic_position` single leaf closeout |
| v4.16.0 / BE-001QP-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `risk_contract` |
| v4.16.0 / BE-001QQ-01 | Added: `root.contracts.qrpc_core.strategy_ir.risk_contract` baseline plan |
| v4.16.0 / BE-001QQ-02 | Added: `root.contracts.qrpc_core.strategy_ir.risk_contract` actual extraction closeout |
| v4.16.0 / BE-001QQ-03 | Added: `root.contracts.qrpc_core.strategy_ir.risk_contract` single leaf closeout |
| v4.16.0 / BE-001QR-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `data_requirement` |
| v4.16.0 / BE-001QS-01 | Added: `root.contracts.qrpc_core.strategy_ir.data_requirement` baseline plan |
| v4.16.0 / BE-001QS-02 | Added: `root.contracts.qrpc_core.strategy_ir.data_requirement` actual extraction closeout |
| v4.16.0 / BE-001QS-03 | Added: `root.contracts.qrpc_core.strategy_ir.data_requirement` single leaf closeout |
| v4.16.0 / BE-001QT-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `execution_contract` |
| v4.16.0 / BE-001QU-01 | Added: `root.contracts.qrpc_core.strategy_ir.execution_contract` baseline plan |
| v4.16.0 / BE-001QU-02 | Added: `root.contracts.qrpc_core.strategy_ir.execution_contract` actual extraction closeout |
| v4.16.0 / BE-001QU-03 | Added: `root.contracts.qrpc_core.strategy_ir.execution_contract` single leaf closeout |
| v4.16.0 / BE-001QV-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `gap_unknown_annotation` |
| v4.16.0 / BE-001QW-01 | Added: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` baseline plan |
| v4.16.0 / BE-001QW-02 | Added: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` actual extraction closeout |
| v4.16.0 / BE-001QW-03 | Added: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` single leaf closeout |
| v4.16.0 / BE-001QX-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `root_validation` |
| v4.16.0 / BE-001QY-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` baseline plan |
| v4.16.0 / BE-001QY-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` actual extraction closeout |
| v4.16.0 / BE-001QY-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` single leaf closeout continues split |
| v4.16.0 / BE-001QZ-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `identity_required_validation` |
| v4.16.0 / BE-001RA-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` baseline plan |
| v4.16.0 / BE-001RA-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` actual extraction closeout |
| v4.16.0 / BE-001RA-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` single leaf closeout |
| v4.16.0 / BE-001RB-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `signal_logic_validation` |
| v4.16.0 / GOV-SAME-PARENT-PARALLEL | Added: guarded same-parent child parallel wave protocol |
| v4.16.0 / BE-001RC-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` baseline plan |
| v4.16.0 / BE-001RC-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` actual extraction closeout |
| v4.16.0 / BE-001RC-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` single leaf closeout |
| v4.16.0 / BE-001RD-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `risk_validation` |
| v4.16.0 / BE-001RE-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` baseline plan |
| v4.16.0 / BE-001RE-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` actual extraction closeout |
| v4.16.0 / BE-001RE-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` single leaf closeout |
| v4.16.0 / BE-001RF-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `data_execution_validation` |
| v4.16.0 / BE-001RG-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` baseline plan |
| v4.16.0 / BE-001RG-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` actual extraction closeout |
| v4.16.0 / BE-001RG-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` single leaf closeout |
| v4.16.0 / BE-001RH-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `unknown_marker_validation` |
| v4.16.0 / BE-001RI-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` baseline plan |
| v4.16.0 / BE-001RI-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` actual extraction closeout |
| v4.16.0 / BE-001RI-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` single leaf closeout |
| v4.16.0 / BE-001RJ-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `test_fixture` |
| v4.16.0 / BE-001RK-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` baseline plan |
| v4.16.0 / BE-001RK-02 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` actual extraction closeout |
| v4.16.0 / BE-001RK-03 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` single leaf closeout |
| v4.16.0 / BE-001RL-01 | Added: `root.contracts.qrpc_core.strategy_ir.root_validation` parent closeout |
| v4.16.0 / BE-001RM-01 | Added: `root.contracts.qrpc_core.strategy_ir` parent closeout |
| v4.16.0 / BE-001RN-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `protocol_primitives` |
| v4.16.0 / BE-001RO-01 | Added: `root.contracts.qrpc_core.protocol_primitives` baseline plan |
| v4.16.0 / BE-001RO-02 | Added: `root.contracts.qrpc_core.protocol_primitives` actual extraction closeout |
| v4.16.0 / BE-001RO-03 | Added: `root.contracts.qrpc_core.protocol_primitives` single leaf closeout |
| v4.16.0 / BE-001RP-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `runtime_protocol_config` |
| v4.16.0 / BE-001RQ-01 | Added: `root.contracts.qrpc_core.runtime_protocol_config` baseline plan |
| v4.16.0 / BE-001RQ-02 | Added: `root.contracts.qrpc_core.runtime_protocol_config` actual extraction closeout |
| v4.16.0 / BE-001RQ-03 | Added: `root.contracts.qrpc_core.runtime_protocol_config` single leaf closeout |
| v4.16.0 / BE-001RR-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `artifact_specs` |
| v4.16.0 / BE-001RS-01 | Added: `root.contracts.qrpc_core.artifact_specs` baseline plan |
| v4.16.0 / BE-001RS-02 | Added: `root.contracts.qrpc_core.artifact_specs` actual extraction closeout |
| v4.16.0 / BE-001RS-03 | Added: `root.contracts.qrpc_core.artifact_specs` single leaf closeout continues split |
| v4.16.0 / BE-001RT-01 | Added: `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `canonical_digest` |
| v4.16.0 / BE-001RU-01 | Added: `root.contracts.qrpc_core.artifact_specs.canonical_digest` baseline plan |
| v4.16.0 / BE-001RU-02 | Added: `root.contracts.qrpc_core.artifact_specs.canonical_digest` actual extraction closeout |
| v4.16.0 / BE-001RU-03 | Added: `root.contracts.qrpc_core.artifact_specs.canonical_digest` single leaf closeout |
| v4.16.0 / BE-001RV-01 | Added: `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `run_backtest_specs` |
| v4.16.0 / BE-001RW-01 | Added: `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` baseline plan |
| v4.16.0 / BE-001RW-02 | Added: `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` actual extraction closeout |
| v4.16.0 / BE-001RW-03 | Added: `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` single leaf closeout |
| v4.16.0 / BE-001RX-01 | Added: `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `artifact_bundle_contract` |
| v4.16.0 / BE-001RY-01 | Added: `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` baseline plan |
| v4.16.0 / BE-001RY-02 | Added: `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` actual extraction closeout |
| v4.16.0 / BE-001RY-03 | Added: `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` single leaf closeout |
| v4.16.0 / BE-001RZ-01 | Added: `root.contracts.qrpc_core.artifact_specs` parent closeout |
| v4.16.0 / BE-001SA-01 | Added: `root.contracts.qrpc_core` parent residual judgment selects `runtime_io_contract` |
| v4.16.0 / BE-001SB-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` baseline plan |
| v4.16.0 / BE-001SB-02 | Added: `root.contracts.qrpc_core.runtime_io_contract` actual extraction closeout |
| v4.16.0 / BE-001SB-03 | Added: `root.contracts.qrpc_core.runtime_io_contract` single leaf closeout continues split |
| v4.16.0 / BE-001SC-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `market_data_io` |
| v4.16.0 / BE-001SD-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.market_data_io` baseline plan |
| v4.16.0 / BE-001SD-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.market_data_io` actual extraction closeout |
| v4.16.0 / BE-001SD-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.market_data_io` single leaf closeout |
| v4.16.0 / BE-001SE-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `decision_flow` |
| v4.16.0 / BE-001SF-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.decision_flow` baseline plan |
| v4.16.0 / BE-001SF-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.decision_flow` actual extraction closeout |
| v4.16.0 / BE-001SF-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.decision_flow` single leaf closeout |
| v4.16.0 / BE-001SG-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `execution_io` |
| v4.16.0 / BE-001SH-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.execution_io` baseline plan |
| v4.16.0 / BE-001SH-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.execution_io` actual extraction closeout |
| v4.16.0 / BE-001SH-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.execution_io` single leaf closeout |
| v4.16.0 / BE-001SI-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `portfolio_state` |
| v4.16.0 / BE-001SJ-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` baseline plan |
| v4.16.0 / BE-001SJ-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` actual extraction closeout |
| v4.16.0 / BE-001SJ-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` single leaf closeout |
| v4.16.0 / BE-001SK-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `runtime_output` |
| v4.16.0 / BE-001SL-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.runtime_output` baseline plan |
| v4.16.0 / BE-001SL-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.runtime_output` actual extraction closeout |
| v4.16.0 / BE-001SL-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.runtime_output` single leaf closeout |
| v4.16.0 / BE-001SM-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `backtest_output` |
| v4.16.0 / BE-001SN-01 | Added: `root.contracts.qrpc_core.runtime_io_contract.backtest_output` baseline plan |
| v4.16.0 / BE-001SN-02 | Added: `root.contracts.qrpc_core.runtime_io_contract.backtest_output` actual extraction closeout |
| v4.16.0 / BE-001SN-03 | Added: `root.contracts.qrpc_core.runtime_io_contract.backtest_output` single leaf closeout |
| v4.16.0 / BE-001SO-01 | Added: `root.contracts.qrpc_core.runtime_io_contract` parent closeout |
| v4.16.0 / BE-001SP-01 | 新增: `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects rfc_execution_contracts |
| v4.16.0 / BE-001SQ-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts equivalence baseline and extraction plan |
| v4.16.0 / BE-001SQ-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts actual extraction complete |
| v4.16.0 / BE-001SQ-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts single leaf closeout continues split |
| v4.16.0 / BE-001SR-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects data_request |
| v4.16.0 / BE-001SS-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request equivalence baseline and extraction plan |
| v4.16.0 / BE-001SS-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request actual extraction complete |
| v4.16.0 / BE-001SS-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request single leaf closeout stops split |
| v4.16.0 / BE-001ST-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects allocation |
| v4.16.0 / BE-001SU-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation equivalence baseline and extraction plan |
| v4.16.0 / BE-001SU-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation actual extraction complete |
| v4.16.0 / BE-001SU-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation single leaf closeout stops split |
| v4.16.0 / BE-001SV-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects order_contract |
| v4.16.0 / BE-001SW-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001SW-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract actual extraction complete |
| v4.16.0 / BE-001SW-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract single leaf closeout stops split |
| v4.16.0 / BE-001SX-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects execution_feedback |
| v4.16.0 / BE-001SY-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback equivalence baseline and extraction plan |
| v4.16.0 / BE-001SY-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback actual extraction complete |
| v4.16.0 / BE-001SY-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single leaf closeout stops split |
| v4.16.0 / BE-001SZ-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects handoff_snapshot |
| v4.16.0 / BE-001TA-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot equivalence baseline and extraction plan |
| v4.16.0 / BE-001TA-02 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot actual extraction complete |
| v4.16.0 / BE-001TA-03 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single leaf closeout stops split |
| v4.16.0 / BE-001TB-01 | 新增: `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment closes parent |
| v4.16.0 / BE-001TC-01 | 新增: `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects test_fixture |
| v4.16.0 / BE-001TD-01 | 新增: `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture equivalence baseline and extraction plan |
| v4.16.0 / BE-001TD-02 | 新增: `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture actual extraction complete |
| v4.16.0 / BE-001TD-03 | 新增: `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture single leaf closeout stops split |
| v4.16.0 / BE-001TE-01 | 新增: `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment closes parent |
| v4.16.0 / BE-001TF-01 | 新增: `root.contracts` root.contracts parent residual judgment selects core_ir |
| v4.16.0 / BE-001TG-01 | 新增: `root.contracts.core_ir` root.contracts.core_ir equivalence baseline and extraction plan |
| v4.16.0 / BE-001TH-01 | 新增: `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v1_contract |
| v4.16.0 / BE-001TI-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TI-02 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract actual extraction complete |
| v4.16.0 / BE-001TI-03 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract single leaf closeout continues split |
| v4.16.0 / BE-001TJ-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects root_graph_contract |
| v4.16.0 / BE-001TK-01 | 新增: `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TK-02 | 新增: `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract actual extraction complete |
| v4.16.0 / BE-001TK-03 | 新增: `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract single leaf closeout stops split |
| v4.16.0 / BE-001TL-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects data_indicator_expression_contract |
| v4.16.0 / BE-001TM-01 | 新增: `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TM-02 | 新增: `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract actual extraction complete |
| v4.16.0 / BE-001TM-03 | 新增: `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract single leaf closeout stops split |
| v4.16.0 / BE-001TN-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects policy_execution_contract |
| v4.16.0 / BE-001TO-01 | 新增: `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TO-02 | 新增: `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract actual extraction complete |
| v4.16.0 / BE-001TO-03 | 新增: `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract single leaf closeout stops split |
| v4.16.0 / BE-001TP-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects test_fixture |
| v4.16.0 / BE-001TQ-01 | 新增: `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture equivalence baseline and extraction plan |
| v4.16.0 / BE-001TQ-02 | 新增: `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture actual extraction complete |
| v4.16.0 / BE-001TQ-03 | 新增: `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture single leaf closeout stops split |
| v4.16.0 / BE-001TR-01 | 新增: `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment closes parent |
| v4.16.0 / BE-001TS-01 | 新增: `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v4_contracts |
| v4.16.0 / BE-001TT-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts equivalence baseline and split plan |
| v4.16.0 / BE-001TU-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects schema_identity_constants |
| v4.16.0 / BE-001TV-01 | 新增: `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants equivalence baseline and extraction plan |
| v4.16.0 / BE-001TV-02 | 新增: `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants actual extraction complete |
| v4.16.0 / BE-001TV-03 | 新增: `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants single leaf closeout stops split |
| v4.16.0 / BE-001TW-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects backtest_artifact_contract |
| v4.16.0 / BE-001TX-01 | 新增: `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TX-02 | 新增: `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract actual extraction complete |
| v4.16.0 / BE-001TX-03 | 新增: `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract single leaf closeout stops split |
| v4.16.0 / BE-001TY-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_contract |
| v4.16.0 / BE-001TZ-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001TZ-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract actual extraction complete |
| v4.16.0 / BE-001TZ-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract single leaf closeout continues split |
| v4.16.0 / BE-001UA-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment selects static_validation |
| v4.16.0 / BE-001UB-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UB-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation actual extraction complete |
| v4.16.0 / BE-001UB-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation single leaf closeout stops split |
| v4.16.0 / BE-001UC-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment closes parent |
| v4.16.0 / BE-001UD-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_graph_contract |
| v4.16.0 / BE-001UE-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract equivalence baseline and extraction plan |
| v4.16.0 / BE-001UE-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract actual extraction complete |
| v4.16.0 / BE-001UE-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract single leaf closeout continues split |
| v4.16.0 / BE-001UF-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects event_catalog |
| v4.16.0 / BE-001UG-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog equivalence baseline and extraction plan |
| v4.16.0 / BE-001UG-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog actual extraction complete |
| v4.16.0 / BE-001UG-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single leaf closeout stops split |
| v4.16.0 / BE-001UH-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects graph_static_validation |
| v4.16.0 / BE-001UI-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UI-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation actual extraction complete |
| v4.16.0 / BE-001UI-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split |
| v4.16.0 / BE-001UJ-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects risk_plane_validation |
| v4.16.0 / BE-001UK-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UK-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation actual extraction complete |
| v4.16.0 / BE-001UK-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single leaf closeout stops split |
| v4.16.0 / BE-001UL-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects event_usage_validation |
| v4.16.0 / BE-001UM-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UM-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation actual extraction complete |
| v4.16.0 / BE-001UM-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout continues split |
| v4.16.0 / BE-001UN-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_party_validation |
| v4.16.0 / BE-001UO-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UO-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation actual extraction complete |
| v4.16.0 / BE-001UO-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single leaf closeout stops split |
| v4.16.0 / BE-001UP-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_reference_resolution |
| v4.16.0 / BE-001UQ-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution equivalence baseline and extraction plan |
| v4.16.0 / BE-001UQ-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution actual extraction complete |
| v4.16.0 / BE-001UQ-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution single leaf closeout stops split |
| v4.16.0 / BE-001UR-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout stops split |
| v4.16.0 / BE-001US-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects graph_acyclic_validation |
| v4.16.0 / BE-001UT-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UT-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation actual extraction complete |
| v4.16.0 / BE-001UT-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation single leaf closeout stops split |
| v4.16.0 / BE-001UU-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split |
| v4.16.0 / BE-001UV-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects machine_identity_validation |
| v4.16.0 / BE-001UW-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UW-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation actual extraction complete |
| v4.16.0 / BE-001UW-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation single leaf closeout stops split |
| v4.16.0 / BE-001UX-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects edge_identity_validation |
| v4.16.0 / BE-001UY-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001UY-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation actual extraction complete |
| v4.16.0 / BE-001UY-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation single leaf closeout stops split |
| v4.16.0 / BE-001UZ-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout stops split |
| v4.16.0 / BE-001VA-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects traversal_helpers |
| v4.16.0 / BE-001VB-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers equivalence baseline and extraction plan |
| v4.16.0 / BE-001VB-02 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers actual extraction complete |
| v4.16.0 / BE-001VB-03 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers single leaf closeout stops split |
| v4.16.0 / BE-001VC-01 | 新增: `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment closes parent |
| v4.16.0 / BE-001VD-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects qs_state_machine_profile |
| v4.16.0 / BE-001VE-01 | 新增: `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile equivalence baseline and extraction plan |
| v4.16.0 / BE-001VE-02 | 新增: `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile actual extraction complete |
| v4.16.0 / BE-001VE-03 | 新增: `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile single leaf closeout stops split |
| v4.16.0 / BE-001VF-01 | 新增: `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects runtime_mode_contract |
| v4.16.0 / BE-001VG-01 | 新增: `root.contracts.core_ir.v4_contracts.runtime_mode_contract` root.contracts.core_ir.v4_contracts.runtime_mode_contract equivalence baseline and extraction plan |
