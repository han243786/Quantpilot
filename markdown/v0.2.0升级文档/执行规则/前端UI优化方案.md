# QuantPilot 前端 UI 统一优化方案

## 执行摘要

当前前端存在**两套完全独立的 UI 系统**：
- **系统 A (v0.1)**：策略/工作区/回测页面 → 深色主题，CSS 类驱动，设计精美
- **系统 B (v0.2)**：Block 5 页面 → 浅色主题，全内联样式，与系统 A 视觉割裂

优化目标：**统一视觉语言，建立全局导航，消除重复代码，保持简单大气易读。**

核心原则：
1. **不重写 v0.1** — 深色主题已是成熟设计，Block 5 页面向它靠拢
2. **最小侵入** — 用共享 CSS 类替代内联样式，不改变组件逻辑
3. **全局导航** — 所有页面可互达，用户不会"迷路"

---

## 现状基线

| 问题 | 严重度 | 影响面 |
|------|--------|--------|
| 两套 UI 系统并存（深色/浅色） | P0 | 5 个 Block 5 页面 + 1 个导航组件 |
| 无全局导航连接策略区与运维区 | P0 | 全站 9 条路由 |
| Block 5 页面零可访问性属性 | P1 | 5 个页面 |
| `API_BASE` 硬编码重复 5 次 | P1 | 5 个页面 |
| 内联样式对象重复（cardStyle 等） | P1 | 5 个页面 × 4 个样式对象 |
| BacktestDetailPage 缺 loading/error 态 | P2 | 1 个页面 |
| `styles.css` `:root` 重复定义 | P2 | 全局 |

---

## 优化方案

### 第一步：建立共享样式层

在 `frontend/src/` 下新建 `shared.css`，定义 Block 5 页面可复用的 CSS 类，色值与 v0.1 深色主题对齐：

```css
/* 共享色值 — 与 v0.1 styles.css:2968 生效的 :root 严格对齐 */
:root {
  --qp-bg: #08111f;
  --qp-panel: rgba(12, 20, 35, 0.84);
  --qp-panel-solid: #0c1423;
  --qp-border: rgba(116, 145, 182, 0.18);
  --qp-border-strong: rgba(130, 163, 206, 0.32);
  --qp-text: #ecf4ff;
  --qp-text-secondary: #8ea0b7;
  --qp-accent: #38bdf8;
  --qp-accent-mint: rgba(45, 212, 191, 0.16);
  --qp-success: #22c55e;
  --qp-danger: #ef4444;
  --qp-warning: #f59e0b;
  --qp-radius: 8px;
  --qp-shadow: 0 18px 40px rgba(0, 0, 0, 0.24);
}

/* Block 5 通用卡片 */
.qp-card {
  background: var(--qp-panel);
  border: 1px solid var(--qp-border);
  border-radius: var(--qp-radius);
  padding: 14px 16px;
  margin-bottom: 10px;
}

/* Block 5 导航条 */
.qp-nav {
  display: flex; gap: 6px; padding: 8px 0 8px;
  border-bottom: 1px solid var(--qp-border);
  margin-bottom: 18px;
  flex-wrap: wrap;
}
.qp-nav a, .qp-nav button {
  padding: 5px 12px; border-radius: var(--qp-radius);
  border: 1px solid var(--qp-border);
  background: var(--qp-panel); color: var(--qp-text-secondary);
  cursor: pointer; font-size: 12px; text-decoration: none;
  transition: background 0.12s;
}
.qp-nav a:hover, .qp-nav button:hover {
  background: rgba(56, 189, 248, 0.1);
  color: var(--qp-text);
}
.qp-nav .active {
  background: var(--qp-accent); color: #0c1423;
  border-color: var(--qp-accent); font-weight: 500;
}

/* 状态标签 */
.qp-badge { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 11px; font-weight: 500; }
.qp-badge--ok { background: rgba(34, 197, 94, 0.14); color: var(--qp-success); }
.qp-badge--warn { background: rgba(245, 158, 11, 0.14); color: var(--qp-warning); }
.qp-badge--err { background: rgba(239, 68, 68, 0.14); color: var(--qp-danger); }

/* 加载 / 错误 / 空态 */
.qp-loading { text-align: center; padding: 48px 0; color: var(--qp-text-secondary); font-size: 14px; }
.qp-error { padding: 12px 16px; background: rgba(239, 68, 68, 0.08); color: var(--qp-danger); border-radius: var(--qp-radius); margin-bottom: 14px; font-size: 13px; }
.qp-empty { color: var(--qp-text-secondary); text-align: center; padding: 32px 0; font-size: 14px; }

/* 按钮 */
.qp-btn { padding: 5px 14px; border-radius: var(--qp-radius); border: 1px solid var(--qp-border); background: var(--qp-panel); color: var(--qp-text); cursor: pointer; font-size: 13px; line-height: 1.5; }
.qp-btn:hover { border-color: var(--qp-border-strong); }
.qp-btn--primary { background: var(--qp-accent); color: #0c1423; border-color: var(--qp-accent); font-weight: 500; }
.qp-btn--danger { background: rgba(239, 68, 68, 0.16); color: var(--qp-danger); border-color: rgba(239, 68, 68, 0.3); }
.qp-btn--danger:hover { background: rgba(239, 68, 68, 0.24); }

/* 页面容器 */
.qp-page { max-width: 960px; margin: 0 auto; padding: 20px 24px; color: var(--qp-text); }
.qp-page h2 { margin: 0 0 14px; font-size: 18px; font-weight: 600; }
.qp-page h3 { margin: 18px 0 10px; font-size: 14px; font-weight: 600; color: var(--qp-text-secondary); }
```

### 第二步：重构 Block 5 组件

**每个页面**：
- 替换内联 `style={}` 为 `className="qp-*"`
- 替换硬编码 `API_BASE` 为 `import { API_BASE } from "../utils/api"`
- 添加 `role` / `aria-label` 属性

**ApprovalPanel.jsx**：
- 卡片 → `className="qp-card"`
- 按钮 → `className="qp-btn qp-btn--primary"` / `qp-btn--danger"`
- 状态标签 → `className="qp-badge qp-badge--ok"` 等
- 加载 → `className="qp-loading"`
- 错误 → `className="qp-error"`
- 删除 `styles` 内联对象

**AlertsPage / SnapshotsPage / RunbookPage / ChaosPage**：
- 同上替换
- 替换各自 `API_BASE` 为共享导入

**Block5Nav.jsx**：
- 替换内联样式为 `className="qp-nav"`
- 添加"策略中心"链接指向 `/strategies`

### 第三步：建立全局导航

新增 `GlobalNav.jsx` 组件，横跨全站：

```
[ QuantPilot ]  策略中心  |  审批  告警  快照  Runbook  混沌
```

- 左侧品牌 + 策略中心链接
- 右侧 Block 5 运维入口
- 当前页面高亮
- 在 `App.jsx` 中作为所有页面的共享外壳渲染

### 第四步：API 基础 URL 集中化

新建 `frontend/src/utils/api.js`：
```js
export const API_BASE = import.meta.env.VITE_BACKEND_ORIGIN || "http://127.0.0.1:3000";
```

所有页面从此处导入，不再各自定义。

### 第五步：修复 BacktestDetailPage

添加 loading 状态和 error 边界：
```jsx
if (loading) return <div className="qp-loading">加载回测数据...</div>;
if (error) return <div className="qp-error">加载失败: {error}</div>;
```

### 第六步：清理 styles.css 冗余

- 合并两个 `:root` 块（保留第二个，删除第一个的空壳）
- 标记已被 `shared.css` 覆盖的 Block 5 原始内联样式不再需要

---

## 实施里程碑

| 阶段 | 内容 | 文件 | 预计 |
|------|------|------|------|
| M1 | 创建 `shared.css` + `api.js` | 2 新建 | 30min |
| M2 | 重构 Block5Nav + 新建 GlobalNav | 1 改 + 1 新建 | 30min |
| M3 | 重构 5 个 Block 5 页面 | 5 改 | 1.5h |
| M4 | App.jsx 加入 GlobalNav 外壳 | 1 改 | 15min |
| M5 | 修复 BacktestDetailPage | 1 改 | 15min |
| M6 | 清理 styles.css :root 冗余 | 1 改 | 10min |
| **合计** | | **12 文件** | **~3h** |

---

## 验收标准

- [ ] 全部页面使用统一深色主题，无浅色内联样式残留
- [ ] GlobalNav 覆盖全站 9 条路由，任意页面可跳转至任意页面
- [ ] 5 个 Block 5 页面的 loading / error / empty 三态完整
- [ ] BacktestDetailPage 具备 loading + error 态
- [ ] `API_BASE` 统一定义在 `utils/api.js`，零处重复
- [ ] `vite build` 零错误零警告
- [ ] Block 5 页面每页至少 1 个 `aria-label` 或 `role` 属性
- [ ] 视觉上深色主题一致，无色彩跳跃
