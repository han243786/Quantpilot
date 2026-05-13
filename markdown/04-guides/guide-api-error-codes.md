# API 错误码参考

> v1.0.7 | 面向开发者、运维者和 API 调用者

---

## 协议级错误

| 错误码 | HTTP | 含义 | 修复建议 |
|--------|:--:|------|---------|
| `unauthorized` | 401 | 未认证 | 设置 `QUANTPILOT_API_KEY` 环境变量，或 `QUANTPILOT_DEV=true` |
| `not_found` | 404 | 资源不存在 | 检查请求路径和资源 ID |
| `internal_error` | 500 | 内部错误 | DEV 模式下查看 detail 字段；生产环境重试或联系支持 |
| `service_unavailable` | 503 | 编译服务关闭 | 检查后端进程是否正常运行 |
| `bad_request` | 400 | 请求格式错误 | 检查 Content-Type 和 JSON 格式 |
| `rate_limited` | 429 | 速率限制 | 等待 Retry-After 秒后重试 |

---

## 能力治理错误

| 错误码 | 含义 | 修复建议 |
|--------|------|---------|
| `capability_gated` | 使用了未启用的能力 | 检查模块 key 是否在 `/api/capabilities` 白名单中 |
| `missing_capability_context` | 缺少 capability 上下文 | 调用 `GET /api/capabilities` 获取 schema_hash |
| `stale_capability_hash` | capability 哈希过期 | 刷新 `/api/capabilities` 后重试 |
| `malformed_capability_hash` | schema_hash 格式错误 | 格式: `sha256:` + 64 位小写十六进制 |
| `permission_boundary_mismatch` | 权限边界不匹配 | 刷新 capability 上下文 |
| `unsupported_runtime_mode` | 不支持的运行模式 | 当前仅支持 `paper` |
| `unsupported_module` | 模块未启用 | 检查模块是否在 capability 白名单中 |
| `unsupported_exchange` | 未验证的交易所 | 已验证: `binance`, `okx` |
| `unsupported_symbol` | 未验证的交易对 | 已验证: `BTCUSDT`, `ETHUSDT`, `SOLUSDT` |
| `unsupported_execution_module` | 不支持的执行模块 | 当前仅支持 `builtin.execution.paper` |
| `invalid_rebalance_schedule` | 不支持的再平衡频率 | 支持: `every_slow`, `every_1d`, `weekly` |
| `invalid_rebalance_target_weights` | 权重格式错误 | 逗号分隔数字，如 `"0.5, 0.3, 0.2"` |
| `invalid_risk_limit` | 风控参数超限 | 检查风控字段的值范围 |

---

## 编译错误码

### 运行时编译 (RUNTIME_COMPILE)

| 错误码 | 含义 | 修复建议 |
|--------|------|---------|
| `runtime_compile_failed` | 图编译合约校验失败 | 检查所有节点正确连线且 graph_id 不含非法字符 |
| `qs_generation_failed` | 图→QS 转换失败 | 检查图结构完整性 |
| `qs_parse_failed` | QS 解析失败 | 检查语法，参考 QuantScript 文档 |
| `qs_lowering_failed` | QS 编译失败 | 检查诊断详情中的 QPQSLOW 错误码 |
| `strategy_ir_compile_failed` | Strategy IR 编译失败 | 检查 Strategy IR JSON 格式 |
| `quantscript_lowering_failed` | QS 正式编译失败 | 检查策略函数语法和指标参数 |

### QPQSLOW 系列 (QS 编译诊断)

| 代码 | 含义 | 修复建议 |
|------|------|---------|
| QPQSLOW001 | 不支持的条件下发 Intent | 条件重写为支持的指标或价差意图，或改为无条件 |
| QPQSLOW002 | 无可执行的 emit Intent | 确保至少一个 emit Intent(...) 可编译 |
| QPQSLOW003 | emit Intent 缺少数据源 | 添加至少一个 fetch/get_data 调用 |
| QPQSLOW004 | 不支持的 Intent 动作 | 使用 BUY 或 SELL |
| QPQSLOW005 | emit Intent 缺 action 参数 | 添加 action 参数 |
| QPQSLOW006 | 缺少 fn strategy() | 在 .qs 文件中声明 `fn strategy() { ... }` |
| QPQSLOW007 | 缺少数据获取调用 | 添加至少一个 fetch/get_data 调用 |
| QPQSLOW008 | 多个 rebalance 指令 | 保持最多一个 rebalance(...) |
| QPQSLOW009 | 不支持的 rebalance 频率 | 使用 `"1d"`, `"slow"` 或 `"weekly"` |
| QPQSLOW010 | 缺少 universe_snapshot | 使用快照依赖操作时提供 universe_snapshot |
| QPQSLOW011 | 不支持的 sort_by 键 | 使用 `symbol`, `market_cap`, `volume_24h`, `listing_age_days` |
| QPQSLOW012 | 不支持的排序方向 | 使用 `asc` 或 `desc` |
| QPQSLOW013 | rebalance 需分配函数 | 使用 equal_weight/fixed_weights/rank_weight/score_weight |
| QPQSLOW014 | rebalance 需 universe 表达式 | 使用 symbols()/universe()/filter()/sort_by()/top() |
| QPQSLOW015 | rebalance 需至少一个标的 | 确保 universe 表达式产生至少一个标的 |
| QPQSLOW016 | fixed_weights 数量不匹配 | 每个标的对应一个权重值 |
| QPQSLOW017 | 负数 fixed_weights | 所有权重值 ≥ 0 |
| QPQSLOW018 | fixed_weights 总和为零 | 至少一个权重大于 0 |
| QPQSLOW019 | 不支持的 rank_weight 方法 | 使用 `"linear"` 或 `"inverse_rank"` |
| QPQSLOW020 | 不支持的 score_weight 归一化 | 使用 `"sum"` |
| QPQSLOW021 | weights 需数值列表 | 使用数值列表字面量 |
| QPQSLOW022 | 指标缺少数据源 | 第一个参数传入 fetch/get_data |
| QPQSLOW023 | 指标周期参数无效 | 周期 > 0 |
| QPQSLOW024 | 移动平均缺少数据源 | 传入 fetch/get_data 或已识别的 MACD 线 |
| QPQSLOW025 | universe 需值表达式 | 传入 symbols()/universe()/filter()/sort_by()/top() |
| QPQSLOW026 | symbols 需列表字面量 | 使用 `symbols(["BTCUSDT"])` |
| QPQSLOW027 | symbols 需字符串 | 列表项使用字符串字面量 |
| QPQSLOW028 | top 需数值计数 | 第二个参数传入数值，如 `top(..., 10)` |
| QPQSLOW999 | 未预期的内部错误 | 检查策略语法，联系支持 |

### QS 系列 (语义分析诊断)

| 代码 | 含义 | 修复建议 |
|------|------|---------|
| QS0401 | 前视风险: 负数序列索引 | 使用 `series[0]` 获取最新或正数回溯 |
| QS0402 | 前视风险: center=true | 不使用 center=true 或改用单侧窗口 |
| QS0403 | trailing 窗口跨度无效 | 使用正数跨度如 `series[1..]` |
| QS0404 | 索引可能超出回看窗口 | 减小索引值或增大 fetch 的回看天数 |
| QS0501 | 预热不足 | 增大 fetch 的回看天数以满足指标周期需求 |
| QS0502 | 指标周期截断 | 浮点周期被截断为整数 |
| QS0503 | fetch lookback < 1 | 已自动设为 1 |
| QS0504 | 指标周期 < 1 | 指标周期必须 ≥ 1 |
| QS0505 | 未知交易对 | 使用 BTCUSDT, ETHUSDT, SOLUSDT |
| QS0601 | 不支持异步函数 | 移除 async 关键字 |
| QS0602 | 不支持递归调用 | 改用迭代方式 |
| QS0603 | 不支持 while 循环 | 改用 for ... in ... 或窗口聚合 |
| QS0604 | 不支持 match 语句 | 改用 if/else |
| QS0605 | 不支持 await 表达式 | 移除 await |
| QS0606 | 仅支持 fetch 类 ? 后缀 | 确保 ? 只用于 fetch 类表达式 |
| QS0607 | 仅支持 Universe 的 for 循环 | 确保 for 循环遍历 universe 值 |
| QS0608 | 不支持简单 import | 使用 `from module import name as alias` |
| QS0609 | 不支持 .push() 构建列表 | 使用列表字面量 `[a, b, c]` |
| QS0610 | 不支持 .ok()/.retryable() | 移除这些辅助方法调用 |
| QS0611 | 缺少 emit Intent() | 策略函数必须包含至少一个 emit Intent() |
| QS0612 | 死代码 emit | if 条件恒为 false 的分支中的 emit 不会执行 |
| QS0613 | 重复的变量定义 | 修改变量名避免冲突 |

---

## Strategy IR 诊断

| 代码 | 含义 | 修复建议 |
|------|------|---------|
| CUSTOM001-012 | 自定义表达式超限 | 使用支持的算术和窗口聚合运算 |
| QPSTRATSPREAD001 | spread_output_code 错误 | 设置 `spread_output_code = 1` |
| QPSTRATSPREAD002 | max_time_diff_ms 缺失 | 设置正值 |
| QPSTRATSPREAD003 | 价差阈值方向 | 使用单向买入阈值如 `spread_signal > 5` |
| QPSTRATSPREAD004 | 价差输入不足 | 提供恰好两个价差输入 |
