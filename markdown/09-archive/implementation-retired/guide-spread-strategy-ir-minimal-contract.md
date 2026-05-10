# 价差 Strategy IR 最小合约

本文档定义了价差的第一个诚实 `Strategy IR` 合约。

它有意图地依赖于以下文档：

- [价差最小合约](./guide-spread-minimal-contract.md)
- [价差图运行时最小设计](./guide-spread-graph-runtime-minimal-design.md)

它不为 `Strategy IR` 定义更广泛的价差语言。它仅允许图/运行时编译已接纳的相同窄切片。

## 目的

`Strategy IR` 是第二个应采纳价差的入口点。

它比正式 QuantScript 更适合作为第二个采纳者，因为：

- 它已有显式的指标类型和参数字段
- 它可以直接镜像图/运行时形态
- 在共享核心切片稳定之前，它不需要重新打开辅助函数驱动的文本 DSL

目标很简单：

- 让 `Strategy IR` 表达完全相同的双输入 `bps` 单侧阈值切片
- 不让 `Strategy IR` 发明更广泛的价差条件语言

## 最小被接纳切片

第一个被接纳的 `Strategy IR` 价差切片精确为：

- `IndicatorKind::Spread`
- 恰好两个输入
- `align_direction_code` 支持，当前落地实现继承图/运行时的默认 `backward` 方向（当代码省略时）
- 显式的正 `max_time_diff_ms`
- `spread_output_code = bps`
- 仅单侧阈值比较

规范目标形态：

```text
spread_signal > 5
```

其中 `spread_signal` 由以下支持：

- 两个价差输入
- `bps` 输出
- 显式的 as-of 对齐策略

阈值保持数值和单侧。

## 严格同构规则

此合约仅在与图/运行时编译保持结构同构时有效。

这意味着 `Strategy IR` 不得接纳图/运行时尚未为该切片接纳的价差特性。

对于第一个落地合约，`Strategy IR` 应在以下方面匹配图/运行时：

- 恰好两个输入
- 仅 `bps` 输出
- 与图/运行时编译相同的当前对齐方向行为
- 显式的正容差
- 单侧 `>` / `>=` 阈值语义

如果图/运行时拒绝某种形态，`Strategy IR` 也必须拒绝它。

## 输入

第一个合约要求：

- 恰好两个声明的输入
- 无隐式第三腿
- 无动态场地扩展
- 无最优腿搜索

两个输入应按与图/运行时编译相同的顺序作为左右价差操作数处理。

## 对齐策略

第一个 `Strategy IR` 价差合约继承与图/运行时相同的时间对齐策略：

- 连接模式：`asof`
- 方向：`backward`、`forward` 或 `nearest`
- 容差：`max_time_diff_ms > 0`
- 不匹配的样本缺失

`Strategy IR` 不得将这些语义隐藏在新的更高级价差策略名称后面。

如果未来的产品版本想要命名对齐预设，那应稍后添加，且仅在第一个窄合约稳定之后。

## 输出策略

第一个 `Strategy IR` 价差合约仅接纳：

- `spread_output_code = bps`

运行时/编译器对 `ratio` 或 `absolute` 的支持存在，并不使它们成为第一个被接纳的 `Strategy IR` 产品合约的一部分。

## 条件策略

第一个被接纳的条件形态是：

- 单侧阈值
- `spread_signal > threshold`
- 或 `spread_signal >= threshold`

此合约不接纳：

- 双侧合并价差规则
- `<` / `<=` 卖出风格价差阈值（第一个切片）
- 线对线价差比较
- 右侧非数值的价差阈值

重点不是这些永远不可能。重点是它们不是第一个稳定的共享核心切片。

## 必需的 `Strategy IR` 形态

第一个 `Strategy IR` 价差合约应继续使用现有的结构化面：

- `IndicatorKind::Spread`
- `indicator.inputs = [left, right]`
- `indicator.params`

必需的参数系列：

- `align_direction_code`
- `max_time_diff_ms`
- `spread_output_code`

必需的逻辑规则系列：

- 在价差信号 ID 上的单一单侧阈值比较

这避免了在 `Strategy IR` 内部发明第二种价差特定迷你语言。

## 明确不允许

第一个 `Strategy IR` 价差合约必须拒绝：

- 超过两个输入
- `spread_output_code != bps`
- 缺少或非正 `max_time_diff_ms`
- 不支持的 `align_direction_code`
- 除单侧阈值外的价差条件形态
- 自定义价差变换
- 价差线/信号风格比较
- 双侧多头/空头价差逻辑
- 在 `Strategy IR` 逻辑文本中编码的通用套利工作流

## 当前实现事实

当前代码事实应诚实描述：

- `Strategy IR` 已具有 `IndicatorKind::Spread`
- 编译器代码已可以从 `Strategy IR` 推导 `SpreadSpec`
- 第一个窄 `Strategy IR` 价差阈值合约现已针对与图/运行时同构的相同切片落地：
  - 恰好两个输入
  - `spread_output_code = bps`
  - 正 `max_time_diff_ms`
  - 在价差信号 ID 上的单侧 `>` / `>=` 阈值
- 无效形态现在显式拒绝，而非静默回退到原始条件文本
- 正式 QuantScript 仍然不接纳相同的价差切片，因此尚不能做出三入口共享核心声明

## 接纳规则

此合约仅在以下所有条件满足时才算落地：

1. `Strategy IR` 仅接受与图/运行时同构的价差切片
2. 无效形态显式拒绝
3. 产生的 Core IR 条件是结构化比较，而非仅原始文本
4. 图/运行时和 `Strategy IR` 可以通过跨入口等价测试进行检查

前四个项目现在对于相同的窄 `bps` 单侧阈值切片成立。

## 下一个实现顺序

一旦价差 `Strategy IR` 的工作开始，顺序应为：

1. 保持窄形态冻结
2. 保持显式拒绝测试为绿色
3. 保持图/运行时与 `Strategy IR` 等价测试到位
4. 只有在那之后才决定正式 QuantScript 是否应成为第三个采纳者

## 使用规则

不要将 `Strategy IR` 价差支持描述为比图/运行时价差支持更广泛。

如果图/运行时窄切片是可执行真实数据，`Strategy IR` 必须保持与该真实数据同构，直到更广泛的价差合约被显式设计和批准。
