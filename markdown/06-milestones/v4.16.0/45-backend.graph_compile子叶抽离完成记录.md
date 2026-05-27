# v4.16.0 backend.graph_compile 子叶抽离完成记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E-04。
> 基准: `35-backend.graph_compile单叶closeout.md`、`41-backend其余八叶模块壳抽离记录.md`。
> 判定: `backend.graph_compile` 子叶抽离完成；只建立 compile、graph、quantscript graph 三个 route facade，不迁移 compile/graph handler 或 diagnostics。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001E 逐叶完成 | 固化 |
| 规范矩阵 | graph/compile/QS route 分界、diagnostics 保留 | 固化 |
| 引导矩阵 | `backend.graph_compile.compile`、`backend.graph_compile.graph`、`backend.graph_compile.quantscript_graph` | 扩展 |
| 模块树 | `backend.graph_compile` | 子叶抽离完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 graph compile |
| 模块树节点 | `backend.graph_compile`、`backend.graph_compile.compile`、`backend.graph_compile.graph`、`backend.graph_compile.quantscript_graph` |
| 真实文件 | `src/backend/graph_compile.rs`、`src/backend/graph_compile/compile.rs`、`src/backend/graph_compile/graph.rs`、`src/backend/graph_compile/quantscript_graph.rs`、`src/compile_api.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs` |
| public 方法 | `register_compile_routes`、`register_graph_routes`、`register_graph_quantscript_routes`、`/api/runtime/compile`、`/api/graph/*` |
| 测试/门禁 | `cargo test -p quantpilot --test api_graph_versions`、compile/graph tests、`tools/check-matrix-governance.ps1` |

---

## 子叶抽离结果

| 子叶 | 职责 | 保留实现 |
| --- | --- | --- |
| `backend.graph_compile.compile` | compile route facade | `src/compile_api.rs` |
| `backend.graph_compile.graph` | graph route facade | `src/graph_api.rs` |
| `backend.graph_compile.quantscript_graph` | QuantScript graph route facade | `src/graph_quantscript_api.rs` |

## 等价结论

graph version、compile diagnostics、QS safety boundary 和 response schema 均保持不变。本批不迁移 compile/graph handler。
