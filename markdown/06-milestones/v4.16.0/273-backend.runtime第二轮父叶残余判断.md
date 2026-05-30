# v4.16.0 backend.runtime 第二轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CI-01  
> 基准: `272-runtime.report_ops第二轮父叶残余判断.md`、`251-backend.runtime父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime stop_split: false`。`backend.runtime.routes` 与 `runtime.report_ops` 均已收口，但 `src/runtime/mod.rs` 仍直接持有 `runtime.evidence_health` handler 残余；下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CI-01 `backend.runtime` 第二轮父叶残余判断 | 父叶判断 |
| 规范矩阵 | 父叶停止条件、下一候选选择、禁止跳步 | 冻结 |
| 引导矩阵 | `root.backend.runtime` | 父叶继续细拆 |
| 模块树 | `runtime.evidence_health` | 下一候选 |

---

## 当前真实结构

已 closeout / 收口的直接子域:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`

父级 `src/runtime/mod.rs` 仍直接持有:

- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`

相关 route facade:

```text
src/backend/runtime/routes/evidence.rs
```

该 facade 已经收口为 route registration owner，但 handler owner 仍在 `src/runtime/mod.rs`。

---

## 残余判断

`backend.runtime` 当前仍不能设置 `stop_split: true`:

1. `get_runtime_evidence_health` 与 `cleanup_runtime_evidence` 仍是 route-facing public handler。
2. `runtime_report_status_counts` 是 evidence health response 的私有 helper，和 evidence health handler 同属一个白箱边界。
3. 这组 handler 已有清晰 route facade、状态读取面、persistence 读取面和 response contract，适合作为下一单子叶。
4. `clean_optional_filter`、`normalized_replay_options`、`RunInProgressGuard` 等 shared helper 暂不作为本轮下一候选，需等待具体 owner 清晰后再判。

因此:

```text
backend.runtime stop_split: false
next: BE-001CJ-01 runtime.evidence_health 单子叶等价基线
```

---

## 明确排除

- 不处理 `runtime.report_ops`，该父叶已 closeout。
- 不处理 `backend.runtime.routes`，该父叶已 closeout。
- 不迁移 `get_runtime_evidence_health`、`cleanup_runtime_evidence` 或 `runtime_report_status_counts`。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner。
- 不处理 shared helpers 或 state/persistence boundary。
- 不启动 release transition guard。

---

## 验证要求

本批为 `no code movement` 父叶判断，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_v1_ops_health
cargo test -p quantpilot --test api_v1_reports
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CJ-01 runtime.evidence_health 单子叶等价基线
```

BE-001CJ-01 只能冻结 evidence health / cleanup 两个 handler 与 `runtime_report_status_counts` helper 的白箱边界。不得直接创建 child 文件、不得迁移 handler、不得修改 schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CI-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime.routes` 与 `runtime.report_ops` 均已 closeout。
3. `backend.runtime stop_split: false`。
4. `src/runtime/mod.rs` 仍直接持有 `get_runtime_evidence_health`、`cleanup_runtime_evidence` 与 `runtime_report_status_counts`。
5. 下一步只能进入 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
6. `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、shared helpers 和 release transition guard 均未迁移。

不得宣称 backend 顶层完成、runtime handler 整理完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `273-backend.runtime第二轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 下一候选固定为 BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线。
4. 本批保持 `no code movement`。
