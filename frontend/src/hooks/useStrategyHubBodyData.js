import { useMemo } from "react";

export function useStrategyHubBodyData(model) {
  const selectedStrategy = useMemo(() => model.selectedStrategy, [model.selectedStrategy]);
  const compareSelection = useMemo(() => model.compareSelection, [model.compareSelection]);

  return {
    selectedStrategy,
    compareSelection
  };
}
