# QuantPilot 前端实施方案

## 文档定位

本文件是当前前端改版的执行版方案，用来替代此前分散的排版与美化草稿。

相关历史材料：

- [Frontend Layout Guide](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/frontend-layout-guide.md)
- [Frontend Visual Guide](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/frontend-visual-guide.md)

本文件只关心三件事：

- 先改什么，后改什么
- 每个阶段改哪些前端文件
- 每个阶段如何验收

---

## 目标

本轮前端实施不追求扩产品边界，只做已有能力的结构收敛和体验兑现：

- 把“策略管理”和“单策略工作台”拆成明确页面
- 把“编辑、编译、运行、回测、分析”从单页堆叠改成分层结构
- 把高频操作入口留在主路径，把低频工程细节下沉
- 用统一的信息架构承接后续视觉优化，而不是先堆样式

---

## 非目标

以下事项不在当前前端实施范围内：

- 不新增后端业务能力
- 不引入新的运行模式或新的交易边界
- 不把未完成能力包装成“已支持”
- 不在本轮重做状态管理模型的全部底层实现

---

## 当前问题

当前前端的主要问题不是单个组件不好看，而是页面职责过载：

- [EditorPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/EditorPage.jsx) 同时承载编辑、编译、运行、回测、事件、历史和账户信息
- [TopToolbar.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/TopToolbar.jsx) 混合了全局动作与单策略动作
- [EventStreamPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/EventStreamPanel.jsx) 承担了事件流、账户摘要、运行历史、回测历史和对比入口
- [PropertyPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/PropertyPanel.jsx) 同时容纳图级、节点级、源码级、诊断级信息
- [graphStore.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.js) 承担职责过多，导致页面重构成本偏高

---

## 目标路由

推荐稳定到以下路由结构：

- `/strategies`
- `/strategies/:strategyId`
- `/strategies/:strategyId/backtests`
- `/backtests/:backtestId`
- `/backtests/compare`

对应职责如下：

- `/strategies`：策略目录与全局管理入口
- `/strategies/:strategyId`：单策略工作台
- `/strategies/:strategyId/backtests`：单策略回测历史
- `/backtests/:backtestId`：单次回测详情
- `/backtests/compare`：多回测对比分析

---

## 页面职责

### 策略目录页

负责：

- 搜索、筛选、分组、排序
- 查看策略状态和最近运行摘要
- 快速进入单策略工作台
- 提供单一的“打开文件位置”入口，定位到已保存策略源码文件或图文件
- 提供策略删除入口，删除前必须显式确认
- 提供“打开空白工作区”入口，用于从当前策略上下文回到新的未保存图
- 策略清单表头与策略行共享同一套列模板，数据字段与构建、研究、文件、管理操作列都必须一一对齐

不负责：

- 直接承载图编辑细节
- 直接承载运行事件流全量内容
- 暴露重复的文件系统入口，避免用户在相近动作之间误判
- 把删除策略包装成批量清理运行、回测或实验历史的入口

动作收口规则：

- 策略清单行级文件动作只保留“打开文件位置”
- 定位路径优先使用已保存 QuantScript 文件；如果没有可用源码文件，再回退到策略图文件
- 后端返回给系统文件管理器的路径必须是已存在的规范化绝对路径
- 前端不再保留与目录级定位重复的行级动作，避免主路径出现失效入口
- 策略清单行级管理动作允许删除策略，但必须经过浏览器确认；删除范围限定为策略图文件、源码副本和版本目录
- 删除策略失败时必须在行内给出可见反馈，不能把后端错误静默吞掉
- 策略中心顶部“打开空白工作区”只创建本地未保存图并进入工作区，不写入持久化策略清单

### 单策略工作台

负责：

- 图编辑与属性配置
- QuantScript 编辑
- 编译、运行、回测动作
- 诊断与问题定位
- 当前运行态与最近结果摘要

不负责：

- 深度回测详情分析
- 多结果横向比较

### 回测详情页

负责：

- 结果摘要
- 工件视图
- 指标与结果解释
- 交易与权益变化分析

### 回测对比页

负责：

- 多回测关键指标并排比较
- 配置差异对照
- 数据集与参数差异定位

---

## 实施原则

- 先拆页面入口，再拆组件边界，最后再进一步拆 store
- 先做信息架构，再做视觉统一
- 先保证主路径稳定，再做高级模式和附加信息
- 新结构采用增量接管，不做一次性推翻

---

## 说明文本收窄方案

目标：

- 减少工作区内常驻说明文本的视觉占用，优先保证操作区、数据区和状态区的扫描效率
- 不删除说明信息，只改变说明信息的默认呈现方式
- 为后续逐块收窄说明文本提供统一交互基线，避免每个工作区各做一套提示样式
- 让界面默认呈现“业务状态和可执行动作”，把开发解释、实现边界和辅助说明下沉到按需查看

适用范围：

- 单策略工作台内的长说明、操作提示、概念解释、面板使用说明
- 回测详情页与对比页中会挤压主体内容的解释型文本
- 策略中心、工作区、详情页中会让主界面显得拥挤的状态卡说明、面板说明和流程说明
- 不适用于错误提示、危险操作确认、必须立即阅读的阻断信息

### 策略中心样板

策略中心是当前说明文本收窄的交互样板。

主标题说明已经落在：

- [StrategyHubInlineNote.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubInlineNote.jsx)
- [strategy-hub.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/strategy-hub.css)

策略中心 8 个状态卡说明已经落在：

- [StrategyHubSharedComponents.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubSharedComponents.jsx)
- [StrategyHubPage.test.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubPage.test.jsx)

当前样板规则：

- 卡片默认只展示标题和数值，不常驻显示解释性小字
- 原说明文本保留在标题旁的轻量说明入口中
- 说明入口不使用强视觉警告样式，不与主操作按钮争夺注意力
- 标题、桥接区、说明栏组成连续 hover 区域，光标穿过间隙时说明不消失
- 光标停留在说明栏上 3 秒后说明栏进入固定态
- 固定前用右上角圆环展示倒计时进度
- 固定后圆环切换为 `x` 关闭按钮
- 固定态下光标离开说明入口、桥接区和说明栏时，说明栏仍保持显示
- 固定态只能由用户点击关闭按钮退出

这套样板是后续治理其它页面说明污染时的默认实现。除非所在组件有明显交互约束，否则不要另起一套提示行为。

交互基线：

- 原本常驻的大段说明文本，优先收纳到对应标题、标签或轻量文案入口上，不额外堆叠大量 `!` 图标
- 鼠标移入说明入口文本时，立即显示悬浮说明窗，并展示完整说明文本
- 鼠标从入口文本移入悬浮窗时，悬浮窗保持显示
- 入口与悬浮窗之间如果存在视觉间距，必须提供连续的悬浮热区，避免光标穿过缝隙时误触关闭
- 鼠标离开入口文本和悬浮窗后，若当前未固定，悬浮窗立即消失
- 鼠标停留在悬浮窗上时，右上角出现圆环进度 UI
- 圆环以顺时针方向绘制，完整绘制时长为 3 秒
- 3 秒结束后，悬浮窗进入固定态，不再因为鼠标移出而消失
- 固定态下，圆环位置切换为 `×` 关闭 UI，仅允许手动关闭
- 固定态下允许用户用鼠标拖动悬浮窗，避免遮挡当前工作区的关键信息

状态定义：

- 默认态：仅显示标题或轻量说明入口，不显示说明窗
- 悬浮预览态：说明窗临时展示，离开即消失
- 固定态：说明窗持续显示，直到用户点击 `×` 主动关闭

说明：

- 主标题级说明可以支持拖动固定窗
- 小型状态卡说明可以先只实现固定和关闭，不强制支持拖动
- 如果后续用户测试发现状态卡说明遮挡主路径，再补拖动能力，而不是扩大卡片常驻文案

实现约束：

- 说明入口应尽量复用现有标题、标签或文案触发区，避免页面遍布说明图标
- 说明入口默认弱化，不与主按钮、风险提示、状态标签争夺视觉焦点
- 悬浮窗尺寸应优先服务中文阅读，避免过窄导致频繁换行
- 悬浮窗出现与消失应直接、克制，不使用拖沓动画
- 固定态与非固定态必须有明确关闭路径，不能制造“关不掉”或“找不到入口”的体验
- 同一工作区内说明收窄组件的图标、时长、固定逻辑和关闭逻辑必须统一

落地规则：

- 后续当某个工作区被指出“说明文本过多”时，优先按本方案收窄，而不是直接删除文案
- 如果某段说明文本长度很短，且不会明显干扰主路径，可保留为轻量静态文案，不强制改成 `!`
- 如果说明内容涉及当前状态、错误原因或操作后果，仍应以显式文本优先，不收进悬浮说明
- 如果说明文本只是解释“为什么这样展示”或“这个数值来自哪里”，优先收进说明入口
- 如果说明文本包含产品能力边界，允许收进说明入口，但不能删除事实或把不支持能力包装成已支持

### 后续收口优先级

下一批前端说明文本治理优先按以下顺序推进：

1. 策略工作区顶部状态、标签和 tab 说明
2. 工作区概览页中过长的 subtitle 文案
3. 诊断页中偏开发视角的英文说明和 compile/runtime 解释
4. 回测详情页与对比页中解释页面结构的 summary 文案
5. EventStreamPanel 和 PropertyPanel 中重复解释视图职责的说明文案

每批只做已有说明的呈现收窄和用户化表达，不扩展功能、不新增能力承诺。

---

## 分阶段实施

## Phase 0：壳层与入口准备

目标：

- 先把新页面壳层和路由入口建立起来

主要修改文件：

- [router.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/router.js)
- [App.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/App.jsx)
- 新增 `frontend/src/pages/StrategyHubPage.jsx`
- 新增 `frontend/src/pages/StrategyWorkspacePage.jsx`
- 新增 `frontend/src/pages/StrategyBacktestsPage.jsx`

验收：

- 新路由可访问
- 旧编辑页仍可回退
- 页面壳层不破坏现有测试主路径

## Phase 1：策略目录与工作台拆分

目标：

- 把“策略管理”和“策略编辑”从一个入口拆开

主要修改文件：

- [EditorPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/EditorPage.jsx)
- [TopToolbar.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/TopToolbar.jsx)
- [ModuleSidebar.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/ModuleSidebar.jsx)
- [PropertyPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/PropertyPanel.jsx)

验收：

- 策略目录页只呈现目录与摘要
- 单策略页只呈现单策略工作内容
- 全局动作与单策略动作不再混放

## Phase 2：运行与历史面板重构

目标：

- 把事件流、账户、运行历史、回测历史和对比入口重新分层

主要修改文件：

- [EventStreamPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/EventStreamPanel.jsx)
- [StrategyRunsPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/StrategyRunsPanel.jsx)
- [StrategyBacktestsPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/StrategyBacktestsPanel.jsx)

验收：

- 事件流是事件流，历史是历史，摘要是摘要
- 刷新、筛选、详情恢复路径清晰
- 回测详情与对比入口不再挤在主编辑区

## Phase 3：诊断与源码工作区重构

目标：

- 把配置、编译、运行、源码四类信息分区

主要修改文件：

- [PropertyPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/PropertyPanel.jsx)
- [StrategyCodePanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/StrategyCodePanel.jsx)
- [DiagnosticsPanel.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/DiagnosticsPanel.jsx)

验收：

- 用户能清楚区分“配置问题”和“编译诊断”
- 诊断项可以稳定定位到对应对象
- 源码编辑不再被埋在混合面板里

## Phase 4：视觉统一与响应式收口

目标：

- 在结构稳定后统一视觉语言与响应式规则
- 在各工作区逐步收窄冗长说明文本，统一切换到 `!` 说明入口 + 悬浮说明窗机制

主要修改文件：

- [styles.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/styles.css)
- 各页面和主面板组件样式
- 说明文本密集的工作区组件

验收：

- 桌面端主路径视觉层级一致
- 1024 / 768 / 560 宽度下主要页面不破版
- 状态语义、留白、标题层级统一
- 长说明默认不再大面积常驻，说明入口、悬浮窗、固定态交互保持一致
- 策略模板库默认收起，只在用户主动展开后展示内置模板，避免模板卡片抢占策略中心主界面注意力
- 策略模板库与策略清单保持同列宽度，策略驾驶舱从模板库所在行开始占据右列空间，避免主界面出现整行模板区
- 策略模板库收起态必须退化为单行紧凑条，不继续占用展开卡片的纵向内边距和标题下边距
- 策略中心左右列可以等高，但左列只允许策略清单承接剩余垂直空间；模板库折叠态和近期活动必须按真实内容高度排列
- 策略清单在中等宽度下必须切换到紧凑列模板；当左列不足以容纳表格与行级操作时，策略中心必须提前切到单列，不能让策略清单 UI 溢出覆盖策略驾驶舱
- 策略工作区内可收起的面板副标题统一收进标题说明入口；通用工作区卡片、修复队列、构建任务通道、模块模板、运行诊断、源码/配置/诊断面板和研究控制台不得继续常驻解释性副标题
- 构建页只保留搜索、折叠控制、推荐模块、最近使用、结构泳道、模块分组和可操作的画布聚焦切换；模块模板与画布顶部不得继续展示当前泳道、当前焦点、自动原因、模块库统计、最近编辑统计、结构健康度或当前选中类型等上下文状态卡
- 策略画布只能使用 React Flow 已注册的边类型；默认曲线边使用 `default`，不得继续写入未注册的 `bezier` 导致浏览器告警
- 策略中心不得继续承载编译、模拟、回测、部署等运行工具栏；策略中心只保留策略管理主路径，单策略运行动作必须进入工作区后执行
- 策略清单行级操作按“主操作 + 更多”分层：默认只显示 `打开工作区`，`打开回测页`、`打开文件位置`、`删除策略` 收进更多菜单；动作能力、aria 标签和测试钩子不得丢失
- 策略工作区必须在页级保留常驻主控制条；`编译`、`启动模拟`、`运行回测`、`停止` 属于主路径，保存、加载、导出、凭证、教程、新建和重置运行时属于低频工具，默认收进 `工具` 展开区
- 回测索引页同一刷新入口不得重复出现；若面板内部已有刷新历史能力，外层分析区不再额外放置同名按钮

### 交互简化收口方案

本阶段采用动作分层而不是功能删除：

- **主路径动作**：用户推进当前任务必须立即看到的动作，例如策略中心的打开工作区、工作区的编译/模拟/回测
- **低频工具动作**：保存、加载、导出、凭证、教程、新建、重置等工程动作，必须可达但不得常驻挤压主路径
- **上下文动作**：只在具体表格行、面板或筛选上下文里出现，避免全局重复
- **破坏性动作**：删除、重置类动作必须保留确认，不进入首屏主按钮组

验收补充：

- 策略中心首屏不出现运行工具栏
- 策略清单每行默认可见按钮不超过 2 个
- 工作区任意标签页都能访问编译、模拟、回测主动作
- 所有被收进菜单或展开区的动作保留原行为与自动化测试入口

### Adobe 式工作区演化计划

本计划吸收 Photoshop / Animate / Premiere Pro 的专业工具布局思想，但不照搬其复杂菜单层级。QuantPilot 的目标是形成稳定外壳、任务型 workspace、上下文 inspector 和时间型研究视图。

#### 设计边界

- 这是一项前端信息架构与视觉布局演化，不新增交易能力、编译能力或运行能力宣称
- 所有入口变更必须遵守 GP §8.9：下沉低频动作，不删除功能，不丢失可访问名称和测试钩子
- 所有 workspace 必须使用已有 capability / runtime / graph 状态作为真实来源，UI 不得提前展示未落地能力为 supported
- 任何新布局都必须支持回退到当前页面路径，不能破坏既有路由、深链和 E2E 流程

#### 目标 Workspace

| Workspace | 对标思路 | QuantPilot 职责 | 首屏主路径 |
|---|---|---|---|
| 策略中心 | Adobe Home / Project Hub | 创建、打开、搜索、筛选、最近活动、策略清单 | 打开当前工作区、打开空白工作区、刷新活动 |
| 构建工作区 | Photoshop / Animate 主编辑器 | 模块库、策略图、属性面板、诊断、源码入口 | 编译、保存上下文、定位阻塞项 |
| 研究回测工作区 | Premiere 时间型工作流 | 回测历史、事件流、资产曲线、对比队列、实验详情 | 运行回测、刷新历史、打开详情、打开对比 |
| 运行监控工作区 | Premiere Program Monitor + 运行控制台 | 模拟/实盘状态、账户、订单、风险事件、运行日志 | 启动模拟、停止、查看风险/执行状态 |

#### 固定外壳

- 左侧导航只承载 workspace 切换，不塞具体业务按钮
- 顶部或页级主控制条只承载当前 workspace 的主路径动作
- 中央区域承载主对象：策略图、资产曲线、事件时间线或运行监控视图
- 右侧 inspector 承载当前选中对象的属性、诊断、上下文动作和低频详情
- 底部或可折叠区域只在时间型任务中使用，用于事件、日志、订单或回放时间线

#### 分阶段落地

1. **Phase A：Workspace Taxonomy**
   - 明确四类 workspace 的路由、页面职责、主路径动作和低频工具动作
   - 输出每类 workspace 的入口矩阵和不可回退清单
   - 不改 UI，仅补测试清单和文档约束

2. **Phase B：构建工作区 Adobe 化**
   - 左侧模块库保持搜索、推荐、最近使用、分组
   - 中央策略图最大化，顶部只保留主控制条
   - 右侧 inspector 按选中对象切换：节点、边、画布、诊断
   - 移除重复状态卡，保留悬浮说明入口

3. **Phase C：研究回测工作区 Premiere 化**
   - 上方展示关键结果与资产曲线
   - 中央或下方承载回测/事件时间线
   - 右侧承载详情、对比队列和筛选上下文
   - 不再在同一层级重复展示刷新、详情、对比入口

4. **Phase D：运行监控工作区**
   - 将运行状态、账户、订单、风险和执行事件拆成清晰区域
   - 模拟与实盘状态必须显式区分，遵守 provider_native / runtime_simulated 标记规则
   - 任何真实交易相关入口必须继续受 capability、风控和执行端状态约束

5. **Phase E：视觉与响应式收口**
   - 统一面板边框、标题层级、状态 pill、图标按钮和紧凑表格
   - 对 1024 / 768 / 560 宽度建立 workspace 级响应式验收
   - 更新视觉回归快照，所有快照更新必须能说明布局意图

#### 验收门禁

- 每个 workspace 首屏主按钮数量必须可解释，低频动作必须可达但不常驻挤压主路径
- 每个被移动的动作必须有组件测试或 E2E 覆盖
- 每次视觉基线更新必须说明是布局意图变化，而不是掩盖回归
- 前端修改完成后至少运行 `cd frontend && npm run test`、`cd frontend && npm run build`、相关 E2E 和 `npm run test:e2e` 中受影响用例
- 文档或文案修改必须运行 UTF-8、用户可见文本和 i18n 门禁

---

## 测试与验收

每个阶段至少完成以下验证：

- `cd frontend && npm run test`
- 对关键页面做最小 E2E 或交互回归
- 对已拆分页面做人眼检查，确认信息层级和动作位置合理

本方案完成的最低标准：

- 主入口不再只有单一编辑页
- 单策略工作台职责明确
- 回测详情与对比路径独立
- 运行、历史、诊断、源码不再挤在同一层级

---

## 维护规则

- 当前前端结构变更，以本文件为执行基线
- 归档文档仅保留背景思路，不再作为直接实施顺序依据
- 新增页面或大改主路径前，先更新本文件的阶段说明与验收项
