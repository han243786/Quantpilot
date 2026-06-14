import { useEffect, useMemo, useRef, useState } from "react";
import { canvasFocusStatusLabel } from "../utils/workspaceContextLabels";
import {
  normalizeWorkspaceIssueFilters,
  persistWorkspaceIssueFilters,
  readStoredWorkspaceIssueFilters,
  workspaceIssueFiltersStorageScope,
  workspaceIssueFiltersSummary
} from "./strategyWorkspaceIssueQueueState";
const DEFAULT_CODE_LANE_STATE = {
  mode: "auto",
  pinnedLaneId: null
};
const CODE_LANE_NOTICE_FADE_DELAY_MS = 4500;

function codeLaneNoticePayload(message, tone = "info") {
  if (!message) return null;
  return { title: "工作区栏位已自动切换", message, tone };
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
  const [activeTab, setActiveTab] = useState("dashboard");
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
          setError(loadError instanceof Error ? loadError.message : "无法加载请求的策略图。");
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
        reason: "该诊断指向策略中间表示，工作区已打开源码栏位以便直接审查。",
        tone: "info",
        focusMode: "selected",
        focusChanged: true
      });
      setCanvasFocusMode("selected");
      return;
    }

    activateCodeInspector("diagnostics", {
      priority: "high",
      reason: "该问题指向图谱诊断，工作区已打开检查栏位和问题聚焦的画布模式。",
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
      reason: "源码栏位保持可用，用于策略中间表示审查、图谱源码编辑和代码修复工作。",
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
          reason: "已从诊断目标中选择策略中间表示，工作区已切换回源码栏位。",
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
