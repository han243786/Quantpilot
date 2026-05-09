# QuantPilot API 参考

> 面向开发者 | 基于 v0.4.2 | 所有请求需携带 `Content-Type: application/json`

---

## 认证

未设置 `QUANTPILOT_API_KEY` 环境变量时，启动时会自动生成随机 key 并打印到日志。所有 `/api/` 路径（除 `/api/health`）需携带：

```
Authorization: Bearer <your-api-key>
```

## 端点列表

### 健康检查

```bash
curl http://127.0.0.1:3000/api/health
# → {"status":"ok"}
```

### 能力发现

```bash
curl http://127.0.0.1:3000/api/capabilities
# → {"api_version":"quantpilot-capabilities/v1","chain_stages":[...],...}
```

### 策略图 — 保存

```bash
curl -X POST http://127.0.0.1:3000/api/graphs/save \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "metadata": {"graph_id": "my_strategy", "name": "双均线策略"},
      "nodes": [
        {"id": "btc_1h", "type": "dataSource", "data": {"symbol": "BTCUSDT", "exchange": "binance", "interval": "1h"}},
        {"id": "ma_cross", "type": "intent", "data": {"indicator": "sma", "fast": 10, "slow": 30}}
      ],
      "edges": [{"source": "btc_1h", "target": "ma_cross"}]
    }
  }'
# → {"graph_id":"my_strategy","version_id":"...","saved_at":...,"collaboration":{...}}
```

### 策略图 — 加载

```bash
curl http://127.0.0.1:3000/api/graphs/my_strategy
# → {"metadata":{...},"nodes":[...],"edges":[...]}
```

### 策略图 — 列表

```bash
curl http://127.0.0.1:3000/api/graphs
# → [{"graph_id":"my_strategy","name":"双均线策略",...}]
```

### 策略图 — 删除

```bash
curl -X DELETE http://127.0.0.1:3000/api/graphs/my_strategy
# → {"graph_id":"my_strategy","deleted":true}
```

### 编译

```bash
curl -X POST http://127.0.0.1:3000/api/runtime/compile \
  -H "Content-Type: application/json" \
  -d '{
    "runtime_config": {
      "metadata": {"graph_id": "my_strategy", "compile_id": "c1", "name": "Test", "version": "1", "mode": "paper"},
      "data_sources": [{"data_id": "btc_1h", "symbol": "BTCUSDT", "exchange": "binance", "interval": "1h", "days": 30}],
      "intent_generators": [{"intent_id": "ma_cross", "kind": "sma", "params": {"fast_period": 10, "slow_period": 30}}],
      "agents": [],
      "risk_controls": [],
      "executions": []
    },
    "graph_json": {"metadata": {"graph_id": "my_strategy"}, "nodes": [...], "edges": [...]}
  }'
```

> **字段说明**: `intent_generators` 是意图列表（策略核心），`agents`/`risk_controls`/`executions` 为必填字段（可为空数组）。前端通过 `graphStoreCompileApi.js` 自动填充这些字段。

### 回测

```bash
curl -X POST http://127.0.0.1:3000/api/runtime/backtest \
  -H "Content-Type: application/json" \
  -d '{
    "runtime_config": {
      "metadata": {"graph_id": "my_strategy", "compile_id": "c1", "name": "Test", "version": "1", "mode": "paper"},
      "data_sources": [{"data_id": "btc_1h", "symbol": "BTCUSDT", "exchange": "binance", "interval": "1h", "days": 30}],
      "intents": [{"intent_id": "ma_cross", "kind": "sma", "params": {"fast_period": 10, "slow_period": 30}}],
      "risks": []
    },
    "backtest": {"replay_source": "deterministic_mock"}
  }'
```

### 回测列表

```bash
curl http://127.0.0.1:3000/api/runtime/backtests
# → [{"backtest_id":"...","summary":{...},...}]
```

### 回测详情

```bash
curl http://127.0.0.1:3000/api/runtime/backtests/<backtest_id>
```

### 凭证管理

```bash
# 列出标签
curl http://127.0.0.1:3000/api/credentials
# → {"services":["okx"]}

# 新增/更新
curl -X POST http://127.0.0.1:3000/api/credentials \
  -H "Content-Type: application/json" \
  -d '{"service":"okx","fields":{"key":"...","secret":"...","passphrase":"..."}}'
# → {"stored":"okx"}

# 删除
curl -X DELETE http://127.0.0.1:3000/api/credentials/okx
# → {"deleted":"okx"}
```

### 模拟运行

```bash
curl -X POST http://127.0.0.1:3000/api/runtime/test-run \
  -H "Content-Type: application/json" \
  -d '{...}'  # 格式同 compile 请求
```

## 错误响应格式

所有错误统一为：

```json
{"error": "<error_code>", "message": "<中文说明>"}
```

常见错误码：`bad_request` / `not_found` / `unauthorized` / `capability_gated`
