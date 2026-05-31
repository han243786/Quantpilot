# v4.16.0 runtime.mutation_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DN-02
> 基准: `350-runtime.mutation_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation_import_pass`
> 判定: `runtime.mutation_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DN-02 `runtime.mutation_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | staged import 拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass` | mutation import pass 方案 |
| 模块树 | `runtime.mutation_import_pass` | `stop_split: false` |

---

## 方案判定

`runtime.mutation_import_pass` 不做 21 文件一次性改写，保持:

```text
runtime.mutation_import_pass stop_split: false
reject_bulk_mutation_rewrite_21_files
```

理由:

1. 21 个 mutation 文件包含 AI proposal、parameter mutation、shared governance 三种不同风险形态。
2. `ai_proposal` 内含 route-facing handler、approval lifecycle、sandbox gate、static check、source/governance/id 和 test-scope wildcard。
3. `parameter_mutation` 内含 create/list/detail、activation/rollback lifecycle、safe-window、transition persistence 和 rollback identity。
4. `shared_governance` 是 mutation 两侧共享 helper，单文件、无 route facade、无 handler owner，适合作为第一批 import pass。
5. 直接 21 文件全改会扩大等价审计面，违背 minimum batch。

---

## 拆分顺序

当前 staged order 固定为:

```text
runtime.mutation.shared_governance_import_pass
runtime.mutation.parameter_mutation_import_pass
runtime.mutation.ai_proposal_import_pass
runtime.mutation_import_pass residual judgement
```

第一候选:

```text
BE-001DO-01 runtime.mutation.shared_governance_import_pass 单子叶等价基线
```

选择原因:

1. `src/runtime/mutation/shared_governance.rs` 只有一个 parent wildcard import 文件。
2. 该文件主要承接 canonical version、target validation、mutation event contract、run event append、governance projection 等共享 helper。
3. 它没有 route facade、frontend caller、schema owner、state owner 或 handler owner。
4. 若后续实际改写成功，预期 parent bridge residual 可从 total 23 降到 22，mutation residual 从 21 降到 20。
5. 先收敛共享 helper 可以降低后续 parameter / AI proposal pocket 的父级导入不透明度。

```text
expected_parent_import_bridge_23_to_22
expected_mutation_import_bridge_21_to_20
```

---

## BE-001DO 允许范围

BE-001DO 只能围绕:

```text
src/runtime/mutation/shared_governance.rs
```

BE-001DO-01 只建立等价基线，不改代码。后续实际抽离批次只允许:

1. 将 `src/runtime/mutation/shared_governance.rs` 顶部 `use super::*` 改为显式 import。
2. 保持以下 helper 的函数名、visibility、返回类型和错误语义:

```text
canonical_runtime_parameter_version
validate_runtime_parameter_mutation_target
runtime_mode_from_events
status_contract_value
mutation_event_contract
build_runtime_parameter_mutation_event
append_parameter_mutation_events_to_run
runtime_parameter_mutation_governance
governance_with_parameter_version
```

3. 不迁移 helper 到其他文件。
4. 不改变 `parameter_mutation` 或 `ai_proposal` caller。
5. 不新增 sibling horizontal link。

---

## 延后范围

以下范围延后:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/**
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/**
src/runtime/mod.rs
src/runtime/run_guard.rs
```

延后理由:

1. `parameter_mutation` 和 `ai_proposal` 均包含 route-facing handler / re-export，不应混入 shared governance 单文件 import pass。
2. `src/runtime/mod.rs` root parent bridge 必须等业务子树完成后再判断。
3. test-only `src/runtime/run_guard.rs` 独立于 mutation 业务边界。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/shared_governance.rs` import。
- 本批不处理 AI proposal 或 parameter mutation 文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 `runtime.mutation.shared_governance_import_pass` 实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001DN-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. `runtime.mutation_import_pass stop_split: false`。
3. 本批拒绝 21 文件一次性改写。
4. 下一步只能进入 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。
5. BE-001DO 当前只允许围绕 `src/runtime/mutation/shared_governance.rs`。
6. `parameter_mutation`、`ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 mutation import 已完成、shared governance import 已改写、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `351-runtime.mutation_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确 `runtime.mutation_import_pass stop_split: false`。
3. 方案拒绝 21 文件整批 rewrite。
4. 下一步固定为 BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
