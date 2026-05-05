# 价差正式 QuantScript 接纳合约

本文档定义了正式 QuantScript 中价差的第一个诚实接纳合约。

该合约现在是第一个价差切片的已落地正式 QuantScript 面。它不将产品边界扩大到现有图/运行时和 Strategy IR 切片之外。

它依赖于以下文档：

- [价差最小合约](./guide-spread-minimal-contract.md)
- [价差图运行时最小设计](./guide-spread-graph-runtime-minimal-design.md)
- [价差 Strategy IR 最小合约](./guide-spread-strategy-ir-minimal-contract.md)

## 目的

图/运行时和 Strategy IR 现在已接纳相同的窄价差切片：

- 恰好两个输入
- 显式的 as-of 对齐
- 正容差
- `bps` 输出
- 单侧 `>` / `>=` 阈值

正式 QuantScript 现在是同一个窄切片的第三个采纳者。

本文档冻结唯一被接纳的辅助函数形态。

## 最小接纳目标

正式 QuantScript 仅接纳这种窄形式：

```qs
let left_aligned = align_asof(left, direction="backward", tolerance_ms=1000)
let right_aligned = align_asof(right, direction="backward", tolerance_ms=1000)
let s = spread(left_aligned, right_aligned, output="bps")
if s > 5 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

或者等价的内联形式：

```qs
if spread(
    align_asof(left, direction="backward", tolerance_ms=1000),
    align_asof(right, direction="backward", tolerance_ms=1000),
    output="bps"
) > 5 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

两种形式必须与现有的图/运行时和 Strategy IR 切片含义完全相同。

## 接纳要求

正式 QuantScript 应仅在所有以下条件仍然成立时接纳价差：

1. 辅助函数仍表示恰好两个输入。
2. `output="bps"` 仍然是第一个切片唯一被接纳的产品输出。
3. 对齐的价差输入通过 `align_asof(...)` 显式构建，且该对齐仍由 `tolerance_ms` 界定。
4. 阈值仍为单侧 `>` 或 `>=`。
5. 由此产生的降级路径仍生成图/运行时和 Strategy IR 已使用的相同 Core IR 形态。
6. 快乐路径不依赖于将匹配器恢复扩展到当前价差辅助函数边界之外。

如果其中任何一项不再成立，应重新考虑此已落地的接纳，而非静默扩大。

## 显式辅助函数形态

第一个被接纳的正式 QuantScript 辅助函数面必须保持狭窄：

- 外部比较目标：`spread(...)`
- `spread(...)` 的位置参数：恰好两个对齐的源表达式
- `spread(...)` 的必需关键字参数：
  - `output="bps"`
- 两个输入上的必需对齐包装：
  - `align_asof(target, direction="backward", tolerance_ms=<正整数>)`
- 被接纳的比较：
  - `spread(...) > <数字>`
  - `spread(...) >= <数字>`
- 被接纳的操作侧：
  - 当前已落地的辅助函数路径仍然通过单侧条件 `emit Intent(...)` 流动，并降级到现有的 `QuoteObserve` 运行时意图形态，而不扩大价差合约

第一个接纳不得暗示：

- 双侧多头/空头价差逻辑
- `<` / `<=` 卖出风格价差接纳
- 比率或绝对输出
- 价差线对线比较
- 自定义价差算术
- 超过两个输入

## 时间对齐策略

正式 QuantScript 在此合约被接纳时，不得将对齐策略隐藏在模糊的辅助函数默认值后面。

对于第一个切片：

- `align_asof(...)` 必须在合约示例和测试中保持显式
- `tolerance_ms` 必须在两个对齐操作数上存在且为正
- 容差内缺少对应点仍意味着价差样本缺失
- 无静默零填充
- 无超越容差的静默向前结转

如果实现方便使用了内部默认值，文档和测试仍必须描述显式的产品规则。

## 输出策略

第一个正式 QuantScript 价差接纳仅包括：

- `output="bps"`

以下内容保持范围外：

- `output="ratio"`
- `output="absolute"`

这精确反映了当前图/运行时和 Strategy IR 合约。

## 降级目标

第一个被接纳的正式 QuantScript 价差切片必须降级到其他两个已落地入口点已使用的相同共享核心形态：

- `CoreIndicatorKind::Spread`
- `SpreadSpec { left, right, align, output=bps, ... }`
- `ScalarExpr::Compare`

该比较必须在结构上等价于：

```text
spread_ref > threshold
```

或

```text
spread_ref >= threshold
```

如果正式 QuantScript 不能诚实地降级到相同形态，则不应被接纳。

## 拒绝要求

正式 QuantScript 当前拒绝：

- 少于或多于两个输入的价差辅助函数
- 任一对齐操作数上省略或非正的 `tolerance_ms`
- 除 `bps` 外的任何 `output`
- 非单侧阈值形态
- 格式错误的辅助函数调用，否则会回退到宽泛的匹配器恢复

第一个被接纳的正式路径应优先使用显式结构化诊断，而非通用的辅助函数参数失败。

## 当前实现事实

当前代码状态：

- 图/运行时已接纳窄 `bps` 单侧阈值切片
- Strategy IR 已接纳相同窄切片
- 图/运行时和 Strategy IR 已有该切片的跨入口等价护栏
- 正式 QuantScript 现在作为第三个落地入口点接纳相同的窄价差切片
- 被接纳的正式价差辅助函数现在降级到图/运行时和 Strategy IR 已使用的相同结构化价差比较形态
- 正式路径中格式错误或未被接纳的价差辅助函数形态仍然归入现有的结构化 `QPQSLOW001` 合约，而非专用的价差接纳合约
- 这些正式拒绝路径现在也有针对非 `bps`、缺少 `align_asof(...)`、非正 `tolerance_ms` 和非单侧阈值情况的 golden-like API 响应形态覆盖

因此，此合约现已是一个落地能力边界，而非仅未来的计划。

## 落地的实现顺序

落地的顺序是：

1. 保持辅助函数形态与现有图/运行时和 Strategy IR 切片完全同构
2. 为窄辅助函数形式添加显式的接纳和拒绝规则
3. 仅将该被接纳的辅助函数形式降级到相同的结构化价差比较
4. 添加跨正式 QuantScript、图/运行时和 Strategy IR 的三入口等价测试
5. 将 `ratio`、`absolute` 和非单侧价差形式保持在被接纳面之外

## 使用规则

不要将正式 QuantScript 价差描述为广泛的价差语言特性。

当前应仅描述为：

- 支持与图/运行时和 Strategy IR 已落地相同的窄双输入 `align_asof(...) + spread(..., output="bps") + 单侧 >/>=` 切片
- 通过现有的结构化 `QPQSLOW001` 合约拒绝更广泛的价差辅助函数形态
