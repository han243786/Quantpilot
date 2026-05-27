# v4.16.0 backend 大模块分层统计

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 基准: `03-后端抽离登记.md`、`06-后端接口边界首批抽离方案.md`、`13-递归模块化全局根流程.md`。
> 判定: `root.backend` 先进入 R1 叶子划分；本批次只登记白箱边界和等价基线，不移动 handler、state owner 或 response schema。

---

## 目标

本文件把 backend 从“一个 src 大包”拆成可治理的顶层白箱网络，作为后续抽离的父级坐标。

当前结论:

1. `backend` 是六大顶层模块之一，路径为 `root.backend`。
2. backend 当前按 3 层理解: `root.backend` -> L2 业务/接口域 -> L3 叶子文件或 route owner。
3. 第一批真实推进仍是 BE-001 `backend.interface_boundary`，即 router/API/facade 父级边界。
4. `backend.runtime`、`backend.graph_compile`、`backend.storage_security` 等仍是候选叶子，不因为本文件宣告完成。
5. `tests_backend.rs` 和 `test_runner.rs` 先登记为测试支撑，不作为生产模块 owner。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 R1 backend 叶子划分、BE-001 后续批次 | 扩展 |
| 规范矩阵 | 父子通信、public 入口登记、状态所有权冻结、测试资产汰换边界 | 扩展 |
| 引导矩阵 | `root.backend`、`backend.interface_boundary`、backend 叶子候选清单 | 扩展 |
| 模块树 | `backend` 父节点、`backend.interface_boundary`、既有 backend 子节点 | 建立 backend 顶层处理口径 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 后端系统、根7.6 v4.16 里程碑 |
| 模块树节点 | `backend`、`backend.interface_boundary`、`backend.runtime`、`backend.graph_compile`、`backend.storage_security`、`backend.capability`、`backend.strategy_config` |
| 真实文件 | `src/app_router.rs`、`src/capability_api.rs`、`src/strategy_config_api.rs`、`src/runtime/mod.rs`、`src/graph_api.rs`、`src/compile_api.rs`、`src/tests_backend.rs` |
| public 方法 | `build_app_router`、`get_capabilities`、`register_strategy_config_routes`、`register_runtime_routes`、`register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes` |
| 测试/门禁 | `cargo check -p quantpilot`、后端 API tests、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 规模事实

统计口径: `src/**/*.rs`，排除 `src/system/**`。

| 指标 | 当前事实 | 结论 |
| --- | --- | --- |
| 文件数 | 约 53 个 Rust 文件 | backend 不是单叶模块 |
| 最大测试文件 | `src/tests_backend.rs` 约 6200 行 | 测试支撑必须单独登记，不能混入生产 owner |
| 最大生产文件 | `src/runtime/mutation.rs`、`src/strategy_config_api.rs`、`src/frontend_api_types.rs`、`src/lib.rs`、`src/test_runner.rs` | runtime、strategy config、API types 和测试 runner 都需要独立边界 |
| 当前父入口 | `src/app_router.rs` 的 `build_app_router` | BE-001 必须先稳定接口父入口 |
| 状态风险 | AppState、runtime record、artifact、credential、storage lifecycle | 不允许在抽离阶段迁移状态所有权或锁边界 |

---

## backend 三层网络

```text
root.backend
  -> backend.interface_boundary
       -> backend.capability
       -> backend.strategy_config
       -> backend.runtime
       -> backend.graph_compile
       -> backend.ops_governance
       -> backend.storage_security
  -> backend.app_state_wiring
  -> backend.test_support
```

这张网络只描述治理坐标。它不改变真实代码路径，也不允许子模块横向绕过父模块通信。

---

## L2 叶子候选

| # | 叶子候选 | 状态 | 真实文件 | public / 接口入口 | 本阶段处理 |
| --- | --- | :--: | --- | --- | --- |
| B0 | `backend.interface_boundary` | 已启动 | `src/app_router.rs`、`src/capability_api.rs`、`src/strategy_config_api.rs`、`src/runtime/mod.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs`、`src/compile_api.rs` | `build_app_router`、`get_capabilities`、各 `register_*_routes` | 先做等价基线和 route owner 白箱 |
| B1 | `backend.runtime` | 候选 | `src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`、`src/runtime_*`、`src/backtest_artifacts.rs` | `register_runtime_routes`、runtime/backtest API | 暂不抽 handler，不迁移状态 |
| B2 | `backend.graph_compile` | 候选 | `src/graph_api.rs`、`src/graph_quantscript_api.rs`、`src/graph_version_compare.rs`、`src/compile_api.rs`、`src/compile_*` | `register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes` | 暂不改 compile 语义 |
| B3 | `backend.strategy_config` | 候选 | `src/strategy_config_api.rs`、`src/frontend_api_types.rs`、`src/frontend_runtime_mapping.rs` | `register_strategy_config_routes`、strategy config API | 暂不拆 response schema |
| B4 | `backend.capability` | 候选 | `src/capability_api.rs` | `get_capabilities`、`/api/capabilities` | 保持能力真源 |
| B5 | `backend.storage_security` | 候选 | `src/storage_lifecycle.rs`、`src/credential_vault.rs`、`src/credential_api.rs`、`src/safe_log.rs`、`src/auth/mod.rs`、`src/auth_middleware.rs`、`src/rate_limiter.rs`、`src/backup.rs` | credential/storage/auth helper 和 API | 暂不改凭证、TTL、quota、认证语义 |
| B6 | `backend.ops_governance` | 候选 | `src/alert_engine.rs`、`src/sandbox_verification.rs`、`src/snapshot_service.rs`、`src/runbook.rs`、`src/chaos_experiment.rs`、`src/hotswap_api.rs`、`src/migration_sender.rs`、`src/collaboration.rs` | alert/sandbox/snapshot/runbook/chaos/hotswap route registration | 先登记，不作为 BE-001 首批迁移目标 |
| B7 | `backend.app_state_wiring` | 候选 | `src/lib.rs`、`src/app_runtime_helpers.rs`、`src/main.rs` | `new_app_state`、`run_server` 兼容桥周边 | 暂不迁移 AppState 所有权 |
| B8 | `backend.test_support` | 候选 | `src/tests_backend.rs`、`src/test_runner.rs`、`src/api_test_scenario.rs` | 后端 API tests、test scenario route | E2E 整理延后，旧测试程序不在本批删除 |

---

## 父子通信硬规则

1. 后端接口变更必须先经过 `backend.interface_boundary`，再进入具体子叶。
2. 子叶不得横向抢 route owner、handler、state owner、response schema 或 artifact schema。
3. `backend.runtime` 与 `backend.graph_compile` 的协作必须经已登记 API/facade 或 compile 契约，不允许临时互调。
4. `backend.storage_security` 只能通过明确 storage/credential/auth helper 或 API 进入，不允许业务模块私拼敏感路径。
5. `backend.test_support` 只能证明等价，不能替代生产 owner。
6. 任何进入发布过渡的横向连接，必须由开发者明确提出；AI 不得主动提出发布过渡。

---

## 本批次不做

- 不移动 Rust 文件。
- 不新增 backend 代码模块。
- 不切换主 API。
- 不删除旧 handler。
- 不迁移 AppState、runtime state、executor state 或锁边界。
- 不改 response schema、artifact schema、error code。
- 不整理 E2E，不删除旧测试程序。
- 不宣称 backend 顶层已经完成。

---

## 后续递归顺序

| 顺序 | 动作 | 进入条件 | 暂停点 |
| --- | --- | --- | --- |
| R1 | 固定 `root.backend` 叶子候选 | 本文件落地并通过治理门禁 | 叶子 owner 不清 |
| R2 | 推进 BE-001 `backend.interface_boundary` | `29-backend.interface_boundary等价基线.md` 已建立 | route 顺序、response schema 或 state owner 变化 |
| R3 | 整理 BE-001 叶子 | route owner 和 public/内部实现分类清楚 | handler 迁移需求出现 |
| R4 | 判断 BE-001 是否继续细分 | 有独立 owner 和等价证据 | 只能靠目录美化解释拆分 |
| R5 | 选择下一个 backend 叶子 | BE-001 当前阶段 closeout 后 | runtime/storage/executor 边界需要开发者决策 |

---

## 验收标准

1. `root.backend` 的顶层叶子候选进入里程碑索引。
2. 模块树能定位 `backend` 父节点和 BE-001 接口边界。
3. 后续 backend 抽离必须引用本文件或 BE-001 等价基线。
4. 治理门禁能发现本文件缺失。
5. 完成本文件后只宣告 backend R1 叶子划分启动，不宣告代码抽离完成。
