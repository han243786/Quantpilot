# 当前状态与发布状态

> 最后更新：2026-05-24 | 当前版本：v3.7.1 ✅ | 当前补丁规划：v3.7.2 closeout 真实性与安全回退收口

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
| v2.3.x | ✅ | MINOR: 错误国际化(41码)+TLS+JWT刷新 + 架构优化(ISP拆分/risk_checker重构/runtime_api域拆分) |

### v1.0.0 三大目标

1. **轻核心 + 重挂载**: 双层插件模型（原子层 + 套件层），核心只保留编译链、沙盒调度、插件协议
2. **重型策略**: 多时间框架、多标的组合、DAG 策略路由、热接管
3. **超级规范化**: 五条流水线（设计→开发→检查→审计→优化）+ 元流水线自审计

## 当前产品真实情况

QuantPilot v3.7.1 继承 v3.7.0 的端到端量化交易沙盒链，并在回归修复后补齐流程门禁基线：

- **桌面应用**: Tauri v2 自绘标题栏 Windows 桌面应用
- **前端**: Adobe 暗色面板设计系统, 图编辑器, 策略工作区, 回测详情/对比, 研究控制台, Toast 通知
- **后端**: Axum API (图保存/加载, 编译, paper 运行, 回测, 能力发现, 告警引擎)
- **执行端**: 独立 Axum 进程 (:3001), 策略部署/启动/停止/热调参, lightweight-charts K 线
- **运行时链**: data → intent → agent → risk → execution → fill
- **QuantScript**: 语法解析 → HIR → lowering → Core IR 完整编译管道
- **安全**: AES-256-GCM 凭证保险库 (PBKDF2 1M/600K 轮), bcrypt 12 轮用户认证, JWT + 刷新令牌轮换+重放检测, 进程间加密通道
- **告警**: 10 条默认规则, resolve_condition 自动恢复, 去重

当前全部 18 种指标的 K 线驱动 Intent 支持是真实的 (详见 README 指标表)。

当前回测支持也是真实的：历史回放, 持久化回测记录, 权益曲线, 夏普/索提诺/卡尔玛等 12 项指标。

执行端支持：OKX Paper 模拟行情 + OKX testnet 实盘模拟 (REST + WebSocket), Paper/Live 模式切换, 策略迁移验证。

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

v3.7.1 系统应被理解为：

- 支持的交易所: `binance`, `okx`
- 支持的交易对: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- 支持的运行时模式: `paper`, `live` (OKX testnet)
- 支持的执行模块: `builtin.execution.paper`, `live.okx`
- 全部已覆盖系统以 GP §10 功能覆盖矩阵为准
- 前端 Toast 通知系统, 术语全中文化, 空状态引导
- 执行端独立进程 (:3001), ParamsPanel 热调参, Paper/Live 切换

## 当前收尾/发布状态

v3.7.0 完成 v3.5.0→v3.7.0 全版本演进 (12 项新功能 + 32 项审计修复 + 12 项 UX 优化 + 14 项 P3 消化)。

v3.7.1 是其后的 PATCH 收口：已修复 S0 登录挂起、P1 凭证 DELETE 405 和 P2 测试进程文件锁问题，并把 pre-commit、CI、closeout/release 三层门禁重新对齐。2026-05-24 全量复查后，v3.7.2 被定义为质量补丁：保持 strict safe fallback，修复 E2E 预期，新增 workspace clippy warning budget，并把 closeout 升级为 23 项；本地 closeout 已 23/23 通过。

使用下面的专用文档作为活跃发布界面：

- [v3.7.0 规划方案](../06-milestones/v3.7.0/01-规划方案.md)
- [v3.7.0 综合优化清单](../06-milestones/v3.7.0/02-综合优化清单.md)
- [v3.7.1 规划方案](../06-milestones/v3.7.1/01-规划方案.md)
- [v3.7.1 综合优化清单](../06-milestones/v3.7.1/02-综合优化清单.md)
- [v3.7.1 closeout 基线](../06-milestones/v3.7.1/03-closeout.md)
- [支持矩阵](../03-implementation/governance/implementation-support-matrix.md)
- [编译链合约](../03-implementation/governance/implementation-compile-chain-contract.md)
- [功能演进契约](../03-implementation/governance/implementation-feature-evolution-contract.md)

当前仓库级状态 (v3.7.1 流程基线):

| 检查项 | 状态 | 备注 |
|--------|:--:|------|
| `cargo fmt --check` | ✅ | 全仓 rustfmt drift 已清理，pre-commit / CI / closeout 均已接入 |
| `cargo check --workspace` | ✅ | 0 错误 |
| `scripts/test.ps1 test --workspace` | ✅ | closeout [12/23] 覆盖 |
| workspace clippy warning budget | ✅ | 当前预算 58，只降不升；不再把 clippy 退出码通过误读为 warning-free |
| executor warning budget | ✅ | 当前预算 0，新增 warning 阻断 |
| 前端 `npm run build` | ✅ | main frontend 已纳入 CI/closeout |
| 执行端前端 `npm run build` | ✅ | `frontend-executor` 已纳入 CI/closeout |
| 前端 `npx vitest run` | ✅ | 最新本地复验 289/289 (96 文件) |
| 前端 `npm run test:e2e` | ✅ | 2026-05-24 本地复验 21/21 通过，safe fallback 用例已按 strict fallback 收口 |
| npm audit | ✅ | frontend transitive `ws` 已通过 audit fix 清零 |
| P1 消化 | ✅ | 14/14 全部完成 (v3.5.1) |
| P2 消化 | ✅ | 18/23 完成 (5 延后至 v3.7.0+) |
| P3 消化 | ✅ | 14 项完成 (v3.7.0) |
| 刷新令牌轮换 | ✅ | SHA-256 重放检测 + 410 GONE |
| 告警自动恢复 | ✅ | 10/10 规则 resolve_condition |
| 编译缓存 | ✅ | SHA-256 key + 双检锁 + 50条 |
| 状态持久化 | ✅ | executor .json 原子写入 |
| 凭证 Zeroizing | ✅ | executor CredentialEntry |
| api_guard 强制 | ✅ | 缺头→401 |
| .unwrap() 清零 | ✅ | executor main.rs 0 处 |
| S0/P1 回归修复 | ✅ | 登录挂起、凭证 DELETE 405 已修复 |
| P2 测试进程锁 | ✅ | `scripts/test.ps1` / `scripts/test.sh` |
| 功能演进契约 | ✅ | 新增能力必须有登记、回归保护矩阵、兼容性与迁移说明 |
| Pre-commit hook 同步 | ✅ | `tools/check-pre-commit-hook.ps1` 已进入 closeout，防止 `.git/hooks/pre-commit` 与 `scripts/pre-commit` 再次漂移 |
| 清理边界门禁 | ✅ | `tools/check-cleanup-boundary.ps1` 已进入 CI/closeout，防止清理脚本触碰真实运行/图版本工件 |
| Rust 格式基线 | ✅ | `cargo fmt --check` 已进入三层门禁 |
| 版本号一致性 | ✅ | 关键元数据和用户可见入口统一到 3.7.1 |
| GP 合规 | ✅ | 当前 GP 已同步到 v3.7.1，v3.7.1 不扩大功能声明 |
| 超级规范化 | ✅ | v3.7.1 对齐 pre-commit / CI / closeout 三层门禁 |
| 完整 closeout | ✅ | v3.7.2 已升级为 23 项；2026-05-24 本地复跑 23/23 通过，最终 `git status --short` 为空 |

## 五维度评分 (v3.7.1 final closeout)

| 维度 | 评分 | 说明 |
|------|:--:|------|
| **功能开发进度** | **9.5/10** | 18 指标全实现 / 实时执行端 + OKX testnet / Paper/Live 切换 / 编译缓存 / Toast 系统 |
| **仓库稳定程度** | **9.4/10** | workspace test 通过 / vitest 289/289 / executor warning 0 / closeout 正在 v3.7.2 收口 |
| **发布就绪度** | **9.6/10** | P1 清零 / GP+超规范化 v3.7.1 对齐 / 版本一致性 / v3.7.2 closeout 23/23 通过 |
| **用户友好程度** | **9.5/10** | 术语全中文化 / 空状态引导 / 进度反馈 / 错误码映射 / ARIA 无障碍 / prefers-reduced-motion |
| **系统整体稳定性** | **9.3/10** | 事务保护 / TOCTOU 修复 / 三阶段无锁恢复 / 状态持久化 / Zeroizing / api_guard 强制 |
| **加权** | **9.4/10** | 加权 = 9.5×0.3 + 9.4×0.3 + 9.3×0.2 + 9.5×0.1 + 9.3×0.1 |

## 下一大版本准备 / V1 冻结方向

- 将当前的正式 QuantScript 主干、已落地的共享核心切片、`risk.profile(...)` / `execution.profile(...)`、价差切片和可执行回测/报告切片视为保留的 `V1` 界面
- 将更广泛的价差合约、`MACD` 共享核心扩展、通用风险/执行 DSL 增长、每笔交易比较、成交时间线比较视为延期工作
- 在宣布 `V1` 关闭之前，优先排雷收口、消除重复真实数据源、保持文档/提示/UI 措辞与保留界面一致，而非扩大功能范围
- 下一大版本启动前必须先建立功能演进登记和回归保护矩阵；不允许在未声明生命周期、fallback 和迁移边界的情况下扩大能力
- v3.7.1 作为 3.x 稳定归档点，后续 3.x 仅接受阻断修复、门禁补强和文档口径修正

## 接受规则

除非以下所有条件都成立，否则不应在 UI、文档或提示中暴露任何功能：

- 后端编译路径存在
- 运行时语义存在
- 验证可以诚实地拒绝不支持的使用
- 事件输出可以解释行为
- 测试覆盖该路径 (且测试实际可运行)
- 前端和文档文本保存为 UTF-8 并验证在渲染产品中正确显示
