# v4.16.0 backend.graph_compile.quantscript_graph.graph_to_qs_generation 抽离记录
> 版本类型: MINOR architecture / implementation
> 执行档位: 标准
> 批次: BE-001FR-03
> 基线: `480-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离方案.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 判定: 实际抽离完成
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 代码动作: actual extraction
> 下一步: BE-001FR-04 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FR-03 `graph_to_qs_generation` 实际抽离记录 | 子叶实际抽离 |
| 规范矩阵 | actual extraction / parent re-export / pub(super) helper / no sibling horizontal link / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation` | child file 落位 |
| 模块树 | `backend.graph_compile.quantscript_graph.graph_to_qs_generation` | generator owner 迁移 |

---

## 实际迁移动作

本批创建:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
```

并从父级迁入:

```text
generate_quantscript_from_graph_value
generate_node_quantscript
quoted
render_json_scalar
```

抽离后:

```text
graph_to_qs_generation_file_created
graph_to_qs_generation_actual_extraction_done
```

---

## 父级接线

父级 `src/backend/graph_compile/quantscript_graph.rs` 新增受控声明:

```rust
mod graph_to_qs_generation;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;
```

父级 artifact projection 内部复用改为:

```rust
graph_to_qs_generation::generate_node_quantscript(node, &nodes, &edges)
```

`generate_node_quantscript` 在 child 中保持:

```rust
pub(super) fn generate_node_quantscript(...)
```

因此 compile / graph / runtime caller 仍经 root parent re-export surface，artifact projection 仍由父级内部受控调用 child helper。

---

## 未迁移边界

本批未迁移:

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

route surface、formal module conversion、artifact target projection、runtime target projection 和 strategy_graph parser 仍留在父叶或后续 sibling 子叶队列。

---

## 等价不变量

本批保持:

1. `graph_id`、`name`、`version`、`mode` metadata 输出不变。
2. node kind `runtime` / `execution` / `plugin` 分支不变。
3. module fallback `module_key` -> `type` -> `unknown.module` 不变。
4. config scalar rendering 与 `quoted` escaping 不变。
5. input edge `from` / `to` block 不变。
6. graph edge `connect {source}.{source_port} -> {target}.{target_port}` 不变。
7. empty graph edge marker `# no connections` 不变。
8. `build_quantscript_node_sources` 仍复用同一个 node renderer。

---

## 不进入范围

本批不处理:

1. 不改 route handler。
2. 不改 formal conversion。
3. 不改 artifact target projection 除受控 helper 调用路径。
4. 不改 parser。
5. 不改 `src/lib.rs` root parent re-export surface。
6. 不新增 sibling horizontal link。
7. 不启动 release transition guard。

---

## 下一步边界

下一步只能进入:

```text
BE-001FR-04
backend.graph_compile.quantscript_graph.graph_to_qs_generation
root.backend.graph_compile.quantscript_graph.graph_to_qs_generation
```

BE-001FR-04 只允许做单叶 closeout 和是否继续细拆判断；不得跳到 parser、formal conversion、artifact projection 或 `backend.graph_compile` 父叶收口。

---

## 验证要求

本批提交前至少执行:

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

AI 声称 BE-001FR-03 完成时，必须说明:

1. `src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs` 已创建。
2. 四个 generator helper 已迁入 child。
3. `generate_node_quantscript` 只是 `pub(super)`，用于父级 `build_quantscript_node_sources` 内部复用。
4. route surface、formal conversion、artifact projection 和 parser 没有迁移。
5. 不得宣称 `backend.graph_compile.quantscript_graph stop_split: true` 或 `backend.graph_compile` 已收口。

---

## 验收标准

1. `481-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `graph_to_qs_generation_actual_extraction_done` 已记录。
3. child file 已进入全量树覆盖。
4. 父级受控 re-export 与 `pub(super)` helper 通信成立。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、QS 窄测试和 `git diff --check` 均通过。
