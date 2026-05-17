# 当前状态与发布状态

> 最后更新：2026-05-17 | 当前版本：v2.3.0 ✅ (错误国际化+TLS+JWT刷新, 31 P1/17 P2修复)

## 版本路线

| 版本 | 状态 | 焦点 |
|------|:--:|------|
| v0.1.0 | ✅ | 私有基线 |
| v0.2.0 | ✅ | 测试框架 + CI |
| v0.3.0 | ✅ | 22 项合规 + 10 信号 |
| v0.4.0 | ✅ | UI / 教程 / 凭证安全 |
| v0.4.1 | ✅ | 安全审计 12 项修复 |
| v0.4.2 | ✅ | 收口排雷 10 项 — 无新功能 |
| v0.4.3 | ✅ | 用户体验与安全收口 5 项 |
| v0.5.0 | ✅ | Adobe 前端重构 + 38 项全量审计 |
| v0.5.1 | ✅ | 全量审计收口排雷 15 项 |
| v0.5.2 | ✅ | 排雷收口 16 项 — 无新功能 |
| v1.0.0 | ✅ | 插件化架构 + 重型策略 + 超级规范化 — 19/19 完成 |
| v1.0.3 | ✅ | 边界防御 15 项 |
| v1.0.4 | ✅ | 诱错测试矩阵 38 场景 |
| v1.0.5 | ✅ | 前端样式深度修复 + 六轮审计 |
| v1.0.6 | ✅ | 用户困惑点全量优化 70/79 |
| v1.0.7 | ✅ | 体验收口 + en-US 补齐 9/9 |
| v1.1.0 | ✅ | 研究级回测 + 多标的策略 — 16/16 完成 |
| v1.1.x | ✅ | 15轮PATCH: 5轮诱错(570项)S0/P1消化 |
| v1.2.0 | ✅ | 架构优化: KlineProvider/RiskMonitor/PBKDF2/编译链/main拆分 |
| v1.2.x | ✅ | 4轮PATCH: 确定性修复+状态机+死代码清理 |
| v1.3.0 | ✅ | 技术债清零: AbortController/t()包裹/TTL驱逐 |
| v1.3.x | ✅ | 7轮PATCH: 4轮诱错(80+)S0清零+快速优化 |
| v1.4.0 | ✅ | 11轮诱错全量消化, S0清零确认, closeout |
| v2.0.0 | ✅ | MAJOR: OKX实盘+多用户认证+插件市场+前端补全+整合包发布 |
| v2.1.x | ✅ | 97项P1/P2/P3全量清零: 断路器+备份+checkpoint+NaN防御+死代码清理 |
| v2.2.x | ✅ | MINOR: 架构重构(Coordinator拆分/QuantPilotError) + i18n完整化(386键/tracing/安全加固) |

### v1.0.0 三大目标

1. **轻核心 + 重挂载**: 双层插件模型（原子层 + 套件层），核心只保留编译链、沙盒调度、插件协议
2. **重型策略**: 多时间框架、多标的组合、DAG 策略路由、热接管
3. **超级规范化**: 五条流水线（设计→开发→检查→审计→优化）+ 元流水线自审计

## 当前产品真实情况

QuantPilot v0.5.1 拥有一条可运行的端到端 beta 链：

- 桌面应用: Tauri v2 自绘标题栏 Windows 桌面应用
- 前端: Adobe 暗色面板设计系统, SVG 图标, 图编辑器, 策略工作区, 回测详情/对比, 研究控制台
- 后端: Axum API (图保存/加载, 编译, paper 运行, 回测, 能力发现)
- 运行时链: data → intent → agent → risk → execution → fill
- QuantScript: 语法解析 → HIR → lowering → Core IR 完整编译管道

当前全部 18 种指标的 K 线驱动 Intent 支持是真实的 (详见 README 指标表)。

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
- 公开 SaaS 服务

前端现在诚实地处理这些差距：

- 不支持的标准模块侧边栏中不显示
- 旧版图仍可加载，但不支持的模块作为显式验证错误浮现
- `/api/capabilities` 暴露支持的模块、运行时模式、指标、交易所和交易对的当前真实数据源

当前声明但不支持的真实情况必须按字面理解：

- `Custom` 仅通过受限的 Strategy IR 表达式路径支持，该路径降低到 Core IR
- `Custom` 不允许任意主机代码、直接风险变异或直接执行绕过
- 插件清单和注册表支持存在于 `qrpc_core` 中，但当前仅为本地元数据；尚不是远程安装或第三方分发界面

## 活跃合约边界

编译链具有固定优先级，应在 UI、文档和测试中一致描述：

- `strategy_ir` 仅是语义预检工件
- `strategy_ir` 可提前使编译失败，但它不取代运行时编译的真实数据源
- `quantscript.formal_source` 在存在时负责运行时 lowering
- 如果正式 QuantScript lowering 不可用，系统回退到图生成的 `runtime_config`
- 当这些工件不一致时，运行时行为遵循运行时编译的真实数据源，而非 `strategy_ir` 预检工件

当前合约详情现在存储在专用文档中，而非在此重复：

- 编译解释：[编译链合约](../03-implementation/governance/implementation-compile-chain-contract.md)
- 运行时和回测解释：[运行时/回测解释合约](../03-implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- 持久化和回放：[持久化/回放合约](../03-implementation/runtime/implementation-persistence-replay-contract.md)
- QuantScript 保留界面：[QuantScript 保留界面合约](../03-implementation/governance/implementation-quantscript-retained-surface-contract.md)

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

- [QuantScript 主干基线](../04-guides/guide-quantscript-trunk-baseline.md)
- [正式 QuantScript 语法指南](../04-guides/guide-formal-quantscript-syntax.md)

## 当前收尾/发布状态

v0.5.1 已完成 15 项 P0/P1/P2 优化。v0.5.2 聚焦于排雷收口 — 修复测试套件回归、激活存储配额、消除架构违规残存。

使用下面的专用文档作为活跃发布界面：

- [v0.5.2 规划方案](../06-milestones/v0.5.2/01-规划方案.md)
- [v0.5.2 综合优化清单](../06-milestones/v0.5.2/02-综合优化清单.md)
- [支持矩阵](../03-implementation/governance/implementation-support-matrix.md)
- [编译链合约](../03-implementation/governance/implementation-compile-chain-contract.md)

当前仓库级状态 (v2.1.3):

| 检查项 | 状态 | 备注 |
|--------|:--:|------|
| `cargo check --workspace` | ✅ | 编译通过 |
| `cargo clippy --workspace` | ✅ | 通过 |
| 前端 `npm run build` | ✅ | 通过 |
| P1 消化 | ✅ | 49/49 全部完成 |
| P2 消化 | ✅ | 28/28 全部完成 |
| P3 消化 | ✅ | 20/20 全部完成 |
| 断路器 (CircuitBreaker) | ✅ | 8/8 测试通过 |
| 自动备份 | ✅ | 每日备份到 storage/backups/ |
| Checkpoint/Restore | ✅ | Sandbox trait + handoff 实现 |
| NaN 防御深度 | ✅ | 51 处 is_finite() 守卫 |
| deny_unknown_fields | ✅ | 28 structs |
| 存储配额强制执行 | ✅ | 已接入 |
| `map_frontend_runtime_config` | ✅ | 函数已删除 (450 行死代码清理) |
| state.rs 重复类型 | ✅ | 900+ 行孤立文件已删除 |

## V1 冻结方向

- 将当前的正式 QuantScript 主干、已落地的共享核心切片、`risk.profile(...)` / `execution.profile(...)`、价差切片和可执行回测/报告切片视为保留的 `V1` 界面
- 将更广泛的价差合约、`MACD` 共享核心扩展、通用风险/执行 DSL 增长、每笔交易比较、成交时间线比较视为延期工作
- 在宣布 `V1` 关闭之前，优先排雷收口、消除重复真实数据源、保持文档/提示/UI 措辞与保留界面一致，而非扩大功能范围

## 接受规则

除非以下所有条件都成立，否则不应在 UI、文档或提示中暴露任何功能：

- 后端编译路径存在
- 运行时语义存在
- 验证可以诚实地拒绝不支持的使用
- 事件输出可以解释行为
- 测试覆盖该路径 (且测试实际可运行)
- 前端和文档文本保存为 UTF-8 并验证在渲染产品中正确显示
