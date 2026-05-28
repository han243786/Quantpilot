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
| v4.16.0 | 推进中: 模块化抽离第一波，已完成 system.entry.backend_process 抽离、经验回填、递归模块化流程、S1-S10 closeout 或静态 closeout、`root.system` 顶层阶段性 closeout；backend 已完成 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D strategy_config L3 模块壳抽离、BE-001E 其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录；BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，下一候选为 `runtime.backtest.execution_start.v4_projection` |
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
