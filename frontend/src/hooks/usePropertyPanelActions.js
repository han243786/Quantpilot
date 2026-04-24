import { translateText } from "../i18n";
import { useGraphStore } from "../store/graphStore";

export function usePropertyPanelActions() {
  const updateNodeConfig = useGraphStore((state) => state.updateNodeConfig);
  const updateNodeName = useGraphStore((state) => state.updateNodeName);
  const removeSelected = useGraphStore((state) => state.removeSelected);
  const updateQuantScriptDraft = useGraphStore((state) => state.updateQuantScriptDraft);
  const updateFormalQuantScriptDraft = useGraphStore((state) => state.updateFormalQuantScriptDraft);
  const updateStrategyIrDraft = useGraphStore((state) => state.updateStrategyIrDraft);
  const resetQuantScriptDraft = useGraphStore((state) => state.resetQuantScriptDraft);
  const resetFormalQuantScriptDraft = useGraphStore((state) => state.resetFormalQuantScriptDraft);
  const resetStrategyIrDraft = useGraphStore((state) => state.resetStrategyIrDraft);
  const applyQuantScriptSource = useGraphStore((state) => state.applyQuantScriptSource);
  const applyFormalQuantScriptSource = useGraphStore((state) => state.applyFormalQuantScriptSource);
  const applyStrategyIrSource = useGraphStore((state) => state.applyStrategyIrSource);

  function handleResetQuantScript(clearError) {
    resetQuantScriptDraft();
    clearError?.("");
  }

  function handleApplyQuantScript(setError) {
    try {
      applyQuantScriptSource();
      setError?.("");
    } catch (error) {
      setError?.(error?.message || translateText("策略图源码解析失败。"));
    }
  }

  function handleResetStrategyIr(clearError) {
    resetStrategyIrDraft();
    clearError?.("");
  }

  function handleResetFormalQuantScript(clearError) {
    resetFormalQuantScriptDraft();
    clearError?.("");
  }

  function handleApplyFormalQuantScript(setError) {
    try {
      applyFormalQuantScriptSource();
      setError?.("");
    } catch (error) {
      setError?.(error?.message || translateText("Formal QuantScript 不能为空。"));
    }
  }

  function handleApplyStrategyIr(setError) {
    try {
      applyStrategyIrSource();
      setError?.("");
    } catch (error) {
      setError?.(error?.message || translateText("Strategy IR JSON 解析失败。"));
    }
  }

  return {
    updateNodeConfig,
    updateNodeName,
    removeSelected,
    updateQuantScriptDraft,
    updateFormalQuantScriptDraft,
    updateStrategyIrDraft,
    handleResetQuantScript,
    handleApplyQuantScript,
    handleResetFormalQuantScript,
    handleApplyFormalQuantScript,
    handleResetStrategyIr,
    handleApplyStrategyIr
  };
}
