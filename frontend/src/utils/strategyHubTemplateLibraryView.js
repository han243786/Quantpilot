export const STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY = "quantpilot_template_visited";

export function getInitialStrategyTemplateLibraryExpanded(storage = globalThis.localStorage) {
  if (!storage) return true;
  if (storage.getItem(STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY)) return false;
  storage.setItem(STRATEGY_HUB_TEMPLATE_LIBRARY_VISITED_KEY, "1");
  return true;
}

export function projectStrategyHubTemplateLibraryView(
  templateLibrary,
  activeTemplateId,
  isExpanded
) {
  const templates = Array.isArray(templateLibrary) ? templateLibrary : [];

  return {
    className: `strategy-template-library strategy-activity-card${
      isExpanded ? " strategy-template-library--expanded" : " strategy-template-library--collapsed"
    }`,
    templates: templates.map((template) => ({
      ...template,
      isLoading: activeTemplateId === template.id,
      symbolsLabel: template.symbols.join(", "),
      supportedModuleCount: template.supportedModules.length,
      symbolCount: template.symbols.length
    }))
  };
}
