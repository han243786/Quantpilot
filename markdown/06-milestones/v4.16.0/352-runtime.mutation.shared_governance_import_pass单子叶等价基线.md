# v4.16.0 runtime.mutation.shared_governance_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DO-01
> 基准: `351-runtime.mutation_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.shared_governance_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DO-01 `runtime.mutation.shared_governance_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 单文件 import 基线 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass` | shared governance import 白箱 |
| 模块树 | `runtime.mutation.shared_governance_import_pass` | 新基线 |

---

## 当前事实

`runtime.mutation.shared_governance_import_pass` 是 `runtime.mutation_import_pass` 拆出的第一批单文件 import 收敛，不是新增业务 owner。当前目标文件为:

```text
src/runtime/mutation/shared_governance.rs
```

该文件当前顶部仍存在:

```rust
use super::*;
```

当前 parent bridge 总分布保持不变:

```text
root 1
run 0
backtest 0
mutation 21
test-only 1
total 23
runtime.mutation.shared_governance_import_pass baseline_frozen
```

---

## 白箱 helper 面

本基线冻结以下 9 个 helper 的名称、visibility、返回类型和错误语义:

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

其中:

1. `canonical_runtime_parameter_version` 负责 target/value 的 canonical JSON sha256 版本生成。
2. `validate_runtime_parameter_mutation_target` 负责 node/module/parameter path 与 capability gate 校验。
3. `runtime_mode_from_events` 从现有 runtime event envelope 推导运行模式，缺省为 `paper`。
4. `status_contract_value` 与 `mutation_event_contract` 固定 status 到 contract 字符串映射。
5. `build_runtime_parameter_mutation_event` 构造 runtime event payload、severity、summary 与 envelope。
6. `append_parameter_mutation_events_to_run` 负责追加 mutation event、补 envelope、更新 active parameter version 并按需持久化 run record。
7. `runtime_parameter_mutation_governance` 与 `governance_with_parameter_version` 负责 governance projection。

---

## 当前隐式输入面

后续显式 import 改写必须从 `use super::*` 中拆出所需输入，至少覆盖:

```text
RuntimeParameterMutationTarget
Value
StatusCode
canonical_json_sha256_digest
json
internal_error
anyhow::anyhow
json_bad_request
SUPPORTED_FRONTEND_MODULE_KEYS
FrontendRuntimeEvent
RuntimeParameterMutationStatus
RuntimeParameterMutationRecord
RuntimeEventEnvelope
AppState
auth::UserId
RuntimeGovernanceSnapshot
load_run_record_from_state
attach_runtime_event_envelope
validate_runtime_event_envelopes
fs::try_exists
io_error
persist_run_record
RuntimeParameterMutationGovernance
```

这些输入只能服务于 `src/runtime/mutation/shared_governance.rs` 顶部 import 收敛；不得借机迁移 caller、handler、schema、state 或 persistence owner。

---

## 等价边界

BE-001DO-02 方案和后续实际改写必须保持:

1. 只处理 `src/runtime/mutation/shared_governance.rs` 的 parent wildcard import。
2. 不迁移 helper 到其他文件，不改变 `pub(super)` visibility。
3. 不改变 mutation status contract、event type、reason code、payload 字段、severity、summary 文案和 envelope sequence 语义。
4. 不改变 run record 读写、内存态更新、磁盘持久化条件或 error mapping。
5. 不改变 `parameter_mutation` 或 `ai_proposal` caller。
6. 不新增 sibling horizontal link，不启动 release transition。

---

## 预期收敛

若后续实际 import pass 成功:

```text
expected_parent_import_bridge_23_to_22
expected_mutation_import_bridge_21_to_20
```

本基线不直接实现该收敛，只冻结下一批抽离方案的输入面。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/mutation/shared_governance.rs` import。
- 本批不处理 `src/runtime/mutation/parameter_mutation.rs` 或 `src/runtime/mutation/parameter_mutation/**`。
- 本批不处理 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**`。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际 import pass 至少补跑:

```powershell
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001DO-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标文件仍为 `src/runtime/mutation/shared_governance.rs`。
3. `use super::*` 尚未改写。
4. 当前 parent bridge 总分布仍为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。
5. 下一步只能进入 BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案。
6. `parameter_mutation`、`ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 shared governance import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 `src/runtime/mutation/shared_governance.rs` 的 9 个 helper、当前 `use super::*` 和预期显式 import 输入面。
3. 下一步固定为 BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
