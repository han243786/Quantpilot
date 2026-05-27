# v4.16.0 system 抽离经验回填

> 版本类型: MINOR architecture / governance。
> 基准: `09-system.entry首批抽离记录.md` 与 `10-system抽离完成记录.md`。
> 执行档位: 重型。
> 判定: 将 system 试水抽离的经验回填为后续抽离批次的执行准则；不启动整理和重构。

---

## 目标

`system.entry.backend_process` 的试水证明，抽离不是简单搬文件，而是先确认父模块职责，再确认 public 入口、关键内部实现、保留外部边界和回归证据。

本文件把这次经验固化为后续后端抽离、前端抽离和测试资产汰换的通用约束:

1. public 入口、兼容入口和关键内部实现必须分开登记。
2. 真实 owner 先于目录形态确定，不能为了目录好看移动不属于该模块的状态、router、handler 或 schema。
3. 允许同一目标分为“入口壳建立”和“内部实现归位”两个提交，但第二步必须重新过适配性校验。
4. closeout 必须写清未迁移边界，防止 AI 把抽离完成误说成整理或重构完成。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 抽离状态机、适配性校验、closeout 口径 | 回填 |
| 规范矩阵 | public/内部实现分类、owner 归属、父子通信、旧入口兼容 | 回填 |
| 引导矩阵 | 模块树、全量树、真实文件、测试/门禁坐标 | 回填 |
| 模块树 | `docs.matrix_governance`、后续抽离目标白箱节点 | 扩展 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根7.6 |
| 模块树节点 | `docs.matrix_governance`、后续目标模块白箱节点 |
| 真实文件 | `markdown/06-milestones/v4.16.0/01-规划方案.md`、`02-落地记录.md`、`03-后端抽离登记.md`、`04-前端抽离登记.md`、`05-测试资产汰换登记.md`、`06-后端接口边界首批抽离方案.md`、`09-system.entry首批抽离记录.md`、`10-system抽离完成记录.md`、`11-system抽离经验回填.md` |
| public 方法 | 后续目标登记时必须区分 public 入口、兼容 public 入口和关键内部实现 |
| 测试/门禁 | `tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、受影响模块编译/单测/契约测试 |

---

## system 经验提炼

| 经验 | system 中的证据 | 后续抽离准则 |
| --- | --- | --- |
| 先建立父模块壳 | `src/system/mod.rs`、`src/system/entry/mod.rs` 先落地 | 新目标先有父模块和子模块坐标，再移动实现 |
| public 与内部实现分离 | `run_server` 是 public，`run_api_server` 是关键内部启动实现 | 白箱登记不得把内部 helper 误写成 public API |
| 兼容入口保留 | `quantpilot::run_server` 通过 re-export 保持不变 | 旧调用方不能被迫改 import 或入口路径 |
| owner 不因顺手迁移 | `new_app_state` 留在 `app_runtime_helpers`，`build_app_router` 留在 `backend.interface_boundary` | 只迁移目标 owner，状态工厂、router、handler 需单独决策 |
| 第二刀重新校验 | `run_api_server` 迁移在 `run_server` 入口壳之后单独完成 | 后续内部实现归位必须重新跑适配性校验和门禁 |
| closeout 写未迁移边界 | `10-system抽离完成记录.md` 明确未迁移边界 | 每个完成记录必须列出仍归旧 owner 的内容 |

---

## 后续抽离执行准则

### 1. 白箱登记分类

每个候选目标必须至少分四类登记:

| 分类 | 定义 | 处理 |
| --- | --- | --- |
| public 入口 | 对外稳定调用点、组件入口、API facade 或 crate export | 可以迁移或包装，但必须保留兼容桥 |
| 兼容 public 入口 | 为旧调用路径保留的 re-export、adapter 或父入口 | 抽离阶段不得删除 |
| 关键内部实现 | 只被父入口调用、但属于同一 owner 的实现函数/组件/helper | 可在二次适配性校验后归位 |
| 保留外部边界 | 属于其他父模块的状态工厂、router、handler、schema、store owner | 不随本批次迁移 |

### 2. 两段式抽离

允许把一个目标拆为两段:

1. 入口壳建立: 新建父模块/子模块文件，移动或包装 public 入口，旧入口通过兼容桥保留。
2. 内部实现归位: 将属于同一 owner 的内部实现移入新模块，重新确认不迁移外部边界。

第二段不是第一段的自动延伸。只要涉及状态、锁、持久化、router、handler、store 或 schema，就必须回到适配性校验。

### 3. owner 复核

移动任何函数、组件或 helper 前，先回答:

1. 它是不是目标父模块的职责真源。
2. 它是否拥有状态、锁、事务、持久化或外部协议。
3. 它的调用方是否需要改变 import 或用户入口。
4. 它失败时是否能回到旧路径。
5. 它是否会让子模块绕过父模块横向通信。

任一答案不清楚，暂停实现并回到方案讨论。

---

## 对 BE-001 的回填

BE-001 后端接口边界后续推进时，应采用 system 试水后的口径:

| 项 | BE-001 处理方式 |
| --- | --- |
| public 入口 | `build_app_router` 和各 `register_*_routes` 作为后端接口边界 public/父级入口登记 |
| 兼容 public 入口 | 旧 route registration 顺序和旧 handler 调用链保留 |
| 关键内部实现 | 只允许归位 route owner 聚合、facade 委托、契约检查，不迁移 handler 业务实现 |
| 保留外部边界 | `AppState`、runtime state、executor state、response schema、artifact schema 仍归原 owner |
| closeout | 必须写明哪些 route owner 已归位，哪些 handler/state/schema 未迁移 |

---

## 对前端抽离的回填

前端抽离也必须区分入口和内部实现:

| 项 | 前端处理方式 |
| --- | --- |
| public 入口 | 页面组件、父 store hook、公开 projection builder |
| 兼容 public 入口 | 原页面入口、原 store action 入口、原 capability consumer |
| 关键内部实现 | 同父模块内的 section、projection helper、action bundle，可在验证后归位 |
| 保留外部边界 | 用户入口、路由、主题、文案语义、supported/unsupported 声明、后端 capability 真源 |
| closeout | 必须写明 UI 行为未改变，哪些 E2E 仍延后 |

---

## 回归证据准则

system 试水使用了编译、入口单测和治理门禁。后续抽离应按风险增加证据，而不是默认依赖庞大 E2E:

| 抽离类型 | 最小证据 | 加强证据 |
| --- | --- | --- |
| 启动/进程入口 | 编译、入口单测、旧 public 入口检查 | smoke 启动、日志/环境变量人工核查 |
| 后端接口边界 | API 契约测试、route owner 人工核查、response schema 对照 | artifact/event snapshot |
| runtime/state 边界 | 专门单测、锁顺序人工审查、持久化读写对照 | 并发小场景、事件序列 |
| 前端页面/组件 | 组件测试、projection 单测、人工截图核查 | 少量稳定 smoke |
| 测试资产汰换 | 替代证据登记、风险窗口 | 新稳定基线 |

---

## 暂停条件

出现以下情况必须暂停实现并讨论:

1. public 入口和关键内部实现无法区分。
2. 候选需要顺手迁移状态工厂、router、handler、schema、store owner 或执行端状态。
3. 兼容 public 入口无法保留。
4. 第二刀内部实现归位缺少新的适配性校验。
5. closeout 无法写清未迁移边界。
6. 抽离结果需要用“整理”“重构”“发布态优化”才能解释为完成。

---

## 非目标

- 不把 system 经验解释为所有模块都应复制 system 目录形态。
- 不启动整理阶段的命名收敛、目录美化或 public 方法收敛。
- 不启动重构阶段的旧实现退役、主入口切换或调用拓扑替换。
- 不主动提出发布版本过渡。
- 不允许子模块为了性能绕过父模块横向连接。

---

## 验收标准

1. v4.16 规划和落地记录引用本经验回填。
2. 后端、前端和测试资产登记能使用 public/内部实现/保留外部边界分类。
3. BE-001 后续推进能引用本文件判断 route owner、handler 和 state owner 的边界。
4. 模块树和全量树覆盖本文件。
5. 治理门禁能发现本经验回填缺失。
