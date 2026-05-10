import { Suspense, lazy } from "react";
import "./strategy-hub.css";
import { useStrategyDirectoryModel } from "../hooks/useStrategyDirectoryModel";
import { StrategyHubSectionFallback } from "./StrategyHubPanelFallbacks";
import TopToolbar from "../components/TopToolbar";

const StrategyHubHeroSection = lazy(() => import("./StrategyHubHeroSection"));
const StrategyHubBodySection = lazy(() => import("./StrategyHubBodySection"));

export default function StrategyHubPage() {
  const model = useStrategyDirectoryModel();

  return (
    <div className="strategy-hub-page" data-testid="strategy-hub-page">
      <TopToolbar variant="default" />
      <Suspense fallback={<StrategyHubSectionFallback title="策略中心总览" />}>
        <StrategyHubHeroSection model={model} />
      </Suspense>
      <Suspense fallback={<StrategyHubSectionFallback title="策略中心工作区" />}>
        <StrategyHubBodySection model={model} />
      </Suspense>
    </div>
  );
}
