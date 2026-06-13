# 递归高速执行协议

> Protocol: recursive_speed_protocol
> Version: recursive-high-speed-v2
> Scope: v4.16+ 递归模块化抽离、整理、等价 closeout。
> Owner: 三矩阵治理层。
> Decision: 高速执行协议独立维护于治理层，不再作为 v4.16 批次文档混入递归流水。
> Effective: GOV-RECURSIVE-COST-CONTROL-01 起生效；当前 Rust 递归游标不因本协议升级移动。
> Superseded by: `governance-next/05-authoritative-operating-model.md` at `GOV-GOVERNANCE-NEXT-PROMOTION-01`.

本文件保留为旧递归高速协议的兼容档案。promotion 后，递归重构默认由 QPCursor、治理热度和局部不变量驱动；本协议只能提供历史规则、旧提速术语和兼容门禁素材，不再覆盖新治理入口。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | 递归执行节奏、轻量两段式、同构叶批处理、同父级子叶并行、成本受控批次、状态游标 | 提速 |
| 规范矩阵 | pre-commit 分流、父子通信、leaf split gate、批处理/并行边界、强制降档触发器 | 收紧 |
| 引导矩阵 | 全量树、模块树、递归状态游标、治理生成器、批次白箱表 | 扩展 |

本协议只降低重复劳动和无效等待，不降低等价证明、父子通信、禁止横向连接、发布过渡保护和 leaf split 判定强度。

---

## 0. v2 成本受控高速层

cost_controlled_recursive_speed

v2 的核心变化是把“可验证步骤”从只能是单叶小步，扩展为同一父级下可审计的 `same_parent_wave`。批次提速只能压缩重复文档、重复游标更新和重复提交；不得压缩每个 child 的白箱边界、等价证据、门禁记录和 split 判定。

### 0.1 效果不退不变量

以下不变量优先级高于任何提速规则:

1. 父子通信规则保持硬规则；child 之间不得直接横向调用。
2. Public API、route、schema、persistence、lock owner、状态机语义和外部契约不得被隐式改变。
3. 每个 child 必须保留独立输入、输出、owner、排除范围、文件写集、等价面和 `leaf_split_decision_result`。
4. 每个 parent closeout 必须确认其所有 child 已关闭、残余 facade 合理、模块树和全量树同步。
5. 发布过渡保护不变；AI 不得主动提出 release transition，开发者明确进入发布过渡后才允许横向连接优化提案。

### 0.2 三档执行判定

| 执行档 | 触发条件 | 允许产物 | 必须门禁 |
| --- | --- | --- | --- |
| Precision single-leaf | 命中高风险触发器，或等价面无法共享 | 单叶 baseline、extract、closeout、parent residual 分开落文档和提交 | 受影响 crate test、`cargo check -p quantpilot`、治理门禁 |
| Standard same-parent wave | 同一 parent 队列已冻结，child 独立，write set 可分离，共享 gate 足以覆盖共同等价面 | 一个批次文档内包含每个 child 的独立白箱行、抽离行、closeout 行和 split 判定；一个 wave 提交 | shared gate + child extra gate、`cargo check -p quantpilot`、治理门禁 |
| Fast closeout | 无代码移动或极薄 facade，split gate 明确 `stop_split: true`，继续细拆只会增加通信成本 | 单个 closeout 文档或批次 closeout 行 | 文档门禁；若有 Rust 改动则补 Rust gate |

`same_parent_wave` 只有在每个 child 都能单独证明时才成立。缺少任一 child 行时，该批次不得 closeout。

### 0.3 强制降档触发器

命中任一条件时，必须从批次模式降回 Precision single-leaf:

1. 需要新增或改变 public API、route、schema、persistence owner、lock owner 或跨 crate 外部契约。
2. 涉及状态机推进顺序、并发锁顺序、交易执行语义、凭证安全、实时交易、沙箱回放真实性或编译契约。
3. 某 child 需要另一个 sibling 的私有 helper、私有类型、私有状态或测试夹具。
4. 共享 gate 不能覆盖某 child 的失败模式，且 child extra gate 不能在同一 wave 内清晰补足。
5. 父级 facade 合并无法维持单向 parent 调用，或失败会污染其他 child 的等价证明。
6. 需要 release transition 才能解释性能收益。

### 0.4 成本上限规则

为了防止治理成本失控，递归执行默认采用以下上限:

1. 同一 parent 下的同构轻量 child 优先合成一个 `same_parent_wave`，不再为每个 child 固定拆成多份独立 milestone。
2. 标准批次只允许一个主 milestone 文档；child 证据用表格和小节承载，除非触发 Precision single-leaf。
3. 游标只在 wave 完成、parent closeout 或决策分叉时更新；不得为纯粹中间提示反复更新。
4. 提交粒度以“可验证 wave step”为单位；不得把同一 wave 拆成无行为差异的多次文档提交。
5. 全量高成本门禁集中在 Rust 改动、parent closeout 和风险升档处；docs-only 游标或索引同步走文档门禁。

### 0.5 批次白箱表

每个 `same_parent_wave` 文档必须包含下列表格列:

| 列 | 含义 |
| --- | --- |
| child | 完整模块树坐标 |
| baseline boundary | 输入、输出、owner、排除范围 |
| write set | Rust 文件、父级 facade、治理文档 |
| movement | 抽离动作或无代码移动说明 |
| equivalence gate | shared gate 与 child extra gate |
| closeout decision | `stop_split` / `continue_split` 与理由 |
| residuals | 仍留在 parent 或转入下一轮的内容 |

### 0.6 末端叶子智能判定

leaf_granularity_smart_judge

只读审计发现: v4.16 递归已关闭大量 child，Rust 文件中位规模偏小，且历史 milestone 明显存在“单叶动作过度文档化”的成本信号。因此后续底层叶子不得再以“还能拆”为继续拆分理由，必须先通过末端叶子评分。

评分目标:

各指标先按 0-100 打分，再按权重计算:

`weighted_delta = 0.40*split_benefit + 0.20*leaf_size_fit - 0.20*risk_penalty - 0.15*governance_cost - 0.05*system_efficiency_penalty`

`normalized_split_score = clamp(40 + weighted_delta, 0, 100)`

`normalized_split_score` 是基础分数，但不再单独承担全部语义。GOV-GOVERNANCE-NEXT-OPTIMIZATION-01 起，脚本必须同时输出:

- `split_decision`: 是否继续细拆，只能是 `STOP` 或 `CONTINUE`。
- `governance_packaging`: 如何治理交付，只能是 `stop_split`、`same_parent_wave`、`standard_same_parent_wave` 或 `precision_single_leaf`。
- `final_decision`: 兼容旧记录的 `STOP / WAVE / SPLIT / PRECISION`。

`40` 是中性基线，用于保留既有阈值并让负向成本可以直接把微叶压回 STOP。`WAVE` 只表示治理打包方式，不得单独解释为继续拆分许可。

| 指标 | 权重 | 说明 |
| --- | --- | --- |
| split_benefit | 40 | 独立 owner、清晰输入输出、可独立测试、降低父级复杂度、减少耦合或共享状态 |
| leaf_size_fit | 20 | 当前体量是否形成继续拆分压力；低于 100 LOC 为低分，超过 800 LOC 为高分，150-600 LOC 默认只给中低分以鼓励终端叶稳定。 |
| risk_penalty | 20 | public API、route/schema、状态机、交易语义、持久化、锁、安全、live execution、跨 crate 或编译契约风险 |
| governance_cost | 15 | 新增文档、facade、门禁、索引、提交和证明成本 |
| system_efficiency_penalty | 5 | 是否增加薄包装、无价值 re-export/import、parent-child 转发层或可读性损耗 |

判定输出:

决策集合固定为 `STOP / WAVE / SPLIT / PRECISION`。

| 决策 | 条件 | 执行动作 |
| --- | --- | --- |
| STOP | `normalized_split_score < 40`，或 LOC < 100 且无强收益 | 停止细拆，登记 `stop_split: true` |
| WAVE | `normalized_split_score` 40-64，且同父级存在同构 child | `split_decision=STOP`，合入 `same_parent_wave` 或仅作为父级 wave 证据，不单独走完整治理 |
| SPLIT | `normalized_split_score >= 65` 且未命中高风险触发器 | 允许继续拆分，优先走标准批次 |
| PRECISION | `normalized_split_score >= 65` 且命中高风险触发器，或 `LOC > 800` 且命中状态机、交易语义、安全、live execution、compiler contract、持久化、route/schema 或锁风险 | 单叶精细治理，不得批处理隐藏风险；先做 precision baseline，再决定是否抽离 |

叶子体量控制:

1. 普通逻辑叶推荐终端区间为 150-600 LOC 或一个完整业务事务。
2. LOC < 100 的 child 默认 `STOP`，除非能证明拆分会显著降低父级复杂度或隔离独立失败模式。
3. LOC < 200 且治理成本高时默认 `WAVE` 或 `STOP`，不得创建独立四段式 milestone。
4. LOC > 800 且复杂度、分支、public surface 或状态 owner 明显增大时，才优先评估 `SPLIT`；若同时命中 G4/G5 风险标签，必须升为 `PRECISION`。
5. 单函数、单 helper、薄 accessor、纯 re-export、薄 facade、DTO pocket 默认不得继续细拆；public contract、交易状态机、安全凭证、持久化、route/schema、live execution、compiler lowering 例外。

评分记录至少包含:

| 字段 | 含义 |
| --- | --- |
| leaf_id | 完整模块树坐标 |
| loc | 当前代码规模 |
| function_count | 函数/方法数量 |
| public_surface | public 或 pub(crate) surface |
| complexity_score | 分支、状态、错误路径和事务复杂度 |
| dependency_score | parent/sibling/cross-crate 依赖压力 |
| split_benefit | 拆分收益分 |
| governance_cost | 治理成本分 |
| risk_score | 风险分 |
| split_decision | STOP / CONTINUE |
| governance_packaging | stop_split / same_parent_wave / standard_same_parent_wave / precision_single_leaf |
| final_decision | STOP / WAVE / SPLIT / PRECISION |
| reason | 主要理由 |

自动评估入口:

`tools/evaluate-leaf-granularity.ps1` 是只读评分脚本。它读取一个或多个候选叶子文件，统计 LOC、函数数、public surface、分支密度、领域关键词、风险标签、治理成本和系统效率损耗，并输出 `normalized_split_score`、`split_decision`、`governance_packaging` 与兼容旧记录的 `final_decision`。

示例:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\evaluate-leaf-granularity.ps1 `
  -LeafId root.contracts.runtime_support.data_module.exchange_surface_wave `
  -ParentId root.contracts.runtime_support.data_module `
  -Path qrpc_runtime/src/data_module/exchange_surface.rs `
  -Depth 4 `
  -SameParentWaveCandidate
```

脚本结果不替代开发者判断，但后续单叶 closeout / 父叶残余判断若不采用脚本结果，必须在 milestone 中说明人工覆盖理由。

### 0.7 末端叶子体量控制 v2

terminal_leaf_control_v2

只读判断进一步确认: 底层叶子过细会同时抬高治理成本、parent-child 转发成本、索引同步成本和人类复核成本。因此 leaf split gate 必须把“继续拆分收益”与“治理负担”放在同一张秤上，而不是只看代码还能不能再切。

`tools/evaluate-leaf-granularity.ps1` 必须输出 `terminal_leaf_control` 区块，并至少包含:

| 字段 | 含义 |
| --- | --- |
| `target_terminal_loc_range` | 普通逻辑叶推荐终端区间，当前固定为 `150-600` |
| `size_bucket` | `micro_under_100` / `small_100_149` / `terminal_target_150_600` / `split_pressure_601_800` / `oversized_over_800` |
| `governance_mode` | `stop_split` / `same_parent_wave` / `standard_same_parent_wave` / `precision_single_leaf` |
| `split_decision` | `STOP` / `CONTINUE`，表示是否继续拆分 |
| `governance_packaging` | 当前证据应该如何落地，避免把 WAVE 误读成继续拆分许可 |
| `final_decision` | 兼容旧 milestone 的 STOP/WAVE/SPLIT/PRECISION |
| `standalone_full_governance_allowed` | 只有 `precision_single_leaf` 可以为 true；普通 SPLIT 优先并入同父级标准 wave |
| `micro_leaf_default_stop` | 微叶默认停止细拆，除非开发者提供强 owner 收益证据 |
| `high_cost_leaf_needs_wave_or_stop` | 小叶且治理成本高时只能走 WAVE 或 STOP，不得创建独立四段式治理 |

治理模式硬规则:

1. `STOP` -> `governance_mode=stop_split`，只登记 closeout 与残余判断，不再拆出 baseline/extract/closeout 三连文档。
2. `WAVE` -> `governance_mode=same_parent_wave`，同父级同构子叶合并为一批，但每个 child 仍保留白箱行和等价证据。
3. `SPLIT` -> `governance_mode=standard_same_parent_wave`，允许继续拆，但默认不得独立四段式；优先在同父级 wave 内推进。
4. `PRECISION` -> `governance_mode=precision_single_leaf`，只有命中 public contract、route/schema、状态机、持久化、锁、安全、live execution、compiler contract 等高风险面时才使用完整单叶治理。
5. 若人工覆盖脚本建议，milestone 必须写明: 覆盖前决策、覆盖后决策、强 owner 收益证据、风险隔离收益、额外门禁。

### 0.8 治理生成与索引降重

governance_generation_and_index_reduction

GOV-GOVERNANCE-NEXT-PROMOTION-01 起，递归治理允许把重复登记从手写迁移为生成器辅助；生成物必须通过新治理 evidence 和兼容门禁。

1. `tools/new-qpcursor-trial.ps1` 可从 `recursive-state.json` 生成 `governance-next/trials/*-qpcursor.md` 草案。
2. QPCursor 草案必须由代理补齐 allowed workset、evidence、governance heat、local invariants，并明确 `legacy_governance_mode: archived_reference`。
3. `check-full-feature-tree.ps1` 必须覆盖 `governance-next`，并检查未跟踪的活跃源码/文档文件是否已被全量树登记。
4. 新增文件不能只依赖 staged 视角；未跟踪文件若属于活跃源码、脚本、配置或治理文档，也必须被提示覆盖。
5. 长期路线: 将 README、docs index、roadmap、module-tree supplement 中重复的 append-only 行收敛为一个权威事件流，其它索引改为脚本生成或周期性同步。旧索引在替代生成器落地前继续作为兼容索引保留。

---

## 1. 智能 pre-commit

smart_pre_commit

`scripts/pre-commit` 调用 `tools/run-smart-pre-commit.ps1`。脚本读取 staged files，并按改动类型决定门禁组合。

| 模式 | 触发 | 必跑 | 默认跳过 |
| --- | --- | --- | --- |
| docs-only | staged files 全部为 markdown / md / txt | diff check、UTF-8、full-feature-tree、matrix governance | cargo、frontend build、vitest |
| rust-only | Rust / Cargo 改动且不含 frontend/tooling | diff check、UTF-8、cargo fmt、cargo check | frontend build、vitest、cargo test no-run |
| frontend-only | frontend / package / vite 改动且不含 Rust/tooling | diff check、UTF-8、frontend build、vitest | cargo |
| tooling | scripts / tools / CI 改动 | diff check、UTF-8、hook sync、governance gates、cargo fmt/check | frontend unless frontend changed |
| mixed | Rust 与 frontend 同批改动 | diff check、UTF-8、Rust gate、frontend gate | full no-run unless forced |
| full | `QUANTPILOT_PRECOMMIT_FULL=1` | legacy full gates | none |

可选环境变量:

- `QUANTPILOT_PRECOMMIT_FULL=1`: 强制全量 legacy gate。
- `QUANTPILOT_PRECOMMIT_SKIP_FRONTEND=1`: 临时跳过 frontend gate；不得用于 frontend 改动 closeout。
- `QUANTPILOT_PRECOMMIT_RUST_TEST="<command>"`: 为 Rust 改动追加 targeted test。

---

## 2. 轻量叶两段式

lightweight_two_step

轻量叶不再默认四段式。满足以下全部条件时，允许从四段式降为两段式:

1. 无 public API / route / schema / persistence / lock owner 变更。
2. 无状态机副作用或并发锁顺序变化。
3. 父级单向调用保持不变。
4. 等价点局部、清晰、可用 targeted test 或 compile gate 验证。
5. `leaf_split_decision_gate` 已明确判定允许轻量执行。

两段式固定为:

1. `baseline_plan`: 合并等价基线和抽离方案。
2. `extract_closeout`: 合并实际抽离记录和单叶 closeout。

禁止用两段式处理重型 handler、状态机、持久化、schema、锁、release transition 或跨模块 owner 迁移。

---

## 3. 同构叶批处理

homogeneous_leaf_batching

同一父叶下结构高度一致的叶子可在一个 batch 中收束多个 child，但每个 child 必须保留独立白箱表、独立 markers 和独立 `leaf_split_decision_result`。

允许条件:

1. 同一 parent。
2. 同类 match branch / import pocket / render helper。
3. 不共享可变状态。
4. 不引入 sibling horizontal link。
5. 同一 targeted test 或 compile gate 能覆盖共同等价面。
6. 每个 child 的下一步仍可单独回退。

强停止条件:

1. 任一候选命中 `communication_cost_rises`。
2. 任一候选需要新的 public API / route / schema / persistence owner。
3. 任一候选的失败模式与其他候选不同到无法共享证明。

---

## 4. 同父级子叶并行

same_parent_parallel_children

同一父级下多个子叶允许并行处理，但只允许在同一个已冻结 parent queue 内并行。该能力用于减少父叶残余判断、基线撰写、抽离记录和验证等待之间的串行空耗，不改变父子通信规则。

允许条件:

1. `same_parent_queue_frozen`: 所有候选子叶来自同一 parent，且 parent baseline / parent residual judgment 已冻结候选队列。
2. `independent_white_box`: 每个子叶都有独立白箱边界、输入输出、处理 owner、排除范围和 `leaf_split_decision_gate`。
3. `write_set_declared`: 每个子叶必须声明将写入的 Rust 文件、文档文件和父级 facade 文件。
4. `no_shared_mutable_state`: 候选之间不共享可变状态、锁顺序、持久化 owner、schema owner 或外部 API owner。
5. `parent_facade_lock`: 若多个子叶都需要改同一个父级 facade，child 文件准备可以并行，但父级 facade 合并必须由一个 parent coordinator 串行完成。
6. `no_sibling_horizontal_link`: 并行不允许引入 sibling horizontal link；所有通信仍经 parent。
7. `shared_gate_sufficient`: 同一 targeted gate 可以覆盖共同等价面；若某子叶需要额外 gate，必须在该子叶自己的证明中列出。

并行批次固定产物:

1. `parallel_wave_manifest`: 列出 parent、并行 child 列表、各 child write set、共享 parent facade lock、共享 gate 和各自 extra gate。
2. 每个 child 仍有独立 `baseline_plan`、`extract_closeout`、`single_leaf_closeout` 或在批次文档中有独立同名章节。
3. 每个 child 仍有独立 `leaf_split_decision_result` 和 `next_recursive_step`。
4. 提交可以按 child 分开提交，也可以按同一 parallel wave 合并提交；无论哪种方式，每个可验证 wave step 必须提交一次。

强停止条件:

1. 任一候选需要新增 public API / route / schema / persistence / lock owner。
2. 任一候选需要另一个 sibling 的内部 helper、私有类型或状态。
3. 任一候选改动失败会污染其他候选的等价证明。
4. 父级 facade 合并无法保持单向 parent 调用。
5. 需要 release transition 才能解释并行收益。

失败恢复:

1. 某个 child 门禁失败时，先从 parallel wave 移除该 child。
2. 其他 child 只有在 write set 和证明仍独立时才允许继续提交。
3. 失败 child 必须回到自己的 baseline 或 parent residual judgment，不得借其他 sibling 的 closeout 继续前进。

---

## 5. 递归治理生成器

recursive_governance_generator

`tools/update-recursive-governance.ps1` 用于创建递归 milestone skeleton 并同步常用索引:

- `markdown/06-milestones/v4.16.0/02-落地记录.md`
- `markdown/06-milestones/README.md`
- `markdown/10-overview/overview-docs-index.md`
- `markdown/10-overview/overview-current-status-and-roadmap.md`
- `markdown/10-overview/overview-full-feature-tree.md`
- `markdown/00-matrix-governance/module-tree.md`

该工具默认 preview，只有传入 `-Apply` 才写文件。生成器不替代判断，只减少重复同步劳动。

---

## 6. 递归状态游标

recursive_state_cursor

`markdown/00-matrix-governance/recursive-state.json` 记录当前递归游标:

- current parent
- current step
- current phase
- closed children
- open residuals
- next recommended child
- allowed speedups
- forbidden carryover prompts

该文件用于恢复上下文和防止一次性问题混入递归。它不是完成证明；每个 batch 仍必须用 milestone 文档、模块树和门禁结果证明。

---

## 不变硬规则

1. parent-child communication rule 保持硬规则。
2. sibling horizontal link 仍禁止。
3. AI 不得主动提出 release transition。
4. 只有开发者明确指出 release transition 时，才能提出子模块横向连接优化。
5. `leaf_split_decision_gate` 必须在后续单叶 closeout / 父叶残余判断中触发。
6. 每个可验证步骤仍需要提交；只是允许轻量叶合并阶段、同构叶批处理、同父级子叶并行和智能门禁分流。
7. 批次可以成为可验证步骤，但必须保留每个 child 的独立白箱行、独立 split 判定、独立残余说明和独立失败隔离路径。
8. 任何提速规则与效果不退不变量冲突时，必须立即降档到 Precision single-leaf。

---

## 门禁分层

| 场景 | 最小门禁 | 升档门禁 |
| --- | --- | --- |
| docs-only 协议、索引、游标同步 | `git diff --check`、UTF-8、full-feature-tree、matrix governance | 若触及 pre-commit 或生成器，再补 hook sync / generator dry run |
| 标准 Rust wave | `cargo fmt --check`、affected crate check/test、`cargo check -p quantpilot`、文档门禁 | 若涉及 API、状态机、持久化或锁，升到 Precision single-leaf |
| parent closeout | 受影响 crate test、`cargo check -p quantpilot`、模块树 / 全量树 / 矩阵治理 | 若 parent 是跨 crate 契约，再补相关 crate test |
| release transition | 仅开发者明确声明后可进入 | 必须新增 release-transition 证明，不得由本协议自动触发 |

---

## 验证命令

本协议变更必须通过:

1. `git diff --check`
2. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
3. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
4. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
5. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1`
6. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\run-smart-pre-commit.ps1`
7. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\update-recursive-governance.ps1 -Number 999 -FileSlug dry-run -BatchId DRY-RUN -NodeId dry.run -StageType governance -Summary "dry run" -NextStep "none"`
8. `cargo fmt --check`
9. `cargo check -p quantpilot`
