# QuantScript Typed HIR + 诊断体系深度研究报告

## 推荐架构与端到端链路

### 面向现实项目的推荐总览

本报告的推荐架构以“统一 IR 优先、协议优先、强约束早失败”为中心：用一条可验证（verifiable）、可诊断（diagnostic-friendly）、可逐步 lowering（progressive lowering）的流水线，把 **QuantScript 文本**与**前端图编辑器**都汇入同一条语义主干（Typed HIR），再稳定 lower 到现有 Core IR。这个思路与主流编译器“AST →（name resolution 后）更编译器友好的高层 IR → 进一步 lowering”的做法一致：例如 Rust 的 HIR 明确是在解析、宏/展开以及**名称解析**之后生成、并做了去语法糖（desugaring）的“更编译器友好 AST”。citeturn10search2turn10search10turn10search18

同时，要避免持续依赖 ad hoc matcher 横向扩张能力，关键在于：**Typed HIR 必须能承载“结构化约束 + 可验证不变量 + 可诊断定位 + 可追溯 source span”**，并在每个阶段把“不可 lower / 不在 beta 边界内”的东西显式拒绝。MLIR 的经验可以借鉴：它支持多层抽象的 IR（dialect 共存）并提供 dialect conversion 框架逐步 lowering；同时强调用 verifiers 去强制 IR 结构与语义不变量，使变换 pass 能假设输入 IR 已被验证。citeturn1view1turn10search16turn10search20turn10search12

### 清晰链路定义

建议将 QuantScript 侧主链路固定为（按责任分段，而非按代码目录）：

- **Parse AST**：文本/图形两种前端都必须产出可对齐的“表层语法树（Surface AST）”或等价结构（图形可先转为 AST 等价节点集合），并保留精确 source span（文本 range 或图节点定位）。citeturn1view2turn8view0  
- **Name resolution**：把 `模块/函数/字段/内置数据源/策略入口` 统一解析为 `DefId/SymbolId`，并产生“候选集 + 失败诊断 + 相关位置”。LSP 的 `relatedInformation` 模型可直接借鉴用于“冲突/候选”展示。citeturn4view1turn4view3  
- **Type inference / type checking**：用面向 DSL 的约束求解（推荐带双向 typing 的本地推断以改善错误定位）推导每个表达式的（值类型 + qualifier：scalar/series + 领域属性：warmup/lookahead）。双向 typing 被大量实践系统用于减少注解负担并改善错误局部性（error locality），适合 DSL 的“可理解报错”目标。citeturn2search4  
- **Typed HIR**：产出**已解析名称、已定型、去语法糖、携带领域分析元数据**的 IR。此层是诊断、静态分析与 lowering 的核心锚点。Rust 的 HIR 概念（AST 的编译器友好版本）与 RFC 设计动机（把语法用途与编译用途拆开）能直接对齐你们“不要继续靠 matcher 横向扩张”的原则。citeturn10search2turn10search18  
- **Lowering to Core IR / runtime config**：Typed HIR 仅 lower 到你们统一 Core IR 所需的最小图灵完备性（甚至避免图灵完备），同时输出 runtime config（数据订阅、频率、窗口、warmup、执行时点、禁用 lookahead 的约束等）。在 lowering 前后跑 verifier，确保“非法构造早失败”。MLIR 的验证与诊断基础设施说明了：source location 及诊断引擎/notes 是保障可调试性的底座。citeturn8view0turn10search20turn10search27  

### 必须给出的分层模型：AST / Typed HIR / Core IR 的职责边界

下面的边界划分是本报告的**判断（面向 QuantPilot 的工程化建议）**，同时尽量贴近 Rust/MLIR 的“层间去语法糖、名称解析后生成 HIR、verifier 强化不变量”经验。citeturn10search2turn10search20

- **AST（表层语法层）负责**：语法结构、语法糖、token/括号/优先级、字面量、源文本 span、最小错误恢复（为了“更多错误一次报出”）。**AST 不负责**：名称绑定（DefId）、类型推断、lookahead/warmup 等领域语义。citeturn1view2  
- **Typed HIR（语义层）负责**：  
  - 名称绑定结果（每个引用点都绑定到 `DefId`）citeturn10search2  
  - 统一类型与 qualifier（scalar/series/signal/maybe）并对非法混用做早失败（或可恢复的 error node）citeturn2search4turn9view0  
  - 领域静态分析：lookahead 风险、warmup 需求、不可 lower 的构造、beta 边界拒绝（feature gating）citeturn6search0turn1view4turn9view3  
  - 诊断锚点：所有报错都能回指到主要 span + 相关 span（multi-span/related）citeturn1view2turn4view0turn4view3  
  **Typed HIR 不负责**：运行时调度细节、执行器特定 SSA/内存布局、跨策略全局优化。  
- **Core IR（执行/统一后端层）负责**：可执行的统一计算图/算子序列、SSA 或等价数据流、运行时调度与资源约束、低层优化与代码生成/解释执行。Core IR **不负责**：从表层语法做复杂模式识别、做“用户可理解的语义级报错”。（Core IR 报错应当主要是 verifier/一致性错误，而不是用户逻辑错误。）这一边界与 MLIR 对“Ops/Values 图结构”及 verifier 的定位一致：低层 IR 重点在结构一致性与可变换性。citeturn10search16turn10search20  

## Typed HIR 形态与数据结构建议

### 设计目标与借鉴点

**事实（借鉴基础）**：  
- Rust 将 HIR 定义为解析 + 名称解析之后生成的“编译器友好 AST”，并去除部分语法糖。citeturn10search2turn10search10  
- MLIR 把 IR 看作 operations（节点）与 values（边）的图结构，每个 value 都有 type；dialect 可共存并逐步 lower。citeturn10search16turn1view1  
- MLIR 强调 verifiers 可强制 Ops 不变量，使 pass 更简单、错误更早暴露。citeturn10search20turn10search12  

**判断（面向量化 DSL 的简化）**：QuantScript 的 Typed HIR 不需要通用语言的复杂控制流/类型多态；其核心是**“指标/特征/信号/下单意图”的声明式数据流**，外加少量受控的条件逻辑。Typed HIR 的结构应更接近“带类型的数据流图 + 少量控制节点”，而不是完整的语句块语言。

### 推荐的 Typed HIR 核心数据结构

以下是“足够强但不过度工程化”的 Typed HIR 最小形态（以结构描述为主，非特定语言实现）：

- **稳定 ID 体系**  
  - `ExprId / StmtId / NodeId`：Typed HIR 内部节点 ID（便于缓存、去重、graph 映射）。  
  - `DefId`：名称解析产物，表示“某个模块/函数/字段/内置变量”的唯一定义点。Rust 的 HIR/DefId 分离经验可作为参照：把“引用点”与“定义点 ID”稳定绑定，诊断可指向两者。citeturn10search6turn10search2  
  - `TypeId`：类型驻留表（interning）或结构化类型对象。  

- **核心节点（建议限制集合，避免 ad hoc 增长）**  
  - `Const(value)`：常量（必须是 scalar）。  
  - `Ref(def: DefId)`：引用（变量/内置 series/参数）。  
  - `Call(callee: DefId, args: [ExprId])`：函数调用（所有可调用都必须先 resolve）。  
  - `Field(base: ExprId, field: DefId)`：字段访问（对 record/quote-bar 等）。  
  - `Let(name: DefId?, value: ExprId)`：绑定（本质是命名共享子表达式）。  
  - `If(cond: ExprId, then: ExprId, else: ExprId)`：受控条件（建议限制 cond 必须是 scalar bool 或 series bool，且明确其语义）。  
  - `Index(base: ExprId, offset: i32)`：历史引用（只允许 `offset >= 0`，严禁未来引用；lookahead 检测直接在此节点触发）。  
  - `EmitSignal(name, expr)` / `TargetPosition(expr)` / `OrderIntent(...)`：与 Strategy IR/Core IR 对接的“策略语义出口节点”。

- **每个表达式节点必须携带的语义字段**（这是 Typed HIR 要成为“诊断核心”的关键）  
  - `span: SpanId`：源位置（文本/图节点）。Rust 的实践强调 span 是诊断的基础数据结构并附着在 HIR/MIR 构造上，从而能做精确定位与 snippet 提示。citeturn1view2  
  - `ty: TypeId`：已推断/已检查的类型。  
  - `qual: Qualifier`：至少区分 `Scalar` 与 `Series`（必要时进一步细分为 `Const/Input/Simple/Series` 风格）。Pine Script 的 qualifier 体系及其层级可作为非常贴近量化脚本的参照。citeturn9view0turn9view1  
  - `time: TimeDependence`：用于 lookahead 检测与执行时点判断（例如 `max_lead`、`max_lag`、`availability` 等，后述）。  
  - `warmup: WarmupReq`：用于 warmup 不足诊断（后述）。  
  - `lowering: LoweringClass`：标记该节点是否可 lower、需要 runtime config、或被 beta 边界拒绝（后述 feature gating 设计可映射）。citeturn6search0turn6search17  

### 用 verifier 固化 Typed HIR 不变量

**事实（可借鉴）**：MLIR 的 verifier 用于保证 IR 结构正确与 Ops 不变量，使 passes 能在“已验证 IR”上工作；这类机制也能作为 debug 工具。citeturn10search20turn10search27  

**判断（落到 QuantScript）**：为 Typed HIR 设计一个轻量 verifier（不是全量优化器），每个 pass 后运行，至少覆盖：
- `Call` 的 callee 必须已 resolve，且参数个数/参数类型满足签名。  
- `Index(offset)` 必须满足 offset 约束（例如只允许历史引用）。  
- `OrderIntent/EmitSignal` 只能出现在策略出口结构中（防止不可 lower 的“在表达式里下单”）。  
- 所有 `ExprId` 的 `ty/qual/span` 必须已填充（允许 `ErrorTy` 做恢复，但必须显式）。  

这与 MLIR “在 verifier 中表达约束与关系（例如 SameOperandsAndResultType）”的思想一致，只是你们需要的是 DSL 规模的版本。citeturn10search12  

## 类型系统建议与检查策略

### 类型与 qualifier 的最小集合

本节把“事实/借鉴/判断”拆开，以确保你们能明确哪些是必须落地、哪些是参考实现。

**事实（量化脚本语义非常接近的参照）**：Pine Script 明确区分 value type 与 qualifier，并将 `const/input/simple/series` 作为决定“何时可用、是否跨 bar 变化”的关键维度，并给出层级 `const < input < simple < series`。citeturn9view0  
这与 QuantScript 要解决的“scalar/series 混用、参数必须为 scalar/const、series 只能 runtime 变化”高度同构。

**判断（QuantScript 推荐类型系统）**：至少落地以下集合（满足你提出的覆盖要求）：

- **值类型（Value Types）**  
  - `Bool`  
  - `Number`（建议单一浮点为主，避免 `Int/Float` 早期引入大量转换规则；若必须，可用 `Int` 仅承载窗口长度等参数）  
- **构造类型（Type Constructors）**  
  - `Scalar<T>`：跨 bar 不变或在策略启动时确定的值（可进一步细分 `Const/Input/Simple` 作为 qualifier，而不是类型）。citeturn9view0  
  - `Series<T>`：随 bar 变化的序列值（核心）。citeturn9view2  
  - `Signal`：**建议作为独立名义类型（nominal type）**，底层可等价于 `Series<Bool>`，但在类型上区分用于：  
    - 限制只能由特定构造函数产生（如 `cross_over`、`turns_true`）。  
    - 限制只能出现在策略出口或特定 combinator 中（减少把任意 `Series<Bool>` 当“交易信号”造成的语义歧义）。  
- **可空/缺失（Nullability）**  
  - `Maybe<T>` 或 `Option<T>`（建议只对 `Series` 启用，即 `Series<Maybe<Number>>`），用于表达缺失数据（停牌、上市前、指标 warmup 前等）。Pine 明确存在 `na`（not available）并在 series 中传播；这是量化脚本最常见的真实世界问题。citeturn9view2  
  - 可借鉴 TypeScript 的 `strictNullChecks` 思路：把 `null/undefined` 变成独立类型，强迫用户（或编译器自动插入）显式处理，从而避免运行时崩溃。citeturn6search2turn6search26  

### 类型检查与推断策略：为“报错可理解”优化

**借鉴（理论支持）**：双向 typing 将类型推断拆为“合成(synthesis)”与“检查(checking)”两种模式，常用于提升错误局部性并减少全局推断导致的连锁误差。citeturn2search4  

**判断（QuantScript 具体落地）**：推荐实现“约束求解 + 局部双向”的折中版本：
- 对大部分表达式做 **synthesize**（推得类型）。  
- 对关键位置做 **check**（传入期望类型）：  
  - 函数参数位置（比如 `sma(series, length)` 的 `length` 必须是 `Scalar<Int>`）。  
  - `if` 条件位置（必须是 `Bool` 的 scalar 或 series，具体取决于你们策略执行模型）。  
  - 策略出口节点（比如 `target_position` 必须是 `Series<Number>` 或 `Series<Maybe<Number>>`）。  
这样做的直接收益是：`scalar/series 混用`、`类型不匹配` 的报错会更贴近用户写错的那个点，而不是在很远的地方爆炸。citeturn2search4  

### 用“时间可得性”支撑 look-ahead 检测

**事实（领域语义基础）**：look-ahead bias 的核心是“在模拟中使用了在当时不可能获得的信息”。学术语境下对此有直接定义：标准 look-ahead bias 指在模拟中使用了在被模拟时间段内不可用的信息，通常会把回测结果向上偏移。citeturn1view4  

**事实（工程实现层面的关键点）**：bar 数据有 start/end time；例如 entity["company","QuantConnect","algorithmic trading platform"] 的文档强调：引擎在 bar 的 end time 才把 bar 交给算法，以避免在 bar 结束前“知道收盘价”；而许多免费数据源把 bar timestamp 标为 start time 并包含 close，容易导致研究出现 look-ahead。citeturn9view4  

**判断（QuantScript 的静态分析设计）**：在 Typed HIR 上做一个轻量“时间依赖抽象解释（abstract interpretation）”，为每个 `ExprId` 计算：
- `max_lead`：表达式依赖未来多少 bars（理想情况下必须为 0）。  
- `max_lag`：表达式需要多少历史 bars（用于 warmup）。  

构造规则示例（用于实现，而非用户可见）：
- `Ref(series)`：`max_lead=0, max_lag=0`  
- `Index(base, offset)`：若 offset 表示“回看 offset”，则 `max_lag = base.max_lag + offset`；并强制 offset 不能为负（否则 `max_lead > 0` 直接报错）。  
- `rolling_sum(base, window)`：`max_lag = base.max_lag + window-1`（window 必须 scalar int）。  
- `centered_window(...)`：直接标记 `max_lead > 0`（不可用于可交易信号）。  

这使 look-ahead 检测成为**可解释的、可定位的**：错误能指向某个节点（比如 `Index(-1)` 或 `center=true`），并给出替代写法（例如显式 shift 到过去）。  

### warmup 需求推导与不足诊断

**事实（行业框架用法）**：entity["company","QuantConnect","algorithmic trading platform"] 的 “Warm Up” 机制本质是把算法 start date 回拨或回放历史数据，以便在开始交易前把指标状态“预热”；且 warmup 阶段不能交易。citeturn9view3turn5search3  

**判断（QuantScript 的静态 warmup 合约）**：在 Typed HIR 中为每个表达式推导 `warmup_bars_required`，并与策略配置（runtime config / graph settings / compile API 参数）对比：
- 若 `required > configured`：  
  - MVP 阶段可以先报 **warning + 强提示**（因为你们是 beta、能力边界受限，可能希望先尽量让策略能跑，同时把风险标红）。  
  - 中期阶段建议升级为 **error**（因为 warmup 不足会系统性扭曲指标/信号，属于“结果不可用”）。  
- 同时输出 `suggested_warmup = required`，让图编辑器可以一键修复（把 warmup 参数改到建议值）。  

## 诊断体系设计

### 诊断必须同时服务三类入口的统一协议

**事实（可借鉴）**：  
- Rust 编译器开发指南强调：错误要尽量标注最小 span、必要时附带其他相关 span，并避免对同一根因重复报错。citeturn1view2  
- LSP 将诊断抽象为 `Diagnostic{range,severity,code,message,relatedInformation,data}`，并明确 `relatedInformation` 用于“同一错误关联的其他位置（例如符号冲突）”。citeturn4view0turn4view1turn4view3turn4view2  
- MLIR 的诊断基础设施把诊断拆为 location、severity、arguments、metadata，并支持 attachNote（notes 必须附着在非 note 诊断上）。citeturn8view0turn8view1  

**判断（QuantPilot 的统一诊断协议）**：你们需要一个“内部诊断模型”（IR/编译层统一），再分别投影到：
- QuantScript 文本 UI（类似编译器输出 + underline）  
- 图编辑器 UI（节点/边/端口高亮 + 属性面板错误列表）  
- 后端 compile API（结构化 JSON，便于前端/CI/日志聚合）

建议内部模型字段如下（命名可调整，但语义建议保持）：

- `code: string`（稳定错误码）  
- `severity: Error | Warning | Info | Hint`（与 LSP 严格对齐）citeturn4view3  
- `stage: Parse | Resolve | Type | Analyze | Lower | Verify`（便于定位责任域）  
- `message: { summary: string, body?: string }`（一行摘要 + 可选详细解释）  
- `primary: SpanRef`（主定位）  
- `labels: [{ span: SpanRef, label: string }]`（多点标注：类似 rustc/clang 的 secondary spans）citeturn1view2  
- `notes: [{ span?: SpanRef, text: string }]`（解释/背景）——可借鉴 MLIR attachNote 模型，notes 可以继承 location 或显式给出。citeturn8view1  
- `hints: [{ text: string, fix?: FixIt }]`（可选 quick fix）  
- `related: [{ span: SpanRef, message: string }]`（与 LSP DiagnosticRelatedInformation 对齐）citeturn4view3  
- `data?: any`（给图编辑器或 code action 的结构化 payload；LSP 也保留 `data` 用于 publishDiagnostics ↔ codeAction 之间传递）。citeturn4view1turn4view2  

### source span 设计：同时覆盖文本、图、API

**事实（可借鉴）**：Rust 明确 Span 是定位基础，并能通过 SourceMap 提取 snippet 来显示错误上下文。citeturn1view2  
**事实（可借鉴）**：MLIR 强调 source location 对可调试性与错误报告非常重要，并提供多种 location attribute 类型以适配不同需求。citeturn8view0  

**判断（QuantPilot span 统一抽象）**：定义 `SpanRef` 为“多态位置”：
- `TextSpan { uri, start(line,col,encoding), end(line,col), byte_range? }`（用于 QuantScript 文本与 LSP range）citeturn4view0turn2search22  
- `GraphSpan { graph_id, node_id, port_id?, property_path? }`（用于图编辑器精准高亮）  
- `SyntheticSpan { reason }`（用于编译器合成节点；必须尽量少用，且附 note 解释来源）

关键是：**所有 Typed HIR 节点都要保留“来源 span”**，图节点也要能映射到同一 `SpanRef`。这样，diagnostics 能真正“同时服务文本/图/API”，而不是三套逻辑各自做 matcher。

### 错误类别与错误码命名方案

**事实（可借鉴）**：Rust 倾向给每个错误分配唯一错误码，并要求错误码带解释文档；错误码本身会出现在 UI。citeturn1view3  

**判断（QuantScript 推荐错误码）**：建议采用“产品前缀 + 分层域 + 序号”的稳定 scheme，便于大规模治理与支持（supportability）：

- 前缀：`QS`（QuantScript）  
- 域（3~5 字母）：  
  - `SYN`（语法）  
  - `RES`（名称解析）  
  - `TYP`（类型）  
  - `SEM`（语义规则：scalar/series、可交易约束等）  
  - `CAUS`（因果/时间可得性：lookahead bias）  
  - `WARM`（warmup）  
  - `LOW`（lowering）  
  - `BETA`（beta 边界/feature gating）  
- 序号：四位或五位（建议四位起步）

示例：`QS-RES-0102`，`QS-CAUS-0401`，`QS-BETA-0701`。

并建议为每个错误码维护“解释页”（可本地或在线）；LSP 支持 `CodeDescription.href` 指向错误码说明页。citeturn4view3turn4view1  

### beta 边界之外能力的显式拒绝：feature gating 的 DSL 版本

**事实（可借鉴）**：Rust 的 feature gate 机制会记录 gated spans，并在 feature 未启用时对每个 gated span 发出诊断。citeturn6search0turn6search16  
**事实（可借鉴）**：Rust 错误码 `E0658` 明确用于“不稳定特性被使用”。citeturn6search17  

**判断（QuantScript 的 beta gating 设计）**：  
- 为每个“未承诺稳定 lower 到 Core IR 的能力”打上 `FeatureTag`（例如 `beta.ml`, `beta.dynamic_universe`, `beta.intrabar`）。  
- 在 Typed HIR 构建时收集 `GatedSpan{feature, span, reason}`，最后统一发出 `QS-BETA-xxxx` 诊断：  
  - 主消息：明确“当前 beta 边界不支持”  
  - note：解释为什么（无法保证可复现实验/不可稳定 lower/风险太大）  
  - hint：替代路径（推荐用图节点/已有算子组合）  
  - 若确有内部开关：提供 compile option（但必须显式，不可默默成功）

### diagnostics 测试与回归机制

**事实（可借鉴）**：MLIR 支持 `verify-diagnostics` 风格的诊断验证测试，确保特定输入产生期望诊断；适用于“诊断协议作为产品契约”。citeturn10search8  
**判断（QuantPilot 落地方式）**：为 QuantScript 建立“诊断金样测试（golden）”：
- 每个错误码至少 1 个最小样例。  
- 覆盖：主 span、相关 span、hint、note 的稳定性。  
- 对图编辑器：用 `GraphSpan` 的 JSON fixture 做快照测试（避免 UI 与编译器协议漂移）。  

## 典型错误案例与理想报错

以下示例为**理想化输出格式设计**（非现有实现），目的是把“错误码 + span + hint + related location + 可理解解释”具体化。示例语法假设 QuantScript 类似表达式 DSL：`import`、`let`、内置数据 `close`、指标库 `ta.*`、策略出口 `target_position(...)` 等。

### 语法错误

**案例一：缺失右括号**

```qs
let x = ta.sma(close, 20
```

```text
error[QS-SYN-0001]: 语法错误：缺少 ')'
  --> main.qs:1:23-1:23
   |
 1 | let x = ta.sma(close, 20
   |                       ^
help: 在参数列表末尾补上 ')'
note: 解析在此处遇到文件结尾，无法完成函数调用表达式
```

**案例二：意外的 token**

```qs
let x = ta.sma(close,, 20)
```

```text
error[QS-SYN-0002]: 语法错误：在参数列表中遇到意外的 ','
  --> main.qs:1:20-1:21
   |
 1 | let x = ta.sma(close,, 20)
   |                    ^^
help: 删除多余的 ','，或在 ',' 前补上缺失的参数
```

### 未知模块 / 未知函数 / 未知字段

**案例三：未知模块**

```qs
import alpha.ta
let x = alpha.ta.sma(close, 20)
```

```text
error[QS-RES-0101]: 未知模块 `alpha`
  --> main.qs:1:8-1:13
   |
 1 | import alpha.ta
   |        ^^^^^
help: 可用模块: ta, math, risk, portfolio, data
note: 模块名区分大小写
```

**案例四：未知函数 + 候选建议**

```qs
let x = ta.smm(close, 20)
```

```text
error[QS-RES-0102]: 未知函数 `ta.smm`
  --> main.qs:1:9-1:15
   |
 1 | let x = ta.smm(close, 20)
   |         ^^^^^^
help: 你是不是要调用 `ta.sma`？
note: 同名空间可用函数: ta.sma, ta.ema, ta.rsi
```

**案例五：未知字段**

```qs
let v = bar.cloze
```

```text
error[QS-RES-0103]: 未知字段 `cloze`
  --> main.qs:1:13-1:18
   |
 1 | let v = bar.cloze
   |             ^^^^^
help: 可用字段: open, high, low, close, volume, end_time
note: 字段解析基于 bar 的类型（QuoteBar/TradeBar 不同字段集）
```

### 类型不匹配与 scalar/series 混用

**案例六：类型不匹配（期望 number，实际 bool）**

```qs
let x = ta.sma(true, 20)
```

```text
error[QS-TYP-0201]: 类型不匹配：`ta.sma` 的第 1 个参数期望 `Series<Number>`，实际为 `Scalar<Bool>`
  --> main.qs:1:15-1:19
   |
 1 | let x = ta.sma(true, 20)
   |               ^^^^
help: 把 `true` 替换为价格序列（例如 `close`），或使用返回布尔的函数
```

**案例七：scalar/series 混用：窗口长度必须是 scalar**

```qs
let len = ta.sma(close, 10)  // len 是 Series<Number>
let x = ta.sma(close, len)   // 把 series 当 length
```

```text
error[QS-SEM-0301]: scalar/series 混用：窗口长度必须是 `Scalar<Int>`，但得到 `Series<Number>`
  --> main.qs:2:20-2:23
   |
 2 | let x = ta.sma(close, len)
   |                    ^^^
help: 使用常量长度（例如 20），或先把 length 变为 scalar 参数（input/config）
note: 动态窗口会导致每个 bar 的窗口大小变化，当前 beta 版本不保证可稳定 lower
```

**案例八：Series 与 Scalar 的算术混用（需要显式广播规则）**

```qs
let x = close + 1
```

```text
warning[QS-SEM-0302]: `Series<Number>` 与 `Scalar<Number>` 混用：将把 scalar 自动提升为 series
  --> main.qs:1:9-1:18
   |
 1 | let x = close + 1
   |         ^^^^^^^^^
note: 自动提升规则：Scalar 会在每个 bar 上按常量使用
hint: 若你希望的是“只在首 bar 加 1”，请改用 simple/初始化语义（见文档）
```

### look-ahead bias 检测

**案例九：显式未来引用（负 offset）**

```qs
let tomorrow = close[-1]
```

```text
error[QS-CAUS-0401]: look-ahead 风险：不允许引用未来数据 `close[-1]`
  --> main.qs:1:16-1:24
   |
 1 | let tomorrow = close[-1]
   |                ^^^^^^^^^
help: 若你需要历史引用，请使用 `close[1]`（上一根 bar）或更大的正整数
note: 未来引用会让回测使用“当时不可得信息”，导致结果失真
```

**案例十：使用“居中窗口”导致未来依赖**

```qs
let x = ta.rolling_mean(close, window=20, center=true)
```

```text
error[QS-CAUS-0402]: look-ahead 风险：`center=true` 的窗口会使用未来 bar
  --> main.qs:1:9-1:55
   |
 1 | let x = ta.rolling_mean(close, window=20, center=true)
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: 删除 `center=true`（使用纯历史窗口），或改用显式 `shift` 后再计算
note: 可交易信号必须满足 `max_lead == 0`
```

**案例十一：下单时点使用“未确认 bar”数据（intrabar/repainting 风险）**

```qs
if cross_over(ta.rsi(close, 14), 70) {
  order.market("BUY", 1)
}
```

```text
warning[QS-CAUS-0403]: 信号可能依赖未确认 bar（repainting / 类 look-ahead 风险）
  --> main.qs:1:4-3:1
   |
 1 | if cross_over(ta.rsi(close, 14), 70) {
   |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
note: 若运行模型允许 intrabar 更新，条件可能在同一根 bar 内多次变化
hint: 使用 `on_bar_close(...)` / `confirmed(...)`（若提供）来要求 bar 收盘后再触发
```

### warmup 不足

**案例十二：warmup 配置不足（静态可推导）**

```qs
config { warmup_bars: 50 }

let x = ta.sma(close, 200)
target_position( x > close ? 1 : 0 )
```

```text
error[QS-WARM-0501]: warmup 不足：策略需要至少 200 bars 的历史数据，但配置为 50
  --> main.qs:3:9-3:26
   |
 3 | let x = ta.sma(close, 200)
   |         ^^^^^^^^^^^^^^^^^^
help: 把 `warmup_bars` 调整为 >= 200
note: 指标在 warmup 完成前通常输出 NA/不稳定值，可能导致信号失真
```

### 不可 lower 的表达式

**案例十三：使用 lambda/高阶函数（不在 DSL 边界）**

```qs
let f = (x) => x + 1
let y = map(close, f)
```

```text
error[QS-LOW-0601]: 不可 lower：当前版本不支持 lambda/高阶函数
  --> main.qs:1:9-2:19
   |
 1 | let f = (x) => x + 1
   |         ^^^^^^^^^^^^
 2 | let y = map(close, f)
   |         ^^^^^^^^^^^^^
help: 用内置算子组合表达（例如 `close + 1`）
note: beta 边界内仅支持可验证、可稳定 lower 到 Core IR 的表达式片段
```

### beta 边界之外能力的显式拒绝

**案例十四：调用实验性模块**

```qs
import ml
let model = ml.fit(features, labels)
```

```text
error[QS-BETA-0701]: 不支持：`ml.fit` 位于 beta 边界之外
  --> main.qs:2:13-2:19
   |
 2 | let model = ml.fit(features, labels)
   |             ^^^^^^
help: 当前版本仅支持规则/指标/信号类表达；暂不支持训练型工作流
note: 该能力尚不能稳定 lower 到 Core IR，且存在数据泄漏/时间对齐风险
```

### 图编辑器与 compile API 的定位示例

**案例十五：图编辑器节点端口类型不匹配（GraphSpan）**

假设图中 `SMA` 节点的 `length` 端口被连接了一个 `Series` 输出（动态变化），而 `length` 必须是 `Scalar<Int>`。

```text
error[QS-SEM-0301]: scalar/series 混用：SMA.length 端口期望 Scalar<Int>，但连接到了 Series<Number>
  at GraphSpan{graph_id="g-42", node_id="n-sma-7", port_id="length"}
help: 把 length 改为常量参数节点（Input/Const），或断开该连接
related:
  - GraphSpan{graph_id="g-42", node_id="n-ta-3", port_id="out"}: 该输出是 Series<Number>
```

同时，compile API 应返回结构化数据（示意）：

```json
{
  "code": "QS-SEM-0301",
  "severity": "Error",
  "stage": "Type",
  "message": {
    "summary": "scalar/series 混用：SMA.length 端口期望 Scalar<Int>，但连接到了 Series<Number>"
  },
  "primary": { "kind": "graph", "graph_id": "g-42", "node_id": "n-sma-7", "port_id": "length" },
  "related": [
    { "span": { "kind": "graph", "graph_id": "g-42", "node_id": "n-ta-3", "port_id": "out" },
      "message": "该输出是 Series<Number>" }
  ],
  "hints": [
    { "text": "把 length 改为常量参数节点（Input/Const），或断开该连接" }
  ]
}
```

## 分阶段路线图与避免过度工程化

### 最小可落地版本

目标：在不重写现有 Strategy IR/Core IR 的前提下，引入 Typed HIR 作为“语义与诊断主锚点”，立刻减少 matcher 扩张与报错不可理解的问题。

必须交付的最小集合（判断）：
- Parse AST：稳定 span，支持基本错误恢复（至少括号/逗号/关键字）。citeturn1view2  
- Name resolution：模块/函数/字段/内置数据源统一 resolve；未知符号给出候选与 related span。citeturn4view3  
- Type checking：实现 `Scalar/Series/Signal/Maybe` 的最小规则；实现 `Series` 自动提升（必要时）但对“窗口长度、symbol、timeframe”等参数强制 scalar。citeturn9view0  
- Typed HIR：节点携带 `span/ty/qual`；引入 `ErrorTy` 支持恢复（避免一次只报一个错）。  
- lookahead MVP：禁止显式未来引用（负 offset）与已知前瞻算子（center window）。citeturn1view4turn9view4  
- warmup MVP：对固定窗口指标推导 `warmup_bars_required`，与配置对比并给出建议值。citeturn9view3  
- Lowering gate：对不可 lower 的节点直接给 `QS-LOW-*`；对 beta 边界之外能力给 `QS-BETA-*`（显式拒绝）。citeturn6search0turn6search17  
- 统一诊断协议（JSON）：能同时被文本前端、图编辑器、compile API 消费；字段对齐 LSP 诊断模型（至少 range/severity/code/message/related/data）。citeturn4view0turn4view1  

与现有 graph/Strategy IR/Core IR 的衔接（判断）：
- MVP 阶段不强制 graph 直接产 Typed HIR：  
  - 方案 A：graph →（现有 Strategy IR）→ 转译成 Typed HIR 做检查/诊断 → 再走原 lowering；  
  - 方案 B：graph 直接生成 Typed HIR（更理想，但改动大）。  
- QuantScript：AST → resolve → type → Typed HIR → lower 到 Core IR（或先 lower 到 Strategy IR 再到 Core IR，视你们现状）。  

### 中期版本

目标：Typed HIR 成为唯一语义真相来源，graph 与 QuantScript 共享同一套静态分析与诊断；诊断体验达到“用户可理解 + 可修复建议 + 可定位”。

建议增强（判断 + 借鉴支撑）：
- 完整实现 multi-span diagnostics 与 dedup（避免 cascading error）。citeturn1view2turn4view3  
- 引入 verifier：每个 lowering pass 后验证 Typed HIR 与 Core IR 的关键不变量（参考 MLIR verifier 作为 pass 假设基础）。citeturn10search20turn10search27  
- lookahead 深化：把“数据可得性”纳入分析（例如 bar end_time、不同频率对齐），并对高风险模式从 warning 升级为 error。citeturn9view4  
- warmup 深化：对复合指标与嵌套窗口推导 warmup；输出“最小 warmup + 建议 warmup”。citeturn9view3  
- 诊断解释页与 codeDescription（LSP 支持链接到错误码说明）。citeturn4view3turn4view1  
- 建立诊断金样测试体系（借鉴 MLIR verify-diagnostics 思路）。citeturn10search8  

### 高阶版本

目标：在仍然“非通用编程语言”的前提下，把 QuantScript 变成“研究级量化 DSL”：可复现实验、可审计、可解释的静态语义约束。

高阶能力（判断，且需严格防止过度工程化）：
- 更完善的“点时（point-in-time）”数据语义：对财务数据/复权数据/公告延迟建模，避免隐含 lookahead（这属于量化研究真实痛点，但实现复杂度高，必须在 beta 边界外逐步引入并强 gating）。citeturn1view4turn9view4  
- 更强的工具链集成：若你们未来要做编辑器体验，可直接映射到 LSP 的 diagnostics 模型；LSP 设计就是为“错误检查/跳转/补全”等工具化而生。citeturn2search5turn4view0  
- 诊断的自动修复（FixIt/CodeAction）：依赖 `data` 字段把结构化修复传给前端（LSP 明确支持 `data` 在 publishDiagnostics 与 codeAction 之间保留）。citeturn4view1turn4view2  

### 为研究级量化 DSL，什么必须做，什么暂时不该做

这里给出明确的取舍清单（判断），并用引用说明其背后的“可借鉴事实”。

必须做（高价值、直接解决你列出的缺口）：
- **Typed HIR + 统一诊断协议**：没有它就会继续 ad hoc matcher 横向扩张，且错误定位无法系统化（Rust/MLIR 都把 source location + IR 作为诊断核心）。citeturn1view2turn8view0  
- **名称解析与类型检查前置**：未知模块/函数/字段、类型不匹配、scalar/series 混用都要在 Typed HIR 前解决或在其构建时解决（Rust HIR 的“在 name resolution 后生成”是直接参照）。citeturn10search2  
- **lookahead 与 warmup 作为静态合约**：  
  - lookahead 是回测最根本错误之一，涉及“时间因果结构”。citeturn1view4turn9view4  
  - warmup 是指标/状态式策略必备工程实践，且行业框架已用“回拨/回放”方式实现。citeturn9view3  
- **beta 边界显式拒绝（feature gating）**：避免“看起来能写、实际不能稳定 lower”的灰区（Rust 的 feature gates 是成熟工程对照）。citeturn6search0turn6search17  
- **verifier 驱动的早失败**：让非法构造在阶段内就报错，而不是在后端 matcher/运行时爆炸（MLIR 明确把 verifier 作为 pass 假设基础）。citeturn10search20  

暂时不该做（高风险过度工程化，且偏离“单机量化 beta、能力边界严格受限”）：
- 完整通用语言特性：用户自定义类型系统、泛型/多态、宏系统、可变作用域捕获的闭包、高阶函数、异步/并发、异常系统。它们会大幅扩大 type inference、lowering、诊断矩阵，而且与你们“量化策略表达层”的目标不匹配。  
- 复杂的全自动全局类型推断：会让错误定位变差，需要大量启发式；推荐先用双向 typing 的局部推断来保证错误局部性。citeturn2search4  
- 过早实现完整 LSP 服务器：如果你们短期主要入口是图编辑器与 compile API，那么先把诊断协议对齐 LSP 模型字段即可；真正的 LSP 服务器可以在“文本编辑器成为核心入口”后再做。citeturn4view0turn2search5  
- 过度追求“全静态消除所有回测偏差”：例如对所有数据源的发布延迟、复权方式、企业行为等做精确点时建模，属于高阶版本且必须放在 beta gating 后逐步开放，否则会拖垮主线交付。citeturn9view4turn1view4  

### 哪些设计最有价值，哪些可能过度工程化

- **最高价值（应优先投入）**：  
  - Typed HIR 节点携带 `span/ty/qual/time/warmup/lowering` 六元信息（直接支撑你列出的所有诊断项）。citeturn1view2turn9view0turn9view3  
  - 统一诊断协议，字段对齐 LSP Diagnostic（range/severity/code/related/data），从一开始就让前端图编辑器与 API 成为“一等公民”。citeturn4view0turn4view1  
  - verifiers + pass 后验证：把“非法构造早失败”工程化，而不是靠 matcher 兜底。citeturn10search20  

- **高概率过度工程化（在 beta 阶段应避免）**：  
  - 复杂类型层级（过多数字类型、单位类型、维度分析）——除非你们 Core IR 已经需要，否则诊断收益不成比例。  
  - 全量跨文件模块系统、包管理与依赖解析 —— 对“单机量化 beta”不是关键路径。  
  - 为所有错误实现自动修复（quick fix）——应先覆盖“最常见 20% 错误码”的 fixit，确保协议稳定后再扩展。citeturn4view1turn4view2