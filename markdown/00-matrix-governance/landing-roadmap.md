# 三矩阵治理落地路线

> 状态: v4.15.0 已完成接管，v4.16.0 起进入模块化三步走的抽离阶段。
> 目标: 把三矩阵从文档草案推进到项目级默认控制平面。
> 范围: 文档治理、提案流程、模块树、引导矩阵、发布过渡协议和后续门禁规划。

---

## 1. 完全落地定义

新治理方案只有同时满足以下条件，才视为完全落地:

1. 所有开发入口都能发现 `markdown/00-matrix-governance/`。
2. 所有变更都能按轻量、标准、重型三档判定。
3. 标准档和重型档都能写出三矩阵影响声明。
4. 重型档必须写出引导坐标，包含全量树节点、模块树节点、真实文件、public 方法、测试或门禁。
5. 关键 public 方法纳入模块树白箱节点。
6. 默认开发态严格执行父子通信，不允许子模块横向直连。
7. 只有开发者明确声明进入发布版本过渡时，才允许提出横向连接、旁路缓存或热路径直连方案。
8. AI 不得主动提议进入发布版本过渡。
9. closeout 门禁能够发现缺失的治理声明、模块树节点或索引漂移。
10. `General_Policy.md`、超级规范化和全量树转为三矩阵的历史主干，不再作为绕过新流程的独立入口。

---

## 2. 里程碑链

| 里程碑 | 名称 | 主目标 | 退出条件 |
| --- | --- | --- | --- |
| v4.12.0 | 三矩阵治理入口启用 | 建立新顶层目录、提案状态机、三档判定、发布过渡协议和种子模块树 | 新目录进入总索引，旧文件暂存为历史主干，后续路线明确 |
| v4.13.0 | 模块树白箱扩面 | 将种子模块树扩展为覆盖主要 active 模块的白箱网络 | 重型变更能定位父模块、public 方法和回归保护 |
| v4.14.0 | 治理门禁自动化 | 将三矩阵声明、模块树漂移、索引漂移和发布过渡保护接入检查脚本 | closeout 能阻断缺失治理声明的变更 |
| v4.15.0 | 三矩阵完全接管 | 完成旧入口导流、模块树覆盖、提案模板强制和治理 closeout | 新治理方案成为默认开发约束体系 |
| v4.16.0 | 模块化抽离第一波 | 面向十万行级重大工程，建立目标登记、批次切片、决策暂停、兼容桥和等价验证流程，并完成 system.entry 启动抽离、经验回填、递归模块化流程、S1-S10 closeout 或静态 closeout、`root.system` 顶层阶段性 closeout；backend 已完成 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D strategy_config L3 模块壳抽离、BE-001E 其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `runtime.backtest.execution_start.v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `runtime.backtest.execution_start.v4_runtime_execution` 单叶 closeout，BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout，BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout，BE-001Z-01 已完成第二轮父叶残余判断，BE-001AA-04 已完成 `record_lifecycle` 单叶 closeout，BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true` | 抽离可以分片推进，整理和重构仍待后续方案 |

---

## 3. 阶段边界

### v4.12.0: 入口启用

本阶段只建立治理控制面，不要求补齐全项目模块树。所有旧文档继续存在，并作为新矩阵的引用主干。

### v4.13.0: 白箱扩面

本阶段补模块树覆盖，不允许借模块化名义直接删除旧实现。重大重构可以新开模块化版本，原版本保留到适配接口、测试和回归保护全部通过。

### v4.14.0: 门禁接入

本阶段把规则变成可检查约束。检查脚本只负责阻断缺失声明和明显漂移，不替代人工设计判断。

### v4.15.0: 完全接管

本阶段做治理 closeout。旧文件不删除，但入口顺序、提案模板和 closeout 口径全部转向三矩阵。完成后，GP、超级规范化和全量树继续作为被引用主干存在，默认开发入口为 `README.md -> markdown/00-matrix-governance/README.md`。

### v4.16.0: 模块化抽离第一波

最新状态补充: BE-001AE-04 `backend.runtime.routes.mutation` 单叶 closeout 已完成，route facade 等价并设置 `stop_split: true`；BE-001AF-02 已建立 `runtime.mutation.parameter_mutation` 抽离方案，下一步只能进入 BE-001AF-03 实际抽离。handler、AppState、锁顺序、schema、frontend caller 和发布过渡均未迁移。

本阶段先完成抽离路线铺设，再用低风险 system.entry 做代码试水并完成启动边界抽离。system 经验已回填为后续抽离准则，要求 public 入口、兼容入口、关键内部实现和保留外部边界分开登记；10 个 system 叶子的等价基线已用于判断后续单叶抽离顺序；S1 启动脚本、S3 Tauri runtime、S4 Tauri config、S7 desktop build/dev scripts 和 S10 配置样例 closeout 均已确认等价且不继续细分；递归模块化流程已明确顶层模块、叶子抽离、叶子整理、细分价值判断和全局根收束。backend 已完成九叶模块壳抽离、九叶逐叶 closeout、`backend.strategy_config` L3 模块壳抽离、其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout；当前结论是该叶不继续细拆，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `v4_request_resolution` 单叶 closeout 并设置 `stop_split: true`，BE-001Q-04 已完成 `v4_runtime_execution` 单叶 closeout 并设置 `stop_split: true`，BE-001R-04 已完成 `legacy_dispatch` 单叶 closeout 并设置 `stop_split: true`，BE-001S-01 已完成父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout 并设置 `stop_split: true`，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout 并设置 `stop_split: true`，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout 并设置 `stop_split: true`，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout 并设置 `stop_split: true`，BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，BE-001AA-02 已建立抽离方案，BE-001AA-03 已完成实际抽离，BE-001AA-04 已完成单叶 closeout 并设置 `stop_split: true`；BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`；BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true`；该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案。目标是让后续模块化工作能先把功能从旧代码中拿出来，并通过父模块、模块树、public 方法、兼容桥和回归证据证明行为不变。

BE-001R-01 已建立 `runtime.backtest.execution_start.legacy_dispatch` 单子叶等价基线，冻结 legacy compile/sandbox dispatch；当前 `no code movement`，下一步只能先做 BE-001R-02 抽离方案。

BE-001R-02 已建立 `runtime.backtest.execution_start.legacy_dispatch` 抽离方案，限定下一批只迁移 legacy compile/sandbox dispatch 最小 helper。

BE-001R-03 已完成 `runtime.backtest.execution_start.legacy_dispatch` 第一轮物理抽离，将 legacy compile/assumption/artifact/sandbox replay 两段式 helper 迁入 `src/runtime/backtest/legacy_dispatch.rs`；父级仍保留 record assembly、artifact views、state write、audit log、schema、persistence、frontend 和发布过渡边界，下一步只能进入 BE-001R-04 单叶 closeout。

BE-001R-04 已完成 `runtime.backtest.execution_start.legacy_dispatch` 单叶 closeout，并设置 `stop_split: true`；下一步回到 `runtime.backtest.execution_start` 父叶残余判断或 `runtime.backtest` 上层队列，不能从 legacy dispatch 子叶继续细拆或外扩。

BE-001S-01 已完成 `runtime.backtest.execution_start` 父叶残余判断，确认四个内部子叶均已 closeout，父叶当前不再私拆 record/state/persistence 边界；下一步回到 `runtime.backtest` 上层队列，默认进入 `runtime.backtest.record_store` 单子叶等价基线。

BE-001T-01 已建立 `runtime.backtest.record_store` 单子叶等价基线，冻结 backtest list/detail/save/discard、transient/persistent record、artifact view、audit 和排除边界；当前 `no code movement`，下一步只能先做 BE-001T-02 抽离方案。

BE-001T-02 已建立 `runtime.backtest.record_store` 抽离方案，限定 BE-001T-03 只迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 四个 handler，并保持父级 re-export、route facade、shared helper owner、state owner、persistence owner、artifact/transient owner、frontend route 和发布过渡边界不变。

BE-001T-03 已完成 `runtime.backtest.record_store` 第一轮物理抽离，将 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 迁入 `src/runtime/backtest/record_store.rs`；route facade、shared helper owner、state owner、persistence owner、artifact/transient owner、frontend route 和发布过渡边界保持不变。

BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout，确认四个 backtest record store handler 等价并设置 `stop_split: true`；下一步必须回到 `runtime.backtest` sibling 队列，默认候选为 `runtime.backtest.replay` 等价基线。

BE-001U-01 已建立 `runtime.backtest.replay` 单子叶等价基线，冻结 replay route、record lookup、query normalization、response mapping、metrics 和排除边界；当前 `no code movement`，下一步只能先做 BE-001U-02 抽离方案。

BE-001U-02 已建立 `runtime.backtest.replay` 抽离方案，限定下一批只迁移 `get_backtest_replay`，并保持父级 re-export、route facade、record lookup、query normalization、response mapping、schema、metrics、state/persistence、artifact schema、frontend caller 和发布过渡边界不变。

BE-001U-03 已完成 `runtime.backtest.replay` 第一轮物理抽离，将 `get_backtest_replay` 迁入 `src/runtime/backtest/replay.rs`；route facade、record lookup、query normalization、response mapping、schema、metrics、state/persistence、artifact schema、frontend caller 和发布过渡边界保持不变。

BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout，确认 `get_backtest_replay` 等价并设置 `stop_split: true`；后续必须回到 `runtime.backtest` sibling 队列，默认候选为 `runtime.backtest.experiment_sweep` 等价基线。

BE-001V-01 已建立 `runtime.backtest.experiment_sweep` 单子叶等价基线，冻结 experiment routes、参数网格、`execute_backtest_request` 复用桥、experiment persistence、save/discard lifecycle、audit、response mapping 和排除边界；当前 `no code movement`，下一步只能先做 BE-001V-02 抽离方案。

BE-001V-02 已建立 `runtime.backtest.experiment_sweep` 抽离方案，限定下一批只迁移 5 个 experiment handler 和 3 个参数网格 helper；当前 `no code movement`，route aggregate、execution_start 复用桥、persistence、response mapping、schema、state、audit、frontend caller 和发布过渡均不迁移。

BE-001V-03 已完成 `runtime.backtest.experiment_sweep` 第一轮物理抽离，5 个 experiment handler 和 3 个参数网格 helper 已迁入 `src/runtime/backtest/experiment_sweep.rs`；下一步只能进入 BE-001V-04 单叶 closeout，不能混入 route aggregate、execution_start、persistence、response mapping、schema、state、audit、frontend caller 或发布过渡。

BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，确认等价但 `stop_split: false`；内部继续细分已由 BE-001W-01 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线承接，不能直接移动 helper、删除 drained parent include 或迁移 shared owner。

BE-001W-01 已建立 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线，冻结参数网格校验、轴归一化、base fallback、去重、variant count 和展开顺序；当前 `no code movement`，下一步只能进入 BE-001W-02 抽离方案，不能直接移动 helper、新增子文件、修改 schema 或宣称 closeout。

BE-001W-02 已建立 `runtime.backtest.experiment_sweep.parameter_grid` 抽离方案，限定下一批只允许迁移 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` 三个 helper 到父级私有子模块；当前 `no code movement`，下一步只能进入 BE-001W-03 实际抽离记录，不能混入 route、handler orchestration、execution_start、persistence、mapping、schema、state、audit、frontend caller 或发布过渡连接。

BE-001W-03 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 第一轮物理抽离，将 3 个参数网格 helper 迁入 `src/runtime/backtest/parameter_grid.rs`；父级 `experiment_sweep` 只保留 handler 编排和 `pub(super)` 调用，下一步只能进入 BE-001W-04 单叶 closeout，不能继续细拆 axis normalization、variant expansion、schema、route、state/persistence、frontend caller 或发布过渡连接。

BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，确认 3 个参数网格 helper 等价并设置 `stop_split: true`；下一步回到 `runtime.backtest.experiment_sweep` 父叶残余判断，默认进入 BE-001X-01，不能继续细拆 parameter_grid 或宣称 experiment_sweep 父叶最终完成、schema/constant/route/shared owner 或发布过渡已完成。

BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，确认 `parameter_grid` 已关闭但父叶仍 `stop_split: false`；下一步默认进入 BE-001Y-01 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线，不能直接移动 start handler、record lifecycle、route、schema、state/persistence、frontend caller 或发布过渡连接。

BE-001Y-01 已建立 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线，冻结 `start_backtest_experiment` 创建编排、guard、QS compile、variant request、`execute_backtest_request` 复用桥和 preview persistence；当前 `no code movement`，下一步只能进入 BE-001Y-02 抽离方案，不能直接移动 start handler、混入 record lifecycle、迁移 route/schema/state/persistence/frontend caller 或启动发布过渡连接。

BE-001Y-02 已建立 `runtime.backtest.experiment_sweep.start_orchestration` 抽离方案，限定下一批只允许迁移 `start_backtest_experiment` 到计划子文件；当前 `no code movement`，下一步只能进入 BE-001Y-03 实际抽离记录，不能混入 record lifecycle、route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001Y-03 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 实际抽离，将 `start_backtest_experiment` 迁入 `src/runtime/backtest/start_orchestration.rs`，并由父级 `experiment_sweep` 通过 `pub(crate) use` 保持兼容出口；下一步只能进入 BE-001Y-04 单叶 closeout，不能混入 record lifecycle、route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001Y-04 已完成 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout，确认 `start_backtest_experiment` 等价并设置 `stop_split: true`；该回流已由 BE-001Z-01 父叶残余判断承接，不能从 start_orchestration 继续细拆或直接移动 record lifecycle、route/schema/state/persistence/response mapping/audit/frontend caller。

BE-001Z-01 已完成 `runtime.backtest.experiment_sweep` 第二轮父叶残余判断，确认 `parameter_grid` 与 `start_orchestration` 均已关闭并设置 `stop_split: true`，父叶仍 `stop_split: false`；下一步只能进入 BE-001AA-01 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，不能直接移动 list/detail/save/discard、route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001AA-01 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线，冻结 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 的 record lifecycle 边界；当前 `no code movement`，下一步只能进入 BE-001AA-02 抽离方案，不能直接移动代码、混入 parameter_grid/start_orchestration、迁移 route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001AA-02 已建立 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案，限定下一批只迁移四个 lifecycle handler 到计划目标文件 `src/runtime/backtest/record_lifecycle.rs`；当前仍是 `no code movement`，下一步只能进入 BE-001AA-03 实际抽离记录，不能混入 parameter_grid、start_orchestration、route/schema/state/persistence/response mapping/audit/frontend caller 或发布过渡连接。

BE-001AA-03 已完成 `runtime.backtest.experiment_sweep.record_lifecycle` 实际抽离，四个 lifecycle handler 已迁入 `src/runtime/backtest/record_lifecycle.rs`，父级只保留私有模块声明和受控 re-export；下一步只能进入 BE-001AA-04 单叶 closeout，不能继续细拆 save/discard、迁移 route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001AA-04 已完成 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout，确认四个 lifecycle handler 等价并设置 `stop_split: true`；下一步只能回到父级进入 BE-001AB-01 第三轮父叶残余判断，不能从 record_lifecycle 继续细拆、迁移 route/schema/state/persistence/response mapping/audit/frontend caller 或启动发布过渡连接。

BE-001AB-01 已完成 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断；`parameter_grid`、`start_orchestration`、`record_lifecycle` 三个子叶均已 closeout 并设置 `stop_split: true`，父叶自身也设置 `stop_split: true`。下一步只能进入 BE-001AC-01 `runtime.backtest` 父叶残余判断，不能宣称 route/schema/state/persistence/response mapping/frontend caller 已迁移或发布过渡已启动。

BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断；`execution_start`、`record_store`、`replay`、`experiment_sweep` 均已完成当前递归范围内 closeout，父叶自身设置 `stop_split: true`。该上层已由 BE-001AD-01 承接完成，BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案，不能删除 drained parent include、迁移 compare/artifact schema/persistence/response mapping/frontend caller 或启动发布过渡。

BE-001AD-01 已完成 `backend.runtime.routes` 父叶残余判断；`run`、`event_stream`、`backtest` 相关递归链路均已完成当前范围内 closeout，但 route aggregate 仍保留 evidence / report / experiment / ops 等路线，因此父叶保持 `stop_split: false`。BE-001AE-04 已完成 `backend.runtime.routes.mutation` route facade 单叶 closeout 并设置 `stop_split: true`；BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，下一步只能进入 BE-001AF-02 抽离方案，不能移动 handler、AppState、锁顺序、schema、frontend caller 或启动发布过渡。

BE-001AE-03 已完成 `backend.runtime.routes.mutation` route facade 最小物理抽离；`src/backend/runtime/routes/mutation.rs` 承接 mutation / AI proposal / approval route group，父级 `src/backend/runtime/routes.rs` 只保留 `pub mod mutation` 与 `mutation::register_routes(router)` 委托。

BE-001AE-04 已完成 `backend.runtime.routes.mutation` 单叶 closeout；route facade 等价并设置 `stop_split: true`。BE-001AF-01 已建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，冻结 create/list/detail/activate/rollback、safe window、activation/rollback event、parameter version、run record append、persistence bridge、状态 owner、schema owner 和 `api_mutation` 证据；下一步只能进入 BE-001AF-02 抽离方案，不能移动 parameter mutation handler、AI proposal、approval review、shared persistence/governance helper、AppState、锁顺序、schema、frontend caller、report/evidence/experiment/ops 或发布过渡连接。

BE-001AF-02 已建立 `runtime.mutation.parameter_mutation` 抽离方案；当前仍为 `no code movement`，固定目标子模块、`src/runtime/mod.rs` 父级 re-export、五个 public handler 迁移清单、本叶私有 helper 迁移清单，以及 AI/shared helper 保留边界。下一步只能进入 BE-001AF-03 实际抽离，不能移动 AI proposal、approval review、AppState、schema、frontend caller、锁顺序或发布过渡连接。

这波模块化改造按十万行级重大工程看待。v4.16.0 不允许形成单个巨型代码变更，而是先把工程拆成父模块级、兼容桥级或验证链级的可回退批次。

抽离主线拆为后端抽离和前端抽离。E2E 整理延后，不作为 v4.16 抽离控制面的前置阻断；后续测试程序会大规模废弃，必须进入测试资产汰换登记，不能静默删除。

整理和重构不在 v4.16.0 执行。遇到目标顺序、接口粒度、状态归属、兼容桥方向、测试废弃边界或旧实现退役判断时，必须先与开发者讨论方案。

---

## 4. 暂存旧文件规则

旧文件保留并继续引用:

- `markdown/General_Policy.md`
- `markdown/01-principles/principles-super-standardization.md`
- `markdown/10-overview/overview-full-feature-tree.md`

后续迁移只允许做导流、索引、章节对齐和规则引用。不得为了显得整洁而一次性删除旧规则。

---

## 5. 模块化重构通道

当原模块依赖复杂、难以原地解耦时，允许按重型档启动模块化重构通道:

1. 在提案中声明目标模块树节点和父模块。
2. 建立新的模块化版本或适配层。
3. 原实现保留，直到新版本通过适配、测试、门禁和人工审计。
4. 新旧实现并存期间必须写清入口选择、兼容桥、状态迁移和回退策略。
5. 删除旧实现只能作为后续重型变更，不能和抽离实现混在一次变更里。

---

## 6. 模块化三步走

| 阶段 | 里程碑归属 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| 抽离 | v4.16.0 | 已完成 system 试水、经验回填、递归流程、10 个 system 叶子 closeout/静态 closeout、`root.system` 顶层阶段性 closeout；backend 已完成 BE-001B 九叶模块壳抽离、BE-001C 九叶逐叶 closeout、BE-001D strategy_config L3 模块壳抽离、BE-001E 其余八叶薄壳抽离和 BE-001E-01 至 BE-001E-08 逐叶完成记录，BE-001F 已完成 `backend.runtime.routes` route aggregate 抽离，BE-001G 已完成 `backend.runtime.routes.run` run route group 抽离和单叶 closeout，BE-001H-03 已完成 `runtime.run.v4_handoff` 抽离与单叶 closeout，BE-001I-03 已完成 `runtime.run.session_start` 抽离与单叶 closeout，BE-001J-05 已完成 `runtime.run.record_store` 抽离与单叶 closeout，BE-001K-04 已完成 `runtime.run.replay_status` 抽离与单叶 closeout，BE-001L-04 已完成 `runtime.event_stream` 抽离与单叶 closeout，BE-001M-04 已完成 `runtime.backtest` route facade 抽离与单叶 closeout，BE-001N-04 已完成 `runtime.backtest.execution_start` 第一轮物理抽离与单叶 closeout，BE-001O-04 已完成 `runtime.backtest.execution_start.v4_projection` 单叶 closeout，BE-001P-04 已完成 `v4_request_resolution` 单叶 closeout，BE-001Q-04 已完成 `v4_runtime_execution` 单叶 closeout，BE-001R-04 已完成 `legacy_dispatch` 单叶 closeout，BE-001S-01 已完成父叶残余判断，BE-001T-04 已完成 `runtime.backtest.record_store` 单叶 closeout，BE-001U-04 已完成 `runtime.backtest.replay` 单叶 closeout，BE-001V-04 已完成 `runtime.backtest.experiment_sweep` 单叶 closeout，BE-001W-04 已完成 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，BE-001X-01 已完成 `runtime.backtest.experiment_sweep` 父叶残余判断，BE-001Y-04 已完成 `start_orchestration` 单叶 closeout，BE-001Z-01 已完成第二轮父叶残余判断，BE-001AA-04 已完成 `record_lifecycle` 单叶 closeout，BE-001AB-01 已完成第三轮父叶残余判断并设置 `runtime.backtest.experiment_sweep` 父叶 `stop_split: true`，BE-001AC-01 已完成 `runtime.backtest` 父叶残余判断并设置父叶 `stop_split: true` | 拆成后端抽离和前端抽离，建立模块化候选版本、兼容桥、等价验证和决策暂停机制；`system.entry.backend_process` 已完成启动边界抽离，后续抽离按 public/内部实现分类、owner 复核、单叶 closeout、暂停解除协议和递归细分判断推进 |
| 整理 | 后续方案 | 未启动 | 收敛目录、命名、public 方法、状态归属和复用层 |
| 重构 | 后续方案 | 未启动 | 切换主入口、替换旧路径、退役旧实现和调整调用拓扑 |

E2E 整理不属于 v4.16.0 的抽离范围。测试资产汰换会在后续方案中处理，当前只登记废弃候选、替代证据和风险窗口。
Latest recursive supplement: BE-001OX-01 closed `backend.ops_governance`; hotswap, sandbox, alerts, snapshots, runbook, and chaos are closed. The backend residual queue now contains `backend.app_state_wiring` and `backend.test_support`, with BE-001OY-01 selecting `backend.app_state_wiring`.
Latest recursive supplement: BE-001OY-01 selected `backend.app_state_wiring` as the next backend top-level residual. AppState owner, lock order, health schema, frontend caller, and release transition remain frozen.
Latest recursive supplement: BE-001OZ-01 closed `backend.app_state_wiring` as a single wiring leaf with `stop_split: true`. The remaining backend top-level residual is `backend.test_support`.
Latest recursive supplement: BE-001PA-01 selected `backend.test_support` as the final backend top-level residual. Test asset retirement, legacy test deletion, E2E cleanup, and production route changes remain frozen.
Latest recursive supplement: BE-001PB-01 closed `backend.test_support` as a single test-support facade with `stop_split: true`. All backend top-level residuals are closed; next step is `backend` parent closeout.
Latest recursive supplement: BE-001PC-01 closed `backend` for the current Rust backend extraction scope. The next root-level Rust residual candidate is `root.contracts`, while `root.executor` remains queued.
Latest recursive supplement: BE-001PD-01 selected `root.contracts` as the next Rust-facing top-level residual. Contract schemas and protocol semantics remain frozen until BE-001PE-01 baseline_plan.
Latest recursive supplement: BE-001PE-01 froze the `root.contracts` baseline and registered api surface, QRPC core, Core IR, compiler bridge, runtime support, QuantScript, and plugin metadata as the contracts child queue.
Latest recursive supplement: BE-001PF-01 selected `contracts.api_surface` as the first contracts child; schema semantics and all Rust behavior remain frozen until the single leaf closeout.
Latest recursive supplement: BE-001PG-01 kept `contracts.api_surface` equivalent and set `stop_split: false` because OpenAPI HTTP and AsyncAPI runtime events are separate schema owners.
Latest recursive supplement: BE-001PH-01 selected `contracts.api_surface.openapi_http`; `contracts/asyncapi/runtime-events.yaml` remains queued as its own schema owner.
Latest recursive supplement: BE-001PI-01 closed `contracts.api_surface.openapi_http` with `stop_split: true`; `contracts/openapi/root.yaml` remains the single OpenAPI HTTP schema owner.
Latest recursive supplement: BE-001PJ-01 selected `contracts.api_surface.asyncapi_runtime_events`; backend SSE handler behavior and runtime event producers remain frozen.
Latest recursive supplement: BE-001PK-01 closed `contracts.api_surface.asyncapi_runtime_events` with `stop_split: true`; all api_surface children are now ready for parent closeout.
Latest recursive supplement: BE-001PL-01 closed `root.contracts.api_surface`; the next contracts residual candidate is `contracts.qrpc_core`.
Latest recursive supplement: BE-001PM-01 selected `contracts.qrpc_core`; protocol structs, plugin contracts, Strategy IR, typed errors, event proto, and Rust behavior remain frozen until the qrpc_core baseline.
Latest recursive supplement: BE-001PN-01 froze the `root.contracts.qrpc_core` baseline and queued nine qrpc_core child owners, with `error_contract` selected next.
Latest recursive supplement: BE-001PO-01 selected `contracts.qrpc_core.error_contract`; only `qrpc_core/src/error.rs` is in scope for the next single leaf closeout.
Latest recursive supplement: BE-001PP-01 closed `contracts.qrpc_core.error_contract` with `stop_split: true`; next qrpc_core residual candidate is `event_envelope_proto`.
Latest recursive supplement: BE-001PQ-01 selected `contracts.qrpc_core.event_envelope_proto`; runtime producers, AsyncAPI schema, backend SSE handlers, and lib.rs DTOs remain frozen.
Latest recursive supplement: BE-001PR-01 closed `contracts.qrpc_core.event_envelope_proto` with `stop_split: true`; next qrpc_core residual candidate is `plugin_contract`.
Latest recursive supplement: BE-001PS-01 selected `contracts.qrpc_core.plugin_contract`; physical `plugins/*` registry placeholders remain queued under `contracts.plugin_metadata`.
Latest recursive supplement: BE-001PT-01 froze the `root.contracts.qrpc_core.plugin_contract` baseline and queued taxonomy, capability, execution/security/dependency, manifest validation, and registry child owners.
Latest recursive supplement: BE-001PU-01 selected `contracts.qrpc_core.plugin_contract.taxonomy_extension`; enum variants, serde rename rules, mapping strings, and all Rust behavior remain frozen until its baseline.
Latest recursive supplement: BE-001PV-01 froze the `taxonomy_extension` extraction baseline; next step may move only PluginKind, ExtensionPoint, and their mapping impls under the plugin contract parent.
Latest recursive supplement: BE-001PV-02 extracted `contracts.qrpc_core.plugin_contract.taxonomy_extension` into a private child module while preserving plugin parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001PV-03 closed `contracts.qrpc_core.plugin_contract.taxonomy_extension` with `stop_split: true`; next plugin contract residual candidate is `capability_contract`.
Latest recursive supplement: BE-001PW-01 selected `contracts.qrpc_core.plugin_contract.capability_contract`; capability strings, parser behavior, version value, and all Rust behavior remain frozen until its baseline.
Latest recursive supplement: BE-001PX-01 froze the `capability_contract` extraction baseline; next step may move only capability version, declaration DTO, enum, parser, and string impls under the plugin contract parent.
Latest recursive supplement: BE-001PX-02 extracted `contracts.qrpc_core.plugin_contract.capability_contract` into a private child module while preserving plugin parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001PX-03 closed `contracts.qrpc_core.plugin_contract.capability_contract` with `stop_split: true`; next plugin contract residual candidate is `execution_security_dependency`.
Latest recursive supplement: BE-001PY-01 selected `contracts.qrpc_core.plugin_contract.execution_security_dependency`; execution/security/dependency DTO fields and all Rust behavior remain frozen until its baseline.
Latest recursive supplement: BE-001PZ-01 froze the `execution_security_dependency` extraction baseline; next step may move only execution, compatibility, security, and dependency DTOs under the plugin contract parent.
Latest recursive supplement: BE-001PZ-02 extracted `contracts.qrpc_core.plugin_contract.execution_security_dependency` into a private child module while preserving plugin parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001PZ-03 closed `contracts.qrpc_core.plugin_contract.execution_security_dependency` with `stop_split: true`; next plugin contract residual candidate is `manifest_validation`.
Latest recursive supplement: BE-001QA-01 selected `contracts.qrpc_core.plugin_contract.manifest_validation`; manifest fields, serde attributes, validation rules, and all Rust behavior remain frozen until its baseline.
Latest recursive supplement: BE-001QB-01 froze the `manifest_validation` extraction baseline; next step may move only manifest schema DTOs and `PluginManifest::validate` under the plugin contract parent.
Latest recursive supplement: BE-001QB-02 extracted `contracts.qrpc_core.plugin_contract.manifest_validation` into a private child module while preserving plugin parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QB-03 closed `contracts.qrpc_core.plugin_contract.manifest_validation` with `stop_split: true`; next plugin contract residual candidate is `registry`.
Latest recursive supplement: BE-001QC-01 selected `contracts.qrpc_core.plugin_contract.registry`; registry method signatures, duplicate-id behavior, lookup/removal semantics, and extension-point filtering remain frozen until its baseline.
Latest recursive supplement: BE-001QD-01 froze the `registry` extraction baseline; next step may move only `PluginRegistry` and its impl under the plugin contract parent.
Latest recursive supplement: BE-001QD-02 extracted `contracts.qrpc_core.plugin_contract.registry` into a private child module while preserving plugin parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QD-03 closed `contracts.qrpc_core.plugin_contract.registry` with `stop_split: true`; all plugin_contract children are now ready for parent closeout.
Latest recursive supplement: BE-001QE-01 closed `contracts.qrpc_core.plugin_contract`; the next qrpc_core residual candidate is `strategy_ir`.
Latest recursive supplement: BE-001QF-01 selected `contracts.qrpc_core.strategy_ir`; Strategy IR DTOs, validation behavior, indicator kind surfaces, gap annotations, and all Rust behavior remain frozen until its baseline.
Latest recursive supplement: BE-001QG-01 froze the `root.contracts.qrpc_core.strategy_ir` baseline and queued version/unknown/error, metadata/source, signal/indicator, logic/position, risk, data, execution, gap/unknown, and root validation child owners.
Latest recursive supplement: BE-001QH-01 selected `contracts.qrpc_core.strategy_ir.version_unknown_error`; version string, unknown marker semantics, and validation error diagnostics remain frozen until its baseline.
Latest recursive supplement: BE-001QI-01 froze the `version_unknown_error` extraction baseline; next step may move only the Strategy IR version constant, unknown wrapper, `is_unknown`, and validation error diagnostic type under the Strategy IR parent.
Latest recursive supplement: BE-001QI-02 extracted `contracts.qrpc_core.strategy_ir.version_unknown_error` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QI-03 closed `contracts.qrpc_core.strategy_ir.version_unknown_error` with `stop_split: true`; next Strategy IR residual candidate is `metadata_source`.
Latest recursive supplement: BE-001QJ-01 selected `contracts.qrpc_core.strategy_ir.metadata_source`; metadata/source fields, serde shape, and source type enum names remain frozen until its baseline.
Latest recursive supplement: BE-001QK-01 froze the `metadata_source` extraction baseline; next step may move only `StrategyMetadata`, `StrategySource`, and `StrategySourceType` under the Strategy IR parent.
Latest recursive supplement: BE-001QK-02 extracted `contracts.qrpc_core.strategy_ir.metadata_source` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QK-03 closed `contracts.qrpc_core.strategy_ir.metadata_source` with `stop_split: true`; next Strategy IR residual candidate is `signal_indicator`.
Latest recursive supplement: BE-001QL-01 selected `contracts.qrpc_core.strategy_ir.signal_indicator`; signal/indicator fields, serde shape, indicator enum variants, and declared/supported indicator registries remain frozen until its baseline.
Latest recursive supplement: BE-001QM-01 froze the `signal_indicator` extraction baseline; next step may move only signal/indicator DTOs, `IndicatorKind`, indicator registry constants, and public registry functions under the Strategy IR parent.
Latest recursive supplement: BE-001QM-02 extracted `contracts.qrpc_core.strategy_ir.signal_indicator` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QM-03 closed `contracts.qrpc_core.strategy_ir.signal_indicator` with `stop_split: true`; next Strategy IR residual candidate is `logic_position`.
Latest recursive supplement: BE-001QN-01 selected `contracts.qrpc_core.strategy_ir.logic_position`; logic/action/position sizing/rebalance fields and serde shape remain frozen until its baseline.
Latest recursive supplement: BE-001QO-01 froze the `logic_position` extraction baseline; next step may move only logic/action/position sizing/rebalance DTOs under the Strategy IR parent.
Latest recursive supplement: BE-001QO-02 extracted `contracts.qrpc_core.strategy_ir.logic_position` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QO-03 closed `contracts.qrpc_core.strategy_ir.logic_position` with `stop_split: true`; next Strategy IR residual candidate is `risk_contract`.
Latest recursive supplement: BE-001QP-01 selected `contracts.qrpc_core.strategy_ir.risk_contract`; risk rule/profile fields, default behavior, and unknownable risk values remain frozen until its baseline.
Latest recursive supplement: BE-001QQ-01 froze the `risk_contract` extraction baseline; next step may move only `StrategyRiskRules` and `StrategyRiskProfileRef` under the Strategy IR parent.
Latest recursive supplement: BE-001QQ-02 extracted `contracts.qrpc_core.strategy_ir.risk_contract` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QQ-03 closed `contracts.qrpc_core.strategy_ir.risk_contract` with `stop_split: true`; next Strategy IR residual candidate is `data_requirement`.
Latest recursive supplement: BE-001QR-01 selected `contracts.qrpc_core.strategy_ir.data_requirement`; data requirement fields, data type enum variants, and unknownable data values remain frozen until its baseline.
Latest recursive supplement: BE-001QS-01 froze the `data_requirement` extraction baseline; next step may move only `DataRequirement` and `DataRequirementType` under the Strategy IR parent.
Latest recursive supplement: BE-001QS-02 extracted `contracts.qrpc_core.strategy_ir.data_requirement` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QS-03 closed `contracts.qrpc_core.strategy_ir.data_requirement` with `stop_split: true`; next Strategy IR residual candidate is `execution_contract`.
Latest recursive supplement: BE-001QT-01 selected `contracts.qrpc_core.strategy_ir.execution_contract`; execution fields, profile fields, default behavior, and unknownable execution values remain frozen until its baseline.
Latest recursive supplement: BE-001QU-01 froze the `execution_contract` extraction baseline; next step may move only `StrategyExecution` and `StrategyExecutionProfileRef` under the Strategy IR parent.
Latest recursive supplement: BE-001QU-02 extracted `contracts.qrpc_core.strategy_ir.execution_contract` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QU-03 closed `contracts.qrpc_core.strategy_ir.execution_contract` with `stop_split: true`; next Strategy IR residual candidate is `gap_unknown_annotation`.
Latest recursive supplement: BE-001QV-01 selected `contracts.qrpc_core.strategy_ir.gap_unknown_annotation`; gap annotation fields, gap enums, and strategy unknown marker fields remain frozen until its baseline.
Latest recursive supplement: BE-001QW-01 froze the `gap_unknown_annotation` extraction baseline; next step may move only `GapAnnotation`, `GapType`, `GapSeverity`, and `StrategyUnknown` under the Strategy IR parent.
Latest recursive supplement: BE-001QW-02 extracted `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QW-03 closed `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` with `stop_split: true`; next Strategy IR residual candidate is `root_validation`.
Latest recursive supplement: BE-001QX-01 selected `contracts.qrpc_core.strategy_ir.root_validation`; root Strategy IR DTO, public validation methods, private validation helpers, and local validation tests remain frozen until its baseline.
Latest recursive supplement: BE-001QY-01 froze the `root_validation` extraction baseline; next step may move only `StrategyIr`, public validation methods, private validation helpers, and local Strategy IR tests under the Strategy IR parent.
Latest recursive supplement: BE-001QY-02 extracted `contracts.qrpc_core.strategy_ir.root_validation` into a private child module while preserving Strategy IR parent re-exports and qrpc-core tests.
Latest recursive supplement: BE-001QY-03 kept `contracts.qrpc_core.strategy_ir.root_validation` open with `continue_split: true`; next child candidate is `identity_required_validation`.
Latest recursive supplement: BE-001QZ-01 selected `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`; version, metadata required fields, top-level required collections, and unique id checks remain frozen until its baseline.
Latest recursive supplement: BE-001RA-01 froze the `identity_required_validation` extraction baseline; next step may move only version, required-field, required-collection, duplicate-id, and `validate_unique_ids` logic under the root validation parent.
Latest recursive supplement: BE-001RA-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` into a private child module while preserving validation ordering and qrpc-core tests.
Latest recursive supplement: BE-001RA-03 closed `contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` with `stop_split: true`; next root_validation residual candidate is `signal_logic_validation`.
Latest recursive supplement: BE-001RB-01 selected `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`; signal detail validation, indicator support validation, logic rule validation, and logic position unknown-marker checks remain frozen until its baseline.
Latest governance supplement: GOV-SAME-PARENT-PARALLEL allows guarded same-parent child parallel waves in the recursive speed protocol; parent-child communication, no sibling horizontal link, declared write sets, parent facade lock, and leaf split gate remain hard rules.
Latest recursive supplement: BE-001RC-01 froze the `signal_logic_validation` extraction baseline with Rust-local facade, visibility, and Cargo gate fields; next step may move only signal/detail, indicator support, logic rule, and logic unknown-marker validation under the root validation parent.
Latest recursive supplement: BE-001RC-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` into a private child module while preserving validation ordering, parent-owned helpers, and qrpc-core checks.
Latest recursive supplement: BE-001RC-03 closed `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` with `stop_split: true`; next root_validation residual candidate is `risk_validation`.
Latest recursive supplement: BE-001RD-01 selected `contracts.qrpc_core.strategy_ir.root_validation.risk_validation`; risk unknownable checks and risk profile id/numeric validation remain frozen until its baseline.
Latest recursive supplement: BE-001RE-01 froze the `risk_validation` extraction baseline with Rust-local facade, visibility, parent-helper, and Cargo gate fields; next step may move only risk unknownable checks and risk profile id/numeric validation under the root validation parent.
Latest recursive supplement: BE-001RE-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` into a private child module while preserving validation ordering, parent-owned helpers, and qrpc-core checks.
Latest recursive supplement: BE-001RE-03 closed `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` with `stop_split: true`; next root_validation residual candidate is `data_execution_validation`.
Latest recursive supplement: BE-001RF-01 selected `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`; data requirement checks plus execution and execution profile validation remain frozen until its baseline.
Latest recursive supplement: BE-001RG-01 froze the `data_execution_validation` extraction baseline with Rust-local facade, visibility, parent-helper, and Cargo gate fields; next step may move only data requirement checks plus execution and execution profile validation under the root validation parent.
Latest recursive supplement: BE-001RG-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` into a private child module while preserving validation ordering, parent-owned helpers, and qrpc-core checks.
Latest recursive supplement: BE-001RG-03 closed `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation` with `stop_split: true`; next root_validation residual candidate is `unknown_marker_validation`.
Latest recursive supplement: BE-001RH-01 selected `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation`; unknownable helper ownership and unknowns path/reason validation remain frozen until its baseline.
Latest recursive supplement: BE-001RI-01 froze the `unknown_marker_validation` extraction baseline with parent wrapper bridge preservation; next step may move only unknownable helper implementation and `unknowns[*]` path/reason validation under the root validation parent.
Latest recursive supplement: BE-001RI-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` into a private child module while preserving validation ordering, parent wrapper bridge mediation, and qrpc-core checks.
Latest recursive supplement: BE-001RI-03 closed `contracts.qrpc_core.strategy_ir.root_validation.unknown_marker_validation` with `stop_split: true`; next step returns to root_validation parent residual judgment.
Latest recursive supplement: BE-001RJ-01 selected `contracts.qrpc_core.strategy_ir.root_validation.test_fixture`; local sample JSON and root validation unit tests remain frozen until its baseline.
Latest recursive supplement: BE-001RK-01 froze the `test_fixture` extraction baseline; next step may move only root validation local tests and `SAMPLE_JSON` into a cfg-test child file.
Latest recursive supplement: BE-001RK-02 extracted `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` into a cfg-test child file while preserving qrpc-core test behavior.
Latest recursive supplement: BE-001RK-03 closed `contracts.qrpc_core.strategy_ir.root_validation.test_fixture` with `stop_split: true`; next step returns to root_validation parent residual judgment.
Latest recursive supplement: BE-001RL-01 closed `contracts.qrpc_core.strategy_ir.root_validation` as a compact parent facade; next step returns to strategy_ir parent residual judgment.
Latest recursive supplement: BE-001RM-01 closed `contracts.qrpc_core.strategy_ir` as a compact parent facade; next step returns to qrpc_core parent residual judgment selecting protocol_primitives.
Latest recursive supplement: BE-001RN-01 selected `contracts.qrpc_core.protocol_primitives`; qrpc-core primitive constants, enums, serde/display/default behavior remain frozen until its baseline.
Latest recursive supplement: BE-001RO-01 froze the `protocol_primitives` extraction baseline; next step may move only primitive constants, enums, Symbol serde/parse/display/default behavior into a qrpc-core child module.
Latest recursive supplement: BE-001RO-02 extracted `contracts.qrpc_core.protocol_primitives` into a private qrpc-core child module while preserving crate-root public exports and qrpc-core checks.
Latest recursive supplement: BE-001RO-03 closed `contracts.qrpc_core.protocol_primitives` with `stop_split: true`; next qrpc_core residual candidate is `runtime_protocol_config`.
Latest recursive supplement: BE-001RP-01 selected `contracts.qrpc_core.runtime_protocol_config`; runtime config DTOs, defaults, and compiled protocol container remain frozen until its baseline.
Latest recursive supplement: BE-001RQ-01 froze the `runtime_protocol_config` extraction baseline; next step may move only runtime config DTOs, universe config DTOs, defaults, and compiled protocol container into a qrpc-core child module.
Latest recursive supplement: BE-001RQ-02 extracted `contracts.qrpc_core.runtime_protocol_config` into a private qrpc-core child module while preserving crate-root public exports and qrpc-core checks.
Latest recursive supplement: BE-001RQ-03 closed `contracts.qrpc_core.runtime_protocol_config` with `stop_split: true`; next qrpc_core residual candidate is `artifact_specs`.
Latest recursive supplement: BE-001RR-01 selected `contracts.qrpc_core.artifact_specs`; canonical digest, run/backtest specs, dataset/execution assumptions, and artifact bundle contracts remain frozen until its baseline.
Latest recursive supplement: BE-001RS-01 froze the `artifact_specs` extraction baseline; next step may move only canonical digest, run/backtest specs, dataset/execution projections, market data snapshot specs, and artifact bundle DTOs into a qrpc-core child module.
Latest recursive supplement: BE-001RS-02 extracted `contracts.qrpc_core.artifact_specs` into a private qrpc-core child module while preserving crate-root public exports and qrpc-core checks.
Latest recursive supplement: BE-001RS-03 kept `contracts.qrpc_core.artifact_specs` open with `continue_split: true`; next child candidate is `canonical_digest`.
Latest recursive supplement: BE-001RT-01 selected `contracts.qrpc_core.artifact_specs.canonical_digest`; digest algorithm, digest DTO, and canonical JSON SHA-256 helper remain frozen until its baseline.
Latest recursive supplement: BE-001RU-01 froze the `canonical_digest` extraction baseline; next step may move only digest algorithm, digest DTO, and canonical JSON SHA-256 helper into an artifact specs child module.
Latest recursive supplement: BE-001RU-02 extracted `contracts.qrpc_core.artifact_specs.canonical_digest` into a private artifact specs child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001RU-03 closed `contracts.qrpc_core.artifact_specs.canonical_digest` with `stop_split: true`; next artifact specs residual candidate is `run_backtest_specs`.
Latest recursive supplement: BE-001RV-01 selected `contracts.qrpc_core.artifact_specs.run_backtest_specs`; run/backtest modes, dataset/execution projections, market data snapshot specs, `RunSpec`, and `BacktestSpec` remain frozen until its baseline.
Latest recursive supplement: BE-001RW-01 froze the `run_backtest_specs` extraction baseline; next step may move only run/backtest modes, dataset/execution projections, market data snapshot specs, `RunSpec`, and `BacktestSpec` into an artifact specs child module.
Latest recursive supplement: BE-001RW-02 extracted `contracts.qrpc_core.artifact_specs.run_backtest_specs` into a private artifact specs child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001RW-03 closed `contracts.qrpc_core.artifact_specs.run_backtest_specs` with `stop_split: true`; next artifact specs residual candidate is `artifact_bundle_contract`.
Latest recursive supplement: BE-001RX-01 selected `contracts.qrpc_core.artifact_specs.artifact_bundle_contract`; strategy/core-IR/compile artifact DTOs and bundle contracts remain frozen until its baseline.
Latest recursive supplement: BE-001RY-01 froze the `artifact_bundle_contract` extraction baseline; next step may move only strategy/core-IR/compile artifact DTOs and bundle contracts into an artifact specs child module.
Latest recursive supplement: BE-001RY-02 extracted `contracts.qrpc_core.artifact_specs.artifact_bundle_contract` into a private artifact specs child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001RY-03 closed `contracts.qrpc_core.artifact_specs.artifact_bundle_contract` with `stop_split: true`; next step returns to artifact specs parent residual judgment.
Latest recursive supplement: BE-001RZ-01 closed `contracts.qrpc_core.artifact_specs` as a compact parent facade; next qrpc_core residual candidate is `runtime_io_contract`.
Latest recursive supplement: BE-001SA-01 selected `contracts.qrpc_core.runtime_io_contract`; runtime input/output DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SB-01 froze the `runtime_io_contract` extraction baseline; next step may move only runtime input/output DTOs from `RawKline` through `BacktestOutput` into a qrpc-core child module.
Latest recursive supplement: BE-001SB-02 extracted `contracts.qrpc_core.runtime_io_contract` into a private qrpc-core child module while preserving crate-root public exports.
Latest recursive supplement: BE-001SB-03 kept `contracts.qrpc_core.runtime_io_contract` open with `continue_split: true`; next child candidate is `market_data_io`.
Latest recursive supplement: BE-001SC-01 selected `contracts.qrpc_core.runtime_io_contract.market_data_io`; raw and normalized market data DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SD-01 froze the `market_data_io` extraction baseline; next step may move only raw and normalized market data DTOs into a runtime IO child module.
Latest recursive supplement: BE-001SD-02 extracted `contracts.qrpc_core.runtime_io_contract.market_data_io` into a private runtime IO child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001SD-03 closed `contracts.qrpc_core.runtime_io_contract.market_data_io` with `stop_split: true`; next runtime IO residual candidate is `decision_flow`.
Latest recursive supplement: BE-001SE-01 selected `contracts.qrpc_core.runtime_io_contract.decision_flow`; intent/action/target/agent/risk decision DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SF-01 froze the `decision_flow` extraction baseline; next step may move only intent/action/target/agent/risk decision DTOs into a runtime IO child module.
Latest recursive supplement: BE-001SF-02 extracted `contracts.qrpc_core.runtime_io_contract.decision_flow` into a private runtime IO child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001SF-03 closed `contracts.qrpc_core.runtime_io_contract.decision_flow` with `stop_split: true`; next runtime IO residual candidate is `execution_io`.
Latest recursive supplement: BE-001SG-01 selected `contracts.qrpc_core.runtime_io_contract.execution_io`; simulated order, execution plan, fill report, open order, and fill result DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SH-01 froze the `execution_io` extraction baseline; next step may move only simulated order, execution plan, fill report, open order, and fill result DTOs into a runtime IO child module.
Latest recursive supplement: BE-001SH-02 extracted `contracts.qrpc_core.runtime_io_contract.execution_io` into a private runtime IO child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001SH-03 closed `contracts.qrpc_core.runtime_io_contract.execution_io` with `stop_split: true`; next runtime IO residual candidate is `portfolio_state`.
Latest recursive supplement: BE-001SI-01 selected `contracts.qrpc_core.runtime_io_contract.portfolio_state`; position, exposure, portfolio state DTOs, and portfolio helper methods remain frozen until its baseline.
Latest recursive supplement: BE-001SJ-01 froze the `portfolio_state` extraction baseline; next step may move only position, exchange exposure, portfolio state DTOs, and portfolio helper methods into a runtime IO child module.
Latest recursive supplement: BE-001SJ-02 extracted `contracts.qrpc_core.runtime_io_contract.portfolio_state` into a private runtime IO child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001SJ-03 closed `contracts.qrpc_core.runtime_io_contract.portfolio_state` with `stop_split: true`; next runtime IO residual candidate is `runtime_output`.
Latest recursive supplement: BE-001SK-01 selected `contracts.qrpc_core.runtime_io_contract.runtime_output`; runtime event, cycle output, and session output DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SL-01 froze the `runtime_output` extraction baseline; next step may move only runtime event, cycle output, and session output DTOs into a runtime IO child module.
Latest recursive supplement: BE-001SL-02 extracted `contracts.qrpc_core.runtime_io_contract.runtime_output` into a private runtime IO child module while preserving parent and crate-root public exports.
Latest recursive supplement: BE-001SL-03 closed `contracts.qrpc_core.runtime_io_contract.runtime_output` with `stop_split: true`; next runtime IO residual candidate is `backtest_output`.
Latest recursive supplement: BE-001SM-01 selected `contracts.qrpc_core.runtime_io_contract.backtest_output`; backtest equity, metric groups, summary, period return, and final output DTOs remain frozen until its baseline.
Latest recursive supplement: BE-001SN-01 froze the `backtest_output` extraction baseline; next step may move only final backtest output DTOs and nested metric DTOs into a runtime IO child module.
Latest recursive supplement: BE-001SN-02 extracted `contracts.qrpc_core.runtime_io_contract.backtest_output` into a private runtime IO child module; the runtime IO parent is now a pure facade.
Latest recursive supplement: BE-001SN-03 closed `contracts.qrpc_core.runtime_io_contract.backtest_output` with `stop_split: true`; next step closes the runtime IO parent.
Latest recursive supplement: BE-001SO-01 closed `contracts.qrpc_core.runtime_io_contract` as a compact parent facade; next qrpc_core residual candidate is `rfc_execution_contracts`.
