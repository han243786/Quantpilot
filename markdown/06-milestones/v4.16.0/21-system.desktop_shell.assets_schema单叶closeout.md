# v4.16.0 system.desktop_shell.assets_schema 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`15-system.desktop_shell.tauri_config单叶closeout.md`、`18-system.desktop_shell.tauri_runtime单叶closeout.md`。
> 执行档位: 标准。
> 判定: S5 `system.desktop_shell.assets_schema` 完成单叶白箱 closeout；资产和 generated schema 只登记，不改生成物，不继续细分。

---

## 目标

本文件确认桌面资产与 Tauri generated schema 的当前边界。

本批次只登记:

1. `src-tauri/icons/*` 是桌面打包图标资产。
2. `src-tauri/gen/schemas/*` 是 Tauri generated schema 资产。
3. `src-tauri/tauri.conf.json` 仍引用现有图标路径。
4. 本批次不重新生成 schema、不替换图标、不改打包资产路径、不把 generated schema 当业务 schema 真源。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S5 完成判定 | 落地 |
| 规范矩阵 | 桌面资产 owner、generated schema 边界、业务 schema 禁止混入 | 扩展 |
| 引导矩阵 | 全量树、模块树、真实文件、资产/schema 门禁坐标 | 扩展 |
| 模块树 | `system.desktop_shell.assets_schema` | 完成 S5 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.desktop_shell.assets_schema` |
| 真实文件 | `src-tauri/icons/32x32.png`、`src-tauri/icons/128x128.png`、`src-tauri/icons/128x128@2x.png`、`src-tauri/icons/icon.ico`、`src-tauri/gen/schemas/acl-manifests.json`、`src-tauri/gen/schemas/capabilities.json`、`src-tauri/gen/schemas/desktop-schema.json`、`src-tauri/gen/schemas/windows-schema.json` |
| public 方法 | Tauri icon asset paths、Tauri generated schema files |
| 关键内部实现 | 图标资产、ACL schema、capability schema、desktop/window generated schemas |
| 测试/门禁 | JSON parse、资产存在性检查、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| 图标资产存在 | 通过 | `32x32.png`、`128x128.png`、`128x128@2x.png`、`icon.ico` 均存在 |
| generated schema 存在 | 通过 | `acl-manifests.json`、`capabilities.json`、`desktop-schema.json`、`windows-schema.json` 均存在 |
| generated schema 可解析 | 通过 | 4 个 schema 文件均通过 PowerShell `ConvertFrom-Json` |
| Tauri config 图标引用 | 通过 | `src-tauri/tauri.conf.json` 仍引用 `icons/128x128.png`、`icons/128x128@2x.png`、`icons/icon.ico` |
| 外部边界 | 通过 | 不涉及后端 API schema、业务配置 schema 或前端设计系统 |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | 桌面图标路径和 generated schema 文件已登记 |
| 兼容入口 | 完成 | 资产路径和消费方式不变 |
| generated schema | 完成 | 只登记生成物，不把生成物改成手写真源 |
| 外部边界 | 完成 | 不拥有业务 schema、API response schema、UI 设计系统或 Tauri config 语义 |
| 等价证据 | 完成 | 资产存在性、schema JSON parse 和 Tauri config 引用均已核查 |
| 继续细分 | 停止 | 图标和 generated schema 没有独立行为 owner；继续拆只会制造资产级碎片 |

---

## 父子通信规则

`system.desktop_shell.assets_schema` 只能经 `system.desktop_shell` 为桌面壳提供资产和 generated schema。它不得直接横向连接后端 schema、前端 UI 设计系统、Tauri runtime 权限语义或 release packaging。

后续如果重新生成 schema、替换图标体系或改变打包资产路径，必须重新打开 S5，并单独说明生成工具、输入来源、diff 证据和回退点。

---

## 不继续细分理由

| 候选子叶 | 不继续拆的原因 |
| --- | --- |
| icons | 只有静态资产，没有独立 public API |
| generated schemas | 生成物由 Tauri 工具链产生，不应拆成手写 owner |
| icon config relation | 已由 S4 Tauri config 登记，S5 只拥有资产存在性和路径稳定 |

---

## 禁止事项

- 不把 generated schema 当业务 schema 或后端 API schema 真源。
- 不把图标替换、品牌调整或 UI 设计系统改动混入 S5 closeout。
- 不手改 Tauri generated schema 后宣称完成。
- 不改 `src-tauri/tauri.conf.json` 的图标路径。
- 不把 S5 closeout 描述成桌面壳整体完成。

---

## 验收标准

1. S5 真实资产、generated schema、public 路径和父级通信规则已登记。
2. 资产存在性和 schema JSON parse 已通过。
3. `system.desktop_shell.assets_schema` 模块树节点标记为单叶 closeout 完成。
4. v4.16 里程碑索引、落地记录、全量树和治理门禁能发现本 closeout 缺失。
5. 本批次不改资产、不重新生成 schema、不继续细分。
