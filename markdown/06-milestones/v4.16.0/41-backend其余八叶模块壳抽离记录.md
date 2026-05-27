# v4.16.0 backend 其余八叶模块壳抽离记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E。
> 基准: `31-39 backend 单叶 closeout`、`40-backend.strategy_config_L3模块壳抽离记录.md`。
> 判定: 继续完成 `backend.strategy_config` 之外的 8 个 L2 叶子薄壳抽离；只建立子 facade 和父子连接，不迁移 handler、schema、state owner、锁、测试资产或旧实现。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | backend R5 局部递归、BE-001E 八叶模块壳抽离 | 扩展 |
| 规范矩阵 | 父子通信、旧实现保留、状态所有权冻结 | 固化 |
| 引导矩阵 | `backend.interface_boundary.*`、`backend.runtime.*`、`backend.graph_compile.*`、`backend.storage_security.*`、`backend.ops_governance.*`、`backend.capability.*`、`backend.app_state_wiring.*`、`backend.test_support.*` | 扩展 |
| 模块树 | backend 其余 8 叶子 | 新增子 facade 坐标 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 九叶模块壳 |
| 模块树节点 | `backend.interface_boundary`、`backend.capability`、`backend.runtime`、`backend.graph_compile`、`backend.storage_security`、`backend.ops_governance`、`backend.app_state_wiring`、`backend.test_support` |
| 真实文件 | `src/backend/interface_boundary/app_state_bridge.rs`、`src/backend/interface_boundary/capability_bridge.rs`、`src/backend/interface_boundary/graph_compile_bridge.rs`、`src/backend/interface_boundary/ops_governance_bridge.rs`、`src/backend/interface_boundary/runtime_bridge.rs`、`src/backend/interface_boundary/storage_security_bridge.rs`、`src/backend/interface_boundary/strategy_config_bridge.rs`、`src/backend/interface_boundary/test_support_bridge.rs`、`src/backend/capability/snapshot.rs`、`src/backend/runtime/routes.rs`、`src/backend/graph_compile/compile.rs`、`src/backend/graph_compile/graph.rs`、`src/backend/graph_compile/quantscript_graph.rs`、`src/backend/storage_security/credential_api.rs`、`src/backend/storage_security/credential_vault.rs`、`src/backend/ops_governance/alerts.rs`、`src/backend/ops_governance/chaos.rs`、`src/backend/ops_governance/hotswap.rs`、`src/backend/ops_governance/runbook.rs`、`src/backend/ops_governance/sandbox.rs`、`src/backend/ops_governance/snapshots.rs`、`src/backend/app_state_wiring/health_route.rs`、`src/backend/app_state_wiring/state_factory.rs`、`src/backend/test_support/scenario.rs` |
| public 方法 | `register_*_routes`、`get_capabilities`、`health`、`attach_state`、`new_app_state`、`CredentialVault` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run --test api_backtest --test api_graph_versions --test api_ai_proposal --test api_auth`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 抽离内容

| L2 叶子 | 新子 facade | 保留实现 | 本批结论 |
| --- | --- | --- | --- |
| `backend.interface_boundary` | `app_state_bridge`、`capability_bridge`、`graph_compile_bridge`、`ops_governance_bridge`、`runtime_bridge`、`storage_security_bridge`、`strategy_config_bridge`、`test_support_bridge` | `src/app_router.rs`、各 L2 facade | 父级 route owner 继续只做分发 |
| `backend.capability` | `capability.snapshot` | `src/capability_api.rs` | capability 真源仍在后端 API |
| `backend.runtime` | `runtime.routes` | `src/runtime/mod.rs` | runtime route 聚合只经子 facade 进入 |
| `backend.graph_compile` | `graph_compile.compile`、`graph_compile.graph`、`graph_compile.quantscript_graph` | `src/compile_api.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs` | compile/graph/QS route facade 分开 |
| `backend.storage_security` | `storage_security.credential_api`、`storage_security.credential_vault` | `src/credential_api.rs`、`src/credential_vault.rs` | 只拆 credential route 和 vault re-export；auth/storage/safe_log 仍暂停 |
| `backend.ops_governance` | `ops_governance.alerts`、`ops_governance.chaos`、`ops_governance.hotswap`、`ops_governance.runbook`、`ops_governance.sandbox`、`ops_governance.snapshots` | 对应旧 handler 文件 | ops route facade 分开，不改 handler |
| `backend.app_state_wiring` | `app_state_wiring.health_route`、`app_state_wiring.state_factory` | `src/app_runtime_helpers.rs` | health/state factory 分开，不迁移 AppState owner |
| `backend.test_support` | `test_support.scenario` | `src/api_test_scenario.rs` | test scenario route 分开，不删除测试程序 |

---

## 等价边界

1. route 顺序保持不变。
2. handler 和 schema 全部保留原文件。
3. `AppState` 字段、锁顺序、runtime state、credential vault 存储格式均不迁移。
4. auth、storage lifecycle、safe log、backup 只登记在 `storage_security` 边界，未在本批拆实现。
5. 测试资产汰换仍未启动，不删除旧测试程序。

---

## closeout 结论

BE-001E 完成 `backend.strategy_config` 之外 8 个 L2 叶子的薄壳抽离。至此 backend 九叶都已经有父子 facade 坐标；下一步若继续推进，应该开始按单个子叶建立等价基线，而不是继续扩大横向批次。
