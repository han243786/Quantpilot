# v4.16.0 runtime.backtest.replay 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001U-01。  
> 前置: `102-runtime.backtest.record_store单叶closeout.md`、`77-runtime.backtest单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `runtime.backtest.replay` 单子叶等价基线；本批只冻结 backtest replay route、record lookup、query normalization、response mapping、metrics、父子通信和回归证据，不移动代码。  
> 代码动作: `no code movement`。

---

## 选择理由

`runtime.backtest.record_store` 已完成单叶 closeout 并设置 `stop_split: true`，下一步必须回到 `runtime.backtest` sibling 队列。`runtime.backtest.replay` 是当前最适合承接的下一片:

1. 它覆盖 `GET /api/runtime/backtests/:backtest_id/replay` 与 `get_backtest_replay`，是 backtest 记录读侧的 replay window 投影。
2. 它只读取 backtest record 并调用 replay response mapping，边界比 experiment sweep 更窄，适合作为下一轮低风险基线。
3. 它与 `runtime.backtest.record_store` 下游相关，但不拥有保存、丢弃、artifact 写入或审计 owner，必须作为 sibling 单独处理。
4. 它被 `api_backtest` 的 `backtest_replay_endpoint_exposes_paginated_ordered_timeline` 覆盖，且通过 `api_evidence_contract` 保护 evidence metrics 和 replay contract。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 backtest handler sibling 队列从 `runtime.backtest.record_store` 转向 `runtime.backtest.replay` | 推进 |
| 规范矩阵 | replay route、父级出口、record lookup、response mapping owner、metrics owner、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.replay` | 新增基线 |
| 模块树 | `runtime.backtest.replay` | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.replay` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.replay` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `get_backtest_replay`、`load_backtest_record_from_state`、`normalized_replay_options`、`backtest_replay_response_from_record`、`runtime_replay_response`、`filtered_replay_events`、`cursor_from_replay_options`、`replay_event_items`、`replay_checkpoints`、`timeline_items_from_events`、`record_replay_page` |
| public 类型 | `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` |
| 代表测试 | `tests/api_backtest.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `UserId` + `backtest_id` | auth middleware、path param | 必须继续走 scoped backtest lookup，不得绕过用户边界 |
| 输入 | `RuntimeReplayQuery` | query string | 支持 `cursor`、`checkpoint`、`sequence_cursor`、`limit`、`stage`、`severity`、`retention_class`、`module_key`、`key_only` |
| 输入 | `AppState` | backend app state | 只读取 backtest record 与 evidence metrics，不迁移 AppState owner |
| 输出 | `RuntimeReplayResponse` | frontend、tests | 不改 `kind`、`record_id`、`graph_id`、cursor、timeline、events、filters、checkpoints 或 account schema |
| 输出 | bad cursor error | API caller | `backtest_replay_response_from_record` 错误仍映射为 `json_bad_request("bad_replay_cursor", message)` |
| 输出 | evidence metric | `RuntimeEvidenceMetrics` | replay 成功后继续调用 `record_replay_page` |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.backtest::register_routes
  -> GET /api/runtime/backtests/:backtest_id/replay
  -> crate::runtime::get_backtest_replay
  -> load_backtest_record_from_state
  -> normalized_replay_options
  -> backtest_replay_response_from_record
  -> RuntimeReplayResponse
```

本批只固定这条链路。后续若抽离实现，也必须保留父级 `runtime` 受控出口和 `backend.runtime.routes.backtest` route facade，不允许绕过父模块新增横向调用。

---

## owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| route facade | `src/backend/runtime/routes/backtest.rs` | `GET /api/runtime/backtests/:backtest_id/replay` | 不改 path/method/handler 调用名 |
| replay handler | `src/runtime/backtest.rs` | `get_backtest_replay` | 本批不移动 |
| record lookup | `src/runtime_persistence.rs` | `load_backtest_record_from_state` | 不改 memory -> artifact directory -> transient fallback 顺序 |
| query normalization | `src/runtime/mod.rs` | `RuntimeReplayQuery`、`normalized_replay_options` | 不改 default limit、max limit、checkpoint/cursor 优先级或 filter cleanup |
| response mapping | `src/runtime_response_mapping.rs` | `backtest_replay_response_from_record`、`runtime_replay_response` | 不迁移 response mapping owner |
| replay event source | `src/runtime_response_mapping.rs` + `src/backtest_artifacts.rs` | `backtest_artifacts.event_log.events` fallback `record.events` | 不改 artifact event log 优先级 |
| replay cursor/filter | `src/runtime_response_mapping.rs` | `filtered_replay_events`、`cursor_from_replay_options`、`replay_event_items`、`replay_checkpoints` | 不改排序、过滤、bad cursor 或 checkpoint 语义 |
| API schema | `src/frontend_api_types.rs` | `RuntimeReplayResponse`、`RuntimeReplayFilters`、`RuntimeReplayRecordKind::Backtest` | 不改 schema owner |
| metrics | `src/lib.rs` | `RuntimeEvidenceMetrics::record_replay_page` | 不迁移 metrics owner |

---

## 等价冻结项

| 行为 | 当前语义 | 证据 |
| --- | --- | --- |
| route 入口 | `GET /api/runtime/backtests/:backtest_id/replay` 进入 `runtime_handlers::get_backtest_replay` | `src/backend/runtime/routes/backtest.rs` |
| record 读取 | handler 经 `load_backtest_record_from_state` 读取用户作用域 record | `src/runtime/backtest.rs`、`src/runtime_persistence.rs` |
| query normalization | `checkpoint` 优先于 `cursor`，`limit` clamp 到 1..50，空 filter 被清理 | `src/runtime/mod.rs` |
| replay source | 有 artifact event log 时优先用 `backtest_artifacts.event_log.events`，否则 fallback 到 `record.events` | `src/runtime_response_mapping.rs` |
| event order | replay response 按 `sequence_no` 排序，缺失时使用 fallback sequence | `filtered_replay_events`、`event_sequence_no` |
| filters | 支持 `stage`、`severity`、`retention_class`、`module_key`、`key_only` | `event_matches_replay_filters` |
| cursor error | 越界 `sequence_cursor` 或 `cursor` 返回 `bad_replay_cursor` | `cursor_from_replay_options`、`get_backtest_replay` |
| response schema | `kind` 为 `backtest`，保留 timeline、events、checkpoints、next/previous cursor | `RuntimeReplayResponse` |
| metrics | 成功 replay page 记录 latency | `record_replay_page` |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、route facade、schema 类型 | 基线不破坏类型 |
| `cargo test --no-run` | 全测试编译 | 不引入测试编译漂移 |
| `cargo test -p quantpilot --test api_backtest` | backtest replay、record detail、saved/current artifact 下游代表链路 | replay 行为不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence metrics 与 replay contract | metrics/contract 不漂移 |
| `cargo test -p quantpilot --test api_run` | runtime replay sibling 保护 | 本批不影响 run replay/status |
| `tools\check-matrix-governance.ps1` | 本基线、模块树、全量树和门禁锚点 | 治理入口不丢 |
| `tools\check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |
| `tools\check-utf8.ps1` | 文档编码 | 新文档保持 UTF-8 |
| `git diff --check` | whitespace | 无尾随空白 |

`api_backtest` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `backtest_replay_endpoint_exposes_paginated_ordered_timeline` | replay route、limit、cursor、sequence_cursor、timeline/event 对齐、retention filter 和 bad cursor |
| `backtest_start_endpoint_supports_deterministic_mock_happy_path` | created/detail record 对 replay source 的上游兼容 |
| `legacy_backtest_artifacts_without_governance_load_with_safe_defaults` | 旧 artifact record 被 lookup 后仍可安全投影 |
| `runtime_report_records_link_backtest_evidence_metadata` | replay/evidence 下游 contract 仍可连到 backtest evidence |

---

## 本批次不做

- 不移动 `src/runtime/backtest.rs` 中的 `get_backtest_replay`。
- 不新建 `src/runtime/backtest/replay.rs`。
- 不改 `GET /api/runtime/backtests/:backtest_id/replay` 的 path、method、payload、response schema 或 error code。
- 不迁移 `runtime.backtest.record_store`，即 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.experiment_sweep`、`backtest_compare`、report owner 或 frontend caller。
- 不迁移 `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint` schema owner。
- 不迁移 `backtest_replay_response_from_record`、`runtime_replay_response`、`filtered_replay_events`、`cursor_from_replay_options`、`timeline_items_from_events` 或 response mapping owner。
- 不迁移 `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、AppState owner、runtime state owner、persistence owner 或 artifact schema owner。
- 不主动提出发布版本过渡或横向连接。ASCII guard: `release transition guard`。
- 不宣称 `runtime.backtest.replay` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.backtest.replay` 抽离方案，即 BE-001U-02，且必须满足:

1. 只讨论是否把 `get_backtest_replay` 迁入计划目标文件，例如 `src/runtime/backtest/replay.rs`。
2. 保持 `backend.runtime.routes.backtest` route facade 和 `crate::runtime::get_backtest_replay` 兼容出口不变。
3. 保持 record lookup、query normalization、response mapping、schema、metrics 和 artifact event log owner 原位。
4. 保留 `api_backtest`、`api_evidence_contract` 和 `api_run` 代表测试作为等价证据。
5. 若引入新文件，必须同步模块树、全量树和治理门禁。

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

AI 声称 `runtime.backtest.replay` 已建立基线时，必须说明本批 `no code movement`，只冻结 backtest replay route、record lookup、query normalization、response mapping、metrics 和排除边界。不得宣称 handler 已迁移、`src/runtime/backtest/replay.rs` 已存在、record_store、execution_start、experiment、compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `103-runtime.backtest.replay单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.backtest.replay` 白箱节点。
3. 全量树能定位本基线和真实 runtime/backtest 文件。
4. 治理门禁能发现本基线文件、`no code movement`、下一候选 BE-001U-02、禁止迁移边界和回归证据缺失。
5. 后续 replay 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
