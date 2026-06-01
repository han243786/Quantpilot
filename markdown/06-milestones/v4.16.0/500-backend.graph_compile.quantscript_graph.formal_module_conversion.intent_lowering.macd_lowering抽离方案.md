# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FZ-02
> 基线: `499-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering`
> 代码动作: no code movement
> 下一步: BE-001FZ-03 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FZ-02 `macd_lowering` 抽离方案 | 方案冻结 |
| 规范矩阵 | extraction plan / branch-level helper / parent-child communication / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | planned child 接口设计 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` | plan_frozen |

---

## Planned Child

下一批 BE-001FZ-03 只允许创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs
```

父级 `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` 只允许新增:

```rust
mod macd_lowering;
```

并将当前 match branch 改为父到子的受控调用:

```rust
"builtin.intent.macd" => {
    macd_lowering::append_macd_lowering_lines(cfg, &source_var, instrument, qs_lines);
}
```

方案标记:

```text
macd_lowering plan_frozen
macd_lowering baseline_frozen
```

---

## Planned Helper Signature

planned helper 固定为:

```rust
use serde_json::Value;

pub(super) fn append_macd_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

该 helper 只服务父级 `intent_lowering`，不得导出给 `formal_module_conversion`、compile API、graph API、runtime sibling 或 frontend。

---

## 允许迁移代码块

BE-001FZ-03 只允许迁移当前 `builtin.intent.macd` branch 内部代码:

```rust
"builtin.intent.macd" => {
    let fast = cfg
        .get("fast_period")
        .and_then(|v| v.as_u64())
        .unwrap_or(12);
    let slow = cfg
        .get("slow_period")
        .and_then(|v| v.as_u64())
        .unwrap_or(26);
    let signal_period = cfg
        .get("signal_period")
        .and_then(|v| v.as_u64())
        .unwrap_or(9);
    qs_lines.push(format!(
        "    let macd_val = macd({}, {}, {}, {})",
        source_var, fast, slow, signal_period
    ));
    qs_lines.push("    if macd_val > 0 {".to_string());
    qs_lines.push(format!(
        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
        instrument
    ));
    qs_lines.push("    } else if macd_val < 0 {".to_string());
    qs_lines.push(format!(
        "        emit Intent(\"SELL\", instrument=\"{}\", quantity=1.0)",
        instrument
    ));
    qs_lines.push("    }".to_string());
}
```

必须保持以下语义:

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

---

## 禁止迁移边界

BE-001FZ-03 不允许处理:

1. 不抽 `shared_intent_context`。
2. 不抽 `builtin.intent.double_ma`。
3. 不抽 `builtin.intent.rsi`。
4. 不抽 `builtin.intent.ma_deviation`。
5. 不抽 `builtin.intent.momentum`。
6. 不抽 `builtin.intent.zscore`。
7. 不改 `builtin.intent.spread_observer` 或 `spread_observer_lowering` child。
8. 不改 unsupported intent `anyhow::bail!`。
9. 不改 `formal_module_conversion.rs`、route surface、parser、artifact target projection、frontend caller 或 runtime caller。
10. 不启动 release transition。

---

## 父子通信规则

BE-001FZ-03 后唯一允许新增连接:

```text
intent_lowering -> macd_lowering
```

现有连接必须保持:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
```

禁止新增:

```text
formal_module_conversion -> macd_lowering
compile_api -> macd_lowering
graph_quantscript_api -> macd_lowering
graph_api -> macd_lowering
runtime sibling -> macd_lowering
frontend -> macd_lowering
sibling horizontal link
```

release transition guard: 当前没有开发者发布过渡决定，不允许为了性能横连 child。

---

## 回退方案

若 BE-001FZ-03 失败，回退只允许:

1. 删除 `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/macd_lowering.rs`。
2. 移除父级 `mod macd_lowering;`。
3. 将 `"builtin.intent.macd"` branch 恢复为 BE-001FZ-01 冻结的原始 inline 逻辑。
4. 保留本抽离方案文档，并在下一轮记录失败原因和新方案，不得静默改写基线。

---

## BE-001FZ-03 验证门禁

实际抽离提交前至少执行:

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
BE-001FZ-03
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
```

BE-001FZ-03 只能创建 planned child、添加父级 `mod macd_lowering;` 并迁移 `builtin.intent.macd` branch；不得顺手移动其它 built-in intent branch、shared context 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001FZ-02 完成时，必须说明:

1. 本批是 `no code movement` 抽离方案。
2. `macd_lowering plan_frozen` 成立，但 child file 尚未创建。
3. 下一步 BE-001FZ-03 才允许迁移 `builtin.intent.macd` branch。
4. 不得宣称 `macd_lowering` 已抽离。
5. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `500-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `macd_lowering plan_frozen` 已记录。
3. planned child、父级 `mod`、helper signature、允许迁移块和回退方案已固定。
4. 下一步固定为 BE-001FZ-03 实际抽离记录。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
