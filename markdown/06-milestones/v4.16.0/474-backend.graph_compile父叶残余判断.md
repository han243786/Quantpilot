# v4.16.0 backend.graph_compile 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FP-01
> 基线: `473-backend父叶残余判断.md`
> 目标父叶: `backend.graph_compile`
> 判定: `backend.graph_compile stop_split: false`
> 模块树坐标: `root.backend.graph_compile`
> 代码动作: no code movement
> 下一步: BE-001FQ-01 `backend.graph_compile.quantscript_graph` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FP-01 `backend.graph_compile` 父叶残余判断 | 进入后端 graph/compile 域 |
| 规范矩阵 | recursive residual judgment / shared helper first / route facade owner / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile` | 选择 graph compile 首个子叶 |
| 模块树 | `backend.graph_compile` | `backend.graph_compile stop_split: false` |

---

## 当前父叶状态

`backend.graph_compile` 当前只持有 route facade 聚合:

```text
src/backend/graph_compile.rs
src/backend/graph_compile/compile.rs
src/backend/graph_compile/graph.rs
src/backend/graph_compile/quantscript_graph.rs
```

三个 child facade 仍委托旧 owner:

```text
src/compile_api.rs
src/graph_api.rs
src/graph_quantscript_api.rs
```

因此父叶不能 closeout:

```text
backend.graph_compile stop_split: false
backend_graph_compile_residual_exists
```

---

## 当前残余复核

旧 owner 文件仍直接持有真实行为:

| 旧文件 | 当前职责 | 残余性质 |
| --- | --- | --- |
| `src/compile_api.rs` | runtime compile、strategy-ir compile、formal QS compile、compile cache、compile semaphore | handler / helper residual |
| `src/graph_api.rs` | graph save/list/load/delete/version/audit/reveal、artifact commit/rollback | persistence / handler residual |
| `src/graph_quantscript_api.rs` | graph QS load/parse、graph to QS generate、QS parse、target projection helper | shared helper / handler residual |

三份旧 owner 当前仍有 root parent wildcard import:

```text
src/compile_api.rs
src/graph_api.rs
src/graph_quantscript_api.rs
backend_graph_compile_parent_wildcard_residual_3
```

本批不改这些 import，只冻结下一步选择。

---

## 下一个子叶选择

本轮选择:

```text
backend_graph_compile_quantscript_graph_next_leaf_ready
BE-001FQ-01
backend.graph_compile.quantscript_graph
root.backend.graph_compile.quantscript_graph
```

先选择 `backend.graph_compile.quantscript_graph` 的原因:

1. `src/graph_quantscript_api.rs` 同时服务 graph route、compile route 与测试支撑。
2. `compile_runtime_protocol_via_qs` 依赖 `generate_quantscript_from_graph_value`、`parse_graph_quantscript_source` 与 `convert_graph_json_to_script_module`。
3. `graph_api.rs` 的 graph artifact/version 写入也依赖 QS artifact attach/generate helper。
4. 先冻结 shared QS graph helper 输入面，可以避免后续拆 `compile` 或 `graph` 时误造横向 sibling 连接。
5. 本叶拥有清晰 public helper / route handler 边界，适合先建等价基线再判断是否细拆。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不迁移 `src/graph_quantscript_api.rs`。
3. 不改 compile cache、compile semaphore、graph persistence、artifact commit/rollback 或 QS parse/lower 语义。
4. 不改 runtime 调用 `compile_runtime_protocol_via_qs` 的路径。
5. 不改 frontend caller、response schema、AppState 或 storage owner。
6. 不启动 release transition。
7. 不宣称 `backend.graph_compile`、`backend` 顶层或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FQ-01
backend.graph_compile.quantscript_graph
root.backend.graph_compile.quantscript_graph
```

BE-001FQ-01 只建立 `backend.graph_compile.quantscript_graph` 单子叶等价基线，冻结 route handler、shared helper、public helper 调用面、测试调用面和当前 parent wildcard residual。不得直接移动函数、改 import、拆 graph/compile sibling 或启动 release transition。

---

## 验证要求

本批是 `no code movement` 父叶残余判断，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## 幻觉检查点

AI 声称 BE-001FP-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `backend.graph_compile stop_split: false`。
3. `src/compile_api.rs`、`src/graph_api.rs` 与 `src/graph_quantscript_api.rs` 尚未迁移。
4. 下一步只能进入 BE-001FQ-01 `backend.graph_compile.quantscript_graph` 单子叶等价基线。
5. 不得宣称 compile / graph / quantscript graph handler 已迁移。
6. 不得宣称 `backend` 顶层或 Rust 重构完成。

---

## 验收标准

1. `474-backend.graph_compile父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend.graph_compile stop_split: false` 已记录。
3. 下一步固定为 BE-001FQ-01 `backend.graph_compile.quantscript_graph` 单子叶等价基线。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
