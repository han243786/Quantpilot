# RFC-020 Plugin Manifest Protocol

## 状态

- status: draft
- version: `quantpilot/plugin-manifest/v1`
- scope: internal boundary only

## 目的

本 RFC 定义 QuantPilot 第八周需要的最小插件前置边界。

目标不是开放插件市场，也不是引入远程安装、依赖解析、签名分发，而是先把以下约束固定下来：

- 插件只能通过 manifest 声明能力
- 插件只能挂在白名单 extension point 上
- 插件不能绕过 `Data -> Intent -> Agent -> Risk -> Execution -> Fill` 主链路
- 兼容性检查必须先于加载和注册

## 非目标

本 RFC 明确不包含：

- 远程 registry
- 第三方安装
- 复杂版本求解
- 动态下载与热更新
- 市场化分发

## 清单结构

最小 manifest 字段：

- `api_version`
- `id`
- `version`
- `kind`
- `display`
- `capability_declarations`
- `extension_points`
- `execution`
- `compatibility`
- `security`
- `dependencies`
- `params_schema` 可选

## 种类白名单

当前只允许五类插件：

- `data`
- `intent`
- `agent`
- `risk`
- `execution`

## 扩展点白名单

当前只允许五个 extension point：

- `data_module_provider`
- `intent_module_provider`
- `agent_module_provider`
- `risk_checker_provider`
- `execution_module_provider`

kind 与 extension point 必须一一对应：

- `data` -> `data_module_provider`
- `intent` -> `intent_module_provider`
- `agent` -> `agent_module_provider`
- `risk` -> `risk_checker_provider`
- `execution` -> `execution_module_provider`

不允许跨层挂接，不允许一个插件直接声明跨多层主链路权限。

## 能力声明

manifest 必须显式声明 capability，而不是隐式暴露代码实现。

最小结构：

```json
{
  "id": "strategy_ir.custom",
  "version": "v1"
}
```

## 兼容性

manifest 必须声明至少两个兼容性锚点：

- `core_ir_version`
- `capability_api_version`

第八周阶段只做字面兼容性校验，不做 semver 范围求解。

## 安全边界

manifest 必须声明：

- `max_compute_ms`
- `max_memory_mb`
- `allow_network`

当前默认原则：

- `allow_network=false`
- 不允许通过 manifest 申请绕过风险控制或执行模块
- 不允许直接声明交易所下单能力

## 与定制 MVP 的关系

受限 `Custom` MVP 仍然优先走内建 lowering 路径，不依赖插件运行。

插件 manifest 在本阶段的作用只有两个：

- 为未来把 built-in 能力表达成 plugin-shaped 对象预留边界
- 把 capability declaration / compatibility / extension point 先固定为稳定协议对象

## 验收条件

若某能力满足以下条件，才允许进入更强的插件化阶段：

- manifest 字段稳定
- extension point 白名单稳定
- compatibility 检查稳定
- 不会绕过主链路
- 测试覆盖注册与校验失败路径
