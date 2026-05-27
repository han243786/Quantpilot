# 引导矩阵

> 继承对象: `markdown/10-overview/overview-full-feature-tree.md`
> 职责: 管需求如何定位到模块、文件、接口、测试、文档和模块树节点。

---

## 1. 双树模型

引导矩阵由两棵树组成:

| 树 | 文件 | 作用 |
| --- | --- | --- |
| 全量树 | `markdown/10-overview/overview-full-feature-tree.md` | 物理文件地图，回答项目里有什么 |
| 模块树 | `module-tree.md` | 逻辑白箱网络，回答模块输入输出、public 方法和通信边界 |

全量树不被模块树替代。模块树也不得创造代码里不存在的模块。

---

## 2. 变更前定位流程

任何变更开始前必须定位:

1. 影响的业务或治理问题。
2. 对应全量树节点。
3. 对应模块树节点。
4. 关键真实文件。
5. 关键 public 方法或接口。
6. 需要更新的测试、门禁或人工检查。
7. 需要同步的文档。

轻量档可声明“无模块树影响”，但必须说明理由。

---

## 3. 提案必须声明的引导坐标

```markdown
## 引导坐标

| 项 | 内容 |
| --- | --- |
| 全量树节点 | 例如 `根2.3 运行时系统` |
| 模块树节点 | 例如 `backend.runtime.mutation` |
| 真实文件 | `src/runtime/mutation.rs` |
| public 方法 | `activate_ai_proposal` |
| 测试/门禁 | `cargo test -p quantpilot --test api_ai_proposal` |
| 同步文档 | `implementation-support-matrix.md` |
```

如果 public 方法尚未进入模块树，重型档必须补白箱节点后才能实现。

---

## 4. 引导矩阵防偏移

以下情况必须暂停并重新定位:

- 找不到全量树节点。
- 找不到父模块。
- 找不到能力真源。
- 真实文件与文档路径不一致。
- 计划修改的 public 方法未登记且影响跨模块调用。
- 文档或测试声明的能力与后端 capability 不一致。

---

## 5. 与全量树维护的关系

新增、删除或重命名 active 文件时，仍按全量树维护规则更新 `overview-full-feature-tree.md`。

新增、拆分、合并模块或修改关键 public 方法时，必须更新 `module-tree.md`。
