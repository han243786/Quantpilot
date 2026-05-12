# QuantPilot 项目总规则

> 生效日期: 2026-05-05 | 所有开发者必须遵守 | 违反的 PR 不予合并

---

## 一、架构铁律（4 条）

### 1.1 QS 是唯一策略定义路径

```
策略定义 → QS 源码 → parse → HIR → lower → Core IR → sandbox
                              ↑
         前端 graph 编辑器 → 可视化/编辑（不产生独立编译路径）
```

- **禁止**：graph 编辑器产出独立的 `RuntimeProtocolCoreConfig` 直接编译
- **允许**：graph 编辑器通过 `generate_quantscript_from_graph_value` → QS 管道编译
- **原因**：双路径曾导致同一策略产出不同的 intent 类型（SMA→0 fills vs SMA→5 fills）

### 1.2 新增功能必须跨三层验证

| 层 | 检查点 | 验收标准 |
|----|--------|---------|
| QS 解析 | `quantscript/src/resolve.rs` | 新语法可解析，未知函数拒绝 |
| Core IR | `qrpc_core_ir/src/lib.rs` | 新 indicator/intent 有对应枚举变体 |
| 运行时 | `qrpc_runtime/src/core_ir_evaluator.rs` | 新 indicator 有 evaluator 实现，非 stub |
| 前端 | `frontend/src/modules/builtinModules.js` | 若影响模块面板，注册新模块 |
| 端到端 | `tests/scenarios/` | 新增 .qs 场景文件验证 |

### 1.3 编译路径不可绕过

```
所有策略编译必须经过：
  QS 解析 → 语义分析 → 类型检查 → lowering → Core IR → sandbox

禁止：
  - 直接构造 RuntimeProtocolCoreConfig 跳过 QS 管道
  - 在 compile_api 中用 map_frontend_runtime_config 绕过 lowering
```

### 1.4 数据流单向原则

```
QS 源码 → graph JSON → 前端可视化
         ↘ .qs 文件持久化

graph JSON 编辑 → 保存 → 重新生成 QS → .qs 文件更新
                          ↑
              若 source_mode="quantscript"，保留原始 QS（R2-2）
```

**禁止**：保存 graph 时无条件覆盖原始 QS 源码（用户注释和复杂表达式会丢失）

---

## 二、代码规范（5 条）

### 2.1 错误消息必须是中文

```rust
// ✅ 正确
bail!("回测需要至少一个启用的 K 线数据源");
Err("不支持的运行模式: 请使用 'paper' 或 'testnet'")

// ❌ 错误
bail!("backtest requires at least one enabled kline data source");
```

- 所有 `bail!`、`anyhow!`、`format!`、`Err()` 中的用户可读文本必须是中文
- API 错误码（如 `"capability_gated"`、`"runtime_compile_failed"`）保留英文（它们是协议标识符）
- 诊断代码（`QS0001`、`QPQSLOW001`）保留英文

### 2.2 测试断言使用中文子串

```rust
// ✅ 正确 — 与实际错误消息一致
assert!(err.to_string().contains("必须至少观察一个代理"));

// ❌ 错误 — 代码已汉化，英文断言永远失败
assert!(err.to_string().contains("must observe at least one agent"));
```

### 2.3 新 indicator/evaluator 必须有单元测试

在 `qrpc_runtime/src/core_ir_evaluator.rs` 的 tests 模块中：
- 每个新信号至少 1 个 smoke test
- 验证输出在合理范围（如 RSI ∈ [0,100]，布林带 upper > middle > lower）

### 2.4 新 TestAction 必须有集成场景

在 `tests/scenarios/` 中创建 .qs 文件：
- 至少包含 `@compile` + `@backtest` 两个 step
- 文件命名：`scenario_<功能名>.qs`

### 2.5 前端字符串使用 `t()` 包裹

```jsx
// ✅ 正确
<button>{t("保存策略图")}</button>

// ❌ 错误 — 硬编码无法国际化
<button>Save Graph</button>
```

尽管当前语言文件为空（默认 zh-CN），使用 `t()` 确保未来可扩展。

---

## 三、文档规范（3 条）

### 3.1 文档分层原则

| 目录 | 写入内容 | 禁止写入 |
|------|---------|---------|
| `01-principles/` | 架构设计哲学 | 实现细节、代码示例 |
| `02-protocol/` | 数据结构/接口 RFC | 实现方案、测试结果 |
| `03-implementation/` | 行为契约、设计笔记 | 用户操作指南 |
| `04-guides/` | 面向用户的操作指南 | 内部设计笔记 |
| `05-testing/` | 测试方案、审计报告 | 过程追踪 |
| `06-milestones/` | 版本规划、里程牌 | 技术规范 |
| `09-archive/` | 已废弃的历史文档 | 当前有效的文档 |

### 3.2 文档必须全中文

- 技术术语（Intent、Indicator、Sandbox、Core IR）保留英文
- 代码块、文件路径、RFC 编号保留原样
- 正文、标题、表格内容必须是中文

### 3.3 里程碑文档命名

```
06-milestones/v0.X.0/
  ├── 01-规划方案.md
  ├── 02-里程碑.md
  ├── 03-前端补齐.md
  ├── ...
  └── XX-xxx.md
```

- 文件名以两位数字前缀编号，表示阅读顺序
- 一份文档只描述一个里程碑或一个主题
- 完成后在 `06-milestones/README.md` 更新状态

---

## 四、变更管理（3 条）

### 4.1 capability contract 变更必须更新固件

当修改以下任何一项时：
- `IndicatorKind` 枚举变体
- `CoreIndicatorKind` 枚举变体
- `IntentKind` 枚举变体
- `CapabilityContract` 字段

**必须同步更新**：
```
frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json
tests/fixtures/runtime/minimal_runtime_request.json  (schema_hash 字段)
```

验证命令：
```bash
cargo test capability_fixture_matches_backend_response_snapshot
cargo test --test api_ai_proposal
```

### 4.2 错误消息变更必须更新测试断言

修改任何 `bail!`/`anyhow!`/`Err()` 中的中文文本后，**必须**运行全量测试并修复所有 `.contains("旧文本")` 断言：

```bash
cargo test --workspace
# 修复所有 golden view 测试和 CLI 测试的断言
```

### 4.3 BacktestOutput / Sandbox 结构体变更

修改 `BacktestOutput`、`FastBacktestSandbox`、`SessionOutput` 等核心结构体的字段时：
- 搜索所有构造点（`BacktestOutput { ... }`）并补齐新字段
- 搜索 `backtest_artifacts.rs` 中的测试构造
- 搜索 `sandbox.rs` 中的所有构造函数

---

## 五、禁止事项（5 条）

### 5.1 禁止硬编码

```rust
// ❌ 禁止
let noise = pseudo_random(...) * trend_close * 0.005;

// ✅ 正确 — 从配置/参数读取
let vol = get_mock_volatility();
let noise = pseudo_random(...) * trend_close * vol;
```

### 5.2 禁止静默忽略参数

```rust
// ❌ 禁止 — save 参数被丢弃
TestActionDef::Backtest { save: _, .. } => { ... }

// ✅ 正确 — 使用或报错
TestActionDef::Backtest { save, .. } => {
    if save { /* 持久化逻辑 */ }
}
```

### 5.3 禁止 stub evaluator

```rust
// ❌ 禁止 — 新 indicator 返回 stub
CoreIndicatorKind::NewIndicator => Ok(CoreIrIndicatorEvaluation {
    reason: "indicator not yet implemented".to_string(),
    ...
})

// ✅ 正确 — 有完整的计算实现
CoreIndicatorKind::NewIndicator => evaluate_new_indicator(...)
```

### 5.4 禁止在图编辑器中绕过 QS 编译

```js
// ❌ 禁止 — 直接发送 FrontendRuntimeConfig 编译
postJson("/runtime/compile", { runtime_config: result.runtime_config })

// ✅ 正确 — 附带 graph_json 走 QS 管道
postJson("/runtime/compile", {
  runtime_config: result.runtime_config,
  graph_json: graph   // ← 必须发送
})
```

### 5.5 禁止跳过端到端验证

新增任何功能后，必须至少执行：
```bash
cargo check          # 编译检查
cargo test --workspace  # 全量测试
npx vite build       # 前端构建
```

---

## 六、快速检查单（PR 提交前）

| # | 检查项 | 命令/方法 |
|---|--------|----------|
| 1 | 错误消息是否全中文 | `grep -r "bail!\|anyhow!\|\.map_err" --include="*.rs" | grep -v target` 抽查 |
| 2 | 测试是否全通过 | `cargo test --workspace` |
| 3 | 前端是否可构建 | `npx vite build` |
| 4 | indicator 是否有 evaluator | 检查 `core_ir_evaluator.rs` dispatch 中没有 `"not yet implemented"` |
| 5 | 新功能是否有 .qs 场景 | `ls tests/scenarios/` |
| 6 | capability 变更是否更新了固件 | `cargo test capability_fixture` |
| 7 | 参数是否被静默忽略 | 搜索 `_: ` 模式 |
| 8 | 文档是否放到正确目录 | 对照 §3.1 分层表 |
| 9 | storage 写入是否声明了生命周期 | 对照 §7.1 三级分类 |
| 10 | storage 是否超过配额 | `du -sh storage/` < 500MB |

---

## 七、存储生命周期（5 条）

### 7.1 三级分类强制原则

所有 `storage/` 下的数据必须明确归类为以下三种生命周期之一，并在写入时声明：

| 生命周期 | 定义 | 默认 TTL | 清理触发 | 示例 |
|---------|------|:--:|------|------|
| **长期 (Permanent)** | 用户显式创建，除非显式删除否则永存 | 无上限 | 仅用户主动 DELETE | graph 文件、QS 源码、审计日志 |
| **暂时 (Temporary)** | 运行时需要维持，用完后可丢弃 | 7 天 | 启动清理 + 定时清理 | 回测工件、运行记录、实验、审批、报告 |
| **瞬间 (Transient)** | 只用几次、很快过时、留着无用 | 1 小时 | 用完即删 + 启动清理 | test-runs、突变提案、沙箱报告、快照、chaos report |

**禁止**：
- 将瞬间数据写入长期目录
- 将暂时数据标记为长期（反过来也不行）
- 创建不明确声明生命周期的 `storage/` 子目录

### 7.2 全局存储配额

```
storage/ 总大小上限: 500 MB
  超过 80%（400 MB）→ 日志告警
  超过 90%（450 MB）→ 强制清理所有过期暂时/瞬间数据
  超过 95%（475 MB）→ 拒绝新的非长期写入

单个子目录上限:
  长期目录: 无上限
  暂时目录: 200 MB
  瞬间目录: 50 MB
```

### 7.3 启动清理

每次服务器启动时，在 `main.rs` 中执行：

```rust
fn startup_storage_cleanup() {
    // 1. 清除所有瞬间数据（超过 TTL 的直接删除）
    // 2. 清除所有暂时数据（超过 TTL 的直接删除）
    // 3. 检查总大小，超过 80% 时告警
    // 4. 检查各子目录配额
}
```

### 7.4 写入时必须声明生命周期

所有新的 `storage/` 写入点必须在代码中：

```rust
// ✅ 正确 — 声明生命周期
const LIFECYCLE: StorageLifecycle = StorageLifecycle::Temporary;
persist_with_ttl(path, data, Duration::from_secs(7 * 24 * 3600))?;
```

### 7.5 测试/开发环境更激进清理

```
开发模式 (QUANTPILOT_DEV=true):
  暂时 TTL: 1 天（而非 7 天）
  瞬间 TTL: 10 分钟（而非 1 小时）
  启动时强制清理所有瞬间数据
```

---

## 八、前端设计规范（7 条）

> 生效日期: 2026-05-09 | 对标 Adobe Photoshop/Illustrator 专业暗色面板风格

### 8.1 配色饱和度限制

```
禁止使用高饱和度颜色作为 UI 主色。

✅ 允许:
  成功绿: #6b9e7a (鼠尾草绿)
  错误红: #c48888 (玫瑰灰)
  警告黄: #c4a55a (琥珀金)
  强调蓝: #1473e6 (Adobe 蓝)
  文本色: #e6e6e6 / #aaaaaa / #909090
  表面色: #0d0d0d (最深背景) / #151515 (抬起) / #1a1a1a (面板) / #242424 (卡片)
          #2e2e2e (卡片hover) / #4a4a4a (边框) / #5e5e5e (强调边框)

❌ 禁止:
  #22c55e / #2ecc71 (高饱和绿)
  #ef4444 / #f23645 / #e74c3c (高饱和红)
  #f59e0b / #f97316 (高饱和橙)
  #3b82f6 / #2962ff (高饱和蓝 — 仅允许 Adobe 蓝 #1473e6)
  #00ff00 / #ff0000 / #ffff00 (纯色 — 仅用于调试)

所有色值必须定义在 design-system.css 的 :root 块中并通过 CSS 变量引用，
禁止在组件样式或内联 style 中硬编码颜色值。
```

### 8.2 圆角限制

```
组件最大圆角: 6px
面板/卡片圆角: 2-4px
输入框/按钮: 2px

❌ 禁止:
  border-radius: > 8px (胶囊、大圆角)
  border-radius: 999px (完全圆形 — 仅允许头像/图标)
  backdrop-filter: blur() (毛玻璃/模糊)
```

### 8.3 背景限制

```
全局背景: 纯色 #0d0d0d，不使用渐变
面板背景: 纯色，使用 --ad-* 令牌

❌ 禁止:
  radial-gradient() (径向渐变)
  linear-gradient() (线性渐变 — 仅在必要时用于极暗的进度条或分隔线)
  backdrop-filter (模糊/毛玻璃)
```

### 8.4 导航与布局

```
主导航: 左侧图标侧边栏 (48px 折叠 / 160px hover 展开)
内容区: margin-left: 48px (随侧边栏展开)
命令面板: ⌘K / Ctrl+K 唤起
标签栏: 32px 下划线指示器

❌ 禁止:
  水平顶部导航条
  页面级子导航 (Block 内导航统一由侧边栏处理)
```

### 8.5 图标

```
图标必须使用 SVG 组件 (components/Icons.jsx)，禁止使用 Unicode emoji。

✅ 允许:
  <IconChart /> <IconPlay /> <IconCheck />

❌ 禁止:
  📊 📝 ✅ ⚠️ (Unicode emoji)
  第三方图标库 (无 npm 依赖)
```

### 8.6 组件令牌

```
所有组件样式必须通过 CSS 变量引用设计令牌。

✅ 正确:
  background: var(--ad-panel);
  border: 1px solid var(--ad-border);
  color: var(--ad-text);

❌ 错误:
  background: #1e1e1e;
  border: 1px solid #404040;
  color: #e6e6e6;
```

### 8.7 新增组件检查单

```
新增任何前端组件前，必须确认:
  1. 颜色是否通过 --ad-* 变量引用（非硬编码）
  2. 圆角是否 ≤ 6px
  3. 是否使用了 SVG 图标而非 emoji
  4. 用户可见字符串是否用 t() 包裹
  5. data-testid 属性是否已设置
```

### 8.8 CSS 文件结构与质量

```
前端 CSS 文件分层:

  层级          文件                          职责
  ────────────  ────────────────────────────  ──────────────────
  设计令牌      design-system.css             全局 --ad-* 变量、重置、App Shell
  共享组件      shared.css                    卡片/按钮/徽章/表格/输入框等通用组件
  页面样式      pages/strategy-hub.css        各页面特有的布局和组件样式
               pages/strategy-workspace.css
               pages/backtest-analysis.css
  全局补充      styles.css                    编辑器页、工具栏、面板等遗留样式

CSS 质量规则:

  1. 禁止空属性值
     ❌ background:        (无值)
     ❌ --custom-prop:     (空自定义属性)
     ✅ background: var(--ad-panel);
     ✅ background: none;

  2. 禁止跨文件重复样式
     ❌ .strategy-card-note 在 strategy-hub.css 和 strategy-workspace.css 中重复定义
     ✅ 通用样式放在 strategy-hub.css，其他文件引用同一套 class

  3. 禁止引用未定义的 CSS 变量
     ❌ background: var(--analysis-surface-glint);  (变量未在任何地方定义)
     ✅ 自定义变量必须在对应页面的 :root 或选择器块中定义

  4. 禁止跨文件重复 @keyframes
     ❌ @keyframes ad-fade-in 在多个文件中定义
     ✅ 动画定义在 design-system.css 或 shared.css 中

  验收命令:
    grep -rn "background:\s*$" frontend/src/ --include="*.css"    # 应返回 0
    grep -rn "^\s*--[^:]*:\s*$" frontend/src/ --include="*.css"   # 空 CSS 变量
    npx vite build                                                 # 必须通过
    npx vitest run                                                 # 必须全量通过
```
