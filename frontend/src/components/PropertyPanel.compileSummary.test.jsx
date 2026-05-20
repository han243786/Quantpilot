import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, render, screen } from "@testing-library/react";
import PropertyPanel from "./PropertyPanel";
import { useGraphStore } from "../store/graphStore";
import { COMPILE_CONTRACT } from "../utils/compileContract";

describe("PropertyPanel compile source-of-truth summary", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        selectedNodeId: null,
        selectedEdgeId: null,
        graph: {
          ...useGraphStore.getState().graph,
          compile_summary: {
            compilable: false,
            last_compile_id: "compile_runtime_failure",
            backend_verified: false,
            protocol_name: "quantpilot/runtime-config/v1",
            config_hash: "cfg_runtime_failure",
            outputs: {
              data_sources: 1,
              intent_generators: 1,
              agents: 1,
              risk_controls: 1,
              executions: 1
            },
            errors: ["Runtime compile rejected the generated output."],
            warnings: [],
            diagnostics: [],
            strategy_ir_check: {
              performed: true,
              compilable: true,
              compile_id: "compile_strategy_ir_ok",
              has_core_ir: true
            },
            artifact_resolution: {
              strategy_ir_role: "semantic_preflight",
              strategy_ir_role_label: "只作语义预检，不决定可运行输出",
              runtime_source: "runtime_fallback",
              runtime_source_label: "图生成的 runtime_config 回退输入",
              source_of_truth: "runtime_compile",
              source_of_truth_label: "以 /api/runtime/compile 输出为准",
              notes: [
                "策略中间表示会先执行语义预检。它可以提前阻断编译，但不决定最终可运行输出。",
                "Formal QuantScript 代码转换不可用，因此运行时编译回退到图生成的 runtime_config；最终可运行结果仍以运行时编译输出为准。"
              ]
            }
          }
        }
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders compile summary through the structured card boundary", () => {
    render(<PropertyPanel />);

    const card = screen.getByTestId("compile-summary-card");
    const subsections = card.querySelectorAll(".property-subsection");

    expect(card).toBeInTheDocument();
    expect(subsections.length).toBeGreaterThanOrEqual(4);
    expect(card.textContent).toContain("Runtime compile rejected the generated output.");
    expect(card.textContent).toContain("compile_runtime_failure");
    expect(card.textContent).toContain("compile_strategy_ir_ok");
    expect(card.textContent).toContain("cfg_runtime_failure");
    expect(card.textContent).toContain("只作语义预检，不决定可运行输出");
    expect(card.textContent).toContain("图生成的 runtime_config 回退输入");
    expect(card.textContent).toContain(COMPILE_CONTRACT.runtimeSourceOfTruthLabel);
    expect(card.textContent).toContain(COMPILE_CONTRACT.conflictMessage);
    expect(card.textContent).toContain(COMPILE_CONTRACT.conflictHint);
  });
});
