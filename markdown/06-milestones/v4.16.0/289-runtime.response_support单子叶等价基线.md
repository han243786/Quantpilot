# v4.16.0 runtime.response_support 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CP-01  
> 基准: `288-backend.runtime第五轮父叶残余判断.md`、`287-runtime.query_support单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.response_support`  
> 模块树坐标: `root.backend.runtime.runtime.response_support`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CP-01 `runtime.response_support` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、response DTO visibility、禁止横向连接、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.response_support` | 新增 planned 子叶坐标 |
| 模块树 | `runtime.response_support` | 白箱登记 |

---

## 当前真实结构

已 closeout sibling / 父级:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`
- `runtime.evidence_health stop_split: true`
- `runtime.backtest stop_split: true`
- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`
- `runtime.mutation.shared_governance stop_split: true`
- `runtime.query_support stop_split: true`
- `backend.runtime stop_split: false`

本批冻结的 response DTO 残余仍在:

```text
src/runtime/mod.rs
src/runtime/run.rs
```

planned child 文件尚未创建。BE-001CP-01 只建立等价基线，不创建 `src/runtime/response_support.rs`，不迁移 response DTO。

---

## 白箱边界

| public / helper | 当前文件 | 输入 | 输出 / 调用意义 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | `src/runtime/mod.rs` | run/backtest discard handler 的 artifact kind 与 id | JSON discard response | 不得改变 `discarded_kind` / `discarded_id` 字段名、类型或响应语义 |
| `MergeRecordsResponse` | `src/runtime/run.rs` | merge records endpoint 收集到的 record entries 与 totals | JSON merge record list response | 不得改变 `records`、`total_conflicts`、`total_suppressed` 字段名、类型或计数语义 |
| `MergeRecordEntry` | `src/runtime/run.rs` | merge record file stem、status、count 与 path | `MergeRecordsResponse.records` item | 不得改变 record entry 字段名、路径表达或冲突/抑制计数语义 |

后续若进入实际抽离，`DiscardRuntimeArtifactResponse` 与 `MergeRecordsResponse` 作为 `pub(crate)` handler signature 的返回类型，类型本体必须保持 `pub(crate)`。构造字段优先使用 `pub(super)`，只向 `src/runtime/*` 父级子模块开放，不升级成外部 public API。`MergeRecordEntry` 优先保持 `pub(super)`；若编译器因 public-ish field exposure 要求更宽 visibility，必须在抽离记录中显式说明。

---

## 调用方基线

| 调用方文件 | 当前依赖 | 禁止事项 |
| --- | --- | --- |
| `src/runtime/run/record_store.rs` | `DiscardRuntimeArtifactResponse` | 不得改变 `DELETE /api/runtime/runs/:run_id` discard response contract |
| `src/runtime/backtest/record_store.rs` | `DiscardRuntimeArtifactResponse` | 不得改变 backtest record discard response contract |
| `src/runtime/backtest/record_lifecycle.rs` | `DiscardRuntimeArtifactResponse` | 不得改变 experiment/backtest lifecycle discard response contract |
| `src/runtime/report_ops/merge_generation_health.rs` | `MergeRecordsResponse`、`MergeRecordEntry` | 不得改变 `/api/v1/merge/records` response contract |

---

## 现有等价证据

当前已有自动化覆盖:

- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_v1_ops_health.rs`

BE-001CP-02 抽离方案不需要先补新 endpoint smoke，但必须把 `api_run`、`api_backtest` 与 `api_v1_reports` / `api_v1_ops_health` 作为实际抽离前后的硬门禁。为保持 runtime 父级回归面，本批仍沿用 `api_mutation` 与 `api_ai_proposal` 作为父级非目标旁路保护。

---

## 父子通信规则

`runtime.response_support` 后续若实际抽离，只能作为 `backend.runtime` 下的 response DTO child。通信路径必须保持父级中介:

```text
runtime child callers
  -> src/runtime/mod.rs controlled response surface
  -> runtime.response_support
```

父级可在后续方案中增加 `mod response_support;` 与普通 `use response_support::{...};`，不得使用 `pub(crate) use response_support::{...};` 制造新的对外 surface。开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连本 planned child。

---

## 明确排除

- 不创建 `src/runtime/response_support.rs`。
- 不迁移 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 或 `MergeRecordEntry`。
- 不迁移 `RunInProgressGuard`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 response schema、route facade、runtime persistence owner、storage lifecycle owner、frontend caller、`AppState`、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_v1_ops_health
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CP-02 runtime.response_support 抽离方案
```

BE-001CP-02 只能决定 planned child 文件、父级声明、controlled import、允许迁移清单、visibility、验证命令和回退点；不得直接宣称 response DTO 已迁移。

---

## 幻觉检查点

AI 声称 BE-001CP-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. planned child 文件 `src/runtime/response_support.rs` 尚未创建。
3. `DiscardRuntimeArtifactResponse` 仍在 `src/runtime/mod.rs`，`MergeRecordsResponse` 与 `MergeRecordEntry` 仍在 `src/runtime/run.rs`。
4. 后续实际迁移必须让 response DTO 类型本体保持 handler signature 所需 visibility，字段优先收敛为 `pub(super)`。
5. 下一步只能进入 BE-001CP-02 抽离方案。
6. run guard、experiment limit、parent include 删除、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 和 release transition guard 均未迁移。

不得宣称 response support 已抽离、`backend.runtime` 已完成、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `289-runtime.response_support单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.response_support` planned 子叶白箱坐标，但不登记不存在的真实文件路径。
3. 治理门禁能阻止跳过 BE-001CP-02 直接创建 child 文件或迁移 response DTO。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
