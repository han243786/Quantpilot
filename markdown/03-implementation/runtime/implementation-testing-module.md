# 测试模块实现

本文档聚焦于 QuantPilot 中的成交模拟和测试基础设施。

其角色不是创建独立的回测世界。其角色是用可靠的成交语义、回放支持和回归覆盖来加强统一的交易沙箱。

关于 CI、回放和 E2E 流程所使用的确定性测试模式边界，请参阅 [implementation-test-mode.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-mode.md)。关于如何解释针对性测试、完整门禁包装器和隔离的 E2E 合约，请参阅 [implementation-test-layer-expectations.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)。

## 当前角色

测试模块服务于两个目的：

- 为模拟和回测模式提供稳定的成交行为
- 为回归和调试提供回放和审计基础

## 现有基础

仓库已包含：

- `ExecutionPlan`
- `FillReport`
- `OpenOrder`
- `FillResult`
- `PortfolioState`
- `RuntimeEvent`
- 独立的 `fill_engine.rs`
- 成交行为和运行时集成的测试

这意味着任务不再是"从头发明成交协议"。任务现在是"将现有的成交逻辑转变为稳定的沙箱组件"。

## 近期目标

1. 保持成交语义与实时模拟一致
2. 支持快速历史回测
3. 添加回放和快照基础
4. 为未来的精确模拟留下扩展点

## 近期任务

### 任务 1：稳定成交引擎 I/O

目标：

- 保持一个清晰的合约：`ExecutionPlan + MarketState -> FillResult`
- 使账户更新边界更易于推理

验收：

- 固定输入在相同模式下产生稳定输出
- 重复提交不会使状态重复记账

### 任务 2：强化事件负载

不再进行大的枚举变动，而是在需要时扩展当前事件负载。

最低期望：

- `ExecutionPlanned` 包含订单状态、剩余数量和相关的限价信息
- `ExecutionFilled` 包含方向、数量、价格和执行状态
- 前端可区分已计划、等待中、部分成交和已完成状态

### 任务 3：回放和快照基础

目标：

- 支持从稳定的运行时输出进行重复执行和调试

最低输出：

- 事件序列导出
- 账户状态快照
- 从固定输入可重复回放

### 任务 4：支持快速回测沙箱

目标：

- 直接与沙箱路线图配合

最低支持：

- K 线或 L1 驱动的执行
- 简化的匹配模型
- 可重复的运行
- 稳定的结果输出

### 任务 5：编码和面向用户文本门禁

目标：

- 防止 UTF-8 回归和乱码重新进入前端和文档

最低检查：

- 拒绝前端源和 markdown 文档中的 UTF-8 BOM
- 拒绝替换字符
- 拒绝先前回归中观察到的已知乱码片段
- 保持检查可在本地 PowerShell 和 CI 中运行

推荐脚本：

- `tools/check-utf8.ps1`
- `tools/check-user-facing-text.ps1`
- `tools/check-gates-smoke.ps1`

推荐扫描范围：

- `frontend/src`
- `frontend/index.html`
- `src/main.rs`
- `markdown`

当前第 2 周状态：

- 两个门禁脚本已接入 `.github/workflows/ci.yml`
- 门禁现在在前端依赖安装之前失败，以便乱码回归更早浮出
- `tools/check-gates-smoke.ps1` 提供一个最小的本地回归样本，写入错误的输入并断言两个门禁都失败

推荐的本地命令：

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-user-facing-text.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-gates-smoke.ps1`
- `cmd /c tools\run-closeout-gates.bat`

Windows 门禁规范化规则：

- 前端门禁应在文档、辅助脚本和 CI 中使用 `cmd /c npm run ...`
- 不要依赖 `npm.ps1` 执行策略豁免作为正常仓库合约的一部分

### 任务 6：测试目录和命名合约

目标：

- 在第 2 周添加更多 API 和 E2E 覆盖之前消除歧义
- 将服务级别、浏览器、fixture 和仓库质量测试保持在稳定位置

仓库约定：

- Rust 单元测试在仅覆盖本地逻辑时保持靠近实现模块
- Rust 服务级别和集成风格 API 测试放在 `tests/` 下
- 共享 Rust fixture 放在 `tests/fixtures/` 下
- 前端 E2E 测试放在 `frontend/tests/e2e/` 下
- 前端 E2E fixture 放在 `frontend/tests/fixtures/` 下
- 仓库范围的质量门禁保持放在 `tools/` 下

推荐文件命名：

- Rust API 测试使用 `api_*.rs`，例如 `api_capabilities.rs`、`api_compile.rs`、`api_run.rs`、`api_backtest.rs`
- Rust 协议或 fixture 密集型测试在焦点比一个端点更窄时使用 `protocol_*.rs` 或 `replay_*.rs`
- Playwright 规范使用 `*.spec.ts`，例如 `editor-smoke.spec.ts`、`capabilities-gating.spec.ts`、`backtest-smoke.spec.ts`
- PowerShell 质量门禁使用 `check-*.ps1`

CI 拆分：

- 阻塞检查：能力合约测试、UTF-8 检查、面向用户文本检查、前端构建
- 近期阻塞后端覆盖：`/api/capabilities` 和编译路径冒烟的服务级别测试
- 夜间或后期检查：更重的回放套件、更大的回测 fixture 套件、完整浏览器矩阵

主要规则是：以所保护的合约命名测试，而非以它们恰好今天调用的内部辅助函数命名。

## 未来扩展

### P1：统计回测行为

- 可重现的随机种子支持
- 统计滑点模型
- 可解释的近似规则

### P2：高保真模拟

- L2/L3 支持
- 队列位置
- 延迟模型
- 更真实的市场冲击行为
