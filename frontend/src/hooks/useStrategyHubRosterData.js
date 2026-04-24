import { useMemo } from "react";
import {
  projectStrategyHubActivityItems,
  projectStrategyHubRosterRows,
  projectStrategyHubRosterToolbar
} from "../utils/strategyHubRosterProjection";

export function useStrategyHubRosterData(model) {
  const activityItems = useMemo(
    () => projectStrategyHubActivityItems(model.activityTimeline),
    [model.activityTimeline]
  );

  const toolbar = useMemo(() => projectStrategyHubRosterToolbar(model), [model]);

  const rosterRows = useMemo(() => projectStrategyHubRosterRows(model), [model]);

  return {
    ...activityItems,
    toolbar,
    rosterRows
  };
}
