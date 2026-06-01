# v4.16.0 backend 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FO-01
> 基线: `472-backend.runtime第十轮父叶残余判断.md`
> 目标父叶: `backend`
> 判定: `backend stop_split: false`
> 模块树坐标: `root.backend`
> 代码动作: no code movement
> 下一步: BE-001FP-01 `backend.graph_compile` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FO-01 `backend` 父叶残余判断 | 顶层回流 |
| 规范矩阵 | recursive residual judgment / top-level leaf selection / facade-first extraction / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend` | 选择下一个后端顶层叶子 |
| 模块树 | `backend` | `backend stop_split: false` |

---

## 当前父叶状态

`backend.runtime` 已完成当前阶段收口:

```text
backend.runtime stop_split: true
backend_runtime_parent_closeout_ready
```

`backend.interface_boundary` 当前仍是父级 route facade / bridge 层，不作为本轮继续拆分入口；它用于维持后端子叶经父级边界通信。

`backend` 顶层仍不能 closeout:

```text
backend stop_split: false
backend_top_level_residual_exists
```

原因是除 `backend.runtime` 外，仍有多个后端顶层叶子只是 facade 或 bridge，真实 handler / state / storage / governance owner 仍在旧文件或子域中:

```text
backend.graph_compile
backend.capability
backend.strategy_config
backend.storage_security
backend.ops_governance
backend.app_state_wiring
backend.test_support
```

---

## 下一个候选选择

本轮选择:

```text
backend_graph_compile_next_leaf_ready
BE-001FP-01
backend.graph_compile
root.backend.graph_compile
```

选择 `backend.graph_compile` 的原因:

1. `src/backend/graph_compile.rs` 已有稳定三子叶 facade: `compile`、`graph`、`quantscript_graph`。
2. 三个 child facade 当前只委托旧 owner: `src/compile_api.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs`。
3. 旧 owner 文件仍持有 route handler、helper、缓存、图版本读写与 QS graph 转换等真实残余。
4. 该域有明确 API 测试与回归边界，可用 `api_graph_versions`、compile/graph 相关测试逐步保护。
5. 相比 `app_state_wiring`、`storage_security`、`ops_governance`，它的 route / handler 边界更局部，更适合作为 runtime 后的下一个递归试点。

当前真实入口面:

```text
src/backend/graph_compile.rs
src/backend/graph_compile/compile.rs
src/backend/graph_compile/graph.rs
src/backend/graph_compile/quantscript_graph.rs
src/compile_api.rs
src/graph_api.rs
src/graph_quantscript_api.rs
```

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不迁移 compile / graph / quantscript graph handler。
3. 不改 graph version persistence、compile cache、QS graph parse/lower 语义。
4. 不改 `AppState`、storage owner、credential owner、ops governance owner 或 test support owner。
5. 不处理 frontend caller 或 response schema。
6. 不启动 release transition。
7. 不宣称 `backend` 顶层或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FP-01
backend.graph_compile
root.backend.graph_compile
```

BE-001FP-01 只负责对 `backend.graph_compile` 做父叶残余判断，确认它是否先进入 child route facade closeout，还是直接选择 `backend.graph_compile.compile` / `backend.graph_compile.graph` / `backend.graph_compile.quantscript_graph` 的单子叶等价基线。不得从本批直接改写 `src/compile_api.rs`、`src/graph_api.rs` 或 `src/graph_quantscript_api.rs`。

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

AI 声称 BE-001FO-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `backend stop_split: false`。
3. `backend.runtime stop_split: true` 已成立。
4. 下一步只能进入 BE-001FP-01 `backend.graph_compile` 父叶残余判断。
5. compile / graph / quantscript graph handler 尚未迁移。
6. `backend` 顶层与 Rust 重构均未完成。

---

## 验收标准

1. `473-backend父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend stop_split: false` 已记录。
3. 下一步固定为 BE-001FP-01 `backend.graph_compile` 父叶残余判断。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
