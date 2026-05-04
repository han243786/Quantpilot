# QuantPilot v0.2.0 — 测试自动化基础设施

> **完成日期**: 2026-05-04  
> **状态**: ✅ 里程碑完成，非正式发布  
> **综合评分**: 9.2/10

---

## 一、核心交付

### 1. Quantscript 测试指令系统

在 formal Quantscript 中新增 `@test` / `@step` / `@assert` 标注语法：

```
@test { name: "场景名称" cover: ["P-03", "STRAT-001"] }

@step("编译策略") { @compile @assert compile.compilable == true }
@step("Paper运行") { @run { mode: "paper", duration: 10s } @assert run.equity > 0 }
@step("回测")     { @backtest { source: "deterministic_mock" } @assert backtest.metrics.step_count >= 100 }
@step("修改参数") { @modify { node: "ma", param: "fast_period", value: 30 } }
@step("等待")     { @wait { condition: "run.equity > 0", timeout: 30s } }
@step("保存")     { @save_run }
```

**支持 9 种测试动作**: `@compile` `@run` `@backtest` `@assert` `@save_run` `@modify` `@wait`

**支持 20+ 种断言上下文**: `compile.compilable` `compile.counts.*` `compile.protocol_name` `run.equity` `run.events.length` `run.has_event("X")` `backtest.metrics.*`

### 2. TestRunner 后端引擎

`POST /api/test/scenario/run` — 接收 .qs 源码，编译→执行→断言→返回结构化 TestReport JSON。

核心特性：
- Graph-path 编译（与前端 `POST /api/runtime/compile` 一致）
- Paper 运行 + deterministic_mock 回测
- 事件类型实时收集（9 种事件类型分布统计）
- 20 项回测指标（Sharpe, turnover, fee_drag, total_fills, buy/sell breakdown...）
- 指标交叉自洽验证通过
- 150 连续请求 0 失败，10 并发 0 失败

### 3. 前端可测试性基础设施

- Handle 端口 `data-testid` 精确标注（`handle-source-{nodeId}-{port}`）
- 工具栏/属性面板按钮 `data-testid` 全补全
- `window.__QUANTPILOT_TEST__` 测试桥接（15 个 API，仅 DEV 模式）
- Playwright v2 测试套件（7 tests, 52s）

### 4. CI 集成

- `.github/workflows/scenario-test.yml` — GitHub Actions 自动运行 3 个 .qs 场景
- 首次通过时间: 4m56s
- 每次 push 自动触发

### 5. 测试场景库

8 个可执行 .qs 场景 + 3 个策略模板 + 5 个压力测试

### 6. 数据质量修复

- `quote_price_map` Kline 盲视 → 订单转为 MARKET 而非 LIMIT
- MarketState fallback `f64::MAX` 无限流动性
- mock 数据 4 段行情 + 伪随机波动（±0.5%）

---

## 二、已知限制

| 限制 | 说明 |
|------|------|
| SMA 策略 QS lowering | `sma()` 内置函数 → `LongTermBuy` intent，回测 0 fills。替代方案: 无条件 `emit Intent` 策略（19 fills） |
| 前端 E2E 未入 CI | Playwright 测试需要同机 backend + frontend，与 backend-scenarios job 分离 |
| historical_replay | 需本地 market data 缓存文件，`storage/cache/historical/` |
| 版本对比 | @compare_backtests 语法已预留，TestRunner 侧待实现 UI 对比 |

---

## 三、文件变更

### 新增（核心）

| 文件 | 行数 | 功能 |
|------|:---:|------|
| `quantscript/src/test_plan.rs` | ~200 | AST 扩展: TestBlock/StepBlock/TestAction |
| `src/test_runner.rs` | ~900 | TestRunner 引擎 |
| `src/api_test_scenario.rs` | ~60 | API 端点 |
| `src/test/testBridge.js` | ~120 | 前端测试桥接 |
| `tests/scenarios/*.qs` | 8 文件 | 可执行场景 |
| `tools/run-scenario.js` | ~80 | 单场景运行器 |
| `tools/run-all-scenarios.js` | ~60 | 跨平台全量运行器 |
| `tools/generate-test-report.js` | ~80 | Markdown 报告生成 |
| `tools/check-qs.js` | ~50 | QS lint 工具 |
| `.github/workflows/scenario-test.yml` | ~90 | CI workflow |

### 修改（关键）

| 文件 | 改动 |
|------|------|
| `quantscript/src/script.rs` | +250 行 TestAction AST + parser |
| `quantscript/src/hir.rs` | +50 行 HirTestBlock |
| `quantscript/src/lib.rs` | +20 行 导出新类型 |
| `quantscript/src/analysis.rs` | +1 行 TestBlock 分支 |
| `quantscript/src/evaluator.rs` | +1 行 TestBlock clone |
| `quantscript/src/resolve.rs` | +60 行 TestBlock→Hir 转换 |
| `qrpc_runtime/src/execution_module.rs` | +10 行 Kline→Quote fallback |
| `qrpc_runtime/src/fill_engine.rs` | +3 行 f64::MAX liquidity |
| `qrpc_runtime/src/data_module.rs` | +60 行 4段行情+伪随机 |
| `src/main.rs` | +2 行 mod |
| `src/app_router.rs` | +6 行 路由 |
| `src/auth_middleware.rs` | +5 行 AtomicBool once |
| `frontend/src/nodes/BaseNodeCard.jsx` | +2 行 Handle testid |
| `frontend/src/components/TopToolbar.jsx` | +7 行 按钮 testid |
| `frontend/src/components/propertyPanelViews.jsx` | +6 行 表单 testid |
| `frontend/src/main.jsx` | +2 行 test bridge |
| `tests/api_backtest.rs` | ~20 行 浮点容差 |
| `Cargo.toml` | +1 行 time crate 依赖 |

### 文档（新增）

| 文件 | 内容 |
|------|------|
| `markdown/测试/测试自动化脚本化方案.md` | 总体设计 |
| `markdown/测试/实机场景化测试指南.md` | 118 步人工测试指南 |
| `markdown/测试/优化清单.md` | 24 项优化清单 |
| `markdown/测试/剩余优化里程碑.md` | 10 项剩余优化 |
| `markdown/测试/收口优化里程碑.md` | 15 项收口优化 |
| `markdown/测试/终轮收口里程碑.md` | L0-L3 细节优化 |
| `markdown/测试/全量审计报告.md` | 三轮审计 + 评分 |
| `markdown/测试/细节优化清单.md` | 语法覆盖度审计 |

---

## 四、未包含内容（v0.3.0 候选）

- 正式 Release Note / GitHub Release
- SMA lowering 对齐（graph double_ma 信号路径）
- 前端 E2E 入 CI
- @compare_backtests 完整实现
- historical_replay 数据准备工具
- 性能 CI 阈值断言
- GitHub Pages 文档站
