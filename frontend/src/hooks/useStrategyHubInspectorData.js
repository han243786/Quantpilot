import { useMemo } from "react";
import {
  getStrategyInspectorNextMove,
  projectInspectorBacktests,
  projectInspectorCompareQueue,
  projectInspectorRuns
} from "../utils/strategyHubInspectorProjection";

export function useStrategyHubInspectorData(selectedStrategy, compareSelection) {
  const nextMove = useMemo(
    () => getStrategyInspectorNextMove(selectedStrategy),
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
    nextMove,
    recentBacktests,
    recentRuns,
    compareQueue
  };
}
