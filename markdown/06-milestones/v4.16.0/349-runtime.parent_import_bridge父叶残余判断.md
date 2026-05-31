# v4.16.0 runtime.parent_import_bridge 父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DM-01
> 基准: `348-runtime.backtest_import_pass第四轮父叶残余判断.md`
> 目标父叶: `runtime.parent_import_bridge`
> 判定: `runtime.parent_import_bridge stop_split: false`
> 当前剩余: root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23
> 下一候选: `runtime.mutation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement
> 下一步: BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 递归选型 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | 剩余依赖分流 |
| 模块树 | `runtime.parent_import_bridge` | `stop_split: false` |

---

## 当前残余分布

BE-001DL-01 closeout 后，`src/runtime/**.rs` 中仍有 23 个文件存在 `use super::*` 或 `super::` 依赖。按模块树分布如下:

| 分组 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 仍是 root parent bridge |
| `runtime.run` | 0 | run import pass 已 closeout |
| `runtime.backtest` | 0 | backtest import pass 已 closeout |
| `runtime.mutation` | 21 | mutation 子树依赖最密集，仍需 staged import pass |
| test-only | 1 | `src/runtime/run_guard.rs` 的 test-only `use super::*` |
| total | 23 | parent import bridge 尚未消除 |

计数锚点: root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。

---

## `runtime.mutation` 候选文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/mutation/shared_governance.rs
```

其余排队区:

```text
src/runtime/mod.rs
src/runtime/run_guard.rs
```

---

## 判断

`runtime.parent_import_bridge` 尚未满足收口条件，设置:

```text
runtime.parent_import_bridge stop_split: false
```

原因:

1. `src/runtime/mutation/**` 仍有 21 个 parent bridge 依赖文件。
2. `src/runtime/mod.rs` 仍保留 root parent bridge。
3. test-only `src/runtime/run_guard.rs` 仍需后续独立判断，不应与业务子树混批。
4. 当前仍存在清晰的 mutation staged import pass 候选，不能宣称 parent import bridge 已消除。
5. release transition 未启动，不能以性能优化名义新增 sibling horizontal link。

---

## 下一候选选择

下一步选择:

```text
BE-001DN-01 runtime.mutation_import_pass 单子叶等价基线
```

选择理由:

1. run 与 backtest 两个业务子树已经完成 import pass closeout。
2. `runtime.mutation` 是最后一个完整业务子树，先处理它比直接动 `src/runtime/mod.rs` root bridge 更符合父子通信规则。
3. mutation 子树可以继续拆成 `runtime.mutation.ai_proposal_import_pass`、`runtime.mutation.parameter_mutation_import_pass` 与 `runtime.mutation.shared_governance_import_pass` 等小批次，不适合 21 文件一次性改写。
4. 等价证据可由 `api_mutation`、`api_ai_proposal`、proposal static check、approval lifecycle、parameter mutation transition lifecycle 等路径覆盖。
5. test-only `run_guard` 更适合在业务子树完成后单独收口，避免与 mutation 行为边界混在一起。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/**` import。
- 本批不处理 `src/runtime/mod.rs`。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

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

后续 `runtime.mutation_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001DM-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `runtime.parent_import_bridge stop_split: false`。
3. 当前剩余分布为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。
4. 下一步只能进入 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线。
5. `runtime.mutation_import_pass` 不能 21 文件一次性改写，必须先建基线并判断是否继续拆 pocket。
6. `src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除、mutation import 已完成或 root bridge 已处理。

---

## 验收标准

1. `349-runtime.parent_import_bridge父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶判断明确 `runtime.parent_import_bridge stop_split: false`。
3. 下一步固定为 BE-001DN-01 `runtime.mutation_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
