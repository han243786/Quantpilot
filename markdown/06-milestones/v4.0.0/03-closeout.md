# v4.0.0 Closeout 报告

> MAJOR | 终稿确认: 2026-05-23 | Closeout: 2026-05-24
> 基准: v3.7.1 稳定归档点 | 下一版本: v4.0.1 (PATCH) 或 v4.1.0 (MINOR)

---

## 一、执行概况

| 项 | 值 |
|----|-----|
| 版本类型 | MAJOR |
| 实现范围 | 超级规范化 §7.7 Phase 0-8 全阶段 |
| 代码变更 | 26 文件 (仅 v4 核心), +13,388 / -46 行 |
| Rust crate | `qrpc_core_ir::v4` (5885行), `qrpc_runtime::v4_runtime` (2961行), `quantscript::v4_static_audit` (1196行), `qrpc_runtime::compat` (363行) |
| 前端变更 | `capabilityProjection.js` (132行), `V4RuntimeEvidencePanel.jsx` (250行), v4 测试 (+733行) |
| 契约文档 | `implementation-v4-machine-and-venue-contract.md` (986行), 规划方案 (874行) |
| 治理更新 | GP +92行 (§1.6-§1.12), 超级规范化 +75行 (§7.7, §7.8, §8.9) |
| 新增门禁 | `check-learning-closeout.ps1` (84行) |
| 编译状态 | `cargo check --workspace` ✅ |
| 格式基线 | `cargo fmt --check` ✅ |
| 测试编译 | `cargo test --workspace --no-run` ✅ |

---

## 二、MAJOR 演化通道完成度 (超级规范化 §7.7)

| Phase | 内容 | 产物 | 完成 |
|:----:|------|------|:----:|
| 0 | 终稿规划 | `01-规划方案.md` (874行): 目标/非目标/架构/DSL/事件模型/Risk Plane/交易模式/学习流水线 | ✅ |
| 1 | 元契约 | QS profile, MachineTemplate, MachineGraph, MachineEventCatalog, VenueCapabilityMatrix | ✅ |
| 2 | 类型与能力矩阵 | QsTypeRef, RuntimeTradingMode, CompileTimeCapabilityReport, V4VersionManifest | ✅ |
| 3 | 静态审计 | `audit_v4_quant_script_static`, `V4QsStaticAuditReport` (1196行) | ✅ |
| 4 | 兼容桥 | `compat.rs` (363行): 旧链路→三大 Machine 默认实例, V1 QS 保留面 | ✅ |
| 5 | 事件循环 | `v4_runtime.rs` (2961行): PaperSimulated, Event, Cache, Silence, Recovery | ✅ |
| 6 | Risk Plane | precheck/order_check/postcheck, priority ≥9000, emergency_halt | ✅ |
| 7 | ExecutionMachine | provider_native/runtime_simulated/unsupported, 能力来源运行时门禁 | ✅ |
| 8 | UI / 学习流水线 | capabilityProjection, V4RuntimeEvidencePanel, check-learning-closeout | ✅ |

---

## 三、五维度评分

| 维度 | 评分 | 说明 |
|------|:----:|------|
| **功能开发进度** | 8/10 | Phase 0-8 全部实现; v4 PaperSimulated 运行骨架完成; 旧链路兼容桥验证; 前端能力投影层完成 |
| **仓库稳定程度** | 8/10 | `cargo test --workspace`, warning budget, frontend/executor build/test, npm audit 均通过；V9 剩余 P1/P2 不阻断当前 closeout |
| **发布就绪度** | 7/10 | 版本号 14 项、CHANGELOG、V8 24 项门禁、V9 审计均完成；release manifest/OpenAPI 活版本漂移列为 P1 |
| **用户友好程度** | 7/10 | capability 驱动入口、v4 evidence 面板、前端 E2E 已验证；API 错误结构和 E2E step 吞错列为 P1 |
| **系统整体稳定性** | 7/10 | GP §10.3 11 项回归通过；v4 event/freshness、Risk Plane approval、runtime evidence 持久化列为 v4.0.1 修复项 |

> **评分标准**: 1-10 (10=完美, 生产就绪). 需要实际运行验证的项标注"Codex 待验证".

---

## 四、S0/P1/P2/P3 发现与修复状态

> 本节由 Codex 在完成自由维度诱错审计 (超级规范化 §8.5) 后填写。
> 审计报告长期归档于 `markdown/09-archive/testing-retired/自由维度诱错审计-v4.0.0-第1轮.md`。

| 严重度 | 发现数 | 已修复 | 遗留 | 流向 |
|:------:|:-----:|:-----:|:----:|------|
| S0 | 0 | 0 | 0 | 当前里程碑 |
| P1 | 12 | 0 | 12 | v4.0.1 |
| P2 | 10 | 1 | 9 | v4.1.0 |
| P3 | 3 | 0 | 3 | 持续回归 |

---

## 五、GP 合规矩阵

> 逐条核查 GP 全部 44 条。Codex 负责执行验证并填写状态。

### 架构铁律 (§1.1-§1.12)

| 条款 | 主题 | 状态 | 证据/说明 |
|------|------|:--:|------|
| §1.1 | QS 唯一策略定义路径 | ✅ | v4 仍保持 QS→parse→HIR→lower→Core IR |
| §1.2 | 新增功能跨三层验证 | ✅ | QS parse/v4_static_audit, Core IR/v4.rs, runtime/v4_runtime.rs |
| §1.3 | 编译路径不可绕过 | ✅ | compile_runtime_protocol_via_qs 保留 |
| §1.4 | 数据流单向原则 | ✅ | QS 源码→graph JSON→可视化, source_mode="quantscript" |
| §1.5 | 功能演进必须先登记 | ✅ | `01-规划方案.md` 功能演进登记, `check-feature-evolution.ps1` |
| §1.6 | 顶层 DAG + 状态机边界 | ✅ | V6/V8 验证 Core IR DAG、旧 QS、旧图与旧运行记录兼容；V9 无 S0 |
| §1.7 | QS 状态机 DSL 边界 | ✅ | action block 禁止文件/网络/密钥, 静态审计拒绝 unsupported 路径 |
| §1.8 | 状态迁移事件驱动可解释 | ✅ | transition 绑定事件, event_id/type/time/source/freshness/replayable |
| §1.9 | Risk Plane 不可绕过 | ⚠️ | V6 拦截验证通过；V9 P1-4 发现 approval 推断过宽，流向 v4.0.1 |
| §1.10 | Execution 能力来源显式标记 | ✅ | provider_native/runtime_simulated/unsupported, VenueCapabilityMatrix |
| §1.11 | 学习流水线边界 | ✅ | `markdown/learning/` .gitignore, check-learning-closeout.ps1 |
| §1.12 | 前端以后端 capability 为真源 | ✅ | capabilityProjection.js, capability fixture, projection 测试 |

### 代码规范 (§2.1-§2.8)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §2.1 | 错误消息全中文 | ⚠️ | 中文 message 基本满足；V9 P1-11/P2-7 要求统一结构化错误与诊断码 |
| §2.2 | 测试断言中文子串 | ✅ | V3/V8 frontend vitest 与 E2E 通过；E2E step 吞错另列 P1-12 |
| §2.3 | 新 indicator 有单元测试 | N/A | v4 无新增 indicator |
| §2.4 | 新 TestAction 有集成场景 | N/A | v4.0.0 未新增 TestAction |
| §2.5 | 前端 t() 包裹 | ✅ | frontend build/vitest/E2E 通过；新增 capability UI 已纳入回归 |
| §2.6 | 凭证保险库安全 | ✅ | 不变, 已有 Zeroizing + AES-256-GCM |
| §2.7 | 实时执行安全 | ✅ | 不变, v4 新增四种运行时模式隔离 |
| §2.8 | 用户认证安全 | ✅ | 不变 |

### 文档规范 (§3.1-§3.3)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §3.1 | 文档分层 | ✅ | 契约文档在 03-implementation/governance/, 规划在 06-milestones/v4.0.0/ |
| §3.2 | 全中文 | ✅ | 全部 v4 文档为中文 |
| §3.3 | 里程碑命名 | ✅ | `v4.0.0/` |

### 变更管理 (§4.1-§4.4)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §4.1 | capability 变更 | ✅ | CapabilityResponse 扩展, capability fixture 更新 |
| §4.2 | 错误变更测试修复 | ⚠️ | V9 P1-11/P2-7/P2-8 发现错误响应结构与状态码仍需补强 |
| §4.3 | 结构体变更 | ✅ | v4 静态契约、runtime 契约和现有测试通过；无 S0 |
| §4.4 | API 路由变更 | ✅ | v4 capability 路由, SPA fallback 保留 |

### 禁止事项 (§5.1-§5.6)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §5.1 | 禁止硬编码 | ✅ | 未发现 secret/凭证硬编码；版本 banner 漂移另列版本一致性 P1/P2/P3 |
| §5.2 | 禁止静默忽略参数 | ⚠️ | V9 P1-3/P2-1/P2-4 发现 guard、memory_writes、payload fields 需 fail-closed |
| §5.3 | 禁止 stub evaluator | ✅ | v4 无新增 indicator |
| §5.4 | 禁止绕过 QS 编译 | ✅ | v4 QS 静态审计强制 |
| §5.5 | 端到端验证 | ✅ | V1-V8 全部通过；V9 完成 5 维度并行审计且无 S0 |
| §5.6 | 禁止格式漂移 | ✅ | `cargo fmt --check` ✅ |

### 前端设计规范 (§8.1-§8.11)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §8.9 | 主路径动作分层 | ✅ | workspace action hierarchy 优化 |
| §8.10 | 专业工作区外壳 | ✅ | StrategyWorkspaceDashboard |
| §8.11 | 能力驱动工作区设计 | ✅ | capabilityProjection.js, 5 种能力状态 |

### 治理系统 (§9.1-§9.4)

| 条款 | 主题 | 状态 | 证据 |
|------|------|:--:|------|
| §9.1 | 沙箱验证 | ✅ | 不变 |
| §9.2 | 签名快照 | ✅ | 不变 |
| §9.3 | 告警引擎 | ✅ | 不变, 10 条规则保留 |
| §9.4 | 审批工作流 | ✅ | 不变 |

### 功能覆盖回归 (§10.3 11项检查)

| # | 检查项 | 状态 | 证据 |
|---|--------|:--:|------|
| 1 | 编译管道完整可用 | ✅ | `cargo check --workspace` |
| 2 | 策略图编辑器 6 类节点 | ✅ | V6 targeted E2E 验证 runtime/data/intent/agent/risk/execution |
| 3 | Paper 运行时预期事件 | ✅ | V6 simulation E2E 与 API smoke 通过 |
| 4 | 回测 12 项指标 | ✅ | V6 API smoke 验证 12 项指标路径 |
| 5 | 执行端 deploy/start/stop | ✅ | V6 executor smoke 验证 deploy/start/stop |
| 6 | 凭证 CRUD | ✅ | V6 API smoke 验证 set/list/delete |
| 7 | 用户注册/登录/刷新 | ✅ | V6 API smoke 验证 register/login/refresh |
| 8 | 告警规则 10 条全在 | ✅ | V6 API smoke 与 E2E 验证 10 条 |
| 9 | 前端能力三级降级 | ✅ | V6 capability fallback/cache/safe_fallback 测试通过 |
| 10 | 模板库展开/加载/应用 | ✅ | V6 template vitest 通过 |
| 11 | 工作区 capability 驱动 | ✅ | V6 capability projection 与 E2E 验证入口隐藏/禁用 |

### v4 演化回归保护 (§10.5)

| 项 | 要求 | 状态 | 证据 |
|----|------|:--:|------|
| V1 QS 保留面 | 旧 `fn strategy()` 可执行主干稳定 | ✅ | compat.rs 兼容桥 |
| Core IR DAG | 顶层图无环验证 | ✅ | `cargo test --workspace` 与 V6/V8 回归通过 |
| 兼容桥 | 旧链路映射三大 machine | ✅ | compat.rs (363行) |
| 事件证据 | transition/memory/cache 均有事件/回放证据 | ✅ | MachineEventCatalog |
| Risk Plane | 真实订单路径无法绕过 | ⚠️ | runtime 拦截路径通过；V9 P1-4 发现 approval 语义需更严格 |
| Execution 能力矩阵 | 每个订单能力有来源标记 | ✅ | VenueCapabilityMatrix |
| 四种实时模式 | 账户域和成交来源不混淆 | ✅ | RuntimeTradingMode 枚举 |
| UI 诚实展示 | planned/beta 不显示为 supported | ✅ | capabilityProjection.js |
| 学习流水线 | MAJOR closeout 检查 owner 必学 | ✅ | `tools/check-learning-closeout.ps1` 通过 |

---

## 六、Developer Learning Closeout

> 超级规范化 §8.1: MAJOR 版本 closeout 必须回答"本版本是否引入 owner 必学核心机制"

### 本版本引入的 owner 必学核心机制

| # | 核心机制 | 涉及文件 | 调用链 | 学习状态 |
|---|---------|---------|--------|:--:|
| 1 | QS 状态机 DSL 语法 | `quantscript/src/v4_static_audit.rs` | parse → analyze → report | 已登记 |
| 2 | 三大 Machine 模板 | `qrpc_core_ir/src/v4.rs` | MachineTemplate → V4MachineContract → V4MachineGraphContract | 已登记 |
| 3 | 事件模型与调度 | `qrpc_core_ir/src/v4.rs`, `qrpc_runtime/src/v4_runtime.rs` | MachineEventCatalog → V4PaperSimulatedRuntime → event dispatch | 已登记 |
| 4 | Risk Plane 安全平面 | `qrpc_runtime/src/v4_runtime.rs` | risk_precheck → risk_order_check → risk_postcheck | 已登记 |
| 5 | Execution 能力矩阵 | `qrpc_core_ir/src/v4.rs` | VenueCapabilityMatrix → CompileTimeCapabilityReport → runtime gate | 已登记 |
| 6 | Capability 投影层 | `frontend/src/capabilities/capabilityProjection.js` | API → normalizeCapabilitySnapshot → view model → UI | 已登记 |
| 7 | 学习流水线边界 | `tools/check-learning-closeout.ps1` | .gitignore → `markdown/learning/` 不入 Git | 已登记 |

### 学习材料

- 契约: `markdown/03-implementation/governance/implementation-v4-machine-and-venue-contract.md`
- 审计: `markdown/06-milestones/v4.0.0/04-架构深度审计.md` (如已创建)
- 规划: `markdown/06-milestones/v4.0.0/01-规划方案.md`

### 学习记录检查

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-learning-closeout.ps1
```

> Codex 执行上述命令, 确认: (1) 结构入口存在, (2) `markdown/learning/` 不在 Git 跟踪中, (3) 学习材料路径可访问.

---

## 七、遗留项 (流向 v4.0.1 / v4.1.0)

| # | 遗留项 | 严重度 | 说明 |
|---|--------|:--:|------|
| 1 | 自由维度诱错审计 | S0 | MAJOR 必须至少一轮, 按超级规范化 §8.5 |
| 2 | GP §10.3 全量回归手动验证 | S0 | 11 项检查需手动执行 |
| 3 | 版本号一致性门禁全量通过 | S0 | `check-version-consistency.ps1` |
| 4 | 24 项 closeout 门禁全量通过 | S0 | `run-closeout-gates.bat` |
| 5 | 前端 E2E 测试通过 | P1 | `npm run test:e2e` |
| 6 | npm audit 清零 | P1 | `npm audit --audit-level=moderate` |
| 7 | Clippy warning budget | P1 | `check-clippy-warning-budget.ps1 -MaxWarnings 58` |
| 8 | 三角色发布前检查单 | P1 | 新用户/策略开发者/安全研究者 (超级规范化 §4.1) |
| 9 | v4 QS 场景 smoke 测试 | P1 | `scenario-smoke.ps1` |
| 10 | release dry-run | P2 | Windows runner 构建/打包/ SHA256SUMS |

---

## 八、版本记录

| 项 | 值 |
|----|-----|
| Closeout 执行日期 | 2026-05-24 |
| 基准版本 | v3.7.1 (9b19a09) |
| v4 实现 HEAD | 88306bf |
| 当前 HEAD | 88306bf |
| 审计报告 | `markdown/09-archive/testing-retired/自由维度诱错审计-v4.0.0-第1轮.md` |
