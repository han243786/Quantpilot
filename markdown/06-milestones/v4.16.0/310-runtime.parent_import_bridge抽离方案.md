# v4.16.0 runtime.parent_import_bridge 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CX-02
> 基准: `309-runtime.parent_import_bridge单子叶等价基线.md`
> 目标子叶: `runtime.parent_import_bridge`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CX-02 `runtime.parent_import_bridge` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | import bridge plan |
| 模块树 | `runtime.parent_import_bridge` | 方案登记 |

---

## 当前依赖分层

基于 BE-001CX-01 当前扫描，`src/runtime/**.rs` 中仍有 46 个文件存在 `use super::*` 或 `super::` 依赖。按层级拆分如下:

| 层级 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 自身仍通过 `use super::*` 获取上层 backend surface |
| `runtime.root_child` | 6 | runtime 直属 support / facade 文件 |
| `runtime.run` | 4 | run 子树 |
| `runtime.backtest` | 11 | backtest 子树 |
| `runtime.report_ops` | 3 | report_ops 子树 |
| `runtime.mutation` | 21 | mutation 子树，依赖最密集 |

---

## 方案判定

采用 staged explicit import pass，不采用一次性全量替换。`runtime.parent_import_bridge` 不是新增 Rust 业务模块，而是父级 import bridge 消除流程。

### 执行顺序

1. BE-001CX-03: `runtime.root_support_import_pilot` 实际抽离。
2. BE-001CX-04: `runtime.root_support_import_pilot` 单叶 closeout。
3. 若 BE-001CX-04 判断继续拆分，则再按子层进入等价基线；若无需继续拆分，则进入下一批 root child / run / backtest / report_ops / mutation。
4. 所有子层 explicit import pass 完成后，才能进入 `runtime.parent_import_bridge` 父叶残余判断。
5. 父叶残余判断仍为 `stop_split: false` 时，继续递归；只有明确 `stop_split: true` 后，才能处理下一个 backend 顶层子叶。

### 第一批允许修改

BE-001CX-03 只允许处理低耦合 root support pilot:

```text
src/runtime/query_support.rs
src/runtime/response_support.rs
```

允许动作:

1. 将上述两个文件顶部的 `use super::*` 收敛为显式 import。
2. 只补足这两个文件实际使用的 schema / helper / serde import。
3. 如 cargo check 证明 visibility 不足，可在最小范围内调整父级受控 surface，但必须记录原因。
4. 更新本里程碑、模块树、全量树和治理门禁。

不允许动作:

1. 不迁移 handler、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState 或 lock order。
2. 不处理 `event_stream`、`evidence_health`、`report_ops`、`run`、`backtest` 或 `mutation`。
3. 不新增 sibling horizontal link。
4. 不启动 release transition。
5. 不删除 `src/runtime/mod.rs` 的 `use super::*`。

---

## 后续批次原则

后续每批必须满足:

1. 每批最多处理一个子树或一个小 support 组合。
2. 每批必须先有方案或 closeout 判断，不得跳过递归节点。
3. child 需要共享 helper 时，优先经父级受控 surface；不得从 sibling 直接 import。
4. 若出现需要横向连接提升性能的想法，必须判定为 release transition 事项；开发者未明确决定发布版本过渡前，AI 不得主动提出或执行。
5. 每批必须能被 `cargo check -p quantpilot` 与相关 API 测试覆盖。

建议顺序:

```text
runtime.root_support_import_pilot
runtime.root_entry_import_pass
runtime.run_import_pass
runtime.backtest_import_pass
runtime.report_ops_import_pass
runtime.mutation_import_pass
runtime.parent_import_bridge_residual_judgement
```

---

## 回退点

若 BE-001CX-03 失败，回退范围仅限:

1. `src/runtime/query_support.rs` 的显式 import 改写。
2. `src/runtime/response_support.rs` 的显式 import 改写。
3. 与 BE-001CX-03 同批新增的治理文档和门禁锚点。

不得回退已完成的 `runtime.parent_include_cleanup`、`backend.runtime` 第九轮父叶残余判断、309 基线或其他已 closeout 子模块。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CX-03 实际 import pilot 后，至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
```

若实际 import 触达 report / evidence surface，还必须补跑对应 `api_v1_reports` 或 `api_evidence_contract`。

---

## 下一步

下一步只允许进入:

```text
BE-001CX-03 runtime.root_support_import_pilot 实际抽离
```

BE-001CX-03 只允许改写 `src/runtime/query_support.rs` 与 `src/runtime/response_support.rs` 的 parent wildcard import，并保持行为等价。不得顺手处理 `src/runtime/mod.rs` 父桥、run/backtest/mutation/report_ops 子树或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CX-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 当前真实残余仍是 parent import bridge。
3. 当前扫描仍是 46 个 runtime 文件存在 `use super::*` 或 `super::` 依赖。
4. 下一步只能进入 BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离。
5. BE-001CX-03 只能处理 `query_support` 与 `response_support` 两个文件。

不得宣称 parent import bridge 已消除、`backend.runtime` 已完成、Rust 重构已完成或 release transition 已启动。

---

## 验收标准

1. `310-runtime.parent_import_bridge抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 staged explicit import pass，不允许一次性批量改写 46 个文件。
3. 下一步固定为 BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
