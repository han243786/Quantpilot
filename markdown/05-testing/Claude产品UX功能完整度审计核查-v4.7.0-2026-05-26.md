# Claude 产品/UX/功能完整度审计核查 — v4.7.0

> 日期: 2026-05-26
> 输入: Claude 产品/UX/功能完整度审计报告
> 核查方式: 只读代码检索 + 已落地文档对照
> 约束: 全量树、GP 矩阵、超级规范化审计→优化闭环

---

## 核查结论

Claude 报告中有一批高价值发现已确认成立, 也有少量需要降级或修正口径的项。后续里程碑只采纳确认成立或部分成立的内容, 不把误报固化为阻断项。

| 编号 | Claude 发现 | 核查结论 | 分流 |
|------|-------------|----------|------|
| C-S0-1 | 首次使用体验完全缺失 | 部分成立。策略中心已有 CTA、说明和空状态; 教程也有顶部工具栏入口。但无首次访问自动触发和完成状态持久化。 | v4.8.2 P2 |
| C-S0-2 | 执行端零国际化 | 成立。`frontend-executor` 无 I18nProvider/useI18n, 大量中文硬编码。 | v4.8.2 P1 |
| C-S0-3 | `zh-CN.js` 双重 Unicode 转义 | 成立。前 9-25 行存在字面量 `\\u...` 键和值。 | v4.8.2 P1 |
| C-P1-4 | QuantScript 编辑器过于原始 | 成立。当前为 textarea, 缺语法高亮/补全/持久化, 错误为原始 HTTP 文本, 粘贴超限用 alert。 | v4.8.2 P1 |
| C-P1-5 | 主前端硬编码中文分散 | 成立。`StrategyWorkspacePage.jsx` loading/error 与部分 toolbar 文案未 t()。 | v4.8.2 P1 |
| C-P1-6 | CSS 质量问题 | 成立。`styles.css` 存在缺逗号选择器, 两套按钮系统, Toast 颜色硬编码。 | v4.8.2 P1 |
| C-P2-7 | 无亮色主题 | 成立。未见 light theme 或 prefers-color-scheme: light token。 | v4.8.2 P3 |
| C-P2-8 | Tab 切换全量重挂载 | 成立。工作区标签使用条件渲染 + lazy Suspense。 | v4.8.2 P2 |
| C-P2-9 | 无 404 页面 | 成立。未知前端路由默认落回策略中心。 | v4.8.2 P2 |
| C-P2-10 | 侧边栏仅 hover 展开 | 成立。CSS 仅 `.ad-sidebar:hover` 显示标签, 缺 focus-within/固定展开。 | v4.8.2 P2 |
| C-P2-11 | 命令面板仅页面导航 | 成立。命令定义只含页面跳转。 | v4.8.2 P2 |
| C-P2-12 | ErrorBoundary 回退简陋 | 成立。仅标题、说明、可选重试, 且部分内联样式。 | v4.8.2 P3 |
| C-P2-13 | 教程无自动触发 | 成立。`useTutorial` 无 first-visit/localStorage 逻辑。 | v4.8.2 P2 |
| C-S0-14 | 无注销端点 | 成立, 但属于账户生命周期范围。按用户要求从本轮执行里程碑裁出。 | 不纳入 |
| C-S0-15 | 无密码重置/找回流程 | 成立, 但属于账户生命周期范围。按用户要求从本轮执行里程碑裁出。 | 不纳入 |
| C-S0-16 | PaperActual 自动交易循环未实现 | 成立。`start_strategy` 对 provider_order_submission_attached 返回 NOT_IMPLEMENTED。 | v4.9.0 P1 |
| C-P1-17 | 插件安全框架未接入运行时 | 成立。`check_security` 注明尚未在生产执行路径接线; PluginSandbox 也有网络/文件系统限制说明。 | v4.9.0 P1 |
| C-P1-18 | OpenAPI 凭证路由结构错误 | 部分成立。`/api/runtime/runs` 已存在, 但 `/api/credentials` 被缩进在 `components` 下而非 `paths` 下。 | v4.8.1 P1 |
| C-P1-19 | 无策略导入/导出 | 部分成立。已有运行配置和 QS 源码导出; 缺完整策略包导入/导出和回测分享包。 | v4.9.0 P2 |
| C-P1-20 | 无用户设置页面 | 成立。未见统一设置页管理语言/主题/编辑器偏好。 | v4.9.0 P2 |
| C-P1-21 | 策略中心无搜索/筛选/排序 | 不成立。策略中心已有搜索、范围、状态、排序控件和模型逻辑。 | 不采纳 |
| C-P2-22 | API 版本混用 | 成立但已有注释说明迁移意图。作为治理统一项, 不列 S/P 阻断。 | v4.9.0 P2 |
| C-P2-23 | AI 提案沙箱 3 次失败后无队列/覆盖机制 | 成立。已有日志和 JoinHandle 监视, 但没有 retry queue/admin override。 | v4.9.0 P2 |
| C-P2-24 | 无中文用户指南 | 成立。当前只有 `guide-user-guide-en.md`。 | v4.8.2 P2 |
| C-P2-25 | 执行端 K 线缺周期选择 | 成立。`KlineChart` 无 timeframe 控制。 | v4.9.0 P2 |

---

## 不采纳或降级说明

- “教程零入口”不准确: `TopToolbar.jsx` 存在 `toolbar-tutorial-action`, `App.jsx` 会挂载 `TutorialOverlay`。
- “策略中心空状态无引导”不准确: `StrategyHubHeroSection.jsx` 有创建首个策略 CTA, `StrategyHubRosterTableSection.jsx` 有空结果提示。
- “策略中心无搜索/筛选/排序”不准确: `useStrategyDirectoryModel.js` 和 `StrategyHubHeroSection.jsx` 已提供搜索、范围、状态、排序。
- “POST /api/runtime/runs 在 spec 中缺失”不准确: OpenAPI 已登记 `/api/runtime/runs` 及 run detail/save/events/replay/status。实际问题是 `/api/credentials` 缩进位置错误。

## 后续里程碑分流

| 版本 | 主题 | 纳入项 |
|------|------|--------|
| v4.8.1 | API 契约与部署治理 | OpenAPI 凭证路径结构、route diff 基线、profile 矩阵、四平面治理 |
| v4.8.2 | 产品/UX/i18n 收敛 | zh-CN 转义、执行端 i18n、QS 编辑器、CSS、教程自动触发、404、侧边栏键盘、命令面板、中文用户指南 |
| v4.9.0 | 产品功能完整度与插件执行安全 | PaperActual 自动 runner、插件安全强制、策略包导入/导出、设置页、API 版本迁移、沙箱队列、执行端图表周期 |
