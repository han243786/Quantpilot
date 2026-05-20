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
