# 里程碑文档

> 当前版本: v4.7.0 | 当前功能里程碑: v4.11.0 v4 策略配置系统一等化 📋 | 当前治理基线: v4.15.0 三矩阵完全接管 ✅ | 当前架构里程碑: v4.16.0 模块化抽离第一波 📋
> 状态: v2.x✅ v3.x✅ v4.0.0✅ v4.1.0✅ v4.2.0✅ v4.3.0✅ v4.4.0✅ v4.5.0✅ v4.6.0✅ v4.7.0✅ v4.8.x✅ v4.9.0✅ v4.10.0✅ v4.11.0📋 v4.12.0✅ v4.13.0✅ v4.14.0✅ v4.15.0✅ v4.16.0📋
> 后续优化: 功能线只在已登记的非账户、非策略搜索/筛选边界内推进；治理线已由三矩阵接管；架构线按抽离、整理、重构三步走推进，当前只启用抽离控制。

---

## 当前活跃

### v4.16.0 — 模块化抽离第一波 (MINOR architecture / governance, 规划)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.16.0/01-规划方案.md` | 十万行级重大工程的后端抽离、前端抽离、E2E 整理延后、测试资产汰换登记和等价验证 | 📋 已创建 |
| 02 | `v4.16.0/02-落地记录.md` | 抽离控制面落地状态、工作线、决策项和禁止事项 | 📋 已创建 |
| 03 | `v4.16.0/03-后端抽离登记.md` | 后端抽离候选、父模块、public 方法、兼容桥和等价证据 | 📋 已创建 |
| 04 | `v4.16.0/04-前端抽离登记.md` | 前端抽离候选、页面/store 边界、UI 行为对照和暂停条件 | 📋 已创建 |
| 05 | `v4.16.0/05-测试资产汰换登记.md` | E2E 整理延后、旧测试废弃候选、替代证据和风险窗口 | 📋 已创建 |
| 06 | `v4.16.0/06-后端接口边界首批抽离方案.md` | BE-001 后端接口边界首批抽离方案，锁定 router/API/facade 边界 | 📋 已创建 |
| 07 | `v4.16.0/07-顶层大模块统计.md` | 统计 6 个逻辑顶层大模块、16 个白箱子节点和物理目录规模 | 📋 已创建 |
| 08 | `v4.16.0/08-system大模块分层统计.md` | 确定 system 分 3 层、10 个叶子模块，并说明与 BE-001 的关系 | 📋 已创建 |
| 09 | `v4.16.0/09-system.entry首批抽离记录.md` | system 试水抽离第一刀，将 public `run_server` 归入 `system.entry.backend_process` 并保留兼容入口 | ✅ 已完成 |
| 10 | `v4.16.0/10-system抽离完成记录.md` | system 抽离完成记录，将 `run_api_server` 与启动期 helper 归入 `system.entry.backend_process` | ✅ 已完成 |
| 11 | `v4.16.0/11-system抽离经验回填.md` | 将 system 试水经验回填为 public/内部实现分类、owner 复核和未迁移边界准则 | ✅ 已完成 |
| 12 | `v4.16.0/12-system十叶模块等价基线.md` | 为 system 10 个叶子模块建立功能等价证据、继续抽离状态和暂停点 | ✅ 已完成 |
| 13 | `v4.16.0/13-递归模块化全局根流程.md` | 固化顶层模块、叶子抽离、叶子整理、细分价值判断和全局根收束流程 | ✅ 已完成 |
| 14 | `v4.16.0/14-system.entry.launch_scripts单叶closeout.md` | S1 启动脚本入口白箱 closeout，确认脚本等价并停止继续细分 | ✅ 已完成 |
| 15 | `v4.16.0/15-system.desktop_shell.tauri_config单叶closeout.md` | S4 Tauri config 白箱 closeout，确认 CSP、窗口和 capability allowlist 等价并停止继续细分 | ✅ 已完成 |
| 16 | `v4.16.0/16-system.runtime_profile.config_examples单叶closeout.md` | S10 配置样例白箱 closeout，确认环境、runtime protocol 和 strategy_ir 样例等价并停止继续细分 | ✅ 已完成 |
| 17 | `v4.16.0/17-system.desktop_shell.tauri_runtime-readiness等价检查.md` | S3 Tauri runtime readiness 等价检查，确认 3000 wait 与启动顺序等价 | ✅ 已完成 |
| 18 | `v4.16.0/18-system.desktop_shell.tauri_runtime单叶closeout.md` | S3 Tauri runtime 白箱 closeout，确认桌面启动 smoke、窗口生命周期和关闭路径等价并停止继续细分 | ✅ 已完成 |
| 19 | `v4.16.0/19-system.build_delivery.desktop_build_scripts单叶closeout.md` | S7 desktop build/dev scripts 白箱 closeout，确认 `build.rs`、`build.bat`、`dev.bat` 等价并停止继续细分 | ✅ 已完成 |
| 20 | `v4.16.0/20-system.entry.backend_process单叶closeout.md` | S2 backend process 白箱 closeout，正式收束启动进程边界并保持 API owner 外置 | ✅ 已完成 |
| 21 | `v4.16.0/21-system.desktop_shell.assets_schema单叶closeout.md` | S5 assets/schema 白箱 closeout，确认桌面图标和 Tauri generated schema 等价并停止继续细分 | ✅ 已完成 |
| 22 | `v4.16.0/22-system.build_delivery.container_proxy单叶closeout.md` | S8 container/proxy 静态白箱 closeout，登记 Dockerfile、compose 和 nginx proxy 边界 | ✅ 已完成 |
| 23 | `v4.16.0/23-system.build_delivery.S6-S9暂停决策记录.md` | S6 workspace manifest 与 S9 CI/release 暂停决策记录，明确不算 closeout 完成 | ✅ 已采纳 |
| 24 | `v4.16.0/24-system顶层阶段性closeout.md` | system 顶层阶段性 closeout，收束当前允许范围；S1-S10 已完成 closeout 或静态 closeout | ✅ 已完成 |
| 25 | `v4.16.0/25-system.build_delivery.S6-S9恢复提案与适配性校验.md` | S6/S9 恢复提案与适配性校验，确认只做文档级 closeout，不改 manifest/workflow/release 语义 | ✅ 已完成 |
| 26 | `v4.16.0/26-system.build_delivery.workspace_manifest单叶closeout.md` | S6 workspace manifest 白箱 closeout，确认 Cargo workspace/package manifest 与 lockfile 边界 | ✅ 已完成 |
| 27 | `v4.16.0/27-system.build_delivery.ci_release单叶closeout.md` | S9 CI/release 白箱 closeout，确认 workflow、packaging 和 release manifest 边界 | ✅ 已完成 |
| 28 | `v4.16.0/28-backend大模块分层统计.md` | backend 顶层分层统计，确认 `root.backend` 的 3 层网络和 9 个 L2 叶子候选 | 📋 已创建 |
| 29 | `v4.16.0/29-backend.interface_boundary等价基线.md` | BE-001A `backend.interface_boundary` 等价基线，锁定 route owner、public 入口和未迁移边界 | 📋 已创建 |
| 30 | `v4.16.0/30-backend九叶模块壳抽离记录.md` | BE-001B backend 九叶模块壳抽离记录，建立 `src/backend/` 父模块和 9 个叶子 facade | ✅ 已完成 |
| 31 | `v4.16.0/31-backend.interface_boundary单叶closeout.md` | BE-001C-01 `backend.interface_boundary` 单叶 closeout，确认父级 route facade 不继续拆分 | ✅ 已完成 |
| 32 | `v4.16.0/32-backend.capability单叶closeout.md` | BE-001C-02 `backend.capability` 单叶 closeout，确认 capability 真源边界不继续拆分 | ✅ 已完成 |
| 33 | `v4.16.0/33-backend.strategy_config单叶closeout.md` | BE-001C-03 `backend.strategy_config` 单叶 closeout，登记 artifact/preflight/diff/AI proposal L3 候选 | ✅ 已完成 |
| 34 | `v4.16.0/34-backend.runtime单叶closeout.md` | BE-001C-04 `backend.runtime` 单叶 closeout，登记 run/backtest/mutation/evidence/persistence L3 候选 | ✅ 已完成 |
| 35 | `v4.16.0/35-backend.graph_compile单叶closeout.md` | BE-001C-05 `backend.graph_compile` 单叶 closeout，登记 graph/QS/compile/diagnostics L3 候选 | ✅ 已完成 |
| 36 | `v4.16.0/36-backend.storage_security单叶closeout.md` | BE-001C-06 `backend.storage_security` 单叶 closeout，登记安全 L3 候选和安全决策暂停点 | ✅ 已完成 |
| 37 | `v4.16.0/37-backend.ops_governance单叶closeout.md` | BE-001C-07 `backend.ops_governance` 单叶 closeout，登记 sandbox/alerts/snapshots/runbook/chaos/hotswap L3 候选 | ✅ 已完成 |
| 38 | `v4.16.0/38-backend.app_state_wiring单叶closeout.md` | BE-001C-08 `backend.app_state_wiring` 单叶 closeout，确认 AppState wiring 不继续拆分 | ✅ 已完成 |
| 39 | `v4.16.0/39-backend.test_support单叶closeout.md` | BE-001C-09 `backend.test_support` 单叶 closeout，确认测试资产汰换前不继续拆分 | ✅ 已完成 |
| 40 | `v4.16.0/40-backend.strategy_config_L3模块壳抽离记录.md` | BE-001D `backend.strategy_config` L3 模块壳抽离，建立 artifact/preflight/diff/AI proposal binding 子叶 facade | ✅ 已完成 |
| 41 | `v4.16.0/41-backend其余八叶模块壳抽离记录.md` | BE-001E backend 其余八叶薄壳抽离，建立 interface/capability/runtime/graph/storage/ops/state/test 子 facade | ✅ 已完成 |
| 42 | `v4.16.0/42-backend.interface_boundary子叶抽离完成记录.md` | BE-001E-01 `backend.interface_boundary` 子叶抽离完成，确认 8 个 bridge facade 等价 | ✅ 已完成 |
| 43 | `v4.16.0/43-backend.capability子叶抽离完成记录.md` | BE-001E-02 `backend.capability` 子叶抽离完成，确认 capability snapshot facade 等价 | ✅ 已完成 |
| 44 | `v4.16.0/44-backend.runtime子叶抽离完成记录.md` | BE-001E-03 `backend.runtime` 子叶抽离完成，确认 runtime routes facade 等价 | ✅ 已完成 |
| 45 | `v4.16.0/45-backend.graph_compile子叶抽离完成记录.md` | BE-001E-04 `backend.graph_compile` 子叶抽离完成，确认 compile/graph/QS route facade 等价 | ✅ 已完成 |
| 46 | `v4.16.0/46-backend.storage_security子叶抽离完成记录.md` | BE-001E-05 `backend.storage_security` 子叶抽离完成，确认 credential API/vault facade 等价且安全暂停保留 | ✅ 已完成 |
| 47 | `v4.16.0/47-backend.ops_governance子叶抽离完成记录.md` | BE-001E-06 `backend.ops_governance` 子叶抽离完成，确认 ops route facade 等价 | ✅ 已完成 |
| 48 | `v4.16.0/48-backend.app_state_wiring子叶抽离完成记录.md` | BE-001E-07 `backend.app_state_wiring` 子叶抽离完成，确认 health/state factory facade 等价 | ✅ 已完成 |
| 49 | `v4.16.0/49-backend.test_support子叶抽离完成记录.md` | BE-001E-08 `backend.test_support` 子叶抽离完成，确认 test scenario facade 等价且测试汰换未启动 | ✅ 已完成 |
| 50 | `v4.16.0/50-backend.runtime.routes单子叶等价基线.md` | BE-001F-01 `backend.runtime.routes` 单子叶等价基线，固定 runtime route aggregate facade 的真实 owner 和回归证据 | 📋 已创建 |
| 51 | `v4.16.0/51-backend.runtime.routes抽离记录.md` | BE-001F-02 `backend.runtime.routes` 抽离记录，接管 runtime route aggregate 列表并保留 handler/state owner 原位 | ✅ 已完成 |
| 52 | `v4.16.0/52-backend.runtime.routes.run单子叶等价基线.md` | BE-001G-01 `backend.runtime.routes.run` 单子叶等价基线，固定 run route group 与 event stream 排除边界 | 📋 已创建 |
| 53 | `v4.16.0/53-backend.runtime.routes.run抽离记录.md` | BE-001G-02 `backend.runtime.routes.run` 抽离记录，接管 run route group 并保留 handler/state owner 原位 | ✅ 已完成 |
| 54 | `v4.16.0/54-backend.runtime.routes.run单叶closeout.md` | BE-001G-03 `backend.runtime.routes.run` 单叶 closeout，确认 route facade 收束并判断 `src/runtime/run.rs` handler 层值得另起基线 | ✅ 已完成 |
| 55 | `v4.16.0/55-runtime.run.v4_handoff单子叶等价基线.md` | BE-001H-01 `runtime.run.v4_handoff` 单子叶等价基线，固定 `/api/runtime/v4/run` handler 层边界与 `api_run` 证据 | 📋 已创建 |
| 56 | `v4.16.0/56-runtime.run.v4_handoff抽离记录.md` | BE-001H-02 `runtime.run.v4_handoff` 抽离记录，将 v4 handoff handler/type/helper 迁入 `src/runtime/run/v4_handoff.rs` 并保留父级兼容出口 | ✅ 已完成 |
| 57 | `v4.16.0/57-runtime.run.v4_handoff单叶closeout.md` | BE-001H-03 `runtime.run.v4_handoff` 单叶 closeout，确认本叶等价并停止内部细分 | ✅ 已完成 |
| 58 | `v4.16.0/58-runtime.run.session_start单子叶等价基线.md` | BE-001I-01 `runtime.run.session_start` 单子叶等价基线，固定 legacy `/api/runtime/test-run` handler 边界与 `api_run` 证据 | 📋 已创建 |
| 59 | `v4.16.0/59-runtime.run.session_start抽离记录.md` | BE-001I-02 `runtime.run.session_start` 抽离记录，将 `start_test_run` 迁入 `src/runtime/run/session_start.rs` 并保留父级兼容出口 | ✅ 已完成 |
| 60 | `v4.16.0/60-runtime.run.session_start单叶closeout.md` | BE-001I-03 `runtime.run.session_start` 单叶 closeout，确认本叶等价并停止内部细分 | ✅ 已完成 |
| 61 | `v4.16.0/61-runtime.run.record_store单子叶等价基线.md` | BE-001J-01 `runtime.run.record_store` 单子叶等价基线，固定 run record list/detail/save/discard 与 persistence/audit 边界 | 📋 已创建 |
| 62 | `v4.16.0/62-runtime.run.record_store真实边界梳理.md` | BE-001J-02 `runtime.run.record_store` 真实边界梳理，校正 route method、共享 helper owner 和最小抽离边界 | 📋 已创建 |
| 63 | `v4.16.0/63-runtime.run.record_store抽离方案.md` | BE-001J-03 `runtime.run.record_store` 抽离方案，锁定四个 handler 的最小移动方案和 shared helper 保留边界 | 📋 已创建 |
| 64 | `v4.16.0/64-runtime.run.record_store抽离记录.md` | BE-001J-04 `runtime.run.record_store` 抽离记录，将四个 handler 迁入 `src/runtime/run/record_store.rs` 并保留父级兼容出口 | ✅ 已完成 |
| 65 | `v4.16.0/65-runtime.run.record_store单叶closeout.md` | BE-001J-05 `runtime.run.record_store` 单叶 closeout，确认本叶等价并停止内部细拆，后续回到 `runtime.run.replay_status` 基线 | ✅ 已完成 |
| 66 | `v4.16.0/66-runtime.run.replay_status单子叶等价基线.md` | BE-001K-01 `runtime.run.replay_status` 单子叶等价基线，固定 replay/status 两个 handler 与 SSE 排除边界 | 📋 已创建 |
| 67 | `v4.16.0/67-runtime.run.replay_status抽离方案.md` | BE-001K-02 `runtime.run.replay_status` 抽离方案，锁定两个 handler 的最小移动方案和 SSE/response/schema/metrics 保留边界 | 📋 已创建 |
| 68 | `v4.16.0/68-runtime.run.replay_status抽离记录.md` | BE-001K-03 `runtime.run.replay_status` 抽离记录，将两个 handler 迁入 `src/runtime/run/replay_status.rs` 并保留 SSE/shared owner | ✅ 已完成 |
| 69 | `v4.16.0/69-runtime.run.replay_status单叶closeout.md` | BE-001K-04 `runtime.run.replay_status` 单叶 closeout，确认本叶等价并停止内部细拆，后续回到父级 `runtime.event_stream` 候选 | ✅ 已完成 |
| 70 | `v4.16.0/70-runtime.event_stream单子叶等价基线.md` | BE-001L-01 `runtime.event_stream` 单子叶等价基线，固定 SSE route、frame order、keep-alive 和父级 route owner；当前不移动代码 | 📋 已创建 |
| 71 | `v4.16.0/71-runtime.event_stream抽离方案.md` | BE-001L-02 `runtime.event_stream` 抽离方案，锁定只移动 `stream_run_events` 到 `src/runtime/event_stream.rs` 并保留父级 route owner | 📋 已创建 |
| 72 | `v4.16.0/72-runtime.event_stream抽离记录.md` | BE-001L-03 `runtime.event_stream` 抽离记录，将 `stream_run_events` 迁入 `src/runtime/event_stream.rs` 并保留父级 route owner | ✅ 已完成 |
| 73 | `v4.16.0/73-runtime.event_stream单叶closeout.md` | BE-001L-04 `runtime.event_stream` 单叶 closeout，确认 SSE handler 等价并停止内部细拆，后续回到 `runtime.backtest` 候选 | ✅ 已完成 |
| 74 | `v4.16.0/74-runtime.backtest单子叶等价基线.md` | BE-001M-01 `runtime.backtest` 单子叶等价基线，冻结 backtest route group、artifact/compare/replay/persistence owner；当前不移动代码 | 📋 已创建 |
| 75 | `v4.16.0/75-runtime.backtest抽离方案.md` | BE-001M-02 `runtime.backtest` 抽离方案，锁定下一批只抽离 backtest route facade 到计划目标文件 | 📋 已创建 |

### v4.15.0 — 三矩阵完全接管 closeout (MINOR governance, 已落地)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.15.0/01-规划方案.md` | 旧入口导流、模块树覆盖、治理 gate 和完全接管 closeout | ✅ 已落地 |
| 02 | `v4.15.0/02-治理closeout.md` | 三矩阵完全接管判定、三档样例、无法自动化项和后续维护责任 | ✅ 已归档 |

### v4.14.0 — 治理门禁自动化 (MINOR governance, 已落地)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.14.0/01-规划方案.md` | 三矩阵声明、引导坐标、模块树漂移、发布过渡保护和 closeout 接入 | ✅ 已落地 |
| 02 | `v4.14.0/02-落地记录.md` | 治理 gate 第一波自动化、closeout 26 项接入和验证记录 | ✅ 已归档 |

### v4.13.0 — 模块树白箱扩面 (MINOR governance, 已落地)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.13.0/01-规划方案.md` | active 模块、关键 public 方法、模块化重构通道和回归保护矩阵 | ✅ 已落地 |
| 02 | `v4.13.0/02-落地记录.md` | 模块树第一波白箱节点、public 方法、回归保护和后续缺口 | ✅ 已归档 |

### v4.12.0 — 三矩阵治理入口启用 (MINOR governance, 已落地)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.12.0/01-规划方案.md` | 三矩阵治理入口、三档执行、父子通信、发布过渡协议和总索引接入 | ✅ 已落地 |
| 02 | `v4.12.0/02-落地记录.md` | v4.12.0 工作线状态、硬规则确认、验证记录和后续缺口分流 | ✅ 已归档 |
| 03 | `../00-matrix-governance/landing-roadmap.md` | v4.12.0 至 v4.15.0 完全落地路线 | ✅ 已铺设 |

### v4.11.0 — v4 策略配置系统一等化 (MINOR, 规划/设计)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.11.0/01-规划方案.md` | 设计指标、功能演进登记、回归保护、GP 合规矩阵 | 📋 已创建 |
| 02 | `v4.11.0/02-策略配置系统端到端设计.md` | 前端、后端、执行端、依赖、契约与测试设计 | 📋 已创建 |
| 03 | `v4.11.0/03-推进约束与防偏移检查单.md` | 从立项到 closeout 的防偏移规则、分歧点暂停协议、路径稳定规则 | 📋 已创建 |

### v4.7.0 — v4 AI 提案 + 性能优化 + 两轮诱错审计 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.7.0/01-规划方案.md` | v4 AI 提案、v4 沙箱回放验证、runtime 性能优化规划 | ✅ 已落实 |
| 02 | `v4.7.0/02-closeout.md` | v4.7.0 实施摘要、验证证据和延后项 | ✅ 已归档 |
| 03 | `v4.7.0/02-元流水线修复方案.md` | track-gate-metrics 修复、check-capability-stack 新增、Draft/Release 模式 | ✅ 已落地 |
| 04 | `../05-testing/自由维度诱错审计-v4.7.0-第1轮.md` | 5维度30发现: 1 S0 + 2 S1 + 6 P1 + 13 P2 + 8 P3 | ✅ S0/S1/P1 已修复 |
| 05 | `../05-testing/自由维度诱错审计-v4.7.0-第2轮.md` | 3维度20发现: 1 P1 + 10 P2 + 8 P3 + 8 PASS；P1/P2-1 已修复 | ✅ 报告已归档 |

### v4.7.1 — 审计阻断项修复 (PATCH, 可选归档)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.7.1/01-规划方案.md` | 第1轮 S1/P1 + 第2轮 P1 已在当前工作区落地；可用于单独 PATCH 归档 | ✅ 修复已落地 |

### v4.8.0 — 双执行切面 + P2 质量收敛 (MINOR, 规划/落地记录已归档)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.8.0/01-规划方案.md` | PaperSimulated + PaperActual 双执行切面, 两轮审计剩余 P2 消化 | ✅ 已创建 |
| 02 | `v4.8.0/02-综合优化清单.md` | W0-W4 优化清单、验收门禁、回归保护矩阵 | ✅ 已创建 |
| 03 | `v4.8.0/03-W1-测试覆盖补全落地记录.md` | W1 测试覆盖补全落地记录 | ✅ 已归档 |
| 04 | `v4.8.0/04-W2-API与格式一致性落地记录.md` | W2 API 与格式一致性落地记录 | ✅ 已归档 |
| 05 | `v4.8.0/05-W3-运行时数值安全落地记录.md` | W3 运行时数值安全落地记录 | ✅ 已归档 |
| 06 | `v4.8.0/06-W4-代码质量与收口落地记录.md` | W4 代码质量与收口落地记录 | ✅ 已归档 |

### v4.8.1 — API 契约 + 部署治理超级规范化 (PATCH, 后续优化规划)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.8.1/01-规划方案.md` | OpenAPI 凭证路径结构、route diff 基线、profile 矩阵、四平面治理、依赖现代化评估；账户相关项已裁出 | ✅ 已落地 |
| 02 | `v4.8.1/02-综合优化清单.md` | P1/P2/P3 优化项、依赖排序、验收门禁、closeout 分流规则 | ✅ 已落地 |
| 03 | `v4.8.1/03-落地记录.md` | OpenAPI route diff clean、executor 路径规范补齐、账户项裁剪边界 | ✅ 已归档 |

### v4.8.2 — 产品/UX/i18n 收敛 (PATCH, 后续优化规划)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.8.2/01-规划方案.md` | zh-CN 转义修复、执行端 i18n、QS 编辑器、CSS 收敛、首次体验、404、中文用户指南 | ✅ 已落地 |
| 02 | `v4.8.2/02-综合优化清单.md` | P1/P2/P3 UX 与 i18n 优化项、误报不采纳项、验收门禁 | ✅ 已落地 |
| 03 | `v4.8.2/03-落地记录.md` | UX/i18n 落地清单与验证证据 | ✅ 已归档 |

### v4.9.0 — 产品功能完整度 + 插件执行安全 (MINOR, 后续优化规划)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.9.0/01-规划方案.md` | PaperActual 自动 runner、插件执行安全、策略包、设置页、API 版本治理、AI 沙箱队列、执行端图表控制 | ✅ 已落地 |
| 02 | `v4.9.0/02-综合优化清单.md` | P1/P2 功能完整度与安全优化项、验收门禁、兼容边界 | ✅ 已落地 |
| 03 | `v4.9.0/03-落地记录.md` | 功能完整度落地清单、验证证据与 PaperActual 安全边界 | ✅ 已归档 |

### v4.10.0 — UX 收口与产品边界固化 (MINOR, 后续优化规划)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.10.0/01-规划方案2.md` | 亮色主题、执行端 i18n 收口、CSS 瘦身、教程入口、Tab keep-alive、产品边界固化 | ✅ 已落地 |
| 02 | `v4.10.0/02-落地记录.md` | v4.10.0 代码与文档落地记录、unsupported 决策、验证记录 | ✅ 已归档 |

### v4.6.0 — v4 LiveActual + OKX 实盘边界 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.6.0/01-规划方案.md` | LiveActual runtime、Risk Plane 强制、ProviderNative 门禁 | ✅ 已落实 |
| 02 | `v4.6.0/02-closeout.md` | 本地安全边界验证与 OKX testnet E2E 延后项 | ✅ 已归档 |

### v4.5.0 — 高级订单类型 + Tick 级回放 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.5.0/01-规划方案.md` | OCO/trailing/GTD/amend、tick replay、microstructure metrics | ✅ 已落实 |
| 02 | `v4.5.0/02-closeout.md` | 本地模拟订单与 tick replay 验证证据 | ✅ 已归档 |

### v4.4.0 — 嵌套状态机第一波 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.4.0/01-规划方案.md` | child_machine 二级嵌套、深度=2、复杂度预算 | ✅ 已落实 |
| 02 | `v4.4.0/02-closeout.md` | 24/24 closeout 通过 | ✅ 已归档 |

### v4.3.0 — v4 回测 + 多交易对策略 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.3.0/01-规划方案.md` | v4 回测引擎、多交易对 machine 展开、v4 策略模板库 | ✅ 已落实 |

### v4.2.0 — 执行端 v4 集成 + P3 消化 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.2.0/01-规划方案.md` | RunnerPool v4 runner、OKX行情驱动v4、执行端v4部署、P3锁内I/O修复 | ✅ 已落实 |

### v4.1.0 — v4 runtime 加固 + P2 审计消化 (MINOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.1.0/01-规划方案.md` | /api/runtime/v4/run、CLI v4-run、event payload校验、条件单补全 | ✅ 已落实 |

### v4.0.0 — 状态机化 QuantScript + 开发者学习流水线 (MAJOR, 已完成) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v4.0.0/01-规划方案.md` | 顶层 DAG + 节点内状态机、QS 状态机 DSL、事件模型、Risk Plane、ExecutionMachine 能力矩阵、学习流水线 | ✅ 已落实 |
| 02 | `v4.0.0/03-closeout.md` | GP 合规矩阵、五维度评分、MAJOR 演化通道 8 Phase、遗留项 | ✅ 已归档 |

---

## 已完成版本

### v3.5.1 — 审计P1清零 + 重点P2修复 (PATCH) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.5.1/01-规划方案.md` | 10项P1 + 9项重点P2: 事务保护/告警引擎/编译缓存/数据安全/前端质量 | ✅ 已完成 |
| 02 | `v3.5.1/02-综合优化清单.md` | 53项审计发现 (0 S0, 14 P1, 23 P2, 16 P3) 完整清单 | ✅ 已收口 |
| 03 | `v3.5.1/03-closeout.md` | 执行概况、五维度评分9/10、GP合规矩阵、遗留项流向v3.6.0 | ✅ 已完成 |

### v3.5.0 — GP全合规 + 前端专业级 + P2质量收口 (MINOR) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.5.0/01-规划方案.md` | 12项: P1(刷新令牌/告警恢复) + P2(bcrypt超时/warnings清零/DEV限速/include!重构/价格涌动/订单动画/ParamsPanel/编译缓存/OKX testnet/模式切换) | ✅ 全部完成 |
| 02 | `v3.5.0/02-综合优化清单.md` | 12项按P1/P2分组，含依赖排序和工时估算 | ✅ 已收口 |

### v3.4.1 — GP §10.3 回归阻断修复 (PATCH) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.4.1/01-规划方案.md` | 2项已修复: S0-1 (集成测试fn缺失) / P1-1 (凭证保险库hostname依赖) | ✅ 已完成 |

### v3.4.0 — (MINOR) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.4.0/01-规划方案.md` | (归档) | ✅ 已完成 |
| 02 | `v3.4.0/02-综合优化清单.md` | (归档) | ✅ 已收口 |

### v3.3.0 — 全量消化 (MINOR) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.3.0/03-closeout.md` | 五维度评分9.1/10、GP合规16/18、12项交付、8项遗留 | ✅ 已完成 |

### v3.2.0 — P3全量消化 + 实盘模式 (MINOR)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.2.0/01-规划方案.md` | 35项: 性能达标/前端专业级/实盘OKX/文档全中文 | 🟢 完成 |

### v3.1.0 — 审计闭环 + 质量达标 (MINOR)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.1.0/01-规划方案.md` | 37项: 序列化安全/前端质量/持久化/测试30+/GP合规矩阵 | 🟢 完成 |

### v3.0.2 — 健壮性收敛 (PATCH)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.0.2/01-规划方案.md` | 15项: 资源上限/HTTP状态码/中文化/前端轮询合并/测试15+ | 🟢 完成 |

### v3.0.1 — 安全与数据完整性紧急修复 (PATCH)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.0.1/01-规划方案.md` | 11项: .bak回滚/审计日志修复/RingBuffer VecDeque/前端错误UI/HMAC签名 | 🟢 完成 |

### v3.0.0 — 实时执行端 (MAJOR)

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v3.0.0/01-规划方案.md` | 5 Phase 27任务 | 🟢 完成 |
| 02 | `v3.0.0/02-综合优化清单.md` | 27任务按Phase分组 | 🟢 完成 |

### v2.3.3 — S0阻断修复 + P1关键收敛 (PATCH) ✅

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v2.3.3/01-规划方案.md` | 9条S0阻断修复 + 15条P1关键项 | ✅ 已完成 |
| 02 | `v2.3.3/02-综合优化清单.md` | S0(9) + P1(15) 按维度分组 | ✅ 已完成 |
| 03 | `v2.3.3/03-closeout.md` | 执行概况、修复列表、验收结果、遗留项 | ✅ 已完成 |

### v2.2.x — 架构重构 + i18n 完整化

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v2.2.0/01-设计文档.md` | 5大目标: Runtime拆分/thiserror/tracing/i18n/TLS, 2项决策纪录 | ✅ 已完成 |

### v2.1.x — 全量清零 + 5轮诱错 + 审计闭环

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v2.1.3/01-closeout.md` | 97项清零/5轮20维282发现/十角色38/38/五维8.98/GP30/33 | ✅ 已完成 |

### v2.0.0 — MAJOR 版本 closeout

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v2.0.0/01-设计文档.md` | OKX实盘+多用户+插件市场+前端补全+打包, 4项决策纪录 | ✅ 已完成 |
| 02 | `v2.0.0/02-综合优化清单.md` | 24项 S0+P1 + 48项 P2/P3 遗留 | ✅ 已完成 |
| 03 | `v2.0.0/03-closeout.md` | 5大功能验收, S0 7/7, P1 17/17, 架构决策5项 | ✅ 已完成 |

### v1.1.0 — 研究级回测与多标的策略支持

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v1.1.0/01-规划方案.md` | S0/P1/P2 方案，含 4 个决策纪录，GP 对照表 | ✅ 已完成 (16/16) |
| 02 | `v1.1.0/02-综合优化清单.md` | 16 项按 S0/P1/P2 分组，含依赖排序和估时 | ✅ 已收口 |
| 03 | `../05-testing/v1.1.0-审计报告.md` | 五维度评分 9.5/10，GP 合规 30/32 ✅ | ✅ 已完成 |

### v1.0.7 — 体验收口与国际化补齐

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v1.0.7/01-规划方案.md` | v1.0.6 延期项收口 + en-US 补齐 + CSS 质量收口；附优化方法；背景、原则、范围、验收总闸 | ✅ 已完成 (9/9) |

### v1.0.6 — 用户困惑点全量优化

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v1.0.6/01-规划方案.md` | 四维度全量用户困惑点审计；背景、原则、范围、验收总闸 | ✅ 完成 |
| 02 | `v1.0.6/02-综合优化清单.md` | 79 项 S0/P1/P2/P3：错误汉化 / 术语统一 / API 格式改善 / 诊断修复 | ✅ 已收口 (79/79) |

### v1.0.5 — 前端样式深度修复 + 六轮审计收口

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v1.0.5/01-规划方案.md` | 前端样式深度修复 | ✅ 完成 |
| 02-11 | `v1.0.5/02-11` | 五轮审计报告 + 优化方案 + OpenAPI/分页方案 | ✅ 完成 |

### v1.0.3 — 边界防御

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v1.0.3/01-规划方案.md` | 第四轮审计残量；边界条件、NaN防护、并发安全 | ✅ 完成 |
| 02 | `v1.0.3/02-综合优化清单.md` | 15 项 S0/P1/P2：时间校验 / NaN守卫 / 编译并发 / 运行互斥 / 端口消息 | ✅ 完成 |

### v0.5.2 — 全量排雷收口

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v0.5.2/01-规划方案.md` | 基于 2026-05-10 五维度全量审计；背景、原则、范围、验收总闸 | ✅ 完成 |
| 02 | `v0.5.2/02-综合优化清单.md` | 16 项 S0/P1/P2：测试套件修复 / 架构排雷 / 存储配额激活 / 质量收口 | ✅ 16/16 完成 |

### v0.5.1 — 全量审计收口排雷

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v0.5.1/01-规划方案.md` | 基于 2026-05-10 独立全量审计 | ✅ 已完成 |
| 02 | `v0.5.1/02-综合优化清单.md` | 15 项 P0/P1/P2：编译路径统一 / 错误汉化 / 存储配额 / 合规收口 / 质量收口 | ✅ 15/15 完成 |

### v0.5.0 — Adobe 风格前端全量重构 + 全量审计

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v0.5.0/01-Adobe风格前端重构方案.md` | 5 Phase：设计系统 / App Shell / 工作区面板化 / 组件重设计 / 收尾 | ✅ 已完成 |
| 02 | `v0.5.0/02-General_Policy全量审计.md` | §1-§8 38 项全量合规审计 | ✅ 已完成 |
| 03 | `v0.5.0/03-扩展审计清单.md` | 6 角度 30 项 + R1/R2/R3 回归审计 13 发现 | ✅ 已完成 |
| 04 | `v0.5.0/04-综合优化清单.md` | 18 项 S0/P1/P2：安全加固 / 体验修复 / 质量收口 | ✅ 13/18 完成 |
| 05 | `v0.5.0/05-遗留问题决策方案.md` | 5 项遗留：JSON 汉化 / 凭证引导 / 测试修复 / npm audit / CI 补全 | 📋 待决策 |

### v0.4.3 — 用户体验与安全收口

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v0.4.3/01-用户体验与安全收口.md` | 5 项：JSON 错误汉化 / API 文档 / CLI 安全 / 强制认证 / Vault 懒初始化 | ✅ 已完成 |

### v0.4.2 — 收口排雷

| # | 文档 | 内容 | 状态 |
|---|------|------|:--:|
| 01 | `v0.4.2/01-收口排雷清单.md` | 10 项排雷/收口/优化：raw print 迁移 / 废弃 API 清理 / 测试修复 / Tauri 启动 | ✅ 已完成 |

---

## 已完成版本摘要

| 版本 | 核心交付 | 文档数 |
|------|---------|:--:|
| v0.1.0 | 私有基线：paper 运行时、图编辑器、QS 编译管道 | — |
| v0.2.0 | TestRunner + @test/@step/@assert 指令 + CI | 1 |
| v0.3.0 | 22 项 P0-P3 修复 + 10 个新信号 + 调试工作区 | 7 |
| v0.4.0 | UI 简洁化 + 教程 + 凭证安全 (3 工作线 10 项) | 5 |
| v0.4.1 | 安全审计 12 项修复 + 凭证按标签存取 + 死代码清理 | 3 |
| v0.5.0~v0.5.2 | 前端重构 + 全量审计 + 排雷收口 | 10 |
| v1.0.3~v1.0.7 | 边界防御 + 用户困惑点 + 体验收口 + 国际化 | 15 |
| v1.1.0 | 研究级回测与多标的策略支持 | 3 |
| v2.0.0~v2.3.3 | MAJOR执行端 + S0/P1闭环 + 架构重构 | 10 |
| v3.0.0~v3.1.0 | 实时执行端 + 审计闭环 | 5 |
| v3.2.0~v3.3.0 | P3消化 + 实盘模式 + 全量交付 | 3 |
| v3.4.0~v3.4.1 | MINOR + GP回归阻断修复 | 3 |
| **v3.5.0~v3.5.1** | **GP全合规 + P2质量收口 + 审计P1清零** | **5** |
| v3.6.0~v3.7.2 | 用户体验优化 + P3消化 + Closeout收口 | 8 |
| v4.0.0 | **MAJOR: 状态机化架构 8 Phase** | 3 |
| v4.1.0 | v4 runtime 加固 + P2 消化 + 用户入口 | 1 |
| v4.2.0 | 执行端 v4 集成 + P3 消化 | 1 |
| v4.3.0 | v4 回测 + 多交易对 + v4 模板库 | 1 |
| v4.4.0 | 嵌套状态机第一波 (深度=2) | 2 |
| v4.5.0~v4.6.0 | 高级订单+Tick回放 / LiveActual+OKX边界 | 4 |
| **v4.7.0** | **v4 AI提案+性能优化+两轮诱错50发现+元流水线** | **5** |
| v4.8.0 | 双执行切面 + P2 质量收敛 | 6 |
| v4.8.1 | API 契约 + 部署治理超级规范化 | 3 |
| v4.8.2 | 产品/UX/i18n 收敛 | 3 |
| v4.9.0 | 产品功能完整度 + 插件执行安全 | 3 |

详细文档分别位于对应版本目录下，审计报告位于 `../05-testing/`。

---

## 过程追踪

`tracking/` 和各历史版本目录下为历史开发过程产生的优化追踪文档和审计报告，仅供历史参考，不反映当前开发方向。
