# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FT-01
> 基线: `483-backend.graph_compile.quantscript_graph父叶残余判断.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> 判定: 等价基线
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion`
> 代码动作: no code movement
> 下一步: BE-001FT-02 `backend.graph_compile.quantscript_graph.formal_module_conversion` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FT-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线 | 子叶基线 |
| 规范矩阵 | equivalence baseline / formal conversion invariants / parent communication rule / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion` | 新增下一层白箱节点 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion` | formal module conversion 边界冻结 |

---

## 当前真实 owner

本批只冻结当前实现，不移动代码:

```text
src/backend/graph_compile/quantscript_graph.rs
```

当前 formal module conversion 仍由父叶 `backend.graph_compile.quantscript_graph` 持有。BE-001FT-01 不创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

基线标记:

```text
formal_module_conversion baseline_frozen
```

---

## 白箱 public/helper 节点

本子叶冻结以下 public/helper surface:

| 节点 | 当前可见性 | 输入 | 输出 | 约束 |
| --- | --- | --- | --- | --- |
| `convert_graph_json_to_script_module` | `pub(crate)` | graph `Value` | `ScriptModule` | 不得改 graph shape 校验、QS source 拼装或 parse 终点 |
| data fetch projection | 函数内分支 | data node config | `fetch(...)` QS line | 不得改 exchange/instrument/timeframe/window defaults |
| risk/execution projection | 函数内分支 | risk / execution node config | `risk.profile(...)` / `execution.profile(...)` QS line | 不得改 profile、fee、slippage、leverage defaults |
| intent lowering projection | 函数内分支 | intent node config + upstream edges | indicator / `emit Intent` QS lines | 不得改 builtin intent support set 或 signal 条件 |
| terminal parse | external parser call | generated QS source | `ScriptModule` | 必须继续通过 `parse_quant_script_module(&qs_source)` |

本基线登记 `convert_graph_json_to_script_module` 为关键 public 方法；后续若实际抽离，必须继续通过父级 `backend.graph_compile.quantscript_graph` 暴露，不得让 compile / graph / runtime sibling 直连 child。

---

## 输入输出等价

输入必须继续是 graph JSON `Value`，并要求:

```text
graph.nodes 必须是数组
graph.edges 必须是数组
```

缺失或类型错误时必须继续返回当前中文错误文本:

```text
graph.nodes 必须是数组
graph.edges 必须是数组
```

输出必须继续是 `quantscript::ScriptModule`，且终点仍是:

```text
parse_quant_script_module(&qs_source)
```

调用方仍通过 `src/lib.rs` 的 root parent re-export surface 使用:

```text
convert_graph_json_to_script_module
```

---

## 分支语义冻结

### data nodes

data node 投影必须保留:

```text
exchange default binance
instrument default BTCUSDT
timeframe default 1d
window_size >= 1.0 else default 200
ping_enabled optional bool
request_interval_ms optional u64
node id default data
var name replace '-' and '.' with '_'
fetch("{instrument}", exchange="...", interval="...", lookback=...)
```

### risk / execution nodes

risk node 投影必须保留:

```text
profile_id or profile_name default global
max_position default 0.2
max_total_leverage default 3.0
max_exchange_leverage default 3.0
min_action_interval_ms default 100
risk.profile(...)
```

execution node 投影必须保留:

```text
profile_id or profile_name or mode default paper
fee_bps default 10.0
slippage_bps default 5.0
execution.profile(...)
```

以下 node type 继续跳过或忽略:

```text
data
intent
agent
runtime
runtime_control
```

未知 node type 必须继续走 `safe_eprintln!` 日志并跳过，不得变成 hard error。

### intent nodes

intent node 投影必须保留:

```text
module_key default empty string
instrument default BTCUSDT
upstream edge by target_node_id
source node id default data
source var replace '-' and '.' with '_'
```

支持集合必须保持:

```text
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
```

未知 intent module 必须继续 `bail!`，错误文本继续包含:

```text
不支持的意图模块
double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer
```

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
| `src/compile_api.rs` | `convert_graph_json_to_script_module` | `compile_runtime_protocol_via_qs` 继续把 conversion error 映射为 `qs_conversion_failed` / `ERR_QS_LOWER_FAILED` |
| `src/lib.rs` | root parent re-export | 继续作为 compile / graph / runtime / test 调用面的父级桥，不得直连 future child |

当前间接证据仍来自 compile/QS pipeline；后续实际抽离时应补跑 compile 与 quantscript 相关测试，不得只依赖文档门禁。

---

## 不进入范围

本批不处理:

1. 不移动 Rust 函数。
2. 不创建 child file。
3. 不改 `convert_graph_json_to_script_module` 任一分支。
4. 不改 `generate_quantscript_from_graph_value` 或 `graph_to_qs_generation` child。
5. 不改 `register_routes`、`load_graph_quantscript`、`parse_graph_quantscript`。
6. 不改 `attach_quantscript_artifacts` 或 runtime target projection。
7. 不改 `parse_graph_quantscript_source` 或 parser helper。
8. 不改 `src/lib.rs` root parent re-export surface。
9. 不新增 sibling horizontal link。
10. 不启动 release transition guard 例外。

---

## 下一步边界

下一步只能进入:

```text
BE-001FT-02
backend.graph_compile.quantscript_graph.formal_module_conversion
root.backend.graph_compile.quantscript_graph.formal_module_conversion
```

BE-001FT-02 只允许形成抽离方案，固定 planned child、父级声明、可见性、调用方适配和测试门禁；不得直接改写 Rust。

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

后续 BE-001FT-03 实际抽离时应补跑:

```powershell
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FT-01 完成时，必须说明:

1. 当前只是 `no code movement` 等价基线。
2. `src/backend/graph_compile/quantscript_graph.rs` 仍是真实 owner。
3. `formal_module_conversion baseline_frozen` 成立，但 child file 尚未创建。
4. `convert_graph_json_to_script_module` 的 graph shape 校验、data/risk/execution/intent 分支和错误文本均未改。
5. 不得宣称 `graph_to_qs_generation`、artifact target projection、strategy graph parser、`backend.graph_compile` 或 Rust 重构已整体收口。

---

## 验收标准

1. `484-backend.graph_compile.quantscript_graph.formal_module_conversion单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `formal_module_conversion baseline_frozen` 已记录。
3. `convert_graph_json_to_script_module` 的输入、输出、分支语义、错误行为、caller 与回退点已冻结。
4. 下一步固定为 BE-001FT-02 抽离方案。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
