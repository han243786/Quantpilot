# v4.16.0 runtime.report_ops.merge_generation_health 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CG-02  
> 基准: `267-runtime.report_ops.merge_generation_health单子叶等价基线.md`、`266-runtime.report_ops父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.report_ops.merge_generation_health`  
> 模块树坐标: `root.backend.runtime.runtime.report_ops.merge_generation_health`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CG-02 `runtime.report_ops.merge_generation_health` 抽离方案 | 抽离方案 |
| 规范矩阵 | test-first、最小迁移清单、父级 re-export、回退点 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.merge_generation_health` | planned child 抽离路径 |
| 模块树 | `runtime.report_ops.merge_generation_health` | 方案登记 |

---

## 方案判定

本子叶选择 test-first。

原因:

1. 三条 endpoint 当前没有专门自动化 smoke。
2. `list_merge_records` 读取 run event payload，`list_config_generations` 读取 generation state，`get_storage_health` 读取多个 store dir 与 `storage_lifecycle::dir_size_bytes`；三者虽然都是 read endpoint，但状态读取面不同。
3. 先补最小 smoke 可以在实际抽离前冻结 HTTP status 与基础 JSON contract，避免物理迁移后才发现 route/schema 缺口。

因此下一步只能进入:

```text
BE-001CG-03 runtime.report_ops.merge_generation_health endpoint smoke 补测
```

BE-001CG-03 只允许新增测试，不创建 child module，不迁移 handler。

---

## 目标文件与父级出口

实际抽离批次 BE-001CG-04 才允许创建:

```text
src/runtime/report_ops/merge_generation_health.rs
```

父级 `src/runtime/report_ops.rs` 的目标形态:

```rust
mod merge_generation_health;
mod runtime_report;
mod v1_report_endpoints;

pub(crate) use merge_generation_health::{
    get_storage_health, list_config_generations, list_merge_records,
};
```

`src/runtime/mod.rs` 现有 `pub(crate) use report_ops::{...}` 清单保持不变。`src/backend/runtime/routes/report_ops.rs` route facade 保持不变。

---

## BE-001CG-03 测试计划

新增测试文件:

```text
tests/api_v1_ops_health.rs
```

覆盖范围:

| Endpoint | 断言 |
| --- | --- |
| `GET /api/v1/merge/records` | HTTP 200、`records` array、`total_conflicts` number、`total_suppressed` number |
| `GET /api/v1/runtime/generations` | HTTP 200、`current_generation` number、`history` array |
| `GET /api/v1/storage/health` | HTTP 200、`total_storage_mb` number、`layers` array、`hot_layer_usage_ratio` number、`disk_watermark_ratio` number、`archive_enabled` bool |

测试只验证最小 JSON contract，不构造真实 merge event、不改运行状态、不迁移 state owner。

---

## BE-001CG-04 允许迁移清单

只有 BE-001CG-03 通过后，BE-001CG-04 才允许迁移以下函数:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`

迁移方式:

1. 新建 `src/runtime/report_ops/merge_generation_health.rs`。
2. 在 child 文件顶部使用 `use super::*;` 保持现有父级可见性与类型访问。
3. 将三个 handler 原样迁入 child。
4. 在 `src/runtime/report_ops.rs` 增加 `mod merge_generation_health;`。
5. 在 `src/runtime/report_ops.rs` 增加受控 `pub(crate) use merge_generation_health::{...};`。
6. 不修改 route path、HTTP method、response schema、状态读取、锁顺序、storage lifecycle 调用或父级 `src/runtime/mod.rs` export 形态。

---

## 明确禁止

- BE-001CG-02 不创建 child 文件、不移动 Rust handler。
- BE-001CG-03 不创建 child 文件、不移动 Rust handler，只补测试。
- BE-001CG-04 不迁移 `MergeRecordsResponse`、`MergeRecordEntry`、run state owner、event schema owner、config generation owner、storage lifecycle owner、`AppState`、schema owner、frontend caller、runtime persistence owner。
- 不处理 `runtime.report_ops.runtime_report`。
- 不处理 `runtime.report_ops.v1_report_endpoints`。
- 不处理 `runtime.evidence_health`。
- 不启动 release transition guard。

---

## 回退点

若 BE-001CG-04 实际抽离失败，回退范围仅限:

- 删除 `src/runtime/report_ops/merge_generation_health.rs`。
- 从 `src/runtime/report_ops.rs` 移除 `mod merge_generation_health` 与对应 `pub(crate) use`。
- 将 `list_merge_records`、`list_config_generations`、`get_storage_health` 放回 `src/runtime/report_ops.rs` 原位置。

BE-001CG-03 的 endpoint smoke 可保留作为后续保护，不作为失败回退的默认删除项。

---

## 验证要求

BE-001CG-02 提交前执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_v1_reports
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CG-03 补测后必须新增并执行:

```powershell
cargo test -p quantpilot --test api_v1_ops_health
```

BE-001CG-04 实际抽离后必须同时执行:

```powershell
cargo test -p quantpilot --test api_v1_ops_health
cargo test -p quantpilot --test api_v1_reports
```

---

## 下一步

下一步只允许进入:

```text
BE-001CG-03 runtime.report_ops.merge_generation_health endpoint smoke 补测
```

---

## 幻觉检查点

AI 声称 BE-001CG-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 方案选择 test-first。
3. planned child 文件尚未创建。
4. 三个 handler 仍在 `src/runtime/report_ops.rs`。
5. 下一步只能进入 BE-001CG-03 endpoint smoke 补测。
6. BE-001CG-04 才允许创建 `src/runtime/report_ops/merge_generation_health.rs` 并迁移三个 handler。
7. `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `268-runtime.report_ops.merge_generation_health抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 治理门禁能阻止跳过 BE-001CG-03 直接实际抽离。
3. 模块树明确 BE-001CG-03 只补测试、BE-001CG-04 才允许实际迁移。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
