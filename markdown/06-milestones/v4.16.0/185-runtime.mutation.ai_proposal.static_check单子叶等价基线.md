# v4.16.0 runtime.mutation.ai_proposal.static_check 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AZ-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`、`src/frontend_api_types.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。当前只冻结 hash identity、model identity、static check result、v4 target detection、config domain binding、v4 artifact analysis 与测试证据；本批 `no code movement`。下一步只能进入 BE-001AZ-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AZ-01 AI proposal static check 单子叶等价基线 | 扩展 |
| 规范矩阵 | 低副作用 validation / analysis helper、父子通信、release transition guard | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.static_check` | 新增白箱节点 |
| 模块树 | `runtime.mutation.ai_proposal.static_check` | 建立单子叶基线 |

---

## 选择理由

`runtime.mutation.ai_proposal.static_check` 是 `runtime.mutation.ai_proposal` closeout 后的第一候选:

1. 该区域由纯 validation / analysis helper 组成，副作用低，适合先抽离。
2. 它直接决定 `RuntimeAiProposalStaticCheckResult` 的 `StaticCheckPassed` / `StaticCheckFailed` 输出，是 AI proposal create flow 的第一层幻觉发现点。
3. `tests/api_ai_proposal.rs` 和 child 内部单测已经覆盖缺少 config binding、v4 source_kind 不匹配、binding 匹配通过与 v4 artifact analysis。
4. 它不拥有 approval review、proposal persistence、state owner、route facade 或 frontend caller，可作为明确白箱节点。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.ai_proposal.static_check` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.ai_proposal.static_check` |
| 父模块 | `runtime.mutation.ai_proposal` |
| 当前真实文件 | `src/runtime/mutation/ai_proposal.rs` |
| route 入口 | `backend.runtime.routes.mutation` |
| handler owner | `runtime.mutation.ai_proposal` |
| schema owner | `src/frontend_api_types.rs` |
| 测试证据 | `tests/api_ai_proposal.rs` + child 内部单测 |
| 下一批次 | BE-001AZ-02 抽离方案 |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `CreateRuntimeAiProposalRequest` | `create_runtime_ai_proposal` | 保持 source_kind、target、reason、model、config_domain_binding |
| 输入 | `old_parameter_version` | `canonical_runtime_parameter_version` | 不改变旧参数版本规范化来源 |
| 输入 | `proposed_parameter_version` | `canonical_runtime_parameter_version` | 不改变新参数版本规范化来源 |
| 输入 | `source_event_count` | run/backtest source context | 0 必须产生 `missing_source_evidence` |
| 输入 | `checked_at_ms` | `current_time_ms` | 不改变 static check 时间来源 |
| 输入 | `V4BacktestArtifact` | v4 backtest artifact owner | 只做 analysis summary，不拥有 artifact schema |
| 输出 | `RuntimeAiProposalStaticCheckResult` | create flow / proposal record | 不改变 status、reason_code、message、checked_at_ms、details |
| 输出 | config binding details | static check details | 不改变 code、target、message |
| 输出 | v4 analysis summary | tests / future evidence | 不改变 `analysis_version`、state / machine counts、risk ratio、fill rate |

---

## 冻结 helper 清单

本基线冻结以下 helper，不移动代码:

- `validate_hash_identity`
- `is_valid_hash_identity`
- `validate_ai_model_identity`
- `ai_proposal_static_check_result`
- `is_v4_ai_proposal_target`
- `expected_config_domain_for_target`
- `validate_ai_proposal_config_domain_binding`
- `analyze_v4_backtest_artifact_for_ai`

---

## 行为基线

| 子域 | 当前函数 | 等价约束 |
| --- | --- | --- |
| hash identity | `validate_hash_identity`、`is_valid_hash_identity` | 保持 `sha256:<64位小写十六进制>` 格式要求和 bad_request error mapping |
| model identity | `validate_ai_model_identity` | 保持 provider/model/model_version 必填 |
| static check aggregate | `ai_proposal_static_check_result` | 保持 missing source、noop version、missing reason、config binding、v4 backtest source 与 non-v4 run source 判断 |
| target detection | `is_v4_ai_proposal_target` | 保持 `module_key` 或 `parameter_path` 以 `v4.` 开头即为 v4 target |
| config domain | `expected_config_domain_for_target` | 保持 builtin / v4 module 到 strategy config proposal domain 的映射 |
| config binding | `validate_ai_proposal_config_domain_binding` | 保持 binding 必填、target_domain、before/after digest、old/new version、evidence anchors 校验 |
| artifact analysis | `analyze_v4_backtest_artifact_for_ai` | 保持 machine trajectory counts、risk decision ratio、fill rate 与 analysis version |

`RuntimeAiProposalStaticCheckDetail` code 基线:

- `missing_source_evidence`
- `noop_parameter_version`
- `missing_reason`
- `strategy_config_ai_binding_required`
- `strategy_config_ai_binding_domain_mismatch`
- `strategy_config_ai_binding_before_digest_invalid`
- `strategy_config_ai_binding_after_digest_invalid`
- `strategy_config_ai_binding_before_digest_mismatch`
- `strategy_config_ai_binding_after_digest_mismatch`
- `strategy_config_ai_binding_evidence_required`
- `v4_proposal_requires_backtest_artifact`
- `non_v4_proposal_requires_run_source`

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> runtime.mutation.ai_proposal
  -> runtime.mutation.ai_proposal.static_check
```

后续若创建 `static_check` child，只允许父叶 `runtime.mutation.ai_proposal` 调用；不得横向接管 approval_review、record_query、source_governance_identity、event_lifecycle、sandbox_trigger、parameter mutation、AppState、schema owner、frontend caller 或 runtime persistence owner。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 本批不做

- 不创建 `src/runtime/mutation/ai_proposal/static_check.rs`。
- 不移动 `validate_hash_identity`、`ai_proposal_static_check_result` 或其他 helper。
- 不拆 `create_runtime_ai_proposal`。
- 不迁移 approval review、record query、source governance、event lifecycle、sandbox trigger、AppState、schema owner、frontend caller 或 route facade。
- 不启动发布过渡。

---

## 未来抽离决策点
| 决策点 | 当前默认 | 原因 |
| --- | --- | --- |
| 目标文件路径 | BE-001AZ-02 决定 | 倾向 `src/runtime/mutation/ai_proposal/static_check.rs`，但需先固定父级声明、visibility 与回退点 |
| helper visibility | BE-001AZ-02 决定 | `ai_proposal_static_check_result` 需被 create flow 调用，artifact analysis 目前只被测试覆盖，需明确是否 `pub(super)` |
| child 内部单测归属 | 跟随 helper 迁移 | 当前相关内部单测已随 AI proposal child 归位，后续应随 static_check helper 迁移 |
| `analyze_v4_backtest_artifact_for_ai` 是否同批迁移 | 倾向同批 | 它是 low-side-effect v4 analysis helper，属于 static check / evidence analysis 语义 |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | no code movement 无格式漂移 |
| `cargo check -p quantpilot` | Rust 类型 | parent / child 可见性未漂移 |
| `cargo test --no-run` | 测试编译 | static check 内部单测仍可编译 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal API | static check pass/fail、capability denial、contract fields 不漂移 |
| `cargo test -p quantpilot --test api_mutation` | 邻接 parameter mutation | shared helper owner 不被误伤 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effects | proposal evidence/report 不漂移 |
| `cargo test -p quantpilot --test api_run` | run/backtest source 邻接域 | source context lookup 不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增基线保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 基线、模块树、全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新基线和真实文件可定位 |
| `git diff --check` | diff whitespace | 本批没有空白错误 |

---

## 幻觉检查点

AI 声称 BE-001AZ-01 完成时，必须说明本批只建立 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线，并且为 `no code movement`。不得宣称 static_check helper 已迁移、目标文件已创建、approval_review / record_query 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.static_check` 白箱节点，包含输入输出、helper 清单、父子通信和排除边界。
3. 治理门禁能发现本基线、`no code movement`、下一批 BE-001AZ-02、关键 helper、禁止迁移边界和验证门禁缺失。
4. 本批验证通过后，后续才能进入 BE-001AZ-02 抽离方案。
