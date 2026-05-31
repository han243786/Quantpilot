# v4.16.0 runtime.mutation.ai_proposal.parent_facade_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FH-04
> 基线: `462-runtime.mutation.ai_proposal.parent_facade_import_pass抽离记录.md`
> 目标子叶: `runtime.mutation.ai_proposal.parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal.rs`
> 代码动作: no code movement
> 下一步: BE-001FI-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FH-04 `runtime.mutation.ai_proposal.parent_facade_import_pass` 单叶 closeout | 单叶收口 |
| 规范矩阵 | single leaf closeout / parent facade import stop_split true / no micro split | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass` | 回到父叶残余判断 |
| 模块树 | `runtime.mutation.ai_proposal.parent_facade_import_pass` | 停止继续细拆 |

---

## closeout 判定

```text
BE-001FH-04
BE-001FI-01
runtime.mutation.ai_proposal.parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.parent_facade_import_pass
parent_facade_import_pass_closeout_complete
runtime.mutation.ai_proposal.parent_facade_import_pass stop_split: true
single_file_ai_proposal_parent_facade_import_pass
RuntimeApprovalListQuery_explicit_parent_import
no code movement
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

当前 residual:

```text
remaining_runtime_parent_import_bridge_1
remaining_mutation_import_bridge_0
remaining_ai_proposal_import_bridge_0
remaining_root_parent_import_bridge_1
```

当前生产级 runtime parent bridge residual 只剩:

```text
src/runtime/mod.rs
```

本叶不继续细拆，原因:

1. child module declaration 只是 `ai_proposal.rs` 的稳定白箱入口，不形成新的行为 owner。
2. public re-export 只是父级 facade 转运面，继续拆会制造 handler-by-handler 微叶。
3. `RuntimeApprovalListQuery` 是编译探针发现并已显式保留的 hidden parent input，不需要再拆成独立 owner。
4. `v4_ai_proposal_tests` 已从 wildcard 改成显式 import，测试输入面已经足够清晰。
5. parent-private unused helper imports 已在 BE-001FH-03 移除，剩余 child declaration / re-export / test module / import alias 不再提供值得独立抽离的行为边界。

因此设置:

```text
runtime.mutation.ai_proposal.parent_facade_import_pass stop_split: true
```

---

## 不进入范围

本批不处理:

1. 不改 `src/runtime/mutation/ai_proposal.rs`。
2. 不改任何 `src/runtime/mutation/ai_proposal/**` child file。
3. 不改 `src/runtime/mod.rs` 的 root bridge residual。
4. 不继续拆 child module declaration、public re-export、test module 或 hidden input alias 微叶。
5. 不宣称 `runtime.mutation.ai_proposal_import_pass stop_split: true`。
6. 不宣称 `runtime.mutation_import_pass`、`runtime.parent_import_bridge`、`backend.runtime` 或 Rust 重构完成。
7. 不启动发布过渡连接。

---

## 细拆否决标记

```text
no_child_module_declaration_micro_split
no_public_reexport_micro_split
no_test_module_micro_split
no_hidden_input_micro_split
no_route_facade_rewrite
no_state_owner_migration
no_schema_owner_migration
no_sibling_horizontal_link
no_release_transition
```

---

## 下一步边界

下一步只允许进入上层父叶残余判断:

```text
BE-001FI-01
runtime.mutation.ai_proposal_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass
```

BE-001FI-01 只能判断 `runtime.mutation.ai_proposal_import_pass` 的剩余 residual 是否清零，不得直接改 Rust。

---

## 验证要求

本批提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot v4_ai_proposal_tests::ai_proposal_approval_requires_binding_and_sandbox_report
```

---

## 幻觉检查点

AI 声称 BE-001FH-04 完成时，必须说明:

1. 本批是 `no code movement`。
2. `runtime.mutation.ai_proposal.parent_facade_import_pass stop_split: true`。
3. `ai_proposal` import pass residual 为 0，但上层仍需 BE-001FI-01 父叶残余判断确认。
4. 当前生产级 runtime parent bridge residual 仍有 `src/runtime/mod.rs`。
5. 下一步只能进入 BE-001FI-01。
6. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。
7. 进度报告指令保持丢弃: `progress_report_instruction_discarded`。

不得声称 ai_proposal_import_pass 已完成、mutation_import_pass 已完成、runtime parent bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `463-runtime.mutation.ai_proposal.parent_facade_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.mutation.ai_proposal.parent_facade_import_pass stop_split: true`。
3. 下一步固定为 BE-001FI-01 父叶残余判断。
4. Rust / 治理 / 全量树门禁均通过。
