# v4.16.0 backend.test_support 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-09。
> 基准: `30-backend九叶模块壳抽离记录.md`、`05-测试资产汰换登记.md`。
> 判定: `backend.test_support` 当前不继续拆分；测试资产汰换未启动前，不删除旧测试程序。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | test support 叶子整理、测试资产汰换延后 | 扩展 |
| 规范矩阵 | 测试只证明等价，不拥有生产 owner | 固化 |
| 引导矩阵 | `backend.test_support`、测试资产汰换登记 | 固化 |
| 模块树 | `backend.test_support` | 单叶 closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 后端测试入口与根7.6 v4.16 |
| 模块树节点 | `backend.test_support` |
| 真实文件 | `src/backend/test_support.rs`、`src/api_test_scenario.rs`、`src/test_runner.rs`、`src/tests_backend.rs`、`markdown/06-milestones/v4.16.0/05-测试资产汰换登记.md` |
| public 方法 | `register_test_scenario_routes`、`TestRunner::execute`、`tests_backend.rs` integration tests |
| 测试/门禁 | `cargo check -p quantpilot`、后端 integration tests、`tools/check-matrix-governance.ps1` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | test scenario request、integration test HTTP request、test runner context |
| 输出 | test scenario response、test report、regression evidence |
| owner | `backend.test_support` 只拥有测试支撑入口，不拥有生产 handler |
| 保留实现 | `src/tests_backend.rs`、`src/test_runner.rs`、`src/api_test_scenario.rs` 均保留原位 |
| 兼容桥 | `backend.interface_boundary -> backend.test_support -> api_test_scenario::register_test_scenario_routes` |
| 回退点 | 回退到 `app_router` 直接调用 test scenario route registration |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 当前不继续拆分 |
| 原因 | 测试程序接下来会被大规模汰换，但汰换策略尚未启动；现在拆测试支撑会制造错误稳定感 |
| 触发再拆条件 | 启动测试资产汰换方案后，再按 API contract、fixture、scenario、legacy tests 分拆 |
| 暂停点 | 删除旧测试、替换 integration test、修改 test scenario route 时必须引用测试资产汰换登记 |

---

## closeout 结论

`backend.test_support` 已完成当前整理 closeout。它暂时停止细分，等待测试资产汰换方案。
