# v4.16.0 runtime.backtest.execution_start.v4_projection 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001O-04。  
> 基准: `82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`、`83-runtime.backtest.execution_start.v4_projection抽离方案.md`、`84-runtime.backtest.execution_start.v4_projection抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start.v4_projection` 已完成等价 closeout，并设置 `stop_split: true`；该子叶不继续细拆。下一候选回到父叶 `runtime.backtest.execution_start`，优先评估 `runtime.backtest.execution_start.v4_request_resolution`。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001O 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | projection helper 私有性、父级私有调用、stop_split 判定、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` | closeout |
| 模块树 | `runtime.backtest.execution_start.v4_projection` 白箱节点 | 更新状态与下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_projection` |
| 父模块 | `runtime.backtest.execution_start` |
| 真实文件 | `src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/execution_start.rs` |
| 保留 owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs` |
| 关键 public 方法 | 无新增 public API；父级只通过 `pub(super)` helper 调用 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| 父级调用 | 等价 | `execution_start.rs` 只私有导入 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` |
| output projection | 等价 | `build_v4_backtest_output` 仍生成 `BacktestOutput`，不改变 `mode = v4_backtest`、summary、portfolio 或 artifact embedding |
| equity curve | 等价 | `v4_equity_curve_from_artifact` 对空 artifact 仍返回空数组，不伪造 zero point |
| win rate | 等价 | `v4_win_rate_from_equity_curve` 仍忽略 flat step 与非有限值 |
| portfolio | 等价 | `v4_portfolio_from_artifact` 仍从 final snapshot 读取 portfolio 字段并保持安全默认值 |
| frontend events | 等价 | `frontend_events_from_v4_backtest_artifact` 与 `v4_frontend_event` 仍保持 payload projection、trace id 与排序语义 |
| 单元测试 | 等价 | `v4_win_rate_counts_up_steps_over_directional_steps` 与 `v4_equity_curve_empty_artifact_does_not_fabricate_zero_point` 已随 helper 保留在 `src/runtime/backtest/v4_projection.rs` |
| owner 边界 | 保留 | request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 均未迁移 |
| 发布过渡 | 未启动 | `release transition guard` 生效，未新增横向直连或性能旁路 |

---

## 当前白箱结构

| helper | 当前 owner | 细分判断 |
| --- | --- | --- |
| `build_v4_backtest_output` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。它是 output projection 主入口 |
| `v4_win_rate_from_equity_curve` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。只服务 output summary |
| `v4_equity_curve_from_artifact` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。父级需要其输出写入 response/artifact view |
| `v4_portfolio_from_artifact` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。只服务 output final portfolio |
| `frontend_events_from_v4_backtest_artifact` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。它是 frontend event projection 主入口 |
| `v4_frontend_event` | `src/runtime/backtest/v4_projection.rs` | 保留在本叶。只服务 frontend event projection |

---

## 细分价值判断

**最终判定**: `stop_split: true`。

理由:

1. 本叶没有 state、IO、锁、route、persistence、schema owner 或外部 API。
2. 本叶只有 v4 artifact projection 一个白箱目标，拆成 output projection 与 frontend event projection 会增加父级导入面，但不会减少跨模块耦合。
3. 三个 `pub(super)` 入口已经足够清晰；其余 helper 均为子模块私有，继续细拆会制造微文件和额外维护面。
4. 若未来发现 request resolution、schema owner 或 frontend caller 需要拆分，必须回到父叶另起基线，不能从本叶继续延展。

---

## 后续递归队列

| 顺序 | 候选 | 进入条件 |
| --- | --- | --- |
| 1 | `runtime.backtest.execution_start.v4_request_resolution` | 下一批若继续，应先建单子叶等价基线，冻结 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` |
| 2 | `runtime.backtest.record_store` | 只有当 execution_start 内部候选完成或暂停后，才回到 `runtime.backtest` sibling 队列 |
| 3 | `runtime.backtest.replay_status` | 必须另起基线，不从 `v4_projection` 内部迁移 |
| 4 | `runtime.backtest.experiment_sweep` | 只能作为 sibling/独立候选处理，不得混入 projection closeout |

---

## 本批次不做

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 record store、replay、experiment、artifact schema、compare owner、response mapping owner、persistence owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`PortfolioState`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不引入发布版本过渡、横向直连、缓存旁路或性能优化提案。
- 不进入整理、重构或删除旧实现。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_projection` 已 closeout 时，必须说明: 本叶只完成 v4 artifact projection helper 与两个单元测试的等价 closeout，并设置 `stop_split: true`。不得宣称 `execute_v4_backtest_request`、v4 request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `85-runtime.backtest.execution_start.v4_projection单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.backtest.execution_start.v4_projection` 已 closeout，并设置 `stop_split: true`。
3. 下一候选回到父叶 `runtime.backtest.execution_start.v4_request_resolution`，后续必须先建等价基线。
4. `src/runtime/backtest/v4_projection.rs` 的三个父级入口与三个私有 helper 被白箱登记。
5. 治理门禁能发现 closeout 文档、stop_split、下一候选、禁止迁移边界和回归证据缺失。
