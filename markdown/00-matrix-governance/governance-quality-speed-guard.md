# 治理质量与速率护栏

> Protocol: governance_quality_speed_guard
> Version: governance-quality-speed-guard-v1.1
> Scope: v4.16+ 所有重构、推进、切面打磨、文档治理和临时修复任务。
> Owner: 三矩阵治理层。

本文件只处理三件事:

1. 清理旧路径文档债，但不把旧债混进功能开发。
2. 把每次任务入口从“递归继续”改成“先判定工作模式”。
3. 防止轻量卡被滥用成逃避治理。

执行顺序固定为:

```text
先堵新债 -> 再防滥用 -> 最后分批清旧债
```

---

## 1. 旧路径文档债处理

旧路径文档债必须单独处理，不得混入推进、重构或切面打磨任务。

分类规则:

| 类型 | 处理方式 |
| --- | --- |
| 新增文件漏登记 | 当前任务内立即修复，不进入债务池 |
| 新增文件误登记 | 当前任务内立即修复，不进入债务池 |
| 历史旧路径引用 | 进入 debt ledger，按批次清理 |
| 文档引用指向真实断链能力 | 暂停并转为质量问题，不只当文档债 |

旧债清理批次只允许做:

```text
路径确认 -> 删除/替换/归档说明 -> 跑门禁 -> closeout
```

旧债清理批次不得顺手改代码；除非确认文档指向的能力本身已经断裂。

### 1.1 当前已知旧路径债

以下旧债来自当前 `check-full-feature-tree.ps1` 和 `check-matrix-governance.ps1` 的缺失路径报告，状态为 `legacy_doc_debt`:

| Debt ID | 缺失路径 | 当前处理 |
| --- | --- | --- |
| GD-OLDPATH-001 | `tests/api_ai_proposal.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-002 | `src/runtime/run/session_start.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-003 | `src/runtime/backtest/legacy_dispatch.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-004 | `tests/api_backtest.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-005 | `tests/api_evidence_contract.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-006 | `tests/api_sse.rs` | 单独文档债批次确认替换或归档 |
| GD-OLDPATH-007 | `tests/api_mutation.rs` | 单独文档债批次确认替换或归档 |

### 1.2 冷文档降级与双巨树保护

以下文档已降级为低频兼容、历史或背景材料，不再作为默认任务入口，也不得在常规任务中被迫维护为高频控制面:

| 文档 | 当前定位 | 维护方式 |
| --- | --- | --- |
| `markdown/10-overview/overview-current-status-and-roadmap.md` | 当前摘要 + 历史路线 | 当前摘要保持紧凑；长历史增量应迁入 milestone、archive 或专门批次 |
| `markdown/10-overview/overview-docs-index.md` | 兼容索引 | 保留可发现性；不得把全部历史细节继续堆成默认入口 |
| `markdown/00-matrix-governance/landing-roadmap.md` | 历史落地路线 | 冻结为 v4.12-v4.16 落地背景；新任务不从它启动 |
| `markdown/10-overview/overview-system-architecture.md` | 架构背景手册 | 只作背景参考；当前接口、契约和门禁以专门协议或源码事实为准 |
| `markdown/00-matrix-governance/recursive-speed-protocol.md` / `recursive-state.json` | 重构模式兼容材料 | 只服务 refactor 历史递归；不得覆盖推进模式或切面打磨模式 |

双巨树保护规则:

1. `markdown/00-matrix-governance/module-tree.md` 和 `markdown/10-overview/overview-full-feature-tree.md` 是高频定位资产，不属于冷文档。
2. 文件体量本身不是拆分理由。只要代码或功能定位速度不受影响，且查询可通过搜索、索引或结构化锚点保持近似 `logn` 的定位效率，就不得因“大”而拆。
3. 只有命中以下任一条件，才允许提出拆分或索引化:
   - 明显拖慢代码、功能或 public 方法定位。
   - 导致门禁不稳定、误报显著增加或审查难以复现。
   - 路径反查、模块反查、owner 反查无法可靠完成。
   - 大文件合并冲突频繁阻断多人或多线程推进。
4. 拆分提案必须先证明“定位效率或门禁可靠性会提升”，不得只写“文件太大”。

---

## 2. 工作模式入口卡

每次任务必须先写或口头确认一个超轻量入口卡。用户只说“继续”时，也必须沿用当前已声明模式；当前模式不明时，先判定模式。

```text
work_mode:
reason:
allowed_scope:
exit_gate:
```

字段要求:

| 字段 | 要求 |
| --- | --- |
| `work_mode` | `refactor` / `advance` / `aspect_polish` / `doc_debt_cleanup` |
| `reason` | 为什么是该模式，而不是其他模式 |
| `allowed_scope` | 本轮允许触碰的代码、契约、测试、文档和治理资产范围 |
| `exit_gate` | 本轮最小收口门禁 |

硬规则:

1. 不得把“继续”默认解释为递归重构。
2. 模式不明时不得直接实现。
3. 局部跳转到其他模式时，必须记录返回条件。
4. 一次性用户问题、进度报告、临时测试请求不得混入长期递归状态。

---

## 3. 轻量卡防滥用

轻量卡只用于小切面、小 UI、小文案、小局部样式、小开发体验优化。

命中任意一条，禁止使用轻量卡，必须升为标准或重型:

1. 改接口、route、schema、event 或 capability。
2. 改状态、锁、持久化、事务、权限或安全边界。
3. 删除旧代码、旧入口、旧契约、旧测试或旧当前态文档。
4. 改模块边界、public owner、模块树当前态或父子通信规则。
5. 影响多个父级模块。
6. 改变测试语义，而不是只补测试或修测试描述。
7. 无法一句话写清 rollback。
8. 实际改动超出 `allowed_files` 或 `allowed_scope`。

轻量卡的最小验收必须包含:

```text
allowed_scope respected
acceptance passed
rollback declared
no higher-tier trigger hit
```

---

## 4. 质量与速率 closeout

每轮 closeout 必须补一句质量速率判断:

```text
quality_speed_guard: no_new_doc_debt / mode_entry_checked / lightweight_not_abused
```

如果任何一项不成立，必须说明:

1. 新债路径。
2. 实际工作模式。
3. 升档或补救计划。
