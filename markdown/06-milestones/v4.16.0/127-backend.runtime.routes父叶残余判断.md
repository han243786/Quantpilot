# v4.16.0 backend.runtime.routes 父叶残余判断

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001AD-01。  
> 基准: `126-runtime.backtest父叶残余判断.md`、`77-runtime.backtest单叶closeout.md`、`73-runtime.event_stream单叶closeout.md`、`54-backend.runtime.routes.run单叶closeout.md`、`51-backend.runtime.routes抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes` 父叶残余判断完成。`run`、`event_stream`、`backtest` 相关递归链路均已完成当前范围内 closeout；但 route aggregate 仍直接持有 mutation / AI proposal / approval / evidence / report / experiment / ops 等路线，因此父叶当前保持 `stop_split: false`。下一步进入 BE-001AE-01 `backend.runtime.routes.mutation` 单子叶等价基线。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AC 后回到 `backend.runtime.routes` 父叶残余判断 | 队列回流 |
| 规范矩阵 | 父叶不停止、候选按剩余耦合度排序、禁止顺手迁移 handler | 固化 |
| 引导矩阵 | `root.backend.runtime.routes` | 父叶残余判断 |
| 模块树 | `backend.runtime.routes` | 保持 `stop_split: false` 并登记下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `backend.runtime.routes` |
| 父模块 | `backend.runtime` |
| route aggregate | `src/backend/runtime/routes.rs` |
| 已完成 route 子叶 | `backend.runtime.routes.run`、`backend.runtime.routes.backtest` |
| 已完成 handler 子叶 | `runtime.run.*`、`runtime.event_stream`、`runtime.backtest.*` |
| 父叶 closeout 判定 | `stop_split: false` |
| 下一候选 | `backend.runtime.routes.mutation` 单子叶等价基线 |
| 下一批次 | BE-001AE-01 |

---

## 保留 owner

| owner | 文件/节点 | 本批次处理 |
| --- | --- | --- |
| runtime route aggregate | `src/backend/runtime/routes.rs` | 保留原位 |
| run route facade | `src/backend/runtime/routes/run.rs` | 已 closeout，保留原位 |
| backtest route facade | `src/backend/runtime/routes/backtest.rs` | 已 closeout，保留原位 |
| runtime mutation handler owner | `src/runtime/mutation.rs` | 保留原位，下一批只建 route 等价基线 |
| runtime report/evidence/ops owner | `src/runtime/mod.rs` | 保留原位 |
| runtime handler facade | `src/runtime/mod.rs` | 保留原位 |
| runtime run parent include | `src/runtime/run.rs` | 保留原位 |
| runtime backtest drained parent include | `src/runtime/backtest.rs` | 保留原位 |
| app state owner | `AppState` | 保留原位 |

---

## 当前父叶结构

| 片段 | 真实职责 | 判定 |
| --- | --- | --- |
| `backend.runtime.routes.run` | run/v4 run/list/detail/save/replay/status route facade | 已 closeout |
| `runtime.event_stream` | SSE handler，route 仍由 aggregate 直接注册 | handler 已 closeout；route facade cleanup 非下一优先级 |
| `backend.runtime.routes.backtest` | backtest route facade | 已 closeout |
| `runtime.backtest` | backtest handler 域 | 已完成父叶残余判断，`stop_split: true` |
| mutation / AI proposal / approval routes | runtime mutation、proposal、approval、claim/approve/reject route group | 值得作为下一候选 |
| evidence / report / experiment / ops routes | evidence health、reports、experiment routes、ops/audit/research reports、storage/config health | 保留为后续候选 |

---

## 残余候选判断

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| `backend.runtime.routes.run` | 停止 | route facade 与 run handler 链路均已完成当前 closeout |
| `runtime.event_stream` | 暂不继续 | handler 已完成 closeout；单 SSE route 可后续 cleanup，不是最大耦合 |
| `backend.runtime.routes.backtest` / `runtime.backtest` | 停止 | route facade 与 handler 父叶均已 closeout |
| `backend.runtime.routes.experiment` | 暂缓 | handler 已收在 `runtime.backtest.experiment_sweep`；route cleanup 可在 mutation 后处理 |
| `backend.runtime.routes.mutation` | 下一候选 | 对应 `src/runtime/mutation.rs` 体量最大，集中 mutation、AI proposal、approval、claim/approve/reject 与锁顺序风险 |
| `backend.runtime.routes.evidence` | 后续候选 | evidence health/cleanup 体量较小，先保留 aggregate owner |
| `backend.runtime.routes.report_ops` | 后续候选 | report、ops/audit/research、storage/config health 仍需单独基线 |

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.{run,backtest}
  -> runtime handlers

backend.runtime
  -> backend.runtime.routes
  -> planned backend.runtime.routes.mutation
  -> runtime.mutation handlers
```

`backend.runtime.routes` 只能经父级 `backend.runtime` 与 `backend.interface_boundary` 暴露 runtime API。BE-001AE-01 之前不得直接新建 mutation route 子文件、不得移动 `src/runtime/mutation.rs` handler、不得改变 approval lock order、不得修改 `AppState`、response schema、evidence schema、frontend caller 或发布过渡连接。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/backend/runtime/routes.rs` 中任何 route。
- 不新建 `src/backend/runtime/routes/mutation.rs`。
- 不迁移 `src/runtime/mutation.rs` 中任何 handler/helper。
- 不迁移 report、evidence、experiment、ops、storage health、config generation 或 merge record routes。
- 不修改 `AppState`、锁顺序、schema、frontend caller 或测试资产。
- 不启动发布过渡，不提出横向连接优化。

---

## 下一步

1. 建立 BE-001AE-01 `backend.runtime.routes.mutation` 单子叶等价基线。
2. 该基线只冻结 mutation / AI proposal / approval route group 的输入输出、path/method、handler owner、lock order、测试证据和排除边界。
3. BE-001AE-01 不得移动代码；若后续进入抽离方案，仍必须先经适配性校验和方案优化。

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
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 幻觉检查点

AI 声称 BE-001AD-01 完成时，必须说明: 本批次只是 `backend.runtime.routes` 父叶残余判断，且为 `no code movement`；父叶仍是 `stop_split: false`；下一候选是 BE-001AE-01 `backend.runtime.routes.mutation` 单子叶等价基线。不得宣称 mutation route 已抽离、`src/runtime/mutation.rs` 已拆分、report/evidence/experiment/ops 已迁移、`AppState` 或锁顺序已改、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `127-backend.runtime.routes父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `backend.runtime.routes` 父叶仍为 `stop_split: false`，且下一候选固定为 `backend.runtime.routes.mutation` / BE-001AE-01。
3. 治理门禁能发现本文档、`no code movement`、`stop_split: false`、下一候选、禁止迁移边界和回归证据缺失。
