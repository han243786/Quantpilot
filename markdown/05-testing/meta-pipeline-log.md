# 元流水线日志

> 记录门禁耗时、误报率、测试覆盖率趋势

## v3.7.0 closeout (2026-05-21)

### 门禁脚本完整性

| 脚本 | 状态 |
|------|:--:|
| check-utf8.ps1 | 通过 |
| check-user-facing-text.ps1 | 通过 |
| check-capability-governance.ps1 | 通过 |
| check-i18n.ps1 | 通过 |
| check-gates-smoke.ps1 | 通过 |
| run-closeout-gates.bat | 通过 |

所有 5 个门禁脚本均存在于 `tools/` 目录下，且 `run-closeout-gates.bat` 就绪。

### 测试覆盖率

| 位置 | "#[test]" 数量 |
|------|:--:|
| src/ | 123 |
| src-executor/ | 9 |
| qrpc_runtime/src/ | 190 |
| quantscript/src/ | 78 |
| tests/ (integration) | 0 |

注意：`tests/` 目录中存在整合测试文件（如 `api_run.rs`, `api_sse.rs` 等），但未使用 `#[test]` 属性，可能使用外部测试 harness。

### 版本号一致性

| 文件 | 声明版本 |
|------|:--:|
| Cargo.toml | 3.7.0 |
| frontend/package.json | 3.7.0 |
| frontend-executor/package.json | 3.7.0 |
| General_Policy.md | v3.7.0 |
| principles-super-standardization.md | v3.7.0 |
| 06-milestones/README.md | v3.5.1 |

**不一致发现**：`markdown/06-milestones/README.md` 声明版本为 `v3.5.1`，而其他所有关键文件均为 `v3.7.0`。需确认该版本号是否应为 `v3.7.0`。

### 门禁耗时

(首次记录, 待CI运行后填写)

### 误报率

(首次记录, 无历史数据)

### v3.7.0 元流水线自进化 (2026-05-21)

基于 v3.5.0→v3.7.0 开发过程回顾，执行 5 项元流水线优化提案。所有提案均与开发者共同决策（§8.6），方案 A 全票通过。

| 提案 | 主题 | 发现问题 | 方案 | 落地位置 |
|:--:|------|---------|:--:|------|
| #1 | 版本递增强制同步 | 跨越 3 版本后批量修正版本号 | A | §8.1 阻断规则 |
| #2 | Agent 输出自动验证 | Agent 产出含编译错误 | A | §3.3 执行规则 |
| #3 | 开发者共同决策 | Agent 隐式做出架构决策 | A | 新增 §8.6 |
| #4 | 编辑工具回退策略 | Edit 工具连续 6 次失败 | A | 新增 §8.7 |
| #5 | Agent 并行度上限控制 | 并行 Agent 可能冲突同文件 | A | §3.3 执行规则 |

**本次自进化版本**: 超级规范化 v3.7.0 → 新增 §8.5 (诱错常态化), §8.6 (共同决策), §8.7 (编辑回退)。更新 §3.3 (Agent验证+隔离), §8.1 (版本一致性阻断)。

**当前超级规范化版本**: v3.7.0 | 条款数: 8 章 33 条规则 + 5 项执行规则

### v4.7.0 元流水线能力栈门禁落地 (2026-05-25)

本轮把元流水线审计提案落到可执行门禁：`track-gate-metrics.ps1` 修复为可解析脚本并新增 `-DryRun`；`check-capability-stack.ps1` 新增为第 25 项 closeout 阻断门禁；超级规范化 §2.2、§7.2、§8.1 已同步。

| 检查项 | 结果 | 说明 |
|--------|:--:|------|
| `track-gate-metrics.ps1 -DryRun` | 通过 | 6 个门禁定义完成结构校验 |
| `check-capability-stack.ps1` | 通过 | schema hash、模块 key、后端 fixture、前端 projection 与元流水线 DryRun 对齐 |
| `check-gates-smoke.ps1` | 通过 | 编码、用户文本、能力治理和元流水线门禁均能拒绝投喂坏样本 |
| `check-full-feature-tree.ps1` | 通过 | 新增脚本已纳入全量树覆盖 |

结构化原始指标落点: `storage/audit/gate-metrics.ndjson`。DryRun 不写入指标文件，只验证定义与 schema。

### v4.8.0 provider 切面分层元流水线执行 (2026-05-25)

本轮把用户提案固化为 v4/v5 provider 边界: v4 只确保 OKX 单一模拟盘 provider 切面; 多交易提供方、多资产类别和全双工 WebSocket 覆盖统一延后到 v5。模拟盘与真实资金 API schema 基本一致、仅 demo/prod flag 或环境参数不同的 provider, 模拟盘切面可视为 API schema 通过, 但真实资金通道仍需单独 gate。

| 检查项 | 状态 | 落点 |
|--------|:--:|------|
| 超级规范化 | 已同步 | §7.2 自审计、§8.1 阻断规则、§8.10 provider 切面分层 |
| GP 矩阵 | 已同步 | v4.8.0 provider 切面分层增补 |
| 全量树 | 已同步 | 执行端 OKX demo provider 路由、OpenAPI 和前端入口登记 |
| v4 venue 契约 | 已同步 | v4 只 OKX, v5 扩 provider/多资产/全双工 WS |
| W0-2 入口 | 已接线 | `/api/executor/provider/okx-demo/orders*` submit/query/cancel |

后续验证命令: `powershell tools/track-gate-metrics.ps1 -DryRun`, `powershell tools/check-full-feature-tree.ps1`, `powershell tools/check-user-facing-text.ps1`, `powershell tools/check-utf8.ps1`, `cargo test --bin executor okx`。

### v4.8.0 W4 收口元流水线执行 (2026-05-25)

本轮在 W4 代码质量与运维收口后执行元流水线，重点检查能力栈一致性、门禁定义可解析性和全量树覆盖。首次全量树检查发现 6 个新增 active 文件未登记，已同步到 `markdown/10-overview/overview-full-feature-tree.md`。

| 检查项 | 状态 | 说明 |
|--------|:--:|------|
| `tools/track-gate-metrics.ps1 -DryRun` | 通过 | 6 个门禁定义完成结构校验 |
| `tools/check-capability-stack.ps1` | 通过 | schema hash、模块 key、后端 fixture、前端 projection 与元流水线 DryRun 对齐 |
| `tools/check-full-feature-tree.ps1` | 通过 | 已登记新增 v4 runtime 拆分文件、StrategyCanvas 交互测试、v4 E2E、auth API 测试 |

新增全量树登记路径:
- `qrpc_runtime/src/v4_runtime_types.rs`
- `qrpc_runtime/src/v4_simulated_execution.rs`
- `qrpc_runtime/src/v4_runtime_tests.rs`
- `frontend/src/components/StrategyCanvas.interaction.test.jsx`
- `frontend/tests/e2e/v4-runtime-contracts.spec.js`
- `tests/api_auth.rs`
