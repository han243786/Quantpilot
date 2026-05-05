# 测试层期望

## 目的

本文档定义了收尾期间如何解释 QuantPilot 测试结果。它不添加新的测试框架或扩大产品范围。

目标是让绿色测试保持诚实：

- 有针对性的测试证明其所命名的合约
- 完整门禁证明当前仓库基线仍然完整
- E2E 证明浏览器级入口行为与隔离的 API 模拟合约一致
- 没有测试层应被描述为 beta 产品并未实际暴露的能力的证明

## 层合约

| 层 | 规范命令 | 证明 | 不证明 |
|---|---|---|---|
| UTF-8 门禁 | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | 活跃前端和 markdown 文件不包含门禁覆盖的编码回归 | 措辞的语义正确性 |
| 面向用户文本门禁 | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1` | 已知乱码和禁止措辞模式在活跃产品路径中不存在 | 每个句子都是产品准确的 |
| 能力治理门禁 | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1` | 生成的能力治理快照匹配前端支持矩阵 | 每个能力的运行时行为 |
| Rust 工作区测试 | `cargo test --workspace` | Rust crates、API 集成测试、运行时合约、编译路径和协议 fixture 保持内部一致 | 前端渲染或浏览器交互 |
| 有针对性的 Rust 测试 | 聚焦的 `cargo test ...` 命令 | 命名的 Rust 合约或回归仍按预期表现 | 无关的 crate 级别健康 |
| 前端单元测试 | `cd frontend; cmd /c npm run test` | 组件、存储、能力投影、编译状态、运行时投影和页面级 UI 合约保持稳定 | 完整浏览器导航或生产资产输出 |
| 有针对性的前端测试 | 聚焦的 `cmd /c npm run test -- src/...` 命令 | 命名的组件、存储或工具合约仍按预期表现 | 完整前端基线健康 |
| 前端构建 | `cd frontend; cmd /c npm run build` | 生产资产可编译，路由级别块可发出 | 运行时正确性或 E2E 行为 |
| 前端 E2E | `cd frontend; cmd /c npm run test:e2e` | 浏览器入口流程、能力回退行为、编译/运行/回测冒烟路径和阻塞路径浮出在隔离的模拟合约下工作 | 实时后端集成、交易所连接性、研究级行为或不支持的产品能力 |
| 收尾包装器 | `cmd /c tools\run-closeout-gates.bat` | 当前基线门禁集在文档化的 Windows 形式上通过 | 风险本地更改不需要更窄的针对性测试 |

## E2E 解释规则

当前 E2E 套件有意保持隔离。它必须保持无需手动预启动后端即可运行。

规则：

- E2E 可使用固定的 API fixture 和路由级别模拟进行浏览器合约覆盖。
- 未模拟的 API 请求是失败，而非可接受的代理回退。
- E2E 应保持小巧，专注于入口行为、回退行为和关键的阻塞路径。
- E2E 通过状态不得被描述为实时后端可用性、外部交易所连接性或广泛策略支持的证明。

## 针对性测试解释规则

针对性测试在收尾期间很有用，因为它们保持反馈快速。它们应按所保护的合约命名和选择。

在以下情况下使用针对性测试：

- 更改触及一个组件、存储、工具或后端合约
- 先前的回归需要紧密的护栏
- 完整门禁在迭代时会很慢

当更改触及共享行为、跨模块合约、面向用户工作流或发布文档时，不要将针对性测试成功视为完整收尾门禁的替代。

## 常见的针对性回归命令

从仓库根目录运行，除非命令显式更改到 `frontend`。

### 编译链措辞和失败指导

在编译摘要、编译操作失败措辞、Strategy IR 预检或运行时编译源措辞更改时使用：

```powershell
cd frontend; cmd /c npm run test -- src/components/PropertyPanel.compileSummary.test.jsx src/utils/actionFailure.test.js src/store/graphStore.strategyIrCompile.test.js
```

### 能力暴露和支持矩阵

在 `/api/capabilities` 解释、前端能力门禁、模块暴露或支持矩阵措辞更改时使用：

```powershell
cd frontend; cmd /c npm run test -- src/capabilities/supportMatrix.test.js src/capabilities/capabilityGovernance.test.js src/components/TopToolbar.capabilities.test.jsx src/components/ModuleSidebar.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx src/store/graphStore.capabilities.test.js
```

### 运行时、回放和持久化详情解释

在运行时解释行、回测详情解释、事件回放或持久化选择形态更改时使用：

```powershell
cd frontend; cmd /c npm run test -- src/utils/runtimeExplanation.test.js src/components/EventStreamPanel.historyExplanation.test.jsx src/components/EventStreamPanel.backtestHistory.test.jsx src/components/EventStreamPanel.backtestArtifacts.test.jsx src/components/EventReplaySection.test.jsx src/pages/BacktestDetailPage.test.jsx src/pages/BacktestComparePage.test.jsx src/pages/StrategyBacktestsPage.test.jsx src/store/graphStoreRuntimeHistoryFlow.test.js src/store/graphStoreRuntimeSelectionState.test.js src/store/graphStorePersistenceConsistency.test.js
```

当 API 详情、回放、持久化运行/回测记录或工件支持的重载行为更改时使用以下后端检查：

```powershell
cargo test --test api_run -- --nocapture
cargo test --test api_backtest -- --nocapture
```

### 正式 QuantScript 保留面

在保留的编写样本、边界 fixture、解析器/降级措辞或正式 QuantScript 编译行为更改时使用：

```powershell
cargo test --test quantscript_real_strategy_authoring -- --nocapture
cargo test -p quantscript --lib
cd frontend; cmd /c npm run test -- src/graph/quantscript.test.js src/components/StrategyCodePanel.authoringView.test.jsx
```

### E2E 浏览器冒烟路径

在能力回退行为或编译/运行/回测浏览器入口路径更改时使用：

```powershell
cd frontend; cmd /c npx playwright test tests/e2e/editor-capabilities-smoke.spec.js tests/e2e/run-simulation.spec.js tests/e2e/run-backtest.spec.js --project=msedge --workers=1
```

这些命令是聚焦的工具。在声称共享收尾切片完成之前，运行完整收尾包装器。

## 完整门禁解释规则

完整收尾包装器是基线置信度检查。在声称收尾切片完成之前，它应通过。

完整门禁应证明：

- 活跃文本门禁仍通过
- 能力治理是最新的
- Rust 工作区行为为绿色
- 前端单元测试为绿色
- 前端生产构建为绿色
- 前端 E2E 冒烟路径在隔离的 API 模拟合约下为绿色

完整门禁不替代代码审查。它也不证明延迟的能力支持。

## 文档规则

在报告验证时，说明运行了哪个层以及它证明了什么。避免模糊摘要，如"测试证明一切正常"。

推荐措辞：

- 针对性的前端回归通过受影响的编译摘要面
- 完整收尾门禁通过当前基线
- E2E 在隔离的 API 模拟合约下通过

避免暗示以下内容的措辞：

- E2E 覆盖了实时后端集成
- 模拟支持的冒烟路径证明实时市场连接性
- 解析器接受测试证明面向发布的 QuantScript 编写支持
- 仅兼容性解析器测试证明保留的可执行 QuantScript 面
