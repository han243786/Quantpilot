# QuantScript Checklist

本清单用于日常开发时快速自检。
先看基线，再看实现事实：

- 未来方向基线：[QuantScript Trunk Baseline](./guide-quantscript-trunk-baseline.md)
- 当前实现事实：[Formal QuantScript Syntax Guide](./guide-formal-quantscript-syntax.md)

如果一个改动同时违反基线、又试图扩大当前产品口径，就不要继续推进。

## 每次开发前先问

- 这次改动是在补强主干能力，还是在把 QuantScript 推向通用语言？
- 这次改动是否属于数据、指标、受约束 universe、最小控制流、`emit Intent(...)` 这五类之一？
- 这次改动是在收敛语义，还是在增加 parser 能接受但运行不稳定的表面语法？
- 这次改动应该进主语法，还是更适合放到 `profile`、`typed custom node`、`Rust module`、`template` 或 graph/tooling 层？

## 主语法准入检查

只有下面问题全部回答“是”，才允许把能力放进主语法：

- 能稳定 lowering 到少量 canonical IR op。
- Text DSL 和 Graph View 能共享同一语义骨架。
- 能写成明确的语法、语义、诊断规则。
- 不依赖 source-level matcher 猜测最终意图。
- 能建立 lowering golden tests。
- 能建立 `text -> IR -> graph` 的 round-trip 约束。
- 不把运行时复杂度偷渡进语法层。

只要有一项回答为“否”，就不要进主语法。

## 明确允许继续建设的方向

- `fetch(...)` / `get_data(...)`、显式 `lookback`、受约束 `resample`、显式对齐
- 白名单指标，如 `sma`、`ema`、`rsi`、`macd`、`momentum`、`zscore`
- 受约束的 `symbols/universe/filter/sort_by/top/rebalance`
- 最小控制流，如 `if / else`、ternary、`&&`、`||`
- 标准化 `emit Intent(...)` 出口

## 默认不要再扩张进主语法的方向

- 风控、执行、broker 细节
- 通用持久化状态、自由跨 bar 可变变量
- `async/await`
- `while`
- 泛化 `for`
- 完整 `match`
- recursion
- `.ok()` / `.retryable()` / `.push(...)` 这类 helper-evaluator convenience 伪装成正式语言能力
- macros
- OOP / objects / methods / maps / arrays 作为通用能力
- 任意用户自定义集合、比较器、权重 DSL

这些需求优先考虑：

- `risk.profile(...)`
- `execution.profile(...)`
- `broker.profile(...)`
- typed custom node
- capability-gated plugin / Rust module
- snippets / subgraph / prompt pack
- 受控 `IR escape hatch`

## 修改 parser / resolve / analysis / lowering 时

- 有没有把“当前 parser 接受”误写成“产品正式支持”？
- 有没有新增只在 parser 层成立、但无法稳定 lowering 的语法？
- 有没有扩语法，却没有同步收紧诊断？
- 有没有把 runtime 特定逻辑继续堆进 source-level matcher？
- 有没有让 resolve 重复承担 lowering 职责？
- 有没有为新语义补齐错误消息、边界说明和失败路径？

## 修改 universe 或 rebalance 相关能力时

- 是否仍然是受约束流水线，而不是通用集合编程？
- 是否仍然保持 compile-time finite universe 的边界？
- 是否明确 point-in-time 语义、tie-break 规则、cadence 语义？
- 是否避免把动态 runtime reselection 伪装成当前正式能力？
- 是否避免把高级 portfolio policy 直接塞进 QuantScript 主语法？

## 修改指标或公式 lowering 时

- 指标是否直接映射到稳定 IR op，而不是靠后期猜测？
- 是否考虑了 warmup、历史长度、短路求值、look-ahead 风险？
- 是否避免让历史依赖指标藏在短路逻辑里失去完整推导？
- 是否为手写公式和 helper 形式都补了回归测试？

## 修改 Intent 出口时

- `Intent` 是否只表达“想做什么”，没有混入“怎么执行”？
- `size` 是否明确区分 `qty`、`notional`、`weight`？
- multi-symbol 场景里 `instrument` 是否明确？
- `confidence`、`ttl`、`metadata` 是否保持为上游语义，而不是偷运执行参数？

## 文档和产品口径检查

- README、路线图、语法指南、提示词、UI 文案是否一致？
- 是否明确区分“当前实现事实”和“未来开发方向”？
- 是否把 parser-accepted 语法误宣传为稳定能力？
- 是否把兼容面、实验面、限制面写清楚？

## 测试检查

- 是否新增或更新 lowering golden tests？
- 是否新增或更新语义诊断测试？
- 是否覆盖失败路径，而不只覆盖 happy path？
- 是否覆盖 user-facing 文档或结构化 diagnostics？
- 如果改的是 Text/Graph 共享语义，是否检查 round-trip？

## 合并前最后五问

- 这个改动有没有让 QuantScript 更像“配置加速器”，而不是“另一门做不完的新语言”？
- 这个改动有没有缩小歧义，而不是制造更多隐式语义？
- 这个改动有没有提高 IR 稳定性，而不是增加 matcher 债务？
- 这个改动有没有让文档、实现、测试更一致？
- 如果今天要向外解释这项能力，我能不能诚实、简短、稳定地描述它？
