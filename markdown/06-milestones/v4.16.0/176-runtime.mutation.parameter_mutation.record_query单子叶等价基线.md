# v4.16.0 runtime.mutation.parameter_mutation.record_query 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AW-01  
> 基线: `175-runtime.mutation.parameter_mutation第二轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.record_query` 单子叶等价基线已建立；当前 `no code movement`，只冻结 list/detail 查询流的输入、输出、排序、filtering、scoped lookup、in-memory 优先级、persistence fallback 和 pagination 语义。下一步只能进入 BE-001AW-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AW-01 record_query 单子叶等价基线 | 基线 |
| 规范矩阵 | list/detail public handler、read model、scoped lookup、pagination | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.record_query` | 新候选叶子 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query` | 白箱边界 |

---

## 白箱边界

| 项 | 基线 |
| --- | --- |
| 父级 owner | `src/runtime/mutation/parameter_mutation.rs` |
| 候选目标文件 | `src/runtime/mutation/parameter_mutation/record_query.rs` |
| 候选 public handler | `list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail` |
| list 输入 | `State<AppState>`、`Query<RuntimeParameterMutationListQuery>` |
| detail 输入 | `auth::UserId`、`State<AppState>`、`Path<String>` |
| list 输出 | `Result<Json<PaginatedResponse<RuntimeParameterMutationRecord>>, (StatusCode, String)>` |
| detail 输出 | `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>` |
| 下一批次 | BE-001AW-02 抽离方案 |

本批不创建 `record_query.rs`，不移动 list/detail handler。

---

## list 查询等价顺序

`list_runtime_parameter_mutations` 必须保持:

1. `list_runtime_parameter_mutation_records(&state.mutation_store_dir)`
2. `.await.map_err(io_error)?`
3. `source_kind` 存在时按 `record.source_kind == source_kind` 过滤
4. `source_id` 经 `clean_optional_filter`
5. `source_id` 存在时按 `record.source_id == source_id` 过滤
6. 按 `created_at_ms` 倒序
7. 同一时间戳下按 `proposal_id` 倒序
8. 构造 `PaginationQuery { limit: query.limit, offset: query.offset }`
9. `paginate(records, pq)`
10. `Ok(Json(...))`

---

## detail 查询等价顺序

`get_runtime_parameter_mutation_detail` 必须保持:

1. `auth::scoped_key(&user_id, &proposal_id)`
2. 优先读取 `state.parameter_mutations`
3. in-memory 命中时 `cloned()` 并 `Ok(Json(record))`
4. 未命中时调用 `load_runtime_parameter_mutation_record`
5. `state.mutation_store_dir.as_ref()`
6. `.await.map(Json)`

---

## 未迁移边界

本批不得迁移:

- `create_runtime_parameter_mutation`
- `activate_runtime_parameter_mutation`
- `rollback_runtime_parameter_mutation`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

不得改变 `RuntimeParameterMutationRecord` response schema、pagination schema、route facade、runtime persistence owner、frontend route 或发布过渡连接。

---

## 父子通信基线

后续若进入实际抽离，通信必须保持:

```text
backend.runtime.routes.mutation
  -> src/runtime/mod.rs facade
  -> src/runtime/mutation/parameter_mutation.rs
  -> record_query::{list_runtime_parameter_mutations, get_runtime_parameter_mutation_detail}
  -> parent-owned imports / helpers via use super::*
```

`record_query` 不得被 route facade、AI proposal、approval review、AppState owner、schema owner、frontend caller 或发布过渡连接直接依赖。

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

## 幻觉检查点

AI 声称 BE-001AW-01 完成时，必须说明当前只是 `record_query` 单子叶等价基线，仍为 `no code movement`；目标文件尚未创建，list/detail handler 尚未迁移。下一步只能进入 BE-001AW-02 抽离方案。不得宣称 record_query 已抽离、`runtime.mutation.parameter_mutation` 父叶已经完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

---

## 验收标准

1. `176-runtime.mutation.parameter_mutation.record_query单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 冻结 list/detail 查询流的输入、输出、排序、filtering、scoped lookup、in-memory 优先级、persistence fallback 与 pagination 语义。
3. 本批无代码移动。
4. 后续只能进入 BE-001AW-02 抽离方案。
