# v4.16.0 backend 九叶模块壳抽离记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001B。
> 基准: `28-backend大模块分层统计.md`、`29-backend.interface_boundary等价基线.md`。
> 判定: 已建立 `src/backend/` 父模块与 9 个叶子 facade；真实 handler、state owner、response schema 和 artifact schema 保持原位。

---

## 抽离结果

本批次完成的是“模块壳抽离”，不是 handler 迁移。

```text
src/backend/mod.rs
  -> interface_boundary.rs
  -> capability.rs
  -> strategy_config.rs
  -> runtime.rs
  -> graph_compile.rs
  -> storage_security.rs
  -> ops_governance.rs
  -> app_state_wiring.rs
  -> test_support.rs
```

`src/app_router.rs` 现在通过 `backend.interface_boundary` 调用各叶子 route facade:

```text
build_app_router
  -> backend.interface_boundary
  -> backend capability / strategy_config / runtime / graph_compile /
     storage_security / ops_governance / app_state_wiring / test_support
  -> existing handler
  -> existing state owner
```

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001B backend 九叶模块壳抽离 | 扩展 |
| 规范矩阵 | 父子通信、route owner facade、旧 handler 保留、状态所有权冻结 | 扩展 |
| 引导矩阵 | `src/backend/`、9 个 backend 叶子、全量树路径覆盖 | 扩展 |
| 模块树 | `backend`、`backend.interface_boundary`、9 个 backend 叶子 | 代码壳落位 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 九叶模块壳与根7.6 v4.16 里程碑 |
| 模块树节点 | `backend`、`backend.interface_boundary`、`backend.capability`、`backend.strategy_config`、`backend.runtime`、`backend.graph_compile`、`backend.storage_security`、`backend.ops_governance`、`backend.app_state_wiring`、`backend.test_support` |
| 真实文件 | `src/backend/mod.rs`、`src/backend/interface_boundary.rs`、`src/backend/capability.rs`、`src/backend/strategy_config.rs`、`src/backend/runtime.rs`、`src/backend/graph_compile.rs`、`src/backend/storage_security.rs`、`src/backend/ops_governance.rs`、`src/backend/app_state_wiring.rs`、`src/backend/test_support.rs`、`src/app_router.rs`、`src/lib.rs` |
| public 方法 | `build_app_router`、`get_capabilities`、`register_strategy_config_routes`、`register_runtime_routes`、`register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes`、`register_credential_routes`、`register_test_scenario_routes` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_graph_versions`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 九叶落位

| 叶子 | 新模块壳 | 保留实现 | 当前状态 |
| --- | --- | --- | --- |
| `backend.interface_boundary` | `src/backend/interface_boundary.rs` | `src/app_router.rs` | 父级 route facade 已接入 |
| `backend.capability` | `src/backend/capability.rs` | `src/capability_api.rs` | `/api/capabilities` 经叶子 facade 进入 |
| `backend.strategy_config` | `src/backend/strategy_config.rs` | `src/strategy_config_api.rs` | strategy config routes 经叶子 facade 进入 |
| `backend.runtime` | `src/backend/runtime.rs` | `src/runtime/mod.rs`、`src/runtime/*.rs` | runtime routes 经叶子 facade 进入 |
| `backend.graph_compile` | `src/backend/graph_compile.rs` | `src/graph_api.rs`、`src/graph_quantscript_api.rs`、`src/compile_api.rs` | graph/compile routes 经叶子 facade 进入 |
| `backend.storage_security` | `src/backend/storage_security.rs` | `src/credential_api.rs`、`src/credential_vault.rs`、`src/storage_lifecycle.rs` | credential route 经叶子 facade 进入 |
| `backend.ops_governance` | `src/backend/ops_governance.rs` | `src/alert_engine.rs`、`src/sandbox_verification.rs`、`src/snapshot_service.rs`、`src/runbook.rs`、`src/chaos_experiment.rs`、`src/hotswap_api.rs` | ops routes 经叶子 facade 进入 |
| `backend.app_state_wiring` | `src/backend/app_state_wiring.rs` | `src/app_runtime_helpers.rs` | health 和 state attach 经叶子 facade 进入；`new_app_state` 保持兼容 re-export |
| `backend.test_support` | `src/backend/test_support.rs` | `src/api_test_scenario.rs`、`src/test_runner.rs`、`src/tests_backend.rs` | test scenario route 经叶子 facade 进入 |

---

## 保留边界

- 不移动现有 handler 文件。
- 不删除旧模块声明。
- 不迁移 `AppState` 字段、runtime record、artifact、credential vault、storage lifecycle 或锁顺序。
- 不改变 route 顺序、response schema、error code 或 artifact schema。
- 不整理 E2E，不删除旧测试程序。
- 不进入发布版本过渡。

---

## 等价证据

| 证据 | 目的 |
| --- | --- |
| `cargo fmt --check` | 格式不漂移 |
| `cargo check -p quantpilot` | 九叶 facade 类型边界通过 |
| `cargo test -p quantpilot --test api_run` | runtime route facade 等价 |
| `cargo test -p quantpilot --test api_backtest` | backtest route facade 等价 |
| `cargo test -p quantpilot --test api_graph_versions` | graph route facade 等价 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence/artifact contract 等价 |
| `cargo test -p quantpilot --test api_ai_proposal` | strategy config / AI proposal route 等价 |
| `tools/check-matrix-governance.ps1` | 模块树和里程碑门禁 |
| `tools/check-full-feature-tree.ps1` | 新代码文件和文档路径覆盖 |
| `tools/check-utf8.ps1` | 文档与代码编码稳定 |

---

## 后续递归状态

`backend` 当前推进到 R2/R3 之间:

1. R1 叶子划分已完成当前版本: 9 个叶子都有代码壳。
2. R2 抽离已完成当前版本: route owner 已经通过 `backend.interface_boundary` 和各叶子 facade 进入。
3. R3 叶子整理未完成: 还没有逐叶 closeout，也没有判断是否继续细分。
4. R4 细分价值判断未完成: runtime、graph/compile、storage/security 等大叶子仍可能值得继续拆。

下一步应先对 9 个叶子做等价整理和单叶 closeout，再判断哪些叶子值得继续细分。

---

## 幻觉检查点

AI 声称 backend 抽离完成时，必须说明“完成的是九叶模块壳抽离”。如果声称 handler 已迁移、runtime 已重构、storage/security 已整理或测试资产已汰换，必须给出单独 closeout 和等价证据；否则视为幻觉风险。
