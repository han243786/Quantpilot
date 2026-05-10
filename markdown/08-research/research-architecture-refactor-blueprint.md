# QuantPilot 从原型到论文级策略研究平台
## 架构重构深度研究报告与实施蓝图

## 执行摘要

QuantPilot 当前已经不再是“只能跑最小策略图”的纯原型。

截至当前代码状态，系统已经具备以下真实能力：

- 后端 runtime 已稳定支持 `LongTermBuy`、`LongTermSell`、`Rsi`、`Macd`、`Momentum`、`ZScore`、`QuoteObserve`
- QuantScript 已具备 AST、`if/for/while/match`、helper function、有限符号执行、手写指标公式 lowering
- RSI 已支持 Wilder / EMA / Cutler 三类平滑路径，并能识别 helper/loop 手工构造的 gain/loss 序列
- backtest 已具备独立 API、持久化记录、详情页、equity curve、基础绩效摘要
- 前端已具备回测历史、回测详情路由与结果展示

但它距离“论文级策略研究平台”仍然有结构性差距。

当前最核心的问题，不再是“完全没有这些功能”，而是以下四类：

1. 契约真实性仍不彻底  
   `StrategyIr` 的 `IndicatorKind` 仍包含 `Spread` 和 `Custom`，但校验层会拒绝它们；前端仍暴露 `spread_observer` 和 `arbitrage` 这类超出当前真实 runtime 能力的模块。

2. 三种策略入口仍未统一到同一执行语义  
   `StrategyIr`、`QuantScript`、前端图编排仍然是三套入口，尚未统一 lower 到一个强类型、可分析、可执行的 Core IR。

3. QuantScript 已经能做很多事，但还不是“通用研究语言”  
   当前更接近“带符号分析能力的策略 DSL + 一批指标与公式模式识别”，还不是任意论文策略都能安全表达、静态分析并一致执行的语言系统。

4. backtest 已经成型，但仍是“基础研究回放系统”，不是完整研究基础设施  
   目前已有 replay、权益曲线、摘要和历史管理，但缺少更完整的订单微结构、资金语义、统计绩效体系和实验工件治理。

这份文档的目标不是推翻当前实现，而是把现状讲清楚，然后给出一条务实的重构路线：

- P0：先修契约真实性
- P1：统一 Core IR
- P2：把 QuantScript 升级成可分析执行语言
- P3：落地 Spread / 受控 Custom
- P4：把 backtest 升级成研究级实验系统

## 一、当前真实能力

### 1.1 Runtime 已真实支持的能力

当前 `IntentKind` 已真实落地：

- `LongTermBuy`
- `LongTermSell`
- `Rsi`
- `Macd`
- `Momentum`
- `ZScore`
- `QuoteObserve`

对应代码：

- `qrpc_core/src/lib.rs`
- `qrpc_runtime/src/intent_module.rs`
- `qrpc_compiler/src/lib.rs`

其中：

- `Rsi` 已支持 `smoothing_method`
- `Macd` 已支持 `fast_period / slow_period / signal_period / histogram_threshold`
- `Momentum` 已支持 `lookback / threshold_ratio`
- `ZScore` 已支持 `window / entry_z`
- `QuoteObserve` 仅是报价观察，不是价差交易策略

### 1.2 QuantScript 已真实支持的能力

当前 QuantScript 已经具备：

- AST
- `let / return / emit Intent / if / for / while / match`
- helper function
- 有限符号执行
- 部分 helper 无法安全内联时回退到 lowering 识别
- 指标与公式 lowering

当前可被真实 lower 的典型能力包括：

- 内建 `rsi(series, period)`
- 内建 `macd(series, fast, slow, signal)`
- 内建 `momentum(series, lookback)`
- 内建 `zscore(series, window)`
- 手写 MA gap ratio
- 手写 MACD histogram
- 手写 Momentum
- 手写 ZScore
- 手写 RSI canonical formula
- EMA-based RSI
- Cutler RSI
- `for` / `while` 循环构造的 gain/loss helper 再推断成 RSI

### 1.3 前端已真实支持的能力

前端当前真实可用的主线是：

- K 线节点
- Quote 节点
- 双均线意图
- 均线偏离意图
- RSI / MACD / Momentum / Z-Score 意图
- 加权代理
- 全局风控
- 模拟执行
- paper runtime
- backtest 历史与详情页

### 1.4 Backtest 已真实支持的能力

当前 backtest 已不再只是测试会话：

- 独立 `/api/runtime/backtest`
- 独立 backtest 持久化记录
- backtest 列表接口
- backtest 详情接口
- 独立前端详情路由
- `equity_curve`
- `step_count`
- `trade_count`
- `total_return_ratio`
- `max_drawdown_ratio`
- `final_equity`

这意味着系统已经具备“基础历史 replay + 权益分析 + run artifact 持久化”的雏形。

## 二、当前宣称与真实能力的差距

这一节只讨论当前仍然存在的真实差距，不重复已经修掉的问题。

### 2.1 StrategyIr 枚举层仍然超出真实能力

`IndicatorKind` 当前仍包含：

- `MaCross`
- `Rsi`
- `Macd`
- `Momentum`
- `Spread`
- `ZScore`
- `Custom`

但当前校验层只认可：

- `MaCross`
- `Rsi`
- `Macd`
- `Momentum`
- `ZScore`

也就是说：

- `Spread` 在 enum 中存在，当前已有受限 beta lowering/runtime 路径，但仍不应对外宣称为完整产品级支持
- `Custom` 在 enum 中存在，但当前 runtime 仍不支持产品级执行路径

这不是“运行时缺实现”那么简单，而是“类型层仍然在对外声称它们是合法能力”。  
如果上游系统、SDK、自动表单、文档生成器只读取 enum，就会被误导。

### 2.2 前端仍然暴露超集模块

前端模块库当前仍包含：

- `builtin.intent.spread_observer`
- `builtin.agent.arbitrage`

但当前后端真实情况是：

- `spread_observer` 只是被映射成 `QuoteObserve`
- `arbitrage` 仍在现货 beta 中被明确禁止

所以这里仍有两类契约失真：

1. “名字像 spread，真实只是 quote observe”
2. “前端看得到 arbitrage，实际不能运行”

### 2.3 QuantScript 仍然不是完整通用策略语言

当前 QuantScript 的真实定位应该是：

“一个具备有限执行语义、有限静态分析能力、且能 lower 到当前 runtime 指标族的 DSL”

它还不是：

- 任意命令式策略语言
- 任意论文公式执行环境
- 通用研究编程语言

根本原因有三点：

1. 语言语义还不完整  
   目前没有完整的赋值系统、状态模型、用户自定义数据结构与统一类型推导体系。

2. lowering 仍然大量依赖模式识别  
   现在已经很强，但核心模式仍然是“把常见公式识别成当前 runtime 已知指标”。

3. evaluator 仍然是有限符号执行  
   对发散分支、不可展开循环、无法静态归约的 helper，当前会回退或拒绝，而不是构建一个完整、通用、严格定义的执行模型。

### 2.4 Backtest 仍不是完整研究基础设施

当前 backtest 已经有基础研究价值，但距离研究级仍有明显缺口。

已有的：

- replay
- equity curve
- 基础 summary
- 历史记录
- 详情页

缺失的关键能力：

- 更完整的绩效指标族
- 订单微结构细节
- 成交价模型切换
- 延迟与执行假设版本化
- 保证金/借券/卖空正式语义
- 多资产与多市场研究
- 训练集/验证集/滚动窗口实验体系
- 统计推断与过拟合诊断

### 2.5 Spread 与 Custom 仍然不应对外宣称支持

这是当前最需要明确写死的一条。

当前不能对外宣称：

- 支持真正的 Spread 指标或 Spread 策略
- 支持真正的 Custom 指标
- 支持真正的 Arbitrage 策略
- 支持论文级完整回测

当前可以对外宣称：

- 支持一组已实现的 K 线型指标意图与基础报价观察
- 支持 QuantScript 的有限策略子集
- 支持基础 replay backtest 和结果持久化

## 三、Capability Gap Matrix

| 能力项 | 当前真实状态 | 当前对外风险 | 结论 |
|---|---|---|---|
| RSI | 已实现，runtime + compiler + QuantScript lowering 已打通 | 低 | 可正式宣称支持 |
| MACD | 已实现，runtime + compiler + QuantScript lowering 已打通 | 低 | 可正式宣称支持 |
| Momentum | 已实现，runtime + compiler + QuantScript lowering 已打通 | 低 | 可正式宣称支持 |
| Z-Score | 已实现，runtime + compiler + QuantScript lowering 已打通 | 低 | 可正式宣称支持 |
| QuoteObserve | 已实现 | 中 | 必须明确它只是报价观察 |
| Spread | enum 中存在，且已有受限 beta lowering/runtime 路径，但未形成完整产品能力 | 高 | 不要宣称正式支持 |
| Custom | enum 中存在，但 runtime 未形成可对外承诺的执行路径 | 高 | 不要宣称正式支持 |
| QuantScript helper/loop RSI 推断 | 已实现常见族 | 中 | 可宣称“支持常见手写 RSI 公式族”，不能宣称“任意自定义公式” |
| Arbitrage agent | 前端暴露，但 compile/backend 拒绝 | 高 | 不要对用户暴露为可用能力 |
| Backtest 基础回放 | 已实现 | 低 | 可宣称基础回测 |
| Research-grade backtest | 未实现 | 高 | 不要宣称研究级 |

## 四、当前代码应当如何对外表述

这一节建议作为产品文案、README、前端提示、API 文档的一致口径。

### 4.1 可以正式宣称支持

- 基于 K 线的 `RSI / MACD / Momentum / Z-Score / MA` 类策略意图
- QuantScript 的有限策略子集
- helper function 封装指标
- 常见手写公式 lowering
- 基础历史回放 backtest
- backtest 历史记录与详情页

### 4.2 可以谨慎宣称支持

- 手写 RSI 公式族  
  必须写清楚仅限当前已识别族：
  - Wilder RSI
  - EMA-based RSI
  - Cutler RSI
  - 常见 gain/loss helper
  - 常见 `for/while + push` 构造的 gain/loss 序列

### 4.3 明确不要宣称支持

- Spread 指标
- Spread 策略
- Arbitrage 策略
- 任意 Custom 指标
- 任意论文公式自动执行
- 研究级完整回测

## 五、目标架构

目标架构不应该是继续堆更多 enum、更多特殊 matcher、更多前端假模块。

应当收敛到统一 Core IR。

### 5.1 Core IR 的最小形态

建议引入四段式统一 IR：

1. `IndicatorNode IR`
2. `SeriesExpr / ScalarExpr IR`
3. `SignalRule IR`
4. `ExecutionRule IR`

三种入口统一 lower：

- `StrategyIr -> Core IR`
- `QuantScript -> Typed HIR -> Core IR`
- `Frontend Graph -> Graph IR -> Core IR`

runtime 只执行 Core IR。

### 5.2 为什么必须这么做

如果不统一：

- 每新增一个指标，要改三套入口
- 每新增一种策略表达，要写新的特判 lowering
- 前端与脚本会越来越容易语义漂移
- backtest 与 live runtime 也会继续分叉

如果统一：

- 能力边界明确
- 静态分析有统一对象
- 运行语义有统一来源
- 文档与 UI 可以直接从 capability/IR 层生成

## 六、如何基本达成宣称的全部功能

这里的“全部功能”不是一步到位实现理想平台，而是把当前宣称收敛成真实、稳定、可演进的系统。

### 6.1 P0：先修契约真实性

目标：

- 去掉所有虚标能力
- 让前端、协议、文档、runtime 完全一致

必须完成：

- 引入 capability registry 或 `/capabilities`
- 前端模块列表按 capability 渲染
- `StrategyIr` 不再单靠 enum 暗示可用能力
- `Spread / Custom / Arbitrage` 全部明确标记为未支持
- `spread_observer` 改名或隐藏

完成标志：

- 用户在任何入口都不会看到“看起来能用、其实不能跑”的功能

### 6.2 P1：统一 Core IR

目标：

- 建立统一执行语义

必须完成：

- 新建 shared `core_ir` crate
- StrategyIr 先 lower 到 Core IR
- 前端图先 lower 到 Core IR
- runtime 开始执行 Core IR

完成标志：

- runtime 不再按“来自前端图还是来自脚本”分叉逻辑

### 6.3 P2：升级 QuantScript

目标：

- 把 QuantScript 从“模式识别 DSL”升级成“可分析执行语言”

必须完成：

- Typed HIR
- `Series / Scalar / Bool / List` 类型体系
- 明确的状态与赋值语义
- window/rolling/shift 标准库
- look-ahead 检测
- warmup 检测

完成标志：

- 常见论文策略不再依赖专用 matcher 才能运行

### 6.4 P3：落地 Spread / 受控 Custom

目标：

- 把当前最明显的“虚标能力”变成真能力

Spread 最少要补：

- 多源输入
- 时间对齐
- spread 定义
- quote / mid / bid / ask 语义

Custom 最少要补：

- 受控表达式版 `CustomExpr`
- 或受控插件版 `WASM Custom`

完成标志：

- `Spread / Custom` 不再只是 enum 存在，而是真的能生成可执行 Core IR

### 6.5 P4：研究级 backtest

目标：

- 从“基础 replay”升级成“研究实验系统”

必须完成：

- 订单语义扩展
- 成交模型参数化
- 更完整绩效指标
- run artifact 版本化
- 多实验对比
- 统计诊断

完成标志：

- backtest 不只是出曲线，而是能支撑研究复现与策略对比

## 七、正式实施建议

### 7.1 当前最优先做什么

优先级建议如下：

1. 修能力契约
2. 建 Core IR
3. QuantScript 升级到 Typed HIR
4. 落 Spread / Custom
5. 升级 backtest

### 7.2 当前不应该做什么

不要继续做这些事：

- 不要继续往 `IndicatorKind` 里加未实现的种类
- 不要继续往前端模块列表里加未落地的节点
- 不要继续靠更多特判 matcher 维持长期架构
- 不要把 `QuoteObserve` 包装成“Spread”
- 不要把基础 replay backtest 文案写成“研究级回测”

## 八、Acceptance Criteria

### P0 验收标准

- 前端不再暴露未支持模块
- 文档不再宣称未实现能力
- API 能返回真实 capability

### P1 验收标准

- 三种入口都能产出统一 Core IR
- runtime 只执行 Core IR

### P2 验收标准

- QuantScript 能稳定表达当前已实现策略族
- 静态分析能发现常见 look-ahead 和 warmup 问题

### P3 验收标准

- Spread 能在数据层、编译层、runtime 层、backtest 层闭环运行
- Custom 至少有一种受控实现路径

### P4 验收标准

- backtest 结果具备更完整指标、工件与复现实验信息

## 九、建议作为正式口径保留的结论

当前最准确的对外结论应该是：

> QuantPilot 当前已支持一组基于 K 线的核心技术指标策略意图、有限但可执行的 QuantScript 策略子集，以及具备结果持久化与详情展示的基础回测系统。  
> 系统仍未正式支持 Spread、Arbitrage、任意 Custom 指标以及研究级完整回测，这些能力需要在统一 Core IR、数据语义和执行语义完成后再对外开放。

这条结论与当前代码是对齐的，也能为下一轮重构留下清晰边界。
