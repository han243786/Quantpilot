# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001GB-02
> 基线: `504-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> 判定: 抽离方案冻结
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> 代码动作: no code movement
> 下一步: BE-001GB-03 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001GB-02 `double_ma_lowering` 抽离方案 | 方案冻结 |
| 规范矩阵 | plan freeze / branch move boundary / parent-child communication / rollback point | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | child 文件计划落地 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | plan_frozen |

---

## 计划变更

BE-001GB-03 只允许新增 planned child:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/double_ma_lowering.rs
```

父级文件:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

父级只允许新增 module declaration:

```rust
mod double_ma_lowering;
```

父级 `builtin.intent.double_ma` branch 只允许替换为受控调用:

```rust
"builtin.intent.double_ma" => {
    double_ma_lowering::append_double_ma_lowering_lines(
        cfg,
        &source_var,
        instrument,
        qs_lines,
    );
}
```

planned helper:

```rust
pub(super) fn append_double_ma_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

计划标记:

```text
double_ma_lowering plan_frozen
double_ma_lowering baseline_frozen
```

---

## 允许迁移块

BE-001GB-03 只允许迁移当前 branch:

```rust
"builtin.intent.double_ma" => {
    let fast = cfg
        .get("fast_period")
        .and_then(|v| v.as_u64())
        .unwrap_or(20);
    let slow = cfg
        .get("slow_period")
        .and_then(|v| v.as_u64())
        .unwrap_or(50);
    qs_lines.push(format!("    let fast = sma({}, {})", source_var, fast));
    qs_lines.push(format!("    let slow = sma({}, {})", source_var, slow));
    qs_lines.push("    if fast > slow {".to_string());
    qs_lines.push(format!(
        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
        instrument
    ));
    qs_lines.push("    }".to_string());
}
```

不得移动 shared context:

```text
module_key
cfg
instrument
node_id
upstream_edge
source_id
source_var
```

不得移动其它分支:

```text
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
unsupported intent
anyhow::bail!
```

---

## 等价不变量

BE-001GB-03 必须保持:

```text
builtin.intent.double_ma
fast_period default 20
slow_period default 50
let fast = sma({}, {})
let slow = sma({}, {})
fast > slow
emit Intent("BUY", instrument="{}", quantity=1.0)
```

QS 行顺序必须保持:

```text
let fast = sma(...)
let slow = sma(...)
if fast > slow {
emit Intent("BUY", instrument="{}", quantity=1.0)
}
```

---

## 父子通信规则

BE-001GB-03 后唯一允许新增连接:

```text
intent_lowering -> double_ma_lowering
```

现有连接保持:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
```

禁止新增:

```text
formal_module_conversion -> double_ma_lowering
compile_api -> double_ma_lowering
graph_quantscript_api -> double_ma_lowering
graph_api -> double_ma_lowering
runtime sibling -> double_ma_lowering
frontend -> double_ma_lowering
sibling horizontal link
```

release transition guard: 当前没有开发者发布过渡决定，不允许以性能为理由绕过父子通信。

---

## 回退点

如果 BE-001GB-03 的编译或测试失败，回退必须只发生在本子叶:

1. 删除 `mod double_ma_lowering;`。
2. 删除 `double_ma_lowering::append_double_ma_lowering_lines(...)` 调用。
3. 将 `builtin.intent.double_ma` branch 放回父级原位置。
4. 删除 planned child file。

不得回退 `spread_observer_lowering`、`macd_lowering` 或 `intent_lowering` 已完成的抽离。

---

## 验证门禁

BE-001GB-03 至少执行:

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
BE-001GB-03
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering
```

BE-001GB-03 只允许创建 planned child、添加父级 `mod double_ma_lowering;` 并移动 `builtin.intent.double_ma` branch。不得顺手处理 `rsi`、`ma_deviation`、`momentum`、`zscore`、shared context、unsupported failure 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001GB-02 完成时，必须说明:

1. 本批是 `no code movement` 抽离方案。
2. `double_ma_lowering plan_frozen` 成立。
3. planned child 尚未创建，`builtin.intent.double_ma` 尚未移动。
4. 下一步只能进入 BE-001GB-03 实际抽离记录。
5. 不得宣称 `double_ma_lowering` 已抽离。
6. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `505-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `double_ma_lowering plan_frozen` 已记录。
3. 下一步固定为 BE-001GB-03 `double_ma_lowering` 实际抽离记录。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
