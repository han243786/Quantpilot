# v4.16.0 backend.capability 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-02。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.capability` 当前不继续拆分；能力真源保持后端拥有，前端只做投影。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | backend 九叶 closeout | 扩展 |
| 规范矩阵 | capability 真源、前端投影边界 | 固化 |
| 引导矩阵 | `backend.capability`、`frontend.capability_projection` | 固化 |
| 模块树 | `backend.capability` | 单叶 closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 capability 与根5 capability projection |
| 模块树节点 | `backend.capability` |
| 真实文件 | `src/backend/capability.rs`、`src/capability_api.rs`、`frontend/src/capabilities/capabilityProjection.js`、`frontend/src/capabilities/capabilityGovernance.js` |
| public 方法 | `get_capabilities`、`GET /api/capabilities` |
| 测试/门禁 | `cargo check -p quantpilot`、`powershell tools/check-capability-governance.ps1`、`cd frontend && npm run test -- --run src/capabilities/capabilityProjection.test.js` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | 后端 capability 声明、权限边界、runtime mode 支持状态 |
| 输出 | capability snapshot、前端 capability projection 输入 |
| owner | 后端是真源；前端只做 projection |
| 保留实现 | `src/capability_api.rs` 仍保留 handler 与 response schema |
| 兼容桥 | `backend.interface_boundary -> backend.capability -> capability_api::get_capabilities` |
| 回退点 | 回退到 `app_router` 直接调用 `get_capabilities` |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 不继续拆分 |
| 原因 | 当前叶子只有单一 public API 和清晰 owner，继续拆会增加文档和调用噪声 |
| 触发再拆条件 | capability schema、permission boundary、market data/provider 能力变成独立 owner 时另起提案 |

---

## closeout 结论

`backend.capability` 已完成当前整理 closeout。它可以作为稳定叶子使用，后续只在 capability 真源扩大时重新评估细分。
