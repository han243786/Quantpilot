# QuantPilot 前后端联调与 E2E 测试体系深度研究与可执行方案

## 结论与推荐方案

QuantPilot 这种“单机量化交易 runtime（含图编辑器、SSE 运行事件流、回测结果页、Rust/Axum API）”的测试体系，要把**确定性（determinism）**与**边界一致性（capabilities / beta boundary）**放在第一优先级；否则端到端浏览器用例会快速变成“不稳定、难定位、难维护”的负担（行业在测试金字塔与契约测试方面已有长期经验共识）。citeturn1search0turn1search1turn1search2

可直接执行的推荐方案（按落地性排序）：

**核心策略（我建议你们从“系统可测性”倒推测试分层，而不是从工具倒推）**  
1) **把“单机 runtime 的确定性回放（fixture/replay）”作为系统测试的底座**：E2E 与前后端联调都跑在“完全离线、可复现”的数据与执行仿真上（避免 CI 访问真实交易所、避免时间/并发导致的随机性）。这是 QuantPilot 与纯前端项目最大的差异（工程判断，针对你们的 runtime 属性与 beta 边界）。  
2) **把 /api/capabilities 变成“可测试的契约”**：用契约测试 + UI 显隐一致性 E2E 双保险，专门防止“能力声明与真实支持边界漂移”（你们已经踩过坑）。契约测试属于行业成熟实践，目的是减少对重型 E2E 的依赖。citeturn1search1turn1search2  
3) **SSE / runtime event stream：用“协议正确性 + 语义不变量（invariants）”来稳定测试**：  
   - 协议层：按照 WHATWG SSE 规范验证 event stream 的解析规则、UTF‑8 编码与 blank line dispatch 等关键点。citeturn7view1turn8view0  
   - 语义层：不对“时间戳/到达时序”做脆弱断言，而对“事件序列号、阶段流转、最终状态/聚合结果”做断言（工程判断，但以规范与 Playwright 的稳定性建议为依据）。citeturn10search3turn10search5  
4) **浏览器 E2E（Playwright）只做“少量 P0 主链路 + 少量 P0 失败路径”**：其余覆盖交给 service-level 联调测试、契约测试、前端组件测试、回放/characterization（golden master）测试，符合测试金字塔的成熟经验。citeturn1search0turn1search1turn2search0  

**推荐技术栈（不是“通用模板”，而是对 QuantPilot 痛点的针对性组合）**  
- **Rust/Axum service-level 联调测试**：直接对 `Router` 做请求（`tower::ServiceExt::oneshot()`），不必开真实端口；该方式是 Axum 官方测试指南推荐路径，速度快、定位准，适合你们“后端单测强但联调弱”的现状。citeturn6view0  
- **SSE 后端测试**：使用 Axum 的 `Sse` 响应与 `KeepAlive`（协议与 keep-alive 机制在 axum 源码文档中已有示例与 API），同时用 WHATWG/MDN 规范来定义断言口径（UTF‑8、blank line、Last-Event-ID 等）。citeturn7view0turn7view1turn5view2turn8view0  
- **契约测试**：  
  - “前端消费者 → 后端提供者”推荐用 **Pact**：前端在写 API client 单测时产出 contract，再由后端做 provider verification（Pact 官方流程与理念）。citeturn1search2turn3search7turn3search11turn3search19  
  - 若你们已经有 OpenAPI：补充 **Spectral** 做 spec lint（减少风格/一致性问题），必要时用 **Schemathesis** 基于 schema 做 property-based API fuzz（非常适合快速补齐“负向/边界”覆盖）。citeturn3search1turn3search9turn3search0turn3search4  
- **浏览器 E2E 与少量可控的视觉回归**：Playwright（原因：自带 trace、web-first assertions、截图比对能力；对 UI flake 的治理手段相对成熟，且你们需要面向图编辑器这类“真实浏览器行为”组件）。citeturn2search7turn10search3turn2search0  
- **前端图编辑器与数据变换逻辑的 property-based 测试**：JS/TS 侧用 fast-check；Rust 侧已有生态（proptest）可对“图 → compile payload”做不变量测试与 shrink，提高回归效率（适用于图结构组合爆炸的场景）。citeturn4search3turn4search2turn4search17  

**落地版本承诺（按你要求的时间窗）**  
- **最小可落地版本（两周内）**：建立“可复现回放底座 + 关键 P0 E2E + capabilities/乱码/错误文案门禁”。（见后文 roadmap）  
- **中期版本（一到两个月）**：补齐契约测试闭环（Pact 或 OpenAPI contract）、系统级 golden master 回归、SSE 断线重连与 Last-Event-ID、图编辑器复杂交互与更多回测页稳定性策略。

## 测试分层与边界定义

**行业共识：测试金字塔强调“数量与速度”在底层、少量 E2E 在顶层**，用来避免“测试越写越慢、越写越不稳定”的反模式。citeturn1search0  
结合 QuantPilot 的 runtime 与 SSE 特性，我建议的分层不是三层，而是**五层**（更贴合你们的系统形态与风险点）：

```
                 ┌──────────────────────────────┐
                 │ 浏览器 E2E（Playwright）      │  少量：P0主链路+关键失败路径
                 └──────────────────────────────┘
             ┌──────────────────────────────────────┐
             │ UI-API 契约一致性（capabilities/UI） │  少量但必须 blocking
             └──────────────────────────────────────┘
         ┌────────────────────────────────────────────┐
         │ 前后端联调系统测试（fixture/replay + API）   │  重点：compile/run/backtest + SSE
         └────────────────────────────────────────────┘
     ┌──────────────────────────────────────────────────┐
     │ 前端组件/集成测试（图编辑器、回测页渲染、错误文案） │  多：快、稳定、定位准
     └──────────────────────────────────────────────────┘
 ┌──────────────────────────────────────────────────────┐
 │ 后端单元/模块测试（你们已较完整）                      │  多：保护核心逻辑
 └──────────────────────────────────────────────────────┘
```

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["practical test pyramid diagram software testing","contract testing pyramid diagram"],"num_per_query":1}

**四类测试形态的合理边界（你明确要求的边界切分）**

**前后端联调测试（system tests，非浏览器）**  
- **目的**：验证“前端准备的 payload / graph 编译产物”与后端 API、runtime、回测引擎、SSE 事件模型在一起能跑通；但不承担复杂 UI 交互与布局断言（避免 flake）。  
- **方式**：Node 或 Rust 驱动（推荐 Rust 驱动后端能力 + Node 驱动前端构建产物的契约），对 API 发请求并消费 SSE，最后对聚合结果断言（工程判断）。  
- **边界**：允许 mock 外部依赖（交易所、网络），但**不 mock 内部模块链路**（Data→Intent→Agent→Risk→Execution→Fill），否则无法发现阶段间协议/语义漂移。

**浏览器 E2E（Playwright）**  
- **目的**：验证用户真正操作路径：图编辑→编译→运行→事件流可视化→回测详情页展示；再加少量关键失败路径（中文错误、能力不支持提示）。  
- **方式**：依赖真实浏览器行为；用 Playwright 的 web-first assertions 来减少时间相关 flake（官方建议）。citeturn10search3turn10search5  
- **边界**：只覆盖 P0，不在 E2E 里验证大量边界条件与组合爆炸（测试金字塔共识）。citeturn1search0  

**契约测试（contract tests）**  
- **目的**：把“接口形状 + 关键语义约束”从 E2E 中剥离出来，降低端到端成本；CDC（consumer-driven contract）是服务演进中降低耦合的成熟模式。citeturn1search1turn1search2  
- **方式**：  
  - `/api/capabilities`：强烈建议做契约（schema + 示例 + 版本策略），并在 UI 做显隐一致性验证（你们已暴露“漂移”风险）。  
  - `compile/run/backtest` API：对请求/响应 schema、错误 envelope、返回码、字段语义做契约。  
- **边界**：契约测试不替代“真实 runtime 行为正确性”；它解决的是“接口演进与一致性”。

**fixture / replay / characterization（golden master）测试**  
- **目的**：在你们这种 runtime 系统中，最能控制回归风险的往往不是 UI 截图，而是“输入固定 → 输出固定”的回放与基线对比（characterization test / golden master 的思路在业界用于快速保护复杂系统现有行为）。citeturn4search0  
- **方式**：固定数据集 + 固定时钟/随机种子 + 固定 capabilities → 输出 event log、最终回测结果 JSON（或其 canonical form），与基线对比。  
- **边界**：基线必须可解释、可更新；否则会变成“无意义的 snapshot 报警”（工程判断）。

## 全链路关键技术设计

这一节聚焦你指定的五个重点：图编辑器整链、SSE 稳定测试、回测页确定性、capabilities 一致性、中文错误与编码。

**图编辑器到后端 compile/run/backtest 的整链测试设计**

**当前适用（两周即可落地）的设计要点**  
1) **确立“图的可测试表示”**：  
   - 图编辑器内部状态（nodes/edges/params）必须能导出为**稳定排序、可版本化**的 JSON（canonical graph JSON）。  
   - 建议加入 `schema_version` 与 `graph_hash` 字段，作为联调测试的“输入指纹”（工程判断：否则同一图因字段顺序/默认值差异导致回归噪声）。  
2) **compile 产物要有“可测的中间表示（IR）”**：  
   - 最少暴露：`compile_warnings[]`、`compile_errors[]`（结构化），`resolved_capabilities`（后端实际采用的能力边界），以及 `ir_hash`。  
   - 这样联调测试可以在不跑全回测的情况下，先断言“图→编译”阶段的稳定性（工程判断）。  
3) **run/backtest 必须支持 test-mode 的 determinism 开关**：  
   - `--test-mode`：禁用外部网络、固定随机种子、固定时钟（或模拟时钟）、固定线程并发策略（至少固定任务调度引入的随机性）。  
   - 这是让 SSE 与回测结果可复现的前提（工程判断）。  

**未来适用（一到两个月）的增强点**  
- 引入 property-based：对“随机生成的图（受 beta 边界约束）”验证 compile 不崩溃、错误可解释、ir_hash 稳定、且错误消息含 code+可读文案（Rust 侧可用 proptest，JS 侧可用 fast-check）。citeturn4search2turn4search3turn4search17  

**SSE / runtime event stream 的稳定测试方法**

**官方文档事实：SSE 的解析与编码规则非常明确**  
- EventSource 事件流**总是以 UTF‑8 解码**，无法指定其他编码。citeturn7view1  
- 空行（blank line）触发事件 dispatch；`data:` 行可多行拼接；`:` 开头行是注释可被忽略；支持 `id`（Last-Event-ID）、`retry` 等字段。citeturn8view0  
- 服务端应返回 `text/event-stream`，并以双换行分隔事件块（MDN 对服务端格式说明清晰）。citeturn5view2turn7view1  

**针对 QuantPilot 的工程落地（当前适用）**  
1) **事件必须有“确定性主键”**：建议每个 run 分配 `run_id`，每条事件带 `seq`（单调递增的 u64），再带 `stage`（Data/Intent/Agent/Risk/Execution/Fill）与 `type`。这样测试断言不会依赖“到达时间”。（工程判断；但与 SSE 的“按接收顺序处理行/事件”约束相匹配）。citeturn8view0  
2) **SSE keep-alive**：Axum 的 `Sse` 支持 `KeepAlive`，可降低代理/网络环境的断开概率（尤其是 CI 环境偶发慢）。citeturn7view0  
3) **测试断言口径分两层**：  
   - 协议层（后端、非浏览器）：抓 raw stream，验证 `Content-Type: text/event-stream`、事件块格式、UTF‑8 可解码、事件能够被解析为 JSON。citeturn5view2turn7view1turn8view0  
   - 语义层（联调/浏览器）：只断言**阶段流转不变量**（例如：同一 `run_id` 的 `seq` 单调递增；必须出现 Data→…→Fill 的关键里程碑；Risk reject 时不应出现 Execution/Fills；最终状态与回测结果一致）。这是减少 flake 的关键工程策略。  
4) **断线重连测试的“可控触发器”**：WHATWG 规定浏览器会在连接关闭后重连，并发送 `Last-Event-ID`（若服务端提供 id）。citeturn8view0turn8view1  
   - 建议在 test-mode 暴露一个 endpoint 或 debug flag：可在指定 `seq` 后主动 close SSE，让测试验证“前端能自动重连并从 Last-Event-ID 继续”。（工程判断，但以规范机制为依据）。citeturn8view0  

**历史回测结果页的 deterministic 测试设计**

这里要特别避免“图表像素级截图 = 真相”的误区；回测页往往包含时间序列、浮点数、图表动画、排序等，极易导致视觉回归 flake（工程判断）。

**当前适用：以“数据确定性”为主，视觉为辅**  
1) **回测结果的权威数据源必须可导出**：比如 `GET /api/backtests/{id}/result.json`，提供稳定 schema 的结果（权益曲线、交易列表、统计指标）。  
2) **对数据做 canonicalization 再断言**：  
   - 对数组按主键排序（trade_id、timestamp），对小数统一 rounding 与格式，移除非业务字段（采集时间、trace_id）。这是 golden master 测试能长期维护的关键（工程判断，与 characterization 的实践一致）。citeturn4search0  
3) **前端页面 deterministic 测试优先用“DOM 语义断言”**：例如关键统计指标文本、表格行数/排序、空态/错误态文案；Playwright 的 web-first assertions 能等到 UI 达到稳定状态再断言，以减少异步渲染引发的 flake。citeturn10search3turn10search5  
4) **视觉回归只覆盖“静态、可控区域”**：Playwright 支持 `toHaveScreenshot()` 做视觉比较，但应限制在无动画、固定 viewport、固定字体的区域（例如：回测报告的摘要卡片，而非实时图表）。citeturn2search0  

**未来适用：数据层与视觉层都更强**  
- 对图表层：对“渲染输入数据”做 snapshot（而非像素图）；若确有必要做像素级，做“去动画、固定时间窗口、固定抗锯齿差异阈值”的治理（工程判断，Playwright 具备视觉比较能力但不保证天然稳定）。citeturn2search0  

**/api/capabilities 与前端模块显隐一致性验证**

你们明确提到“能力声明与真实支持边界容易漂移”，并且 beta 边界必须严格尊重（paper only、BTCUSDT only、binance/okx only、execution limited）。我建议把 capabilities 当成**系统的“外显契约与门禁中心”**。

**官方与行业依据**  
- CDC（consumer-driven contracts）强调“消费者把对提供者的期望显式化”，以降低演进耦合；Pact 的官方流程就是把这种期望固化成可验证的 contract。citeturn1search1turn1search2turn1search10  

**QuantPilot 的可执行做法（当前适用）**  
1) **capabilities schema 固化**：无论你们用 Pact 还是 OpenAPI，都要让 capabilities 有可验证 schema（包含：支持的市场/交易所/品种/模式、execution 限制、是否启用某些模块）。  
2) **双向一致性测试**：  
   - Provider（后端）测试：在 test-mode 下 `/api/capabilities` 返回必须匹配 schema，且与后端实际路由/模块开关一致（避免“宣称支持但实际 404/501”）。  
   - Consumer（前端）测试：用同一份 schema 生成 TS 类型或校验器；并在组件测试中验证模块显隐与 capabilities 映射表一致（工程判断）。  
3) **E2E 做“显隐烟测”**：在浏览器中启动后端不同 capabilities profile（例如：只开 binance、关闭 okx；或禁用某类 execution），验证 UI 的入口是否正确显示/隐藏，并且点击被隐藏能力的 deep link 会给出清晰错误（这能抓住最致命的“漂移”）。  

**中文错误提示、用户可读文案、编码问题纳入测试**

这里必须同时处理**编码正确性**与**文案质量**两类问题。

**官方标准事实：JSON 与 SSE 的 UTF‑8 约束**  
- RFC 8259 明确：跨系统交换的 JSON 文本必须使用 UTF‑8 编码，且不得在网络传输的 JSON 前加 BOM。citeturn5view1  
- SSE/EventSource 的事件流总是以 UTF‑8 解码。citeturn7view1  

**QuantPilot 的工程落地（当前适用）**  
1) **统一错误 envelope（可读 + 可机读）**：至少包含 `code`（稳定枚举）、`message`（用户可读中文）、`details`（技术细节，可选）、`hint`（下一步操作建议，可选）、`capability_violation`（是否越界）。这是把“错误提示质量不足”变成可测试对象的前提（工程判断）。  
2) **编码回归用例**：  
   - 后端联调测试：构造包含中文的典型错误（例如“仅支持 BTCUSDT / 仅支持 paper”），验证响应体字节可 UTF‑8 解码、字段值正确，且不含 BOM（依据 RFC 8259）。citeturn5view1  
   - SSE 测试：发送包含中文的 event.data，验证浏览器侧显示不乱码（依据 SSE UTF‑8 解码规则）。citeturn7view1  
3) **文案质量测试（可操作化）**：在测试中对关键错误码做“三要素断言”：  
   - 面向用户：中文 message 不为空、长度合理、包含可行动建议（例如“请切换到 paper 模式”）。  
   - 面向定位：details/trace_id 存在（但不直接展示给用户）。  
   - 面向边界：若是 capabilities 越界，必须明确指出边界（paper only、BTCUSDT only 等）。  
   这类断言是工程判断，但可以显著减少“错误提示质量不足”的回归。

## 关键场景清单与优先级

下表给出 **至少 12 个**关键场景（我给 15 个），并按 P0/P1/P2 给出推荐落点（层级）与主要断言口径。所有场景都严格以你描述的 beta 边界为前提（不假设 live trading、多品种、多市场）。

| 优先级 | 场景（用户视角/系统视角） | 推荐测试层（主） | 断言重点（稳定性友好） |
|---|---|---|---|
| P0 | 图编辑器加载“最小策略图”→ compile 成功 → run 启动 → SSE 收到 Data/Intent/Agent/Risk/Execution/Fill 关键里程碑 → 前端事件流 UI 正确渲染 | 浏览器 E2E + 联调系统测试 | 不断言时间戳；断言 `run_id` 一致、`seq` 递增、阶段里程碑齐全（工程判断；SSE 有明确解析/顺序语义）。citeturn8view0 |
| P0 | 回测：同一 fixture 数据集下 backtest 结束 → 回测详情页关键指标（收益、回撤、交易数）与 golden JSON 一致 | 联调系统测试 + 前端组件测试 | 数据 canonicalization 后对齐；页面只断言摘要文本/表格排序（characterization 思想）。citeturn4search0 |
| P0 | `/api/capabilities` 返回 paper only + BTCUSDT only + 交易所仅 entity["company","Binance","crypto exchange"]/entity["company","OKX","crypto exchange"] → 前端只显示对应入口；隐藏/禁用其他入口 | 契约测试 + 浏览器 E2E 烟测 | UI 显隐与 capabilities 映射一致；深链进入禁用功能必须给清晰错误（CDC 目的）。citeturn1search1turn1search2 |
| P0 | 越界操作：选择非 BTCUSDT 或非 paper 运行 → 后端拒绝 → 前端显示中文可读错误且不乱码 | 联调系统测试 + 浏览器 E2E | RFC 8259 UTF‑8 & BOM；SSE UTF‑8；错误 envelope 三要素（工程判断 + 标准约束）。citeturn5view1turn7view1 |
| P0 | SSE 断线：运行中服务端主动 close（test-mode）→ 浏览器自动重连 → 基于 Last-Event-ID 继续消费 | 联调系统测试（协议层） + E2E（轻量） | Last-Event-ID 与重连机制为规范内容；断言事件不重复/不丢关键里程碑（工程判断基于规范）。citeturn8view0turn8view1 |
| P1 | 编译失败：图有缺失参数/非法连线 → compile 返回结构化错误 → 前端定位到图节点并提示中文 | 前端组件测试 + 联调系统测试 | 错误 code 稳定、指向 node_id/edge_id；前端 highlight 正确（工程判断）。 |
| P1 | Risk 拒绝路径：策略触发超限（如超仓/频率）→ SSE 出现 RiskRejected → 无 Execution/Fills → UI 状态为“被风控拒绝” | 联调系统测试 | 断言阶段不变量：RiskRejected 后不得出现 Execution/Fills（工程判断）。 |
| P1 | Execution 限制：execution module limited（例如仅市价/仅单向）→ 若用户配置不支持 order 类型 → 后端拒绝并返回“边界解释清晰”的错误文案 | 契约测试 + 联调系统测试 | capabilities 与实际限制一致；错误文案包含边界（工程判断）。 |
| P1 | 回测列表/历史记录：同一 backtest_id 在列表中可见、点击进入详情页不丢状态 | 浏览器 E2E（少量） | 只断言关键路由与核心 DOM；避免对图表像素断言（工程判断；Playwright 支持截图但需谨慎）。citeturn2search0 |
| P1 | 运行事件流 UI：高频事件下（fixture 生成）前端不会卡死；关键事件可过滤/搜索 | 前端组件测试 + 性能烟测（nightly） | 断言功能可用与关键交互；性能只做趋势监控（工程判断）。 |
| P2 | OpenAPI / schema lint：新增字段、错误码时必须通过 Spectral 规则与向后兼容检查 | CI 规则（blocking） | Spectral 为 OpenAPI lint 提供规则体系（官方文档）。citeturn3search1turn3search9 |
| P2 | API 负向 fuzz：基于 schema 自动生成非法输入，服务端不 panic、返回合理 4xx | nightly（Schemathesis） | Schemathesis 基于 OpenAPI 自动生成大量用例（官方文档）。citeturn3search0turn3search4 |
| P2 | 图编辑器 property-based：随机图（受 beta 边界约束）序列化→反序列化不丢信息；compile 不崩溃 | JS fast-check / Rust proptest | PBT 框架支持生成与 shrink（官方文档）。citeturn4search3turn4search2 |
| P2 | 视觉回归：回测摘要卡片、关键中文提示在不同字体/分辨率下不乱 | Playwright screenshot（nightly） | Playwright 支持 `toHaveScreenshot` 做视觉比较（官方文档）。citeturn2search0 |
| P2 | Trace 可观测性：E2E 失败自动保留 trace/screenshot/video 便于定位 | CI（nightly + on-failure） | Playwright Trace Viewer 与报告工具链为官方能力。citeturn10search1turn10search10 |

## 目录结构、fixture、稳定性与 mock 策略

这一节给出你要求的：目录结构建议、fixture 组织建议、稳定性策略、mock 与真实后端边界。

**推荐目录结构（单仓或多仓都可按此落）**

```text
repo/
  backend/                          # Rust/Axum
    crates/...
    src/...
    tests/
      api/                           # Router oneshot / service tests
      sse/                           # SSE protocol tests (raw stream parsing)
      fixtures/
        market/
          btcusdt_1m_2024-01-01_2024-01-02.parquet
        exchange_replay/
          binance_paper_orders.jsonl
          okx_paper_orders.jsonl
        graphs/
          minimal_strategy.v1.json
          risk_reject_case.v1.json
        expected/
          backtest_minimal_strategy.v1.result.json
      helpers/
        test_server.rs               # spawn test-mode server (optional)
        canonicalize.rs              # JSON canonicalization utilities
  frontend/                         # JS/TS graph editor + backtest UI
    src/...
    tests/
      unit/
      component/
      fixtures/
        graphs/                      # same fixtures as backend, symlink or copy
        snapshots/
  qe/                               # “质量工程”跨栈资产（强烈建议独立出来）
    contracts/
      capabilities.schema.json
      openapi.yaml                  # 如果已有
      spectral.yaml                 # lint rules
    e2e/
      playwright.config.ts
      tests/
        p0_smoke.spec.ts
        capabilities.spec.ts
        i18n_errors.spec.ts
      fixtures/
        profiles/
          paper_only.json           # test-mode 启动参数/能力配置
    scripts/
      start_test_backend.sh
      run_e2e.sh
```

**为什么要把 graphs/ 与 expected/ 放进 fixtures 并跨前后端共享？**  
因为 QuantPilot 的主链路由“图（前端）→ compile/run/backtest（后端）→ SSE/回测页（前端）”贯穿，最容易漂移的就是“图的语义与后端解释的一致性”。共享 fixture 能把“漂移”早发现、早定位（工程判断）。

**fixture 组织建议（当前适用）**  
- **graphs/：用版本化命名**（`name.v1.json`），包含 `schema_version` 与 `graph_hash`（防止隐式变更）。  
- **market/：最小数据集优先**：只覆盖 BTCUSDT、只覆盖你们回测算法所需最小时间窗（如 1 天 1m K 线），避免每次 CI 拉巨量数据（工程判断）。  
- **expected/：保存 canonical JSON**：建议保存“结果数据层”的 golden master，而不是保存 UI 截图；截图作为补充。citeturn4search0turn2search0  
- **exchange_replay/：只保存 paper 交易回放**：严格遵守 beta 边界，且保证离线可重复（工程判断）。

**稳定性策略（把 flake 当成系统性问题治理）**

**针对浏览器测试的官方建议（应当遵循）**  
- 使用 Playwright 的 **web-first assertions**，它会等待条件满足并重试，减少因异步渲染造成的 flake。citeturn10search3turn10search5  
- 使用语义化 locators（例如 `getByRole`），locators 是 Playwright 自动等待与可重试的核心机制。citeturn9search0turn9search2  

**QuantPilot 特有的稳定性治理（工程判断，但高度建议）**  
- **禁用测试中的真实外部网络**：任何访问交易所/行情的真实网络都极易让 CI 不稳定，也会破坏 determinism。  
- **时间与随机性统一注入**：运行时使用可注入时钟/随机种子；回测结果必须不依赖墙钟时间。  
- **事件序列语义化**：给 SSE 事件加 `seq` 与 `run_id`，避免对 timing 做断言。  
- **把“错误文案质量”变成断言对象**：避免“能跑通但用户看不懂”在 beta 期反复出现。  

**mock 与真实后端的边界（务实分界线）**  
- **必须真实的**：compile、runtime 阶段流转（Data→Intent→Agent→Risk→Execution→Fill）、回测核心计算逻辑、/api/capabilities 的最终裁决。  
- **允许 mock/替身的**：交易所网络交互（binance/okx 的真实 API）、网络不确定因素、不可控时钟、随机源。  
- **前端测试中允许 mock 的**：非主链路页面的数据请求、用于组件测试的 API 响应与 SSE 数据（通过固定 fixture），以减少 UI 层测试成本（工程判断）。

## CI 集成与质量门禁

你们需要明确哪些测试 blocking、哪些 nightly，否则会出现两种极端：要么 PR 永远红（没人信测试），要么 PR 永远绿（测试形同虚设）。

**blocking 与 nightly 的推荐划分（当前适用）**

| 阶段 | 运行频率 | 是否 blocking | 包含内容 | 依据/理由 |
|---|---|---|---|---|
| Rust 单测 + lint | 每个 PR | 是 | `cargo test`、关键模块单测、clippy/fmt（如已接入） | 你们后端单测已强，应继续作为基础门槛（工程判断）。 |
| 后端 service-level 联调（Router oneshot） | 每个 PR | 是 | compile/run/backtest API 的关键 happy path + 关键越界错误；不启动真实端口 | Axum 推荐可直接测 Router（速度快）。citeturn6view0 |
| capabilities 契约门禁 | 每个 PR | 是 | `/api/capabilities` schema 校验 + 前端显隐映射表单测 | 你们的已知痛点，且成本低、收益高（工程判断 + CDC 思路）。citeturn1search1turn1search2 |
| Playwright P0 烟测（少量） | 每个 PR | 是 | 3~6 条 P0：图加载→compile→run→SSE→回测页；中文错误不乱码 | Playwright web-first assertions 与 trace 有助于稳定与定位。citeturn10search3turn10search1 |
| Playwright 全量 E2E | nightly | 否（nightly） | 更多交互、更多失败路径、更多浏览器矩阵 | 避免拖慢 PR；同时保留覆盖。citeturn1search0 |
| 视觉回归（少量静态区域） | nightly | 否 | `toHaveScreenshot` 仅覆盖稳定区域 | Playwright 支持视觉比较，但需治理以免 flake。citeturn2search0 |
| OpenAPI lint（Spectral） | 每个 PR | 视情况（建议是） | OpenAPI 风格与一致性规则 | Spectral 为 OpenAPI lint 提供规则体系。citeturn3search1turn3search9 |
| Schema fuzz（Schemathesis） | 每周/按需 | 否 | 基于 schema 的负向与边界自动化 | Schemathesis 的价值在于快发现边界 bug，但用例量大更适合非 PR 阶段。citeturn3search0turn3search4 |

**CI 产物（可观测性）**  
- Playwright：失败保留 trace/screenshot，并在报告中可打开（官方工具链）。citeturn10search1turn10search10  
- 后端联调：失败时输出 run_id、graph_hash、事件日志（JSONL），便于复现（工程判断）。  

## 风险列表与演进路线图

**会让测试体系变脆弱的设计（风险清单）**  
1) **让 E2E 承担过多覆盖**：E2E 一多就慢且 flaky，最终团队绕开测试（测试金字塔的经典失败模式）。citeturn1search0  
2) **SSE 断言绑定“时间戳/间隔/到达时间”**：SSE 的语义是按流解析与 dispatch，网络与调度会让 timing 不可控；应断言 seq/阶段不变量。citeturn8view0  
3) **CI 访问真实交易所 API**：网络抖动、限频、账号与密钥问题都会让测试不可依赖（工程判断）。  
4) **回测结果缺少 determinism 开关**（时钟/随机/并发未控制）：任何一次“同输入不同输出”都会摧毁 golden master 与联调测试价值（工程判断）。  
5) **capabilities 只是文档/前端常量，而非后端权威输出**：这会使“能力漂移”无法被自动化及时发现（工程判断；CDC 的目的正是避免此类漂移）。citeturn1search1turn1search2  
6) **错误文案无结构化 error code**：只靠字符串匹配会让测试脆弱且难国际化；同时难以保证“用户可读质量”（工程判断）。  
7) **视觉回归覆盖动态图表全屏截图**：极易因渲染差异/动画导致误报；应限制区域或转为数据层断言。citeturn2search0  

**优先级排序的落地 roadmap**

**两周内最小可落地版本（当前适用）**  
- **第一个里程碑：可复现回放底座（第 1 周）**  
  - 加 `--test-mode`：固定时钟/随机种子、禁外网、指定能力 profile（paper only、BTCUSDT only、binance/okx only）。  
  - 建立 `fixtures/graphs` 与 `fixtures/market`（最小 BTCUSDT 数据集）与 `expected/backtest_result`。  
  - 新增后端联调 test harness：用 Axum Router oneshot 覆盖 compile/backtest 的核心 API（官方推荐的 Router 测法）。citeturn6view0  
  - 新增 SSE 协议层测试：验证 `text/event-stream`、UTF‑8、blank line dispatch、事件可解析（WHATWG + MDN）。citeturn7view1turn5view2turn8view0  

- **第二个里程碑：P0 E2E + capabilities + 中文门禁（第 2 周）**  
  - Playwright 建 3~6 条 P0 烟测：  
    - 图加载→compile→run→SSE→回测详情页关键指标  
    - capabilities 显隐一致性（至少 2 个 profile）  
    - 中文错误提示不乱码（API 返回 + SSE 返回 + UI 展示）  
  - 失败保留 trace/screenshot（便于定位）。citeturn10search1turn10search10  
  - 在 CI 上把上述 P0 与 capabilities/乱码测试设为 blocking。

**一到两个月中期版本（未来适用）**  
- **契约闭环**：  
  - Pact：前端 API client 单测产出 contract，后端 provider verification 作为 PR 门禁（Pact 官方流程）。citeturn1search2turn3search7turn3search11turn3search19  
  - 或 OpenAPI 合约：Spectral lint + schema 校验 +（可选）Schemathesis fuzz 增强负向覆盖。citeturn3search1turn3search0turn3search4  
- **更强 determinism**：引入更严格的 simulated clock、并发确定化策略；把回测结果与事件日志做 golden master（characterization）。citeturn4search0  
- **图编辑器深覆盖**：对图序列化/反序列化、参数约束、局部 compile 的不变量做 property-based（fast-check / proptest）。citeturn4search3turn4search2  
- **SSE 断线重连与 Last-Event-ID 的完整覆盖**：包括断线点续传、重复事件去重策略、前端 UI 的 reconnect 状态提示（规范依据）。citeturn8view0turn8view1  
- **视觉回归谨慎扩展**：仅对静态区域进行 `toHaveScreenshot`，并建立“更新基线的审核流程”（Playwright 视觉比较机制）。citeturn2search0