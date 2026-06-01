# v4.16.0 backend.graph_compile.quantscript_graph 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FS-01
> 基线: `482-backend.graph_compile.quantscript_graph.graph_to_qs_generation单叶closeout.md`
> 目标父叶: `backend.graph_compile.quantscript_graph`
> 判定: 父叶仍有残余，本轮选择 formal module conversion
> 模块树坐标: `root.backend.graph_compile.quantscript_graph`
> 代码动作: no code movement
> 下一步: BE-001FT-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FS-01 `backend.graph_compile.quantscript_graph` 父叶残余判断 | 回到父叶 / 选择下一子叶 |
| 规范矩阵 | recursive residual judgment / stop_split false / child selection / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph` | 子叶队列继续推进 |
| 模块树 | `backend.graph_compile.quantscript_graph` | `stop_split: false` |

---

## 已完成子叶确认

上一轮子叶已经 closeout:

```text
backend.graph_compile.quantscript_graph.graph_to_qs_generation stop_split: true
graph_to_qs_generation_closeout_done
```

该子叶已迁入:

```text
src/backend/graph_compile/quantscript_graph/graph_to_qs_generation.rs
```

并且停止继续拆分 graph_metadata_rendering / node_block_rendering / scalar_rendering / edge_connect_rendering 微叶。

---

## 父叶当前残余

`src/backend/graph_compile/quantscript_graph.rs` 仍承载以下责任簇:

| 残余簇 | 代表入口 | 当前判断 |
| --- | --- | --- |
| `route_surface` | `register_routes`、`load_graph_quantscript`、`parse_graph_quantscript` | 暂缓，route surface 很薄，单独抽离收益低 |
| `formal_module_conversion` | `convert_graph_json_to_script_module` | 进入下一轮，职责独立且转换分支较长 |
| `artifact_target_projection` | `attach_quantscript_artifacts`、runtime targets、label targets | 稍后，依赖 generator 与 diagnostics target |
| `strategy_graph_parser` | `parse_graph_quantscript_source`、`parse_qs_scalar`、`parse_qs_node_header`、`parse_qs_connect` | 稍后，parser 语义风险高，需要独立基线 |

因此父叶继续保持:

```text
backend.graph_compile.quantscript_graph stop_split: false
backend_graph_compile_quantscript_graph_residual_exists
```

---

## 下一子叶选择

本轮选择:

```text
BE-001FT-01
backend.graph_compile.quantscript_graph.formal_module_conversion
root.backend.graph_compile.quantscript_graph.formal_module_conversion
formal_module_conversion_selected
```

优先选择 `formal_module_conversion` 的原因:

1. `convert_graph_json_to_script_module` 是 graph JSON -> `ScriptModule` 的独立转换通道。
2. 它仍在父文件中承载 data / risk / execution / intent 多分支 lowering。
3. 它与已完成的 graph-to-QS source generation 是 sibling，不应通过横向连接互相调用。
4. 它比 parser 风险低，且比 route surface 更有真实 owner 收益。
5. 它能为后续 parser 与 artifact target projection 提供更清晰的父级边界。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不移动 `convert_graph_json_to_script_module`。
3. 不拆 `register_routes`、`load_graph_quantscript` 或 `parse_graph_quantscript`。
4. 不拆 `attach_quantscript_artifacts`、`build_quantscript_runtime_targets` 或 label target helper。
5. 不拆 `parse_graph_quantscript_source` 或 `parse_qs_*` parser helper。
6. 不改 compile / graph / runtime caller。
7. 不新增 sibling horizontal link。
8. 不启动 release transition guard 之外的发布态优化。

---

## 下一步边界

下一步只能进入:

```text
BE-001FT-01
backend.graph_compile.quantscript_graph.formal_module_conversion
root.backend.graph_compile.quantscript_graph.formal_module_conversion
```

BE-001FT-01 只能建立单子叶等价基线，冻结 `convert_graph_json_to_script_module` 的输入、输出、分支语义、错误行为、caller 与回退点；不得直接创建 child 文件或迁移函数。

---

## 验证要求

本批是 `no code movement` 父叶残余判断，提交前至少执行:

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

AI 声称 BE-001FS-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `graph_to_qs_generation` 已 `stop_split: true`。
3. `backend.graph_compile.quantscript_graph stop_split: false`。
4. 下一步只能进入 BE-001FT-01 `formal_module_conversion` 单子叶等价基线。
5. 不得宣称 route surface、artifact target projection、strategy_graph_parser、`backend.graph_compile` 或 Rust 重构已经收口。

---

## 验收标准

1. `483-backend.graph_compile.quantscript_graph父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend.graph_compile.quantscript_graph stop_split: false` 已记录。
3. `formal_module_conversion_selected` 已记录。
4. 下一步固定为 BE-001FT-01 单子叶等价基线。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
