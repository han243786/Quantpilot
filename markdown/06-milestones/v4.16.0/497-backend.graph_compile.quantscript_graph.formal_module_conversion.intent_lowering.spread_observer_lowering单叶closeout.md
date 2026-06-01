# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering 单叶 closeout
> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001FX-04
> 基线: `496-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离记录.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 判定: 单叶 closeout，停止继续细拆
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 代码动作: no code movement
> 下一步: BE-001FY-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FX-04 `spread_observer_lowering` 单叶 closeout | 子叶收口 |
| 规范矩阵 | closeout / stop_split true / equivalence evidence / parent residual return | 轻量档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | child 白箱节点收口 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | stop_split: true |

---

## 完成证据

已完成:

```text
spread_observer_lowering baseline_frozen
spread_observer_lowering plan_frozen
spread_observer_lowering actual_extraction_done
spread_observer_lowering closeout_done
spread_observer_lowering stop_split: true
```

真实文件:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs
```

父级只保留:

```text
mod spread_observer_lowering;
spread_observer_lowering::append_spread_observer_lowering_lines
```

---

## 继续细拆判断

本叶不继续拆分。

理由:

1. `upstream_sources`、`left_source`、`right_source` 共同服务双源 spread observer 输入面。
2. `max_time_diff_ms`、`spread_output_code`、`comparison_threshold`、`comparison_op_code` 是同一 branch 的 config decode。
3. `align_asof`、`spread(...)`、`emit Intent("BUY", instrument="{}", quantity=1.0)` 是同一 QuantScript rendering 序列。
4. 将 source collection、config decode、QS line rendering 再拆成微叶会增加父子接线，但不会产生稳定 owner。
5. 当前 helper 足够小，继续细拆不符合三档执行原则。

因此:

```text
source_collection_micro_leaf rejected
config_decode_micro_leaf rejected
qs_line_rendering_micro_leaf rejected
```

---

## 等价保持点

closeout 继续冻结以下语义:

```text
builtin.intent.spread_observer
upstream_sources
left_source
right_source
source_var fallback
max_time_diff_ms default 5000
spread_output_code Some(1) -> bps
spread_output_code other -> ratio
comparison_threshold default 0.0
comparison_op_code Some(3) -> >=
comparison_op_code other -> >
align_asof
field({}, name="bid")
field({}, name="ask")
spread({}_left, {}_right, output="{}")
emit Intent("BUY", instrument="{}", quantity=1.0)
```

---

## 父子通信规则

保留的唯一新增连接:

```text
intent_lowering -> spread_observer_lowering
```

上层连接仍是:

```text
formal_module_conversion -> intent_lowering
```

继续禁止:

```text
formal_module_conversion -> spread_observer_lowering
compile_api -> spread_observer_lowering
graph_quantscript_api -> spread_observer_lowering
graph_api -> spread_observer_lowering
runtime sibling -> spread_observer_lowering
frontend -> spread_observer_lowering
sibling horizontal link
release transition
```

---

## 下一步边界

下一步只能回到父叶:

```text
BE-001FY-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
```

BE-001FY-01 只允许判断 `intent_lowering` 父叶残余并选择下一个子叶；不得直接移动其它 built-in intent branch、shared context、unsupported intent failure 或 release transition。

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

AI 声称 BE-001FX-04 完成时，必须说明:

1. 当前是 `no code movement` 单叶 closeout。
2. `spread_observer_lowering stop_split: true`。
3. 下一步回到 BE-001FY-01 `intent_lowering` 父叶残余判断。
4. 不得宣称 `intent_lowering` 已收口。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `497-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `spread_observer_lowering closeout_done` 与 `spread_observer_lowering stop_split: true` 已记录。
3. 下一步固定为 BE-001FY-01 `intent_lowering` 父叶残余判断。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
