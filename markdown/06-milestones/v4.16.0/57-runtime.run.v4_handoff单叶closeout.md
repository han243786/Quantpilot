# v4.16.0 runtime.run.v4_handoff 单叶 closeout
> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001H-03。  
> 基准: `55-runtime.run.v4_handoff单子叶等价基线.md`、`56-runtime.run.v4_handoff抽离记录.md`。  
> 判定: `runtime.run.v4_handoff` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆。`stop_split: true`。后续应回到 `runtime.run` 父级 sibling 队列，而不是继续把本叶拆成更小 helper 文件。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001H handler 层单叶 closeout | 收口 |
| 规范矩阵 | 父级出口、state owner、provider 边界、细分停止条件 | 固化 |
| 引导矩阵 | `runtime.run.v4_handoff` 白箱节点 | 收口 |
| 模块树 | `runtime.run.v4_handoff` | closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.v4_handoff` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 与根7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.v4_handoff` |
| 真实文件 | `src/runtime/run/v4_handoff.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/backend/runtime/routes/run.rs` |
| public 方法 | `start_v4_runtime_run`、`runtime_v4_static_bundle`、`runtime_simulated_v4_matrix` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 整理结论

| 检查项 | 结论 | 说明 |
| --- | --- | --- |
| route 入口 | 等价 | `/api/runtime/v4/run` 仍经 `backend.runtime.routes.run -> crate::runtime::start_v4_runtime_run` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `run_v4_handoff` 私有子模块与 `pub(crate)` re-export |
| state owner | 保留 | `run_in_progress` 仍归 `AppState`，不迁移锁顺序或内存序语义 |
| provider 边界 | 保留 | 只登记 RuntimeSimulated，不引入 provider 真连接 |
| sibling 边界 | 保留 | `start_test_run`、record store、replay/status、SSE、mutation、backtest handler 不属于本叶 |
| 共享 helper | 受控 | `runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` 由父级导入供 backtest 复用，不形成 sibling 横连 |

---

## 内部细分价值判断

| 候选内部子叶 | 是否继续细拆 | 判断 |
| --- | :--: | --- |
| request / response schema | 否 | 字段少且只服务单 route，拆出会增加文件跳转和父级桥接成本 |
| source / graph resolution | 否 | 逻辑虽有分支，但当前只服务 v4 handoff；`api_run` 已能覆盖 source、graph、missing input 和 rejected handoff |
| initial event / default payload | 否 | 逻辑窄，主要依附 event catalog fallback，继续拆分收益低 |
| handoff response projection | 否 | 只是 response schema projection，独立文件没有新的 owner 收益 |
| simulated capability matrix | 暂不在本叶内拆 | 该 helper 已被 backtest 复用；若未来要独立，应另起父级共享节点，例如 `runtime.v4_capability_matrix`，不能在本叶内部横向连出去 |

**最终判定**: `runtime.run.v4_handoff` 当前不继续细拆。它已经是一个可维护的叶子模块，内部 helper 均服务同一条 v4 handoff route；继续拆只会制造更细的父子桥，而不会明显降低复杂度。

---

## 等价证据

| 证据 | 覆盖 |
| --- | --- |
| `cargo check -p quantpilot` | 子模块可见性、父级 re-export、route handler 类型 |
| `cargo test -p quantpilot --test api_run` | `/api/runtime/v4/run` source/graph/initial event/error path |
| `cargo test -p quantpilot --test api_backtest` | shared simulated matrix helper 对 backtest 路径不漂移 |
| `tools/check-matrix-governance.ps1` | closeout 文档、模块树、全量树锚点 |
| `tools/check-full-feature-tree.ps1` | 新 closeout 文档路径与真实文件覆盖 |

---

## 禁止事项

- 不把本 closeout 宣称为 `src/runtime/run.rs` 全部完成。
- 不迁移 `start_test_run`、record store、replay/status、SSE、AppState owner、persistence 或 backtest handler。
- 不扩大 provider 支持，不引入 provider 真连接。
- 不主动提出发布版本过渡。
- 不把 `runtime_v4_static_bundle` / `runtime_simulated_v4_matrix` 的 backtest 复用解释为 sibling 横向连接许可。

---

## 后续入口

本叶 closeout 后，递归流程返回 `runtime.run` 父级 sibling 队列。可选下一候选:

| 候选 | 建议 |
| --- | --- |
| `runtime.run.session_start` | 默认下一候选；覆盖 legacy `/api/runtime/test-run` 与运行启动写入链，需先建等价基线 |
| `runtime.run.record_store` | 可选；涉及 list/detail/save/discard 和 persistence，风险高于 session start |
| `runtime.run.replay_status` | 可选；涉及 replay/status projection，适合在 record owner 明确后处理 |
| `runtime.event_stream` | 独立 route 子叶；不属于 `backend.runtime.routes.run` closeout，也不属于本叶 |

下一步若继续 runtime run handler，必须先建立新候选的单子叶等价基线，不得直接迁移实现。

---

## 验收标准

1. `57-runtime.run.v4_handoff单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.run.v4_handoff` closeout 完成并停止内部细分。
3. 全量树覆盖本 closeout 文档与 `src/runtime/run/v4_handoff.rs`。
4. 治理门禁能发现本 closeout 文档缺失。
5. `api_run` 与 `api_backtest` 等价检查通过。
