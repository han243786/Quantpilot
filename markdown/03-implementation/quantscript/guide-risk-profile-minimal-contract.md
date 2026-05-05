# QuantScript `risk.profile(...)` 最小合约

## 目的

本文档定义了将风险配置从 QuantScript 主干语法移出并移入 profile 形态边界的最小合约。

目标不是引入一个通用的风险 DSL。目标是让 QuantScript、图运行时编译和 Strategy IR 拥有一个小的、诚实的、共享的方式来指向现有的全局风险模块，而无需将更多风险语义推入主干语言。

## 为何存在

当前产品方向是：

- 保持 QuantScript 主干专注于数据、指标、受约束的 universe 流程、最小控制流和标准化的 `emit Intent(...)`
- 将风险/执行/经纪商复杂性向外移动
- 避免将主要语法扩展为第二门研究语言

因此 `risk.profile(...)` 仅在满足以下条件时才被允许：

- 小
- 显式
- 受能力门禁控制
- 与现有运行时风险配置一一对应

## 当前运行时锚点

今天支持的运行时风险模块仍然是：

- `builtin.risk.global`

该模块上已实际存在的运行时字段是：

- `max_position`
- `max_total_leverage`
- `max_exchange_leverage`
- `min_action_interval_ms`

这些是第一个 `risk.profile(...)` 合约可能暴露的唯一字段。

## 最小合约

### 形态

第一个合约是一个单一的内建辅助函数：

```qs
risk.profile("global")
```

或者：

```qs
risk.profile("global", max_position=0.2, max_total_leverage=3.0, max_exchange_leverage=3.0, min_action_interval_ms=100)
```

当前实现说明：

- 在正式 QuantScript 路径中，`risk.profile(...)` 当前必须在 `fn strategy()` 内部以单个顶层语句出现
- 在通用语句解析器被诚实扩展之前，不要将此调用拆分为多行

### 必需的定位参数

- `profile_id: string`

第一个版本仅允许：

- `"global"`

这使合约与现有的 `builtin.risk.global` 模块保持一致，而非假装已经存在一个 profile 市场或多个风险引擎。

### 允许的关键字字段

- `max_position`
  - 浮点数
  - 必须 `> 0`
  - 映射到运行时 `RiskConfig.max_position_ratio`
- `max_total_leverage`
  - 浮点数
  - 必须 `>= 1`
  - 映射到运行时 `RiskConfig.max_total_leverage`
- `max_exchange_leverage`
  - 浮点数
  - 必须 `>= 1`
  - 映射到运行时 `RiskConfig.max_exchange_leverage`
- `min_action_interval_ms`
  - 整数
  - 必须 `>= 0`
  - 映射到运行时 `RiskConfig.min_action_interval_ms`

### 默认值

如果某个字段被省略，合约应回退到当前图/运行时编译路径为 `builtin.risk.global` 使用的相同默认值：

- `max_position = 0.2`
- `max_total_leverage = 3.0`
- `max_exchange_leverage = 3.0`
- `min_action_interval_ms = 100`

这些默认值现在在以下各处共享：

- 正式 QuantScript 降级
- 图运行时编译
- Strategy IR 降级

## 语义

第一个 `risk.profile(...)` 合约**不**引入新的风险引擎。它仅选择和参数化现有的全局风险模块。

这意味着：

- 无内联风险表达式
- 无用户定义的风险公式
- 无条件风险分支
- 无组合策略语言
- 无来自 QuantScript 主干的自定义风险插件选择

编译结果仍应降级到现有的运行时 `RiskConfig` 形态和现有的前端/运行时模块键：

- `builtin.risk.global`

## 范围外

第一个合约不得包含：

- `stop_loss_ratio`
- `take_profit_ratio`
- `max_drawdown_ratio`
- `max_trades_per_day`
- 动态的逐交易对覆盖
- 用户定义的风险谓词
- 跨 K 线的有状态风险脚本
- 自定义风险模块选择
- 经纪商特定风险行为

这些可能在以后重新审视，但它们不是第一个最小 profile 合约的一部分。

## 跨入口对齐规则

此合约只有在相同语义形态可以在以下各处表达时才有价值：

- 正式 QuantScript
- 图运行时编译
- Strategy IR

对于第一步，该共享语义形态简单来说就是：

- 一个全局风险 profile
- 一小套数值限制
- 一对一降级到 `builtin.risk.global`

如果某个增加项不能保持这种一对一降级，它就不应被接纳到第一个合约中。

## 建议的诊断信息

当实现开始时，第一个诊断信息应保持狭窄且面向产品：

- 不支持的 `profile_id`
- 不支持的关键字字段
- 非数值字段值
- 超出范围的数值字段值
- 重复的风险 profile 声明

当这些成为稳定的产品合约时，不要泄漏底层辅助函数或解析器错误字符串。

## 实现顺序

1. 在文档中正式化此合约。
2. 使其与 `builtin.risk.global` 一一对应。
3. 添加能力和编译路径支持。
4. 添加图/运行时和 Strategy IR 对齐。
5. 添加结构化诊断和往返测试。

## 当前状态

第一个编译路径实现现已跨以下路径落地：

- 正式 QuantScript
- 图运行时编译
- Strategy IR

当前边界：

- 仅支持 `profile_id="global"`
- 仅支持上述四个数值字段
- 降级目标仍是 `builtin.risk.global`
- 此合约不替换 `risk_rules`；它为当前运行时提供更窄的 profile 形态路径
- 能力输出尚未针对 profile 特定报告进行扩展；当前落地范围是编译/运行时降级加跨入口测试

## 接纳规则

第一个 `risk.profile(...)` 实现仅在以下条件下可接受：

- 它不将 QuantScript 主干扩展为通用的风险 DSL
- 它一对一降级到现有的运行时风险配置
- 图运行时编译和 Strategy IR 可以诚实地表达相同合约
- 不支持的字段显式失败
- 文档、测试和能力输出都在相同边界上达成一致
