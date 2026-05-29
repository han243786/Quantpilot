# v4.16.0 runtime.mutation.ai_proposal 第五轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BI-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`188-runtime.mutation.ai_proposal.static_check单叶closeout.md`、`193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`、`198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`、`203-runtime.mutation.ai_proposal.record_query单叶closeout.md`、`208-runtime.mutation.ai_proposal.approval_review单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 与 `approval_review` 均已 closeout 并设置 `stop_split: true`，但父叶仍存在 approval_persistence、sandbox_trigger、status_transition 与 proposal create orchestration 等稳定残余职责，父叶继续保持 `stop_split: false`。下一步只能进入 BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BI-01 AI proposal 第五轮父叶残余判断 | 递归回到父叶 |
| 规范矩阵 | closed child 不回收、父级残余排序、禁止发布过渡 | 约束收紧 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 残余职责排序 |
| 模块树 | `runtime.mutation.ai_proposal` | `stop_split: false` |

---

## 已 closeout 子叶

| 子叶 | 状态 | 判定 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check` | BE-001AZ-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.source_governance_identity` | BE-001BB-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.event_lifecycle` | BE-001BD-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.record_query` | BE-001BF-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.approval_review` | BE-001BH-04 closeout | `stop_split: true`，不继续细拆 |

---

## 当前父叶残余

`src/runtime/mutation/ai_proposal.rs` 当前仍直接承接以下稳定职责:

| 残余职责 | 当前函数 / 入口 | 是否值得继续拆 | 原因 |
| --- | --- | --- | --- |
| proposal create orchestration | `create_runtime_ai_proposal` | 后续候选 | create flow 仍是事务编排 owner，当前先不混入 persistence / sandbox / status helper |
| approval_persistence | `persist_approval`、`load_approval_from_disk` | 是，下一候选 | approval record disk read/write helper 形成低副作用 persistence owner，可先白箱化，供 approval_review 与 sandbox trigger 继续经父级受控调用 |
| sandbox_trigger | `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 与 create path background sandbox task | 是 | sandbox gate、retry、JoinHandle monitoring、sandbox_report_url 回写有独立外部证据，但应等 approval persistence 边界稳定后再判断 |
| status_transition | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` | 是 | 状态迁移 guard 与 proposal lifecycle update 是独立状态机残余，可在 persistence / sandbox 边界稳定后再判断 |

---

## 下一候选判定

下一候选固定为:

```text
BE-001BJ-01 runtime.mutation.ai_proposal.approval_persistence 单子叶等价基线
```

选择 `approval_persistence` 的原因:

1. `persist_approval` 与 `load_approval_from_disk` 只围绕 `RuntimeApprovalRecord` 的磁盘写入 / 读取，职责集中。
2. 本候选副作用边界低于 sandbox trigger，不涉及 background task、retry 或 JoinHandle monitoring。
3. 本候选能先把 approval record disk fallback、atomic write、serde decode 和 `approval_store_dir` 归属冻结，降低后续拆 sandbox/status 时的依赖噪音。
4. 后续实际抽离时只能由父级 `runtime.mutation.ai_proposal` 连接 `approval_persistence` child；`approval_review` 不得横向直接连接 sibling。
5. 本候选不改变 `AppState`、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 非目标边界

BE-001BI-01 不移动代码，也不创建 `approval_persistence.rs`。后续 BE-001BJ-01 也只能建立等价基线，不得直接创建目标文件。

当前不得迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `approval_persistence`
- `sandbox_trigger`
- `status_transition`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 或 `approval_review` 已 closeout 子叶。

---

## 验证计划

本批 `no code movement`，只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BI-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal` 第五轮父叶残余判断，`approval_persistence` 尚未建立基线也尚未抽离。不得宣称 `runtime.mutation.ai_proposal` 父级完成、approval_persistence 已拆分、sandbox_trigger 已迁移、status_transition 已迁移、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `209-runtime.mutation.ai_proposal第五轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树继续将 `runtime.mutation.ai_proposal` 标记为 `stop_split: false`。
3. 下一候选固定为 BE-001BJ-01 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线。
4. 本批不产生代码变更，也不回收已 closeout 子叶。
