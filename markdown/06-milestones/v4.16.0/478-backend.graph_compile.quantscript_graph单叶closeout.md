# v4.16.0 backend.graph_compile.quantscript_graph 单叶 closeout
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FQ-04
> 基线: `477-backend.graph_compile.quantscript_graph抽离记录.md`
> 目标子叶: `backend.graph_compile.quantscript_graph`
> 判定: 等价成立，但本叶不停止细拆
> 模块树坐标: `root.backend.graph_compile.quantscript_graph`
> 代码动作: no code movement
> 下一步: BE-001FR-01 `backend.graph_compile.quantscript_graph.graph_to_qs_generation` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FQ-04 `backend.graph_compile.quantscript_graph` 单叶 closeout | 子叶 closeout / 继续细拆判断 |
| 规范矩阵 | equivalence closeout / stop_split false / child selection / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph` | 选择下一层子叶 |
| 模块树 | `backend.graph_compile.quantscript_graph` | `stop_split: false` |

---

## closeout 判定

BE-001FQ-03 已完成实际抽离:

```text
src/backend/graph_compile/quantscript_graph.rs
src/graph_quantscript_api.rs deleted
backend_graph_compile_quantscript_graph_parent_wildcard_residual_0
```

route/helper 等价成立，caller adaptation 通过 `src/lib.rs` 的 root parent re-export surface 保持，未新增 compile / graph / runtime sibling horizontal link。

但本叶当前不应 `stop_split: true`:

```text
backend.graph_compile.quantscript_graph stop_split: false
```

原因是本叶已经成为真实 owner，且内部仍同时承载多个可独立白箱化的责任簇。

---

## 内部责任簇判断

| 候选簇 | 当前职责 | 是否值得继续拆 | 理由 |
| --- | --- | --- | --- |
| `route_surface` | `register_routes`、`load_graph_quantscript`、`parse_graph_quantscript` | 暂缓 | 很薄，主要依赖 parser 和 AppState，单独拆收益低 |
| `graph_to_qs_generation` | `generate_quantscript_from_graph_value`、`generate_node_quantscript`、graph node/edge rendering | 是 | QS 生成逻辑大、分支多、测试价值高，适合作为下一层第一刀 |
| `formal_module_conversion` | `convert_graph_json_to_script_module` | 稍后 | 与 formal QuantScript lowering 语义相关，适合在 generation 后评估 |
| `artifact_target_projection` | `attach_quantscript_artifacts`、target / label / node source projection | 稍后 | 与 graph persistence 和 runtime diagnostics 相关，需单独基线 |
| `strategy_graph_parser` | `parse_graph_quantscript_source`、`parse_qs_*` | 稍后 | parser 语义风险高，需在 generator baseline 后单独处理 |

---

## 下一层选择

下一步选择:

```text
BE-001FR-01
backend.graph_compile.quantscript_graph.graph_to_qs_generation
root.backend.graph_compile.quantscript_graph.graph_to_qs_generation
```

优先选择 `graph_to_qs_generation` 的原因:

1. 它是 graph JSON 到 strategy_graph QuantScript 源码的主要出口。
2. 它拥有最多 module_key 分支，最容易产生 AI 幻觉式遗漏。
3. 它直接影响 compile、graph save artifact 和 lib tests。
4. 它可以先建立等价基线，再决定是否把 node rendering / intent branch / scalar rendering 继续细拆。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不拆 route handler、parser、artifact target projection 或 formal module conversion。
3. 不改 `src/lib.rs` root parent re-export surface。
4. 不改 compile / graph / runtime caller。
5. 不启动 release transition。
6. 不宣称 `backend.graph_compile`、`backend` 顶层或 Rust 重构完成。

---

## 下一步边界

BE-001FR-01 只允许建立 `graph_to_qs_generation` 等价基线，冻结:

```text
generate_quantscript_from_graph_value
generate_node_quantscript
quoted
render_json_scalar
```

BE-001FR-01 不得直接移动 generator 函数、不得修改 parser、不得改 route、不得新增 sibling horizontal link。

---

## 验证要求

本批是 `no code movement` closeout，提交前至少执行:

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

AI 声称 BE-001FQ-04 完成时，必须说明:

1. 本批是 `no code movement` 单叶 closeout。
2. BE-001FQ-03 的实际抽离已完成，但 `backend.graph_compile.quantscript_graph stop_split: false`。
3. 下一步只能进入 BE-001FR-01 `graph_to_qs_generation` 等价基线。
4. 不得宣称 parser、artifact targets、formal conversion 或 `backend.graph_compile` 父叶已经收口。

---

## 验收标准

1. `478-backend.graph_compile.quantscript_graph单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend.graph_compile.quantscript_graph stop_split: false` 已记录。
3. 下一步固定为 BE-001FR-01 `graph_to_qs_generation` 等价基线。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
