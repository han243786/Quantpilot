# v4.16.0 runtime.backtest.execution_start 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001N-04。  
> 基准: `78-runtime.backtest.execution_start单子叶等价基线.md`、`79-runtime.backtest.execution_start抽离方案.md`、`80-runtime.backtest.execution_start抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start` 已完成等价 closeout，但本叶不设置 `stop_split: true`；内部 `runtime.backtest.execution_start.v4_projection` 值得进入下一轮单子叶等价基线。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001N 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、`pub(super)` experiment 复用桥、helper 私有性、继续细分判定 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start` | closeout |
| 模块树 | `runtime.backtest.execution_start` 白箱节点 | 更新状态与下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start` |
| 父模块 | `runtime.backtest` |
| 真实文件 | `src/runtime/backtest/execution_start.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs` |
| 保留 owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs` |
| 关键 public 方法 | `start_backtest_run` 继续由父级 `runtime` re-export 给 route facade |
| 内部复用桥 | `execute_backtest_request` 保持 `pub(super)`，只给父级 `runtime` 内部和 experiment sweep 复用 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `POST /api/runtime/backtest` 仍经 `backend.runtime.routes.backtest -> runtime::start_backtest_run` |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `pub(crate) use backtest_execution_start::start_backtest_run` |
| experiment 复用 | 等价 | `execute_backtest_request` 只升为 `pub(super)` 内部桥，不成为对外 public API |
| legacy backtest | 等价 | legacy graph compile、sandbox run、artifact view 和 transient record 写入语义不变 |
| v4 backtest | 等价 | `execute_v4_backtest_request`、v4 graph/symbol/event helper 和 deterministic replay 输出语义不变 |
| 单元测试归属 | 已整理 | `v4_win_rate_from_equity_curve` 与 `v4_equity_curve_from_artifact` 单元测试已随 helper 移入 `src/runtime/backtest/execution_start.rs` |
| sibling 边界 | 保留 | record store、replay、experiment、artifact schema、compare owner、persistence owner、schema owner、state owner、frontend caller 均未迁移 |
| 发布过渡 | 未启动 | `release transition guard` 保持生效，未引入横向直连 |

---

## 当前白箱结构

| 函数簇 | 当前 owner | 细分判断 |
| --- | --- | --- |
| `start_backtest_run` | `runtime.backtest.execution_start` | 保留。本函数是 route handler 外壳，继续通过父级 re-export 暴露 |
| `execute_backtest_request` | `runtime.backtest.execution_start` | 保留。它同时服务 `start_backtest_run` 与 `start_backtest_experiment`，是父级内部兼容桥 |
| `execute_v4_backtest_request` | `runtime.backtest.execution_start` | 暂保留。它仍需要 AppState、request、record 写入和 v4 projection，当前不先拆 |
| `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` | `runtime.backtest.execution_start` | 值得后续登记，但不作为第一候选 |
| `build_v4_backtest_output`、`v4_win_rate_from_equity_curve`、`v4_equity_curve_from_artifact`、`v4_portfolio_from_artifact`、`frontend_events_from_v4_backtest_artifact`、`v4_frontend_event` | `runtime.backtest.execution_start` | 值得继续细拆，默认下一候选为 `runtime.backtest.execution_start.v4_projection` |

---

## 细分价值判断

**最终判定**: `runtime.backtest.execution_start` 不停止内部细分。它已经从 `src/runtime/backtest.rs` 中抽成独立白箱，但文件内部仍混合了 route handler、legacy execution、v4 execution、v4 request resolution、v4 response projection、frontend event projection 和 transient record 写入。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.backtest.execution_start.v4_projection` | 值得先拆 | 输入/输出清晰，主要处理 `V4BacktestArtifact -> BacktestOutput / PortfolioSnapshot / FrontendRuntimeEvent`，已有单元测试证据，几乎不触碰 AppState 与锁 |
| `runtime.backtest.execution_start.v4_request_resolution` | 值得后续登记 | graph/symbol/event type 解析边界清晰，但与 `execute_v4_backtest_request` 的错误响应和 contract validation 耦合更高 |
| `runtime.backtest.execution_start.legacy_dispatch` | 暂不拆 | legacy path 仍和 compile artifact、sandbox session、record 写入和 shared owner 交织，先拆会扩大回归面 |
| `runtime.backtest.execution_start.record_write_bridge` | 不在本轮拆 | transient spill、record persistence、artifact views 和 state owner 是共享边界，不能私有化到 execution_start 内部 |
| `runtime.backtest.record_store` | 不在本叶内拆 | 它是 `runtime.backtest` sibling，必须回到父级另起基线 |
| `runtime.backtest.replay_status` | 不在本叶内拆 | 它是 replay sibling，不能混入 execution_start closeout |
| `runtime.backtest.experiment_sweep` | 不在本叶内拆 | experiment route/handler 当前只通过 `execute_backtest_request` 复用桥连接，不归本叶私有 |

---

## 父子通信收口

```text
backend.runtime.routes
  -> backend.runtime.routes.backtest
  -> runtime::start_backtest_run
  -> runtime::backtest_execution_start::start_backtest_run
  -> execute_backtest_request
      -> legacy execution path
      -> execute_v4_backtest_request
          -> v4 request resolution
          -> v4_projection candidate
  -> BacktestRunResponse
```

本叶只能通过父级 `runtime` 和 route facade 暴露创建路径。下一轮即使拆 `v4_projection`，也必须由 `runtime.backtest.execution_start` 父模块调用，不能让 record store、replay、experiment、frontend caller 或 persistence owner 横向接入。

---

## 后续递归队列

| 顺序 | 候选 | 进入条件 |
| --- | --- | --- |
| 1 | `runtime.backtest.execution_start.v4_projection` | 下一批先建单子叶等价基线，冻结 pure projection helper、单元测试和 API 回归证据 |
| 2 | `runtime.backtest.execution_start.v4_request_resolution` | 仅在 v4 projection closeout 后再评估，不能抢先迁移 |
| 3 | `runtime.backtest.record_store` | 只有当 execution_start 内部值得细拆的候选完成或暂停后，才回到 `runtime.backtest` sibling 队列 |
| 4 | `runtime.backtest.replay_status` | 必须另起基线，不从 execution_start 内部直接迁移 |

---

## 本批次不做

- 不迁移 record store、replay、experiment、artifact schema、compare owner、persistence owner、schema owner、state owner 或 frontend caller。
- 不迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`、`start_backtest_experiment` 或 `compare_backtests`。
- 不改变 `POST /api/runtime/backtest` route path/method。
- 不改变 `BacktestRunResponse`、`BacktestRecord`、runtime event envelope 或 v4 artifact schema。
- 不把 `execute_backtest_request` 变成对外 public API。
- 不引入发布版本过渡、横向直连、缓存旁路或性能优化提案。
- 不删除旧实现，不进入整理/重构阶段。

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

AI 声称 `runtime.backtest.execution_start` 已 closeout 时，必须说明: 本叶已完成创建路径 handler/helper 的等价 closeout，但尚未停止内部细分；下一候选是 `runtime.backtest.execution_start.v4_projection`。不得宣称 record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `81-runtime.backtest.execution_start单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.backtest.execution_start` 已 closeout，且明确不设置 `stop_split: true`。
3. 下一候选固定为 `runtime.backtest.execution_start.v4_projection`，后续必须先建等价基线。
4. `src/runtime/backtest/execution_start.rs` 的真实文件边界和父级 re-export 被全量树覆盖。
5. 治理门禁能发现本 closeout 文档、下一候选、禁止迁移边界和回归证据缺失。
