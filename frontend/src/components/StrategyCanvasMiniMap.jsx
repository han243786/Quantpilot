import { useCallback, useMemo, useRef, useState } from "react";
import { Panel, useReactFlow, useStore, useViewport } from "@xyflow/react";

const minimapNodeColors = {
  data: "var(--ad-accent)",
  intent: "var(--ad-text-secondary)",
  agent: "var(--ad-warning)",
  risk: "var(--ad-error)",
  execution: "var(--ad-success)",
  runtime: "var(--ad-text-muted)"
};

const minimapSizes = {
  compact: { width: 188, height: 124, padding: 12, toolbarHeight: 34 },
  expanded: { width: 272, height: 176, padding: 16, toolbarHeight: 36 }
};

const NODE_WIDTH = 250;
const NODE_HEIGHT = 140;

function getMiniMapNodeColor(type) {
  return minimapNodeColors[type] || "var(--ad-text-muted)";
}

function buildMiniMapCurve(source, target) {
  const dx = Math.max((target.x - source.x) * 0.45, 16);
  return `M ${source.x} ${source.y} C ${source.x + dx} ${source.y}, ${target.x - dx} ${target.y}, ${target.x} ${target.y}`;
}

export default function StrategyCanvasMiniMap({
  graphNodes,
  graphEdges,
  highlightedNodeIds
}) {
  const reactFlow = useReactFlow();
  const { x, y, zoom } = useViewport();
  const viewportSize = useStore((state) => ({ width: state.width, height: state.height }));
  const dragStateRef = useRef(null);
  const rootRef = useRef(null);
  const [sizeMode, setSizeMode] = useState("compact");

  const miniSize = minimapSizes[sizeMode];
  const canvasWidth = miniSize.width;
  const canvasHeight = miniSize.height - miniSize.toolbarHeight;

  const bounds = useMemo(() => {
    if (!graphNodes.length) {
      return {
        minX: 0,
        minY: 0,
        width: 1000,
        height: 700
      };
    }

    const minX = Math.min(...graphNodes.map((node) => node.position.x));
    const minY = Math.min(...graphNodes.map((node) => node.position.y));
    const maxX = Math.max(...graphNodes.map((node) => node.position.x + NODE_WIDTH));
    const maxY = Math.max(...graphNodes.map((node) => node.position.y + NODE_HEIGHT));
    const padding = 120;

    return {
      minX: minX - padding,
      minY: minY - padding,
      width: Math.max(maxX - minX + padding * 2, 1),
      height: Math.max(maxY - minY + padding * 2, 1)
    };
  }, [graphNodes]);

  const scale = useMemo(() => {
    const availableWidth = canvasWidth - miniSize.padding * 2;
    const availableHeight = canvasHeight - miniSize.padding * 2;
    return Math.min(availableWidth / bounds.width, availableHeight / bounds.height);
  }, [bounds, canvasHeight, canvasWidth, miniSize.padding]);

  const contentWidth = bounds.width * scale;
  const contentHeight = bounds.height * scale;
  const offsetX = (canvasWidth - contentWidth) / 2;
  const offsetY = (canvasHeight - contentHeight) / 2;

  const viewportBounds = useMemo(() => {
    const flowWidth = viewportSize.width || 1;
    const flowHeight = viewportSize.height || 1;
    return {
      left: -x / zoom,
      top: -y / zoom,
      width: flowWidth / zoom,
      height: flowHeight / zoom
    };
  }, [viewportSize.height, viewportSize.width, x, y, zoom]);

  const projectX = useCallback(
    (flowX) => offsetX + (flowX - bounds.minX) * scale,
    [bounds.minX, offsetX, scale]
  );
  const projectY = useCallback(
    (flowY) => offsetY + (flowY - bounds.minY) * scale,
    [bounds.minY, offsetY, scale]
  );

  const viewportRect = useMemo(
    () => ({
      left: projectX(viewportBounds.left),
      top: projectY(viewportBounds.top),
      width: Math.max(viewportBounds.width * scale, 22),
      height: Math.max(viewportBounds.height * scale, 16)
    }),
    [projectX, projectY, scale, viewportBounds.height, viewportBounds.left, viewportBounds.top, viewportBounds.width]
  );

  const nodeRects = useMemo(() => {
    const rects = new Map();
    graphNodes.forEach((node) => {
      const width = Math.max(NODE_WIDTH * scale, sizeMode === "compact" ? 18 : 24);
      const height = Math.max(NODE_HEIGHT * scale, sizeMode === "compact" ? 10 : 14);
      rects.set(node.id, {
        left: projectX(node.position.x),
        top: projectY(node.position.y),
        width,
        height,
        centerX: projectX(node.position.x) + width / 2,
        centerY: projectY(node.position.y) + height / 2
      });
    });
    return rects;
  }, [graphNodes, projectX, projectY, scale, sizeMode]);

  const edgePaths = useMemo(
    () =>
      graphEdges
        .map((edge) => {
          const source = nodeRects.get(edge.source_node_id);
          const target = nodeRects.get(edge.target_node_id);
          if (!source || !target) return null;

          return {
            id: edge.id,
            highlighted:
              highlightedNodeIds.includes(edge.source_node_id) &&
              highlightedNodeIds.includes(edge.target_node_id),
            path: buildMiniMapCurve(
              { x: source.left + source.width, y: source.centerY },
              { x: target.left, y: target.centerY }
            )
          };
        })
        .filter(Boolean),
    [graphEdges, highlightedNodeIds, nodeRects]
  );

  const moveViewportToClientPoint = useCallback(
    (clientX, clientY) => {
      const root = rootRef.current;
      if (!root) return;

      const rect = root.getBoundingClientRect();
      const relativeX = Math.min(Math.max(clientX - rect.left, 0), rect.width);
      const relativeY = Math.min(Math.max(clientY - rect.top, 0), rect.height);
      const flowX = bounds.minX + (relativeX - offsetX) / scale;
      const flowY = bounds.minY + (relativeY - offsetY) / scale;
      const dragOffset = dragStateRef.current?.offset || { x: 0, y: 0 };

      reactFlow.setCenter(flowX - dragOffset.x, flowY - dragOffset.y, { zoom, duration: 0 });
    },
    [bounds.minX, bounds.minY, offsetX, offsetY, reactFlow, scale, zoom]
  );

  const onMiniMapPointerDown = useCallback(
    (event) => {
      event.preventDefault();
      const root = event.currentTarget;
      const rect = root.getBoundingClientRect();
      const relativeX = Math.min(Math.max(event.clientX - rect.left, 0), rect.width);
      const relativeY = Math.min(Math.max(event.clientY - rect.top, 0), rect.height);
      const flowX = bounds.minX + (relativeX - offsetX) / scale;
      const flowY = bounds.minY + (relativeY - offsetY) / scale;
      const viewportCenterX = viewportBounds.left + viewportBounds.width / 2;
      const viewportCenterY = viewportBounds.top + viewportBounds.height / 2;
      const isInsideViewport =
        relativeX >= viewportRect.left &&
        relativeX <= viewportRect.left + viewportRect.width &&
        relativeY >= viewportRect.top &&
        relativeY <= viewportRect.top + viewportRect.height;

      dragStateRef.current = {
        pointerId: event.pointerId,
        offset: isInsideViewport
          ? { x: flowX - viewportCenterX, y: flowY - viewportCenterY }
          : { x: 0, y: 0 }
      };

      root.setPointerCapture?.(event.pointerId);
      moveViewportToClientPoint(event.clientX, event.clientY);
    },
    [
      bounds.minX,
      bounds.minY,
      moveViewportToClientPoint,
      offsetX,
      offsetY,
      scale,
      viewportBounds.height,
      viewportBounds.left,
      viewportBounds.top,
      viewportBounds.width,
      viewportRect.height,
      viewportRect.left,
      viewportRect.top,
      viewportRect.width
    ]
  );

  const onMiniMapPointerMove = useCallback(
    (event) => {
      if (!dragStateRef.current || dragStateRef.current.pointerId !== event.pointerId) return;
      moveViewportToClientPoint(event.clientX, event.clientY);
    },
    [moveViewportToClientPoint]
  );

  const onMiniMapPointerUp = useCallback((event) => {
    if (dragStateRef.current?.pointerId !== event.pointerId) return;
    dragStateRef.current = null;
    event.currentTarget.releasePointerCapture?.(event.pointerId);
  }, []);

  return (
    <Panel position="bottom-right" className="quantpilot-minimap-panel">
      <div className="quantpilot-minimap-shell">
        <div className={`quantpilot-minimap-frame quantpilot-minimap-${sizeMode}`}>
          <div className="quantpilot-minimap-toolbar">
            <div className="quantpilot-minimap-zoom-group">
              <button
                type="button"
                className="quantpilot-minimap-tool-btn"
                onClick={() => reactFlow.zoomIn()}
                aria-label="放大"
              >
                +
              </button>
              <button
                type="button"
                className="quantpilot-minimap-tool-btn"
                onClick={() => reactFlow.zoomOut()}
                aria-label="缩小"
              >
                -
              </button>
              <button
                type="button"
                className="quantpilot-minimap-tool-btn"
                onClick={() => reactFlow.fitView({ duration: 180 })}
                aria-label="适应画布"
              >
                []
              </button>
            </div>
            <div className="quantpilot-minimap-size-switch" aria-label="小地图尺寸">
              <button
                type="button"
                className={`quantpilot-minimap-size-btn ${sizeMode === "compact" ? "active" : ""}`}
                onClick={() => setSizeMode("compact")}
                aria-label="紧凑视图"
                title="紧凑视图"
              >
                <span className="size-glyph compact" />
              </button>
              <button
                type="button"
                className={`quantpilot-minimap-size-btn ${sizeMode === "expanded" ? "active" : ""}`}
                onClick={() => setSizeMode("expanded")}
                aria-label="展开视图"
                title="展开视图"
              >
                <span className="size-glyph expanded" />
              </button>
            </div>
          </div>
          <div
            ref={rootRef}
            className="quantpilot-minimap-canvas"
            style={{ width: canvasWidth, height: canvasHeight }}
            onPointerDown={onMiniMapPointerDown}
            onPointerMove={onMiniMapPointerMove}
            onPointerUp={onMiniMapPointerUp}
            onPointerCancel={onMiniMapPointerUp}
          >
            <div className="quantpilot-minimap-grid" />
            <svg
              className="quantpilot-minimap-edges"
              viewBox={`0 0 ${canvasWidth} ${canvasHeight}`}
              preserveAspectRatio="none"
              aria-hidden="true"
            >
              {edgePaths.map((edge) => (
                <path
                  key={edge.id}
                  d={edge.path}
                  className={`quantpilot-minimap-edge ${edge.highlighted ? "highlighted" : ""}`}
                />
              ))}
            </svg>
            {graphNodes.map((node) => {
              const rect = nodeRects.get(node.id);
              if (!rect) return null;

              return (
                <div
                  key={node.id}
                  className={`quantpilot-minimap-node ${highlightedNodeIds.includes(node.id) ? "highlighted" : ""}`}
                  style={{
                    left: rect.left,
                    top: rect.top,
                    width: rect.width,
                    height: rect.height,
                    background: getMiniMapNodeColor(node.type)
                  }}
                />
              );
            })}
            <div
              className="quantpilot-minimap-viewport"
              style={{
                left: viewportRect.left,
                top: viewportRect.top,
                width: viewportRect.width,
                height: viewportRect.height
              }}
            />
          </div>
        </div>
      </div>
    </Panel>
  );
}
