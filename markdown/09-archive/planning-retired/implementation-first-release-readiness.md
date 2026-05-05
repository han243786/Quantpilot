# 首次发布就绪

本文件是基线提交就绪和首次发布所有者决策的单一活跃入口。

它不重新开放功能范围。
已完成的清理记录归档于[已退休规划文档](../../archive/planning-retired/README.md)。

## 当前状态

上次接受的基线审查日期：`2026-04-24`

最新重新检查日期：`2026-04-28`

最新的 `2026-04-28` 依赖和视觉/布局清理保持相同的 paper-runtime beta 边界。之前的 `2026-04-26` 完整收尾包装器在 P0 和 P1 收尾后通过。
已建立的产品界面保持不变，仍然是 paper-runtime beta。
当前工作树在整洁快照之前仍需要基线交接审查：

- 工作树包含广泛的有意源代码/文档变更集，应作为一个基线批次审查或在提交前拆分
- 生成的审查截图可能作为手动证据存在，不应与运行时产品真实数据混淆
- 剩余的依赖审计风险仅接受用于私有基线使用，并且仍然阻塞公开发布声明

仓库在接受的门禁通过后，技术上已准备好进行可信的私有基线快照：

- 收尾门禁合约是明确的
- 活跃文档指向当前 beta 边界
- 活跃的 markdown 和前端文件应保持 UTF-8 无 BOM
- 乱码由文本检查守卫
- 占位符 [LICENSE](/D:/rust-js-pr/QuantPilot/quantpilot/LICENSE) 使当前法律状态明确

当前的发布决策是明确的所有者决策，而非隐藏的功能工作：

- `LICENSE` 仍然是占位符
- 所有者原则上允许私有基线提交
- 所有者接受当前的收尾门禁集作为预期的私有基线门禁集
- 所有者接受当前的前端依赖审计风险仅用于私有基线使用
- 公开仓库可见性仍然受阻，直到单独的公开发布批准替换占位符私有/许可姿态
- 公开发布就绪不得在依赖审计、依赖迁移策略和出站许可决策仍然未决时声明

## 线程关闭说明

线程于 `2026-04-24` 在私有基线提交 `ad0b903 Close private baseline readiness` 后关闭。

下一个线程应从干净的私有基线工作树开始，并保持相同的发布边界：

- 本次收尾不暗示新的功能范围
- 仓库保持私有
- 接受的私有基线门禁仍然是 `cmd /c tools\run-closeout-gates.bat`
- 当前的前端审计风险仅接受用于私有基线使用
- 公开发布仍然受阻，直到依赖迁移、最终许可证文本和公开可见性批准在以后的所有者批准的线程中处理

## 仓库可见性决策

所有者决策：

所有者报告所有三个发布评分维度现在至少为 `9/10`：

- 功能开发进度 `>= 9/10`
- 仓库稳定性 `>= 9/10`
- 发布就绪度 `>= 9/10`

所有者仍然选择在任何公开发布之前保持仓库私有。
通过 `9/10` 阈值不会自动授权公开仓库可见性、公开发布标签或出站许可证替换。
该分数仅是私有基线就绪信号；它不是公开发布就绪声明。

当前的保留所有权利占位符许可证仍然是预期的私有状态，直到公开发布资格被明确重新考虑。

## 当前所有者决策

| 决策 | 所有者回答 | 影响 |
|---|---|---|
| 功能开发进度 `>= 9/10` | 是 | 分数阈值已满足。 |
| 仓库稳定性 `>= 9/10` | 是 | 分数阈值已满足。 |
| 发布就绪度 `>= 9/10` | 是 | 分数阈值仅对私有基线已满足。 |
| 可创建私有基线提交 | 是 | 原则上允许。 |
| 当前门禁集被接受为基线门禁集 | 是 | `tools\run-closeout-gates.bat` 是私有基线门禁。 |
| 当前前端审计风险接受用于仅私有基线使用 | 是 | Vite/esbuild 审计发现不阻塞私有基线，但仍然阻塞公开发布声明。 |
| 仓库在公开发布前保持私有 | 是 | 公开发布和出站许可证替换仍然受阻。 |

## 公开发布阻塞项

当前状态面向私有基线。
在本文档中所有阻塞项通过实现或所有者决策关闭之前，不要将仓库描述为公开发布就绪。

| 阻塞项 | 当前状态 | 所需收尾 |
|---|---|---|
| 前端依赖审计 | `npm audit --audit-level=moderate` 报告仅 Vite/esbuild 链，在通过 `frontend/package-lock.json` 中的 `postcss@8.5.12` 移除了 `postcss <8.5.10` 中等发现后。`npm audit fix --dry-run --audit-level=moderate` 对剩余链没有非破坏性修复，指向破坏性的 Vite/Vitest 迁移路径。所有者接受此剩余风险仅用于私有基线使用。 | 仍然阻塞公开发布声明，直到主要依赖迁移完成且完整门禁为绿色。 |
| 依赖升级策略 | `npm outdated` 显示审计修复路径不是补丁级更新。所有者选择不将该迁移强制推入私有基线。 | 在任何公开发布之前，将 Vite/Vitest 主要迁移视为未来的专用 P2 批次。 |
| 出站许可证 | `LICENSE` 仍然是保留所有权利占位符文本。 | 仅在所有者和公开发布资格以及最终许可证文审批后替换。 |
| 仓库可见性 | 当前姿态仅为私有基线。 | 保持私有，直到存在单独的公开发布批准。 |

当前运行的本地审计证据存储在 `storage/audit/npm-audit-2026-04-24.json`。
该文件有意被忽略，不是产品真实数据。
版本化的仅私有风险接受记录是[私有基线风险登记册](./implementation-private-baseline-risk-register.md)。

## 基线放行或停止

基线提交仅在以下所有条件成立时才能进行：

- `cmd /c tools\run-closeout-gates.bat` 通过
- 活跃文档仍然匹配当前 beta 边界
- 生成的和运行时工件保持在版本化产品真实数据之外
- `LICENSE` 反映基线的预期法律状态
- 仓库可见性保持私有，除非存在单独的公开发布批准
- 所有者接受特定的门禁集作为基线门禁集

如果上述任何项为假，则尚不创建基线提交。

## 所有者操作

对于当前的收尾阶段，仅所有者的操作是：

1. 保持仓库可见性私有，直到功能开发进度、仓库稳定性和发布就绪度都至少为 `9/10`。
2. 保持 `tools\run-closeout-gates.bat` 作为接受的私有基线门禁，除非以后的所有者决策替换它。
3. 仅在接受的基线检查为绿色时创建私有基线提交。
4. 保持当前的审计风险接受仅限私有；不要使用它来声明公开发布就绪。
5. 仅在重新考虑公开发布资格时重新审视出站许可证文本。

不要将此扩大为路线图。

## 提交前验证

从仓库根目录运行规范包装器：

```powershell
cmd /c tools\run-closeout-gates.bat
```

包装器是基线置信度检查。
它涵盖 UTF-8、面向用户文本、能力治理、Rust 工作区测试、前端单元测试、前端生产构建以及隔离的 API 模拟合约下的前端 E2E。

使用[测试层期望](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)报告门禁证明了什么。
不要将绿色门禁描述为不支持的产品能力的证明。

## 收尾执行流程

本节是剩余清理阶段的活跃工作流程。
它不添加新的功能范围。

使用每个批次移除风险、关闭漂移和验证保留的 beta 界面。在执行这些步骤时不要扩大产品界面。

### 范围守卫

在任何实现批次之前，确认提议的更改满足以下所有条件：

- 它修复了现有缺陷、漂移、门禁失败、发布卫生问题或措辞不匹配
- 它保持在当前 paper-runtime beta 边界内
- 它不添加新的公共能力声明
- 它不扩展支持矩阵
- 它不扩展保留的正式 QuantScript 主干
- 它不引入后端能力和运行时事实之外的第二个真实数据源

如果任何项为假，将工作推迟到当前收尾阶段之外。

### 当前优化检查清单

本检查清单是活跃的 `2026-04-26` 收尾队列。
它仅用于缺陷移除、漂移修复、发布卫生和最终打磨。
不得用于添加产品广度、扩大支持矩阵、扩展保留的 QuantScript 主干或引入第二个真实数据源。

#### P0：阻塞门禁恢复

状态：于 `2026-04-26` 完成。

| 项 | 目标 | 范围 | 验收 | 状态 |
|---|---|---|---|---|
| 修复运行历史 E2E 回归 | 保存的模拟在刷新后出现在运行历史卡片中。 | 运行时工件保存接线、运行列表 API 映射、E2E 模拟合约、运行历史过滤器。 | `cd frontend; cmd /c npm run test:e2e` 通过运行模拟冒烟路径。 | 完成 |
| 修复回测历史 E2E 回归 | 保存的回测在刷新后出现在回测历史卡片中。 | 运行时工件保存接线、回测列表 API 映射、E2E 模拟合约、回测历史过滤器。 | `cd frontend; cmd /c npm run test:e2e` 通过运行回测冒烟路径。 | 完成 |
| 移除浏览器 alert 使用 | 行级操作失败通过现有的内联失败界面渲染。 | `StrategyHubRosterRowActions.jsx`、action-failure 文案、相关测试。 | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1` 通过。 | 完成 |
| 验证中文文本渲染 | 前端和文档文本保持可读的 UTF-8，非乱码。 | 活跃 markdown、面向用户的前端字符串、批次触及的渲染 UI 界面。 | UTF-8 和面向用户文本检查通过，更改的 UI 文案在渲染形式中审查。 | 完成 |
| 重新运行接受的收尾门禁 | 在阻塞修复后恢复基线置信度。 | 完整仓库门禁包装器。 | `cmd /c tools\run-closeout-gates.bat` 通过，无需隐藏手动设置。 | 完成 |

#### P1：合约和行为收尾

状态：于 `2026-04-26` 完成。

| 项 | 目标 | 范围 | 验收 | 状态 |
|---|---|---|---|---|
| 对齐历史过滤 | 当前的图、编译、数据集、参数、状态和时间过滤器不会错误地隐藏新保存的记录。 | 运行历史、回测历史、策略中心计数器、列表投影。 | 针对性测试覆盖刷新记录路径，E2E 冒烟保持稳定。 | 完成 |
| 重新检查保存和刷新顺序 | 历史刷新在记录实际符合列表条件后发生。 | 保存/提升操作、丢弃路径、临时与已保存状态、前端刷新触发器。 | 慢速本地运行不产生虚假的空历史卡片。 | 完成 |
| 保持详情重载基于持久化事实 | 列表、事件摘要和详情页面在 ID、事件计数和工件上一致。 | 运行时持久化、回测工件、详情 DTO、前端详情映射。 | 重载的运行/回测详情页面匹配相应的列表记录。 | 完成 |
| 审计能力驱动的 UI 暴露 | 可见操作从后端能力真实数据启用、禁用、隐藏或解释。 | 模块侧边栏、工具栏操作、工作区卡片、回退状态、支持矩阵。 | 能力治理检查和相关 UI 测试通过。 | 完成 |
| 收紧编译链措辞 | `strategy_ir` 保持仅预检，运行时编译保持可运行的真实数据。 | 编译摘要、诊断标签、操作失败、README 和活跃文档。 | 没有 UI 或文档文案声明第二个可运行的真实数据源。 | 完成 |
| 统一运行时和回测解释 | 实时事件历史、持久化详情、诊断和比较视图渲染相同的运行时事实。 | 诊断投影、事件卡片、运行详情、回测详情、比较页面。 | 相同的风险、执行、成交和数据质量事实在不同界面上一致显示。 | 完成 |
| 加固保留的 QuantScript 界面 | 支持的示例编译；不支持的结构以稳定诊断提前失败。 | 正式语法样本、fixture、lowering 诊断、保留界面文档。 | 测试和文档保持在保留的 V1 主干内。 | 完成 |
| 打磨现有前端布局 | 修复重叠、溢出、不可读控件和密集的解释文案，不添加新屏幕。 | 策略中心、工作区、回测页面、共享 CSS、紧凑卡片和表格。 | 关键桌面和窄视口保持可用，无阻塞文本重叠。 | 门禁覆盖界面完成；手动视觉审查保持 P2/仅审查。 |

来自 `2026-04-26` 的 P1 验收证据：

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd frontend; cmd /c npm run test`
- `cd frontend; cmd /c npx playwright test tests/e2e/run-simulation.spec.js tests/e2e/run-backtest.spec.js tests/e2e/editor-capabilities-smoke.spec.js`
- `cmd /c tools\run-closeout-gates.bat`

最新 P1 实现说明来自 `2026-04-26`：

- 历史过滤现在在保存后仍有过时的图、编译、数据集、参数、状态或时间过滤器时，保持当前选择的持久化运行或回测可见。
- 保存流程作为 `保存 -> 刷新历史 -> 加载持久化详情` 覆盖，用于模拟运行和回测。
- 重载的运行和回测详情状态从持久化详情响应重建，而非从临时内存事件状态。
- 能力驱动的 UI 暴露通过能力治理快照、支持矩阵测试、模块侧边栏测试、工具栏回退测试和禁止声明文本搜索重新检查。
- 编译链措辞根据活跃合约重新检查：`strategy_ir` 保持语义预检，运行时编译保持可运行的真实数据。
- 运行时和回测解释界面通过诊断、事件历史、回测工件、详情和比较测试重新检查。
- QuantScript 保留界面行为通过支持的正式示例和不支持结构的诊断测试重新检查。

P2 清理期间解决的 P1 残余测试说明：

- `StrategyWorkspacePage.codeMode.test.jsx` 不再发出之前的 React `act(...)` 警告；测试拥有的 store fixture 现在拥有协作审计刷新。
- 可选的视觉审查不再被路由漂移阻塞。审查脚本针对当前策略中心、策略工作区、回测详情和回测比较路由，其 fixture 覆盖当前的支持 API 读取。
- `2026-04-28` 视觉审查通过在截图捕获前冻结运动，使过时的动画叠加不再看起来像产品状态。
- 回测详情使用显式的事件流详情模式，保持窄视口图表和源在自然流中而非重叠。
- 事件流 JSX kicker 标签不再在渲染的审查截图中暴露 Unicode 转义字面量。

#### P1 第 8 项仅视觉/布局问题

在 P1 批次中未对此项采取实现工作。
要带入 P2/仅审查工作的已知问题：

- 可选的响应式截图测试现在可以捕获当前的截图集，但截图仍然是手动审查证据而非像素差异质量门禁。
- 策略中心首屏快照在空状态路径中显示密集的状态和名册控件；这仍然需要手动窄视口审查。
- 策略中心的重复 `可运行策略` 标签在 P2 无决策清理期间移除；指标保留该标签，操作卡片现在为 `运行就绪`。
- 完整的手动重叠和溢出审查仍然在规范冒烟门禁之外，应保持仅审查，直到显式启用。

#### P2：仓库和发布卫生

状态：工件卫生和文档对齐于 `2026-04-26` 完成；提交切片仍然是基线交接关注点，因为工作树仍然包含广泛的有意产品变更集。

| 项 | 目标 | 范围 | 验收 | 状态 |
|---|---|---|---|---|
| 压缩当前工作树 | 使每个剩余的产品变更可审查和可解释。 | 当前广泛的修改文件集。 | 状态不再混合无关的产品更改、生成文件和本地工件。 | 工件分离完成；产品差异仍然广泛，必须作为一个有意的基线批次审查或提交。 |
| 清理本地工件 | 保持构建、运行时、测试和审计残留物在产品真实数据之外。 | `target/`、`target-test-*`、`frontend/dist/`、`frontend/test-results/`、`storage/*`、本地日志。 | `git status --short` 仅显示有意的产品或文档更改。 | 可见的本地工件已完成清除：`storage/*`、`frontend/dist/`、`frontend/test-results/` 和 `codex-vite-dev.log` 已移除。 |
| 重新检查忽略边界 | 确保生成/运行时工件保持被忽略。 | `.gitignore`、README 工件说明、清理脚本文档。 | 忽略的输出匹配本文档中的工件边界。 | 完成；忽略规则和清理脚本文档现在覆盖实验、图版本、审计 JSON 和本地 Vite 日志。 |
| 记录依赖审计状态 | 保持私有基线风险接受与公开发布就绪分开。 | `npm audit`、私有风险登记册、README 发布措辞。 | 私有基线措辞诚实，公开发布就绪措辞不存在。 | 完成；依赖审计仍然是仅私有接受的风险和公开发布阻塞项。 |
| 保持 markdown 索引精简 | 保留一个当前收尾条目并归档完成的台账。 | 文档根目录、概览索引、规划 README、归档规划 README。 | 读者可以从本文档找到活跃队列，无需跟踪完成的台账。 | 完成；活跃规划通过本文档路由，完成的台账保持归档。 |
| 维护法律诚实性 | 保持占位符许可姿态明确。 | `LICENSE`、README、发布文档。 | 公开发布仍然受阻，直到存在所有者批准的许可证和可见性决策。 | 完成；占位符许可姿态保持明确。 |

来自 `2026-04-26` 并在 `2026-04-28` 更新的 P2 执行证据：

- `.gitignore` 现在排除 `storage/experiments/`、`storage/graphs/versions/` 和 `codex-vite-dev.log`。
- `tools\cleanup-artifacts.ps1` 支持显式的 `-IncludeRuntimeArtifacts` 清理运行时工件、本地图快照、图版本和审计 JSON。
- P2 无决策残余清理移除了工作区测试 `act(...)` 警告，修复了可选的视觉审查路由/API fixture 漂移，并去重了策略中心 `可运行策略` 状态措辞。
- `npm audit fix` 通过将 `frontend` 锁文件解析为 `postcss@8.5.12` 移除了 `postcss <8.5.10` 中等发现。剩余的审计结果是仅 Vite/esbuild 链。
- 可选的响应式视觉审查通过减少运动捕获稳定，回测详情现在使用详情模式事件流布局以避免移动端重叠。
- README 仓库卫生说明现在匹配活跃的工件边界。
- 本地生成输出在 `storage/*`、`frontend/dist/`、`frontend/test-results/` 和 `codex-vite-dev.log` 下已从工作目录移除。
- 完整收尾验证在此清理后重新运行：`cmd /c tools\run-closeout-gates.bat` 于 `2026-04-26` 通过。

P2 残余交接：

- 工作树在前端、后端、测试和 markdown 中仍然有意广泛。由于这些是产品/源代码更改而非生成工件，它们在 P2 清理期间未被还原或删除。
- 在基线提交之前，将广泛的差异作为一个有意的基线批次审查或拆分为更小的提交。不要将重新生成的本地工件混入该审查。

#### 执行顺序

1. 修复 `alert` 和任何受影响的文本渲染问题。
2. 修复运行和回测历史 E2E 回归。
3. 重新运行前端 E2E，然后完整收尾包装器。
4. 压缩和清理工作树。
5. 仅更新反映收尾状态所需的活跃文档和索引。
6. 将新功能想法、插件扩展、公开发布和依赖主要迁移保留在此收尾队列之外，除非所有者显式打开单独的批次。

### P0：阻塞门禁和事实失败

在基线置信度可声称之前，必须清除 P0 工作。

1. 运行当前基线门禁。
   命令：`cmd /c tools\run-closeout-gates.bat`。
   验收：包装器通过，无需隐藏手动设置。

2. 修复 Rust 格式化漂移。
   命令：`cargo fmt --all -- --check`。
   验收：命令通过。仅使用 `cargo fmt --all` 应用格式化，然后重新检查。

3. 修复严格的 Rust lint 失败。
   命令：`cargo clippy --workspace --all-targets -- -D warnings`。
   验收：命令通过，无需压制有意义的警告。仅当异常是局部的、由代码形态文档化并且比语义重构更安全时，才允许 lint。

4. 立即修复任何完整门禁失败。
   范围：UTF-8 门禁、面向用户文本门禁、能力治理快照、Rust 测试、前端测试、前端构建或 E2E 冒烟。
   验收：失败的层单独通过，然后包装器通过。

5. 移除虚假的产品真实数据。
   范围：README、活跃文档、前端文案、fixture、支持矩阵和能力驱动的 UI 暴露。
   验收：没有文本或可见操作声称实盘交易、研究级回测、真正套利平台支持、第三方插件市场支持或任意主机代码 QuantScript 支持。

6. 防止 E2E 合约漂移。
   命令：`cd frontend; cmd /c npm run test:e2e`。
   验收：冒烟路径在隔离的 API 模拟合约下通过，未模拟的 API 请求仍然是失败。

### P1：核心合约收尾

P1 工作加固现有产品界面而不增加广度。

1. 对齐能力驱动的前端暴露。
   范围：工具栏操作、模块侧边栏、工作区卡片、回退状态和缓存能力行为。
   验收：可见操作仅从当前能力合约和保留的 beta 边界启用、禁用、隐藏或解释。

2. 收紧编译链解释。
   范围：编译路由、编译摘要、诊断标签、存储工件、前端操作失败和文档。
   验收：`strategy_ir` 保持语义预检，`quantscript.formal_source` 在存在时拥有运行时 lowering，运行时编译保持可运行的真实数据源。

3. 统一运行时和回测解释。
   范围：运行时诊断、事件投影、运行详情、回测详情、比较视图和事件流卡片。
   验收：相同的运行时事实在实时事件历史和持久化详情视图之间一致渲染。

4. 清扫持久化和回放一致性。
   范围：图版本、运行历史、回测历史、回放检查点、实验记录和存储支持的详情加载。
   验收：重载路径不依赖临时内存状态或特殊重建。

5. 加固保留的 QuantScript 界面。
   范围：正式语法、lowering 诊断、编写样本、fixture 和文档。
   验收：支持的样本一致编译，不支持的结构以稳定诊断提前失败，文档不暗示更广泛的研究语言。

6. 仅为现有承诺添加或刷新针对性回归覆盖。
   验收：测试保护固定合约，不成为新功能通道。

### P2：发布卫生和基线清理

P2 工作保持仓库整洁和发布故事诚实。

1. 在基线快照之前清理被忽略的本地工件。
   范围：`target/`、`target-test-*`、`frontend/dist/`、`frontend/test-results/`、本地 Playwright 输出和 `storage/test-*`。
   验收：`git status --short` 保持干净，被忽略的残留物不被误认为产品真实数据。

2. 保持活跃文档精简。
   范围：README、本就绪文件、支持矩阵、测试期望和规划 README。
   验收：读者可以从活跃文档找到当前 beta 边界、门禁集和仅所有者发布阻塞项，无需跟踪已归档的路线图。

3. 在不强制范围扩展的情况下审计依赖风险。
   范围：`npm audit`、前端依赖更新和锁文件更改。
   验收：中等或更高发现仅通过低风险升级记录和修复，除非所有者显式接受破坏性依赖迁移。
   当前结果：`postcss <8.5.10` 中等发现已通过锁文件补丁到 `postcss@8.5.12` 修复。剩余的 Vite/esbuild 审计发现没有非破坏性的自动修复。所有者仅接受此剩余风险用于私有基线使用，因此它仍然是公开发布阻塞项，直到专用依赖迁移关闭它。

4. 维护发布法律诚实性。
   范围：`LICENSE`、README 发布措辞和公开可见性说明。
   验收：私有基线状态保持明确，公开发布保持受阻，直到单独的所有者决策替换占位符许可姿态。

5. 保持手动审查测试与冒烟门禁分开。
   范围：性能和视觉审查 Playwright 规范。
   验收：仅审查测试通过其环境开关保持可选，不与规范的 E2E 冒烟合约混淆。

6. 在共享清理后重新运行完整收尾包装器。
   命令：`cmd /c tools\run-closeout-gates.bat`。
   验收：基线门禁在清理后通过。

### 批次报告规则

每个收尾批次仅应报告以下事实：

- 移除了哪些现有风险或漂移
- 哪些针对性检查通过
- 完整收尾包装器是否通过
- 哪些仅所有者决策（如果有）仍然阻塞公开发布

不要将批次报告为新能力交付。

## 显式门禁命令

包装器应与这些命令保持一致：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; cmd /c npm run test
cd frontend; cmd /c npm run build
cd frontend; cmd /c npm run test:e2e
```

门禁措辞不得在本地文档、辅助脚本和 CI 之间漂移。

当前约束：

- 前端 `npm` 门禁必须使用 `cmd /c npm run ...`
- E2E 必须保持无需手动预启动后端即可运行
- 未模拟的 E2E API 请求是失败，而非可接受的代理回退

## 工件边界

基线提交应捕获产品真实数据，而非本地运行时残留物。

将这些排除在基线快照之外：

- `target/` 下的 Rust 构建输出
- 前端构建输出和本地 Playwright 输出
- `storage/runs/` 和 `storage/backtests/` 下的本地运行时工件
- `storage/audit/*.json` 下的本地审计工件
- `storage/test-*` 下的测试运行时工件
- `storage/graphs/*.json` 和 `storage/graphs/*.qs` 下的本地图快照
- 本地环境覆盖和临时辅助文件

## 活跃配套文档

- [私有基线风险登记册](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-private-baseline-risk-register.md)
- [当前状态与发布状态](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [支持矩阵](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [测试层期望](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)
- [活跃 QRPC RFC 索引](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
- [已归档功能收尾台账](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-functional-closeout-task-table.md)
- [已归档 P2 收尾清单](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-non-blocking-closeout-list.md)
