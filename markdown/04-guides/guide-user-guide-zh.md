# QuantPilot 用户指南

> v4.8.2 | 中文版

---

## 快速开始

### 一键启动

```bat
.\start.bat
```

或使用 PowerShell:

```powershell
.\start.ps1
```

默认情况下, 后端监听 `3000`, 主前端开发服务监听 `5173`, Tauri 桌面窗口会自动打开。

### 分开启动

```powershell
# 仅启动后端
cargo run

# 仅启动主前端
cd frontend
npm install
npm run dev

# 仅启动执行端前端
cd frontend-executor
npm install
npm run dev
```

### 常用环境变量

| 变量 | 用途 | 默认值 |
|------|------|--------|
| `QUANTPILOT_DEV` | 开发模式, 用于本地快速调试 | `false` |
| `QUANTPILOT_API_KEY` | API 访问密钥 | 自动生成 |
| `QUANTPILOT_RATE_LIMIT_RPS` | 每秒请求限流 | `100` |
| `QUANTPILOT_PORT` | 后端监听端口 | `3000` |

---

## 核心概念

### 策略图

QuantPilot 的策略由有向图表达。推荐的主链路是:

```text
Data -> Intent -> Agent -> Risk -> Execution -> Fill
```

| 阶段 | 作用 | 示例 |
|------|------|------|
| Data | 市场数据来源 | OKX K 线 |
| Intent | 信号生成 | RSI、MACD、双均线 |
| Agent | 决策组合 | 多信号加权、组合再平衡 |
| Risk | 风险控制 | 仓位上限、杠杆限制 |
| Execution | 执行意图 | Paper 模拟执行或 OKX demo provider |
| Fill | 成交回执 | 模拟成交、回放证据 |

### 已验证指标

当前已覆盖 18 类指标: MA Cross、RSI、MACD、Momentum、Spread、ZScore、QuoteObserve、ATR、Bollinger Bands、OBV、CMF、ADX、Stochastic、CCI、Parabolic SAR、Keltner Channel、Donchian Channel。

### 已验证交易所与标的

- 交易所: `binance`、`okx`
- 交易对: `BTCUSDT`、`ETHUSDT`、`SOLUSDT`

### 运行模式

- **PaperSimulated**: 本地模拟撮合, 可配置手续费、滑点、延迟。
- **PaperActual**: 使用 OKX demo/testnet profile 的 provider 路径, 必须保留非真实资金边界。
- **LiveActual**: 真实资金能力保持更高安全等级, 不作为普通策略自动运行入口。

---

## 创建策略

### 使用图编辑器

1. 在左侧模块栏拖入 Data 节点, 例如 K 线数据。
2. 在右侧属性面板配置交易所、交易对、周期和 lookback。
3. 拖入 Intent 节点, 例如 RSI 或双均线信号, 并连接到 Data。
4. 继续连接 Agent、Risk、Execution 和 Fill 节点。
5. 点击编译, 查看诊断、协议摘要和能力约束。

### 使用 QuantScript

打开 QuantScript 编辑器, 编写或粘贴 `.qs` 源码。编辑器会保留草稿, 支持 `Ctrl+Enter` 运行测试。

```text
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

---

## 回测与运行

### 运行回测

1. 编译策略图。
2. 在工作区或策略中心打开回测入口。
3. 选择确定性 mock、历史数据或 v4 replay 输入。
4. 查看权益曲线、交易记录、回撤、夏普、索提诺和 v4 evidence。

### 参数扫描

参数扫描适合比较手续费、滑点、延迟或策略阈值。建议每次只变更一组关键参数, 便于定位收益和风险变化来源。

### 执行端

执行端是独立进程, 默认端口 `3001`。它用于策略部署、状态轮询、K 线观察、订单/资产面板和 v4 evidence 检查。PaperActual 相关路径必须使用 demo/testnet profile, 不应连接真实资金环境。

---

## 日常功能

### 策略包导入/导出

顶部工具栏提供策略包导入/导出。导出文件包含策略图、QuantScript 源码、能力上下文和版本信息; 导入时会生成新的本地策略 ID, 避免覆盖当前图。

### 设置页

设置页用于管理:

- 界面语言
- 主题偏好
- QuantScript 草稿自动保存
- 当前 profile 摘要

### 告警、快照与运行手册

- 告警页展示数据新鲜度、风险拒绝率、事件缺口和存储水位等规则。
- 快照页用于查看策略或运行状态的签名快照。
- 运行手册提供常见故障的诊断、恢复和验证步骤。

---

## 故障排除

| 现象 | 处理 |
|------|------|
| 前端显示能力加载失败 | 确认后端 `3000` 已启动, 并检查 `/api/capabilities` |
| 编译失败 | 先查看诊断面板中的节点 ID、错误码和修复建议 |
| QuantScript 测试失败 | 检查语法、指标参数和测试断言; 错误面板会显示人类可读说明 |
| 执行端 K 线为空 | 确认执行端 `3001` 已启动, 策略已部署, market data 轮询未报错 |
| 导入策略包失败 | 检查 JSON 文件大小、格式版本和 `graph` 字段是否存在 |

---

## 质量门禁

本地开发常用检查:

```powershell
cargo check --workspace
cargo test --workspace --no-run

cd frontend
npm run build
npx vitest run

cd ..\frontend-executor
npm run build
```

更多错误码说明见 [API 错误码指南](./guide-api-error-codes.md)。
