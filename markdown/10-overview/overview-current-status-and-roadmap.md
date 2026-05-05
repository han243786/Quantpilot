# 当前状态与发布状态

## 当前产品真实情况

QuantPilot 已经拥有一条可运行的端到端 beta 链：

- 前端图编辑器、验证、编译、运行时事件显示和回测详情页面
- 后端 Axum API 用于图保存/加载、编译、paper 运行、回测和能力发现
- 运行时链用于 data -> intent -> agent -> risk -> execution -> fill
- QuantScript AST、辅助函数分析、公式 lowering 和运行时支持的指标 Intent

当前 K 线驱动的 Intent 支持是真实的：

- 双移动平均
- 移动平均偏差
- RSI
- MACD
- Momentum
- ZScore

当前基本回测支持也是真实的：

- 历史回放
- 持久化回测记录
- 权益曲线
- 基本摘要指标

## 不得夸大之处

以下项目仍然不是真正的平台能力，不得描述为已支持：

- 当前现货 beta 中的真正套利代理支持
- 具有完整市场微观结构语义的研究级回测
- 任何 paper 策略都可以直接在 QuantScript 中表达

前端现在诚实地处理这些差距：

- 不支持的标准模块侧边栏中不显示
- 旧版图仍可加载，但不支持的模块作为显式验证错误浮现
- 后端暴露 `/api/capabilities` 作为支持的模块、运行时模式、指标、交易所和交易对的当前真实数据源
- `/api/capabilities` 现在同时暴露兼容性字段和结构化支持条目：
  - `strategy_ir.indicator_support`
  - `runtime.mode_support`
  - `runtime.execution_module_support`
  - `market_data.exchange_support`
  - `market_data.symbol_support`
  - `frontend.declared_module_keys`
  - `frontend.module_support`

当前声明但不支持的真实情况必须按字面理解：

- `Custom` 仅通过受限的 Strategy IR 表达式路径支持，该路径降低到 Core IR
- `Custom` 不允许任意主机代码、直接风险变异或直接执行绕过
- 插件清单和注册表支持现在存在于 `qrpc_core` 中，但当前的插件市场切片仍然是仅本地元数据；它还不是远程安装或第三方分发界面
- 旧消费者仍可读取旧版摘要字段，但新消费者应优先使用结构化支持条目

## 活跃合约边界

编译链具有固定优先级，应在 UI、文档和测试中一致描述：

- `strategy_ir` 仅是语义预检工件
- `strategy_ir` 可提前使编译失败，但它不取代运行时编译的真实数据源
- `quantscript.formal_source` 在存在时负责运行时 lowering
- 如果正式 QuantScript lowering 不可用，系统回退到图生成的 `runtime_config`
- 当这些工件不一致时，运行时行为遵循运行时编译的真实数据源，而非 `strategy_ir` 预检工件

当前合约详情现在存储在专用文档中，而非在此重复：

- 编译解释：[编译链合约](../implementation/governance/implementation-compile-chain-contract.md)
- 运行时和回测解释：[运行时/回测解释合约](../implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- 持久化和回放：[持久化/回放合约](../implementation/runtime/implementation-persistence-replay-contract.md)
- QuantScript 保留界面：[QuantScript 保留界面合约](../implementation/governance/implementation-quantscript-retained-surface-contract.md)

## 当前架构边界

今天，系统应被理解为：

- 一个以 `BTCUSDT`、`ETHUSDT` 和 `SOLUSDT` 为中心的 paper/runtime beta
- 支持的交易所限于 `binance` 和 `okx`
- 支持的运行时模式限于 `paper`
- 支持的执行模块限于 `builtin.execution.paper`
- 支持的前端模块限于后端可一对一编译的模块

QuantScript 比配置外壳更强，但仍然不是一个完整的研究语言。解析器仍然接受一些更广泛的语法，但未来的开发必须收缩到狭窄的主干：

- 数据获取/对齐
- 白名单指标
- 受限的 universe/filter/score/top-k 管道
- 最小控制流
- 标准化的 `emit Intent(...)`

风险/执行细节、通用状态和通用语言功能不是预期的增长路径。

使用这些文档作为活跃参考：

- [QuantScript 主干基线](../guides/quantscript/guide-quantscript-trunk-baseline.md)
- [正式 QuantScript 语法指南](../guides/quantscript/guide-formal-quantscript-syntax.md)
- [V1 冻结/取消范围清单](../guides/quantscript/guide-v1-freeze-descope-checklist.md)

## 当前收尾/发布状态

当前的优化优先级是发布状态确认，而非能力扩展。
使用下面的专用文档作为活跃发布界面：

- [首次发布就绪](../implementation/planning/implementation-first-release-readiness.md)
- [支持矩阵](../implementation/governance/implementation-support-matrix.md)
- [测试层期望](../implementation/runtime/implementation-test-layer-expectations.md)
- [已归档功能收尾台账](../archive/planning-retired/implementation-functional-closeout-task-table.md)

当前仓库级状态：

- `cargo test --workspace` 通过
- `cargo clippy --workspace --all-targets -- -D warnings` 通过
- 前端单元测试通过
- 规范的 Windows 前端 gate 形式是 `cmd /c npm run ...`
- 截至最新的 `2026-04-26` P0/P1 收尾，`cmd /c npm run test:e2e` 从 `frontend` 通过，无需手动预启动后端，因为套件保持在隔离的 API 模拟合约上
- UTF-8 和面向用户文本门禁通过
- 能力治理快照是最新的
- P1 历史过滤和保存流程现在使新保存的运行/回测记录在过时过滤器中保持可见，然后重新加载持久化详情状态
- 已接受的收尾包装器通过；剩余清理是 P2 仓库卫生和公开发布阻塞项处理
- 可选的视觉审查路由/API fixture 漂移于 `2026-04-26` 修复并于 `2026-04-28` 重新检查；规范现在在设置 `VISUAL_REVIEW=1` 时使用减少运动捕获拍摄策略中心、策略工作区、回测详情和回测比较截图
- `postcss <8.5.10` 中等审计发现已通过 `postcss@8.5.12` 修复；剩余的 npm 审计风险是 Vite/esbuild 链，仍然仅接受用于私有基线使用，并且仍然阻塞公开发布声明

## V1 冻结方向

- 将当前的正式 QuantScript 主干、已落地的共享核心切片、向外移动的 `risk.profile(...)` / `execution.profile(...)`、第一个狭窄的价差切片和第一个可执行的回测/报告切片视为保留的 `V1` 界面
- 将更广泛的价差合约、`MACD` 共享核心扩展、通用风险/执行 DSL 增长、每笔交易比较、成交时间线比较和更广泛的研究报告扩展视为延期工作
- 在宣布 `V1` 关闭之前，优先删除重复的真实数据源、压缩已完成的队列项、保持文档/提示/UI 措辞与保留界面一致，而非扩大功能范围

## 接受规则

除非以下所有条件都成立，否则不应在 UI、文档或提示中暴露任何功能：

- 后端编译路径存在
- 运行时语义存在
- 验证可以诚实地拒绝不支持的使用
- 事件输出可以解释行为
- 测试覆盖该路径
- 前端和文档文本保存为 UTF-8 并验证在渲染产品中正确显示
