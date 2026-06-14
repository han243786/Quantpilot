import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  AnalysisHero,
  AnalysisSection,
  AnalysisStatusBanner,
  StrategyRouteBar
} from "./index";

describe("backtestViews shared analysis layout", () => {
  it("renders route bars with clickable ancestors and current leaf", async () => {
    const user = userEvent.setup();
    const onOpen = vi.fn();

    render(
      <StrategyRouteBar
        items={[
          { label: "Strategies", onClick: onOpen },
          { label: "alpha" },
          { label: "Backtests", current: true }
        ]}
      />
    );

    await user.click(screen.getByRole("button", { name: "Strategies" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(screen.getByText("alpha")).toBeInTheDocument();
    expect(screen.getByText("Backtests")).toHaveClass("strategy-route-bar__current--active");
  });

  it("renders hero summaries, sections, and status variants through the shared entry", () => {
    render(
      <>
        <AnalysisHero
          testId="analysis-hero"
          kicker="Research"
          title="Backtest Detail"
          subtitle="Persisted analysis"
          meta="strategy:alpha"
          summaryItems={[{ label: "Return", value: "+12.00%" }]}
        />
        <AnalysisSection
          testId="analysis-section"
          kicker="Artifacts"
          title="Core artifacts"
          summary="Manifest and metrics"
        >
          <span>child content</span>
        </AnalysisSection>
        <AnalysisStatusBanner variant="error" testId="analysis-status">
          Load failed
        </AnalysisStatusBanner>
      </>
    );

    expect(screen.getByTestId("analysis-hero")).toHaveTextContent("Backtest Detail");
    expect(screen.getByText("+12.00%")).toBeInTheDocument();
    expect(screen.getByTestId("analysis-section")).toHaveTextContent("child content");
    expect(screen.getByTestId("analysis-status")).toHaveClass("analysis-status-banner--error");
  });
});
