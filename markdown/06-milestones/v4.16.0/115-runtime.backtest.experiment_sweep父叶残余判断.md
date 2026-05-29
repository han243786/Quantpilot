# v4.16.0 runtime.backtest.experiment_sweep 父叶残余判断

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001X-01。  
> 基准: `114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.experiment_sweep` 父叶残余判断完成。`parameter_grid` 已关闭并设置 `stop_split: true`；父叶自身仍不设置 `stop_split: true`，下一步优先为 `runtime.backtest.experiment_sweep.start_orchestration` 建立单子叶等价基线，默认 BE-001Y-01。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001W 后回到 `experiment_sweep` 父叶残余判断 | 队列分流 |
| 规范矩阵 | 关闭 `parameter_grid`，保留父叶继续细分通道 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 父叶残余判断 |
| 模块树 | `runtime.backtest.experiment_sweep` | 确认下一内部候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 父模块 | `runtime.backtest` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 已完成子叶 | `runtime.backtest.experiment_sweep.parameter_grid` |
| 已完成子叶文件 | `src/runtime/backtest/parameter_grid.rs` |
| 下一候选 | `runtime.backtest.experiment_sweep.start_orchestration` |
| 下一批次 | BE-001Y-01 |

---

## 保留 owner

| owner | 文件/节点 | 本批次处理 |
| --- | --- | --- |
| route owner | `src/backend/runtime/routes.rs` | 保留原位 |
| runtime re-export | `src/runtime/mod.rs` | 保留原位 |
| drained parent include | `src/runtime/backtest.rs` | 保留原位 |
| experiment parent | `src/runtime/backtest/experiment_sweep.rs` | 只做残余判断 |
| parameter grid child | `src/runtime/backtest/parameter_grid.rs` | 已 closeout，`stop_split: true` |
| execution bridge | `src/runtime/backtest/execution_start.rs` | 保留父级内部复用桥 |
| persistence owner | `src/runtime_persistence.rs` | 保留原位 |
| response mapping owner | `src/runtime_response_mapping.rs` | 保留原位 |
| frontend schema owner | `src/frontend_api_types.rs` | 保留原位 |
| app state owner | `AppState` | 保留原位 |
| graph audit owner | `persist_graph_audit_entry` / `build_graph_audit_entry` | 保留原位 |

---

## 当前父叶结构

| 残余片段 | 真实职责 | 判定 |
| --- | --- | --- |
| `start_backtest_experiment` | capability/config guard、QS compile、base assumptions、variant request assembly、调用 `execute_backtest_request`、preview experiment persistence | 值得进入下一子叶基线 |
| `parameter_grid::build_experiment_overrides` | 参数网格校验、base fallback、去重、variant count、展开顺序 | 已 closeout，`stop_split: true` |
| `list_experiments` | 从 experiment store 读取、列表投影、created_at 倒序、分页 | 暂缓，后续与 lifecycle 一起判断 |
| `get_experiment_detail` | 用户作用域 record lookup 与 detail response mapping | 暂缓，后续与 lifecycle 一起判断 |
| `save_experiment_record` | variant backtest 持久化、transient 清理、experiment saved 状态、audit | 后续候选，先不动 |
| `discard_experiment_record` | saved conflict、experiment file/state 清理、transient variant 清理 | 后续候选，先不动 |

---

## 残余候选判断

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep.start_orchestration` | 值得继续，默认下一候选 | 创建路径是父叶剩余风险最高的编排段，集中接触 capability guard、QS compile、variant request、`execute_backtest_request` 复用桥、preview persistence 和 response assembly；先建等价基线能保护后续抽离 |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 后续候选 | list/detail/save/discard 共享 experiment persistence、variant backtest lifecycle、state cleanup 和 audit，但需要在 start orchestration 之后再确认是否拆为一个或多个子叶 |
| `runtime.backtest.experiment_sweep.read_projection` | 暂不单拆 | list/detail 与 persistence/response mapping owner 强绑定，单独拆可能制造微文件 |
| `runtime.backtest.experiment_sweep.save_discard_lifecycle` | 暂不单拆 | save/discard 与 backtest record persistence、transient cleanup、audit 和 state owner 强绑定，需后续重新基线 |
| `runtime.backtest.experiment_sweep.parameter_grid` | 停止 | BE-001W-04 已 closeout，`stop_split: true` |
| `runtime.backtest.experiment_sweep.persistence` | 不拆 | persistence helper 继续属于 `src/runtime_persistence.rs` |
| `runtime.backtest.experiment_sweep.response_projection` | 不拆 | `experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes` 属于 response mapping owner |
| `runtime.backtest.experiment_sweep.route_registration` | 不拆 | route 真实 owner 仍是 `src/backend/runtime/routes.rs` |

---

## 父子通信规则

```text
backend.runtime.routes
  -> crate::runtime::{start_backtest_experiment,list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime::backtest_experiment_sweep
  -> parameter_grid / execution_start bridge / persistence / response mapping / graph audit
```

`runtime.backtest.experiment_sweep` 只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 experiment API。后续 `start_orchestration` 子叶也只能被父级私有调用，不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/runtime/backtest/experiment_sweep.rs` 代码。
- 不新增 `src/runtime/backtest/experiment_start.rs` 或其他子文件。
- 不删除 `src/runtime/backtest.rs` drained parent include。
- 不迁移 experiment route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不私有化 persistence、response mapping、schema、AppState、audit、frontend caller 或测试资产。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 下一步

1. 进入 BE-001Y-01 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线。
2. BE-001Y-01 只能冻结 `start_backtest_experiment` 的输入输出、capability/config guard、QS compile、variant request assembly、`execute_backtest_request` 复用桥、preview persistence 和排除边界。
3. BE-001Y-01 仍然是 `no code movement`；真实文件名和物理抽离方式只能在后续抽离方案中确认。

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

AI 声称 BE-001X-01 完成时，必须说明: 本批次只是 `runtime.backtest.experiment_sweep` 父叶残余判断，`no code movement`；`parameter_grid` 已 closeout 并设置 `stop_split: true`，但父叶自身仍 `stop_split: false`。不得宣称 start orchestration 已抽离、record lifecycle 已抽离、route facade 已迁移、schema/state/persistence/frontend caller 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `115-runtime.backtest.experiment_sweep父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `runtime.backtest.experiment_sweep` 父叶残余判断完成，但不宣称父叶最终关闭。
3. 下一候选固定为 `runtime.backtest.experiment_sweep.start_orchestration` / BE-001Y-01。
4. 治理门禁能发现本残余判断文档、`no code movement`、下一候选、禁止迁移边界和回归证据缺失。
