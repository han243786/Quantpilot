# v4.16.0 backend.graph_compile.quantscript_graph.graph_to_qs_generation 单叶 closeout
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FR-04
> 基线: `481-backend.graph_compile.quantscript_graph.graph_to_qs_generation抽离记录.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 判定: 等价成立，本叶停止细拆
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation`
> 代码动作: no code movement
> 下一步: BE-001FS-01 `backend.graph_compile.quantscript_graph` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FR-04 `graph_to_qs_generation` 单叶 closeout | 子叶 closeout / stop_split 判断 |
| 规范矩阵 | equivalence closeout / stop_split true / parent residual return / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.graph_to_qs_generation` | 单叶收口 |
| 模块树 | `backend.graph_compile.quantscript_graph.graph_to_qs_generation` | `stop_split: true` |

---

## closeout 判定

BE-001FR-03 已完成实际抽离:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
graph_to_qs_generation_actual_extraction_done
```

父级 `src/backend/graph_compile/quantscript_graph.rs` 只保留:

```rust
mod graph_to_qs_generation;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;
```

以及父级 artifact projection 对 node renderer 的受控内部调用:

```text
graph_to_qs_generation::generate_node_quantscript
```

等价成立，当前设置:

```text
backend.graph_compile.quantscript_graph.graph_to_qs_generation stop_split: true
```

---

## 不继续细拆理由

| 候选微叶 | 判断 | 理由 |
| --- | --- | --- |
| `graph_metadata_rendering` | 不拆 | 只服务 generator header，拆出会增加私有参数传递 |
| `node_block_rendering` | 不拆 | 与 edge/input 收集绑定，当前 `pub(super)` 已是父级最小通信面 |
| `scalar_rendering` | 不拆 | `quoted` 与 `render_json_scalar` 很小，拆出不会形成稳定 owner |
| `edge_connect_rendering` | 不拆 | 只在 graph output 中使用，单独拆会降低可读性 |

本叶现在的复杂度主要来自同一条 graph JSON -> QuantScript source 生成链，继续拆分会制造更多 helper 接线和可见性边界，不会显著降低幻觉风险。

---

## 保留不变量

后续不得改变:

```text
graph_id
name
version
mode
runtime
execution
plugin
unknown.module
connect {source}.{source_port} -> {target}.{target_port}
# no connections
```

---

## 下一步边界

下一步只能回到父叶残余判断:

```text
BE-001FS-01
backend.graph_compile.quantscript_graph
root.backend.graph_compile.quantscript_graph
```

父叶仍有候选残余:

```text
route_surface
formal_module_conversion
artifact_target_projection
strategy_graph_parser
```

BE-001FS-01 只能做父叶残余判断和下一颗子叶选择，不得直接迁移 parser、formal conversion、artifact projection 或 route handler。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不拆 parser。
3. 不拆 formal conversion。
4. 不拆 artifact target projection。
5. 不拆 route surface。
6. 不改 compile / graph / runtime caller。
7. 不新增 sibling horizontal link。
8. 不启动 release transition guard。

---

## 验证要求

本批是 `no code movement` closeout，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## 幻觉检查点

AI 声称 BE-001FR-04 完成时，必须说明:

1. 本批是 `no code movement` 单叶 closeout。
2. `backend.graph_compile.quantscript_graph.graph_to_qs_generation stop_split: true`。
3. 父叶 `backend.graph_compile.quantscript_graph` 仍保持 `stop_split: false`。
4. 下一步只能进入 BE-001FS-01 父叶残余判断。
5. 不得宣称 parser、formal conversion、artifact projection、route surface 或 `backend.graph_compile` 已收口。

---

## 验收标准

1. `482-backend.graph_compile.quantscript_graph.graph_to_qs_generation单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `graph_to_qs_generation_closeout_done` 已记录。
3. 本叶设置 `stop_split: true`。
4. 下一步固定为 BE-001FS-01 父叶残余判断。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
