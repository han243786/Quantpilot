# v4.16.0 backend.runtime.routes 第四轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BV-01  
> 基准: `239-backend.runtime.routes.evidence单叶closeout.md`、`238-backend.runtime.routes.evidence抽离记录.md`、`235-backend.runtime.routes第三轮父叶残余判断.md`  
> 判定: `backend.runtime.routes` 第四轮父叶残余判断完成。run / backtest / mutation / experiment / evidence 五个 route child 已完成当前递归范围内 closeout；父叶仍直接持有 event_stream 与 report_ops route，因此继续保持 `stop_split: false`。下一步只能进入 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BV-01 route aggregate 父叶残余判断 | 父叶判断 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes` | 更新下一候选 |
| 模块树 | `backend.runtime.routes` | `stop_split: false` |

---

## 当前父叶状态

| 子叶 | 状态 | 结论 |
| --- | --- | --- |
| `backend.runtime.routes.run` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.backtest` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.mutation` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.experiment` | 已 closeout | `stop_split: true` |
| `backend.runtime.routes.evidence` | 已 closeout | `stop_split: true` |

当前 route aggregate file: `src/backend/runtime/routes.rs`。

已闭合 evidence route child file: `src/backend/runtime/routes/evidence.rs`。

父叶仍直接持有:

| 候选 | 当前 route | 判定 |
| --- | --- | --- |
| `backend.runtime.routes.event_stream` | `/api/runtime/runs/:run_id/events` -> `runtime_handlers::stream_run_events` | 下一候选 |
| `backend.runtime.routes.report_ops` | `/api/runtime/reports*`、`/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health`、`/api/v1/reports/*` | 后续候选 |

---

## 下一候选选择

选择 `backend.runtime.routes.event_stream`，理由:

1. route order 中 event_stream 位于 evidence 之前，抽离后可继续保持 `run -> event_stream -> evidence -> mutation` 的清晰顺序。
2. event stream 当前只有一条 route registration，适合做最小 route facade 抽离。
3. `runtime.event_stream` handler 层已在 BE-001L-04 完成单叶 closeout，route facade 抽离风险较低。
4. `backend.runtime.routes.report_ops` 覆盖 runtime reports、merge records、config generations、storage health、ops/audit/research reports，边界更宽，应在 event_stream route facade 收束后再单独处理。

---

## 非目标边界

BE-001BV-01 不创建文件、不移动代码、不迁移:

- `src/backend/runtime/routes/event_stream.rs`。
- `stream_run_events` handler。
- SSE frame / replay cursor / keep-alive 语义。
- report_ops route group。
- runtime reports handler。
- ops/audit/research report handler。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。

---

## 父子通信规则

已闭合子叶继续只能经父级:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.{run, backtest, evidence, mutation, experiment}
```

下一步 event_stream 只能先建立等价基线，不得直接创建 planned route child file。发布过渡前不得主动提出横向连接或性能旁路。

---

## 回归证据

本父叶判断继承上一批已通过验证:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BW-01 backend.runtime.routes.event_stream 单子叶等价基线
```

BE-001BW-01 只允许冻结 event stream route facade 边界，不得直接创建 `src/backend/runtime/routes/event_stream.rs`，不得迁移 handler、schema owner、`AppState`、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BV-01 完成时，必须说明 `backend.runtime.routes` 父叶仍是 `stop_split: false`，只是 evidence route child 已 closeout 并设置 `stop_split: true`。不得宣称 event_stream/report_ops route 已迁移、`backend.runtime.routes` 父叶完成、Rust backend 重构完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. 父叶明确保持 `stop_split: false`。
2. 下一候选固定为 BE-001BW-01 `backend.runtime.routes.event_stream` 单子叶等价基线。
3. `240-backend.runtime.routes第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 本批保持 `no code movement`。
