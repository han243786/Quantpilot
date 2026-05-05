# RFC-017 回测工件协议

## 状态

当前状态：draft

适用范围：

- `POST /api/runtime/compile` 返回的编译工件包
- `POST /api/runtime/backtest` 中嵌入的编译工件包
- `storage/backtests/*.json` 下持久化的回测记录

## 目标

本 RFC 定义了当前以运行为中心的 beta 版本的稳定工件边界。

其直接目的是将回测持久化从零散的临时数据转储中规范化：

- `protocol_name`
- `config_hash`
- `core_ir`
- 运行时/回测输出 JSON

取而代之，系统应持久化并暴露带有稳定名称、模式版本和摘要规则的显式工件。

## 工件集

当前工件包包含三个对象：

1. `StrategyArtifact`（策略工件）
2. `CompileArtifact`（编译工件）
3. `CoreIrArtifact`（Core IR 工件）

这些对象共同以 `CompileArtifactBundle`（编译工件包）的形式呈现。

## 版本化对象

### StrategyArtifact（策略工件）

```json
{
  "schema_version": "quantpilot/strategy-artifact/v1",
  "artifact_id": "strategy_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "strategy_id": "graph_test",
  "name": "Test Graph",
  "source_kind": "frontend_graph",
  "source_ref": "graph_test",
  "metadata": {
    "runtime_mode": "paper"
  },
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

目的：

- 标识面向用户的策略来源
- 区分来源出处与编译输出
- 为存储和后续工件投影提供稳定的父级对象

### CoreIrArtifact（Core IR 工件）

```json
{
  "schema_version": "quantpilot/core-ir-artifact/v1",
  "artifact_id": "core_ir_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "ir_version": "quantpilot/core-ir/v1",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "core_ir": {}
}
```

目的：

- 冻结运行时使用的精确降级 Core IR
- 为 `RunSpec` 提供规范的摘要锚点
- 使未来的工件投影可引用而无需重新推导 Core IR

### CompileArtifact（编译工件）

```json
{
  "schema_version": "quantpilot/compile-artifact/v1",
  "artifact_id": "compile_artifact_<digest-prefix>",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "protocol_name": "quantpilot/minimal-sim/v1",
  "config_hash": "runtime-spec-...",
  "strategy_artifact_id": "strategy_artifact_...",
  "core_ir_artifact_id": "core_ir_artifact_...",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "runtime_config": {}
}
```

目的：

- 冻结编译后的运行时协议配置
- 将来源工件与降级后的 Core IR 工件关联起来
- 提供运行/回测规范所使用的稳定编译边界

## 摘要规则

所有工件摘要使用：

- 算法：`sha256_canonical_json`
- 规范形式：对工件负载执行 `serde_json::to_vec(...)`
- 输出格式：小写十六进制字符串

说明：

- `artifact_id` 不是完整摘要；它是从摘要前缀派生的可读标识符
- `config_hash` 保留为现有编译消费者使用的运行时协议哈希
- 工件摘要与 `config_hash` 扮演不同角色，不应混用

## 边界规则

- `StrategyArtifact` 关注来源标识和出处，而非运行时语义
- `CompileArtifact` 关注编译后的运行时输入边界
- `CoreIrArtifact` 关注可执行的降级表示
- `RFC-019` 中定义的输出工件应引用此包，而非引入并行的编译标识

## 当前实现

当前代码路径：

- 共享模式类型：`qrpc_core/src/lib.rs`
- 编译端点组装：`src/main.rs`
- 回测端点持久化：`src/main.rs`

## 范围外

本 RFC 尚未定义多运行比较工件布局。

回测输出侧工件在 `RFC-019` 中定义。
