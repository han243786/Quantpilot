# QPCursor 总游标协议

QPCursor 是 QuantPilot 新治理的当前执行坐标。它不是普通任务备注，而是代理接管契约。

目标:

```text
任何代理拿到一个 QPCursor，
不需要读聊天历史，
只要按游标读取指定文档、模块、叶子和门禁，
就能进入正确开发节奏。
```

## 1. 必须回答的问题

每个 QPCursor 必须同时回答:

1. 当前处于哪种开发模式。
2. 当前是该模式里的第几步。
3. 当前处于超级规范化哪条通道、哪个阶段。
4. 当前落在哪个用户功能切面。
5. 当前深入到全量树、模块树的哪个节点和哪个叶子。
6. 下一步允许做什么、禁止做什么、完成后如何推进游标。

## 2. 短游标

短游标用于任务标题和人工快速识别。

```text
QPC://v4.10.0/FM:F4/SSP:FEATURE_IMPL+CAP_TRUTH_UI/UFF:realtime_executor.orders/FFT:root3.executor/MT:executor.live_runner.order_panel/LEAF:OrderPanel.jsx
```

字段含义:

| 字段 | 含义 |
| --- | --- |
| `FM:F4` | 切面打磨模式，第 4 步内部优化 |
| `SSP` | 当前超级规范化通道位置 |
| `UFF` | 用户功能切面 |
| `FFT` | 全量树位置 |
| `MT` | 模块树位置 |
| `LEAF` | 文件、组件、函数、测试或文档叶子 |

## 3. 长游标字段

长游标用于代理接管和机器校验，可以用 Markdown 表、JSON 或其他可校验结构承载。试运行阶段不强制新建 YAML 文件。

必需字段:

| 字段 | 职责 |
| --- | --- |
| `cursor_version` | 游标协议版本 |
| `cursor_id` | 当前游标唯一 ID |
| `status` | draft、claimed、reading_context、executing、evidence_pending、gate_pending、handoff_ready、closed、blocked |
| `repo_baseline` | 版本、分支、基线提交、里程碑 |
| `mode_stack` | 推进、重构、切面打磨的可嵌套模式栈 |
| `super_pipeline` | SSP、FE、MAJ、CAP、GATE、AUD 通道位置 |
| `scope` | UFF、FFT、MT、LEAF 四轴定位 |
| `interface_freeze` | API、capability、event schema、persistence schema、UI route 等冻结项 |
| `allowed_workset` | 可编辑文件、只读文件、变更后需同步文档 |
| `next_action` | 下一步原子动作、完成条件、停止条件 |
| `evidence` | 必跑命令、人工检查、已执行证据、未决风险 |

## 4. 模式栈

总游标不是单态，而是模式栈。一个切面打磨任务内部可以调用重构或推进模式。

```text
mode_stack:
  - mode: facet_polish
    step: FM.F4_internal_optimization
  - mode: refactor
    step: RM.R3_relink_module_tree
```

解释:

```text
外层任务 = 切面打磨
当前内部动作 = 重构
当前重构目的 = 整理该切面内部模块树
```

## 5. 三种开发模式

### 推进模式 PM

用于在已有秩序内新增功能或扩展能力。

| 步骤 | 含义 |
| --- | --- |
| PM.P0 | 任务定界 |
| PM.P1 | 功能演进判定 |
| PM.P2 | 全量树功能声明 |
| PM.P3 | 模块树结构声明 |
| PM.P4 | 契约、capability、API、数据结构接入 |
| PM.P5 | 叶子实现 |
| PM.P6 | 测试与回归保护 |
| PM.P7 | 文档、全量树、closeout 证据同步 |

### 重构模式 RM

用于偿还系统熵债，冻结旧行为并重连结构。

| 步骤 | 含义 |
| --- | --- |
| RM.R0 | 熵源声明 |
| RM.R1 | 旧行为冻结 |
| RM.R2 | 代码、模块、依赖盘点 |
| RM.R3 | 模块树重连 |
| RM.R4 | 代码抽离、移动、适配 |
| RM.R5 | 兼容桥、回退路径、拒绝路径 |
| RM.R6 | 全量树与模块树同步 |
| RM.R7 | 回归验证 |
| RM.R8 | 退出到推进模式或切面打磨模式 |

### 切面打磨模式 FM

用于冻结外部接口后，对完整用户功能切面做深度优化。

| 步骤 | 含义 |
| --- | --- |
| FM.F0 | 切面选择 |
| FM.F1 | 外部接口冻结 |
| FM.F2 | 依赖地图 |
| FM.F3 | 切面内部模式选择 |
| FM.F4 | 内部推进、重构、优化 |
| FM.F5 | 回灌整合 |
| FM.F6 | 外溢回归 |
| FM.F7 | closeout 与切面状态更新 |

## 6. 接管协议

任何代理接到 QPCursor 后，只能按以下顺序工作:

1. 读取当前游标。
2. 校验 `cursor_version`、`status`、`mode_stack`。
3. 读取游标指定的源文档。
4. 读取 `allowed_workset` 中的文件。
5. 复述当前模式、步骤、允许动作、禁止动作。
6. 执行 `next_action`。
7. 运行 `required_commands`。
8. 写入 evidence。
9. 推进、阻断或请求扩范围。

## 7. 生成器辅助

`tools/new-qpcursor-trial.ps1` 可以从旧 `recursive-state.json` 生成 QPCursor 草案，用于减少手工重复和接力漂移。

生成器输出只能是草案，必须人工或代理补齐:

1. `allowed_workset` 的真实文件范围。
2. `evidence` 的实际命令结果。
3. `trial judgment` 的质量判断。
4. 若与旧治理冲突，必须保留 `legacy_governance_authority: preserved` 并回退旧治理。

示例:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\new-qpcursor-trial.ps1 `
  -TrialId 0003 `
  -Scope root.contracts.runtime_support.v4_runtime_support.simulated_execution_engine `
  -LeafPath qrpc_runtime/src/v4_simulated_execution.rs
```

## 8. 停止规则

如果代理发现需要触碰游标外文件、改变外部接口、改变 capability、改变 API、改变持久化 schema、改变执行模式语义，必须停止并输出范围变更请求。
