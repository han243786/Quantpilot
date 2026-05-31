# v4.16.0 runtime.backtest_import_pass 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DF-01
> 基准: `332-runtime.backtest.record_store_import_pass单叶closeout.md`
> 目标父叶: `runtime.backtest_import_pass`
> 判定: `runtime.backtest_import_pass stop_split: false`
> 下一候选: `runtime.backtest.replay_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DF-01 `runtime.backtest_import_pass` 父叶残余判断 | parent residual judgment |
| 规范矩阵 | explicit import pass、backtest import pocket、minimum batch、release transition guard | 递归选型 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 剩余依赖分流 |
| 模块树 | `runtime.backtest_import_pass` | `stop_split: false` |

---

## 当前残余分布

BE-001DE-04 closeout 后，`src/runtime/**.rs` 中仍有 33 个文件存在 `use super::*` 或 `super::` 依赖。按模块树分布如下:

| 分组 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 仍是 root parent bridge |
| `runtime.run` | 0 | run import pass 已完成并 closeout |
| `runtime.backtest` | 10 | `record_store` import pocket 已 closeout，剩余 replay / sweep / execution / v4 helper 等 parent bridge 依赖 |
| `runtime.mutation` | 21 | mutation 子树仍为最大残余队列，后续单独 staged pass |
| `runtime.run_guard` | 1 | `src/runtime/run_guard.rs` 的 test-only `use super::*` |

计数锚点: root 1 / run 0 / backtest 10 / mutation 21 / test-only 1 / total 33。

### `runtime.backtest` 剩余候选文件

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/replay.rs
src/runtime/backtest/start_orchestration.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

`src/runtime/backtest/record_store.rs` 已由 `runtime.backtest.record_store_import_pass stop_split: true` 关闭，不再进入本轮 backtest import pocket 候选。

### 其他排队区

```text
src/runtime/mod.rs
src/runtime/mutation/**
src/runtime/run_guard.rs
```

---

## 判断

`runtime.backtest_import_pass` 尚未满足收口条件，设置:

```text
runtime.backtest_import_pass stop_split: false
```

原因:

1. `src/runtime/backtest/replay.rs` 仍是独立 direct route handler import pocket，风险和范围小于 experiment / execution 子树。
2. `src/runtime/backtest/experiment_sweep.rs`、`parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 具有父子链，必须后续单独拆 pocket。
3. `src/runtime/backtest/execution_start.rs` 与 `v4_*` / legacy helper 仍有多层历史边界，不能与 replay 同批。
4. `src/runtime/mod.rs`、`src/runtime/mutation/**` 和 test-only `src/runtime/run_guard.rs` 不属于本父叶下一刀。
5. 当前仍存在清晰小批次候选，不能宣称 `runtime.backtest_import_pass` 已完成。

---

## 下一候选选择

下一步选择:

```text
BE-001DG-01 runtime.backtest.replay_import_pass 单子叶等价基线
```

选择理由:

1. `src/runtime/backtest/replay.rs` 是 direct route singleton，只有一个 `get_backtest_replay` public handler，适合在 `record_store` 后继续做小批次 import 收敛。
2. replay 的等价保护可由 `api_backtest` 覆盖 replay endpoint、record lookup、query normalization、timeline response 和 metrics。
3. 先处理 replay 可继续降低 backtest direct route 层 parent import 依赖，再回到 experiment / execution 这类更复杂子树。
4. 未启动 release transition，未新增 sibling horizontal link。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/replay.rs` import。
- 本批不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 本批不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
- 本批不处理 `src/runtime/mod.rs`。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 `runtime.backtest.replay_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_backtest
```

---

## 幻觉检查点

AI 声称 BE-001DF-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.backtest_import_pass stop_split: false`。
3. 当前剩余 parent bridge 依赖文件数为 33。
4. 当前分布为 root 1、run 0、backtest 10、mutation 21、test-only 1。
5. 下一步只能进入 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线。
6. `src/runtime/mod.rs`、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `333-runtime.backtest_import_pass父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶判断明确 `runtime.backtest_import_pass stop_split: false`。
3. 下一步固定为 BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
