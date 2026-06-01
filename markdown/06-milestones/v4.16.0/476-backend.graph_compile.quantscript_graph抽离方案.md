# v4.16.0 backend.graph_compile.quantscript_graph 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FQ-02
> 基线: `475-backend.graph_compile.quantscript_graph单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph`
> 代码动作: no code movement
> 下一步: BE-001FQ-03 `backend.graph_compile.quantscript_graph` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FQ-02 `backend.graph_compile.quantscript_graph` 抽离方案 | 子叶抽离方案 |
| 规范矩阵 | planned move / import rewrite / root parent re-export / caller adaptation / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph` | 固定真实实现迁移路径 |
| 模块树 | `backend.graph_compile.quantscript_graph` | 实际抽离前置方案 |

---

## 方案目标

BE-001FQ-03 的实际抽离目标是把 `src/graph_quantscript_api.rs` 的真实实现迁入:

```text
src/backend/graph_compile/quantscript_graph.rs
```

迁移后 `backend.graph_compile.quantscript_graph` 应成为 QS graph route、graph-to-QS helper、QS parser helper、artifact attach helper 与 runtime target projection helper 的真实 owner。

本批不移动代码，只固定下一步可执行方案。

---

## planned move

BE-001FQ-03 应执行以下 planned move:

1. 用 `src/graph_quantscript_api.rs` 的真实实现替换当前 `src/backend/graph_compile/quantscript_graph.rs` 薄壳 facade。
2. 将原 `register_graph_quantscript_routes` 收敛为本叶 public route entry:

```text
pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

3. 删除或清空旧 root owner `src/graph_quantscript_api.rs` 的真实逻辑；优先直接删除旧文件，并同步移除 `src/lib.rs` 中的:

```text
mod graph_quantscript_api;
use graph_quantscript_api::*;
```

4. 在 `src/lib.rs` 通过 root parent re-export surface 保持旧 caller 可见性:

```text
pub(crate) use backend::graph_compile::quantscript_graph::{
    attach_quantscript_artifacts,
    build_compile_runtime_targets_from_graph,
    build_quantscript_runtime_targets,
    convert_graph_json_to_script_module,
    generate_quantscript_from_graph_value,
    parse_graph_quantscript_source,
};
```

---

## import rewrite

迁移后不得继续使用:

```text
use super::*;
```

BE-001FQ-03 应把 parent wildcard import 改成显式 crate imports，并保留 `explicit crate imports` 检查标记。预期 import pocket:

```rust
use crate::{
    current_time_ms, internal_error, json_bad_request, not_found_io_error, validate_graph_id,
    AppState, CompileRuntimeTargets, ParseGraphQuantScriptRequest,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use quantscript::{parse_quant_script_module, ScriptModule};
use serde_json::Value;
use tokio::fs;
```

`safe_eprintln!` 仍由 crate root macro 提供，不新增工具层依赖。

---

## visibility rewrite

迁移时函数可见性应按白箱边界重写:

| 函数 | 迁移后可见性 | 原因 |
| --- | --- | --- |
| `register_routes` | `pub(crate)` | 由 `backend.graph_compile` 父级 route facade 调用 |
| `load_graph_quantscript` | private | 只由本叶 route 注册使用 |
| `parse_graph_quantscript` | private | 只由本叶 route 注册使用 |
| `generate_quantscript_from_graph_value` | `pub(crate)` | compile / graph / tests 仍需经 root parent re-export 调用 |
| `convert_graph_json_to_script_module` | `pub(crate)` | compile pipeline 仍需调用 |
| `attach_quantscript_artifacts` | `pub(crate)` | graph save/version 与 tests 仍需调用 |
| `build_quantscript_runtime_targets` | `pub(crate)` | artifact attach 和测试证据面保留 |
| `build_compile_runtime_targets_from_graph` | `pub(crate)` | runtime run/backtest 与 compile 仍需调用 |
| `parse_graph_quantscript_source` | `pub(crate)` | route、compile 与 tests 仍需调用 |
| private parser/generator helpers | private | 不暴露给 sibling |

---

## caller adaptation

BE-001FQ-03 不应让 sibling 直接横连:

```text
src/compile_api.rs -> backend::graph_compile::quantscript_graph
src/graph_api.rs -> backend::graph_compile::quantscript_graph
src/runtime/run/session_start.rs -> backend::graph_compile::quantscript_graph
src/runtime/backtest/execution_start.rs -> backend::graph_compile::quantscript_graph
src/tests_backend.rs -> backend::graph_compile::quantscript_graph
```

这些 caller 继续通过 crate root parent re-export surface 获得旧 helper 名称。这样可以保持 `use super::*` 的等价面，同时避免 compile / graph / runtime 直接绕过父级边界。

允许修改 `src/lib.rs` 的 module declaration 与 re-export；不允许在本批启动 release transition，也不允许为了性能建立 sibling horizontal link。

---

## 等价不变量

BE-001FQ-03 必须保持:

1. `GET /api/graphs/:graph_id/quantscript` route path、`validate_graph_id`、`{graph_id}.qs` 读取路径与 not_found 映射不变。
2. `POST /api/quantscript/graph/parse` route path、`bad_request` error shape 与中文错误消息不变。
3. `generate_quantscript_from_graph_value` 的 graph metadata、node、edge、intent module、connect 输出不变。
4. `convert_graph_json_to_script_module` 的 data/risk/execution/intent lowering 不变。
5. `attach_quantscript_artifacts` 的 formal source、runtime targets、node source targets、label targets 写入不变。
6. `build_compile_runtime_targets_from_graph` 的反序列化失败降级 warning 不变。
7. `parse_graph_quantscript_source` 的 strategy_graph header、nodes、graph connect parser 语义不变。

---

## 不进入范围

BE-001FQ-03 不处理:

1. 不拆 `compile_api.rs` 或 `graph_api.rs` 的真实 handler。
2. 不改 compile cache、compile semaphore、graph persistence、version compare、artifact commit/rollback。
3. 不改 runtime run/backtest 的执行流程。
4. 不改 frontend caller、response schema、AppState、storage owner 或 release workflow。
5. 不启动 release transition。
6. 不宣称 `backend.graph_compile`、`backend` 顶层或 Rust 重构完成。

---

## BE-001FQ-03 验证要求

实际抽离提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot graph_quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

若 BE-001FQ-03 改动触及 route registration 或 graph persistence，额外执行对应 API tests；若只完成 planned move 且 narrow tests 通过，可以不跑全量 frontend。

---

## 幻觉检查点

AI 声称 BE-001FQ-02 完成时，必须说明:

1. 本批是 `no code movement` 抽离方案。
2. `src/graph_quantscript_api.rs` 仍未迁移。
3. 下一步只能进入 BE-001FQ-03 实际抽离记录。
4. BE-001FQ-03 的调用适配必须通过 root parent re-export surface，不能新增 sibling horizontal link。
5. 不得宣称 `backend.graph_compile.quantscript_graph` 已实际抽离。

---

## 验收标准

1. `476-backend.graph_compile.quantscript_graph抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. planned move、import rewrite、root parent re-export surface、caller adaptation 和验证要求已冻结。
3. 下一步固定为 BE-001FQ-03 实际抽离记录。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
