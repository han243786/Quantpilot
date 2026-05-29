# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AR-01  
> 基准: `164-runtime.mutation.parameter_mutation.transition_lifecycle第五轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 单子叶等价基线已建立。下一步只能进入 BE-001AR-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AR-01 rollback_record_identity 等价基线 | 基线 |
| 规范矩阵 | 父子通信、rollback id identity、digest contract、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 新候选白箱 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 冻结输入输出 |

---

## 当前状态

当前只建立等价基线，目标文件尚未创建；ASCII guard: `target file not created`。不得创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`。

`runtime_parameter_mutation_rollback_record_id` 仍留在 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 父级，由 `rollback_flow.rs` 通过父级 helper 名称调用。

---

## 白箱边界

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `runtime_parameter_mutation_rollback_record_id` | `source_id`、`rollback_of`、`RuntimeParameterMutationTarget`、`created_at_ms`、`source_event_count`、`proposed_parameter_version` | `Result<String, (StatusCode, String)>` rollback proposal id | `rollback_flow` via parent helper | 不得改变 digest input、prefix、slice length 或 error mapping |

---

## 输入冻结

| 输入 | 来源 | 必须保持 |
| --- | --- | --- |
| `source_id` | rollback source run / activated proposal | 仍写入 digest input `"source_id"` |
| `rollback_of` | activated proposal id | 仍写入 digest input `"rollback_of"` |
| `target` | `RuntimeParameterMutationTarget` | 仍写入 digest input `"target"` |
| `created_at_ms` | rollback record creation time | 仍写入 digest input `"created_at_ms"` 且作为 output prefix 时间段 |
| `source_event_count` | source run events length | 仍写入 digest input `"source_event_count"` |
| `proposed_parameter_version` | rollback target parameter version | 仍写入 digest input `"proposed_parameter_version"` |

---

## 输出冻结

| 输出 | 约束 |
| --- | --- |
| digest | 仍来自 `canonical_json_sha256_digest(&json!(...))` |
| error mapping | digest error 仍经 `internal_error(anyhow::anyhow!(error))` 转为 `(StatusCode, String)` |
| id prefix | 仍为 `parameter_rollback_` |
| digest segment | 仍使用 `&digest.value[..12]` |
| final id | 仍为 `parameter_rollback_{created_at_ms}_{digest[..12]}` |

---

## 父子通信规则

```text
rollback_flow.rs
  -> transition_lifecycle::runtime_parameter_mutation_rollback_record_id
transition_lifecycle.rs
  -> rollback_record_identity child (仅在后续实际抽离批次允许)
```

`rollback_record_identity` 只能作为 `transition_lifecycle` 的 child 被父级管理。BE-001AR-01 只建立等价基线；后续若实际抽离，rollback_flow 仍经父级受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。

---

## 真实文件

| 文件 | 角色 |
| --- | --- |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` | 当前 helper owner，保留 target file not created 状态 |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` | 唯一调用方 |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` | sibling 已 closeout |
| `tests/api_mutation.rs` | rollback 主回归证据 |

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `rollback_record_identity.rs`。
- 不回改已 closeout 的 `rollback_flow`。
- 不改变 `runtime_parameter_mutation_rollback_record_id` 的签名、digest input、prefix、slice length 或 error mapping。
- 不迁移 activation handler、rollback handler、boundary helper、snapshot helper、transition record persistence helper、proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

ASCII guard: `closed child` boundaries remain protected; `release transition guard` remains excluded.

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批进入 BE-001AR-02 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 抽离方案。只能固定目标文件、父级 path attribute、helper visibility、调用面和回退点；不得移动代码。

---

## 幻觉检查点

AI 声称 BE-001AR-01 完成时，必须说明当前只是等价基线，`runtime_parameter_mutation_rollback_record_id` 仍留在 `transition_lifecycle` 父级，target file not created，下一步只能进入 BE-001AR-02 抽离方案。不得宣称 rollback id 已抽离、rollback_flow 已回改、transition_lifecycle 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `165-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树记录 `rollback_record_identity` 已建立等价基线，但代码未移动。
3. 等价基线冻结 `runtime_parameter_mutation_rollback_record_id` 的输入、digest、prefix、slice 和 error mapping。
4. 本批明确 `no code movement`，不创建目标文件、不回改 `rollback_flow`、不迁移 AppState/schema/frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AR-02。
