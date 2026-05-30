# v4.16.0 runtime.report_ops.merge_generation_health 补测记录

> 版本类型: MINOR architecture / governance / test  
> 执行档位: 标准  
> 批次: BE-001CG-03  
> 基准: `268-runtime.report_ops.merge_generation_health抽离方案.md`、`267-runtime.report_ops.merge_generation_health单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.report_ops.merge_generation_health`  
> 模块树坐标: `root.backend.runtime.runtime.report_ops.merge_generation_health`  
> 代码动作: endpoint smoke test only

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CG-03 `runtime.report_ops.merge_generation_health` endpoint smoke 补测 | 补测 |
| 规范矩阵 | test-first 证据、实际抽离前置门禁 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.merge_generation_health` | 测试证据补齐 |
| 模块树 | `runtime.report_ops.merge_generation_health` | 自动化 smoke 登记 |

---

## 本批变更

新增:

```text
tests/api_v1_ops_health.rs
```

覆盖:

| Endpoint | Handler | 最小断言 |
| --- | --- | --- |
| `/api/v1/merge/records` | `list_merge_records` | HTTP 200、`records` array、`total_conflicts` number、`total_suppressed` number |
| `/api/v1/runtime/generations` | `list_config_generations` | HTTP 200、`current_generation` number、`history` array |
| `/api/v1/storage/health` | `get_storage_health` | HTTP 200、`total_storage_mb` number、`layers` array、`hot_layer_usage_ratio` number、`disk_watermark_ratio` number、`archive_enabled` bool，且包含 `runs` layer |

本批不创建 child module，不迁移 handler，不修改 route facade，不改变 state owner。

---

## 等价意义

BE-001CG-03 只把 BE-001CG-01 登记的测试缺口补成自动化 smoke。它证明三条 endpoint 在当前结构下能返回最小 JSON contract，为 BE-001CG-04 实际抽离提供迁移前基线。

当前真实状态仍是:

```text
list_merge_records -> src/runtime/report_ops.rs
list_config_generations -> src/runtime/report_ops.rs
get_storage_health -> src/runtime/report_ops.rs
```

planned child 文件尚未创建。

---

## 明确排除

- 不创建 `src/runtime/report_ops/merge_generation_health.rs`。
- 不迁移 `list_merge_records`、`list_config_generations`、`get_storage_health`。
- 不迁移 `MergeRecordsResponse`、`MergeRecordEntry`、run state owner、event schema owner、config generation owner、storage lifecycle owner、`AppState`、schema owner、frontend caller、runtime persistence owner。
- 不处理 `runtime.report_ops.runtime_report`。
- 不处理 `runtime.report_ops.v1_report_endpoints`。
- 不处理 `runtime.evidence_health`。
- 不启动 release transition guard。

---

## 验证要求

提交前执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_v1_ops_health
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
BE-001CG-04 runtime.report_ops.merge_generation_health 实际抽离
```

BE-001CG-04 才允许创建 `src/runtime/report_ops/merge_generation_health.rs` 并迁移三个 handler。

---

## 幻觉检查点

AI 声称 BE-001CG-03 完成时，必须说明:

1. 本批次只新增 endpoint smoke 测试。
2. `tests/api_v1_ops_health.rs` 已覆盖三条 v1 support/health endpoint 的基础 JSON contract。
3. planned child 文件尚未创建。
4. 三个 handler 仍在 `src/runtime/report_ops.rs`。
5. 下一步只能进入 BE-001CG-04 实际抽离。
6. `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `269-runtime.report_ops.merge_generation_health补测记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `tests/api_v1_ops_health.rs` 进入模块树与全量树。
3. `cargo test -p quantpilot --test api_v1_ops_health` 通过。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
