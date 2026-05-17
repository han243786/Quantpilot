# v2.3.0 Closeout

> MINOR 版本 | 2026-05-18 | 错误国际化 + TLS + JWT刷新 + 审计清零
>
> 执行基调: [General_Policy](../../General_Policy.md) + [超级规范化](../../01-principles/principles-super-standardization.md)

---

## 一、版本轨迹

| 版本 | 类型 | 核心变更 |
|------|------|---------|
| v2.2.1 | MINOR | 架构重构 (Coordinator拆分/QuantPilotError) + i18n完整化 + 可观测性 |
| **v2.3.0** | **MINOR** | **错误国际化 + TLS + JWT刷新 + P1/P2清零** |

---

## 二、四大功能目标验收

### 1. 后端错误消息国际化 ✅
- `src/error_codes.rs` — 41 个语言中立错误码 (ERR_GRAPH_ID_EMPTY, ERR_COMPILE_FAILED 等)
- `src/api_errors.rs` — `json_bad_request_with_code()` 向后兼容接线
- `frontend/src/utils/errorMessages.js` — 30+ en/zh 本地化映射
- API 响应新增 `error_code` 字段: `{"error":"bad_request","error_code":"STRATEGY_EMPTY_INTENT","message":"..."}`
- 零破坏现有 API (中文 message 字段保留)

### 2. TLS 传输安全 ✅
- `nginx.conf` — TLS 反向代理 (443→3000, HTTP→HTTPS redirect)
- Dockerfile — HEALTHCHECK + curl 安装
- TLSv1.2/TLSv1.3, ciphers HIGH:!aNULL:!MD5

### 3. JWT 令牌刷新 ✅
- `POST /api/auth/refresh` — 验证旧 token → 签发新 24h token
- 保留 username claim (v2.3.0 审计修复)
- 内置于 `register_auth_routes`, 继承 auth_rate_limit

### 4. 测试与审计 ✅
- 专项诱错: 9 发现 (P1×2 已修复)
- P1 修复 3 项 (fill_engine LRU/杠杆 floor/后台任务)
- P2 修复 5 项 (确认对话框/Docker HEALTHCHECK/api_errors/reveal/TopToolbar)

---

## 三、质量统计

| 指标 | 数值 |
|------|:--:|
| 版本诱错审计 | 1 轮 5 维度 (v2.3.0 专项) |
| S0 发现 | 0 |
| P1 发现/修复 | 2/2 |
| P2 发现/修复 | 4/2 |
| P3 发现 | 3 |
| 全量 P1 累积修复 | 31/76 |
| 全量 P2 累积修复 | 17/154 |

### 门禁基线

```bash
cargo check        ✅ PASS (3 warnings)
vite build         ✅ PASS (2.99s)
```

---

## 四、新增/修改文件

| 文件 | 用途 |
|------|------|
| `src/error_codes.rs` | 41 个 API 错误码常量 |
| `src/api_errors.rs` | `json_bad_request_with_code()` 接线 |
| `frontend/src/utils/errorMessages.js` | 前端错误码→本地化文本映射 |
| `src/auth/mod.rs` | JWT refresh 端点 + login_user_by_id |
| `nginx.conf` | TLS 反向代理配置 |
| `Dockerfile` | HEALTHCHECK + curl |
| `src/frontend_api_types.rs` | ApiErrorResponse.error_code 字段 |
| `src/main.rs` | ApiErrorResponse + error_codes 模块注册 |
| `qrpc_runtime/src/lib.rs` | 杠杆 floor 统一 + 后台任务注释 |
| `qrpc_runtime/src/fill_engine.rs` | processed_results 1000条上限 |
| `qrpc_runtime/src/risk_checker.rs` | NaN clamp guard |
| `qrpc_runtime/src/backtest_metrics.rs` | 移除仅手续费伪 PnL |
| `frontend/src/components/TopToolbar.jsx` | 停止运行时 confirm() |
| `frontend/src/pages/ChaosPage.jsx` | 混沌实验 confirm() |

---

## 五、决策纪录

### 决策: 错误消息编码方案
**选择**: B — 混合方案 (API 返回 error_code + 中文 message 向下兼容)
**理由**: 不破坏 CLI/脚本解析中文消息的现有消费者

### 决策: TLS 方案
**选择**: B — nginx sidecar (Docker 反向代理)
**理由**: 生产级 TLS 终止是运维关注点, 不应与应用耦合

---

## 六、延入 v2.3.1

| 类别 | 项数 | 焦点 |
|------|:--:|------|
| 架构重构 | 5 | risk_checker 拆分/runtime_api.rs 按域拆分/data_fetch_counts优化 |
| 安全 | 3 | RiskMonitor 接线/jsonwebtoken v10/TLS 应用层 |
| 前端 | 5 | AssetCandlesPanel i18n/StrategyCodePanel i18n/E2E 条件等待 |
| 序列化 | 4 | tagged enum Unknown 变体/schema_version 补充/request_json 去重 |
| 配置 | 3 | Docker 非 root/NSIS 权限/Tauri 跨平台脚本 |
| **合计** | **20** | |
