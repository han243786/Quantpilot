# General_Policy 合规矩阵 v2.0.0

> 刷新日期: 2026-05-17 | 基准: General_Policy.md §1-§8 | 审计: 5维度自由维度诱错 (72项发现) + v2.0.0 MAJOR开发

| 条款 | 状态 | 验证 |
|------|:--:|------|
| §1.1 QS 唯一路径 | ✅ | 编译链 graph→QS→CoreIR→Runtime 唯一 |
| §1.2 跨三层验证 | ✅ | 18 指标全对应 |
| §1.3 编译不可绕过 | ✅ | 无绕过路径 |
| §1.4 数据流单向 | ✅ | source_mode 保护 |
| §2.1 错误全中文 | ✅ | v2.0.0 新增 auth/live_execution 错误全中文 |
| §2.2 测试断言中文 | ✅ | 前端269测试全中文断言 |
| §2.3 indicator 测试 | ✅ | 21 test |
| §2.5 前端 t() | ✅ | v2.0.0 全量包裹完成 (TopToolbar/BacktestComparePage/StrategyWorkspacePage等) |
| §3.1 文档分层 | ✅ | v2.0.0 设计文档+优化清单+closeout |
| §3.2 文档全中文 | ✅ | 全部文档中文 |
| §3.3 里程碑命名 | ✅ | v2.0.0 01/02/03 编号 |
| §4.1 capability 变更 | ✅ | live_execution 模块已声明 |
| §4.2 错误变更测试修复 | ✅ | TOCTOU修复 + 测试期望值更新 |
| §5.1 禁止硬编码 | ✅ | MARKET_PUBLIC_KEY → env var |
| §5.2 禁止静默忽略 | ✅ | RNG失败→panic; 7处`let _=`→错误日志; 原子写入错误传播 |
| §5.3 禁止 stub | ✅ | live_execution 2处stub已文档化(GP§5.3显式注释) |
| §5.4 禁止绕过 QS | ✅ | gen_screenshots cfg门控 |
| §5.5 端到端验证 | ✅ | cargo check + clippy + npm build + npm test 全量通过 |
| §7.1 三级分类 | ✅ | 14目录全分类 |
| §7.2 存储配额 | ✅ | ensure_storage_quota 生效 |
| §7.3 启动清理 | ✅ | startup_storage_cleanup + assert_market_public_key_is_production |
| §7.4 写入声明生命周期 | ✅ | 7处新增tmp+rename原子写入 (collaboration/runtime_api/state/storage_lifecycle/credential_vault/data_module) |
| §7.5 DEV 激进清理 | ✅ | Transient全部删除 |
| §8.1 配色 | ✅ | 0违规色值 |
| §8.2 圆角 | ✅ | ≤6px |
| §8.3 背景 | ✅ | 纯色 #0d0d0d |
| §8.4 导航 | ✅ | 左侧侧边栏 |
| §8.5 图标 | ✅ | SVG |
| §8.6 组件令牌 | ✅ | --ad-* |
| §8.7 组件检查单 | ✅ | data-testid 完整 |

**合规率: 28/28 ✅** (v2.0.0 无新增违规)

---

### v4.8.0 GP 矩阵增补: provider 切面分层

| 条款 | v4.8.0 决策 | 验证 |
|------|-------------|------|
| §5.1 禁止硬编码/边界漂移 | v4 只允许 OKX 单一 provider profile; 非 OKX provider、多资产 provider 适配不得提前进入 v4 | `rg "market\\.okx|okx-paper|binance|alpaca|ibkr|futu"` 结合代码审查 |
| §5.2 禁止静默忽略 | 不支持 provider 必须 fail-closed, 不得 fallback 到 generic provider | provider mode 校验、OpenAPI 423/502 响应 |
| §5.5 端到端验证 | W0-2 必须证明 OKX 模拟盘 submit/query/cancel 回执路径和审计摘要 | `cargo test --bin executor okx` + 手动 demo 凭证 smoke |
| §7.1 三级分类 | v5 多 provider/多资产/全双工 WS 覆盖归类为 MAJOR provider program | v5 规划单独登记 |
| §8.x UI 文案 | UI、日志和文档只能写"OKX 模拟盘 / 非真实资金", 不得写成真实实盘 | `tools/check-user-facing-text.ps1` + 人审 |

v4.8.0 通过标准: OKX 模拟盘 provider 回执路径可用; 美股、港股、A股、贵金属、大宗商品、期货、期权等 provider 扩张全部延后到 v5。若 provider 不支持 WebSocket 全双工或等效实时订单/行情事件回执, 视为关键基础设施缺失, 不进入适配支持。

---

### 五轮自由维度诱错审计 (v1.1.2~v1.1.14)

| 轮次 | 维度 | 发现 | S0修复 | P1修复 |
|------|------|:--:|:--:|:--:|
| 第1轮 | A~E (代码安全) | 92 | 18 | 16 |
| 第2轮 | F~J (上层质量) | 88 | 7 | 10 |
| 第3轮 | K~T (纵深安全) | 204 | 10 | 13 |
| 第4轮 | U~Y (领域正确性) | 86 | 6 | 5 |
| 第5轮 | Z~AD (系统完整性) | 100 | 3 | 4 |
| **合计** | **25维度** | **570** | **44** | **48** |

**修复率**: S0 44/44 (100%) | P1 48/120+ (40%)

---

### v1.2.0 架构级优化

| 目标 | 状态 | 验证 |
|------|:--:|------|
| KlineProvider O(N²)→O(N)滑动窗口 | ✅ | `cargo check` |
| RiskMonitor独立风控组件 | ✅ | 9/9单元测试 |
| 编译链graph→QS完整配置 | ✅ | `npm test` 92/92 |
| PBKDF2凭证密钥迁移 | ✅ | `cargo check` |
| main.rs部分拆分 | ✅ | `state.rs`+`middleware.rs`提取 |

---

### 测试覆盖趋势

| 指标 | v1.0.0 | v1.0.2 | v1.1.2 |
|------|:--:|:--:|:--:|
| Rust #[test] | ~270 | 291 | 291+ |
| 前端 .test.* 文件 | 86 | 86 | 92 |
| 前端测试用例 | - | - | 269 |
| QS 场景文件 | 23 | 19 | 27 |
| 门禁脚本 | 7 | 7 | 7 |
| 元流水线脚本 | 1 | 1 | 1 |
| E2E 测试 | - | - | 17 |

### 安全加固 (v1.1.2 新增)

| 加固项 | 覆盖 |
|------|------|
| 路径遍历防护 | discard(2) + reveal(1) — 3端点 |
| 原子写入 | 5文件 9处调用全部 tmp+rename |
| 快照签名完整性 | from_sequence + to_sequence |
| 审批竞态修复 | approve/reject/claim 写锁保护 |
| 后台任务防吞 | JoinHandle 监视 + await |
