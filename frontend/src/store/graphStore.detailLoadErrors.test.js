import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useGraphStore } from "./graphStore";

describe("graphStore detail load failure paths", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("formats run detail load failures as reason plus next action", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        text: async () => "Run record missing."
      })
    );

    const result = await useGraphStore.getState().loadRunDetail("run_missing");

    expect(result).toBeNull();
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "原因：Run record missing. 后续：检查后端可用性，以及所选运行记录是否仍然存在后，再重新加载运行详情。"
    );
  });

  it("formats backtest detail load failures as reason plus next action", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        text: async () => "Backtest record missing."
      })
    );

    const result = await useGraphStore.getState().loadBacktestDetail("backtest_missing");

    expect(result).toBeNull();
    expect(useGraphStore.getState().runtime.backendError).toContain(
      "原因：Backtest record missing. 后续：检查后端可用性，以及所选回测记录是否仍然存在后，再重新加载回测详情。"
    );
  });
});
