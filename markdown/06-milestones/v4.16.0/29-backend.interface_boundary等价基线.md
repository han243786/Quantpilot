# v4.16.0 backend.interface_boundary 等价基线

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001A。
> 基准: `06-后端接口边界首批抽离方案.md`、`28-backend大模块分层统计.md`。
> 判定: 先把 `backend.interface_boundary` 的 route owner、public/接口入口、保留边界和回归证据固定下来；本批不移动 handler。

---

## 目标

BE-001 的第一步不是改代码，而是建立后端接口父模块的等价基线。后续任何 backend 代码抽离都必须能回到这张基线回答四个问题:

1. 输入从哪里进来。
2. 输出由谁负责。
3. route owner 属于哪个叶子。
4. 哪些 handler、state、schema 明确没有迁移。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001A 文档基线、后续 BE-001B/BE-001C 代码批次 | 扩展 |
| 规范矩阵 | 父子通信、public 方法登记、旧 handler 保留、状态所有权冻结 | 扩展 |
| 引导矩阵 | `backend.interface_boundary` 白箱节点、route owner 表、后端 API tests | 扩展 |
| 模块树 | `backend.interface_boundary`、`backend.capability`、`backend.strategy_config`、`backend.runtime`、`backend.graph_compile` | 固定接口父边界 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 后端 API/运行/编译入口 |
| 模块树节点 | `backend.interface_boundary` |
| 真实文件 | `src/app_router.rs`、`src/capability_api.rs`、`src/strategy_config_api.rs`、`src/runtime/mod.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs`、`src/compile_api.rs`、`src/alert_engine.rs`、`src/sandbox_verification.rs`、`src/snapshot_service.rs`、`src/runbook.rs`、`src/chaos_experiment.rs`、`src/credential_api.rs`、`src/api_test_scenario.rs` |
| public 方法 | `build_app_router`、`get_capabilities`、`register_strategy_config_routes`、`register_runtime_routes`、`register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_graph_versions`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_ai_proposal`、`tools/check-matrix-governance.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | HTTP request | frontend、tests、CLI/local tools | 不改 `/api/*` 语义 |
| 输入 | `AppState` | `src/app_runtime_helpers.rs`、`system.entry.backend_process` 启动链 | 不迁移 AppState owner |
| 输入 | route registration | backend 子叶 | 不删除旧 handler |
| 输出 | Axum Router | `build_app_router` 调用方 | `build_app_router` 仍是父入口 |
| 输出 | API response | frontend、tests | 不改 response schema 和 error code |
| 输出 | SSE / artifact / diagnostics | runtime、graph/compile、strategy config | 不改 artifact schema |

---

## route owner 基线

| 接口域 | 父模块 | 真实文件 | 入口 | 当前处理 |
| --- | --- | --- | --- | --- |
| router root | `backend.interface_boundary` | `src/app_router.rs` | `build_app_router` | 父入口保持不变 |
| capability | `backend.capability` | `src/capability_api.rs` | `get_capabilities`、`/api/capabilities` | 能力真源保持后端拥有 |
| strategy config | `backend.strategy_config` | `src/strategy_config_api.rs` | `register_strategy_config_routes` | 不改 preflight、artifact、diff 语义 |
| runtime | `backend.runtime` | `src/runtime/mod.rs` | `register_runtime_routes` | 不迁移 runtime state 或 handler |
| graph | `backend.graph_compile` | `src/graph_api.rs` | `register_graph_routes` | 不绕过 graph version 记录 |
| QuantScript graph | `backend.graph_compile` | `src/graph_quantscript_api.rs` | `register_graph_quantscript_routes` | 不允许任意主机代码 |
| compile | `backend.graph_compile` | `src/compile_api.rs` | `register_compile_routes` | 不把 strategy_ir 当 runtime 真源 |
| credential | `backend.storage_security` | `src/credential_api.rs` | `register_credential_routes` | 不改凭证存储语义 |
| ops/sandbox | `backend.ops_governance` | `src/sandbox_verification.rs`、`src/alert_engine.rs`、`src/snapshot_service.rs`、`src/runbook.rs`、`src/chaos_experiment.rs` | `register_sandbox_verification_routes`、`register_alert_routes`、`register_snapshot_routes`、`register_runbook_routes`、`register_chaos_routes` | 只登记，不迁移 |
| hotswap / test scenario | `backend.ops_governance`、`backend.test_support` | `src/hotswap_api.rs`、`src/api_test_scenario.rs` | app router wrapper、`register_test_scenario_routes` | 只登记，不作为业务抽离完成证据 |

---

## public / 接口入口分类

| 分类 | 方法 | 处理规则 |
| --- | --- | --- |
| 父级 public 入口 | `build_app_router` | 后续 facade 必须挂在它下面，不得绕过 |
| 对外 API handler | `get_capabilities` | 能力真源不得被前端静态判断替代 |
| route registration | `register_strategy_config_routes`、`register_runtime_routes`、`register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes` | 先作为接口入口登记，后续是否改可见性需单独提案 |
| ops route registration | `register_alert_routes`、`register_sandbox_verification_routes`、`register_snapshot_routes`、`register_runbook_routes`、`register_chaos_routes`、`register_credential_routes`、`register_test_scenario_routes` | 本批只登记 route owner，不做抽离完成声明 |
| 保留外部边界 | `new_app_state`、runtime record/artifact helpers、storage/credential helpers | 不属于 BE-001A 迁移范围 |

---

## 兼容桥

当前兼容桥保持旧路径:

```text
system.entry.backend_process
  -> run_api_server
  -> backend.interface_boundary
  -> build_app_router
  -> register_*_routes
  -> existing handler
  -> existing state owner
```

后续如果进入 BE-001B 代码批次，只允许增加轻量 facade 或 route owner 聚合层:

```text
build_app_router
  -> backend interface facade
  -> register_*_routes
  -> existing handler
```

失败回退点仍是旧路径，不删除旧 handler，不改 state owner。

---

## 等价证据

注意: 后端 integration test 必须使用 `cargo test -p quantpilot --test <name>` 形式。`cargo test -p quantpilot api_run` 这类过滤器可能只得到 0 tests 且返回成功，不能作为 BE-001 等价证据。

| 证据 | 覆盖范围 | 当前口径 |
| --- | --- | --- |
| `cargo check -p quantpilot` | 后端编译与类型边界 | BE-001A 文档后仍应通过 |
| `cargo test -p quantpilot --test api_run` | runtime run API | route owner 不能断 |
| `cargo test -p quantpilot --test api_backtest` | backtest API 和 artifact | response schema 不能变 |
| `cargo test -p quantpilot --test api_graph_versions` | graph version API | graph owner 不能漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | runtime evidence contract | artifact/evidence 字段不能变 |
| `cargo test -p quantpilot --test api_ai_proposal` | strategy config / AI proposal 接口 | capability 与配置边界不能漂移 |
| `tools/check-matrix-governance.ps1` | 三矩阵与模块树入口 | 文档入口不能断 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 | 新文档与真实文件引用不能漂移 |

---

## 幻觉检查点

AI 或开发者声称 BE-001 已完成时，必须同时说明:

1. 完成的是 `backend.interface_boundary` 等价基线，还是代码 facade 抽离。
2. `build_app_router` 是否仍是父入口。
3. 哪些 `register_*_routes` 已登记 route owner。
4. 哪些 handler、state owner、response schema 明确没有迁移。
5. 跑过哪些后端 API tests 或替代证据。

说不清上述任一项时，视为幻觉风险，停止继续扩张。

---

## 本批次不做

- 不移动 `src/*.rs` 文件。
- 不把 `src/lib.rs` 或 `new_app_state` 并入接口边界。
- 不迁移 runtime handler。
- 不迁移 compile handler。
- 不迁移 credential/storage/auth 状态。
- 不改 `/api/*` route 顺序。
- 不整理 E2E，不删除旧测试程序。
- 不宣称 `backend.runtime`、`backend.graph_compile`、`backend.storage_security` 已完成抽离。

---

## 验收标准

1. BE-001A 进入 v4.16 里程碑索引。
2. 模块树和全量树能定位 `backend.interface_boundary` 等价基线。
3. route owner 表覆盖当前关键 `register_*_routes`。
4. 后续代码批次必须先引用本等价基线。
5. 治理门禁能发现本文件缺失。
