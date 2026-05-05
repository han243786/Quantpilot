# QuantPilot Spread / Custom / 插件化推进顺序深度研究

## 最终推荐路径

**主推荐方案：Path C（Spread 内建；Custom 受限表达式；插件化最后进入）**。

**我的判断（结合你给出的现实约束）**：  
在已有 beta、且必须保护主链路 **Data → Intent → Agent → Risk → Execution → Fill** 的前提下，最稳的推进顺序是先把“可验证、可回放、可解释”的**语义核心**做实，再把变化与扩展压在“受限表达式层”和“插件扩展点”里。原因是：Spread 的核心难点并不在“能不能扩”，而在“跨源时间语义 + 对齐规则 + 缺失值规则 + 回放一致性”——这些一旦漂移，会立刻击穿验证、回放与风险控制；而 Custom 的最大风险是“变成另一个编程语言/运行时”，导致 capability 漂移与测试失控，因此更适合先做成可验证、可拒绝、可降级（lowerable）的受限表达式层（类 CEL / Starlark 的思路），最后再引入插件系统承接非核心扩展。citeturn7search0turn7search1turn0search1turn0search0turn4search1

**推荐的推进顺序（严格按“真实边界”驱动）**：  
1) **Spread 先进入 Core IR（受控内建语义）**：把跨源对齐、时间一致性、缺失值与采样/重采样等语义固定为可验证规则，并让运行时与回放完全同语义。对齐语义可借鉴 kdb+/pandas 的 as-of join / window join 设计，但要在 IR 层显式化（而非隐式由实现决定）。citeturn1search1turn1search0turn6search2turn7search0  
2) **Custom 做成受限表达式层（Restricted Expression IR / DSL），并强制 lowering 到 Core IR**：Custom 不直接触碰 Execution/Risk 的关键动作，仅能声明式地产生“可解释的意图/信号/参数”，并通过静态检查与资源上界保证可终止、可复现。可借鉴 CEL 的“非图灵完备、确定性求值”与在 Kubernetes 中用于校验/约束的使用方式；或借鉴 Starlark 的“无外部副作用、确定性”。citeturn0search1turn7search3turn7search7turn0search0turn0search12turn0search21  
3) **插件化最后进入（先最小边界、再治理），且只能挂在主链路允许的 extension points**：插件系统一上来就做“可安装第三方插件 + 依赖解析 + 版本矩阵 + registry + 签名”会显著扩大稳定性与安全的风险面；更合理的是先在内部引入“manifest + capability 声明 + 校验 + 受控生命周期”的最小形态，等 Spread/Custom 语义、编译链路、回放语义稳定后再开放更强治理。citeturn2search4turn3search1turn2search5turn3search0turn1search3turn5search0

---

## 路径对比与决策标准

### A/B/C 对比表

| 路径 | 收益（短期） | 风险（beta 稳定性） | 复杂度（工程/治理） | 与“能力真实边界驱动 UI/模块”的适配度 |
|---|---|---|---|---|
| Path A：先实现 Spread/受控 Custom，再做插件化 | 快速把核心能力跑通；语义更可控 | 中（扩展压力后置，但仍可能在 Custom 上失控） | 中 | 高 |
| Path B：先做轻量插件边界，再让 Spread/Custom 走插件 | 早期“看起来很扩展”；团队可并行 | 高（语义漂移、版本/依赖/沙箱问题提前爆炸） | 高 | 中（capability 发现会更复杂） |
| Path C：Spread 内建，Custom 受限表达式，插件化最后进入 | 语义最稳定；回放/解释最容易做实 | 低（最少破坏主链路） | 中（但复杂度可分阶段引入） | 最高 |

**事实（可验证的外部经验）**：  
- 可扩展 IR 的主流做法通常不是“一开始就插件化所有语义”，而是先建立可验证的核心（verification/invariants），再通过“方言/扩展层”做渐进 lowering 和可组合转换；MLIR 明确以“dialects + lowering/conversion + verification”来应对扩展与碎片化。citeturn7search0turn7search1turn7search13turn0search14  
- 受限表达式语言在工程上常用于“把可配置的逻辑限制在可终止、可安全、可审计的范围”，例如 CEL 的确定性求值、非图灵完备特性，以及在 Kubernetes API 中用于校验/约束。citeturn0search1turn7search3turn0search21  
- “加载用户代码但必须先过验证器”的典型例子是 eBPF：内核通过 verifier 做 CFG/终止性等检查（至少历史上通过禁环等手段防止不可终止路径），以保护系统稳定性。citeturn2search3turn2search0turn2search6turn2search18  

### 针对你提出的关键问题的结论

**Spread 应该先作为受控内建语义进入 Core IR，还是直接插件化？**  
我的判断：**先内建进 Core IR（受控语义）**。Spread 的本质是“跨源时间语义与对齐规则”，它直接影响信号、风控与执行一致性；如果把它放进插件边界，等于允许多个实现各自定义对齐/缺失值/采样语义，必然导致 capability 漂移与回放不可比。能借鉴 as-of join / window join 的成熟语义，但必须在 Core IR 层显式化并强制一致。citeturn1search1turn6search2turn7search0  

**Custom 应该是 restricted expression IR、受限 DSL、还是插件边界？**  
我的判断：**优先做成“受限表达式层（restricted expression IR/DSL）”，并强制 lowering 到 Core IR**。理由是：  
- 受限表达式可以像 CEL 一样做到确定性、可静态检查、可拒绝、可资源上界控制；同时表达“自定义逻辑”需求，而不引入完整插件运行时的供应链/隔离/依赖治理问题。citeturn0search1turn7search7turn0search21  
- 或像 Starlark 一样强调“无外部副作用、确定性”，更符合“可回放、可解释”的目标。citeturn0search0turn0search12  

**插件系统应该在什么阶段进入，才不会破坏当前 beta 的稳定性？**  
我的判断：**在 Spread/Custom 的语义、验证与编译链路稳定后**，以“最小 manifest + capability 声明 + 最小验证机制”的内部插件形态进入（先不开放第三方安装、先不做复杂依赖解析/registry）。类似 VS Code 这类系统也是通过 manifest 声明与激活机制来约束扩展加载与生命周期，但它们同时也体现了一个事实：manifest/activation 只是起点，真正的稳定性来自清晰的扩展点设计与严格 API 合约。citeturn2search4turn2search1turn2search8  

**capability discovery、validation、compile path、runtime semantics 如何与扩展能力保持一致？**  
我的判断：用“**声明式 capability → 编译期校验/选择 → 运行时强制边界 → 回放同语义**”的闭环。可借鉴：  
- Kubernetes Discovery API：服务端发布可用的 group/version/resource/verbs 供客户端发现，形成“能力真实边界”的机器可读来源。citeturn3search1turn3search13  
- OpenAPI 的“机器可发现能力描述”理念：消费者无需读源码即可理解服务能力。citeturn3search6turn3search2  
- OSGi 的 requirement/capability 与 wiring introspection：把“依赖/能力”变成可解析对象，便于诊断与治理（但你们短期不必上完整 OSGi 复杂度）。citeturn3search7turn3search3  

### 判断标准：何时进 Core IR / 受限表达式层 / 插件扩展点

**我的判断（结合外部借鉴）**：可以用下列“硬门槛”决定归属，减少争论与漂移。

进入 **Core IR** 的门槛（满足其一就倾向 Core IR）：  
- 该能力**会改变时间/数据语义**（对齐、采样、缺失值、时区、因果约束），影响回放一致性与解释口径；这类语义一旦分叉，会导致同一策略在不同实现下不可比。借鉴：IR 设计强调 invariants/verification，避免语义未定义带来的不可控优化/不一致。citeturn7search0turn7search1turn7search2  
- 该能力是主链路关键阶段（Risk/Execution）必需的“统一语义”，必须可审计、可复现。借鉴：事件溯源（Event Sourcing）强调用事件序列重建状态，用于审计与回放；核心链路越关键，越应在统一语义下可回放。citeturn4search1turn4search13  

进入 **受限表达式层（Restricted Expression IR/DSL）** 的门槛：  
- 需求是“允许用户/策略写一点逻辑”，但必须 **确定性、可终止、可资源上界**，且不允许 I/O/副作用。CEL 明确强调确定性求值与非图灵完备；Starlark 强调无外部副作用与确定性。citeturn0search1turn7search3turn0search0turn0search12  
- 必须能被**静态拒绝**（diagnostics 明确指出不允许的构造/函数/资源超限），并且能**lower 到 Core IR**，保证最终运行语义统一（类 MLIR dialect lowering）。citeturn7search1turn7search17turn7search0  

进入 **插件扩展点** 的门槛：  
- 能力属于“非核心语义”，如新增数据源连接器、策略模板库、可视化/解释器适配、非关键路径的计算算子包；其语义不应改变 Core IR 的时间/对齐规则。借鉴：Kubernetes 把 API 能力通过 discovery 公开，但扩展资源/控制器依然要遵循 API 服务器的统一契约与 admission/validation 链条。citeturn3search1turn0search21  
- 能力可以被**能力声明（capability declaration）**完整描述，并可通过契约测试/验证器判定是否兼容当前版本。借鉴：OpenAPI 的“可发现能力描述”、以及 eBPF “先过 verifier 才能进入系统”的门禁思想。citeturn3search6turn2search3turn2search0  

---

## Spread 语义建议

这一节按你的硬要求覆盖：**数据对齐、时序一致性、跨源引用、缺失值、采样周期**，并把建议落实为“可验证语义”，避免实现漂移。

### 事实：行业里已有的可借鉴语义基元

- **As-of join（时间最近匹配）**是金融时序数据对齐的常见基元：  
  - kdb+ 的 as-of join（aj / asof）语义是“为左表每条记录取右表中**不晚于该时刻的最近一条**匹配记录，否则为空”。citeturn1search1turn1search5  
  - pandas 的 `merge_asof` 提供 backward/forward/nearest 三种方向选择，并要求按 key 排序，同时支持按 `by` 键分组匹配。citeturn1search0turn1search12  
- **Window join（时间窗口聚合）**用于“在一个时间区间内聚合另一条流/表”的值；kdb+ 的 wj/wj1 区分了“区间进入时的 prevailing（阶跃函数）是否算有效值”。citeturn6search2turn6search10  
- **重采样（resample）**是把不规则/高频数据转换为规则频率（如 bar）的常见操作，且需要显式指定 closed/label/origin 等细节才能避免歧义；pandas 在 API 层面把这些参数显式化。citeturn6search0turn6search4turn6search17  

### 我的判断：QuantPilot 的 Spread 需要在 Core IR 中“显式化”的最小语义面

将 Spread 的语义拆成四层，每层都要进入 IR 的可验证字段（否则就会“实现决定语义”）：

**时间轴与因果约束（Clock & Causality）**  
- 每个输入流/数据源必须携带两类时间：  
  1) **event_time（事件发生时间）**：用于策略逻辑、回放与对齐；  
  2) **ingest_time / observed_time（观测/入库时间）**：用于诊断延迟、构建“当时是否可见”的证明。  
  这类双时间思想在流处理领域很普遍（区分事件时间与处理/观测时间，并用 watermark 处理迟到数据），尽管你们不一定需要完整 watermark 机制，但至少需要在语义上区分“发生”与“可见”。citeturn1search10turn1search6  
- 对任何会驱动交易决策的对齐，默认使用 **as-of（backward）**，以降低引入未来信息的风险（look-ahead bias）。look-ahead bias 的定义是“回测使用了当时不可得的信息”，会导致不现实的结果。citeturn4search0turn4search12turn4search20  

**对齐策略（Alignment Policy）**  
在 Core IR 为 Spread 提供一个统一的 `align(policy)`，至少支持：  
- `asof_backward(tolerance)`：取不晚于 t 的最近值（默认）；  
- `asof_forward(tolerance)`：取不早于 t 的最近值（只允许在**解释型**或**非交易决策**路径使用，或需显式标注“允许未来观测”）；  
- `nearest(tolerance)`：通常不适合交易决策，因为可能跨越 t 两侧隐含未来信息；若提供，必须强制声明用途与审计标签。  
这些方向与容忍度设计可直接借鉴 pandas 的 `merge_asof`（backward/forward/nearest + tolerance）。citeturn1search0turn1search12  

**跨源引用（Cross-source Reference）**  
- Spread 必须强制所有跨源引用在 IR 里显式指定：`(source_id, instrument_id, field, event_time_basis)`，禁止“只写 symbol 自动猜源”。  
- 对同名 symbol 跨源的情况，必须要求显式 `source`，否则编译期拒绝（这是一类高频造成语义漂移的隐患）。  
- 对跨源字段的单位/币种/合约乘数差异，要求在 IR 的 type/metadata 层携带“度量信息”，并在编译期做一致性检查（不一致要拒绝或要求显式转换）。  
这里的核心不是“类型系统做多强”，而是把“解释口径”固定下来，避免 runtime 隐式转换导致回放不一致。借鉴：IR 领域强调通过 invariants/verification 固化约束，避免未定义语义。citeturn7search0turn7search4turn7search2  

**缺失值与阶跃函数（Missingness & Step Function）**  
- 缺失值必须成为一等语义：`missing_reason ∈ {no_data_before_t, out_of_tolerance, market_closed, feed_gap, filtered}`，而不是简单的 null。  
- 对“报价类/状态类”数据（quote, best bid/ask, trading status），默认视为**阶跃函数（prevailing value）**：即在下一次更新前保持有效；这与 kdb+ wj 中“prevailing quote on entry”视为有效的语义相符。citeturn6search2turn6search10  
- 对“成交类/事件类”数据（trade prints, fills），默认不做阶跃延续，缺失就是缺失。  
- 对 forward-fill（ffill）的使用必须受控：只能用于被标注为“可持有状态”的字段，并且必须有 `max_staleness` 上限，防止长时间停更导致“陈旧值泄漏”。pandas 的 resample/ffill 语义说明了 forward fill 的基本行为与 limit 参数。citeturn6search1turn6search17  

**采样周期与重采样（Sampling / Resampling）**  
- 所有 bar/周期数据必须在 IR 中携带：  
  - `period`（例如 1s/1m/5m/1D），  
  - `closed` 与 `label`（区间闭合方向与标记位置），  
  - `origin/offset`（桶对齐基准），  
  - `calendar/timezone`（交易日历与时区）。  
pandas 的 resample API 把这些关键歧义点变成显式参数，正是你们需要借鉴的“避免隐式语义”方向。citeturn6search0turn6search4  

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["as-of join time series diagram","time series resampling alignment diagram","window join time series illustration"],"num_per_query":1}

### Spread 在 Core IR 的最小落地形态（建议）

**我的判断**：为了避免“为了扩展而扩展”的漂移，Spread 的第一版 Core IR 只需要覆盖最常见、最可验证的 3 个算子族：  
1) `AlignAsof(left, right, keys, direction=backward, tolerance, by)`（对齐）——对齐语义固定；  
2) `WindowAgg(left, right, window=[start,end], agg, prevailing=true/false)`（窗口聚合）——把 prevailing 与否显式化；  
3) `Resample(series, period, ohlc/last/mean..., closed,label,origin,offset, fill_policy)`（重采样）——把歧义点显式化。  
其余高级形态（多级窗口、复杂触发、动态 watermark）先不做，避免把流处理系统的复杂度提前引入。citeturn1search1turn6search2turn6search0turn7search0  

---

## Custom 的边界建议

你提出的“最怕：扩展导致 capability 漂移、语义不一致、测试失控”，本质上是：Custom 如果变成“任意代码”，就会绕开主链路语义与验证体系。这里建议把 Custom 收敛为“可验证、可拒绝、可 lowering”的受限表达式层。

### 事实：受限表达式的可行形态

- CEL 的规范明确：给定环境下表达式**确定性求值**到值或错误；并且 CEL 被定位为可嵌入的、安全的表达式语言。citeturn0search1turn7search19turn7search3  
- Kubernetes 明确在 API 中使用 CEL 来声明校验规则/约束条件，这与“把自定义限制在校验层、并保持主系统稳定”高度同构。citeturn0search21turn0search1  
- Starlark 被设计为配置/嵌入式语言，强调无外部副作用与确定性，这也是“可回放、可解释”需要的性质。citeturn0search0turn0search12turn0search16  
- eBPF 的模式表明：允许加载用户程序但必须先过 verifier（CFG/终止性/资源限制）才能保护系统稳定。citeturn2search3turn2search0turn2search17  

### 我的判断：QuantPilot Custom 应该“允许什么 / 不允许什么”

**允许（第一阶段必须够用，但不扩张成语言）**  
- 纯表达式：字面量、算术/逻辑运算、条件表达式（if/ternary）、结构体/record 字段访问。citeturn7search19turn0search1  
- 有界集合操作：对小集合/窗口的 `map/filter/reduce`（如果提供），必须有静态上界或由引擎保证有界。借鉴：CEL 被强调为非图灵完备、面向安全执行；这类约束要么不提供循环/递归，要么提供严格上界。citeturn7search3turn7search7  
- 调用白名单函数：技术指标（SMA/EMA/STD）、价格转换、货币转换（如果已有静态汇率输入）、简单聚合等。  
- 读取能力：仅能读取由 backend capability 声明暴露的字段/数据源（capability-driven），并且必须携带“数据版本/时序基准”，否则编译期拒绝。借鉴：Kubernetes Discovery/OpenAPI 的理念是能力可发现且可验证；你们要把这种思想收敛到内部 capability 描述上。citeturn3search1turn3search6  

**不允许（必须硬拒绝，避免语义与安全失控）**  
- 任意 I/O：网络、文件、时间系统调用、随机数（除非显式注入可回放的 seed/随机源）。无外部副作用与确定性是受限语言常见硬约束。citeturn0search0turn0search12turn0search1  
- 任意循环/递归/无界迭代：任何可能导致不可终止或资源不可控的结构都应拒绝，或改成由引擎提供“有界窗口算子”。借鉴：eBPF verifier 为避免不可终止路径采取 CFG/循环限制等检查；CEL 也以非图灵完备为安全目标之一。citeturn2search3turn7search3turn2search18  
- 修改系统状态：禁止写入账户状态、直接下单、绕过 Risk。Custom 只能产生“意图/信号/参数”，真正的动作仍必须走主链路。这里属于你们“保护主链路”的核心治理原则。citeturn4search1turn4search13  

### Custom 如何诊断、lower、拒绝

**诊断（Diagnostics）**  
- 语法/类型错误：指出 AST 节点位置、期望类型、实际类型。借鉴：CEL 的设计目标包含可 check（类型检查）与可嵌入。citeturn7search7turn7search11  
- 能力不可用：当表达式引用某数据源/字段/函数，但 backend capability 未声明可用时，报 `CapabilityNotAvailable`，并输出“缺失 capability 的名字/版本/所需参数”。借鉴：Kubernetes 通过 discovery 发布支持的资源与动词；你们内部也应以声明为准。citeturn3search1turn3search13  
- 资源超限：表达式复杂度（节点数）、执行步数、内存预算超过上限直接拒绝；借鉴：eBPF verifier 与受限表达式系统普遍需要资源上界。citeturn2search17turn2search3  

**lower（Lowering 到 Core IR）**  
- 关键原则：Custom 不是第二套运行时，而是“语法糖/便捷层”。最终必须降解成 Core IR 中已有的算子与数据流节点。借鉴：MLIR 的 dialect conversion/渐进 lowering 框架展示了“高层表示逐步降到低层合法操作集合”的通用方法。citeturn7search1turn7search17turn7search0  
- 具体做法（我的判断）：  
  - 将 Custom AST → “Expression IR（带类型/来源/时间语义注解）”；  
  - Expression IR → 调用 Core IR 的 `ComputeNode`/`SignalNode`/`ParamNode`（你们已有 Core IR 路线时可直接映射）；  
  - Spread 对齐/重采样等必须调用 Core IR 内建算子，禁止在 Custom 中自定义对齐逻辑（避免语义分叉）。citeturn1search0turn1search1turn7search0  

**拒绝（Hard Reject）与降级（Soft Fallback）**  
- Hard Reject：涉及 I/O、无界循环/递归、直接触碰 Execution/Risk 的任何企图。citeturn2search3turn0search0turn0search1  
- Soft Fallback：例如策略引用某“可选能力”，backend 未提供时，前端 capability-driven UI 隐藏该功能，但策略仍可在“解释/模拟模式”显示为不可执行（明确边界）。这与 OpenAPI/Kubernetes discovery 的“能力可发现、客户端据此调整行为”一致。citeturn3search6turn3search1  

---

## 插件系统的最低可行边界与治理范围

你要求“必须给出最低可行边界：最小 manifest、最小 capability、最小验证机制”，并且要明确“哪些现在不该做”。这一节给出 **MVP 插件系统**，且严格约束其不能绕开主链路。

### 事实：成熟系统通常先从 manifest 与声明式扩展点开始

- VS Code 扩展必须有 manifest（package.json），并通过 activation events / contribution points 声明其扩展点与激活时机；这是“声明式、可静态分析”的插件入口方式。citeturn2search4turn2search1turn2search8  
- 供应链治理（签名/安全更新）领域常用 TUF/Sigstore 等体系，但它们都强调规范化与安全属性；引入这些能力意味着显著的流程与基础设施投入。citeturn2search5turn3search0turn2search2  
- WebAssembly/WASI 路线提供“内存安全沙箱 + 能力（capabilities）授权”的模型，可用于强隔离插件执行；WASI 明确采用 capability-based security，并把外部资源访问建模为 capability。citeturn1search3turn5search0turn5search10  

### 最小 manifest（建议字段）

**我的判断**：第一阶段不需要做复杂的包格式，先用内部可加载的 JSON/YAML manifest（与构建产物绑定）即可，字段最少包含：

- `id`：全局唯一（建议反向域名/组织前缀）  
- `version`：SemVer（至少 major/minor/patch）  
- `apiVersion`：插件接口版本（与宿主兼容矩阵相关）  
- `entry`：入口（可执行文件 / wasm module / 内部类名）  
- `capabilities`：声明式能力列表（见下）  
- `extensionPoints`：声明该插件挂载在哪些主链路阶段（例如 `data.connector`, `agent.feature`, `explain.renderer`；**禁止** `execution.direct`）  
- `limits`：资源上限（CPU 时间/内存/并发数）  
- `replay`：回放承诺（是否纯函数、是否依赖外部时钟等）

版本字段与 SemVer 的基本规则可依赖规范本身；至少让“破坏兼容”体现为 major 变化。citeturn0search3turn0search7  

### 最小 capability 声明（建议结构）

**事实 + 借鉴**：  
- Kubernetes discovery 发布“支持的资源/版本/动词”；OSGi 有 requirement/capability 模型并可 introspect wiring；这些都说明“能力必须可声明、可发现、可诊断”。citeturn3search1turn3search7turn3search3  

**我的判断**：QuantPilot 的 capability 声明第一阶段用最小三元组即可：  
- `name`：能力名（稳定标识）  
- `version`：能力版本（SemVer 或简化版）  
- `contract`：输入/输出 schema（可用内部 schema ID；或直接引用 OpenAPI/JSON Schema 风格结构以便工具化）  
并补充两个关键属性：  
- `determinism`：是否确定性（回放要求）  
- `timeSemantics`：该能力涉及时间对齐/采样时，必须指向 Core IR 的内建语义（而不是插件自定义）

OpenAPI 的核心价值在于“机器可发现能力描述”；即使你们不直接用 OpenAPI 文件，也建议借鉴其“schema 可发现、可验证”的精神。citeturn3search6turn3search2  

### 最小验证机制（必须有“门禁”）

**事实（可借鉴的门禁模式）**：eBPF 通过 verifier 在加载前检查安全属性；这是“扩展不破坏稳定性”的关键结构。citeturn2search3turn2search0turn2search17  

**我的判断：MVP 验证至少包含三类检查**  
1) **声明一致性**：manifest schema 校验；capability contract 格式校验。  
2) **兼容性**：`apiVersion` 与宿主版本匹配；capability 的 required host features 是否满足（类 discovery）。citeturn3search1turn3search13  
3) **语义门禁**：  
   - 任何涉及时间对齐/采样的能力必须调用 Core IR 的内建算子；  
   - 插件不得直接调用 Execution；必须通过 Intent/Agent 产物进入 Risk 再到 Execution；  
   - 资源上界 enforce（超限拒绝/熔断）。  

### sandbox boundary（现实选择）

**我的判断（按“现在不空想”）**：  
- **短期（阶段 1/2）**：先用**进程边界/容器边界**或“同进程但强约束 API（能力声明 + 白名单调用）”，配合严格审计与回放记录；不要一上来就引入 wasm 组件模型，否则会引入 ABI/工具链/调试成本。  
- **中期（阶段 3）**：若要开放第三方插件，再引入 wasm/WASI 作为更强沙箱：WebAssembly 强调内存安全沙箱；WASI 明确用 capability-based security 建模外部资源访问，非常契合“插件只能拿到被授予的能力”。citeturn1search3turn5search0turn5search10turn5search1  

### governance / registry / signing（现在先不做的与以后再做的）

**事实**：  
- TUF 的目标是保护软件更新系统，即使仓库或签名密钥被攻破也能提供防护；Sigstore 提供签名与可验证日志以改善供应链安全。这些都属于“开放生态/第三方分发”阶段的关键能力，但引入成本高。citeturn2search2turn2search5turn3search0  

**我的判断**：  
- **现在不做**：公开 registry、第三方安装、签名/透明日志、复杂依赖解析、跨语言运行时矩阵。  
- **以后再做（当你们确实开放第三方时）**：  
  - 私有/公开 registry；  
  - 签名与验证（可借鉴 Sigstore keyless 签名与透明日志，或采用 TUF 保护更新链路）；  
  - 依赖解析与隔离（参考 OSGi 的复杂 wiring 之前，先评估你们是否真的需要动态多版本并存）。citeturn3search0turn2search5turn3search7  

---

## 三阶段 Roadmap 与退出条件

### 阶段目标：语义先稳，再扩展点，最后治理开放

#### 第一阶段：Spread 内建语义 + Custom 受限表达式落地

**交付物（我的判断）**  
- Core IR 增加 Spread 最小算子族：`AlignAsof` / `WindowAgg` / `Resample`，并把时间语义、缺失值语义、采样参数显式化。citeturn1search1turn6search2turn6search0  
- Custom：引入受限表达式（可选择 CEL 风格或 Starlark 风格），实现：  
  - 解析/类型检查（check）  
  - capability 校验（引用必须存在）  
  - lowering 到 Core IR（禁止自定义对齐语义）citeturn7search7turn0search1turn7search1turn7search17  
- 回放一致性：将策略运行关键事件序列记录为可回放日志/事件流（至少覆盖 Data/Intent/Agent/Risk/Execution/Fill 的关键输入输出），确保“同输入同输出”。事件溯源模式用于通过事件重建状态、审计与回放。citeturn4search1turn4search13  

**退出条件（必须可验证）**  
- Spread：同一组基准用例在不同数据源组合下回放一致，并能解释每一次对齐选择（选择了哪条 as-of，缺失原因是什么）。citeturn1search1turn1search0  
- Custom：表达式在给定环境下确定性求值；任何不允许构造被明确拒绝并给出诊断；所有表达式都能 lowering 到 Core IR（或明确标记“不支持”）。citeturn0search1turn7search3turn2search3  

#### 第二阶段：最小插件边界进入（内部插件），capability discovery 闭环

**交付物（我的判断）**  
- 插件 manifest（最小字段集）+ capability 声明（最小三元组 + determinism/timeSemantics）。citeturn2search4turn3search6turn0search3  
- capability discovery API：后端发布当前实例支持的 capabilities（类似 Kubernetes discovery 的“服务端权威声明”），前端按 capability 决定 UI 模块展示。citeturn3search1turn3search13  
- 最小验证器：安装/加载插件前做 schema 校验、版本兼容校验、语义门禁（禁止绕过主链路）。citeturn2search3turn2search1  

**退出条件**  
- 在不改主链路的情况下，能新增/替换“非核心能力”（如数据连接器、指标算子包、解释渲染器）并做到：  
  - capability 声明可发现；  
  - 回放可复现；  
  - 插件失败不会影响核心链路（隔离/降级策略生效）。citeturn3search1turn4search13  

#### 第三阶段：插件治理与更强隔离（必要时引入 WASM/WASI）

**交付物（只在确有第三方生态需求时）**  
- 插件包分发与签名验证：引入 Sigstore/TUF 类机制以防供应链攻击与不可信更新。citeturn3search0turn2search2turn2search5  
- 更强沙箱：评估 wasm/WASI（capability-based security）以限制插件对外部资源访问，降低“插件等于任意代码”的风险。citeturn1search3turn5search0turn5search10  
- 依赖解析与版本策略：在 SemVer 之上定义宿主 API 兼容策略与插件依赖策略。citeturn0search3turn3search7  

**退出条件**  
- 可以安全地把插件交给非核心团队/第三方：签名可验证、回放可复现、隔离可证明、升级可回滚。供应链安全框架（TUF/Sigstore）之所以存在，就是为了在更新系统被攻击时仍具韧性。citeturn2search2turn3search0turn2search5  

---

## 现在不必要的设计

**我的判断（结合“已有 beta、先稳主链路”的现实条件）**：以下设计现在做很可能是“平台空想”，会直接引入治理/测试/稳定性负担，且与近期目标（真实可运行、可验证、可回放、可解释）不成比例。

- **完整第三方插件生态**：公开 marketplace/registry、插件评分、自动更新等。没有签名/审计/回滚体系之前，这会把供应链风险引入核心系统；TUF/Sigstore 这类体系说明供应链治理本身就是一项工程。citeturn2search2turn3search0  
- **复杂依赖解析（多版本并存、菱形依赖自动解）**：像 OSGi 这类系统可以做很强的 wiring/解析，但复杂度与诊断成本极高；你们短期更需要“少依赖、强约束、版本跟随”。citeturn3search7turn3search3  
- **把 Spread 语义插件化**：这会让对齐/缺失值/采样语义分叉，最终伤害回放一致性与风险解释。对齐语义应该像 as-of join 那样被固定为核心语义。citeturn1search1turn1search0  
- **让 Custom 拥有完整语言能力（I/O、循环、任意库）**：这会把 Custom 变成“第二运行时”，与确定性、可审计目标冲突。受限表达式（CEL/Starlark）与“先验证再执行”（eBPF verifier）给出了更贴近你们目标的借鉴方向。citeturn0search1turn0search0turn2search3  
- **过早引入 wasm 组件模型作为默认插件形态**：WASI 的 capability-based security 很契合长期目标，但工具链与调试成本并不低；除非你们已经明确要开放第三方、不可信代码执行，否则先用最小边界把语义与验证闭环做实更关键。citeturn5search0turn1search3turn5search1  

---

### 总结性落点（回到你的“现实原则”）

- **先统一 sandbox 与 Core IR，再扩插件化**：对应推荐路径中“Spread 内建 + Custom lowering + 插件化最后”。citeturn7search0turn4search1  
- **先诚实边界，再扩能力**：用 capability discovery（服务端权威声明）驱动前端模块显示，借鉴 Kubernetes discovery / OpenAPI 的“可发现能力描述”。citeturn3search1turn3search6  
- **最怕扩展导致漂移**：把漂移风险最大的语义（Spread 的时间对齐、缺失值、采样）压进 Core IR 的可验证语义；把 Custom 压进受限表达式；插件先做最小门禁再谈开放。citeturn1search1turn0search1turn2search3