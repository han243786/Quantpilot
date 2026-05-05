# 策略模板库

## 当前边界

当前模板库是前端拥有的起始界面。

它有意保持狭窄：

- 使用规范本地模板列表
- 仅从当前支持的 builtin 模块构建起始图
- 将选定模板加载到当前工作草稿中
- 不创建第二套后端模板传输
- 加载时不自动持久化图版本

## 当前起始模板

### `dual_ma_trend`

目的：

- 从简单的 BTC 趋势跟踪图开始

模块：

- `builtin.data.kline`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

交易对：

- `BTCUSDT`

### `rsi_reversion`

目的：

- 从轻量级 ETH 均值回归图开始

模块：

- `builtin.runtime.control`
- `builtin.data.kline`
- `builtin.intent.rsi`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

交易对：

- `ETHUSDT`

### `multi_symbol_rebalance`

目的：

- 从当前 beta 多交易对再平衡界面开始

模块：

- `builtin.data.kline`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

交易对：

- `BTCUSDT`
- `ETHUSDT`
- `SOLUSDT`

## 加载规则

加载模板应按字面理解：

- 选定模板替换当前内存中的工作草稿
- 加载的图保持在现有图/运行时配置界面上
- 持久化历史、实验和回测索引数据保持在草稿重置之外
- 如果操作员希望加载的草稿成为持久化的图版本，应显式保存

## 这不是什么

当前模板库不是：

- 后端模板注册表
- 市场
- 第二套图 DTO 系列
- 现有图/运行时界面之外的第二套起始图协议

未来的扩展只应在当前本地规范列表不足以支持的产品界面时发生。
