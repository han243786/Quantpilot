# v4.4.0 Closeout — 嵌套状态机第一波

> 日期: 2026-05-25  
> 版本类型: MINOR  
> 结论: **通过**。v4.4.0 已完成实现、文档同步、治理同步、全量树同步和 24/24 closeout 门禁。

## 一、交付范围

| 模块 | 交付 |
|---|---|
| Core IR | `MachineState.child_machine` 支持同模板二级 child machine；静态契约拒绝三级嵌套、重复 id、模板不一致 |
| QuantScript | v4 static audit 接受 `state { machine ... }`；新增 `QSV4118` 深度超限诊断；state 外直接嵌套仍拒绝 |
| Runtime | v4 PaperSimulated 支持父 transition 优先、子 machine memory 隔离、层级 snapshot、嵌套 backtest trajectory 展开 |
| 复杂度预算 | `ComplexityBudgetContract` / `ComplexityMetrics` 增加嵌套深度和事件处理路径指标 |
| 前端 | `V4RuntimeEvidencePanel` 递归展示嵌套 machine；新增 `ComplexityBudgetPanel` |
| 治理/文档 | OpenAPI、能力治理快照、全量树、README、CHANGELOG、里程碑索引、当前状态文档同步到 v4.4.0 |

## 二、验证结果

| 验证 | 结果 |
|---|:--:|
| `cargo fmt --check` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo test --workspace` | ✅ |
| `cargo test -p qrpc-core-ir` | ✅ |
| `cargo test -p quantscript v4_static_audit` | ✅ |
| `cargo test -p qrpc-runtime v4_` | ✅ |
| `npx vitest run src/components/V4RuntimeEvidencePanel.test.jsx src/utils/v4RuntimeEvidence.test.js` | ✅ |
| `npm run build` (frontend) | ✅ |
| `npm run build` (frontend-executor) | ✅ |
| `tools/check-version-consistency.ps1` | ✅ |
| `tools/check-capability-governance.ps1` | ✅ |
| `tools/check-full-feature-tree.ps1` | ✅ |
| `tools/run-closeout-gates.bat` | ✅ 24/24 |

## 三、自由维度诱错

审计报告: `markdown/09-archive/testing-retired/自由维度诱错审计-v4.4.0-第1轮.md`

结论: 无 S0。已覆盖类型契约、QS 静态审计、runtime 路由、证据回放、UI/治理五个维度。

## 四、保留边界

- 深度 ≥3 的嵌套仍为 reserved。
- 子 machine 不允许跨父通信。
- 顶层 DAG 不随本版本改变。
- 不新增默认嵌套策略模板。
- 嵌套状态机保持 beta，必须持续展示复杂度预算和层级 evidence。
