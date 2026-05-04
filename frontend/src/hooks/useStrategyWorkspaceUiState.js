import { useEffect, useMemo, useRef, useState } from "react";
import { canvasFocusStatusLabel } from "../utils/workspaceContextLabels";
import {
  DEFAULT_WORKSPACE_ISSUE_FILTERS,
  filterWorkspaceIssueQueue,
  filterWorkspaceIssueQueueByNodeType,
  filterWorkspaceIssueQueueBySource,
  workspaceIssueQueueNodeTypeOrder,
  workspaceIssueQueueSourceOrder
} from "../utils/strategyWorkspaceIssueQueue";
const WORKSPACE_ISSUE_FILTERS_STORAGE_KEY = "quantpilot_workspace_issue_filters";
const DEFAULT_CODE_LANE_STATE = {
  mode: "auto",
  pinnedLaneId: null
};
const CODE_LANE_NOTICE_FADE_DELAY_MS = 4500;

function codeLaneNoticePayload(message, tone = "info") {
  if (!message) return null;
  return { title: "Workspace lane changed automatically", message, tone };
}

function workspaceIssueFiltersStorageScope(strategyId, graphId) {
  return strategyId || graphId || "draft_graph";
}

function readStoredWorkspaceIssueFilters(scope) {
  if (typeof window === "undefined" || !window.localStorage) {
    return DEFAULT_WORKSPACE_ISSUE_FILTERS;
  }

  try {
    const raw = window.localStorage.getItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY);
    if (!raw) return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") {
      return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    }
    const storedFilters = parsed[scope];
    if (!storedFilters || typeof storedFilters !== "object") {
      return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    }
    return {
      ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
      ...storedFilters
    };
  } catch {
    return DEFAULT_WORKSPACE_ISSUE_FILTERS;
  }
}

function persistWorkspaceIssueFilters(scope, filters) {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }

  try {
    const raw = window.localStorage.getItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) : {};
    const nextPayload = parsed && typeof parsed === "object" ? parsed : {};
    nextPayload[scope] = {
      ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
      ...(filters || {})
    };
    window.localStorage.setItem(
      WORKSPACE_ISSUE_FILTERS_STORAGE_KEY,
      JSON.stringify(nextPayload)
    );
  } catch {
    // Keep the workspace usable if storage is unavailable.
  }
}

function normalizeWorkspaceIssueFilters(filters, items = []) {
  const nextFilters = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const baseFilteredItems = filterWorkspaceIssueQueue(
    items,
    nextFilters.severityFilter,
    nextFilters.actionableOnly
  );
  const orderedSources = workspaceIssueQueueSourceOrder(items);
  if (
    nextFilters.sourceFilter !== "all" &&
    !orderedSources.includes(nextFilters.sourceFilter)
  ) {
    nextFilters.sourceFilter = "all";
  }
  if (nextFilters.sourceFilter === "all") {
    nextFilters.nodeTypeFilter = "all";
    return nextFilters;
  }
  const sourceFilteredItems = filterWorkspaceIssueQueueBySource(
    baseFilteredItems,
    nextFilters.sourceFilter
  );
  const orderedNodeTypes = workspaceIssueQueueNodeTypeOrder(sourceFilteredItems);
  if (
    nextFilters.nodeTypeFilter !== "all" &&
    !orderedNodeTypes.includes(nextFilters.nodeTypeFilter)
  ) {
    nextFilters.nodeTypeFilter = "all";
  }
  return nextFilters;
}

function workspaceIssueFiltersSummary(filters) {
  const current = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const parts = [];

  if (current.severityFilter !== "all") {
    parts.push(
      current.severityFilter === "error"
        ? "Errors"
        : current.severityFilter === "warning"
          ? "Warnings"
          : current.severityFilter
    );
  }
  if (current.actionableOnly) {
    parts.push("Actionable only");
  }
  if (current.sourceFilter !== "all") {
    parts.push(current.sourceFilter);
  }
  if (current.nodeTypeFilter !== "all") {
    parts.push(current.nodeTypeFilter);
  }

  return parts.length > 0 ? parts.join(" / ") : "No active filters";
}

export function useStrategyWorkspaceUiState({
  strategyId,
  graphId,
  loadGraphById,
  selectedEdgeId,
  selectedCompileDiagnosticTarget,
  issueQueue,
  codeInspectorPanels
}) {
  const issueFilterStorageScope = workspaceIssueFiltersStorageScope(strategyId, graphId);
  const [status, setStatus] = useState(() =>
    !strategyId || graphId === strategyId ? "ready" : "loading"
  );
  const [error, setError] = useState("");
  const [activeTab, setActiveTab] = useState("overview");
  const [activeCodeInspector, setActiveCodeInspector] = useState("params");
  const [codeLaneState, setCodeLaneState] = useState(DEFAULT_CODE_LANE_STATE);
  const [codeLaneNotice, setCodeLaneNotice] = useState(null);
  const [isCodeLaneNoticeVisible, setIsCodeLaneNoticeVisible] = useState(false);
  const [isCodeLaneNoticeHovered, setIsCodeLaneNoticeHovered] = useState(false);
  const [expandedCodeInspectors, setExpandedCodeInspectors] = useState([]);
  const [canvasFocusMode, setCanvasFocusMode] = useState("selected");
  const [issueQueueFilters, setIssueQueueFilters] = useState(() =>
    readStoredWorkspaceIssueFilters(issueFilterStorageScope)
  );
  const previousSelectedEdgeIdRef = useRef(selectedEdgeId);
  const previousDiagnosticTargetKeyRef = useRef(null);

  useEffect(() => {
    let disposed = false;

    if (!strategyId || graphId === strategyId) {
      setStatus("ready");
      setError("");
      return () => {
        disposed = true;
      };
    }

    setStatus("loading");
    setError("");

    void loadGraphById(strategyId)
      .then(() => {
        if (!disposed) {
          setStatus("ready");
        }
      })
      .catch((loadError) => {
        if (!disposed) {
          setStatus("error");
          setError(loadError instanceof Error ? loadError.message : "Unable to load the requested strategy graph.");
        }
      });

    return () => {
      disposed = true;
    };
  }, [graphId, loadGraphById, strategyId]);

  useEffect(() => {
    setIssueQueueFilters(readStoredWorkspaceIssueFilters(issueFilterStorageScope));
  }, [issueFilterStorageScope]);

  useEffect(() => {
    setIssueQueueFilters((current) => {
      const normalized = normalizeWorkspaceIssueFilters(current, issueQueue);
      const changed =
        normalized.severityFilter !== current.severityFilter ||
        normalized.actionableOnly !== current.actionableOnly ||
        normalized.showSourceFilters !== current.showSourceFilters ||
        normalized.sourceFilter !== current.sourceFilter ||
        normalized.nodeTypeFilter !== current.nodeTypeFilter;
      if (changed) {
        persistWorkspaceIssueFilters(issueFilterStorageScope, normalized);
      }
      return changed ? normalized : current;
    });
  }, [issueFilterStorageScope, issueQueue]);

  const selectedCompileDiagnosticTargetKey = useMemo(() => {
    if (!selectedCompileDiagnosticTarget) return null;
    return [
      selectedCompileDiagnosticTarget.scope || "graph",
      selectedCompileDiagnosticTarget.node_id || "",
      selectedCompileDiagnosticTarget.edge_id || "",
      selectedCompileDiagnosticTarget.field || "",
      selectedCompileDiagnosticTarget.label || ""
    ].join(":");
  }, [selectedCompileDiagnosticTarget]);

  const activeInspectorDefinition =
    codeInspectorPanels.find((panel) => panel.id === activeCodeInspector) ||
    codeInspectorPanels[0];
  const canvasWorkspaceContext = useMemo(
    () => ({
      laneId: activeInspectorDefinition.id,
      laneLabel: `${activeInspectorDefinition.label} lane`
    }),
    [activeInspectorDefinition]
  );

  useEffect(() => {
    if (codeLaneNotice) {
      setIsCodeLaneNoticeVisible(true);
    } else {
      setIsCodeLaneNoticeVisible(false);
      setIsCodeLaneNoticeHovered(false);
    }
  }, [codeLaneNotice]);

  useEffect(() => {
    if (!codeLaneNotice || !isCodeLaneNoticeVisible || isCodeLaneNoticeHovered) {
      return undefined;
    }

    const timeoutId = window.setTimeout(() => {
      setIsCodeLaneNoticeVisible(false);
    }, CODE_LANE_NOTICE_FADE_DELAY_MS);

    return () => window.clearTimeout(timeoutId);
  }, [codeLaneNotice, isCodeLaneNoticeHovered, isCodeLaneNoticeVisible]);

  function handleIssueQueueFiltersChange(patch) {
    setIssueQueueFilters((current) => {
      const nextFilters = normalizeWorkspaceIssueFilters(
        {
          ...current,
          ...(typeof patch === "function" ? patch(current) : patch)
        },
        issueQueue
      );
      persistWorkspaceIssueFilters(issueFilterStorageScope, nextFilters);
      return nextFilters;
    });
  }

  function activateCodeInspector(panelId, options = {}) {
    const {
      priority = "low",
      pin = false,
      reason = null,
      tone = "info",
      focusMode = null,
      focusChanged = false
    } = options;

    if (pin) {
      setActiveCodeInspector(panelId);
      setCodeLaneState({ mode: "manual", pinnedLaneId: panelId });
      setCodeLaneNotice(null);
      return;
    }

    if (codeLaneState.mode === "manual" && priority !== "high") {
      return;
    }

    setActiveCodeInspector(panelId);
    if (priority === "high") {
      setCodeLaneState(DEFAULT_CODE_LANE_STATE);
      setCodeLaneNotice(
        reason
          ? {
              ...codeLaneNoticePayload(reason, tone),
              focusLabel: focusMode ? canvasFocusStatusLabel(focusMode) : null,
              focusChanged
            }
          : null
      );
      return;
    }

    setCodeLaneNotice(null);
  }

  function resumeCodeLaneAutoFollow() {
    setCodeLaneState(DEFAULT_CODE_LANE_STATE);
    setCodeLaneNotice(null);
  }

  function toggleExpandedInspector(panelId) {
    setExpandedCodeInspectors((current) =>
      current.includes(panelId)
        ? current.filter((item) => item !== panelId)
        : [...current, panelId]
    );
  }

  function handleRouteDiagnostic(diagnostic) {
    const target = diagnostic?.target || null;
    setActiveTab("code");

    if (target?.scope === "strategy_ir") {
      activateCodeInspector("code", {
        priority: "high",
        reason: "This diagnostic targets Strategy IR, so the workspace opened the source lane for direct review.",
        tone: "info",
        focusMode: "selected",
        focusChanged: true
      });
      setCanvasFocusMode("selected");
      return;
    }

    activateCodeInspector("diagnostics", {
      priority: "high",
      reason: "This issue points to graph diagnostics, so the workspace opened the checks lane and issue-focused canvas mode.",
      tone: "warning",
      focusMode: "issues",
      focusChanged: true
    });
    setCanvasFocusMode("issues");
  }

  function handleActivateSourceLane() {
    setActiveTab("code");
    activateCodeInspector("code", {
      priority: "high",
      reason: "The source lane stays available for Strategy IR review, graph source edits, and code-focused repair work.",
      tone: "info",
      focusMode: canvasFocusMode,
      focusChanged: false
    });
  }

  useEffect(() => {
    const previousSelectedEdgeId = previousSelectedEdgeIdRef.current;
    if (selectedEdgeId && selectedEdgeId !== previousSelectedEdgeId) {
      activateCodeInspector("params", { priority: "low" });
    }
    previousSelectedEdgeIdRef.current = selectedEdgeId;
  }, [selectedEdgeId]);

  useEffect(() => {
    const previousDiagnosticTargetKey = previousDiagnosticTargetKeyRef.current;
    if (
      selectedCompileDiagnosticTarget &&
      selectedCompileDiagnosticTargetKey &&
      selectedCompileDiagnosticTargetKey !== previousDiagnosticTargetKey
    ) {
      if (selectedCompileDiagnosticTarget.scope === "strategy_ir") {
        activateCodeInspector("code", {
          priority: "high",
          reason: "Strategy IR was selected from the diagnostics target, so the workspace switched back to the source lane.",
          tone: "info",
          focusMode: "selected",
          focusChanged: true
        });
      } else {
        activateCodeInspector("diagnostics", { priority: "low" });
      }
    }
    previousDiagnosticTargetKeyRef.current = selectedCompileDiagnosticTargetKey;
  }, [selectedCompileDiagnosticTarget, selectedCompileDiagnosticTargetKey]);

  function handleSelectIssueQueueItem(item) {
    if (item.routeDiagnostic) {
      handleRouteDiagnostic(item.routeDiagnostic);
      return;
    }

    setActiveTab("diagnostics");
  }

  return {
    status,
    error,
    activeTab,
    setActiveTab,
    activeCodeInspector,
    setActiveCodeInspector,
    expandedCodeInspectors,
    toggleExpandedInspector,
    codeLaneState,
    codeLaneNotice,
    isCodeLaneNoticeVisible,
    issueQueueFilters,
    diagnosticsQueueScope: workspaceIssueFiltersSummary(issueQueueFilters),
    handleIssueQueueFiltersChange,
    canvasFocusMode,
    setCanvasFocusMode,
    canvasWorkspaceContext,
    activateCodeInspector,
    resumeCodeLaneAutoFollow,
    handleCodeLaneNoticeMouseEnter: () => {
      setIsCodeLaneNoticeHovered(true);
      setIsCodeLaneNoticeVisible(true);
    },
    handleCodeLaneNoticeMouseLeave: () => {
      setIsCodeLaneNoticeHovered(false);
      setIsCodeLaneNoticeVisible(true);
    },
    handleRouteDiagnostic,
    handleActivateSourceLane,
    handleSelectIssueQueueItem
  };
}

