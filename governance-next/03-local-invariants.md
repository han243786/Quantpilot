# 局部不变量

局部不变量是 General Policy 的模块级投影。它比全局政策更贴近代码，比普通注释更正式。

## 1. 为什么需要局部不变量

General Policy 不承担实体层覆盖责任。实体层覆盖由全量树、模块树、用户功能切面树和总游标承担。

GP 只定义跨实体不变量、禁止行为、升级触发条件和发布审计标准。每条需要日常执行的 GP 规则，都应尽量绑定到具体模块、边、切面或门禁。

## 2. 绑定对象

局部不变量可以绑定到:

1. 模块树节点。
2. 模块间调用边。
3. 用户功能切面。
4. capability 投影点。
5. 运行时状态机。
6. 门禁脚本。

## 3. 推荐格式

```text
module: executor.order_panel
scope:
  user_function_facet: realtime_executor.orders
  full_feature_tree: root3_executor
  module_tree: frontend_executor.order_panel

local_invariants:
  - 不得把 LiveActual 显示为 supported。
  - 不得改变 execution mode 语义。
  - 不得绕过后端或执行端状态来源。

policy_refs:
  - GP: Execution 能力来源必须显式标记。
  - GP: 前端能力入口以后端 capability 为真源。

stop_if:
  - 需要新增 API 字段。
  - 需要调整 capability response。
  - 需要触碰真实下单路径。
```

## 4. 执行规则

1. 游标进入某模块前，必须加载该模块的局部不变量。
2. 局部不变量可以引用 GP，但不能复制大段 GP。
3. 如果局部不变量与旧治理冲突，试运行阶段以旧治理为准。
4. 如果 GP 条款长期无法绑定到任何模块、边、门禁或触发条件，应标记为低执行力条款，等待重写或降级。

## 5. 最小要求

每个试运行模块至少声明:

1. 它属于哪个用户功能切面。
2. 它属于哪个全量树节点。
3. 它属于哪个模块树节点。
4. 它暴露的 public surface。
5. 它禁止越过的边。
6. 它触发升档的条件。
