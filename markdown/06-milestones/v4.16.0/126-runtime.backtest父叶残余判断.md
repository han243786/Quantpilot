# v4.16.0 runtime.backtest 父叶残余判断

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001AC-01。  
> 基准: `125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`、`106-runtime.backtest.replay单叶closeout.md`、`102-runtime.backtest.record_store单叶closeout.md`、`98-runtime.backtest.execution_start父叶残余判断.md`、`77-runtime.backtest单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest` 父叶残余判断完成。`execution_start`、`record_store`、`replay`、`experiment_sweep` 均已完成当前递归范围内的 closeout；`runtime.backtest` 父叶当前设置 `stop_split: true`。下一步回到 `backend.runtime.routes` 上层队列，默认进入 BE-001AD-01 `backend.runtime.routes` 父叶残余判断。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AB 后回到 `runtime.backtest` 父叶残余判断 | 队列回流 |
| 规范矩阵 | 父叶停止细分、保留 owner、禁止发布过渡偷渡 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest` | 父叶残余判断 |
| 模块树 | `runtime.backtest` | 设置父叶停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest` |
| 父模块 | `backend.runtime.routes` |
| route facade | `src/backend/runtime/routes/backtest.rs` |
| drained parent include | `src/runtime/backtest.rs` |
| runtime facade | `src/runtime/mod.rs` |
| 已完成子叶 | `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`runtime.backtest.experiment_sweep` |
| 父叶 closeout 判定 | `stop_split: true` |
| 下一候选 | `backend.runtime.routes` 父叶残余判断 |
| 下一批次 | BE-001AD-01 |

---

## 保留 owner

| owner | 文件/节点 | 本批次处理 |
| --- | --- | --- |
| runtime route aggregate | `src/backend/runtime/routes.rs` | 保留原位 |
| backtest route facade | `src/backend/runtime/routes/backtest.rs` | 保留原位 |
| runtime facade | `src/runtime/mod.rs` | 保留原位 |
| drained parent include | `src/runtime/backtest.rs` | 保留原位，不在本批删除 |
| execution start | `src/runtime/backtest/execution_start.rs` | 已完成父叶残余判断 |
| record store | `src/runtime/backtest/record_store.rs` | 已 closeout，`stop_split: true` |
| replay | `src/runtime/backtest/replay.rs` | 已 closeout，`stop_split: true` |
| experiment sweep | `src/runtime/backtest/experiment_sweep.rs` | 已完成父叶残余判断，`stop_split: true` |
| compare owner | `src/backtest_compare.rs` | 保留原位 |
| artifact schema owner | `src/backtest_artifacts.rs` | 保留原位 |
| persistence owner | `src/runtime_persistence.rs` | 保留原位 |
| response mapping owner | `src/runtime_response_mapping.rs` | 保留原位 |
| frontend schema owner | `src/frontend_api_types.rs` | 保留原位 |
| app state owner | `AppState` | 保留原位 |

---

## 当前父叶结构

| 片段 | 真实职责 | 判定 |
| --- | --- | --- |
| `backend.runtime.routes.backtest` | backtest route registration facade | 已 closeout，`stop_split: true` |
| `runtime.backtest.execution_start` | backtest 创建路径、v4/legacy 分流、record assembly、transient spill、audit | 已完成父叶残余判断 |
| `runtime.backtest.record_store` | backtest list/detail/save/discard | 已 closeout，`stop_split: true` |
| `runtime.backtest.replay` | backtest replay handler | 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep` | experiment sweep 创建、列表、详情、保存、丢弃 | 已完成父叶残余判断，`stop_split: true` |
| `src/runtime/backtest.rs` | drained parent include placeholder | 本批不删除 |

---

## 残余候选判断

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| `runtime.backtest.execution_start.record_finalize` | 不拆 | BE-001S-01 已判定不从 execution_start 私拆 record/state/persistence 边界 |
| `runtime.backtest.record_store` | 停止 | BE-001T-04 已 closeout，`stop_split: true` |
| `runtime.backtest.replay` | 停止 | BE-001U-04 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep` | 停止 | BE-001AB-01 已完成第三轮父叶残余判断，`stop_split: true` |
| `runtime.backtest.compare` | 不纳入本父叶 | compare owner 仍是 `src/backtest_compare.rs`，不得为了完成 backtest 父叶而迁移 |
| `runtime.backtest.artifact_schema` | 不纳入本父叶 | artifact schema owner 仍是 `src/backtest_artifacts.rs` |
| `runtime.backtest.persistence` | 不纳入本父叶 | persistence owner 仍是 `src/runtime_persistence.rs` |
| `runtime.backtest.response_projection` | 不纳入本父叶 | response mapping owner 仍是 `src/runtime_response_mapping.rs` |
| `runtime.backtest.frontend_caller` | 不纳入本父叶 | frontend caller 和 schema owner 仍保留原 owner |
| `runtime.backtest.drained_parent_cleanup` | 暂不执行 | 删除 drained include 属于后续物理 cleanup；本批只做父叶残余判断 |

---

## 父子通信规则

```text
backend.runtime.routes
  -> backend.runtime.routes.backtest
  -> crate::runtime::{start_backtest_run,list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record,get_backtest_replay}
  -> runtime.backtest.{execution_start,record_store,replay}
  -> persistence / response mapping / artifact schema / AppState

backend.runtime.routes
  -> crate::runtime::{start_backtest_experiment,list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime.backtest.experiment_sweep
```

`runtime.backtest` 只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 backtest/experiment API。不得让 `record_store`、`replay`、`experiment_sweep`、`backtest_compare`、persistence owner、response mapping owner、schema owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动或删除 `src/runtime/backtest.rs`。
- 不迁移 `src/backend/runtime/routes/backtest.rs`。
- 不迁移 `src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 或 `AppState`。
- 不把 report、mutation、evidence、approval、ops routes 混入本父叶。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 下一步

1. 回到 `backend.runtime.routes` 上层队列。
2. 默认下一批建立 BE-001AD-01 `backend.runtime.routes` 父叶残余判断。
3. BE-001AD-01 只能判断 runtime route aggregate 中 run、event_stream、backtest/experiment、report、mutation、evidence、approval、ops 当前是否仍存在值得递归的候选；不得直接迁移代码。

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
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001AC-01 完成时，必须说明: 本批次只是 `runtime.backtest` 父叶残余判断，`no code movement`；`execution_start`、`record_store`、`replay`、`experiment_sweep` 已完成当前递归范围内 closeout；`runtime.backtest` 父叶现在设置 `stop_split: true`。不得宣称 `backend.runtime.routes` 顶层已完成、drained parent include 已删除、compare/artifact schema/persistence/response mapping/frontend caller 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `126-runtime.backtest父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `runtime.backtest` 父叶残余判断完成，并设置父叶 `stop_split: true`。
3. 下一候选固定为 `backend.runtime.routes` 父叶残余判断 / BE-001AD-01。
4. 治理门禁能发现本文档、`no code movement`、`stop_split: true`、下一候选、禁止迁移边界和回归证据缺失。
