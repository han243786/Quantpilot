# v4.16.0 backend.runtime.routes.run 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001G-03。
> 基准: `52-backend.runtime.routes.run单子叶等价基线.md`、`53-backend.runtime.routes.run抽离记录.md`。
> 判定: `backend.runtime.routes.run` route facade 完成单叶整理与等价 closeout；route facade 本身不继续细拆，但其保留的真实 handler owner `src/runtime/run.rs` 值得进入下一轮 handler 层递归。

---

## closeout 结论

`backend.runtime.routes.run` 当前已经完成本阶段能做的 route facade 抽离:

1. `src/backend/runtime/routes/run.rs` 只注册 run route group。
2. `src/backend/runtime/routes.rs` 继续作为父级 runtime route aggregate，并通过 `run::register_routes` 委托 run routes。
3. `/api/runtime/runs/:run_id/events` event stream 仍留在父级 `backend.runtime.routes`，没有混入 run 子叶。
4. `src/runtime/run.rs` 继续拥有真实 handler、state owner、persistence 与 event projection 调用边界。
5. 本 closeout 不改变任何 path、method、payload、response schema、error code、lock order 或 AppState owner。

因此 `backend.runtime.routes.run` 作为 route facade 可以阶段性收束；后续若继续，目标不应是拆 37 行 facade，而应回到真实 handler owner 建立新的等价基线。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001G 单叶 closeout、下一轮 handler 递归入口 | 收束 |
| 规范矩阵 | route facade 停止细分、handler owner 继续冻结 | 固化 |
| 引导矩阵 | `backend.runtime.routes.run`、`src/runtime/run.rs` | closeout 与后续坐标 |
| 模块树 | `backend.runtime.routes.run` | 更新 closeout 现状 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend runtime、根7.6 v4.16 |
| 模块树节点 | `backend.runtime.routes.run` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime/run.rs`、`src/runtime_persistence.rs`、`src/runtime_event_projection.rs` |
| public 方法 | `backend.runtime.routes.run::register_routes`、`start_test_run`、`start_v4_runtime_run`、`list_runs`、`save_run_record`、`get_run_detail`、`discard_run_record`、`get_run_replay`、`get_run_status`、`stream_run_events` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_backtest`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 等价事实

| 事实 | 当前状态 | 判定 |
| --- | --- | --- |
| route facade 规模 | `src/backend/runtime/routes/run.rs` 37 行 | 不值得继续拆 facade |
| handler owner 规模 | `src/runtime/run.rs` 544 行 | 值得另起 handler 层递归 |
| route group | `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs*` 非 SSE routes | 已由 `backend.runtime.routes.run` 接管 |
| event stream | `/api/runtime/runs/:run_id/events` | 保留父级，后续可作为 `runtime.event_stream` 单子叶 |
| 测试证据 | `api_run`、`api_sse`、`api_backtest` | 已能覆盖 run、SSE 排除边界和父聚合稳定性 |

---

## `src/runtime/run.rs` 内部拆分价值判断

`src/runtime/run.rs` 内部已经具备多个稳定职责，满足“独立 owner、独立验证证据、长期演进频率不同”的继续细分信号:

| 候选 handler 子叶 | 代表方法/类型 | 是否值得继续拆 | 原因 |
| --- | --- | :--: | --- |
| `runtime.run.v4_handoff` | `start_v4_runtime_run`、`resolve_v4_runtime_run_graph`、`handoff_initial_event`、`runtime_simulated_v4_matrix`、`V4RuntimeRunRequest` / `V4RuntimeRunResponse` | 值得 | v4 QS / graph handoff 与 runtime simulation 独立，`api_run` 已有多条 v4 测试 |
| `runtime.run.session_start` | `start_test_run`、capability guard、QS compile、sandbox session | 值得 | legacy paper run 启动路径涉及编译、capability、in-memory state 和 run guard |
| `runtime.run.record_store` | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` | 值得 | run record persistence、audit entry 和 discard 规则可以独立验证 |
| `runtime.run.replay_status` | `get_run_replay`、`get_run_status` | 值得 | replay/status 依赖 event projection 与 pagination，边界清楚 |
| `runtime.event_stream` | `stream_run_events` | 值得，但不属于本 route facade | SSE frame、keepalive、event envelope 应独立建基线 |
| `runtime.run.shared_legacy_types` | `RuntimeApprovalListQuery`、`MergeRecordsResponse`、`MergeRecordEntry` | 暂停 | 这些类型因 `include!("run.rs")` 架构被 sibling handlers 使用，不能在本批直接清理 |

---

## 继续细拆的推荐顺序

若继续推进 handler 层，推荐从 `runtime.run.v4_handoff` 开始:

1. 它占 `src/runtime/run.rs` 前半段，边界清楚，主要围绕 v4 source/graph handoff 和 paper simulated runtime。
2. 它的测试覆盖强，`api_run` 已覆盖 source、preparsed graph、initial event、missing source、handoff reject、event catalog missing 等路径。
3. 它不直接负责 saved run record、SSE streaming、report source 或 mutation approval。

但这不是本 closeout 的实现动作。下一步必须先建立 `runtime.run.v4_handoff` 等价基线，再决定是否移动 handler 内部实现。

---

## 本 closeout 不做

- 不移动 `src/runtime/run.rs` handler。
- 不拆 `src/runtime/run.rs` 的 v4 handoff、session start、record store、replay/status 或 SSE 实现。
- 不清理 `include!("run.rs")` 带来的 sibling type 共用问题。
- 不迁移 persistence owner、event projection owner、AppState owner、runtime state owner 或 lock order。
- 不宣称 `backend.runtime.routes`、`backend.runtime` 或 `root.backend` 已完成。
- 不把 handler 层继续细拆解释为已进入整理或重构阶段。

---

## 等价证据

| 证据 | 覆盖范围 | 判定 |
| --- | --- | --- |
| `cargo check -p quantpilot` | route facade、handler 可见性、Axum handler 类型 | 已通过 |
| `cargo test -p quantpilot --test api_run` | run/v4 run/list/detail/save/replay/status/report | 已通过 |
| `cargo test -p quantpilot --test api_sse` | event stream 保留在父 aggregate 后的可用性 | 已通过 |
| `cargo test -p quantpilot --test api_backtest` | 父 aggregate 仍承载 backtest routes | 已通过 |
| `tools/check-matrix-governance.ps1` | closeout 记录、模块树、全量树锚点 | 已通过 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 已通过 |

---

## 后续入口

下一步有两个合规选择:

| 选择 | 条件 | 下一步 |
| --- | --- | --- |
| 继续 handler 层递归 | 开发者确认继续拆 run handler | 建立 `runtime.run.v4_handoff` 单子叶等价基线 |
| 改走其他 runtime route 子叶 | 开发者选择切换目标 | 从 `runtime.event_stream`、`runtime.backtest`、`runtime.mutation_ai_proposal`、`runtime.report_experiment` 中单选一个建基线 |

默认建议: 继续 `runtime.run.v4_handoff`，因为它独立、测试强、状态迁移风险小于 `session_start` 和 `record_store`。

---

## 验收标准

1. `54-backend.runtime.routes.run单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树更新 `backend.runtime.routes.run` 的 closeout 状态。
3. 全量树能定位本 closeout。
4. 治理门禁能发现本文件缺失。
5. closeout 明确说明 route facade 本身停止细分，但 `src/runtime/run.rs` handler 层值得另起基线继续拆。
