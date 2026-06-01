# v4.16.0 backend.graph_compile.quantscript_graph 抽离记录
> 版本类型: MINOR architecture / implementation
> 执行档位: 标准
> 批次: BE-001FQ-03
> 基线: `476-backend.graph_compile.quantscript_graph抽离方案.md`
> 目标子叶: `backend.graph_compile.quantscript_graph`
> 判定: 实际抽离完成
> 模块树坐标: `root.backend.graph_compile.quantscript_graph`
> 代码动作: actual extraction
> 下一步: BE-001FQ-04 `backend.graph_compile.quantscript_graph` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FQ-03 `backend.graph_compile.quantscript_graph` 实际抽离记录 | 子叶实际抽离 |
| 规范矩阵 | actual extraction / root parent re-export surface / import rewrite / no sibling horizontal link / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph` | QS graph 真实 owner 落位 |
| 模块树 | `backend.graph_compile.quantscript_graph` | route/helper owner 迁移 |

---

## 实际迁移动作

本批执行:

```text
git mv -f src/graph_quantscript_api.rs src/backend/graph_compile/quantscript_graph.rs
```

迁移后:

```text
src/backend/graph_compile/quantscript_graph.rs
```

成为 QS graph route、graph-to-QS helper、QS parser helper、artifact attach helper 与 runtime target projection helper 的真实 owner。

旧 root owner 已删除:

```text
src/graph_quantscript_api.rs
```

---

## import rewrite

旧 parent wildcard import 已移除:

```text
use super::*;
```

迁移后使用 explicit crate imports:

```rust
use crate::{
    current_time_ms, internal_error, json_bad_request, not_found_io_error, validate_graph_id,
    AppState, CompileRuntimeTargets, ParseGraphQuantScriptRequest,
};
```

当前 parent wildcard residual:

```text
backend_graph_compile_quantscript_graph_parent_wildcard_residual_0
```

---

## route owner 改写

旧 route 函数:

```text
register_graph_quantscript_routes
```

已收敛为本叶 route entry:

```text
register_routes
```

父级仍通过:

```text
src/backend/graph_compile.rs
quantscript_graph::register_routes(router)
```

进行 route registration，不改变 `build_app_router` 的外部行为。

---

## root parent re-export surface

`src/lib.rs` 已移除旧 root module:

```text
mod graph_quantscript_api;
use graph_quantscript_api::*;
```

并新增受控 root parent re-export surface:

```text
pub(crate) use backend::graph_compile::quantscript_graph::{
    attach_quantscript_artifacts,
    build_compile_runtime_targets_from_graph,
    convert_graph_json_to_script_module,
    generate_quantscript_from_graph_value,
    parse_graph_quantscript_source,
};
```

`build_quantscript_runtime_targets` 当前无外部 caller，保留为本叶内部 helper，不进入 root re-export surface。

---

## caller adaptation

以下 caller 未改成 sibling 直连:

```text
src/compile_api.rs
src/graph_api.rs
src/runtime/run/session_start.rs
src/runtime/backtest/execution_start.rs
src/tests_backend.rs
```

它们继续经 crate root parent re-export surface 获得 helper 名称，避免 compile / graph / runtime 直接横向依赖 `backend.graph_compile.quantscript_graph`。

---

## 等价不变量

本批保持:

1. `GET /api/graphs/:graph_id/quantscript` 路径、`validate_graph_id`、`{graph_id}.qs` 读取和 not_found 映射不变。
2. `POST /api/quantscript/graph/parse` 路径、`bad_request` error shape 与中文错误消息不变。
3. `generate_quantscript_from_graph_value` 的 graph metadata、node、edge、intent module、connect 输出不变。
4. `convert_graph_json_to_script_module` 的 data/risk/execution/intent lowering 不变。
5. `attach_quantscript_artifacts` 的 formal source、runtime targets、node source targets、label targets 写入不变。
6. `build_compile_runtime_targets_from_graph` 的反序列化失败降级 warning 不变。
7. `parse_graph_quantscript_source` 的 strategy_graph header、nodes、graph connect parser 语义不变。

---

## 文档同步

本批同步:

```text
markdown/10-overview/overview-full-feature-tree.md
markdown/00-matrix-governance/module-tree.md
markdown/General_Policy.md
```

`overview-full-feature-tree.md` 不再引用已删除的 `src/graph_quantscript_api.rs` 作为当前文件；当前 owner 是 `src/backend/graph_compile/quantscript_graph.rs`。

---

## 不进入范围

本批不处理:

1. 不拆 `src/compile_api.rs` 或 `src/graph_api.rs` 的真实 handler。
2. 不改 compile cache、compile semaphore、graph persistence、version compare、artifact commit/rollback。
3. 不改 runtime run/backtest 的执行流程。
4. 不改 frontend caller、response schema、AppState、storage owner 或 release workflow。
5. 不启动 release transition。
6. 不宣称 `backend.graph_compile`、`backend` 顶层或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FQ-04
backend.graph_compile.quantscript_graph
root.backend.graph_compile.quantscript_graph
```

BE-001FQ-04 只允许做单叶 closeout 与是否继续细拆判断；不得跳过 closeout 直接进入 `backend.graph_compile.compile` 或 `backend.graph_compile.graph`。

---

## 验证要求

本批提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot graph_quantscript --lib
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

实际执行中 `cargo test -p quantpilot graph_quantscript --lib` 只完成编译且过滤结果为 0 tests，因此补跑 `cargo test -p quantpilot quantscript --lib`，覆盖 54 个 QuantScript / graph-QS 相关 lib tests。

---

## 幻觉检查点

AI 声称 BE-001FQ-03 完成时，必须说明:

1. `src/graph_quantscript_api.rs` 已删除，真实 owner 已迁入 `src/backend/graph_compile/quantscript_graph.rs`。
2. `backend_graph_compile_quantscript_graph_parent_wildcard_residual_0` 已成立。
3. caller adaptation 通过 root parent re-export surface 完成，没有新增 sibling horizontal link。
4. 本批不代表 `backend.graph_compile stop_split: true`，compile / graph sibling 仍未处理。
5. 下一步只能进入 BE-001FQ-04 单叶 closeout。

---

## 验收标准

1. `477-backend.graph_compile.quantscript_graph抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/backend/graph_compile/quantscript_graph.rs` 承接真实实现。
3. `src/graph_quantscript_api.rs` 不再作为当前活跃文件存在。
4. root parent re-export surface 已建立且 caller 不直连 sibling。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、narrow Rust tests 和 `git diff --check` 均通过。
