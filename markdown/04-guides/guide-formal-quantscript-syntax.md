# 正式 QuantScript 语法指南

本文档是 QuantPilot 中正式 QuantScript 的当前语法参考。

它描述了以下代码实际实现的语言：

- `quantscript/src/script.rs`
- `quantscript/src/types.rs`
- `quantscript/src/resolve.rs`
- `quantscript/src/analysis.rs`
- `quantscript/src/lowering/mod.rs`

当前 lowering 内部拆分为：

- `quantscript/src/lowering/orchestrator.rs`
- `quantscript/src/lowering/context.rs`
- `quantscript/src/lowering/shared.rs`
- `quantscript/src/lowering/semantic.rs`
- `quantscript/src/lowering/diagnostics.rs`
- `quantscript/src/lowering/universe.rs`
- `quantscript/src/lowering/binding_sources.rs`
- `quantscript/src/lowering/source_recovery.rs`
- `quantscript/src/lowering/bindings.rs`
- `quantscript/src/lowering/fallback.rs`
- `quantscript/src/lowering/intents.rs`

如果旧的路线图、研究笔记或归档文档与本文件关于当前语法存在分歧，以本文件为准。

## 开发基线

本文件是解析器、解析器、分析和 lowering 路径当前实际实现的真实数据源。

它不是未来语言扩展的真实数据源。

未来的 QuantScript 开发必须遵循：

- [QuantScript 主干基线](./guide-quantscript-trunk-baseline.md)

这意味着：

- 解析器接受的语法不是自动的产品认可
- 仅解析的遗留构造不得用于证明将 QuantScript 扩展为通用语言是正当的
- 当当前语法比主干基线更广泛时，未来的工作应向基线收敛，而不是扩展更广泛的界面

## 范围

本指南涵盖 `quantscript.formal_source` 中携带的正式 QuantScript 产品路径。

它不描述：

- `strategy_graph` 图源导入/导出文本
- 已弃用的基于部分配置风格的 QuantScript
- 尚未实现的未来 Typed HIR 提案

## 当前 lowering 合约

正式 QuantScript 不是通用语言。可执行路径目前期望：

- 一个顶层 `fn strategy() { ... }`
- 至少一个从 `strategy` 可达的 `fetch(...)` 或 `get_data(...)` 调用
- 在 `strategy` 中至少有一个 `emit Intent(...)`
- 多交易对策略在使用时仍必须在编译时降低为一组有限的、按交易对展开的 `fetch(...)` 和 `emit Intent(...)` 语句
- 可选的 `risk.profile("global", ...)` 可以作为 `strategy` 内的单个顶层语句出现，且仅降低到现有的 `builtin.risk.global` 运行时模块
- 可选的 `execution.profile("paper", fee_bps=..., slippage_bps=...)` 可以作为 `strategy` 内的单个顶层语句出现，且仅降低到现有的 `builtin.execution.paper` 运行时模块
- 价差语义仍比当前辅助函数界面更窄；已落地的正式价差切片限于显式的 `align_asof(...) + spread(..., output="bps") + 单边 >/>=`，更广泛的已解析 `spread(...)` 形态仍然不是稳定的共享核心能力

解析器接受的语法多于运行时 lowering 路径保证的语法。
保留的可执行示例应仅来自 `quantscript/authoring_samples/`；
有意的拒绝 fixture 属于 `quantscript/boundary_samples/`，以便活跃的编写界面不混合成功和失败样本。
本文档显式标记该边界。

## 词法规则

### 注释

- 行注释以 `#` 开头。
- 没有块注释语法。
- 注释剥离是基于行的。

重要限制：

- `#` 在双引号字符串内被保护。
- `#` 在单引号字符串内未被可靠保护。
- 如果字符串可能包含 `#`，优先使用双引号。

### 空白和布局

- 解析器面向行。
- 空行被忽略。
- 块由 `{` 和 `}` 分隔。
- 函数 / `if` / `else if` / `else` / `for` / `while` / `match` 头必须以 `{` 在同一行结束。
- `} else if ... {` 和 `} else {` 在一行上被规范化并接受。

### 标识符

在表达式中，标识符由 ASCII 字母、数字和 `_` 组成。

使用简单的 ASCII 名称，例如：

```qs
closes
fast_ma
signal_1
```

## 文件结构

正式 QuantScript 文件只能包含：

- `import ...`
- `from ... import ...`
- `fn ...`
- `async fn ...`

顶层任何其他内容都被拒绝。

### 简化顶层语法

```text
module      := item*
item        := import_decl | from_import_decl | function_decl
import_decl := "import" module_name
from_import := "from" module_ref "import" import_name ("," import_name)*
function    := ["async"] "fn" name "(" params? ")" ["->" type] "{"
```

## 导入

### 普通导入

```qs
import math
import signals
```

### From-import

```qs
from data import fetch
from data import fetch as get_data
from signals@1.2 import rsi, macd
from transforms import field, resample, align_asof
```

规则：

- `from module import a, b, c` 受支持。
- `from module@version import ...` 受支持。
- `as` 别名仅在 `from ... import ... as ...` 中受支持。
- 普通 `import foo as bar` 不受支持。
- 正式 QuantScript 编译现在拒绝普通模块别名导入（如 `import foo as bar`），错误代码 `QS0608`。
- 即使辅助函数对运行时 lowering 无意义，导入也会在语法上被解析。

## 函数

### 语法

```qs
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}

fn moving_average(series: Series<Number>, period: Number) -> Number {
    return series[period..].sum() / period
}

async fn preload(symbols: List<String>) -> List<String> {
    return symbols
}
```

规则：

- `fn` 和 `async fn` 都被解析。
- 参数以逗号分隔。
- 每个参数可选地有类型注解：`name: Type`。
- 函数可选地有返回类型：`-> Type`。
- 当前可执行合约仍然以 `fn strategy()` 为中心。

实际边界：

- 辅助函数受支持并可参与规范化/lowering。
- 递归函数不受支持。
- 正式 QuantScript 编译现在拒绝直接递归辅助函数调用，错误代码 `QS0605`
- `async fn` 和 `await` 是可解析的遗留语法，但它们不是可执行策略代码的稳定运行时 lowering 合约，也不是未来主干方向的一部分。
- 正式 QuantScript 编译现在拒绝 `async fn`，错误代码 `QS0601`，以及 `await` 表达式，错误代码 `QS0602`

## 类型注解

解析器接受以下类型名称：

- `Unknown`
- `Unit`
- `Bool` / `bool`
- `Number` / `number`
- `String` / `string`
- `Symbol` / `symbol`
- `Universe` / `universe`
- `Signal` / `signal`
- `Scalar<T>`
- `Series<T>`
- `Maybe<T>`
- `List<T>`

示例：

```qs
fn helper(series: Series<Number>, period: Number) -> Number {
    return series[period..].mean()
}

fn names() -> List<String> {
    return ["BTCUSDT", "ETHUSDT"]
}
```

说明：

- 类型注解是可选的。
- 不支持的类型名称被拒绝。
- 即使类型注解被解析，当前的 lowering 仍然严重依赖解析器推断的 series/number 语义。
- `Symbol` 和 `Universe` 现在被解析器和 lowering 路径识别，但 `Universe` 仍然是受限能力，而非通用集合 API。

## 语句

### `let`

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let fast: Number = sma(closes, 20)
let mut out = []
```

规则：

- 语法：`let [mut] pattern [: Type] = expr`
- `mut` 被解析。
- 绑定模式存储为文本；简单的标识符绑定是受支持的路径。

重要限制：

- 没有像 `x = y` 这样的独立赋值语句。
- 使用 `let` 引入绑定。
- 可变的列表构建便利方法（如 `out.push(...)`）不是正式可执行主干的一部分；正式 QuantScript 编译现在拒绝它们，错误代码 `QS0609`。

### `return`

```qs
return
return score
return closes[20..].mean()
```

### `emit Intent(...)`

```qs
emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
emit Intent("SELL", instrument="BTCUSDT", quantity=0.5)
```

规则：

- `emit Intent(...)` 是一种专用的语句形式。
- 参数可以是位置参数或命名参数。
- 命名参数可以使用 `:` 或 `=`。

示例：

```qs
emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
emit Intent(action="BUY", instrument="BTCUSDT", quantity=1.0)
```

当前 lowering 边界：

- 运行时路径需要一个 action。
- 可执行 lowering 还需要至少一个可从策略推断的数据源。
- 实际上，保持 `emit Intent(...)` 接近标准交易字段，如 `instrument` 和 `quantity`。
- 条件性 `emit Intent(...)` 不是通用的回退界面。
- 如果周围条件未映射到支持的指标或价差 Intent，lowering 现在拒绝脚本，而非静默生成通用运行时 Intent。
- 已知的 lowering 合约失败现在作为结构化的正式 QuantScript 编译诊断浮现。
- 当正式 QuantScript 编译成功时，生成的 `core_ir.metadata.source_kind` 现在显式为 `formal_quant_script`，而非折叠到通用的运行时协议源标签中。
- 直接单源移动平均比较（如 `if sma(data, 20) > sma(data, 100)`）现在降低为结构化的 Core IR 谓词：`ScalarExpr::Compare`，基于共享的 `SeriesExpr::WindowAgg` 节点，而非仅回退到原始条件文本。
- 直接单边 RSI 阈值比较（如 `if rsi(data, 14) < 25`）现在也降低为结构化的 Core IR 谓词，基于共享的指标引用加数字阈值。当前的运行时 Intent 形态仍然将双边 RSI 买卖合约合并到一个节点中，因此双边 RSI 形式暂时继续回退到原始条件文本。
- 直接单边 `momentum` 和 `zscore` 阈值比较（如 `if momentum(data, 20) > 0.03` 或 `if zscore(data, 20) < -2`）现在也降低为结构化的 Core IR 谓词，基于降低的指标引用加原始有符号阈值。双边形式在 lowering 必须将两侧合并到单个运行时 Intent 时仍然留在原始文本路径上。
- 第一个落地的正式价差切片现在也降低为结构化的 Core IR 谓词，但仅限于狭窄的辅助函数形式 `spread(align_asof(...), align_asof(...), output="bps") > threshold` 或 `>= threshold`；比率输出、绝对值输出、`<` / `<=` 和辅助函数派生的价差运算仍保持在已接纳的正式界面之外。
- 示例包括用于不支持的条件性 `emit Intent(...)` 或格式错误的价差辅助函数条件的 `QPQSLOW001`，用于不支持的运行时操作的 `QPQSLOW004`，以及当策略 lowering 无法推断任何可达的 `fetch(...)` 或 `get_data(...)` 源时的 `QPQSLOW007`。
- Universe/rebalance 合约失败现在也作为结构化诊断浮现，包括用于不支持的 `rebalance(..., every=...)` 值的 `QPQSLOW009`，当依赖快照的 universe 操作在没有 `universe_snapshot` 的情况下编译时的 `QPQSLOW010`，以及用于不支持的 universe 排序顺序的 `QPQSLOW012`。
- Universe 输入形态合约也在迁移到结构化诊断：当辅助函数如 `filter/sort_by/top` 缺少其 universe 输入或未收到 universe 值输入时的 `QPQSLOW025`，当 `symbols(...)` 缺少其列表输入或未收到列表字面量时的 `QPQSLOW026`，当 `symbols([...])` 包含非字符串项时的 `QPQSLOW027`，以及当 `top(...)` 未收到数字计数参数时的 `QPQSLOW028`。
- Allocation/weights 约束也开始使用结构化诊断，包括当 `rebalance(...)` 缺少其分配辅助函数或未收到分配辅助函数时的 `QPQSLOW013`，当分配辅助函数缺少其选择输入或该输入不是 universe 值时的 `QPQSLOW014`，当分配解析为空交易对集合时的 `QPQSLOW015`，`fixed_weights` 计数不匹配时的 `QPQSLOW016`，负固定权重时的 `QPQSLOW017`，总和为零的固定权重时的 `QPQSLOW018`，不支持的 `rank_weight(..., method=...)` 值时的 `QPQSLOW019`，不支持的 `score_weight(..., normalize=...)` 值时的 `QPQSLOW020`，以及当 `weights=...` 缺失或不是数字列表字面量时的 `QPQSLOW021`。
- 指标输入合约也在迁移到结构化 lowering 诊断：当辅助函数如 `rsi/macd/momentum/zscore` 的第一个参数缺失或未降低到 `fetch/get_data` 源时的 `QPQSLOW022`，当 period/lookback/window 参数缺失、非数字或非正数时的 `QPQSLOW023`，以及当移动平均辅助函数缺少源输入、未收到有效源输入，或在 `ema(...)` 兼容性路径中未收到已识别的 `MACD` 线时的 `QPQSLOW024`。

### `if / else if / else`

```qs
if fast > slow {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
} else if fast < slow {
    emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
} else {
    log_warn("flat")
}
```

### `for`

```qs
for s in selected {
    let closes = fetch(s, interval="1d", lookback=200)?
    emit Intent("BUY", instrument=s, quantity=1.0)
}
```

边界：

- 对 `Universe` 的迭代支持正式 lowering
- 非 `Universe` 的 `for` 循环是可解析的遗留语法，但正式 QuantScript 编译现在拒绝它们，错误代码 `QS0606`
- 当前的 lowering 路径在运行时编译之前将循环展开为每个交易对单独的分支
- 这不是通用的运行时投资组合循环或动态 universe 状态机
- 此兼容性界面不得被视为在主要语言中扩展通用循环语义的许可

### `while`

```qs
while i < 10 {
    log_warn("loop")
}
```

当前状态：

- `while` 仅是可解析语法
- 它在推荐的可执行稳定主干之外
- 正式 QuantScript 编译现在提前拒绝它，错误代码 `QS0603`
- 未来的 QuantScript 开发不应将 `while` 扩展为通用策略控制流界面

### `match`

```qs
match read_data("BTCUSDT") {
    Ok(k) => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
    Err(e) => log_error(e)
}
```

重要限制：

- match 分支是基于行的。
- 每个分支体要么是：
  - 单个表达式，或者
  - 单个 `emit Intent(...)` 语句
- 块风格 match 分支不受支持。
- match 模式存储为原始文本；模式语言今天不是正式指定的类型检查模式系统。
- `match` 不是推荐未来主干的一部分，应视为有限的遗留/兼容性界面，除非证明需要更狭窄的 IR 支持的产品需求
- 正式 QuantScript 编译现在提前拒绝它，错误代码 `QS0604`

### 表达式语句

任何表达式都可用作语句：

```qs
log_warn("retry")
```

## 表达式

### 字面量

```qs
42
3.14
"BTCUSDT"
'BUY'
true
false
[1, 2, 3]
[]
```

说明：

- 数字字面量是十进制数。
- 负数被解析为一元 `-` 应用于正数。
- 单引号和双引号字符串都被标记化。
- 双引号字符串支持转义，如 `\"`、`\\`、`\n`、`\t`。
- 单引号字符串更简单，应视为纯文本字面量。

### 调用

```qs
fetch("BTCUSDT", interval="1d", lookback=200)
sma(closes, 20)
align_asof(series, direction="nearest", tolerance_ms=10000)
```

调用参数可以是：

- 位置参数：`sma(closes, 20)`
- 命名参数使用 `=`：`fetch("BTCUSDT", interval="1d")`
- 命名参数使用 `:`：`helper(period: 14)`

当使用当前的 universe 辅助函数时，`fetch(...)` 也可以接收 `Symbol` 类型的循环绑定作为位置参数 0：

```qs
let closes = fetch(s, interval="1d", lookback=200)?
```

### 成员访问

```qs
closes.mean()
closes.last()
scope.stddev()
```

### 索引

```qs
closes[0]
closes[14]
closes[-1]
```

说明：

- 负索引可被解析。
- 它们仍可能触发语义诊断，如前瞻风险。

### 切片

```qs
closes[20..]
closes[..20]
closes[10..20]
```

### 范围

```qs
1..10
start..end
```

### 前缀运算符

```qs
-value
!flag
not flag
await task
```

### 后缀运算符

```qs
fetch("BTCUSDT", interval="1d", lookback=200)?
get_data("BTCUSDT")?
```

`?` 被解析为后缀 try 运算符，通常用于 `fetch(...)` / `get_data(...)`。

- 正式 QuantScript 编译现在拒绝在非 fetch-like 表达式上的后缀 `?`，错误代码 `QS0607`。
- 在当前可执行主干中，后缀 `?` 是 fetch-like 数据源便利方法，而非通用的结果/错误传播特性。

### 二元运算符

支持的中缀运算符：

- `*`、`/`、`%`
- `+`、`-`
- `>`、`>=`、`<`、`<=`
- `==`、`!=`
- `&&`、`||`
- `and`、`or`

不支持：

- 按位 `&`
- 按位 `|`

### 优先级

从低到高：

1. 范围 `..`
2. 逻辑 `or` / `||`
3. 逻辑 `and` / `&&`
4. 相等 `== !=`
5. 比较 `> >= < <=`
6. 加法 `+ -`
7. 乘法 `* / %`
8. 前缀 `await`、一元 `-`、一元 `!`
9. 后缀 调用 / 成员 / 索引 / 切片 / `?`

## 内建函数和辅助函数

解析器接受任意调用名称。
解析器和 lowering 层仅对子集赋予特殊含义。

### Fetch-like 源

- `fetch`
- `get_data`

`fetch` / `get_data` 的当前运行时 lowering 默认值：

- 位置参数 0：交易对字符串或 `Symbol` 绑定，默认 `BTCUSDT`
- 命名参数 `exchange`，默认 `binance`
- 命名参数 `interval`，默认 `1d`
- 命名参数 `lookback`，默认 `200`

示例：

```qs
let closes = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
```

### 当前识别的 Universe 辅助函数

- `symbols`
- `universe`
- `filter`
- `sort_by`
- `top`
- `equal_weight`
- `fixed_weights`
- `rank_weight`
- `score_weight`
- `rebalance`

这些辅助函数仅受当前受限的编译时 universe 路径支持。

支持的形式：

```qs
let selected = symbols(["BTCUSDT", "ETHUSDT"])
```

```qs
let base = universe(exchange="binance", market="spot", quote="USDT")
let ranked = sort_by(base, key="market_cap", order="desc")
let selected = top(ranked, 10)
```

当前语义：

- `symbols([...])` 接受字符串字面量列表并返回 `Universe`
- `universe(...)` 从编译请求的 `universe_snapshot` 读取
- `universe_snapshot.as_of_ms` 现在是 lowering 合约中元数据支持的选择的一部分
- 每个 `UniverseAssetRecord` 现在可以携带：
  - 平面元数据，如 `market_cap`、`volume_24h` 和 `listing_age_days`
  - `listed_at_ms`，用于时间点上市资格
  - `metadata_history`，其中使用 `as_of_ms` 之前或等于的最新点
- `filter(...)` 当前支持快照支持的过滤键，如：
  - `quote`
  - `exchange`
  - `market`
  - `min_market_cap`
  - `min_volume_24h`
  - `min_listing_age_days`
- `sort_by(...)` 当前支持：
  - `key="symbol"`
  - `key="market_cap"`，这需要 `universe_snapshot`
  - `key="volume_24h"`，这需要 `universe_snapshot`
  - `key="listing_age_days"`，这需要 `universe_snapshot`
- `top(...)` 将 `Universe` 截断为先前过滤/排序后的前 `N` 个条目
- `equal_weight(universe_expr)` 当前标记选定的 `Universe` 用于等权投资组合再平衡 lowering
- `fixed_weights(universe_expr, weights=[...])` 当前为选定的 `Universe` 分配固定的归一化权重向量
- `rank_weight(universe_expr, method="linear" | "inverse_rank")` 当前按信号分数对选定交易对排序，并从排名顺序推导权重
- `score_weight(universe_expr, normalize="sum")` 当前将选定的信号分数归一化为权重
- `rebalance(<allocation>, every="1d")`
- `rebalance(<allocation>, every="slow")`
- `rebalance(<allocation>, every="weekly")`
  当前启用正式的投资组合再平衡 lowering 路径

当前硬边界：

- 这些辅助函数不是通用的集合转换
- 它们不会在运行时连续评估
- 它们在正式 lowering 期间仅解析一次
- 元数据支持的选择现在在 `universe_snapshot.as_of_ms` 处时间点感知，但仅针对请求中提供的单个编译时快照
- 当前的 `for s in selected { ... }` 循环仍然展开为具体的每个交易对分支
- 当前的 `rebalance(equal_weight(...), ...)` lowering 改为将显式的目标 universe 携带到运行时投资组合再平衡中
- 这些分配辅助函数不是通用的分配 DSL
- `rebalance(...)` 当前仅支持：
  - `equal_weight(universe_expr)`
  - `fixed_weights(universe_expr, weights=[...])`
  - `rank_weight(universe_expr, method="linear" | "inverse_rank")`
  - `score_weight(universe_expr, normalize="sum")`
- `rebalance(...)` 当前仅支持 `every="slow"` / `every="1d"` / `every="weekly"` 节奏形式

### 受限的投资组合再平衡辅助函数

当前正式 QuantScript 路径现在支持最小的投资组合级再平衡入口。

支持的形式：

```qs
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
```

额外支持的形式：

```qs
let base = symbols(["BTCUSDT", "ETHUSDT"])
rebalance(fixed_weights(base, weights=[0.7, 0.3]), every="slow")
```

```qs
let selected = top(sort_by(base, key="market_cap", order="desc"), 3)
rebalance(rank_weight(selected, method="inverse_rank"), every="1d")
```

```qs
let selected = top(sort_by(base, key="market_cap", order="desc"), 3)
rebalance(score_weight(selected, normalize="sum"), every="1d")
```

当前可执行语义：

- `rebalance(equal_weight(selected), ...)` 不会在脚本空间中创建单独的投资组合对象
- 它将降低的代理标记为投资组合再平衡代理
- 选定的 `Universe` 作为显式的再平衡目标交易对集携带到后端/运行时
- 后端/运行时现在将节奏存储为类型化的 `RebalanceSchedule` 而非仅字符串辅助函数标志
- 当前运行时首先使用选定的分配模型构建后端 `PortfolioTargetDecision`
- 风险当前将该投资组合目标传递到执行，无需单独的投资组合约束 DSL
- 执行然后将当前持仓与目标权重进行比较，并生成最终的买卖篮子
- 显式再平衡目标集中的交易对即使它们在当前评估轮次中未发出新信号，也可能被卖出至零

当前分配语义：

- `equal_weight(...)`
  - 为每个选定的交易对分配相同的目标权重
- `fixed_weights(..., weights=[...])`
  - 每个选定的交易对一个数字权重
  - 权重在 lowering 期间归一化，使其总和为 1
- `rank_weight(..., method="linear")`
  - 按信号分数降序排列选定的交易对
  - 分配与 `N, N-1, ..., 1` 成比例的权重
- `rank_weight(..., method="inverse_rank")`
  - 按信号分数降序排列选定的交易对
  - 分配与 `1/1, 1/2, ..., 1/N` 成比例的权重
- `score_weight(..., normalize="sum")`
  - 分配与选定的正信号分数成比例的权重
  - 当前仅支持 `normalize="sum"`

当前节奏语义：

- `every="slow"` 表示再平衡代理在每个慢周期上评估
- `every="1d"` 表示再平衡代理最多每 24 小时评估一次
- `every="weekly"` 表示再平衡代理最多每 7 个滚动天评估一次
- 当前的 24 小时节流基于上次再平衡评估时间戳，而非上次成交时间戳
- 当前的 7 天节流也基于上次再平衡评估时间戳，而非上次成交时间戳
- 如果再平衡评估运行且未产生成交，`every="1d"` 仍然延迟下次再平衡评估 24 小时
- 如果再平衡评估运行且未产生成交，`every="weekly"` 仍然延迟下次再平衡评估 7 天

当前边界：

- 该路径仍然依赖有限的编译时 `Universe`
- 运行时不会在每个 bar 上连续重建 `Universe`
- 仅支持上面列出的受限分配辅助函数
- 投资组合目标生成存在，但高级投资组合约束尚不是正式 QuantScript 的一部分
- 正式 QuantScript 中还没有用户定义的权重函数、任意比较器或任意目标权重映射 DSL

当前后端投资组合风险支持：

- 后端运行时现在支持风险策略对象上的投资组合目标限制字段：
  - `max_single_weight`
  - `max_turnover`
  - `min_trade_weight`
  - `max_new_positions_per_rebalance`
- 这些约束在执行计算最终订单篮子之前应用于后端 `PortfolioTargetDecision` 对象
- 当前行为是保守的：
  - `max_single_weight` 独立钳制每个目标权重
  - `max_turnover` 将投资组合增量向当前权重缩放
  - `min_trade_weight` 通过将小增量 snapping 回当前权重来移除小增量
  - `max_new_positions_per_rebalance` 仅保留优先级最高的新条目，其余置零
- 正式 QuantScript 尚不提供这些字段的直接语法
- 目前，这些约束仅可通过后端/运行时风险策略对象配置

### 编译时 Universe 快照要求

快照支持的 universe 选择在脚本源中不是自包含的。

如果脚本使用：

- `universe(...)`
- 带有快照支持元数据过滤器的 `filter(...)`
- `sort_by(..., key="market_cap")`
- `sort_by(..., key="volume_24h")`
- `sort_by(..., key="listing_age_days")`

则正式编译请求必须提供 `universe_snapshot`。

没有 `universe_snapshot`，lowering 失败并返回结构化的编译错误。

当前时间点元数据合约：

- `universe_snapshot.as_of_ms` 是 lowering 期间使用的元数据选择时间戳
- 如果资产提供 `metadata_history`，lowering 使用具有 `entry.as_of_ms <= universe_snapshot.as_of_ms` 的最新条目
- 如果资产提供 `listed_at_ms`，即使后来存在元数据，该资产在此时间之前被视为不合格
- 如果没有符合条件的历点存在，lowering 在可用时回退到平面顶层字段
- 这改善了编译时 universe 选择的时间点正确性，但尚不提供运行时动态重新选择

### 内建数学和序列缩减器

- `abs`
- `avg`
- `first`
- `last`
- `max`
- `mean`
- `min`
- `pow`
- `sqrt`
- `std`
- `stddev`
- `sum`
- `variance`

这些可以作为自由函数或在有意义时作为成员风格调用出现：

```qs
mean(closes[20..])
closes[20..].mean()
first(closes)
closes.last()
```

### 由 resolve/lowering 识别的指标辅助函数

- `sma`
- `ema`
- `rsi`
- `macd`
- `momentum`
- `zscore`
- `z_score`

### 变化辅助函数和平滑化别名

增益类辅助函数：

- `gains`
- `gain`
- `up_moves`
- `positive_changes`
- `positive_deltas`

损失类辅助函数：

- `losses`
- `loss`
- `down_moves`
- `negative_changes`
- `negative_deltas`

平滑化别名：

- `rma`
- `wilders`
- `smma`

### 当前识别的导入转换辅助函数

- `field`
- `resample`
- `align`
- `align_asof`
- `spread`

这些与当前受限的价差/报价观察 lowering 路径特别相关。

### 求值器折叠的成员风格辅助函数

当用于兼容值时，求值器可以折叠：

- `.len()`
- `.sum()`
- `.mean()`
- `.avg()`
- `.min()`
- `.max()`
- `.std()`
- `.stddev()`
- `.variance()`
- `.first()`
- `.last()`
- `.ok()`
- `.retryable()`

以及可变列表构造可以使用：

- `.push(...)`

说明：

- `.ok()` 和 `.retryable()` 当前是围绕 fetch-like 表达式的辅助函数/求值器便利方法。
- 正式 QuantScript 编译现在拒绝可执行策略代码中的 `.ok()` / `.retryable()` 辅助函数便利方法，错误代码 `QS0610`。
- 它们在解析代码中的存在并不意味着语言具有完整的结果/错误类型系统。
- `.push(...)` 也仅是辅助函数/求值器便利方法；正式 QuantScript 编译现在拒绝可执行策略代码中的它，错误代码 `QS0609`。

## Lowering 友好模式

当你编写以下家族之一时，当前运行时路径最强：

- 直接移动平均交叉，使用 `sma(...)` / `ema(...)`
- 手动移动平均窗口，如 `closes[20..].sum() / 20`
- 通过 `rsi(...)` 的 RSI
- 通过 `macd(...)` 或已识别的手动 EMA 公式的 MACD
- 通过 `momentum(...)` 或已识别的手动公式的动量
- 通过 `zscore(...)` 或已识别的手动公式的 z-score
- 狭窄的已接纳价差辅助函数形式 `align_asof(...) + spread(..., output="bps") + 单边 >/>=`；更广泛的价差公式和辅助函数派生形式仍然被拒绝
- 受限的编译时展开多交易对策略，使用 `symbols(...)` 或 `universe(...)` 加 `for s in selected`
- 受限的投资组合再平衡，使用支持的分配辅助函数加 `every="slow"`、`every="1d"` 或 `every="weekly"`

示例：

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum(closes, 14)

    if score > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

```qs
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(sort_by(base, key="market_cap", order="desc"), 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
```

此示例仅在编译请求包含匹配的 `universe_snapshot` 时受支持。

```qs
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
```

此示例当前降低为：

- 编译时展开的每个交易对 `fetch(...)` 和 `emit Intent(...)`
- 后端 `PortfolioRebalance` 代理策略
- 携带到运行时中的显式再平衡目标交易对集
- 跨选定交易对的运行时等权目标计算
- 根据后端 `RebalanceSchedule` 的运行时节奏门控

## 诊断相关的语义规则

语法解析器与可执行合约相比更宽松。
当前的语义分析额外检查以下规则：

- 未解析的名称
- 重复函数
- 非布尔条件
- 来自无效历史访问的前瞻风险
- 当 `lookback` 小于所需序列历史时预热不足
- 需要快照元数据但在没有 `universe_snapshot` 的情况下编译的 universe 辅助函数
- 在 lowering 期间不支持的 universe 排序键或排序顺序
- 在 lowering 期间不支持的快照支持的过滤键
- 在 lowering 期间不支持的 `rebalance(..., every=...)` 值
- 除上述受限辅助函数集之外不支持的 `rebalance(...)` 分配形式
- 可执行策略代码中的直接递归辅助函数调用
- 可执行策略代码中的非 `Universe` 的 `for` 循环
- `every="1d"` 按经过的运行时时间节流再平衡评估；它不与交易所会话或日历日边界绑定
- `every="weekly"` 按经过的运行时时间节流再平衡评估；它不与日历周边界绑定

当前预热规则：

- 仅 `fetch(...)` / `get_data(...)` 上的显式 `lookback=` 计入
- 语义分析中没有隐藏的默认预热假设

## 重要不支持或不稳定的领域

以下要么不支持、仅解析，要么不是稳定可执行合约的一部分：

- 导入/函数之外的任意顶层语句
- 独立的重新赋值语句，如 `x = y`
- 块风格 match 分支
- 正式的解构模式系统
- 递归
- 通用异步策略执行
- 任意主机代码执行
- 完整的用户定义状态模型
- 完整的异常/结果/特质/类/模块系统
- 运行时动态 universe 刷新或每 bar top-N 重新选择
- 当前受限的分配辅助函数再平衡路径之外的通用投资组合/篮子语义
- 任意用户定义的集合转换、lambda 或 `Universe` 上的自定义比较器
- 自定义投资组合权重函数、任意目标权重映射或用户定义的排名/分数 DSL
- 由运行时刷新 `Universe` 成员资格驱动的动态投资组合再平衡
- 运行时动态或每 bar universe 重新选择；当前时间点元数据仍然仅适用于单个编译时 `universe_snapshot.as_of_ms`
- 正式 QuantScript 中直接提供的板块上限、换手率控制或其他高级投资组合策略 DSL
- 超出当前 `slow` / 滚动 24 小时 / 滚动 7 天形式的日历感知再平衡计划

这些构造中的几个在此有意列出为非主干领域，而非未来扩展的待办事项。
在决定缺失能力是应进入语言还是应存在于配置文件、自定义节点、模块或工具中时，使用主干基线指南。

## 实际建议

为保持在稳定路径内：

- 定义恰好一个 `fn strategy()`
- 使用 `fetch(...)` 或 `get_data(...)` 显式带 `lookback=...`
- 保持绑定简单，清晰命名中间序列
- 优先使用已识别的辅助函数，如 `sma`、`ema`、`rsi`、`macd`、`momentum`、`zscore`
- 如果使用多交易对选择，保持使用 `symbols(...)` / `universe(...)` + `filter(...)` + `sort_by(...)` + `top(...)`
- 当使用元数据支持的 universe 选择时，传递一个 `universe_snapshot`，其 `as_of_ms` 匹配您想要的回测或运行选择时间
- 当您需要时间点正确的排名/过滤而非单个平面最新快照时，在 `universe_snapshot` 中使用 `listed_at_ms` 或 `metadata_history`
- 如果使用投资组合再平衡，保持使用受限的分配辅助函数加 `every="slow"`、`every="1d"` 或 `every="weekly"`
- 当您希望每个慢周期进行再平衡时，使用 `every="slow"`
- 当滚动 24 小时节流可接受时，使用 `every="1d"`
- 当滚动 7 天节流可接受时，使用 `every="weekly"`
- 每当使用元数据支持的 universe 选择时，在正式编译请求中提供 `universe_snapshot`
- 仅对受限的 `Universe` 迭代使用 `for`，如 `for s in selected`
- 不要依赖 `match`、递归或可执行正式 QuantScript 中的通用集合循环
- 不要依赖普通 `import foo as bar` 或任意后缀 `?`，好像正式 QuantScript 支持完整的模块/错误系统
- 不要依赖 `.ok()`、`.retryable()` 或 `.push(...)`，好像它们是稳定的正式语言特性
- 优先使用双引号字符串
- 保持标识符为 ASCII

## 最小示例

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```
