# v4.16.0 runtime.backtest 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001M-01。  
> 基准: `73-runtime.event_stream单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest` 单子叶等价基线，不移动代码，`no code movement`，不迁移 backtest handler、route facade、artifact schema、persistence owner、state owner、compare owner、replay helper 或 frontend caller。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 父级 runtime route sibling 队列从 `runtime.event_stream` closeout 转向 `runtime.backtest` | 推进 |
| 规范矩阵 | backtest route group、artifact views、transient spill、compare、replay、persistence 与 shared helper owner | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest` 白箱节点 | 新增基线 |
| 模块树 | `runtime.backtest` | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/backtest.rs`、`src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/runtime/mod.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| public 方法 | `start_backtest_run`、`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`、`compare_backtests` |
| 保留 shared helper | `execute_backtest_request`、`execute_v4_backtest_request`、`build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、`load_backtest_record_from_state`、`persist_backtest_record`、`list_backtest_records`、`normalized_replay_options`、`run_replay_response_from_record` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 当前白箱边界

| 项 | 当前 owner | 说明 |
| --- | --- | --- |
| route group | `src/backend/runtime/routes.rs` | 直接注册 backtest start/list/detail/save/discard/replay/compare routes |
| handler group | `src/runtime/backtest.rs` | 承载 backtest start、list、detail、save、discard、replay handler 与 v4 backtest 编排 |
| compare handler | `src/backtest_compare.rs` | `compare_backtests` 单独读取两个 backtest record 并生成 `BacktestCompareResponse` |
| artifact views | `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact bundle、transient spill 和 manifest 校验继续保留原 owner |
| persistence | `src/runtime_persistence.rs` | `persist_backtest_record`、`load_backtest_record_from_state`、`list_backtest_records` 保留 shared persistence owner |
| response mapping | `src/runtime_response_mapping.rs` | `backtest_run_response`、`backtest_detail_response_from_record`、`backtest_replay_response_from_record` 等 projection 不迁移 |
| schema | `src/frontend_api_types.rs` | `FrontendRunRequest`、`BacktestRunResponse`、`BacktestDetailResponse`、`RuntimeReplayQuery`、`RuntimeReplayResponse`、`BacktestCompareRequest`、`BacktestCompareResponse` 保留 schema owner |
| tests | `tests/api_backtest.rs`、`tests/api_evidence_contract.rs` | 覆盖 start/list/save/detail/replay/compare、artifact governance、v4 evidence contract |

---

## 真实 route 边界

| route | handler | 说明 |
| --- | --- | --- |
| `POST /api/runtime/backtest` | `start_backtest_run` | 创建 deterministic / historical / v4 backtest record 与 artifact views |
| `GET /api/runtime/backtests` | `list_backtests` | 列出已保存 backtest records |
| `POST /api/runtime/backtests/compare` | `compare_backtests` | 对两个 backtest artifacts 进行指标与假设比较 |
| `POST /api/runtime/backtests/:backtest_id/save` | `save_backtest_record` | 将 transient 或 in-memory backtest record 持久化 |
| `GET /api/runtime/backtests/:backtest_id` | `get_backtest_detail` | 读取 backtest detail 与 artifact views |
| `DELETE /api/runtime/backtests/:backtest_id` | `discard_backtest_record` | 删除 transient in-memory / transient artifact record，不删除正式保存记录 |
| `GET /api/runtime/backtests/:backtest_id/replay` | `get_backtest_replay` | 返回 backtest replay timeline，复用 `RuntimeReplayQuery` 与 replay options |

---

## 输入输出基线

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `FrontendRunRequest` | frontend、tests、本地 API caller | JSON request | backtest options、runtime kind 和 graph/source 必须保持原解析语义 |
| `BacktestCompareRequest` | frontend compare panel、tests | JSON request | 必须恰好两个 `backtest_id`，仍通过 scoped lookup |
| `RuntimeReplayQuery` | replay API query | pagination/filter query | 与 run replay 共用 options，不私有化到 backtest |
| `UserId` | auth middleware | scoped user id | 只用于 scoped backtest lookup 与保存/删除边界 |
| `AppState` | `backend.app_state_wiring` | shared app state | 不迁移 `backtests`、`backtest_store_dir`、`transient_backtest_store_dir` 或锁顺序 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestRunResponse` | frontend、tests | JSON response | 必须继续带 backtest id、output、spec、governance 与 artifact views |
| `BacktestDetailResponse` | frontend detail panel、tests | JSON response | 必须保留 artifact governance、diagnostics source 与 detail schema |
| `BacktestCompareResponse` | frontend compare panel、tests | JSON response | 必须保持左右 backtest id、metrics、equity/trade/assumption compare |
| `RuntimeReplayResponse` | replay panel、tests | JSON response | 必须保持 `kind=backtest`、record id、cursor/filter 和 event order |
| artifact bundle | filesystem、frontend artifact viewer | manifest + event log + metrics + trade ledger + equity curve | 不改变 bundle 文件、digest 或 governance rebuild 语义 |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `start_backtest_run` | `UserId`、`AppState`、`FrontendRunRequest` | `BacktestRunResponse` | `POST /api/runtime/backtest` | 不得混入 experiment/report/mutation 或 frontend state |
| `list_backtests` | `AppState`、pagination query | paginated backtest list | `GET /api/runtime/backtests` | 不得改变排序、分页或 saved-only 语义 |
| `get_backtest_detail` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `GET /api/runtime/backtests/:backtest_id` | 不得绕过 scoped lookup 或 artifact normalization |
| `save_backtest_record` | `UserId`、`AppState`、`backtest_id` | `BacktestDetailResponse` | `POST /api/runtime/backtests/:backtest_id/save` | 不得绕过 persistence/audit/governance |
| `discard_backtest_record` | `UserId`、`AppState`、`backtest_id` | discard response | `DELETE /api/runtime/backtests/:backtest_id` | 不得删除正式保存记录 |
| `get_backtest_replay` | `UserId`、`AppState`、`backtest_id`、`RuntimeReplayQuery` | `RuntimeReplayResponse` | `GET /api/runtime/backtests/:backtest_id/replay` | 不得私有化 replay query/options/schema |
| `compare_backtests` | `UserId`、`AppState`、`BacktestCompareRequest` | `BacktestCompareResponse` | `POST /api/runtime/backtests/compare` | 不得迁移 compare core/narrative owner |
| `execute_backtest_request` | state、user、request、id suffix | `BacktestRecord` | `start_backtest_run`、experiment sibling | 不得把 experiment owner 混入本叶 |
| `execute_v4_backtest_request` | state、user、request、graph、id suffix | `BacktestRecord` | v4 backtest path | 不得绕过 v4 handoff/evidence/gov owner |

---

## 明确排除

- 不迁移 `src/runtime/backtest.rs` 中的任何 handler 或 helper；本批只有基线。
- 不把 `start_backtest_experiment` 或 `/api/runtime/experiments/*` 纳入 `runtime.backtest`；它们属于后续 `runtime.report_experiment` 或 sibling。
- 不迁移 report、mutation、run routes、`runtime.event_stream`、`runtime.run.*` 或 frontend route。
- 不迁移 `src/backtest_artifacts.rs`、artifact schema、manifest、transient spill、digest 或 governance rebuild 逻辑。
- 不迁移 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 或 AppState owner。
- 不把 `normalized_replay_options`、`run_replay_response_from_record`、`backtest_replay_response_from_record` 私有化到 backtest。
- 不修改 route path、method、handler 调用、response schema、error code 或 storage semantics。
- 不主动提出发布版本过渡或横向连接。

---

## 适配性风险与暂停条件

| 风险 | 处理 |
| --- | --- |
| experiment route 被混入 backtest 基线 | 中止；`start_backtest_experiment` 与 `/api/runtime/experiments/*` 必须另起 sibling |
| compare route owner 与 backtest handler owner 混淆 | 中止；`compare_backtests` 当前由 `src/backtest_compare.rs` 持有 |
| artifact schema 或 manifest digest 被顺手改动 | 中止；artifact owner 不属于本基线迁移目标 |
| `normalized_replay_options` 被私有化 | 中止；该 helper 同时服务 run replay 与 backtest replay |
| v4 backtest helper 触碰 v4 handoff 或 evidence contract | 中止并另起方案；这会跨到 v4 runtime/evidence owner |
| state / persistence / transient store owner 需要迁移 | 中止并另起 storage/state 方案，不能混入 handler 基线 |

---

## 下一步

本基线通过后，下一批若继续，应进入 `BE-001M-02 runtime.backtest 抽离方案`。方案必须先决定第一轮最小抽离目标是 backtest route facade 还是 handler 文件切片，并说明为什么不会改变 route aggregate、compare owner、artifact owner、persistence owner、replay helper、schema owner 或 frontend caller。下一批仍不得直接移动代码，必须先完成方案。

---

## 幻觉检查点

AI 声称 `runtime.backtest` 已建立基线时，必须说明这只是 backtest route group 的等价基线；当前没有迁移 handler，没有新建 backtest 模块文件，没有改变 artifact、persistence、compare、replay、state 或 frontend。不得宣称 runtime route aggregate 全部完成，也不得把 experiment/report/mutation 说成本叶的一部分。

---

## 验收标准

1. `74-runtime.backtest单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.backtest` 白箱节点。
3. 全量树覆盖本基线文档和真实文件。
4. 治理门禁能发现本基线文档缺失。
5. `api_backtest`、`api_evidence_contract` 与 `api_run` 继续证明 backtest routes、artifact governance、v4 evidence 和 shared replay helpers 等价。
