# v0.5.0 Adobe 风格前端全量重构方案

> 对标 Photoshop/Illustrator 专业面板式设计 | 全量重构 | 估 5 天

---

## 一、设计目标

```
当前问题:
  两套 CSS 变量体系混用 (qp-* + tv-*)
  顶部水平导航占用垂直空间
  7 个标签页切换频繁, 核心操作埋藏深
  ~8000 行 CSS 中存在大量重复样式

目标:
  左侧图标导航 → 释放垂直空间
  面板式可调整布局 → 专业软件体验
  统一设计令牌 → 修改一处全局生效
  ~4000 行精简 CSS → 无重复声明
```

---

## 二、设计令牌

```css
:root {
  /* 表面 — Photoshop 暗色主题 */
  --ad-bg: #0d0d0d;            /* 页面最深背景 */
  --ad-bg-raised: #181818;     /* 抬起背景 */
  --ad-panel: #1e1e1e;         /* 面板背景 */
  --ad-panel-hover: #242424;   /* 悬停面板 */
  --ad-card: #2d2d2d;          /* 卡片 */
  --ad-card-hover: #383838;    /* 悬停卡片 */
  --ad-border: #404040;        /* 柔和边框 */
  --ad-border-strong: #555555; /* 激活边框 */

  /* 文本 */
  --ad-text: #e6e6e6;
  --ad-text-secondary: #aaaaaa;
  --ad-text-muted: #6e6e6e;
  --ad-text-inverse: #ffffff;

  /* 强调 — Adobe 蓝 */
  --ad-accent: #1473e6;
  --ad-accent-hover: #1a7ef5;
  --ad-accent-soft: rgba(20, 115, 230, 0.15);
  --ad-accent-border: rgba(20, 115, 230, 0.35);

  /* 语义 */
  --ad-success: #2ecc71;
  --ad-success-soft: rgba(46, 204, 113, 0.12);
  --ad-warning: #f5a623;
  --ad-warning-soft: rgba(245, 166, 35, 0.12);
  --ad-error: #e74c3c;
  --ad-error-soft: rgba(231, 76, 60, 0.12);

  /* 字体 */
  --ad-font-ui: -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  --ad-font-mono: "JetBrains Mono", "Cascadia Code", "Consolas", monospace;
  --ad-font-size-xs: 11px;
  --ad-font-size-sm: 12px;
  --ad-font-size-md: 13px;
  --ad-font-size-lg: 15px;
  --ad-font-size-xl: 18px;

  /* 间距 — 4px 网格 */
  --ad-space-1: 4px;
  --ad-space-2: 8px;
  --ad-space-3: 12px;
  --ad-space-4: 16px;
  --ad-space-5: 20px;
  --ad-space-6: 24px;

  /* 圆角 */
  --ad-radius-sm: 2px;
  --ad-radius-md: 4px;
  --ad-radius-lg: 6px;
  --ad-radius-panel: 8px;

  /* 阴影 */
  --ad-shadow-sm: 0 1px 2px rgba(0,0,0,0.3);
  --ad-shadow-md: 0 4px 8px rgba(0,0,0,0.4);
  --ad-shadow-lg: 0 8px 24px rgba(0,0,0,0.5);

  /* 侧边栏 */
  --ad-sidebar-width-collapsed: 48px;
  --ad-sidebar-width-expanded: 160px;
}
```

---

## 三、导航图标映射

使用 Unicode 符号，不引入图标库依赖：

```
const NAV_ICONS = {
  strategies:   "\u{1F4CA}",  // 📊
  quantscript:  "\u{1F4DD}",  // 📝
  approvals:    "\u{2705}",   // ✅
  alerts:       "\u{26A0}",   // ⚠️
  snapshots:    "\u{1F4F7}",  // 📷
  runbook:      "\u{1F4D6}",  // 📖
  chaos:        "\u{1F9F0}",  // 🧰
};
```

---

## 四、Phase 1 — 设计系统基础

### 目标
统一 CSS 变量为 `--ad-*` 体系，全局重置，基础排版。

### 文件
| 操作 | 文件 | 改动量 |
|------|------|:--:|
| 新建 | `frontend/src/design-system.css` | ~400 行 |
| 修改 | `frontend/src/main.jsx` | 1 行 import |

### 全局重置
```
*, *::before, *::after { box-sizing: border-box; }
html, body, #root { height: 100%; margin: 0; }
body {
  background: var(--ad-bg);
  color: var(--ad-text);
  font-family: var(--ad-font-ui);
  font-size: var(--ad-font-size-md);
  -webkit-font-smoothing: antialiased;
}
```

---

## 五、Phase 2 — App Shell

### 目标
水平 GlobalNav → 左侧图标侧边栏。删除 Block5Nav 子导航（侧边栏已覆盖）。

### 左侧侧边栏行为
- 默认 48px 宽，只显图标
- Hover 展开至 160px，显示文字标签
- 覆盖式展开（不挤压内容区）
- 激活项显示蓝色左边框 + 浅蓝背景
- 顶部 "QP" 品牌标识，展开为 "QuantPilot"

### 文件
| 操作 | 文件 | 改动量 |
|------|------|:--:|
| 新建 | `frontend/src/components/LeftSidebar.jsx` | ~120 行 |
| 修改 | `frontend/src/App.jsx` | 用 `<AppShell>` 包裹, 移除 GlobalNav |
| 修改 | `frontend/src/shared.css` | 删除 `.qp-global-nav`, `.qp-subnav` |
| 删除 | `frontend/src/components/GlobalNav.jsx` | — |
| 删除 | `frontend/src/components/Block5Nav.jsx` | — |
| 修改 | 5 个 Block5 页面 | 移除 `<Block5Nav />` 引用 |

### 侧边栏结构
```
<nav className="ad-sidebar">
  <div className="ad-sidebar-brand">QP / QuantPilot</div>
  <div className="ad-sidebar-section">
    <SidebarItem icon="📊" label="策略" route="strategies" />
    <SidebarItem icon="📝" label="QS" route="quantscript" />
  </div>
  <div className="ad-sidebar-divider" />
  <div className="ad-sidebar-section">
    <SidebarItem icon="✅" label="审批" route="approvals" />
    <SidebarItem icon="⚠️" label="告警" route="alerts" />
    <SidebarItem icon="📷" label="快照" route="snapshots" />
    <SidebarItem icon="📖" label="故障手册" route="runbook" />
    <SidebarItem icon="🧰" label="混沌" route="chaos" />
  </div>
</nav>
```

---

## 六、Phase 3 — 工作区面板化

### 目标
7 标签页 → 4 标签页。编译/运行/回测按钮移至浮动命令栏。面板可拖拽调整大小。

### 标签页精简
| 旧 | 新 | 去向 |
|----|-----|------|
| 仪表盘 | 仪表盘 | 保留 |
| 总览 | — | 并入仪表盘 |
| 构建 | 画布 | 重命名, 主工作区 |
| 诊断 | — | 并入画布底部折叠条 |
| 研究 | 研究 | 保留 |
| 调试 | — | 调试数据嵌入画布内联 |
| 源码 | 源码 | 保留 |

### 浮动命令栏
```
┌─ 画布区域 ──────────────────────────┐
│                                      │
│                                      │
│                    ┌──┬────┬───┐     │
│                    │编│启  │回 │     │
│                    │译│动  │测 │     │
│                    └──┴────┴───┘     │
└──────────────────────────────────────┘
```

### TopToolbar 精简
- 删除 WorkspaceToolbarLayout（239-396 行）
- 工作区不再渲染 TopToolbar
- DefaultToolbarLayout 保留（独立编辑器页面用）

### 面板可调整大小
使用 `usePanelResize` hook（~40 行），拖拽 4px 分隔条调整左右面板宽度。

### 文件
| 操作 | 文件 | 改动量 |
|------|------|:--:|
| 修改 | `StrategyWorkspacePage.jsx` | 4 标签页 + 页头简化 |
| 修改 | `TopToolbar.jsx` | 删除 WorkspaceToolbarLayout |
| 修改 | `strategy-workspace.css` | 面板式布局 CSS |
| 修改 | `StrategyWorkspaceCodeTab.jsx` | 适配新面板系统 |

---

## 七、Phase 4 — 组件重设计

### 目标
所有组件统一迁移到 `--ad-*` 令牌体系。

### 4A — 画布节点卡片
- `BaseNodeCard.jsx`: 更小内边距, 更紧凑标题, 彩色圆点状态指示器

### 4B — 属性面板
- `PropertyPanel.jsx` + `propertyPanelViews.jsx`: 扁平区域头, 更密集字段, 等宽字体数值

### 4C — Block5 页面统一
全部替换为 Adobe 风格按钮/卡片/徽章：

| 旧类名 | 新类名 |
|--------|--------|
| `.qp-page` | `.ad-page` |
| `.qp-card` | `.ad-card` |
| `.qp-card__header` | `.ad-card__header` |
| `.qp-card__title` | `.ad-card__title` |
| `.qp-card__meta` | `.ad-card__meta` |
| `.qp-card__body` | `.ad-card__body` |
| `.qp-btn` | `.ad-btn` |
| `.qp-btn--primary` | `.ad-btn--primary` |
| `.qp-btn--ghost` | `.ad-btn--ghost` |
| `.qp-btn--sm` | `.ad-btn--sm` |
| `.qp-badge` | `.ad-badge` |
| `.qp-loading` | `.ad-loading` |
| `.qp-error` | `.ad-error` |
| `.qp-empty` | `.ad-empty` |
| `.qp-fade-in` | `.ad-fade-in` |

### 4D — 策略中心页 / 回测详情页 / QS 编辑器
CSS 变量全部迁移到 `--ad-*` 体系。

### 文件
| 操作 | 文件 |
|------|------|
| 修改 | `BaseNodeCard.jsx` |
| 修改 | `PropertyPanel.jsx`, `propertyPanelViews.jsx` |
| 修改 | `AlertsPage.jsx` 等 5 个 Block5 页面 |
| 修改 | `StrategyHubPage.jsx`, `strategy-hub.css` |
| 修改 | `BacktestDetailPage.jsx`, `BacktestComparePage.jsx`, `BacktestAnalysisLayout.jsx`, `backtest-analysis.css` |
| 修改 | `QuantScriptEditor.jsx` |
| 修改 | `TutorialOverlay.jsx` |

---

## 八、Phase 5 — 收尾

### 目标
删除旧 CSS 变量块，去重，精简代码量。

### 步骤
1. 删除 `styles.css` 中第 7-19 行和第 2965-2992 行的两个旧 `:root` 块
2. 删除 `shared.css` 中第 8-29 行的 `:root` 块
3. 删除 ~1500 行重复样式（旧版 `styles.css` 中 pre-2965 和 post-2965 的重复声明）
4. 验证无文件引用已删除的变量名 (`--qp-*`, `--tv-*`, `var(--border)`, `var(--text)` 等)
5. 全量 `vite build` + `vitest run` 通过

### 预期效果
```
CSS 总行数: ~8170 → ~4000 (减少 50%)
设计令牌: 2 套混用 → 1 套统一
重复样式: ~2000 行 → 0 行
```

---

## 九、文件变更汇总

| Phase | 新建 | 修改 | 删除 | 估时 |
|-------|:--:|:--:|:--:|:--:|
| 1 设计系统 | 1 | 1 | 0 | 0.5d |
| 2 App Shell | 1 | 7 | 2 | 1d |
| 3 工作区 | 0 | 4 | 0 | 1d |
| 4 组件 | 0 | 17 | 0 | 1.5d |
| 5 收尾 | 0 | 6 | 0 | 1d |
| **合计** | **2** | **35** | **2** | **5d** |

### 依赖关系
```
Phase 1 ──→ Phase 2 ──→ Phase 3
              │              │
              └── Phase 4 ───┘
                      │
                      v
                  Phase 5
```

Phase 4 可与 Phase 2 并行（只需 Phase 1 令牌）。

---

## 十、风险与约束

| 风险 | 缓解 |
|------|------|
| data-testid 属性丢失 | 所有现有 data-testid 原样保留 |
| CSS 变量迁移遗漏 | Phase 5 前全局 grep 校验 |
| 教程目标选择器失效 | 教程步骤使用的 data-testid 不变 |
| General_Policy 违反 | 所有用户可见字符串保持 t() 包裹 |

### 硬约束
- 不引入新 npm 依赖
- 所有功能保持可用
- `npx vite build` 持续通过
- `npx vitest run` 通过率不下降
