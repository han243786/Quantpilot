# QuantPilot

QuantPilot 是一个单机量化交易沙盒, 聚焦于诚实的能力边界、可复现的运行时行为和发布时契约纪律。

当前版本: **v0.5.1** | 前一版本: [v0.5.0](CHANGELOG.md) | 全局规则: [General_Policy.md](markdown/General_Policy.md)

当前发布姿态: 私有基线已通过接受基线门禁。公开发布前必须完成外向许可证决策和剩余 npm audit 风险接受。

## 产品边界

QuantPilot v0.5.1 的实际能力范围:

- **运行时模式**: paper (纸面交易), testnet (测试网)
- **交易所**: `binance`, `okx`
- **交易对**: `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- **桌面应用**: Tauri v2 自绘标题栏 Windows 桌面应用 (`start.bat` 一键启动)
- **前端**: Adobe 暗色面板设计系统、图编辑器、策略工作区、回测详情/对比、研究控制台
- **QuantScript**: 语法解析 → HIR → lowering → Core IR 完整编译管道

### 支持的全部指标 (18 种)

| # | 指标 | 状态 | # | 指标 | 状态 |
|---|------|:--:|---|------|:--:|
| 1 | MA Cross (双移动平均) | ✅ | 10 | OBV (能量潮) | ✅ |
| 2 | MA Deviation (移动平均偏差) | ✅ | 11 | CMF (资金流量) | ✅ |
| 3 | RSI (相对强弱) | ✅ | 12 | ADX (平均趋向) | ✅ |
| 4 | MACD | ✅ | 13 | Stochastic (随机) | ✅ |
| 5 | Momentum (动量) | ✅ | 14 | CCI (商品通道) | ✅ |
| 6 | ZScore (Z 分数) | ✅ | 15 | Parabolic SAR | ✅ |
| 7 | Spread (价差) | ✅ | 16 | Keltner Channel | ✅ |
| 8 | QuoteObserve (报价观察) | ✅ | 17 | Donchian Channel | ✅ |
| 9 | ATR (真实波幅) | ✅ | 18 | Bollinger Bands (布林带) | ✅ |

## 非宣称能力

以下项目不得描述为已支持的产品能力:

- 实盘交易 (live trading)
- 研究级回测语义 (research-grade backtest)
- 真正套利平台支持
- 第三方插件市场
- 通过 QuantScript 执行任意主机代码
- 公开 SaaS 服务

## 编译路径权威

编译产物优先级固定:

- `strategy_ir` 仅是语义预检工件, 不替代运行时编译
- `quantscript.formal_source` 存在时拥有运行时 lowering 权威
- QS 管道 (graph JSON → QS 源码 → parse → lower → Core IR) 是唯一编译入口 (§1.1, §1.3)
- 可执行输出始终遵循运行时编译, 而非 `strategy_ir` 预检结果

命名边界也已固定:

- `quantscript.formal_source` 是正式的 QuantScript 产品路径
- `strategy_graph` / graph-source 工件是图序列化和导入/导出辅助, 不是正式的 QuantScript 语言
- 旧版基于 section 的 QuantScript 配置解析仅作为 crate 内部兼容保留, 不作为主入口

语法参考:

- [QuantScript 主干基线](./markdown/04-guides/guide-quantscript-trunk-baseline.md)
- [正式 QuantScript 语法指南](./markdown/04-guides/guide-formal-quantscript-syntax.md)

## 快速启动

### 桌面应用 (Tauri)

从仓库根目录运行:

```bat
.\start.bat
```

Tauri 自动启动后端 (端口 3000) 和前端 Vite dev server (端口 5173), 并在桌面窗口中加载应用。

要求: Rust 工具链 + Node.js 18+ + WebView2 (Windows 11 已内置)。

### 仅后端 (无桌面窗口)

```powershell
cargo run
```

后端监听 `http://127.0.0.1:3000`。

### 仅前端 (无桌面窗口)

```powershell
cd frontend
npm install           # 首次需要
npm run dev
```

前端 dev server 监听 `http://127.0.0.1:5173`。

## 环境变量

| 变量 | 用途 | 默认值 |
|------|------|--------|
| `VITE_BACKEND_ORIGIN` | Vite dev proxy 后端地址 | `http://127.0.0.1:3000` |
| `VITE_API_BASE_URL` | 浏览器直连 API 地址 | 从当前 origin 派生 `/api` |
| `QUANTPILOT_DEV` | DEV 模式 (缩短 TTL, 强制清理瞬态数据) | 空 (`false`) |
| `QUANTPILOT_STORAGE_WATERMARK_MB` | 存储告警阈值 | `400` (80%) |

参见:

- `./.env.example`
- `./frontend/.env.example`

## 质量门禁

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; cmd /c npm run test
cd frontend; cmd /c npm run build
cd frontend; cmd /c npm run test:e2e
```

一键收口门禁包装器:

```powershell
.\tools\run-closeout-gates.bat
```

E2E 门禁合约:

- `cmd /c npm run test:e2e` 从 `frontend` 运行, 无需手动预启动后端
- Playwright E2E 使用隔离的 API-mock 合约, 不得向 `127.0.0.1:3000` 泄漏请求

npm 审计:

- `cd frontend; npm audit --audit-level=moderate` 是公开发布阻断检查
- 当前 `npm audit` **0 漏洞** (v0.5.1 已清零)
- 之前的 `postcss <8.5.10` 中等发现通过 lockfile 补丁修复为 `postcss@8.5.12`

## 仓库卫生

以下内容由 `.gitignore` 忽略:

- Rust `target/` 输出
- 前端 `node_modules/`、`dist/`、Playwright 输出、测试结果
- `storage/runs/`、`storage/backtests/`、`storage/experiments/` 下的运行时工件
- `storage/graphs/*.json`、`storage/graphs/*.qs`、`storage/graphs/versions/` 下的本地图快照
- `storage/audit/*.json` 下的本地审计 JSON
- 本地辅助日志 (如 `codex-vite-dev.log`)

## 文档入口

- [文档根](./markdown/README.md)
- [文档索引](./markdown/10-overview/overview-docs-index.md)
- [当前状态与路线图](./markdown/10-overview/overview-current-status-and-roadmap.md)
- [支持矩阵](./markdown/03-implementation/governance/implementation-support-matrix.md)
- [编译链合约](./markdown/03-implementation/governance/implementation-compile-chain-contract.md)
- [QuantScript 保留界面合约](./markdown/03-implementation/governance/implementation-quantscript-retained-surface-contract.md)
- [运行时/回测解释合约](./markdown/03-implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- [持久化/回放合约](./markdown/03-implementation/runtime/implementation-persistence-replay-contract.md)
- [能力治理](./markdown/03-implementation/governance/implementation-capability-governance.md)
- [能力治理注册表快照](./markdown/03-implementation/governance/implementation-capability-governance-registry.generated.md)
- [制品治理](./markdown/03-implementation/governance/implementation-artifact-governance.md)
- [测试模块实现](./markdown/03-implementation/runtime/implementation-testing-module.md)
- [交易沙盒实现](./markdown/03-implementation/runtime/implementation-trading-sandbox.md)
- [活跃 QRPC RFC 索引 (`RFC-001` 至 `RFC-020`)](./markdown/02-protocol/README.md)
- [API 参考](./markdown/04-guides/guide-api-reference.md)
- [策略模板库](./markdown/04-guides/guide-strategy-template-library.md)

## 发布状态

仓库可进行技术优化而无需猜测法律策略。所有者对公开发布的决策已明确:

- 在任何公开发布之前, 仓库保持私有状态, 使用当前保留所有权利的占位声明
- 仅当重新考虑公开发布资格时, 才将 [LICENSE](./LICENSE) 中的占位文本替换为最终批准的外向许可证文本
- 仅在通过接受基线门禁后创建私有基线提交

当前发布状态摘要:

- `LICENSE` 仍为占位文本
- `tools\run-closeout-gates.bat` 是已接受的私有基线门禁集
- 前端依赖审计风险已明确接受(仅限私有基线使用); 在 Vite/Vitest 迁移策略被接受并验证之前, 仍为公开发布阻断项
- 公开发布仍被阻断, 直到做出单独的公开发布批准和外向许可证决策

## 发布就绪度

v0.5.1 发布就绪度总结 (详见 [v0.5.2 里程碑](./markdown/06-milestones/v0.5.2/01-规划方案.md)):

| 维度 | 分数 | 关键风险 |
|------|:----:|------|
| 功能开发进度 | 7/10 | 18/18 指标 evaluator; QS 管道主编译路径 |
| 仓库稳定程度 | 4/10 | 后端测试编译崩溃 (65 错); 前端测试 12 文件失败 |
| 发布就绪度 | 3/10 | 测试套件全线崩溃; 存储配额未完成 |
| 用户友好程度 | 6/10 | 主路径全中文; npm audit 清零 |
| 系统整体稳定性 | 4/10 | 编译通过但回归检测缺失 |

**v0.5.2 焦点: 排雷收口为主, 不纳新功能, 不画新饼, 不引入新思想。**

