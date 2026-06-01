# QuantPilot 项目总规则 v4.0.0

> 生效日期: 2026-05-24 | 所有开发者必须遵守 | 违反的 PR 不予合并
> 重构: v2.0→v3.0 拆分为阻断规则(门禁可查) + 审计规则(里程碑审查)
> 每个条款标注 **检查方式**: 🛡️门禁 / 🔍审计 / 🛡️+🔍
> 更新: v3.7.1 增加 Rust 格式基线、功能演进登记和防回退规则，`cargo fmt --check` 纳入 pre-commit / CI / closeout
> v4.0.0: 状态机化 QuantScript, Risk Plane, ExecutionMachine 能力来源, 前端以后端 capability 为真源, 开发者学习流水线边界已实施
> v4.10.0: 固化单机交易工具产品边界；账户系统与策略中心搜索/筛选标记为 unsupported
> v4.15.0: 本文件由三矩阵治理接管；变更入口先走 `00-matrix-governance/README.md`，再引用本文件查证实现约束

---

## 一、架构铁律（12 条）

### §1.1 QS 是唯一策略定义路径 🛡️

```
策略定义 → QS 源码 → parse → HIR → lower → Core IR → sandbox
                              ↑
         前端 graph 编辑器 → 可视化/编辑（不产生独立编译路径）
```

- **禁止**：graph 编辑器产出独立的 `RuntimeProtocolCoreConfig` 直接编译
- **允许**：graph 编辑器通过 `generate_quantscript_from_graph_value` → QS 管道编译
- **门禁**: `grep -r "map_frontend_runtime_config\|RuntimeProtocolCoreConfig {" src/ --include="*.rs" | grep -v "//.*豁免\|gen_screenshots\|tests/"` 应返回 0

### §1.2 新增功能必须跨三层验证 🔍

| 层 | 检查点 | 验收标准 |
|----|--------|---------|
| QS 解析 | `quantscript/src/resolve.rs` | 新语法可解析，未知函数拒绝 |
| Core IR | `qrpc_core_ir/src/lib.rs` | 新 indicator/intent 有对应枚举变体 |
| 运行时 | `qrpc_runtime/src/core_ir_evaluator.rs` | 新 indicator 有 evaluator 实现，非 stub |
| 执行端前端 | `frontend-executor/src/` | 执行端策略控制与热调参界面可用 |
| 前端 | `frontend/src/modules/builtinModules.js` | 若影响模块面板，注册新模块 |
| 端到端 | `tests/scenarios/` | 新增 .qs 场景文件验证 |

### §1.3 编译路径不可绕过 🛡️

```
所有策略编译必须经过：
  QS 解析 → 语义分析 → 类型检查 → lowering → Core IR → sandbox
```
- **门禁**: 所有 `compile_runtime_protocol_config` 调用必须来自 `compile_runtime_protocol_via_qs` 的输出

### §1.4 数据流单向原则 🔍

```
QS 源码 → graph JSON → 前端可视化
         ↘ .qs 文件持久化
```
- **禁止**：保存 graph 时无条件覆盖原始 QS 源码
- 若 `source_mode="quantscript"`，保留原始 QS

### §1.5 功能演进必须先登记 🛡️+🔍 (v3.7.1 新增)

- 新增功能、能力扩展、语义变更必须先写入里程碑 `01-规划方案.md` 的“功能演进登记”。
- 每个新增能力必须列出能力 ID、生命周期、用户可见入口、涉及层、依赖能力、非目标、fallback / 拒绝行为。
- 每个新增能力必须有“回归保护矩阵”，明确受影响的既有能力、风险、保护证据和验证命令。
- 不新增功能的 PATCH 版本必须在“非目标”中明确写明“不新增功能”或“不扩大功能范围”。
- **门禁**: `powershell tools/check-feature-evolution.ps1` 必须通过。

### §1.6 顶层 DAG 必须保留，复杂度只能下沉到可治理状态机 🔍 (v4.0.0 新增)

- 量化基准链路仍为 `data -> intent -> agent -> risk -> execution -> fill`。
- 顶层图必须保持有向无环；新增状态机能力不得让顶层图出现循环依赖。
- 节点内部可以状态机化，但必须使用登记过的 `ObservationMachine` / `DecisionMachine` / `ExecutionMachine` 或后续正式登记模板。
- 旧链路必须能通过兼容桥映射到默认 machine 实例，不得破坏旧图、旧 QS、旧运行记录的读取边界。
- **审计**: MAJOR closeout 必须检查 Core IR DAG、旧图加载、旧 QS 编译和旧运行记录回放。

### §1.7 QS 状态机 DSL 不是通用脚本语言 🛡️+🔍 (v4.0.0 新增)

- QS 状态机语义必须以声明式 `transition` 为主，受控 `action block` 为辅。
- `action block` 只能读取声明输入、事件上下文、局部变量和本节点 typed memory。
- `action block` 只能写本节点 memory、emit 输出和 diagnostic 记录。
- 禁止 QS 直接访问文件、网络、系统 API、密钥或真实下单接口。
- 禁止无限循环、递归、动态 eval、跨节点直接修改状态或 memory。
- 解析器接受不等于产品支持；任何新语法必须进入保留面合约、静态审计和拒绝路径。
- **门禁**: 新 QS 状态机语义必须有 parse/analyze 拒绝测试；unsupported 路径必须产生结构化诊断。

### §1.8 状态迁移必须事件驱动并可解释 🛡️+🔍 (v4.0.0 新增)

- 任何 machine 的 `transition` 必须绑定明确事件来源。
- 没有事件来源的状态迁移禁止进入 runtime。
- 事件必须携带 `event_id`, `event_type`, `event_time`, `source`, `payload`, `freshness`, `sequence`, `replayable`。
- `memory` 变化、cache 返回、silence 进入/退出、recovery 开始/完成必须形成事件或可回放证据。
- 用户自定义优先级不能覆盖因果顺序、安全层级、DAG 依赖和确定性兜底排序。
- **审计**: 运行/回放详情必须能解释状态为何迁移、由哪个事件触发、输入新鲜度如何。

### §1.9 Risk Plane 不可绕过 🛡️+🔍 (v4.0.0 新增)

- 风控逻辑可以用 `DecisionMachine` 表达，但运行时必须有独立高优先级 Risk Plane。
- 所有真实下单路径必须经过 `risk_precheck -> risk_order_check -> risk_postcheck`。
- `LiveActual` 模式下，ExecutionMachine 不得直接调用 VenueAdapter。
- `emergency_halt` 高于所有 QS 逻辑；`reduce_only` 只允许降低敞口；`freeze_open` 禁止新开仓。
- `stale` 或 `recovering` 数据默认不得扩大真实风险敞口，除非 Risk Plane 明确允许。
- **门禁**: 真实订单发送路径必须有 Risk Plane 拦截测试；缺失风控上下文时必须拒绝。

### §1.10 Execution 能力来源必须显式标记 🛡️+🔍 (v4.0.0 新增)

- 每个订单能力必须标记为 `provider_native`、`runtime_simulated` 或 `unsupported`。
- `provider_native` 表示交易所/券商原生支持。
- `runtime_simulated` 表示 QuantPilot 本地模拟或合成，必须在事件、日志、UI 和报告中显式标记。
- `unsupported` 表示不支持且未模拟，必须在编译期或运行前拒绝。
- 不允许把 `unsupported` 静默降级为其他订单语义。
- 不允许把本地触发逻辑描述为交易所原生支持。
- `runtime_simulated` 执行路径必须有本地订单、成交、手续费、资产账本和 provider submission detached 证据。
- **门禁**: VenueCapabilityMatrix 与 `/api/capabilities`、支持矩阵、前端文案必须一致。

### §1.11 开发者学习流水线边界 🔍 (v4.0.0 新增)

- 核心学习元流水线进入仓库，个人学习记录放入 `markdown/learning/`。
- `markdown/learning/` 必须保持本地忽略，不推 GitHub。
- 学习记录只能在用户明确要求“记录本轮学习”“生成学习记录”等指令时写入。
- 学习流水线不进入每次强制门禁，但 MAJOR closeout 必须检查是否存在 owner 必学核心机制。
- 面向所有开发者的学习流水线版本必须在 owner 多轮体验后另行设计，不得在第一版中提前泛化。
- **审计**: closeout 报告必须包含 Developer Learning Closeout 或明确说明本版本无新增 owner 必学机制；`tools/check-learning-closeout.ps1` 负责检查结构入口和本地学习记录边界。

### §1.12 前端能力入口必须以后端 capability 为唯一真源 🛡️+🔍 (v4.0.0 新增)

- 前端可以定义布局、排序、i18n 标签和视图投影，但不得自行决定某个工作区、工具栏动作、模块、运行模式或执行语义是否可用。
- 工作区 tab、工作区 surface、工具栏 action、模块面板可见性、启用状态、禁用原因和能力来源必须从 `GET /api/capabilities` 的结构化响应投影而来。

### §1.13 产品定位边界: 单机交易工具 🔍 (v4.10.0 新增)

- QuantPilot 当前定位为单人本地使用的专业量化策略研究与交易桌面工具，不按 SaaS、团队协作后台或多租户账号系统设计。
- 现有 `register` / `login` / `refresh` 仅服务本地会话、凭证隔离和桌面壳内部访问边界，不得扩展描述为完整账户系统。
- 明确 unsupported: `auth.logout`, `auth.password_reset`, `auth.2fa`, `account.profile`, RBAC / 管理员用户管理 UI。
- 策略中心保持全量可滚动列表，不新增搜索、筛选、分页或排序能力；明确 unsupported: `hub.search`, `hub.filter`。
- 如果未来产品定位转向 SaaS、团队协作或大量策略目录管理，必须以独立 MAJOR/MINOR 规划重新登记，不得在小版本中顺手加入。
- **审计**: README、支持矩阵、里程碑、UI 文案不得把上述 unsupported 项描述为已支持、计划中或缺陷待修。
- 新增用户可见入口时，必须先扩展后端 `CapabilityResponse`、OpenAPI、capability fixture、支持矩阵、能力治理注册表和回归测试；禁止只新增 React 静态数组或 CSS 入口。
- `loading`、`cache`、`safe_fallback` 三种能力加载状态必须有明确 UI 行为。`safe_fallback` 不得恢复上一版本完整工作区，只能保留最小只读或明确禁用入口。
- 前端不得把 `declared_only`、`unsupported`、`planned`、`restricted` 或 `runtime_simulated` 显示为无条件 `supported`。
- 本地静态列表只允许作为骨架、排序偏好、文案 fallback 或旧版本兼容桥；不得作为能力边界真源。
- **门禁**: `tools/check-capability-governance.ps1`、后端 capability fixture 测试、前端能力投影测试必须同时覆盖；新增 UI surface/action 必须有“后端移除或降级后前端禁用/隐藏”的回归测试。
- **审计**: closeout 必须抽查至少一个工作区入口和一个工具栏 action，确认其状态来自后端 capability snapshot 而非前端硬编码。

---

## 二、代码规范（5 条 + 3 条 v3.0 新增）

### §2.1 错误消息必须是中文 🔍

```rust
// ✅ 正确
bail!("回测需要至少一个启用的 K 线数据源");

// ❌ 错误
bail!("backtest requires at least one enabled kline data source");
```
- API 错误码（如 `"capability_gated"`）保留英文（协议标识符）
- 诊断代码（`QS0001`、`QPQSLOW001`）保留英文

### §2.2 测试断言使用中文子串 🔍

### §2.3 新 indicator/evaluator 必须有单元测试 🔍

### §2.4 新 TestAction 必须有集成场景 🔍

### §2.5 前端字符串使用 `t()` 包裹 🔍

### §2.6 凭证保险库安全 🛡️ (v3.0 新增)

- 加密算法: AES-256-GCM (ring crate)，禁止降级
- 密钥派生: PBKDF2 ≥1,000,000 轮 (执行端 `credential_vault_v2.rs`); 测试端 ≥600,000 轮 (禁止 SHA-256 直派生用于新密钥)
- 机器密钥: 独立随机文件 (`.executor-machine-key`), 派生不再依赖 hostname (v3.5.1 fix)
- 内存清零: 所有密钥/凭证在 Drop 时 Zeroizing; v3.7.0 S1: 执行端 `CredentialEntry` 使用 `Zeroizing<String>`
- 原子写入: `write(.tmp) → fsync → rename → fsync(parent) → bak 回滚`
- **门禁**: `grep "Zeroizing" src-executor/credential_vault_v2.rs` 确认使用; `grep -r "fs::write\|fs::rename" src/credential_vault.rs | grep -v "tmp\|sync_all\|bak"` 应返回 0

### §2.7 实时执行安全边界 🛡️ (v3.0 新增)

- OKX 签名: HMAC-SHA256, 时间戳 ISO8601, 禁止明文传输 secret
- 每日风险限制: ≤100 订单/天, ≤$1000 名义价值/单
- 速率限制: ≥200ms 请求间隔
- 错误清洗: 所有 API 响应在日志输出前清除 secret/key/sign/passphrase 字段
- **门禁**: `grep -r "secret\|api_key\|passphrase" qrpc_runtime/src/live_execution.rs | grep -v "safe_\|sanitize\|clear\|清洗\|遮蔽"` 应返回 0

### §2.8 本地会话认证安全 🛡️ (v3.0 新增, v4.10.0 边界重命名)

- 该条只约束现有本地会话和凭证隔离端点, 不代表完整账户系统支持。
- 密码: bcrypt ≥12 轮, 禁止明文存储; bcrypt verify 使用 `tokio::spawn_blocking` 避免阻塞工作线程 (v3.5.0 P2-1)
- JWT: HS256, 24h 过期, 密钥来自环境变量/持久化随机文件
- 刷新令牌轮换: 每次刷新生成新 token, 旧 token 立即失效; SHA-256 哈希存储用于重放检测 (v3.5.0 P1-1)
- 重放检测: 旧 token 重放 → `410 GONE` + 撤销整个 token family, 强制重新登录 (v3.5.1 P2-3)
- Token family 撤销: 重放检测触发后, 该 family 下全部活跃 token 被标记为已撤销; 后续任何刷新请求均拒绝
- 注册限流: auth 端点独立 6次/分钟/IP
- 不新增注销、密码找回、2FA、RBAC、用户资料页或远程账户恢复语义。
- DEV 模式: `QUANTPILOT_DEV=true` 跳过认证 + 跳过速率限制 (v3.5.0 P2-3)
- **门禁**: `grep "spawn_blocking" src/auth/mod.rs` 确认 bcrypt verify 异步化

---

## 三、文档规范（3 条）— 不变

§3.1-§3.3 保持原有内容（文档分层、全中文、里程碑编号）。

---

## 四、变更管理（3 条 + 1 条 v3.0 新增）

§4.1-§4.3 保持原有内容（capability 变更固件更新、错误变更测试修复、结构体变更）。

### §4.4 API 路由变更 🛡️ (v3.0 新增)

- 新增路由必须在 `app_router.rs` 注册
- SPA fallback (`dist/index.html`) 不可删除
- 路由变更同步更新 `contracts/openapi/root.yaml`
- **门禁**: 新增 handler 函数 → `grep` 确认在 `build_app_router` 中有对应 `.route()` 调用

---

## 五、禁止事项（6 条）

### §5.1 禁止硬编码 🛡️+🔍

### §5.2 禁止静默忽略参数 🔍

### §5.3 禁止 stub evaluator 🔍

### §5.4 禁止在图编辑器中绕过 QS 编译 🛡️

### §5.5 禁止跳过端到端验证 🛡️

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd frontend && npm run build
cd frontend && npm run test
cd frontend && npm audit --audit-level=moderate  # v3.0 新增
```

### §5.6 禁止 Rust 格式漂移 🛡️

- **规则**: Rust 源码必须通过 `cargo fmt --check`。格式化基线只允许由 `cargo fmt` 生成，不允许手工维护局部风格。
- **门禁**: `cargo fmt --check` 在 pre-commit、CI 和 closeout 三层执行。
- **处理**: 若失败，单独执行 `cargo fmt` 并作为格式基线提交；不得把格式漂移带入功能变更。

---

## 六、快速检查单（PR 提交前）— 不变

§6.1-§6.10 保持原有 10 项检查单，新增 2 项：

| 11 | npm audit 清零 | `cd frontend && npm audit --audit-level=moderate` |
| 12 | E2E 关键路径通过 | `cd frontend && npm run test:e2e` |
| 13 | Rust 格式无漂移 | `cargo fmt --check` |

---

## 七、存储生命周期（5 条）— 不变

§7.1-§7.5 保持原有内容。v3.0 新增注解：

> **注意**: 原子写入规则已提升至 §2.6（凭证保险库安全），所有 `storage/` 写入点均需遵循 `write(.tmp) → fsync → rename → fsync(parent)`。

---

## 八、前端设计规范（v3.7.1 补充）— 不变

§8.1-§8.8 保持原有内容。

### §8.9 主路径动作分层 🔍

- 页面只能常驻当前任务的主路径动作；低频工具、上下文动作和破坏性动作必须下沉到明确的菜单、展开区或具体对象行内。
- 下沉动作不得被删除，必须保留原行为、可访问名称、键盘可达性和自动化测试钩子。
- 全局页面不得重复承载同一能力入口；若面板内部已有刷新、筛选、详情或对比能力，外层只保留导航或摘要入口。
- 前端新增功能时必须先判断动作层级，避免把功能追加成首屏按钮堆叠。

### §8.10 专业工作区外壳 🔍

- 复杂页面必须先归入明确 workspace：策略中心、构建、研究回测、运行监控；不得把多个 workspace 的主任务混在同一首屏。
- 左侧导航、页级主控制条、中央主对象、右侧 inspector 和时间线区域的职责必须稳定，新增能力必须先选择归属区域。
- 运行、回测、执行、订单等时间型或状态型视图必须显式区分输入、当前结果、历史/时间线和详情上下文。
- Workspace 布局调整不得制造能力宣称；真实支持边界仍以 capability、编译器、运行时和风控为准。

### §8.11 能力驱动的工作区设计 🛡️+🔍

- 工作区设计稿只能定义信息架构和交互层级，不能直接定义“能力已支持”。
- 工作区一级入口必须消费 capability projection；后端未声明的入口不得可点击，后端降级的入口必须展示禁用原因。
- 工具栏按钮、快捷动作、卡片 CTA 和面板内二级入口必须共用同一份 capability projection，禁止各组件分别维护支持判断。
- 视觉状态必须区分 `supported`、`declared_only`、`unsupported`、`cache`、`safe_fallback`，并在需要时展示能力来源或拒绝原因。
- 前端新增工作区入口时，必须同时补 E2E 或组件测试，验证该入口可访问；同时补诱错测试，验证后端 capability 缺失时入口不会假装可用。

---

## 九、治理系统约束（4 条 v3.0 新增）

### §9.1 沙箱验证 🛡️ (v3.0 新增)

- AI 提案必须通过独立沙箱回放验证 (`run_sandbox_verification`)
- 验证失败 (CandidateUnderperforms) 不允许合并
- 验证超时 (120s) 不阻塞流程但记录告警
- **门禁**: `grep -r "run_sandbox_verification" src/runtime/mutation.rs` 确认调用存在

### §9.2 签名快照 🛡️ (v3.0 新增)

- 快照签名: SHA-256(capability_hash + strategy_version + parameter_version + core_ir_digest + event_slice_bounds + created_at_ms)
- 恢复前验证签名完整性
- 快照持久化: 原子写入 + fsync
- **门禁**: `grep "canonical_json_sha256_digest\|验签" src/snapshot_service.rs`

### §9.3 告警引擎 🔍 (v3.0 新增)

- 10 条默认告警规则不可删除，仅可追加
- 10 条规则全部有 `resolve_condition` 恢复条件 (v3.5.0 P1-2); 10 条默认规则均已补充恢复条件表达式
- 自动恢复: 触发条件不成立时自动标记 Resolved; 优先评估 `rule.resolve_condition` 自定义表达式, `None` 时回退硬编码阈值 (v3.5.1 P1-2)
- 告警去重: `INSERT OR IGNORE` + 内存 `HashSet<AlertFingerprint>` 双重去重, 消除 TOCTOU 竞态 (v3.5.1 P1-5)
- 恢复检查三阶段无锁: 内存状态更新 → 写锁释放 → 磁盘I/O, 不阻塞并发读 (v3.5.1 P1-6)
- **审计**: 每里程碑检查告警规则是否被意外删除

### §9.4 审批工作流 🔍 (v3.0 新增)

- 审批单过期自动标记 Expired
- AI 提案状态与审批单状态联动 (Accept→Activated, Expired→Expired)
- 锁顺序: approval_records → ai_proposals（反序死锁）
- 事务保护: 令牌/凭证相关状态变更操作包裹 `BEGIN IMMEDIATE` / `COMMIT`, 崩溃后不会丢失一致性 (v3.5.1 P1-1)
- **审计**: 每里程碑检查审批竞态和锁顺序

---

## 规则总表

| 条款 | 类型 | 检查方式 |
|------|:--:|:--:|
| §1.1 QS唯一路径 | 阻断 | 🛡️ |
| §1.2 跨三层验证 | 阻断 | 🔍 |
| §1.3 编译不可绕过 | 阻断 | 🛡️ |
| §1.4 数据流单向 | 高 | 🔍 |
| §1.5 功能演进先登记 | 阻断 | 🛡️+🔍 |
| §1.6 顶层DAG与状态机边界 | 高 | 🔍 |
| §1.7 QS状态机DSL边界 | 阻断 | 🛡️+🔍 |
| §1.8 事件驱动迁移 | 阻断 | 🛡️+🔍 |
| §1.9 Risk Plane不可绕过 | 阻断 | 🛡️+🔍 |
| §1.10 Execution能力来源 | 阻断 | 🛡️+🔍 |
| §1.11 学习流水线边界 | 中 | 🔍 |
| §1.12 前端能力以后端为真源 | 阻断 | 🛡️+🔍 |
| §2.1 错误全中文 | 高 | 🔍 |
| §2.2 测试断言中文 | 中 | 🔍 |
| §2.3 indicator 测试 | 中 | 🔍 |
| §2.4 集成场景 | 中 | 🔍 |
| §2.5 前端 t() | 中 | 🔍 |
| §2.6 凭证保险库 | 阻断 | 🛡️ |
| §2.7 实时执行 | 阻断 | 🛡️ |
| §2.8 本地会话认证 | 阻断 | 🛡️ |
| §3.1 文档分层 | 中 | 🔍 |
| §3.2 文档全中文 | 中 | 🔍 |
| §3.3 里程碑命名 | 中 | 🔍 |
| §4.1 capability 变更 | 高 | 🛡️ |
| §4.2 错误变更测试 | 高 | 🛡️ |
| §4.3 结构体变更 | 高 | 🔍 |
| §4.4 API路由变更 | 高 | 🛡️ |
| §5.1 禁止硬编码 | 高 | 🛡️+🔍 |
| §5.2 禁止静默忽略 | 阻断 | 🔍 |
| §5.3 禁止 stub | 阻断 | 🔍 |
| §5.4 禁止绕过QS | 阻断 | 🛡️ |
| §5.5 端到端验证 | 阻断 | 🛡️ |
| §5.6 禁止格式漂移 | 阻断 | 🛡️ |
| §7.1 三级分类 | 高 | 🛡️ |
| §7.2 存储配额 | 高 | 🛡️ |
| §7.3 启动清理 | 高 | 🛡️ |
| §7.4 写入声明生命周期 | 高 | 🛡️ |
| §7.5 DEV 激进清理 | 中 | 🛡️ |
| §8.1-8.11 前端设计 | 中 | 🛡️+🔍 |
| §9.1 沙箱验证 | 阻断 | 🛡️ |
| §9.2 签名快照 | 阻断 | 🛡️ |
| §9.3 告警引擎 | 高 | 🔍 |
| §9.4 审批工作流 | 高 | 🔍 |
| §10.4 功能演进回归保护 | 阻断 | 🛡️+🔍 |
| §10.5 v4演化回归保护 | 阻断 | 🛡️+🔍 |

**总计: 44 条** (阻断 23 / 高 13 / 中 8)
🛡️ 门禁可查: 27 条 | 🔍 审计人工: 24 条

---

## 十、功能覆盖矩阵 §10 🆕 (v3.7.0)

> 本节列出系统已实现的大规模功能模块、各自的覆盖范围、以及各模块间的依赖关系。
> 每次 MAJOR/MINOR 版本发布时必须对照此矩阵检查是否有功能退化或覆盖缺失。

### §10.1 核心系统覆盖

| 系统 | 覆盖范围 | 关键文件 | 依赖 |
|------|---------|---------|------|
| **QS 编译管道** | graph JSON→QS源码→parse→HIR→lower→Core IR | `compile_api.rs`, `backend/graph_compile/quantscript_graph.rs`, `quantscript/src/lowering/` | §1.1, §1.3 |
| **策略图编辑器** | React Flow 可视化编辑, 6类节点(数据/意图/代理/风控/执行/运行时), 模板库 | `StrategyCanvas.jsx`, `strategyTemplates.js`, `builtinModules.js` | §1.1, §5.4 |
| **Paper 运行时** | 实时沙盒, 慢/快双周期, 模拟成交引擎 | `qrpc_runtime/src/lib.rs`, `fill_engine.rs`, `sandbox/mod.rs` | §1.2 |
| **回测引擎** | 历史回放/确定性Mock, 夏普/索提诺/卡尔玛等12项指标 | `backtest_metrics.rs`, `sandbox/mod.rs`, `BacktestDetailPage.jsx` | §1.2, §2.3 |
| **执行端 (Executor)** | 独立进程 :3001, 策略部署/启动/停止/热调参, OKX Paper行情, lightweight-charts K线 | `src-executor/` (10文件), `frontend-executor/src/` (7文件) | §2.7, §4.4 |
| **编译缓存** | 以 `(graph_hash, compile_options_hash)` 为 key 缓存编译产物, LRU 50 条淘汰 (v3.5.0 P2-8) | `compile_api.rs` | §1.1 |
| **Paper/Live 模式切换** | ExecutorTopBar Paper/Live 拨动开关, 确认弹窗, SSE 广播, WS 重连 (v3.5.0 P2-10) | `ExecutorTopBar.jsx`, `live_execution.rs` | §2.7 |
| **OKX testnet** | `ExchangeConnectorOKX`, WS 连接 OKX testnet, REST 下单, 根据 `execution_mode` 切换 (v3.5.0 P2-9) | `ws_client.rs`, `okx_rest.rs` | §2.7 |
| **ParamsPanel 热调参** | 参数 schema 加载, 数值滑块/枚举下拉/布尔开关, pending→commit/rollback (v3.5.0 P2-7) | `StrategyParamsPanel.jsx` | §8.1 |
| **凭证保险库** | AES-256-GCM, PBKDF2 1M轮(执行端)/600K轮(测试端), Zeroizing全量覆盖, .bak回滚, 独立随机机器密钥(不依赖hostname) | `credential_vault.rs`, `credential_vault_v2.rs` | §2.6 |
| **本地会话认证** | bcrypt(12轮, spawn_blocking异步), JWT HS256 24h, 刷新令牌轮换+重放检测(410 GONE), 令牌族撤销, 注册限流, DEV模式跳过认证+限速; 不代表完整账户系统 | `auth/mod.rs`, `auth_middleware.rs`, `rate_limiter.rs` | §2.8 |
| **告警引擎** | 10条默认规则全部有resolve_condition, INSERT OR IGNORE+内存指纹双重去重(TOCTOU已修复), 三阶段无锁恢复, 持久化 | `alert_engine.rs`, `AlertsPage.jsx` | §9.3 |
| **签名快照** | SHA-256签名, 5项指纹, 恢复前验签, 原子写入 | `snapshot_service.rs`, `SnapshotsPage.jsx` | §9.2 |
| **沙箱验证** | AI提案独立回放, catch_unwind+3重试, CandidateUnderperforms阻断 | `sandbox_verification.rs`, `mutation.rs` | §9.1 |
| **审批工作流** | L1/L2/L3三级, 过期自动Expired, 状态联动 | `mutation.rs`, `ApprovalPanel.jsx` | §9.4 |
| **插件市场** | Ed25519签名验证, 生产密钥强制, manifest校验 | `plugin_market.rs`, `plugin_runtime_registry.rs` | §5.3 |
| **进程间加密** | qrpc_session共享crate, AES-256-GCM+HMAC-SHA256, 临时密钥交换 | `qrpc_session/src/lib.rs` | §2.7 |
| **存储生命周期** | Permanent/Temporary/Transient三级, 500MB配额, 启动清理, DEV激进模式 | `storage_lifecycle.rs`, `backup.rs` | §7.1-§7.5 |
| **前端设计系统** | Adobe暗色面板, --ad-* CSS令牌, 响应式断点, React.memo, 空状态引导, 主路径动作分层, 专业工作区外壳 | `design-system.css`, `shared.css`, `styles.css` | §8.1-§8.10 |

### §10.2 功能间交叉依赖

```
QS编译管道 ← 策略图编辑器 → 模板库
    ↓                           ↓
Paper运行时 ← 回测引擎 ← 执行端(独立进程)
    ↓              ↓
告警引擎    签名快照/沙箱验证/审批工作流
    ↓
凭证保险库 ← 本地会话认证 → 进程间加密
    ↓
存储生命周期(所有子系统)
    ↓
前端设计系统(所有UI)
```

### §10.3 版本更新触发检查 §10 🆕

**每次 MAJOR/MINOR 版本发布时**，必须执行以下检查：

| # | 检查项 | 方法 |
|---|--------|------|
| 1 | 编译管道是否完整可用 | `cargo check --workspace` + 部署策略到执行端 |
| 2 | 策略图编辑器所有6类节点是否可拖拽/连线/配置 | 手动点击每个节点类型 |
| 3 | Paper运行时是否产生预期事件 | 启动模拟→检查 EventStreamPanel |
| 4 | 回测是否产生12项指标 | 运行双均线回测→检查 BacktestDetailPage |
| 5 | 执行端是否可部署/启动/停止 | `POST /api/executor/strategies` → start → stop |
| 6 | 凭证是否可保存/读取/删除 | CredentialPanel 完整CRUD |
| 7 | 用户是否可注册/登录/刷新token | `POST /api/auth/register` → login → refresh |
| 8 | 告警规则是否10条全部存在 | `GET /api/v1/alerts/rules` |
| 9 | 前端能力加载三级降级是否正常 | 离线→缓存→远程 |
| 10 | 模板库是否可展开/加载/应用 | 展开模板库→点击加载→画布出现节点 |
| 11 | 工作区入口是否完全由后端 capability 驱动 | 移除或降级 fixture 中某个 workspace surface/action → 前端入口隐藏或禁用 |

### §10.4 功能演进回归保护 🛡️+🔍 (v3.7.1 新增)

每次 MAJOR/MINOR 版本推进、或任何 PATCH 版本扩大能力范围时，必须执行功能演进回归保护：

| 项 | 要求 |
|----|------|
| 功能演进登记 | 新能力 / 能力扩展 / 语义变更必须写入里程碑规划 |
| 回归保护矩阵 | 必须列出受影响的既有能力和验证命令 |
| 支持矩阵同步 | `implementation-support-matrix.md` 必须与真实 capability 一致 |
| 后端真源同步 | 新 UI surface/action 必须进入 `CapabilityResponse`、OpenAPI、fixture 和 capability governance |
| 用户入口同步 | README、当前状态总览、UI 文案不得先于真实能力宣称 supported |
| 兼容性说明 | 旧图、旧策略、旧凭证、旧运行记录受影响时必须有迁移或拒绝行为 |
| 测试证据 | 新能力至少一个自动化测试；核心工作流必须有场景 / E2E / 手动验证记录 |

**门禁**: `powershell tools/check-feature-evolution.ps1`。
**审计**: closeout 报告必须记录新增能力证据和旧能力回归结果。

### §10.5 v4 演化回归保护 🛡️+🔍 (v4.0.0 新增)

v4.0.0 状态机化演化必须在 v3.7.1 稳定线之上侧向生长。每个实现阶段必须确认旧能力不回退。

| 项 | 要求 |
|----|------|
| V1 QS 保留面 | 旧 `fn strategy()` 可执行主干保持稳定；新语法不得静默扩大 V1 支持面 |
| Core IR DAG | 顶层图仍可验证无环；状态机内部复杂度不得污染顶层 DAG |
| 兼容桥 | 旧 data/intent/agent/risk/execution/fill 链路可映射为三大 machine 默认实例 |
| 事件证据 | transition、memory、cache、silence、recovery 均有事件或回放证据 |
| Risk Plane | 真实订单路径无法绕过 precheck/order_check/postcheck |
| Execution 能力矩阵 | 每个订单能力有 `provider_native` / `runtime_simulated` / `unsupported` 来源 |
| 四种实时模式 | `PaperActual` / `PaperSimulated` / `LiveActual` / `LiveSimulated` 的账户域和成交来源不可混淆 |
| UI 诚实展示 | planned/beta/restricted 不得被显示为 supported |
| 学习流水线 | MAJOR closeout 检查 owner 必学机制；个人学习记录不入 Git |

**门禁**: 能自动化的部分必须逐步纳入 `tools/check-*.ps1` 或 Rust/前端测试；未自动化前由 closeout GP 合规矩阵和自由维度审计覆盖。

**审计**: v4.0.0 每个阶段 closeout 必须列出本节各项状态、证据文件和未覆盖风险。
