export const FLOW_NODE_WIDTH = 250;
export const FLOW_NODE_HEIGHT = 140;
const VIEWPORT_MARGIN = 80;

export function isNodeVisibleInViewport(node, viewport, viewportSize) {
  const zoom = viewport?.zoom ?? 1;
  const translateX = viewport?.x ?? 0;
  const translateY = viewport?.y ?? 0;
  const width = viewportSize?.width ?? 0;
  const height = viewportSize?.height ?? 0;

  if (width <= 0 || height <= 0) return true;

  const screenLeft = node.position.x * zoom + translateX;
  const screenTop = node.position.y * zoom + translateY;
  const screenRight = screenLeft + FLOW_NODE_WIDTH * zoom;
  const screenBottom = screenTop + FLOW_NODE_HEIGHT * zoom;

  return !(
    screenRight < -VIEWPORT_MARGIN ||
    screenLeft > width + VIEWPORT_MARGIN ||
    screenBottom < -VIEWPORT_MARGIN ||
    screenTop > height + VIEWPORT_MARGIN
  );
}

export function collectVisibleNodeIds(nodes, viewport, viewportSize) {
  return new Set(
    nodes
      .filter((node) => isNodeVisibleInViewport(node, viewport, viewportSize))
      .map((node) => node.id)
  );
}
