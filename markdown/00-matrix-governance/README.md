# 三矩阵治理总览

> 生效范围: 兼容档案、历史规则、旧门禁素材。
> 目的: 保留 QuantPilot v4.12-v4.16 旧三矩阵治理事实，供新治理引用和门禁兼容。
> 状态: `GOV-GOVERNANCE-NEXT-PROMOTION-01` 起不再是默认权威入口；默认入口为 `governance-next/README.md`。

---

## 1. 三矩阵分工

| 顶层矩阵 | 文件 | 继承对象 | 管什么 |
| --- | --- | --- | --- |
| 流程矩阵 | `process-matrix.md` | 超级规范化 | 变更如何提出、校验、优化、实现、验证和收口 |
| 规范矩阵 | `standard-matrix.md` | General Policy | 硬规则、禁止项、父子通信、回退、冲突、并发锁和 AI 幻觉发现 |
| 引导矩阵 | `guidance-matrix.md` | 全量树 | 从需求定位到模块、文件、接口、测试、文档和模块树节点 |
| 模块树 | `module-tree.md` | 全量树新增白箱层 | 模块输入、输出、关键 public 方法、父子关系和通信边界 |

旧文件不删除。`General_Policy.md`、`principles-super-standardization.md` 和 `overview-full-feature-tree.md` 继续作为历史主干和兼容事实来源被引用，但新任务不再从本文件启动治理流程。

---

## 2. 配套协议

| 文件 | 职责 |
| --- | --- |
| `proposal-flow.md` | 所有变更的提案状态机、三档执行判定表和提案模板 |
| `proposal-examples.md` | 轻量、标准、重型三档提案样例 |
| `release-transition-protocol.md` | 发布过渡期的横向连接、旁路缓存、热路径直连和可撤销证明 |
| `landing-roadmap.md` | v4.12.0 至 v4.15.0 的治理完全落地路线 |
| `recursive-speed-protocol.md` | v4.16+ 递归模块化的高速执行协议、智能门禁、两段式、同构批处理、同父级 wave、成本受控降档、末端叶子智能判定、terminal leaf control v2、QPCursor 生成、未跟踪文件预检和状态游标规则 |
| `recursive-state.json` | 当前递归游标，记录 parent、phase、closed children、open residuals 和一次性提示黑名单 |

---

## 3. 总铁律

1. 所有变更都必须声明三矩阵影响，轻量变更也要声明“无行为影响 / 无模块树影响”。
2. 默认开发态禁止子模块横向直连，子模块必须经父模块、登记接口、事件、adapter 或契约层通信。
3. 横向连接只能在开发者明确声明“发布版本过渡”后被提案，AI 不得主动提出进入发布过渡。
4. 关键 public 方法必须进入模块树白箱节点；无法指出真实文件、真实方法、真实测试的 AI 结论不得作为事实。
5. 命中更高执行档时必须升档，不得因改动很少、只是文档或测试通过而降档。

---

## 4. 使用入口

promote 后，默认使用入口改为:

1. `governance-next/README.md`。
2. `governance-next/05-authoritative-operating-model.md`。
3. `governance-next/01-qpcursor-protocol.md`。
4. `governance-next/02-governance-heat-trigger.md`。
5. `governance-next/03-local-invariants.md`。

本目录只在以下场景读取:

1. 兼容门禁需要确认旧三矩阵文件存在。
2. 新治理需要引用旧规则的历史来源。
3. 递归游标仍暂存在 `recursive-state.json`。
4. 模块树和全量树尚未迁入 QPCursor 状态文件。

旧流程不得覆盖 QPCursor 的 allowed workset、stop_if、治理热度和 evidence。

---

## 5. 落地里程碑

| 里程碑 | 职责 |
| --- | --- |
| v4.12.0 | 三矩阵治理入口启用 |
| v4.13.0 | 模块树白箱扩面 |
| v4.14.0 | 治理门禁自动化 |
| v4.15.0 | 三矩阵完全接管 closeout |
| v4.16.0 / GOV-RECURSIVE-COST-CONTROL-01 | 递归高速协议升级为 `recursive-high-speed-v2`，允许同父级可审计批次作为可验证步骤，同时强制保留 child 级白箱证据和降档触发器 |
| v4.16.0 / GOV-LEAF-GRANULARITY-SMART-JUDGE-01 | 末端叶子智能判定接管 leaf split gate，按 split benefit、leaf size、risk、governance cost 和 system efficiency 决定 STOP/WAVE/SPLIT/PRECISION |
| v4.16.0 / GOV-LEAF-GRANULARITY-JUDGE-TOOL-01 | 只读叶子粒度评分脚本接入治理体系，输出 `normalized_split_score` 和 STOP/WAVE/SPLIT/PRECISION 证据 |
| v4.16.0 / GOV-TERMINAL-LEAF-CONTROL-V2-01 | 将只读发现固化为 `terminal_leaf_control_v2`，脚本输出 `governance_mode`，限制底层小叶独立四段式治理 |
| v4.16.0 / GOV-GOVERNANCE-NEXT-OPTIMIZATION-01 | 叶子判定拆分为 `split_decision` 与 `governance_packaging`，超大高风险叶升为 precision baseline，并接入 QPCursor 生成器、未跟踪文件预检和索引降重路线 |
| v4.16.0 / GOV-GOVERNANCE-NEXT-PROMOTION-01 | `governance-next/` 正式成为默认权威入口，旧三矩阵降级为兼容档案和历史门禁素材 |
