# v4.16.0 runtime.parent_import_bridge 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DA-01
> 基准: `320-runtime.report_ops_import_pass单叶closeout.md`
> 目标父叶: `runtime.parent_import_bridge`
> 判定: `runtime.parent_import_bridge stop_split: false`
> 下一候选: `runtime.run_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DA-01 `runtime.parent_import_bridge` 父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 递归选型 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | 剩余依赖分流 |
| 模块树 | `runtime.parent_import_bridge` | `stop_split: false` |

---

## 当前残余分布

BE-001CZ-04 closeout 后，`src/runtime/**.rs` 中仍有 38 个文件存在 `use super::*` 或 `super::` 依赖。按模块树分布如下:

| 分组 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 仍是上层 parent bridge |
| `runtime.run` | 4 | run 子树已完成 handler 抽离，但 child 仍依赖 parent wildcard |
| `runtime.backtest` | 11 | backtest 子树依赖面较大，包含 execution / replay / v4 projection |
| `runtime.mutation` | 21 | mutation 子树依赖最密集，包含 AI proposal 与 parameter mutation |
| `runtime.run_guard` | 1 | `src/runtime/run_guard.rs` 的 test-only `use super::*` |

计数锚点: root 1 / run 4 / backtest 11 / mutation 21 / test-only 1。

---

## 判断

`runtime.parent_import_bridge` 尚未满足收口条件，设置:

```text
runtime.parent_import_bridge stop_split: false
```

原因:

1. `src/runtime/mod.rs` 仍保留 root parent bridge。
2. run/backtest/mutation 子树仍有明确的 parent wildcard import。
3. test-only `src/runtime/run_guard.rs` 仍需后续单独判定，不应与业务子树混批。
4. 当前还有足够清晰的小批次候选，不能宣称 parent import bridge 已消除。

---

## 下一候选选择

下一步选择:

```text
BE-001DB-01 runtime.run_import_pass 单子叶等价基线
```

选择理由:

1. `runtime.run` 只有 4 个依赖文件，是剩余业务子树中最小的可验证批次。
2. 这 4 个文件对应的 run handler 已在前序递归中完成抽离，当前只需要冻结 import 边界，不重新移动 handler。
3. `api_run`、`api_sse` 与 run record / replay / session start 路径可以形成稳定等价证据。
4. 先处理小子树能验证 child-subtree explicit import pass 的做法，再进入更重的 `runtime.backtest` 与 `runtime.mutation`。

候选文件:

```text
src/runtime/run/**
src/runtime/run/v4_handoff.rs
src/runtime/run/session_start.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
```

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `runtime.run` import。
- 本批不处理 `src/runtime/mod.rs`。
- 本批不处理 `src/runtime/backtest/**`、`src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
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

后续 `runtime.run_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
```

---

## 幻觉检查点

AI 声称 BE-001DA-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.parent_import_bridge stop_split: false`。
3. 当前剩余 parent bridge 依赖文件数为 38。
4. 下一步只能进入 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线。
5. `src/runtime/mod.rs`、backtest/mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `321-runtime.parent_import_bridge父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶判断明确 `runtime.parent_import_bridge stop_split: false`。
3. 下一步固定为 BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
