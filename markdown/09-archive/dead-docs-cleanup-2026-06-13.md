# 死文档检查清理记录

> 日期: 2026-06-13 | 类型: 文档治理 | 范围: 活跃 markdown 目录

## 检查口径

本轮只处理低风险死文档，不删除历史证据。

判定条件：

1. 长期未被当前治理流程触达。
2. 不在 `markdown/README.md`、`overview-docs-index.md`、模块树或当前递归状态中承担入口职责。
3. 已被三矩阵提案流程或其他当前权威文档替代。
4. 移动后不影响文档门禁。

## 发现

| 候选 | 判定 | 处理 |
|------|------|------|
| `markdown/templates/design-doc-template.md` | 旧设计文档模板，v1/v2 时期使用；v4.15+ 已由 `00-matrix-governance/proposal-flow.md` 接管 | 归档到 `markdown/09-archive/planning-retired/design-doc-template.md` |
| `markdown/10-overview/overview-project-briefing-v2.3.2.md` | v2.3.2 时间点项目全量简报；当前 v4.16 总览入口已由 `overview-current-status-and-roadmap.md`、`overview-docs-index.md` 和三矩阵治理接管 | 归档到 `markdown/09-archive/overview-retired/overview-project-briefing-v2.3.2.md` |
| `markdown/implementation/` | 旧非编号实现目录残留；当前入口已统一为 `markdown/03-implementation/`，本轮检查时目录内无文件 | 删除空目录 |
| `markdown/07-reviews/` | 旧 reviews 编号目录残留；当前可追溯 review 材料已在 `markdown/09-archive/reviews-retired/`，本轮检查时目录内无文件 | 删除空目录 |
| `markdown/06-milestones/v0.2.0/CHANGELOG-v0.2.0.md` | 未被当前索引直接引用，但属于版本历史证据 | 保留 |
| `markdown/06-milestones/v3.7.2/04-closeout.md` | 未被当前索引直接引用，但属于 closeout 证据链 | 保留 |
| `markdown/02-protocol/RFC-001` ~ `RFC-020` | 多数最近未改，但仍由 RFC README 与协议层承担权威语义 | 保留 |

## 结论

本轮只清理确认退役的活跃目录文件，并补齐归档 README。后续如继续做死文档治理，应优先扫描：

1. 活跃目录中的未引用模板、旧规划和重复入口。
2. 当前总览索引没有覆盖的 orphan 文档。
3. 历史生成物是否已经进入 `09-archive/*-retired/`。

## 追加检查：空旧目录残留

追加检查发现，活跃根目录中还有 `markdown/implementation/` 与 `markdown/07-reviews/` 两个空目录残留。它们不含文件，不被 Git 跟踪，也不承担当前入口职责；删除后不影响历史证据链。
