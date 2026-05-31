# v4.16.0 runtime.mutation_import_pass 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FJ-01
> 基线: `464-runtime.mutation.ai_proposal_import_pass第十二轮父叶残余判断.md`
> 目标父叶: `runtime.mutation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FK-01 `runtime.parent_import_bridge` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FJ-01 `runtime.mutation_import_pass` 第三轮父叶残余判断 | 父叶收口 |
| 规范矩阵 | recursive residual judgment / staged explicit import pass / parent stop_split true | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass` | 回到 root parent bridge |
| 模块树 | `runtime.mutation_import_pass` | 父叶完成 |

---

## 父叶残余判定

```text
BE-001FJ-01
BE-001FK-01
runtime.mutation_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation_import_pass
mutation_import_pass third_parent_residual_judgment
runtime.mutation_import_pass stop_split: true
no code movement
remaining_runtime_parent_import_bridge_1
remaining_mutation_import_bridge_0
remaining_root_parent_import_bridge_1
old_three_leaf_pause_target_cancelled
progress_report_instruction_discarded
```

本轮不做 Rust 代码移动，只判断 `runtime.mutation_import_pass` 是否仍有可继续抽离的生产级 `use super::*` residual。

已完成 closeout 的 mutation pockets:

1. `runtime.mutation.shared_governance_import_pass`
2. `runtime.mutation.parameter_mutation_import_pass`
3. `runtime.mutation.ai_proposal_import_pass`

真实 residual 复核:

```text
src/runtime/mutation/**
use super::*
```

`src/runtime/mutation/**` 下已无生产级 `use super::*` residual。因此本父叶可以收口:

```text
runtime.mutation_import_pass stop_split: true
```

当前生产级 runtime parent bridge residual 仍只剩:

```text
src/runtime/mod.rs
```

---

## 不进入范围

本批不处理:

1. 不修改 `src/runtime/mutation/**`。
2. 不修改 `src/runtime/mod.rs`。
3. 不处理 test-local `use super::*`。
4. 不宣称 `runtime.parent_import_bridge stop_split: true`。
5. 不宣称 `backend.runtime` 或 Rust 重构完成。
6. 不启动发布过渡连接。

---

## 下一步边界

下一步只允许回到 root parent bridge 父叶残余判断:

```text
BE-001FK-01
runtime.parent_import_bridge
root.backend.runtime.runtime.parent_import_bridge
```

BE-001FK-01 只能判断 root parent bridge 当前剩余 residual，并选择下一枚 child import pocket 或进入 root residual 处理；不得直接改写尚未建基线的 root bridge。

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
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001FJ-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. `runtime.mutation_import_pass stop_split: true`。
3. `mutation` import pass residual 为 0。
4. 上层 `runtime.parent_import_bridge` 仍需 BE-001FK-01 父叶残余判断。
5. 当前生产级 runtime parent bridge residual 仍有 `src/runtime/mod.rs`。
6. 下一步只能进入 BE-001FK-01。
7. 旧三叶暂停目标保持取消: `old_three_leaf_pause_target_cancelled`。
8. 进度报告指令保持丢弃: `progress_report_instruction_discarded`。

不得声称 runtime parent bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `465-runtime.mutation_import_pass第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.mutation_import_pass stop_split: true`。
3. 下一步固定为 BE-001FK-01 root parent bridge 父叶残余判断。
4. Rust / 治理 / 全量树门禁均通过。
