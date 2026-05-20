import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StrategyHubTemplateLibrarySection from "./StrategyHubTemplateLibrarySection";

describe("StrategyHubTemplateLibrarySection", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("expands starter templates by default on first visit and routes load requests", async () => {
    const applyTemplate = vi.fn().mockResolvedValue({
      metadata: { graph_id: "template_dual_ma_trend_1" }
    });

    render(
      <StrategyHubTemplateLibrarySection
        model={{
          templateLibrary: [
            {
              id: "dual_ma_trend",
              title: "双均线趋势",
              category: "趋势",
              description: "趋势跟随起始策略图。",
              supportedModules: ["builtin.data.kline", "builtin.intent.double_ma"],
              symbols: ["BTCUSDT"]
            }
          ],
          applyTemplate
        }}
      />
    );

    expect(screen.getByTestId("strategy-template-library")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-template-library")).toHaveClass(
      "strategy-template-library--expanded"
    );
    const toggle = screen.getByTestId("strategy-template-library-toggle");
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByTestId("strategy-template-card-dual_ma_trend")).toBeInTheDocument();

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(screen.getByTestId("strategy-template-library")).toHaveClass(
      "strategy-template-library--collapsed"
    );
    expect(screen.queryByTestId("strategy-template-card-dual_ma_trend")).not.toBeInTheDocument();

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByTestId("strategy-template-library")).toHaveClass(
      "strategy-template-library--expanded"
    );
    expect(screen.getByTestId("strategy-template-card-dual_ma_trend")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("strategy-template-load-dual_ma_trend"));

    await waitFor(() => {
      expect(applyTemplate).toHaveBeenCalledWith("dual_ma_trend");
    });
  });
});
