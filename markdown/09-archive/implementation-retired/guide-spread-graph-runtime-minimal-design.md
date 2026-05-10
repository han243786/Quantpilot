# 价差图运行时最小设计

本文档定义了图/运行时编译路径上价差的第一个实现设计。

它依赖于[价差最小合约](./guide-spread-minimal-contract.md)。它不扩大该合约。它仅将第一个被接纳的切片转化为可执行的图/运行时设计目标。

根据当前代码状态，此处描述的第一个仅图/运行时切片现已针对窄 `bps` 加单侧阈值路径实现。因此，本文档既作为设计边界，也作为该第一个切片的当前落地范围描述。

## 目标

图/运行时路径上的第一个可执行价差切片是：

- 恰好两个输入
- 报价/序列价差，带有显式的 `asof` 策略
- 仅 `bps` 输出
- 仅单侧阈值条件

规范目标：

```text
spread_bps(left, right, align=backward, tolerance_ms=1000) > 5
```

这仍然比通用的价差策略语言更窄。

## 为什么图/运行时优先

图/运行时编译已经是代码库中最具体的价差锚点：

- `builtin.intent.spread_observer` 已存在
- 图配置已携带 `max_time_diff_ms`
- 图配置已携带 `align_direction_code`
- 图配置已携带 `spread_output_code`
- 编译器/运行时已经知道如何构建双输入 `SpreadSpec`

这使得图/运行时成为价差切片的诚实第一个可执行真实数据源。

正式 QuantScript 和 Strategy IR 应在以后跟随此形态，而非首先发明自己的更宽变体。

## 当前实现事实

今天图/运行时路径仅部分与期望的产品形态对齐：

- 图编译已将 `builtin.intent.spread_observer` 映射到 `IntentKind::QuoteObserve`
- Core IR 降级已经可以发出 `CoreIndicatorKind::Spread` 加 `SpreadSpec`
- `SpreadSpec` 已携带：
  - 两个序列输入
  - `AlignAsofSpec`
  - 输出类型
  - 可选的重采样/窗口配置

但当前产品措辞仍然是诚实的：后端仅暴露 `QuoteObserve` 语义，尚未完全产品化的价差策略通道。

因此，此设计不再是纯前瞻性的。窄的仅图/运行时切片现已实现，但整个价差通道仍未完成。

## 最小可执行形态

第一个图/运行时实现应仅接受此窄形态：

- 模块键：`builtin.intent.spread_observer`
- 恰好两个 `input_refs`
- `spread_output_code = bps`
- `align_direction_code` 在 `{ backward, forward, nearest }` 中
- `max_time_diff_ms > 0`
- 单侧阈值比较

阈值本身应以当前图/运行时路径表示单侧 `RSI`、`momentum` 和 `zscore` 的相同方式表示：

- `comparison_shape_code`
- `comparison_op_code`
- `comparison_threshold`

对于第一个切片：

- 被接纳的 `comparison_shape_code`：`buy`
- 被接纳的操作符：`>` 或 `>=`
- 被接纳的阈值单位：`bps`

这样得到一种稳定的可执行含义：

- "当以 bps 为单位的对齐价差超过阈值时，做多/观察-正信号"

## 配置合约

第一个图/运行时配置合约应为：

- `max_time_diff_ms`
- `align_direction_code`
- `spread_output_code`
- `comparison_shape_code`
- `comparison_op_code`
- `comparison_threshold`

第一个设计不应添加新的仅图价差字段，除非严格必要。

当前价差模块已有其他字段，例如：

- `field_code`
- `resample_period_ms`
- `resample_agg_code`
- `window_size`
- `window_agg_code`

这些现在应保持实现辅助性质，而非产品前沿声明。

对于第一个可执行切片：

- 保持 `field_code` 在当前默认路径上
- 不要将重采样/窗口语义扩展到第一个面向用户的价差合约中

## 时间对齐策略

图/运行时实现必须使时间对齐策略显式化。

第一个版本应强制：

- 连接模式：`asof`
- 方向来自 `align_direction_code`
- 容差来自 `max_time_diff_ms`

行为规则：

- 如果在容差内不存在对应点，则不产生价差样本
- 没有隐式的零填充
- 没有无界的向前结转

建议的第一个默认值：

- `align_direction_code = backward`
- `max_time_diff_ms = 5000`

这些默认值已匹配当前图/运行时价差形态配置，且足够窄以保持第一个合约诚实。

## 输出策略

第一个图/运行时实现应仅接纳：

- `spread_output_code = bps`

如果其他输出代码因兼容性仍在内部被接受，它们不应被描述为第一个产品化的价差切片。

这意味着实现应清晰区分：

- 兼容性面
- 产品接纳切片

## Core IR 目标

第一个图/运行时切片应降级为：

- `CoreIndicatorKind::Spread`
- `SpreadSpec { left, right, align, output=bps, ... }`
- 结构化的单侧阈值 `ScalarExpr::Compare`

设计目标是：

- 不要停留在 `describe_runtime_intent_condition(...)`
- 如果图配置已匹配被接纳的窄形态，不要将第一个切片留在原始条件文本上

这是价差的关键共享核心步骤。

## 验证规则

第一个图/运行时实现应在验证或编译时拒绝以下情况：

- 少于或多于两个输入
- `spread_output_code` 不等于 `bps`
- 缺少或非正 `max_time_diff_ms`
- 不支持的 `align_direction_code`
- 缺少阈值元数据
- 非买入/非单侧阈值形态

设计应优先选择显式拒绝，而非静默降级到更广泛的 `QuoteObserve` 故事。

当前已落地的编译护栏：

- `QPSPREAD001` 用于阈值切片上的非 `bps` 输出
- `QPSPREAD002` 用于缺少或非正 `max_time_diff_ms`
- `QPSPREAD003` 用于非单侧或不完整的阈值元数据

## 能力和措辞规则

在第一个切片实现和测试之前：

- 将 `builtin.intent.spread_observer` 描述为部分/受保护能力
- 不要宣传"真正的价差策略支持"

在第一个切片实现之后：

- 能力输出和 UI 措辞可仅描述窄的被接纳切片
- 仍不得声称通用的价差 DSL 或套利引擎

## 必需的测试

第一个图/运行时实现直到以下所有测试存在才算完成：

1. 图/运行时编译成功测试，针对双输入 `bps` 单侧阈值
2. 图/运行时编译拒绝测试，针对非 `bps` 输出
3. 图/运行时编译拒绝测试，针对缺少/无效的对齐容差
4. 图/运行时编译拒绝测试，针对非单侧阈值形态
5. Core IR 断言条件为结构化比较，而非仅原始文本

图/运行时和 Strategy IR 之间现已存在针对相同窄 `bps` 阈值切片的跨入口等价测试。正式 QuantScript 尚不接纳该切片，因此尚不能做出三入口等价声明。

## 有意不在此设计中

此第一个图/运行时实现不包括：

- 正式 QuantScript 价差接纳
- Strategy IR 价差接纳
- 双侧价差阈值
- 价差线对线比较
- 比率/绝对价差作为产品化的一等输出
- 场地策略
- 多腿套利
- 通用价差脚本

## 此设计后的下一步

一旦此图/运行时设计被接受，下一个具体的实现步骤应是：

1. 保持图/运行时切片窄且有测试覆盖
2. 保持图/运行时与 Strategy IR 等价护栏到位
3. 只有在那时才决定正式 QuantScript 是否应为下一个采纳入口点
