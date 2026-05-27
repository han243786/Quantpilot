# v4.16.0 backend.runtime 子叶抽离完成记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E-03。
> 基准: `34-backend.runtime单叶closeout.md`、`41-backend其余八叶模块壳抽离记录.md`。
> 判定: `backend.runtime` 子叶抽离完成；只建立 `backend.runtime.routes` facade，不迁移 runtime handler、state owner、event stream 或 persistence。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001E 逐叶完成 | 固化 |
| 规范矩阵 | runtime route facade、state owner 冻结 | 固化 |
| 引导矩阵 | `backend.runtime.routes` | 扩展 |
| 模块树 | `backend.runtime` | 子叶抽离完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 runtime |
| 模块树节点 | `backend.runtime`、`backend.runtime.routes` |
| 真实文件 | `src/backend/runtime.rs`、`src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs` |
| public 方法 | `register_routes`、`register_runtime_routes`、`/api/runtime/run`、`/api/runtime/backtest`、`/api/runtime/runs/:run_id/events` |
| 测试/门禁 | `cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`tools/check-matrix-governance.ps1` |

---

## 子叶抽离结果

| 子叶 | 职责 | 保留实现 |
| --- | --- | --- |
| `backend.runtime.routes` | runtime route aggregate facade | `src/runtime/mod.rs` |

## 等价结论

runtime run、backtest、AI proposal、SSE、persistence projection 和 runtime state owner 均保留原位。本批不迁移 runtime 内部模块。
