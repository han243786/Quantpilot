# v4.16.0 runtime.backtest.record_store 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001T-01。  
> 基准: `98-runtime.backtest.execution_start父叶残余判断.md`、`77-runtime.backtest单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `runtime.backtest.record_store` 单子叶等价基线；本批只冻结 backtest list/detail/save/discard、transient/persistent record、artifact view、audit、父子通信和回归证据，不移动代码。  
> 代码动作: `no code movement`。

---

## 选择理由

`runtime.backtest.execution_start` 已完成父叶残余判断，下一步必须回到 `runtime.backtest` 上层 sibling 队列。`runtime.backtest.record_store` 是最适合承接的下一片:

1. 它覆盖 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`，正好是 backtest 创建后的记录查询、保存和丢弃边界。
2. 它同时接触 `AppState.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir`、`runtime_persistence`、`backtest_artifacts` 和 response projection，状态与持久化边界清楚，值得单独做等价基线。
3. 它被 `api_backtest` 的 created/list/detail/save/discard、legacy governance fallback、compare/replay downstream 代表链路覆盖。
4. 它不需要混入 `runtime.backtest.replay`、`runtime.backtest.experiment_sweep`、`backtest_compare` 或 artifact schema owner；这些仍应作为后续 sibling 单独处理。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001T handler sibling 等价基线 | 扩展 |
| 规范矩阵 | record store owner、state/persistence owner、artifact view、audit 写入、父子通信 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.record_store` | 扩展 |
| 模块树 | `runtime.backtest.record_store` | 建立单子叶基线 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.record_store` |
| 真实文件 | `src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` |
| public 方法 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`delete_transient_backtest_record`、`backtest_list_item_from_record`、`backtest_detail_response_from_record`、`sanitize_storage_path_segment`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `AppState` | `backend.app_state_wiring` | 只读取既有 `backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir`，不迁移 owner |
| 输入 | `UserId` + `backtest_id` | auth middleware、path param | detail/save/discard 必须继续使用 scoped backtest key 或安全路径段 |
| 输入 | `PaginationQuery` | `/api/runtime/backtests` | 不改变分页语义或排序语义 |
| 输出 | `PaginatedResponse<BacktestListItem>` | frontend、tests | 不改 list schema，不改变 created_at 倒序排序 |
| 输出 | `BacktestDetailResponse` | frontend、tests | 不改 governance、actor、account、events、artifact view 字段映射 |
| 输出 | 持久化 backtest artifact directory | `backtest_store_dir` | 必须继续走 `persist_backtest_record` 与 artifact bundle 写入 |
| 输出 | transient cleanup | `transient_backtest_store_dir` | save/discard 后仍调用 `delete_transient_backtest_record` |
| 输出 | graph audit entry | `audit_store_dir` | save 时仍使用 `GraphAuditAction::BacktestCreated` 和现有 actor guard |
| 输出 | discard response | frontend、tests | 已保存记录必须 conflict；仅 transient/in-memory record 可丢弃 |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.backtest::register_routes
  -> crate::runtime::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}
  -> AppState.backtests / runtime_persistence / backtest_artifacts / runtime_response_mapping / collaboration audit
  -> existing JSON response
```

本批只固定这条链路。下一步若抽离实现，也必须保留父级 `runtime` 受控出口和 `backend.runtime.routes.backtest` route facade，不允许绕过父模块新增横向调用。

---

## owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| route facade | `src/backend/runtime/routes/backtest.rs` | `GET /api/runtime/backtests`、`GET /api/runtime/backtests/:backtest_id`、`POST /api/runtime/backtests/:backtest_id/save`、`DELETE /api/runtime/backtests/:backtest_id` | 不改 path/method；真实 discard route 没有 `/discard` 后缀 |
| record handler | `src/runtime/backtest.rs` | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` | 本批不移动 |
| state lookup | `src/runtime_persistence.rs` | `load_backtest_record_from_state` | 不改 `state.backtests` -> artifact directory -> transient record 的 fallback 顺序 |
| persistence | `src/runtime_persistence.rs` | `list_backtest_records`、`persist_backtest_record`、`sanitize_storage_path_segment` | 不迁移 persistence owner |
| artifact/transient | `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、`delete_transient_backtest_record`、`load_backtest_record_from_directory`、`load_transient_backtest_record` | 不迁移 artifact schema 或 transient store owner |
| response projection | `src/runtime_response_mapping.rs` | `backtest_list_item_from_record`、`backtest_detail_response_from_record`、`normalize_backtest_record` | 不改 schema |
| audit | `src/collaboration.rs` | `persist_graph_audit_entry`、`build_graph_audit_entry`、`GraphAuditAction::BacktestCreated` | 不迁移 graph audit owner |
| frontend schema | `src/frontend_api_types.rs` | `BacktestListItem`、`BacktestDetailResponse`、`DiscardRuntimeArtifactResponse` | 不改 schema owner |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、schema、helper 调用 | 基线不破坏类型 |
| `cargo test -p quantpilot --test api_backtest` | list/detail/save/discard、legacy governance fallback、compare/replay downstream 代表链路 | backtest record store 行为不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence contract 与 replay window 下游字段 | record detail/replay 下游语义不漂移 |
| `cargo test -p quantpilot --test api_run` | runtime sibling 保护 | 本批不影响 run record store / replay status |
| `tools/check-matrix-governance.ps1` | 本基线、模块树、全量树和门禁锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |

`api_backtest` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `backtest_start_endpoint_supports_deterministic_mock_happy_path` | created/list/detail/save/discard 合同与 artifact views |
| `backtest_start_endpoint_applies_execution_assumption_overrides_to_manifest` | list filters 与 execution assumption tag |
| `backtest_replay_endpoint_exposes_paginated_ordered_timeline` | saved/current record 对 downstream replay 的代表兼容 |
| `legacy_backtest_artifacts_without_governance_load_with_safe_defaults` | 旧 artifact manifest 缺 governance 时的安全默认 |
| `backtest_compare_endpoint_reports_same_execution_assumptions` | saved/detail record 对 compare 的代表兼容 |
| `backtest_compare_endpoint_reports_different_execution_assumptions` | saved/detail record 对 compare diff 的代表兼容 |

---

## 本批次不做

- 不移动 `src/runtime/backtest.rs` 中的 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`。
- 不新建 record_store 物理文件。
- 不改 `GET /api/runtime/backtests`、`GET /api/runtime/backtests/:backtest_id`、`POST /api/runtime/backtests/:backtest_id/save` 或 `DELETE /api/runtime/backtests/:backtest_id` 的 path、method、payload、response schema 或 error code。
- 不迁移 `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir`、AppState 字段 owner、runtime state owner、lock order、persistence owner 或 artifact schema owner。
- 不拆 `runtime.backtest.replay`、`runtime.backtest.experiment_sweep`、`runtime.backtest.compare`、`runtime.backtest.execution_start`、`backend.runtime.routes.backtest`、frontend caller 或 report owner。
- 不主动提出发布版本过渡或横向连接。ASCII guard: `release transition guard`。
- 不宣称 `runtime.backtest.record_store` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.backtest.record_store` 抽离方案，即 BE-001T-02，且必须满足:

1. 只迁移 backtest list/detail/save/discard handler 边界，不混入 replay、experiment、compare 或 execution_start。
2. 保持 `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir`、artifact view、安全路径清洗和已保存记录 conflict 语义不变。
3. 保留 `api_backtest`、`api_evidence_contract` 和 `api_run` 代表测试作为等价证据。
4. 若引入新文件，必须同步模块树、全量树和治理门禁。

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
git diff --check
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 幻觉检查点

AI 声称 `runtime.backtest.record_store` 已建立基线时，必须说明本批 `no code movement`，只冻结 backtest list/detail/save/discard、transient/persistent record、artifact view、audit 和排除边界。不得宣称 handler 已迁移、record_store 物理文件已存在、replay/experiment/compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `99-runtime.backtest.record_store单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.backtest.record_store` 白箱节点。
3. 全量树能定位本基线和真实 runtime/backtest 文件。
4. 治理门禁能发现本基线文件、`no code movement`、下一候选 BE-001T-02、禁止迁移边界和回归证据缺失。
5. 后续 record store 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
