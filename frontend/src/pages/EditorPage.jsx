import { Suspense, lazy, startTransition, useEffect, useState } from "react";
import TopToolbar from "../components/TopToolbar";
import ModuleSidebar from "../components/ModuleSidebar";
import StrategyCanvas from "../components/StrategyCanvas";
import PropertyPanel from "../components/PropertyPanel";
import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import { backtestDetailPath, navigateTo } from "../router";

const EventStreamPanel = lazy(() => import("../components/EventStreamPanel"));

function EventPanelFallback() {
  const { t } = useI18n();
  return (
    <section className="event-panel event-panel-loading" aria-label={t("运行与回测面板加载中")}>
      <div className="event-panel-header">
        <div className="event-panel-intro">
          <div className="panel-title">{t("运行与回测面板")}</div>
          <div className="panel-subtitle">
            {t("首屏先优先保证编辑器稳定显示，运行历史与回测分析随后加载。")}
          </div>
        </div>
      </div>
      <div className="event-panel-loading-body">
        <div className="event-panel-loading-card" />
        <div className="event-panel-loading-card event-panel-loading-card-wide" />
        <div className="event-panel-loading-card" />
      </div>
    </section>
  );
}

export default function EditorPage() {
  const [showEventPanel, setShowEventPanel] = useState(false);
  const graphId = useGraphStore((state) => state.graph.metadata?.graph_id || "");

  useEffect(() => {
    const schedule = () => {
      startTransition(() => {
        setShowEventPanel(true);
      });
    };

    if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
      const frameId = window.requestAnimationFrame(schedule);
      return () => window.cancelAnimationFrame(frameId);
    }

    const timeoutId = window.setTimeout(schedule, 0);
    return () => window.clearTimeout(timeoutId);
  }, []);

  return (
    <div className="editor-page">
      <TopToolbar />
      <div className="main-workspace">
        <ModuleSidebar />
        <StrategyCanvas />
        <PropertyPanel />
      </div>
      <Suspense fallback={<EventPanelFallback />}>
        {showEventPanel ? (
          <EventStreamPanel
            onOpenBacktestDetail={(backtestId) =>
              navigateTo(backtestDetailPath(backtestId, graphId))
            }
          />
        ) : (
          <EventPanelFallback />
        )}
      </Suspense>
    </div>
  );
}
