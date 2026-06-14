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
- [Claude 产品/UX/功能完整度审计核查](../09-archive/testing-retired/Claude产品UX功能完整度审计核查-v4.7.0-2026-05-26.md)
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
- BE-001IF-01 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan；下一步: BE-001IF-02 backend.strategy_config.diff.artifact_diff extract_closeout。
- BE-001IF-02 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff actual extraction complete；下一步: BE-001IG-01 backend.strategy_config.diff.artifact_diff single_leaf_closeout。
- BE-001IG-01 `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff single leaf closeout sets stop_split true；下一步: BE-001IH-01 backend.strategy_config.diff parent residual judgment。
- BE-001IH-01 `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects evidence_diff；下一步: BE-001II-01 backend.strategy_config.diff.evidence_diff baseline_plan。
- BE-001II-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan；下一步: BE-001II-02 backend.strategy_config.diff.evidence_diff extract_closeout。
- BE-001II-02 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff actual extraction complete；下一步: BE-001IJ-01 backend.strategy_config.diff.evidence_diff single_leaf_closeout。
- BE-001IJ-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff single leaf closeout keeps stop_split false；下一步: BE-001IK-01 backend.strategy_config.diff.evidence_diff parent residual judgment。
- BE-001IK-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory；下一步: BE-001IL-01 backend.strategy_config.diff.evidence_diff.machine_trajectory baseline_plan。
- BE-001IL-01 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan；下一步: BE-001IL-02 backend.strategy_config.diff.evidence_diff.machine_trajectory extract_closeout。
- BE-001IL-02 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete；下一步: BE-001IM-01 backend.strategy_config.diff.evidence_diff.machine_trajectory single_leaf_closeout。
- BE-001IM-01 `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory single leaf closeout stops further split；下一步: BE-001IN-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- BE-001IN-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane；下一步: BE-001IO-01 backend.strategy_config.diff.evidence_diff.risk_plane baseline_plan。
- BE-001IO-01 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan；下一步: BE-001IO-02 backend.strategy_config.diff.evidence_diff.risk_plane extract_closeout。
- BE-001IO-02 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane actual extraction complete；下一步: BE-001IP-01 backend.strategy_config.diff.evidence_diff.risk_plane single_leaf_closeout。
- BE-001IP-01 `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane single leaf closeout stops further split；下一步: BE-001IQ-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- BE-001IQ-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability；下一步: BE-001IR-01 backend.strategy_config.diff.evidence_diff.execution_capability baseline_plan。
- BE-001IR-01 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan；下一步: BE-001IR-02 backend.strategy_config.diff.evidence_diff.execution_capability extract_closeout。
- BE-001IR-02 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability actual extraction complete；下一步: BE-001IS-01 backend.strategy_config.diff.evidence_diff.execution_capability single_leaf_closeout。
- BE-001IS-01 `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability single leaf closeout stops further split；下一步: BE-001IT-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- BE-001IT-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics；下一步: BE-001IU-01 backend.strategy_config.diff.evidence_diff.metrics baseline_plan。
- BE-001IU-01 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan；下一步: BE-001IU-02 backend.strategy_config.diff.evidence_diff.metrics extract_closeout。
- BE-001IU-02 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics actual extraction complete；下一步: BE-001IV-01 backend.strategy_config.diff.evidence_diff.metrics single_leaf_closeout。
- BE-001IV-01 `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics single leaf closeout stops further split；下一步: BE-001IW-01 backend.strategy_config.diff.evidence_diff parent_residual_judgment。
- BE-001IW-01 `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent closeout retains report assembly and shared helpers；下一步: BE-001IX-01 backend.strategy_config.diff parent_residual_judgment。
- BE-001IX-01 `backend.strategy_config.diff` backend.strategy_config.diff parent closeout keeps facade and child mediation；下一步: BE-001IY-01 backend.strategy_config parent_residual_judgment。
- BE-001IY-01 `backend.strategy_config` backend.strategy_config parent residual judgment selects ai_proposal_binding；下一步: BE-001IZ-01 backend.strategy_config.ai_proposal_binding baseline_plan。
- BE-001IZ-01 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-op route pocket baseline and plan；下一步: BE-001IZ-02 backend.strategy_config.ai_proposal_binding extract_closeout。
- BE-001IZ-02 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-code extraction closeout complete；下一步: BE-001JA-01 backend.strategy_config.ai_proposal_binding single_leaf_closeout。
- BE-001JA-01 `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding single leaf closeout stops further split；下一步: BE-001JB-01 backend.strategy_config parent_residual_judgment。
- BE-001JB-01 `backend.strategy_config` backend.strategy_config parent closeout keeps route aggregation facade；下一步: BE-001JC-01 backend parent_residual_judgment。
- BE-001JC-01 `backend` backend parent residual judgment selects storage_security safety baseline；下一步: BE-001JD-01 backend.storage_security baseline_plan。
- BE-001JD-01 `backend.storage_security` backend.storage_security safety equivalence baseline and extraction plan；下一步: BE-001JD-02 backend.storage_security extract_closeout。
- BE-001JD-02 `backend.storage_security` backend.storage_security facade extraction closeout keeps sensitive semantics paused；下一步: BE-001JE-01 backend.storage_security single_leaf_closeout。
- BE-001JE-01 `backend.storage_security` backend.storage_security single leaf closeout keeps stop_split false；下一步: BE-001JF-01 backend.storage_security parent_residual_judgment。
- BE-001JF-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_api；下一步: BE-001JG-01 backend.storage_security.credential_api baseline_plan。
- BE-001JG-01 `backend.storage_security.credential_api` backend.storage_security.credential_api route facade baseline and plan；下一步: BE-001JG-02 backend.storage_security.credential_api extract_closeout。
- BE-001JG-02 `backend.storage_security.credential_api` backend.storage_security.credential_api facade extraction closeout complete；下一步: BE-001JH-01 backend.storage_security.credential_api single_leaf_closeout。
- BE-001JH-01 `backend.storage_security.credential_api` backend.storage_security.credential_api single leaf closeout stops further facade split；下一步: BE-001JI-01 backend.storage_security parent_residual_judgment。
- BE-001JI-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault；下一步: BE-001JJ-01 backend.storage_security.credential_vault baseline_plan。
- BE-001JJ-01 `backend.storage_security.credential_vault` backend.storage_security.credential_vault re-export facade baseline and plan；下一步: BE-001JJ-02 backend.storage_security.credential_vault extract_closeout。
- BE-001JJ-02 `backend.storage_security.credential_vault` backend.storage_security.credential_vault facade extraction closeout complete；下一步: BE-001JK-01 backend.storage_security.credential_vault single_leaf_closeout。
- BE-001JK-01 `backend.storage_security.credential_vault` backend.storage_security.credential_vault single leaf closeout stops further facade split；下一步: BE-001JL-01 backend.storage_security parent_residual_judgment。
- BE-001JL-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault_implementation；下一步: BE-001JM-01 backend.storage_security.credential_vault_implementation baseline_plan。
- BE-001JM-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation safety baseline and extraction plan；下一步: BE-001JM-02 backend.storage_security.credential_vault_implementation extract_closeout。
- BE-001JM-02 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation actual extraction complete；下一步: BE-001JN-01 backend.storage_security.credential_vault_implementation single_leaf_closeout。
- BE-001JN-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation single leaf closeout keeps stop_split false；下一步: BE-001JO-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001JO-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management；下一步: BE-001JP-01 backend.storage_security.credential_vault_implementation.machine_key_management baseline_plan。
- BE-001JP-01 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan；下一步: BE-001JP-02 backend.storage_security.credential_vault_implementation.machine_key_management extract_closeout。
- BE-001JP-02 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management actual extraction complete；下一步: BE-001JP-03 backend.storage_security.credential_vault_implementation.machine_key_management single_leaf_closeout。
- BE-001JP-03 `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management single leaf closeout stops further split；下一步: BE-001JQ-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001JQ-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec；下一步: BE-001JR-01 backend.storage_security.credential_vault_implementation.crypto_codec baseline_plan。
- BE-001JR-01 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan；下一步: BE-001JR-02 backend.storage_security.credential_vault_implementation.crypto_codec extract_closeout。
- BE-001JR-02 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec actual extraction complete；下一步: BE-001JR-03 backend.storage_security.credential_vault_implementation.crypto_codec single_leaf_closeout。
- BE-001JR-03 `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec single leaf closeout stops further split；下一步: BE-001JS-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001JS-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects vault_persistence_restore；下一步: BE-001JT-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore baseline_plan。
- BE-001JT-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan；下一步: BE-001JT-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore extract_closeout。
- BE-001JT-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore actual extraction complete；下一步: BE-001JT-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore single_leaf_closeout。
- BE-001JT-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore single leaf closeout keeps stop_split false；下一步: BE-001JU-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_residual_judgment。
- BE-001JU-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects load_restore_entry；下一步: BE-001JV-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry baseline_plan。
- BE-001JV-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan；下一步: BE-001JV-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry extract_closeout。
- BE-001JV-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry actual extraction complete；下一步: BE-001JV-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single_leaf_closeout。
- BE-001JV-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single leaf closeout stops further split；下一步: BE-001JW-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_residual_judgment。
- BE-001JW-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects atomic_save_commit；下一步: BE-001JX-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit baseline_plan。
- BE-001JX-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan；下一步: BE-001JX-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit extract_closeout。
- BE-001JX-02 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit actual extraction complete；下一步: BE-001JX-03 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single_leaf_closeout。
- BE-001JX-03 `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single leaf closeout stops further split；下一步: BE-001JY-01 backend.storage_security.credential_vault_implementation.vault_persistence_restore parent_closeout。
- BE-001JY-01 `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent closeout stops persistence split；下一步: BE-001JZ-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001JZ-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects service_crud；下一步: BE-001KA-01 backend.storage_security.credential_vault_implementation.service_crud baseline_plan。
- BE-001KA-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan；下一步: BE-001KA-02 backend.storage_security.credential_vault_implementation.service_crud extract_closeout。
- BE-001KA-02 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud actual extraction complete；下一步: BE-001KA-03 backend.storage_security.credential_vault_implementation.service_crud single_leaf_closeout。
- BE-001KA-03 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud single leaf closeout keeps stop_split false；下一步: BE-001KB-01 backend.storage_security.credential_vault_implementation.service_crud parent_residual_judgment。
- BE-001KB-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_mutation_commit；下一步: BE-001KC-01 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit baseline_plan。
- BE-001KC-01 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan；下一步: BE-001KC-02 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit extract_closeout。
- BE-001KC-02 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit actual extraction complete；下一步: BE-001KC-03 backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single_leaf_closeout。
- BE-001KC-03 `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single leaf closeout stops further split；下一步: BE-001KD-01 backend.storage_security.credential_vault_implementation.service_crud parent_residual_judgment。
- BE-001KD-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection；下一步: BE-001KE-01 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection baseline_plan。
- BE-001KE-01 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan；下一步: BE-001KE-02 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection extract_closeout。
- BE-001KE-02 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection actual extraction complete；下一步: BE-001KE-03 backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single_leaf_closeout。
- BE-001KE-03 `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split；下一步: BE-001KF-01 backend.storage_security.credential_vault_implementation.service_crud parent_closeout。
- BE-001KF-01 `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent closeout stops CRUD split；下一步: BE-001KG-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001KG-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects secret_pattern_extraction；下一步: BE-001KH-01 backend.storage_security.credential_vault_implementation.secret_pattern_extraction baseline_plan。
- BE-001KH-01 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan；下一步: BE-001KH-02 backend.storage_security.credential_vault_implementation.secret_pattern_extraction extract_closeout。
- BE-001KH-02 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction actual extraction complete；下一步: BE-001KH-03 backend.storage_security.credential_vault_implementation.secret_pattern_extraction single_leaf_closeout。
- BE-001KH-03 `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction single leaf closeout stops further split；下一步: BE-001KI-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001KI-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects type_surface；下一步: BE-001KJ-01 backend.storage_security.credential_vault_implementation.type_surface baseline_plan。
- BE-001KJ-01 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan；下一步: BE-001KJ-02 backend.storage_security.credential_vault_implementation.type_surface extract_closeout。
- BE-001KJ-02 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface actual extraction complete；下一步: BE-001KJ-03 backend.storage_security.credential_vault_implementation.type_surface single_leaf_closeout。
- BE-001KJ-03 `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface single leaf closeout stops further split；下一步: BE-001KK-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001KK-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects implementation_test_harness；下一步: BE-001KL-01 backend.storage_security.credential_vault_implementation.implementation_test_harness baseline_plan。
- BE-001KL-01 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan；下一步: BE-001KL-02 backend.storage_security.credential_vault_implementation.implementation_test_harness extract_closeout。
- BE-001KL-02 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness actual extraction complete；下一步: BE-001KL-03 backend.storage_security.credential_vault_implementation.implementation_test_harness single_leaf_closeout。
- BE-001KL-03 `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness single leaf closeout stops further split；下一步: BE-001KM-01 backend.storage_security.credential_vault_implementation parent_residual_judgment。
- BE-001KM-01 `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment closes implementation parent；下一步: BE-001KN-01 backend.storage_security parent_residual_judgment。
- BE-001KN-01 `backend.storage_security` backend.storage_security parent residual judgment selects credential_api_handler_implementation；下一步: BE-001KO-01 backend.storage_security.credential_api_handler_implementation baseline_plan。
- BE-001KO-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan；下一步: BE-001KO-02 backend.storage_security.credential_api_handler_implementation extract_closeout。
- BE-001KO-02 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation actual extraction complete；下一步: BE-001KO-03 backend.storage_security.credential_api_handler_implementation single_leaf_closeout。
- BE-001KO-03 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation single leaf closeout continues split；下一步: BE-001KP-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- BE-001KP-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection；下一步: BE-001KQ-01 backend.storage_security.credential_api_handler_implementation.list_projection baseline_plan。
- BE-001KQ-01 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan；下一步: BE-001KQ-02 backend.storage_security.credential_api_handler_implementation.list_projection extract_closeout。
- BE-001KQ-02 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete；下一步: BE-001KQ-03 backend.storage_security.credential_api_handler_implementation.list_projection single_leaf_closeout。
- BE-001KQ-03 `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection single leaf closeout stops further split；下一步: BE-001KR-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- BE-001KR-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope；下一步: BE-001KS-01 backend.storage_security.credential_api_handler_implementation.key_scope baseline_plan。
- BE-001KS-01 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan；下一步: BE-001KS-02 backend.storage_security.credential_api_handler_implementation.key_scope extract_closeout。
- BE-001KS-02 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete；下一步: BE-001KS-03 backend.storage_security.credential_api_handler_implementation.key_scope single_leaf_closeout。
- BE-001KS-03 `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope single leaf closeout stops further split；下一步: BE-001KT-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- BE-001KT-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation；下一步: BE-001KU-01 backend.storage_security.credential_api_handler_implementation.set_mutation baseline_plan。
- BE-001KU-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan；下一步: BE-001KU-02 backend.storage_security.credential_api_handler_implementation.set_mutation extract_closeout。
- BE-001KU-02 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete；下一步: BE-001KU-03 backend.storage_security.credential_api_handler_implementation.set_mutation single_leaf_closeout。
- BE-001KU-03 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation single leaf closeout continues split；下一步: BE-001KV-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- BE-001KV-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation；下一步: BE-001KW-01 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation baseline_plan。
- BE-001KW-01 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan；下一步: BE-001KW-02 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation extract_closeout。
- BE-001KW-02 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete；下一步: BE-001KW-03 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single_leaf_closeout。
- BE-001KW-03 `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single leaf closeout stops further split；下一步: BE-001KX-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- BE-001KX-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects storage_commit；下一步: BE-001KY-01 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit baseline_plan。
- BE-001KY-01 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan；下一步: BE-001KY-02 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit extract_closeout。
- BE-001KY-02 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete；下一步: BE-001KY-03 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single_leaf_closeout。
- BE-001KY-03 `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single leaf closeout stops further split；下一步: BE-001KZ-01 backend.storage_security.credential_api_handler_implementation.set_mutation parent_residual_judgment。
- BE-001KZ-01 `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment closes parent；下一步: BE-001LA-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- BE-001LA-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation；下一步: BE-001LB-01 backend.storage_security.credential_api_handler_implementation.delete_mutation baseline_plan。
- BE-001LB-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan；下一步: BE-001LB-02 backend.storage_security.credential_api_handler_implementation.delete_mutation extract_closeout。
- BE-001LB-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete；下一步: BE-001LB-03 backend.storage_security.credential_api_handler_implementation.delete_mutation single_leaf_closeout。
- BE-001LB-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation single leaf closeout continues split；下一步: BE-001LC-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- BE-001LC-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation；下一步: BE-001LD-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation baseline_plan。
- BE-001LD-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan；下一步: BE-001LD-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation extract_closeout。
- BE-001LD-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete；下一步: BE-001LD-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single_leaf_closeout。
- BE-001LD-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single leaf closeout stops further split；下一步: BE-001LE-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- BE-001LE-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects delete_commit；下一步: BE-001LF-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit baseline_plan。
- BE-001LF-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan；下一步: BE-001LF-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit extract_closeout。
- BE-001LF-02 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete；下一步: BE-001LF-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single_leaf_closeout。
- BE-001LF-03 `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single leaf closeout stops further split；下一步: BE-001LG-01 backend.storage_security.credential_api_handler_implementation.delete_mutation parent_residual_judgment。
- BE-001LG-01 `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment closes parent；下一步: BE-001LH-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment。
- BE-001LH-01 `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment closes parent；下一步: BE-001LI-01 backend.storage_security parent_residual_judgment。
- BE-001LI-01 `backend.storage_security` backend.storage_security parent residual judgment closes parent；下一步: BE-001LJ-01 backend parent_residual_judgment。
- BE-001LJ-01 `backend` backend parent residual judgment selects ops_governance；下一步: BE-001LK-01 backend.ops_governance baseline_plan。
- BE-001LK-01 `backend.ops_governance` backend.ops_governance equivalence baseline and extraction plan；下一步: BE-001LK-02 backend.ops_governance extract_closeout。
- BE-001LK-02 `backend.ops_governance` backend.ops_governance facade extraction closeout；下一步: BE-001LK-03 backend.ops_governance single_leaf_closeout。
- BE-001LK-03 `backend.ops_governance` backend.ops_governance single leaf closeout continues split；下一步: BE-001LL-01 backend.ops_governance parent_residual_judgment。
- BE-001LL-01 `backend.ops_governance` backend.ops_governance parent residual judgment selects hotswap；下一步: BE-001LM-01 backend.ops_governance.hotswap baseline_plan。
- BE-001LM-01 `backend.ops_governance.hotswap` backend.ops_governance.hotswap equivalence baseline and extraction plan；下一步: BE-001LM-02 backend.ops_governance.hotswap extract_closeout。
- BE-001LM-02 `backend.ops_governance.hotswap` backend.ops_governance.hotswap actual extraction complete；下一步: BE-001LM-03 backend.ops_governance.hotswap single_leaf_closeout。
- BE-001LM-03 `backend.ops_governance.hotswap` backend.ops_governance.hotswap single leaf closeout stops further split；下一步: BE-001LN-01 backend.ops_governance parent_residual_judgment。
- BE-001LN-01 `backend.ops_governance` backend.ops_governance parent residual judgment selects sandbox；下一步: BE-001LO-01 backend.ops_governance.sandbox baseline_plan。
- BE-001LO-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox equivalence baseline and extraction plan；下一步: BE-001LO-02 backend.ops_governance.sandbox extract_closeout。
- BE-001LO-02 `backend.ops_governance.sandbox` backend.ops_governance.sandbox actual extraction complete；下一步: BE-001LO-03 backend.ops_governance.sandbox single_leaf_closeout。
- BE-001LO-03 `backend.ops_governance.sandbox` backend.ops_governance.sandbox single leaf closeout continues split；下一步: BE-001LP-01 backend.ops_governance.sandbox parent_residual_judgment。
- BE-001LP-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects report_api；下一步: BE-001LQ-01 backend.ops_governance.sandbox.report_api baseline_plan。
- BE-001LQ-01 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan；下一步: BE-001LQ-02 backend.ops_governance.sandbox.report_api extract_closeout。
- BE-001LQ-02 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api actual extraction complete；下一步: BE-001LQ-03 backend.ops_governance.sandbox.report_api single_leaf_closeout。
- BE-001LQ-03 `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api single leaf closeout stops further split；下一步: BE-001LR-01 backend.ops_governance.sandbox parent_residual_judgment。
- BE-001LR-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects verification_run；下一步: BE-001LS-01 backend.ops_governance.sandbox.verification_run baseline_plan。
- BE-001LS-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan；下一步: BE-001LS-02 backend.ops_governance.sandbox.verification_run extract_closeout。
- BE-001LS-02 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run actual extraction complete；下一步: BE-001LS-03 backend.ops_governance.sandbox.verification_run single_leaf_closeout。
- BE-001LS-03 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run single leaf closeout continues split；下一步: BE-001LT-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- BE-001LT-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit；下一步: BE-001LU-01 backend.ops_governance.sandbox.verification_run.report_commit baseline_plan。
- BE-001LU-01 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan；下一步: BE-001LU-02 backend.ops_governance.sandbox.verification_run.report_commit extract_closeout。
- BE-001LU-02 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete；下一步: BE-001LU-03 backend.ops_governance.sandbox.verification_run.report_commit single_leaf_closeout。
- BE-001LU-03 `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit single leaf closeout stops further split；下一步: BE-001LV-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- BE-001LV-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate；下一步: BE-001LW-01 backend.ops_governance.sandbox.verification_run.proposal_gate baseline_plan。
- BE-001LW-01 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan；下一步: BE-001LW-02 backend.ops_governance.sandbox.verification_run.proposal_gate extract_closeout。
- BE-001LW-02 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete；下一步: BE-001LW-03 backend.ops_governance.sandbox.verification_run.proposal_gate single_leaf_closeout。
- BE-001LW-03 `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate single leaf closeout stops further split；下一步: BE-001LX-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- BE-001LX-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window；下一步: BE-001LY-01 backend.ops_governance.sandbox.verification_run.replay_window baseline_plan。
- BE-001LY-01 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan；下一步: BE-001LY-02 backend.ops_governance.sandbox.verification_run.replay_window extract_closeout。
- BE-001LY-02 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete；下一步: BE-001LY-03 backend.ops_governance.sandbox.verification_run.replay_window single_leaf_closeout。
- BE-001LY-03 `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout stops further split；下一步: BE-001LZ-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- BE-001LZ-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly；下一步: BE-001MA-01 backend.ops_governance.sandbox.verification_run.report_assembly baseline_plan。
- BE-001MA-01 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan；下一步: BE-001MA-02 backend.ops_governance.sandbox.verification_run.report_assembly extract_closeout。
- BE-001MA-02 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete；下一步: BE-001MA-03 backend.ops_governance.sandbox.verification_run.report_assembly single_leaf_closeout。
- BE-001MA-03 `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout stops further split；下一步: BE-001MB-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment。
- BE-001MB-01 `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment closes parent；下一步: BE-001MC-01 backend.ops_governance.sandbox parent_residual_judgment。
- BE-001MC-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation；下一步: BE-001MD-01 backend.ops_governance.sandbox.metrics_evaluation baseline_plan。
- BE-001MD-01 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan；下一步: BE-001MD-02 backend.ops_governance.sandbox.metrics_evaluation extract_closeout。
- BE-001MD-02 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation actual extraction complete；下一步: BE-001MD-03 backend.ops_governance.sandbox.metrics_evaluation single_leaf_closeout。
- BE-001MD-03 `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation single leaf closeout stops further split；下一步: BE-001ME-01 backend.ops_governance.sandbox parent_residual_judgment。
- BE-001ME-01 `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects comparison_metrics；下一步: BE-001MF-01 backend.ops_governance.sandbox.comparison_metrics baseline_plan。
- BE-001MF-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan；下一步: BE-001MF-02 backend.ops_governance.sandbox.comparison_metrics extract_closeout。
- BE-001MF-02 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics actual extraction complete；下一步: BE-001MF-03 backend.ops_governance.sandbox.comparison_metrics single_leaf_closeout。
- BE-001MF-03 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split；下一步: BE-001MG-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- BE-001MG-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape；下一步: BE-001MH-01 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape baseline_plan。
- BE-001MH-01 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan；下一步: BE-001MH-02 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape extract_closeout。
- BE-001MH-02 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete；下一步: BE-001MH-03 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single_leaf_closeout。
- BE-001MH-03 `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout stops further split；下一步: BE-001MI-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- BE-001MI-01 `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection；下一步: BE-001MJ-01 backend.ops_governance.sandbox.comparison_metrics.backtest_projection baseline_plan。
- BE-001MJ-01 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan；下一步: BE-001MJ-02 backend.ops_governance.sandbox.comparison_metrics.backtest_projection extract_closeout。
- BE-001MJ-02 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete；下一步: BE-001MJ-03 backend.ops_governance.sandbox.comparison_metrics.backtest_projection single_leaf_closeout。
- BE-001MJ-03 `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout stops further split；下一步: BE-001MK-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment。
- BE-001MK-01 `backend.ops_governance.sandbox.comparison_metrics` parent residual judgment closes parent; next step: BE-001ML-01 backend.ops_governance.sandbox parent_residual_judgment.
- BE-001ML-01 `backend.ops_governance.sandbox` parent residual judgment selects proposal_loader; next step: BE-001MM-01 backend.ops_governance.sandbox.proposal_loader baseline_plan.
- BE-001MM-01 `backend.ops_governance.sandbox.proposal_loader` equivalence baseline and extraction plan; next step: BE-001MM-02 backend.ops_governance.sandbox.proposal_loader extract_closeout.
- BE-001MM-02 `backend.ops_governance.sandbox.proposal_loader` actual extraction complete; next step: BE-001MM-03 backend.ops_governance.sandbox.proposal_loader single_leaf_closeout.
- BE-001MM-03 `backend.ops_governance.sandbox.proposal_loader` single leaf closeout stops further split; next step: BE-001MN-01 backend.ops_governance.sandbox parent_residual_judgment.
- BE-001MN-01 `backend.ops_governance.sandbox` parent residual judgment selects report_disk_loader; next step: BE-001MO-01 backend.ops_governance.sandbox.report_disk_loader baseline_plan.
- BE-001MO-01 `backend.ops_governance.sandbox.report_disk_loader` equivalence baseline and extraction plan; next step: BE-001MO-02 backend.ops_governance.sandbox.report_disk_loader extract_closeout.
- BE-001MO-02 `backend.ops_governance.sandbox.report_disk_loader` actual extraction complete; next step: BE-001MO-03 backend.ops_governance.sandbox.report_disk_loader single_leaf_closeout.
- BE-001MO-03 `backend.ops_governance.sandbox.report_disk_loader` single leaf closeout stops further split; next step: BE-001MP-01 backend.ops_governance.sandbox parent_residual_judgment.
- BE-001MP-01 `backend.ops_governance.sandbox` parent residual judgment closes parent; next step: BE-001MQ-01 backend.ops_governance parent_residual_judgment.
- BE-001MQ-01 `backend.ops_governance` parent residual judgment selects alerts; next step: BE-001MR-01 backend.ops_governance.alerts baseline_plan.
- BE-001MR-01 `backend.ops_governance.alerts` equivalence baseline and extraction plan; next step: BE-001MR-02 backend.ops_governance.alerts extract_closeout.
- BE-001MR-02 `backend.ops_governance.alerts` actual extraction complete; next step: BE-001MR-03 backend.ops_governance.alerts single_leaf_closeout.
- BE-001MR-03 `backend.ops_governance.alerts` single leaf closeout continues split; next step: BE-001MS-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001MS-01 `backend.ops_governance.alerts` parent residual judgment selects rule_catalog; next step: BE-001MT-01 backend.ops_governance.alerts.rule_catalog baseline_plan.
- BE-001MT-01 `backend.ops_governance.alerts.rule_catalog` equivalence baseline and extraction plan; next step: BE-001MT-02 backend.ops_governance.alerts.rule_catalog extract_closeout.
- BE-001MT-02 `backend.ops_governance.alerts.rule_catalog` actual extraction complete; next step: BE-001MT-03 backend.ops_governance.alerts.rule_catalog single_leaf_closeout.
- BE-001MT-03 `backend.ops_governance.alerts.rule_catalog` single leaf closeout stops further split; next step: BE-001MU-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001MU-01 `backend.ops_governance.alerts` parent residual judgment selects acknowledge_flow; next step: BE-001MV-01 backend.ops_governance.alerts.acknowledge_flow baseline_plan.
- BE-001MV-01 `backend.ops_governance.alerts.acknowledge_flow` equivalence baseline and extraction plan; next step: BE-001MV-02 backend.ops_governance.alerts.acknowledge_flow extract_closeout.
- BE-001MV-02 `backend.ops_governance.alerts.acknowledge_flow` actual extraction complete; next step: BE-001MV-03 backend.ops_governance.alerts.acknowledge_flow single_leaf_closeout.
- BE-001MV-03 `backend.ops_governance.alerts.acknowledge_flow` single leaf closeout stops further split; next step: BE-001MW-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001MW-01 `backend.ops_governance.alerts` parent residual judgment selects trigger_engine; next step: BE-001MX-01 backend.ops_governance.alerts.trigger_engine baseline_plan.
- BE-001MX-01 `backend.ops_governance.alerts.trigger_engine` equivalence baseline and extraction plan; next step: BE-001MX-02 backend.ops_governance.alerts.trigger_engine extract_closeout.
- BE-001MX-02 `backend.ops_governance.alerts.trigger_engine` actual extraction complete; next step: BE-001MX-03 backend.ops_governance.alerts.trigger_engine single_leaf_closeout.
- BE-001MX-03 `backend.ops_governance.alerts.trigger_engine` single leaf closeout stops further split; next step: BE-001MY-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001MY-01 `backend.ops_governance.alerts` parent residual judgment selects predicate_checks; next step: BE-001MZ-01 backend.ops_governance.alerts.predicate_checks baseline_plan.
- BE-001MZ-01 `backend.ops_governance.alerts.predicate_checks` equivalence baseline and extraction plan; next step: BE-001MZ-02 backend.ops_governance.alerts.predicate_checks extract_closeout.
- BE-001MZ-02 `backend.ops_governance.alerts.predicate_checks` actual extraction complete; next step: BE-001MZ-03 backend.ops_governance.alerts.predicate_checks single_leaf_closeout.
- BE-001MZ-03 `backend.ops_governance.alerts.predicate_checks` single leaf closeout stops further split; next step: BE-001NA-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001NA-01 `backend.ops_governance.alerts` parent residual judgment selects persistence; next step: BE-001NA-02 backend.ops_governance.alerts.persistence baseline_plan.
- BE-001NA-02 `backend.ops_governance.alerts.persistence` equivalence baseline and extraction plan; next step: BE-001NA-03 backend.ops_governance.alerts.persistence extract_closeout.
- BE-001NA-03 `backend.ops_governance.alerts.persistence` actual extraction complete; next step: BE-001NA-04 backend.ops_governance.alerts.persistence single_leaf_closeout.
- BE-001NA-04 `backend.ops_governance.alerts.persistence` single leaf closeout stops further split; next step: BE-001NB-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001NB-01 `backend.ops_governance.alerts` parent residual judgment selects startup_initialization; next step: BE-001NB-02 backend.ops_governance.alerts.startup_initialization baseline_plan.
- BE-001NB-02 `backend.ops_governance.alerts.startup_initialization` equivalence baseline and extraction plan; next step: BE-001NB-03 backend.ops_governance.alerts.startup_initialization extract_closeout.
- BE-001NB-03 `backend.ops_governance.alerts.startup_initialization` actual extraction complete; next step: BE-001NB-04 backend.ops_governance.alerts.startup_initialization single_leaf_closeout.
- BE-001NB-04 `backend.ops_governance.alerts.startup_initialization` single leaf closeout stops further split; next step: BE-001NC-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001NC-01 `backend.ops_governance.alerts` parent residual judgment selects read_routes; next step: BE-001NC-02 backend.ops_governance.alerts.read_routes baseline_plan.
- BE-001NC-02 `backend.ops_governance.alerts.read_routes` equivalence baseline and extraction plan; next step: BE-001NC-03 backend.ops_governance.alerts.read_routes extract_closeout.
- BE-001NC-03 `backend.ops_governance.alerts.read_routes` actual extraction complete; next step: BE-001NC-04 backend.ops_governance.alerts.read_routes single_leaf_closeout.
- BE-001NC-04 `backend.ops_governance.alerts.read_routes` single leaf closeout stops further split; next step: BE-001ND-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001ND-01 `backend.ops_governance.alerts.route_facade` static closeout and recovery_bridge selection; next step: BE-001NE-01 backend.ops_governance.alerts.recovery_bridge baseline_plan.
- BE-001NE-01 `backend.ops_governance.alerts.recovery_bridge` equivalence baseline and extraction plan; next step: BE-001NE-02 backend.ops_governance.alerts.recovery_bridge extract_closeout.
- BE-001NE-02 `backend.ops_governance.alerts.recovery_bridge` actual extraction complete; next step: BE-001NE-03 backend.ops_governance.alerts.recovery_bridge single_leaf_closeout.
- BE-001NE-03 `backend.ops_governance.alerts.recovery_bridge` single leaf closeout stops further split; next step: BE-001NF-01 backend.ops_governance.alerts parent_residual_judgment.
- BE-001NF-01 `backend.ops_governance.alerts` parent residual judgment closes parent; next step: BE-001NG-01 backend.ops_governance parent_residual_judgment.
- BE-001NG-01 `backend.ops_governance` parent residual judgment selects snapshots; next step: BE-001NH-01 backend.ops_governance.snapshots baseline_plan.
- BE-001NH-01 `backend.ops_governance.snapshots` equivalence baseline and extraction plan; next step: BE-001NH-02 backend.ops_governance.snapshots extract_closeout.
- BE-001NH-02 `backend.ops_governance.snapshots` actual extraction complete; next step: BE-001NH-03 backend.ops_governance.snapshots single_leaf_closeout.
- BE-001NH-03 `backend.ops_governance.snapshots` single leaf closeout continues split; next step: BE-001NI-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NI-01 `backend.ops_governance.snapshots` parent residual judgment selects snapshot_id_validation; next step: BE-001NJ-01 backend.ops_governance.snapshots.snapshot_id_validation baseline_plan.
- BE-001NJ-01 `backend.ops_governance.snapshots.snapshot_id_validation` equivalence baseline and extraction plan; next step: BE-001NJ-02 backend.ops_governance.snapshots.snapshot_id_validation extract_closeout.
- BE-001NJ-02 `backend.ops_governance.snapshots.snapshot_id_validation` actual extraction complete; next step: BE-001NJ-03 backend.ops_governance.snapshots.snapshot_id_validation single_leaf_closeout.
- BE-001NJ-03 `backend.ops_governance.snapshots.snapshot_id_validation` single leaf closeout stops further split; next step: BE-001NK-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NK-01 `backend.ops_governance.snapshots` parent residual judgment selects create_flow; next step: BE-001NL-01 backend.ops_governance.snapshots.create_flow baseline_plan.
- BE-001NL-01 `backend.ops_governance.snapshots.create_flow` equivalence baseline and extraction plan; next step: BE-001NL-02 backend.ops_governance.snapshots.create_flow extract_closeout.
- BE-001NL-02 `backend.ops_governance.snapshots.create_flow` actual extraction complete; next step: BE-001NL-03 backend.ops_governance.snapshots.create_flow single_leaf_closeout.
- BE-001NL-03 `backend.ops_governance.snapshots.create_flow` single leaf closeout stops further split; next step: BE-001NM-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NM-01 `backend.ops_governance.snapshots` parent residual judgment selects read_routes; next step: BE-001NN-01 backend.ops_governance.snapshots.read_routes baseline_plan.
- BE-001NN-01 `backend.ops_governance.snapshots.read_routes` equivalence baseline and extraction plan; next step: BE-001NN-02 backend.ops_governance.snapshots.read_routes extract_closeout.
- BE-001NN-02 `backend.ops_governance.snapshots.read_routes` actual extraction complete; next step: BE-001NN-03 backend.ops_governance.snapshots.read_routes single_leaf_closeout.
- BE-001NN-03 `backend.ops_governance.snapshots.read_routes` single leaf closeout stops further split; next step: BE-001NO-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NO-01 `backend.ops_governance.snapshots` parent residual judgment selects restore_flow; next step: BE-001NP-01 backend.ops_governance.snapshots.restore_flow baseline_plan.
- BE-001NP-01 `backend.ops_governance.snapshots.restore_flow` equivalence baseline and extraction plan; next step: BE-001NP-02 backend.ops_governance.snapshots.restore_flow extract_closeout.
- BE-001NP-02 `backend.ops_governance.snapshots.restore_flow` actual extraction complete; next step: BE-001NP-03 backend.ops_governance.snapshots.restore_flow single_leaf_closeout.
- BE-001NP-03 `backend.ops_governance.snapshots.restore_flow` single leaf closeout stops further split; next step: BE-001NQ-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NQ-01 `backend.ops_governance.snapshots` parent residual judgment selects persistence; next step: BE-001NR-01 backend.ops_governance.snapshots.persistence baseline_plan.
- BE-001NR-01 `backend.ops_governance.snapshots.persistence` equivalence baseline and extraction plan; next step: BE-001NR-02 backend.ops_governance.snapshots.persistence extract_closeout.
- BE-001NR-02 `backend.ops_governance.snapshots.persistence` actual extraction complete; next step: BE-001NR-03 backend.ops_governance.snapshots.persistence single_leaf_closeout.
- BE-001NR-03 `backend.ops_governance.snapshots.persistence` single leaf closeout stops further split; next step: BE-001NS-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NS-01 `backend.ops_governance.snapshots` parent residual judgment selects signature_contract; next step: BE-001NT-01 backend.ops_governance.snapshots.signature_contract baseline_plan.
- BE-001NT-01 `backend.ops_governance.snapshots.signature_contract` equivalence baseline and extraction plan; next step: BE-001NT-02 backend.ops_governance.snapshots.signature_contract extract_closeout.
- BE-001NT-02 `backend.ops_governance.snapshots.signature_contract` actual extraction complete; next step: BE-001NT-03 backend.ops_governance.snapshots.signature_contract single_leaf_closeout.
- BE-001NT-03 `backend.ops_governance.snapshots.signature_contract` single leaf closeout stops further split; next step: BE-001NU-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NU-01 `backend.ops_governance.snapshots.route_facade` static closeout and parent closeout selection; next step: BE-001NV-01 backend.ops_governance.snapshots parent_residual_judgment.
- BE-001NV-01 `backend.ops_governance.snapshots` parent residual judgment closes parent; next step: BE-001NW-01 backend.ops_governance parent_residual_judgment.
- BE-001NW-01 `backend.ops_governance` parent residual judgment selects runbook; next step: BE-001NX-01 backend.ops_governance.runbook baseline_plan.
- BE-001NX-01 `backend.ops_governance.runbook` equivalence baseline and extraction plan; next step: BE-001NX-02 backend.ops_governance.runbook extract_closeout.
- BE-001NX-02 `backend.ops_governance.runbook` actual extraction complete; next step: BE-001NX-03 backend.ops_governance.runbook single_leaf_closeout.
- BE-001NX-03 `backend.ops_governance.runbook` single leaf closeout continues split; next step: BE-001NY-01 backend.ops_governance.runbook parent_residual_judgment.
- BE-001NY-01 `backend.ops_governance.runbook` parent residual judgment selects scenario_catalog; next step: BE-001NZ-01 backend.ops_governance.runbook.scenario_catalog baseline_plan.
- BE-001NZ-01 `backend.ops_governance.runbook.scenario_catalog` equivalence baseline and extraction plan; next step: BE-001NZ-02 backend.ops_governance.runbook.scenario_catalog extract_closeout.
- BE-001NZ-02 `backend.ops_governance.runbook.scenario_catalog` actual extraction complete; next step: BE-001NZ-03 backend.ops_governance.runbook.scenario_catalog single_leaf_closeout.
- BE-001NZ-03 `backend.ops_governance.runbook.scenario_catalog` single leaf closeout; next step: BE-001OA-01 backend.ops_governance.runbook parent_residual_judgment.
- BE-001OA-01 `backend.ops_governance.runbook` parent residual judgment selects read_routes; next step: BE-001OB-01 backend.ops_governance.runbook.read_routes baseline_plan.
- BE-001OB-01 `backend.ops_governance.runbook.read_routes` equivalence baseline and extraction plan; next step: BE-001OB-02 backend.ops_governance.runbook.read_routes extract_closeout.
- BE-001OB-02 `backend.ops_governance.runbook.read_routes` actual extraction complete; next step: BE-001OB-03 backend.ops_governance.runbook.read_routes single_leaf_closeout.
- BE-001OB-03 `backend.ops_governance.runbook.read_routes` single leaf closeout; next step: BE-001OC-01 backend.ops_governance.runbook parent_residual_judgment.
- BE-001OC-01 `backend.ops_governance.runbook` parent residual judgment selects route_facade; next step: BE-001OD-01 backend.ops_governance.runbook.route_facade baseline_plan.
- BE-001OD-01 `backend.ops_governance.runbook.route_facade` equivalence baseline and extraction plan; next step: BE-001OD-02 backend.ops_governance.runbook.route_facade extract_closeout.
- BE-001OD-02 `backend.ops_governance.runbook.route_facade` actual extraction complete; next step: BE-001OD-03 backend.ops_governance.runbook.route_facade single_leaf_closeout.
- BE-001OD-03 `backend.ops_governance.runbook.route_facade` single leaf closeout; next step: BE-001OE-01 backend.ops_governance.runbook parent_closeout.
- BE-001OE-01 `backend.ops_governance.runbook` parent closeout; next step: BE-001OF-01 backend.ops_governance parent_residual_judgment.
- BE-001OF-01 `backend.ops_governance` parent residual judgment selects chaos; next step: BE-001OG-01 backend.ops_governance.chaos baseline_plan.
- BE-001OG-01 `backend.ops_governance.chaos` equivalence baseline and extraction plan; next step: BE-001OG-02 backend.ops_governance.chaos extract_closeout.
- BE-001OG-02 `backend.ops_governance.chaos` actual extraction complete; next step: BE-001OG-03 backend.ops_governance.chaos single_leaf_closeout.
- BE-001OG-03 `backend.ops_governance.chaos` single leaf closeout continues split; next step: BE-001OH-01 backend.ops_governance.chaos parent_residual_judgment.
- BE-001OH-01 `backend.ops_governance.chaos` parent residual judgment selects report_persistence; next step: BE-001OI-01 backend.ops_governance.chaos.report_persistence baseline_plan.
- BE-001OI-01 `backend.ops_governance.chaos.report_persistence` equivalence baseline and extraction plan; next step: BE-001OI-02 backend.ops_governance.chaos.report_persistence extract_closeout.
- BE-001OI-02 `backend.ops_governance.chaos.report_persistence` actual extraction complete; next step: BE-001OI-03 backend.ops_governance.chaos.report_persistence single_leaf_closeout.
- BE-001OI-03 `backend.ops_governance.chaos.report_persistence` single leaf closeout; next step: BE-001OJ-01 backend.ops_governance.chaos parent_residual_judgment.
- BE-001OJ-01 `backend.ops_governance.chaos` parent residual judgment selects experiment_creation; next step: BE-001OK-01 backend.ops_governance.chaos.experiment_creation baseline_plan.
- BE-001OK-01 `backend.ops_governance.chaos.experiment_creation` equivalence baseline and extraction plan; next step: BE-001OK-02 backend.ops_governance.chaos.experiment_creation extract_closeout.
- BE-001OK-02 `backend.ops_governance.chaos.experiment_creation` actual extraction complete; next step: BE-001OK-03 backend.ops_governance.chaos.experiment_creation single_leaf_closeout.
- BE-001OK-03 `backend.ops_governance.chaos.experiment_creation` single leaf closeout continues split; next step: BE-001OL-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- BE-001OL-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects perturbation_execution; next step: BE-001OM-01 backend.ops_governance.chaos.experiment_creation.perturbation_execution baseline_plan.
- BE-001OM-01 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` equivalence baseline and extraction plan; next step: BE-001OM-02 backend.ops_governance.chaos.experiment_creation.perturbation_execution extract_closeout.
- BE-001OM-02 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` actual extraction complete; next step: BE-001OM-03 backend.ops_governance.chaos.experiment_creation.perturbation_execution single_leaf_closeout.
- BE-001OM-03 `backend.ops_governance.chaos.experiment_creation.perturbation_execution` single leaf closeout; next step: BE-001ON-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- BE-001ON-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects report_projection; next step: BE-001OO-01 backend.ops_governance.chaos.experiment_creation.report_projection baseline_plan.
- BE-001OO-01 `backend.ops_governance.chaos.experiment_creation.report_projection` equivalence baseline and extraction plan; next step: BE-001OO-02 backend.ops_governance.chaos.experiment_creation.report_projection extract_closeout.
- BE-001OO-02 `backend.ops_governance.chaos.experiment_creation.report_projection` actual extraction complete; next step: BE-001OO-03 backend.ops_governance.chaos.experiment_creation.report_projection single_leaf_closeout.
- BE-001OO-03 `backend.ops_governance.chaos.experiment_creation.report_projection` single leaf closeout; next step: BE-001OP-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment.
- BE-001OP-01 `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects memory_commit; next step: BE-001OQ-01 backend.ops_governance.chaos.experiment_creation.memory_commit baseline_plan.
- BE-001OQ-01 `backend.ops_governance.chaos.experiment_creation.memory_commit` equivalence baseline and extraction plan; next step: BE-001OQ-02 backend.ops_governance.chaos.experiment_creation.memory_commit extract_closeout.
- BE-001OQ-02 `backend.ops_governance.chaos.experiment_creation.memory_commit` actual extraction complete; next step: BE-001OQ-03 backend.ops_governance.chaos.experiment_creation.memory_commit single_leaf_closeout.
- BE-001OQ-03 `backend.ops_governance.chaos.experiment_creation.memory_commit` single leaf closeout; next step: BE-001OR-01 backend.ops_governance.chaos.experiment_creation parent_closeout.
- BE-001OR-01 `backend.ops_governance.chaos.experiment_creation` parent closeout; next step: BE-001OS-01 backend.ops_governance.chaos parent_residual_judgment.
- BE-001OS-01 `backend.ops_governance.chaos` parent residual judgment selects read_routes; next step: BE-001OT-01 backend.ops_governance.chaos.read_routes baseline_plan.
- BE-001OT-01 `backend.ops_governance.chaos.read_routes` equivalence baseline and extraction plan; next step: BE-001OT-02 backend.ops_governance.chaos.read_routes extract_closeout.
- BE-001OT-02 `backend.ops_governance.chaos.read_routes` actual extraction complete; next step: BE-001OT-03 backend.ops_governance.chaos.read_routes single_leaf_closeout.
- BE-001OT-03 `backend.ops_governance.chaos.read_routes` single leaf closeout; next step: BE-001OU-01 backend.ops_governance.chaos parent_residual_judgment.
- BE-001OU-01 `backend.ops_governance.chaos` parent residual judgment selects route_facade; next step: BE-001OV-01 backend.ops_governance.chaos.route_facade baseline_plan.
- BE-001OV-01 `backend.ops_governance.chaos.route_facade` equivalence baseline and extraction plan; next step: BE-001OV-02 backend.ops_governance.chaos.route_facade extract_closeout.
- BE-001OV-02 `backend.ops_governance.chaos.route_facade` actual extraction complete; next step: BE-001OV-03 backend.ops_governance.chaos.route_facade single_leaf_closeout.
- BE-001OV-03 `backend.ops_governance.chaos.route_facade` single leaf closeout; next step: BE-001OW-01 backend.ops_governance.chaos parent_closeout.
- BE-001OW-01 `backend.ops_governance.chaos` parent closeout; next step: BE-001OX-01 backend.ops_governance parent_closeout.
- BE-001OX-01 `backend.ops_governance` parent closeout; next step: BE-001OY-01 backend parent_residual_judgment selects backend.app_state_wiring.
- BE-001OY-01 `backend` parent residual judgment selects `backend.app_state_wiring`; next step: BE-001OZ-01 backend.app_state_wiring single_leaf_closeout.
- BE-001OZ-01 `backend.app_state_wiring` single leaf closeout sets `stop_split: true`; next step: BE-001PA-01 backend parent_residual_judgment selects backend.test_support.
- BE-001PA-01 `backend` parent residual judgment selects `backend.test_support`; next step: BE-001PB-01 backend.test_support single_leaf_closeout.
- BE-001PB-01 `backend.test_support` single leaf closeout sets `stop_split: true`; next step: BE-001PC-01 backend parent_closeout.
- BE-001PC-01 `backend` parent closeout; next step: BE-001PD-01 root parent_residual_judgment selects root.contracts.
- BE-001PD-01 `root` parent residual judgment selects `root.contracts`; next step: BE-001PE-01 root.contracts baseline_plan.
- BE-001PE-01 `root.contracts` baseline plan freezes the contracts parent and queues `contracts.api_surface`, `contracts.qrpc_core`, `contracts.core_ir`, `contracts.compiler_bridge`, `contracts.runtime_support`, `contracts.quantscript`, and `contracts.plugin_metadata`; next step: BE-001PF-01 root.contracts parent_residual_judgment selects contracts.api_surface.
- BE-001PF-01 `root.contracts` parent residual judgment selects `contracts.api_surface`; next step: BE-001PG-01 root.contracts.api_surface single_leaf_closeout.
- BE-001PG-01 `root.contracts.api_surface` single leaf closeout sets `stop_split: false`; next step: BE-001PH-01 root.contracts.api_surface parent_residual_judgment selects contracts.api_surface.openapi_http.
- BE-001PH-01 `root.contracts.api_surface` parent residual judgment selects `contracts.api_surface.openapi_http`; next step: BE-001PI-01 root.contracts.api_surface.openapi_http single_leaf_closeout.
- BE-001PI-01 `root.contracts.api_surface.openapi_http` single leaf closeout sets `stop_split: true`; next step: BE-001PJ-01 root.contracts.api_surface parent_residual_judgment selects contracts.api_surface.asyncapi_runtime_events.
- BE-001PJ-01 `root.contracts.api_surface` parent residual judgment selects `contracts.api_surface.asyncapi_runtime_events`; next step: BE-001PK-01 root.contracts.api_surface.asyncapi_runtime_events single_leaf_closeout.
- BE-001PK-01 `root.contracts.api_surface.asyncapi_runtime_events` single leaf closeout sets `stop_split: true`; next step: BE-001PL-01 root.contracts.api_surface parent_closeout.
- BE-001PL-01 `root.contracts.api_surface` parent closeout; next step: BE-001PM-01 root.contracts parent_residual_judgment selects contracts.qrpc_core.
- BE-001PM-01 `root.contracts` parent residual judgment selects `contracts.qrpc_core`; next step: BE-001PN-01 root.contracts.qrpc_core baseline_plan.
- BE-001PN-01 `root.contracts.qrpc_core` baseline plan freezes qrpc_core and queues error_contract, event_envelope_proto, plugin_contract, strategy_ir, protocol_primitives, runtime_protocol_config, artifact_specs, runtime_io_contract, and rfc_execution_contracts; next step: BE-001PO-01 root.contracts.qrpc_core parent_residual_judgment selects error_contract.
- BE-001PO-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.error_contract`; next step: BE-001PP-01 root.contracts.qrpc_core.error_contract single_leaf_closeout.
- BE-001PP-01 `root.contracts.qrpc_core.error_contract` single leaf closeout sets `stop_split: true`; next step: BE-001PQ-01 root.contracts.qrpc_core parent_residual_judgment selects event_envelope_proto.
- BE-001PQ-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.event_envelope_proto`; next step: BE-001PR-01 root.contracts.qrpc_core.event_envelope_proto single_leaf_closeout.
- BE-001PR-01 `root.contracts.qrpc_core.event_envelope_proto` single leaf closeout sets `stop_split: true`; next step: BE-001PS-01 root.contracts.qrpc_core parent_residual_judgment selects plugin_contract.
- BE-001PS-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.plugin_contract`; next step: BE-001PT-01 root.contracts.qrpc_core.plugin_contract baseline_plan.
- BE-001PT-01 `root.contracts.qrpc_core.plugin_contract` baseline frozen; children are taxonomy_extension, capability_contract, execution_security_dependency, manifest_validation, and registry. Next step: BE-001PU-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects taxonomy_extension.
- BE-001PU-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `contracts.qrpc_core.plugin_contract.taxonomy_extension`; next step: BE-001PV-01 root.contracts.qrpc_core.plugin_contract.taxonomy_extension baseline_plan.
- BE-001PV-01 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` equivalence baseline and extraction plan; next step: BE-001PV-02 root.contracts.qrpc_core.plugin_contract.taxonomy_extension extract_closeout.
- BE-001PV-02 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` actual extraction complete; next step: BE-001PV-03 root.contracts.qrpc_core.plugin_contract.taxonomy_extension single_leaf_closeout.
- BE-001PV-03 `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` single leaf closeout sets `stop_split: true`; next step: BE-001PW-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects capability_contract.
- BE-001PW-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `contracts.qrpc_core.plugin_contract.capability_contract`; next step: BE-001PX-01 root.contracts.qrpc_core.plugin_contract.capability_contract baseline_plan.
- BE-001PX-01 `root.contracts.qrpc_core.plugin_contract.capability_contract` equivalence baseline and extraction plan; next step: BE-001PX-02 root.contracts.qrpc_core.plugin_contract.capability_contract extract_closeout.
- BE-001PX-02 `root.contracts.qrpc_core.plugin_contract.capability_contract` actual extraction complete; next step: BE-001PX-03 root.contracts.qrpc_core.plugin_contract.capability_contract single_leaf_closeout.
- BE-001PX-03 `root.contracts.qrpc_core.plugin_contract.capability_contract` single leaf closeout sets `stop_split: true`; next step: BE-001PY-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects execution_security_dependency.
- BE-001PY-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `contracts.qrpc_core.plugin_contract.execution_security_dependency`; next step: BE-001PZ-01 root.contracts.qrpc_core.plugin_contract.execution_security_dependency baseline_plan.
- BE-001PZ-01 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` equivalence baseline and extraction plan; next step: BE-001PZ-02 root.contracts.qrpc_core.plugin_contract.execution_security_dependency extract_closeout.
- BE-001PZ-02 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` actual extraction complete; next step: BE-001PZ-03 root.contracts.qrpc_core.plugin_contract.execution_security_dependency single_leaf_closeout.
- BE-001PZ-03 `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` single leaf closeout sets `stop_split: true`; next step: BE-001QA-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects manifest_validation.
- BE-001QA-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `contracts.qrpc_core.plugin_contract.manifest_validation`; next step: BE-001QB-01 root.contracts.qrpc_core.plugin_contract.manifest_validation baseline_plan.
- BE-001QB-01 `root.contracts.qrpc_core.plugin_contract.manifest_validation` equivalence baseline and extraction plan; next step: BE-001QB-02 root.contracts.qrpc_core.plugin_contract.manifest_validation extract_closeout.
- BE-001QB-02 `root.contracts.qrpc_core.plugin_contract.manifest_validation` actual extraction complete; next step: BE-001QB-03 root.contracts.qrpc_core.plugin_contract.manifest_validation single_leaf_closeout.
- BE-001QB-03 `root.contracts.qrpc_core.plugin_contract.manifest_validation` single leaf closeout sets `stop_split: true`; next step: BE-001QC-01 root.contracts.qrpc_core.plugin_contract parent_residual_judgment selects registry.
- BE-001QC-01 `root.contracts.qrpc_core.plugin_contract` parent residual judgment selects `contracts.qrpc_core.plugin_contract.registry`; next step: BE-001QD-01 root.contracts.qrpc_core.plugin_contract.registry baseline_plan.
- BE-001QD-01 `root.contracts.qrpc_core.plugin_contract.registry` equivalence baseline and extraction plan; next step: BE-001QD-02 root.contracts.qrpc_core.plugin_contract.registry extract_closeout.
- BE-001QD-02 `root.contracts.qrpc_core.plugin_contract.registry` actual extraction complete; next step: BE-001QD-03 root.contracts.qrpc_core.plugin_contract.registry single_leaf_closeout.
- BE-001QD-03 `root.contracts.qrpc_core.plugin_contract.registry` single leaf closeout sets `stop_split: true`; next step: BE-001QE-01 root.contracts.qrpc_core.plugin_contract parent_closeout.
- BE-001QE-01 `root.contracts.qrpc_core.plugin_contract` parent closeout; next step: BE-001QF-01 root.contracts.qrpc_core parent_residual_judgment selects strategy_ir.
- BE-001QF-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.strategy_ir`; next step: BE-001QG-01 root.contracts.qrpc_core.strategy_ir baseline_plan.
- BE-001QG-01 `root.contracts.qrpc_core.strategy_ir` baseline frozen; children are version_unknown_error, metadata_source, signal_indicator, logic_position, risk_contract, data_requirement, execution_contract, gap_unknown_annotation, and root_validation. Next step: BE-001QH-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects version_unknown_error.
- BE-001QH-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.version_unknown_error`; next step: BE-001QI-01 root.contracts.qrpc_core.strategy_ir.version_unknown_error baseline_plan.
- BE-001QI-01 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` equivalence baseline and extraction plan; next step: BE-001QI-02 root.contracts.qrpc_core.strategy_ir.version_unknown_error extract_closeout.
- BE-001QI-02 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` actual extraction complete; next step: BE-001QI-03 root.contracts.qrpc_core.strategy_ir.version_unknown_error single_leaf_closeout.
- BE-001QI-03 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` single leaf closeout sets `stop_split: true`; next step: BE-001QJ-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects metadata_source.
- BE-001QJ-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.metadata_source`; next step: BE-001QK-01 root.contracts.qrpc_core.strategy_ir.metadata_source baseline_plan.
- BE-001QK-01 `root.contracts.qrpc_core.strategy_ir.metadata_source` equivalence baseline and extraction plan; next step: BE-001QK-02 root.contracts.qrpc_core.strategy_ir.metadata_source extract_closeout.
- BE-001QK-02 `root.contracts.qrpc_core.strategy_ir.metadata_source` actual extraction complete; next step: BE-001QK-03 root.contracts.qrpc_core.strategy_ir.metadata_source single_leaf_closeout.
- BE-001QK-03 `root.contracts.qrpc_core.strategy_ir.metadata_source` single leaf closeout sets `stop_split: true`; next step: BE-001QL-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects signal_indicator.
- BE-001QL-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.signal_indicator`; next step: BE-001QM-01 root.contracts.qrpc_core.strategy_ir.signal_indicator baseline_plan.
- BE-001QM-01 `root.contracts.qrpc_core.strategy_ir.signal_indicator` equivalence baseline and extraction plan; next step: BE-001QM-02 root.contracts.qrpc_core.strategy_ir.signal_indicator extract_closeout.
- BE-001QM-02 `root.contracts.qrpc_core.strategy_ir.signal_indicator` actual extraction complete; next step: BE-001QM-03 root.contracts.qrpc_core.strategy_ir.signal_indicator single_leaf_closeout.
- BE-001QM-03 `root.contracts.qrpc_core.strategy_ir.signal_indicator` single leaf closeout sets `stop_split: true`; next step: BE-001QN-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects logic_position.
- BE-001QN-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.logic_position`; next step: BE-001QO-01 root.contracts.qrpc_core.strategy_ir.logic_position baseline_plan.
- BE-001QO-01 `root.contracts.qrpc_core.strategy_ir.logic_position` equivalence baseline and extraction plan; next step: BE-001QO-02 root.contracts.qrpc_core.strategy_ir.logic_position extract_closeout.
- BE-001QO-02 `root.contracts.qrpc_core.strategy_ir.logic_position` actual extraction complete; next step: BE-001QO-03 root.contracts.qrpc_core.strategy_ir.logic_position single_leaf_closeout.
- BE-001QO-03 `root.contracts.qrpc_core.strategy_ir.logic_position` single leaf closeout sets `stop_split: true`; next step: BE-001QP-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects risk_contract.
- BE-001QP-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.risk_contract`; next step: BE-001QQ-01 root.contracts.qrpc_core.strategy_ir.risk_contract baseline_plan.
- BE-001QQ-01 `root.contracts.qrpc_core.strategy_ir.risk_contract` equivalence baseline and extraction plan; next step: BE-001QQ-02 root.contracts.qrpc_core.strategy_ir.risk_contract extract_closeout.
- BE-001QQ-02 `root.contracts.qrpc_core.strategy_ir.risk_contract` actual extraction complete; next step: BE-001QQ-03 root.contracts.qrpc_core.strategy_ir.risk_contract single_leaf_closeout.
- BE-001QQ-03 `root.contracts.qrpc_core.strategy_ir.risk_contract` single leaf closeout sets `stop_split: true`; next step: BE-001QR-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects data_requirement.
- BE-001QR-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.data_requirement`; next step: BE-001QS-01 root.contracts.qrpc_core.strategy_ir.data_requirement baseline_plan.
- BE-001QS-01 `root.contracts.qrpc_core.strategy_ir.data_requirement` equivalence baseline and extraction plan; next step: BE-001QS-02 root.contracts.qrpc_core.strategy_ir.data_requirement extract_closeout.
- BE-001QS-02 `root.contracts.qrpc_core.strategy_ir.data_requirement` actual extraction complete; next step: BE-001QS-03 root.contracts.qrpc_core.strategy_ir.data_requirement single_leaf_closeout.
- BE-001QS-03 `root.contracts.qrpc_core.strategy_ir.data_requirement` single leaf closeout sets `stop_split: true`; next step: BE-001QT-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects execution_contract.
- BE-001QT-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.execution_contract`; next step: BE-001QU-01 root.contracts.qrpc_core.strategy_ir.execution_contract baseline_plan.
- BE-001QU-01 `root.contracts.qrpc_core.strategy_ir.execution_contract` equivalence baseline and extraction plan; next step: BE-001QU-02 root.contracts.qrpc_core.strategy_ir.execution_contract extract_closeout.
- BE-001QU-02 `root.contracts.qrpc_core.strategy_ir.execution_contract` actual extraction complete; next step: BE-001QU-03 root.contracts.qrpc_core.strategy_ir.execution_contract single_leaf_closeout.
- BE-001QU-03 `root.contracts.qrpc_core.strategy_ir.execution_contract` single leaf closeout sets `stop_split: true`; next step: BE-001QV-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects gap_unknown_annotation.
- BE-001QV-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.gap_unknown_annotation`; next step: BE-001QW-01 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation baseline_plan.
- BE-001QW-01 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` equivalence baseline and extraction plan; next step: BE-001QW-02 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation extract_closeout.
- BE-001QW-02 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` actual extraction complete; next step: BE-001QW-03 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation single_leaf_closeout.
- BE-001QW-03 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` single leaf closeout sets `stop_split: true`; next step: BE-001QX-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment selects root_validation.
- BE-001QX-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation`; next step: BE-001QY-01 root.contracts.qrpc_core.strategy_ir.root_validation baseline_plan.
- BE-001QY-01 `root.contracts.qrpc_core.strategy_ir.root_validation` equivalence baseline and extraction plan; next step: BE-001QY-02 root.contracts.qrpc_core.strategy_ir.root_validation extract_closeout.
- BE-001QY-02 `root.contracts.qrpc_core.strategy_ir.root_validation` actual extraction complete; next step: BE-001QY-03 root.contracts.qrpc_core.strategy_ir.root_validation single_leaf_closeout.
- BE-001QY-03 `root.contracts.qrpc_core.strategy_ir.root_validation` single leaf closeout sets `continue_split: true`; next step: BE-001QZ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects identity_required_validation.
- BE-001QZ-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`; next step: BE-001RA-01 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation baseline_plan.
- BE-001RA-01 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` equivalence baseline and extraction plan; next step: BE-001RA-02 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation extract_closeout.
- BE-001RA-02 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` actual extraction complete; next step: BE-001RA-03 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation single_leaf_closeout.
- BE-001RA-03 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` single leaf closeout sets `stop_split: true`; next step: BE-001RB-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects signal_logic_validation.
- BE-001RB-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`; next step: BE-001RC-01 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation baseline_plan.
- GOV-SAME-PARENT-PARALLEL updates the recursive speed protocol to allow guarded same-parent child parallel waves; the active Rust cursor remains BE-001RC-01.
- BE-001RC-01 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` equivalence baseline freezes signal/detail, indicator support, logic rule, and logic unknown-marker validation before extraction; next step: BE-001RC-02 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation extract_closeout.
- BE-001RC-02 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` actual extraction complete; `qrpc_core/src/strategy_ir/root_validation/signal_logic_validation.rs` now owns signal/detail, indicator support, logic rule, and logic unknown-marker validation while parent helpers remain parent-owned. Next step: BE-001RC-03 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation single_leaf_closeout.
- BE-001RC-03 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` single leaf closeout sets `stop_split: true`; next step: BE-001RD-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects risk_validation.
- BE-001RD-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.risk_validation`; next step: BE-001RE-01 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation baseline_plan.
- BE-001RE-01 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` equivalence baseline freezes risk unknownable checks and risk profile id/numeric validation before extraction; next step: BE-001RE-02 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation extract_closeout.
- BE-001RE-02 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` actual extraction complete; `qrpc_core/src/strategy_ir/root_validation/risk_validation.rs` now owns risk unknownable checks and risk profile id/numeric validation while parent helpers remain parent-owned. Next step: BE-001RE-03 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation single_leaf_closeout.
- BE-001RE-03 `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` single leaf closeout sets `stop_split: true`; next step: BE-001RF-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects data_execution_validation.
- BE-001RF-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`; next step: BE-001RG-01 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation baseline_plan.
- BE-001RG-01 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` equivalence baseline freezes data requirement checks plus execution and execution profile validation before extraction; next step: BE-001RG-02 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation extract_closeout.
- BE-001RG-02 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` actual extraction complete; `qrpc_core/src/strategy_ir/root_validation/data_execution_validation.rs` now owns data requirement checks plus execution and execution profile validation while parent helpers remain parent-owned. Next step: BE-001RG-03 root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation single_leaf_closeout.
- BE-001RG-03 `root.contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` single leaf closeout sets `stop_split: true`; next step: BE-001RH-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment selects unknown_marker_validation.
- BE-001RH-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation`; next step: BE-001RI-01 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation baseline_plan.
- BE-001RI-01 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` freezes unknown marker helper ownership and `unknowns[*]` path/reason validation baseline; next step: BE-001RI-02 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation extract_closeout.
- BE-001RI-02 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` actual extraction complete; `qrpc_core/src/strategy_ir/root_validation/unknown_marker_validation.rs` now owns unknowns path/reason validation and unknownable helper implementation while parent wrappers preserve sibling mediation. Next step: BE-001RI-03 root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation single_leaf_closeout.
- BE-001RI-03 `root.contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` single leaf closeout sets `stop_split: true`; next step: BE-001RJ-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment.
- BE-001RJ-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment selects `contracts.qrpc_core.strategy_ir.root_validation.test_fixture`; next step: BE-001RK-01 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture baseline_plan.
- BE-001RK-01 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` freezes local test fixture and root validation unit-test baseline; next step: BE-001RK-02 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture extract_closeout.
- BE-001RK-02 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` actual extraction complete; `qrpc_core/src/strategy_ir/root_validation/tests.rs` now owns the sample fixture and root validation unit tests. Next step: BE-001RK-03 root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture single_leaf_closeout.
- BE-001RK-03 `root.contracts.qrpc_core.strategy_ir.root_validation.test_fixture` single leaf closeout sets `stop_split: true`; next step: BE-001RL-01 root.contracts.qrpc_core.strategy_ir.root_validation parent_residual_judgment.
- BE-001RL-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent residual judgment closes the root validation parent; next step: BE-001RM-01 root.contracts.qrpc_core.strategy_ir parent_residual_judgment.
- BE-001RM-01 `root.contracts.qrpc_core.strategy_ir` parent residual judgment closes the Strategy IR parent; next step: BE-001RN-01 root.contracts.qrpc_core parent_residual_judgment selects protocol_primitives.
- BE-001RN-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.protocol_primitives`; next step: BE-001RO-01 root.contracts.qrpc_core.protocol_primitives baseline_plan.
- BE-001RO-01 `root.contracts.qrpc_core.protocol_primitives` freezes primitive constants, enums, serde/display/default behavior, and crate facade baseline; next step: BE-001RO-02 root.contracts.qrpc_core.protocol_primitives extract_closeout.
- BE-001RO-02 `root.contracts.qrpc_core.protocol_primitives` actual extraction complete; `qrpc_core/src/protocol_primitives.rs` now owns primitive constants, enums, symbol serde/parse behavior, display behavior, and primitive defaults. Next step: BE-001RO-03 root.contracts.qrpc_core.protocol_primitives single_leaf_closeout.
- BE-001RO-03 `root.contracts.qrpc_core.protocol_primitives` single leaf closeout sets `stop_split: true`; next step: BE-001RP-01 root.contracts.qrpc_core parent_residual_judgment selects runtime_protocol_config.
- BE-001RP-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.runtime_protocol_config`; next step: BE-001RQ-01 root.contracts.qrpc_core.runtime_protocol_config baseline_plan.
- BE-001RQ-01 `root.contracts.qrpc_core.runtime_protocol_config` freezes runtime config DTOs, universe config, defaults, compiled protocol, and crate facade baseline; next step: BE-001RQ-02 root.contracts.qrpc_core.runtime_protocol_config extract_closeout.
- BE-001RQ-02 `root.contracts.qrpc_core.runtime_protocol_config` actual extraction complete; `qrpc_core/src/runtime_protocol_config.rs` now owns runtime config DTOs, universe config, defaults, and compiled protocol container. Next step: BE-001RQ-03 root.contracts.qrpc_core.runtime_protocol_config single_leaf_closeout.
- BE-001RQ-03 `root.contracts.qrpc_core.runtime_protocol_config` single leaf closeout sets `stop_split: true`; next step: BE-001RR-01 root.contracts.qrpc_core parent_residual_judgment selects artifact_specs.
- BE-001RR-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.artifact_specs`; next step: BE-001RS-01 root.contracts.qrpc_core.artifact_specs baseline_plan.
- BE-001RS-01 `root.contracts.qrpc_core.artifact_specs` freezes canonical digest, run/backtest specs, dataset/execution assumption projections, and artifact bundle baseline; next step: BE-001RS-02 root.contracts.qrpc_core.artifact_specs extract_closeout.
- BE-001RS-02 `root.contracts.qrpc_core.artifact_specs` actual extraction complete; `qrpc_core/src/artifact_specs.rs` now owns canonical digest, run/backtest specs, projections, and artifact bundle contracts. Next step: BE-001RS-03 root.contracts.qrpc_core.artifact_specs single_leaf_closeout.
- BE-001RS-03 `root.contracts.qrpc_core.artifact_specs` single leaf closeout sets `continue_split: true`; next step: BE-001RT-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects canonical_digest.
- BE-001RT-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `contracts.qrpc_core.artifact_specs.canonical_digest`; next step: BE-001RU-01 root.contracts.qrpc_core.artifact_specs.canonical_digest baseline_plan.
- BE-001RU-01 `root.contracts.qrpc_core.artifact_specs.canonical_digest` freezes digest algorithm, digest DTO, and canonical JSON SHA-256 helper baseline; next step: BE-001RU-02 root.contracts.qrpc_core.artifact_specs.canonical_digest extract_closeout.
- BE-001RU-02 `root.contracts.qrpc_core.artifact_specs.canonical_digest` actual extraction complete; `qrpc_core/src/artifact_specs/canonical_digest.rs` now owns digest algorithm, digest DTO, and canonical JSON SHA-256 helper. Next step: BE-001RU-03 root.contracts.qrpc_core.artifact_specs.canonical_digest single_leaf_closeout.
- BE-001RU-03 `root.contracts.qrpc_core.artifact_specs.canonical_digest` single leaf closeout sets `stop_split: true`; next step: BE-001RV-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects run_backtest_specs.
- BE-001RV-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `contracts.qrpc_core.artifact_specs.run_backtest_specs`; next step: BE-001RW-01 root.contracts.qrpc_core.artifact_specs.run_backtest_specs baseline_plan.
- BE-001RW-01 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` freezes run/backtest modes, dataset/execution projections, market data snapshot specs, `RunSpec`, and `BacktestSpec`; next step: BE-001RW-02 root.contracts.qrpc_core.artifact_specs.run_backtest_specs extract_closeout.
- BE-001RW-02 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` actual extraction complete; `qrpc_core/src/artifact_specs/run_backtest_specs.rs` now owns run/backtest modes, projections, snapshot specs, `RunSpec`, and `BacktestSpec`. Next step: BE-001RW-03 root.contracts.qrpc_core.artifact_specs.run_backtest_specs single_leaf_closeout.
- BE-001RW-03 `root.contracts.qrpc_core.artifact_specs.run_backtest_specs` single leaf closeout sets `stop_split: true`; next step: BE-001RX-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment selects artifact_bundle_contract.
- BE-001RX-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment selects `contracts.qrpc_core.artifact_specs.artifact_bundle_contract`; next step: BE-001RY-01 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract baseline_plan.
- BE-001RY-01 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` freezes strategy/core-IR/compile artifact DTOs and bundle contract baseline; next step: BE-001RY-02 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract extract_closeout.
- BE-001RY-02 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` actual extraction complete; `qrpc_core/src/artifact_specs/artifact_bundle_contract.rs` now owns strategy/core-IR/compile artifact DTOs and bundle contract. Next step: BE-001RY-03 root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract single_leaf_closeout.
- BE-001RY-03 `root.contracts.qrpc_core.artifact_specs.artifact_bundle_contract` single leaf closeout sets `stop_split: true`; next step: BE-001RZ-01 root.contracts.qrpc_core.artifact_specs parent_residual_judgment.
- BE-001RZ-01 `root.contracts.qrpc_core.artifact_specs` parent residual judgment closes artifact_specs parent; next step: BE-001SA-01 root.contracts.qrpc_core parent_residual_judgment selects runtime_io_contract.
- BE-001SA-01 `root.contracts.qrpc_core` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract`; next step: BE-001SB-01 root.contracts.qrpc_core.runtime_io_contract baseline_plan.
- BE-001SB-01 `root.contracts.qrpc_core.runtime_io_contract` freezes runtime input/output DTOs from `RawKline` through `BacktestOutput`; next step: BE-001SB-02 root.contracts.qrpc_core.runtime_io_contract extract_closeout.
- BE-001SB-02 `root.contracts.qrpc_core.runtime_io_contract` actual extraction complete; `qrpc_core/src/runtime_io_contract.rs` now owns runtime input/output DTOs from `RawKline` through `BacktestOutput`. Next step: BE-001SB-03 root.contracts.qrpc_core.runtime_io_contract single_leaf_closeout.
- BE-001SB-03 `root.contracts.qrpc_core.runtime_io_contract` single leaf closeout sets `continue_split: true`; next step: BE-001SC-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects market_data_io.
- BE-001SC-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.market_data_io`; next step: BE-001SD-01 root.contracts.qrpc_core.runtime_io_contract.market_data_io baseline_plan.
- BE-001SD-01 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` freezes raw and normalized market data DTOs; next step: BE-001SD-02 root.contracts.qrpc_core.runtime_io_contract.market_data_io extract_closeout.
- BE-001SD-02 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` actual extraction complete; `qrpc_core/src/runtime_io_contract/market_data_io.rs` now owns raw and normalized market data DTOs. Next step: BE-001SD-03 root.contracts.qrpc_core.runtime_io_contract.market_data_io single_leaf_closeout.
- BE-001SD-03 `root.contracts.qrpc_core.runtime_io_contract.market_data_io` single leaf closeout sets `stop_split: true`; next step: BE-001SE-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects decision_flow.
- BE-001SE-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.decision_flow`; next step: BE-001SF-01 root.contracts.qrpc_core.runtime_io_contract.decision_flow baseline_plan.
- BE-001SF-01 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` freezes intent/action/target/agent/risk decision DTOs; next step: BE-001SF-02 root.contracts.qrpc_core.runtime_io_contract.decision_flow extract_closeout.
- BE-001SF-02 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` actual extraction complete; `qrpc_core/src/runtime_io_contract/decision_flow.rs` now owns intent/action/target/agent/risk decision DTOs. Next step: BE-001SF-03 root.contracts.qrpc_core.runtime_io_contract.decision_flow single_leaf_closeout.
- BE-001SF-03 `root.contracts.qrpc_core.runtime_io_contract.decision_flow` single leaf closeout sets `stop_split: true`; next step: BE-001SG-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects execution_io.
- BE-001SG-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.execution_io`; next step: BE-001SH-01 root.contracts.qrpc_core.runtime_io_contract.execution_io baseline_plan.
- BE-001SH-01 `root.contracts.qrpc_core.runtime_io_contract.execution_io` freezes simulated order, execution plan, fill report, open order, and fill result DTO baseline; next step: BE-001SH-02 root.contracts.qrpc_core.runtime_io_contract.execution_io extract_closeout.
- BE-001SH-02 `root.contracts.qrpc_core.runtime_io_contract.execution_io` actual extraction complete; `qrpc_core/src/runtime_io_contract/execution_io.rs` now owns simulated order, execution plan, fill report, open order, and fill result DTOs. Next step: BE-001SH-03 root.contracts.qrpc_core.runtime_io_contract.execution_io single_leaf_closeout.
- BE-001SH-03 `root.contracts.qrpc_core.runtime_io_contract.execution_io` single leaf closeout sets `stop_split: true`; next step: BE-001SI-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects portfolio_state.
- BE-001SI-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.portfolio_state`; next step: BE-001SJ-01 root.contracts.qrpc_core.runtime_io_contract.portfolio_state baseline_plan.
- BE-001SJ-01 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` freezes portfolio state DTOs and helper method baseline; next step: BE-001SJ-02 root.contracts.qrpc_core.runtime_io_contract.portfolio_state extract_closeout.
- BE-001SJ-02 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` actual extraction complete; `qrpc_core/src/runtime_io_contract/portfolio_state.rs` now owns position, exposure, portfolio state DTOs, and helper methods. Next step: BE-001SJ-03 root.contracts.qrpc_core.runtime_io_contract.portfolio_state single_leaf_closeout.
- BE-001SJ-03 `root.contracts.qrpc_core.runtime_io_contract.portfolio_state` single leaf closeout sets `stop_split: true`; next step: BE-001SK-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects runtime_output.
- BE-001SK-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.runtime_output`; next step: BE-001SL-01 root.contracts.qrpc_core.runtime_io_contract.runtime_output baseline_plan.
- BE-001SL-01 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` freezes runtime event, cycle output, and session output DTO baseline; next step: BE-001SL-02 root.contracts.qrpc_core.runtime_io_contract.runtime_output extract_closeout.
- BE-001SL-02 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` actual extraction complete; `qrpc_core/src/runtime_io_contract/runtime_output.rs` now owns runtime event, cycle output, and session output DTOs. Next step: BE-001SL-03 root.contracts.qrpc_core.runtime_io_contract.runtime_output single_leaf_closeout.
- BE-001SL-03 `root.contracts.qrpc_core.runtime_io_contract.runtime_output` single leaf closeout sets `stop_split: true`; next step: BE-001SM-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment selects backtest_output.
- BE-001SM-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment selects `contracts.qrpc_core.runtime_io_contract.backtest_output`; next step: BE-001SN-01 root.contracts.qrpc_core.runtime_io_contract.backtest_output baseline_plan.
- BE-001SN-01 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` freezes final backtest output DTO baseline; next step: BE-001SN-02 root.contracts.qrpc_core.runtime_io_contract.backtest_output extract_closeout.
- BE-001SN-02 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` actual extraction complete; `qrpc_core/src/runtime_io_contract/backtest_output.rs` now owns final backtest output DTOs and nested metric DTOs. Next step: BE-001SN-03 root.contracts.qrpc_core.runtime_io_contract.backtest_output single_leaf_closeout.
- BE-001SN-03 `root.contracts.qrpc_core.runtime_io_contract.backtest_output` single leaf closeout sets `stop_split: true`; next step: BE-001SO-01 root.contracts.qrpc_core.runtime_io_contract parent_residual_judgment closes parent.
- BE-001SO-01 `root.contracts.qrpc_core.runtime_io_contract` parent residual judgment closes the runtime IO parent; next step: BE-001SP-01 root.contracts.qrpc_core parent_residual_judgment selects rfc_execution_contracts.
- BE-001SP-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects rfc_execution_contracts；下一步: BE-001SQ-01 root.contracts.qrpc_core.rfc_execution_contracts baseline_plan。
- BE-001SQ-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts equivalence baseline and extraction plan；下一步: BE-001SQ-02 root.contracts.qrpc_core.rfc_execution_contracts extract_closeout。
- BE-001SQ-02 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts actual extraction complete；下一步: BE-001SQ-03 root.contracts.qrpc_core.rfc_execution_contracts single_leaf_closeout。
- BE-001SQ-03 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts single leaf closeout continues split；下一步: BE-001SR-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects data_request。
- BE-001SR-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects data_request；下一步: BE-001SS-01 root.contracts.qrpc_core.rfc_execution_contracts.data_request baseline_plan。
- BE-001SS-01 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request equivalence baseline and extraction plan；下一步: BE-001SS-02 root.contracts.qrpc_core.rfc_execution_contracts.data_request extract_closeout。
- BE-001SS-02 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request actual extraction complete；下一步: BE-001SS-03 root.contracts.qrpc_core.rfc_execution_contracts.data_request single_leaf_closeout。
- BE-001SS-03 `root.contracts.qrpc_core.rfc_execution_contracts.data_request` root.contracts.qrpc_core.rfc_execution_contracts.data_request single leaf closeout stops split；下一步: BE-001ST-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects allocation。
- BE-001ST-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects allocation；下一步: BE-001SU-01 root.contracts.qrpc_core.rfc_execution_contracts.allocation baseline_plan。
- BE-001SU-01 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation equivalence baseline and extraction plan；下一步: BE-001SU-02 root.contracts.qrpc_core.rfc_execution_contracts.allocation extract_closeout。
- BE-001SU-02 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation actual extraction complete；下一步: BE-001SU-03 root.contracts.qrpc_core.rfc_execution_contracts.allocation single_leaf_closeout。
- BE-001SU-03 `root.contracts.qrpc_core.rfc_execution_contracts.allocation` root.contracts.qrpc_core.rfc_execution_contracts.allocation single leaf closeout stops split；下一步: BE-001SV-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects order_contract。
- BE-001SV-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects order_contract；下一步: BE-001SW-01 root.contracts.qrpc_core.rfc_execution_contracts.order_contract baseline_plan。
- BE-001SW-01 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract equivalence baseline and extraction plan；下一步: BE-001SW-02 root.contracts.qrpc_core.rfc_execution_contracts.order_contract extract_closeout。
- BE-001SW-02 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract actual extraction complete；下一步: BE-001SW-03 root.contracts.qrpc_core.rfc_execution_contracts.order_contract single_leaf_closeout。
- BE-001SW-03 `root.contracts.qrpc_core.rfc_execution_contracts.order_contract` root.contracts.qrpc_core.rfc_execution_contracts.order_contract single leaf closeout stops split；下一步: BE-001SX-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects execution_feedback。
- BE-001SX-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects execution_feedback；下一步: BE-001SY-01 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback baseline_plan。
- BE-001SY-01 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback equivalence baseline and extraction plan；下一步: BE-001SY-02 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback extract_closeout。
- BE-001SY-02 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback actual extraction complete；下一步: BE-001SY-03 root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single_leaf_closeout。
- BE-001SY-03 `root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback` root.contracts.qrpc_core.rfc_execution_contracts.execution_feedback single leaf closeout stops split；下一步: BE-001SZ-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment selects handoff_snapshot。
- BE-001SZ-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment selects handoff_snapshot；下一步: BE-001TA-01 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot baseline_plan。
- BE-001TA-01 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot equivalence baseline and extraction plan；下一步: BE-001TA-02 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot extract_closeout。
- BE-001TA-02 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot actual extraction complete；下一步: BE-001TA-03 root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single_leaf_closeout。
- BE-001TA-03 `root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot` root.contracts.qrpc_core.rfc_execution_contracts.handoff_snapshot single leaf closeout stops split；下一步: BE-001TB-01 root.contracts.qrpc_core.rfc_execution_contracts parent_residual_judgment closes parent。
- BE-001TB-01 `root.contracts.qrpc_core.rfc_execution_contracts` root.contracts.qrpc_core.rfc_execution_contracts parent residual judgment closes parent；下一步: BE-001TC-01 root.contracts.qrpc_core parent_residual_judgment closes parent。
- BE-001TC-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment selects test_fixture；下一步: BE-001TD-01 root.contracts.qrpc_core.test_fixture baseline_plan。
- BE-001TD-01 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture equivalence baseline and extraction plan；下一步: BE-001TD-02 root.contracts.qrpc_core.test_fixture extract_closeout。
- BE-001TD-02 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture actual extraction complete；下一步: BE-001TD-03 root.contracts.qrpc_core.test_fixture single_leaf_closeout。
- BE-001TD-03 `root.contracts.qrpc_core.test_fixture` root.contracts.qrpc_core.test_fixture single leaf closeout stops split；下一步: BE-001TE-01 root.contracts.qrpc_core parent_residual_judgment closes parent。
- BE-001TE-01 `root.contracts.qrpc_core` root.contracts.qrpc_core parent residual judgment closes parent；下一步: BE-001TF-01 root.contracts parent_residual_judgment selects core_ir。
- BE-001TF-01 `root.contracts` root.contracts parent residual judgment selects core_ir；下一步: BE-001TG-01 root.contracts.core_ir baseline_plan。
- BE-001TG-01 `root.contracts.core_ir` root.contracts.core_ir equivalence baseline and extraction plan；下一步: BE-001TH-01 root.contracts.core_ir parent_residual_judgment selects v1_contract。
- BE-001TH-01 `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v1_contract；下一步: BE-001TI-01 root.contracts.core_ir.v1_contract baseline_plan。
- BE-001TI-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract equivalence baseline and extraction plan；下一步: BE-001TI-02 root.contracts.core_ir.v1_contract extract_closeout。
- BE-001TI-02 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract actual extraction complete；下一步: BE-001TI-03 root.contracts.core_ir.v1_contract single_leaf_closeout。
- BE-001TI-03 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract single leaf closeout continues split；下一步: BE-001TJ-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects root_graph_contract。
- BE-001TJ-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects root_graph_contract；下一步: BE-001TK-01 root.contracts.core_ir.v1_contract.root_graph_contract baseline_plan。
- BE-001TK-01 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract equivalence baseline and extraction plan；下一步: BE-001TK-02 root.contracts.core_ir.v1_contract.root_graph_contract extract_closeout。
- BE-001TK-02 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract actual extraction complete；下一步: BE-001TK-03 root.contracts.core_ir.v1_contract.root_graph_contract single_leaf_closeout。
- BE-001TK-03 `root.contracts.core_ir.v1_contract.root_graph_contract` root.contracts.core_ir.v1_contract.root_graph_contract single leaf closeout stops split；下一步: BE-001TL-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects data_indicator_expression_contract。
- BE-001TL-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects data_indicator_expression_contract；下一步: BE-001TM-01 root.contracts.core_ir.v1_contract.data_indicator_expression_contract baseline_plan。
- BE-001TM-01 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract equivalence baseline and extraction plan；下一步: BE-001TM-02 root.contracts.core_ir.v1_contract.data_indicator_expression_contract extract_closeout。
- BE-001TM-02 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract actual extraction complete；下一步: BE-001TM-03 root.contracts.core_ir.v1_contract.data_indicator_expression_contract single_leaf_closeout。
- BE-001TM-03 `root.contracts.core_ir.v1_contract.data_indicator_expression_contract` root.contracts.core_ir.v1_contract.data_indicator_expression_contract single leaf closeout stops split；下一步: BE-001TN-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects policy_execution_contract。
- BE-001TN-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects policy_execution_contract；下一步: BE-001TO-01 root.contracts.core_ir.v1_contract.policy_execution_contract baseline_plan。
- BE-001TO-01 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract equivalence baseline and extraction plan；下一步: BE-001TO-02 root.contracts.core_ir.v1_contract.policy_execution_contract extract_closeout。
- BE-001TO-02 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract actual extraction complete；下一步: BE-001TO-03 root.contracts.core_ir.v1_contract.policy_execution_contract single_leaf_closeout。
- BE-001TO-03 `root.contracts.core_ir.v1_contract.policy_execution_contract` root.contracts.core_ir.v1_contract.policy_execution_contract single leaf closeout stops split；下一步: BE-001TP-01 root.contracts.core_ir.v1_contract parent_residual_judgment selects test_fixture。
- BE-001TP-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment selects test_fixture；下一步: BE-001TQ-01 root.contracts.core_ir.v1_contract.test_fixture baseline_plan。
- BE-001TQ-01 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture equivalence baseline and extraction plan；下一步: BE-001TQ-02 root.contracts.core_ir.v1_contract.test_fixture extract_closeout。
- BE-001TQ-02 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture actual extraction complete；下一步: BE-001TQ-03 root.contracts.core_ir.v1_contract.test_fixture single_leaf_closeout。
- BE-001TQ-03 `root.contracts.core_ir.v1_contract.test_fixture` root.contracts.core_ir.v1_contract.test_fixture single leaf closeout stops split；下一步: BE-001TR-01 root.contracts.core_ir.v1_contract parent_residual_judgment closes parent。
- BE-001TR-01 `root.contracts.core_ir.v1_contract` root.contracts.core_ir.v1_contract parent residual judgment closes parent；下一步: BE-001TS-01 root.contracts.core_ir parent_residual_judgment selects v4_contracts。
- BE-001TS-01 `root.contracts.core_ir` root.contracts.core_ir parent residual judgment selects v4_contracts；下一步: BE-001TT-01 root.contracts.core_ir.v4_contracts baseline_plan。
- BE-001TT-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts equivalence baseline and split plan；下一步: BE-001TU-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects schema_identity_constants。
- BE-001TU-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects schema_identity_constants；下一步: BE-001TV-01 root.contracts.core_ir.v4_contracts.schema_identity_constants baseline_plan。
- BE-001TV-01 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants equivalence baseline and extraction plan；下一步: BE-001TV-02 root.contracts.core_ir.v4_contracts.schema_identity_constants extract_closeout。
- BE-001TV-02 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants actual extraction complete；下一步: BE-001TV-03 root.contracts.core_ir.v4_contracts.schema_identity_constants single_leaf_closeout。
- BE-001TV-03 `root.contracts.core_ir.v4_contracts.schema_identity_constants` root.contracts.core_ir.v4_contracts.schema_identity_constants single leaf closeout stops split；下一步: BE-001TW-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects backtest_artifact_contract。
- BE-001TW-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects backtest_artifact_contract；下一步: BE-001TX-01 root.contracts.core_ir.v4_contracts.backtest_artifact_contract baseline_plan。
- BE-001TX-01 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract equivalence baseline and extraction plan；下一步: BE-001TX-02 root.contracts.core_ir.v4_contracts.backtest_artifact_contract extract_closeout。
- BE-001TX-02 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract actual extraction complete；下一步: BE-001TX-03 root.contracts.core_ir.v4_contracts.backtest_artifact_contract single_leaf_closeout。
- BE-001TX-03 `root.contracts.core_ir.v4_contracts.backtest_artifact_contract` root.contracts.core_ir.v4_contracts.backtest_artifact_contract single leaf closeout stops split；下一步: BE-001TY-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects machine_contract。
- BE-001TY-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_contract；下一步: BE-001TZ-01 root.contracts.core_ir.v4_contracts.machine_contract baseline_plan。
- BE-001TZ-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract equivalence baseline and extraction plan；下一步: BE-001TZ-02 root.contracts.core_ir.v4_contracts.machine_contract extract_closeout。
- BE-001TZ-02 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract actual extraction complete；下一步: BE-001TZ-03 root.contracts.core_ir.v4_contracts.machine_contract single_leaf_closeout。
- BE-001TZ-03 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract single leaf closeout continues split；下一步: BE-001UA-01 root.contracts.core_ir.v4_contracts.machine_contract parent_residual_judgment selects static_validation。
- BE-001UA-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment selects static_validation；下一步: BE-001UB-01 root.contracts.core_ir.v4_contracts.machine_contract.static_validation baseline_plan。
- BE-001UB-01 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation equivalence baseline and extraction plan；下一步: BE-001UB-02 root.contracts.core_ir.v4_contracts.machine_contract.static_validation extract_closeout。
- BE-001UB-02 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation actual extraction complete；下一步: BE-001UB-03 root.contracts.core_ir.v4_contracts.machine_contract.static_validation single_leaf_closeout。
- BE-001UB-03 `root.contracts.core_ir.v4_contracts.machine_contract.static_validation` root.contracts.core_ir.v4_contracts.machine_contract.static_validation single leaf closeout stops split；下一步: BE-001UC-01 root.contracts.core_ir.v4_contracts.machine_contract parent_residual_judgment closes parent。
- BE-001UC-01 `root.contracts.core_ir.v4_contracts.machine_contract` root.contracts.core_ir.v4_contracts.machine_contract parent residual judgment closes parent；下一步: BE-001UD-01 root.contracts.core_ir.v4_contracts parent_residual_judgment selects machine_graph_contract。
- BE-001UD-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects machine_graph_contract；下一步: BE-001UE-01 root.contracts.core_ir.v4_contracts.machine_graph_contract baseline_plan。
- BE-001UE-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract equivalence baseline and extraction plan；下一步: BE-001UE-02 root.contracts.core_ir.v4_contracts.machine_graph_contract extract_closeout。
- BE-001UE-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract actual extraction complete；下一步: BE-001UE-03 root.contracts.core_ir.v4_contracts.machine_graph_contract single_leaf_closeout。
- BE-001UE-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract single leaf closeout continues split；下一步: BE-001UF-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment selects event_catalog。
- BE-001UF-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects event_catalog；下一步: BE-001UG-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog baseline_plan。
- BE-001UG-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog equivalence baseline and extraction plan；下一步: BE-001UG-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog extract_closeout。
- BE-001UG-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog actual extraction complete；下一步: BE-001UG-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single_leaf_closeout。
- BE-001UG-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog` root.contracts.core_ir.v4_contracts.machine_graph_contract.event_catalog single leaf closeout stops split；下一步: BE-001UH-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment selects graph_static_validation。
- BE-001UH-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects graph_static_validation；下一步: BE-001UI-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation baseline_plan。
- BE-001UI-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation equivalence baseline and extraction plan；下一步: BE-001UI-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation extract_closeout。
- BE-001UI-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation actual extraction complete；下一步: BE-001UI-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single_leaf_closeout。
- BE-001UI-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split；下一步: BE-001UJ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects risk_plane_validation。
- BE-001UJ-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects risk_plane_validation；下一步: BE-001UK-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation baseline_plan。
- BE-001UK-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation equivalence baseline and extraction plan；下一步: BE-001UK-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation extract_closeout。
- BE-001UK-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation actual extraction complete；下一步: BE-001UK-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single_leaf_closeout。
- BE-001UK-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.risk_plane_validation single leaf closeout stops split；下一步: BE-001UL-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects event_usage_validation。
- BE-001UL-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects event_usage_validation；下一步: BE-001UM-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation baseline_plan。
- BE-001UM-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation equivalence baseline and extraction plan；下一步: BE-001UM-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation extract_closeout。
- BE-001UM-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation actual extraction complete；下一步: BE-001UM-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single_leaf_closeout。
- BE-001UM-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout continues split；下一步: BE-001UN-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent_residual_judgment selects event_party_validation。
- BE-001UN-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_party_validation；下一步: BE-001UO-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation baseline_plan。
- BE-001UO-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation equivalence baseline and extraction plan；下一步: BE-001UO-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation extract_closeout。
- BE-001UO-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation actual extraction complete；下一步: BE-001UO-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single_leaf_closeout。
- BE-001UO-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_party_validation single leaf closeout stops split；下一步: BE-001UP-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent_residual_judgment selects event_reference_resolution。
- BE-001UP-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation parent residual judgment selects event_reference_resolution；下一步: BE-001UQ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution baseline_plan。
- BE-001UQ-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution equivalence baseline and extraction plan；下一步: BE-001UQ-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution extract_closeout。
- BE-001UQ-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution actual extraction complete；下一步: BE-001UQ-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution single_leaf_closeout。
- BE-001UQ-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation.event_reference_resolution single leaf closeout stops split；下一步: BE-001UR-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single_leaf_closeout。
- BE-001UR-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.event_usage_validation single leaf closeout stops split；下一步: BE-001US-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects graph_acyclic_validation。
- BE-001US-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects graph_acyclic_validation；下一步: BE-001UT-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation baseline_plan。
- BE-001UT-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation equivalence baseline and extraction plan；下一步: BE-001UT-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation extract_closeout。
- BE-001UT-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation actual extraction complete；下一步: BE-001UT-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation single_leaf_closeout。
- BE-001UT-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.graph_acyclic_validation single leaf closeout stops split；下一步: BE-001UU-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single_leaf_closeout。
- BE-001UU-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout continues split；下一步: BE-001UV-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects machine_identity_validation。
- BE-001UV-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects machine_identity_validation；下一步: BE-001UW-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation baseline_plan。
- BE-001UW-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation equivalence baseline and extraction plan；下一步: BE-001UW-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation extract_closeout。
- BE-001UW-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation actual extraction complete；下一步: BE-001UW-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation single_leaf_closeout。
- BE-001UW-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.machine_identity_validation single leaf closeout stops split；下一步: BE-001UX-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent_residual_judgment selects edge_identity_validation。
- BE-001UX-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation parent residual judgment selects edge_identity_validation；下一步: BE-001UY-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation baseline_plan。
- BE-001UY-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation equivalence baseline and extraction plan；下一步: BE-001UY-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation extract_closeout。
- BE-001UY-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation actual extraction complete；下一步: BE-001UY-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation single_leaf_closeout。
- BE-001UY-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation.edge_identity_validation single leaf closeout stops split；下一步: BE-001UZ-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single_leaf_closeout。
- BE-001UZ-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation` root.contracts.core_ir.v4_contracts.machine_graph_contract.graph_static_validation single leaf closeout stops split；下一步: BE-001VA-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment closes parent。
- BE-001VA-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment selects traversal_helpers；下一步: BE-001VB-01 root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers baseline_plan。
- BE-001VB-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers equivalence baseline and extraction plan；下一步: BE-001VB-02 root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers extract_closeout。
- BE-001VB-02 `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers actual extraction complete；下一步: BE-001VB-03 root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers single_leaf_closeout。
- BE-001VB-03 `root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers` root.contracts.core_ir.v4_contracts.machine_graph_contract.traversal_helpers single leaf closeout stops split；下一步: BE-001VC-01 root.contracts.core_ir.v4_contracts.machine_graph_contract parent_residual_judgment closes parent。
- BE-001VC-01 `root.contracts.core_ir.v4_contracts.machine_graph_contract` root.contracts.core_ir.v4_contracts.machine_graph_contract parent residual judgment closes parent；下一步: BE-001VD-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VD-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects qs_state_machine_profile；下一步: BE-001VE-01 root.contracts.core_ir.v4_contracts.qs_state_machine_profile baseline_plan。
- BE-001VE-01 `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile equivalence baseline and extraction plan；下一步: BE-001VE-02 root.contracts.core_ir.v4_contracts.qs_state_machine_profile extract_closeout。
- BE-001VE-02 `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile actual extraction complete；下一步: BE-001VE-03 root.contracts.core_ir.v4_contracts.qs_state_machine_profile single_leaf_closeout。
- BE-001VE-03 `root.contracts.core_ir.v4_contracts.qs_state_machine_profile` root.contracts.core_ir.v4_contracts.qs_state_machine_profile single leaf closeout stops split；下一步: BE-001VF-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VF-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects runtime_mode_contract；下一步: BE-001VG-01 root.contracts.core_ir.v4_contracts.runtime_mode_contract baseline_plan。
- BE-001VG-01 `root.contracts.core_ir.v4_contracts.runtime_mode_contract` root.contracts.core_ir.v4_contracts.runtime_mode_contract equivalence baseline and extraction plan；下一步: BE-001VG-02 root.contracts.core_ir.v4_contracts.runtime_mode_contract extract_closeout。
- BE-001VG-02 `root.contracts.core_ir.v4_contracts.runtime_mode_contract` root.contracts.core_ir.v4_contracts.runtime_mode_contract actual extraction complete；下一步: BE-001VG-03 root.contracts.core_ir.v4_contracts.runtime_mode_contract single_leaf_closeout。
- BE-001VG-03 `root.contracts.core_ir.v4_contracts.runtime_mode_contract` root.contracts.core_ir.v4_contracts.runtime_mode_contract single leaf closeout stops split；下一步: BE-001VH-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VH-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects venue_capability_matrix；下一步: BE-001VI-01 root.contracts.core_ir.v4_contracts.venue_capability_matrix baseline_plan。
- BE-001VI-01 `root.contracts.core_ir.v4_contracts.venue_capability_matrix` root.contracts.core_ir.v4_contracts.venue_capability_matrix equivalence baseline and extraction plan；下一步: BE-001VI-02 root.contracts.core_ir.v4_contracts.venue_capability_matrix extract_closeout。
- BE-001VI-02 `root.contracts.core_ir.v4_contracts.venue_capability_matrix` root.contracts.core_ir.v4_contracts.venue_capability_matrix actual extraction complete；下一步: BE-001VI-03 root.contracts.core_ir.v4_contracts.venue_capability_matrix single_leaf_closeout。
- BE-001VI-03 `root.contracts.core_ir.v4_contracts.venue_capability_matrix` root.contracts.core_ir.v4_contracts.venue_capability_matrix single leaf closeout stops split；下一步: BE-001VJ-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VJ-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects qs_type_system_contract；下一步: BE-001VK-01 root.contracts.core_ir.v4_contracts.qs_type_system_contract baseline_plan。
- BE-001VK-01 `root.contracts.core_ir.v4_contracts.qs_type_system_contract` root.contracts.core_ir.v4_contracts.qs_type_system_contract equivalence baseline and extraction plan；下一步: BE-001VK-02 root.contracts.core_ir.v4_contracts.qs_type_system_contract extract_closeout。
- BE-001VK-02 `root.contracts.core_ir.v4_contracts.qs_type_system_contract` root.contracts.core_ir.v4_contracts.qs_type_system_contract actual extraction complete；下一步: BE-001VK-03 root.contracts.core_ir.v4_contracts.qs_type_system_contract single_leaf_closeout。
- BE-001VK-03 `root.contracts.core_ir.v4_contracts.qs_type_system_contract` root.contracts.core_ir.v4_contracts.qs_type_system_contract single leaf closeout stops split；下一步: BE-001VL-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VL-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects version_manifest；下一步: BE-001VM-01 root.contracts.core_ir.v4_contracts.version_manifest baseline_plan。
- BE-001VM-01 `root.contracts.core_ir.v4_contracts.version_manifest` root.contracts.core_ir.v4_contracts.version_manifest equivalence baseline and extraction plan；下一步: BE-001VM-02 root.contracts.core_ir.v4_contracts.version_manifest extract_closeout。
- BE-001VM-02 `root.contracts.core_ir.v4_contracts.version_manifest` root.contracts.core_ir.v4_contracts.version_manifest actual extraction complete；下一步: BE-001VM-03 root.contracts.core_ir.v4_contracts.version_manifest single_leaf_closeout。
- BE-001VM-03 `root.contracts.core_ir.v4_contracts.version_manifest` root.contracts.core_ir.v4_contracts.version_manifest single leaf closeout stops split；下一步: BE-001VN-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VN-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects plugin_governance_contract；下一步: BE-001VO-01 root.contracts.core_ir.v4_contracts.plugin_governance_contract baseline_plan。
- BE-001VO-01 `root.contracts.core_ir.v4_contracts.plugin_governance_contract` root.contracts.core_ir.v4_contracts.plugin_governance_contract equivalence baseline and extraction plan；下一步: BE-001VO-02 root.contracts.core_ir.v4_contracts.plugin_governance_contract extract_closeout。
- BE-001VO-02 `root.contracts.core_ir.v4_contracts.plugin_governance_contract` root.contracts.core_ir.v4_contracts.plugin_governance_contract actual extraction complete；下一步: BE-001VO-03 root.contracts.core_ir.v4_contracts.plugin_governance_contract single_leaf_closeout。
- BE-001VO-03 `root.contracts.core_ir.v4_contracts.plugin_governance_contract` root.contracts.core_ir.v4_contracts.plugin_governance_contract single leaf closeout stops split；下一步: BE-001VP-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VP-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects reproducibility_contract；下一步: BE-001VQ-01 root.contracts.core_ir.v4_contracts.reproducibility_contract baseline_plan。
- BE-001VQ-01 `root.contracts.core_ir.v4_contracts.reproducibility_contract` root.contracts.core_ir.v4_contracts.reproducibility_contract equivalence baseline and extraction plan；下一步: BE-001VQ-02 root.contracts.core_ir.v4_contracts.reproducibility_contract extract_closeout。
- BE-001VQ-02 `root.contracts.core_ir.v4_contracts.reproducibility_contract` root.contracts.core_ir.v4_contracts.reproducibility_contract actual extraction complete；下一步: BE-001VQ-03 root.contracts.core_ir.v4_contracts.reproducibility_contract single_leaf_closeout。
- BE-001VQ-03 `root.contracts.core_ir.v4_contracts.reproducibility_contract` root.contracts.core_ir.v4_contracts.reproducibility_contract single leaf closeout stops split；下一步: BE-001VR-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VR-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects complexity_budget_contract；下一步: BE-001VS-01 root.contracts.core_ir.v4_contracts.complexity_budget_contract baseline_plan。
- BE-001VS-01 `root.contracts.core_ir.v4_contracts.complexity_budget_contract` root.contracts.core_ir.v4_contracts.complexity_budget_contract equivalence baseline and extraction plan；下一步: BE-001VS-02 root.contracts.core_ir.v4_contracts.complexity_budget_contract extract_closeout。
- BE-001VS-02 `root.contracts.core_ir.v4_contracts.complexity_budget_contract` root.contracts.core_ir.v4_contracts.complexity_budget_contract actual extraction complete；下一步: BE-001VS-03 root.contracts.core_ir.v4_contracts.complexity_budget_contract single_leaf_closeout。
- BE-001VS-03 `root.contracts.core_ir.v4_contracts.complexity_budget_contract` root.contracts.core_ir.v4_contracts.complexity_budget_contract single leaf closeout stops split；下一步: BE-001VT-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VT-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects developer_learning_pipeline_contract；下一步: BE-001VU-01 root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract baseline_plan。
- BE-001VU-01 `root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract` root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract equivalence baseline and extraction plan；下一步: BE-001VU-02 root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract extract_closeout。
- BE-001VU-02 `root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract` root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract actual extraction complete；下一步: BE-001VU-03 root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract single_leaf_closeout。
- BE-001VU-03 `root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract` root.contracts.core_ir.v4_contracts.developer_learning_pipeline_contract single leaf closeout stops split；下一步: BE-001VV-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001VV-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects static_contract_bundle；下一步: BE-001VW-01 root.contracts.core_ir.v4_contracts.static_contract_bundle baseline_plan。
- BE-001VW-01 `root.contracts.core_ir.v4_contracts.static_contract_bundle` root.contracts.core_ir.v4_contracts.static_contract_bundle equivalence baseline and extraction plan；下一步: BE-001VW-02 root.contracts.core_ir.v4_contracts.static_contract_bundle extract_closeout。
- BE-001VW-02 `root.contracts.core_ir.v4_contracts.static_contract_bundle` root.contracts.core_ir.v4_contracts.static_contract_bundle actual extraction complete；下一步: BE-001VW-03 root.contracts.core_ir.v4_contracts.static_contract_bundle single_leaf_closeout。
- BE-001VW-03 `root.contracts.core_ir.v4_contracts.static_contract_bundle` root.contracts.core_ir.v4_contracts.static_contract_bundle single leaf closeout continues split；下一步: BE-001VX-01 root.contracts.core_ir.v4_contracts.static_contract_bundle parent_residual_judgment selects static_validation。
- BE-001VX-01 `root.contracts.core_ir.v4_contracts.static_contract_bundle` root.contracts.core_ir.v4_contracts.static_contract_bundle parent residual judgment selects static_validation；下一步: BE-001VY-01 root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation baseline_plan。
- BE-001VY-01 `root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation` root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation equivalence baseline and extraction plan；下一步: BE-001VY-02 root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation extract_closeout。
- BE-001VY-02 `root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation` root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation actual extraction complete；下一步: BE-001VY-03 root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation single_leaf_closeout。
- BE-001VY-03 `root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation` root.contracts.core_ir.v4_contracts.static_contract_bundle.static_validation single leaf closeout stops split；下一步: BE-001VZ-01 root.contracts.core_ir.v4_contracts.static_contract_bundle parent_residual_judgment closes parent。
- BE-001VZ-01 `root.contracts.core_ir.v4_contracts.static_contract_bundle` root.contracts.core_ir.v4_contracts.static_contract_bundle parent residual judgment closes parent；下一步: BE-001WA-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001WA-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects compile_time_capability_report；下一步: BE-001WB-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report baseline_plan。
- BE-001WB-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report equivalence baseline and extraction plan；下一步: BE-001WB-02 root.contracts.core_ir.v4_contracts.compile_time_capability_report extract_closeout。
- BE-001WB-02 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report actual extraction complete；下一步: BE-001WB-03 root.contracts.core_ir.v4_contracts.compile_time_capability_report single_leaf_closeout。
- BE-001WB-03 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report single leaf closeout continues split；下一步: BE-001WC-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report parent_residual_judgment selects report_builder。
- BE-001WC-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report parent residual judgment selects report_builder；下一步: BE-001WD-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder baseline_plan。
- BE-001WD-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder equivalence baseline and extraction plan；下一步: BE-001WD-02 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder extract_closeout。
- BE-001WD-02 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder actual extraction complete；下一步: BE-001WD-03 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder single_leaf_closeout。
- BE-001WD-03 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_builder single leaf closeout stops split；下一步: BE-001WE-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report parent_residual_judgment selects report_validation。
- BE-001WE-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report parent residual judgment selects report_validation；下一步: BE-001WF-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation baseline_plan。
- BE-001WF-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation equivalence baseline and extraction plan；下一步: BE-001WF-02 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation extract_closeout。
- BE-001WF-02 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation actual extraction complete；下一步: BE-001WF-03 root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation single_leaf_closeout。
- BE-001WF-03 `root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation` root.contracts.core_ir.v4_contracts.compile_time_capability_report.report_validation single leaf closeout stops split；下一步: BE-001WG-01 root.contracts.core_ir.v4_contracts.compile_time_capability_report parent_residual_judgment closes parent。
- BE-001WG-01 `root.contracts.core_ir.v4_contracts.compile_time_capability_report` root.contracts.core_ir.v4_contracts.compile_time_capability_report parent residual judgment closes parent；下一步: BE-001WH-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001WH-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment selects core_ir_compat_bridge；下一步: BE-001WI-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge baseline_plan。
- BE-001WI-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge equivalence baseline and extraction plan；下一步: BE-001WI-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge extract_closeout。
- BE-001WI-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge actual extraction complete；下一步: BE-001WI-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge single_leaf_closeout。
- BE-001WI-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge single leaf closeout continues split；下一步: BE-001WJ-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent_residual_judgment selects core_ir_validation。
- BE-001WJ-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent residual judgment selects core_ir_validation；下一步: BE-001WK-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation baseline_plan。
- BE-001WK-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation equivalence baseline and extraction plan；下一步: BE-001WK-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation extract_closeout。
- BE-001WK-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation actual extraction complete；下一步: BE-001WK-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation single_leaf_closeout。
- BE-001WK-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation single leaf closeout continues split；下一步: BE-001WL-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation parent_residual_judgment selects reference_validation。
- BE-001WL-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation parent residual judgment selects reference_validation；下一步: BE-001WM-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation baseline_plan。
- BE-001WM-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation equivalence baseline and extraction plan；下一步: BE-001WM-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation extract_closeout。
- BE-001WM-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation actual extraction complete；下一步: BE-001WM-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation single_leaf_closeout。
- BE-001WM-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation.reference_validation single leaf closeout stops split；下一步: BE-001WN-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation parent_residual_judgment。
- BE-001WN-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.core_ir_validation parent residual judgment closes parent；下一步: BE-001WO-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent_residual_judgment。
- BE-001WO-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent residual judgment selects compat_graph_builder；下一步: BE-001WP-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder baseline_plan。
- BE-001WP-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder equivalence baseline and extraction plan；下一步: BE-001WP-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder extract_closeout。
- BE-001WP-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder actual extraction complete；下一步: BE-001WP-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder single_leaf_closeout。
- BE-001WP-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder single leaf closeout continues split；下一步: BE-001WQ-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent_residual_judgment selects event_catalog_builder。
- BE-001WQ-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent residual judgment selects event_catalog_builder；下一步: BE-001WR-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder baseline_plan。
- BE-001WR-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder equivalence baseline and extraction plan；下一步: BE-001WR-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder extract_closeout。
- BE-001WR-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder actual extraction complete；下一步: BE-001WR-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder single_leaf_closeout。
- BE-001WR-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.event_catalog_builder single leaf closeout stops split；下一步: BE-001WS-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent_residual_judgment。
- BE-001WS-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent residual judgment selects machine_builder；下一步: BE-001WT-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder baseline_plan。
- BE-001WT-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder equivalence baseline and extraction plan；下一步: BE-001WT-02 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder extract_closeout。
- BE-001WT-02 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder actual extraction complete；下一步: BE-001WT-03 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder single_leaf_closeout。
- BE-001WT-03 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder.machine_builder single leaf closeout stops split；下一步: BE-001WU-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent_residual_judgment。
- BE-001WU-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge.compat_graph_builder parent residual judgment closes parent；下一步: BE-001WV-01 root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent_residual_judgment。
- BE-001WV-01 `root.contracts.core_ir.v4_contracts.core_ir_compat_bridge` root.contracts.core_ir.v4_contracts.core_ir_compat_bridge parent residual judgment closes parent；下一步: BE-001WW-01 root.contracts.core_ir.v4_contracts parent_residual_judgment。
- BE-001WW-01 `root.contracts.core_ir.v4_contracts` root.contracts.core_ir.v4_contracts parent residual judgment closes parent；下一步: BE-001WX-01 root.contracts.core_ir parent_residual_judgment。
- BE-001WX-01 `root.contracts.core_ir` root.contracts.core_ir parent residual judgment closes parent；下一步: BE-001WY-01 root.contracts parent_residual_judgment selects contracts.compiler_bridge。
- BE-001WY-01 `root.contracts` root.contracts parent residual judgment selects contracts.compiler_bridge；下一步: BE-001WZ-01 root.contracts.compiler_bridge baseline_plan。
- BE-001WZ-01 `root.contracts.compiler_bridge` root.contracts.compiler_bridge equivalence baseline and split plan；下一步: BE-001XA-01 root.contracts.compiler_bridge parent_residual_judgment selects runtime_protocol_validation。
- BE-001XA-01 `root.contracts.compiler_bridge` root.contracts.compiler_bridge parent residual judgment selects runtime_protocol_validation；下一步: BE-001XB-01 root.contracts.compiler_bridge.runtime_protocol_validation baseline_plan。
- BE-001XB-01 `root.contracts.compiler_bridge.runtime_protocol_validation` root.contracts.compiler_bridge.runtime_protocol_validation equivalence baseline and extraction plan；下一步: BE-001XC-01 root.contracts.compiler_bridge.runtime_protocol_validation extract_closeout。
- BE-001XC-01 `root.contracts.compiler_bridge.runtime_protocol_validation` root.contracts.compiler_bridge.runtime_protocol_validation actual extraction complete；下一步: BE-001XD-01 root.contracts.compiler_bridge.runtime_protocol_validation single_leaf_closeout。
- BE-001XD-01 `root.contracts.compiler_bridge.runtime_protocol_validation` root.contracts.compiler_bridge.runtime_protocol_validation single leaf closeout stops split；下一步: BE-001XE-01 root.contracts.compiler_bridge parent_residual_judgment。
- BE-001XE-01 `root.contracts.compiler_bridge` root.contracts.compiler_bridge parent residual judgment selects runtime_protocol_lowering；下一步: BE-001XF-01 root.contracts.compiler_bridge.runtime_protocol_lowering baseline_plan。
- BE-001XF-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering equivalence baseline and extraction plan；下一步: BE-001XG-01 root.contracts.compiler_bridge.runtime_protocol_lowering extract_closeout。
- BE-001XG-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering actual extraction complete；下一步: BE-001XH-01 root.contracts.compiler_bridge.runtime_protocol_lowering single_leaf_closeout。
- BE-001XH-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering single leaf closeout keeps split open；下一步: BE-001XI-01 root.contracts.compiler_bridge.runtime_protocol_lowering parent_residual_judgment。
- BE-001XI-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering parent residual judgment selects intent_signal_lowering；下一步: BE-001XJ-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering baseline_plan。
- BE-001XJ-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering equivalence baseline and extraction plan；下一步: BE-001XK-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering extract_closeout。
- BE-001XK-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering actual extraction complete；下一步: BE-001XL-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering single_leaf_closeout。
- BE-001XL-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering single leaf closeout keeps split open；下一步: BE-001XM-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering parent_residual_judgment。
- BE-001XM-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering parent residual judgment selects condition_lowering；下一步: BE-001XN-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering baseline_plan。
- BE-001XN-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering equivalence baseline and extraction plan；下一步: BE-001XO-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering extract_closeout。
- BE-001XO-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering actual extraction complete；下一步: BE-001XP-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering single_leaf_closeout。
- BE-001XP-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering` root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.condition_lowering single leaf closeout stops split；下一步: BE-001XQ-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering parent_residual_judgment。
- BE-001XQ-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` parent residual judgment selects fallback_description；下一步: BE-001XR-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description baseline_plan。
- BE-001XR-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description` equivalence baseline and extraction plan；下一步: BE-001XR-02 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description extract_closeout。
- BE-001XR-02 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description` actual extraction complete；下一步: BE-001XR-03 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description single_leaf_closeout。
- BE-001XR-03 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering.fallback_description` single leaf closeout stops split；下一步: BE-001XS-01 root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering parent_residual_judgment。
- BE-001XS-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.intent_signal_lowering` parent residual judgment closes parent；下一步: BE-001XT-01 root.contracts.compiler_bridge.runtime_protocol_lowering parent_residual_judgment。
- BE-001XT-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` parent residual judgment selects agent_policy_lowering；下一步: BE-001XU-01 root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering baseline_plan。
- BE-001XU-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering` equivalence baseline and extraction plan；下一步: BE-001XU-02 root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering extract_closeout。
- BE-001XU-02 `root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering` actual extraction complete；下一步: BE-001XU-03 root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering single_leaf_closeout。
- BE-001XU-03 `root.contracts.compiler_bridge.runtime_protocol_lowering.agent_policy_lowering` single leaf closeout stops split；下一步: BE-001XV-01 root.contracts.compiler_bridge.runtime_protocol_lowering parent_residual_judgment。
- BE-001XV-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` parent residual judgment selects risk_policy_lowering；下一步: BE-001XW-01 root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering baseline_plan。
- BE-001XW-01 `root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering` equivalence baseline and extraction plan；下一步: BE-001XW-02 root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering extract_closeout。
- BE-001XW-02 `root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering` actual extraction complete；下一步: BE-001XW-03 root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering single_leaf_closeout。
- BE-001XW-03 `root.contracts.compiler_bridge.runtime_protocol_lowering.risk_policy_lowering` single leaf closeout stops split；下一步: BE-001XX-01 root.contracts.compiler_bridge.runtime_protocol_lowering parent_residual_judgment。
- BE-001XX-01 `root.contracts.compiler_bridge.runtime_protocol_lowering` parent residual judgment closes parent；下一步: BE-001XY-01 root.contracts.compiler_bridge parent_residual_judgment。
- BE-001XY-01 `root.contracts.compiler_bridge` parent residual judgment selects strategy_ir_lowering；下一步: BE-001XZ-01 root.contracts.compiler_bridge.strategy_ir_lowering baseline_plan。
- BE-001XZ-01 `root.contracts.compiler_bridge.strategy_ir_lowering` equivalence baseline and extraction plan；下一步: BE-001YA-01 root.contracts.compiler_bridge.strategy_ir_lowering extract_closeout。
- BE-001YA-01 `root.contracts.compiler_bridge.strategy_ir_lowering` extract closeout complete；下一步: BE-001YA-02 root.contracts.compiler_bridge.strategy_ir_lowering single_leaf_closeout。
- BE-001YA-02 `root.contracts.compiler_bridge.strategy_ir_lowering` single leaf closeout stops split；下一步: BE-001YB-01 root.contracts.compiler_bridge parent_residual_judgment。
- BE-001YB-01 `root.contracts.compiler_bridge` parent residual judgment closes parent；下一步: BE-001YC-01 root.contracts parent_residual_judgment。
Latest recursive supplement: BE-001YC-01 selected `contracts.runtime_support` as the next `root.contracts` child. The next step is BE-001YD-01 `root.contracts.runtime_support` baseline_plan over the `qrpc-runtime` crate boundary.
Latest recursive supplement: BE-001YD-01 froze the `root.contracts.runtime_support` baseline over the `qrpc-runtime` crate and queued runtime facade/coordinator, data, intent, agent, evaluator, execution, fill, risk, sandbox, v4, live, plugin, ops compatibility, and metric children.
Latest recursive supplement: BE-001YE-01 selected `runtime_facade_coordinator` as the first `root.contracts.runtime_support` child; it owns the `qrpc_runtime/src/lib.rs` facade, `RuntimeCoordinator`, provider wiring, session orchestration, and crate-root coordinator tests.
Latest recursive supplement: BE-001YF-01 froze the `runtime_facade_coordinator` baseline; next movement may extract `RuntimeCoordinator`, `ConfigGenerationEntry`, coordinator helpers, and coordinator tests into `qrpc_runtime/src/runtime_facade_coordinator.rs` while preserving the crate-root facade.
Latest recursive supplement: BE-001YF-02 extracted `runtime_facade_coordinator`; `qrpc_runtime/src/runtime_facade_coordinator.rs` now owns `RuntimeCoordinator`, `ConfigGenerationEntry`, coordinator helpers, and coordinator tests while `qrpc_runtime/src/lib.rs` keeps the crate facade.
Latest recursive supplement: BE-001YF-03 kept `runtime_facade_coordinator` open with `continue_split: true`; next child candidate is `constructor_provider_wiring`.
Latest recursive supplement: BE-001YG-01 selected `constructor_provider_wiring` under `runtime_facade_coordinator`; next step is its baseline over `RuntimeCoordinator` constructors and provider injection APIs.
Latest recursive supplement: BE-001YH-01 froze the `constructor_provider_wiring` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/constructor_provider_wiring.rs` and move only the constructor/provider `impl RuntimeCoordinator` methods.
Latest recursive supplement: BE-001YH-02 extracted `constructor_provider_wiring`; `qrpc_runtime/src/runtime_facade_coordinator/constructor_provider_wiring.rs` now owns the public constructor/provider injection methods while the parent keeps runtime behavior.
Latest recursive supplement: BE-001YH-03 closed `constructor_provider_wiring` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YI-01 selected `session_cycle_orchestration`; next baseline covers `run_session`, `run_slow_cycle`, `run_fast_cycle`, and `run_cycle` orchestration.
Latest recursive supplement: BE-001YJ-01 froze the `session_cycle_orchestration` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/session_cycle_orchestration.rs` and move only session/cycle orchestration methods.
Latest recursive supplement: BE-001YJ-02 extracted `session_cycle_orchestration`; `qrpc_runtime/src/runtime_facade_coordinator/session_cycle_orchestration.rs` now owns session/cycle orchestration while provider helper implementations remain in the parent.
Latest recursive supplement: BE-001YJ-03 closed `session_cycle_orchestration` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YK-01 selected `execution_market_entrypoints`; next baseline covers `submit_execution_plan` and `on_market_data`.
Latest recursive supplement: BE-001YL-01 froze the `execution_market_entrypoints` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/execution_market_entrypoints.rs` and move only the two public entrypoint methods.
Latest recursive supplement: BE-001YL-02 extracted `execution_market_entrypoints`; `qrpc_runtime/src/runtime_facade_coordinator/execution_market_entrypoints.rs` now owns execution and market-data public entrypoints.
Latest recursive supplement: BE-001YL-03 closed `execution_market_entrypoints` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YM-01 selected `state_config_accessors`; next baseline covers public state/provider/risk accessors while excluding config generation methods.
Latest recursive supplement: BE-001YN-01 froze the `state_config_accessors` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/state_config_accessors.rs` and move only accessor/control methods.
Latest recursive supplement: BE-001YN-02 extracted `state_config_accessors`; `qrpc_runtime/src/runtime_facade_coordinator/state_config_accessors.rs` now owns public state/provider/risk/execution-assumption accessors.
Latest recursive supplement: BE-001YN-03 closed `state_config_accessors` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YO-01 selected `config_generation`; next baseline covers module config swap/apply and generation history accessors.
Latest recursive supplement: BE-001YP-01 froze the `config_generation` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/config_generation.rs` and move only config generation methods.
Latest recursive supplement: BE-001YP-02 extracted `config_generation`; `qrpc_runtime/src/runtime_facade_coordinator/config_generation.rs` now owns module config swap/apply and generation history methods.
Latest recursive supplement: BE-001YP-03 closed `config_generation` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YQ-01 selected `provider_delegation_helpers`; next baseline covers data, intent, agent, merge, risk, execution, and open-order provider helper methods.
Latest recursive supplement: BE-001YR-01 froze the `provider_delegation_helpers` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/provider_delegation_helpers.rs` and move only provider helper methods.
Latest recursive supplement: BE-001YR-02 extracted `provider_delegation_helpers`; `qrpc_runtime/src/runtime_facade_coordinator/provider_delegation_helpers.rs` now owns the provider helper chain.
Latest recursive supplement: BE-001YR-03 closed `provider_delegation_helpers` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YS-01 selected `portfolio_projection`; next baseline covers portfolio update events, quote maps, mark-price refresh, exposure projection, and equity estimates.
Latest recursive supplement: BE-001YT-01 froze the `portfolio_projection` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/portfolio_projection.rs` and move only portfolio projection methods.
Latest recursive supplement: BE-001YT-02 extracted `portfolio_projection`; `qrpc_runtime/src/runtime_facade_coordinator/portfolio_projection.rs` now owns portfolio event projection and mark-to-market refresh.
Latest recursive supplement: BE-001YT-03 closed `portfolio_projection` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent residual judgment.
Latest recursive supplement: BE-001YU-01 selected `coordinator_test_harness`; next baseline covers coordinator integration tests, noop providers, reject-all risk fixture, and sample config.
Latest recursive supplement: BE-001YV-01 froze the `coordinator_test_harness` baseline; next movement may add `qrpc_runtime/src/runtime_facade_coordinator/coordinator_test_harness.rs` and move only test harness code.
Latest recursive supplement: BE-001YV-02 extracted `coordinator_test_harness`; `qrpc_runtime/src/runtime_facade_coordinator/coordinator_test_harness.rs` now owns coordinator-specific test fixtures and integration tests.
Latest recursive supplement: BE-001YV-03 closed `coordinator_test_harness` with `stop_split: true`; next recursive step returns to `runtime_facade_coordinator` parent closeout.
Latest recursive supplement: BE-001YW-01 closed `runtime_facade_coordinator` as a compact parent facade; next recursive step returns to `root.contracts.runtime_support` parent residual judgment.
Latest recursive supplement: BE-001YX-01 selected `data_module`; next baseline covers provider facade, source mapping, diagnostics/quality, exchange parsing, mock data, historical cache, and tests.
Latest recursive supplement: BE-001YY-01 froze the `data_module` baseline; next recursive step selects the first data-module child.
Latest recursive supplement: BE-001YZ-01 selected `data_module.source_mapping`; next baseline covers Core IR data binding to runtime data source config restoration.
Latest recursive supplement: BE-001ZA-01 froze the `data_module.source_mapping` baseline; next movement may add a source mapping child module without changing call sites.
Latest recursive supplement: BE-001ZA-02 extracted `data_module.source_mapping`; `qrpc_runtime/src/data_module/source_mapping.rs` now owns Core IR source mapping and hint parsing.
Latest recursive supplement: BE-001ZA-03 closed `data_module.source_mapping` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment.
Latest recursive supplement: BE-001ZB-01 selected `data_module.quality_diagnostics`; next baseline covers quality snapshots, health flags, previews, and summaries.
Latest recursive supplement: BE-001ZC-01 froze the `data_module.quality_diagnostics` baseline; next movement may add a quality diagnostics child module while preserving parent-visible helpers.
Latest recursive supplement: BE-001ZC-02 extracted `data_module.quality_diagnostics`; `qrpc_runtime/src/data_module/quality_diagnostics.rs` now owns quality snapshots, health flags, previews, summaries, and cached snapshot status refresh.
Latest recursive supplement: BE-001ZC-03 closed `data_module.quality_diagnostics` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment.
Latest recursive supplement: BE-001ZD-01 selected `data_module.collection_orchestration`; next baseline covers provider entrypoint, source deduplication, data collection composition, and runtime event assembly.
Latest governance supplement: GOV-RECURSIVE-COST-CONTROL-01 upgraded recursive speed governance to `recursive-high-speed-v2`; the Rust cursor remains BE-001ZE-01, and future same-parent waves must keep child-level white-box rows, split decisions, residuals, and forced precision downgrade triggers.
Latest governance supplement: GOV-LEAF-GRANULARITY-SMART-JUDGE-01 adds terminal-leaf scoring to recursive governance; bottom leaves must resolve to STOP/WAVE/SPLIT/PRECISION from split benefit, leaf size fit, risk, governance cost, and system efficiency before further split.
Latest recursive supplement: BE-001ZE-01 froze `data_module.collection_orchestration` baseline; leaf granularity smart judge resolves to `SPLIT`, and the next movement may extract only the provider collect implementation.
Latest recursive supplement: BE-001ZE-02 extracted `data_module.collection_orchestration`; `qrpc_runtime/src/data_module/collection_orchestration.rs` now owns the provider collect transaction.
Latest recursive supplement: BE-001ZE-03 closed `data_module.collection_orchestration` with `stop_split: true`; next data-module residual candidate is `exchange_endpoints`.
Latest recursive supplement: BE-001ZF-01 selected `data_module.exchange_surface_wave`; the next baseline should batch `exchange_endpoints` and `exchange_payload_parsing` instead of creating a standalone endpoint micro leaf.
Latest recursive supplement: BE-001ZG-01 froze `data_module.exchange_surface_wave` baseline; next movement may extract endpoint/provider helpers and OKX/Binance payload parsers into one exchange surface child.
Latest recursive supplement: BE-001ZG-02 extracted `data_module.exchange_surface_wave`; `qrpc_runtime/src/data_module/exchange_surface.rs` now owns exchange endpoint/provider helpers and OKX/Binance raw payload parsing.
Latest governance supplement: GOV-LEAF-GRANULARITY-JUDGE-TOOL-01 adds `tools/evaluate-leaf-granularity.ps1`; bottom-leaf closeout now has read-only `normalized_split_score` evidence for STOP/WAVE/SPLIT/PRECISION.
Latest recursive supplement: BE-001ZG-03 closed `data_module.exchange_surface_wave` with `stop_split: true`; the next recursive step returns to `data_module` parent residual judgment, expected candidate `normalization`.
Latest recursive supplement: BE-001ZH-01 selected `data_module.normalization`; next baseline covers raw-to-normalized Kline/Quote conversion while excluding mock, historical cache, HTTP transport, in-memory cache, provider orchestration, and tests.
Latest recursive supplement: BE-001ZI-01 froze the `data_module.normalization` baseline; next movement may add `qrpc_runtime/src/data_module/normalization.rs` and move only normalization helpers.
Latest recursive supplement: BE-001ZI-02 extracted `data_module.normalization`; `qrpc_runtime/src/data_module/normalization.rs` now owns raw-to-normalized Kline/Quote conversion and quote snapshot construction.
Latest governance supplement: GOV-TERMINAL-LEAF-CONTROL-V2-01 integrates the read-only over-splitting and governance-cost findings into `terminal_leaf_control_v2`; the leaf judge now emits `terminal_leaf_control.governance_mode`, and standalone full leaf governance is reserved for `precision_single_leaf`.
Latest governance supplement: GOV-GOVERNANCE-NEXT-OPTIMIZATION-01 separates `split_decision` from `governance_packaging`, forces oversized high-risk leaves into precision baseline, adds QPCursor draft generation, checks untracked active files in full-feature-tree, and records index reduction as a pre-promote route.
Latest recursive supplement: BE-001ZI-03 closed `data_module.normalization` with `stop_split: true`; terminal leaf control score is 30, and the next recursive step returns to `data_module` parent residual judgment with expected candidate `mock_data_generation`.
Latest recursive supplement: BE-001ZJ-01 selected `data_module.mock_data_generation`; next baseline covers mock volatility config, deterministic pseudo-random generation, raw mock quote/kline generation, and backtest mock bars.
Latest recursive supplement: BE-001ZK-01 froze the `data_module.mock_data_generation` baseline; next movement may add the planned mock data generation child module and move only the mock generation surface.
Latest recursive supplement: BE-001ZK-02 extracted `data_module.mock_data_generation`; `qrpc_runtime/src/data_module/mock_data_generation.rs` now owns deterministic mock quote/kline generation and backtest mock bars.
Latest recursive supplement: BE-001ZK-03 closed `data_module.mock_data_generation` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment with expected candidate `historical_cache`.
Latest recursive supplement: BE-001ZL-01 selected `data_module.historical_cache`; next baseline covers historical cache path safety, fresh/stale load, persistence, historical raw kline fetch, and backtest historical bars.
Latest recursive supplement: BE-001ZM-01 froze the `data_module.historical_cache` baseline; next movement may add the planned historical cache child module and move only the historical cache surface.
Latest recursive supplement: BE-001ZM-02 extracted `data_module.historical_cache`; `qrpc_runtime/src/data_module/historical_cache.rs` now owns historical replay cache load, persistence, stale fallback, and historical kline fetch orchestration.
Latest recursive supplement: BE-001ZM-03 closed `data_module.historical_cache` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment with expected candidate `http_transport`.
Latest recursive supplement: BE-001ZN-01 selected `data_module.http_transport`; next baseline covers ping probing, OKX live fetch, JSON request retry/fallback, Tokio blocking bridge, and Windows PowerShell fallback.
Latest recursive supplement: BE-001ZO-01 froze the `data_module.http_transport` baseline; next movement may add the planned transport child module while preserving parent-owned client/cache/breaker orchestration and parent-mediated historical-cache transport helper reuse.
Latest recursive supplement: BE-001ZO-02 extracted `data_module.http_transport`; `qrpc_runtime/src/data_module/http_transport.rs` now owns ping probing, OKX live fetch, JSON retry/fallback, Tokio blocking bridge, and Windows PowerShell fallback.
Latest recursive supplement: BE-001ZO-03 closed `data_module.http_transport` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment with expected candidate `test_harness` or parent closeout.
Latest recursive supplement: BE-001ZP-01 selected `data_module.test_harness`; next baseline covers the local unit-test fixtures/assertions currently embedded in `qrpc_runtime/src/data_module.rs`.
Latest recursive supplement: BE-001ZQ-01 froze the `data_module.test_harness` baseline; next movement may add the planned test harness child file and move only local unit tests and fixtures.
Latest recursive supplement: BE-001ZQ-02 extracted `data_module.test_harness`; `qrpc_runtime/src/data_module/test_harness.rs` now owns local data-module fixtures and assertions.
Latest recursive supplement: BE-001ZQ-03 closed `data_module.test_harness` with `stop_split: true`; next recursive step returns to `data_module` parent residual judgment for parent closeout.
Latest recursive supplement: BE-001ZR-01 closed `data_module` as a parent; next recursive step returns to `runtime_support` parent residual judgment.
Latest recursive supplement: BE-001ZS-01 selected `runtime_support.intent_module`; next baseline covers runtime intent generation from normalized data and Core IR signal semantics.
Latest recursive supplement: BE-001ZT-01 froze `runtime_support.intent_module` baseline with terminal STOP decision; next recursive step is single leaf closeout.
Latest recursive supplement: BE-001ZT-02 closed `runtime_support.intent_module` with `stop_split: true`; next recursive step returns to `runtime_support` parent residual judgment with expected candidate `agent_module`.
Latest recursive supplement: BE-001ZU-01 selected `runtime_support.agent_module`; next baseline covers weighted signals, portfolio rebalance, cross-venue arbitrage, shared scoring/portfolio helpers, and local tests.
Latest recursive supplement: BE-001ZV-01 froze `runtime_support.agent_module` baseline; child queue starts with `weighted_signal_decisions` and keeps public provider dispatch parent-mediated.
Latest recursive supplement: BE-001ZW-01 selected `agent_module.weighted_signal_decisions`; next baseline covers slow-cycle weighted signal decisions and proposed actions.
Latest recursive supplement: BE-001ZX-01 froze `agent_module.weighted_signal_decisions` baseline; next movement may add a child file and move only weighted signal decision flow plus grouping helper.
Latest recursive supplement: BE-001ZX-02 extracted `agent_module.weighted_signal_decisions`; weighted signal decision construction and grouping now live in `qrpc_runtime/src/agent_module/weighted_signal_decisions.rs`.
Latest recursive supplement: BE-001ZX-03 closed `agent_module.weighted_signal_decisions` with `stop_split: true`; next recursive step returns to `agent_module` parent residual judgment with expected candidate `portfolio_rebalance`.
Latest recursive supplement: BE-001ZY-01 selected `agent_module.portfolio_rebalance`; next baseline covers cadence gates, rebalance universe, target allocation, current weights, and portfolio target DTO assembly.
Latest recursive supplement: BE-001ZZ-01 froze `agent_module.portfolio_rebalance` baseline; next movement may add a child file and move only cadence, rebalance plan, target decision, and target-weight assignment logic.
Latest recursive supplement: BE-001ZZ-02 extracted `agent_module.portfolio_rebalance`; rebalance cadence and target allocation now live in `qrpc_runtime/src/agent_module/portfolio_rebalance.rs`.
Latest recursive supplement: BE-001ZZ-03 closed `agent_module.portfolio_rebalance` with `stop_split: true`; next recursive step returns to `agent_module` parent residual judgment with expected candidate `cross_venue_arbitrage`.
Latest recursive supplement: BE-002AA-01 selected `agent_module.cross_venue_arbitrage`; next baseline covers fast-cycle quote arbitrage, explicit spread-signal arbitrage, spread triggers, cost buffers, leg selection, and sell-side availability clamps.
Latest recursive supplement: BE-002AB-01 froze `agent_module.cross_venue_arbitrage` baseline; next movement may add a child file and move only arbitrage decision construction plus total cost buffer calculation.
Latest recursive supplement: BE-002AB-02 extracted `agent_module.cross_venue_arbitrage`; arbitrage decision construction and total cost buffer calculation now live in `qrpc_runtime/src/agent_module/cross_venue_arbitrage.rs`.
Latest recursive supplement: BE-002AB-03 closed `agent_module.cross_venue_arbitrage` with `stop_split: true`; next recursive step returns to `agent_module` parent residual judgment with expected candidate `shared_scoring_portfolio_helpers`.
Latest recursive supplement: BE-002AC-01 selected `agent_module.shared_scoring_portfolio_helpers`; next baseline must decide whether helper ownership stays parent-owned or moves behind parent-mediated access.
Latest recursive supplement: BE-002AD-01 keeps `agent_module.shared_scoring_portfolio_helpers` parent-owned; no helper child file is opened because parent-mediated access is already the cleanest boundary.
Latest recursive supplement: BE-002AD-02 closed `agent_module.shared_scoring_portfolio_helpers` with `stop_split: true`; next recursive step returns to `agent_module` parent residual judgment with expected candidate `test_harness`.
Latest recursive supplement: BE-002AE-01 selected `agent_module.test_harness`; next baseline covers local agent-module tests and fixtures while excluding production code.
Latest recursive supplement: BE-002AF-01 froze `agent_module.test_harness` baseline; next movement may add a test-only child file and move only the local test module.
Latest recursive supplement: BE-002AF-02 extracted `agent_module.test_harness`; local fixtures and agent-module tests now live in `qrpc_runtime/src/agent_module/test_harness.rs`.
Latest recursive supplement: BE-002AF-03 closed `agent_module.test_harness` with `stop_split: true`; next recursive step returns to `agent_module` parent closeout judgment.
Latest recursive supplement: BE-002AG-01 closed `agent_module` as a parent; weighted signal decisions, portfolio rebalance, cross-venue arbitrage, shared helpers, and test harness are closed, and the next runtime-support residual candidate is `core_ir_evaluator`.
Latest recursive supplement: BE-002AH-01 selected `runtime_support.core_ir_evaluator`; next baseline freezes indicator registry, builtin indicator families, custom/spread expression evaluation, series utilities, and local evaluator tests.
Latest recursive supplement: BE-002AI-01 froze `core_ir_evaluator` baseline; first child queue uses same-parent waves for classic indicators, advanced indicators, spread/custom expression evaluation, shared lookup/math helpers, and test harness.
Latest recursive supplement: BE-002AJ-01 selected `core_ir_evaluator.classic_indicator_wave`; next baseline covers MA family, RSI, MACD, Momentum, ZScore, and QuoteObserve evaluator entrypoints.
Latest recursive supplement: BE-002AK-01 froze `classic_indicator_wave` baseline; next movement may add `qrpc_runtime/src/core_ir_evaluator/classic_indicator_wave.rs` and move only the six classic evaluator entrypoints.
Latest recursive supplement: BE-002AK-02 extracted `classic_indicator_wave`; `qrpc_runtime/src/core_ir_evaluator/classic_indicator_wave.rs` now owns MA family, RSI, MACD, Momentum, ZScore, and QuoteObserve evaluator entrypoints.
Latest recursive supplement: BE-002AK-03 closed `classic_indicator_wave` with `stop_split: true`; next recursive step returns to `core_ir_evaluator` parent residual judgment with expected candidate `advanced_indicator_wave`.
Latest recursive supplement: BE-002AL-01 selected `core_ir_evaluator.advanced_indicator_wave`; next baseline covers ATR, Bollinger Bands, OBV, CMF, ADX, Stochastic, CCI, Parabolic SAR, Keltner Channel, and Donchian Channel evaluator entrypoints.
Latest recursive supplement: BE-002AM-01 froze `advanced_indicator_wave` baseline; next movement may add the advanced indicator child file and move only ten advanced evaluator entrypoints.
Latest recursive supplement: BE-002AM-02 extracted `advanced_indicator_wave`; `qrpc_runtime/src/core_ir_evaluator/advanced_indicator_wave.rs` now owns ATR, Bollinger Bands, OBV, CMF, ADX, Stochastic, CCI, Parabolic SAR, Keltner Channel, and Donchian Channel evaluator entrypoints.
Latest recursive supplement: BE-002AM-03 closed `advanced_indicator_wave` with `stop_split: true`; next recursive step returns to `core_ir_evaluator` parent residual judgment with expected candidate `spread_custom_expression_wave`.
Latest recursive supplement: BE-002AN-01 selected `core_ir_evaluator.spread_custom_expression_wave`; next baseline covers Custom evaluator, legacy/typed Spread evaluator, series expression machinery, and custom value expression helpers.
Latest recursive supplement: BE-002AO-01 froze `spread_custom_expression_wave` baseline; next movement may add the expression wave child file and move Spread/Custom expression evaluation machinery only.
Latest recursive supplement: BE-002AO-02 extracted `spread_custom_expression_wave`; `qrpc_runtime/src/core_ir_evaluator/spread_custom_expression_wave.rs` now owns Custom, legacy Spread, typed Spread, series expression materialization, aggregation, alignment, scope discovery, and reference-price helpers.
Latest recursive supplement: BE-002AO-03 closed `spread_custom_expression_wave` with `stop_split: true`; next recursive step returns to `core_ir_evaluator` parent residual judgment with expected candidate `shared_lookup_math_helpers`.
Latest recursive supplement: BE-002AP-01 selected `core_ir_evaluator.shared_lookup_math_helpers`; next baseline decides whether shared lookup/math helpers remain parent-owned or move behind parent-mediated access.
Latest recursive supplement: BE-002AQ-01 keeps `core_ir_evaluator.shared_lookup_math_helpers` parent-owned; no child file is opened because extraction would add sibling-helper pressure or wrapper churn.
Latest recursive supplement: BE-002AQ-02 closed `core_ir_evaluator.shared_lookup_math_helpers` with `stop_split: true`; next recursive step returns to `core_ir_evaluator` parent residual judgment with expected candidate `test_harness`.
Latest recursive supplement: BE-002AR-01 selected `core_ir_evaluator.test_harness`; next baseline should move only local evaluator tests while production registry, helpers, and evaluator children remain parent-owned.
Latest recursive supplement: BE-002AS-01 froze `core_ir_evaluator.test_harness` baseline; next movement may add a test-only child file and move only local evaluator tests and fixtures.
Latest recursive supplement: BE-002AS-02 extracted `core_ir_evaluator.test_harness`; local evaluator tests now live in `qrpc_runtime/src/core_ir_evaluator/test_harness.rs`.
Latest recursive supplement: BE-002AS-03 closed `core_ir_evaluator.test_harness` with `stop_split: true`; next recursive step returns to `core_ir_evaluator` parent closeout judgment.
Latest recursive supplement: BE-002AT-01 closed `core_ir_evaluator` as a parent; the next runtime-support residual candidate is `execution_module`.
Latest recursive supplement: BE-002AU-01 selected `runtime_support.execution_module`; next baseline freezes execution provider facade, order planning, payload semantics, market/portfolio helpers, and tests.
Latest recursive supplement: BE-002AV-01 froze `execution_module` baseline; only the local test harness is opened as a child while production provider/planning/helper ownership remains parent-owned.
Latest recursive supplement: BE-002AW-01 selected `execution_module.test_harness`; next baseline should move only local execution-module tests and fixtures.
Latest recursive supplement: BE-002AX-01 froze `execution_module.test_harness` baseline; next movement may add a test-only child file and move only local execution-module tests.
Latest recursive supplement: BE-002AX-02 extracted `execution_module.test_harness`; local execution-module tests now live in `qrpc_runtime/src/execution_module/test_harness.rs`.
Latest recursive supplement: BE-002AX-03 closed `execution_module.test_harness` with `stop_split: true`; next recursive step returns to `execution_module` parent closeout judgment.
Latest recursive supplement: BE-002AY-01 closed `execution_module` as a parent; the next runtime-support residual candidate is `fill_engine`.
Latest recursive supplement: BE-002AZ-01 selected `runtime_support.fill_engine`; next baseline freezes fill lifecycle, event projection, reservation/portfolio accounting, fill report helpers, and tests.
Latest recursive supplement: BE-002BA-01 froze `fill_engine` baseline; next recursive step selects `event_projection_wave` as the lowest-risk same-parent extraction.
Latest recursive supplement: BE-002BB-01 selected `fill_engine.event_projection_wave`; next baseline covers only open, partial, cancel, reject, and fill event builders.
Latest recursive supplement: BE-002BC-01 froze `event_projection_wave` baseline; next movement may add the child file and move only the five event builders.
Latest recursive supplement: BE-002BD-01 extracted `event_projection_wave`; `qrpc_runtime/src/fill_engine/event_projection_wave.rs` now owns fill-engine event builders.
Latest recursive supplement: BE-002BD-02 closed `event_projection_wave` with `stop_split: true`; next recursive step returns to `fill_engine` parent residual judgment with expected candidate `portfolio_reservation_accounting`.
Latest recursive supplement: BE-002BE-01 selected `fill_engine.portfolio_reservation_accounting`; next baseline decides the accounting helper movement and public `apply_fill_to_portfolio` handling.
Latest recursive supplement: BE-002BF-01 froze `portfolio_reservation_accounting` baseline; next movement may move six private accounting helpers while public `apply_fill_to_portfolio` remains parent-owned.
Latest recursive supplement: BE-002BG-01 extracted `portfolio_reservation_accounting`; `qrpc_runtime/src/fill_engine/portfolio_reservation_accounting.rs` now owns private reservation/accounting helpers.
Latest recursive supplement: BE-002BG-02 closed `portfolio_reservation_accounting` with `stop_split: true`; next recursive step returns to `fill_engine` parent residual judgment with expected candidate `fill_report_execution_helpers`.
Latest recursive supplement: BE-002BH-01 selected `fill_engine.fill_report_execution_helpers`; next baseline decides helper movement for marketability, assumptions, order reconstruction, and fill-report construction.
Latest recursive supplement: BE-002BI-01 froze `fill_report_execution_helpers` baseline; next movement may move nine private execution/fill-report helpers.
Latest recursive supplement: BE-002BJ-01 extracted `fill_report_execution_helpers`; `qrpc_runtime/src/fill_engine/fill_report_execution_helpers.rs` now owns private execution and fill-report helpers.
Latest recursive supplement: BE-002BJ-02 closed `fill_report_execution_helpers` with `stop_split: true`; next recursive step returns to `fill_engine` parent residual judgment with expected candidate `test_harness`.
Latest recursive supplement: BE-002BK-01 selected `fill_engine.test_harness`; next baseline should move only local fill-engine tests and fixtures while production orchestration and public portfolio mutation remain parent-owned.
Latest recursive supplement: BE-002BL-01 froze `fill_engine.test_harness` baseline; next movement may add a test-only child module and move only local fill-engine tests.
Latest recursive supplement: BE-002BM-01 extracted `fill_engine.test_harness`; `qrpc_runtime/src/fill_engine/test_harness.rs` now owns local fill-engine tests and fixtures.
Latest recursive supplement: BE-002BM-02 closed `fill_engine.test_harness` with `stop_split: true`; all planned `fill_engine` children are now closed.
Latest recursive supplement: BE-002BN-01 closed `runtime_support.fill_engine` as a parent; next recursive step returns to `runtime_support` parent residual judgment.
Latest recursive supplement: BE-002BO-01 selected `runtime_support.risk_support`; next baseline should freeze the `qrpc_runtime/src/risk_checker.rs` risk-checking boundary and preserve runtime facade/provider separation.
Latest recursive supplement: BE-002BP-01 froze `risk_support` baseline; a limited same-parent wave queue opens event payload projection, direction/cross-symbol constraints, action clamps, portfolio target clamps, exposure math, and test harness while public provider/evaluate surface remains parent-owned.
Latest recursive supplement: BE-002BQ-01 selected `risk_support.event_payload_projection`; next baseline should move only event payload, reason summary, and stats projection helpers.
Latest recursive supplement: BE-002BR-01 froze `risk_support.event_payload_projection` baseline; next movement may add a projection child file and move only five event/summary/stats helpers.
Latest recursive supplement: BE-002BS-01 extracted `risk_support.event_payload_projection`; `qrpc_runtime/src/risk_checker/event_payload_projection.rs` now owns risk event payload, reason summary, and stats projection helpers.
Latest recursive supplement: BE-002BS-02 closed `risk_support.event_payload_projection` with `stop_split: true`; next recursive step returns to `risk_support` parent residual judgment with expected candidate `direction_cross_constraints`.
Latest recursive supplement: BE-002BT-01 selected `risk_support.direction_cross_constraints`; next baseline should move only direction-conflict and cross-symbol constraint passes.
Latest recursive supplement: BE-002BU-01 froze `risk_support.direction_cross_constraints` baseline; next movement may add a constraint child file and move only two post-processing helpers.
Latest recursive supplement: BE-002BV-01 extracted `risk_support.direction_cross_constraints`; `qrpc_runtime/src/risk_checker/direction_cross_constraints.rs` now owns direction-conflict and cross-symbol constraint helpers.
Latest recursive supplement: BE-002BV-02 closed `risk_support.direction_cross_constraints` with `stop_split: true`; next recursive step returns to `risk_support` parent residual judgment with expected candidate `action_clamp_helpers`.
Latest recursive supplement: BE-002BW-01 selected `risk_support.action_clamp_helpers`; next baseline should move only action-list clamp helpers while portfolio-target clamps and exposure math remain outside.
Latest recursive supplement: BE-002BX-01 froze `risk_support.action_clamp_helpers` baseline; next movement may add an action-clamp child file and move six action-list clamp helpers.
Latest recursive supplement: BE-002BY-01 extracted `risk_support.action_clamp_helpers`; `qrpc_runtime/src/risk_checker/action_clamp_helpers.rs` now owns action-list clamp helpers.
Latest recursive supplement: BE-002BY-02 closed `risk_support.action_clamp_helpers` with `stop_split: true`; next recursive step returns to `risk_support` parent residual judgment with expected candidate `portfolio_target_clamp_helpers`.
Latest recursive supplement: BE-002BZ-01 selected `risk_support.portfolio_target_clamp_helpers`; next baseline should move only portfolio-target clamp orchestration and target-weight mutation helpers.
Latest recursive supplement: BE-002CA-01 froze `risk_support.portfolio_target_clamp_helpers` baseline; next movement may add a portfolio-target clamp child file and move nine target-weight helpers.
Latest recursive supplement: BE-002CB-01 extracted `risk_support.portfolio_target_clamp_helpers`; `qrpc_runtime/src/risk_checker/portfolio_target_clamp_helpers.rs` now owns portfolio-target clamp helpers.
Latest recursive supplement: BE-002CB-02 closed `risk_support.portfolio_target_clamp_helpers` with `stop_split: true`; next recursive step returns to `risk_support` parent residual judgment with expected candidate `exposure_math_helpers`.
Latest recursive supplement: BE-002CC-01 selected `risk_support.exposure_math_helpers`; next baseline should move only shared exposure and equity math helpers.
Latest recursive supplement: BE-002CD-01 froze `risk_support.exposure_math_helpers` baseline; next movement may add an exposure math child file and move four shared math helpers.
Latest recursive supplement: BE-002CE-01 extracted `risk_support.exposure_math_helpers`; `qrpc_runtime/src/risk_checker/exposure_math_helpers.rs` now owns the shared exposure/equity math helpers.
Latest recursive supplement: BE-002CE-02 closed `risk_support.exposure_math_helpers` with `stop_split: true`; next recursive step returns to `risk_support` parent residual judgment with expected candidate `test_harness`.
Latest recursive supplement: BE-002CF-01 selected `risk_support.test_harness`; next baseline should move only local risk-checker tests and fixtures.
Latest recursive supplement: BE-002CG-01 froze `risk_support.test_harness` baseline; next movement may add a test-only child module and move only local risk-checker tests.
Latest recursive supplement: BE-002CH-01 extracted `risk_support.test_harness`; `qrpc_runtime/src/risk_checker/test_harness.rs` now owns local risk-checker tests and fixtures.
Latest recursive supplement: BE-002CH-02 closed `risk_support.test_harness` with `stop_split: true`; all planned `risk_support` children are now closed.
Latest recursive supplement: BE-002CI-01 closed `runtime_support.risk_support`; next recursive step returns to `runtime_support` parent residual judgment with expected candidate `sandbox_replay_timeline`.
Latest recursive supplement: BE-002CJ-01 selected `runtime_support.sandbox_replay_timeline`; next baseline should freeze `qrpc_runtime/src/sandbox` before any movement.
Latest recursive supplement: BE-002CK-01 froze `runtime_support.sandbox_replay_timeline` baseline; the child queue opens timeline providers, unified timeline, replay builder, sandbox surfaces, realtime/fast backtest sandbox, and tests.
Latest recursive supplement: BE-002CL-01 selected `sandbox_replay_timeline.timeline_data_providers`; next baseline should cover timeline provider implementations before unified timeline movement.
Latest recursive supplement: BE-002CM-01 froze `sandbox_replay_timeline.timeline_data_providers` baseline; next movement may move provider trait/types and interval conversion before `UnifiedTimeline`.
Latest recursive supplement: BE-002CN-01 extracted `sandbox_replay_timeline.timeline_data_providers`; `qrpc_runtime/src/sandbox/timeline_data_providers.rs` now owns provider implementations and `timeline.rs` keeps `UnifiedTimeline`.
Latest recursive supplement: BE-002CN-02 closed `sandbox_replay_timeline.timeline_data_providers` with `stop_split: true`; next recursive step returns to sandbox parent residual judgment with expected candidate `unified_timeline`.
Latest recursive supplement: BE-002CO-01 selected `sandbox_replay_timeline.unified_timeline`; next baseline should freeze `UnifiedTimeline` before movement.
Latest recursive supplement: BE-002CP-01 froze `sandbox_replay_timeline.unified_timeline` baseline; next movement may move only the `UnifiedTimeline` struct and impl.
Latest recursive supplement: BE-002CQ-01 extracted `sandbox_replay_timeline.unified_timeline`; `qrpc_runtime/src/sandbox/unified_timeline.rs` now owns `UnifiedTimeline`.
Latest recursive supplement: BE-002CQ-02 closed `sandbox_replay_timeline.unified_timeline` with `stop_split: true`; next recursive step returns to sandbox parent residual judgment with expected candidate `replay_builder`.
Latest recursive supplement: BE-002CR-01 selected `sandbox_replay_timeline.replay_builder`; next baseline should inspect the existing `replay.rs` module before any movement.
Latest recursive supplement: BE-002CS-01 closed `sandbox_replay_timeline.replay_builder` with `stop_split: true`; next recursive step returns to sandbox parent residual judgment with expected candidate `sandbox_mode_surface`.
Latest recursive supplement: BE-002CT-01 selected `sandbox_replay_timeline.sandbox_mode_surface`; next baseline should freeze public sandbox mode/control DTOs and trait.
Latest recursive supplement: BE-002CU-01 froze `sandbox_replay_timeline.sandbox_mode_surface` baseline; next movement may add a mode surface child and keep concrete sandbox implementations parent-owned.
Latest recursive supplement: BE-002CV-01 extracted `sandbox_replay_timeline.sandbox_mode_surface`; `qrpc_runtime/src/sandbox/mode_surface.rs` now owns public sandbox mode/control constants, DTOs, and trait.
Latest recursive supplement: BE-002CV-02 closed `sandbox_replay_timeline.sandbox_mode_surface` with `stop_split: true`; next recursive step returns to sandbox parent residual judgment with expected candidate `realtime_sandbox`.
Latest recursive supplement: BE-002CW-01 selected `sandbox_replay_timeline.realtime_sandbox`; next baseline should freeze `RealTimeSandbox` movement before code changes.
Latest recursive supplement: BE-002CX-01 froze `sandbox_replay_timeline.realtime_sandbox` baseline; next movement may add a realtime sandbox child and keep fast-backtest parent-owned.
Latest recursive supplement: BE-002CY-01 extracted `sandbox_replay_timeline.realtime_sandbox`; `qrpc_runtime/src/sandbox/realtime_sandbox.rs` now owns `RealTimeSandbox` and its impl blocks, and the next step is single leaf closeout.
Latest recursive supplement: BE-002CY-02 closed `sandbox_replay_timeline.realtime_sandbox` with `stop_split: true`; the next recursive step returns to sandbox parent residual judgment with expected candidate `fast_backtest_sandbox`.
Latest recursive supplement: BE-002CZ-01 selected `sandbox_replay_timeline.fast_backtest_sandbox`; next baseline should freeze only `FastBacktestSandbox` movement while tests and shared helpers stay parent-owned.
Latest recursive supplement: BE-002DA-01 froze `sandbox_replay_timeline.fast_backtest_sandbox` baseline; next movement may add a fast backtest child and move `FastBacktestSandbox` plus fast-backtest-only benchmark projection.
Latest recursive supplement: BE-002DB-01 extracted `sandbox_replay_timeline.fast_backtest_sandbox`; `qrpc_runtime/src/sandbox/fast_backtest_sandbox.rs` now owns fast backtest sandbox behavior and benchmark equity projection.
Latest recursive supplement: BE-002DB-02 closed `sandbox_replay_timeline.fast_backtest_sandbox` with `stop_split: true`; the next recursive step returns to sandbox parent residual judgment with expected candidate `test_harness`.
Latest recursive supplement: BE-002DC-01 selected `sandbox_replay_timeline.test_harness`; next baseline should move only local sandbox tests while shared helpers stay parent-owned.
Latest recursive supplement: BE-002DD-01 froze `sandbox_replay_timeline.test_harness` baseline; next movement may add a test-only child and move only local sandbox tests and fixture.
Latest recursive supplement: BE-002DE-01 extracted `sandbox_replay_timeline.test_harness`; `qrpc_runtime/src/sandbox/test_harness.rs` now owns local sandbox tests and fixture.
Latest recursive supplement: BE-002DE-02 closed `sandbox_replay_timeline.test_harness` with `stop_split: true`; all planned sandbox_replay_timeline children are closed and the next step is parent closeout.
Latest recursive supplement: BE-002DF-01 closed `sandbox_replay_timeline` as a parent; next recursive step returns to `runtime_support` parent residual judgment.
Latest recursive supplement: BE-002DG-01 selected `runtime_support.v4_runtime_support`; next baseline should freeze v4 runtime orchestration, simulated execution, DTO/type surface, and local v4 runtime tests.
Latest recursive supplement: BE-002DH-01 froze `runtime_support.v4_runtime_support` baseline; terminal leaf control resolves to `WAVE`, and the next step is v4 runtime support parent residual judgment.
Latest recursive supplement: BE-002DI-01 selected `runtime_support.v4_runtime_support.type_surface`; next baseline should freeze v4 DTOs, constants, alias, and runtime type definition.
Latest recursive supplement: BE-002DJ-01 froze `runtime_support.v4_runtime_support.type_surface` baseline; next movement may extract only the v4 type surface and preserve runtime behavior.
Latest recursive supplement: BE-002DK-01 extracted `runtime_support.v4_runtime_support.type_surface`; `qrpc_runtime/src/v4_runtime/type_surface.rs` now owns the public v4 type surface.
Latest recursive supplement: BE-002DK-02 closed `runtime_support.v4_runtime_support.type_surface` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002DL-01 selected `runtime_support.v4_runtime_support.graph_symbol_expansion`; next baseline should freeze multi-symbol v4 graph expansion.
Latest recursive supplement: BE-002DM-01 froze `runtime_support.v4_runtime_support.graph_symbol_expansion` baseline; next movement may extract only graph symbol expansion helpers.
Latest recursive supplement: BE-002DN-01 extracted `runtime_support.v4_runtime_support.graph_symbol_expansion`; `qrpc_runtime/src/v4_runtime/graph_symbol_expansion.rs` now owns multi-symbol graph expansion helpers.
Latest recursive supplement: BE-002DN-02 closed `runtime_support.v4_runtime_support.graph_symbol_expansion` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002DO-01 selected `runtime_support.v4_runtime_support.runtime_constructor_mode_gate`; next baseline should freeze constructor and runtime-mode gate behavior.
Latest recursive supplement: BE-002DP-01 froze `runtime_support.v4_runtime_support.runtime_constructor_mode_gate` baseline; next movement may extract constructor and execution-capability policy attachment methods only.
Latest recursive supplement: BE-002DQ-01 extracted `runtime_support.v4_runtime_support.runtime_constructor_mode_gate`; `qrpc_runtime/src/v4_runtime/runtime_constructor_mode_gate.rs` now owns v4 constructors and execution-capability policy attachment.
Latest recursive supplement: BE-002DQ-02 closed `runtime_support.v4_runtime_support.runtime_constructor_mode_gate` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002DR-01 selected `runtime_support.v4_runtime_support.event_replay_orchestration`; next baseline should freeze replay/input orchestration and artifact projection.
Latest recursive supplement: BE-002DS-01 froze `runtime_support.v4_runtime_support.event_replay_orchestration` baseline; next movement may extract replay/input orchestration and local output/idle helpers.
Latest recursive supplement: BE-002DT-01 extracted `runtime_support.v4_runtime_support.event_replay_orchestration`; `qrpc_runtime/src/v4_runtime/event_replay_orchestration.rs` now owns replay/input orchestration and local output helpers.
Latest recursive supplement: BE-002DT-02 closed `runtime_support.v4_runtime_support.event_replay_orchestration` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002DU-01 selected `runtime_support.v4_runtime_support.machine_transition_engine`; next baseline should freeze transition matching and application.
Latest recursive supplement: BE-002DV-01 froze `runtime_support.v4_runtime_support.machine_transition_engine` baseline; next movement may extract transition mechanics while gates and simulated execution hooks stay parent-owned.
Latest recursive supplement: BE-002DW-01 extracted `runtime_support.v4_runtime_support.machine_transition_engine`; `qrpc_runtime/src/v4_runtime/machine_transition_engine.rs` now owns transition processing and matching helpers.
Latest recursive supplement: BE-002DW-02 closed `runtime_support.v4_runtime_support.machine_transition_engine` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002DX-01 selected `runtime_support.v4_runtime_support.risk_execution_gate`; next baseline should freeze risk-plane and execution-capability gate behavior.
Latest recursive supplement: BE-002DY-01 froze `runtime_support.v4_runtime_support.risk_execution_gate` baseline; next movement may extract gate decision evaluation and recording.
Latest recursive supplement: BE-002DZ-01 extracted `runtime_support.v4_runtime_support.risk_execution_gate`; `qrpc_runtime/src/v4_runtime/risk_execution_gate.rs` now owns risk/execution gate decision evaluation and recording.
Latest recursive supplement: BE-002DZ-02 closed `runtime_support.v4_runtime_support.risk_execution_gate` with `stop_split: true`; next step returns to v4_runtime_support parent residual judgment.
Latest recursive supplement: BE-002EA-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine`; next baseline should freeze simulated execution behavior before code movement.
Latest recursive supplement: BE-002EB-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine` precision baseline; next movement may extract simulated execution behavior into a dedicated v4 runtime child.
Latest recursive supplement: BE-002EC-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine`; next step is single leaf closeout.
Latest recursive supplement: BE-002EC-02 kept `runtime_support.v4_runtime_support.simulated_execution_engine` open with `continue_split: true`; next child candidate is `runtime_adapter`.
Latest recursive supplement: BE-002ED-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.runtime_adapter`; next baseline should freeze the adapter layer only.
Latest recursive supplement: BE-002EE-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.runtime_adapter` baseline; next movement may extract the adapter layer.
Latest recursive supplement: BE-002EF-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.runtime_adapter`; next step is single leaf closeout.
Latest recursive supplement: BE-002EF-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.runtime_adapter` with `stop_split: true`; next simulated execution child candidate is `order_lifecycle_flow`.
Latest governance supplement: GOV-GOVERNANCE-NEXT-PROMOTION-01 promotes `governance-next/` to the default governance authority; old matrix governance remains as compatibility archive and legacy gate material.
Latest recursive supplement: BE-002EG-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.order_lifecycle_flow`; next baseline freezes order lifecycle helper movement and excludes fill ledger, trigger sweep, validation helper families, and release-transition optimization.
Latest recursive supplement: BE-002EH-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.order_lifecycle_flow` baseline; next movement may extract order lifecycle helpers only.
Latest recursive supplement: BE-002EI-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.order_lifecycle_flow`; next step is single leaf closeout.
Latest recursive supplement: BE-002EI-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.order_lifecycle_flow` with `stop_split: true`; next step returns to simulated execution parent residual judgment.
Latest recursive supplement: BE-002EJ-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.market_trigger_flow`; next baseline freezes trigger mechanics without pulling fill ledger or snapshot projection.
Latest recursive supplement: BE-002EK-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.market_trigger_flow` baseline; next movement may extract market trigger helpers only.
Latest recursive supplement: BE-002EL-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.market_trigger_flow`; next step is single leaf closeout.
Latest recursive supplement: BE-002EL-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.market_trigger_flow` with `stop_split: true`; next step returns to simulated execution parent residual judgment.
Latest recursive supplement: BE-002EM-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.fill_ledger_accounting`; next baseline freezes fill and ledger mutation without moving `submit_order` wholesale.
Latest recursive supplement: BE-002EN-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.fill_ledger_accounting` baseline; next movement may extract fill/accounting helpers only.
Latest recursive supplement: BE-002EO-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.fill_ledger_accounting`; next step is single leaf closeout.
Latest recursive supplement: BE-002EO-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.fill_ledger_accounting` with `stop_split: true`; next step returns to simulated execution parent residual judgment.
Latest recursive supplement: BE-002EP-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.snapshot_metrics_projection`; next baseline freezes snapshot, asset curve, and microstructure projection helpers.
Latest recursive supplement: BE-002EQ-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.snapshot_metrics_projection` baseline; next movement may extract projection helpers only.
Latest recursive supplement: BE-002ER-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.snapshot_metrics_projection`; next step is single leaf closeout.
Latest recursive supplement: BE-002ER-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.snapshot_metrics_projection` with `stop_split: true`; next step returns to simulated execution parent residual judgment.
Latest recursive supplement: BE-002ES-01 selected `runtime_support.v4_runtime_support.simulated_execution_engine.validation_capability_helpers`; next baseline freezes validation and capability helper movement while excluding payload parsing and direct-submit orchestration.
Latest recursive supplement: BE-002ET-01 froze `runtime_support.v4_runtime_support.simulated_execution_engine.validation_capability_helpers` baseline; next movement may extract validation/capability helpers with parent re-exports only.
Latest recursive supplement: BE-002EU-01 extracted `runtime_support.v4_runtime_support.simulated_execution_engine.validation_capability_helpers`; next step is single leaf closeout.
Latest recursive supplement: BE-002EU-02 closed `runtime_support.v4_runtime_support.simulated_execution_engine.validation_capability_helpers`; next step returns to simulated execution parent residual judgment.
Latest recursive supplement: BE-002EV-01 closed `runtime_support.v4_runtime_support.simulated_execution_engine`; next step returns to v4 runtime support parent residual judgment.
Latest recursive supplement: BE-002EW-01 selected `runtime_support.v4_runtime_support.test_harness`; next baseline freezes local v4 runtime tests and fixtures only.
Latest recursive supplement: BE-002EX-01 froze `runtime_support.v4_runtime_support.test_harness` baseline; next movement may move local v4 runtime tests into a test-only child module.
Latest recursive supplement: BE-002EY-01 extracted `runtime_support.v4_runtime_support.test_harness`; next step is single leaf closeout.
Latest recursive supplement: BE-002EY-02 kept `runtime_support.v4_runtime_support.test_harness` open; next step is parent residual judgment inside the test harness.
Latest recursive supplement: BE-002EZ-01 selected `runtime_support.v4_runtime_support.test_harness.fixture_builders`; next baseline freezes shared v4 test fixture movement only.
Latest recursive supplement: BE-002FA-01 froze `runtime_support.v4_runtime_support.test_harness.fixture_builders` baseline; next movement may move shared fixture helpers only.
Latest recursive supplement: BE-002FB-01 extracted `runtime_support.v4_runtime_support.test_harness.fixture_builders`; next step is single leaf closeout.
Latest recursive supplement: BE-002FB-02 closed `runtime_support.v4_runtime_support.test_harness.fixture_builders`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FC-01 selected `runtime_support.v4_runtime_support.test_harness.payload_validation_tests`; next baseline freezes malformed payload/config rejection tests only.
Latest recursive supplement: BE-002FD-01 froze `runtime_support.v4_runtime_support.test_harness.payload_validation_tests` baseline; next movement may move five payload/config rejection tests only.
Latest recursive supplement: BE-002FE-01 extracted `runtime_support.v4_runtime_support.test_harness.payload_validation_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002FE-02 closed `runtime_support.v4_runtime_support.test_harness.payload_validation_tests`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FF-01 selected `runtime_support.v4_runtime_support.test_harness.graph_replay_scenarios`; next baseline freezes graph/replay scenario tests only.
Latest recursive supplement: BE-002FG-01 froze `runtime_support.v4_runtime_support.test_harness.graph_replay_scenarios` baseline; next movement may move five graph/replay scenario tests only.
Latest recursive supplement: BE-002FH-01 extracted `runtime_support.v4_runtime_support.test_harness.graph_replay_scenarios`; next step is single leaf closeout.
Latest recursive supplement: BE-002FH-02 closed `runtime_support.v4_runtime_support.test_harness.graph_replay_scenarios`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FI-01 selected `runtime_support.v4_runtime_support.test_harness.simulated_execution_scenarios`; next baseline freezes simulated execution scenario tests only.
Latest recursive supplement: BE-002FJ-01 froze `runtime_support.v4_runtime_support.test_harness.simulated_execution_scenarios` baseline; next movement may move nine simulated execution scenario tests only.
Latest recursive supplement: BE-002FK-01 extracted `runtime_support.v4_runtime_support.test_harness.simulated_execution_scenarios`; next step is single leaf closeout.
Latest recursive supplement: BE-002FK-02 closed `runtime_support.v4_runtime_support.test_harness.simulated_execution_scenarios`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FL-01 selected `runtime_support.v4_runtime_support.test_harness.runtime_recovery_snapshot_tests`; next baseline freezes recovery/cache snapshot tests only.
Latest recursive supplement: BE-002FM-01 froze `runtime_support.v4_runtime_support.test_harness.runtime_recovery_snapshot_tests` baseline; next movement may move two recovery/cache snapshot tests only.
Latest recursive supplement: BE-002FN-01 extracted `runtime_support.v4_runtime_support.test_harness.runtime_recovery_snapshot_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002FN-02 closed `runtime_support.v4_runtime_support.test_harness.runtime_recovery_snapshot_tests`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FO-01 selected `runtime_support.v4_runtime_support.test_harness.live_capability_guard_tests`; next baseline freezes live/capability/risk guard tests only.
Latest recursive supplement: BE-002FP-01 froze `runtime_support.v4_runtime_support.test_harness.live_capability_guard_tests` baseline; next movement may move eleven live/capability/risk guard tests only.
Latest recursive supplement: BE-002FQ-01 extracted `runtime_support.v4_runtime_support.test_harness.live_capability_guard_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002FQ-02 closed `runtime_support.v4_runtime_support.test_harness.live_capability_guard_tests`; next step returns to test harness parent residual judgment.
Latest recursive supplement: BE-002FR-01 closed `runtime_support.v4_runtime_support.test_harness`; next step returns to v4 runtime support parent residual judgment.
Latest recursive supplement: BE-002FS-01 closed `runtime_support.v4_runtime_support`; next step returns to runtime_support parent residual judgment.
Latest recursive supplement: BE-002FT-01 closed `runtime_support`; next step returns to root.contracts parent residual judgment.
Latest recursive supplement: BE-002FU-01 selected `contracts.quantscript`; next baseline freezes the QuantScript parser/HIR/resolve/lowering/diagnostics/static-audit contract.
Latest recursive supplement: BE-002FV-01 froze `contracts.quantscript` baseline; next step selects the first QuantScript child.
Latest recursive supplement: BE-002FW-01 selected `contracts.quantscript.syntax_ast_surface`; next baseline freezes parser, AST, type, and HIR DTO surfaces.
Latest recursive supplement: BE-002FX-01 froze `contracts.quantscript.syntax_ast_surface`; next step decides facade extraction versus structural closeout.
Latest recursive supplement: BE-002FX-02 extracted `contracts.quantscript.syntax_ast_surface`; next step is single leaf closeout.
Latest recursive supplement: BE-002FX-03 closed `contracts.quantscript.syntax_ast_surface`; next step returns to QuantScript parent residual judgment.
Latest recursive supplement: BE-002FY-01 selected `contracts.quantscript.legacy_config_compat`; next baseline freezes deprecated config-style compatibility.
Latest recursive supplement: BE-002FZ-01 froze `contracts.quantscript.legacy_config_compat`; next movement may extract deprecated config compatibility from the crate root.
Latest recursive supplement: BE-002FZ-02 extracted `contracts.quantscript.legacy_config_compat`; next step is single leaf closeout.
Latest recursive supplement: BE-002FZ-03 closed `contracts.quantscript.legacy_config_compat`; next step returns to QuantScript parent residual judgment.
Latest recursive supplement: BE-002GA-01 selected `contracts.quantscript.typed_resolution`; next baseline freezes typed-HIR resolution, `Resolved*` exports, callable classification helpers, resolver diagnostics, and typed-HIR behavior.
Latest recursive supplement: BE-002GB-01 froze `contracts.quantscript.typed_resolution`; next step decides actual extraction versus structural closeout because `resolve.rs` is already the physical typed-resolution owner.
Latest recursive supplement: BE-002GB-02 structurally extracted `contracts.quantscript.typed_resolution`; `quantscript/src/resolve/mod.rs` now owns the resolver module with Rust module path preserved.
Latest recursive supplement: BE-002GB-03 closed the structural extraction step for `contracts.quantscript.typed_resolution` but kept `stop_split: false`; next step selects an internal resolver child.
Latest recursive supplement: BE-002GC-01 selected `contracts.quantscript.typed_resolution.public_type_surface`; next baseline freezes public resolver DTOs and enums before behavior movement.
Latest recursive supplement: BE-002GD-01 froze `contracts.quantscript.typed_resolution.public_type_surface`; next movement may extract public resolver DTOs/enums into `quantscript/src/resolve/public_type_surface.rs`.
Latest recursive supplement: BE-002GD-02 extracted `contracts.quantscript.typed_resolution.public_type_surface`; public resolver DTOs/enums now live in `quantscript/src/resolve/public_type_surface.rs`.
Latest recursive supplement: BE-002GD-03 closed `contracts.quantscript.typed_resolution.public_type_surface` with `stop_split: true`; next step returns to typed-resolution parent residual judgment.
Latest recursive supplement: BE-002GE-01 selected `contracts.quantscript.typed_resolution.callable_classification_surface`; next baseline freezes callable/helper classifiers and registry seeding.
Latest recursive supplement: BE-002GF-01 froze `contracts.quantscript.typed_resolution.callable_classification_surface`; next movement may extract callable/helper classifiers only.
Latest recursive supplement: BE-002GF-02 extracted `contracts.quantscript.typed_resolution.callable_classification_surface`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GF-03 closed `contracts.quantscript.typed_resolution.callable_classification_surface` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GG-01 selected `contracts.quantscript.typed_resolution.semantic_inference_surface`; next baseline freezes expression semantic and manual indicator inference only.
Latest recursive supplement: BE-002GH-01 froze `contracts.quantscript.typed_resolution.semantic_inference_surface`; next movement may extract semantic inference helpers only.
Latest recursive supplement: BE-002GH-02 extracted `contracts.quantscript.typed_resolution.semantic_inference_surface`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GH-03 closed `contracts.quantscript.typed_resolution.semantic_inference_surface` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GI-01 selected `contracts.quantscript.typed_resolution.type_inference_binding_surface`; next baseline freezes type inference and binding helpers only.
Latest recursive supplement: BE-002GJ-01 froze `contracts.quantscript.typed_resolution.type_inference_binding_surface`; next movement may extract type inference and binding helpers only.
Latest recursive supplement: BE-002GJ-02 extracted `contracts.quantscript.typed_resolution.type_inference_binding_surface`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GJ-03 closed `contracts.quantscript.typed_resolution.type_inference_binding_surface` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GK-01 selected `contracts.quantscript.typed_resolution.resolver_orchestration_surface`; next baseline freezes module/function/block/statement/expression lowering only.
Latest recursive supplement: BE-002GL-01 froze `contracts.quantscript.typed_resolution.resolver_orchestration_surface`; next movement may extract resolver orchestration only.
Latest recursive supplement: BE-002GL-02 extracted `contracts.quantscript.typed_resolution.resolver_orchestration_surface`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GL-03 closed `contracts.quantscript.typed_resolution.resolver_orchestration_surface` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GM-01 selected `contracts.quantscript.typed_resolution.resolver_support_surface`; next baseline freezes resolver support helpers only.
Latest recursive supplement: BE-002GN-01 froze `contracts.quantscript.typed_resolution.resolver_support_surface`; next movement may extract support helpers only.
Latest recursive supplement: BE-002GN-02 extracted `contracts.quantscript.typed_resolution.resolver_support_surface`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GN-03 closed `contracts.quantscript.typed_resolution.resolver_support_surface` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GO-01 selected `contracts.quantscript.typed_resolution.resolver_test_harness`; next baseline freezes resolver-local tests only.
Latest recursive supplement: BE-002GP-01 froze `contracts.quantscript.typed_resolution.resolver_test_harness`; next movement may extract resolver-local tests only.
Latest recursive supplement: BE-002GP-02 extracted `contracts.quantscript.typed_resolution.resolver_test_harness`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GP-03 closed `contracts.quantscript.typed_resolution.resolver_test_harness` with `stop_split: true`; next step returns to typed_resolution parent residual judgment.
Latest recursive supplement: BE-002GQ-01 closed `contracts.quantscript.typed_resolution`; next step returns to QuantScript parent residual judgment.
Latest recursive supplement: BE-002GR-01 selected `contracts.quantscript.analysis_diagnostics`; next baseline freezes analysis and diagnostic DTO boundaries.
Latest recursive supplement: BE-002GS-01 froze `contracts.quantscript.analysis_diagnostics`; next movement may structurally extract analysis and diagnostics under one parent.
Latest recursive supplement: BE-002GS-02 structurally extracted `contracts.quantscript.analysis_diagnostics`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GS-03 kept `contracts.quantscript.analysis_diagnostics` open; next step is parent residual judgment inside analysis_diagnostics.
Latest recursive supplement: BE-002GT-01 selected `contracts.quantscript.analysis_diagnostics.unsupported_construct_gate`; next baseline freezes the first analysis diagnostic gate.
Latest recursive supplement: BE-002GU-01 froze `contracts.quantscript.analysis_diagnostics.unsupported_construct_gate`; next movement may extract unsupported construct diagnostics only.
Latest recursive supplement: BE-002GU-02 extracted `contracts.quantscript.analysis_diagnostics.unsupported_construct_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GU-03 closed `contracts.quantscript.analysis_diagnostics.unsupported_construct_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002GV-01 selected `contracts.quantscript.analysis_diagnostics.lookahead_window_gate`; next baseline freezes QS0401/QS0402/QS0403 lookahead-window diagnostics.
Latest recursive supplement: BE-002GW-01 froze `contracts.quantscript.analysis_diagnostics.lookahead_window_gate`; next movement may extract lookahead-window diagnostics only.
Latest recursive supplement: BE-002GW-02 extracted `contracts.quantscript.analysis_diagnostics.lookahead_window_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GW-03 closed `contracts.quantscript.analysis_diagnostics.lookahead_window_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002GX-01 selected `contracts.quantscript.analysis_diagnostics.warmup_fetch_gate`; next baseline freezes required warmup and fetch lookback diagnostics.
Latest recursive supplement: BE-002GY-01 froze `contracts.quantscript.analysis_diagnostics.warmup_fetch_gate`; next movement may extract required warmup and fetch lookback diagnostics only.
Latest recursive supplement: BE-002GY-02 extracted `contracts.quantscript.analysis_diagnostics.warmup_fetch_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002GY-03 closed `contracts.quantscript.analysis_diagnostics.warmup_fetch_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002GZ-01 selected `contracts.quantscript.analysis_diagnostics.indirect_recursion_gate`; next baseline freezes call-graph indirect recursion diagnostics.
Latest recursive supplement: BE-002HA-01 froze `contracts.quantscript.analysis_diagnostics.indirect_recursion_gate`; next movement may extract indirect recursion diagnostics only.
Latest recursive supplement: BE-002HA-02 extracted `contracts.quantscript.analysis_diagnostics.indirect_recursion_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HA-03 closed `contracts.quantscript.analysis_diagnostics.indirect_recursion_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HB-01 selected `contracts.quantscript.analysis_diagnostics.symbol_whitelist_gate`; next baseline freezes known symbol catalog and QS0505 whitelist diagnostics.
Latest recursive supplement: BE-002HC-01 froze `contracts.quantscript.analysis_diagnostics.symbol_whitelist_gate`; next movement may extract QS0505 whitelist diagnostics only.
Latest recursive supplement: BE-002HC-02 extracted `contracts.quantscript.analysis_diagnostics.symbol_whitelist_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HC-03 closed `contracts.quantscript.analysis_diagnostics.symbol_whitelist_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HD-01 selected `contracts.quantscript.analysis_diagnostics.fetch_lookback_warning_gate`; next baseline freezes QS0503 fetch lookback warnings.
Latest recursive supplement: BE-002HE-01 froze `contracts.quantscript.analysis_diagnostics.fetch_lookback_warning_gate`; next movement may extract QS0503 fetch lookback warnings only.
Latest recursive supplement: BE-002HE-02 extracted `contracts.quantscript.analysis_diagnostics.fetch_lookback_warning_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HE-03 closed `contracts.quantscript.analysis_diagnostics.fetch_lookback_warning_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HF-01 selected `contracts.quantscript.analysis_diagnostics.index_bounds_gate`; next baseline freezes data source lookback map and QS0404 index-bound diagnostics.
Latest recursive supplement: BE-002HG-01 froze `contracts.quantscript.analysis_diagnostics.index_bounds_gate`; next movement may extract QS0404 index-bound diagnostics only.
Latest recursive supplement: BE-002HG-02 extracted `contracts.quantscript.analysis_diagnostics.index_bounds_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HG-03 closed `contracts.quantscript.analysis_diagnostics.index_bounds_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HH-01 selected `contracts.quantscript.analysis_diagnostics.dead_code_emit_gate`; next baseline freezes QS0612 constant-false emit diagnostics.
Latest recursive supplement: BE-002HI-01 froze `contracts.quantscript.analysis_diagnostics.dead_code_emit_gate`; next movement may extract QS0612 dead-code emit diagnostics only.
Latest recursive supplement: BE-002HI-02 extracted `contracts.quantscript.analysis_diagnostics.dead_code_emit_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HI-03 closed `contracts.quantscript.analysis_diagnostics.dead_code_emit_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HJ-01 selected `contracts.quantscript.analysis_diagnostics.strategy_presence_gate`; next baseline freezes QS0610/QS0611 strategy presence diagnostics.
Latest recursive supplement: BE-002HK-01 froze `contracts.quantscript.analysis_diagnostics.strategy_presence_gate`; next movement may extract QS0610/QS0611 strategy presence diagnostics only.
Latest recursive supplement: BE-002HK-02 extracted `contracts.quantscript.analysis_diagnostics.strategy_presence_gate`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HK-03 closed `contracts.quantscript.analysis_diagnostics.strategy_presence_gate` with `stop_split: true`; next step returns to analysis_diagnostics parent residual judgment.
Latest recursive supplement: BE-002HL-01 closed `contracts.quantscript.analysis_diagnostics`; next step returns to quantscript parent residual judgment.
Latest recursive supplement: BE-002HM-01 selected `contracts.quantscript.evaluator_normalization`; next baseline freezes `quantscript/src/evaluator.rs`.
Latest recursive supplement: BE-002HN-01 froze `contracts.quantscript.evaluator_normalization`; next movement may convert evaluator into a module directory shell.
Latest recursive supplement: BE-002HN-02 converted `contracts.quantscript.evaluator_normalization` into `quantscript/src/evaluator/mod.rs`; next step is evaluator_normalization parent residual judgment.
Latest recursive supplement: BE-002HO-01 selected `contracts.quantscript.evaluator_normalization.folding_value_wave`; next baseline freezes folding/value helpers as one coarse wave.
Latest recursive supplement: BE-002HP-01 froze `contracts.quantscript.evaluator_normalization.folding_value_wave`; next movement may extract folding/value helpers into a child module.
Latest recursive supplement: BE-002HP-02 extracted `contracts.quantscript.evaluator_normalization.folding_value_wave`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HP-03 closed `contracts.quantscript.evaluator_normalization.folding_value_wave` with `stop_split: true`; next step returns to evaluator_normalization parent residual judgment.
Latest recursive supplement: BE-002HQ-01 selected `contracts.quantscript.evaluator_normalization.helper_inline_execution_wave`; next baseline freezes helper inline execution as one coarse wave.
Latest recursive supplement: BE-002HR-01 froze `contracts.quantscript.evaluator_normalization.helper_inline_execution_wave`; next movement may extract helper inline execution into a child module.
Latest recursive supplement: BE-002HR-02 extracted `contracts.quantscript.evaluator_normalization.helper_inline_execution_wave`; next step is single leaf closeout and split judgment.
Latest recursive supplement: BE-002HR-03 closed `contracts.quantscript.evaluator_normalization.helper_inline_execution_wave` with `stop_split: true`; next step returns to evaluator_normalization parent residual judgment.
Latest recursive supplement: BE-002HS-01 closed `contracts.quantscript.evaluator_normalization`; next step returns to quantscript parent residual judgment.
Latest recursive supplement: BE-002HT-01 selected `contracts.quantscript.runtime_lowering`; next baseline freezes the existing `quantscript/src/lowering/` subtree.
Latest recursive supplement: BE-002HU-01 froze `contracts.quantscript.runtime_lowering`; next step selects one lowering child through parent residual judgment.
Latest recursive supplement: BE-002HV-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface`; next baseline freezes public lowering entrypoints and runtime config assembly.
Latest recursive supplement: BE-002HW-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface`; next residual judgment chooses an internal child before Rust movement.
Latest recursive supplement: BE-002HX-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.profile_detection_surface`; next baseline freezes risk/execution profile parsing.
Latest recursive supplement: BE-002HY-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.profile_detection_surface`; next movement may extract profile detection into its child module.
Latest recursive supplement: BE-002HY-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.profile_detection_surface`; next step is single leaf closeout.
Latest recursive supplement: BE-002HY-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.profile_detection_surface` with `stop_split: true`; next step returns to lowering_orchestrator_surface parent residual judgment.
Latest recursive supplement: BE-002HZ-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness`; next baseline freezes the colocated lowering integration tests.
Latest recursive supplement: BE-002IA-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness`; next movement may extract only the parent test module.
Latest recursive supplement: BE-002IA-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness`; next step is single leaf closeout.
Latest recursive supplement: BE-002IA-03 kept `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness` split open under `PRECISION`; next step is internal parent residual judgment.
Latest recursive supplement: BE-002IB-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.manual_formula_tests`; next baseline freezes manual formula recognition tests.
Latest recursive supplement: BE-002IC-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.manual_formula_tests`; next movement may extract only manual formula recognition/rejection tests.
Latest recursive supplement: BE-002IC-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.manual_formula_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002IC-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.manual_formula_tests` with `stop_split: true`; next step returns to the test harness parent residual judgment.
Latest recursive supplement: BE-002ID-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.spread_lowering_tests`; next baseline freezes spread lowering tests.
Latest recursive supplement: BE-002IE-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.spread_lowering_tests`; next movement may extract only spread admission/rejection tests.
Latest recursive supplement: BE-002IE-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.spread_lowering_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002IE-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.spread_lowering_tests` with `stop_split: true`; next step returns to the test harness parent residual judgment.
Latest recursive supplement: BE-002IF-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.rebalance_lowering_tests`; next baseline freezes rebalance helper lowering tests.
Latest recursive supplement: BE-002IG-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.rebalance_lowering_tests`; next movement may extract only rebalance helper lowering tests.
Latest recursive supplement: BE-002IG-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.rebalance_lowering_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002IG-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.rebalance_lowering_tests` with `stop_split: true`; next step returns to the test harness parent residual judgment.
Latest recursive supplement: BE-002IH-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.basic_runtime_smoke_tests`; next baseline freezes basic runtime smoke tests.
Latest recursive supplement: BE-002II-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.basic_runtime_smoke_tests`; next movement may extract only basic runtime smoke tests.
Latest recursive supplement: BE-002II-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.basic_runtime_smoke_tests`; next step is single leaf closeout.
Latest recursive supplement: BE-002II-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness.basic_runtime_smoke_tests` with `stop_split: true`; next step returns to the test harness parent residual judgment.
Latest recursive supplement: BE-002IJ-01 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.integration_test_harness`; next step returns to lowering_orchestrator_surface parent residual judgment.
Latest recursive supplement: BE-002IK-01 selected `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.entrypoint_runtime_config_assembly`; next baseline freezes runtime config assembly behind the public lowering entrypoints.
Latest recursive supplement: BE-002IL-01 froze `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.entrypoint_runtime_config_assembly`; next movement may extract only private runtime config assembly.
Latest recursive supplement: BE-002IL-02 extracted `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.entrypoint_runtime_config_assembly`; next step is single leaf closeout.
Latest recursive supplement: BE-002IL-03 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface.entrypoint_runtime_config_assembly` with `stop_split: true`; next step returns to lowering_orchestrator_surface parent residual judgment.
Latest recursive supplement: BE-002IM-01 closed `contracts.quantscript.runtime_lowering.lowering_orchestrator_surface`; next step returns to runtime_lowering parent residual judgment.
Latest recursive supplement: BE-002IN-01 selected `contracts.quantscript.runtime_lowering.universe_lowering`; next baseline freezes the oversized high-risk universe lowering owner under precision single-leaf governance.
Latest recursive supplement: BE-002IO-01 froze `contracts.quantscript.runtime_lowering.universe_lowering`; next step selects an internal child, starting from `rebalance_directive_detection`.
Latest recursive supplement: BE-002IP-01 selected `contracts.quantscript.runtime_lowering.universe_lowering.rebalance_directive_detection`; next baseline freezes rebalance directive detection only.
Latest recursive supplement: BE-002IQ-01 froze `contracts.quantscript.runtime_lowering.universe_lowering.rebalance_directive_detection`; next movement may extract only rebalance directive detection.
Latest recursive supplement: BE-002IQ-02 extracted `contracts.quantscript.runtime_lowering.universe_lowering.rebalance_directive_detection`; next step is single leaf closeout.
Latest recursive supplement: BE-002IQ-03 closed `contracts.quantscript.runtime_lowering.universe_lowering.rebalance_directive_detection` with `stop_split: true`; next step returns to universe_lowering parent residual judgment.
Latest recursive supplement: BE-002IR-01 selected `contracts.quantscript.runtime_lowering.universe_lowering.universe_construct_expansion`; next baseline freezes universe construct expansion only.
Latest recursive supplement: BE-002IS-01 froze `contracts.quantscript.runtime_lowering.universe_lowering.universe_construct_expansion`; next movement may extract only AST expansion and substitution.
Latest recursive supplement: BE-002IS-02 extracted `contracts.quantscript.runtime_lowering.universe_lowering.universe_construct_expansion`; next step is single leaf closeout.
Latest recursive supplement: BE-002IS-03 closed `contracts.quantscript.runtime_lowering.universe_lowering.universe_construct_expansion` with `stop_split: true`; next step returns to universe_lowering parent residual judgment.
Latest recursive supplement: BE-002IT-01 closed `contracts.quantscript.runtime_lowering.universe_lowering`; next step returns to runtime_lowering parent residual judgment.
Latest recursive supplement: BE-002IU-01 selected `contracts.quantscript.runtime_lowering.intent_inference`; next baseline freezes intent inference before any Rust movement.
Latest recursive supplement: BE-002IV-01 froze `contracts.quantscript.runtime_lowering.intent_inference`; next step selects one internal intent inference child.
Latest recursive supplement: BE-002IW-01 selected `contracts.quantscript.runtime_lowering.intent_inference.intent_collection_orchestration`; next baseline freezes orchestration movement only.
Latest recursive supplement: BE-002IX-01 froze `contracts.quantscript.runtime_lowering.intent_inference.intent_collection_orchestration`; next movement may extract only top-level intent orchestration.
Latest recursive supplement: BE-002IX-02 extracted `contracts.quantscript.runtime_lowering.intent_inference.intent_collection_orchestration`; next step is single leaf closeout.
Latest recursive supplement: BE-002IX-03 closed `contracts.quantscript.runtime_lowering.intent_inference.intent_collection_orchestration` with `stop_split: true`; next step returns to intent_inference parent residual judgment.
Latest recursive supplement: BE-002IY-01 selected `contracts.quantscript.runtime_lowering.intent_inference.single_indicator_intent_inference`; next baseline freezes single-indicator intent construction only.
Latest recursive supplement: BE-002IZ-01 froze `contracts.quantscript.runtime_lowering.intent_inference.single_indicator_intent_inference`; next movement may extract only the single-indicator builder.
Latest recursive supplement: BE-002IZ-02 extracted `contracts.quantscript.runtime_lowering.intent_inference.single_indicator_intent_inference`; next step is single leaf closeout.
Latest recursive supplement: BE-002IZ-03 closed `contracts.quantscript.runtime_lowering.intent_inference.single_indicator_intent_inference` with `stop_split: true`; next step returns to intent_inference parent residual judgment.
Latest recursive supplement: BE-002JA-01 selected `contracts.quantscript.runtime_lowering.intent_inference.spread_intent_inference`; next baseline freezes formal spread matching, operand decoding, params, and resample/align parsing only.
Latest recursive supplement: BE-002JB-01 froze `contracts.quantscript.runtime_lowering.intent_inference.spread_intent_inference`; next movement may extract only the spread builder and spread operand/params helpers.
Latest recursive supplement: BE-002JB-02 extracted `contracts.quantscript.runtime_lowering.intent_inference.spread_intent_inference`; next step is single leaf closeout.
Latest recursive supplement: BE-002JB-03 closed `contracts.quantscript.runtime_lowering.intent_inference.spread_intent_inference` with `stop_split: true`; next step returns to intent_inference parent residual judgment.
Latest recursive supplement: BE-002JC-01 closed `contracts.quantscript.runtime_lowering.intent_inference`; next step returns to runtime_lowering parent residual judgment.
Latest recursive supplement: BE-002JD-01 selected `contracts.quantscript.runtime_lowering.manual_formula_fallback`; next baseline freezes handwritten manual formula fallback recognition only.
Latest recursive supplement: BE-002JE-01 froze `contracts.quantscript.runtime_lowering.manual_formula_fallback`; next step selects one internal fallback child before Rust movement.
Latest recursive supplement: BE-002JF-01 selected `contracts.quantscript.runtime_lowering.manual_formula_fallback.manual_rsi_formula`; next baseline freezes RSI manual formula fallback recognition only.
Latest recursive supplement: BE-002JG-01 froze `contracts.quantscript.runtime_lowering.manual_formula_fallback.manual_rsi_formula`; next movement may extract RSI shell, RS pair matching, balanced smoothing checks, and method mapping only.
