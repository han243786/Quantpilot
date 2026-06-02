import { describe, expect, it, vi } from "vitest";
import {
  STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY,
  getInitialStrategyTemplateLibraryExpanded,
  projectStrategyHubTemplateLibraryView
} from "./strategyHubTemplateLibraryView";

describe("strategyHubTemplateLibraryView", () => {
  it("expands on first visit and persists the visit marker", () => {
    const storage = {
      getItem: vi.fn().mockReturnValue(null),
      setItem: vi.fn()
    };

    expect(getInitialStrategyTemplateLibraryExpanded(storage)).toBe(true);
    expect(storage.getItem).toHaveBeenCalledWith(STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY);
    expect(storage.setItem).toHaveBeenCalledWith(
      STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY,
      "1"
    );
  });

  it("collapses after the visit marker exists", () => {
    const storage = {
      getItem: vi.fn().mockReturnValue("1"),
      setItem: vi.fn()
    };

    expect(getInitialStrategyTemplateLibraryExpanded(storage)).toBe(false);
    expect(storage.setItem).not.toHaveBeenCalled();
  });

  it("projects template cards and shell state", () => {
    const view = projectStrategyHubTemplateLibraryView(
      [
        {
          id: "dual_ma_trend",
          title: "Dual MA",
          category: "Trend",
          description: "Starter",
          supportedModules: ["builtin.data.kline", "builtin.intent.double_ma"],
          symbols: ["BTCUSDT", "ETHUSDT"]
        }
      ],
      "dual_ma_trend",
      true
    );

    expect(view.className).toContain("strategy-template-library--expanded");
    expect(view.templates).toEqual([
      expect.objectContaining({
        id: "dual_ma_trend",
        isLoading: true,
        symbolsLabel: "BTCUSDT, ETHUSDT",
        supportedModuleCount: 2,
        symbolCount: 2
      })
    ]);
  });

  it("normalizes non-array template libraries", () => {
    expect(projectStrategyHubTemplateLibraryView(null, "", false)).toEqual(
      expect.objectContaining({
        className: expect.stringContaining("strategy-template-library--collapsed"),
        templates: []
      })
    );
  });
});
