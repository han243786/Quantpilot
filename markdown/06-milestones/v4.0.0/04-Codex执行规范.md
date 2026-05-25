# v4.0.0 Codex 执行规范 — 约束验收清单

> 本文件是 Claude (文档约束方) 对 Codex (实现方) 的精确执行指令.
> 每条指令包含: 命令/检查内容/通过标准/阻断级别.
> Codex 不得跳过、不得简化、不得自行判断某项不重要.

---

## 前置条件

Codex 必须先完成以下 2 项, 才能开始后续验证:

### P0-1: 版本号统一

**依据**: `02-版本号统一规范.md`

**验证命令**:
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-version-consistency.ps1
```

**通过标准**: 脚本输出全部 PASS, 无 FAIL.

**阻断**: S0 — 版本不一致禁止继续.

---

### P0-2: CHANGELOG 插入

**依据**: `02-CHANGELOG条目规范.md`

**验证**: `grep "v4.0.0" CHANGELOG.md` 有输出, 且条目在 v3.7.1 之前.

**阻断**: S0 — CHANGELOG 缺失禁止 closeout.

---

## 验证清单

### V1: `cargo test --workspace` 全量通过

```powershell
cargo test --workspace
```

- **通过标准**: 0 FAILED
- **阻断**: S0
- **输出**: 粘贴最后 5 行 (含 test result)

---

### V2: `cargo clippy` warning budget

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-clippy-warning-budget.ps1 -MaxWarnings 58
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-executor-warning-budget.ps1 -MaxWarnings 0
```

- **通过标准**: 两项均通过 (warning 数 ≤ 预算)
- **阻断**: S0

---

### V3: 前端构建 + 测试 + E2E + npm audit

```powershell
cd frontend
npm run build
npm run test
npm run test:e2e
npm audit --audit-level=moderate
```

- **通过标准**: 全部通过, npm audit 无 moderate+ 漏洞
- **阻断**: S0 (build/test), P1 (audit)

---

### V4: 执行端前端构建 + 执行端编译/测试

```powershell
cd frontend-executor
npm run build
cd ..
cargo check --bin executor
powershell scripts\test.ps1 test --bin executor
```

- **通过标准**: 全部通过
- **阻断**: S0

---

### V5: QS 场景 smoke

```powershell
powershell scripts\scenario-smoke.ps1
```

- **通过标准**: 全部场景通过
- **阻断**: S0 (closeout 阻断)

---

### V6: GP §10.3 功能覆盖回归检查 (11 项)

**注意**: 以下 11 项检查, 部分需要手动, 部分可自动化. Codex 必须逐项执行并在 closeout 报告中记录结果.

| # | 检查项 | 方法 | 通过 | 备注 |
|---|--------|------|:--:|------|
| 1 | 编译管道 | `cargo check --workspace` | _ | |
| 2 | 策略图 6 类节点 | 手动点击每种节点类型 | _ | |
| 3 | Paper 运行时 | 启动模拟 → 检查 EventStreamPanel | _ | |
| 4 | 回测 12 指标 | 运行双均线回测 → 检查详情 | _ | |
| 5 | 执行端 deploy/start/stop | POST 策略 → start → stop | _ | |
| 6 | 凭证 CRUD | CredentialPanel 完整 CRUD | _ | |
| 7 | 用户注册/登录/刷新 | POST register → login → refresh | _ | |
| 8 | 告警规则 10 条 | GET /api/v1/alerts/rules | _ | |
| 9 | 前端能力降级 | 离线→缓存→远程 三级 | _ | |
| 10 | 模板库 | 展开→加载→应用 | _ | |
| 11 | 工作区 capability 驱动 | 降级 fixture → 前端入口隐藏 | _ | |

**阻断**: 任何一项失败 → S0, 必须修复后重新执行全部 11 项.

---

### V7: Developer Learning Closeout

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-learning-closeout.ps1
```

- **通过标准**: 脚本通过, 确认: (1) 结构入口存在, (2) `markdown/learning/` 不入 Git
- **阻断**: S0 (MAJOR 强制)

---

### V8: 全量 closeout 门禁

```powershell
.\tools\run-closeout-gates.bat
```

- **通过标准**: 24 项全部通过
- **阻断**: S0

---

### V9: 自由维度诱错审计 (至少 5 维度)

**依据**: 超级规范化 §3.2, §8.5

Codex 必须使用 Codex 的多 Agent 能力, 按以下 5 维度并行执行审计:

| Agent | 维度 | 审计范围 |
|-------|------|---------|
| A | 逻辑/契约/GP | v4 新增类型定义、状态机契约、静态校验、GP 合规 |
| B | 并发/竞态 | v4 runtime 事件循环、Risk Plane 锁顺序、compat 桥并发安全 |
| C | 边界/数值 | v4 事件字段、transition 条件、Risk Plane 拦截、Execution 能力矩阵 |
| D | 持久化 | v4 runtime evidence 存储、事件回放、memory snapshot、compat 迁移 |
| E | API/错误 | v4 capability 端点、错误消息中文、诊断代码、状态码 |

**每个 Agent 必须产出**: 发现清单 (S0/P1/P2/P3), 标注: 严重度、文件:行号、触发条件、修复建议、违反的 GP 条款.

**Agent 产出验证**: 每个执行代码修改的 Agent 必须在报告完成前运行 `cargo check --workspace` 确认 0 错误.

**审计报告**: 合并输出到 `markdown/05-testing/自由维度诱错审计-v4.0.0-第1轮.md`.

**阻断**: S0 发现必须当前修复, 修复后重新验证全部门禁.

---

### V10: closeout 报告定稿

基于 V1-V9 的执行结果, Codex 更新 `03-closeout.md`:

1. 填写"三、五维度评分"中所有 `_/10` 项的实际评分
2. 填写"四、S0/P1/P2/P3 发现与修复状态"表
3. 将 GP 合规矩阵中所有 `_` 状态更新为 `✅` 或 `❌`, 带证据
4. 填写"六、Developer Learning Closeout"的学习状态
5. 更新"七、遗留项"中已修复项的状态
6. 更新"八、版本记录"中的最终 HEAD

---

## 执行顺序

```
P0-1 (版本号) → P0-2 (CHANGELOG) → V1 (测试) → V2 (clippy) → V3 (前端)
    → V4 (执行端) → V5 (QS smoke) → V6 (回归检查) → V7 (学习检查)
    → V8 (全量门禁) → V9 (诱错审计) → V10 (closeout 定稿)
```

- P0-1 和 P0-2 必须先完成, 其余按顺序
- 任何 V 项失败必须修复后重跑该项, 不跳过后面的项
- V8 全量门禁应在 V1-V7 全部通过后执行

---

## 禁止事项

1. **禁止跳过任何验证项**. 即使"觉得没问题"也必须执行.
2. **禁止修改 closeout 报告结构**. 只能填写空白 (标注 `_`) 的字段.
3. **禁止删除或弱化任何 GP 条款**.
4. **禁止在 V9 审计中只执行 <5 维度**.
5. **禁止在未完成 P0-1 版本号统一的情况下运行任何验证**.
6. **禁止将 `markdown/learning/` 中的文件加入 Git**.
