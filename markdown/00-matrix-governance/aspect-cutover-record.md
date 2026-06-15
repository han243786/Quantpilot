# 切面打磨裁剪记录

> Protocol: aspect_cutover_record
> Version: aspect-cutover-record-v1
> Scope: 切面打磨模式进入裁剪阶段时的强制记录。
> Owner: 三矩阵治理层。

本文件是 `aspect-polish-protocol.md` 的运行层补丁。它不替代切面打磨协议，只负责把“裁剪是否可执行”变成可检查记录。

---

## 当前状态

```yaml
aspect_polish_cutover:
  status: none
  source_slice: none
  mirror_slice: none
  frozen_interfaces:
    - none
  shared_state: none
  migration_needed: false
  dual_run_required: false
  feature_flag: none
  old_path_retirement: none
  rollback_plan: none
  cutover_gate: none
  cutover_result: not_started
```

---

## 使用规则

1. `status: none` 表示当前没有进行中的切面裁剪。
2. 进入裁剪前必须将 `source_slice`、`mirror_slice`、`frozen_interfaces`、`rollback_plan` 和 `cutover_gate` 填为真实值。
3. `shared_state` 不是 `none` 时必须升为重型切面。
4. `migration_needed: true` 或 `dual_run_required: true` 时，必须补迁移计划或双跑证明。
5. `cutover_result: passed` 前不得删除旧路径或旧当前态治理资产。

---

## Closeout 最小证据

```text
aspect_cutover_record:
  source_slice:
  mirror_slice:
  frozen_interfaces:
  independent_run:
  old_path_retirement:
  rollback_plan:
  cutover_gate:
  cutover_result:
```
