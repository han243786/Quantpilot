import { useEffect, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import { useStrategyResearchActions } from "./useStrategyResearchActions";
import { useStrategyResearchSelectors } from "./strategyResearchSelectors";
import { useStrategyResearchUiState } from "./useStrategyResearchUiState";

export function useStrategyResearchModel() {
  const [panelNotice, setPanelNotice] = useState(null);
  const graphId = useGraphStore((state) => state.graph.metadata?.graph_id || "");
  const uiState = useStrategyResearchUiState(graphId);
  const selectors = useStrategyResearchSelectors(uiState);
  const actions = useStrategyResearchActions(uiState, {
    onNotice(type, message) {
      setPanelNotice({
        id: Date.now() + Math.random(),
        type,
        message
      });
    }
  });

  useEffect(() => {
    if (!panelNotice || panelNotice.type === "error") return undefined;
    const timeoutId = window.setTimeout(() => {
      setPanelNotice((current) => (current?.id === panelNotice.id ? null : current));
    }, 3200);
    return () => window.clearTimeout(timeoutId);
  }, [panelNotice]);

  return {
    ...selectors,
    ...uiState,
    ...actions,
    panelNotice,
    setPanelNotice
  };
}
