# v4.16.0 backend.runtime.routes.event_stream 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BW-04  
> 基准: `243-backend.runtime.routes.event_stream抽离记录.md`、`242-backend.runtime.routes.event_stream抽离方案.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes.event_stream` 单叶 closeout 完成。本叶只承接 `/api/runtime/runs/:run_id/events` GET route registration；继续拆成更小 route facade 不会形成新的稳定 owner，只会增加父级接线和治理碎片。因此本节点设置 `stop_split: true`。下一步只能回到 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BW-04 event stream route facade 单叶 closeout | 单叶收口 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.event_stream` | 关闭细分 |
| 模块树 | `backend.runtime.routes.event_stream` | `stop_split: true` |

---

## 当前白箱边界

| 项 | 当前 owner | 结论 |
| --- | --- | --- |
| route facade | `src/backend/runtime/routes/event_stream.rs` | 保持 |
| route | `/api/runtime/runs/:run_id/events` GET | 单 route，无继续拆分价值 |
| handler | `src/runtime/event_stream.rs` -> `stream_run_events` | 未迁移 |
| parent delegate | `src/backend/runtime/routes.rs` -> `event_stream::register_routes(router)` | 保持 |
| route order | `run -> event_stream -> evidence -> mutation` | 保持 |

---

## 细分价值判断

不继续细拆，理由:

1. 本叶只有一条 route registration，拆成更小 facade 不会形成新的模块 owner。
2. `stream_run_events` handler 已在 BE-001L-04 的 `runtime.event_stream` handler 层完成 closeout，route facade 不应重新拆 handler 内部职责。
3. SSE frame contract、keepalive contract、run record lookup 和 state owner 均属于 handler / runtime 层，不属于 route facade 层。
4. 继续拆只会增加 `backend.runtime.routes` 父级接线和治理碎片，不能提升解耦度。

---

## 保留边界

BE-001BW-04 不迁移、不修改:

- `src/backend/runtime/routes/event_stream.rs`。
- `src/backend/runtime/routes.rs`。
- `stream_run_events` handler body。
- `load_run_record_from_state`。
- `json_sse_event`。
- `SSE_EVENT_DELAY_MS`。
- `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")`。
- SSE event name、payload shape、frame order、delay 或 keepalive。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。
- report_ops route group。

---

## 父子通信规则

关闭后固定:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.event_stream
  -> crate::runtime::stream_run_events
```

`backend.runtime.routes.event_stream` 只作为 route facade。不得横向接管 `backend.runtime.routes.report_ops`、evidence、runtime report generation、frontend caller、runtime persistence owner 或 executor。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

---

## 回归证据

本 closeout 继承 BE-001BW-03 已通过的验证:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_sse
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
BE-001BX-01 backend.runtime.routes 第五轮父叶残余判断
```

不得从 event_stream route child 继续细拆；不得跳过父叶判断直接处理 report_ops；不得迁移 handler、SSE contract、`AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BW-04 完成时，必须说明只完成 event_stream route facade closeout 并设置 `stop_split: true`。不得宣称 report_ops 已处理、`backend.runtime.routes` 父叶完成、SSE handler 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `backend.runtime.routes.event_stream` 设置 `stop_split: true`。
2. `244-backend.runtime.routes.event_stream单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 下一步固定为 BE-001BX-01 `backend.runtime.routes` 第五轮父叶残余判断。
4. 本批保持 `no code movement`。
