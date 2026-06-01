# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FU-01
> 基线: `487-backend.graph_compile.quantscript_graph.formal_module_conversion单叶closeout.md`
> 目标父叶: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> 判定: 父叶仍有残余，本轮选择 intent_lowering
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion`
> 代码动作: no code movement
> 下一步: BE-001FV-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FU-01 `formal_module_conversion` 父叶残余判断 | 回到父叶 / 选择下一子叶 |
| 规范矩阵 | recursive residual judgment / stop_split false / child selection / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion` | 子叶队列继续推进 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion` | `stop_split: false` |

---

## 已完成子叶确认

上一轮子叶已经 closeout:

```text
formal_module_conversion closeout_done
formal_module_conversion stop_split: false
formal_module_conversion actual_extraction_done
```

实际代码已经迁入:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
convert_graph_json_to_script_module
```

父级仍只通过受控 re-export 对外暴露:

```rust
mod formal_module_conversion;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
```

---

## 父叶当前残余

`src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs` 仍承载以下职责簇:

| 残余簇 | 代表行为 | 当前判断 |
| --- | --- | --- |
| `input_shape_validation` | `graph.nodes` / `graph.edges` required array validation | 暂缓，体量很小，先保持在父叶入口 |
| `data_source_lowering` | `data` node -> `fetch(...)`，含 exchange / instrument / interval / lookback / request options | 稍后，可独立成较薄子叶 |
| `profile_lowering` | `risk.profile(...)` / `execution.profile(...)` | 稍后，风险和执行 profile 可作为一组 |
| `intent_lowering` | intent node upstream edge resolution, built-in strategy branches, emit calls | 本轮选择，职责最长且分支最多 |
| `unsupported_intent_failure` | unsupported intent `anyhow::bail!` | 跟随 intent lowering 基线冻结 |
| `unsupported_node_logging` | unknown node `safe_eprintln!` | 暂缓，保持非阻断日志语义 |
| `terminal_parse` | `parse_quant_script_module(&qs_source)` -> `ScriptModule` | 暂缓，作为父叶终点保持稳定 |

因此父叶继续保持:

```text
formal_module_conversion parent_residual_judgment
formal_module_conversion stop_split: false
backend.graph_compile.quantscript_graph.formal_module_conversion residual_exists
```

---

## 下一子叶选择

本轮选择:

```text
BE-001FV-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
intent_lowering_selected
```

优先选择 `intent_lowering` 的原因:

1. 它是当前父叶中最长的真实职责簇。
2. 它同时处理 upstream edge resolution、source var 派生、built-in strategy 分支和 emit 行为。
3. 它覆盖 `builtin.intent.double_ma`、`builtin.intent.rsi`、`builtin.intent.ma_deviation`、`builtin.intent.macd`、`builtin.intent.momentum`、`builtin.intent.zscore` 与 `builtin.intent.spread_observer`。
4. unsupported intent failure 与 intent 分支紧密相连，适合先在等价基线中冻结。
5. 抽离后可以让 data source lowering、profile lowering 与 terminal parse 的边界更容易复核。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不创建 `intent_lowering.rs`。
3. 不移动 `data`、`risk`、`execution` 或 `intent` 分支。
4. 不改 `parse_quant_script_module` 终点。
5. 不改 unsupported node `safe_eprintln!` 日志语义。
6. 不改 `src/compile_api.rs`、`src/lib.rs` 或 route surface。
7. 不新增 sibling horizontal link。
8. 不启动 release transition guard 之外的发布态优化。

---

## 下一步边界

下一步只能进入:

```text
BE-001FV-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
```

BE-001FV-01 只能建立单子叶等价基线，冻结 intent node 输入、upstream edge resolution、source var 规则、所有 built-in intent 分支、unsupported intent failure、caller 和回归证据。不得直接创建 child file 或移动代码。

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

AI 声称 BE-001FU-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `formal_module_conversion stop_split: false`。
3. 下一步只进入 BE-001FV-01 `intent_lowering` 单子叶等价基线。
4. 不得宣称 `intent_lowering` 已抽离。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `488-backend.graph_compile.quantscript_graph.formal_module_conversion父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `formal_module_conversion parent_residual_judgment` 已记录。
3. `intent_lowering_selected` 已记录。
4. 下一步固定为 BE-001FV-01 `intent_lowering` 单子叶等价基线。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
