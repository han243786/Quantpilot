# v4.16.0 backend.runtime 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-04。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.runtime` 当前完成 facade closeout，且明显值得继续细分；本批不迁移 runtime handler、record、artifact 或锁边界。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | runtime 叶子整理、下一轮 L3 候选 | 扩展 |
| 规范矩阵 | runtime state owner、事件、artifact、审批与 mutation 边界 | 固化 |
| 引导矩阵 | `backend.runtime`、runtime API tests | 扩展 |
| 模块树 | `backend.runtime` | 单叶 closeout 与继续细分登记 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 runtime 系统 |
| 模块树节点 | `backend.runtime` |
| 真实文件 | `src/backend/runtime.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`、`src/runtime_persistence.rs`、`src/runtime_event_projection.rs`、`src/runtime_response_mapping.rs`、`src/runtime_validation.rs`、`src/backtest_artifacts.rs` |
| public 方法 | `register_runtime_routes`、`/api/runtime/v4/run`、`/api/runtime/backtest`、`/api/runtime/runs/:run_id/events`、`build_backtest_artifact_views` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | runtime request、v4 graph/QS source、backtest request、mutation request |
| 输出 | run record、backtest record、artifact views、SSE event、AI proposal/mutation result |
| owner | `backend.runtime` 拥有 runtime API facade；record/state/lock 仍在既有实现 |
| 保留实现 | `src/runtime/*.rs`、`runtime_*`、`backtest_artifacts.rs` 均未迁移 |
| 兼容桥 | `backend.interface_boundary -> backend.runtime -> runtime::register_runtime_routes` |
| 回退点 | 回退到 `app_router` 直接调用 `register_runtime_routes` |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 值得继续拆分 |
| 原因 | run、backtest、mutation/approval、evidence/artifact、persistence/validation/projection 具备独立 owner、状态和测试证据 |
| 建议 L3 子叶 | `backend.runtime.run`、`backend.runtime.backtest`、`backend.runtime.mutation_approval`、`backend.runtime.evidence_artifact`、`backend.runtime.persistence_projection` |
| 暂停点 | 涉及 AppState 字段、锁顺序、artifact schema、SSE event 语义或旧测试汰换时必须重新提案 |

---

## closeout 结论

`backend.runtime` 已完成当前 facade 整理 closeout。它是九叶中最值得递归处理的叶子之一，下一步应先做 L3 等价基线，而不是直接搬 handler。
