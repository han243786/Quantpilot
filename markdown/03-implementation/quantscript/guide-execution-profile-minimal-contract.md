# QuantScript `execution.profile(...)` 最小合约

## 目的

本文档定义了将执行配置从 QuantScript 主干语法移出并移入 profile 形态边界的最小合约。

目标不是引入一个通用的执行 DSL。目标是让 QuantScript、图运行时编译和 Strategy IR 拥有一个小的、诚实的、共享的方式来指向现有的纸面执行模块，而无需将更多执行语义推入主干语言。

## 当前运行时锚点

今天支持的运行时执行模块仍然是：

- `builtin.execution.paper`

第一个合约已暴露的实际模块字段是：

- `fee_bps`
- `slippage_bps`

## 最小合约

### 形态

第一个合约是一个单一的内建辅助函数：

```qs
execution.profile("paper")
```

或者：

```qs
execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)
```

当前实现说明：

- 在正式 QuantScript 路径中，`execution.profile(...)` 当前必须在 `fn strategy()` 内部以单个顶层语句出现
- 在通用语句解析器被诚实扩展之前，不要将此调用拆分为多行

### 必需的定位参数

- `profile_id: string`

第一个版本仅允许：

- `"paper"`

### 允许的关键字字段

- `fee_bps`
  - 浮点数
  - 必须 `>= 0`
  - 一对一映射到运行时 `taker_fee_bps`
  - 投影回前端执行节点配置 `fee_bps`
- `slippage_bps`
  - 浮点数
  - 必须 `>= 0`
  - 一对一映射到运行时 `default_slippage_bps`
  - 投影回前端执行节点配置 `slippage_bps`

### 默认值

如果字段被省略，合约回退到当前图/运行时编译路径为 `builtin.execution.paper` 使用的相同默认值：

- `fee_bps = 10.0`
- `slippage_bps = 5.0`

此默认值现在在以下各处共享：

- 正式 QuantScript 降级
- 图运行时编译
- Strategy IR 降级

## 语义

第一个 `execution.profile(...)` 合约**不**引入新的执行引擎。它仅选择和参数化现有的纸面执行模块。

这意味着：

- 无内联执行表达式
- 无经纪商路由语言
- 无场地切换策略
- 无来自 QuantScript 主干的逐订单执行覆盖
- 无自定义执行插件选择

编译结果仍然降级到现有的运行时执行形态和现有的前端/运行时模块键：

- `builtin.execution.paper`

## 范围外

第一个合约不得包含：

- `mode`
- `order_type`
- `time_in_force`
- `slippage_model`
- `latency_assumption_ms`
- `capital_base`
- 自定义执行模块选择
- 经纪商特定执行行为

这些可能在以后重新审视，但它们不是第一个最小 profile 合约的一部分。

对于下一个 `P4` 回测切片，本文档应与[回测执行假设最小合约](./guide-backtest-execution-assumptions-minimal-contract.md)一起阅读。

该计划中的划分是：

- `execution.profile(...)` 拥有可复用的策略默认值
- 回测请求拥有运行范围的覆盖

`latency_ms` 应在第一个回测假设切片中保持请求范围，而不是进入主干 QuantScript 语法。

## 跨入口对齐规则

此合约只有在相同语义形态可以在以下各处表达时才有价值：

- 正式 QuantScript
- 图运行时编译
- Strategy IR

对于第一步，该共享语义形态简单来说就是：

- 一个纸面执行 profile
- 一个可选的滑点设置
- 一对一降级到 `builtin.execution.paper`

## 建议的诊断信息

当实现开始时，诊断信息应保持狭窄且面向产品：

- 不支持的 `profile_id`
- 不支持的关键字字段
- 非数值 `fee_bps`
- 负值 `fee_bps`
- 非数值 `slippage_bps`
- 负值 `slippage_bps`
- 重复的执行 profile 声明

当这些成为稳定的产品合约时，不要泄漏底层辅助函数或解析器错误字符串。

## 当前状态

第一个编译路径实现现已跨以下路径落地：

- 正式 QuantScript
- 图运行时编译
- Strategy IR

当前边界：

- 仅支持 `profile_id="paper"`
- 仅支持 `fee_bps` 和 `slippage_bps`
- 降级目标仍是 `builtin.execution.paper`
- 此合约不替换 Strategy IR 中更广泛的 `execution` 块；它为当前运行时提供更窄的 profile 形态路径
- 能力输出尚未针对 profile 特定报告进行扩展；当前落地范围是编译/运行时降级加跨入口测试
- `latency_ms` 有意保持在 `execution.profile(...)` 之外，属于第一个 `P4` 切片的回测请求层
