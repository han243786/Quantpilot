# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AN-01  
> 基准: `154-runtime.mutation.parameter_mutation.transition_lifecycle第三轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 单子叶等价基线已建立。下一步只能进入 BE-001AN-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AN-01 activation_snapshot_side_effect 等价基线 | 基线 |
| 规范矩阵 | 三档执行、父子通信、stop_split、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 新候选白箱 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` | 冻结输入输出 |

---

## 真实文件

- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `tests/api_mutation.rs`

当前只建立等价基线，目标文件尚未创建。不得创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs`。

---

## 白箱边界

| 项 | 当前基线 |
| --- | --- |
| 候选方法 | `auto_snapshot_on_activation` |
| 输入 | `AppState`、`auth::UserId`、`RuntimeParameterMutationRecord` |
| 调用方 | `activation_flow::activate_runtime_parameter_mutation` 经父级 `transition_lifecycle` helper |
| 输出 | config generation side effect、snapshot file、in-memory snapshot |
| 返回 | async `()`，错误只通过 `safe_eprintln!` 记录 |
| 当前 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 父级 |

---

## 输入基线

| 输入 | 来源 | 约束 |
| --- | --- | --- |
| `state.config_generation` | `AppState` | 使用 `fetch_add(1, SeqCst)`，不改变 generation 递增语义 |
| `state.config_generation_history` | `AppState` | push `qrpc_runtime::ConfigGenerationEntry`，最多保留 100 条 |
| `state.evidence_metrics` | `AppState` | 只读取 proposal rejected / rollback attempt baseline，不改 metric |
| `state.snapshot_store_dir` | `AppState` | 用于写入 `snap-auto-{now_ms}.json` |
| `state.snapshots` | `AppState` | 使用 `auth::scoped_key(user_id, snapshot_id)` 写入内存快照 |
| mutation governance | `RuntimeParameterMutationRecord` | 读取 deployment revision、capability hash、strategy version、proposed parameter version |

---

## 输出基线

| 输出 | 当前语义 |
| --- | --- |
| config generation entry | `generation`、`activated_at_ms`、`deployment_revision`、`parameter_version` |
| history truncation | `MAX_GENERATION_HISTORY = 100`，超出部分从头部 drain |
| snapshot id | `snap-auto-{now_ms}` |
| `DeploymentSignatureSnapshot` | 固定 payload: deployment revision、capability hash、strategy version、parameter version、empty event slice、created_at_ms、signature |
| snapshot signature | `canonical_json_sha256_digest` over capability hash / strategy version / parameter version / created_at_ms，失败 fallback `signature-unavailable` |
| atomic write | `crate::runtime_persistence::atomic_write_json(&path, &snapshot).await` |
| write failure | `safe_eprintln!("[snapshot] 原子写入快照失败: {}", e)`，不改变 handler response |
| in-memory snapshot | `state.snapshots.write().await.insert(scoped_key, snapshot)` |

---

## 时序基线

```text
activation_flow::activate_runtime_parameter_mutation
  -> persist_runtime_parameter_mutation_transition
  -> auto_snapshot_on_activation
     -> current_time_ms
     -> config_generation.fetch_add
     -> config_generation_history push / truncate
     -> evidence metric baseline reads
     -> DeploymentSignatureSnapshot build
     -> atomic_write_json
     -> snapshots insert
```

必须保持 activation response 在 snapshot write 失败时不失败；snapshot side effect 只是 activation after-effect，不拥有 route response schema。

---

## 父子通信规则

`activation_snapshot_side_effect` 只能作为 `transition_lifecycle` 的 child 被父级管理。后续若实际抽离，`activation_flow` 仍只能经父级 `transition_lifecycle` 的受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。

ASCII guard: `release transition guard`。

---

## 排除边界

- 不迁移 Rust 代码。
- 不创建目标文件。
- 不修改 snapshot payload、signature、id、history cap 或 atomic write。
- 不迁移 `mutation_lifecycle_entry`。
- 不迁移 `persist_runtime_parameter_mutation_transition`。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。

---

## 验证记录

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check -p quantpilot` | PASS |
| `cargo test --no-run` | PASS |
| `cargo test -p quantpilot --test api_mutation` | PASS |
| `cargo test -p quantpilot --test api_ai_proposal` | PASS |
| `cargo test -p quantpilot --test api_evidence_contract` | PASS |
| `cargo test -p quantpilot --test api_run` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1` | PASS |
| `git diff --check` | PASS |

---

## 下一步

下一批进入 BE-001AN-02 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect` 抽离方案。只能固定目标文件、父级 path attribute、helper visibility、调用面和回退点；不得移动代码。

---

## 幻觉检查点

AI 声称 BE-001AN-01 完成时，必须说明当前只是等价基线，`auto_snapshot_on_activation` 仍留在父级，目标文件未创建，下一步只能进入 BE-001AN-02 抽离方案。不得宣称 snapshot helper 已抽离、shared lifecycle/persistence helper 已拆分、rollback helper 已迁移、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `155-runtime.mutation.parameter_mutation.transition_lifecycle.activation_snapshot_side_effect单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树记录 `activation_snapshot_side_effect` 已建立等价基线，但代码未移动。
3. 全量树记录 BE-001AN-01 并把下一步固定为 BE-001AN-02 抽离方案。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AN-02。
