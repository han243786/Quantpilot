# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering 单叶 closeout
> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001FZ-04
> 基线: `501-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离记录.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 判定: 单叶 closeout，停止继续细拆
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 代码动作: no code movement
> 下一步: BE-001GA-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FZ-04 `macd_lowering` 单叶 closeout | 子叶收口 |
| 规范矩阵 | closeout / stop_split true / equivalence evidence / parent residual return | 轻量档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | child 白箱节点收口 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | stop_split: true |

---

## 完成证据

已完成:

```text
macd_lowering baseline_frozen
macd_lowering plan_frozen
macd_lowering actual_extraction_done
macd_lowering closeout_done
macd_lowering stop_split: true
```

真实文件:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

父级只保留:

```text
mod macd_lowering;
macd_lowering::append_macd_lowering_lines
```

helper 输入面:

```rust
pub(super) fn append_macd_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

---

## 继续细拆判断

本叶不继续拆分。

理由:

1. `fast_period`、`slow_period`、`signal_period` 共同构成单一 MACD indicator config decode。
2. `macd({}, {}, {}, {})`、`macd_val > 0`、`macd_val < 0` 是同一 signal rendering 与 branching 序列。
3. BUY / SELL emit 只依赖 `instrument` 和固定 quantity，拆成 emit 微叶会增加父子接线但不形成独立 owner。
4. 当前 helper 只承接 `builtin.intent.macd` 一个 branch，足够小，继续拆分不符合三档执行原则。
5. 后续应回到 `intent_lowering` 父叶继续判断其它残余，而不是在本叶内部制造微叶。

因此:

```text
macd_config_decode_micro_leaf rejected
macd_signal_rendering_micro_leaf rejected
macd_buy_sell_emit_micro_leaf rejected
```

---

## 等价保持点

closeout 继续冻结以下语义:

```text
builtin.intent.macd
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

## 父子通信规则

保留的唯一新增连接:

```text
intent_lowering -> macd_lowering
```

上层连接仍是:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
```

继续禁止:

```text
formal_module_conversion -> macd_lowering
compile_api -> macd_lowering
graph_quantscript_api -> macd_lowering
graph_api -> macd_lowering
runtime sibling -> macd_lowering
frontend -> macd_lowering
sibling horizontal link
release transition
```

---

## 下一步边界

下一步只能回到父叶:

```text
BE-001GA-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
intent_lowering stop_split: false
```

BE-001GA-01 只允许判断 `intent_lowering` 父叶残余并选择下一个子叶；不得直接移动 `double_ma`、`rsi`、`ma_deviation`、`momentum`、`zscore`、shared context、unsupported intent failure 或 release transition。

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
cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source
```

---

## 幻觉检查点

AI 声称 BE-001FZ-04 完成时，必须说明:

1. 当前是 `no code movement` 单叶 closeout。
2. `macd_lowering closeout_done` 与 `macd_lowering stop_split: true`。
3. 下一步回到 BE-001GA-01 `intent_lowering` 父叶残余判断。
4. 不得宣称 `intent_lowering` 已收口。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `502-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `macd_lowering closeout_done` 与 `macd_lowering stop_split: true` 已记录。
3. 下一步固定为 BE-001GA-01 `intent_lowering` 父叶残余判断。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、MACD 定向测试和 `git diff --check` 均通过。
