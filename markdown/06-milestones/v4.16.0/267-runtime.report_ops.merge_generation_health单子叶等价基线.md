# v4.16.0 runtime.report_ops.merge_generation_health 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CG-01  
> 基准: `266-runtime.report_ops父叶残余判断.md`、`265-runtime.report_ops.v1_report_endpoints单叶closeout.md`、`259-runtime.report_ops.runtime_report单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.report_ops.merge_generation_health`  
> 模块树坐标: `root.backend.runtime.runtime.report_ops.merge_generation_health`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、禁止跳步、测试缺口继承、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.merge_generation_health` | 新增 planned 子叶坐标 |
| 模块树 | `runtime.report_ops.merge_generation_health` | 白箱登记 |

---

## 当前真实结构

已 closeout sibling:

- `runtime.report_ops.runtime_report stop_split: true`
- `runtime.report_ops.v1_report_endpoints stop_split: true`

本批冻结的父级残余 public handler:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`

当前真实文件:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/report_ops/v1_report_endpoints.rs
src/runtime/mod.rs
src/backend/runtime/routes/report_ops.rs
src/runtime/run.rs
src/storage_lifecycle.rs
```

planned child 文件尚未创建。BE-001CG-01 只建立等价基线，不创建 `src/runtime/report_ops/merge_generation_health.rs`，不迁移 handler。

---

## 路由与 handler 基线

| Endpoint | Method | Handler | 当前 handler 文件 | Route facade |
| --- | --- | --- | --- | --- |
| `/api/v1/merge/records` | GET | `list_merge_records` | `src/runtime/report_ops.rs` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/runtime/generations` | GET | `list_config_generations` | `src/runtime/report_ops.rs` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/storage/health` | GET | `get_storage_health` | `src/runtime/report_ops.rs` | `src/backend/runtime/routes/report_ops.rs` |

父级 export 仍由 `src/runtime/mod.rs` 通过受控 `pub(crate) use report_ops::{...}` 暴露给 route facade。

---

## 白箱边界

| public 方法 | 输入 | 状态读取 / 依赖 | 输出 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `list_merge_records` | `auth::UserId`、`State<AppState>` | `auth::scoped_key`、`state.runs.read()`、`event.source_id == "merge_engine"`、payload `input_count` / `output_count` / `conflicts` / `suppressed` / `merge_policy` | `MergeRecordsResponse` | 不得迁移 run state owner、event schema owner 或 merge event producer |
| `list_config_generations` | `State<AppState>` | `state.config_generation.load(Ordering::Relaxed)`、`state.config_generation_history.lock().await` | JSON `current_generation` / `history` | 不得迁移 config generation owner、锁顺序或 AppState 字段 owner |
| `get_storage_health` | `State<AppState>` | `run_store_dir`、`backtest_store_dir`、`report_store_dir`、`approval_store_dir`、`snapshot_store_dir`、`alert_store_dir`、`sandbox_report_store_dir`、`chaos_store_dir`、`storage_lifecycle::dir_size_bytes` | JSON `total_storage_mb` / `layers` / `hot_layer_usage_ratio` / `disk_watermark_ratio` / `archive_enabled` | 不得迁移 storage lifecycle owner、目录 owner 或归档策略 |

`MergeRecordsResponse` 与 `MergeRecordEntry` 当前仍由 `src/runtime/run.rs` 提供给同一 `runtime` 父模块范围使用。本基线不迁移 response type owner，不私造 schema，不改变字段可见性。

---

## 父级通信规则

`runtime.report_ops.merge_generation_health` 后续若实际抽离，只能经 `runtime.report_ops` 父级暴露给 `src/runtime/mod.rs`，再由 `backend.runtime.routes.report_ops` route facade 调用。

开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、run state owner、config generation owner 或 `AppState` 横向直连该子叶。

---

## 测试缺口

当前没有发现专门覆盖以下三条 endpoint 的自动化 smoke:

- `/api/v1/merge/records`
- `/api/v1/runtime/generations`
- `/api/v1/storage/health`

BE-001CG-02 抽离方案必须显式选择:

1. 先补最小 endpoint smoke，再进入实际抽离。
2. 或继承 broad regression 风险，但必须说明为什么这三条 endpoint 暂不补测。

在该选择完成前，不得直接创建 child 文件或迁移 handler。

---

## 明确排除

- 不处理 `runtime.report_ops.runtime_report`，该 child 已 closeout。
- 不处理 `runtime.report_ops.v1_report_endpoints`，该 child 已 closeout。
- 不处理 `runtime.evidence_health`。
- 不迁移 `get_runtime_evidence_health`、`cleanup_runtime_evidence` 或 `runtime_report_status_counts`。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、run state owner、event schema owner、config generation owner、storage lifecycle owner。
- 不启动 release transition guard。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_v1_reports
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CG-02 runtime.report_ops.merge_generation_health 抽离方案
```

BE-001CG-02 只能决定测试优先级、目标 child 文件、父级 re-export、允许迁移清单和回退点；不得直接宣称 handler 已迁移。

---

## 幻觉检查点

AI 声称 BE-001CG-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. planned child 文件尚未创建。
3. `list_merge_records`、`list_config_generations`、`get_storage_health` 仍在 `src/runtime/report_ops.rs`。
4. 三条 endpoint 的专门自动化 smoke 缺口已登记。
5. 下一步只能进入 BE-001CG-02 抽离方案。
6. `runtime_report`、`v1_report_endpoints`、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `267-runtime.report_ops.merge_generation_health单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.report_ops.merge_generation_health` planned 子叶白箱节点。
3. 治理门禁能阻止跳过 BE-001CG-02 直接迁移 handler。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
