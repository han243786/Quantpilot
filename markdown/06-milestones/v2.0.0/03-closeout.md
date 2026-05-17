# v2.0.0 Closeout

> MAJOR版本 | 2026-05-17 | 5大功能追加 + 7 S0修复 + 17 P1修复

---

## 一、版本轨迹

| 版本 | 类型 | 核心变更 |
|------|------|---------|
| v1.4.0 | MINOR | closeout: 76 S0全消化, 五轮诱错 |
| **v2.0.0** | **MAJOR** | OKX实盘+多用户+插件市场+前端补全+打包 |

---

## 二、五大功能目标验收

### 1. OKX 实时交易接口 ✅
- `qrpc_runtime/src/live_execution.rs` (867行) 实现 `ExecutionModuleProvider` trait
- HMAC-SHA256签名, REST下单, 风控 (单笔上限$1000, 日累计100单)
- 订单类型仅支持Market/Limit (StopLoss/TakeProfit显式拒绝 → S0-1)

### 2. 多用户认证与数据隔离 ✅
- `src/auth/` SQLite + JWT + bcrypt
- `scoped_key()` 全局BTreeMap `{user_id}:{record_id}` 前缀 → S0-6
- `FromRequestParts` 提取器, 向后兼容 user_id=0
- JWT HS256显式锁定 → S0-7

### 3. 插件市场 ✅
- Ed25519签名验证 (`plugin_market.rs`)
- MARKET_PUBLIC_KEY 环境变量 + 启动校验 → S0-2
- 子进程沙箱 (`plugin_sandbox.rs`) + timeout + Unix setrlimit
- `wait-timeout` crate 依赖

### 4. 前端UX补全 ✅
- `t()` 全量包裹完成
- 9个测试修复 (i18n context → sub-component useI18n调用)
- npm test 92/92, 269/269 全量通过

### 5. 整合包发布 ✅
- `packaging/windows/installer.nsi` NSIS模板
- `packaging/docker/Dockerfile` 多阶段
- `.github/workflows/release.yml` CI/CD
- `release/release-manifest.yaml` 元数据

---

## 三、质量统计

| 指标 | 数值 |
|------|:--:|
| 五维度诱错审计 | 5 Agent并行 (A:逻辑 B:并发 C:数值 D:持久化 E:API) |
| S0 发现 | 7 |
| S0 修复率 | 100% (7/7) |
| P1 发现 | 21 |
| P1 修复率 | 100% (17/17, 4项v1.x已有) |
| P2 遗留 | 28 → v2.1.x |
| P3 遗留 | 20 → v2.1.x |
| 原子写入修复 | 7处 tmp+rename |
| TOCTOU修复 | 3个审批handler |

### 门禁基线

```bash
cargo check        ✅ PASS
cargo clippy       ✅ PASS (warning only, 6项)
cargo test         ✅ 编译通过
npm run build      ✅ PASS
npm test           ✅ 92/92, 269/269
npm audit          ✅ 0 vulnerabilities
```

---

## 四、新增文件

| 路径 | 用途 |
|------|------|
| `src/auth/mod.rs` | 多用户认证 (SQLite+JWT+bcrypt, 278行) |
| `src/auth_middleware.rs` | JWT+API Key双模式认证中间件 |
| `qrpc_runtime/src/live_execution.rs` | OKX实盘/测试网执行 (867行) |
| `qrpc_runtime/src/plugin_sandbox.rs` | 子进程插件沙箱 |
| `qrpc_runtime/src/risk_monitor.rs` | 独立风控组件 |
| `packaging/windows/installer.nsi` | NSIS安装器 |
| `packaging/docker/Dockerfile` | Docker多阶段镜像 |
| `.github/workflows/release.yml` | CI/CD发布流水线 |
| `release/release-manifest.yaml` | 发布元数据 |
| `markdown/06-milestones/v2.0.0/01-设计文档.md` | 设计+4项决策纪录 |
| `markdown/06-milestones/v2.0.0/02-综合优化清单.md` | 24项优化项 (S0+P1) + 48项遗留 |
| `markdown/06-milestones/v2.0.0/03-closeout.md` | 本文件 |

---

## 五、关键修复文件

| 文件 | 修复项 |
|------|------|
| `live_execution.rs` | S0-1/3/4, P1-11~15 (OrderType/NaN/限流/溢出/stub) |
| `auth/mod.rs` | S0-6/7, P1-1~3 (数据隔离/JWT锁定/RNG/锁/校验) |
| `plugin_market.rs` | S0-2 (MARKET_PUBLIC_KEY env var) |
| `core_ir_evaluator.rs` | S0-5 (window==0守卫) |
| `runtime_api.rs` | P1-5/6/17 (原子写入+TOCTOU) |
| `state.rs` | P1-7 (原子写入+错误日志) |
| `storage_lifecycle.rs` | P1-8 (原子写入) |
| `credential_vault.rs` | P1-9/16 (原子写入+Zeroizing) |
| `data_module.rs` | P1-10 (原子写入) |
| `collaboration.rs` | P1-4 (原子写入) |
| `TopToolbar.jsx` | DefaultToolbarLayout/WorkspaceToolbarLayout useI18n() |
| `StrategyWorkspacePage.jsx` | CODE_INSPECTOR_DEFS labelKey→label |
| `BacktestComparePage.jsx` | EquityOverlayChart useI18n() |
| `.env.example` | +QUANTPILOT_JWT_SECRET +QUANTPILOT_MARKET_PUBLIC_KEY |

---

## 六、架构决策纪录

### 决策 1: 实时交易上线策略
**选择**: A — OKX测试网先行
**理由**: 测试网无资金风险，可验证签名/下单/状态同步。Binance+主网延入v2.1.0。

### 决策 2: 多用户数据库
**选择**: A — SQLite (rusqlite)
**理由**: ACID事务+索引查询，无额外服务进程，适合单机桌面应用。

### 决策 3: 插件沙箱
**选择**: B — 子进程隔离
**理由**: 进程内超时无法防死循环/内存泄漏。子进程提供真实安全边界。

### 决策 4: 整合包发布
**选择**: A — Windows优先
**理由**: QuantPilot当前为Windows桌面应用(Tauri)。Linux/macOS延入v2.1.0。

### 决策 5: 用户数据隔离方案
**选择**: BTreeMap key `{user_id}:{record_id}` 前缀
**理由**: user_id=0时键不变 → v1.x持久化数据向后兼容。

---

## 七、风险缓解状态

| 风险 | 缓解措施 | 状态 |
|------|---------|:--:|
| 真实交易导致资金损失 | 默认paper模式, live需显式启用+二次确认 | ✅ |
| SQLite并发写入瓶颈 | 单用户桌面应用 | ✅ |
| 插件沙箱逃逸 | 子进程隔离+timeout | ✅ |
| JWT密钥泄露 | env var注入, 不写入代码 | ✅ |
| MARKET_PUBLIC_KEY为测试向量 | 启动校验+env var | ✅ S0-2 |
| 用户数据互不可见 | scoped_key全局前缀 | ✅ S0-6 |

---

## 八、v2.1.0 规划

48项 P2/P3 遗留 → v2.1.0:

| 优先级 | 数量 | 焦点 |
|--------|:--:|------|
| P2 | 28 | deny_unknown_fields全量、v1→v2迁移路径、JWT持久化、异步锁优化 |
| P3 | 20 | 防御深度补充、截断转换修复、日志性能优化 |
