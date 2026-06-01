# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 抽离记录
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FV-03
> 基线: `490-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离方案.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 实际抽离完成
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: actual extraction
> 下一步: BE-001FV-04 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FV-03 `intent_lowering` 实际抽离记录 | 实际抽离 |
| 规范矩阵 | actual extraction / parent-only call / equivalence verification / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | child owner 落地 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | actual_extraction_done |

---

## 代码变更

本批创建 planned child:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

child 承接:

```rust
pub(super) fn append_intent_lowering_lines(
    nodes: &[Value],
    edges: &[Value],
    qs_lines: &mut Vec<String>,
) -> anyhow::Result<()>
```

父级 `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs` 只新增:

```rust
mod intent_lowering;
intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;
```

抽离标记:

```text
intent_lowering actual_extraction_done
```

---

## 等价确认

本批只移动 intent block，保持以下父级职责不变:

```text
graph.nodes validation
graph.edges validation
data_source_lowering
profile_lowering
unsupported_node_logging
qs_lines.push("}".to_string())
qs_lines.join("\n")
parse_quant_script_module(&qs_source)
```

child 内继续保留七个 built-in branch:

```text
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
```

unsupported intent 继续 hard fail:

```text
anyhow::bail!
不支持的意图模块
double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer
```

父子通信仍是唯一新增连接:

```text
formal_module_conversion -> intent_lowering
```

没有新增:

```text
compile_api -> intent_lowering
graph_quantscript_api -> intent_lowering
graph_api -> intent_lowering
runtime sibling -> intent_lowering
frontend -> intent_lowering
sibling horizontal link
```

---

## 不进入范围

本批没有处理:

1. 不改 `convert_graph_json_to_script_module` public/helper surface。
2. 不改 data source lowering。
3. 不改 risk / execution profile lowering。
4. 不改 unsupported node `safe_eprintln!`。
5. 不改 terminal `parse_quant_script_module(&qs_source)`。
6. 不改 `src/compile_api.rs`、`src/lib.rs`、route surface、artifact projection 或 parser。
7. 不启动 release transition。

---

## 验证记录

已执行:

```powershell
cargo fmt
cargo check -p quantpilot
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

结果:

```text
cargo check -p quantpilot passed
quantscript --lib: 54 passed
quantscript_real_strategy_authoring: 4 passed
api_graph_versions: 1 passed
```

提交前仍需执行完整治理门禁:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## 下一步边界

下一步只能进入:

```text
BE-001FV-04
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
```

BE-001FV-04 只能做单叶 closeout 与是否继续细拆判断。不得顺手拆 data_source_lowering、profile_lowering、terminal_parse、parser、route surface 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001FV-03 完成时，必须说明:

1. `intent_lowering actual_extraction_done` 成立。
2. child file 已创建并承接 intent block。
3. 父级只通过 `mod intent_lowering` 与 `append_intent_lowering_lines` 单向调用 child。
4. 七个 built-in intent 分支和 unsupported intent failure 保持不变。
5. 不得宣称 `intent_lowering` 已 closeout。
6. 不得宣称 `formal_module_conversion`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `491-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` 进入全量树覆盖。
3. `intent_lowering actual_extraction_done` 已记录。
4. 目标 Rust 回归和治理门禁均通过。
5. 下一步固定为 BE-001FV-04 单叶 closeout。
