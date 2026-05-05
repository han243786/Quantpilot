# 价差正式 QuantScript 最小实现计划

本文档将已接纳的价差辅助函数边界转化为正式 QuantScript 的最小诚实实现计划。

它不扩大合约。它记录了已为已收窄的辅助函数形态落地的实现：

- 两个输入上的 `align_asof(...)`
- `spread(..., output="bps")`
- 单侧 `>` / `>=` 阈值

它依赖于以下文档：

- [价差最小合约](./guide-spread-minimal-contract.md)
- [价差 Strategy IR 最小合约](./guide-spread-strategy-ir-minimal-contract.md)
- [价差正式 QuantScript 接纳合约](./guide-spread-formal-admission-contract.md)

## 实现目标

使正式 QuantScript 成为已落地价差切片的第三个采纳者，而无需更改辅助函数语言。

目标可执行形态是：

```qs
if spread(
    align_asof(left, direction="backward", tolerance_ms=1000),
    align_asof(right, direction="backward", tolerance_ms=1000),
    output="bps"
) > 5 {
    emit Intent("OBSERVE", instrument="BTCUSDT")
}
```

目标降级形态是图/运行时和 Strategy IR 已使用的相同形态：

- `CoreIndicatorKind::Spread`
- `SpreadSpec`
- `ScalarExpr::Compare`

## 非目标

在此步骤中不要实现以下任何内容：

- 新的价差辅助函数语法
- `spread(..., align=..., tolerance_ms=...)` 直接参数接纳
- `output="ratio"`
- `output="absolute"`
- `<` / `<=` 卖出风格价差阈值
- 双侧价差规则
- 价差线对线比较
- 自定义价差算术
- 三输入或 N 输入价差

## 最小代码变更集

最小诚实实现应分三个阶段进行。

### 阶段 1. 显式接纳门禁

负责人：

- `quantscript/src/lowering/intents.rs`

目标：

- 继续使用现有的价差辅助函数面
- 在被接纳切片之外的所有内容看起来像受支持的产品能力之前拒绝它们

必需的检查：

1. 价差表达式必须来自 `match_explicit_spread_call(...)`
2. 恰好两个输入
3. 两个操作数必须已携带显式的 `align_asof(...)` 元数据
4. 两个操作数必须在以下方面一致：
   - `align_direction_code`
   - `tolerance_ms`
5. `tolerance_ms` 必须为正
6. `output` 必须解析为 `SpreadOutputKind::Bps`
7. 关系必须是 `>` 或 `>=`
8. 阈值必须为数值
9. 操作侧必须保持在被接纳的单侧观察路径内

实现说明：

- 不要放宽现有的解析器或辅助函数解码逻辑
- 不要添加新的回退匹配器
- 不要在被接纳的正式路径上保留更广泛的比率/绝对价差形态

### 阶段 2. 结构化比较桥接

负责人：

- `quantscript/src/lowering/intents.rs`

目标：

- 一旦窄价差切片被接纳，携带图/运行时和 Strategy IR 已使用的相同阈值比较桥接元数据

生成的运行时意图上必需的参数：

- `spread_output_code = 1`
- `comparison_shape_code = 1`
- `comparison_op_code = 2 or 3`
- `comparison_threshold = <数字>`

保留：

- `IntentKind::QuoteObserve`
- 现有的 `SpreadSpec` 形态运行时参数，例如
  - `align_direction_code`
  - `max_time_diff_ms`
  - field/window/resample 参数

不要：

- 仅依赖 `spread_trigger_bps` 作为被接纳切片
- 在比较桥接参数缺失时声称结构化比较接纳

### 阶段 3. 三入口等价护栏

负责人：

- `src/main.rs`

目标：

- 证明正式 QuantScript 现在将相同的被接纳价差切片降级到图/运行时和 Strategy IR 已使用的相同 Core IR 条件形态

添加：

1. 一个被接纳辅助函数形式的正式成功测试
2. 一个未被接纳形态的正式拒绝测试
3. 一个比较以下三者的跨入口等价测试：
   - 正式 QuantScript
   - 图/运行时编译
   - Strategy IR

等价视图应保持与其他共享核心切片相同的规范化规则：

- 允许命名差异，如引用名称或数据 ID
- 不隐藏条件形态差异

## 落地的实现顺序

落地的实现按以下顺序进行：

1. 添加显式接纳门禁
2. 添加未被接纳价差形态的拒绝测试
3. 在被接纳路径上添加结构化比较桥接参数
4. 添加一个证明结构化比较降级的正式成功测试
5. 添加三入口等价护栏
6. 仅在以上所有完成后，将路线图措辞从"计划接纳合约"更新为"已落地第三采纳者"

## 建议的初始拒绝集

初始拒绝集应保持较小且与合约直接对齐：

1. `output="ratio"`
2. 一个或两个操作数上缺少 `align_asof(...)`
3. 非正 `tolerance_ms`
4. `<` 或 `<=`
5. 当前仍归入 `QPQSLOW001` 的格式错误辅助函数形态

除非当前结构化降级合约确实需要，否则不要发明新的正式价差诊断系列。

## 可能更改的文件

预期的产品代码更改：

- `quantscript/src/lowering/intents.rs`
- 可能 `src/main.rs` 仅用于正式编译端点测试

代码落地后的预期文档更改：

- `markdown/guides/quantscript/guide-spread-formal-admission-contract.md`
- `markdown/guides/quantscript/guide-spread-minimal-contract.md`
- `markdown/guides/quantscript/guide-quantscript-first-lowering-split-patch-plan.md`
- `markdown/overview/overview-current-status-and-roadmap.md`
- `markdown/guides/quantscript/guide-formal-quantscript-syntax.md`

## 接纳规则

此计划仅在以下所有条件满足时才算完成：

1. 正式 QuantScript 仅接纳接纳合约中描述的精确窄辅助函数面
2. 正式 QuantScript 显式拒绝未被接纳的价差形态
3. 被接纳的正式路径降级到图/运行时和 Strategy IR 使用的相同结构化价差比较
4. 三入口等价护栏为绿色
5. 文档已更新，说明正式 QuantScript 现在是窄价差切片的第三个采纳者

这些条件现已满足窄正式价差切片的条件。
