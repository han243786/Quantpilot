import { useEffect, useRef, useState } from "react";
import { usePropertyPanelActions } from "./usePropertyPanelActions";
import { usePropertyPanelSelectors } from "./propertyPanelSelectors";
import { findTargetRangeInSource } from "./propertyPanelShared";

export function usePropertyPanelModel() {
  const selectors = usePropertyPanelSelectors();
  const actions = usePropertyPanelActions();
  const [applyError, setApplyError] = useState("");
  const [formalApplyError, setFormalApplyError] = useState("");
  const [strategyIrApplyError, setStrategyIrApplyError] = useState("");
  const strategyIrEditorRef = useRef(null);

  useEffect(() => {
    if (
      selectors.selectedCompileDiagnosticTarget?.scope !== "strategy_ir" ||
      !strategyIrEditorRef.current
    ) {
      return;
    }
    const range = findTargetRangeInSource(
      selectors.strategyIrSource,
      selectors.selectedCompileDiagnosticTarget
    );
    if (!range) return;
    const [start, end] = range;
    strategyIrEditorRef.current.focus();
    strategyIrEditorRef.current.setSelectionRange(start, end);
  }, [selectors.selectedCompileDiagnosticTarget, selectors.strategyIrSource]);

  return {
    ...selectors,
    ...actions,
    applyError,
    setApplyError,
    formalApplyError,
    setFormalApplyError,
    strategyIrApplyError,
    setStrategyIrApplyError,
    strategyIrEditorRef
  };
}
