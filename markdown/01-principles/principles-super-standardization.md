# 超级规范化 v4.7.0

> 生效日期: 2026-05-24 | 本文件是项目开发、检查、审计、优化的唯一执行标准
> 重构: v1.0→v3.0 精简三层流水线、元流水线自进化触发条件
> 更新: v3.4.0 新增 §8.4 | v3.7.0 新增 §8.5 自由维度诱错审计常态化 | v3.7.1 收口 pre-commit / CI / closeout 三层门禁 + 功能演进通道 + Rust 格式基线
> v4.0.0: MAJOR 演化通道已实施, 状态机化 QS/Risk Plane/ExecutionMachine 已落地, 前端以后端 capability 为真源, 开发者学习流水线 closeout 检查已接入
> v4.7.0: v4 高级订单/tick replay、LiveActual 安全边界、v4 AI 提案分析、OpenAPI/治理快照/全量树同步已接入
> v4.15.0: 本文件由三矩阵治理接管；变更入口先走 `../00-matrix-governance/README.md`，再引用本文件查证流程约束

---

## 一、总纲

QuantPilot 的开发过程受 **三层门禁流水线** 约束：

```
功能演进提案/登记 → 后端能力真源登记 → MAJOR演化终稿(如适用) → 分阶段开发 → 日常开发门禁(pre-commit) → PR/CI门禁 → AI并行审计(自由维度诱错) → 发布前检查单 → closeout/release
  │                         │                    │                              │                         │               │
  │                         │                    │                              │                         │               └── 五维度评分 + GP合规矩阵 + 学习流水线检查 + release dry-run
  │                         │                    │                              │                         └────────────────── 3角色/十角色手动验证
  │                         │                    │                              └────────────────────────────────────────── 11维度 AI并行诱错
  │                         │                    └──────────────────────────────────────────────────────────────────────────── 26项 closeout 门禁
  │                         └──────────────────────────────────────────────────────────────────────────────────────────────── v4演化边界/兼容桥/非目标
  └────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── 代码 + 测试 + 文档
```

### 版本号规则

| 层级 | 触发条件 | 示例 |
|------|---------|------|
| **PATCH** | S0修复、P1收敛、bug修复 | v2.3.2 → v2.3.3 |
| **MINOR** | 完整流水线周期结束（上述三层全通过） | v2.3.3 → v2.4.0 |
| **MAJOR** | 架构变更、协议 breaking change、多系统新增 | v2.4.0 → v3.0.0 |

### AI 辅助开发的决策边界

| 需要询问 | 不需要询问 |
|---------|-----------|
| 架构方案二选一或多选一 | 单文件内 bug 修复 |
| 涉及 ≥3 个文件的跨模块变更 | 文档格式修正 |
| API 路由或数据模型的 breaking change | GP 违规的明显修正 |
| 里程碑范围变更 (新增/删除优化项) | 门禁脚本执行 |
| 版本号升级 (MAJOR/MINOR) | PATCH 版本号递增 |
| 新增 npm/cargo 依赖 | 非功能性代码清理 |
| 存储或安全策略变更 | 测试用例修复 |

---

## 二、第一层：自动门禁

### 2.1 日常开发门禁：Pre-commit Hook

`scripts/pre-commit` 在 `git commit` 时自动执行 staged-file 智能分流：

```
powershell tools/run-smart-pre-commit.ps1
```

docs-only 默认只跑 diff / UTF-8 / full-feature-tree / matrix governance；rust-only 默认跑 cargo fmt 与 cargo check；frontend-only 默认跑 build 与 vitest；tooling 改动额外检查 hook sync。任何一步失败则提交被拒。

日常开发门禁只拦截明显破坏，避免把 E2E、audit、完整场景测试放进每次提交。

### 2.2 PR / CI 门禁矩阵

| # | 检查项 | 命令 | 阻断级别 |
|---|--------|------|:--:|
| 1 | UTF-8 编码 | `powershell tools/check-utf8.ps1` | 阻断 |
| 2 | 面向用户文本 | `powershell tools/check-user-facing-text.ps1` | 阻断 |
| 3 | 能力治理快照与后端真源对齐 | `powershell tools/check-capability-governance.ps1` | 阻断 |
| 4 | i18n 覆盖 | `powershell tools/check-i18n.ps1` | 阻断 |
| 5 | 版本号一致性 | `powershell tools/check-version-consistency.ps1` | 阻断 |
| 6 | 功能演进契约 | `powershell tools/check-feature-evolution.ps1` | 阻断 |
| 7 | 三矩阵治理 | `powershell tools/check-matrix-governance.ps1` | Closeout 阻断 |
| 8 | Developer Learning Closeout | `powershell tools/check-learning-closeout.ps1` | Closeout 阻断 |
| 9 | Pre-commit hook 同步 | `powershell tools/check-pre-commit-hook.ps1` | 阻断 |
| 10 | 清理边界 | `powershell tools/check-cleanup-boundary.ps1` | 阻断 |
| 11 | Rust 格式 | `cargo fmt --check` | 阻断 |
| 12 | Rust 编译 | `cargo check --workspace` | 阻断 |
| 13 | Rust 测试全量 | `powershell scripts/test.ps1 test --workspace` | 阻断 |
| 14 | Clippy warning budget | `powershell tools/check-clippy-warning-budget.ps1 -MaxWarnings 58` | 阻断 |
| 15 | 执行端 warning budget | `powershell tools/check-executor-warning-budget.ps1 -MaxWarnings 0` | 阻断 |
| 16 | 前端构建 | `cd frontend && npm run build` | 阻断 |
| 17 | 前端测试 | `cd frontend && npm run test` | 阻断 |
| 18 | 前端 E2E | `cd frontend && npm run test:e2e` | 阻断 |
| 19 | npm 审计 | `cd frontend && npm audit --audit-level=moderate` | 阻断 |
| 20 | 执行端前端构建 | `cd frontend-executor && npm run build` | 阻断 |
| 21 | 执行端编译 | `cargo check --bin executor` | 阻断 |
| 22 | 执行端测试 | `powershell scripts/test.ps1 test --bin executor` | 阻断 |
| 23 | QS 场景 smoke | `powershell scripts/scenario-smoke.ps1` | Closeout 阻断 |
| 24 | 干净工作区 | `powershell tools/check-clean-worktree.ps1` | Closeout/CI 阻断 |
| 25 | 全量树完整性 | `powershell tools/check-full-feature-tree.ps1` | Closeout 阻断 | 🆕 v4.0.0 |
| 26 | 能力栈一致性与元流水线 DryRun | `powershell tools/check-capability-stack.ps1` | Closeout 阻断 | 🆕 v4.7.0 |

说明：v3.7.1 起，Rust 格式基线由 `cargo fmt` 生成，`cargo fmt --check` 在 pre-commit / CI / closeout 三层阻断格式漂移。

说明：v3.7.2 起，workspace clippy warning 进入预算门禁，当前预算为 58，只降不升；executor warning 预算为 0，任何新增 executor warning 都会阻断。

说明：面向用户文本门禁默认扫描 README、前端源码、当前规范、实现契约、用户指南和总览。历史里程碑与归档报告不作为当前产品文案扫描；需要专项审计时可显式传入 `-Paths`。

### 2.3 Closeout / Release 门禁

```powershell
.\tools\run-closeout-gates.bat
```

执行 26 项 closeout 门禁，任一失败则整体不通过。Closeout 额外执行三矩阵治理、Developer Learning Closeout、QS 场景 smoke、干净工作区、全量树完整性和能力栈一致性检查：

```
powershell tools/check-matrix-governance.ps1
powershell tools/check-learning-closeout.ps1
powershell scripts/scenario-smoke.ps1
powershell tools/check-clean-worktree.ps1
powershell tools/check-full-feature-tree.ps1
powershell tools/check-capability-stack.ps1
```

Release workflow 必须至少完成一次手动 dry-run，确认 Windows runner 上能构建、打包、生成 SHA256SUMS。只有 tag 触发时才允许发布 GitHub Release。

---

## 三、第二层：AI 并行审计（自由维度诱错）

### 3.1 触发条件

每个里程碑 closeout 前 **必须** 执行至少一轮自由维度全量诱错审计。

### 3.2 审计维度

每次执行至少 5 个维度并行，维度不重复：

| 轮次 | 示例维度 | 目标 |
|------|---------|------|
| 第1轮 | A:逻辑/契约/GP, B:并发/竞态, C:边界/数值, D:持久化, E:API/错误 | 代码级漏洞全面扫描 |
| 第2轮 | F:前端交互, G:编译链完整性, H:测试质量, I:文档一致性, J:性能复杂度 | 上层质量和用户体验 |

后续轮次自由组合新角度。

> v3.5.1 实践: 首次执行 5 维度并行审计, 产出 53 项发现 (14 P1, 23 P2, 16 P3), 所有 P1 已在下个里程碑清零。

### 3.3 执行规则

- 每个维度至少 1 个独立 Agent 并行执行
- Agent 输出按 S0/P1/P2/P3 分级的发现清单
- 每个发现标注: 严重度、文件:行号、触发条件、修复建议、违反的 GP 条款
- **Agent 产出验证** (v3.7.0 新增): 每个执行代码修改的 Agent 必须在报告完成前运行 `cargo check --workspace` (后端) 或 `npm run build` (前端) 并确认 0 错误。Agent prompt 中须明确包含此要求。
- **Agent 文件隔离** (v3.7.0 新增): 同一文件被多个 Agent 修改时，必须在 Agent prompt 中明确声明文件边界（如"不得修改文件 X"），或使用 `isolation: "worktree"` 隔离工作区。避免并行 Agent 产出合并冲突。
- S0 **必须当前里程碑修复**
- P1 **下个里程碑修复**
- P2/P3 持续回归消化

### 3.4 验收标准

```bash
# 所有 S0 发现必须有对应的修复 commit
git log --oneline HEAD~10..HEAD | grep -c "S0"

# 修复后全部门禁通过
.\tools\run-closeout-gates.bat

# 自由维度审计报告落库
ls markdown/05-testing/自由维度诱错审计-*.md
```

---

## 四、第三层：发布前检查单

### 4.1 三角色手动验证

十角色诱错简化为 3 个最关键角色的发布前手动检查单：

| # | 角色 | 检查场景 | 验收 |
|---|------|---------|------|
| 1 | **新用户** | 首次打开→创建策略→编译→回测→查看结果 | 全程无崩溃，中文错误提示 |
| 2 | **策略开发者** | DAG边界测试、多时间框架、QS源码编辑、并发文件操作 | 核心路径可用 |
| 3 | **安全研究者** | 路径注入、XSS、凭证泄露、速率限制 | 无高危发现 |

### 4.2 执行时机

每 MINOR 版本 closeout 前执行。MAJOR 版本需额外执行完整十角色验证。

---

## 五、Closeout 审计

### 5.1 五维度评分

| 维度 | 考察内容 | 评分 |
|------|---------|:--:|
| 功能开发进度 | 指标覆盖、编译链完整、前端页面可用 | 1-10 |
| 仓库稳定程度 | 测试全量通过、编译 0 error、Clipy 0 warning、npm audit 清零 | 1-10 |
| 发布就绪度 | 门禁全部通过、无架构违规、文档同步 | 1-10 |
| 用户友好程度 | 全中文错误、界面可用、空状态引导、术语易懂 | 1-10 |
| 系统整体稳定性 | 回归检测、存储配额、编译路径合规、安全加固 | 1-10 |

### 5.2 GP 合规矩阵

逐条核查全部条款，输出状态矩阵。违规项注明修复计划。

### 5.3 Closeout 报告

存放于 `markdown/06-milestones/vX.Y.Z/03-closeout.md`，包含：

- 执行概况
- S0/P1/P2/P3 完成率
- 五维度评分
- GP 合规矩阵
- Developer Learning Closeout（MAJOR 版本必须填写）
- 遗留项（流向下一里程碑）

---

## 六、优化流水线

### 6.1 触发机制

Closeout 审计报告中的发现转化为下个里程碑的优化项。

| 严重度 | 处理时限 |
|--------|---------|
| S0 | 当前里程碑必须修复 |
| P1 | 下个里程碑必须修复 |
| P2 | 两个里程碑内修复 |
| P3 | 持续回归 |

> v3.5.1 实践: 审计发现自动转化为下一里程碑优化项, 实现审计→优化的闭环流程。

### 6.2 优化项模板

```
### [编号] [标题]
**严重度**: S0/P1/P2/P3
**来源**: 审计发现 / 用户反馈 / 门禁失败
**策略**: 违反的 GP 条款
**问题**: 具体描述
**方案**: 步骤级操作
**验收**: 可执行的验证命令或可观察行为
```

### 6.3 里程碑文档

存放于 `markdown/06-milestones/vX.Y.Z/`：

- `01-规划方案.md` — 目标、非目标、方案、验收、风险
- `02-综合优化清单.md` — 全部优化项（按 S0/P1/P2/P3 分组）
- `03-closeout.md` — closeout 报告

---

## 七、元流水线（自进化）

### 7.1 定义

元流水线是**优化流水线的流水线**，确保规范文档和流程本身不断进化。

### 7.2 自审计

每版本closeout前执行：

| 检查项 | 内容 |
|--------|------|
| 门禁脚本完整性 | 所有 `tools/check-*.ps1` 可执行，输出格式稳定 |
| 测试覆盖率趋势 | `#[test]` 数量、`.test.*` 文件数变化 |
| 门禁耗时 | 每次 CI 全量耗时，超过 10 分钟必须分析 |
| 误报率 | 手动 override 的门禁失败记录和原因 |
| GP §10.3 回归检查 | 重大功能覆盖回归检查执行记录 |
| 功能演进登记完整性 | 新增能力是否有登记、回归保护矩阵、兼容性与迁移说明 |
| 结构化指标来源 | `tools/track-gate-metrics.ps1` 以 NDJSON 记录门禁耗时、通过状态和失败摘要；closeout 前必须通过 DryRun |
| 前端入口真源对齐 | 工作区、工具栏、模块面板是否由后端 capability projection 驱动 |
| v4 MAJOR 演化通道 | 状态机 DSL、Risk Plane、ExecutionMachine、学习流水线是否按阶段推进 |
| v4/v5 provider 切面分层 | v4 是否只保留 OKX 单一 provider; 多交易提供方、多资产类别和全双工 WS 覆盖是否延后登记到 v5 |
| 学习流水线同步 | 新核心机制是否需要 owner 学习材料、复述和本地学习记录 |

数据记录到 `markdown/05-testing/meta-pipeline-log.md`。
结构化原始指标写入 `storage/audit/gate-metrics.ndjson`，`meta-pipeline-log.md` 只记录版本级摘要、异常分析和流程改进决策。

### 7.3 审计脚本版本管理

所有 `tools/check-*.ps1` 脚本：
- 受 git 版本控制
- 自身通过 UTF-8 编码
- 修改必须注明原因
- 输出格式必须稳定

### 7.4 策略检查工具质量跟踪

当前检查工具: `check-utf8.ps1`, `check-user-facing-text.ps1`, `check-capability-governance.ps1`, `check-capability-stack.ps1`, `check-i18n.ps1`, `track-gate-metrics.ps1`

| 指标 | 说明 |
|------|------|
| 误报率 | 检查报告违规但人工确认为误报 / 总报告次数 |
| 漏报率 | 应被检查捕获但遗漏的违规（由审计流水线补充发现） |
| 覆盖范围 | 检查覆盖的代码文件占比 |

### 7.5 规范文档自进化 🆕 (v3.0)

**触发条件**:

| 事件 | 动作 |
|------|------|
| MAJOR 版本发布 | GP 合规矩阵全量重审，所有条款逐条验证 |
| 自由维度诱错发现 S0 | 评估是否需要新增 GP 条款或修改现有条款 |
| 门禁脚本新增/修改 | 同步更新超级规范化 §2.2 门禁矩阵 |
| 每 MINOR closeout | 自审计指标更新，规范文档版本号同步 |
| 规范文档自身修改 | 修改原因写入 commit message，CHANGELOG 记录 |

**版本管理**:

- GP 和超级规范化自身受语义化版本管理
- MAJOR: 条款重构（如 v2→v3 分层）
- MINOR: 新增/删除条款
- PATCH: 措辞修正、链接修复

### 7.6 功能演进通道 🆕 (v3.7.1)

新增功能不再直接进入实现清单，必须先通过功能演进通道：

| 阶段 | 产物 | 阻断条件 |
|------|------|----------|
| 能力提案 | 功能 ID、用户入口、非目标、生命周期 | 未说明能力边界 |
| 功能演进登记 | `01-规划方案.md` 的“功能演进登记” | 缺少涉及层、依赖能力、fallback/拒绝行为 |
| 回归保护矩阵 | `01-规划方案.md` 的“回归保护矩阵” | 未列出受影响旧能力和验证命令 |
| 实现 | 代码、测试、文档、后端 capability、支持矩阵同步 | UI 入口未由后端 capability 驱动，或文档宣称先于 capability 和测试 |
| closeout | `03-closeout.md` 记录新增能力证据 | 证据缺失或旧能力回归未修复 |

执行契约见 `markdown/03-implementation/governance/implementation-feature-evolution-contract.md`。自动检查由 `tools/check-feature-evolution.ps1` 执行。

### 7.7 MAJOR 演化通道 🆕 (v4.0.0)

MAJOR 版本不得直接从终稿进入大规模实现。凡涉及 QS 语义、Core IR、runtime、Risk Plane、ExecutionMachine、交易模式或学习流水线的架构演化，必须先通过 MAJOR 演化通道。

| 阶段 | 产物 | 阻断条件 |
|------|------|----------|
| Phase 0: 终稿规划 | `vX.0.0/01-规划方案.md`、能力登记、非目标、回归保护矩阵 | 缺少兼容桥、非目标或拒绝行为 |
| Phase 1: 元契约 | QS profile、MachineTemplate、VenueCapabilityMatrix、Learning Pipeline 契约 | 未说明与 V1 保留面的关系 |
| Phase 2: 类型与能力矩阵 | 强类型、能力来源、provider_native/runtime_simulated/unsupported | 能力来源不明确或静默降级 |
| Phase 3: 静态审计 | parse/analyze/report，不接真实运行 | 新语法无法静态拒绝不支持路径 |
| Phase 4: 兼容桥 | 旧链路映射到默认 machine 实例 | 旧图、旧 QS 或旧运行记录兼容性未说明 |
| Phase 5: 事件循环 | event、cache、soft silence、memory snapshot | 状态迁移无事件来源或不可解释 |
| Phase 6: Risk Plane | precheck/order_check/postcheck | 真实下单路径可能绕过风控 |
| Phase 7: ExecutionMachine | 主流订单语义和 VenueAdapter 能力矩阵 | unsupported 能力未拒绝 |
| Phase 8: UI / 学习流水线 | 状态可视化、能力来源标识、大版本学习检查 | UI 宣称先于真实能力或学习检查缺失 |

执行原则:

- V1 QuantScript 保留面必须稳定保留，不得被状态机 DSL 静默扩大。
- V2 状态机语义应侧向生长，先静态审计，再接 Core IR，再接 runtime。
- 旧链路必须通过兼容桥映射到 `ObservationMachine` / `DecisionMachine` / `ExecutionMachine` 默认实例。
- 任一阶段发现旧能力退化，必须停止继续推进并回到回归保护矩阵修复。

### 7.8 前端后端能力真源通道 🆕 (v4.0.0)

凡新增或调整用户可见入口，尤其是工作区 tab、工作区 surface、工具栏 action、模块面板项、运行模式和执行语义，必须先通过前端后端能力真源通道。

| 阶段 | 产物 | 阻断条件 |
|------|------|----------|
| 后端能力契约 | `CapabilityResponse`、OpenAPI、fixture、Rust snapshot 测试 | 后端未声明 surface/action，前端已显示可用入口 |
| 前端投影层 | `normalizeCapabilitySnapshot` / capability projection / 类型化 view model | 组件直接读取静态数组判断支持状态 |
| UI 消费点 | 工作区、工具栏、模块面板共用同一 projection | 各组件维护不同支持判断或禁用原因 |
| 诱错回归 | 移除/降级 fixture 中某 surface/action 的前端测试 | 后端缺失能力时入口仍可点击或文案仍显示 supported |
| 治理同步 | `check-capability-governance.ps1`、支持矩阵、用户文案门禁 | snapshot drift、文档/README/UI 先于真实能力宣称支持 |

执行原则:

- 后端 capability response 是能力存在性、启用状态、拒绝原因和来源标记的唯一真源。
- 前端静态定义只允许保存排序、布局、骨架和本地化标签，不得决定能力是否 supported。
- `safe_fallback` 只能保留最小只读或禁用态入口，禁止恢复上一版本完整工作区。
- cache 模式可以显示缓存来源，但写操作必须继续携带 capability hash，由后端最终校验。
- 工作区视觉优化不得绕过本通道；视觉完成不等于能力对齐完成。

---

## 八、执行规则

### 8.1 阻断规则

以下情况**禁止进入下一阶段**：

- 任何阻断级门禁未通过
- 自由维度诱错 S0 发现未修复
- 发布前检查单 3 角色未通过
- GP 合规矩阵有 ❌ 项未修复
- CHANGELOG 缺失当前版本条目
- **版本号一致性检查未通过** (v3.7.1 收口): `powershell tools/check-version-consistency.ps1` 必须通过。每个 closeout 前必须验证 Cargo、Tauri、前端 package、package-lock、README、文档索引、超级规范化、overview 和执行端 HTML 标题均已递增至当前版本。
- **功能演进契约检查未通过** (v3.7.1 元流水线): `powershell tools/check-feature-evolution.ps1` 必须通过。新增能力若没有功能演进登记、回归保护矩阵、兼容性与迁移说明，禁止进入 closeout。
- **Rust 格式基线检查未通过** (v3.7.1 收口): `cargo fmt --check` 必须通过。若失败，必须先执行 `cargo fmt` 形成格式基线，再继续功能或发布流程。
- **MAJOR 演化通道未完成** (v4.0.0 起): 涉及状态机 DSL、Risk Plane、ExecutionMachine、交易模式、学习流水线的 MAJOR 版本，必须按 §7.7 逐阶段留下产物和拒绝行为；不得越过静态审计直接接入真实运行。
- **前端后端能力真源通道未完成** (v4.0.0 起): 新增或调整工作区、工具栏、模块面板、运行/回测/执行入口时，必须按 §7.8 留下后端 capability 契约、前端 projection、诱错回归和治理同步证据。
- **Developer Learning Closeout 缺失** (v4.0.0 起): MAJOR 版本 closeout 必须回答“本版本是否引入 owner 必学核心机制”，并记录学习材料、涉及文件、调用链、复述/校准状态。
- **元流水线耗时追踪 DryRun 未通过** (v4.7.0 起): `powershell tools/track-gate-metrics.ps1 -DryRun` 必须在 closeout 前通过；脚本不可解析、门禁定义缺失或输出 schema 异常时禁止 closeout。
- **v4 provider 范围漂移** (v4.8.0 起): v4 阶段禁止引入非 OKX provider 或多资产 provider 适配; 美股、港股、A股、贵金属、大宗商品、期货、期权等统一延后到 v5。缺少 WebSocket 全双工或等效实时订单/行情事件回执的 provider 视为关键基础设施缺失, 不进入适配支持。

### 8.2 紧急豁免

```
1. commit message 注明 [EMERGENCY] + 原因
2. 下个里程碑补齐所有跳过的检查
3. 紧急豁免本身被元流水线追踪
```

### 8.3 与 General_Policy 的关系

| General_Policy | 本文件 |
|----------------|--------|
| 规定"做什么、不做什么" | 规定"怎么检查、怎么保证" |
| 每条款标注检查方式 🛡️/🔍 | 每条款对应的门禁/审计机制 |
| 代码级规范 | 流程级规范 |
| 实体法 | 程序法 |

两者通过 **检查方式标注** 形成互锁:
- GP 标注 🛡️ → 超级规范化 §2.2 门禁矩阵有对应检查项
- GP 标注 🔍 → 超级规范化 §3 自由维度审计覆盖
- GP 无标注 → 超级规范化 §4 发布前检查单或 §5 closeout 审计覆盖

### 8.4 重大功能覆盖回归检查 🆕 (v3.4.0)

**触发条件**: 每次 MAJOR 或 MINOR 版本开发完成后, closeout 之前, **必须** 执行 GP §10.3 的 10 项功能覆盖检查。

**执行规则**:

| 规则 | 说明 |
|------|------|
| **触发时机** | 任何新增模块(如执行端)/新增子系统(如进程间加密)/重大重构(如存储路径配置化)完成后 |
| **检查内容** | 对照 GP §10.1 功能覆盖矩阵, 逐项验证全部已覆盖系统是否仍正常工作 |
| **检查方法** | 执行 GP §10.3 的 10 项快速验证, 任何一项失败 → 阻断 closeout |
| **发现处理** | 若某项已覆盖功能退化或失效, 必须修复后方可 closeout, 并在 closeout 报告中记录退化原因和修复方案 |

**设计意图**: 在 v3.0.0~v3.4.0 开发中, 多次出现新功能开发导致旧功能退化的情况:
- 执行端开发期间, QS 场景测试编译中断(集成测试 include! 模式不兼容)
- 存储路径配置化仅覆盖凭证文件, 30+ 其他路径仍硬编码
- 能力加载回退过于激进导致模块不可点击
- 前端版本号分散在多文件中未统一更新

本规则确保此类问题在 closeout 前被系统性发现, 而非等待管理员手动测试时暴露。

**执行记录**:

| 版本 | 日期 | 检查结果 | 发现退化数 | 修复状态 |
|------|------|---------|:--------:|:-------:|
| v3.5.0 | 2026-05-21 | 通过 (P1清零) | 14 P1 + 23 P2 | 全部修复 |
| v3.6.0 | 2026-05-21 | 通过 | 0 | N/A |
| v3.7.0 | 2026-05-21 | 通过 | 0 | N/A |

### 8.5 自由维度诱错审计常态化 🆕 (v3.7.0)

每 MAJOR/MINOR 版本 closeout 前必须执行至少一轮全维度诱错审计。审计维度覆盖 §3.2 所列全部维度, 确保每个 closeout 前都经过系统性漏洞扫描。

**执行规则**:

| 规则 | 说明 |
|------|------|
| **触发时机** | 每 MAJOR/MINOR 版本 closeout 前, 至少执行一轮 |
| **最小维度要求** | 5 维度并行, 覆盖 §3.2 第1轮全部维度 (A~E) |
| **执行方式** | 每个维度至少 1 个独立 Agent 并行执行 |
| **输出要求** | 按 S0/P1/P2/P3 分级的发现清单, 标注严重度、文件:行号、触发条件、修复建议、违反的 GP 条款 |
| **验收标准** | 审计报告落库 markdown/05-testing/, 全部门禁通过, S0 清零 |

**历史依据**: v3.5.0 首次执行全维度诱错审计, 证明该流程能系统性发现代码级漏洞。v3.7.0 将其提升为常态化要求, 确保每个版本 closeout 前都经过相同标准的审计覆盖。

### 8.6 开发者共同决策 🆕 (v3.7.0)

**原则**: AI Agent 是开发加速器, 但架构决策权和最终方案确定权属于开发者。

**触发条件**: AI Agent 在执行优化过程中, 遇到以下任一情况时, **必须**向开发者提出优化提案并等待决策后再执行:

| 需要共同决策 | 不需要共同决策 |
|-------------|--------------|
| 架构方案二选一或多选一 | 单文件内 bug 修复 |
| 涉及 ≥3 个文件的跨模块变更 | 文档格式修正 |
| API 路由或数据模型的变更 | GP 违规的明显修正 |
| 存储或安全策略变更 | 门禁脚本执行 |
| 新增外部依赖 (npm/cargo) | PATCH 版本号递增 |
| 里程碑范围变更 (新增/删除优化项) | 非功能性代码清理 |
| 任何不确定最优方案的优化项 | 测试用例修复 |

**提案格式**:
```
## 优化提案 #[编号]: [标题]
**问题**: [具体描述]
**方案 A** (推荐): [描述 + 理由]
**方案 B**: [描述 + 理由]
**方案 C**: [描述 + 理由]
选择哪个方案？
```

**执行记录**: 每次决策后在本节追加一行 `| 日期 | 提案编号 | 主题 | 选择方案 | 结果 |`

| 日期 | 提案编号 | 主题 | 选择方案 | 结果 |
|------|:--:|------|:--:|------|
| 2026-05-21 | #1 | 版本递增强制同步 | A | 新增至 §8.1 阻断规则 |
| 2026-05-21 | #2 | Agent 输出自动验证 | A | 新增至 §3.3 执行规则 |
| 2026-05-21 | #3 | 开发者共同决策规则 | A | 新增 §8.6 |
| 2026-05-21 | #4 | 编辑工具回退策略 | A | 新增 §8.7 |
| 2026-05-21 | #5 | Agent 并行度上限控制 | A | 新增至 §3.3 执行规则 |

### 8.7 编辑工具回退策略 🆕 (v3.7.0)

**问题**: Edit 工具在包含混合空格/制表符、UTF-8 中文字符的文件上可能因不可见字符差异导致匹配失败。

**策略**: 对同一文件的 Edit 尝试 **3 次失败** 后, 必须立即回退到基于脚本的替换方式 (sed/awk/python 脚本), 不再继续尝试 Edit。此规则适用于 Agent 和开发者操作。

**实践经验**: v3.7.0 开发中 `auth/mod.rs` 的 Edit 操作连续 6 次失败, 最终通过 sed + awk 脚本完成替换。3 次阈值基于"2 次可能是格式问题可调整, 3 次以上大概率是工具限制"的经验判断。

### 8.8 功能演进防回退规则 🆕 (v3.7.1)

**原则**: 项目允许在版本推进中追加更丰富功能，但新增功能必须是可登记、可验证、可拒绝、可回滚的能力演进，而不是散落在代码和文档中的正向宣称。

**阻断规则**:

| 情况 | 处理 |
|------|------|
| 新增用户可见入口但无 capability / 支持矩阵更新 | 阻断 |
| 新增用户可见入口未由后端 capability projection 驱动 | 阻断 |
| 新增 QS / runtime / executor 语义但无 Rust 测试 | 阻断 |
| 新增主要用户工作流但无场景、E2E 或手动验证记录 | 阻断 |
| 修改持久化格式但无兼容性和迁移说明 | 阻断 |
| 删除或弱化既有测试 / 场景但无 retired 记录 | 阻断 |
| 文档宣称 supported，但实现仍是 beta / restricted / planned | 阻断 |

**验收路径**: `tools/check-feature-evolution.ps1` 负责检查契约与里程碑结构；`tools/run-closeout-gates.bat` 负责执行回归门禁；GP §10.3 与自由维度审计负责发现无法自动化覆盖的功能退化。

### 8.9 v4 状态机化演化防偏规则 🆕 (v4.0.0)

**原则**: v4 状态机化不是推倒重来，而是在 v3.7.1 稳定线之上建立第二代语义层。任何实现必须保持“顶层 DAG 可审计、节点内部状态机化、Risk Plane 不可绕过、Execution 能力来源显式、学习流水线本地私有”的边界。

**阻断规则**:

| 情况 | 处理 |
|------|------|
| 直接修改 V1 QS 保留面导致旧策略语义变化 | 阻断 |
| 新状态机 transition 无事件来源 | 阻断 |
| `memory` 变化不进入快照、事件或回放证据 | 阻断 |
| `stale` / `recovering` 数据可扩大真实风险敞口但无风控授权 | 阻断 |
| 真实订单路径未经过 Risk Plane | 阻断 |
| Execution 能力未标记 `provider_native` / `runtime_simulated` / `unsupported` | 阻断 |
| `unsupported` 能力被静默降级或假装支持 | 阻断 |
| `runtime_simulated` 执行没有订单、成交、手续费、资产账本或 provider detached 证据 | 阻断 |
| UI 或文档把 planned/beta 能力描述为 supported | 阻断 |
| 工作区、工具栏或模块面板绕过 capability projection 直接声明可用 | 阻断 |
| `markdown/learning/` 个人学习记录进入 Git | 阻断 |

**学习流水线规则**:

- 学习流水线不进入每次强制门禁。
- MAJOR closeout 必须增加 Developer Learning Closeout。
- 个人学习记录只在用户明确要求“记录本轮学习”或“生成学习记录”时写入。
- `markdown/learning/` 必须保持本地忽略，不推 GitHub。

### 8.10 v4/v5 交易提供方切面分层 🆕 (v4.8.0)

**原则**: v4 的目标不是扩大交易提供方覆盖面，而是把 OKX 单一 provider 的模拟盘回执路径打穿, 证明执行端抽象可以承载 provider-native 回执。多 provider、多资产类别和真实资金扩张统一进入 v5。

**范围规则**:

| 范围 | 决策 |
|------|------|
| v4 provider | 只确保 OKX 单一 provider 切面 |
| v4 `PaperSimulated` | OKX production market-data WebSocket (public tickers + business candles) 或归档/回放数据 + 本地模拟撮合, provider order submission 必须 detached; 不需要交易 API key |
| v4 `PaperActual` | OKX 模拟盘下单 API, demo flag=1, REST 固定 `x-simulated-trading: 1`, 审计标注"OKX 模拟盘 / 非真实资金"; 只验证执行 API schema 和回执 |
| v4 `LiveSimulated` / `LiveActual` | 延后, 不进入 v4.8.0 执行端切面 |
| v5 provider 扩张 | 美股、港股、A股、贵金属、大宗商品、期货、期权和其他主流支持 WebSocket 的 provider |

**准入规则**:

- 模拟盘与真实资金 API schema 基本一致、仅 demo/prod flag 或环境参数不同的 provider, 模拟盘切面可视为 API schema 通过。
- 模拟盘行情不可作为策略质量验收依据。OKX demo / testnet K 线与 ticker 只可做 provider 连通性观察; 实时模拟优先使用无需交易 API key 的 OKX production market-data WebSocket (public tickers + business candles); 策略回测、参数优化和策略有效性验收必须使用真实公共行情源或已归档、可复现、通过质量校验的回放数据。
- API schema 通过不等于真实资金通道开放; 真实资金仍必须单独通过 Risk Plane、凭证保险库、审计、额度和 production gate。
- 不支持 WebSocket 全双工或等效实时订单/行情事件回执的 provider, 视为关键基础设施缺失, 本产品不做适配。
- v4 文档、代码、UI 和 OpenAPI 若宣称非 OKX provider supported, 一律视为范围漂移并阻断。
