# v4.16.0 runtime.backtest 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001M-04。  
> 基准: `74-runtime.backtest单子叶等价基线.md`、`75-runtime.backtest抽离方案.md`、`76-runtime.backtest抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: backtest route facade 本身完成等价 closeout，不继续细拆；`src/runtime/backtest.rs` handler 域仍值得另起单子叶等价基线继续递归。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001M `runtime.backtest` route facade 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、handler owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest` | closeout |
| 模块树 | `backend.runtime.routes.backtest`、`runtime.backtest` 白箱节点 | 更新状态与下一候选 |

---

## 等价判定

| 检查项 | 结论 |
| --- | --- |
| route path | 等价。所有 backtest route path 保持不变 |
| route method | 等价。GET/POST/DELETE 保持不变 |
| handler 调用 | 等价。继续调用 `runtime_handlers::*` 与 `backtest_compare::compare_backtests` |
| route 顺序 | 等价。父级先接入 backtest route facade，再接入 run route facade，再注册 event stream、evidence、mutation、report、experiment、approval 和 ops routes |
| response schema | 未变更 |
| error code | 未变更 |
| AppState / store dir / 锁顺序 | 未变更 |
| frontend caller | 未变更 |

---

## 真实 route facade 结果

| route | method | 当前 owner | handler |
| --- | --- | --- | --- |
| `/api/runtime/backtest` | POST | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::start_backtest_run` |
| `/api/runtime/backtests` | GET | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::list_backtests` |
| `/api/runtime/backtests/compare` | POST | `src/backend/runtime/routes/backtest.rs` | `backtest_compare::compare_backtests` |
| `/api/runtime/backtests/:backtest_id/save` | POST | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::save_backtest_record` |
| `/api/runtime/backtests/:backtest_id` | GET | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::get_backtest_detail` |
| `/api/runtime/backtests/:backtest_id` | DELETE | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::discard_backtest_record` |
| `/api/runtime/backtests/:backtest_id/replay` | GET | `src/backend/runtime/routes/backtest.rs` | `runtime_handlers::get_backtest_replay` |

---

## 保留边界

- `src/runtime/backtest.rs` 仍拥有 backtest handler 与 shared helper。
- `src/backend/runtime/routes.rs` 仍是 runtime parent route aggregate，只委托 `src/backend/runtime/routes/backtest.rs`。
- `src/backtest_compare.rs` 仍拥有 compare API handler。
- `src/backtest_artifacts.rs` 仍拥有 artifact views 与 artifact schema。
- `src/runtime_persistence.rs` 仍拥有 persisted/transient backtest record IO。
- `src/runtime_response_mapping.rs` 仍拥有 replay/detail/list response mapping。
- `src/frontend_api_types.rs` 仍拥有前后端 API 类型。
- `runtime.event_stream`、`runtime.run`、experiment/report/mutation/approval 均排除在本叶 closeout 之外。
- 不主动提出发布版本过渡，不新增子模块横向连接。ASCII guard: `release transition guard`。

保留的关键 shared helper:

| helper | 保留原因 |
| --- | --- |
| `build_backtest_artifact_views` | artifact views helper retained |
| `maybe_spill_transient_backtest_record` | transient spill helper retained |
| `load_backtest_record_from_state` | load backtest helper retained |
| `persist_backtest_record` | persist backtest helper retained |
| `normalized_replay_options` | shared replay options helper retained |

---

## 细分价值判断

`backend.runtime.routes.backtest` route facade 本身停止细分，`stop_split: true`。它只承担 route registration，继续拆会制造无意义的微文件。

`src/runtime/backtest.rs` handler 域值得继续细拆，原因如下:

| 候选 | 价值判断 | 理由 |
| --- | --- | --- |
| `runtime.backtest.execution_start` | 值得拆 | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` 和 v4 backtest helper 聚合了执行路径、artifact 生成、governance event 和 transient store 写入 |
| `runtime.backtest.record_store` | 值得拆 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 与 persistence/audit 关系清晰，适合仿照 `runtime.run.record_store` |
| `runtime.backtest.replay_status` | 可能值得拆 | `get_backtest_replay` 与 replay query/options/response mapping 边界清楚，但体量较小，需先建基线再判断 |
| `runtime.backtest.experiment_sweep` | 值得单独登记 | experiment routes 当前不属于 backtest route facade，但实现仍在 `src/runtime/backtest.rs`，应作为后续 sibling 或独立候选处理，不得混入 route facade closeout |

默认下一候选为 `runtime.backtest.execution_start`，因为它是当前 backtest handler 域中风险最高、牵连最多、最需要先冻结等价基线的执行入口。

---

## 下一步

下一批进入 `BE-001N-01 runtime.backtest.execution_start 单子叶等价基线`。该批只允许建立基线，不直接迁移代码；必须先冻结 `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request`、v4 helper、artifact generation、governance event、transient spill 和 `api_backtest` 证据。

不得直接移动 record store、replay、experiment、artifact schema、compare owner、persistence owner、schema owner、state owner 或 frontend caller。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
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

AI 声称 `runtime.backtest` 已 closeout 时，必须说明只完成 route facade 的 closeout。不得宣称 `src/runtime/backtest.rs` handler 域已经拆完，也不得宣称 artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller、experiment routes 或发布过渡已经迁移。

---

## 验收标准

1. `77-runtime.backtest单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `src/backend/runtime/routes/backtest.rs` 等价并停止 route facade 内部细分。
3. closeout 明确 `src/runtime/backtest.rs` handler 域仍值得继续递归，默认下一候选为 `runtime.backtest.execution_start`。
4. closeout 明确 record store、replay、experiment、artifact schema、compare、persistence、schema、state、frontend 和发布过渡均未迁移。
5. 本批验证通过后，后续才能进入 BE-001N-01 等价基线。
