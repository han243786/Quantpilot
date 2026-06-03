# 后端主进程交接提示词

状态：已准备，可手动粘贴给后端重构主进程。

只有在后端进程已经准备接收前端收口上下文时，才使用这段提示词。

```text
后端主重构进程：前端隔离递归重构已经关闭，并已准备好供后续整合使用。

当前前端状态：
- 最新前端准备记录：FE-0224。
- 前端状态文件：markdown/00-frontend-refactor-governance/frontend-recursive-state.json。
- 前端状态已关闭：current_parent=root.frontend，current_child_queue=[]，next_parent=null。
- 前端模块树已在本地关闭：markdown/00-frontend-refactor-governance/frontend-module-tree.md。
- 前端全量功能补充树已在本地关闭：markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md。

除非后端 closeout 已经明确打开整合步骤，否则不要合并全局治理。

后端 closeout 准备好后，使用这些前端前置文件：
- markdown/00-frontend-refactor-governance/frontend-global-merge-back-map.md
- markdown/00-frontend-refactor-governance/frontend-e2e-current-state-inventory.md
- markdown/00-frontend-refactor-governance/frontend-backend-main-thread-handoff-prompt.md
- markdown/00-frontend-refactor-governance/records/FE-0224-frontend-global-e2e-preflight-handoff.md

前端延后事项：
- 全局治理 merge-back 等待后端 closeout。
- E2E spec-body cleanup 等待后端路由和模块所有权稳定。

必须遵守的整合纪律：
- 将前端本地文档作为源证据，不要视为已经合并过的全局事实。
- 后端单线递归期间，不要编辑 E2E spec body。
- 除非开发者明确开启 release-transition 工作，否则不要引入发布过渡快捷连接。
- 后端 closeout 之后，执行独立的全局整合轮次，并运行治理检查、前端 build/test、后端检查和选定 E2E smoke。
```
