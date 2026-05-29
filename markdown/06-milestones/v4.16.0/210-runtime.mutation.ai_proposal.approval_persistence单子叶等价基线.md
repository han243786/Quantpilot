# v4.16.0 runtime.mutation.ai_proposal.approval_persistence 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BJ-01  
> 基线: `209-runtime.mutation.ai_proposal第五轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线，冻结 approval record 的磁盘写入、磁盘读取、not_found 错误、serde decode 错误和父级调用边界。当前 `no code movement`，下一步只能进入 BE-001BJ-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BJ-01 approval persistence 单子叶等价基线 | 递归进入子叶 |
| 规范矩阵 | 父子通信、磁盘持久化等价、closed child 不横连 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_persistence` | 新增白箱候选 |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence` | `stop_split: pending` |

---

## 目标白箱

```text
root.backend.runtime.mutation.ai_proposal.approval_persistence
```

当前目标仍在父文件中:

```text
src/runtime/mutation/ai_proposal.rs
```

计划目标文件为:

```text
src/runtime/mutation/ai_proposal/approval_persistence.rs
```

该文件只能在 BE-001BJ-03 实际抽离时创建。BE-001BJ-01 不创建文件、不移动代码、不改 handler、不改测试。

---

## 当前函数边界

| 函数 | 当前位置 | 输入 | 输出 | 等价约束 |
| --- | --- | --- | --- | --- |
| `persist_approval` | `src/runtime/mutation/ai_proposal.rs` | `&FsPath` store dir、`&RuntimeApprovalRecord` | `std::io::Result<()>` | 必须先 `fs::create_dir_all`，再按 `approval.approval_id` 写入同名 JSON |
| `load_approval_from_disk` | `src/runtime/mutation/ai_proposal.rs` | `&FsPath` store dir、`approval_id: &str` | `Result<RuntimeApprovalRecord, (StatusCode, String)>` | 必须按 `approval_id` 读取同名 JSON；缺失映射为 `json_bad_request("not_found", ...)` |

冻结的真实读写语义:

1. store dir 来源继续是 `approval_store_dir`，类型边界继续是 `FsPath`。
2. 写入路径继续是 `store_dir.join(format!("{}.json", approval.approval_id))`。
3. 写入前继续执行 `fs::create_dir_all(store_dir).await?`。
4. 写入 helper 继续调用 `crate::runtime_persistence::atomic_write_json(&file_path, approval).await`，不得绕过 runtime persistence owner。
5. 读取路径继续是 `store_dir.join(format!("{}.json", approval_id))`。
6. 读取 helper 继续使用 `fs::read(&file_path).await`。
7. 缺失审批单继续返回 `json_bad_request("not_found", format!("审批单 '{}' 不存在", approval_id))`。
8. JSON decode 继续使用 `serde_json::from_slice(&json)`。
9. decode 错误继续映射为 `internal_error(anyhow::anyhow!("{}", error))`。

---

## 父级调用边界

`approval_persistence` 后续只能由父级 `runtime.mutation.ai_proposal` 连接。当前调用方冻结如下:

| 调用点 | 调用函数 | 等价要求 |
| --- | --- | --- |
| proposal create orchestration | `create_runtime_ai_proposal` -> `persist_approval` | 创建审批单后持久化路径不变 |
| approval review child | `approval_review` 通过 `use super::*` 调用 `persist_approval` 与 `load_approval_from_disk` | sibling 不得横向直连 planned persistence child |
| sandbox background task | `load_sandbox_report_for_proposal` / create path task 间接更新 approval 后调用 `persist_approval` | sandbox_report_url 回写和失败 lifecycle 持久化不变 |

`src/runtime/mutation/ai_proposal/approval_review.rs` 继续只能经 `use super::*` 访问父级受控 helper。后续即使创建 `approval_persistence` child，也不得让 `approval_review` 直接横向 import sibling。

---

## 非目标边界

BE-001BJ-01 不迁移、不改写、不重排以下节点:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `sandbox_trigger`
- `status_transition`
- `AppState`
- schema owner
- frontend caller
- route facade
- runtime persistence owner
- release transition guard

不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 或 `approval_review`。不得宣称 Rust backend 重构完成。

---

## 下一步

下一步固定为:

```text
BE-001BJ-02 runtime.mutation.ai_proposal.approval_persistence 抽离方案
```

BE-001BJ-02 只能建立抽离方案，固定目标文件、父级 path-attributed child、helper import / visibility、允许迁移清单、回退点与验证门禁。实际创建 `approval_persistence.rs` 必须等到 BE-001BJ-03。

---

## 验证计划

本批 `no code movement`，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BJ-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.approval_persistence` 单子叶等价基线，尚未创建 `approval_persistence.rs`，也尚未迁移 `persist_approval` 或 `load_approval_from_disk`。不得宣称 approval persistence 已抽离、sandbox_trigger 已迁移、status_transition 已迁移、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.approval_persistence` 白箱候选，状态为 `stop_split: pending`。
3. 本批不产生 Rust 代码变更，不创建 `approval_persistence.rs`。
4. 下一步固定为 BE-001BJ-02 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案。
