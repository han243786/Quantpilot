import { useEffect, useState } from "react";

const DEFAULT_RUN_FILTERS = {
  historyFilter: "",
  historyCompileFilter: "",
  historyFromTime: "",
  historyToTime: "",
  historyStatusFilter: "all",
  historySortOrder: "desc",
  historyPage: 1,
  historyPageSize: 6
};

const DEFAULT_BACKTEST_FILTERS = {
  backtestHistoryFilter: "",
  backtestCompileFilter: "",
  backtestDatasetFilter: "",
  backtestParameterFilter: "",
  backtestFromTime: "",
  backtestToTime: "",
  backtestPage: 1,
  backtestPageSize: 6
};

const DEFAULT_EVENT_FILTERS = {
  eventNodeScope: "auto",
  eventTypeFilter: "all",
  eventSearchTerm: ""
};

export function useStrategyResearchUiState(graphId) {
  const [runFilters, setRunFilters] = useState({
    ...DEFAULT_RUN_FILTERS,
    historyFilter: graphId || ""
  });
  const [backtestFilters, setBacktestFilters] = useState({
    ...DEFAULT_BACKTEST_FILTERS,
    backtestHistoryFilter: graphId || ""
  });
  const [eventFilters, setEventFilters] = useState(DEFAULT_EVENT_FILTERS);

  useEffect(() => {
    setRunFilters((current) => ({
      ...current,
      historyFilter: graphId || ""
    }));
    setBacktestFilters((current) => ({
      ...current,
      backtestHistoryFilter: graphId || ""
    }));
    setEventFilters(DEFAULT_EVENT_FILTERS);
  }, [graphId]);

  return {
    runFilters,
    setRunHistoryFilter(value) {
      setRunFilters((current) => ({ ...current, historyFilter: value, historyPage: 1 }));
    },
    setRunHistoryCompileFilter(value) {
      setRunFilters((current) => ({ ...current, historyCompileFilter: value, historyPage: 1 }));
    },
    setRunHistoryFromTime(value) {
      setRunFilters((current) => ({ ...current, historyFromTime: value, historyPage: 1 }));
    },
    setRunHistoryToTime(value) {
      setRunFilters((current) => ({ ...current, historyToTime: value, historyPage: 1 }));
    },
    setRunHistoryStatusFilter(value) {
      setRunFilters((current) => ({ ...current, historyStatusFilter: value, historyPage: 1 }));
    },
    setRunHistorySortOrder(value) {
      setRunFilters((current) => ({ ...current, historySortOrder: value, historyPage: 1 }));
    },
    setRunHistoryPage(value) {
      setRunFilters((current) => ({ ...current, historyPage: value }));
    },
    setRunHistoryPageSize(value) {
      setRunFilters((current) => ({ ...current, historyPageSize: value, historyPage: 1 }));
    },
    backtestFilters,
    setBacktestHistoryFilter(value) {
      setBacktestFilters((current) => ({
        ...current,
        backtestHistoryFilter: value,
        backtestPage: 1
      }));
    },
    setBacktestCompileFilter(value) {
      setBacktestFilters((current) => ({
        ...current,
        backtestCompileFilter: value,
        backtestPage: 1
      }));
    },
    setBacktestDatasetFilter(value) {
      setBacktestFilters((current) => ({
        ...current,
        backtestDatasetFilter: value,
        backtestPage: 1
      }));
    },
    setBacktestParameterFilter(value) {
      setBacktestFilters((current) => ({
        ...current,
        backtestParameterFilter: value,
        backtestPage: 1
      }));
    },
    setBacktestFromTime(value) {
      setBacktestFilters((current) => ({ ...current, backtestFromTime: value, backtestPage: 1 }));
    },
    setBacktestToTime(value) {
      setBacktestFilters((current) => ({ ...current, backtestToTime: value, backtestPage: 1 }));
    },
    setBacktestPage(value) {
      setBacktestFilters((current) => ({ ...current, backtestPage: value }));
    },
    setBacktestPageSize(value) {
      setBacktestFilters((current) => ({
        ...current,
        backtestPageSize: value,
        backtestPage: 1
      }));
    },
    eventFilters,
    setEventTypeFilter(value) {
      setEventFilters((current) => ({ ...current, eventTypeFilter: value }));
    },
    setEventSearchTerm(value) {
      setEventFilters((current) => ({ ...current, eventSearchTerm: value }));
    },
    setEventNodeScope(value) {
      setEventFilters((current) => ({ ...current, eventNodeScope: value }));
    }
  };
}
