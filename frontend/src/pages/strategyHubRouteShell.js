export const STRATEGY_HUB_PAGE_CLASS_NAME = "strategy-hub-page";
export const STRATEGY_HUB_PAGE_TEST_ID = "strategy-hub-page";
export const STRATEGY_HUB_ROUTE_HEADING = "策略中心";

export const STRATEGY_HUB_VISUALLY_HIDDEN_HEADING_STYLE = Object.freeze({
  position: "absolute",
  width: "1px",
  height: "1px",
  overflow: "hidden",
  clip: "rect(0,0,0,0)",
  whiteSpace: "nowrap"
});

export const STRATEGY_HUB_SECTION_DEFS = Object.freeze([
  { id: "hero", fallbackTitle: "策略中心总览" },
  { id: "body", fallbackTitle: "策略中心工作区" }
]);

export function buildStrategyHubPageShellProps() {
  return {
    className: STRATEGY_HUB_PAGE_CLASS_NAME,
    "data-testid": STRATEGY_HUB_PAGE_TEST_ID
  };
}

export function getStrategyHubSectionDef(sectionId) {
  return STRATEGY_HUB_SECTION_DEFS.find((section) => section.id === sectionId) || null;
}

export function buildStrategyHubFallbackProps(sectionId) {
  const section = getStrategyHubSectionDef(sectionId);

  return {
    title: section?.fallbackTitle || "策略中心面板加载中"
  };
}
