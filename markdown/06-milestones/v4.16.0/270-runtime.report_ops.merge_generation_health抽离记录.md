# v4.16.0 runtime.report_ops.merge_generation_health 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CG-04  
> 基准: `269-runtime.report_ops.merge_generation_health补测记录.md`、`268-runtime.report_ops.merge_generation_health抽离方案.md`、`267-runtime.report_ops.merge_generation_health单子叶等价基线.md`  
> 判定: 已按 test-first 方案创建 child module 并迁移三条 v1 support/health endpoint handler。route facade 与 `src/runtime/mod.rs` 保持不变；下一步只能进入 BE-001CG-05 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CG-04 `runtime.report_ops.merge_generation_health` 实际抽离 | 实际抽离 |
| 规范矩阵 | test-first、父子通信、受控 re-export | 继承 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.merge_generation_health` | child module 落位 |
| 模块树 | `src/runtime/report_ops/merge_generation_health.rs` | 新增真实 child 文件 |

---

## 实际变更

新增 child module:

```text
src/runtime/report_ops/merge_generation_health.rs
```

迁入 handler:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`

继承 smoke 测试:

```text
tests/api_v1_ops_health.rs
```

父级 `src/runtime/report_ops.rs` 只新增受控出口:

```rust
mod merge_generation_health;

pub(crate) use merge_generation_health::{
    get_storage_health, list_config_generations, list_merge_records,
};
```

---

## 保持不变

- `src/runtime/mod.rs` 未改变。
- `src/backend/runtime/routes/report_ops.rs` 未改变。
- `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health` 路由未改变。
- `MergeRecordsResponse`、`MergeRecordEntry` 仍保持原 owner。
- run state owner、event schema owner、config generation owner、storage lifecycle owner 未迁移。
- `runtime.report_ops.runtime_report` 与 `runtime.report_ops.v1_report_endpoints` 未处理。
- `runtime.evidence_health` 未处理。
- `AppState`、schema owner、frontend caller、runtime persistence owner 未迁移。
- release transition guard 未启动。

---

## 验证结果

已执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_v1_ops_health
cargo test -p quantpilot --test api_v1_reports
```

结果:

```text
passed
```

本批提交前仍需执行:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CG-05 runtime.report_ops.merge_generation_health 单叶 closeout
```

closeout 需要判断 merge records、runtime generations 与 storage health projection 三个 handler 是否还有继续细拆价值，并决定 `runtime.report_ops.merge_generation_health stop_split`。不得处理 `runtime.evidence_health`，不得修改 schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CG-04 完成时，必须说明:

1. `src/runtime/report_ops/merge_generation_health.rs` 已创建。
2. `list_merge_records`、`list_config_generations`、`get_storage_health` 已从 `src/runtime/report_ops.rs` 迁入 child。
3. 父级仅新增 `mod merge_generation_health` 与受控 `pub(crate) use merge_generation_health::{...}`。
4. `src/runtime/mod.rs` 和 route facade 未改变。
5. 下一步只能进入 BE-001CG-05 单叶 closeout。
6. `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `270-runtime.report_ops.merge_generation_health抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/report_ops/merge_generation_health.rs` 进入模块树与全量树 active file coverage。
3. `cargo test -p quantpilot --test api_v1_ops_health` 与 `cargo test -p quantpilot --test api_v1_reports` 通过。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
5. 下一步固定为 BE-001CG-05 单叶 closeout。
