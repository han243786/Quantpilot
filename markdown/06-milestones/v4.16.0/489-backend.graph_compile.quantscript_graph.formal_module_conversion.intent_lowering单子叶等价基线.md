# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FV-01
> 基线: `488-backend.graph_compile.quantscript_graph.formal_module_conversion父叶残余判断.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 等价基线
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: no code movement
> 下一步: BE-001FV-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FV-01 `intent_lowering` 单子叶等价基线 | 子叶基线 |
| 规范矩阵 | equivalence baseline / intent branch invariants / parent communication rule / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | 新增下一层白箱节点 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | intent lowering 边界冻结 |

---

## 当前真实 owner

本批只冻结当前实现，不移动代码:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
convert_graph_json_to_script_module
```

当前 `intent_lowering` 仍是 `convert_graph_json_to_script_module` 内部分支，尚未创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

基线标记:

```text
intent_lowering baseline_frozen
```

---

## 白箱 public/helper 节点

本子叶冻结以下 helper surface:

| 节点 | 当前可见性 | 输入 | 输出 | 约束 |
| --- | --- | --- | --- | --- |
| intent node scan | 函数内分支 | `nodes` array | intent node iteration | 只处理 `node.type == "intent"` |
| module key resolution | 函数内分支 | intent node | `module_key` | 缺失时继续使用空字符串并进入 unsupported failure |
| instrument resolution | 函数内分支 | intent config | instrument text | 缺失时 default `BTCUSDT` |
| upstream edge resolution | 函数内分支 | `edges` array + `target_node_id` | source node id | 找不到时 default `data` |
| source var normalization | 函数内分支 | source node id | QS variable name | `-` 与 `.` 必须替换为 `_` |
| built-in intent branch lowering | 函数内分支 | module_key + config + source var | indicator / emit QS lines | 不得更改支持集合、默认值或 BUY/SELL 条件 |
| unsupported intent failure | 函数内分支 | unsupported module_key | `anyhow::bail!` | 不得降级为静默跳过 |

后续实际抽离时，child 只能由父叶 `formal_module_conversion` 调用，不得让 compile / graph / runtime sibling 直接调用 `intent_lowering`。

---

## 输入输出等价

输入仍由父级传入:

```text
nodes
edges
qs_lines
```

intent lowering 必须继续只读取:

```text
node.type
node.module_key
node.config
node.id
edge.target_node_id
edge.source_node_id
```

输出必须继续是追加到 `qs_lines` 的 QS source lines，且最终仍由父级执行:

```text
qs_lines.push(...)
parse_quant_script_module(&qs_source)
```

不得让 intent child 返回 `ScriptModule`，也不得绕过父级 terminal parse。

---

## 分支语义冻结

### 通用 intent 输入

通用规则必须保持:

```text
module_key default empty string
instrument default BTCUSDT
node_id default empty string
upstream edge selected by target_node_id
source_id default data
source_var = source_id.replace(['-', '.'], "_")
```

### builtin.intent.double_ma

必须保持:

```text
fast_period default 20
slow_period default 50
let fast = sma(source_var, fast)
let slow = sma(source_var, slow)
if fast > slow emit BUY quantity 1.0
```

### builtin.intent.rsi

必须保持:

```text
period default 14
oversold_threshold or oversold default 30.0
let {node_id}_signal = rsi(source_var, period)
if signal < oversold emit BUY quantity 1.0
```

### builtin.intent.ma_deviation

必须保持:

```text
lookback default 15
baseline_period default 150
let ma_dev = sma(source_var, lookback) / sma(source_var, baseline)
if ma_dev > 1 emit SELL quantity 1.0
```

### builtin.intent.macd

必须保持:

```text
fast_period default 12
slow_period default 26
signal_period default 9
let macd_val = macd(source_var, fast, slow, signal_period)
if macd_val > 0 emit BUY quantity 1.0
else if macd_val < 0 emit SELL quantity 1.0
```

### builtin.intent.momentum

必须保持:

```text
lookback default 10
threshold_ratio or threshold default 0.02
let {node_id}_signal = momentum(source_var, lookback)
if signal > threshold emit BUY quantity 1.0
```

### builtin.intent.zscore

必须保持:

```text
window default 20
entry_z default 2.0
let {node_id}_signal = zscore(source_var, window)
if signal < -abs(entry_z) emit BUY quantity 1.0
```

### builtin.intent.spread_observer

必须保持:

```text
all upstream sources from edges whose target_node_id equals node_id
left_source = first upstream source or source_var
right_source = second upstream source or left_source
max_time_diff_ms default 5000
spread_output_code 1 -> bps else ratio
comparison_threshold default 0.0
comparison_op_code 3 -> >= else >
align_asof(field(left_source, name="bid"), direction="backward", tolerance_ms=...)
align_asof(field(right_source, name="ask"), direction="backward", tolerance_ms=...)
spread(left, right, output=...)
if signal op threshold emit BUY quantity 1.0
```

### unsupported intent

未知 intent module 必须继续 hard fail:

```text
anyhow::bail!
不支持的意图模块
double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer
```

不得回退为 `safe_eprintln!` 或静默跳过。

---

## 调用面冻结

当前已知 caller:

```text
src/compile_api.rs
src/lib.rs
```

caller 关系:

| Caller | 使用节点 | 约束 |
| --- | --- | --- |
| `src/compile_api.rs` | `convert_graph_json_to_script_module` | conversion error 继续映射到 `qs_conversion_failed` / `ERR_QS_LOWER_FAILED` |
| `src/lib.rs` | root parent re-export | compile / graph / runtime / test 调用面继续只接触父级 surface |

后续 BE-001FV-03 实际抽离时，必须保持父子通信规则:

```text
formal_module_conversion -> intent_lowering
```

不得出现:

```text
compile_api -> intent_lowering
graph_quantscript_api -> intent_lowering
graph_api -> intent_lowering
runtime sibling -> intent_lowering
```

---

## 不进入范围

本批不处理:

1. 不移动 Rust 代码。
2. 不创建 `intent_lowering.rs`。
3. 不改 data source lowering。
4. 不改 risk / execution profile lowering。
5. 不改 unsupported node `safe_eprintln!`。
6. 不改 `parse_quant_script_module` terminal parse。
7. 不改 `src/compile_api.rs` 或 `src/lib.rs`。
8. 不新增 sibling horizontal link。
9. 不启动 release transition guard 之外的发布态优化。

---

## 下一步边界

下一步只能进入:

```text
BE-001FV-02
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
```

BE-001FV-02 只允许形成抽离方案，固定 planned child、函数签名、父级调用方式、回归测试和回退点；不得直接改写 Rust。

---

## 验证要求

本批是 `no code movement` 等价基线，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

后续 BE-001FV-03 实际抽离时应补跑:

```powershell
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FV-01 完成时，必须说明:

1. 当前只是 `no code movement` 等价基线。
2. `intent_lowering baseline_frozen` 成立，但 child file 尚未创建。
3. `intent_lowering` 仍由 `convert_graph_json_to_script_module` 内部分支持有。
4. 七个 built-in intent 分支、默认值、BUY/SELL 条件和 unsupported intent failure 均未改。
5. 不得宣称 `formal_module_conversion`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `489-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `intent_lowering baseline_frozen` 已记录。
3. built-in intent 支持集合、默认值、emit 行为、unsupported intent failure、caller 与回归点均已冻结。
4. 下一步固定为 BE-001FV-02 抽离方案。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
