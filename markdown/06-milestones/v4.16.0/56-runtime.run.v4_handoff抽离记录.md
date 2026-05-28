# v4.16.0 runtime.run.v4_handoff 抽离记录
> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001H-02。  
> 基准: `55-runtime.run.v4_handoff单子叶等价基线.md`。  
> 判定: `runtime.run.v4_handoff` 已完成第一轮物理抽离；`/api/runtime/v4/run` 的 request/response、graph resolution、initial event、handoff projection 和 v4 simulated capability matrix 已迁入 `src/runtime/run/v4_handoff.rs`，父级 `runtime` 只保留受控 re-export 与共享 helper 出口。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001H handler 层抽离 | 推进 |
| 规范矩阵 | 父子通信、state owner 保留、provider 边界 | 固化 |
| 引导矩阵 | `runtime.run.v4_handoff` 白箱节点 | 更新 |
| 模块树 | `runtime.run.v4_handoff` 真实文件 | 更新 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.v4_handoff` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 与根7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.v4_handoff` |
| 真实文件 | `src/runtime/run/v4_handoff.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/backend/runtime/routes/run.rs` |
| public 方法 | `start_v4_runtime_run`、`runtime_v4_static_bundle`、`runtime_simulated_v4_matrix` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 实际变更

| 文件 | 处理 |
| --- | --- |
| `src/runtime/run/v4_handoff.rs` | 新增 v4 handoff handler 子模块，承载 `start_v4_runtime_run` 与其 helper/type |
| `src/runtime/mod.rs` | 声明 `run_v4_handoff` 子模块，并通过父级 re-export 暴露 `start_v4_runtime_run` |
| `src/runtime/run.rs` | 移除 v4 handoff 段落，保留 `start_test_run`、record store、replay/status、SSE sibling owner |
| `src/backend/runtime/routes/run.rs` | 不改 route path，只继续通过 `crate::runtime::start_v4_runtime_run` 调用父级出口 |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> crate::runtime::start_v4_runtime_run
  -> runtime::run_v4_handoff::start_v4_runtime_run
```

共享 helper 受控桥:

```text
src/runtime/backtest.rs
  -> runtime_v4_static_bundle / runtime_simulated_v4_matrix
  -> runtime::run_v4_handoff::{runtime_v4_static_bundle, runtime_simulated_v4_matrix}
```

该桥只为保持既有 backtest v4 QS / simulated matrix 行为等价，不代表允许 sibling 模块横向连接，也不代表进入发布版本过渡。

---

## 白箱边界

| 子域 | owner | 说明 |
| --- | --- | --- |
| route handler | `src/runtime/run/v4_handoff.rs` | `start_v4_runtime_run` 已迁入子模块 |
| request / response schema | `src/runtime/run/v4_handoff.rs` | `V4RuntimeRunRequest`、`V4RuntimeRunResponse`、`V4RuntimeRunDiagnostic`、`V4RuntimeRunHandoff` 字段不变 |
| source / graph resolution | `src/runtime/run/v4_handoff.rs` | `resolve_v4_runtime_run_graph` 语义不变 |
| event derivation | `src/runtime/run/v4_handoff.rs` | `handoff_initial_event` 与 `default_v4_payload_value` 语义不变 |
| handoff projection | `src/runtime/run/v4_handoff.rs` | `v4_runtime_handoff_response` response schema 不变 |
| simulated capability matrix | `src/runtime/run/v4_handoff.rs` | `runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` 仍只登记 RuntimeSimulated |
| run lock | `AppState.run_in_progress` | owner 与 AcqRel / Release 语义不变 |

---

## 禁止事项

- 不改 `/api/runtime/v4/run` route path、method、payload、response schema 或 error code。
- 不迁移 `run_in_progress`、AppState 字段 owner、runtime state owner、lock order 或 persistence。
- 不扩大 provider 支持，不引入 provider 真连接，不把 RuntimeSimulated 解释为真实 provider 可用。
- 不移动 `start_test_run`、record store、replay/status、SSE、mutation、backtest handler 或 evidence report owner。
- 不删除 `src/runtime/run.rs`，不宣称整个 run handler 已抽离完成。
- 不主动提出发布版本过渡，也不建立子模块横向直连。

---

## 等价证据

| 证据 | 覆盖 |
| --- | --- |
| `cargo check -p quantpilot` | 新子模块可见性、父级 re-export、route handler 类型 |
| `cargo test -p quantpilot --test api_run` | `/api/runtime/v4/run` source/graph/initial event/error path 等价 |
| `cargo test -p quantpilot --test api_backtest` | 共享 simulated matrix helper 对 backtest 路径不漂移 |
| `tools/check-matrix-governance.ps1` | 56 抽离记录、模块树和全量树锚点 |
| `tools/check-full-feature-tree.ps1` | 新代码文件与新里程碑文件纳入全量树 |

---

## 后续判断

`runtime.run.v4_handoff` 已从 `src/runtime/run.rs` 中抽出，下一步应先做单子叶整理 / closeout，确认该子模块是否继续细分。当前初判:

| 候选 | 值得继续细分 | 理由 |
| --- | :--: | --- |
| request/response schema | 暂不 | 字段少，主要服务单 route |
| graph resolution | 可能 | QS source audit 与 preparsed graph path 可成为内部 helper，但需先 closeout |
| event derivation | 暂不 | 逻辑窄，测试覆盖明确 |
| simulated capability matrix | 可能 | 已被 backtest 复用，但迁移到共享 runtime capability owner 需要另起方案 |

因此本批次不继续拆内部 helper。若继续推进，必须先对 `runtime.run.v4_handoff` 做 closeout，再决定是停止细分，还是把 `runtime.run.v4_handoff.graph_resolution` 或 capability matrix 另立子叶。

---

## 验收标准

1. `src/runtime/run/v4_handoff.rs` 存在，且全量树覆盖该路径。
2. `src/runtime/mod.rs` 保留 `pub(crate) use run_v4_handoff::start_v4_runtime_run` 兼容出口。
3. `src/runtime/run.rs` 不再拥有 v4 handoff handler/type/helper，但继续保留 legacy run sibling owner。
4. 模块树 `runtime.run.v4_handoff` 指向新真实文件与 56 抽离记录。
5. `api_run` 和 `api_backtest` 等价检查通过。
