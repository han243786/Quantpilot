# Changelog

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
