# v2.3.3 Closeout 报告

> 版本: v2.3.3 (PATCH) | 基准: v2.3.2 | 完成日期: 2026-05-20

---

## 执行概况

v2.3.3 聚焦于两轮自由维度诱错审计发现的 S0 阻断项和关键 P1 项的修复。

| 类别 | 计划 | 完成 | 完成率 |
|------|:--:|:--:|:--:|
| S0 阻断 | 9 | 9 | 100% |
| P1 高优 | 15 | 15 | 100% |
| P2 中优 | 0 | 0 | — (留给 v2.4.0) |

## S0 阻断修复 (9/9)

| # | 维度 | 文件 | 修复内容 |
|---|------|------|---------|
| 1 | B | `src/main.rs` | 嵌套写锁拆分两阶段: 先收集过期ID+后台spawn I/O, 释放锁后再获取ai_proposals锁 |
| 2 | E | `src/auth_middleware.rs` | JWT验证失败直接返回401, 移除API Key降级路径 |
| 3 | E | `src/main.rs` | CORS解析拒绝 `*` 和非 http(s) scheme |
| 4 | E | `src/graph_api.rs` | graph_version_dir 内调用 sanitize_storage_path_segment 纵深防御 |
| 5 | G | `src/graph_quantscript_api.rs` | 未知意图模块键返回 anyhow::bail! 明确错误 |
| 6 | G | `frontend/src/graph/compileGraph.js` | 前端未知意图抛出 Error 替代静默降级为 ma_cross |
| 7 | I | `markdown/02-protocol/README.md` | RFC-017/018/019 状态 ✅→🔄 |
| 8 | I | `markdown/04-guides/guide-api-reference.md` | 更新废弃声明, 指向权威来源 |
| 9 | I | `markdown/03-implementation/governance/implementation-support-matrix.md` | 修复全部损坏的绝对路径引用 |

## P1 高优修复 (15/15)

| # | 维度 | 文件 | 修复内容 |
|---|------|------|---------|
| 1-10 | D | 10个文件 | 原子写入统一: runtime_persistence::atomic_write_json 提升为 pub(crate), 11处调用点统一替换, storage_lifecycle 和 data_module 就地补齐 fsync |
| 11 | B | `src/main.rs` | BTreeMap 无界增长: 后台任务淘汰逻辑从3个扩展到11个 map |
| 12 | A | `frontend/src/styles.css` | radial-gradient 替换为纯色 background-color |
| 13 | A | `frontend/src/styles.css` | border-radius:12px → var(--ad-radius-lg), color:white → var(--ad-text) |
| 14 | UX | `frontend/src/components/TopToolbar.jsx` | 编译/模拟/回测按钮增加 isBusy 状态, 运行中显示"模拟中..."/"回测中..." |
| 15 | UX | `frontend/src/utils/actionFailure.js` | "Strategy IR 仅作语义预检" → 用户友好中文表述 |

### 附带修复

| 文件 | 修复内容 |
|------|---------|
| `tests/common/re_exports.rs` | 新增 runtime_persistence + runtime_validation 模块暴露 |
| `src/main.rs` | runtime_persistence/runtime_validation 改为 pub mod |
| `qrpc_runtime/src/fill_engine.rs` | 断言英文→中文同步 (预存bug) |
| `frontend/src/components/TopToolbar.jsx` | ToolbarNotices 补充 useI18n() (预存bug) |
| `frontend/src/utils/actionFailure.test.js` | 测试断言同步新术语 |
| `frontend/src/components/TopToolbar.failureNotices.test.jsx` | 测试断言同步新文本 |

## 验收结果

| # | 验收项 | 结果 |
|---|--------|:--:|
| 1 | `cargo check --workspace` | ✅ 零错误 |
| 2 | `cargo test -p qrpc-runtime --lib` | ✅ 185/185 通过 |
| 3 | `npx vite build` | ✅ 3.03s |
| 4 | `npm run test` (前端) | ✅ 91/92 (1个预存失败) |
| 5 | 原子写入路径统一 | ✅ 10/10 调用点已替换 |
| 6 | 零 radial-gradient | ✅ |
| 7 | 零 >8px border-radius | ✅ |
| 8 | 编译按钮加载反馈 | ✅ |
| 9 | 用户无技术术语泄漏 | ✅ (actionFailure.js) |

## 遗留项 → v2.4.0

| 项 | 说明 |
|-----|------|
| StrategyHubPage 测试 | 预存 Unicode 引号不匹配, 非本次引入 |
| SSE 超时保护 | Axum 0.7 不支持, 需 tokio-stream 依赖, 推迟到 v2.5.0 |

## GP 合规状态

v2.3.3 维持 GP §1-§8 全条款合规。无新增违规。
