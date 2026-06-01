# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FT-02
> 基线: `484-backend.graph_compile.quantscript_graph.formal_module_conversion单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion`
> 代码动作: no code movement
> 下一步: BE-001FT-03 `backend.graph_compile.quantscript_graph.formal_module_conversion` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FT-02 `formal_module_conversion` 抽离方案 | 子叶抽离方案 |
| 规范矩阵 | planned child / visibility / parent communication rule / rollback point / test gates | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion` | 固定实际抽离动作 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion` | planned child file |

---

## planned child

BE-001FT-03 只允许创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

父级文件保持:

```text
src/backend/graph_compile/quantscript_graph.rs
```

父级声明固定为:

```rust
mod formal_module_conversion;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
```

方案冻结标记:

```text
formal_module_conversion plan_frozen
```

---

## 允许迁移清单

BE-001FT-03 只允许迁移以下函数:

```text
convert_graph_json_to_script_module
```

目标可见性保持:

```rust
pub(crate) fn convert_graph_json_to_script_module(
    graph_value: &Value,
) -> anyhow::Result<ScriptModule>
```

迁移时函数体必须整体搬迁，不得拆分 data / risk / execution / intent 分支，不得改错误文本或 defaults。

---

## 禁止迁移清单

BE-001FT-03 不得迁移:

```text
register_routes
load_graph_quantscript
parse_graph_quantscript
generate_quantscript_from_graph_value
attach_quantscript_artifacts
build_quantscript_node_sources
build_quantscript_label_targets
build_quantscript_runtime_targets
build_compile_runtime_targets_from_graph
parse_graph_quantscript_source
parse_qs_scalar
parse_qs_node_header
parse_qs_connect
```

这些簇分别属于 route surface、graph-to-QS generation、artifact target projection、runtime target projection 和 strategy_graph parser，必须另起基线或回到父叶残余判断。

---

## import 与通信规则

child file 只允许引入:

```rust
use quantscript::{parse_quant_script_module, ScriptModule};
use serde_json::Value;
```

child 内部继续允许直接使用:

```rust
anyhow::anyhow!
anyhow::bail!
safe_eprintln!
```

父级只能保留必要声明:

```rust
mod formal_module_conversion;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
```

父级 `quantscript::{parse_quant_script_module, ScriptModule}` import 在 BE-001FT-03 后应删除，因为 formal conversion child 成为该 import 的 owner。

不得新增 compile / graph / runtime sibling horizontal link，也不得改 `src/lib.rs` root parent re-export surface。

本批和 BE-001FT-03 均不得启动 release transition guard。

---

## 等价保护点

BE-001FT-03 必须保持:

1. graph shape 校验: `graph.nodes` 与 `graph.edges` 必须为数组。
2. data projection: exchange / instrument / timeframe / window / ping / interval defaults 不变。
3. risk projection: risk profile、position、leverage、action interval defaults 不变。
4. execution projection: profile、fee、slippage defaults 不变。
5. intent support set: double_ma / rsi / ma_deviation / macd / momentum / zscore / spread_observer 不变。
6. unknown node type: 继续 `safe_eprintln!` 后跳过，不变为 hard error。
7. unknown intent module: 继续 `anyhow::bail!`，supported list 文本不变。
8. terminal parse: 继续 `parse_quant_script_module(&qs_source)`。
9. compile caller 映射: `src/compile_api.rs` 继续把 conversion error 映射为 `qs_conversion_failed` / `ERR_QS_LOWER_FAILED`。

---

## 回退点

如果 BE-001FT-03 编译失败或测试暴露语义漂移，只回退本次 planned child:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
src/backend/graph_compile/quantscript_graph.rs
```

不得回退 BE-001FQ-03 已完成的 `src/graph_quantscript_api.rs` 删除，也不得动 graph_to_qs_generation child、compile / graph / runtime sibling 或 `src/lib.rs` root parent re-export surface。

---

## 验证要求

本批是 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

BE-001FT-03 实际抽离必须执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot quantscript --lib
cargo test -p quantpilot --test quantscript_real_strategy_authoring
cargo test -p quantpilot --test api_graph_versions
```

---

## 幻觉检查点

AI 声称 BE-001FT-02 完成时，必须说明:

1. 当前只是 `no code movement` 抽离方案。
2. planned child 固定为 `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`。
3. 下一步 BE-001FT-03 只允许迁移 `convert_graph_json_to_script_module`。
4. 父级只允许新增 `mod formal_module_conversion` 与受控 re-export。
5. 不得宣称 route surface、graph generation、artifact projection、strategy graph parser、`backend.graph_compile` 或 Rust 重构已收口。

---

## 验收标准

1. `485-backend.graph_compile.quantscript_graph.formal_module_conversion抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `formal_module_conversion plan_frozen` 已记录。
3. planned child、父级声明、允许迁移清单、禁止迁移清单、回退点和测试门禁已固定。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
