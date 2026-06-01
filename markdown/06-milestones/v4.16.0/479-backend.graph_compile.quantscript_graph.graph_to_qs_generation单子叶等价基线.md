# v4.16.0 backend.graph_compile.quantscript_graph.graph_to_qs_generation 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FR-01
> 基线: `478-backend.graph_compile.quantscript_graph单叶closeout.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 判定: 等价基线
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 代码动作: no code movement
> 下一步: BE-001FR-02 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FR-01 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线 | 子叶基线 |
| 规范矩阵 | equivalence baseline / generator invariants / parent communication rule / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation` | 新增下一层白箱节点 |
| 模块树 | `backend.graph_compile.quantscript_graph.graph_to_qs_generation` | graph-to-QS generation 边界冻结 |

---

## 当前真实 owner

本批只冻结当前实现，不移动代码:

```text
src/backend/graph_compile/quantscript_graph.rs
```

当前 generator 仍由父叶 `backend.graph_compile.quantscript_graph` 持有，下一步才允许规划 child file。BE-001FR-01 不创建:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
```

---

## 白箱 public/helper 节点

本子叶冻结以下 public/helper surface:

| 节点 | 当前可见性 | 输入 | 输出 | 约束 |
| --- | --- | --- | --- | --- |
| `generate_quantscript_from_graph_value` | `pub(crate)` | graph `Value` | strategy_graph QuantScript source | 不得改 metadata、nodes、edges、connect 输出 |
| `generate_node_quantscript` | private | node、nodes、edges | 单节点 QuantScript block | 不得改 module fallback、config rendering 或 inputs 映射 |
| `quoted` | private | string | JSON quoted string | 不得改字符串 escaping 语义 |
| `render_json_scalar` | private | JSON scalar | QS scalar text | 不得把 object/array 静默降级为业务语义 |

`generate_node_quantscript` 当前还有 sibling-like 内部复用点:

```text
build_quantscript_node_sources
```

因此后续实际抽离必须由父叶控制通信面，优先使用 `pub(super)` helper 或父级委托，不得让 artifact projection 子簇直接横向改 generator 子簇。

---

## 等价不变量

### graph metadata

`generate_quantscript_from_graph_value` 必须保留:

```text
graph_id
name
version
mode
```

默认值与读取顺序不得改变；缺失字段仍走当前 fallback。

### node rendering

`generate_node_quantscript` 必须保留:

```text
runtime
execution
plugin
module_key
type
unknown.module
name
category
config
inputs
```

节点类型分支、module key fallback、name/category/config 输出、输入边收集顺序均不得改变。

### edge rendering

edge 输出必须继续使用当前形状:

```text
connect {source}.{source_port} -> {target}.{target_port}
# no connections
```

没有边时仍输出 `# no connections`；有边时不得改变 source/target/port 拼接语义。

### scalar rendering

`render_json_scalar` 必须保留:

```text
string -> quoted
number -> direct
bool -> direct
null -> null
object/array -> serde_json fallback or null fallback
```

后续抽离不得借机改变复杂 JSON 的 fallback 行为。

---

## 调用面冻结

当前已知 caller:

```text
src/compile_api.rs
src/graph_api.rs
src/tests_backend.rs
src/backend/graph_compile/quantscript_graph.rs
```

caller 关系:

| Caller | 使用节点 | 约束 |
| --- | --- | --- |
| `src/compile_api.rs` | `generate_quantscript_from_graph_value` | 继续经 root parent re-export surface，不得直连 child |
| `src/graph_api.rs` | `generate_quantscript_from_graph_value` | save graph artifact 行为不变 |
| `src/tests_backend.rs` | `generate_quantscript_from_graph_value`、parser round-trip | 不得降低 round-trip 证据 |
| `build_quantscript_node_sources` | `generate_node_quantscript` | 后续抽离必须保留父级受控 helper 通信 |

---

## 不进入范围

本批不处理:

1. 不移动 Rust 函数。
2. 不创建 child file。
3. 不改 `register_routes`、`load_graph_quantscript`、`parse_graph_quantscript`。
4. 不改 `convert_graph_json_to_script_module`。
5. 不改 `attach_quantscript_artifacts` 或 runtime target projection。
6. 不改 `parse_graph_quantscript_source`。
7. 不改 `src/lib.rs` root parent re-export surface。
8. 不新增 sibling horizontal link。
9. 不启动 release transition guard 例外。

---

## 下一步边界

下一步只能进入:

```text
BE-001FR-02
backend.graph_compile.quantscript_graph.graph_to_qs_generation
root.backend.graph_compile.quantscript_graph.graph_to_qs_generation
```

BE-001FR-02 只允许形成抽离方案，固定 planned child、父级声明、可见性、调用方适配和测试门禁；不得直接改写 Rust。

---

## 验证要求

本批是 `no code movement` 等价基线，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

后续 BE-001FR-03 实际抽离时应补跑:

```powershell
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FR-01 完成时，必须说明:

1. 当前只是 `no code movement` 等价基线。
2. `src/backend/graph_compile/quantscript_graph.rs` 仍是真实 owner。
3. `graph_to_qs_generation baseline_frozen` 成立，但 child file 尚未创建。
4. `build_quantscript_node_sources` 仍依赖 `generate_node_quantscript`，下一步必须处理父级受控通信。
5. 不得宣称 `convert_graph_json_to_script_module`、`attach_quantscript_artifacts`、`parse_graph_quantscript_source` 或 `backend.graph_compile` 已收口。

---

## 验收标准

1. `479-backend.graph_compile.quantscript_graph.graph_to_qs_generation单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend.graph_compile.quantscript_graph.graph_to_qs_generation baseline_frozen` 已记录。
3. 关键 public/helper 节点已冻结。
4. 隐性调用点 `build_quantscript_node_sources` 已登记。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
