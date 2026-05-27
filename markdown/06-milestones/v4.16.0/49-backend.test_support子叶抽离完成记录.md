# v4.16.0 backend.test_support 子叶抽离完成记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E-08。
> 基准: `39-backend.test_support单叶closeout.md`、`41-backend其余八叶模块壳抽离记录.md`。
> 判定: `backend.test_support` 子叶抽离完成；只建立 test scenario route facade，不删除旧测试程序，不启动测试资产汰换。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001E 逐叶完成、测试资产汰换延后 | 固化 |
| 规范矩阵 | 测试只证明等价、不拥有生产 owner | 固化 |
| 引导矩阵 | `backend.test_support.scenario`、测试资产汰换登记 | 扩展 |
| 模块树 | `backend.test_support` | 子叶抽离完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 test support、根7.6 v4.16 |
| 模块树节点 | `backend.test_support`、`backend.test_support.scenario` |
| 真实文件 | `src/backend/test_support.rs`、`src/backend/test_support/scenario.rs`、`src/api_test_scenario.rs`、`src/test_runner.rs`、`src/tests_backend.rs`、`markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md` |
| public 方法 | `register_test_scenario_routes`、`TestRunner::execute`、integration tests |
| 测试/门禁 | 后端 integration tests、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 子叶抽离结果

| 子叶 | 职责 | 保留实现 |
| --- | --- | --- |
| `backend.test_support.scenario` | test scenario route facade | `src/api_test_scenario.rs` |

## 等价结论

测试 route、legacy integration tests、test runner 和测试资产汰换登记均保持原位。本批不删除旧测试程序。
