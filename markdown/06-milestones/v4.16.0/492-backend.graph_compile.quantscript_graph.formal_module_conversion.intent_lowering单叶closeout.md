# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 单叶 closeout
> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001FV-04
> 基线: `491-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离记录.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 等价成立，但本叶不停止细拆
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: no code movement
> 下一步: BE-001FW-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FV-04 `intent_lowering` 单叶 closeout | closeout / 继续细拆判断 |
| 规范矩阵 | equivalence closeout / stop_split false / parent-child only call / release transition guard | 轻量档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | closeout_done |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | `stop_split: false` |

---

## closeout 判定

BE-001FV-03 的实际抽离等价成立:

```text
intent_lowering actual_extraction_done
intent_lowering closeout_done
```

当前 child 文件:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

继续由父级文件单向调用:

```rust
// src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
mod intent_lowering;
intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;
parse_quant_script_module(&qs_source)
```

child 暴露的受控父子通信面保持:

```rust
pub(super) fn append_intent_lowering_lines(
    nodes: &[Value],
    edges: &[Value],
    qs_lines: &mut Vec<String>,
) -> anyhow::Result<()>
```

本批不移动 Rust 代码，不改函数体，不改可见性，不改错误语义，不改 `safe_eprintln!` 未知节点日志边界。

---

## 不停止细拆理由

本叶已经从 `formal_module_conversion` 中抽离，但内部仍是多职责 intent lowering unit:

| 内部责任 | 当前状态 | 判断 |
| --- | --- | --- |
| intent node scan | 与七个分支共享 `nodes` 输入 | 仍是父叶残余 |
| upstream edge resolution | 解析 `edges` 并生成 `source_var` | 可独立成公共上下文 |
| `builtin.intent.double_ma` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.rsi` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.ma_deviation` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.macd` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.momentum` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.zscore` | 独立分支 | 可作为后续分支候选 |
| `builtin.intent.spread_observer` | 双上游、align_asof、spread 输出和 threshold 语义更重 | 值得优先判断 |
| unsupported intent failure | `anyhow::bail!` 硬失败 | 不得静默降级 |

因此本叶设置:

```text
intent_lowering stop_split: false
```

下一步必须先做父叶残余判断，由 BE-001FW-01 选择是否先拆 shared context、`spread_observer` 或其它 built-in branch。不得在本 closeout 内直接创建新 child。

---

## 保留不变量

后续递归必须保持:

```text
module_key default empty string
instrument default BTCUSDT
upstream edge selected by target_node_id
source var replaces dash and dot
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
unsupported intent
anyhow::bail!
parse_quant_script_module(&qs_source)
```

父子通信规则:

```text
formal_module_conversion -> intent_lowering
```

禁止新增:

```text
compile_api -> intent_lowering
graph_quantscript_api -> intent_lowering
graph_api -> intent_lowering
runtime sibling -> intent_lowering
frontend -> intent_lowering
sibling horizontal link
```

release transition guard: 只有开发者明确决定进入发布版本过渡时，后续提案才可以讨论 sibling horizontal link 或性能连接。本批没有进入发布过渡。

---

## 不进入范围

本批不得:

1. 修改 Rust 代码。
2. 拆 `data_source_lowering`、`profile_lowering`、`terminal_parse` 或 parser。
3. 拆任一 built-in intent branch。
4. 新建 `spread_observer`、shared context 或 branch child。
5. 修改 route surface、artifact target projection、strategy graph parser、frontend caller 或 runtime caller。
6. 新增 sibling horizontal link。
7. 启动 release transition。
8. 宣称 `formal_module_conversion`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 下一步边界

下一步只能进入:

```text
BE-001FW-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
parent residual judgment
```

BE-001FW-01 只允许判断 `intent_lowering` 内部剩余责任的下一候选，不得直接改 Rust。若选择分支级拆分，后续仍必须按递归流程先建单子叶等价基线。

---

## 验证要求

本批是 `no code movement` closeout，提交前至少执行:

```text
git diff --check
tools\check-utf8.ps1
tools\check-matrix-governance.ps1
tools\check-full-feature-tree.ps1
cargo fmt --check
cargo check -p quantpilot
```

BE-001FV-03 的实际抽离目标回归已通过并在抽离记录中固化:

```text
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
54 passed
4 passed
1 passed
```

---

## 幻觉检查点

AI 声称 BE-001FV-04 完成时，必须说明:

1. 本批是 `no code movement` 单叶 closeout。
2. `intent_lowering actual_extraction_done` 与 `intent_lowering closeout_done` 成立。
3. `intent_lowering stop_split: false`，本叶仍需要 BE-001FW-01 父叶残余判断。
4. 下一步只能进入 BE-001FW-01，不得直接拆分支。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构完成。

---

## 验收标准

1. `492-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `intent_lowering closeout_done` 已记录。
3. `intent_lowering stop_split: false` 已记录。
4. 下一步固定为 BE-001FW-01 父叶残余判断。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
