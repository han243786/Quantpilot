# v4.16.0 backend.graph_compile.quantscript_graph 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FQ-01
> 基线: `474-backend.graph_compile父叶残余判断.md`
> 目标子叶: `backend.graph_compile.quantscript_graph`
> 判定: 等价基线
> 模块树坐标: `root.backend.graph_compile.quantscript_graph`
> 代码动作: no code movement
> 下一步: BE-001FQ-02 `backend.graph_compile.quantscript_graph` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FQ-01 `backend.graph_compile.quantscript_graph` 单子叶等价基线 | 子叶基线 |
| 规范矩阵 | equivalence baseline / shared helper surface / route handler freeze / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph` | 冻结 QS graph 白箱输入输出 |
| 模块树 | `backend.graph_compile.quantscript_graph` | 等价基线 |

---

## 当前真实文件

当前 facade:

```text
src/backend/graph_compile/quantscript_graph.rs
```

当前旧 owner:

```text
src/graph_quantscript_api.rs
```

当前旧 owner 仍保留:

```text
use super::*;
backend_graph_compile_quantscript_graph_parent_wildcard_residual_1
```

本批不改 Rust，只冻结等价边界。

---

## route 输入面

`register_graph_quantscript_routes` 当前注册两条 route:

```text
GET  /api/graphs/:graph_id/quantscript
POST /api/quantscript/graph/parse
```

handler 边界:

| handler | 输入 | 输出 | 不变量 |
| --- | --- | --- | --- |
| `load_graph_quantscript` | `AppState.graph_store_dir` + `graph_id` | QS source text | 保留 `validate_graph_id`、`{graph_id}.qs` 路径和 not_found 映射 |
| `parse_graph_quantscript` | `ParseGraphQuantScriptRequest.source` | graph JSON | 保留 `bad_request` error shape 与中文错误消息 |

---

## shared helper 输入面

`backend.graph_compile.quantscript_graph` 不是单纯 route facade，它还承担 graph / compile / runtime 共用 helper:

| public/helper | 当前调用方 | 等价不变量 |
| --- | --- | --- |
| `generate_quantscript_from_graph_value` | `compile_api.rs`、`graph_api.rs`、`tests_backend.rs` | graph metadata、node/edge、intent module、connect 输出保持一致 |
| `convert_graph_json_to_script_module` | `compile_api.rs` | data/risk/execution/intent lowering 输入保持一致 |
| `attach_quantscript_artifacts` | `graph_api.rs`、`tests_backend.rs` | formal source、runtime targets、node source targets、label targets 写入保持一致 |
| `build_quantscript_runtime_targets` | `attach_quantscript_artifacts`、测试 | source_to_node、runtime_node_id、execution_node_id 映射保持一致 |
| `build_compile_runtime_targets_from_graph` | `compile_api.rs`、`src/runtime/run/session_start.rs`、`src/runtime/backtest/execution_start.rs` | 反序列化失败降级与 warning 行为保持一致 |
| `parse_graph_quantscript_source` | route handler、`compile_api.rs`、`tests_backend.rs` | strategy_graph header、nodes、graph connect parser 语义保持一致 |

这些 helper 是后续抽离的高风险输入面，BE-001FQ-02 必须优先设计 re-export / caller 适配，不能让 compile、graph、runtime 直接形成 sibling horizontal link。

---

## 测试与证据面

当前直接证据:

```text
src/tests_backend.rs
attach_quantscript_artifacts_preserves_node_source_targets
attach_quantscript_artifacts_preserves_formal_source
generate_quantscript_from_graph_value
parse_graph_quantscript_source
```

相关回归建议:

```powershell
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
cargo test -p quantpilot
```

本批只要求 no-code gates；实际抽离时必须根据迁移范围选择更窄的 API / QS / graph tests。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不迁移 `src/graph_quantscript_api.rs`。
3. 不改 `src/compile_api.rs`、`src/graph_api.rs` 或 runtime caller。
4. 不改 route path、response schema、QS parser、QS generator、target projection 或 artifact attach 语义。
5. 不新增 sibling horizontal link。
6. 不启动 release transition。
7. 不宣称 `backend.graph_compile`、`backend` 顶层或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FQ-02
backend.graph_compile.quantscript_graph
root.backend.graph_compile.quantscript_graph
```

BE-001FQ-02 只允许建立抽离方案，必须明确 planned move / import rewrite / re-export surface / caller adaptation，不得直接移动代码或修改函数体。

---

## 验证要求

本批是 `no code movement` 等价基线，提交前至少执行:

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

AI 声称 BE-001FQ-01 完成时，必须说明:

1. 本批是 `no code movement` 等价基线。
2. `src/graph_quantscript_api.rs` 尚未迁移。
3. 当前 parent wildcard residual 仍为 1。
4. helper 调用面同时覆盖 compile、graph、runtime 和测试。
5. 下一步只能进入 BE-001FQ-02 抽离方案。
6. 不得宣称 `backend.graph_compile.quantscript_graph` 已实际抽离。

---

## 验收标准

1. `475-backend.graph_compile.quantscript_graph单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. route handler 与 shared helper 输入面已冻结。
3. 下一步固定为 BE-001FQ-02 抽离方案。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
