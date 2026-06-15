# 发布过渡例外授权记录

> Protocol: release_transition_exception
> Version: release-transition-exception-v1
> Scope: 发布版本过渡期横向连接、旁路缓存、热路径直连的授权记录。
> Owner: 三矩阵治理层。

本文件是 `release-transition-protocol.md` 的运行层补丁。默认开发态仍禁止子模块横向直连；只有开发者明确声明进入发布过渡，并填写本记录后，AI 才能围绕具体性能边提出方案。

---

## 当前状态

```yaml
exception_id: none
status: none
approved_by: none
scope: none
performance_evidence: none
direct_edge_added: none
why_parent_facade_insufficient: none
rollback: none
expiry: none
review_date: none
```

---

## 状态枚举

| 状态 | 含义 |
| --- | --- |
| `none` | 当前没有发布过渡例外 |
| `proposed` | 开发者已要求评估，但未批准实现 |
| `approved` | 开发者已批准，可在 scope 内实现 |
| `expired` | 例外到期，不再允许继续依赖 |
| `retired` | 例外已撤销，回到父子通信路径 |

---

## 硬规则

1. AI 不得主动把 `status` 从 `none` 改成 `proposed` 或 `approved`。
2. `status: approved` 必须同时具备 `approved_by`、`performance_evidence`、`direct_edge_added`、`rollback` 和 `review_date`。
3. 例外只能登记为发布态性能边，不能反向改写开发态模块树默认结构。
4. 缺少回退方案或复审日期时，不得进入实现。
