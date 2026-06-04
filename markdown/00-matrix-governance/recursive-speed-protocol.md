# 递归高速执行协议

> Protocol: recursive_speed_protocol
> Scope: v4.16+ 递归模块化抽离、整理、等价 closeout。
> Owner: 三矩阵治理层。
> Decision: 高速执行协议独立维护于治理层，不再作为 v4.16 批次文档混入递归流水。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | 递归执行节奏、轻量两段式、同构叶批处理、同父级子叶并行、状态游标 | 提速 |
| 规范矩阵 | pre-commit 分流、父子通信、leaf split gate、批处理/并行边界 | 收紧 |
| 引导矩阵 | 全量树、模块树、递归状态游标、治理生成器 | 扩展 |

本协议只降低重复劳动和无效等待，不降低等价证明、父子通信、禁止横向连接、发布过渡保护和 leaf split 判定强度。

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

## 4. 递归治理生成器

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
