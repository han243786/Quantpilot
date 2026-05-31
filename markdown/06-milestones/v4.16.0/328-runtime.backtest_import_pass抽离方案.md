# v4.16.0 runtime.backtest_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DD-02
> 基准: `327-runtime.backtest_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.backtest_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DD-02 `runtime.backtest_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | explicit import pass、minimum batch、backtest import pocket、release transition guard | 拆分判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 分批路线 |
| 模块树 | `runtime.backtest_import_pass` | `stop_split: false` |

---

## 适配性结论

不采用 11 文件整批 import rewrite。wide batch rejection: no 11-file batch。

理由:

1. `runtime.backtest_import_pass` 同时覆盖 direct route files、experiment sweep 子树和 execution start / v4 子树，整批会把三种依赖面混在一起。
2. `execution_start.rs` 内部还挂载 `legacy_dispatch`、`v4_projection`、`v4_request_resolution` 与 `v4_runtime_execution`，一次处理会同时触碰 v4 request、runtime execution、projection 三条链。
3. `experiment_sweep.rs` 内部还挂载 `parameter_grid`、`record_lifecycle` 与 `start_orchestration`，它与 backtest record CRUD 的等价证据不同。
4. 直接 route singleton 文件可以先验证 backtest parent surface 的显式 import 写法，风险低、反馈快。

因此本叶保持:

```text
runtime.backtest_import_pass stop_split: false
```

---

## 分批路线

后续按以下 pocket 递归:

```text
runtime.backtest.record_store_import_pass
runtime.backtest.replay_import_pass
runtime.backtest.experiment_sweep_import_pass
runtime.backtest.execution_start_import_pass
runtime.backtest_import_pass closeout
runtime.parent_import_bridge residual judgement
```

当前只锁定第一批:

```text
BE-001DE-01 runtime.backtest.record_store_import_pass 单子叶等价基线
```

---

## 第一批范围

第一批只处理:

```text
src/runtime/backtest/record_store.rs
```

目标是将该文件的:

```rust
use super::*;
```

改为显式 import。预计仅影响 backtest list/detail/save/discard route handlers，不处理 replay、experiment、execution start 或 v4 runtime execution。

### 预计父级输入面

`record_store.rs` 后续实际抽离需要显式输入:

- Axum: `State`、`Path`、`Query`、`Json`、`StatusCode`。
- schema / state: `AppState`、`PaginatedResponse`、`PaginationQuery`、`BacktestDetailResponse`、`BacktestListItem`、`DiscardRuntimeArtifactResponse`。
- auth / persistence: `auth`、`list_backtest_records`、`load_backtest_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`。
- response mapping: `backtest_detail_response_from_record`、`backtest_list_item_from_record`。
- governance / audit: `build_graph_audit_entry`、`persist_graph_audit_entry`、`GraphAuditAction`。
- utility: `io_error`、`paginate`、`sanitize_storage_path_segment`。
- filesystem: `tokio::fs`。

---

## 等价保护

第一批实际抽离时必须保持:

1. `list_backtests` 的 saved records 读取、降序排序和 pagination 不变。
2. `get_backtest_detail` 的 scoped lookup 和 response projection 不变。
3. `save_backtest_record` 的 persistence、transient deletion、in-memory update 和 audit 写入不变。
4. `discard_backtest_record` 对 saved record 的 conflict 保护、transient deletion 与 memory cleanup 不变。
5. `DiscardRuntimeArtifactResponse` 继续经父级白箱输入，不新增 sibling horizontal link。

---

## 排除项

- 本批不修改 Rust 代码。
- BE-001DD-02 不直接改写 `src/runtime/backtest/record_store.rs`。
- 第一批不处理 `src/runtime/backtest/replay.rs`。
- 第一批不处理 `src/runtime/backtest/experiment_sweep.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 或 `parameter_grid.rs`。
- 第一批不处理 `src/runtime/backtest/execution_start.rs`、`legacy_dispatch.rs`、`v4_projection.rs`、`v4_request_resolution.rs` 或 `v4_runtime_execution.rs`。
- 完整排除路径锚点: `src/runtime/backtest/record_lifecycle.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/legacy_dispatch.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_runtime_execution.rs`。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不新增 sibling horizontal link。
- 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 `runtime.backtest.record_store_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_backtest
```

---

## 幻觉检查点

AI 声称 BE-001DD-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 本方案拒绝 11 文件整批 import rewrite。
3. `runtime.backtest_import_pass stop_split: false`。
4. 下一步只能进入 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线。
5. 尚未改写 `src/runtime/backtest/record_store.rs`。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已抽离、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `328-runtime.backtest_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确 `runtime.backtest_import_pass stop_split: false`。
3. 下一步固定为 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
