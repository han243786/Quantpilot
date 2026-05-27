# v4.16.0 system.runtime_profile.config_examples 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`13-递归模块化全局根流程.md`。
> 执行档位: 重型。
> 判定: S10 `system.runtime_profile.config_examples` 完成单叶白箱 closeout；不改环境变量样例、runtime protocol 样例或 strategy_ir schema/example，不进入整理或重构。

---

## 目标

本批次确认 `system.runtime_profile.config_examples` 作为运行配置样例叶子，是否能稳定承载环境变量模板、runtime protocol 示例和 strategy_ir v0 schema/example。

本批次不修改 `.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json` 或 `config/strategy_ir.v0.example.json`，只记录真实职责、入口、保留外部边界、等价证据和停止条件。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 单叶 closeout、递归模块化 R2/R3 | 落地 |
| 规范矩阵 | 配置样例 owner、schema/example 兼容、runtime 真源边界 | 落地 |
| 引导矩阵 | `system.runtime_profile.config_examples` 真实文件、入口、门禁 | 扩展 |
| 模块树 | `system.runtime_profile.config_examples` | 完成 S10 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.runtime_profile.config_examples` |
| 真实文件 | `.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json`、`config/strategy_ir.v0.example.json` |
| public 方法 | 环境变量示例入口、runtime protocol 示例入口、strategy_ir v0 schema/example 入口 |
| 关键内部实现 | `QUANTPILOT_DEV`、`QUANTPILOT_LOG_FORMAT`、JWT/API key 示例、storage/executor/proxy 示例、runtime protocol generators/agents/global_risk/runtime_mode、strategy_ir v0 schema/example |
| 测试/门禁 | JSON parse、样例文件存在性、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 真实行为基线

| 文件 | 当前职责 | 等价口径 |
| --- | --- | --- |
| `.env.example` | 环境变量模板，覆盖 dev/log/security/rate limit/storage/executor/proxy 示例 | 不改变默认运行配置语义 |
| `config/runtime_protocol.example.yaml` | runtime protocol 示例，包含 generators、agents、global_risk、runtime_mode | 只作为示例，不作为 runtime 行为真源 |
| `config/strategy_ir.v0.schema.json` | strategy_ir v0 schema | schema 字段变化必须单独登记 |
| `config/strategy_ir.v0.example.json` | strategy_ir v0 example | example 必须与 schema 语义保持可解释 |

---

## public/内部实现分类

| 分类 | 内容 | 处理 |
| --- | --- | --- |
| public 入口 | `.env.example`、runtime protocol 示例、strategy_ir schema/example | 文件路径和示例入口不变 |
| 兼容 public 入口 | 文档、开发者复制 `.env.example`、工具读取 schema/example 的路径 | 抽离阶段不得删除、重命名或搬迁 |
| 关键内部实现 | 环境变量键、runtime protocol 样例结构、strategy_ir schema/example 内容 | 只登记，不改内容 |
| 保留外部边界 | runtime 行为真源、编译器真源、后端 capability 真源、执行端状态 | S10 不拥有这些 owner |

---

## 等价证据

| 证据 | 结果 |
| --- | --- |
| `.env.example` 文件存在 | 通过 |
| `config/runtime_protocol.example.yaml` 文件存在 | 通过 |
| `config/strategy_ir.v0.schema.json` JSON parse | 通过 |
| `config/strategy_ir.v0.example.json` JSON parse | 通过 |
| 环境变量示例覆盖 | `QUANTPILOT_DEV`、`QUANTPILOT_LOG_FORMAT`、JWT/API key、storage、executor、trusted proxy |
| runtime protocol 示例覆盖 | generators、agents、global_risk、runtime_mode |
| 治理门禁 | `tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

本批次未执行 runtime protocol YAML 的语义加载，也未执行 strategy_ir schema validation。后续如要改变 schema 字段、runtime protocol 结构或默认环境变量语义，必须单独做契约验证。

---

## closeout 判定

| 项 | 判定 |
| --- | --- |
| 单叶抽离状态 | S10 完成白箱 closeout |
| 是否改配置样例 | 否 |
| 是否改变 runtime 行为 | 否 |
| 是否继续细分 | 暂不值得继续细分 |
| 是否进入整理/重构 | 否 |
| 后续动作 | 仅在 schema 字段、runtime protocol 结构、默认环境变量或示例路径变化时重新打开 |

---

## 不继续细分的理由

`system.runtime_profile.config_examples` 当前是配置样例和 schema/example 资产集合。它们有稳定路径和引用价值，但没有独立运行时 owner；继续拆成环境变量、runtime protocol、strategy_ir schema 三个 L3 叶子，会提前制造配置字段级治理碎片。

因此 S10 作为叶子停止细分。

---

## 禁止事项

- 不把 `.env.example` 当作真实运行配置。
- 不把 runtime protocol 示例当作 runtime 行为真源。
- 不把 strategy_ir example 当作编译器真源。
- 不改变 schema 字段或默认环境变量语义。
- 不把 S10 closeout 描述成 runtime profile 全部重构完成。

---

## 验收标准

1. S10 的 public 样例入口和兼容入口已登记。
2. S10 的关键内部样例和保留外部边界已登记。
3. S10 的 JSON parse 等价证据已登记。
4. S10 明确不继续细分。
5. 后续 system 推进可转向 S7 desktop build scripts。
