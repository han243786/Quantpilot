# v4.16.0 runtime.run.session_start 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001I-02。  
> 基准: `58-runtime.run.session_start单子叶等价基线.md`。  
> 判定: `runtime.run.session_start` 已完成第一轮物理抽离；legacy `/api/runtime/test-run` 的 `start_test_run` handler 已迁入 `src/runtime/run/session_start.rs`，父级 `runtime` 保留受控 re-export，route facade、request/response schema、run lock、state owner、record/replay/status、SSE 和 persistence 语义不变。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001I handler 层抽离 | 推进 |
| 规范矩阵 | session start owner、run lock、state owner、父子通信 | 固化 |
| 引导矩阵 | `runtime.run.session_start` 白箱节点 | 更新 |
| 模块树 | `runtime.run.session_start` 真实文件 | 更新 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.session_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 与根7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.session_start` |
| 真实文件 | `src/runtime/run/session_start.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_validation.rs`、`src/runtime_event_projection.rs`、`src/runtime_response_mapping.rs`、`src/compile_api.rs`、`src/capability_api.rs`、`src/collaboration.rs`、`src/graph_quantscript_api.rs`、`src/frontend_runtime_mapping.rs` |
| public 方法 | `start_test_run`、`FrontendRunRequest`、`RunStartResponse`、`validate_runtime_capability_guard`、`validate_runtime_config_capabilities`、`compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`build_compile_runtime_targets_from_graph`、`merge_runtime_targets`、`runtime_governance_snapshot`、`collect_frontend_events`、`prepend_capability_snapshot_event`、`attach_runtime_event_envelopes`、`validate_runtime_event_envelopes`、`account_summary`、`normalize_actor_identity`、`collaboration_with_run_actor`、`run_start_response` |

---

## 实际抽离

| 文件 | 变更 | 等价说明 |
| --- | --- | --- |
| `src/runtime/run/session_start.rs` | 新增 session start handler 子模块，承载 `start_test_run` | 从 `src/runtime/run.rs` 物理迁入，保留函数签名和返回类型 |
| `src/runtime/mod.rs` | 声明 `run_session_start` 子模块，并通过 `pub(crate) use run_session_start::start_test_run` 暴露父级出口 | `crate::runtime::start_test_run` 兼容入口不变 |
| `src/runtime/run.rs` | 移除 `start_test_run` 实现，继续保留 record/replay/status/SSE sibling | 不迁移 state owner、record store、replay/status、SSE 或 persistence |
| `src/backend/runtime/routes/run.rs` | 不改 route path 或调用目标 | `/api/runtime/test-run` 仍调用 `runtime_handlers::start_test_run` |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> crate::runtime::start_test_run
  -> runtime::run_session_start::start_test_run
  -> compile_runtime_protocol_via_qs
  -> compile_runtime_protocol_config
  -> RealTimeSandbox::new(RuntimeCoordinator::new(compiled))
  -> state.runs.insert(scoped run record)
  -> RunStartResponse
```

本批只改变 handler 的物理文件位置；不改变 HTTP path、method、payload、response schema、error code、run lock、event order 或 in-memory run record 写入语义。

---

## owner 保留

| 子域 | owner | 本批状态 |
| --- | --- | --- |
| route facade | `src/backend/runtime/routes/run.rs` | 不变 |
| handler | `src/runtime/run/session_start.rs` | 已迁入新子模块 |
| request / response schema | `src/frontend_api_types.rs`、`src/runtime_response_mapping.rs` | 不变 |
| capability guard | `src/runtime_validation.rs` | 不变 |
| compile path | `src/compile_api.rs` | 不变 |
| run lock | `AppState.run_in_progress` / `RunInProgressGuard` | owner 不变 |
| run record state | `AppState.runs` | owner 不变 |
| record store / replay / status | `src/runtime/run.rs` | 不迁移 |
| SSE | `src/runtime/run.rs` | 不迁移 |
| persistence | `src/runtime_persistence.rs` | 不迁移 |

---

## 等价证据

| 命令 | 必须证明 |
| --- | --- |
| `cargo fmt --check` | 新模块格式稳定 |
| `cargo check -p quantpilot` | 父级 re-export、route facade、handler 依赖可见性正确 |
| `cargo test -p quantpilot --test api_run` | `/api/runtime/test-run`、run response、target mapping、report/replay 代表链路不漂移 |
| `tools/check-matrix-governance.ps1` | BE-001I-02、模块树、全量树和门禁锚点完整 |
| `tools/check-full-feature-tree.ps1` | 新文件和抽离记录被全量树覆盖 |

---

## 本批次不做

- 不改 `/api/runtime/test-run` route path、method、payload、response schema 或 error code。
- 不迁移 `run_in_progress` owner、AppState 字段 owner、runtime state owner、lock order、`state.runs` owner 或 persistence。
- 不拆 `runtime.run.record_store`、`runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.v4_handoff` 或 backtest/mutation/report owner。
- 不清理旧中文字符串或历史注释编码问题；这属于整理/文档治理，不属于本抽离批次。
- 不宣称 `runtime.run.session_start` 已完成单叶 closeout。

---

## 后续判断

下一步应对 `runtime.run.session_start` 做单子叶整理 / closeout，再判断内部是否值得继续细拆。当前初判:

| 候选 | 初判 | 原因 |
| --- | --- | --- |
| `runtime.run.session_start.capability_guard` | 暂不拆 | guard 真实 owner 在 `runtime_validation`，本叶只是调用者 |
| `runtime.run.session_start.compile_path` | 暂不拆 | compile 真实 owner 在 `compile_api`，本叶只是 orchestration |
| `runtime.run.session_start.session_execution` | 可讨论 | sandbox session、event projection、record 写入同处启动链路，若继续拆需先证明能降低复杂度 |
| `runtime.run.record_store` | 值得另起 sibling | record list/detail/save/discard 与 session start 不是同一职责 |
| `runtime.run.replay_status` | 值得另起 sibling | replay/status projection 可独立于 session start |

因此本批不继续拆内部 helper。若继续推进，必须先完成 `runtime.run.session_start` closeout，再决定是停止细分，还是另立更小子叶。

---

## 验收标准

1. `src/runtime/run/session_start.rs` 存在，且承载 `start_test_run`。
2. `src/runtime/mod.rs` 保留 `pub(crate) use run_session_start::start_test_run` 兼容出口。
3. `src/backend/runtime/routes/run.rs` 不改 `/api/runtime/test-run` route。
4. 模块树 `runtime.run.session_start` 指向新真实文件与 59 抽离记录。
5. 全量树覆盖 `src/runtime/run/session_start.rs` 和本抽离记录。
6. `api_run` 代表测试继续通过。
