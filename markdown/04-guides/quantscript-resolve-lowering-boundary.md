# QuantScript Resolve 与 Lowering 边界

## 目的

本文档冻结了 `quantscript` 中 `resolve` 和 `lowering` 之间当前的第三周边界。

关于旧的单文件 `lowering.rs` 的首次机械拆分，请参阅：

- [QuantScript 首次 Lowering 拆分补丁计划](./quantscript/guide-quantscript-first-lowering-split-patch-plan.md)

当前代码状态：

- 旧的单文件 `quantscript/src/lowering.rs` 已被 `quantscript/src/lowering/mod.rs` 替换
- `context`、`shared`、`diagnostics` 和 `universe` 已被提取
- `semantic` 现在持有别名感知的语义桥接辅助函数，由 `bindings`、`fallback` 和 `intents` 共享
- `binding_sources` 现在拥有源推断和解码辅助函数
- `source_recovery` 现在拥有之前混入 `fallback` 的变更源重建辅助函数
- `bindings` 现在专注于指标绑定组装，而 `intents` 仍然是专用的 lowering 模块
- `fallback` 现已提取到 `quantscript/src/lowering/fallback.rs`
- 底层匹配器测试已从 `orchestrator` 移出到所属的 lowering 模块中
- `binding_sources` 不再依赖 `fallback`，辅助函数指标组装已移回 `bindings`，仅价差的操作数/匹配类型已拉入 `intents`，`helper_env` 现在拥有狭窄的辅助函数环境填充以及 `bindings` 和 `source_recovery` 使用的共享 stmt-binding 遍历，而 `fallback` 现在暴露更薄的手动公式门面而非广泛的匹配器列表，但 `source_recovery`、`bindings`、`fallback` 和 `intents` 仍然共享几个内部辅助函数界面，因此结构拆分领先于最终的耦合清理
- `source_recovery` 不再直接访问 `bindings::collect_bindings_from_stmts`；辅助函数体本地绑定恢复现在通过更狭窄的 `helper_env` 界面进行，而非依赖完整的 `bindings` 收集路径
- 剩余的通用缺失参数辅助函数字符串现在主要局限在 `shared.rs`；最近的清理传递已将 `universe`、`binding_sources` 和 `intents` 中稳定、用户可见的失败拉回到结构化 `QPQSLOW` 合约之后
- 第一个最小的 `risk.profile(...)` 合约现已一对一地在正式 QuantScript、图运行时编译和 Strategy IR 中实现；它有意暂时绕过 `resolve` 语义，直接降低到现有的 `builtin.risk.global` 运行时形态
- 价差现在也有正式的极小合约文档，冻结了时间对齐和第一个预期的共享核心切片；图/运行时、Strategy IR 和正式 QuantScript 现在都已落地相同的狭窄 `bps` 单边阈值切片，而更广泛的价差运算和辅助函数派生形式仍保持在已接纳的共享核心路径之外

目标很简单：

- 保持 `resolve` 作为稳定语义事实的所有者
- 保持 `lowering` 作为运行时绑定构造的所有者
- 防止 `resolve` 漂移成为第二个完整的 lowering 管道

## Resolve 拥有

`resolve` 负责稳定、可重用且不需要完整运行时上下文的语义信息。

- 名称解析
- 基础类型推断
- builtin/imported/helper 分类
- 成员能力分类
- 调用风格和成员风格辅助函数的统一返回类型规则
- `ResolveResult.expr_semantics` 用于标准化表达式语义

今天，`ResolveResult.expr_semantics` 已产生的稳定语义包括：

- `SeriesView`
- `WindowAggregateView`
- `BoundaryLookbackPair`
- `BalancedSmoothedChangePair`
- `ManualIndicatorFormula::Momentum`
- `ManualIndicatorFormula::MovingAverage`
- `ManualIndicatorFormula::ZScore`
- `ManualIndicatorFormula::MacdLine`
- `ManualIndicatorFormula::MacdHistogram`

## Resolve 不拥有

`resolve` 不得成为重新实现完整运行时 lowering 的地方。

它不拥有：

- 运行时绑定构造
- 数据源绑定构造
- 指标绑定构造
- 执行 Intent 构造
- 复杂的跨语句数据流
- 递归或无界辅助函数展开
- 完整策略级 lowering

`resolve` 中的别名感知识别有意保持有限。

- 只允许最少的 let-to-expr 别名跟踪
- 别名恢复仅用于稳定已知值得标准化的语义事实
- 一旦某个模式需要更广泛的流推理、递归展开或运行时特定解释，它就留在 `lowering` 中

## Lowering 拥有

`lowering` 负责将已解析的语义事实转化为面向运行时的绑定和配置。

它拥有：

- 运行时绑定的源恢复
- 运行时指标绑定构造
- 运行时配置的数据源推断
- 预热和窗口传播到面向运行时的输出
- 针对非标准化或更复杂形式的回退匹配器路径
- 最终降低到 Core IR 或运行时配置
- 当条件无法映射到支持的运行时 Intent 形态时，显式拒绝条件性 `emit Intent(...)` 语句
- 已知可执行合约故障的结构化降低诊断，如不支持的条件性 `emit Intent(...)` 形态

当稳定的语义注解可用时，`lowering` 应首先消费它，只做最低限度的剩余提取工作。

当没有稳定的注解可用时，`lowering` 仍可使用本地匹配器和回退逻辑。

当前模块形态：

- `orchestrator` 拥有顶层 lowering 流程
- `universe` 拥有编译时 universe 展开和再平衡指令恢复
- `semantic` 拥有 resolve-to-lowering 语义桥接辅助函数和别名感知表达式定位
- `binding_sources` 拥有源推断、fetch/source 恢复和解码辅助函数
- `source_recovery` 拥有源推断和匹配器层共享的变更源重建辅助函数
- `bindings` 拥有运行时指标绑定组装
- `fallback` 拥有匹配器繁重的兼容性恢复路径
- `intents` 拥有面向运行时的 Intent 构造

## RSI 边界

RSI 是当前有意拆分边界的示例。

已移入 `resolve`：

- `BalancedSmoothedChangePair { period, smoothing }`

仍保留在 `lowering` 中：

- 外层 RSI 公式外壳
- 最终的 `RsiMethod` 映射
- 围绕完整公式形态的剩余运行时导向恢复

这是有意为之。核心稳定参数层在 `resolve` 中标准化；外层外壳保留在 `lowering` 中，直到它变得同样稳定并值得共享。

## 剩余的回退层

`lowering` 中剩余的回退匹配器并不都是相同的。它们应在两个不同层中处理。

### 永久运行时回退

除非产品边界本身发生变化，这些应保留在 `lowering` 中。

- 外层 RSI 外壳匹配器：`match_manual_rsi_formula`、`match_rsi_rs_pair`、`match_rs_pair_from_denominator`、`match_rs_pair_expr`
- 依赖运行时解释的源恢复辅助函数：`balanced_smoothed_change_pair_source`
- `source_recovery` 中的变更源重建辅助函数：`gain_loss_source_binding`、`guarded_abs_change_source`、`clamped_change_source`、`guarded_change_source`、`oriented_change_source`
- `binding_sources` 中的解码侧恢复辅助函数：`decode_smoothed_change_binding`

它们留在这里是因为它们依赖面向运行时的源恢复、符号/方向解释或公式外壳确认，而非稳定的可重用参数事实。

已知的可执行合约故障现在应尽可能以结构化编译诊断的形式浮现。当前示例包括：

- `QPQSLOW001` 用于不支持的条件性 `emit Intent(...)` lowering，包括不再泄漏通用缺失参数辅助函数错误的形式不正确的价差辅助函数形态
- `QPQSLOW004` 用于不支持的运行时 `Intent` 操作
- `QPQSLOW007` 当正式 lowering 无法推断任何可达的 `fetch(...)` 或 `get_data(...)` 源时
- `QPQSLOW009` 用于不支持的再平衡 `every=...` 值
- `QPQSLOW010` 当使用依赖快照的 universe 操作而没有 `universe_snapshot` 时
- `QPQSLOW012` 用于不支持的 universe 排序顺序
- `QPQSLOW013` 当 `rebalance(...)` 缺少其分配辅助函数或未收到支持的分配辅助函数调用时
- `QPQSLOW014` 当再平衡分配辅助函数缺少其选择输入或未收到 universe 值的选择时
- `QPQSLOW015` 当再平衡分配解析为空交易对集合时
- `QPQSLOW016` 用于固定权重计数与选定 universe 不匹配
- `QPQSLOW017` 用于负固定权重
- `QPQSLOW018` 用于总和为零的固定权重
- `QPQSLOW019` 用于不支持的 `rank_weight(..., method=...)` 值
- `QPQSLOW020` 用于不支持的 `score_weight(..., normalize=...)` 值
- `QPQSLOW021` 当 `weights=...` 缺失或不是数字列表字面量时
- `QPQSLOW022` 当指标辅助函数（如 `rsi`、`macd`、`momentum` 或 `zscore`）缺少其第一个参数或未在那里接收到 fetch/get_data 源时
- `QPQSLOW023` 当指标 period/lookback/window 参数缺失、非数字或不大于零时
- `QPQSLOW024` 当移动平均辅助函数缺少源输入、未接收到 fetch/get_data 源，或 `ema(...)` 未接收到已识别的 MACD 线时
- `QPQSLOW025` 当 universe 辅助函数（如 `filter/sort_by/top`）缺少 universe 输入或未接收到 universe 值输入时
- `QPQSLOW026` 当 `symbols(...)` 缺少其列表输入或未接收到列表字面量时
- `QPQSLOW027` 当 `symbols([...])` 包含非字符串项时
- `QPQSLOW028` 当 `top(...)` 未接收到数字计数参数时
- 直接单源移动平均比较现在首先针对共享的 Core IR 辅助函数进行验证，成功的 lowering 发出结构化的 `ScalarExpr::Compare` 而非仅保留原始条件文本
- 直接单边 RSI 阈值比较现在也重用共享的 Core IR 辅助函数，并发出结构化的 `ScalarExpr::Compare`；双边 RSI 形态仍保留在原始文本路径上，因为当前运行时 Intent 合并不保留两个单独的 RSI 谓词
- 直接单边 `momentum` 和 `zscore` 阈值比较现在也重用共享的 indicator-threshold compare 辅助函数；lowering 为单边路径保留有符号的比较阈值，并在相反侧分支合并到同一运行时 Intent 时显式丢弃结构化的比较元数据

### 过渡性回退

这些今天仍是本地匹配器，但它们并不都具有相同的提升就绪度。

优先考虑的提升候选：

已提升到 resolve-first 路径：

- `match_zscore_operands`
  主路径现在通过 `ResolvedManualIndicatorFormula::ZScore` 加上 resolve-first target/span 辅助函数。只有旧的三操作数兼容性尾部保留在 `lowering` 中本地。
- `match_manual_moving_average_window`
  主路径现在通过 `ResolvedManualIndicatorFormula::MovingAverage` 加上 resolve-first target/span 辅助函数。只有旧的拆分 `sum()/period` 兼容性尾部保留在 `lowering` 中本地。
- `match_sum_window_call`
  主路径现在通过 `ResolvedExprSemantic::WindowAggregateView` 加上 resolve-first target/span 辅助函数。只有旧的能力形态 AST 回退尾部保留在 `lowering` 中本地。
- `match_latest_lookback_pair`
  主路径现在通过 resolve-first 语义，优先使用 `ResolvedExprSemantic::BoundaryLookbackPair`，然后在该对已提升为动量公式时使用已标准化的 `ResolvedManualIndicatorFormula::Momentum`。只有旧的别名形态 AST 回退尾部保留在 `lowering` 中本地。

暂时保留在 lowering 中：

- `match_ema_spread`
  这仍然依赖源方向和面向运行时的 MACD 解释细节。
- `match_macd_line_signal_pair`
  这仍然承载面向运行时的线/信号方向，在该合约明确之前不应提升。
- 外层 RSI 外壳匹配器
  稳定的变化对核心已部分标准化，但最终的公式外壳和 `RsiMethod` 映射今天仍属于 lowering。

提升候选在常见情况下已经优先使用 `ResolveResult` 语义。它们剩余的本地逻辑主要作为旧版、部分标准化或别名形态表达式的兼容性路径存在。

规则是：

- 如果匹配器仅恢复稳定的参数事实，则它是提升候选
- 如果匹配器恢复运行时源身份、符号、方向或最终策略含义，则它留在 `lowering` 中

## 新 Resolve 语义的接纳规则

仅当以下所有条件都满足时，新的匹配器结果才应移入 `ResolveResult`。

1. 结果归结为小的、稳定的参数集。
2. 结果可在多个 lowering 调用点重用。
3. 结果不需要完整的运行时上下文。
4. 结果可通过有界的别名感知规则识别。
5. 结果减少了 `lowering` 中重复的 AST 形态匹配。
6. 结果由以下两者覆盖：
   - resolve 级语义注解测试
   - lowering 级消费或回退回归测试

如果某个规则不符合这些标准，则它留在 `lowering` 中。

## 实用规则

向前移动稳定的参数事实。

不要向前移动完整的运行时解释。

这就是当前的边界。
