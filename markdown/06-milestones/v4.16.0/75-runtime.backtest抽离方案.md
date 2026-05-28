# v4.16.0 runtime.backtest 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001M-02。  
> 基准: `74-runtime.backtest单子叶等价基线.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest` 抽离方案，`no code movement`。下一批若实施，只允许抽离 backtest route facade 到 `src/backend/runtime/routes/backtest.rs`，不得迁移 handler、artifact schema、compare owner、persistence owner、replay helper、state owner、schema owner、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001M `runtime.backtest` 从等价基线进入抽离方案 | 推进 |
| 规范矩阵 | backtest route facade 最小移动、handler owner 保留、artifact/compare/replay/persistence owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest` | 抽离方案 |
| 模块树 | `runtime.backtest` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime/backtest.rs`、`src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/runtime/mod.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| 下一批计划目标文件 | `src/backend/runtime/routes/backtest.rs` |
| 当前 public 方法 | `start_backtest_run`、`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`、`compare_backtests` |
| 保留 shared helper | `execute_backtest_request`、`execute_v4_backtest_request`、`build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、`load_backtest_record_from_state`、`persist_backtest_record`、`list_backtest_records`、`normalized_replay_options`、`backtest_replay_response_from_record` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 方案选择

第一轮实际抽离选择 route facade，而不是 handler 文件切片。

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| backtest route facade | 采纳 | 只移动 route registration，能把 `backend.runtime.routes` 父级聚合继续瘦身，同时不碰 handler、artifact、persistence 和 state owner |
| handler 文件切片 | 暂缓 | `src/runtime/backtest.rs` 同时包含 start/list/detail/save/discard/replay、v4 backtest、experiment helper 和 shared replay 关系；第一刀直接切 handler 容易扩大牵连 |
| artifact/persistence 拆分 | 排除 | artifact bundle、transient spill、manifest digest、persistence lookup 都是共享 owner，不属于本轮 handler/route 抽离 |
| compare owner 拆分 | 排除 | `compare_backtests` 已在 `src/backtest_compare.rs`，本轮只让 route facade 调用它，不迁移 compare core/narrative |

---

## 最小抽离方案

| 项 | 方案 |
| --- | --- |
| 目标 facade | 下一批新建 `src/backend/runtime/routes/backtest.rs`，只承载 backtest route group registration |
| 父级 route | `src/backend/runtime/routes.rs` 新增 `pub mod backtest;`，用 `backtest::register_routes(router)` 替代当前内联 backtest route 链 |
| handler owner | `src/runtime/backtest.rs` 不变，继续通过 `crate::runtime::*` 兼容出口暴露 handler |
| compare owner | `src/backtest_compare.rs` 不变，route facade 继续调用 `backtest_compare::compare_backtests` |
| run route facade | `backend.runtime.routes.run` 不变，不接管 backtest |
| event stream | `runtime.event_stream` 不变，不接管 backtest replay 或 route |
| artifact/persistence/schema | `src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 不变 |

下一批的目标 facade 形态应保持与 `backend.runtime.routes.run` 类似:

```rust
use axum::{
    routing::{get, post},
    Router,
};

use crate::{backtest_compare, runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.backtest";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/backtest",
            post(runtime_handlers::start_backtest_run),
        )
        .route(
            "/api/runtime/backtests",
            get(runtime_handlers::list_backtests),
        )
        .route(
            "/api/runtime/backtests/compare",
            post(backtest_compare::compare_backtests),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/save",
            post(runtime_handlers::save_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id",
            get(runtime_handlers::get_backtest_detail)
                .delete(runtime_handlers::discard_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/replay",
            get(runtime_handlers::get_backtest_replay),
        )
}
```

父级 `src/backend/runtime/routes.rs` 目标形态:

```rust
pub mod backtest;
pub mod run;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = backtest::register_routes(router);
    let router = run::register_routes(router);

    router
        // event stream, evidence, mutation, report, experiment, approval and ops routes remain here
}
```

---

## 明确排除

- 不迁移 `src/runtime/backtest.rs` 中的任何 handler、helper、v4 backtest helper 或 experiment helper。
- 不把 `start_backtest_experiment`、`/api/runtime/experiments/*`、report、mutation、approval、event stream、`runtime.run` 或 run routes 放入 backtest route facade。
- 不改变 backtest route path、method、handler 调用、route order、response schema 或 error code。
- 不迁移 `src/backtest_artifacts.rs`、artifact schema、manifest、transient spill、digest 或 governance rebuild 逻辑。
- 不迁移 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、AppState owner、store dir 或锁顺序。
- 不私有化 `normalized_replay_options`、`RuntimeReplayQuery`、`RuntimeReplayResponse` 或 `backtest_replay_response_from_record`。
- 不主动提出发布版本过渡，不新增子模块横向连接。

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| route path 漂移 | 下一批必须只复制现有 route 链，不改 path/method/handler |
| compare import 遗漏 | `src/backend/runtime/routes/backtest.rs` 继续 `use crate::{backtest_compare, runtime as runtime_handlers, AppState};` |
| experiment route 被误迁入 | 中止；`start_backtest_experiment` 和 `/api/runtime/experiments/*` 留在父级或后续 sibling |
| handler 切片诱惑 | 中止；本轮实际抽离只处理 route facade，handler 文件切片必须另起后续方案 |
| artifact/persistence 牵连 | 中止；若需要改 artifact、persistence、schema、state owner，则说明本方案粒度过大 |
| 回归失败 | `api_backtest`、`api_evidence_contract` 或 `api_run` 失败时先修复等价缺口，不继续推进 closeout |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 route path、route method、handler 调用、response schema 或 error code。
2. 需要移动 `start_backtest_run`、`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay` 或 `compare_backtests` 的实现。
3. 需要迁移 `execute_backtest_request`、`execute_v4_backtest_request`、artifact helper、persistence helper、response mapping 或 schema owner。
4. 需要把 experiment/report/mutation/event stream/run routes 混入 backtest facade。
5. 需要改变 AppState、store dir、transient spill、manifest digest 或锁顺序。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过保持现有 import 和父级 route facade 解决。
7. `cargo test -p quantpilot --test api_backtest`、`api_evidence_contract` 或 `api_run` 出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批进入 `BE-001M-03 runtime.backtest 抽离记录`。实施范围只能是:

1. 新建 `src/backend/runtime/routes/backtest.rs`。
2. 将 `src/backend/runtime/routes.rs` 中的 backtest route group 迁入该 facade。
3. 在 `src/backend/runtime/routes.rs` 保留父级注册顺序: 先 `backtest::register_routes(router)`，再 `run::register_routes(router)`，再注册 event stream、evidence、mutation、report、experiment、approval 和 ops routes。
4. 保持所有 handler、artifact、compare、persistence、schema、state 和 frontend owner 不变。
5. 补 BE-001M-03 抽离记录与模块树/全量树/门禁状态。

---

## 幻觉检查点

AI 声称 `runtime.backtest` 已有抽离方案时，必须说明本批没有迁移代码，只允许下一批最小移动 backtest route registration 到 `src/backend/runtime/routes/backtest.rs`。不得宣称 handler 已抽离，不得宣称 artifact/persistence/schema 已迁移，不得把 experiment/report/mutation/event stream 混入 backtest。

---

## 验收标准

1. `75-runtime.backtest抽离方案.md` 进入 v4.16 里程碑索引、全量树和模块树。
2. 治理门禁能发现本方案缺失。
3. 方案明确下一批只抽离 backtest route facade 到 `src/backend/runtime/routes/backtest.rs`。
4. 方案明确 `src/runtime/backtest.rs`、`src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs` 和 `src/frontend_api_types.rs` 均保持原 owner。
5. 方案明确 `start_backtest_experiment`、report、mutation、event stream、run routes 和发布版本过渡不属于本批。
6. 本批不发生代码移动。
