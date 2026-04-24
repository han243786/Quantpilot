import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "@testing-library/react";
import { useGraphStore } from "./graphStore";

describe("graphStore exportRuntimeConfig", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("reuses backend-verified runtime config from compileCurrentGraph", async () => {
    const compiled = {
      compile_summary: { compilable: true, errors: [] },
      runtime_config: {
        metadata: {
          graph_id: "graph_backend",
          compile_id: "compile_backend",
          name: "Backend Graph",
          version: "1.0.0",
          mode: "paper"
        }
      },
      runtime_targets: {
        source_to_node: { data_backend: "data_node" },
        runtime_node_id: "runtime_node",
        execution_node_id: "execution_node"
      }
    };
    const compileCurrentGraph = vi.fn().mockResolvedValue(compiled);

    act(() => {
      useGraphStore.setState({ compileCurrentGraph });
    });

    const result = await useGraphStore.getState().exportRuntimeConfig();

    expect(compileCurrentGraph).toHaveBeenCalledTimes(1);
    expect(result).toBe(compiled);
    expect(result.runtime_config.metadata.graph_id).toBe("graph_backend");
    expect(result.runtime_targets.runtime_node_id).toBe("runtime_node");
  });
});
