# RFC-016 Capability Discovery Protocol

## 状态

当前状态：draft

适用范围：QuantPilot beta capability discovery API

当前实现入口：`GET /api/capabilities`

## 目标

本协议定义 QuantPilot 当前对外暴露的能力发现结构，用于解决以下问题：

- 前端必须以真实后端能力为准，而不是静态猜测
- 文档、UI、编译入口和 runtime 边界必须使用同一份能力描述
- 已声明但未开放的能力必须显式区分，不能混入“已支持”
- capability 演进时必须兼容旧消费者，同时允许新消费者读取更稳定的结构

本协议不负责：

- 插件生态治理
- 运行时详细语义
- 回测工件协议
- 前端具体展示方式

## 核心原则

- capability response 是当前 beta 的唯一真实边界来源
- capability response 必须区分 `declared` 与 `supported`
- capability response 允许保留兼容旧字段，但新实现必须优先消费结构化 support entries
- capability response 只能描述真实 compile/runtime/front-end exposure 边界，不能描述愿景功能

## 顶层结构

响应对象：

```json
{
  "api_version": "quantpilot-capabilities/v1",
  "strategy_ir": {},
  "runtime": {},
  "market_data": {},
  "frontend": {}
}
```

其中：

- `api_version` 是 capability 协议版本
- `strategy_ir` 描述 Strategy IR 层的声明与支持边界
- `runtime` 描述运行模式和执行模块边界
- `market_data` 描述当前 beta 可用市场数据边界
- `frontend` 描述当前前端允许暴露的模块边界

## 通用支持状态

所有新的结构化 support entry 使用统一状态：

- `supported`
- `declared_only`

含义：

- `supported` 表示当前版本真实可用
- `declared_only` 表示协议或枚举已保留，但当前版本尚未开放

## strategy_ir

### 兼容字段

```json
{
  "declared_indicator_kinds": ["ma_cross", "rsi", "macd", "momentum", "spread", "z_score", "custom"],
  "supported_indicator_kinds": ["ma_cross", "rsi", "macd", "momentum", "z_score"]
}
```

这两个字段保留给旧消费者使用。

### 新字段

```json
{
  "indicator_support": [
    { "kind": "ma_cross", "status": "supported", "reason": null },
    { "kind": "spread", "status": "declared_only", "reason": "..." },
    { "kind": "custom", "status": "declared_only", "reason": "..." }
  ]
}
```

字段说明：

- `kind`: `IndicatorKind`
- `status`: `supported | declared_only`
- `reason`: 对未开放能力的原因说明；`supported` 时应为 `null`

约束：

- `declared_indicator_kinds` 必须等于 `indicator_support[*].kind`
- `supported_indicator_kinds` 必须等于 `indicator_support[*].kind where status == supported`

## runtime

### 兼容字段

```json
{
  "supported_modes": ["paper"],
  "supported_execution_modules": ["builtin.execution.paper"]
}
```

### 新字段

```json
{
  "mode_support": [
    { "key": "paper", "status": "supported", "reason": null }
  ],
  "execution_module_support": [
    { "key": "builtin.execution.paper", "status": "supported", "reason": null }
  ]
}
```

字段说明：

- `key`: 模式或模块键
- `status`: `supported | declared_only`
- `reason`: 仅对未开放项填写

当前 beta 约束：

- 仅 `paper` 模式可用
- 仅 `builtin.execution.paper` 可用

## market_data

### 兼容字段

```json
{
  "supported_exchanges": ["binance", "okx"],
  "supported_symbols": ["BTCUSDT"]
}
```

### 新字段

```json
{
  "exchange_support": [
    { "key": "binance", "status": "supported", "reason": null },
    { "key": "okx", "status": "supported", "reason": null }
  ],
  "symbol_support": [
    { "key": "BTCUSDT", "status": "supported", "reason": null }
  ]
}
```

当前 beta 约束：

- 交易所仅 `binance` 与 `okx`
- 交易对仅 `BTCUSDT`

## frontend

### 兼容字段

```json
{
  "supported_module_keys": ["..."],
  "unsupported_module_reasons": {
    "builtin.intent.spread_observer": "...",
    "builtin.agent.arbitrage": "..."
  }
}
```

这两个字段保留给旧前端逻辑。

### 新字段

```json
{
  "declared_module_keys": ["..."],
  "module_support": [
    { "module_key": "builtin.data.kline", "status": "supported", "reason": null },
    { "module_key": "builtin.intent.spread_observer", "status": "declared_only", "reason": "..." },
    { "module_key": "builtin.agent.arbitrage", "status": "declared_only", "reason": "..." }
  ]
}
```

字段说明：

- `declared_module_keys`: 前端内建模块全集
- `module_support`: 每个模块的当前边界
- `module_key`: 模块键
- `status`: `supported | declared_only`
- `reason`: 未开放原因

约束：

- `declared_module_keys` 必须覆盖 `module_support[*].module_key`
- `supported_module_keys` 必须等于 `module_support[*].module_key where status == supported`
- `unsupported_module_reasons` 必须至少覆盖 `module_support[*] where status != supported and reason != null`

## 当前 beta 真实边界

当前版本真实边界如下：

- Strategy IR 已支持：
  - `ma_cross`
  - `rsi`
  - `macd`
  - `momentum`
  - `z_score`
- Strategy IR 仅声明未开放：
  - `spread`
  - `custom`
- frontend 模块已支持：
  - `builtin.data.kline`
  - `builtin.data.quote`
  - `builtin.intent.double_ma`
  - `builtin.intent.ma_deviation`
  - `builtin.intent.rsi`
  - `builtin.intent.macd`
  - `builtin.intent.momentum`
  - `builtin.intent.zscore`
  - `builtin.agent.weighted`
  - `builtin.risk.global`
  - `builtin.execution.paper`
  - `builtin.runtime.control`
- frontend 模块仅声明未开放：
  - `builtin.intent.spread_observer`
  - `builtin.agent.arbitrage`
- runtime 模式已支持：
  - `paper`
- execution 模块已支持：
  - `builtin.execution.paper`
- 市场数据边界已支持：
  - exchange: `binance`, `okx`
  - symbol: `BTCUSDT`

## 兼容性策略

- 新消费者应优先读取：
  - `indicator_support`
  - `mode_support`
  - `execution_module_support`
  - `exchange_support`
  - `symbol_support`
  - `declared_module_keys`
  - `module_support`
- 旧消费者可继续读取：
  - `declared_indicator_kinds`
  - `supported_indicator_kinds`
  - `supported_modes`
  - `supported_execution_modules`
  - `supported_exchanges`
  - `supported_symbols`
  - `supported_module_keys`
  - `unsupported_module_reasons`
- 新字段与旧字段必须在语义上保持一致
- 若未来新增状态枚举，旧字段仍必须能退化表达“当前可用集合”

## 实现要求

- capability 输出必须有 service-level 测试覆盖
- capability 输出变更时，前端 capability consumer 必须同步验证
- UI、文档和提示口径必须以 capability 协议为准
- 未开放能力不得通过普通用户路径暴露为已支持

## 不允许的情况

- 文档写“支持”，但 capability 返回未开放
- capability 写“支持”，但 compile/runtime 实际拒绝
- 前端依赖静态模块表绕过 capability discovery
- 用 roadmap 或设计稿内容填充 capability response
