# Changelog

## v3.7.1 — 回归修复 + 流程收口基线 (2026-05-22)
- S0 登录挂起修复: 缓存 `ring::rand::SystemRandom`, refresh token 生成移出 DB 锁范围。
- P1 凭证 DELETE 405 修复: Axum 0.7 路由参数改为 `:service`。
- P2 测试文件锁收口: 新增 `scripts/test.ps1` / `scripts/test.sh`, 测试前停止本仓库运行进程。
- 门禁收口: pre-commit / CI / `tools\run-closeout-gates.bat` 统一接入 UTF-8、版本一致性、executor warning 预算和 QS 场景 smoke。
- 发布流水线修正: Windows release workflow 改为 PowerShell 打包路径, 支持手动 dry-run 构建。
- 已知状态: executor warning 债务预算为 49；完整 17 项 closeout 门禁仍需发布前跑通。

## v3.3.0 — 全量消化 (2026-05-20)
- session_crypto: 空payload + 损坏密文 2 tests
- executor_state: RingBuffer capacity=1 + empty latest 2 tests
- credential_vault encrypt_vault Vec::with_capacity 预分配

## v3.2.2 — 健壮性收敛 (2026-05-20)
- stop_strategy 从RunnerPool移除runner
- start_strategy 幂等保护
- credential_vault .bak崩溃恢复
- KlinePool MAX_SYMBOLS=100 + LRU淘汰
- KLINE_POOL_CAPACITY 命名常量

## v3.2.1 — 紧急修复 (2026-05-20)
- ExecutorApp error状态自动恢复
- 版本号统一: package.json(2.0→3.2), src-tauri(2.0→3.2), README(v2.3→v3.2)
- 前端catch{} → console.warn (4处)
- StrategyGraphPanel EventSource.onerror
- main.jsx unhandledrejection处理

## v3.2.0 — 性能达标 (2026-05-20)
- kline_buffers/pending_params BTreeMap→HashMap
- KlineChart + ExecutorTopBar React.memo
- 执行端响应式 CSS (768px/560px)
- live_runner trigger detection 3 tests

## v3.1.0 — 审计闭环 (2026-05-20)
- StrategyPackage/MigrationPackage/VaultData deny_unknown_fields
- 版本号 2.5.0→3.1.0

## v3.0.2
- MAX_STRATEGIES=50 资源上限
- HTTP状态码: 签名→401, 锁定→423
- 中文化: salt/nonce/密钥→全中文
- migration_api QS溯源验证

### v3.0.1
- credential_vault_v2 .bak备份+回滚
- audit_log Mutex守卫修复 + fsync
- RingBuffer Vec→VecDeque O(1)
- ws_client saturating_mul防溢出
- kline_buffer NaN/Inf过滤
- recent_bars 单次迭代优化
- ExecutorApp catch→error state

## v3.0.0 — 实时执行端 (2026-05-20)
- 独立进程 Axum :3001
- WebSocket 直连交易所
- lightweight-charts K线引擎
- 策略迁移 (Core IR + graph JSON + 签名)
- OKX Paper 模拟盘验证
- 多策略标签页并发
- 热调参 API
- 进程间加密 (AES-256-GCM + HMAC)
- 凭证保险库 v2 (PBKDF2 1M轮)
- 审计日志
- GP v3.0 + 超级规范化 v3.0

## v2.5.0 — P2/P3 消化 (进行中, 2026-05-20)

### 序列化安全
- PortfolioState: deny_unknown_fields + debug_assert_invariants 跨字段一致性检查
- snapshot_service: 抽取 build_signature_input 共享函数, 消除创建/验证两侧代码重复

### 安全加固
- 凭证脱敏阈值 8→4, 防止短 API key 日志明文泄漏
- 存储错误消息去路径化 (仅显示目录名)
- safe_log to_lowercase() 安全性注释

### 健壮性
- merge.rs: truncate 前按 net_strength 降序排序, 保留最强信号
- 启动流程: 13个存储目录并行创建 (tokio::spawn)
- 后台任务 BTreeMap 淘汰逻辑标注
- #[ignore] 测试添加文档注释
- RateLimiter O(n) retain 标注后台清理改进计划

### 代码质量
- chaos_experiment: 魔法数值 10000→DEFAULT_CHAOS_MAX_DURATION_MS
- core_ir_evaluator: 清理无用 `let _ = window`
- frontend: 全局 unhandledrejection 处理
- graphStorePersistenceHelpers: schema 版本比较注释

### 规范文档
- General_Policy v3.0: 33条 🛡️/🔍 两层分层
- 超级规范化 v3.0: 精简三层流水线 + §7.5 元流水线自进化触发条件

## v2.4.0 — 全量 P1 消化 (2026-05-20)

### 代码安全
- 空头策略敞口计算修复 (净值 = long - short, 替代 abs() 求和)
- 回测权益曲线 NaN 防护 (is_finite 检查)
- Mock 波动率 clamp(1e-6, 1.0) 防极端值注入
- 硬编码默认值提取为具名常量 (DEFAULT_SPREAD_TRIGGER_BPS)
- LazyLock<Runtime> 全局单例替代重复创建线程池
- 沙箱验证 catch_unwind + 3次退避重试
- RateLimiter max_rps=0 拒绝服务防护 (.max(1))
- Auth 限流器代理检测 (X-Forwarded-For)

### 编译链
- 运行时目标反序列化失败改为 warn + default
- Formal QS 编译降级/5xx 回退添加 console.warn
- 编译 JoinError 提取 panic payload 日志
- 前后端 ID sanitize 统一: 后端为权威来源, 前端版本标记为近似
- normalize 错误添加 QPQSLOW000 分类

### 前端性能
- StrategyCanvas React.memo 包裹 + 节点上限 1000
- QS 编辑器 maxLength=50000
- 事件搜索 200ms 防抖
- SMA 滑动窗口 O(N) 替代 O(N*period)
- signal_kind HashMap 预索引 O(1) 替代 O(N*M)

### API/安全
- CSP base-uri 'self'
- Permissions-Policy 头
- CORS 拒绝通配符 origin
- JWT 失败不降级 API Key
- graph_version_dir 纵深防御 sanitize

### 交互体验
- 编译/模拟/回测按钮加载状态 (isBusy + 文字反馈)
- 技术术语替换 (Strategy IR → 自然语言)
- 通知延长至 5s (错误持久)
- beforeunload i18n 双语提示

### 测试
- api_errors: 7 个单元测试
- storage_lifecycle: 4 个单元测试 (TTL/Lifecycle)

## v2.3.3 — S0 阻断修复 + P1 关键收敛 (2026-05-20)

### S0 阻断 (9/9)
- 嵌套写锁拆分: approval_records → ai_proposals 死锁修复
- JWT 失败直接 401, 移除 API Key 降级
- CORS 拒绝通配符和非 http(s) scheme
- graph_version_dir 纵深防御 sanitize
- 未知意图模块键 → anyhow::bail! 明确报错
- 前端未知意图 → throw Error 替代静默降级 ma_cross
- RFC-017/018/019 状态同步 ✅→🔄
- API 参考文档更新废弃声明
- 支持矩阵死链接修复

### P1 (15/15)
- 11 处原子写入统一 fsync (atomic_write_json pub(crate))
- BTreeMap 无界增长: 淘汰逻辑扩展至 11 个 map
- CSS: radial-gradient 替换, border-radius:12px→var(--ad-radius-lg)
- 按钮加载状态, 技术术语替换
- 测试 re-export + pub mod 修复

## v2.3.x — 错误国际化与架构优化 (2026-05-18)

### v2.3.2 — 架构优化
- **ExecutionModuleProvider ISP 拆分**: 1 trait → 2 (ExecutionPlanner + ExecutionSubmitter)
- **risk_checker 完整重构**: `evaluate_risk_decision` 322行→154行, 3个提取函数
- **runtime_api.rs 按域拆分**: 3755行→4文件 (mod/backtest/run/mutation), `src/runtime/` 模块目录
- jsonwebtoken v9→v10 升级
- 基准文档化 (compute_benchmark_equity 假设声明)
- start.bat 版本号+可靠端口检测修复

### v2.3.1 — 部署安全
- Docker 非 root 用户运行 (useradd quantpilot)
- NSIS 安装器权限降级 (admin→user, PROGRAMFILES→LOCALAPPDATA)
- StrategyHubHeroSection 搜索 maxLength=200
- Docker HEALTHCHECK + curl

### v2.3.0 — 错误国际化 + TLS + JWT刷新
- `src/error_codes.rs` — 41个语言中立错误码
- `src/api_errors.rs` — `json_bad_request_with_code()` 向后兼容接线
- `frontend/src/utils/errorMessages.js` — 30+ en/zh 本地化映射
- `POST /api/auth/refresh` — JWT 令牌刷新端点
- `nginx.conf` — TLS 反向代理 (443→3000)
- P1×3 + P2×5 审计修复

---

## v2.2.x — 架构重构与质量根基 (2026-05-17)

### v2.2.0 — 架构根基重构
- **QuantPilotError**: 新建 `qrpc_core/src/error.rs` — 7变体类型化错误枚举 (Validation/Runtime/Io/Serialization/Plugin/Capability/Internal)
- **RuntimeCoordinator 拆分**: 22字段 god object → 10字段 + 3子结构体
  - `RuntimeState` (portfolio/fetch_counts/timestamps)
  - `ConfigTracker` (deployment revisions/generation/history)
  - `MergeCoordinator` (engine/policy/records)
- **4 新文件**: error.rs, runtime_state.rs, config_tracker.rs, merge_coordinator.rs

### v2.2.1 — i18n完整化 + 可观测性 + 安全
- **en-US 全量审核**: 318处 `// TODO: translate` → 清零
- **前端状态标签 i18n**: `runtimeStatus.js` 硬编码中文 → `translateText()`
- **日期格式化**: AssetCandlesPanel/DrawdownChart 硬编码 `"zh-CN"` → `getGlobalLocale()`
- **tracing 引入**: tracing + tracing-subscriber, 支持 JSON/compact 双格式, QUANTPILOT_LOG_FORMAT 配置
- **登录速率限制**: 5次/分钟/IP 独立限流 (auth_rate_limit_middleware)
- **健康检查丰富化**: /api/health 返回组件级状态 (credential_vault/auth_db/storage/active_alerts)

### 累计改动
- **7 新文件**, **15+ 修改文件**
- `cargo check` ✅ | `vite build` ✅

---

## v2.1.x — P1/P2/P3 全量清零与系统加固 (2026-05-17)

97 项审计发现全部消化清零。v2.1.0→v2.1.3 四个 PATCH 版本，跨越安全、数据完整性、金融逻辑、插件系统、持久化、前端、运维七大领域。

### v2.1.0 — P1 全量消化 (49/49)

**安全认证 (7)**: ConnectInfo 注入验证、用户名泄露修复、路径遍历防护、reveal 边界检查、deny_unknown_fields (24 structs)、API key 脱敏、CORS 收紧

**数据完整性 (10)**: AI 提案状态机校验、实验创建持久化、credential_vault .bak 崩溃恢复、graph .bak 恢复、approval_id 原子计数器、warm_persisted_state scoped_key、schema_version 兼容检查、transient_backtest_store_dir 纳入 storage

**金融逻辑 (6)**: 挂单手续费使用 assumptions.taker_fee_bps、成交后 mark-to-market 更新、跨 Agent 方向冲突检测、NaN 数量告警、权重除零 guard、OKX 时间戳毫秒精度

**插件系统 (7)**: entrypoint 校验、unregister 机制、PluginRegistry::remove()、version_req 格式校验、Ed25519 签名验证 (scan_atoms)、setrlimit 返回值检查

**持久化 (5)**: reqwest Client 复用、merge_records/config_history 上限、TTL 清理频率提升 (60min→10min)、alert_firings 清理

**前端 (7)**: App.jsx/BacktestComparePage/LeftSidebar/TopToolbar t() 全量包裹、SnapshotsPage 请求体修复、AssetCandlesPanel key 修复、BacktestDetailPage useMemo

**运维 (7)**: 优雅关闭超时 (30s)、日志级别配置 (QUANTPILOT_LOG_LEVEL)、QUANTPILOT_BIND_ADDR 文档化、OrderSide Display impl

### v2.1.1 — P2 精选消化 (7/10)

JWT 密钥持久化 (storage/.jwt_secret)、OKX 错误脱敏、编译信号量全 handler 覆盖、agent_module 魔法数字提取 (4 常量)、merge.rs concentration_limit 截断修复、setrlimit 错误处理

### v2.1.2 — v2.0.0 P2 高危消化 (6/6)

卖空无持仓检查、fee.abs()→fee.max(0.0)、enforce_max_* Some(0) 校验、DEV 模式错误路径泄露修复

### v2.1.3 — 重构收尾 + P3 清零

**新模块**: CircuitBreaker (三态断路器, 8 tests)、Backup (每日自动备份)、Sandbox checkpoint/restore

**重构**: spawn_blocking 编译、frontend_runtime_mapping.rs 死代码清理 (1025→575 行)、state.rs 孤立文件删除 (900+ 行)、safe_log Mutex→RwLock

**P3 清零 (20)**: NaN 守卫 51 处 is_finite()、gap_count u32→u64、MOCK_VOLATILITY NaN 注入防护、PluginManifest deny_unknown_fields、rate_limiter bucket 阈值优化、backtest_metrics 溢出防护、credential_api 空字节检查、PORT fallback 错误改进

### 累计改动
- **30+ 文件**修改/新建
- **3 新模块**: circuit_breaker, backup, Sandbox checkpoint/restore
- **1350+ 行死代码**删除
- **51 处 NaN 防御**
- **28 structs** deny_unknown_fields
- **cargo check** 始终通过

---

## v2.0.0 — OKX 实盘 + 多用户 + 插件市场 + 前端补全 + 打包 (2026-05-17)

MAJOR 版本。五大功能追加 + 7 S0 修复 + 17 P1 修复。

### 功能
- OKX 实时交易接口 (live_execution.rs 867 行)
- 多用户认证与数据隔离 (SQLite + JWT + bcrypt)
- 插件市场 (Ed25519 签名 + 子进程沙箱)
- 前端 UX 补全 (t() 全量包裹)
- 整合包发布 (NSIS + Docker + CI/CD)

### 质量
- S0: 7/7 ✅ | P1: 17/17 ✅ | P2: 28→v2.1.x | P3: 20→v2.1.x
- 原子写入修复 7 处 tmp+rename
- TOCTOU 修复 3 个审批 handler

---

## v1.1.0 — 研究级回测与多标的策略支持

将回测系统从基础回放升级为业界标准研究级回测，新增 16 项功能。S0+P1+P2 全部完成，GP 合规 30/32，加权评分 9.5/10。

### S0 阻断级 (6/6)

- **BacktestSummary 分组子结构体**: risk_adjusted (夏普/索提诺/卡尔玛/VaR/CVaR)、trade_analysis (盈亏比/连胜连亏)、drawdown_analysis (回撤天数)、benchmark_comparison (Alpha/Beta/IR)
- **成交模型三层升级**: SlippageModel (固定/波动率缩放/盘口深度) + MarketImpactModel (平方根) + bid/ask 真实价差 + LatencyModel (固定/正态/按交易所)
- **统一时间轴回放引擎**: UnifiedTimeline 替代 ReplayDataModule，合并所有数据源时间戳，asof-join 对齐，支持多标的 + 多频率
- **多标的信号路由**: WeightedSignals 按 (Exchange, Symbol) 分组独立决策，消除 ETH 信号被静默吞掉的问题
- **基准权益曲线**: 等权重买入持有基准自动生成，月度收益率热力图
- **跨标的联合风控**: Phase 2 检查 `max_cross_symbol_leverage`，超标按比例缩减

### P1 高优 (6/6)

- 买卖价差集成: FillEngine 使用 real bid/ask，从波动率估算价差（非 OHLC 极值）
- 波动率缩放滑点 + 市场冲击模型: 接入 Sandbox → Coordinator → FillEngine 全链路
- 前端 5 指标 Hero 卡 + 指标对比表 + 夏普颜色编码
- 回撤面积图 (DrawdownChart) + 基准叠加线 (灰色虚线)
- 跨标的联合风控 Phase 2
- 多频率数据重采样: ResampleKlineProvider (1h→4h/1d OHLC 聚合)

### P2 加分 (4/4)

- VaR/CVaR (95% 历史模拟法) + 偏度/峰度
- 月度收益率热力图 (纯 CSS Grid 着色)
- `period_returns` 字段 + 回测周期分解
- OpenAPI/文档同步

### 技术债清偿

- clippy -D warnings 通过 (修复 collapsible_if / 移除死代码 / `args.get(0)`→`args.first()`)
- Gate 3 能力治理快照生成
- `summarize_equity_curve` 移除冗余 `trade_ledger` 参数
- 旧回测数据加载向后兼容 (serde default)

### 文件统计

- 新增 8 文件 (slippage, backtest_metrics, sandbox/{mod,timeline,replay}, DrawdownChart, MonthlyReturnsHeatmap, 里程碑文档)
- 修改 18 文件 (跨 3 Rust crate + 6 前端组件)
- 测试: 前端 269/269, E2E 17/17, Rust lib 136/138 (2 ignore)

## v1.0.7 — 体验收口与国际化补齐

v1.0.6 延期 9 项收口 + en-US 补齐 + CSS 质量收口。纯质量里程碑，无新功能。

## v1.0.6 — 用户困惑点全量优化

基于四维度全量审计（79 项发现），系统性消除用户困惑点。S0/P1 全部闭环，完成率 89%。

### S0 阻断级 (13/13)

- 错误消息全中文化：runtime_validation (12 类)、compile_diagnostics (8 处)、sandbox_verification (3 处)、auth_middleware
- API 格式统一：compile_api 裸字符串→JSON、auth RFC 9457→标准 JSON、api_errors 保留错误详情
- 诊断修复：QS0609 代码冲突→QS0613、QS0502 eprintln→中文消息、8 个诊断 None span→Span::expr()
- App.jsx TDZ 崩溃修复：route 声明移到 routeRef 之前
- start.bat 完整修复：移除 cargo lock 冲突、QUANTPILOT_DEV=true、端口检测
- graphStore capability 消息：英文→中文
- runtime_diagnostics：ms→毫秒、Yes/No→是/否、RuntimeWarning/Error→运行告警/运行错误

### P1 高优 (22/22)

- QS 编译器术语去技术化："形式化 QuantScript"→"QuantScript"、"下层转换"→"编译"、"可执行主干"→"strategy() 函数体"
- 错误消息指引：auth 告知环境变量名、QPQSLOW 消息列出有效函数/值、QS0603 给出替代方案
- QPQSLOW999 不再被过滤，附带内部错误提示
- 分页 total 保留：fetchJson/postJson/deleteJson 移除自动解包，新增 unwrapPage()
- 客户端错误截断 200→2000 字符
- loading 阶段动态文本（"正在连接后端..."等），超时 8s→5s
- 10 条告警规则行动描述全部具体化
- Runbook 12 个英文术语添加中文注释
- 9 个诊断 span 补充（1 个 Span::binding + 8 个 Span::expr）
- QS0404 错误消息 QS0505 消息优化
- 排序稳定性：4 处 sort 添加 graph_id tie-breaker
- TutorialOverlay 目标元素缺失检查
- App.jsx 离线横幅 t() 包裹、存储配额添加导航按钮、beforeunload 自定义消息、Tauri 桌面隐藏离线横幅

### P2 中优 (17/24)

- 速率限制消息含阈值 + 死链接修复
- 沙箱验证 partial fidelity 不再抑制其他警告
- CLI 帮助全面中文化 + credential 子命令
- runbook TTL/debug/compatibility 等术语中文注释
- errorText.js 标点标准化匹配
- router 无效 URL console.warn + 反抖日志
- i18n 缺失翻译 dev 模式 console.warn
- start.bat taskkill 改为端口精确清理
- main.rs 窗口 label 健壮化
- 治理面板 7 个字段添加 tooltip
- ApiErrorResponse 统一结构体

### P3 低优 (18/20)

- CSS 令牌化：~75 处 font-size/border-radius 硬编码→设计令牌
- 内联样式迁移：RunbookPage(7)、ChaosPage(6)、SnapshotsPage(4)、AlertsPage(3) → CSS 类
- en-US.js Unicode 转义→实际字符
- recoverLatestRunnableGraph 重试提示
- Tauri API 失败 dev 模式日志

### 延期至 v1.0.7
- en-US 翻译补齐（~250 键）
- CSP 生产加固
- start.bat → PowerShell 迁移
- actionFailure 结构化返回
- enum description() 方法

---


## v1.0.5

前端样式深度修复。P0 阻断级 CSS 语法错误修复。

### P0 修复
- styles.css 多行选择器逗号补充 (~30+ 处): .primary-btn/.ghost-btn/.danger-btn/.status-pill/.toolbar-notice 等分组选择器从后代选择器修正为正确的并列选择器
- design-system.css 补齐 --danger/--warning/--success 兼容别名 (→ var(--ad-error/warning/success))
- styles.css 修复 21 处 background 双值无效语法 (background: var(--ad-panel); rgba(...) → 合并为单一声明)
- styles.css 修复 background-image 空值 (minimap-grid 补径向渐变点阵)
- CSS 构建 warning 从 23 个降至 2 个

### 策略中心排版
- 状态条 8 列单行 → 4 列双行布局
- Hero 区域增加底部边框分隔线
- CTA 按钮内联样式迁移为 CSS 类
- Hero 操作按钮扁平化去嵌套

### P2 令牌扩展
- 新增 --ad-space-2-5(10px) --ad-space-3-5(14px) --ad-space-4-5(18px) 间距令牌
- 新增 --ad-border-rgb 变量, 全局替换 85 处 rgba(116,145,182,*) 硬编码
- 新增 --ad-chart-line-a / --ad-chart-line-b 图表色令牌
- 5 个页面 --tv-* 变量统一为 --ad-* (15 处)

### P1 内联样式迁移 (部分完成)
- QuantScriptEditor.jsx: 内联样式 → 12 个 CSS 类 (~80% 减少)
- StrategyWorkspaceSourceTab.jsx: 内联样式 → 3+ 复用 CSS 类 (~85% 减少)
- StrategyWorkspaceDebugTab.jsx: 内联样式 → 8 个 CSS 类 (~75% 减少)
- StrategyWorkspacePage.jsx: 7 处重复骨架屏样式合并为 .tab-skeleton

### 全量诱错审计
- 3 个 agent 并行审计 30 个维度 (数据/视觉/交互)
- 发现 2 S0 + 8 P1 + 8 P2 = 18 项
- 报告: markdown/06-milestones/v1.0.5/02-全量诱错审计报告.md

### S0 修复 (2/2)
- 3 处破坏性操作添加 confirm 确认对话框 (resetGraph/resetRuntime/deleteCredential)
- beforeunload 事件处理: 策略工作区/QS 编辑器页面关闭时提示未保存更改

### P1 修复 (8/8)
- saveGraph 添加 API 失败回滚: 乐观更新后在 /graphs/save 失败时恢复之前的状态快照
- --ad-text-muted 对比度修复: #909090→#9e9e9e (WCAG AA 4.6:1)
- prefers-reduced-motion 全覆盖: sidebar/tab/skeleton/card/btn 动画静默
- compileCurrentGraph 添加 compileStatus loading 锁防并发编译
- KPI/操作卡片 strong 元素添加 text-overflow: ellipsis
- workspace 页头名称/ID 添加溢出省略号保护
- dashboard-grid 添加响应式断点 (1024px→2列, 640px→1列)
- PropertyPanel 3 个 ErrorBoundary 添加 key+onRetry 支持点击重试

### P2 修复 (8/8)
- formatValue 添加 Number.isFinite 守卫 (NaN/Infinity 显示 "-")
- localStorage 函数添加 typeof window SSR 守卫 (loadGraphFromStorage/safeSetItem)
- qs-source-code 添加 overscroll-behavior: contain 防双滚动条
- 新增 @media print 基础样式 (隐藏侧边栏/固定元素, 白色背景)
- api.js 统一 VITE_API_BASE_URL, 添加与 graphStorePersistenceHelpers 的分工注释
- CommandPalette 添加焦点捕获 (Tab/Shift+Tab 在输入框和列表项间循环)
- 保存按钮文本动态化: "{saving ? "保存中..." : "保存策略图"}"

### 第二轮审计 S0 修复 (4/4)
- 8 个页面 `<div>` → `<main>` 语义化 (Alerts/Chaos/Snapshots/Runbook/Hub/Workspace/Detail/Approval)
- 8 个页面添加 `<h1>` (4 个 qp-page 升级 h2→h1, 4 个添加 sr-only h1)
- LeftSidebar 7 个导航项 `<button>` → `<a href>` + `aria-current="page"`
- CSS `.ad-sidebar-item` 添加 `text-decoration: none` 适配锚点标签

### 第二轮审计 P1 修复 (5/5)
- 3 处输入框补 `<label>` (ModuleSidebar 搜索 / ApprovalPanel 拒绝原因)
- 4 处魔数提取常量 (RECOVERY_MAX_RETRIES/RECOVERY_RETRY_DELAY_MS/PIN_COUNTDOWN_MS/PIN_COUNTDOWN_INTERVAL_MS)
- 3 处关键空 catch 补 console.warn
- **i18n 集成**: ErrorBoundary/CommandPalette/SourceTab/QuantScriptEditor 4 页面 t() 包裹完成，剩余 4 页面已加 useI18n 导入
- **Locale 文件填充**: zh-CN 374 条目自映射, en-US 78 条英文翻译 + 296 条 null 占位

### 第二轮审计 P2 修复 (3/5)
- backtestAnalysisShared.jsx formatTime/formatPercent 去重 (改为 re-export strategyHubFormatters)
- 4 项未使用导出清理 (defaultModules/attachValidation + 2 个 graphStore re-export)
- localStorage 敏感数据已文档标注

### P3 收尾 (全部完成)
- P3-1: 4 处交互元素补 transition (sidebar-search/canvas-focus-toolbar__tab/canvas-recommendation-target/canvas-focus-target)
- P3-2: 9 处 border-radius 硬编码 → --ad-radius-* 令牌

### 剩余项补充修复
- P1-4: ApprovalPanel 2 个高频 flex-row → .approval-flex-row CSS 类 (~20% 内联样式减少)
- P2-4: useStrategyDirectoryModel 2 个关键函数 (toggleStrategySelection/openBlankWorkspace) 补 useCallback
- P2-5: AlertsPage role="list" 包裹 listitem 容器 (5 个页面中完成 1 个典型)

### 验证
- vite build 通过 (warning 2)
- vitest run 86/86 243/243 全量通过
- --tv-* 引用: 0
- rgba(116,145,182,*) 硬编码: 0

### 第三轮审计 S0 修复 (5/5)
- **S0-1 SSE重连**: 监听器引用存储到 EventSource._on* 属性, _reconnect 正确转移 runtime_event/account/run_completed/onerror
- **S0-2 刷新恢复**: initialize() 增加第三回退——localStorage 中可运行 graph 即使不在后端索引也恢复
- **S0-3 清理闭包**: useEffect 空[]→useRef 跟踪 runtimeRef.current, cleanup 读取最新状态
- **S0-4 跨标签保护**: storage listener 验证 graph 有效性 + null→删除提示
- **S0-5 互斥锁**: startRuntime/startBacktest 加 runtimeActionLock, POST 完成释放

### 第三轮审计 P1 修复 (4/7)
- P1-1: safeSetItem JSON.stringify 加 try-catch 防循环引用崩溃
- P1-5: 凭证保存 catch 补错误状态 (saveError), UI 显示红色错误消息
- P1-7: 重连定时器 ID→_reconnectTimer, closeController 清除
- P1-8: 跨标签 graph 验证 (与 S0-4 合并修复)

### 第三轮审计 P2 修复 (1/8)
- P2-16: 空回测数据→空状态页面 (修复无限 loading spinner)

### 第三轮审计 P1 补充修复 (3/3)
- P1-2: 编译按钮添加诊断计数徽章 (issueSummary: E/W)
- P1-4: 1180px 以下 workspace 显示 "建议≥1180px" 提示横幅
- P1-6: compileCurrentGraph 提交前检查 graph_id 变化, 放弃过期编译结果

### 第三轮审计 P2 补充修复 (2/8)
- P2-20: _schema 版本不兼容时丢弃旧数据并 console.warn

### 补充修复 (3 项)
- **P1-3**: focusFinding 设 diagnosticFocusRequested 标记, 代码tab 脉冲动画提示
- **P2-19**: loadRunDetailFlow 加 selectedRunStatus (loading→ready/error)
- **P2-23**: 策略作用域化方案已文档化, 需 12+ 文件重构 (留待后续)

### 按方案执行 (6/7)
- P1-9: RunbookPage/SnapshotsPage/ChaosPage i18n 包裹完成
- P2-17: saveGraph 添加 savingGraph 锁, 防止并发 POST
- P2-18: compileStatus 注释说明双重锁覆盖
- P2-21: compileResultNotice 持久编译状态字段
- P2-22: startBacktest graph_id 缓存比较跳过冗余编译

### P2-23 完成 (对比选择策略作用域)
- graphStoreRuntimeHistoryState.js: _strategyId() + getCompareSelection() wrapper
- toggle/clear/replace/sanitize 4 函数升级为 `{ [strategyId]: [] }` 格式
- buildBacktestHistoryReadyState 同步升级
- 3 reader hooks 添加 `?.[graph_id]` + Array.isArray 回退
- store 初始值 + 3 重置点 `[]` → `{}`
- 4 测试文件更新

### 回归检测
- 空 background: 0
- 硬编码中文字符串 (3 i18n 页面): 0
- console.log/debugger: 0
- vite build: ✓ (warning 2, unchanged)
- vitest run: 86/86 243/243 ✓

### 第四轮审计 P1 修复 (7/7)
- P1-01: ChaosPage API 路径 chaos-experiments→chaos/experiments
- P1-02: utils/api.js VITE_API_BASE_URL 兼容 (自动剥离尾部 /api)
- P1-03: SSE 微批处理: 50ms窗口内事件合并为单次 set() 减少渲染压力
- P1-04: 事件队列上限 500→200 + 渲染时 slice 限制 DOM 节点数
- P1-05: vite.config.js recharts→chart-vendor 独立分包
- P1-06: SSE runtime_event/account JSON.parse 包裹 try-catch
- P1-07: AppShellFallback 8秒后显示"跳过等待"按钮

### 第五轮全量审计
- 3 个新角度 (测试质量/数据流一致性/边界场景) 发现 33 项 (2 S0 + 9 P1 + 17 P2 + 5 P3)
- 全部附带解决方案与验收标准
- 报告: markdown/06-milestones/v1.0.5/08-第五轮全量审计报告-含方案与验收.md

### 第五轮 P1 修复 (6/9 代码项)
- P1-04: buildRuntimeCompletionState 添加 backendError: null (ghost error 修复)
- P1-05: 7 编辑器动作 + removeSelected 添加 compileResult: null (compileResult 陈旧修复)
- P1-06: loadLatestGraph + loadGraphById 添加 compileResult: null
- P1-07: buildBacktestCompletionState events.slice(0,200) 绑定上限
- P1-08: resetGraph 追加 formalQuantScriptOverride/formalQuantScriptDraft/compileResultNotice/savingGraph/runtimeActionLock/parameterMutations 清除
- P1-09: router.js null byte + >128字符策略ID 拒绝→重定向 hub

### 全局最终状态
- 五轮 15 维度审计: **120 项发现**
### 第五轮 P2 修复 (10/17)
- P2-01: resetGraph 调用 closeController 关闭 SSE 连接
- P2-02: resetGraph 补全 parameterMutations/savingGraph/runtimeActionLock (已于 P1-08 完成)
- P2-05: diagnosticFocusRequested 确认已在 P1-03 正确使用 (agent 误报)
- P2-06: highlightedNodeIds 追加 .slice(0, 50) 上限
- P2-08: 历史错误 4 构建器统一为 `message` (始终覆盖, 消除不一致)
- P2-09: navigateTo 100ms 防重复导航 (修复幽灵历史条目)
- P2-12: .ad-sidebar-item__label 添加 overflow/text-overflow/ellipsis
- P2-13: .graph-title 添加 overflow/text-overflow/max-width

### 剩余 P2 补充修复 (6项)
- P2-03: loadLatestGraph/loadGraphById 前检查 runtime.status→stopRuntime
- P2-07: timeline 添加 .slice(0,200) 上限 (回测+历史)
- P2-10: beforeunload 改为 useRef+始终注册 (修复移除/添加间隙 race)
- P2-11: 策略中心过滤器持久化到 URL query params (用户选择方案)
- P2-14: 面包屑 route-bar__link/__current 添加 max-width+overflow+ellipsis

### 最后5项收尾
- 凭证 DELETE 错误反馈 (空 catch → 显示错误消息)
- SSE event._timeLabel 预格式化 (toLocaleTimeString → 缓存)
- Content-Type header (POST无body不需要, 误报跳过)
- 注释/文档收尾

### 120项最终分类统计
| 类别 | 数量 | 状态 |
|------|:--:|------|
| ✅ 已代码修复 | **92** | 完成 |
| ❌ OpenAPI文档缺口 | 6 | 需后端配合更新 root.yaml |
| ❌ 测试编写任务 | 7 | 需新测试文件和基础设施 |
| ❌ 架构/基础设施决策 | 10 | Web Worker/auth/RTL/分页等 |
| ❌ 理论风险无触发路径 | 5 | selectedNodeId/缩放/空ID等 |
| **合计** | **120** | — |

### 最大公约数优化 (执行中)
- 第三阶段高ROI: paste限制(3行)+hash保留(1行)+auth扩展(3行)+分页(已纳入API客户端)
- 第四阶段运行时: O(n*e)→O(n+e)边索引+visibilitychange标签页感知+IndexedDB配额事件
- Agent方案文档: markdown/06-milestones/v1.0.5/09-最大公约数优化方案.md

### 全局终态 (120项审计完成)
- S0: 13/13 ✅ | P1: 34/37 ✅ | P2: 33/38 ✅ | P3: 12/12 ✅
- 可修复代码缺陷: **98/98 (100%)**
### 架构优化完成 (Store 规范)
- **API_BASE 全项目统一**: `src/api/client.js` 为唯一来源
  - utils/api.js 导入 client
  - graphStorePersistenceHelpers.js 导入 client
  - runtimeApproval.js 消除 VITE_BACKEND_ORIGIN
  - 17 处组件调用方去掉手动 `/api` 前缀
- **SSE 批处理窗口提取常量**: `SSE_BATCH_WINDOW_MS = 50`
- **锁统一**: 3 套锁 → 单 `actionLock` 字段 (null|"compiling"|"saving"|"runtime")
  - graphStorePersistenceHelpers.js: withLock() 工具函数
  - saveGraph → actionLock:"saving"
  - compileCurrentGraph → actionLock:"compiling"
  - startRuntime/startBacktest → actionLock:"runtime"
  - savingGraph/compileStatus/runtimeActionLock 旧字段移除, 0 读取方
- **IndexedDB 回退 UI**: App.jsx 监听 qp-storage-quota-exceeded → 显示黄色可关闭横幅
- **分页参数**: withPagination() 已就绪, 等后端支持 ?limit=&offset=

### 测试编写 B1-B3 (3/5)
- B3: saveGraph rollback test (并发锁 + localStorage 写入验证)
- B1: editorActions test (createNode/setSelected/updateNodeConfig)
- B2: LeftSidebar component test (7导航项/锚点/aria-current)
- B4: SSE transport test 已完成 (4/5 pass, 1 skipped-fake timer)
- B5: i18n/axe-core 留待后续
- 测试: 88→91 files, 249→260 tests
- VITE_BACKEND_ORIGIN 残留: 0
- API_BASE `/api` 硬编码残留: 0
- vitest: 88/88 files, 249/249 tests ✓

### Stage 1+2 产出
- **SSE 传输测试**: graphStoreRuntimeTransport.test.js (4 tests: URL/手动关闭/监听器转发/重连耗尽)
- **runtimeActionLock 测试**: graphStore.runtimeActionLock.test.js (3 tests: 锁阻止startRuntime/startBacktest/释放)
- **统一 API 客户端**: src/api/client.js (get/post/del + withPagination + getAuthHeaders)
- **O(n*e)→O(n+e)**: validation.js 预建边索引, edgesByTarget/edgesBySource Map
- **visibilitychange**: App.jsx 标签页可见性监听
- **IndexedDB 配额事件**: qp-storage-quota-exceeded CustomEvent

### 全局终态
- 测试: 86→88 files, 243→249 tests
- 可修复代码缺陷: **100/100 (100%)**
- 不可修复: 20 项 (OpenAPI 6 + 测试编写 5 + Wontfix 5 + 架构 4)

### 合规检查与文档整理
- README: 版本号 v1.0.3→v1.0.5, 就绪度评分 8.4→9.3
- CHANGELOG: 补 v1.0.3 条目, 清理 v0.2.0 节污染内容
- OpenAPI: 版本标签 0.2.0→1.0.5
- RFC-020: 11 个英文标题全部汉化
- 架构文档: 测试数量 86/243→92/269

## v1.0.3

边界防御。NaN/Inf 输入校验, 时间倒流校验, 编译并发 Semaphore, 运行互斥锁。

### S0
- parse_date_range_ms 时间倒流校验 (end_ms <= start_ms → bail!)
- NaN/Inf 输入校验 (is_finite 守卫)
- 编译并发 Semaphore (同时 4 个)

### P1
- Paper 运行互斥锁
- 删除策略后优雅降级
- 端口冲突中文错误
- data_module 429 退避重试
- 插件市场 HTTP 状态码检查
- 核心持久化 struct deny_unknown_fields

### P2
- 跨标签 storage 事件监听
- 配额阈值 >= 修正
- QUANTPILOT_PORT=0 拒绝
- 回测 K 线数量上限 500_000
- 网络 5xx/4xx 分类
- Tauri wait_for_backend 进程归属校验

## v1.0.4

前端排版美化。CSS 修复与视觉层次优化。

### 修复
- 修复 3 个页面 CSS 文件中 25 处空 `background:` 属性（无效 CSS）
- 修复 6 处空 CSS 自定义属性 (`--analysis-surface-bg` / `--analysis-summary-bg` / `--analysis-card-bg`)
- 修复未定义的 CSS 变量引用 (`--analysis-surface-glint`)
- 修复 `--surface-glow` 孤立值行导致 CSS 语法错误

### 视觉层次
- 设计令牌值优化: 面板 #1e1e1e→#1a1a1a, 卡片 #2d2d2d→#242424, 边框 #404040→#4a4a4a
- 卡片增加 inset 高光线 + hover 微升 (translateY(-1px)) + 阴影增强
- 侧边栏背景加深 (#141414→#0f0f0f) 与内容区形成对比
- Tab/侧边栏激活态指示器 2px→3px 加粗
- 表格行 hover 背景反馈
- 按钮添加 active 缩放反馈 (scale(0.97))

### 排版
- 正文行高 1.45→1.6 提升中文可读性
- 标题行高 1.15→1.25
- 正文增加 0.01em letter-spacing
- 卡片内边距 10px12px→12px14px
- 新增 `--ad-font-size-hero: 24px` 令牌
- 策略中心状态条 8列单行→4列双行布局 (更宽松透气)
- 策略中心 hero 区域增加底部边框分隔线
- CTA 按钮移除内联 style，改用 CSS 类
- hero 操作按钮去嵌套 StrategyTaskGroup 包裹，扁平化布局

### 代码质量
- styles.css 旧变量替换: var(--panel)→var(--ad-panel), var(--panel-2)→var(--ad-panel), var(--text)→var(--ad-text)
- 按钮圆角 8px→var(--ad-radius-panel) 统一令牌
- strategy-workspace.css 移除 152 行与 strategy-hub.css 重复的 strategy-card-note 样式
- CSS 文件大小: strategy-workspace.css 减少约 152 行
- General_Policy.md 新增 §8.8 CSS 文件结构与质量规范
- 滚动条颜色统一使用 --ad-border / --ad-border-strong 令牌

### 验证
- vite build 通过
- vitest run 86/86 文件 243/243 测试全量通过
- 空 background: 属性 0 处
- 旧 var(--panel) 引用 0 处

## v1.0.2

体验优化与发布就绪。

### 优化
- 插件 trait object 全量重构: 18 个 IndicatorEvaluator 注册表替代硬编码 match
- CoreIndicatorKind 新增 PartialOrd/Ord/Hash derive
- DAG 验证接入编译管道 (两个入口)
- SSE 断开自动重连 (指数退避, 最多 5 次) + "reconnecting" 状态
- 策略中心新增 "开始使用" CTA 引导按钮
- 前端骨架屏 (skeleton-pulse 动画)
- 回测对比页叠加权益曲线 (Recharts)
- 后端优雅关闭 (bg_handle.abort())
- 错误提示加操作建议
- localStorage _schema 版本标记
- 旧格式兼容代码清理 (normalizeGraphShape fallback 移除)
- Vec::with_capacity 预分配 (agent/execution/risk/fill_engine)
- 测试辅助函数去重 (assert_complete_event_envelopes → tests/common)

### 发布
- Tauri 便携包 (dist/QuantPilot/)
- 版本号统一为 1.0.2

## v1.0.1

三回合全量审计质量深化。69 发现中 64 项修复。

### S0
- validate_dag() 接线到编译管道

### P1
- PluginLifecycleState 转移警卫 (activate/deactivate/mark_faulted)
- OrderStatus::can_transition_to() 状态机
- credential_vault 数据降级告警 (unwrap_or_default→unwrap_or_else)
- SSE EventSource 卸载清理 (useEffect cleanup)
- 后台任务 JoinHandle 保存

### P2
- 嵌套锁标注 (锁顺序注释)
- localStorage _schema: 1
- workspace sha2 统一
- Vec::with_capacity 预分配
- 测试辅助函数去重
- 旧格式兼容代码清理
- 前端 "reconnecting" 状态

## v1.0.0

插件化架构 + 重型策略 + 超级规范化。首个整合包。

### Phase 1 协议补全
- RFC-001 DataRequest struct 落地 (MarketScope / PrimaryDataType / SourceType / Timeframe / PrecisionPolicy)
- RFC-010 Allocation struct 落地 (AllocationMethod: EqualWeight/FixedWeight/RankWeight/ScoreWeight/RiskParity)
- RFC-012 Order struct 落地 (OrderStatus 生命周期: Created→Expired)
- RFC-013 ExecutionFeedback struct 落地 (FeedbackKind 7 种)
- RFC-020 PluginManifest 扩展: PluginType(Atom/Suite) + AtomRef + hot_handoff + asset_management
- OrderType 扩展: StopLoss/StopLossLimit/TakeProfit/TakeProfitLimit
- RFC README: 全部 20 RFC 标注实现状态 (19✅ 1🔄 0📋)

### Phase 2 插件架构
- RuntimePluginRegistry: scan_atoms() / atoms() / validate_suite() / check_security()
- PluginSecurityAction: AccessCredentials(拒绝) / NetworkCall(需声明) / WriteState
- PluginMarketClient: 远端拉取 index.json + fetch_manifest() + 本地协议校验
- PluginManifest::validate() 追加套件校验和热接管前提检查

### Phase 3 重型策略
- CoreStrategyIr 新增 edges: Vec<CoreIREdge> 支持 DAG 路由
- validate_dag(): DFS 环检测
- Sandbox trait 新增 handoff() 方法, RealTimeSandbox 实现热接管
- HandoffSnapshot: 持仓/未结订单/现金完整快照 + validate_completeness()
- Allocation::apply_to_targets(): 按权重分配资金 + min/max 约束

### Phase 4 流水线固化
- Pre-commit hook: cargo check + test --no-run + build + vitest
- 元流水线: track-gate-metrics.ps1 (6 项门禁耗时追踪)
- 设计文档模板: design-doc-template.md

### Phase 5 收口整合
- 全量审计: 0 阻断发现, 加权 7.4/10
- 文档同步: README / CHANGELOG / 里程碑 / 总览全部更新

## 0.5.2

全量排雷收口。16 项 S0/P1/P2 完成。

### S0 测试套件修复
- 后端测试编译修复: `tests/common/mod.rs` 的 `include!()` 模式断裂修复, 4 模块 `pub` + 重导出, `safe_eprintln!` 宏移入 `main.rs`
- 前端测试 DOM 修复: 17 文件更新 — CSS 选择器 `.strategy-*` → `.ad-*` / `[aria-label]`, tab 索引重映射, EventStreamPanel 子组件测试重写, supportMatrix 18 指标同步
- 全量回归验证: `cargo test --workspace` 编译通过, `npm run test` 86/86 文件 243/243 测试通过

### P1 架构排雷
- 消除 `map_frontend_runtime_config` 剩余调用: 3 处 → 0, `merge_runtime_targets` 改为接受 `CompileRuntimeTargets`
- 存储配额激活: 核查确认 `ensure_storage_quota` 已含全局 475MB 检查
- data_module 汉化: 8 处 Binance/OKX 解析错误英文→中文
- Rust warning 清零: 31 → 0 (quantscript dead_code + quantpilot 死代码清理 + 变量修复)
- 测试有效性抽检: 3/3 抽检测试能捕获回归

### P2 质量收口
- 文档同步: README v0.1.0→v0.5.1, overview 状态更新, 里程碑状态更新
- 告警阈值验证: WARN=400MB / FORCE=450MB / REJECT=475MB 与 §7.2 一致
- DEV 清理验证: `QUANTPILOT_DEV=true` 强制清理瞬态数据
- 版本号验证: 4 处一致为 0.5.1

## 0.5.1

全量审计收口排雷。15 项 P0/P1/P2 完成。

### P0 架构排雷
- 编译路径统一: 删除 `map_frontend_runtime_config` 直接编译路径，QS 管道成为唯一编译入口 (`compile_runtime_protocol_via_qs`)
- 53 处英文错误消息汉化: 11 个文件中所有 `bail!`/`anyhow!`/`Err()` 用户可读文本改为中文
- 存储配额强制执行: 激活 `GLOBAL_MAX_BYTES`、新增 450MB/475MB 阈值、每目录配额、`startup_storage_cleanup` 90% 激进清理 + DEV 模式强制清理瞬态数据

### P1 合规收口
- `persist_with_ttl` + `ensure_storage_quota`: 11 个写入路径全部声明 `StorageLifecycle` 并执行配额检查
- 6 个缺失的 indicator 单元测试: MA Cross, RSI, MACD, Momentum, Z-Score, QuoteObserve 各新增 smoke test
- 6 处硬编码魔数消除: 提取为命名常量
- 3 处英文测试断言 + 2 处文档问题修复
- CHANGELOG 补全 v0.4.2 ~ v0.5.0

## 0.5.0

Adobe 风格前端全量重构 + 38 项 General_Policy 全量审计。

### 前端重构
- Adobe 暗色面板设计系统 (`--ad-*` CSS 令牌)
- App Shell: 左侧 48px 图标侧边栏 + 160px hover 展开
- 工作区面板化: 策略编辑器 / 回测面板 / 研究控制台
- 组件重设计: 对标 Adobe Photoshop/Illustrator 专业暗色风格
- SVG 图标组件库替换 Unicode emoji

### 后端审计
- §1-§8 38 项全量合规审计
- 6 角度 30 项扩展审计 + R1/R2/R3 回归审计 13 发现
- 18 项 S0/P1/P2: 安全加固 / 体验修复 / 质量收口

## 0.4.3

用户体验与安全收口。5 项完成。

- JSON 错误汉化: `json_rejection_middleware` 返回中文错误
- API 文档字段名同步
- CLI 安全: 交互式凭证输入替代命令行参数
- 强制认证: 全局 auth middleware
- Vault 懒初始化: 避免启动时文件系统依赖

## 0.4.2

收口排雷。10 项完成。

- raw print 迁移: 40 处 `eprintln!`/`println!` → `safe_eprintln!`
- 废弃 API 清理: `problem_*` 系列全部迁移
- 测试修复
- Tauri 启动优化

## 0.4.1

安全审计全量修复。12 项发现中 11 项已修复，1 项已文档化。

### 安全修复
- credential_vault: SecretString wrapper (Drop 时 Zeroize) / 原子写入 (tmp + rename + bak) / AAD 绑定 / OnceLock 去竞态 / 删除静默密钥降级
- CLI: 交互式 stdin 输入替代命令行参数传递凭证明文
- 日志: 动态注册 vault 字段值到脱敏模块 / while let 全量替换
- 环境变量: 删除 set_var("HTTPS_PROXY") 全局副作用
- 前端: 凭证面板关闭时清零 React state

### 新增
- /api/credentials 路由: GET list / POST set / DELETE delete
- CredentialInput 组件: 动态字段渲染 / OkxCredentialInput 预设
- TopToolbar 凭证管理面板
- storage/.machine_key 随机机器密钥

### 文档
- api安全方案.md: 本地凭证存储设计 + 安全边界与已知限制
- v0.4.1/01-03 三个里程碑文档

## 0.4.0

审计驱动收口。三条工作线 10 项全量完成。

### UI 简洁化
- 字段裁剪: 面板默认 ≤4 字段 + 折叠展开
- 面板重整: ResearchConsole + EventStreamPanel 去重叠
- 仪表盘布局: 默认首页替代 6 标签页
- CSS 变量统一

### 引导教程
- TutorialOverlay: 5 步分步引导 + 前进/后退/退出
- createTutorialSteps(t) i18n 支持

### 凭证安全
- CredentialVault: AES-256-GCM 加密本地凭证文件
- CredentialInput 组件: type=password / autocomplete=off
- Zeroizing 内存保护 / safe_log 日志脱敏

## 0.3.0

全量合规优化与基础设施补强。22 项 P0-P3 + 10 个新信号 + I-1/I-2 双路径合并。

## 0.2.0

测试自动化全链路。@test/@step/@assert 指令 + TestRunner + 前端测试桥 + Playwright + CI。

## 0.1.0

初始私有基线。Paper 运行时沙箱 / 图编辑器 / QS 编译管道 / 6 类 K 线 Intent / 回测。
