# OpenAPI 补全 + 分页契约 — 完整方案

> 3 个 Agent 并行研究 | 2026-05-12

---

## 核心发现

| Agent | 发现 |
|-------|------|
| Agent 1 (OpenAPI gap) | 6 个 A 级 gap + **10 个额外 gap** (G7-G16) + **1 个 CRITICAL bug** |
| Agent 2 (Backend trace) | **前后端字段 100% 匹配**, A1-A5 纯文档问题 |
| Agent 3 (Pagination) | 完整分页契约: offset-based, `PaginatedResponse<T>`, 8 端点 |

---

## CRITICAL Bug: SnapshotsPage 创建快照不发送 body

**Agent 1 发现**: `SnapshotsPage.jsx:35` 发送 `fetch(..., { method: "POST" })` **无 body**, 但后端 `CreateSnapshotRequest` 需要 10 个必需字段。

**需要决策**: 快照创建功能——
- **选项 A**: 前端补全 10 个字段（需从运行时状态提取 deployment_revision/capability_hash 等）
- **选项 B**: 后端改为从服务端自动提取（`POST /v1/snapshots/create` 无 body 由后端自动生成）
- **选项 C**: 如果功能暂未启用，暂缓修复

---

## A1-A6: OpenAPI Spec 补全 (Agent 1 已写精确 YAML)

| Gap | 端点 | 需添加 |
|-----|------|--------|
| A1.1 | POST /api/runtime/test-run | requestBody: actor, capability_context, runtime_config, runtime_targets |
| A1.2 | POST /api/runtime/backtest | requestBody: 同上 + backtest_options |
| A1.3 | POST /api/graphs/save | requestBody: graph, version_label?, save_note?, actor? |
| A1.4 | POST /api/runtime/compile | requestBody: runtime_config, graph_json |
| A1.5 | POST /api/strategy-ir/compile | requestBody: graph_id, compile_id, strategy_ir |
| A1.6 | POST /api/quantscript/formal/compile | requestBody: graph_id, compile_id, source, runtime_template |
| A2 | /api/credentials (全部) | 新增 GET/POST/DELETE 三个端点 |
| A3 | /api/test/scenario/run | 新增完整端点定义 |
| A4 | POST snapshots/{id}/restore | requestBody: actor_id, reason? |
| A5 | POST save 端点 | requestBody: {} (空对象) |
| A6 | 版本前缀 | 文档化说明 |

### 额外发现的 Gap (G7-G16)

| Gap | 严重度 | 说明 |
|-----|:--:|------|
| G7 | **CRITICAL** | SnapshotsPage create 无 body → 后端需要 10 字段 |
| G8 | HIGH | ChaosPage 缺 injection 字段 |
| G9 | HIGH | AlertsPage acknowledge 缺 requestBody |
| G10 | MEDIUM | experiments/backtest-sweep 缺 requestBody |
| G11 | MEDIUM | backtests/compare 缺 requestBody |
| G12 | MEDIUM | graph/versions/restore 缺 requestBody |
| G13 | LOW | hotswap POST 缺 requestBody |
| G14 | **HIGH** | GET /api/hotswap 路径错误 → 应为 /api/hotswap/list |
| G15 | LOW | /api/runtime/compile 缺响应 schema |
| G16 | LOW | hotswap/list 路径在 spec 中缺失 |

---

## C2: 分页契约 (Agent 3 设计)

### 响应格式
```json
{ "data": [...], "total": 42, "limit": 20, "offset": 0 }
```

### 请求参数
| 参数 | 默认 | 最大 | 说明 |
|------|:--:|:--:|------|
| limit | 20 | 100 | 每页记录数 |
| offset | 0 | — | 跳过数量 |

### 后端需修改文件

| 文件 | 改动 |
|------|------|
| `src/frontend_api_types.rs` | 新增 `PaginationQuery`, `PaginatedResponse<T>`, `paginate()` |
| `src/runtime_api.rs` | 5 个 list handler 接入分页 |
| `src/graph_api.rs` | `list_graphs` 接入分页 |
| `src/alert_engine.rs` | `list_alerts` 接入分页 |
| `src/snapshot_service.rs` | `list_snapshots` 接入分页 |
| `contracts/openapi/root.yaml` | 新增 `PaginatedResponse` schema + 8 端点 update |

### 实施阶段
1. 共享类型 (frontend_api_types.rs)
2. 参考端点 (list_backtests)
3. 其余运行时端点
4. Graph/Alert/Snapshot
5. OpenAPI spec
6. 前端消费方接入

---

## 需要你的决策

**1. CRITICAL: SnapshotsPage 创建快照 bug (G7)**

前端不发送 body，后端需要 10 个必需字段。选择方案？

**2. OpenAPI YAML 补全方式**

Agent 1 已写出精确 YAML。可以直接编辑 `contracts/openapi/root.yaml`。确认执行？

**3. 分页契约确认**

Agent 3 的设计: offset-based, `{data, total, limit, offset}`, 默认 20/最大 100。这是最终 API breaking change——所有列表端点响应从裸数组变为包装对象。确认？
