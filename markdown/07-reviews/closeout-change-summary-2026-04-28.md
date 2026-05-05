# 收尾变更摘要 2026-04-28

本说明仅记录当前收尾批次。
它不是路线图，不是新能力列表，也不是公开发布就绪声明。

## 范围

本批次仅收紧了现有行为和仓库卫生：

- 运行时和回测持久化界面已与现有已保存、临时和已丢弃工件状态对齐
- 策略中心和策略工作区的窄屏布局仅针对溢出、换行和可读性进行了收紧
- 能力和发布措辞保持在现有 paper-runtime beta 边界内
- 已完成的规划台账已移至 `markdown/archive/planning-retired/` 下
- 为已落地的运行时工件操作、策略研究选择器和基础节点卡片渲染添加了针对性测试
- 本地构建、测试、Rust 目标和视觉审查输出未纳入版本化产品真实数据

## 非范围

本批次不引入：

- 新的交易能力
- 新的交易所、交易对、策略或 QuantScript 语言支持
- 插件市场支持
- 研究级回测声明
- 公开发布就绪
- 当前占位符 `LICENSE` 的替代

## 工件边界

以下输出仅为本地证据或构建产物，不得保留为产品真实数据：

- `frontend/dist/`
- `frontend/test-results/`
- `target/`
- `markdown/visual-review/`
- 生成的视觉审查 PNG 截图

视觉审查截图可重新生成用于检查，但应在审查后删除，除非所有者明确要求存档的证据集。

## 当前发布措辞

当前发布姿态仍然为：

- 私有基线只有在接受的基线门禁通过后方可进行
- 公开发布不得描述为已就绪
- 剩余的公开发布阻塞项包括 Vite/esbuild 审计链、最终出站许可证文本和仓库可见性批准

## 使用的验证

本批次已通过针对性视觉审查和收尾卫生检查进行验证：

- `VISUAL_REVIEW=1 npx playwright test tests/e2e/visual-responsive-review.spec.js --project=msedge`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1`
- `git diff --check`
