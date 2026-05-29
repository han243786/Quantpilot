# v4.16.0 runtime.backtest.experiment_sweep.record_lifecycle 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001AA-02。  
> 基准: `121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`、`120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`、`119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.record_lifecycle` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001AA-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AA record_lifecycle 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级私有子模块、list/detail/save/discard 等价、禁止横向连接、测试策略 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` | 细化 |
| 模块树 | `runtime.backtest.experiment_sweep.record_lifecycle` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.record_lifecycle` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 当前 sibling 文件 | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs` |
| 计划目标文件 | `src/runtime/backtest/record_lifecycle.rs` |
| 计划迁移 handler | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 父级保留声明 | `mod parameter_grid;`、`mod start_orchestration;`、`pub(crate) use start_orchestration::start_backtest_experiment;` |
| 计划新增父级声明 | `mod record_lifecycle;` 与受控 `pub(crate) use record_lifecycle::{...};` |
| 继续保留 sibling | `parameter_grid`、`start_orchestration` |
| 继续保留 shared owner | route registration、schema、state、persistence、response mapping、audit、frontend caller、release transition guard |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 抽离目标

下一批实际抽离只允许把四个 experiment record lifecycle handler 从 `src/runtime/backtest/experiment_sweep.rs` 移入计划目标文件 `src/runtime/backtest/record_lifecycle.rs`。父级 `runtime.backtest.experiment_sweep` 继续作为白箱父节点，保留 `parameter_grid`、`start_orchestration` 和 record lifecycle 的受控 re-export。

| 方法 | 当前职责 | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `list_experiments` | experiment record list、list projection、created_at 倒序、pagination | `runtime.backtest.experiment_sweep.record_lifecycle` 私有子模块 | path/method/response、排序方向、分页顺序、response mapping owner |
| `get_experiment_detail` | scoped experiment lookup、detail response projection | `runtime.backtest.experiment_sweep.record_lifecycle` 私有子模块 | user scope、error mapping、detail response schema |
| `save_experiment_record` | variant backtest 固化、transient cleanup、experiment saved 状态、state cache、audit | `runtime.backtest.experiment_sweep.record_lifecycle` 私有子模块 | 写入顺序、audit action、failure propagation |
| `discard_experiment_record` | saved conflict、safe path cleanup、experiment cache/file 删除、transient variant cleanup | `runtime.backtest.experiment_sweep.record_lifecycle` 私有子模块 | conflict 语义、安全路径、避免误删已保存 variant |

---

## 实施方案

1. 在 BE-001AA-03 新建计划目标文件 `src/runtime/backtest/record_lifecycle.rs`。
2. 将 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 原样从 `src/runtime/backtest/experiment_sweep.rs` 移入该文件。
3. 在子文件顶部使用 `use super::*;` 复用父级上下文，避免扩大 public API。
4. 父级 `src/runtime/backtest/experiment_sweep.rs` 增加私有子模块声明:

```rust
mod record_lifecycle;

pub(crate) use record_lifecycle::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
};
```

5. 父级继续保留 `mod parameter_grid;`、`mod start_orchestration;` 与 `pub(crate) use start_orchestration::start_backtest_experiment;`。
6. 不改 `src/runtime/mod.rs` 的外部兼容 re-export 语义；route aggregate 仍通过 `crate::runtime::{list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}` 进入。
7. 不改 `src/backend/runtime/routes.rs` 或 `src/backend/runtime/routes/backtest.rs` 的 route registration。
8. 不迁移 `list_experiment_records`、`load_experiment_record_from_state`、`load_backtest_record_from_state`、`persist_backtest_record`、`persist_experiment_record`、`delete_transient_backtest_record`、`experiment_list_item_from_record`、`experiment_detail_response_from_record`、schema、AppState、audit 或 frontend caller。
9. 完成代码移动后补 BE-001AA-03 实际抽离记录，再进入 BE-001AA-04 单子叶 closeout。

---

## 必须保持的等价语义

| 行为 | 既有语义 | 不得改变 |
| --- | --- | --- |
| list read | `list_experiment_records(state.experiment_store_dir.as_ref())` | 不得改为直接读 state cache |
| list projection | `.map(experiment_list_item_from_record)` | 不得在子叶私造 list item schema |
| list order | `items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms))` | 不得改升序或先分页后排序 |
| pagination | `paginate(items, pagination)` | 不得绕过统一分页 helper |
| detail lookup | `load_experiment_record_from_state(&state, &user_id, &experiment_id)` | 不得绕过 scoped lookup |
| detail response | `experiment_detail_response_from_record(record)` | 不得私造 detail response |
| save variant loop | 每个 variant 先 load backtest record，再 persist formal backtest，再 delete transient backtest | 不得在任一 variant 失败后继续 saved write |
| save state | `record.saved = true` 后 persist experiment，再写 `state.experiments` scoped cache | 不得只写文件或只写内存 |
| save audit | actor 存在时写 `GraphAuditAction::ExperimentCreated` 和 `Saved backtest sweep ...` | 不得吞 audit 失败或改 action |
| discard conflict | saved experiment 返回 `StatusCode::CONFLICT` | 不得允许 saved experiment discard |
| discard safe path | `sanitize_storage_path_segment(&experiment_id)` | 不得恢复未清洗路径拼接 |
| discard experiment cleanup | 移除 scoped cache，存在文件则 remove_file | 不得遗漏 cache 或 file cleanup |
| transient variant detection | 正式 backtest store 不存在的 variant 才作为 transient | 不得误删已保存 variant backtest |
| transient variant cleanup | 移除 `state.backtests` scoped cache，再删除 transient file | 不得漏清 state cache 或 transient store |
| discard response | `DiscardRuntimeArtifactResponse { discarded_id: experiment_id, discarded_kind: "experiment" }` | 不得改 kind 或 id |

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `start_backtest_experiment` | 已归 `runtime.backtest.experiment_sweep.start_orchestration`，`stop_split: true` |
| `parameter_grid` helper | 已归 `runtime.backtest.experiment_sweep.parameter_grid`，`stop_split: true` |
| route registration | route owner 仍是 `backend.runtime.routes` / backtest route facade |
| `src/runtime/mod.rs` 兼容出口语义 | 只允许保持当前 `crate::runtime::*` 行为，不扩大公开面 |
| drained parent include | `src/runtime/backtest.rs` 保持 drained parent include 事实 |
| persistence owner | persistence helper 继续归 `src/runtime_persistence.rs` |
| transient helper owner | `delete_transient_backtest_record` 继续归 `src/backtest_artifacts.rs` |
| response mapping owner | list/detail response projection 继续归 `src/runtime_response_mapping.rs` |
| schema owner | response/request/record schema 继续归 `src/frontend_api_types.rs` |
| AppState / lock owner | state、store dir 和 scoped cache owner 不迁移 |
| audit owner | 只调用既有 graph audit helper，不私有化 audit owner |
| frontend caller | 不改 API path、payload、response schema 或 caller |
| 发布过渡 | 不主动提出横向连接或性能旁路。ASCII guard: `release transition guard` |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| 子模块可见性失败 | 先用 `use super::*;`，只在必要时补显式 import，不新增 public API |
| handler re-export 重名 | 父级先移除本地四个函数，再 `pub(crate) use record_lifecycle::{...};` |
| persistence helper 可见性失败 | 不改变 helper owner；若需要扩大可见性，暂停并回到方案讨论 |
| response mapping 可见性失败 | 不复制 mapping 逻辑；只能维持既有 owner 调用 |
| state lock 顺序漂移 | 移动时不改 `state.experiments`、`state.backtests` 锁获取位置 |
| transient cleanup 漂移 | 保留正式 store 存在性判断，不改为直接删除所有 variant |
| route aggregate 被误迁移 | BE-001AA-03 不改 route 文件，测试只证明调用路径等价 |
| 发布过渡旁路被提出 | 未收到开发者明确发布过渡指令时直接拒绝进入该路径 |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 experiment API path、method、payload、response schema 或 error code。
2. 需要迁移 route registration、schema、state、persistence、response mapping、audit 或 frontend caller。
3. 需要移动 `start_backtest_experiment`、`parameter_grid` 或 `execution_start`。
4. 需要改变 saved conflict、created_at 倒序、pagination、scoped lookup、variant persistence、transient cleanup、state cache 或 audit 语义。
5. 需要把 persistence/response mapping helper 扩大到新的 public API。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过私有子模块 import 解决。
7. 代表测试出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 BE-001AA-03 `runtime.backtest.experiment_sweep.record_lifecycle` 实际抽离记录: 按本方案只移动四个 handler 到 planned record_lifecycle child file，保留父级私有 re-export、parameter_grid、start_orchestration、route aggregate、schema、state、persistence、response mapping、audit、frontend caller 和发布过渡边界。完成后再做 BE-001AA-04 单子叶 closeout，判断 `record_lifecycle` 是否设置 `stop_split: true`，并决定 `runtime.backtest.experiment_sweep` 父叶是否还存在值得继续细拆的残余。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已有抽离方案时，必须说明本批 `no code movement`，只是确认 BE-001AA-03 的移动边界。不得宣称 record lifecycle 已抽离、planned record_lifecycle child file 已存在、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.experiment_sweep.record_lifecycle` 节点标记抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、当前真实文件、计划目标文件和下一步 BE-001AA-03。
4. 治理门禁能发现本方案、`no code movement`、四个目标 handler、`use super::*`、planned record_lifecycle child file、排除边界、发布过渡保护和回归证据。
5. 后续 BE-001AA-03 实际抽离必须引用本方案，不得把 route、start_orchestration、parameter_grid、execution_start、persistence、mapping、schema、state、audit 或 frontend caller 混入第一轮迁移。
