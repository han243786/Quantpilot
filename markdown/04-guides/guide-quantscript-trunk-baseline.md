# QuantScript 主干基线

本文件把 2026-04-17 用户提供的研究报告《QuantScript 在配置加速器定位下的主干语法面报告》固化为仓库内的开发基线。

用途只有一个：约束未来的 QuantScript 设计与实现方向，避免语法面再次向“通用语言”膨胀。

如果未来的路线图、实验文档、语法提案与本文件冲突，以本文件为准；如果本文件与“当前已经实现了什么”冲突，则当前实现事实仍以 [Formal QuantScript Syntax Guide](./guide-formal-quantscript-syntax.md) 为准，但后续演化必须向本基线收敛，而不是继续放大偏离。

日常开发时，配合使用：

- [QuantScript Checklist](./quantscript-checklist.md)

## 核心定位

QuantScript 的产品定位不是通用研究语言，而是：

- 面向高频策略配置场景的受约束 DSL
- 文本 DSL 与 Graph View 共享同一 Structured IR
- 最终稳定 lowering 到少量 canonical Rust IR op
- 复杂能力后移到 profile、typed custom node、module、template 与 graph/tooling 层

一句话原则：

**能稳定 canonicalize 成少量 IR op 的，才进入主语法；不能的，不进入主语法。**

## 主干语法只保留五类能力

### 1. 数据获取与对齐

保留：

- `fetch(...)` / `get_data(...)`
- 显式 `lookback`
- 受约束的 `resample`
- 显式对齐、确认边界、时间框与字段声明

要求：

- 默认禁止隐式 lookahead
- lower timeframe 明细与复杂多源请求不进入主语法
- 多 symbol 小规模显式列表与大 universe 语义分离

### 2. 白名单指标与常见变换

保留：

- `sma`
- `ema`
- `rsi`
- `macd`
- `momentum`
- `zscore` / `z_score`
- `rolling.*` 这一类可稳定 lowering 的白名单算子

要求：

- 指标必须直接映射到稳定 IR op
- 不新增“用户自定义指标子语言”
- 依赖历史完整推进的指标，不允许被短路语义悄悄吞掉

### 3. 受约束的 universe/filter/score/top-k 流水线

保留：

- `symbols(...)`
- `universe(...)`
- `filter(...)`
- `sort_by(...)`
- `top(...)`
- 当前受限的 `rebalance(...)` 分配辅助路径

要求：

- 这是声明式候选集流水线，不是通用集合编程
- `score` 必须能稳定 lower 为逐标的标量
- tie-break、rebalance cadence、point-in-time 语义必须固定

### 4. 最小控制流

保留：

- `if / else`
- ternary
- `&&`
- `||`

要求：

- 控制流保持表达式导向
- 返回值类型必须清晰一致
- 不再把更强控制流作为主干发展方向

### 5. 信号到 Intent 的标准映射

保留：

- `emit Intent(...)`
- 明确的 `action`
- 明确的 `instrument`
- 明确的 `size` 模式
- 可选 `confidence`、`ttl`、`metadata`

要求：

- Intent 层回答“想做什么”，不回答“怎么执行”
- `size` 必须区分 `qty`、`notional`、`weight`
- 风控、订单细节、broker 语义不进入主语法出口

## 不再扩张进主语法的能力

下列能力即使 parser 目前部分可接受，也不应再作为未来主语法建设方向：

- 风控细节、执行细节、broker 细节
- 通用持久化状态与自由跨 bar 可变变量
- `async/await`
- `while`
- 通用 `for` 扩张
- `match` 的完整模式系统
- recursion
- macros
- OOP / objects / methods / maps / arrays 作为通用语言能力
- 任意用户自定义组合、比较器、权重 DSL

这些能力的替代出口应是：

- `risk.profile(...)`
- `execution.profile(...)`
- `broker.profile(...)`
- typed custom node
- capability-gated plugin / Rust module
- 受控 `IR escape hatch`
- snippets / subgraph / prompt pack

## 工程约束

未来任何 QuantScript 新能力，只有同时满足下面条件，才允许进入主语法：

1. 能 lowering 到少量稳定的 canonical IR op。
2. Text 与 Graph 可以共享同一语义骨架，而不是各自发明语义。
3. 能写成清晰的语法/语义/诊断规则，而不是依赖大量模糊 matcher 猜测。
4. 能建立对应的 lowering golden tests。
5. 能建立 text -> IR -> graph 的 round-trip 约束。
6. 不把运行时复杂性偷渡进语法层。

如果做不到，应进入 profile、node、module、template 或 tooling 层，而不是继续加语法。

## 对当前仓库的直接含义

从本文件生效起，后续 QuantScript 开发应遵守下面几条：

- 文档、路线图、语法提案都必须先对齐这五类主干能力。
- parser 已接受但不符合本基线的语法，只能视为兼容历史或过渡状态，不能继续扩大宣传口径。
- 新增能力时，优先补齐 canonical IR 契约、能力门禁和测试，而不是先放宽语法。
- `guide-formal-quantscript-syntax.md` 负责描述“现在实现了什么”；本文件负责约束“未来该往哪里收缩”。

## 推荐实施顺序

- 短期：冻结核心 IR 契约，收紧主干语法口径，补 lowering 与 round-trip 测试。
- 中期：把 risk/execution/state 需求迁到 profile、组合子和 typed custom node。
- 长期：建设 graph 编辑、模板库、prompt pack、IR escape hatch 等外层生态。

## 参考关系

- 当前实现事实：[`guide-formal-quantscript-syntax.md`](./guide-formal-quantscript-syntax.md)
- 本开发基线的来源：用户提供的研究报告《QuantScript 在配置加速器定位下的主干语法面报告》
