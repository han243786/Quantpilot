# GOV-GOVERNANCE-NEXT-PROMOTION-01 新治理权威入口接管

## 1. 批次信息

| 字段 | 内容 |
| --- | --- |
| batch | `GOV-GOVERNANCE-NEXT-PROMOTION-01` |
| milestone | v4.16.0 |
| 类型 | 文档治理 / 权威入口切换 |
| 代码移动 | 无 |
| Rust 递归游标 | 不移动，继续停在 `BE-002EG-01` |

## 2. 决策

`governance-next/` 从旁路孵化区升级为默认权威治理入口。旧 `markdown/00-matrix-governance/` 不删除，但降级为兼容档案、历史规则和旧门禁素材库。

promote 后默认关系为:

```text
governance_next_authority: active
legacy_governance_mode: archived_reference
qpcursor_required: true
```

## 3. 接管范围

| 区域 | 新角色 |
| --- | --- |
| `governance-next/README.md` | 默认治理入口 |
| `governance-next/05-authoritative-operating-model.md` | 权威运行模型 |
| `governance-next/01-qpcursor-protocol.md` | 代理接管游标协议 |
| `governance-next/02-governance-heat-trigger.md` | G0-G5 治理热度 |
| `governance-next/03-local-invariants.md` | 模块、接口、边界局部不变量 |
| `markdown/00-matrix-governance/README.md` | 旧三矩阵兼容档案 |
| `markdown/00-matrix-governance/recursive-speed-protocol.md` | 旧递归提速素材，不再覆盖 QPCursor |
| `markdown/00-matrix-governance/recursive-state.json` | 当前递归游标暂存 |

## 4. promotion 依据

| 依据 | 结论 |
| --- | --- |
| QPCursor 样本 | `risk_execution_gate` 和 `simulated_execution_engine` 均达到 handoff_ready |
| 成品质量 | `split_decision` 与 `governance_packaging` 已分离，避免把 wave 误读为继续拆分许可 |
| 运行效率 | terminal leaf control v2、叶子粒度评分脚本、QPCursor 生成、未跟踪文件预检和索引降重路线已进入治理体系 |
| 成本控制 | 新治理允许 Light / Standard / Precision 三档执行，减少旧治理固定重文档成本 |
| 接力能力 | 游标、allowed workset、stop_if、evidence 成为代理接管核心字段 |

## 5. 旧治理降级规则

1. 旧三矩阵不得再被声明为默认治理入口。
2. 旧递归高速协议不得覆盖 QPCursor 的 allowed workset、stop_if 和 evidence。
3. 旧文件仍需保留，直到对应信息迁入新治理或生成器后再单独提出收敛方案。
4. 兼容门禁仍可检查旧文件存在和基础结构，但通过旧门禁不代表新治理完成。

## 6. 对当前递归进程的影响

本批次不改变 Rust 抽离进度。当前递归状态保持:

```text
current_parent: root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine
current_step: BE-002EG-01
current_phase: parent_residual_judgment
next_recommended_child: root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine.order_lifecycle_flow
```

下一轮代码递归继续从 `order_lifecycle_flow` 的 parent residual judgment 进入，但默认执行入口改为新治理。

## 7. 门禁

本批次应运行:

1. `git diff --check`
2. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
3. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
4. `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
