# v4.16.0 backend.runtime.routes 第六轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BZ-01  
> 基准: `249-backend.runtime.routes.report_ops单叶closeout.md`、`248-backend.runtime.routes.report_ops抽离记录.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes` 第六轮父叶残余判断完成。run / backtest / mutation / experiment / evidence / event_stream / report_ops 七个 route child 已完成当前递归范围内 closeout；父叶不再直接持有 route registration，只作为 `backend.runtime -> backend.runtime.routes -> route child -> runtime handler` 的聚合 facade。因此本节点设置 `stop_split: true`。下一步只能回到 BE-001CA-01 `backend.runtime` 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BZ-01 route aggregate 父叶残余判断 | 父叶收口 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes` | 关闭 route aggregate 细分 |
| 模块树 | `backend.runtime.routes` | `stop_split: true` |

---

## 当前父叶状态

| 子叶 | 状态 | 结论 |
| --- | --- | --- |
| `backend.runtime.routes.run` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.backtest` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.mutation` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.experiment` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.evidence` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.event_stream` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.report_ops` | 已 closeout | `stop_split: true` |

当前 route aggregate file: `src/backend/runtime/routes.rs`。

已闭合 route child files:

- `src/backend/runtime/routes/run.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/backend/runtime/routes/experiment.rs`
- `src/backend/runtime/routes/evidence.rs`
- `src/backend/runtime/routes/event_stream.rs`
- `src/backend/runtime/routes/report_ops.rs`

---

## 残余判断

`src/backend/runtime/routes.rs` 当前只保留:

1. `MODULE_ID`。
2. 七个 route child module declaration。
3. `register_routes(router)` 中的父级委托顺序。

已不再直接注册任何 route path，也不拥有 runtime handler、schema owner、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

结论:

```text
backend.runtime.routes stop_split: true
```

继续拆本父叶会把单纯的委托顺序拆成更小 wiring facade，不会产生新的稳定 owner，也会增加父级接线和治理碎片。

---

## 保留 owner

| owner | 文件/节点 | 本批次处理 |
| --- | --- | --- |
| runtime route aggregate | `src/backend/runtime/routes.rs` | 设置父叶 stop_split: true |
| run route facade | `src/backend/runtime/routes/run.rs` | 已 closeout |
| backtest route facade | `src/backend/runtime/routes/backtest.rs` | 已 closeout |
| mutation route facade | `src/backend/runtime/routes/mutation.rs` | 已 closeout |
| experiment route facade | `src/backend/runtime/routes/experiment.rs` | 已 closeout |
| evidence route facade | `src/backend/runtime/routes/evidence.rs` | 已 closeout |
| event stream route facade | `src/backend/runtime/routes/event_stream.rs` | 已 closeout |
| report ops route facade | `src/backend/runtime/routes/report_ops.rs` | 已 closeout |
| runtime handlers | `src/runtime/*` | 保留原 owner |
| app state owner | `AppState` | 保留原位 |

---

## 父子通信规则

固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.{run, backtest, event_stream, evidence, mutation, experiment, report_ops}
  -> crate::runtime::{handlers}
```

`backend.runtime.routes` 只能经父级 `backend.runtime` 与 `backend.interface_boundary` 暴露 runtime API。后续若继续拆 runtime handler、schema owner、state owner、runtime persistence owner、storage lifecycle owner 或 frontend caller，必须回到上级 `backend.runtime` 父叶残余判断重新选择候选。发布过渡前不得主动提出横向连接或性能旁路。

---

## 回归证据

本父叶判断继承 BE-001BY-03 / BE-001BY-04 已通过验证:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001CA-01 backend.runtime 父叶残余判断
```

BE-001CA-01 应从上一级确认 `backend.runtime.routes` 已设置 `stop_split: true`，并重新判断 runtime handler、schema、state/persistence、storage lifecycle、frontend caller 等剩余 owner 是否值得另起候选。不得从 `backend.runtime.routes` 内继续细拆，也不得启动发布过渡。

---

## 幻觉检查点

AI 声称 BE-001BZ-01 完成时，必须说明 `backend.runtime.routes` 只完成 route aggregate 父叶收口并设置 `stop_split: true`；runtime handlers、schema owner、`AppState`、runtime persistence owner、storage lifecycle owner 和 frontend caller 均未迁移。不得宣称 `backend.runtime` 父叶完成、backend 顶层完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. 父叶明确设置 `backend.runtime.routes stop_split: true`。
2. 下一步固定为 BE-001CA-01 `backend.runtime` 父叶残余判断。
3. `250-backend.runtime.routes第六轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 本批保持 `no code movement`。
