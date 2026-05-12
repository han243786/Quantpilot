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
    <main className="strategy-hub-page" data-testid="strategy-hub-page">
      <h1 style={{position:"absolute",width:"1px",height:"1px",overflow:"hidden",clip:"rect(0,0,0,0)",whiteSpace:"nowrap"}}>策略中心</h1>
      <TopToolbar variant="default" />
      <Suspense fallback={<StrategyHubSectionFallback title="策略中心总览" />}>
        <StrategyHubHeroSection model={model} />
      </Suspense>
      <Suspense fallback={<StrategyHubSectionFallback title="策略中心工作区" />}>
        <StrategyHubBodySection model={model} />
      </Suspense>
    </main>
  );
}
