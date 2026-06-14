import { Suspense, lazy } from "react";
import "./strategy-hub.css";
import { useStrategyDirectoryModel } from "../hooks/useStrategyDirectoryModel";
import { StrategyHubSectionFallback } from "./StrategyHubPanelFallbacks";
import {
  STRATEGY_HUB_ROUTE_HEADING,
  STRATEGY_HUB_VISUALLY_HIDDEN_HEADING_STYLE,
  buildStrategyHubFallbackProps,
  buildStrategyHubPageShellProps
} from "./strategyHubRouteShell";

const StrategyHubHeroSection = lazy(() => import("./StrategyHubHeroSection"));
const StrategyHubBodySection = lazy(() => import("./StrategyHubBodySection"));

export default function StrategyHubPage() {
  const model = useStrategyDirectoryModel();

  return (
    <main {...buildStrategyHubPageShellProps()}>
      <h1 style={STRATEGY_HUB_VISUALLY_HIDDEN_HEADING_STYLE}>{STRATEGY_HUB_ROUTE_HEADING}</h1>
      <Suspense fallback={<StrategyHubSectionFallback {...buildStrategyHubFallbackProps("hero")} />}>
        <StrategyHubHeroSection model={model} />
      </Suspense>
      <Suspense fallback={<StrategyHubSectionFallback {...buildStrategyHubFallbackProps("body")} />}>
        <StrategyHubBodySection model={model} />
      </Suspense>
    </main>
  );
}
