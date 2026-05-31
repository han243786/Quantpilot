# v4.16.0 runtime.parent_import_bridge 第四轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FK-01
> 基线: `465-runtime.mutation_import_pass第三轮父叶残余判断.md`
> 目标父叶: `runtime.parent_import_bridge`
> 判定: `runtime.parent_import_bridge stop_split: false`
> 当前剩余: root 1 / run 0 / backtest 0 / report_ops 0 / mutation 0 / production total 1
> 下一候选: `runtime.root_parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement
> 下一步: BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FK-01 `runtime.parent_import_bridge` 第四轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | parent import bridge / explicit import pass / root facade boundary / release transition guard | 递归选型 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | 剩余依赖分流 |
| 模块树 | `runtime.parent_import_bridge` | `stop_split: false` |

---

## 当前残余分布

BE-001FJ-01 closeout 后，`src/runtime/**.rs` 中生产级 parent wildcard import residual 只剩 1 个文件:

| 分组 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 仍是 root parent bridge |
| `runtime.run` | 0 | run import pass 已 closeout |
| `runtime.backtest` | 0 | backtest import pass 已 closeout |
| `runtime.report_ops` | 0 | report ops import pass 已 closeout |
| `runtime.mutation` | 0 | mutation import pass 已 closeout |
| production total | 1 | parent import bridge 尚未完全消除 |

计数锚点:

```text
root 1 / run 0 / backtest 0 / report_ops 0 / mutation 0 / production total 1
remaining_runtime_parent_import_bridge_1
remaining_mutation_import_bridge_0
remaining_root_parent_import_bridge_1
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

当前生产级 residual:

```text
src/runtime/mod.rs
```

test-local residual 不纳入本父叶生产级收口条件，后续可独立判断:

```text
src/runtime/run_guard.rs
src/runtime/mutation/ai_proposal/static_check.rs
```

---

## 判断

`runtime.parent_import_bridge` 尚未满足收口条件，设置:

```text
runtime.parent_import_bridge stop_split: false
```

原因:

1. `src/runtime/mod.rs` 仍通过 `use super::*` 承接 crate root surface。
2. 子模块已经从 wildcard import 收敛为显式 parent surface，但 root facade 仍需先冻结当前 public/private/re-export 输入面。
3. `src/runtime/mod.rs` 同时承担 module declaration、public re-export、private helper bridge、query/schema alias surface，不能直接无基线改写。
4. test-local residual 不应与 root production facade 混批。
5. release transition 未启动，不能以性能优化名义新增 sibling horizontal link。

---

## 下一候选选择

下一步选择:

```text
BE-001FL-01 runtime.root_parent_facade_import_pass 单子叶等价基线
```

选择理由:

1. 所有业务子树 import pass 均已完成，剩余生产级 residual 只在 `src/runtime/mod.rs`。
2. root facade 需要先冻结 `use super::*` 当前提供给 children 的隐式输入面，避免显式化时漏掉 parent-visible schema/helper。
3. `cargo check -p quantpilot` 当前两个 warning 均指向 `src/runtime/mod.rs`，说明 root bridge 已成为明确下一步。
4. 下一步仍是基线，不直接改 Rust；实际 import rewrite 只能在后续方案与记录中发生。

---

## 排除项

本批不处理:

1. 不修改 Rust 代码。
2. 不直接改写 `src/runtime/mod.rs`。
3. 不处理 test-local `use super::*`。
4. 不宣称 `runtime.parent_import_bridge stop_split: true`。
5. 不宣称 `backend.runtime` 或 Rust 重构完成。
6. 不启动发布过渡连接。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

后续 `runtime.root_parent_facade_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001FK-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.parent_import_bridge stop_split: false`。
3. 当前生产级剩余分布为 root 1 / run 0 / backtest 0 / report_ops 0 / mutation 0 / production total 1。
4. 下一步只能进入 BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线。
5. `src/runtime/mod.rs` 尚未处理。
6. test-local residual 不与本批混批。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧三叶暂停目标仍为取消状态，进度报告指令保持丢弃。

不得宣称 runtime parent bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `466-runtime.parent_import_bridge第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶判断明确 `runtime.parent_import_bridge stop_split: false`。
3. 下一步固定为 BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
