# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering 抽离记录
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FZ-03
> 基线: `500-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离方案.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 判定: 实际抽离完成
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 代码动作: actual extraction
> 下一步: BE-001FZ-04 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FZ-03 `macd_lowering` 实际抽离记录 | 代码抽离 |
| 规范矩阵 | actual extraction / parent-child communication / equivalence preservation / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | child file 落地 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | actual_extraction_done |

---

## 实际变更

新增 child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs
```

父级文件:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

父级只新增 module declaration:

```rust
mod macd_lowering;
```

父级 `builtin.intent.macd` branch 现在只保留受控调用:

```rust
"builtin.intent.macd" => {
    macd_lowering::append_macd_lowering_lines(cfg, &source_var, instrument, qs_lines);
}
```

child helper:

```rust
pub(super) fn append_macd_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

抽离标记:

```text
macd_lowering actual_extraction_done
macd_lowering plan_frozen
macd_lowering baseline_frozen
```

---

## 等价保持

本批只迁移 `builtin.intent.macd` branch，没有改变参数 fallback、QS line、BUY / SELL emit 顺序:

```text
fast_period default 12
slow_period default 26
signal_period default 9
macd({}, {}, {}, {})
macd_val > 0
emit Intent("BUY", instrument="{}", quantity=1.0)
macd_val < 0
emit Intent("SELL", instrument="{}", quantity=1.0)
```

父级 `source_var` 和 `instrument` 仍由 `intent_lowering` 统一解析，child 只消费父级传入的显式输入面。

---

## 父子通信结果

新增且唯一允许的 child 连接已经落地:

```text
intent_lowering -> macd_lowering
```

现有连接保持:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
```

未新增:

```text
formal_module_conversion -> macd_lowering
compile_api -> macd_lowering
graph_quantscript_api -> macd_lowering
graph_api -> macd_lowering
runtime sibling -> macd_lowering
frontend -> macd_lowering
sibling horizontal link
```

release transition guard: 本批未启动发布过渡，也未为了性能横连 child。

---

## 未迁移内容

本批没有迁移:

1. `shared_intent_context`。
2. `builtin.intent.double_ma`。
3. `builtin.intent.rsi`。
4. `builtin.intent.ma_deviation`。
5. `builtin.intent.momentum`。
6. `builtin.intent.zscore`。
7. `builtin.intent.spread_observer` 或 `spread_observer_lowering` child。
8. unsupported intent `anyhow::bail!`。
9. `formal_module_conversion.rs`、route surface、parser、artifact target projection、frontend caller 或 runtime caller。
10. release transition。

---

## 验证门禁

提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source
```

---

## 下一步边界

下一步只能进入:

```text
BE-001FZ-04
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
```

BE-001FZ-04 只能做单叶 closeout，确认 `macd_lowering` 等价并判断是否值得继续细拆；不得跳过 closeout 直接处理其它 built-in intent branch。

---

## 幻觉检查点

AI 声称 BE-001FZ-03 完成时，必须说明:

1. `macd_lowering actual_extraction_done` 成立。
2. `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs` 已创建。
3. 父级只保留 `mod macd_lowering;` 和 `macd_lowering::append_macd_lowering_lines` 受控调用。
4. 其它 built-in intent branch、shared context、unsupported failure 和 release transition 均未迁移。
5. 不得宣称 `macd_lowering` 已 closeout、`intent_lowering` 已收口或 Rust 重构完成。

---

## 验收标准

1. `501-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `macd_lowering actual_extraction_done` 已记录。
3. 新 child file 进入全量树覆盖。
4. 父级通信只通过 `intent_lowering -> macd_lowering`。
5. 下一步固定为 BE-001FZ-04 单叶 closeout。
6. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、MACD 定向测试和 `git diff --check` 均通过。
