# v4.16.0 runtime.backtest.experiment_sweep 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001AB-01。  
> 基准: `124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md`、`123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`、`120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.experiment_sweep` 第三轮父叶残余判断完成。`parameter_grid`、`start_orchestration`、`record_lifecycle` 均已完成单叶 closeout 并设置 `stop_split: true`；父叶自身当前也设置 `stop_split: true`。下一步回到 `runtime.backtest` 上层队列，默认进入 BE-001AC-01 `runtime.backtest` 父叶残余判断。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AA 后回到 `experiment_sweep` 父叶残余判断 | 队列回流 |
| 规范矩阵 | 父叶停止细分、子叶关闭条件、父子通信规则 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 父叶残余判断 |
| 模块树 | `runtime.backtest.experiment_sweep` | 设置父叶停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 父模块 | `runtime.backtest` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 已完成子叶 | `runtime.backtest.experiment_sweep.parameter_grid`、`runtime.backtest.experiment_sweep.start_orchestration`、`runtime.backtest.experiment_sweep.record_lifecycle` |
| 已完成子叶文件 | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/record_lifecycle.rs` |
| 父叶 closeout 判定 | `stop_split: true` |
| 下一候选 | `runtime.backtest` 父叶残余判断 |
| 下一批次 | BE-001AC-01 |

---

## 保留 owner

| owner | 文件/节点 | 本批次处理 |
| --- | --- | --- |
| route owner | `src/backend/runtime/routes.rs` / `src/backend/runtime/routes/backtest.rs` | 保留原位 |
| runtime re-export | `src/runtime/mod.rs` | 保留原位 |
| drained parent include | `src/runtime/backtest.rs` | 保留原位 |
| experiment parent | `src/runtime/backtest/experiment_sweep.rs` | 只做残余判断 |
| parameter grid child | `src/runtime/backtest/parameter_grid.rs` | 已 closeout，`stop_split: true` |
| start orchestration child | `src/runtime/backtest/start_orchestration.rs` | 已 closeout，`stop_split: true` |
| record lifecycle child | `src/runtime/backtest/record_lifecycle.rs` | 已 closeout，`stop_split: true` |
| execution bridge | `src/runtime/backtest/execution_start.rs` | 保留父级内部复用桥 |
| persistence owner | `src/runtime_persistence.rs` | 保留原位 |
| response mapping owner | `src/runtime_response_mapping.rs` | 保留原位 |
| frontend schema owner | `src/frontend_api_types.rs` | 保留原位 |
| app state owner | `AppState` | 保留原位 |
| graph audit owner | `persist_graph_audit_entry` / `build_graph_audit_entry` | 保留原位 |

---

## 当前父叶结构

| 片段 | 真实职责 | 判定 |
| --- | --- | --- |
| `parameter_grid` | 参数网格校验、base fallback、去重、variant count、展开顺序 | 已 closeout，`stop_split: true` |
| `start_orchestration` | experiment 创建编排、guard、QS compile、variant request、`execute_backtest_request` 复用桥、preview persistence | 已 closeout，`stop_split: true` |
| `record_lifecycle` | experiment list/detail/save/discard、variant backtest 固化、transient cleanup、state cache、audit、response mapping 调用 | 已 closeout，`stop_split: true` |
| `experiment_sweep.rs` parent | 私有子模块声明与受控 `pub(crate) use` 兼容出口 | 当前设置 `stop_split: true` |

---

## 残余候选判断

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep.parameter_grid` | 停止 | BE-001W-04 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep.start_orchestration` | 停止 | BE-001Y-04 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 停止 | BE-001AA-04 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep.read_projection` | 不拆 | 已被 `record_lifecycle` 统一覆盖，继续拆会制造微文件 |
| `runtime.backtest.experiment_sweep.save_discard_lifecycle` | 不拆 | 已被 `record_lifecycle` 覆盖，save/discard 不应绕过 lifecycle 父子通信 |
| `runtime.backtest.experiment_sweep.persistence` | 不拆 | persistence helper 继续属于 `src/runtime_persistence.rs` |
| `runtime.backtest.experiment_sweep.response_projection` | 不拆 | response mapping owner 继续属于 `src/runtime_response_mapping.rs` |
| `runtime.backtest.experiment_sweep.audit` | 不拆 | graph audit helper 继续属于 audit/collaboration owner |
| `runtime.backtest.experiment_sweep.route_registration` | 不拆 | route 真实 owner 仍是 `src/backend/runtime/routes.rs` / backtest route facade |

---

## 父子通信规则

```text
backend.runtime.routes
  -> crate::runtime::{start_backtest_experiment,list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime.backtest.experiment_sweep
  -> parameter_grid / start_orchestration / record_lifecycle
  -> persistence / response mapping / graph audit / AppState
```

`runtime.backtest.experiment_sweep` 只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 experiment API。其内部三个子叶只能被父级私有调用，不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/runtime/backtest/experiment_sweep.rs` 代码。
- 不删除 `src/runtime/backtest.rs` drained parent include。
- 不迁移 experiment route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不私有化 persistence、response mapping、schema、AppState、audit、frontend caller 或测试资产。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 下一步

1. 回到 `runtime.backtest` 上层队列。
2. 默认下一批建立 BE-001AC-01 `runtime.backtest` 父叶残余判断。
3. BE-001AC-01 只能判断 execution_start、record_store、replay、experiment_sweep 当前是否都已完成允许范围内的递归收口；不得直接迁移 compare、artifact schema、state/persistence、schema、frontend caller 或 route owner。

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
git diff --check
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001AB-01 完成时，必须说明: 本批次只是 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断，`no code movement`；`parameter_grid`、`start_orchestration`、`record_lifecycle` 三个子叶已 closeout 并设置 `stop_split: true`；父叶自身现在也设置 `stop_split: true`。不得宣称 `runtime.backtest` 顶层已完成、route facade/schema/state/persistence/frontend caller 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断完成，并设置父叶 `stop_split: true`。
3. 下一候选固定为 `runtime.backtest` 父叶残余判断 / BE-001AC-01。
4. 治理门禁能发现本文档、`no code movement`、`stop_split: true`、下一候选、禁止迁移边界和回归证据缺失。
