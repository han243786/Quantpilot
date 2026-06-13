# QuantPilot 工件治理

## 目的

本文档是持久化工件的 P1 治理层。它定义了：

- 工件清单版本规则
- 保留默认策略
- 安全清理边界
- 工件模式变更时所需的证据

本文档不引入新的运行时或回测功能。它仅治理已有工件的存储和维护。

## 工件范围

当前仓库中的存储模式包括：

- `storage/graphs/*.json`
- `storage/graphs/*.qs`
- `storage/graphs/latest.json`
- `storage/runs/*.json`
- `storage/backtests/<backtest_id>/*`（如果存在）
- `storage/audit/*.json`
- 测试和确定性回放运行创建的 `storage/test-*` 目录
- 根级别存储日志，如 `*.log`、`*.err.log`、`*.out.log`

## 模式版本策略

工件系列必须携带显式的 `schema_version` 值。当前仓库已包含版本化工件，例如：

- `quantpilot/reproducibility-manifest/v1`
- `quantpilot/backtest-spec/v1`
- `quantpilot/run-spec/v1`
- `quantpilot/strategy-artifact/v1`
- `quantpilot/compile-artifact/v1`

### 版本化规则

- 任何结构性的工件变更必须保留或有意识地提升 `schema_version`。
- 版本提升需要：
  - 在本文件中更新文档
  - 更新 fixture 或快照
  - 提供迁移说明或兼容性声明
- 不得静默重用现有的 `schema_version`。
- 仅当旧读取器可以安全忽略时，向后兼容的增补字段才可以保持在同一版本中。

## 清单期望

可重现性清单是回测包的工件锚点。它至少应保留：

- 清单标识：`manifest_id`、`schema_version`、`created_at_ms`
- 运行标识：`backtest_id`、`graph_id`、`compile_id`
- 编译标识：`protocol_name`、`config_hash`
- 摘要和账户快照
- 嵌入的编译和运行规范或其稳定引用

### 当前治理期望

- `manifest.json` 是回测工件检查的入口点。
- 配套文件如 `metrics.json`、`event_log.json`、`equity_curve.json` 和 `trade_ledger.json` 是次要视图。
- 前端页面应优先使用清单驱动或工件优先的读取方式，而非假设传统的摘要负载。

## 保留默认策略

Beta 阶段保留策略偏保守。默认策略是"保留面向生产的记录，清理临时测试材料"。

| 存储区域 | 默认保留 | 清理默认 |
|---|---|---|
| `storage/graphs/*.json` | 保留 | 清理脚本从不触碰 |
| `storage/graphs/latest.json` | 保留 | 清理脚本从不触碰 |
| `storage/graphs/*.qs` | 保留 | 清理脚本从不触碰 |
| `storage/runs/*.json` | Beta 期间保留 | 默认不删除 |
| `storage/backtests/**` | Beta 期间保留 | 默认不删除 |
| `storage/audit/*.json` | Beta 期间保留 | 默认不删除 |
| `storage/test-*` | 临时 | 有资格清理 |
| `storage/*.log`、`storage/*.err.log`、`storage/*.out.log` | 运维用途 | 仅可选清理 |

### 此默认策略的原因

- 图和运行记录仍有助于调试和回归审查
- 回测包是当前以工件为先的历史页面真实数据源
- 测试生成的工件快速累积，产生噪音但不增加长期产品价值

## 清理策略

仓库现在包含一个安全的清理入口点：

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\cleanup-artifacts.ps1`

默认行为：

- 仅 dry-run 模式
- 限定于 `storage/`
- 针对早于所选阈值的临时 `storage/test-*` 目录
- 不触碰面向生产的图、运行或回测工件

可选行为：

- `-IncludeLogs` 还包含根级别存储日志
- `-Mode execute` 在相同路径检查通过后执行删除操作

## 安全规则

- 清理必须解析仓库 `storage/` 目录下的所有目标。
- 清理绝不能递归到声明的存储根目录之外。
- 清理绝不能删除 `storage/graphs/latest.json`。
- 在默认模式下，清理绝不能删除面向生产的图、运行或回测。
- 在 dry-run 模式下，清理输出必须在删除前列出每个目标。

## 变更检查清单

当工件模式或保留行为发生变化时，更新：

- 本治理文档
- fixture 样本或快照
- 任何读取工件的前端测试
- 任何清理工具的假设

## 参考

- [implementation-support-matrix.md](./implementation-support-matrix.md)
- [implementation-testing-module.md](../runtime/implementation-testing-module.md)
- [Current Status And Release State](../../10-overview/overview-current-status-and-roadmap.md)
