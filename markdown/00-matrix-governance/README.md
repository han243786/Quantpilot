# 三矩阵治理总览

> 生效范围: 全部变更。
> 目的: 把 QuantPilot 的开发治理升级为可支撑无限规模扩张的控制平面。
> 状态: v4.15.0 起成为默认开发约束体系。

---

## 1. 三矩阵分工

| 顶层矩阵 | 文件 | 继承对象 | 管什么 |
| --- | --- | --- | --- |
| 流程矩阵 | `process-matrix.md` | 超级规范化 | 变更如何提出、校验、优化、实现、验证和收口 |
| 规范矩阵 | `standard-matrix.md` | General Policy | 硬规则、禁止项、父子通信、回退、冲突、并发锁和 AI 幻觉发现 |
| 引导矩阵 | `guidance-matrix.md` | 全量树 | 从需求定位到模块、文件、接口、测试、文档和模块树节点 |
| 模块树 | `module-tree.md` | 全量树新增白箱层 | 模块输入、输出、关键 public 方法、父子关系和通信边界 |

旧文件不删除。`General_Policy.md`、`principles-super-standardization.md` 和 `overview-full-feature-tree.md` 先作为三矩阵的历史主干被引用，后续只在有明确迁移方案时再逐步收敛。

---

## 2. 配套协议

| 文件 | 职责 |
| --- | --- |
| `proposal-flow.md` | 所有变更的提案状态机、三档执行判定表和提案模板 |
| `proposal-examples.md` | 轻量、标准、重型三档提案样例 |
| `release-transition-protocol.md` | 发布过渡期的横向连接、旁路缓存、热路径直连和可撤销证明 |
| `landing-roadmap.md` | v4.12.0 至 v4.15.0 的治理完全落地路线 |
| `recursive-speed-protocol.md` | v4.16+ 递归模块化的高速执行协议、智能门禁、两段式、同构批处理和状态游标规则 |
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

每次变更先阅读并执行:

1. `proposal-flow.md` 判定轻量、标准或重型。
2. `guidance-matrix.md` 定位影响节点。
3. `standard-matrix.md` 检查硬规则。
4. `process-matrix.md` 推进状态机。
5. 如涉及发布性能优化，再进入 `release-transition-protocol.md`。
6. 如处于 v4.16+ 递归模块化执行，再进入 `recursive-speed-protocol.md` 和 `recursive-state.json`。

---

## 5. 落地里程碑

| 里程碑 | 职责 |
| --- | --- |
| v4.12.0 | 三矩阵治理入口启用 |
| v4.13.0 | 模块树白箱扩面 |
| v4.14.0 | 治理门禁自动化 |
| v4.15.0 | 三矩阵完全接管 closeout |
