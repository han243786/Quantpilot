import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StrategyHubTemplateLibrarySection from "./StrategyHubTemplateLibrarySection";

describe("StrategyHubTemplateLibrarySection", () => {
  it("renders starter templates and routes load requests through the model", async () => {
    const applyTemplate = vi.fn().mockResolvedValue({
      metadata: { graph_id: "template_dual_ma_trend_1" }
    });

    render(
      <StrategyHubTemplateLibrarySection
        model={{
          templateLibrary: [
            {
              id: "dual_ma_trend",
              title: "Dual moving-average trend",
              category: "Trend",
              description: "Starter graph for a trend-following strategy.",
              supportedModules: ["builtin.data.kline", "builtin.intent.double_ma"],
              symbols: ["BTCUSDT"]
            }
          ],
          applyTemplate
        }}
      />
    );

    expect(screen.getByTestId("strategy-template-library")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-template-card-dual_ma_trend")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("strategy-template-load-dual_ma_trend"));

    await waitFor(() => {
      expect(applyTemplate).toHaveBeenCalledWith("dual_ma_trend");
    });
  });
});
