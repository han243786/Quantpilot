import { useMemo } from "react";
import {
  projectInspectorBacktests,
  projectInspectorCompareQueue,
  projectInspectorRuns,
  projectStrategyHubInspectorOverview
} from "../utils/strategyHubInspectorProjection";

export function useStrategyHubInspectorData(selectedStrategy, compareSelection) {
  const overview = useMemo(
    () => projectStrategyHubInspectorOverview(selectedStrategy),
    [selectedStrategy]
  );

  const recentBacktests = useMemo(
    () => projectInspectorBacktests(selectedStrategy, compareSelection),
    [selectedStrategy, compareSelection]
  );

  const recentRuns = useMemo(() => projectInspectorRuns(selectedStrategy), [selectedStrategy]);

  const compareQueue = useMemo(
    () => projectInspectorCompareQueue(compareSelection),
    [compareSelection]
  );

  return {
    overview,
    recentBacktests,
    recentRuns,
    compareQueue
  };
}
