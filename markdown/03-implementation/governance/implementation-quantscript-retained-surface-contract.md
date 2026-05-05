# QuantScript 保留面合约

此文件是 `CL-P1-005` 的活跃措辞边界。

## 目标

保持正式 QuantScript `V1` 严格、诚实，并仅限于保留的可执行主干。

## 保留的源类别

- 正面保留示例位于 `quantscript/authoring_samples/`
- 有意的保留边界失败位于 `quantscript/boundary_samples/`
- crate 测试内部仅用于兼容性的解析器示例不得被描述为面向发布的编写示例
- 当解析器兼容性测试涉及保留可执行主干之外的语法时，必须自称为仅兼容性

## 保留的可执行真实结果

- 解析器接受不是产品支持
- 受支持的产品入口点仍然包括：
  - `analyze_formal_quant_script(...)`
  - `parse_formal_quant_script_config(...)`
  - `/api/quantscript/formal/compile`
- 不支持的构造应通过稳定的 `QS06xx` 或 `QPQSLOWxxx` 诊断失败，而非落入模糊的后期阶段错误

## 收口规则

- 不要将负面的边界 fixture 保留在活跃的编写示例文件夹内
- 不要将仅解析器的旧语法描述为已接纳的可执行主干的一部分
- 不要将解析器接受、测试通过状态或 fixture 存在性用作面向发布的编写支持的简写
- 在没有在活跃文档中书面更新合约的情况下，不要扩大保留面

## 当前实现锚点

- `quantscript/QUANTSCRIPT_SUPPORTED_SURFACE.md`
- `quantscript/QUANTSCRIPT_REAL_STRATEGY_AUTHORING_TRIAL.md`
- `markdown/guides/quantscript/guide-formal-quantscript-syntax.md`
- `markdown/guides/quantscript/guide-v1-freeze-descope-checklist.md`
- `tests/quantscript_real_strategy_authoring.rs`
