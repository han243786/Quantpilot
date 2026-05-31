# v4.16.0 runtime.parent_import_bridge 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DC-01
> 基准: `325-runtime.run_import_pass单叶closeout.md`
> 目标父叶: `runtime.parent_import_bridge`
> 判定: `runtime.parent_import_bridge stop_split: false`
> 下一候选: `runtime.backtest_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 递归选型 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | 剩余依赖分流 |
| 模块树 | `runtime.parent_import_bridge` | `stop_split: false` |

---

## 流程口径清理

本批确认取消“固定完成三个叶子后暂停”的临时目标。后续执行口径回到干净递归:

```text
父叶残余判断 -> 单子叶等价基线 -> 抽离方案 -> 实际抽离 -> 单叶 closeout -> 回到父叶残余判断
```

每个可验收递归步完成后单独提交；除非出现真实设计决策、等价缺口或门禁失败，不设置固定叶子数量暂停点。

---

## 当前残余分布

BE-001DB-04 closeout 后，`src/runtime/**.rs` 中仍有 34 个文件存在 `use super::*` 或 `super::` 依赖。按模块树分布如下:

| 分组 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 仍是 root parent bridge |
| `runtime.run` | 0 | 4 个 run child 已完成 explicit import pass |
| `runtime.backtest` | 11 | backtest 子树仍有 execution / replay / sweep / v4 projection 等 parent bridge 依赖 |
| `runtime.mutation` | 21 | mutation 子树依赖最密集，后续需要更细 staged pass |
| `runtime.run_guard` | 1 | `src/runtime/run_guard.rs` 的 test-only `use super::*` |

计数锚点: root 1 / run 0 / backtest 11 / mutation 21 / test-only 1 / total 34。

### `runtime.backtest` 候选文件

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/record_store.rs
src/runtime/backtest/replay.rs
src/runtime/backtest/start_orchestration.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

### 其余排队区

```text
src/runtime/mod.rs
src/runtime/mutation/**
src/runtime/run_guard.rs
```

---

## 判断

`runtime.parent_import_bridge` 尚未满足收口条件，设置:

```text
runtime.parent_import_bridge stop_split: false
```

原因:

1. `src/runtime/mod.rs` 仍保留 root parent bridge。
2. `src/runtime/backtest/**` 与 `src/runtime/mutation/**` 仍有明确 parent wildcard import 或 parent path import。
3. test-only `src/runtime/run_guard.rs` 仍需后续独立判断，不应与业务子树混批。
4. 当前还存在清晰的小批次候选，不能宣称 parent import bridge 已消除。

---

## 下一候选选择

下一步选择:

```text
BE-001DD-01 runtime.backtest_import_pass 单子叶等价基线
```

选择理由:

1. `runtime.backtest` 剩余 11 个文件，规模小于 `runtime.mutation` 的 21 个文件。
2. backtest 子树有较明确的 `api_backtest` 覆盖面，适合作为 run 之后的下一批 staged explicit import pass。
3. 先处理业务子树，再回到 `src/runtime/mod.rs` root bridge 和 test-only `run_guard`，更符合父子通信规则。
4. 未启动 release transition，未新增 sibling horizontal link。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/**` import。
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

后续 `runtime.backtest_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_backtest
```

---

## 幻觉检查点

AI 声称 BE-001DC-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.parent_import_bridge stop_split: false`。
3. 当前剩余 parent bridge 依赖文件数为 34。
4. 当前分布为 root 1、run 0、backtest 11、mutation 21、test-only 1。
5. 下一步只能进入 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线。
6. `src/runtime/mod.rs`、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `326-runtime.parent_import_bridge父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶判断明确 `runtime.parent_import_bridge stop_split: false`。
3. 下一步固定为 BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
