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
| 76 | `v4.16.0/76-runtime.backtest抽离记录.md` | BE-001M-03 `runtime.backtest` 抽离记录，将 backtest route group 迁入 `src/backend/runtime/routes/backtest.rs` 并保留 handler/artifact/compare/persistence owner | ✅ 已完成 |
| 77 | `v4.16.0/77-runtime.backtest单叶closeout.md` | BE-001M-04 `runtime.backtest` 单叶 closeout，确认 route facade 等价并判断 handler 域值得进入 `runtime.backtest.execution_start` 基线 | ✅ 已完成 |
| 78 | `v4.16.0/78-runtime.backtest.execution_start单子叶等价基线.md` | BE-001N-01 `runtime.backtest.execution_start` 单子叶等价基线，冻结 backtest 创建路径、legacy/v4 执行入口和排除边界；当前不移动代码 | 📋 已创建 |
| 79 | `v4.16.0/79-runtime.backtest.execution_start抽离方案.md` | BE-001N-02 `runtime.backtest.execution_start` 抽离方案，锁定下一批只移动 backtest 创建路径 handler/helper 并保留 experiment 复用桥 | 📋 已创建 |
| 80 | `v4.16.0/80-runtime.backtest.execution_start抽离记录.md` | BE-001N-03 `runtime.backtest.execution_start` 抽离记录，将 backtest 创建路径 handler/helper 迁入 `src/runtime/backtest/execution_start.rs` 并保留父级兼容桥 | ✅ 已完成 |
| 81 | `v4.16.0/81-runtime.backtest.execution_start单叶closeout.md` | BE-001N-04 `runtime.backtest.execution_start` 单叶 closeout，确认等价并判断内部 `v4_projection` 值得继续细拆 | ✅ 已完成 |
| 82 | `v4.16.0/82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md` | BE-001O-01 `runtime.backtest.execution_start.v4_projection` 单子叶等价基线，冻结 v4 artifact projection helper 与排除边界；当前不移动代码 | 📋 已创建 |
| 83 | `v4.16.0/83-runtime.backtest.execution_start.v4_projection抽离方案.md` | BE-001O-02 `runtime.backtest.execution_start.v4_projection` 抽离方案，锁定下一批只移动 projection helper 与现有单元测试 | 📋 已创建 |
| 84 | `v4.16.0/84-runtime.backtest.execution_start.v4_projection抽离记录.md` | BE-001O-03 `runtime.backtest.execution_start.v4_projection` 抽离记录，将 projection helper 与现有单元测试迁入 `src/runtime/backtest/v4_projection.rs` | ✅ 已完成 |
| 85 | `v4.16.0/85-runtime.backtest.execution_start.v4_projection单叶closeout.md` | BE-001O-04 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，确认等价并设置 `stop_split: true`，下一候选回到 `v4_request_resolution` | ✅ 已完成 |
| 86 | `v4.16.0/86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md` | BE-001P-01 `runtime.backtest.execution_start.v4_request_resolution` 单子叶等价基线，冻结 v4 请求识别、graph/symbol/event resolution；当前不移动代码 | 📋 已创建 |
| 87 | `v4.16.0/87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md` | BE-001P-02 `runtime.backtest.execution_start.v4_request_resolution` 抽离方案，锁定下一批只移动四个 request resolution helper | 📋 已创建 |
| 88 | `v4.16.0/88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md` | BE-001P-03 `runtime.backtest.execution_start.v4_request_resolution` 抽离记录，将四个 request resolution helper 迁入父级私有子模块 | 📋 已创建 |
| 89 | `v4.16.0/89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md` | BE-001P-04 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout，确认等价并设置 `stop_split: true` | ✅ 已完成 |
| 90 | `v4.16.0/90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md` | BE-001Q-01 `runtime.backtest.execution_start.v4_runtime_execution` 单子叶等价基线，冻结 deterministic replay、v4 runtime execution 和 artifact output；当前不移动代码 | 📋 已创建 |
| 91 | `v4.16.0/91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md` | BE-001Q-02 `runtime.backtest.execution_start.v4_runtime_execution` 抽离方案，限定下一批只迁移 deterministic runtime execution 最小 helper | 📋 已创建 |
| 92 | `v4.16.0/92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md` | BE-001Q-03 `runtime.backtest.execution_start.v4_runtime_execution` 抽离记录，将 deterministic bars/ticks 与 blocking runtime replay 迁入父级私有子模块 | 📋 已创建 |
| 93 | `v4.16.0/93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md` | BE-001Q-04 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout，确认等价并设置 `stop_split: true` | ✅ 已完成 |
| 94 | `v4.16.0/94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md` | BE-001R-01 `runtime.backtest.execution_start.legacy_dispatch` 单子叶等价基线，冻结 legacy compile/sandbox dispatch 且当前不移动代码 | 📋 已创建 |
| 95 | `v4.16.0/95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md` | BE-001R-02 `runtime.backtest.execution_start.legacy_dispatch` 抽离方案，限定下一批只迁移 legacy compile/sandbox dispatch 最小 helper | 📋 已创建 |
| 96 | `v4.16.0/96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md` | BE-001R-03 `runtime.backtest.execution_start.legacy_dispatch` 抽离记录，将 legacy compile/sandbox dispatch 迁入父级私有子模块 | 📋 已创建 |
| 97 | `v4.16.0/97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md` | BE-001R-04 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout，确认等价并设置 `stop_split: true` | ✅ 已完成 |
| 98 | `v4.16.0/98-runtime.backtest.execution_start父叶残余判断.md` | BE-001S-01 `runtime.backtest.execution_start` 父叶残余判断，确认回到 `runtime.backtest.record_store` 上层队列 | ✅ 已完成 |
| 99 | `v4.16.0/99-runtime.backtest.record_store单子叶等价基线.md` | BE-001T-01 `runtime.backtest.record_store` 单子叶等价基线，冻结 backtest list/detail/save/discard 边界 | 📋 已创建 |
| 100 | `v4.16.0/100-runtime.backtest.record_store抽离方案.md` | BE-001T-02 `runtime.backtest.record_store` 抽离方案，限定下一批只迁移四个 handler 并保留 shared owner | 📋 已创建 |
| 101 | `v4.16.0/101-runtime.backtest.record_store抽离记录.md` | BE-001T-03 `runtime.backtest.record_store` 抽离记录，将四个 handler 迁入 `src/runtime/backtest/record_store.rs` | 📋 已创建 |
| 102 | `v4.16.0/102-runtime.backtest.record_store单叶closeout.md` | BE-001T-04 `runtime.backtest.record_store` 单叶 closeout，确认等价并设置 `stop_split: true` | ✅ 已完成 |
| 103 | `v4.16.0/103-runtime.backtest.replay单子叶等价基线.md` | BE-001U-01 `runtime.backtest.replay` 单子叶等价基线，冻结 replay route、query、response mapping 和 metrics 边界 | 📋 已创建 |
| 104 | `v4.16.0/104-runtime.backtest.replay抽离方案.md` | BE-001U-02 `runtime.backtest.replay` 抽离方案，限定下一批只迁移 `get_backtest_replay` 并保留 shared owner | 📋 已创建 |
| 105 | `v4.16.0/105-runtime.backtest.replay抽离记录.md` | BE-001U-03 `runtime.backtest.replay` 抽离记录，将 `get_backtest_replay` 迁入 `src/runtime/backtest/replay.rs` | 📋 已创建 |
| 106 | `v4.16.0/106-runtime.backtest.replay单叶closeout.md` | BE-001U-04 `runtime.backtest.replay` 单叶 closeout，确认等价并设置 `stop_split: true` | 📋 已创建 |
| 107 | `v4.16.0/107-runtime.backtest.experiment_sweep单子叶等价基线.md` | BE-001V-01 `runtime.backtest.experiment_sweep` 单子叶等价基线，冻结 experiment routes、参数网格、复用桥和生命周期边界 | 📋 已创建 |
| 108 | `v4.16.0/108-runtime.backtest.experiment_sweep抽离方案.md` | BE-001V-02 `runtime.backtest.experiment_sweep` 抽离方案，限定下一批只迁移 experiment handler/helper | 📋 已创建 |
| 109 | `v4.16.0/109-runtime.backtest.experiment_sweep抽离记录.md` | BE-001V-03 `runtime.backtest.experiment_sweep` 抽离记录，将 experiment handler/helper 迁入 `src/runtime/backtest/experiment_sweep.rs` | 📋 已创建 |
| 110 | `v4.16.0/110-runtime.backtest.experiment_sweep单叶closeout.md` | BE-001V-04 `runtime.backtest.experiment_sweep` 单叶 closeout，确认等价并登记 `parameter_grid` 下一候选 | 📋 已创建 |
| 111 | `v4.16.0/111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md` | BE-001W-01 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线，冻结参数网格 helper 边界 | 📋 已创建 |
| 112 | `v4.16.0/112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md` | BE-001W-02 `runtime.backtest.experiment_sweep.parameter_grid` 抽离方案，限定下一批只迁移 3 个 helper | 📋 已创建 |
| 113 | `v4.16.0/113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md` | BE-001W-03 `runtime.backtest.experiment_sweep.parameter_grid` 抽离记录，将 3 个 helper 迁入 `src/runtime/backtest/parameter_grid.rs` | 📋 已创建 |
| 114 | `v4.16.0/114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md` | BE-001W-04 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，确认等价并设置 `stop_split: true` | 📋 已创建 |
| 115 | `v4.16.0/115-runtime.backtest.experiment_sweep父叶残余判断.md` | BE-001X-01 `runtime.backtest.experiment_sweep` 父叶残余判断，下一候选为 `start_orchestration` | 📋 已创建 |
| 116 | `v4.16.0/116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md` | BE-001Y-01 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线，当前 `no code movement` | 📋 已创建 |
| 117 | `v4.16.0/117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md` | BE-001Y-02 `runtime.backtest.experiment_sweep.start_orchestration` 抽离方案，限定下一批只迁移 `start_backtest_experiment` | 📋 已创建 |
| 118 | `v4.16.0/118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md` | BE-001Y-03 `runtime.backtest.experiment_sweep.start_orchestration` 抽离记录，将 `start_backtest_experiment` 迁入 `src/runtime/backtest/start_orchestration.rs` | 📋 已创建 |
| 119 | `v4.16.0/119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md` | BE-001Y-04 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout，确认等价并设置 `stop_split: true` | 📋 已创建 |
| 120 | `v4.16.0/120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md` | BE-001Z-01 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，下一候选为 `record_lifecycle` | 📋 已创建 |
| 121 | `v4.16.0/121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md` | BE-001AA-01 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，当前 `no code movement` | 📋 已创建 |
| 122 | `v4.16.0/122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md` | BE-001AA-02 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案，限定下一批只迁移四个 lifecycle handler | 📋 已创建 |
| 123 | `v4.16.0/123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md` | BE-001AA-03 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离记录，四个 lifecycle handler 已迁入 `src/runtime/backtest/record_lifecycle.rs` | ✅ 已完成 |
| 124 | `v4.16.0/124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md` | BE-001AA-04 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout，确认等价并设置 `stop_split: true` | ✅ 已完成 |
| 125 | `v4.16.0/125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md` | BE-001AB-01 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断，确认三子叶均已 closeout 并设置父叶 `stop_split: true` | ✅ 已完成 |
| 126 | `v4.16.0/126-runtime.backtest父叶残余判断.md` | BE-001AC-01 `runtime.backtest` 父叶残余判断，确认当前 handler 域已收束并设置父叶 `stop_split: true` | ✅ 已完成 |
| 127 | `v4.16.0/127-backend.runtime.routes父叶残余判断.md` | BE-001AD-01 `backend.runtime.routes` 父叶残余判断，确认 route aggregate 仍有 mutation 等残余候选并保持 `stop_split: false` | ✅ 已完成 |
| 128 | `v4.16.0/128-backend.runtime.routes.mutation单子叶等价基线.md` | BE-001AE-01 `backend.runtime.routes.mutation` 单子叶等价基线，冻结 mutation / AI proposal / approval route group | ✅ 已完成 |
| 129 | `v4.16.0/129-backend.runtime.routes.mutation抽离方案.md` | BE-001AE-02 `backend.runtime.routes.mutation` 抽离方案，只规划 route facade 最小迁移 | 📋 已创建 |
| 130 | `v4.16.0/130-backend.runtime.routes.mutation抽离记录.md` | BE-001AE-03 `backend.runtime.routes.mutation` route facade 实际抽离记录 | 📋 已创建 |
| 131 | `v4.16.0/131-backend.runtime.routes.mutation单叶closeout.md` | BE-001AE-04 `backend.runtime.routes.mutation` 单叶 closeout，确认 route facade 等价并设置 `stop_split: true` | 📋 已创建 |
| 132 | `v4.16.0/132-runtime.mutation.parameter_mutation单子叶等价基线.md` | BE-001AF-01 `runtime.mutation.parameter_mutation` 单子叶等价基线，冻结参数变更 handler 生命周期 | 📋 已创建 |
| 133 | `v4.16.0/133-runtime.mutation.parameter_mutation抽离方案.md` | BE-001AF-02 `runtime.mutation.parameter_mutation` 抽离方案，固定目标子模块、父级 re-export 和 shared helper 保留边界 | 📋 已创建 |
| 134 | `v4.16.0/134-runtime.mutation.parameter_mutation抽离记录.md` | BE-001AF-03 `runtime.mutation.parameter_mutation` 抽离记录，将参数变更 handler 迁入子模块并保留父级兼容出口 | 📋 已创建 |
| 135 | `v4.16.0/135-runtime.mutation.parameter_mutation单叶closeout.md` | BE-001AF-04 `runtime.mutation.parameter_mutation` 单叶 closeout，判定继续细拆并登记 transition lifecycle 下一基线 | 📋 已创建 |
| 136 | `v4.16.0/136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md` | BE-001AG-01 `transition_lifecycle` 单子叶等价基线，冻结 activation / rollback lifecycle | 📋 已创建 |
| 137 | `v4.16.0/137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md` | BE-001AG-02 `transition_lifecycle` 抽离方案，固定目标文件、父级出口和迁移清单 | 📋 已创建 |
| 138 | `v4.16.0/138-runtime.mutation.parameter_mutation.transition_lifecycle抽离记录.md` | BE-001AG-03 `transition_lifecycle` 实际抽离记录，迁移 activation / rollback handler | 📋 已创建 |
| 139 | `v4.16.0/139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md` | BE-001AG-04 `transition_lifecycle` 单叶 closeout，设置 `stop_split: false` 并登记 boundary_safety 下一基线 | 📋 已创建 |
| 140 | `v4.16.0/140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md` | BE-001AH-01 `boundary_safety` 单子叶等价基线，冻结 boundary / safe window 纯策略 | 📋 已创建 |
| 141 | `v4.16.0/141-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离方案.md` | BE-001AH-02 `boundary_safety` 抽离方案，固定目标文件和 wrapper 方式 | 📋 已创建 |
| 142 | `v4.16.0/142-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety抽离记录.md` | BE-001AH-03 `boundary_safety` 实际抽离，迁移 boundary / safe-window helper | 📋 已创建 |
| 143 | `v4.16.0/143-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单叶closeout.md` | BE-001AH-04 `boundary_safety` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 144 | `v4.16.0/144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md` | BE-001AI-01 `transition_lifecycle` 父叶残余判断，父叶保持 `stop_split: false`，下一步进入 `activation_flow` 基线 | 📋 已创建 |
| 145 | `v4.16.0/145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md` | BE-001AJ-01 `activation_flow` 单子叶等价基线，冻结 activation handler 状态机 | 📋 已创建 |
| 146 | `v4.16.0/146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md` | BE-001AJ-02 `activation_flow` 抽离方案，固定目标文件与父级 re-export | 📋 已创建 |
| 147 | `v4.16.0/147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md` | BE-001AJ-03 `activation_flow` 实际抽离，迁移 activation public handler | 📋 已创建 |
| 148 | `v4.16.0/148-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单叶closeout.md` | BE-001AJ-04 `activation_flow` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 149 | `v4.16.0/149-runtime.mutation.parameter_mutation.transition_lifecycle第二轮父叶残余判断.md` | BE-001AK-01 `transition_lifecycle` 第二轮父叶残余判断，下一候选为 `rollback_flow` | 📋 已创建 |
| 150 | `v4.16.0/150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md` | BE-001AL-01 `rollback_flow` 单子叶等价基线，冻结 rollback transaction 状态机 | 📋 已创建 |
| 151 | `v4.16.0/151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md` | BE-001AL-02 `rollback_flow` 抽离方案，固定目标文件与父级 re-export | 📋 已创建 |
| 152 | `v4.16.0/152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md` | BE-001AL-03 `rollback_flow` 实际抽离，迁移 rollback public handler | 📋 已创建 |
| 153 | `v4.16.0/153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md` | BE-001AL-04 `rollback_flow` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 154 | `v4.16.0/154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md` | BE-001AM-01 `transition_lifecycle` 第三轮父叶残余判断，下一候选为 `activation_snapshot_side_effect` | 📋 已创建 |
| 155 | `v4.16.0/155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md` | BE-001AN-01 `activation_snapshot_side_effect` 单子叶等价基线 | 📋 已创建 |
| 156 | `v4.16.0/156-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离方案.md` | BE-001AN-02 `activation_snapshot_side_effect` 抽离方案 | 📋 已创建 |
| 157 | `v4.16.0/157-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect抽离记录.md` | BE-001AN-03 `activation_snapshot_side_effect` 实际抽离 | 📋 已创建 |
| 158 | `v4.16.0/158-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单叶closeout.md` | BE-001AN-04 `activation_snapshot_side_effect` 单叶 closeout | 📋 已创建 |
| 159 | `v4.16.0/159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md` | BE-001AO-01 `transition_lifecycle` 第四轮父叶残余判断，下一候选为 `transition_record_persistence` | 📋 已创建 |
| 160 | `v4.16.0/160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md` | BE-001AP-01 `transition_record_persistence` 单子叶等价基线，冻结 lifecycle entry 与 transition persistence | 📋 已创建 |
| 161 | `v4.16.0/161-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离方案.md` | BE-001AP-02 `transition_record_persistence` 抽离方案，固定目标文件、父级声明和回退点 | 📋 已创建 |
| 162 | `v4.16.0/162-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence抽离记录.md` | BE-001AP-03 `transition_record_persistence` 实际抽离，迁移 lifecycle entry 与 transition persistence helper | 📋 已创建 |
| 163 | `v4.16.0/163-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单叶closeout.md` | BE-001AP-04 `transition_record_persistence` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 164 | `v4.16.0/164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md` | BE-001AQ-01 `transition_lifecycle` 第五轮父叶残余判断，下一候选为 `rollback_record_identity` | 📋 已创建 |
| 165 | `v4.16.0/165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md` | BE-001AR-01 `rollback_record_identity` 单子叶等价基线，冻结 rollback id digest contract | 📋 已创建 |
| 166 | `v4.16.0/166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md` | BE-001AR-02 `rollback_record_identity` 抽离方案，固定目标文件、父级声明和回退点 | 📋 已创建 |
| 167 | `v4.16.0/167-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离记录.md` | BE-001AR-03 `rollback_record_identity` 实际抽离，迁移 rollback id helper | 📋 已创建 |
| 168 | `v4.16.0/168-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单叶closeout.md` | BE-001AR-04 `rollback_record_identity` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 169 | `v4.16.0/169-runtime.mutation.parameter_mutation.transition_lifecycle第六轮父叶残余判断.md` | BE-001AS-01 `transition_lifecycle` 第六轮父叶残余判断，父叶设置 `stop_split: true` | 📋 已创建 |
| 170 | `v4.16.0/170-runtime.mutation.parameter_mutation父叶残余判断.md` | BE-001AT-01 `parameter_mutation` 父叶残余判断，下一候选为 `proposal_creation` | 📋 已创建 |
| 171 | `v4.16.0/171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md` | BE-001AU-01 `proposal_creation` 单子叶等价基线，冻结 create handler 与 record id helper | 📋 已创建 |
| 172 | `v4.16.0/172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md` | BE-001AU-02 `proposal_creation` 抽离方案，固定目标文件、父级声明和 handler re-export | 📋 已创建 |
| 173 | `v4.16.0/173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md` | BE-001AU-03 `proposal_creation` 实际抽离，迁移 create handler 与 record id helper | 📋 已创建 |
| 174 | `v4.16.0/174-runtime.mutation.parameter_mutation.proposal_creation单叶closeout.md` | BE-001AU-04 `proposal_creation` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 175 | `v4.16.0/175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md` | BE-001AV-01 `parameter_mutation` 第二轮父叶残余判断，下一候选为 `record_query` | 📋 已创建 |
| 176 | `v4.16.0/176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md` | BE-001AW-01 `record_query` 单子叶等价基线，冻结 list/detail 查询流 | 📋 已创建 |
| 177 | `v4.16.0/177-runtime.mutation.parameter_mutation.record_query抽离方案.md` | BE-001AW-02 `record_query` 抽离方案，固定目标文件、父级声明和双 handler re-export | 📋 已创建 |
| 178 | `v4.16.0/178-runtime.mutation.parameter_mutation.record_query抽离记录.md` | BE-001AW-03 `record_query` 实际抽离，迁移 list/detail handler | 📋 已创建 |
| 179 | `v4.16.0/179-runtime.mutation.parameter_mutation.record_query单叶closeout.md` | BE-001AW-04 `record_query` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 180 | `v4.16.0/180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md` | BE-001AX-01 `parameter_mutation` 第三轮父叶残余判断，父叶设置 `stop_split: true` | 📋 已创建 |
| 181 | `v4.16.0/181-runtime.mutation.ai_proposal单子叶等价基线.md` | BE-001AY-01 `runtime.mutation.ai_proposal` 单子叶等价基线，冻结 AI proposal / approval handler 域 | 📋 已创建 |
| 182 | `v4.16.0/182-runtime.mutation.ai_proposal抽离方案.md` | BE-001AY-02 `runtime.mutation.ai_proposal` 抽离方案，固定目标文件、父级声明和 public handler re-export | 📋 已创建 |
| 183 | `v4.16.0/183-runtime.mutation.ai_proposal抽离记录.md` | BE-001AY-03 `runtime.mutation.ai_proposal` 实际抽离，迁移 AI proposal / approval handler | 📋 已创建 |
| 184 | `v4.16.0/184-runtime.mutation.ai_proposal单叶closeout.md` | BE-001AY-04 `runtime.mutation.ai_proposal` 单叶 closeout，设置 `stop_split: false`，下一候选为 `static_check` | 📋 已创建 |
| 185 | `v4.16.0/185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md` | BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线，冻结 validation / analysis helper | 📋 已创建 |
| 186 | `v4.16.0/186-runtime.mutation.ai_proposal.static_check抽离方案.md` | BE-001AZ-02 `runtime.mutation.ai_proposal.static_check` 抽离方案，固定目标文件、helper import 和 visibility | 📋 已创建 |
| 187 | `v4.16.0/187-runtime.mutation.ai_proposal.static_check抽离记录.md` | BE-001AZ-03 `runtime.mutation.ai_proposal.static_check` 实际抽离，helper 与静态检查单测迁入 child | 📋 已创建 |
| 188 | `v4.16.0/188-runtime.mutation.ai_proposal.static_check单叶closeout.md` | BE-001AZ-04 `runtime.mutation.ai_proposal.static_check` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 189 | `v4.16.0/189-runtime.mutation.ai_proposal父叶残余判断.md` | BE-001BA-01 `runtime.mutation.ai_proposal` 父叶残余判断，下一候选为 `source_governance_identity` | 📋 已创建 |
| 190 | `v4.16.0/190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md` | BE-001BB-01 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线 | 📋 已创建 |
| 191 | `v4.16.0/191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md` | BE-001BB-02 `runtime.mutation.ai_proposal.source_governance_identity` 抽离方案 | 📋 已创建 |
| 192 | `v4.16.0/192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md` | BE-001BB-03 `runtime.mutation.ai_proposal.source_governance_identity` 实际抽离记录 | 📋 已创建 |
| 193 | `v4.16.0/193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md` | BE-001BB-04 `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 194 | `v4.16.0/194-runtime.mutation.ai_proposal第二轮父叶残余判断.md` | BE-001BC-01 `runtime.mutation.ai_proposal` 第二轮父叶残余判断，下一候选为 `event_lifecycle` | 📋 已创建 |
| 195 | `v4.16.0/195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md` | BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线 | 📋 已创建 |
| 196 | `v4.16.0/196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md` | BE-001BD-02 `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案 | 📋 已创建 |
| 197 | `v4.16.0/197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md` | BE-001BD-03 `runtime.mutation.ai_proposal.event_lifecycle` 抽离记录 | 📋 已创建 |
| 198 | `v4.16.0/198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md` | BE-001BD-04 `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout | 📋 已创建 |
| 199 | `v4.16.0/199-runtime.mutation.ai_proposal第三轮父叶残余判断.md` | BE-001BE-01 `runtime.mutation.ai_proposal` 第三轮父叶残余判断 | 📋 已创建 |
| 200 | `v4.16.0/200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md` | BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线 | 📋 已创建 |
| 201 | `v4.16.0/201-runtime.mutation.ai_proposal.record_query抽离方案.md` | BE-001BF-02 `runtime.mutation.ai_proposal.record_query` 抽离方案 | 📋 已创建 |
| 202 | `v4.16.0/202-runtime.mutation.ai_proposal.record_query抽离记录.md` | BE-001BF-03 `runtime.mutation.ai_proposal.record_query` 抽离记录 | 📋 已创建 |
| 203 | `v4.16.0/203-runtime.mutation.ai_proposal.record_query单叶closeout.md` | BE-001BF-04 `runtime.mutation.ai_proposal.record_query` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 204 | `v4.16.0/204-runtime.mutation.ai_proposal第四轮父叶残余判断.md` | BE-001BG-01 `runtime.mutation.ai_proposal` 第四轮父叶残余判断，下一候选为 `approval_review` | 📋 已创建 |
| 205 | `v4.16.0/205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md` | BE-001BH-01 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线 | 📋 已创建 |
| 206 | `v4.16.0/206-runtime.mutation.ai_proposal.approval_review抽离方案.md` | BE-001BH-02 `runtime.mutation.ai_proposal.approval_review` 抽离方案 | 📋 已创建 |
| 207 | `v4.16.0/207-runtime.mutation.ai_proposal.approval_review抽离记录.md` | BE-001BH-03 `runtime.mutation.ai_proposal.approval_review` 实际抽离记录 | 📋 已创建 |
| 208 | `v4.16.0/208-runtime.mutation.ai_proposal.approval_review单叶closeout.md` | BE-001BH-04 `runtime.mutation.ai_proposal.approval_review` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 209 | `v4.16.0/209-runtime.mutation.ai_proposal第五轮父叶残余判断.md` | BE-001BI-01 `runtime.mutation.ai_proposal` 第五轮父叶残余判断，下一候选为 `approval_persistence` | 📋 已创建 |
| 210 | `v4.16.0/210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md` | BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线 | 📋 已创建 |
| 211 | `v4.16.0/211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md` | BE-001BJ-02 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案 | 📋 已创建 |
| 212 | `v4.16.0/212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md` | BE-001BJ-03 `runtime.mutation.ai_proposal.approval_persistence` 实际抽离记录 | 📋 已创建 |
| 213 | `v4.16.0/213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md` | BE-001BJ-04 `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 214 | `v4.16.0/214-runtime.mutation.ai_proposal第六轮父叶残余判断.md` | BE-001BK-01 `runtime.mutation.ai_proposal` 第六轮父叶残余判断，下一候选为 `sandbox_trigger` | 📋 已创建 |
| 215 | `v4.16.0/215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md` | BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线 | 📋 已创建 |
| 216 | `v4.16.0/216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md` | BE-001BL-02 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案 | 📋 已创建 |
| 217 | `v4.16.0/217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md` | BE-001BL-03 `runtime.mutation.ai_proposal.sandbox_trigger` 实际抽离记录 | 📋 已创建 |
| 218 | `v4.16.0/218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md` | BE-001BL-04 `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 219 | `v4.16.0/219-runtime.mutation.ai_proposal第七轮父叶残余判断.md` | BE-001BM-01 `runtime.mutation.ai_proposal` 第七轮父叶残余判断，下一候选为 `status_transition` | 📋 已创建 |
| 220 | `v4.16.0/220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md` | BE-001BN-01 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线 | 📋 已创建 |
| 221 | `v4.16.0/221-runtime.mutation.ai_proposal.status_transition抽离方案.md` | BE-001BN-02 `runtime.mutation.ai_proposal.status_transition` 抽离方案 | 📋 已创建 |
| 222 | `v4.16.0/222-runtime.mutation.ai_proposal.status_transition抽离记录.md` | BE-001BN-03 `runtime.mutation.ai_proposal.status_transition` 实际抽离记录 | 📋 已创建 |
| 223 | `v4.16.0/223-runtime.mutation.ai_proposal.status_transition单叶closeout.md` | BE-001BN-04 `runtime.mutation.ai_proposal.status_transition` 单叶 closeout | 📋 已创建 |
| 224 | `v4.16.0/224-runtime.mutation.ai_proposal第八轮父叶残余判断.md` | BE-001BO-01 `runtime.mutation.ai_proposal` 第八轮父叶残余判断，下一候选为 `proposal_creation` | 📋 已创建 |
| 225 | `v4.16.0/225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md` | BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线 | 📋 已创建 |
| 226 | `v4.16.0/226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md` | BE-001BP-02 `runtime.mutation.ai_proposal.proposal_creation` 抽离方案 | 📋 已创建 |
| 227 | `v4.16.0/227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md` | BE-001BP-03 `runtime.mutation.ai_proposal.proposal_creation` 实际抽离记录 | 📋 已创建 |
| 228 | `v4.16.0/228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md` | BE-001BP-04 `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout | 📋 已创建 |
| 229 | `v4.16.0/229-runtime.mutation.ai_proposal第九轮父叶残余判断.md` | BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断 | 📋 已创建 |
| 230 | `v4.16.0/230-backend.runtime.routes第二轮父叶残余判断.md` | BE-001BR-01 `backend.runtime.routes` 第二轮父叶残余判断 | 📋 已创建 |
| 231 | `v4.16.0/231-backend.runtime.routes.experiment单子叶等价基线.md` | BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线 | 📋 已创建 |
| 232 | `v4.16.0/232-backend.runtime.routes.experiment抽离方案.md` | BE-001BS-02 `backend.runtime.routes.experiment` 抽离方案 | 📋 已创建 |
| 233 | `v4.16.0/233-backend.runtime.routes.experiment抽离记录.md` | BE-001BS-03 `backend.runtime.routes.experiment` 实际抽离记录 | 📋 已创建 |
| 234 | `v4.16.0/234-backend.runtime.routes.experiment单叶closeout.md` | BE-001BS-04 `backend.runtime.routes.experiment` 单叶 closeout | 📋 已创建 |
| 235 | `v4.16.0/235-backend.runtime.routes第三轮父叶残余判断.md` | BE-001BT-01 `backend.runtime.routes` 第三轮父叶残余判断 | 📋 已创建 |
| 236 | `v4.16.0/236-backend.runtime.routes.evidence单子叶等价基线.md` | BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线 | 📋 已创建 |
| 237 | `v4.16.0/237-backend.runtime.routes.evidence抽离方案.md` | BE-001BU-02 `backend.runtime.routes.evidence` 抽离方案 | 📋 已创建 |
| 238 | `v4.16.0/238-backend.runtime.routes.evidence抽离记录.md` | BE-001BU-03 `backend.runtime.routes.evidence` 实际抽离记录 | 📋 已创建 |
| 239 | `v4.16.0/239-backend.runtime.routes.evidence单叶closeout.md` | BE-001BU-04 `backend.runtime.routes.evidence` 单叶 closeout | 📋 已创建 |
| 240 | `v4.16.0/240-backend.runtime.routes第四轮父叶残余判断.md` | BE-001BV-01 `backend.runtime.routes` 第四轮父叶残余判断，下一候选为 `event_stream` | 📋 已创建 |
| 241 | `v4.16.0/241-backend.runtime.routes.event_stream单子叶等价基线.md` | BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线 | 📋 已创建 |
| 242 | `v4.16.0/242-backend.runtime.routes.event_stream抽离方案.md` | BE-001BW-02 `backend.runtime.routes.event_stream` 抽离方案 | 📋 已创建 |
| 243 | `v4.16.0/243-backend.runtime.routes.event_stream抽离记录.md` | BE-001BW-03 `backend.runtime.routes.event_stream` 实际抽离记录 | 📋 已创建 |
| 244 | `v4.16.0/244-backend.runtime.routes.event_stream单叶closeout.md` | BE-001BW-04 `backend.runtime.routes.event_stream` 单叶 closeout | 📋 已创建 |
| 245 | `v4.16.0/245-backend.runtime.routes第五轮父叶残余判断.md` | BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断，下一候选为 `report_ops` | 📋 已创建 |
| 246 | `v4.16.0/246-backend.runtime.routes.report_ops单子叶等价基线.md` | BE-001BY-01 `backend.runtime.routes.report_ops` 单子叶等价基线 | 📋 已创建 |
| 247 | `v4.16.0/247-backend.runtime.routes.report_ops抽离方案.md` | BE-001BY-02 `backend.runtime.routes.report_ops` 抽离方案 | 📋 已创建 |
| 248 | `v4.16.0/248-backend.runtime.routes.report_ops抽离记录.md` | BE-001BY-03 `backend.runtime.routes.report_ops` 实际抽离记录 | 📋 已创建 |
| 249 | `v4.16.0/249-backend.runtime.routes.report_ops单叶closeout.md` | BE-001BY-04 `backend.runtime.routes.report_ops` 单叶 closeout | 📋 已创建 |
| 250 | `v4.16.0/250-backend.runtime.routes第六轮父叶残余判断.md` | BE-001BZ-01 `backend.runtime.routes` 第六轮父叶残余判断，route aggregate 收口 | 📋 已创建 |
| 251 | `v4.16.0/251-backend.runtime父叶残余判断.md` | BE-001CA-01 `backend.runtime` 父叶残余判断，选择 `runtime.report_ops` 下一候选 | 📋 已创建 |
| 252 | `v4.16.0/252-runtime.report_ops单子叶等价基线.md` | BE-001CB-01 `runtime.report_ops` 单子叶等价基线 | 📋 已创建 |
| 253 | `v4.16.0/253-runtime.report_ops抽离方案.md` | BE-001CB-02 `runtime.report_ops` 抽离方案 | 📋 已创建 |
| 254 | `v4.16.0/254-runtime.report_ops抽离记录.md` | BE-001CB-03 `runtime.report_ops` 实际抽离记录 | 📋 已创建 |
| 255 | `v4.16.0/255-runtime.report_ops单叶closeout.md` | BE-001CB-04 `runtime.report_ops` 单叶 closeout，设置 `stop_split: false` | 📋 已创建 |
| 256 | `v4.16.0/256-runtime.report_ops.runtime_report单子叶等价基线.md` | BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线 | 📋 已创建 |
| 257 | `v4.16.0/257-runtime.report_ops.runtime_report抽离方案.md` | BE-001CC-02 `runtime.report_ops.runtime_report` 抽离方案 | 📋 已创建 |
| 258 | `v4.16.0/258-runtime.report_ops.runtime_report抽离记录.md` | BE-001CC-03 `runtime.report_ops.runtime_report` 实际抽离记录 | 📋 已创建 |
| 259 | `v4.16.0/259-runtime.report_ops.runtime_report单叶closeout.md` | BE-001CC-04 `runtime.report_ops.runtime_report` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 260 | `v4.16.0/260-runtime.report_ops父叶残余判断.md` | BE-001CD-01 `runtime.report_ops` 父叶残余判断，选择 `v1_report_endpoints` 下一候选 | 📋 已创建 |
| 261 | `v4.16.0/261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md` | BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线 | 📋 已创建 |
| 262 | `v4.16.0/262-runtime.report_ops.v1_report_endpoints抽离方案.md` | BE-001CE-02 `runtime.report_ops.v1_report_endpoints` test-first 抽离方案 | 📋 已创建 |
| 263 | `v4.16.0/263-runtime.report_ops.v1_report_endpoints补测记录.md` | BE-001CE-03 `runtime.report_ops.v1_report_endpoints` endpoint smoke 补测记录 | 📋 已创建 |
| 264 | `v4.16.0/264-runtime.report_ops.v1_report_endpoints抽离记录.md` | BE-001CE-04 `runtime.report_ops.v1_report_endpoints` 实际抽离记录 | 📋 已创建 |
| 265 | `v4.16.0/265-runtime.report_ops.v1_report_endpoints单叶closeout.md` | BE-001CE-05 `runtime.report_ops.v1_report_endpoints` 单叶 closeout | 📋 已创建 |
| 266 | `v4.16.0/266-runtime.report_ops父叶残余判断.md` | BE-001CF-01 `runtime.report_ops` 父叶残余判断 | 📋 已创建 |
| 267 | `v4.16.0/267-runtime.report_ops.merge_generation_health单子叶等价基线.md` | BE-001CG-01 `runtime.report_ops.merge_generation_health` 单子叶等价基线 | 📋 已创建 |
| 268 | `v4.16.0/268-runtime.report_ops.merge_generation_health抽离方案.md` | BE-001CG-02 `runtime.report_ops.merge_generation_health` test-first 抽离方案 | 📋 已创建 |
| 269 | `v4.16.0/269-runtime.report_ops.merge_generation_health补测记录.md` | BE-001CG-03 `runtime.report_ops.merge_generation_health` endpoint smoke 补测记录 | 📋 已创建 |
| 270 | `v4.16.0/270-runtime.report_ops.merge_generation_health抽离记录.md` | BE-001CG-04 `runtime.report_ops.merge_generation_health` 实际抽离记录 | 📋 已创建 |
| 271 | `v4.16.0/271-runtime.report_ops.merge_generation_health单叶closeout.md` | BE-001CG-05 `runtime.report_ops.merge_generation_health` 单叶 closeout | 📋 已创建 |
| 272 | `v4.16.0/272-runtime.report_ops第二轮父叶残余判断.md` | BE-001CH-01 `runtime.report_ops` 第二轮父叶残余判断 | 📋 已创建 |
| 273 | `v4.16.0/273-backend.runtime第二轮父叶残余判断.md` | BE-001CI-01 `backend.runtime` 第二轮父叶残余判断 | 📋 已创建 |
| 274 | `v4.16.0/274-runtime.evidence_health单子叶等价基线.md` | BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线 | 📋 已创建 |
| 275 | `v4.16.0/275-runtime.evidence_health抽离方案.md` | BE-001CJ-02 `runtime.evidence_health` 抽离方案 | 📋 已创建 |
| 276 | `v4.16.0/276-runtime.evidence_health抽离记录.md` | BE-001CJ-03 `runtime.evidence_health` 实际抽离 | 📋 已创建 |
| 277 | `v4.16.0/277-runtime.evidence_health单叶closeout.md` | BE-001CJ-04 `runtime.evidence_health` 单叶 closeout | 📋 已创建 |
| 278 | `v4.16.0/278-backend.runtime第三轮父叶残余判断.md` | BE-001CK-01 `backend.runtime` 第三轮父叶残余判断 | 📋 已创建 |
| 279 | `v4.16.0/279-runtime.mutation.shared_governance单子叶等价基线.md` | BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线 | 📋 已创建 |
| 280 | `v4.16.0/280-runtime.mutation.shared_governance抽离方案.md` | BE-001CL-02 `runtime.mutation.shared_governance` 抽离方案 | 📋 已创建 |
| 281 | `v4.16.0/281-runtime.mutation.shared_governance抽离记录.md` | BE-001CL-03 `runtime.mutation.shared_governance` 实际抽离 | 📋 已创建 |
| 282 | `v4.16.0/282-runtime.mutation.shared_governance单叶closeout.md` | BE-001CL-04 `runtime.mutation.shared_governance` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 283 | `v4.16.0/283-backend.runtime第四轮父叶残余判断.md` | BE-001CM-01 `backend.runtime` 第四轮父叶残余判断，选择 `runtime.query_support` 下一候选 | 📋 已创建 |
| 284 | `v4.16.0/284-runtime.query_support单子叶等价基线.md` | BE-001CN-01 `runtime.query_support` 单子叶等价基线 | 📋 已创建 |
| 285 | `v4.16.0/285-runtime.query_support抽离方案.md` | BE-001CN-02 `runtime.query_support` 抽离方案 | 📋 已创建 |
| 286 | `v4.16.0/286-runtime.query_support抽离记录.md` | BE-001CN-03 `runtime.query_support` 实际抽离 | 📋 已创建 |
| 287 | `v4.16.0/287-runtime.query_support单叶closeout.md` | BE-001CN-04 `runtime.query_support` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 288 | `v4.16.0/288-backend.runtime第五轮父叶残余判断.md` | BE-001CO-01 `backend.runtime` 第五轮父叶残余判断，选择 `runtime.response_support` 下一候选 | 📋 已创建 |
| 289 | `v4.16.0/289-runtime.response_support单子叶等价基线.md` | BE-001CP-01 `runtime.response_support` 单子叶等价基线 | 📋 已创建 |
| 290 | `v4.16.0/290-runtime.response_support抽离方案.md` | BE-001CP-02 `runtime.response_support` 抽离方案 | 📋 已创建 |
| 291 | `v4.16.0/291-runtime.response_support抽离记录.md` | BE-001CP-03 `runtime.response_support` 实际抽离 | 📋 已创建 |
| 292 | `v4.16.0/292-runtime.response_support单叶closeout.md` | BE-001CP-04 `runtime.response_support` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 293 | `v4.16.0/293-backend.runtime第六轮父叶残余判断.md` | BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断，选择 `runtime.run_guard` 下一候选 | 📋 已创建 |
| 294 | `v4.16.0/294-runtime.run_guard单子叶等价基线.md` | BE-001CR-01 `runtime.run_guard` 单子叶等价基线 | 📋 已创建 |
| 295 | `v4.16.0/295-runtime.run_guard抽离方案.md` | BE-001CR-02 `runtime.run_guard` 抽离方案 | 📋 已创建 |
| 296 | `v4.16.0/296-runtime.run_guard抽离记录.md` | BE-001CR-03 `runtime.run_guard` 实际抽离 | 📋 已创建 |
| 297 | `v4.16.0/297-runtime.run_guard单叶closeout.md` | BE-001CR-04 `runtime.run_guard` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 298 | `v4.16.0/298-backend.runtime第七轮父叶残余判断.md` | BE-001CS-01 `backend.runtime` 第七轮父叶残余判断，选择 `runtime.experiment_limit` 下一候选 | 📋 已创建 |
| 299 | `v4.16.0/299-runtime.experiment_limit单子叶等价基线.md` | BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线 | 📋 已创建 |
| 300 | `v4.16.0/300-runtime.experiment_limit抽离方案.md` | BE-001CT-02 `runtime.experiment_limit` test-first 抽离方案 | 📋 已创建 |
| 301 | `v4.16.0/301-runtime.experiment_limit补测记录.md` | BE-001CT-03 `runtime.experiment_limit` endpoint smoke 补测 | 📋 已创建 |
| 302 | `v4.16.0/302-runtime.experiment_limit抽离记录.md` | BE-001CT-04 `runtime.experiment_limit` 实际抽离 | 📋 已创建 |
| 303 | `v4.16.0/303-runtime.experiment_limit单叶closeout.md` | BE-001CT-05 `runtime.experiment_limit` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 304 | `v4.16.0/304-backend.runtime第八轮父叶残余判断.md` | BE-001CU-01 `backend.runtime` 第八轮父叶残余判断，选择 `runtime.parent_include_cleanup` 下一候选 | 📋 已创建 |
| 305 | `v4.16.0/305-runtime.parent_include_cleanup单子叶等价基线.md` | BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线，冻结 drained include cleanup 删除边界 | 📋 已创建 |
| 306 | `v4.16.0/306-runtime.parent_include_cleanup抽离方案.md` | BE-001CV-02 `runtime.parent_include_cleanup` 抽离方案，限定三条 include 与三个 drained 文件 cleanup | 📋 已创建 |
| 307 | `v4.16.0/307-runtime.parent_include_cleanup清理记录.md` | BE-001CV-03 `runtime.parent_include_cleanup` 实际 cleanup，删除三条 drained include 与三个 drained 文件 | 📋 已创建 |
| 308 | `v4.16.0/308-backend.runtime第九轮父叶残余判断.md` | BE-001CW-01 `backend.runtime` 第九轮父叶残余判断，锁定 `runtime.parent_import_bridge` 下一候选 | 📋 已创建 |
| 309 | `v4.16.0/309-runtime.parent_import_bridge单子叶等价基线.md` | BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线，冻结 parent import bridge 与 46 文件依赖面 | 📋 已创建 |
| 310 | `v4.16.0/310-runtime.parent_import_bridge抽离方案.md` | BE-001CX-02 `runtime.parent_import_bridge` 抽离方案，固定 staged explicit import pass 与首批 root support pilot | 📋 已创建 |
| 311 | `v4.16.0/311-runtime.root_support_import_pilot抽离记录.md` | BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离，改写 query/response support parent wildcard import | 📋 已创建 |
| 312 | `v4.16.0/312-runtime.root_support_import_pilot单叶closeout.md` | BE-001CX-04 `runtime.root_support_import_pilot` 单叶 closeout，设置 `stop_split: true` 并转入 root entry import pass | 📋 已创建 |
| 313 | `v4.16.0/313-runtime.root_entry_import_pass单子叶等价基线.md` | BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线，冻结 root entry 候选与 test-only super import 判定 | 📋 已创建 |
| 314 | `v4.16.0/314-runtime.root_entry_import_pass抽离方案.md` | BE-001CY-02 `runtime.root_entry_import_pass` 抽离方案，限定 BE-001CY-03 只处理 event_stream 与 evidence_health | 📋 已创建 |
| 315 | `v4.16.0/315-runtime.root_entry_import_pass抽离记录.md` | BE-001CY-03 `runtime.root_entry_import_pass` 实际抽离，改写 event_stream/evidence_health parent wildcard import | 📋 已创建 |
| 316 | `v4.16.0/316-runtime.root_entry_import_pass单叶closeout.md` | BE-001CY-04 `runtime.root_entry_import_pass` 单叶 closeout，设置 `stop_split: true` 并转入 report_ops import pass | 📋 已创建 |
| 317 | `v4.16.0/317-runtime.report_ops_import_pass单子叶等价基线.md` | BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线，冻结 report_ops facade 与 3 child 的 import 转运风险 | 📋 已创建 |
| 318 | `v4.16.0/318-runtime.report_ops_import_pass抽离方案.md` | BE-001CZ-02 `runtime.report_ops_import_pass` 抽离方案，固定 report_ops four-file pocket 同批处理 | 📋 已创建 |
| 319 | `v4.16.0/319-runtime.report_ops_import_pass抽离记录.md` | BE-001CZ-03 `runtime.report_ops_import_pass` 实际抽离，改写 report_ops four-file pocket parent wildcard import | 📋 已创建 |
| 320 | `v4.16.0/320-runtime.report_ops_import_pass单叶closeout.md` | BE-001CZ-04 `runtime.report_ops_import_pass` 单叶 closeout，设置 `stop_split: true` 并回到 parent import bridge 残余判断 | 📋 已创建 |
| 321 | `v4.16.0/321-runtime.parent_import_bridge父叶残余判断.md` | BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断，确认剩余 38 个依赖文件并选择 `runtime.run_import_pass` | 📋 已创建 |
| 322 | `v4.16.0/322-runtime.run_import_pass单子叶等价基线.md` | BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线，冻结 4 个 run child 的 import 收敛边界 | 📋 已创建 |
| 323 | `v4.16.0/323-runtime.run_import_pass抽离方案.md` | BE-001DB-02 `runtime.run_import_pass` 抽离方案，固定 4 个 run child 同批 explicit import rewrite | 📋 已创建 |
| 324 | `v4.16.0/324-runtime.run_import_pass抽离记录.md` | BE-001DB-03 `runtime.run_import_pass` 实际抽离，改写 4 个 run child parent wildcard import | 📋 已创建 |
| 325 | `v4.16.0/325-runtime.run_import_pass单叶closeout.md` | BE-001DB-04 `runtime.run_import_pass` 单叶 closeout，设置 `stop_split: true` 并回到 parent import bridge 残余判断 | 📋 已创建 |
| 326 | `v4.16.0/326-runtime.parent_import_bridge父叶残余判断.md` | BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断，确认剩余 34 个依赖文件并选择 `runtime.backtest_import_pass` | 📋 已创建 |
| 327 | `v4.16.0/327-runtime.backtest_import_pass单子叶等价基线.md` | BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线，冻结 11 个 backtest 残余文件 | 📋 已创建 |
| 328 | `v4.16.0/328-runtime.backtest_import_pass抽离方案.md` | BE-001DD-02 `runtime.backtest_import_pass` 抽离方案，拒绝 11 文件整批并选择 `runtime.backtest.record_store_import_pass` | 📋 已创建 |
| 329 | `v4.16.0/329-runtime.backtest.record_store_import_pass单子叶等价基线.md` | BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线，冻结 `record_store.rs` import 输入面 | 📋 已创建 |
| 330 | `v4.16.0/330-runtime.backtest.record_store_import_pass抽离方案.md` | BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案，固定 BE-001DE-03 只改 `record_store.rs` import | 📋 已创建 |
| 331 | `v4.16.0/331-runtime.backtest.record_store_import_pass抽离记录.md` | BE-001DE-03 `runtime.backtest.record_store_import_pass` 实际抽离，改写 `record_store.rs` parent wildcard import | 📋 已创建 |
| 332 | `v4.16.0/332-runtime.backtest.record_store_import_pass单叶closeout.md` | BE-001DE-04 `runtime.backtest.record_store_import_pass` 单叶 closeout，设置 `stop_split: true` 并回到父叶残余判断 | 📋 已创建 |
| 333 | `v4.16.0/333-runtime.backtest_import_pass父叶残余判断.md` | BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断，确认剩余 33 个依赖文件并选择 `runtime.backtest.replay_import_pass` | 📋 已创建 |
| 334 | `v4.16.0/334-runtime.backtest.replay_import_pass单子叶等价基线.md` | BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线，冻结 `replay.rs` import 输入面 | 📋 已创建 |
| 335 | `v4.16.0/335-runtime.backtest.replay_import_pass抽离方案.md` | BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案，固定 BE-001DG-03 只改 `replay.rs` import | 📋 已创建 |
| 336 | `v4.16.0/336-runtime.backtest.replay_import_pass抽离记录.md` | BE-001DG-03 `runtime.backtest.replay_import_pass` 实际抽离，改写 `replay.rs` parent wildcard import | 📋 已创建 |
| 337 | `v4.16.0/337-runtime.backtest.replay_import_pass单叶closeout.md` | BE-001DG-04 `runtime.backtest.replay_import_pass` 单叶 closeout，确认 `stop_split: true` | 📋 已创建 |
| 338 | `v4.16.0/338-runtime.backtest_import_pass第二轮父叶残余判断.md` | BE-001DH-01 `runtime.backtest_import_pass` 第二轮父叶残余判断，选择 experiment sweep import pass | 📋 已创建 |
| 339 | `v4.16.0/339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md` | BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线，冻结四文件 pocket | 📋 已创建 |
| 340 | `v4.16.0/340-runtime.backtest.experiment_sweep_import_pass抽离方案.md` | BE-001DI-02 `runtime.backtest.experiment_sweep_import_pass` 抽离方案，固定四文件 import rewrite | 📋 已创建 |
| 341 | `v4.16.0/341-runtime.backtest.experiment_sweep_import_pass抽离记录.md` | BE-001DI-03 `runtime.backtest.experiment_sweep_import_pass` 实际抽离，四文件 parent import 已收敛 | 📋 已创建 |
| 342 | `v4.16.0/342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md` | BE-001DI-04 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout，设置 stop_split true 并回父叶判断 | 📋 已创建 |
| 343 | `v4.16.0/343-runtime.backtest_import_pass第三轮父叶残余判断.md` | BE-001DJ-01 `runtime.backtest_import_pass` 第三轮父叶残余判断，锁定 execution_start import pass | 📋 已创建 |
| 344 | `v4.16.0/344-runtime.backtest.execution_start_import_pass单子叶等价基线.md` | BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线，冻结五文件 pocket | 📋 已创建 |
| 345 | `v4.16.0/345-runtime.backtest.execution_start_import_pass抽离方案.md` | BE-001DK-02 `runtime.backtest.execution_start_import_pass` 抽离方案，固定五文件 import rewrite | 📋 已创建 |
| 346 | `v4.16.0/346-runtime.backtest.execution_start_import_pass抽离记录.md` | BE-001DK-03 `runtime.backtest.execution_start_import_pass` 实际抽离，backtest import residual 清零 | 📋 已创建 |
| 347 | `v4.16.0/347-runtime.backtest.execution_start_import_pass单叶closeout.md` | BE-001DK-04 `runtime.backtest.execution_start_import_pass` 单叶 closeout，设置 stop_split true 并回父叶判断 | 📋 已创建 |
| 348 | `v4.16.0/348-runtime.backtest_import_pass第四轮父叶残余判断.md` | BE-001DL-01 `runtime.backtest_import_pass` 第四轮父叶残余判断，设置 stop_split true 并回上层父叶 | 📋 已创建 |
| 349 | `v4.16.0/349-runtime.parent_import_bridge父叶残余判断.md` | BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断，锁定 mutation import pass | 📋 已创建 |
| 350 | `v4.16.0/350-runtime.mutation_import_pass单子叶等价基线.md` | BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线，冻结 21 个 mutation parent bridge 文件 | 📋 已创建 |
| 351 | `v4.16.0/351-runtime.mutation_import_pass抽离方案.md` | BE-001DN-02 `runtime.mutation_import_pass` 抽离方案，选择 shared_governance import pass | 📋 已创建 |
| 352 | `v4.16.0/352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md` | BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线，冻结 `shared_governance.rs` import 输入面 | 📋 已创建 |
| 353 | `v4.16.0/353-runtime.mutation.shared_governance_import_pass抽离方案.md` | BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案，固定单文件 explicit import rewrite | 📋 已创建 |
| 354 | `v4.16.0/354-runtime.mutation.shared_governance_import_pass抽离记录.md` | BE-001DO-03 `runtime.mutation.shared_governance_import_pass` 抽离记录，改写 `shared_governance.rs` parent wildcard import | 📋 已创建 |
| 355 | `v4.16.0/355-runtime.mutation.shared_governance_import_pass单叶closeout.md` | BE-001DO-04 `runtime.mutation.shared_governance_import_pass` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 356 | `v4.16.0/356-runtime.mutation_import_pass父叶残余判断.md` | BE-001DP-01 `runtime.mutation_import_pass` 父叶残余判断，选择 parameter_mutation import pass | 📋 已创建 |
| 357 | `v4.16.0/357-runtime.mutation.parameter_mutation_import_pass单子叶等价基线.md` | BE-001DQ-01 `runtime.mutation.parameter_mutation_import_pass` 单子叶等价基线，冻结 10 个 parameter mutation residual 文件 | 📋 已创建 |
| 358 | `v4.16.0/358-runtime.mutation.parameter_mutation_import_pass抽离方案.md` | BE-001DQ-02 `runtime.mutation.parameter_mutation_import_pass` 抽离方案，拒绝 10 文件整批 rewrite 并选择 record_query import pass | 📋 已创建 |
| 359 | `v4.16.0/359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md` | BE-001DR-01 `runtime.mutation.parameter_mutation.record_query_import_pass` 单子叶等价基线，冻结 record query 读路径输入面 | 📋 已创建 |
| 360 | `v4.16.0/360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md` | BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案，固定单文件 import rewrite | 📋 已创建 |
| 361 | `v4.16.0/361-runtime.mutation.parameter_mutation.record_query_import_pass抽离记录.md` | BE-001DR-03 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离记录，改写 `record_query.rs` parent wildcard import | 📋 已创建 |
| 362 | `v4.16.0/362-runtime.mutation.parameter_mutation.record_query_import_pass单叶closeout.md` | BE-001DR-04 `runtime.mutation.parameter_mutation.record_query_import_pass` 单叶 closeout，设置 `stop_split: true` 并回父叶判断 | 📋 已创建 |
| 363 | `v4.16.0/363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md` | BE-001DS-01 `runtime.mutation.parameter_mutation_import_pass` 父叶残余判断，选择 proposal_creation import pass | 📋 已创建 |
| 364 | `v4.16.0/364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md` | BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线，冻结 proposal creation 输入面 | 📋 已创建 |
| 365 | `v4.16.0/365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md` | BE-001DT-02 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案，固定单文件 import rewrite | 📋 已创建 |
| 366 | `v4.16.0/366-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离记录.md` | BE-001DT-03 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离记录，改写 `proposal_creation.rs` parent wildcard import | 📋 已创建 |
| 367 | `v4.16.0/367-runtime.mutation.parameter_mutation.proposal_creation_import_pass单叶closeout.md` | BE-001DT-04 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout，设置 `stop_split: true` 并回父叶判断 | 📋 已创建 |
| 368 | `v4.16.0/368-runtime.mutation.parameter_mutation_import_pass第二轮父叶残余判断.md` | BE-001DU-01 `runtime.mutation.parameter_mutation_import_pass` 第二轮父叶残余判断，锁定 transition_lifecycle import pass | 📋 已创建 |
| 369 | `v4.16.0/369-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass单子叶等价基线.md` | BE-001DV-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 单子叶等价基线，冻结 7 文件 lifecycle 输入面 | 📋 已创建 |
| 370 | `v4.16.0/370-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass抽离方案.md` | BE-001DV-02 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 抽离方案，选择 boundary_safety import pass | 📋 已创建 |
| 371 | `v4.16.0/371-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单子叶等价基线.md` | BE-001DW-01 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单子叶等价基线 | 📋 已创建 |
| 372 | `v4.16.0/372-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离方案.md` | BE-001DW-02 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离方案，固定单文件 import rewrite | 📋 已创建 |
| 373 | `v4.16.0/373-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass抽离记录.md` | BE-001DW-03 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 抽离记录，清理 `boundary_safety.rs` parent wildcard import | 📋 已创建 |
| 374 | `v4.16.0/374-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass单叶closeout.md` | BE-001DW-04 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety_import_pass` 单叶 closeout，设置 `stop_split: true` | 📋 已创建 |
| 375 | `v4.16.0/375-runtime.mutation.parameter_mutation.transition_lifecycle_import_pass父叶残余判断.md` | BE-001DX-01 `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断，选择 rollback_record_identity import pass | 📋 已创建 |

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
| v4.16.0 / BE-001DY-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单子叶等价基线已建立，冻结 rollback id import pass 输入面 |
| v4.16.0 / BE-001DY-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001DY-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 抽离记录已建立，rollback_record_identity parent import 已收敛 |
| v4.16.0 / BE-001DY-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001DZ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 父叶残余判断已建立，选择 transition_record_persistence import pass |
| v4.16.0 / BE-001EA-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单子叶等价基线已建立，冻结 lifecycle persistence 输入面 |
| v4.16.0 / BE-001EA-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EA-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 抽离记录已建立，transition_record_persistence parent import 已收敛 |
| v4.16.0 / BE-001EA-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EB-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第三轮父叶残余判断已建立，选择 activation_snapshot_side_effect import pass |
| v4.16.0 / BE-001EC-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单子叶等价基线已建立，冻结 activation snapshot 输入面 |
| v4.16.0 / BE-001EC-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EC-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 抽离记录已建立，activation_snapshot_side_effect parent import 已收敛 |
| v4.16.0 / BE-001EC-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001ED-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第四轮父叶残余判断已建立，选择 activation_flow import pass |
| v4.16.0 / BE-001EE-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单子叶等价基线已建立，冻结 activation flow 输入面 |
| v4.16.0 / BE-001EE-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EE-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 抽离记录已建立，activation_flow parent import 已收敛 |
| v4.16.0 / BE-001EE-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EF-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第五轮父叶残余判断已建立，选择 rollback_flow import pass |
| v4.16.0 / BE-001EG-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单子叶等价基线已建立，冻结 rollback flow 输入面 |
| v4.16.0 / BE-001EG-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EG-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 抽离记录已建立，rollback_flow parent import 已收敛 |
| v4.16.0 / BE-001EG-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EH-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第六轮父叶残余判断已建立，选择 parent_facade import pass |
| v4.16.0 / BE-001EI-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单子叶等价基线已建立，冻结 parent facade 输入面 |
| v4.16.0 / BE-001EI-02 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EI-03 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 抽离记录已建立，parent facade import 已显式化 |
| v4.16.0 / BE-001EI-04 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle.parent_facade_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EJ-01 | 新增: `runtime.mutation.parameter_mutation.transition_lifecycle_import_pass` 第七轮父叶残余判断已建立，设置 stop_split true |
| v4.16.0 / BE-001EK-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 第三轮父叶残余判断已建立，选择 parent_facade import pass |
| v4.16.0 / BE-001EL-01 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单子叶等价基线已建立，冻结 parent facade 输入面 |
| v4.16.0 / BE-001EL-02 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离方案已建立，固定单文件删除 parent wildcard import |
| v4.16.0 / BE-001EL-03 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 抽离记录已建立，parent wildcard import 已收敛为显式 `mutation_event_contract` 输入 |
| v4.16.0 / BE-001EL-04 | 新增: `runtime.mutation.parameter_mutation.parent_facade_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EM-01 | 新增: `runtime.mutation.parameter_mutation_import_pass` 第四轮父叶残余判断已建立，设置 stop_split true |
| v4.16.0 / BE-001EN-01 | 新增: `runtime.mutation_import_pass` 第二轮父叶残余判断已建立，选择 ai_proposal import pass |
| v4.16.0 / BE-001EO-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 单子叶等价基线已建立，冻结 ai proposal 输入面 |
| v4.16.0 / BE-001EO-02 | 新增: `runtime.mutation.ai_proposal_import_pass` 抽离方案已建立，选择 record_query import pass |
| v4.16.0 / BE-001EP-01 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001EP-02 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EP-03 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 抽离记录已建立，record_query import 已显式化 |
| v4.16.0 / BE-001EP-04 | 新增: `runtime.mutation.ai_proposal.record_query_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EQ-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第三轮父叶残余判断已建立，选择 source_governance_identity import pass |
| v4.16.0 / BE-001ER-01 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001ER-02 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001ER-03 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离记录已建立，source governance import 已显式化 |
| v4.16.0 / BE-001ER-04 | 新增: `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001ES-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第四轮父叶残余判断已建立，选择 static_check import pass |
| v4.16.0 / BE-001ET-01 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001ET-02 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001ET-03 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 抽离记录已建立，static_check import 已显式化 |
| v4.16.0 / BE-001ET-04 | 新增: `runtime.mutation.ai_proposal.static_check_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EU-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第五轮父叶残余判断已建立，选择 event_lifecycle import pass |
| v4.16.0 / BE-001EV-01 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001EV-02 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EV-03 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离记录已建立，event_lifecycle import 已显式化 |
| v4.16.0 / BE-001EV-04 | 新增: `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EW-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第六轮父叶残余判断已建立，选择 approval_persistence import pass |
| v4.16.0 / BE-001EX-01 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001EX-02 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EX-03 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离记录已建立，approval_persistence import 已显式化 |
| v4.16.0 / BE-001EX-04 | 新增: `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001EY-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第七轮父叶残余判断已建立，选择 status_transition import pass |
| v4.16.0 / BE-001EZ-01 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001EZ-02 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001EZ-03 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 抽离记录已建立，status_transition import 已显式化 |
| v4.16.0 / BE-001EZ-04 | 新增: `runtime.mutation.ai_proposal.status_transition_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FA-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第八轮父叶残余判断已建立，选择 sandbox_trigger import pass |
| v4.16.0 / BE-001FB-01 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单子叶等价基线已建立 |
| v4.16.0 / BE-001FB-02 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001FB-03 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 抽离记录已建立，sandbox_trigger import 已显式化 |
| v4.16.0 / BE-001FB-04 | 新增: `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FC-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第九轮父叶残余判断已建立，选择 approval_review import pass |
| v4.16.0 / BE-001FD-01 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 单子叶等价基线已建立，下一步进入 BE-001FD-02 抽离方案 |
| v4.16.0 / BE-001FD-02 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离方案已建立，下一步进入 BE-001FD-03 实际抽离记录 |
| v4.16.0 / BE-001FD-03 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 抽离记录已建立，`approval_review.rs` import 已显式化 |
| v4.16.0 / BE-001FD-04 | 新增: `runtime.mutation.ai_proposal.approval_review_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FE-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十轮父叶残余判断已建立，选择 proposal_creation import pass |
| v4.16.0 / BE-001FF-01 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单子叶等价基线已建立，冻结 create handler 输入面 |
| v4.16.0 / BE-001FF-02 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001FF-03 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 抽离记录已建立，`proposal_creation.rs` import 已显式化 |
| v4.16.0 / BE-001FF-04 | 新增: `runtime.mutation.ai_proposal.proposal_creation_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FG-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十一轮父叶残余判断已建立，选择 parent facade import pass |
| v4.16.0 / BE-001FH-01 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 单子叶等价基线已建立，冻结 parent facade 输入面 |
| v4.16.0 / BE-001FH-02 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离方案已建立，固定单文件 import rewrite |
| v4.16.0 / BE-001FH-03 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 抽离记录已建立，parent facade import 已显式化 |
| v4.16.0 / BE-001FH-04 | 新增: `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FI-01 | 新增: `runtime.mutation.ai_proposal_import_pass` 第十二轮父叶残余判断已建立，设置 stop_split true |
| v4.16.0 / BE-001FJ-01 | 新增: `runtime.mutation_import_pass` 第三轮父叶残余判断已建立，设置 stop_split true |
| v4.16.0 / BE-001FK-01 | 新增: `runtime.parent_import_bridge` 第四轮父叶残余判断已建立，选择 root parent facade import pass |
| v4.16.0 / BE-001FL-01 | 新增: `runtime.root_parent_facade_import_pass` 单子叶等价基线已建立，冻结 `src/runtime/mod.rs` root facade 输入面 |
| v4.16.0 / BE-001FL-02 | 新增: `runtime.root_parent_facade_import_pass` 抽离方案已建立，固定单文件 root import cleanup |
| v4.16.0 / BE-001FL-03 | 新增: `runtime.root_parent_facade_import_pass` 抽离记录已建立，`src/runtime/mod.rs` root import residual 已清除 |
| v4.16.0 / BE-001FL-04 | 新增: `runtime.root_parent_facade_import_pass` 单叶 closeout 已建立，设置 stop_split true |
| v4.16.0 / BE-001FM-01 | 新增: `runtime.parent_import_bridge` 第五轮父叶残余判断已建立，生产级 parent bridge 设置 stop_split true |
| v4.16.0 / BE-001FN-01 | 新增: `backend.runtime` 第十轮父叶残余判断已建立，设置 stop_split true 并回到 backend 父叶 |
| v4.16.0 / BE-001FO-01 | 新增: `backend` 父叶残余判断已建立，选择 backend.graph_compile 作为下一顶层叶子 |
| v4.16.0 / BE-001FP-01 | 新增: `backend.graph_compile` 父叶残余判断已建立，选择 quantscript_graph 作为首个子叶 |
| v4.16.0 / BE-001FQ-01 | 新增: `backend.graph_compile.quantscript_graph` 单子叶等价基线已建立，冻结 route 与 shared helper 输入面 |
| v4.16.0 / BE-001FQ-02 | 新增: `backend.graph_compile.quantscript_graph` 抽离方案已建立，固定 planned move、import rewrite 与 root parent re-export surface |
| v4.16.0 / BE-001FQ-03 | 新增: `backend.graph_compile.quantscript_graph` 实际抽离完成，真实实现迁入 backend child 并删除旧 root owner |
| v4.16.0 / BE-001FQ-04 | 新增: `backend.graph_compile.quantscript_graph` 单叶 closeout 完成，判定继续细拆并选择 `graph_to_qs_generation` |
| v4.16.0 / BE-001FR-01 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线已建立，冻结 graph-to-QS generator 输入面 |
| v4.16.0 / BE-001FR-02 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 抽离方案已建立，固定 planned child 与四函数迁移清单 |
| v4.16.0 / BE-001FR-03 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 实际抽离完成，child file 承接四个 generator helper |
| v4.16.0 / BE-001FR-04 | 新增: `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单叶 closeout 已完成，设置 stop_split true |
| v4.16.0 / BE-001FS-01 | 新增: `backend.graph_compile.quantscript_graph` 父叶残余判断已完成，选择 `formal_module_conversion` |
| v4.16.0 / BE-001FT-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 单子叶等价基线已建立，冻结 formal conversion 输入输出与分支语义 |
| v4.16.0 / BE-001FT-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 抽离方案已建立，固定 planned child 与单函数迁移清单 |
| v4.16.0 / BE-001FT-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 实际抽离完成，`convert_graph_json_to_script_module` 已迁入 child |
| v4.16.0 / BE-001FT-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 单叶 closeout 完成，判定继续细拆 |
| v4.16.0 / BE-001FU-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` 父叶残余判断已完成，选择 `intent_lowering` |
| v4.16.0 / BE-001FV-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单子叶等价基线已建立，冻结七个 built-in intent 分支 |
| v4.16.0 / BE-001FV-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 抽离方案已建立，固定 planned child 与 helper signature |
| v4.16.0 / BE-001FV-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 实际抽离完成，child file 承接 intent block |
| v4.16.0 / BE-001FV-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 单叶 closeout 完成，判定继续细拆 |
| v4.16.0 / BE-001FW-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，选择 `spread_observer_lowering` |
| v4.16.0 / BE-001FX-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单子叶等价基线已建立 |
| v4.16.0 / BE-001FX-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 抽离方案已建立 |
| v4.16.0 / BE-001FX-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 实际抽离完成 |
| v4.16.0 / BE-001FX-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单叶 closeout 完成 |
| v4.16.0 / BE-001FY-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，选择 `macd_lowering` |
| v4.16.0 / BE-001FZ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单子叶等价基线已建立 |
| v4.16.0 / BE-001FZ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 抽离方案已建立 |
| v4.16.0 / BE-001FZ-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 实际抽离完成 |
| v4.16.0 / BE-001FZ-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单叶 closeout 完成 |
| v4.16.0 / BE-001GA-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，选择 `double_ma_lowering` |
| v4.16.0 / BE-001GB-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单子叶等价基线已建立 |
| v4.16.0 / BE-001GB-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 抽离方案已建立 |
| v4.16.0 / BE-001GB-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 实际抽离完成 |
| v4.16.0 / BE-001GB-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` 单叶 closeout 完成 |
| v4.16.0 / BE-001GC-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` 父叶残余判断已完成，选择 `rsi_lowering` |
| v4.16.0 / BE-001GD-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单子叶等价基线已建立 |
| v4.16.0 / BE-001GD-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 抽离方案已建立 |
| v4.16.0 / BE-001GD-03 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 实际抽离完成 |
| v4.16.0 / BE-001GD-04 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` 单叶 closeout 完成 |
| v4.16.0 / GOV-LEAF-SPLIT-GATE | 新增: 递归叶子细分判定硬规则已固化，后续单叶 closeout / 父叶残余判断必须触发 `leaf_split_decision_gate` |
| v4.16.0 / BE-001GE-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects ma_deviation_lowering |
| v4.16.0 / BE-001GF-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GF-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` ma_deviation_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GG-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects momentum_lowering |
| v4.16.0 / BE-001GH-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GH-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` momentum_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GI-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects zscore_lowering |
| v4.16.0 / BE-001GJ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering baseline and extraction plan frozen |
| v4.16.0 / BE-001GJ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.zscore_lowering` zscore_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GK-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects shared_intent_context |
| v4.16.0 / BE-001GL-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context baseline and extraction plan frozen |
| v4.16.0 / BE-001GL-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` shared_intent_context actual extraction and closeout complete |
| v4.16.0 / BE-001GM-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent residual judgment selects unsupported_intent_failure |
| v4.16.0 / BE-001GN-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure equivalence baseline and extraction plan |
| v4.16.0 / BE-001GN-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` unsupported_intent_failure actual extraction and closeout complete |
| v4.16.0 / BE-001GO-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` intent_lowering parent closeout sets stop_split true |
| v4.16.0 / BE-001GP-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects data_source_lowering |
| v4.16.0 / BE-001GQ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering equivalence baseline and extraction plan |
| v4.16.0 / BE-001GQ-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` data_source_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GR-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects profile_lowering |
| v4.16.0 / BE-001GS-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering equivalence baseline and extraction plan |
| v4.16.0 / BE-001GS-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` profile_lowering actual extraction and closeout complete |
| v4.16.0 / BE-001GT-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects input_shape_validation |
| v4.16.0 / BE-001GU-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001GU-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` input_shape_validation actual extraction and closeout complete |
| v4.16.0 / BE-001GV-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects terminal_parse |
| v4.16.0 / BE-001GW-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse equivalence baseline and extraction plan |
| v4.16.0 / BE-001GW-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse` terminal_parse actual extraction and closeout complete |
| v4.16.0 / BE-001GX-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent residual judgment selects unsupported_node_logging |
| v4.16.0 / BE-001GY-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging equivalence baseline and extraction plan |
| v4.16.0 / BE-001GY-02 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging` unsupported_node_logging actual extraction and closeout complete |
| v4.16.0 / BE-001GZ-01 | 新增: `backend.graph_compile.quantscript_graph.formal_module_conversion` formal_module_conversion parent closeout sets stop_split true |
| v4.16.0 / BE-001HA-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects strategy_graph_parser |
| v4.16.0 / BE-001HB-01 | 新增: `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser equivalence baseline and extraction plan |
| v4.16.0 / BE-001HB-02 | 新增: `backend.graph_compile.quantscript_graph.strategy_graph_parser` strategy_graph_parser actual extraction and closeout complete |
| v4.16.0 / BE-001HC-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects artifact_target_projection |
| v4.16.0 / BE-001HD-01 | 新增: `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001HD-02 | 新增: `backend.graph_compile.quantscript_graph.artifact_target_projection` artifact_target_projection actual extraction and closeout complete |
| v4.16.0 / BE-001HE-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent residual judgment selects route_surface |
| v4.16.0 / BE-001HF-01 | 新增: `backend.graph_compile.quantscript_graph.route_surface` route_surface equivalence baseline and extraction plan |
| v4.16.0 / BE-001HF-02 | 新增: `backend.graph_compile.quantscript_graph.route_surface` route_surface actual extraction and closeout complete |
| v4.16.0 / BE-001HG-01 | 新增: `backend.graph_compile.quantscript_graph` quantscript_graph parent closeout sets stop_split true |
| v4.16.0 / BE-001HH-01 | 新增: `backend.graph_compile` backend.graph_compile parent residual judgment selects compile |
| v4.16.0 / BE-001HI-01 | 新增: `backend.graph_compile.compile` backend.graph_compile.compile equivalence baseline and extraction plan |
| v4.16.0 / BE-001HI-02 | 新增: `backend.graph_compile.compile` backend.graph_compile.compile actual extraction and closeout complete |
| v4.16.0 / BE-001HJ-01 | 新增: `backend.graph_compile` backend.graph_compile parent residual judgment selects graph |
| v4.16.0 / BE-001HK-01 | 新增: `backend.graph_compile.graph` backend.graph_compile.graph equivalence baseline and extraction plan |
| v4.16.0 / BE-001HK-02 | 新增: `backend.graph_compile.graph` backend.graph_compile.graph actual extraction and closeout complete |
| v4.16.0 / BE-001HL-01 | 新增: `backend.graph_compile` backend.graph_compile parent closeout sets stop_split true |
| v4.16.0 / BE-001HM-01 | 新增: `backend` backend parent residual judgment selects capability |
| v4.16.0 / BE-001HN-01 | 新增: `backend.capability` backend.capability equivalence baseline and extraction plan |
| v4.16.0 / BE-001HN-02 | 新增: `backend.capability` backend.capability actual extraction and closeout complete |
| v4.16.0 / BE-001HO-01 | 新增: `backend` backend parent residual judgment selects strategy_config |
| v4.16.0 / BE-001HP-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects artifact |
| v4.16.0 / BE-001HQ-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact equivalence baseline and extraction plan |
| v4.16.0 / BE-001HQ-02 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact route owner extraction complete |
| v4.16.0 / BE-001HR-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects schema_model |
| v4.16.0 / BE-001HS-01 | 新增: `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model equivalence baseline and extraction plan |
| v4.16.0 / BE-001HS-02 | 新增: `backend.strategy_config.artifact.schema_model` backend.strategy_config.artifact.schema_model actual extraction complete |
| v4.16.0 / BE-001HT-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects domain_projection |
| v4.16.0 / BE-001HU-01 | 新增: `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001HU-02 | 新增: `backend.strategy_config.artifact.domain_projection` backend.strategy_config.artifact.domain_projection actual extraction complete |
| v4.16.0 / BE-001HV-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent residual judgment selects builder_core |
| v4.16.0 / BE-001HW-01 | 新增: `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core equivalence baseline and extraction plan |
| v4.16.0 / BE-001HW-02 | 新增: `backend.strategy_config.artifact.builder_core` backend.strategy_config.artifact.builder_core actual extraction complete |
| v4.16.0 / BE-001HX-01 | 新增: `backend.strategy_config.artifact` backend.strategy_config.artifact parent closeout sets stop_split true |
| v4.16.0 / BE-001HY-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects preflight |
| v4.16.0 / BE-001HZ-01 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight equivalence baseline and extraction plan |
| v4.16.0 / BE-001HZ-02 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight actual extraction complete |
| v4.16.0 / BE-001IA-01 | 新增: `backend.strategy_config.preflight` backend.strategy_config.preflight single leaf closeout sets stop_split true |
| v4.16.0 / BE-001IB-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects diff |
| v4.16.0 / BE-001IC-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001IC-02 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff actual extraction complete |
| v4.16.0 / BE-001ID-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001IE-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects artifact_diff |
| v4.16.0 / BE-001IF-01 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001IF-02 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff actual extraction complete |
| v4.16.0 / BE-001IG-01 | 新增: `backend.strategy_config.diff.artifact_diff` backend.strategy_config.diff.artifact_diff single leaf closeout sets stop_split true |
| v4.16.0 / BE-001IH-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent residual judgment selects evidence_diff |
| v4.16.0 / BE-001II-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff equivalence baseline and extraction plan |
| v4.16.0 / BE-001II-02 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff actual extraction complete |
| v4.16.0 / BE-001IJ-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001IK-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory |
| v4.16.0 / BE-001IL-01 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory equivalence baseline and extraction plan |
| v4.16.0 / BE-001IL-02 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory actual extraction complete |
| v4.16.0 / BE-001IM-01 | 新增: `backend.strategy_config.diff.evidence_diff.machine_trajectory` backend.strategy_config.diff.evidence_diff.machine_trajectory single leaf closeout stops further split |
| v4.16.0 / BE-001IN-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane |
| v4.16.0 / BE-001IO-01 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane equivalence baseline and extraction plan |
| v4.16.0 / BE-001IO-02 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane actual extraction complete |
| v4.16.0 / BE-001IP-01 | 新增: `backend.strategy_config.diff.evidence_diff.risk_plane` backend.strategy_config.diff.evidence_diff.risk_plane single leaf closeout stops further split |
| v4.16.0 / BE-001IQ-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability |
| v4.16.0 / BE-001IR-01 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability equivalence baseline and extraction plan |
| v4.16.0 / BE-001IR-02 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability actual extraction complete |
| v4.16.0 / BE-001IS-01 | 新增: `backend.strategy_config.diff.evidence_diff.execution_capability` backend.strategy_config.diff.evidence_diff.execution_capability single leaf closeout stops further split |
| v4.16.0 / BE-001IT-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics |
| v4.16.0 / BE-001IU-01 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics equivalence baseline and extraction plan |
| v4.16.0 / BE-001IU-02 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics actual extraction complete |
| v4.16.0 / BE-001IV-01 | 新增: `backend.strategy_config.diff.evidence_diff.metrics` backend.strategy_config.diff.evidence_diff.metrics single leaf closeout stops further split |
| v4.16.0 / BE-001IW-01 | 新增: `backend.strategy_config.diff.evidence_diff` backend.strategy_config.diff.evidence_diff parent closeout retains report assembly and shared helpers |
| v4.16.0 / BE-001IX-01 | 新增: `backend.strategy_config.diff` backend.strategy_config.diff parent closeout keeps facade and child mediation |
| v4.16.0 / BE-001IY-01 | 新增: `backend.strategy_config` backend.strategy_config parent residual judgment selects ai_proposal_binding |
| v4.16.0 / BE-001IZ-01 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-op route pocket baseline and plan |
| v4.16.0 / BE-001IZ-02 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding no-code extraction closeout complete |
| v4.16.0 / BE-001JA-01 | 新增: `backend.strategy_config.ai_proposal_binding` backend.strategy_config.ai_proposal_binding single leaf closeout stops further split |
| v4.16.0 / BE-001JB-01 | 新增: `backend.strategy_config` backend.strategy_config parent closeout keeps route aggregation facade |
| v4.16.0 / BE-001JC-01 | 新增: `backend` backend parent residual judgment selects storage_security safety baseline |
| v4.16.0 / BE-001JD-01 | 新增: `backend.storage_security` backend.storage_security safety equivalence baseline and extraction plan |
| v4.16.0 / BE-001JD-02 | 新增: `backend.storage_security` backend.storage_security facade extraction closeout keeps sensitive semantics paused |
| v4.16.0 / BE-001JE-01 | 新增: `backend.storage_security` backend.storage_security single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JF-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_api |
| v4.16.0 / BE-001JG-01 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api route facade baseline and plan |
| v4.16.0 / BE-001JG-02 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api facade extraction closeout complete |
| v4.16.0 / BE-001JH-01 | 新增: `backend.storage_security.credential_api` backend.storage_security.credential_api single leaf closeout stops further facade split |
| v4.16.0 / BE-001JI-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault |
| v4.16.0 / BE-001JJ-01 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault re-export facade baseline and plan |
| v4.16.0 / BE-001JJ-02 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault facade extraction closeout complete |
| v4.16.0 / BE-001JK-01 | 新增: `backend.storage_security.credential_vault` backend.storage_security.credential_vault single leaf closeout stops further facade split |
| v4.16.0 / BE-001JL-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_vault_implementation |
| v4.16.0 / BE-001JM-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation safety baseline and extraction plan |
| v4.16.0 / BE-001JM-02 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation actual extraction complete |
| v4.16.0 / BE-001JN-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JO-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects machine_key_management |
| v4.16.0 / BE-001JP-01 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan |
| v4.16.0 / BE-001JP-02 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management actual extraction complete |
| v4.16.0 / BE-001JP-03 | 新增: `backend.storage_security.credential_vault_implementation.machine_key_management` backend.storage_security.credential_vault_implementation.machine_key_management single leaf closeout stops further split |
| v4.16.0 / BE-001JQ-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects crypto_codec |
| v4.16.0 / BE-001JR-01 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec equivalence baseline and extraction plan |
| v4.16.0 / BE-001JR-02 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec actual extraction complete |
| v4.16.0 / BE-001JR-03 | 新增: `backend.storage_security.credential_vault_implementation.crypto_codec` backend.storage_security.credential_vault_implementation.crypto_codec single leaf closeout stops further split |
| v4.16.0 / BE-001JS-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects vault_persistence_restore |
| v4.16.0 / BE-001JT-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan |
| v4.16.0 / BE-001JT-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore actual extraction complete |
| v4.16.0 / BE-001JT-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001JU-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects load_restore_entry |
| v4.16.0 / BE-001JV-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan |
| v4.16.0 / BE-001JV-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry actual extraction complete |
| v4.16.0 / BE-001JV-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry single leaf closeout stops further split |
| v4.16.0 / BE-001JW-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent residual judgment selects atomic_save_commit |
| v4.16.0 / BE-001JX-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001JX-02 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit actual extraction complete |
| v4.16.0 / BE-001JX-03 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit single leaf closeout stops further split |
| v4.16.0 / BE-001JY-01 | 新增: `backend.storage_security.credential_vault_implementation.vault_persistence_restore` backend.storage_security.credential_vault_implementation.vault_persistence_restore parent closeout stops persistence split |
| v4.16.0 / BE-001JZ-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects service_crud |
| v4.16.0 / BE-001KA-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud equivalence baseline and extraction plan |
| v4.16.0 / BE-001KA-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud actual extraction complete |
| v4.16.0 / BE-001KA-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud single leaf closeout keeps stop_split false |
| v4.16.0 / BE-001KB-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_mutation_commit |
| v4.16.0 / BE-001KC-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001KC-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit actual extraction complete |
| v4.16.0 / BE-001KC-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit` backend.storage_security.credential_vault_implementation.service_crud.service_mutation_commit single leaf closeout stops further split |
| v4.16.0 / BE-001KD-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent residual judgment selects service_read_projection |
| v4.16.0 / BE-001KE-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001KE-02 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection actual extraction complete |
| v4.16.0 / BE-001KE-03 | 新增: `backend.storage_security.credential_vault_implementation.service_crud.service_read_projection` backend.storage_security.credential_vault_implementation.service_crud.service_read_projection single leaf closeout stops further split |
| v4.16.0 / BE-001KF-01 | 新增: `backend.storage_security.credential_vault_implementation.service_crud` backend.storage_security.credential_vault_implementation.service_crud parent closeout stops CRUD split |
| v4.16.0 / BE-001KG-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects secret_pattern_extraction |
| v4.16.0 / BE-001KH-01 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction equivalence baseline and extraction plan |
| v4.16.0 / BE-001KH-02 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction actual extraction complete |
| v4.16.0 / BE-001KH-03 | 新增: `backend.storage_security.credential_vault_implementation.secret_pattern_extraction` backend.storage_security.credential_vault_implementation.secret_pattern_extraction single leaf closeout stops further split |
| v4.16.0 / BE-001KI-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects type_surface |
| v4.16.0 / BE-001KJ-01 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface equivalence baseline and extraction plan |
| v4.16.0 / BE-001KJ-02 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface actual extraction complete |
| v4.16.0 / BE-001KJ-03 | 新增: `backend.storage_security.credential_vault_implementation.type_surface` backend.storage_security.credential_vault_implementation.type_surface single leaf closeout stops further split |
| v4.16.0 / BE-001KK-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment selects implementation_test_harness |
| v4.16.0 / BE-001KL-01 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness equivalence baseline and extraction plan |
| v4.16.0 / BE-001KL-02 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness actual extraction complete |
| v4.16.0 / BE-001KL-03 | 新增: `backend.storage_security.credential_vault_implementation.implementation_test_harness` backend.storage_security.credential_vault_implementation.implementation_test_harness single leaf closeout stops further split |
| v4.16.0 / BE-001KM-01 | 新增: `backend.storage_security.credential_vault_implementation` backend.storage_security.credential_vault_implementation parent residual judgment closes implementation parent |
| v4.16.0 / BE-001KN-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment selects credential_api_handler_implementation |
| v4.16.0 / BE-001KO-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan |
| v4.16.0 / BE-001KO-02 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation actual extraction complete |
| v4.16.0 / BE-001KO-03 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation single leaf closeout continues split |
| v4.16.0 / BE-001KP-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection |
| v4.16.0 / BE-001KQ-01 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001KQ-02 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete |
| v4.16.0 / BE-001KQ-03 | 新增: `backend.storage_security.credential_api_handler_implementation.list_projection` backend.storage_security.credential_api_handler_implementation.list_projection single leaf closeout stops further split |
| v4.16.0 / BE-001KR-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope |
| v4.16.0 / BE-001KS-01 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan |
| v4.16.0 / BE-001KS-02 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete |
| v4.16.0 / BE-001KS-03 | 新增: `backend.storage_security.credential_api_handler_implementation.key_scope` backend.storage_security.credential_api_handler_implementation.key_scope single leaf closeout stops further split |
| v4.16.0 / BE-001KT-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation |
| v4.16.0 / BE-001KU-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan |
| v4.16.0 / BE-001KU-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete |
| v4.16.0 / BE-001KU-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation single leaf closeout continues split |
| v4.16.0 / BE-001KV-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation |
| v4.16.0 / BE-001KW-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001KW-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete |
| v4.16.0 / BE-001KW-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single leaf closeout stops further split |
| v4.16.0 / BE-001KX-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects storage_commit |
| v4.16.0 / BE-001KY-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001KY-02 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete |
| v4.16.0 / BE-001KY-03 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single leaf closeout stops further split |
| v4.16.0 / BE-001KZ-01 | 新增: `backend.storage_security.credential_api_handler_implementation.set_mutation` backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment closes parent |
| v4.16.0 / BE-001LA-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation |
| v4.16.0 / BE-001LB-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan |
| v4.16.0 / BE-001LB-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete |
| v4.16.0 / BE-001LB-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation single leaf closeout continues split |
| v4.16.0 / BE-001LC-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation |
| v4.16.0 / BE-001LD-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan |
| v4.16.0 / BE-001LD-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete |
| v4.16.0 / BE-001LD-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single leaf closeout stops further split |
| v4.16.0 / BE-001LE-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects delete_commit |
| v4.16.0 / BE-001LF-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001LF-02 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete |
| v4.16.0 / BE-001LF-03 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single leaf closeout stops further split |
| v4.16.0 / BE-001LG-01 | 新增: `backend.storage_security.credential_api_handler_implementation.delete_mutation` backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment closes parent |
| v4.16.0 / BE-001LH-01 | 新增: `backend.storage_security.credential_api_handler_implementation` backend.storage_security.credential_api_handler_implementation parent residual judgment closes parent |
| v4.16.0 / BE-001LI-01 | 新增: `backend.storage_security` backend.storage_security parent residual judgment closes parent |
| v4.16.0 / BE-001LJ-01 | 新增: `backend` backend parent residual judgment selects ops_governance |
| v4.16.0 / BE-001LK-01 | 新增: `backend.ops_governance` backend.ops_governance equivalence baseline and extraction plan |
| v4.16.0 / BE-001LK-02 | 新增: `backend.ops_governance` backend.ops_governance facade extraction closeout |
| v4.16.0 / BE-001LK-03 | 新增: `backend.ops_governance` backend.ops_governance single leaf closeout continues split |
| v4.16.0 / BE-001LL-01 | 新增: `backend.ops_governance` backend.ops_governance parent residual judgment selects hotswap |
| v4.16.0 / BE-001LM-01 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap equivalence baseline and extraction plan |
| v4.16.0 / BE-001LM-02 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap actual extraction complete |
| v4.16.0 / BE-001LM-03 | 新增: `backend.ops_governance.hotswap` backend.ops_governance.hotswap single leaf closeout stops further split |
| v4.16.0 / BE-001LN-01 | 新增: `backend.ops_governance` backend.ops_governance parent residual judgment selects sandbox |
| v4.16.0 / BE-001LO-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox equivalence baseline and extraction plan |
| v4.16.0 / BE-001LO-02 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox actual extraction complete |
| v4.16.0 / BE-001LO-03 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox single leaf closeout continues split |
| v4.16.0 / BE-001LP-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects report_api |
| v4.16.0 / BE-001LQ-01 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan |
| v4.16.0 / BE-001LQ-02 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api actual extraction complete |
| v4.16.0 / BE-001LQ-03 | 新增: `backend.ops_governance.sandbox.report_api` backend.ops_governance.sandbox.report_api single leaf closeout stops further split |
| v4.16.0 / BE-001LR-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects verification_run |
| v4.16.0 / BE-001LS-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan |
| v4.16.0 / BE-001LS-02 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run actual extraction complete |
| v4.16.0 / BE-001LS-03 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run single leaf closeout continues split |
| v4.16.0 / BE-001LT-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit |
| v4.16.0 / BE-001LU-01 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan |
| v4.16.0 / BE-001LU-02 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete |
| v4.16.0 / BE-001LU-03 | 新增: `backend.ops_governance.sandbox.verification_run.report_commit` backend.ops_governance.sandbox.verification_run.report_commit single leaf closeout stops further split |
| v4.16.0 / BE-001LV-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate |
| v4.16.0 / BE-001LW-01 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan |
| v4.16.0 / BE-001LW-02 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete |
| v4.16.0 / BE-001LW-03 | 新增: `backend.ops_governance.sandbox.verification_run.proposal_gate` backend.ops_governance.sandbox.verification_run.proposal_gate single leaf closeout stops further split |
| v4.16.0 / BE-001LX-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window |
| v4.16.0 / BE-001LY-01 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan |
| v4.16.0 / BE-001LY-02 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete |
| v4.16.0 / BE-001LY-03 | 新增: `backend.ops_governance.sandbox.verification_run.replay_window` backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout stops further split |
| v4.16.0 / BE-001LZ-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly |
| v4.16.0 / BE-001MA-01 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan |
| v4.16.0 / BE-001MA-02 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete |
| v4.16.0 / BE-001MA-03 | 新增: `backend.ops_governance.sandbox.verification_run.report_assembly` backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout stops further split |
| v4.16.0 / BE-001MB-01 | 新增: `backend.ops_governance.sandbox.verification_run` backend.ops_governance.sandbox.verification_run parent residual judgment closes parent |
| v4.16.0 / BE-001MC-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation |
| v4.16.0 / BE-001MD-01 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan |
| v4.16.0 / BE-001MD-02 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation actual extraction complete |
| v4.16.0 / BE-001MD-03 | 新增: `backend.ops_governance.sandbox.metrics_evaluation` backend.ops_governance.sandbox.metrics_evaluation single leaf closeout stops further split |
| v4.16.0 / BE-001ME-01 | 新增: `backend.ops_governance.sandbox` backend.ops_governance.sandbox parent residual judgment selects comparison_metrics |
| v4.16.0 / BE-001MF-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan |
| v4.16.0 / BE-001MF-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics actual extraction complete |
| v4.16.0 / BE-001MF-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split |
| v4.16.0 / BE-001MG-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape |
| v4.16.0 / BE-001MH-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan |
| v4.16.0 / BE-001MH-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete |
| v4.16.0 / BE-001MH-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout stops further split |
| v4.16.0 / BE-001MI-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics` backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection |
| v4.16.0 / BE-001MJ-01 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan |
| v4.16.0 / BE-001MJ-02 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete |
| v4.16.0 / BE-001MJ-03 | 新增: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout stops further split |
| v4.16.0 / BE-001MK-01 | Added: `backend.ops_governance.sandbox.comparison_metrics` parent residual judgment closes parent |
| v4.16.0 / BE-001ML-01 | Added: `backend.ops_governance.sandbox` parent residual judgment selects proposal_loader |
| v4.16.0 / BE-001MM-01 | Added: `backend.ops_governance.sandbox.proposal_loader` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MM-02 | Added: `backend.ops_governance.sandbox.proposal_loader` actual extraction complete |
| v4.16.0 / BE-001MM-03 | Added: `backend.ops_governance.sandbox.proposal_loader` single leaf closeout stops further split |
| v4.16.0 / BE-001MN-01 | Added: `backend.ops_governance.sandbox` parent residual judgment selects report_disk_loader |
| v4.16.0 / BE-001MO-01 | Added: `backend.ops_governance.sandbox.report_disk_loader` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MO-02 | Added: `backend.ops_governance.sandbox.report_disk_loader` actual extraction complete |
| v4.16.0 / BE-001MO-03 | Added: `backend.ops_governance.sandbox.report_disk_loader` single leaf closeout stops further split |
| v4.16.0 / BE-001MP-01 | Added: `backend.ops_governance.sandbox` parent residual judgment closes parent |
| v4.16.0 / BE-001MQ-01 | Added: `backend.ops_governance` parent residual judgment selects alerts |
| v4.16.0 / BE-001MR-01 | Added: `backend.ops_governance.alerts` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MR-02 | Added: `backend.ops_governance.alerts` actual extraction complete |
| v4.16.0 / BE-001MR-03 | Added: `backend.ops_governance.alerts` single leaf closeout continues split |
| v4.16.0 / BE-001MS-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects rule_catalog |
| v4.16.0 / BE-001MT-01 | Added: `backend.ops_governance.alerts.rule_catalog` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MT-02 | Added: `backend.ops_governance.alerts.rule_catalog` actual extraction complete |
| v4.16.0 / BE-001MT-03 | Added: `backend.ops_governance.alerts.rule_catalog` single leaf closeout stops further split |
| v4.16.0 / BE-001MU-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects acknowledge_flow |
| v4.16.0 / BE-001MV-01 | Added: `backend.ops_governance.alerts.acknowledge_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MV-02 | Added: `backend.ops_governance.alerts.acknowledge_flow` actual extraction complete |
| v4.16.0 / BE-001MV-03 | Added: `backend.ops_governance.alerts.acknowledge_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001MW-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects trigger_engine |
| v4.16.0 / BE-001MX-01 | Added: `backend.ops_governance.alerts.trigger_engine` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MX-02 | Added: `backend.ops_governance.alerts.trigger_engine` actual extraction complete |
| v4.16.0 / BE-001MX-03 | Added: `backend.ops_governance.alerts.trigger_engine` single leaf closeout stops further split |
| v4.16.0 / BE-001MY-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects predicate_checks |
| v4.16.0 / BE-001MZ-01 | Added: `backend.ops_governance.alerts.predicate_checks` equivalence baseline and extraction plan |
| v4.16.0 / BE-001MZ-02 | Added: `backend.ops_governance.alerts.predicate_checks` actual extraction complete |
| v4.16.0 / BE-001MZ-03 | Added: `backend.ops_governance.alerts.predicate_checks` single leaf closeout stops further split |
| v4.16.0 / BE-001NA-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects persistence |
| v4.16.0 / BE-001NA-02 | Added: `backend.ops_governance.alerts.persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NA-03 | Added: `backend.ops_governance.alerts.persistence` actual extraction complete |
| v4.16.0 / BE-001NA-04 | Added: `backend.ops_governance.alerts.persistence` single leaf closeout stops further split |
| v4.16.0 / BE-001NB-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects startup_initialization |
| v4.16.0 / BE-001NB-02 | Added: `backend.ops_governance.alerts.startup_initialization` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NB-03 | Added: `backend.ops_governance.alerts.startup_initialization` actual extraction complete |
| v4.16.0 / BE-001NB-04 | Added: `backend.ops_governance.alerts.startup_initialization` single leaf closeout stops further split |
| v4.16.0 / BE-001NC-01 | Added: `backend.ops_governance.alerts` parent residual judgment selects read_routes |
| v4.16.0 / BE-001NC-02 | Added: `backend.ops_governance.alerts.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NC-03 | Added: `backend.ops_governance.alerts.read_routes` actual extraction complete |
| v4.16.0 / BE-001NC-04 | Added: `backend.ops_governance.alerts.read_routes` single leaf closeout stops further split |
| v4.16.0 / BE-001ND-01 | Added: `backend.ops_governance.alerts.route_facade` static closeout and recovery_bridge selection |
| v4.16.0 / BE-001NE-01 | Added: `backend.ops_governance.alerts.recovery_bridge` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NE-02 | Added: `backend.ops_governance.alerts.recovery_bridge` actual extraction complete |
| v4.16.0 / BE-001NE-03 | Added: `backend.ops_governance.alerts.recovery_bridge` single leaf closeout stops further split |
| v4.16.0 / BE-001NF-01 | Added: `backend.ops_governance.alerts` parent residual judgment closes parent |
| v4.16.0 / BE-001NG-01 | Added: `backend.ops_governance` parent residual judgment selects snapshots |
| v4.16.0 / BE-001NH-01 | Added: `backend.ops_governance.snapshots` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NH-02 | Added: `backend.ops_governance.snapshots` actual extraction complete |
| v4.16.0 / BE-001NH-03 | Added: `backend.ops_governance.snapshots` single leaf closeout continues split |
| v4.16.0 / BE-001NI-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects snapshot_id_validation |
| v4.16.0 / BE-001NJ-01 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NJ-02 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` actual extraction complete |
| v4.16.0 / BE-001NJ-03 | Added: `backend.ops_governance.snapshots.snapshot_id_validation` single leaf closeout stops further split |
| v4.16.0 / BE-001NK-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects create_flow |
| v4.16.0 / BE-001NL-01 | Added: `backend.ops_governance.snapshots.create_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NL-02 | Added: `backend.ops_governance.snapshots.create_flow` actual extraction complete |
| v4.16.0 / BE-001NL-03 | Added: `backend.ops_governance.snapshots.create_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001NM-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects read_routes |
| v4.16.0 / BE-001NN-01 | Added: `backend.ops_governance.snapshots.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NN-02 | Added: `backend.ops_governance.snapshots.read_routes` actual extraction complete |
| v4.16.0 / BE-001NN-03 | Added: `backend.ops_governance.snapshots.read_routes` single leaf closeout stops further split |
| v4.16.0 / BE-001NO-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects restore_flow |
| v4.16.0 / BE-001NP-01 | Added: `backend.ops_governance.snapshots.restore_flow` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NP-02 | Added: `backend.ops_governance.snapshots.restore_flow` actual extraction complete |
| v4.16.0 / BE-001NP-03 | Added: `backend.ops_governance.snapshots.restore_flow` single leaf closeout stops further split |
| v4.16.0 / BE-001NQ-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects persistence |
| v4.16.0 / BE-001NR-01 | Added: `backend.ops_governance.snapshots.persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NR-02 | Added: `backend.ops_governance.snapshots.persistence` actual extraction complete |
| v4.16.0 / BE-001NR-03 | Added: `backend.ops_governance.snapshots.persistence` single leaf closeout stops further split |
| v4.16.0 / BE-001NS-01 | Added: `backend.ops_governance.snapshots` parent residual judgment selects signature_contract |
| v4.16.0 / BE-001NT-01 | Added: `backend.ops_governance.snapshots.signature_contract` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NT-02 | Added: `backend.ops_governance.snapshots.signature_contract` actual extraction complete |
| v4.16.0 / BE-001NT-03 | Added: `backend.ops_governance.snapshots.signature_contract` single leaf closeout stops further split |
| v4.16.0 / BE-001NU-01 | Added: `backend.ops_governance.snapshots.route_facade` static closeout and parent closeout selection |
| v4.16.0 / BE-001NV-01 | Added: `backend.ops_governance.snapshots` parent residual judgment closes parent |
| v4.16.0 / BE-001NW-01 | Added: `backend.ops_governance` parent residual judgment selects runbook |
| v4.16.0 / BE-001NX-01 | Added: `backend.ops_governance.runbook` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NX-02 | Added: `backend.ops_governance.runbook` actual extraction complete |
| v4.16.0 / BE-001NX-03 | Added: `backend.ops_governance.runbook` single leaf closeout continues split |
| v4.16.0 / BE-001NY-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects scenario_catalog |
| v4.16.0 / BE-001NZ-01 | Added: `backend.ops_governance.runbook.scenario_catalog` equivalence baseline and extraction plan |
| v4.16.0 / BE-001NZ-02 | Added: `backend.ops_governance.runbook.scenario_catalog` actual extraction complete |
| v4.16.0 / BE-001NZ-03 | Added: `backend.ops_governance.runbook.scenario_catalog` single leaf closeout |
| v4.16.0 / BE-001OA-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects read_routes |
| v4.16.0 / BE-001OB-01 | Added: `backend.ops_governance.runbook.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OB-02 | Added: `backend.ops_governance.runbook.read_routes` actual extraction complete |
| v4.16.0 / BE-001OB-03 | Added: `backend.ops_governance.runbook.read_routes` single leaf closeout |
| v4.16.0 / BE-001OC-01 | Added: `backend.ops_governance.runbook` parent residual judgment selects route_facade |
| v4.16.0 / BE-001OD-01 | Added: `backend.ops_governance.runbook.route_facade` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OD-02 | Added: `backend.ops_governance.runbook.route_facade` actual extraction complete |
| v4.16.0 / BE-001OD-03 | Added: `backend.ops_governance.runbook.route_facade` single leaf closeout |
| v4.16.0 / BE-001OE-01 | Added: `backend.ops_governance.runbook` parent closeout |
| v4.16.0 / BE-001OF-01 | Added: `backend.ops_governance` parent residual judgment selects chaos |
| v4.16.0 / BE-001OG-01 | Added: `backend.ops_governance.chaos` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OG-02 | Added: `backend.ops_governance.chaos` actual extraction complete |
| v4.16.0 / BE-001OG-03 | Added: `backend.ops_governance.chaos` single leaf closeout continues split |
| v4.16.0 / BE-001OH-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects report_persistence |
| v4.16.0 / BE-001OI-01 | Added: `backend.ops_governance.chaos.report_persistence` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OI-02 | Added: `backend.ops_governance.chaos.report_persistence` actual extraction complete |
| v4.16.0 / BE-001OI-03 | Added: `backend.ops_governance.chaos.report_persistence` single leaf closeout |
| v4.16.0 / BE-001OJ-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects experiment_creation |
| v4.16.0 / BE-001OK-01 | Added: `backend.ops_governance.chaos.experiment_creation` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OK-02 | Added: `backend.ops_governance.chaos.experiment_creation` actual extraction complete |
| v4.16.0 / BE-001OK-03 | Added: `backend.ops_governance.chaos.experiment_creation` single leaf closeout continues split |
| v4.16.0 / BE-001OL-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects perturbation_execution |
| v4.16.0 / BE-001OM-01 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OM-02 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` actual extraction complete |
| v4.16.0 / BE-001OM-03 | Added: `backend.ops_governance.chaos.experiment_creation.perturbation_execution` single leaf closeout |
| v4.16.0 / BE-001ON-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects report_projection |
| v4.16.0 / BE-001OO-01 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OO-02 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` actual extraction complete |
| v4.16.0 / BE-001OO-03 | Added: `backend.ops_governance.chaos.experiment_creation.report_projection` single leaf closeout |
| v4.16.0 / BE-001OP-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent residual judgment selects memory_commit |
| v4.16.0 / BE-001OQ-01 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OQ-02 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` actual extraction complete |
| v4.16.0 / BE-001OQ-03 | Added: `backend.ops_governance.chaos.experiment_creation.memory_commit` single leaf closeout |
| v4.16.0 / BE-001OR-01 | Added: `backend.ops_governance.chaos.experiment_creation` parent closeout |
| v4.16.0 / BE-001OS-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects read_routes |
| v4.16.0 / BE-001OT-01 | Added: `backend.ops_governance.chaos.read_routes` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OT-02 | Added: `backend.ops_governance.chaos.read_routes` actual extraction complete |
| v4.16.0 / BE-001OT-03 | Added: `backend.ops_governance.chaos.read_routes` single leaf closeout |
| v4.16.0 / BE-001OU-01 | Added: `backend.ops_governance.chaos` parent residual judgment selects route_facade |
| v4.16.0 / BE-001OV-01 | Added: `backend.ops_governance.chaos.route_facade` equivalence baseline and extraction plan |
| v4.16.0 / BE-001OV-02 | Added: `backend.ops_governance.chaos.route_facade` actual extraction complete |
| v4.16.0 / BE-001OV-03 | Added: `backend.ops_governance.chaos.route_facade` single leaf closeout |
| v4.16.0 / BE-001OW-01 | Added: `backend.ops_governance.chaos` parent closeout |
| v4.16.0 / BE-001OX-01 | Added: `backend.ops_governance` parent closeout |
| v4.16.0 / BE-001OY-01 | Added: `backend` parent residual judgment selects `backend.app_state_wiring` |
| v4.16.0 / BE-001OZ-01 | Added: `backend.app_state_wiring` single leaf closeout |
| v4.16.0 / BE-001PA-01 | Added: `backend` parent residual judgment selects `backend.test_support` |
| v4.16.0 / BE-001PB-01 | Added: `backend.test_support` single leaf closeout |
| v4.16.0 / BE-001PC-01 | Added: `backend` parent closeout |
| v4.16.0 / BE-001PD-01 | Added: `root` parent residual judgment selects `root.contracts` |
| v4.16.0 / BE-001PE-01 | `v4.16.0/910-root.contracts.baseline_plan.md` | `root.contracts` baseline plan and L2 child queue |
| v4.16.0 / BE-001PF-01 | `v4.16.0/911-root.contracts.parent_residual_judgment.api_surface.md` | `root.contracts` selects `contracts.api_surface` |
| v4.16.0 / BE-001PG-01 | `v4.16.0/912-root.contracts.api_surface.single_leaf_closeout.md` | `root.contracts.api_surface` closeout continues split |
| v4.16.0 / BE-001PH-01 | `v4.16.0/913-root.contracts.api_surface.parent_residual_judgment.openapi_http.md` | `root.contracts.api_surface` selects `openapi_http` |
| v4.16.0 / BE-001PI-01 | `v4.16.0/914-root.contracts.api_surface.openapi_http.single_leaf_closeout.md` | `root.contracts.api_surface.openapi_http` closeout |
| v4.16.0 / BE-001PJ-01 | `v4.16.0/915-root.contracts.api_surface.parent_residual_judgment.asyncapi_runtime_events.md` | `root.contracts.api_surface` selects `asyncapi_runtime_events` |
| v4.16.0 / BE-001PK-01 | `v4.16.0/916-root.contracts.api_surface.asyncapi_runtime_events.single_leaf_closeout.md` | `root.contracts.api_surface.asyncapi_runtime_events` closeout |
| v4.16.0 / BE-001PL-01 | `v4.16.0/917-root.contracts.api_surface.parent_closeout.md` | `root.contracts.api_surface` parent closeout |
| v4.16.0 / BE-001PM-01 | `v4.16.0/918-root.contracts.parent_residual_judgment.qrpc_core.md` | `root.contracts` selects `contracts.qrpc_core` |
| v4.16.0 / BE-001PN-01 | `v4.16.0/919-root.contracts.qrpc_core.baseline_plan.md` | `root.contracts.qrpc_core` baseline plan |
| v4.16.0 / BE-001PO-01 | `v4.16.0/920-root.contracts.qrpc_core.parent_residual_judgment.error_contract.md` | `root.contracts.qrpc_core` selects `error_contract` |
| v4.16.0 / BE-001PP-01 | `v4.16.0/921-root.contracts.qrpc_core.error_contract.single_leaf_closeout.md` | `root.contracts.qrpc_core.error_contract` closeout |
| v4.16.0 / BE-001PQ-01 | `v4.16.0/922-root.contracts.qrpc_core.parent_residual_judgment.event_envelope_proto.md` | `root.contracts.qrpc_core` selects `event_envelope_proto` |
| v4.16.0 / BE-001PR-01 | `v4.16.0/923-root.contracts.qrpc_core.event_envelope_proto.single_leaf_closeout.md` | `root.contracts.qrpc_core.event_envelope_proto` closeout |
| v4.16.0 / BE-001PS-01 | `v4.16.0/924-root.contracts.qrpc_core.parent_residual_judgment.plugin_contract.md` | `root.contracts.qrpc_core` selects `plugin_contract` |
| v4.16.0 / BE-001PT-01 | `v4.16.0/925-root.contracts.qrpc_core.plugin_contract.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract` baseline plan |
| v4.16.0 / BE-001PU-01 | `v4.16.0/926-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.taxonomy_extension.md` | `root.contracts.qrpc_core.plugin_contract` selects `taxonomy_extension` |
| v4.16.0 / BE-001PV-01 | `v4.16.0/927-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` baseline plan |
| v4.16.0 / BE-001PV-02 | `v4.16.0/928-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.extract_closeout.md` | `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` actual extraction complete |
| v4.16.0 / BE-001PV-03 | `v4.16.0/929-root.contracts.qrpc_core.plugin_contract.taxonomy_extension.single_leaf_closeout.md` | `root.contracts.qrpc_core.plugin_contract.taxonomy_extension` single leaf closeout |
| v4.16.0 / BE-001PW-01 | `v4.16.0/930-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.capability_contract.md` | `root.contracts.qrpc_core.plugin_contract` selects `capability_contract` |
| v4.16.0 / BE-001PX-01 | `v4.16.0/931-root.contracts.qrpc_core.plugin_contract.capability_contract.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract.capability_contract` baseline plan |
| v4.16.0 / BE-001PX-02 | `v4.16.0/932-root.contracts.qrpc_core.plugin_contract.capability_contract.extract_closeout.md` | `root.contracts.qrpc_core.plugin_contract.capability_contract` actual extraction complete |
| v4.16.0 / BE-001PX-03 | `v4.16.0/933-root.contracts.qrpc_core.plugin_contract.capability_contract.single_leaf_closeout.md` | `root.contracts.qrpc_core.plugin_contract.capability_contract` single leaf closeout |
| v4.16.0 / BE-001PY-01 | `v4.16.0/934-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.execution_security_dependency.md` | `root.contracts.qrpc_core.plugin_contract` selects `execution_security_dependency` |
| v4.16.0 / BE-001PZ-01 | `v4.16.0/935-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` baseline plan |
| v4.16.0 / BE-001PZ-02 | `v4.16.0/936-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.extract_closeout.md` | `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` actual extraction complete |
| v4.16.0 / BE-001PZ-03 | `v4.16.0/937-root.contracts.qrpc_core.plugin_contract.execution_security_dependency.single_leaf_closeout.md` | `root.contracts.qrpc_core.plugin_contract.execution_security_dependency` single leaf closeout |
| v4.16.0 / BE-001QA-01 | `v4.16.0/938-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.manifest_validation.md` | `root.contracts.qrpc_core.plugin_contract` selects `manifest_validation` |
| v4.16.0 / BE-001QB-01 | `v4.16.0/939-root.contracts.qrpc_core.plugin_contract.manifest_validation.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract.manifest_validation` baseline plan |
| v4.16.0 / BE-001QB-02 | `v4.16.0/940-root.contracts.qrpc_core.plugin_contract.manifest_validation.extract_closeout.md` | `root.contracts.qrpc_core.plugin_contract.manifest_validation` actual extraction complete |
| v4.16.0 / BE-001QB-03 | `v4.16.0/941-root.contracts.qrpc_core.plugin_contract.manifest_validation.single_leaf_closeout.md` | `root.contracts.qrpc_core.plugin_contract.manifest_validation` single leaf closeout |
| v4.16.0 / BE-001QC-01 | `v4.16.0/942-root.contracts.qrpc_core.plugin_contract.parent_residual_judgment.registry.md` | `root.contracts.qrpc_core.plugin_contract` selects `registry` |
| v4.16.0 / BE-001QD-01 | `v4.16.0/943-root.contracts.qrpc_core.plugin_contract.registry.baseline_plan.md` | `root.contracts.qrpc_core.plugin_contract.registry` baseline plan |
| v4.16.0 / BE-001QD-02 | `v4.16.0/944-root.contracts.qrpc_core.plugin_contract.registry.extract_closeout.md` | `root.contracts.qrpc_core.plugin_contract.registry` actual extraction complete |
| v4.16.0 / BE-001QD-03 | `v4.16.0/945-root.contracts.qrpc_core.plugin_contract.registry.single_leaf_closeout.md` | `root.contracts.qrpc_core.plugin_contract.registry` single leaf closeout |
| v4.16.0 / BE-001QE-01 | `v4.16.0/946-root.contracts.qrpc_core.plugin_contract.parent_closeout.md` | `root.contracts.qrpc_core.plugin_contract` parent closeout |
| v4.16.0 / BE-001QF-01 | `v4.16.0/947-root.contracts.qrpc_core.parent_residual_judgment.strategy_ir.md` | `root.contracts.qrpc_core` selects `strategy_ir` |
| v4.16.0 / BE-001QG-01 | `v4.16.0/948-root.contracts.qrpc_core.strategy_ir.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir` baseline plan |
| v4.16.0 / BE-001QH-01 | `v4.16.0/949-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.version_unknown_error.md` | `root.contracts.qrpc_core.strategy_ir` selects `version_unknown_error` |
| v4.16.0 / BE-001QI-01 | `v4.16.0/950-root.contracts.qrpc_core.strategy_ir.version_unknown_error.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.version_unknown_error` baseline plan |
| v4.16.0 / BE-001QI-02 | `v4.16.0/951-root.contracts.qrpc_core.strategy_ir.version_unknown_error.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.version_unknown_error` actual extraction complete |
| v4.16.0 / BE-001QI-03 | `v4.16.0/952-root.contracts.qrpc_core.strategy_ir.version_unknown_error.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.version_unknown_error` single leaf closeout |
| v4.16.0 / BE-001QJ-01 | `v4.16.0/953-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.metadata_source.md` | `root.contracts.qrpc_core.strategy_ir` selects `metadata_source` |
| v4.16.0 / BE-001QK-01 | `v4.16.0/954-root.contracts.qrpc_core.strategy_ir.metadata_source.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.metadata_source` baseline plan |
| v4.16.0 / BE-001QK-02 | `v4.16.0/955-root.contracts.qrpc_core.strategy_ir.metadata_source.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.metadata_source` actual extraction complete |
| v4.16.0 / BE-001QK-03 | `v4.16.0/956-root.contracts.qrpc_core.strategy_ir.metadata_source.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.metadata_source` single leaf closeout |
| v4.16.0 / BE-001QL-01 | `v4.16.0/957-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.signal_indicator.md` | `root.contracts.qrpc_core.strategy_ir` selects `signal_indicator` |
| v4.16.0 / BE-001QM-01 | `v4.16.0/958-root.contracts.qrpc_core.strategy_ir.signal_indicator.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.signal_indicator` baseline plan |
| v4.16.0 / BE-001QM-02 | `v4.16.0/959-root.contracts.qrpc_core.strategy_ir.signal_indicator.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.signal_indicator` actual extraction complete |
| v4.16.0 / BE-001QM-03 | `v4.16.0/960-root.contracts.qrpc_core.strategy_ir.signal_indicator.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.signal_indicator` single leaf closeout |
| v4.16.0 / BE-001QN-01 | `v4.16.0/961-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.logic_position.md` | `root.contracts.qrpc_core.strategy_ir` selects `logic_position` |
| v4.16.0 / BE-001QO-01 | `v4.16.0/962-root.contracts.qrpc_core.strategy_ir.logic_position.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.logic_position` baseline plan |
| v4.16.0 / BE-001QO-02 | `v4.16.0/963-root.contracts.qrpc_core.strategy_ir.logic_position.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.logic_position` actual extraction complete |
| v4.16.0 / BE-001QO-03 | `v4.16.0/964-root.contracts.qrpc_core.strategy_ir.logic_position.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.logic_position` single leaf closeout |
| v4.16.0 / BE-001QP-01 | `v4.16.0/965-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.risk_contract.md` | `root.contracts.qrpc_core.strategy_ir` selects `risk_contract` |
| v4.16.0 / BE-001QQ-01 | `v4.16.0/966-root.contracts.qrpc_core.strategy_ir.risk_contract.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.risk_contract` baseline plan |
| v4.16.0 / BE-001QQ-02 | `v4.16.0/967-root.contracts.qrpc_core.strategy_ir.risk_contract.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.risk_contract` actual extraction complete |
| v4.16.0 / BE-001QQ-03 | `v4.16.0/968-root.contracts.qrpc_core.strategy_ir.risk_contract.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.risk_contract` single leaf closeout |
| v4.16.0 / BE-001QR-01 | `v4.16.0/969-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.data_requirement.md` | `root.contracts.qrpc_core.strategy_ir` selects `data_requirement` |
| v4.16.0 / BE-001QS-01 | `v4.16.0/970-root.contracts.qrpc_core.strategy_ir.data_requirement.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.data_requirement` baseline plan |
| v4.16.0 / BE-001QS-02 | `v4.16.0/971-root.contracts.qrpc_core.strategy_ir.data_requirement.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.data_requirement` actual extraction complete |
| v4.16.0 / BE-001QS-03 | `v4.16.0/972-root.contracts.qrpc_core.strategy_ir.data_requirement.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.data_requirement` single leaf closeout |
| v4.16.0 / BE-001QT-01 | `v4.16.0/973-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.execution_contract.md` | `root.contracts.qrpc_core.strategy_ir` selects `execution_contract` |
| v4.16.0 / BE-001QU-01 | `v4.16.0/974-root.contracts.qrpc_core.strategy_ir.execution_contract.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.execution_contract` baseline plan |
| v4.16.0 / BE-001QU-02 | `v4.16.0/975-root.contracts.qrpc_core.strategy_ir.execution_contract.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.execution_contract` actual extraction complete |
| v4.16.0 / BE-001QU-03 | `v4.16.0/976-root.contracts.qrpc_core.strategy_ir.execution_contract.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.execution_contract` single leaf closeout |
| v4.16.0 / BE-001QV-01 | `v4.16.0/977-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.gap_unknown_annotation.md` | `root.contracts.qrpc_core.strategy_ir` selects `gap_unknown_annotation` |
| v4.16.0 / BE-001QW-01 | `v4.16.0/978-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` baseline plan |
| v4.16.0 / BE-001QW-02 | `v4.16.0/979-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` actual extraction complete |
| v4.16.0 / BE-001QW-03 | `v4.16.0/980-root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` single leaf closeout |
| v4.16.0 / BE-001QX-01 | `v4.16.0/981-root.contracts.qrpc_core.strategy_ir.parent_residual_judgment.root_validation.md` | `root.contracts.qrpc_core.strategy_ir` selects `root_validation` |
| v4.16.0 / BE-001QY-01 | `v4.16.0/982-root.contracts.qrpc_core.strategy_ir.root_validation.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` baseline plan |
| v4.16.0 / BE-001QY-02 | `v4.16.0/983-root.contracts.qrpc_core.strategy_ir.root_validation.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` actual extraction complete |
| v4.16.0 / BE-001QY-03 | `v4.16.0/984-root.contracts.qrpc_core.strategy_ir.root_validation.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` continues split |
| v4.16.0 / BE-001QZ-01 | `v4.16.0/985-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.identity_required_validation.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` selects `identity_required_validation` |
| v4.16.0 / BE-001RA-01 | `v4.16.0/986-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` baseline plan |
| v4.16.0 / BE-001RA-02 | `v4.16.0/987-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` actual extraction complete |
| v4.16.0 / BE-001RA-03 | `v4.16.0/988-root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` single leaf closeout |
| v4.16.0 / BE-001RB-01 | `v4.16.0/989-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.signal_logic_validation.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` selects `signal_logic_validation` |
| v4.16.0 / GOV-SAME-PARENT-PARALLEL | `v4.16.0/990-governance.same_parent_parallel_children.protocol_update.md` | Recursive speed protocol allows guarded same-parent child parallel waves |
| v4.16.0 / BE-001RC-01 | `v4.16.0/991-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.baseline_plan.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` baseline plan |
| v4.16.0 / BE-001RC-02 | `v4.16.0/992-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.extract_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` actual extraction complete |
| v4.16.0 / BE-001RC-03 | `v4.16.0/993-root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation.single_leaf_closeout.md` | `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` single leaf closeout |
| v4.16.0 / BE-001RD-01 | `v4.16.0/994-root.contracts.qrpc_core.strategy_ir.root_validation.parent_residual_judgment.risk_validation.md` | `root.contracts.qrpc_core.strategy_ir.root_validation` selects `risk_validation` |
