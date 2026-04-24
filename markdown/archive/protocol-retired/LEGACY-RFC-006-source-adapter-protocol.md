# Legacy RFC-006 Source Adapter Protocol

数据源适配协议定义系统如何把 `DataRequest` 翻译成具体数据源调用。它属于实现层协议，而不是 QRPC 的核心语义层协议。

## 当前定位

在新的 QRPC 口径下，需要明确以下边界：

- `DataRequest.primary_data_type` 定义一级数据类型
- `DataRequest.source_type` 定义数据类型的语义子维度
- `Source Adapter` 只负责把这些语义请求翻译成具体调用

因此，`source_type` 不是适配器字段，也不是交易所端点标识。适配器只能消费它，不能重新定义它。

## 适配层职责

- 选择合适的数据提供方和能力描述
- 构造请求参数
- 处理认证、限流和错误映射
- 把原始响应送入规范化流程

## 非职责

- 不改变 `DataRequest` 的语义
- 不直接把交易所结构暴露给上层
- 不替代规范化协议
