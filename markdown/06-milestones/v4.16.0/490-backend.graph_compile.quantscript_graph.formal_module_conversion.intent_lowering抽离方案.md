# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FV-02
> 基线: `489-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: no code movement
> 下一步: BE-001FV-03 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FV-02 `intent_lowering` 抽离方案 | 方案冻结 |
| 规范矩阵 | extraction plan / planned child / parent-only call / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | planned child 接口设计 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | plan_frozen |

---

## Planned Child

下一批只允许创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

父级只允许新增:

```rust
mod intent_lowering;
```

并在原 intent loop 位置改为父到子的受控调用:

```rust
intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;
```

方案标记:

```text
intent_lowering plan_frozen
```

---

## Planned Helper Signature

planned helper 固定为:

```rust
pub(super) fn append_intent_lowering_lines(
    nodes: &[Value],
    edges: &[Value],
    qs_lines: &mut Vec<String>,
) -> anyhow::Result<()>
```

约束:

1. `pub(super)` 只允许父叶 `formal_module_conversion` 调用。
2. `nodes` 与 `edges` 只读借用，不得取得 graph owner。
3. `qs_lines` 继续由父级创建并最终 join。
4. helper 只负责追加 intent lowering lines，不得调用 `parse_quant_script_module`。
5. helper 继续返回 `anyhow::Result<()>`，用于保持 unsupported intent hard fail。

---

## 允许移动的代码

BE-001FV-03 只允许移动当前 `formal_module_conversion.rs` 中的 intent block:

```text
// Add indicator/emit calls for intent nodes
for node in nodes {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
    if node_type == "intent" {
        ...
        match module_key {
            "builtin.intent.double_ma" => { ... }
            "builtin.intent.rsi" => { ... }
            "builtin.intent.ma_deviation" => { ... }
            "builtin.intent.macd" => { ... }
            "builtin.intent.momentum" => { ... }
            "builtin.intent.zscore" => { ... }
            "builtin.intent.spread_observer" => { ... }
            _ => anyhow::bail!(...)
        }
    }
}
```

移动后父级必须继续保留:

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

---

## 不允许移动的代码

BE-001FV-03 不得移动:

1. `convert_graph_json_to_script_module` 函数签名。
2. graph shape validation。
3. data node fetch projection。
4. risk / execution profile projection。
5. unknown non-intent node `safe_eprintln!`。
6. terminal `parse_quant_script_module(&qs_source)`。
7. parent module re-export in `src/backend/graph_compile/quantscript_graph.rs`。
8. `src/compile_api.rs` caller。
9. `src/lib.rs` root parent surface。

---

## 等价保持点

实际抽离后必须保持:

```text
module_key default empty string
instrument default BTCUSDT
node_id default empty string
source_id default data
source_var = source_id.replace(['-', '.'], "_")
```

七个 built-in branch 必须保持:

```text
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
```

unsupported intent 必须继续:

```text
anyhow::bail!
double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer
```

不得用 `safe_eprintln!` 替代 unsupported intent hard fail。

---

## 父子通信规则

BE-001FV-03 后唯一允许的新增连接是:

```text
formal_module_conversion -> intent_lowering
```

不得新增:

```text
compile_api -> intent_lowering
graph_quantscript_api -> intent_lowering
graph_api -> intent_lowering
runtime sibling -> intent_lowering
frontend -> intent_lowering
```

发布过渡协议不由 AI 主动提出，本批不启动 release transition。

---

## 回退方案

若 BE-001FV-03 验证失败，回退只允许:

1. 删除 planned child file。
2. 移除父级 `mod intent_lowering;`。
3. 将 intent loop 恢复到 `formal_module_conversion.rs` 原位置。
4. 保留已提交的 BE-001FV-01 / BE-001FV-02 治理记录，新增失败 closeout 或修正记录，不得改写历史语义。

---

## 下一步边界

下一步只能进入:

```text
BE-001FV-03
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
```

BE-001FV-03 只允许执行上述 planned child 创建与单 block 移动。不得顺手处理 data_source_lowering、profile_lowering、terminal_parse、parser、route surface 或 release transition。

---

## 验证要求

本批是 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

下一批 BE-001FV-03 实际抽离必须补跑:

```powershell
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FV-02 完成时，必须说明:

1. 当前只是 `no code movement` 抽离方案。
2. `intent_lowering plan_frozen` 成立，但 child file 尚未创建。
3. 下一步 BE-001FV-03 只能移动 intent block。
4. 不得宣称 `intent_lowering` 已抽离。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `490-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `intent_lowering plan_frozen` 已记录。
3. planned child、helper signature、父级调用方式、允许移动代码、不允许移动代码、回退方案和下一批测试门禁均已冻结。
4. 下一步固定为 BE-001FV-03 实际抽离记录。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
