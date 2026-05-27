# v4.16.0 backend.capability 子叶抽离完成记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E-02。
> 基准: `32-backend.capability单叶closeout.md`、`41-backend其余八叶模块壳抽离记录.md`。
> 判定: `backend.capability` 子叶抽离完成；只建立 `backend.capability.snapshot` facade，不改变 capability 真源。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001E 逐叶完成 | 固化 |
| 规范矩阵 | capability 真源、前端只投影 | 固化 |
| 引导矩阵 | `backend.capability.snapshot` | 扩展 |
| 模块树 | `backend.capability` | 子叶抽离完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 capability |
| 模块树节点 | `backend.capability`、`backend.capability.snapshot` |
| 真实文件 | `src/backend/capability.rs`、`src/backend/capability/snapshot.rs`、`src/capability_api.rs` |
| public 方法 | `get_capabilities`、`GET /api/capabilities` |
| 测试/门禁 | `cargo check -p quantpilot`、capability governance tests、`tools/check-matrix-governance.ps1` |

---

## 子叶抽离结果

| 子叶 | 职责 | 保留实现 |
| --- | --- | --- |
| `backend.capability.snapshot` | capability snapshot route facade | `src/capability_api.rs` |

## 等价结论

`/api/capabilities` 的真实数据源、response schema 和前端投影关系保持不变。前端仍不得用静态数组替代后端 capability 真源。
