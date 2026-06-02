# 当前状态与发布状态

> 最后更新：2026-05-30 | 当前版本：v4.7.0 ✅ | 当前治理基线：v4.15.0 三矩阵完全接管 | 当前架构推进：v4.16.0 模块化抽离第一波

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

QuantPilot v4.7.0 继承 v4.4.0 的嵌套状态机能力，并补齐 v4 执行回放、PaperActual 安全边界和 AI 提案分析：

- **桌面应用**: Tauri v2 自绘标题栏 Windows 桌面应用
- **前端**: Adobe 暗色面板设计系统, 图编辑器, 策略工作区, 回测详情/对比, 研究控制台, Toast 通知
- **后端**: Axum API (图保存/加载, 编译, paper 运行, 回测, 能力发现, 告警引擎)
- **执行端**: 独立 Axum 进程 (:3001), 策略部署/启动/停止/热调参, lightweight-charts K 线
- **运行时链**: data → intent → agent → risk → execution → fill
- **QuantScript**: 语法解析 → HIR → lowering → Core IR 完整编译管道
- **v4 runtime**: 后端 `/api/runtime/v4/run`、CLI `v4-run`、前端 `start_v4_simulation` 入口；执行端 `RunnerPool` 可部署/启动/停止 v4 graph；`/api/runtime/backtest` 可用 `runtime_kind=v4` 生成 `v4_artifact`；嵌套状态机支持深度 ≤2，tick replay 支持确定性排序、微结构指标和高级订单 evidence
- **安全**: AES-256-GCM 凭证保险库 (PBKDF2 1M/600K 轮), 本地会话/JWT 边界, 刷新令牌轮换+重放检测, 进程间加密通道
- **告警**: 10 条默认规则, resolve_condition 自动恢复, 去重
- **产品定位**: 单人本地桌面工具; 账户系统、2FA/RBAC、用户资料页和策略中心搜索/筛选均为 unsupported

当前全部 18 种指标的 K 线驱动 Intent 支持是真实的 (详见 README 指标表)。

当前回测支持也是真实的：历史回放, 持久化回测记录, 权益曲线, 夏普/索提诺/卡尔玛等 12 项指标, 以及 v4 deterministic MachineGraph replay 的 `v4_artifact`。v4 回放证据会保留 tick 输入规模、微结构指标、订单生命周期和嵌套状态机轨迹。

执行端支持：OKX Paper 模拟行情 + OKX testnet 实盘模拟 (REST + WebSocket), Paper/Live 模式切换, 策略迁移验证。

## 不得夸大之处

以下项目仍然不是真正的平台能力，不得描述为已支持：

- 当前现货 beta 中的真正套利代理支持
- 具有完整市场微观结构语义的研究级回测
- 任何 paper 策略都可以直接在 QuantScript 中表达
- 公开 SaaS 服务
- 多用户账户系统、注销/密码找回/2FA/RBAC
- 策略中心搜索、筛选、分页或排序工作台

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

v4.7.0 系统应被理解为：

- 支持的交易所: `binance`, `okx`
- 支持的交易对: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- 支持的运行时模式: `paper`, `live` (OKX testnet)
- 支持的执行模块: `builtin.execution.paper`, `live.okx`
- 全部已覆盖系统以 GP §10 功能覆盖矩阵为准
- 前端 Toast 通知系统, 术语全中文化, 空状态引导
- 执行端独立进程 (:3001), ParamsPanel 热调参, Paper/Live 切换
- v4 PaperSimulated runtime 已具备本地用户启动入口和执行端 v4 runner 集成；真实下单仍由 Risk Plane 和 ExecutionMachine 能力来源约束
- v4.16.0 模块化抽离已完成 system 入口抽离: `run_server`、`run_api_server` 和启动期 helper 归入 `system.entry.backend_process`，`quantpilot::run_server` 兼容入口保持不变；system 经验已回填为后续抽离准则，S1-S10 已完成 closeout 或静态 closeout，`root.system` 顶层阶段性 closeout 已刷新；backend 已完成 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D `backend.strategy_config` L3 模块壳抽离、BE-001E 其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout 并判定 `stop_split: true`，BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout 并判定 `stop_split: true`，BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并判定 `stop_split: true`，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并判定 `stop_split: true`，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并判定 `stop_split: true`，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并判定 `stop_split: true`，BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并判定 `stop_split: true`，BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断并设置父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，递归模块化流程已明确
- BE-001AD-01 已完成 `backend.runtime.routes` 父叶残余判断；route aggregate 仍有 evidence / report / experiment / ops 等残余候选，因此父叶保持 `stop_split: false`，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout并设置 `stop_split: true`；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout；BE-001AN-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单叶 closeout 并设置 `stop_split: true`，下一步进入 BE-001AO-01 父叶残余判断。
- 最新递归方案: BE-001AD-01 已完成 `backend.runtime.routes` 父叶残余判断，父叶保持 `stop_split: false`；BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout；BE-001AF-04 已完成 `runtime.mutation.parameter_mutation` 单叶 closeout并设置 `stop_split: false`；BE-001AN-04 已完成 `activation_snapshot_side_effect` 单叶 closeout，下一步只能进入 BE-001AO-01 父叶残余判断。
- BE-001AO-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第四轮父叶残余判断；`boundary_safety`、`activation_flow`、`rollback_flow` 与 `activation_snapshot_side_effect` 均已 closeout 并设置 `stop_split: true`，但父叶仍保持 `stop_split: false`。下一步只能进入 BE-001AP-01 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线。
- BE-001AP-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线；当前只冻结 lifecycle entry、transition persistence、in-memory index 和 activation / rollback 调用点，目标文件未创建。下一步只能进入 BE-001AP-02 抽离方案。
- BE-001AP-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案；当前固定目标文件、父级 path attribute、helper import、visibility 和回退点，但仍是 `no code movement`。下一步只能进入 BE-001AP-03 实际抽离。
- BE-001AP-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 实际抽离；lifecycle entry 与 transition persistence helper 已迁入 child，父级仍控制调用面。下一步只能进入 BE-001AP-04 单叶 closeout。
- BE-001AP-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AQ-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断。
- BE-001AQ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第五轮父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AR-01 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线。
- BE-001AR-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线；当前冻结 rollback id digest contract，目标文件未创建。下一步只能进入 BE-001AR-02 抽离方案。
- BE-001AR-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案；当前固定目标文件、父级 path attribute、helper import、visibility 和回退点，但仍是 `no code movement`。下一步只能进入 BE-001AR-03 实际抽离。
- BE-001AR-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 实际抽离；`runtime_parameter_mutation_rollback_record_id` 已迁入 child，父级保留受控 import。下一步只能进入 BE-001AR-04 单叶 closeout。
- BE-001AR-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001AS-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断。
- BE-001AS-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle` 第六轮父叶残余判断；父叶已设置 `stop_split: true`。下一步只能进入 BE-001AT-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
- BE-001AT-01 已完成 `runtime.mutation.parameter_mutation` 父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AU-01 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线。
- BE-001AU-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AU-02 抽离方案。
- BE-001AU-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation` 抽离方案；当前 `no code movement`，目标文件、父级 path attribute、handler re-export、`use super::*`、迁移清单和回退点已固定。下一步只能进入 BE-001AU-03 实际抽离。
- BE-001AU-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 实际抽离；`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 child，父级继续保留 list/detail handler。下一步只能进入 BE-001AU-04 单叶 closeout。
- BE-001AU-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
- BE-001AV-01 已完成 `runtime.mutation.parameter_mutation` 第二轮父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AW-01 `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线。
- BE-001AW-01 已建立 `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AW-02 抽离方案。
- BE-001AW-02 已建立 `runtime.mutation.parameter_mutation.record_query` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AW-03 实际抽离，第一轮只允许迁移 list/detail handler。
- BE-001AW-03 已完成 `runtime.mutation.parameter_mutation.record_query` 实际抽离；list/detail handler 已迁入 child，下一步只能进入 BE-001AW-04 单叶 closeout。
- BE-001AW-04 已完成 `runtime.mutation.parameter_mutation.record_query` 单叶 closeout 并设置 `stop_split: true`。下一步只能进入 BE-001AX-01 `runtime.mutation.parameter_mutation` 父叶残余判断。
- BE-001AX-01 已完成 `runtime.mutation.parameter_mutation` 第三轮父叶残余判断并设置父叶 `stop_split: true`。下一步只能进入 BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线。
- BE-001AY-01 已建立 `runtime.mutation.ai_proposal` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AY-02 抽离方案。
- BE-001AY-02 已建立 `runtime.mutation.ai_proposal` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AY-03 实际抽离。
- BE-001AY-03 已完成 `runtime.mutation.ai_proposal` 实际抽离；AI proposal / approval handlers 已迁入 `src/runtime/mutation/ai_proposal.rs`，下一步只能进入 BE-001AY-04 单叶 closeout。
- BE-001AY-04 已完成 `runtime.mutation.ai_proposal` 单叶 closeout 并设置 `stop_split: false`。下一步只能进入 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。
- BE-001AZ-01 已建立 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001AZ-02 抽离方案。
- BE-001AZ-02 已建立 `runtime.mutation.ai_proposal.static_check` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001AZ-03 实际抽离。
- BE-001AZ-03 已完成 `runtime.mutation.ai_proposal.static_check` 实际抽离；helper 与静态检查单测已迁入 child，下一步只能进入 BE-001AZ-04 单叶 closeout。
- BE-001AZ-04 已完成 `runtime.mutation.ai_proposal.static_check` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BA-01 `runtime.mutation.ai_proposal` 父叶残余判断。
- BE-001BA-01 已完成 `runtime.mutation.ai_proposal` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BB-01 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线。
- BE-001BB-01 已建立 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BB-02 抽离方案。
- BE-001BB-02 已建立 `runtime.mutation.ai_proposal.source_governance_identity` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BB-03 实际抽离。
- BE-001BB-03 已完成 `runtime.mutation.ai_proposal.source_governance_identity` 实际抽离；source/governance/id helper 已迁入 child，下一步只能进入 BE-001BB-04 单叶 closeout。
- BE-001BB-04 已完成 `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BC-01 `runtime.mutation.ai_proposal` 父叶残余判断。
- BE-001BC-01 已完成 `runtime.mutation.ai_proposal` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线。
- BE-001BD-01 已建立 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BD-02 抽离方案。
- BE-001BD-02 已建立 `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BD-03 实际抽离。
- BE-001BD-03 已完成 `runtime.mutation.ai_proposal.event_lifecycle` 实际抽离；event/lifecycle helper 已迁入 `src/runtime/mutation/ai_proposal/event_lifecycle.rs`，下一步只能进入 BE-001BD-04 单叶 closeout。
- BE-001BD-04 已完成 `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BE-01 `runtime.mutation.ai_proposal` 父叶残余判断。
- BE-001BE-01 已完成 `runtime.mutation.ai_proposal` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线。
- BE-001BF-01 已建立 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BF-02 抽离方案。
- BE-001BF-02 已建立 `runtime.mutation.ai_proposal.record_query` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BF-03 实际抽离。
- BE-001BF-03 已完成 `runtime.mutation.ai_proposal.record_query` 实际抽离；list/detail/read-through loader 已迁入 child，下一步只能进入 BE-001BF-04 单叶 closeout。
- BE-001BF-04 已完成 `runtime.mutation.ai_proposal.record_query` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BG-01 父叶残余判断。
- BE-001BG-01 已完成 `runtime.mutation.ai_proposal` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BH-01 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线。
- BE-001BH-01 已建立 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BH-02 抽离方案。
- BE-001BH-02 已建立 `runtime.mutation.ai_proposal.approval_review` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BH-03 实际抽离。
- BE-001BH-03 已完成 `runtime.mutation.ai_proposal.approval_review` 实际抽离；approval list/detail/approve/reject/claim 五个 handler 已迁入 `src/runtime/mutation/ai_proposal/approval_review.rs`，下一步只能进入 BE-001BH-04 单叶 closeout。
- BE-001BH-04 已完成 `runtime.mutation.ai_proposal.approval_review` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断。
- BE-001BI-01 已完成 `runtime.mutation.ai_proposal` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线。
- BE-001BJ-01 已建立 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BJ-02 抽离方案。
- BE-001BJ-02 已建立 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BJ-03 实际抽离。
- BE-001BJ-03 已完成 `runtime.mutation.ai_proposal.approval_persistence` 实际抽离；`persist_approval` 与 `load_approval_from_disk` 已迁入 `src/runtime/mutation/ai_proposal/approval_persistence.rs`，下一步只能进入 BE-001BJ-04 单叶 closeout。
- BE-001BJ-04 已完成 `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BK-01 `runtime.mutation.ai_proposal` 第六轮父叶残余判断。
- BE-001BK-01 已完成 `runtime.mutation.ai_proposal` 第六轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线。
- BE-001BL-01 已建立 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BL-02 抽离方案。
- BE-001BL-02 已建立 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BL-03 实际抽离。
- BE-001BL-03 已完成 `runtime.mutation.ai_proposal.sandbox_trigger` 实际抽离；下一步只能进入 BE-001BL-04 单叶 closeout。
- BE-001BL-04 已完成 `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BM-01 父叶残余判断。
- BE-001BM-01 已完成 `runtime.mutation.ai_proposal` 第七轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BN-01 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线。
- BE-001BN-01 已建立 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BN-02 抽离方案。
- BE-001BN-02 已建立 `runtime.mutation.ai_proposal.status_transition` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BN-03 实际抽离。
- BE-001BN-03 已完成 `runtime.mutation.ai_proposal.status_transition` 实际抽离；三个状态 helper 已迁入 `src/runtime/mutation/ai_proposal/status_transition.rs`，下一步只能进入 BE-001BN-04 单叶 closeout。
- BE-001BN-04 已完成 `runtime.mutation.ai_proposal.status_transition` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BO-01 `runtime.mutation.ai_proposal` 父叶残余判断。
- BE-001BO-01 已完成 `runtime.mutation.ai_proposal` 第八轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线。
- BE-001BP-01 已建立 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BP-02 抽离方案。
- BE-001BP-02 已建立 `runtime.mutation.ai_proposal.proposal_creation` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BP-03 实际抽离。
- BE-001BP-03 已完成 `runtime.mutation.ai_proposal.proposal_creation` 实际抽离；`create_runtime_ai_proposal` 已迁入 `src/runtime/mutation/ai_proposal/proposal_creation.rs`，下一步只能进入 BE-001BP-04 单叶 closeout。
- BE-001BP-04 已完成 `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断。
- BE-001BQ-01 已完成 `runtime.mutation.ai_proposal` 父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001BR-01 `backend.runtime.routes` 父叶残余判断。
- BE-001BR-01 已完成 `backend.runtime.routes` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线。
- BE-001BS-01 已建立 `backend.runtime.routes.experiment` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BS-02 抽离方案。
- BE-001BS-02 已建立 `backend.runtime.routes.experiment` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BS-03 实际抽离。
- BE-001BS-03 已完成 `backend.runtime.routes.experiment` 实际抽离；`src/backend/runtime/routes/experiment.rs` 已创建，下一步只能进入 BE-001BS-04 单叶 closeout。
- BE-001BS-04 已完成 `backend.runtime.routes.experiment` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BT-01 `backend.runtime.routes` 父叶残余判断。
- BE-001BT-01 已完成 `backend.runtime.routes` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线。
- BE-001BU-01 已建立 `backend.runtime.routes.evidence` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BU-02 抽离方案。
- BE-001BU-02 已建立 `backend.runtime.routes.evidence` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BU-03 实际抽离。
- BE-001BU-03 已完成 `backend.runtime.routes.evidence` 实际抽离；`src/backend/runtime/routes/evidence.rs` 已创建，下一步只能进入 BE-001BU-04 单叶 closeout。
- BE-001BU-04 已完成 `backend.runtime.routes.evidence` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BV-01 `backend.runtime.routes` 父叶残余判断。
- BE-001BV-01 已完成 `backend.runtime.routes` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线。
- BE-001BW-01 已建立 `backend.runtime.routes.event_stream` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BW-02 抽离方案。
- BE-001BW-02 已建立 `backend.runtime.routes.event_stream` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BW-03 实际抽离。
- BE-001BW-03 已完成 `backend.runtime.routes.event_stream` 实际抽离；下一步只能进入 BE-001BW-04 单叶 closeout。
- BE-001BW-04 已完成 `backend.runtime.routes.event_stream` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断。
- BE-001BX-01 已完成 `backend.runtime.routes` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001BY-01 `backend.runtime.routes.report_ops` 单子叶等价基线。
- BE-001BY-01 已建立 `backend.runtime.routes.report_ops` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001BY-02 抽离方案。
- BE-001BY-02 已建立 `backend.runtime.routes.report_ops` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001BY-03 实际抽离。
- BE-001BY-03 已完成 `backend.runtime.routes.report_ops` 实际抽离；下一步只能进入 BE-001BY-04 单叶 closeout。
- BE-001BY-04 已完成 `backend.runtime.routes.report_ops` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001BZ-01 父叶残余判断。
- BE-001BZ-01 已完成 `backend.runtime.routes` 第六轮父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001CA-01 `backend.runtime` 父叶残余判断。
- BE-001CA-01 已完成 `backend.runtime` 父叶残余判断；`backend.runtime.routes` 已关闭，但 `src/runtime/mod.rs` 仍有 report/evidence/ops handler 残余，因此父叶保持 `stop_split: false`，下一步只能进入 BE-001CB-01 `runtime.report_ops` 单子叶等价基线。
- BE-001CB-01 已建立 `runtime.report_ops` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CB-02 抽离方案，且 v1 ops/report endpoints 的测试缺口必须在方案中显式处理。
- BE-001CB-02 已建立 `runtime.report_ops` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CB-03 实际抽离，且不得迁移 `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或发布过渡连接。
- BE-001CB-03 已完成 `runtime.report_ops` 实际抽离；`src/runtime/report_ops.rs` 已创建并通过 `src/runtime/mod.rs` re-export 保持兼容出口。下一步只能进入 BE-001CB-04 单叶 closeout，且 v1 ops/report endpoint 测试缺口仍需在 closeout 中判断。
- BE-001CB-04 已完成 `runtime.report_ops` 单叶 closeout；抽离等价成立但该叶设置 `stop_split: false`。下一步只能进入 BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线，v1 ops/report endpoint 测试缺口仍保留。
- BE-001CC-01 已建立 `runtime.report_ops.runtime_report` 单子叶等价基线；当前 `no code movement`，目标文件尚未创建。下一步只能进入 BE-001CC-02 抽离方案。
- BE-001CC-02 已建立 `runtime.report_ops.runtime_report` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CC-03 实际抽离。
- BE-001CC-03 已完成 `runtime.report_ops.runtime_report` 实际抽离；`src/runtime/report_ops/runtime_report.rs` 已创建并通过 `src/runtime/report_ops.rs` re-export 保持兼容出口。下一步只能进入 BE-001CC-04 单叶 closeout。
- BE-001CC-04 已完成 `runtime.report_ops.runtime_report` 单叶 closeout；该子叶等价成立并设置 `stop_split: true`。父级 `runtime.report_ops` 仍为 `stop_split: false`，下一步只能进入 BE-001CD-01 父叶残余判断。
- BE-001CD-01 已完成 `runtime.report_ops` 父叶残余判断；父级仍保留 v1 report endpoints 与 merge/generation/storage health endpoints，因此 `stop_split: false`。下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。
- BE-001CE-01 已建立 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线；当前 `no code movement`，目标 child 文件尚未创建，v1 report endpoint 专门测试缺口已冻结。下一步只能进入 BE-001CE-02 抽离方案。
- BE-001CE-02 已建立 `runtime.report_ops.v1_report_endpoints` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CE-03 endpoint smoke 补测，不迁移 handler。
- BE-001CE-03 已完成 `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测；新增 `tests/api_v1_reports.rs` 覆盖三条 `/api/v1/reports/*` 基础 JSON contract。下一步只能进入 BE-001CE-04 实际抽离。
- BE-001CE-04 已完成 `runtime.report_ops.v1_report_endpoints` 实际抽离；`src/runtime/report_ops/v1_report_endpoints.rs` 已创建并承接三个 v1 report handler。下一步只能进入 BE-001CE-05 单叶 closeout。
- BE-001CE-05 已完成 `runtime.report_ops.v1_report_endpoints` 单叶 closeout；本叶设置 `stop_split: true`。下一步只能进入 BE-001CF-01 `runtime.report_ops` 父叶残余判断。
- BE-001CF-01 已完成 `runtime.report_ops` 父叶残余判断；父级仍保留 `list_merge_records`、`list_config_generations`、`get_storage_health`，因此 `stop_split: false`。下一步只能进入 BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线。
- BE-001CG-01 已建立 `runtime.report_ops.merge_generation_health` 单子叶等价基线；当前 `no code movement`，planned child 文件尚未创建，下一步只能进入 BE-001CG-02 抽离方案。
- BE-001CG-02 已建立 `runtime.report_ops.merge_generation_health` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CG-03 endpoint smoke 补测。
- BE-001CG-03 已完成 `runtime.report_ops.merge_generation_health` endpoint smoke 补测；新增 `tests/api_v1_ops_health.rs` 覆盖三条 v1 support/health endpoint 的最小 JSON contract。下一步只能进入 BE-001CG-04 实际抽离。
- BE-001CG-04 已完成 `runtime.report_ops.merge_generation_health` 实际抽离；`src/runtime/report_ops/merge_generation_health.rs` 承接三条 v1 support/health handler，父级只保留受控 re-export。下一步只能进入 BE-001CG-05 单叶 closeout。
- BE-001CG-05 已完成 `runtime.report_ops.merge_generation_health` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001CH-01 `runtime.report_ops` 父叶残余判断。
- BE-001CH-01 已完成 `runtime.report_ops` 第二轮父叶残余判断；三个 child 均已 closeout，父级设置 `stop_split: true`。下一步只能进入 BE-001CI-01 `backend.runtime` 父叶残余判断。
- BE-001CI-01 已完成 `backend.runtime` 第二轮父叶残余判断；父级保持 `stop_split: false`，下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
- BE-001CJ-01 已建立 `runtime.evidence_health` 单子叶等价基线；当前 `no code movement`，planned child 文件尚未创建，下一步只能进入 BE-001CJ-02 抽离方案。
- BE-001CJ-02 已建立 `runtime.evidence_health` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CJ-03 实际抽离。
- BE-001CJ-03 已完成 `runtime.evidence_health` 实际抽离；`src/runtime/evidence_health.rs` 已承接 evidence health / cleanup handler，下一步只能进入 BE-001CJ-04 单叶 closeout。
- BE-001CJ-04 已完成 `runtime.evidence_health` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。
- BE-001CK-01 已完成 `backend.runtime` 第三轮父叶残余判断；父级仍持有 mutation shared governance 与 query/guard/response support 残余，因此保持 `backend.runtime stop_split: false`，下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。
- BE-001CL-01 已建立 `runtime.mutation.shared_governance` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CL-02 抽离方案。
- BE-001CL-02 已建立 `runtime.mutation.shared_governance` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CL-03 实际抽离。
- BE-001CL-03 已完成 `runtime.mutation.shared_governance` 实际抽离；9 个 shared governance helper 已迁入 child，下一步只能进入 BE-001CL-04 单叶 closeout。
- BE-001CL-04 已完成 `runtime.mutation.shared_governance` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001CM-01 `backend.runtime` 第四轮父叶残余判断。
- BE-001CM-01 已完成 `backend.runtime` 第四轮父叶残余判断；父级仍有 query DTO / run guard / response support / experiment limit 残余，因此保持 `backend.runtime stop_split: false`，下一步只能进入 BE-001CN-01 `runtime.query_support` 单子叶等价基线。
- BE-001CN-01 已建立 `runtime.query_support` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CN-02 抽离方案。
- BE-001CN-02 已建立 `runtime.query_support` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CN-03 实际抽离。
- BE-001CN-03 已完成 `runtime.query_support` 实际抽离；`src/runtime/query_support.rs` 承接 Query DTO 与 filter/replay normalization，下一步只能进入 BE-001CN-04 单叶 closeout。
- BE-001CN-04 已完成 `runtime.query_support` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。
- BE-001CO-01 已完成 `backend.runtime` 第五轮父叶残余判断；父级仍有 response support / run guard / experiment limit / parent include residual，因此保持 `backend.runtime stop_split: false`，下一步只能进入 BE-001CP-01 `runtime.response_support` 单子叶等价基线。
- BE-001CP-01 已建立 `runtime.response_support` 单子叶等价基线；当前 `no code movement`，planned child 文件尚未创建，下一步只能进入 BE-001CP-02 抽离方案。
- BE-001CP-02 已建立 `runtime.response_support` 抽离方案；当前 `no code movement`，下一步 BE-001CP-03 才允许创建 child 并迁移 3 个 response DTO。
- BE-001CP-03 已完成 `runtime.response_support` 实际抽离；`src/runtime/response_support.rs` 已创建并承接 3 个 response DTO，`src/runtime/run.rs` 降为 drained include，下一步只能进入 BE-001CP-04 单叶 closeout。
- BE-001CP-04 已完成 `runtime.response_support` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。
- BE-001CQ-01 已完成 `backend.runtime` 第六轮父叶残余判断；父级保持 `stop_split: false`，下一步只能进入 BE-001CR-01 `runtime.run_guard` 单子叶等价基线。
- BE-001CR-01 已建立 `runtime.run_guard` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CR-02 抽离方案。
- BE-001CR-02 已建立 `runtime.run_guard` 抽离方案；当前 `no code movement`，方案选择不单独开 test-first 批次，下一步只能进入 BE-001CR-03 实际抽离。
- BE-001CR-03 已完成 `runtime.run_guard` 实际抽离；`src/runtime/run_guard.rs` 已创建，下一步只能进入 BE-001CR-04 单叶 closeout。
- BE-001CR-04 已完成 `runtime.run_guard` 单叶 closeout；`runtime.run_guard stop_split: true`，下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。
- BE-001CS-01 已完成 `backend.runtime` 第七轮父叶残余判断；父级保持 `stop_split: false`，下一步只能进入 BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线。
- BE-001CT-01 已建立 `runtime.experiment_limit` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001CT-02 抽离方案。
- BE-001CT-02 已建立 `runtime.experiment_limit` test-first 抽离方案；当前 `no code movement`，下一步只能进入 BE-001CT-03 endpoint smoke 补测。
- BE-001CT-03 已完成 `runtime.experiment_limit` endpoint smoke 补测；`api_experiments` 已覆盖 36 个变体超过 27 上限的 bad_request，下一步只能进入 BE-001CT-04 实际抽离。
- BE-001CT-04 已完成 `runtime.experiment_limit` 实际抽离；`src/runtime/experiment_limit.rs` 已创建，下一步只能进入 BE-001CT-05 单叶 closeout。
- BE-001CT-05 已完成 `runtime.experiment_limit` 单叶 closeout；`runtime.experiment_limit stop_split: true`，下一步只能进入 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断。
- BE-001CU-01 已完成 `backend.runtime` 第八轮父叶残余判断；父级真实残余只剩 `include!("run.rs")`、`include!("mutation.rs")` 与 `include!("backtest.rs")` drained parent include cleanup，因此保持 `backend.runtime stop_split: false`，下一步只能进入 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线。
- BE-001CV-01 已建立 `runtime.parent_include_cleanup` 单子叶等价基线；当前 `no code movement`，只冻结三条 drained `include!(...)`、三个 drained 文件和 public 出口等价影响面，下一步只能进入 BE-001CV-02 抽离方案。
- BE-001CV-02 已建立 `runtime.parent_include_cleanup` 抽离方案；当前 `no code movement`，下一批 BE-001CV-03 只允许删除三条 drained `include!(...)` 与三个 drained 文件，不处理 `backend.runtime` 父叶 closeout 或发布过渡。
- BE-001CV-03 已完成 `runtime.parent_include_cleanup` 实际 cleanup；三条 drained `include!(...)` 与三个 drained 文件已删除，public handler owner、route facade、schema/frontend/state/persistence owner、lock order 与 release transition guard 未变更，下一步只能进入 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断。
- BE-001CW-01 已完成 `backend.runtime` 第九轮父叶残余判断；`src/runtime/mod.rs` 已无行为体和 drained include，但仍保留 parent import bridge，因此 `backend.runtime stop_split: false`，下一步只能进入 BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线。
- BE-001CX-01 已建立 `runtime.parent_import_bridge` 单子叶等价基线；当前 46 个 runtime 文件仍存在 `use super::*` / `super::` 依赖，下一步只能进入 BE-001CX-02 抽离方案，不能直接批量改写 Rust import。
- BE-001CX-02 已建立 `runtime.parent_import_bridge` 抽离方案；后续采用 staged explicit import pass，首个实际批次固定为 BE-001CX-03 `runtime.root_support_import_pilot`，只处理 `query_support` 与 `response_support`。
- BE-001CX-03 已完成 `runtime.root_support_import_pilot` 实际抽离；`query_support` 与 `response_support` 已改为显式 import，runtime parent bridge 依赖文件数从 46 降为 44，下一步只能进入 BE-001CX-04 单叶 closeout。
- BE-001CX-04 已完成 `runtime.root_support_import_pilot` 单叶 closeout；该 pilot 设置 `stop_split: true`，parent import bridge 仍剩 44 个依赖文件，下一步只能进入 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线。
- BE-001CY-01 已建立 `runtime.root_entry_import_pass` 单子叶等价基线；冻结 root entry 候选文件并确认 `run_guard.rs` 为 test-only super import，下一步只能进入 BE-001CY-02 抽离方案。
- BE-001CY-02 已建立 `runtime.root_entry_import_pass` 抽离方案；下一步 BE-001CY-03 只允许处理 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs`，`report_ops`、test-only `run_guard` 与 `src/runtime/mod.rs` 父桥全部延后。
- BE-001CY-03 已完成 `runtime.root_entry_import_pass` 实际抽离；`event_stream` 与 `evidence_health` 已改为显式 import，parent bridge 依赖文件数从 44 降为 42，下一步只能进入 BE-001CY-04 单叶 closeout。
- BE-001CY-04 已完成 `runtime.root_entry_import_pass` 单叶 closeout；该 pass 设置 `stop_split: true`，下一步只能进入 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线。
- BE-001CZ-01 已建立 `runtime.report_ops_import_pass` 单子叶等价基线；冻结 `report_ops` parent facade 与 3 个 child 的 transitive parent surface risk，下一步只能进入 BE-001CZ-02 抽离方案。
- BE-001CZ-02 已建立 `runtime.report_ops_import_pass` 抽离方案；下一步 BE-001CZ-03 只允许同批处理 report_ops four-file pocket，不能混入 root parent bridge 或其他 runtime 子树。
- BE-001CZ-03 已完成 `runtime.report_ops_import_pass` 实际抽离；report_ops four-file pocket 已改为显式 import，parent bridge 依赖文件数从 42 降为 38，下一步只能进入 BE-001CZ-04 单叶 closeout。
- BE-001CZ-04 已完成 `runtime.report_ops_import_pass` 单叶 closeout；该 pass 设置 `stop_split: true`，parent import bridge 仍剩 38 个依赖文件，下一步只能进入 BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断。
- BE-001DA-01 已完成 `runtime.parent_import_bridge` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线。
- BE-001DB-01 已建立 `runtime.run_import_pass` 单子叶等价基线；冻结 4 个 run child 的 import 收敛边界，下一步只能进入 BE-001DB-02 抽离方案。
- BE-001DB-02 已建立 `runtime.run_import_pass` 抽离方案；下一步 BE-001DB-03 只允许同批改写 4 个 run child 的 explicit import。
- BE-001DB-03 已完成 `runtime.run_import_pass` 实际抽离；4 个 run child 已移除 parent wildcard import，parent bridge 依赖文件数从 38 降为 34，下一步只能进入 BE-001DB-04 单叶 closeout。
- BE-001DB-04 已完成 `runtime.run_import_pass` 单叶 closeout；该 pass 设置 `stop_split: true`，下一步只能进入 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断。
- BE-001DC-01 已完成 `runtime.parent_import_bridge` 父叶残余判断；父叶保持 `stop_split: false`，剩余分布为 root 1、run 0、backtest 11、mutation 21、test-only 1，下一步只能进入 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线。
- BE-001DD-01 已建立 `runtime.backtest_import_pass` 单子叶等价基线；冻结 11 个 backtest 残余文件与父级输入面，下一步只能进入 BE-001DD-02 抽离方案。
- BE-001DD-02 已建立 `runtime.backtest_import_pass` 抽离方案；拒绝 11 文件整批改写，下一步只能进入 BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线。
- BE-001DE-01 已建立 `runtime.backtest.record_store_import_pass` 单子叶等价基线；冻结 `src/runtime/backtest/record_store.rs` 的 4 个 public 方法与预期显式输入面，下一步只能进入 BE-001DE-02 抽离方案。
- BE-001DE-02 已建立 `runtime.backtest.record_store_import_pass` 抽离方案；下一步 BE-001DE-03 只允许改写 `src/runtime/backtest/record_store.rs` 顶部 import。
- BE-001DE-03 已完成 `runtime.backtest.record_store_import_pass` 实际抽离；`record_store.rs` 已移除 parent wildcard import，parent bridge 依赖文件数从 34 降为 33，下一步只能进入 BE-001DE-04 单叶 closeout。
- BE-001DE-04 已完成 `runtime.backtest.record_store_import_pass` 单叶 closeout；该 import pocket 设置 `stop_split: true`，parent bridge 依赖文件数仍为 33，下一步只能进入 BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断。
- BE-001DF-01 已完成 `runtime.backtest_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，剩余分布为 root 1、run 0、backtest 10、mutation 21、test-only 1，下一步只能进入 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线。
- BE-001DG-01 已建立 `runtime.backtest.replay_import_pass` 单子叶等价基线；冻结 `src/runtime/backtest/replay.rs` 的 `get_backtest_replay` 边界与预期显式输入面，下一步只能进入 BE-001DG-02 抽离方案。
- BE-001DG-02 已建立 `runtime.backtest.replay_import_pass` 抽离方案；下一步 BE-001DG-03 只允许改写 `src/runtime/backtest/replay.rs` 顶部 import。
- BE-001DG-03 已完成 `runtime.backtest.replay_import_pass` 实际抽离；`replay.rs` 已移除 parent wildcard import，parent bridge 依赖文件数从 33 降为 32，下一步只能进入 BE-001DG-04 单叶 closeout。
- BE-001DG-04 已完成 `runtime.backtest.replay_import_pass` 单叶 closeout；本叶设置 `stop_split: true`，下一步只能进入 BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断。
- BE-001DH-01 已完成 `runtime.backtest_import_pass` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。
- BE-001DI-01 已建立 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线；冻结四文件 pocket，下一步只能进入 BE-001DI-02 抽离方案。
- BE-001DI-02 已建立 `runtime.backtest.experiment_sweep_import_pass` 抽离方案；下一步 BE-001DI-03 只允许四文件 import rewrite。
- BE-001DI-03 已完成 `runtime.backtest.experiment_sweep_import_pass` 实际抽离；四文件 parent import 已收敛，parent bridge 依赖文件数从 32 降为 28，下一步只能进入 BE-001DI-04 单叶 closeout。
- BE-001DI-04 已完成 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout；设置 `stop_split: true`，旧的三叶暂停目标取消，下一步只能进入 BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断。
- BE-001DJ-01 已完成 `runtime.backtest_import_pass` 第三轮父叶残余判断；backtest 剩余 5 个 execution_start 组依赖，下一步只能进入 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。
- BE-001DK-01 已建立 `runtime.backtest.execution_start_import_pass` 单子叶等价基线；冻结五文件 pocket，下一步只能进入 BE-001DK-02 抽离方案。
- BE-001DK-02 已建立 `runtime.backtest.execution_start_import_pass` 抽离方案；下一步 BE-001DK-03 只允许五文件 import rewrite。
- BE-001DK-03 已完成 `runtime.backtest.execution_start_import_pass` 实际抽离；backtest parent bridge residual 清零，parent bridge 总数从 28 降为 23，下一步只能进入 BE-001DK-04 单叶 closeout。
- BE-001DK-04 已完成 `runtime.backtest.execution_start_import_pass` 单叶 closeout；设置 `stop_split: true`，backtest residual 保持 0，下一步只能进入 BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断。
- BE-001DL-01 已完成 `runtime.backtest_import_pass` 第四轮父叶残余判断；backtest 父叶设置 `stop_split: true`，下一步只能进入 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断。
- BE-001DM-01 已完成 `runtime.parent_import_bridge` 父叶残余判断；父叶保持 `stop_split: false`，剩余分布 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1，下一步只能进入 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线。
- BE-001DN-01 已建立 `runtime.mutation_import_pass` 单子叶等价基线；冻结 21 个 mutation parent bridge 文件，下一步只能进入 BE-001DN-02 抽离方案。
- BE-001DN-02 已建立 `runtime.mutation_import_pass` 抽离方案；拒绝 21 文件整批 rewrite，下一步只能进入 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。
- BE-001DO-01 已建立 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线；冻结 `src/runtime/mutation/shared_governance.rs` 的 9 个 helper 与显式 import 输入面，下一步只能进入 BE-001DO-02 抽离方案。
- BE-001DO-02 已建立 `runtime.mutation.shared_governance_import_pass` 抽离方案；下一步只允许单文件改写 `src/runtime/mutation/shared_governance.rs` 顶部 import 并进入 BE-001DO-03 抽离记录。
- BE-001DO-03 已完成 `runtime.mutation.shared_governance_import_pass` 实际抽离；parent bridge 剩余降为 root 1 / run 0 / backtest 0 / mutation 20 / test-only 1，下一步只能进入 BE-001DO-04 单叶 closeout。
- BE-001DO-04 已完成 `runtime.mutation.shared_governance_import_pass` 单叶 closeout；设置 `stop_split: true`，下一步只能进入 BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断。
- BE-001DP-01 已完成 `runtime.mutation_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线。
- BE-001DQ-01 已建立 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线；冻结 10 个 parameter mutation residual 文件，下一步只能进入 BE-001DQ-02 抽离方案并先判断是否继续拆小 pocket。
- BE-001DQ-02 已建立 `runtime.mutation.parameter_mutation_import_pass` 抽离方案；拒绝 10 文件整批 rewrite，下一步只能进入 BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线。
- BE-001DR-01 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线；冻结 `record_query.rs` 读路径输入面，下一步只能进入 BE-001DR-02 抽离方案。
- BE-001DR-02 已建立 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案；下一步只允许单文件改写 `src/runtime/mutation/parameter_mutation/record_query.rs` 顶部 import。
- BE-001DR-03 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 实际抽离；parent bridge 剩余降为 total 21 / mutation 19，下一步只能进入 BE-001DR-04 单叶 closeout。
- BE-001DR-04 已完成 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout；设置 `stop_split: true`，旧三叶暂停目标保持取消，下一步只能进入 BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
- BE-001DS-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线。
- BE-001DT-01 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线；冻结 `proposal_creation.rs` 输入面，下一步只能进入 BE-001DT-02 抽离方案。
- BE-001DT-02 已建立 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案；下一步只允许单文件改写 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 顶部 import。
- BE-001DT-03 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 实际抽离；parent bridge 剩余降为 total 20 / mutation 18，下一步只能进入 BE-001DT-04 单叶 closeout。
- BE-001DT-04 已完成 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout；设置 `stop_split: true`，旧三叶暂停目标保持取消，下一步只能进入 BE-001DU-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
- BE-001DU-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线。
- BE-001DV-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线；冻结 7 文件 lifecycle 输入面，下一步只能进入 BE-001DV-02 抽离方案。
- BE-001DV-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案；拒绝 7 文件同批 rewrite，下一步只能进入 BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线。
- BE-001DW-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线；冻结 `boundary_safety.rs` 输入面，下一步只能进入 BE-001DW-02 抽离方案。
- BE-001DW-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案；下一步只能进入 BE-001DW-03 单文件 import rewrite，不得改函数体、可见性、facade、activation / rollback sibling 或 release transition。
- BE-001DW-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 实际抽离；`boundary_safety.rs` parent wildcard import 已清理，下一步只能进入 BE-001DW-04 单叶 closeout。
- BE-001DW-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout；设置 `stop_split: true`，下一步只能进入 BE-001DX-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断。
- BE-001DX-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001DY-01 `rollback_record_identity_import_pass` 单子叶等价基线。

## 当前收尾/发布状态

v3.7.0 完成 v3.5.0→v3.7.0 全版本演进 (12 项新功能 + 32 项审计修复 + 12 项 UX 优化 + 14 项 P3 消化)。

v4.0.0 是 MAJOR 架构收口：在 v3.7.1 稳定基线上落地状态机化 QuantScript、事件模型、Risk Plane、ExecutionMachine 能力来源、前端 capability 真源和 Developer Learning Pipeline。

v4.7.0 是当前 MINOR 集成收口：在 v4.5.0 tick replay 和高级订单、v4.6.0 PaperActual 安全边界基础上，补齐 v4 AI 提案静态约束、沙箱回放比较和回测工件分析。当前按 `v4.7.0/01-规划方案.md` 执行实现与验证。

后续优化和治理路线按三矩阵治理与超级规范化的审计→优化闭环推进：

| 版本 | 类型 | 目标 | 状态 |
|------|------|------|:--:|
| v4.8.0 | MINOR | 双执行切面 + P2 质量收敛；用户侧统一为 `PaperSimulated` 和 `PaperActual`，真实资金自动交易不作为当前能力宣称 | 规划与 W1-W4 落地记录已归档 |
| v4.8.1 | PATCH | OpenAPI 凭证路径结构、route diff 基线、部署 profile 矩阵、四平面治理；账户相关项按用户要求裁出 | 已落地 |
| v4.8.2 | PATCH | zh-CN 转义修复、执行端 i18n、QS 编辑器、CSS 收敛、首次体验、404、中文用户指南 | 已落地 |
| v4.9.0 | MINOR | PaperActual 安全启动门禁、插件执行安全、策略包导入导出、设置页、API 版本治理、AI 沙箱队列、执行端图表控制 | 已落地 |
| v4.10.0 | MINOR | 亮色主题、执行端 i18n 收口、教程自动触发、Tab keep-alive、产品边界 unsupported 固化 | 已落地 |
| v4.11.0 | MINOR | v4 策略配置系统一等化，聚合 artifact、preflight、diff、AI proposal 配置域绑定 | 推进中 |
| v4.12.0 | MINOR governance | 三矩阵治理入口启用，建立流程矩阵、规范矩阵、引导矩阵、模块树和完全落地路线 | 已落地 |
| v4.13.0 | MINOR governance | 模块树白箱扩面，覆盖 active 模块、关键 public 方法和模块化重构通道 | 已落地 |
| v4.14.0 | MINOR governance | 治理门禁自动化，检查三矩阵声明、引导坐标、模块树漂移和发布过渡保护 | 已落地 |
| v4.15.0 | MINOR governance | 三矩阵完全接管 closeout，旧入口导流并形成治理收口报告 | 当前治理基线 |
| v4.16.0 | MINOR architecture / governance | 模块化抽离第一波，已完成 system 试水、经验回填、递归模块化流程、S1-S10 closeout 或静态 closeout、`root.system` 顶层阶段性 closeout；backend 已完成 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D strategy_config L3 模块壳抽离、BE-001E 其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录；BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout；BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `v4_runtime_execution` 单叶 closeout，BE-001R-04 已完成 `legacy_dispatch` 单叶 closeout 并判定 `stop_split: true`，BE-001S-01 已完成父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并判定 `stop_split: true`，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并判定 `stop_split: true`，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并判定 `stop_split: true`，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并判定 `stop_split: true`，BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，BE-001AA-04 已完成 `record_lifecycle` 单叶 closeout 并判定 `stop_split: true`，BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`，BE-001BI-01 已完成 `runtime.mutation.ai_proposal` 第五轮父叶残余判断 | 控制面落地中 |

使用下面的专用文档作为活跃发布与治理界面：

- [三矩阵治理入口](../00-matrix-governance/README.md)
- [三矩阵治理完全落地路线](../00-matrix-governance/landing-roadmap.md)
- [递归高速执行协议](../00-matrix-governance/recursive-speed-protocol.md)
- [v3.7.0 规划方案](../06-milestones/v3.7.0/01-规划方案.md)
- [v3.7.0 综合优化清单](../06-milestones/v3.7.0/02-综合优化清单.md)
- [v3.7.1 规划方案](../06-milestones/v3.7.1/01-规划方案.md)
- [v3.7.1 综合优化清单](../06-milestones/v3.7.1/02-综合优化清单.md)
- [v3.7.1 closeout 基线](../06-milestones/v3.7.1/03-closeout.md)
- [v4.0.0 规划方案](../06-milestones/v4.0.0/01-规划方案.md)
- [v4.0.0 closeout 报告](../06-milestones/v4.0.0/03-closeout.md)
- [v4.7.0 规划方案](../06-milestones/v4.7.0/01-规划方案.md)
- [v4.8.0 规划方案](../06-milestones/v4.8.0/01-规划方案.md)
- [v4.8.1 规划方案](../06-milestones/v4.8.1/01-规划方案.md)
- [v4.8.1 综合优化清单](../06-milestones/v4.8.1/02-综合优化清单.md)
- [v4.8.2 规划方案](../06-milestones/v4.8.2/01-规划方案.md)
- [v4.9.0 规划方案](../06-milestones/v4.9.0/01-规划方案.md)
- [v4.10.0 规划方案](../06-milestones/v4.10.0/01-规划方案2.md)
- [v4.10.0 落地记录](../06-milestones/v4.10.0/02-落地记录.md)
- [v4.11.0 规划方案](../06-milestones/v4.11.0/01-规划方案.md)
- [v4.12.0 规划方案](../06-milestones/v4.12.0/01-规划方案.md)
- [v4.12.0 落地记录](../06-milestones/v4.12.0/02-落地记录.md)
- [v4.13.0 规划方案](../06-milestones/v4.13.0/01-规划方案.md)
- [v4.13.0 落地记录](../06-milestones/v4.13.0/02-落地记录.md)
- [v4.14.0 规划方案](../06-milestones/v4.14.0/01-规划方案.md)
- [v4.14.0 落地记录](../06-milestones/v4.14.0/02-落地记录.md)
- [v4.15.0 规划方案](../06-milestones/v4.15.0/01-规划方案.md)
- [v4.15.0 治理 closeout](../06-milestones/v4.15.0/02-治理closeout.md)
- [v4.16.0 模块化抽离第一波规划](../06-milestones/v4.16.0/01-规划方案.md)
- [Claude 产品/UX/功能完整度审计核查](../05-testing/Claude产品UX功能完整度审计核查-v4.7.0-2026-05-26.md)
- [支持矩阵](../03-implementation/governance/implementation-support-matrix.md)
- [编译链合约](../03-implementation/governance/implementation-compile-chain-contract.md)
- [功能演进契约](../03-implementation/governance/implementation-feature-evolution-contract.md)

当前仓库级状态 (v4.7.0 closeout 已完成):

| 检查项 | 状态 | 备注 |
|--------|:--:|------|
| `cargo fmt --check` | ✅ | 全仓 rustfmt drift 已清理，pre-commit / CI / closeout 均已接入 |
| `cargo check --workspace` | ✅ | 0 错误 |
| `scripts/test.ps1 test --workspace` | ✅ | closeout [13/26] 覆盖 |
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
| Pre-commit hook 同步 | ✅ | `tools/check-pre-commit-hook.ps1` 已进入 closeout，防止 `.git/hooks/pre-commit` 与 `scripts/pre-commit` 再次漂移；递归高速执行改为 staged-file 智能分流 |
| 清理边界门禁 | ✅ | `tools/check-cleanup-boundary.ps1` 已进入 CI/closeout，防止清理脚本触碰真实运行/图版本工件 |
| 三矩阵治理门禁 | ✅ | `tools/check-matrix-governance.ps1` 已接入 closeout [7/26]，用于检查治理入口、提案模板、模块树漂移和发布过渡协议 |
| Rust 格式基线 | ✅ | `cargo fmt --check` 已进入三层门禁 |
| v4 runtime 入口 | ✅ | `/api/runtime/v4/run`、CLI `v4-run`、前端 `start_v4_simulation` 已接入 |
| 执行端 v4 集成 | ✅ | RunnerPool、OKX Market 事件、部署 API、SSE evidence、执行端面板按 v4.2.0 规划落实 |
| v4 回测 + 多交易对 | ✅ | `runtime_kind=v4` 回测、`v4_artifact`、回测详情 evidence、v4 模板和多交易对展开按 v4.3.0 规划落实 |
| v4 执行回放与高级订单 | ✅ | tick replay、OCO bracket、trailing stop、GTD 过期、cancel/replace amend 和微结构指标按 v4.5.0 规划落实 |
| PaperActual 安全边界 | ✅ | v4 PaperActual demo 边界、Risk Plane 强制和 runtime_simulated 阻断按 v4.6.0 规划落实 |
| v4 AI 提案与回放分析 | ✅ | v4 AI proposal 回测来源约束、沙箱回放比较和工件分析摘要按 v4.7.0 规划落实 |
| 版本号一致性 | ✅ | 关键元数据和用户可见入口统一到 4.7.0 |
| GP 合规 | ✅ | 当前 GP 已同步到 v4.0.0，v4.7.0 已复核执行回放、PaperActual 边界、AI 提案和 evidence 保护矩阵 |
| 超级规范化 | ✅ | v4.0.0 对齐 MAJOR 演化通道、前端真源通道和学习流水线 closeout |
| 完整 closeout | ✅ | closeout 门禁已扩展为 26 项，第 7 项覆盖三矩阵治理，第 26 项覆盖能力栈一致性与元流水线 DryRun |

## 五维度评分 (v4.7.0 closeout)

| 维度 | 评分 | 说明 |
|------|:--:|------|
| **功能开发进度** | **9.5/10** | 18 指标全实现 / 实时执行端 + OKX testnet / Paper/Live 切换 / 编译缓存 / Toast 系统 |
| **仓库稳定程度** | **9.4/10** | workspace test 通过 / vitest 289/289 / executor warning 0 / closeout 正在 v3.7.2 收口 |
| **发布就绪度** | **9.4/10** | P1 清零 / GP+超规范化 v4.0.0 对齐 / v4.7.0 版本一致性 / 26 项 closeout 门禁已接入 |
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
- BE-001DY-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线；当前为 `no code movement`，冻结 `runtime_parameter_mutation_rollback_record_id` 的 digest/id 语义和预期显式 import 输入面。下一步只能进入 BE-001DY-02 抽离方案；旧三叶暂停目标保持取消。
- BE-001DY-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离方案；当前为 `no code movement`，下一步只允许改写 `rollback_record_identity.rs` 顶部 import。下一步只能进入 BE-001DY-03 实际抽离记录。
- BE-001DY-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 实际抽离；`rollback_record_identity.rs` 已移除 parent wildcard import，residual 降为 total 18 / mutation 16 / parameter_mutation 6 / transition_lifecycle 5。下一步只能进入 BE-001DY-04 单叶 closeout。
- BE-001DY-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单叶 closeout；本 import pocket 设置 `stop_split: true`，下一步只能进入 BE-001DZ-01 `transition_lifecycle_import_pass` 父叶残余判断。
- BE-001DZ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EA-01 `transition_record_persistence_import_pass` 单子叶等价基线。
- BE-001EA-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线；当前为 `no code movement`，冻结 lifecycle entry 与 persistence helper 输入面。下一步只能进入 BE-001EA-02 抽离方案。
- BE-001EA-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案；当前为 `no code movement`，下一步只允许改写 `transition_record_persistence.rs` 顶部 import。下一步只能进入 BE-001EA-03 实际抽离记录。
- BE-001EA-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 实际抽离；`transition_record_persistence.rs` 已移除 parent wildcard import，residual 降为 total 17 / mutation 15 / parameter_mutation 5 / transition_lifecycle 4。下一步只能进入 BE-001EA-04 单叶 closeout。
- BE-001EA-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单叶 closeout；本 import pocket 设置 `stop_split: true`，下一步只能进入 BE-001EB-01 `transition_lifecycle_import_pass` 父叶残余判断。
- BE-001EB-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EC-01 `activation_snapshot_side_effect_import_pass` 单子叶等价基线。
- BE-001EC-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线；当前为 `no code movement`，冻结 activation snapshot side-effect 输入面。下一步只能进入 BE-001EC-02 抽离方案。
- BE-001EC-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案；当前为 `no code movement`，下一步只允许改写 `activation_snapshot_side_effect.rs` 顶部 import。下一步只能进入 BE-001EC-03 实际抽离记录。
- BE-001EC-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 实际抽离；`activation_snapshot_side_effect.rs` 已移除 parent wildcard import，residual 降为 total 16 / mutation 14 / parameter_mutation 4 / transition_lifecycle 3。下一步只能进入 BE-001EC-04 单叶 closeout。
- BE-001EC-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单叶 closeout；本 import pocket 设置 `stop_split: true`。下一步只能进入 BE-001ED-01 `transition_lifecycle_import_pass` 父叶残余判断。
- BE-001ED-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EE-01 `activation_flow_import_pass` 单子叶等价基线。
- BE-001EE-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线；当前为 `no code movement`，冻结 activation flow 输入面。下一步只能进入 BE-001EE-02 抽离方案。
- BE-001EE-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案；当前为 `no code movement`，下一步只允许改写 `activation_flow.rs` 顶部 import。下一步只能进入 BE-001EE-03 实际抽离记录。
- BE-001EE-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 实际抽离；`activation_flow.rs` 已移除 parent wildcard import，函数体与 sibling 未改。下一步只能进入 BE-001EE-04 单叶 closeout。
- BE-001EE-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单叶 closeout；该 import pocket 设置 `stop_split: true`，下一步只能进入 BE-001EF-01 父叶残余判断。
- BE-001EF-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EG-01 `rollback_flow_import_pass` 单子叶等价基线。
- BE-001EG-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线；当前为 `no code movement`，冻结 rollback flow 输入面。下一步只能进入 BE-001EG-02 抽离方案。
- BE-001EG-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离方案；当前为 `no code movement`，下一步只允许改写 `rollback_flow.rs` 顶部 import。下一步只能进入 BE-001EG-03 实际抽离记录。
- BE-001EG-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 实际抽离；`rollback_flow.rs` 已移除 parent wildcard import，函数体与 sibling 未改。下一步只能进入 BE-001EG-04 单叶 closeout。
- BE-001EG-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单叶 closeout；本 import pocket 设置 `stop_split: true`，下一步只能进入 BE-001EH-01 父叶残余判断。
- BE-001EH-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EI-01 `parent_facade_import_pass` 单子叶等价基线。
- BE-001EI-01 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线；当前 `no code movement`，冻结 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` parent facade 输入面。下一步只能进入 BE-001EI-02 抽离方案。
- BE-001EI-02 已建立 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EI-03 单文件实际 import rewrite。
- BE-001EI-03 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 实际抽离；`transition_lifecycle.rs` 已移除 parent wildcard import，下一步只能进入 BE-001EI-04 单叶 closeout。
- BE-001EI-04 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EJ-01 父叶残余判断。
- BE-001EJ-01 已完成 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第七轮父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001EK-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
- BE-001EK-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第三轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EL-01 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线。
- BE-001EL-01 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线；当前 `no code movement`，冻结 `src/runtime/mutation/parameter_mutation.rs` parent facade 输入面。下一步只能进入 BE-001EL-02 抽离方案。
- BE-001EL-02 已建立 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EL-03 单文件实际 import rewrite。
- BE-001EL-03 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 实际抽离；`parameter_mutation.rs` 已移除 parent wildcard import 并显式导入 `mutation_event_contract`，下一步只能进入 BE-001EL-04 单叶 closeout。
- BE-001EL-04 已完成 `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EM-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断。
- BE-001EM-01 已完成 `runtime.mutation.parameter_mutation_import_pass` 第四轮父叶残余判断并设置 `stop_split: true`；下一步只能进入 BE-001EN-01 `runtime.mutation_import_pass` 父叶残余判断。
- BE-001EN-01 已完成 `runtime.mutation_import_pass` 第二轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001EO-01 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线。
- BE-001EO-01 已建立 `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001EO-02 抽离方案。
- BE-001EO-02 已建立 `runtime.mutation.ai_proposal_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。
- BE-001EP-01 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001EP-02 抽离方案。
- BE-001EP-02 已建立 `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EP-03 单文件实际 import rewrite。
- BE-001EP-03 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 实际抽离；`record_query.rs` 已移除 parent wildcard import，下一步只能进入 BE-001EP-04 单叶 closeout。
- BE-001EP-04 已完成 `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EQ-01 父叶残余判断。
- BE-001EQ-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第三轮父叶残余判断并保持 `stop_split: false`；下一步只能进入 BE-001ER-01 `source_governance_identity_import_pass` 单子叶等价基线。
- BE-001ER-01 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001ER-02 抽离方案。
- BE-001ER-02 已建立 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001ER-03 单文件实际 import rewrite。
- BE-001ER-03 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 实际抽离；`source_governance_identity.rs` 已移除 parent wildcard import，下一步只能进入 BE-001ER-04 单叶 closeout。
- BE-001ER-04 已完成 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001ES-01 父叶残余判断。
- BE-001ES-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001ET-01 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线。
- BE-001ET-01 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001ET-02 抽离方案。
- BE-001ET-02 已建立 `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001ET-03 单文件实际 import rewrite。
- BE-001ET-03 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 实际抽离；`static_check.rs` 已移除 parent wildcard import，下一步只能进入 BE-001ET-04 单叶 closeout。
- BE-001ET-04 已完成 `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EU-01 父叶残余判断。
- BE-001EU-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线。
- BE-001EV-01 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001EV-02 抽离方案。
- BE-001EV-02 已建立 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EV-03 单文件实际 import rewrite。
- BE-001EV-03 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 实际抽离；`event_lifecycle.rs` 已移除 parent wildcard import，下一步只能进入 BE-001EV-04 单叶 closeout。
- BE-001EV-04 已完成 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EW-01 父叶残余判断。
- BE-001EW-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EX-01 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线。
- BE-001EX-01 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001EX-02 抽离方案。
- BE-001EX-02 已建立 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EX-03 单文件实际 import rewrite。
- BE-001EX-03 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 实际抽离；`approval_persistence.rs` 已移除 parent wildcard import，下一步只能进入 BE-001EX-04 单叶 closeout。
- BE-001EX-04 已完成 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001EY-01 父叶残余判断。
- BE-001EY-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001EZ-01 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线。
- BE-001EZ-01 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线；当前 `no code movement`，下一步只能进入 BE-001EZ-02 抽离方案。
- BE-001EZ-02 已建立 `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案；当前 `no code movement`，下一步只能进入 BE-001EZ-03 单文件实际 import rewrite。
- BE-001EZ-03 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 实际抽离；`status_transition.rs` 已移除 parent wildcard import，下一步只能进入 BE-001EZ-04 单叶 closeout。
- BE-001EZ-04 已完成 `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001FA-01 父叶残余判断。
- BE-001FA-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断；父叶继续 `stop_split: false`，下一步只能进入 BE-001FB-01 `sandbox_trigger_import_pass` 等价基线。
- BE-001FB-01 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线；下一步只能进入 BE-001FB-02 抽离方案。
- BE-001FB-02 已建立 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案；下一步只能进入 BE-001FB-03 单文件实际 import rewrite。
- BE-001FB-03 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 实际抽离；`sandbox_trigger.rs` 已移除 parent wildcard import，下一步只能进入 BE-001FB-04 单叶 closeout。
- BE-001FB-04 已完成 `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001FC-01 父叶残余判断。
- BE-001FC-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断；父叶继续 `stop_split: false`，下一步只能进入 BE-001FD-01 `approval_review_import_pass` 等价基线。
- BE-001FD-01 已建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线；本批 `no code movement`，冻结 approval list/detail/approve/reject/claim 五个 handler 的 import 输入面、锁顺序、状态机、lifecycle 与持久化顺序；下一步只能进入 BE-001FD-02 抽离方案。
- BE-001FD-02 已建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案；本批 `no code movement`，下一步 BE-001FD-03 只允许改写 `approval_review.rs` 顶部 import，不得改变函数体、锁顺序、reviewer count、lifecycle、status transition 或 persist order。
- BE-001FD-03 已完成 `runtime.mutation.ai_proposal.approval_review_import_pass` 实际抽离；`approval_review.rs` 顶部 `use super::*` 已移除并改为显式 import，五个 handler 函数体未改。下一步只能进入 BE-001FD-04 单叶 closeout。
- BE-001FD-04 已建立 `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout；本批 `no code movement`，设置 `stop_split: true`，下一步只能进入 BE-001FE-01 父叶残余判断。
- BE-001FE-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断；本批 `no code movement`，父叶继续 `stop_split: false`，下一步只能进入 BE-001FF-01 `proposal_creation_import_pass` 等价基线。
- BE-001FF-01 已建立 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线；本批 `no code movement`，冻结 `create_runtime_ai_proposal` 的输入面、状态机、自动审批、事件写入、持久化顺序与 sandbox trigger。下一步只能进入 BE-001FF-02 抽离方案。
- BE-001FF-02 已建立 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案；本批 `no code movement`，BE-001FF-03 只允许改写 `proposal_creation.rs` 顶部 import，不得改函数体、自动审批、事件写入、persist order、sandbox trigger 或 sibling owner。
- BE-001FF-03 已完成 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 实际抽离；`proposal_creation.rs` 已移除 `use super::*` 并改为显式 import。下一步只能进入 BE-001FF-04 单叶 closeout。
- BE-001FF-04 已完成 `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout；设置 `stop_split: true`，下一步只能进入 BE-001FG-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断。
- BE-001FG-01 已建立 `runtime.mutation.ai_proposal_import_pass` 第十一轮父叶残余判断；父叶保持 `stop_split: false`，下一步只能进入 BE-001FH-01 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线。
- BE-001FH-01 已建立 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线；当前 `no code movement`，冻结 `src/runtime/mutation/ai_proposal.rs` parent facade 输入面。下一步只能进入 BE-001FH-02 抽离方案。
- BE-001FH-02 已建立 `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案；当前 `no code movement`，下一步 BE-001FH-03 只允许改写 `src/runtime/mutation/ai_proposal.rs` 的 import 面。
- BE-001FH-03 已完成 `runtime.mutation.ai_proposal.parent_facade_import_pass` 实际抽离；`src/runtime/mutation/ai_proposal.rs` 已移除 parent wildcard import 并显式保留 `RuntimeApprovalListQuery` hidden input。下一步只能进入 BE-001FH-04 单叶 closeout。
- BE-001FH-04 已完成 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout；本批 `no code movement`，设置 `stop_split: true`。下一步只能进入 BE-001FI-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断。
- BE-001FI-01 已完成 `runtime.mutation.ai_proposal_import_pass` 第十二轮父叶残余判断；本批 `no code movement`，父叶设置 `stop_split: true`。下一步只能进入 BE-001FJ-01 `runtime.mutation_import_pass` 父叶残余判断。
- BE-001FJ-01 已完成 `runtime.mutation_import_pass` 第三轮父叶残余判断；本批 `no code movement`，父叶设置 `stop_split: true`。下一步只能进入 BE-001FK-01 `runtime.parent_import_bridge` 父叶残余判断。
- BE-001FK-01 已完成 `runtime.parent_import_bridge` 第四轮父叶残余判断；本批 `no code movement`，父叶保持 `stop_split: false`。下一步只能进入 BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线。
- BE-001FL-01 已建立 `runtime.root_parent_facade_import_pass` 单子叶等价基线；本批 `no code movement`，冻结 `src/runtime/mod.rs` 的 module declaration、public re-export、private helper bridge、query_support 与 response_support parent surface。下一步只能进入 BE-001FL-02 抽离方案。
- BE-001FL-02 已建立 `runtime.root_parent_facade_import_pass` 抽离方案；本批 `no code movement`，固定 BE-001FL-03 只能删除 `src/runtime/mod.rs` 中两个 unused root import residual，不新增替代 import。
- BE-001FL-03 已完成 `runtime.root_parent_facade_import_pass` 实际抽离；`src/runtime/mod.rs` 已删除 `use super::*` 与 `use axum::extract::Query`，未新增替代 import。下一步只能进入 BE-001FL-04 单叶 closeout。
- BE-001FL-04 已完成 `runtime.root_parent_facade_import_pass` 单叶 closeout；本批 `no code movement`，设置 `stop_split: true`，下一步只能进入 BE-001FM-01 `runtime.parent_import_bridge` 父叶残余判断。
- BE-001FM-01 已完成 `runtime.parent_import_bridge` 第五轮父叶残余判断；本批 `no code movement`，生产级 parent wildcard residual 为 0，父叶设置 `stop_split: true`。下一步只能进入 BE-001FN-01 `backend.runtime` 父叶残余判断。
- BE-001FN-01 已完成 `backend.runtime` 第十轮父叶残余判断；本批 `no code movement`，`backend.runtime stop_split: true`，下一步只能进入 BE-001FO-01 `backend` 父叶残余判断。
- BE-001FO-01 已完成 `backend` 父叶残余判断；本批 `no code movement`，`backend stop_split: false`，下一步只能进入 BE-001FP-01 `backend.graph_compile` 父叶残余判断。
- BE-001FP-01 已完成 `backend.graph_compile` 父叶残余判断；本批 `no code movement`，`backend.graph_compile stop_split: false`，下一步只能进入 BE-001FQ-01 `backend.graph_compile.quantscript_graph` 单子叶等价基线。
- BE-001FQ-01 已建立 `backend.graph_compile.quantscript_graph` 单子叶等价基线；本批 `no code movement`，冻结 route handler、shared helper、compile/graph/runtime/test 调用面，下一步只能进入 BE-001FQ-02 抽离方案。
- BE-001FQ-02 已建立 `backend.graph_compile.quantscript_graph` 抽离方案；本批 `no code movement`，下一步只能进入 BE-001FQ-03 实际抽离记录，通过 `src/backend/graph_compile/quantscript_graph.rs` 接管真实实现，并以 root parent re-export surface 保持 compile/graph/runtime/test 调用面。
- BE-001FQ-03 已完成 `backend.graph_compile.quantscript_graph` 实际抽离；`src/graph_quantscript_api.rs` 已删除，真实实现迁入 `src/backend/graph_compile/quantscript_graph.rs`，`src/lib.rs` 通过 root parent re-export surface 保持 helper 调用面。下一步只能进入 BE-001FQ-04 单叶 closeout。
- BE-001FQ-04 已完成 `backend.graph_compile.quantscript_graph` 单叶 closeout；等价成立但本叶保持 `stop_split: false`，下一步只能进入 BE-001FR-01 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线。
- BE-001FR-01 已建立 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线；本批 `no code movement`，冻结 generator / node renderer / scalar renderer 与 `build_quantscript_node_sources` 隐性调用点。下一步只能进入 BE-001FR-02 抽离方案。
- BE-001FR-02 已建立 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 抽离方案；本批 `no code movement`，固定 planned child 与四函数迁移清单。下一步只能进入 BE-001FR-03 实际抽离记录。
- BE-001FR-03 已完成 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 实际抽离；child file 承接 graph-to-QS generator，父级保留 re-export 与 `pub(super)` 内部 helper 通信。下一步只能进入 BE-001FR-04 单叶 closeout。
- BE-001FR-04 已完成 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单叶 closeout 并设置 `stop_split: true`；下一步只能进入 BE-001FS-01 `backend.graph_compile.quantscript_graph` 父叶残余判断。
- BE-001FS-01 已完成 `backend.graph_compile.quantscript_graph` 父叶残余判断；父叶仍保持 `stop_split: false`，下一步只能进入 BE-001FT-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线。
- BE-001FT-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线；本批 `no code movement`，冻结 `convert_graph_json_to_script_module` 输入输出、分支语义、错误行为和 caller 映射。下一步只能进入 BE-001FT-02 抽离方案。
- BE-001FT-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion` 抽离方案；本批 `no code movement`，下一步 BE-001FT-03 只允许迁移 `convert_graph_json_to_script_module` 到 planned child。
- BE-001FT-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 实际抽离；`convert_graph_json_to_script_module` 已迁入 child，父级通过 `mod formal_module_conversion` 与受控 re-export 保持调用面。下一步只能进入 BE-001FT-04 单叶 closeout。
- BE-001FT-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 单叶 closeout；本批 `no code movement`，确认等价成立但本叶保持 `stop_split: false`。下一步只能进入 BE-001FU-01 父叶残余判断。
- BE-001FU-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion` 父叶残余判断；本批 `no code movement`，确认父叶仍有 input/data/profile/intent/parse 残余，下一步只能进入 BE-001FV-01 `intent_lowering` 单子叶等价基线。
- BE-001FV-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单子叶等价基线；本批 `no code movement`，冻结 upstream edge、source var、七个 built-in intent 分支和 unsupported intent failure。下一步只能进入 BE-001FV-02 抽离方案。
- BE-001FV-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 抽离方案；本批 `no code movement`，固定 planned child `intent_lowering.rs`、`append_intent_lowering_lines` helper signature 和父到子单向调用。下一步只能进入 BE-001FV-03 实际抽离记录。
- BE-001FV-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 实际抽离；child file 承接 intent block，父级保留 `mod intent_lowering` 与 `append_intent_lowering_lines` 单向调用。下一步只能进入 BE-001FV-04 单叶 closeout。
- BE-001FV-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单叶 closeout；本批 `no code movement`，确认等价成立但本叶保持 `stop_split: false`。下一步只能进入 BE-001FW-01 父叶残余判断。
- BE-001FW-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；本批 `no code movement`，选择 `spread_observer_lowering`。下一步只能进入 BE-001FX-01 单子叶等价基线。
- BE-001FX-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单子叶等价基线；本批 `no code movement`，冻结 spread observer branch 输入、fallback 和 QS line 顺序。下一步只能进入 BE-001FX-02 抽离方案。
- BE-001FX-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 抽离方案；本批 `no code movement`，固定 planned child、helper signature、父级 `mod spread_observer_lowering;` 和父到子调用方式。下一步只能进入 BE-001FX-03 实际抽离记录。
- BE-001FX-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 实际抽离；child file 承接 spread observer branch，父级只保留 `mod spread_observer_lowering;` 和受控调用。下一步只能进入 BE-001FX-04 单叶 closeout。
- BE-001FX-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单叶 closeout；本批 `no code movement`，设置 `spread_observer_lowering stop_split: true`。下一步只能进入 BE-001FY-01 `intent_lowering` 父叶残余判断。
- BE-001FY-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；本批 `no code movement`，选择 `macd_lowering`。下一步只能进入 BE-001FZ-01 单子叶等价基线。
- BE-001FZ-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单子叶等价基线；本批 `no code movement`，冻结 MACD 参数 fallback、QS line、BUY/SELL emit 顺序和父子通信规则。下一步只能进入 BE-001FZ-02 抽离方案。
- BE-001FZ-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 抽离方案；本批 `no code movement`，固定 planned child、父级 `mod`、helper signature、允许迁移块和回退方案。下一步只能进入 BE-001FZ-03 实际抽离记录。
- BE-001FZ-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 实际抽离；child file 承接 MACD branch，父级只保留 `mod macd_lowering;` 与受控调用。下一步只能进入 BE-001FZ-04 单叶 closeout。
- BE-001FZ-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单叶 closeout；本批 `no code movement`，设置 `macd_lowering stop_split: true`。下一步只能进入 BE-001GA-01 `intent_lowering` 父叶残余判断。
- BE-001GA-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；本批 `no code movement`，父叶保持 `stop_split: false`，本轮选择 `double_ma_lowering`。下一步只能进入 BE-001GB-01 单子叶等价基线。
- BE-001GB-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单子叶等价基线；本批 `no code movement`，冻结 double MA 参数 fallback、SMA line、BUY emit 与父子通信规则。下一步只能进入 BE-001GB-02 抽离方案。
- BE-001GB-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 抽离方案；本批 `no code movement`，固定 planned child、helper signature 和允许迁移块。下一步只能进入 BE-001GB-03 实际抽离记录。
- BE-001GB-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 实际抽离；child file 承接 double MA branch，父级只保留 `mod double_ma_lowering;` 与受控调用。下一步只能进入 BE-001GB-04 单叶 closeout。
- BE-001GB-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单叶 closeout；本批 `no code movement`，设置 `double_ma_lowering stop_split: true`。下一步只能进入 BE-001GC-01 `intent_lowering` 父叶残余判断。
- BE-001GC-01 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断；本批 `no code movement`，父叶保持 `stop_split: false`，本轮选择 `rsi_lowering`。下一步只能进入 BE-001GD-01 单子叶等价基线。
- BE-001GD-01 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单子叶等价基线；本批 `no code movement`，冻结 RSI 参数 fallback、QS line、BUY emit 与父子通信规则。下一步只能进入 BE-001GD-02 抽离方案。
- BE-001GD-02 已建立 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 抽离方案；本批 `no code movement`，固定 planned child、helper signature、允许迁移块和回退方案。下一步只能进入 BE-001GD-03 实际抽离记录。
- BE-001GD-03 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 实际抽离；child file 承接 RSI branch，父级只保留 `mod rsi_lowering;` 与受控调用。下一步只能进入 BE-001GD-04 单叶 closeout。
- BE-001GD-04 已完成 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单叶 closeout；本批 `no code movement`，设置 `rsi_lowering stop_split: true`。下一步只能进入 BE-001GE-01 `intent_lowering` 父叶残余判断。
- GOV-LEAF-SPLIT-GATE 已固化递归叶子细分判定硬规则；后续新增单叶 closeout / 父叶残余判断必须触发 `leaf_split_decision_gate`，缺少基础门槛、强拆分触发、强停止条件、判定结果或下一步时治理门禁应失败。
- BE-001GE-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects ma_deviation_lowering；下一步: BE-001GF-01 ma_deviation_lowering baseline_plan。
- BE-001GF-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering baseline and extraction plan frozen；下一步: BE-001GF-02 ma_deviation_lowering extract_closeout。
- BE-001GF-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering actual extraction and closeout complete；下一步: BE-001GG-01 intent_lowering parent residual judgment。
- BE-001GG-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects momentum_lowering；下一步: BE-001GH-01 momentum_lowering baseline_plan。
- BE-001GH-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering baseline and extraction plan frozen；下一步: BE-001GH-02 momentum_lowering extract_closeout。
- BE-001GH-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering actual extraction and closeout complete；下一步: BE-001GI-01 intent_lowering parent residual judgment。
- BE-001GI-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects zscore_lowering；下一步: BE-001GJ-01 zscore_lowering baseline_plan。
- BE-001GJ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering baseline and extraction plan frozen；下一步: BE-001GJ-02 zscore_lowering extract_closeout。
- BE-001GJ-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering actual extraction and closeout complete；下一步: BE-001GK-01 intent_lowering parent residual judgment。
- BE-001GK-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects shared_intent_context；下一步: BE-001GL-01 shared_intent_context baseline_plan。
- BE-001GL-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context baseline and extraction plan frozen；下一步: BE-001GL-02 shared_intent_context extract_closeout。
- BE-001GL-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context actual extraction and closeout complete；下一步: BE-001GM-01 intent_lowering parent residual judgment。
- BE-001GM-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects unsupported_intent_failure；下一步: BE-001GN-01 unsupported_intent_failure baseline_plan。
- BE-001GN-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure equivalence baseline and extraction plan；下一步: BE-001GN-02 unsupported_intent_failure extract_closeout。
- BE-001GN-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure actual extraction and closeout complete；下一步: BE-001GO-01 intent_lowering parent residual closeout。
- BE-001GO-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent closeout sets stop_split true；下一步: BE-001GP-01 formal_module_conversion parent residual judgment。
- BE-001GP-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects data_source_lowering；下一步: BE-001GQ-01 data_source_lowering baseline_plan。
- BE-001GQ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering equivalence baseline and extraction plan；下一步: BE-001GQ-02 data_source_lowering extract_closeout。
- BE-001GQ-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering actual extraction and closeout complete；下一步: BE-001GR-01 formal_module_conversion parent residual judgment。
- BE-001GR-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects profile_lowering；下一步: BE-001GS-01 profile_lowering baseline_plan。
- BE-001GS-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering equivalence baseline and extraction plan；下一步: BE-001GS-02 profile_lowering extract_closeout。
- BE-001GS-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering actual extraction and closeout complete；下一步: BE-001GT-01 formal_module_conversion parent residual judgment。
- BE-001GT-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects input_shape_validation；下一步: BE-001GU-01 input_shape_validation baseline_plan。
- BE-001GU-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation equivalence baseline and extraction plan；下一步: BE-001GU-02 input_shape_validation extract_closeout。
- BE-001GU-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation actual extraction and closeout complete；下一步: BE-001GV-01 formal_module_conversion parent residual judgment。
- BE-001GV-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects terminal_parse；下一步: BE-001GW-01 terminal_parse baseline_plan。
- BE-001GW-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse equivalence baseline and extraction plan；下一步: BE-001GW-02 terminal_parse extract_closeout。
- BE-001GW-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse actual extraction and closeout complete；下一步: BE-001GX-01 formal_module_conversion parent residual judgment。
- BE-001GX-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects unsupported_node_logging；下一步: BE-001GY-01 unsupported_node_logging baseline_plan。
- BE-001GY-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging equivalence baseline and extraction plan；下一步: BE-001GY-02 unsupported_node_logging extract_closeout。
- BE-001GY-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging actual extraction and closeout complete；下一步: BE-001GZ-01 formal_module_conversion parent closeout。
- BE-001GZ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent closeout sets stop_split true；下一步: BE-001HA-01 quantscript_graph parent residual judgment。
- BE-001HA-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects strategy_graph_parser；下一步: BE-001HB-01 strategy_graph_parser baseline_plan。
- BE-001HB-01 `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser equivalence baseline and extraction plan；下一步: BE-001HB-02 strategy_graph_parser extract_closeout。
- BE-001HB-02 `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser actual extraction and closeout complete；下一步: BE-001HC-01 quantscript_graph parent residual judgment。
- BE-001HC-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects artifact_target_projection；下一步: BE-001HD-01 artifact_target_projection baseline_plan。
- BE-001HD-01 `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection equivalence baseline and extraction plan；下一步: BE-001HD-02 artifact_target_projection extract_closeout。
- BE-001HD-02 `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection actual extraction and closeout complete；下一步: BE-001HE-01 quantscript_graph parent residual judgment。
- BE-001HE-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects route_surface；下一步: BE-001HF-01 route_surface baseline_plan。
- BE-001HF-01 `backend.graph_compile.quantscript_graph.route_surface` route_surface equivalence baseline and extraction plan；下一步: BE-001HF-02 route_surface extract_closeout。
- BE-001HF-02 `backend.graph_compile.quantscript_graph.route_surface` route_surface actual extraction and closeout complete；下一步: BE-001HG-01 quantscript_graph parent closeout。
- BE-001HG-01 `backend.graph_compile.quantscript_graph` quantscript_graph parent closeout sets stop_split true；下一步: BE-001HH-01 backend.graph_compile parent residual judgment。
- BE-001HH-01 `backend.graph_compile` backend.graph_compile parent residual judgment selects compile；下一步: BE-001HI-01 backend.graph_compile.compile baseline_plan。
- BE-001HI-01 `backend.graph_compile.compile` backend.graph_compile.compile equivalence baseline and extraction plan；下一步: BE-001HI-02 backend.graph_compile.compile extract_closeout。
- BE-001HI-02 `backend.graph_compile.compile` backend.graph_compile.compile actual extraction and closeout complete；下一步: BE-001HJ-01 backend.graph_compile parent residual judgment。
- BE-001HJ-01 `backend.graph_compile` backend.graph_compile parent residual judgment selects graph；下一步: BE-001HK-01 backend.graph_compile.graph baseline_plan。
- BE-001HK-01 `backend.graph_compile.graph` backend.graph_compile.graph equivalence baseline and extraction plan；下一步: BE-001HK-02 backend.graph_compile.graph extract_closeout。
- BE-001HK-02 `backend.graph_compile.graph` backend.graph_compile.graph actual extraction and closeout complete；下一步: BE-001HL-01 backend.graph_compile parent closeout。
- BE-001HL-01 `backend.graph_compile` backend.graph_compile parent closeout sets stop_split true；下一步: BE-001HM-01 backend parent residual judgment。
- BE-001HM-01 `backend` backend parent residual judgment selects capability；下一步: BE-001HN-01 backend.capability baseline_plan。
- BE-001HN-01 `backend.capability` backend.capability equivalence baseline and extraction plan；下一步: BE-001HN-02 backend.capability extract_closeout。
- BE-001HN-02 `backend.capability` backend.capability actual extraction and closeout complete；下一步: BE-001HO-01 backend parent residual judgment。
- BE-001HO-01 `backend` backend parent residual judgment selects strategy_config；下一步: BE-001HP-01 backend.strategy_config parent residual judgment。
- BE-001HP-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects artifact；下一步: BE-001HQ-01 backend.strategy_config.artifact baseline_plan。
- BE-001HQ-01 `backend.strategy_config.artifact` backend.strategy_config.artifact equivalence baseline and extraction plan；下一步: BE-001HQ-02 backend.strategy_config.artifact extract_closeout。
- BE-001HQ-02 `backend.strategy_config.artifact` backend.strategy_config.artifact route owner extraction complete；下一步: BE-001HR-01 backend.strategy_config.artifact parent residual judgment。
- BE-001HR-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects schema_model；下一步: BE-001HS-01 backend.strategy_config.artifact.schema_model baseline_plan。
- BE-001HS-01 `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan；下一步: BE-001HS-02 backend.strategy_config.artifact.schema_model extract_closeout。
- BE-001HS-02 `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model actual extraction complete；下一步: BE-001HT-01 backend.strategy_config.artifact parent residual judgment。
- BE-001HT-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects domain_projection；下一步: BE-001HU-01 backend.strategy_config.artifact.domain_projection baseline_plan。
- BE-001HU-01 `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan；下一步: BE-001HU-02 backend.strategy_config.artifact.domain_projection extract_closeout。
- BE-001HU-02 `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection actual extraction complete；下一步: BE-001HV-01 backend.strategy_config.artifact parent residual judgment。
- BE-001HV-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects builder_core；下一步: BE-001HW-01 backend.strategy_config.artifact.builder_core baseline_plan。
- BE-001HW-01 `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan；下一步: BE-001HW-02 backend.strategy_config.artifact.builder_core extract_closeout。
- BE-001HW-02 `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core actual extraction complete；下一步: BE-001HX-01 backend.strategy_config.artifact parent closeout。
- BE-001HX-01 `backend.strategy_config.artifact` backend.strategy_config.artifact parent closeout sets stop_split true；下一步: BE-001HY-01 backend.strategy_config parent residual judgment。
- BE-001HY-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects preflight；下一步: BE-001HZ-01 backend.strategy_config.preflight baseline_plan。
- BE-001HZ-01 `backend.strategy_config.preflight` backend.strategy_config.preflight equivalence baseline and extraction plan；下一步: BE-001HZ-02 backend.strategy_config.preflight extract_closeout。
- BE-001HZ-02 `backend.strategy_config.preflight` backend.strategy_config.preflight actual extraction complete；下一步: BE-001IA-01 backend.strategy_config.preflight single_leaf_closeout。
- BE-001IA-01 `backend.strategy_config.preflight` backend.strategy_config.preflight single leaf closeout sets stop_split true；下一步: BE-001IB-01 backend.strategy_config parent residual judgment。
- BE-001IB-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects diff；下一步: BE-001IC-01 backend.strategy_config.diff baseline_plan。
- BE-001IC-01 `backend.strategy_config.diff` backend.strategy_config.diff equivalence baseline and extraction plan；下一步: BE-001IC-02 backend.strategy_config.diff extract_closeout。
- BE-001IC-02 `backend.strategy_config.diff` backend.strategy_config.diff actual extraction complete；下一步: BE-001ID-01 backend.strategy_config.diff single_leaf_closeout。
- BE-001ID-01 `backend.strategy_config.diff` backend.strategy_config.diff single leaf closeout keeps stop_split false；下一步: BE-001IE-01 backend.strategy_config.diff parent residual judgment。
- BE-001IE-01 `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects artifact_diff；下一步: BE-001IF-01 backend.strategy_config.diff.artifact_diff baseline_plan。
