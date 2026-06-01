# v4.16.0 backend.graph_compile.quantscript_graph.graph_to_qs_generation 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FR-02
> 基线: `479-backend.graph_compile.quantscript_graph.graph_to_qs_generation单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 代码动作: no code movement
> 下一步: BE-001FR-03 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FR-02 `graph_to_qs_generation` 抽离方案 | 子叶抽离方案 |
| 规范矩阵 | planned child / visibility / parent communication rule / rollback point / test gates | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation` | 固定实际抽离动作 |
| 模块树 | `backend.graph_compile.quantscript_graph.graph_to_qs_generation` | planned child file |

---

## planned child

BE-001FR-03 只允许创建:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
```

父级文件保持:

```text
src/backend/graph_compile/quantscript_graph.rs
```

父级声明固定为:

```rust
mod graph_to_qs_generation;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;
```

如果 `build_quantscript_node_sources` 继续留在父级，则父级只能通过:

```rust
graph_to_qs_generation::generate_node_quantscript
```

调用 child 内部 helper；该 helper 在 child 中只能设为 `pub(super)`，不得提升为 `pub(crate)` 或 root re-export。

---

## 允许迁移清单

BE-001FR-03 只允许迁移以下函数:

```text
generate_quantscript_from_graph_value
generate_node_quantscript
quoted
render_json_scalar
```

其中:

| 函数 | 目标可见性 | 说明 |
| --- | --- | --- |
| `generate_quantscript_from_graph_value` | `pub(crate)` | 保持 compile / graph / test caller 经 root parent re-export surface 调用 |
| `generate_node_quantscript` | `pub(super)` | 只服务父级 artifact projection 内部复用 |
| `quoted` | private | 只属于 generator child |
| `render_json_scalar` | private | 只属于 generator child |

---

## 禁止迁移清单

BE-001FR-03 不得迁移:

```text
register_routes
load_graph_quantscript
parse_graph_quantscript
convert_graph_json_to_script_module
attach_quantscript_artifacts
build_quantscript_node_sources
build_quantscript_label_targets
build_quantscript_runtime_targets
build_compile_runtime_targets_from_graph
parse_graph_quantscript_source
```

这些簇分别属于 route surface、formal module conversion、artifact target projection、runtime target projection 和 strategy_graph parser，必须另起基线。

---

## import 与通信规则

child file 只允许引入:

```rust
use serde_json::Value;
```

不得引入 `crate::*`、不得引入 parent wildcard、不得引入 axum、tokio、quantscript parser 或 AppState。

父级只能保留必要声明/调用:

```rust
mod graph_to_qs_generation;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;
graph_to_qs_generation::generate_node_quantscript(...)
```

不得新增 compile / graph / runtime sibling horizontal link，也不得改 `src/lib.rs` root parent re-export surface。

本批和 BE-001FR-03 均不得启动 release transition guard。

---

## 等价保护点

BE-001FR-03 必须保持:

1. graph metadata 输出: `graph_id`、`name`、`version`、`mode`。
2. node kind 输出: `runtime`、`execution`、默认 `plugin`。
3. module fallback: `module_key` -> `type` -> `unknown.module`。
4. config scalar rendering: string/number/bool/null/object/array fallback 不变。
5. input edge rendering: `from` / `to` block 不变。
6. graph edge rendering: `connect {source}.{source_port} -> {target}.{target_port}`。
7. empty graph edge marker: `# no connections`。
8. `build_quantscript_node_sources` 的 node source 生成仍复用同一个 node renderer。

---

## 回退点

如果 BE-001FR-03 编译失败或测试暴露语义漂移，只回退本次 planned child:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
src/backend/graph_compile/quantscript_graph.rs
```

不得回退 BE-001FQ-03 已完成的 `src/graph_quantscript_api.rs` 删除，也不得动 compile / graph / runtime sibling。

---

## 验证要求

本批是 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

BE-001FR-03 实际抽离必须执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FR-02 完成时，必须说明:

1. 当前只是 `no code movement` 抽离方案。
2. planned child 固定为 `src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs`。
3. 下一步 BE-001FR-03 只允许迁移四个 generator helper。
4. `generate_node_quantscript` 只能作为 `pub(super)` 给父级内部复用。
5. 不得宣称 parser、formal conversion、artifact projection、route surface 或 `backend.graph_compile` 已收口。

---

## 验收标准

1. `480-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `graph_to_qs_generation plan_frozen` 已记录。
3. planned child、父级声明、允许迁移清单、禁止迁移清单、回退点和测试门禁已固定。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
