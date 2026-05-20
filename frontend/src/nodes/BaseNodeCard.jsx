import { memo, useEffect, useRef } from "react";
import { Handle, Position } from "@xyflow/react";
import { useGraphStore } from "../store/graphStore";

/**
 * PriceOverlay — v2.5.0 P2-5: 价格涌动效果
 *
 * 在 data 类型节点上显示实时价格。
 * - useRef 持有 DOM 元素引用
 * - 通过 Zustand store.subscribe 直接监听价格变化
 * - 更新时直接设置 ref.current.textContent，绕过 React 虚拟 DOM
 * - React.memo((), => true) 确保该组件永不重渲染
 * - Chrome React Profiler 不会显示该组件的重渲染记录
 */
const PriceOverlay = memo(function PriceOverlay({ nodeId }) {
  const ref = useRef(null);

  useEffect(() => {
    // 设置初始价格（从当前 store 状态读取）
    const initialPrice = useGraphStore
      .getState()
      .graph.nodes.find((n) => n.id === nodeId)?.runtime_state?.metrics
      ?.latest_price;
    if (initialPrice != null && ref.current) {
      ref.current.textContent = String(initialPrice);
    }

    // 订阅 store 变化 — 直接操作 DOM，不触发 React 重渲染
    const unsub = useGraphStore.subscribe((state, prevState) => {
      if (state.graph.nodes === prevState.graph.nodes) return;

      const node = state.graph.nodes.find((n) => n.id === nodeId);
      if (!node || node.type !== "data") return;

      const price = node.runtime_state?.metrics?.latest_price;
      if (price != null && ref.current) {
        ref.current.textContent = String(price);
      }
    });

    return unsub;
  }, [nodeId]);

  return (
    <span
      ref={ref}
      className="ticker-price-overlay"
      data-testid={`ticker-price-${nodeId}`}
    >
      --
    </span>
  );
}, () => true);

function stopCanvasControlEvent(event) {
  event.stopPropagation();
}

const BaseNodeCard = memo(function BaseNodeCard({ data, selected }) {
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const updateNodeConfig = useGraphStore((state) => state.updateNodeConfig);
  const toggleNodeCollapse = useGraphStore((state) => state.toggleNodeCollapse);
  const {
    nodeId,
    nodeType,
    runtimeStatus,
    title,
    subtitle,
    inputPorts,
    outputPorts,
    highlighted,
    simplified,
    handlesConnectable,
    summaryValues,
    quickFieldDefinitions,
    issueMessage,
    metricLabel,
    collapsed,
    dimmed,
    focusMode,
    recommendationRole
  } = data;

  const isSelected = selected;
  const isSimplified = Boolean(simplified) && !isSelected;
  const focusClassName = focusMode ? `focus-${focusMode}` : "";
  const dimmedClassName = dimmed && !isSelected ? "dimmed" : "";
  const recommendationClassName = recommendationRole
    ? `recommendation-${recommendationRole}`
    : "";
  const recommendationLabel =
    recommendationRole === "path-end"
      ? "下一步修复"
      : recommendationRole === "path"
        ? "修复路径"
        : recommendationRole === "path-start"
          ? "当前选中"
          : recommendationRole === "recommended"
            ? "推荐"
            : null;

  return (
    <div
      className={`node-card node-${nodeType} ${isSelected ? "selected" : ""} ${highlighted ? "highlighted" : ""} ${isSimplified ? "simplified" : "rich"} ${focusClassName} ${dimmedClassName} ${recommendationClassName} status-${runtimeStatus}`.trim()}
      data-node-card-id={nodeId}
      data-node-card-type={nodeType}
      data-node-card-variant={isSimplified ? "simplified" : "full"}
      data-node-card-price-overlay={nodeType === "data" ? "active" : undefined}
      onClick={() => setSelectedNode(nodeId)}
    >
      {/* v2.5.0 P2-5: 价格涌动效果 — data 节点显示实时价格覆盖层 */}
      {nodeType === "data" && <PriceOverlay nodeId={nodeId} />}

      {inputPorts.length > 0 ? (
        inputPorts.map((port, index) => (
          <Handle
            key={port.key}
            type="target"
            position={Position.Left}
            id={port.key}
            isConnectable={handlesConnectable}
            style={{ top: 48 + index * 22 }}
            className={`port-handle port-${nodeType} ${handlesConnectable ? "" : "port-handle-passive"}`.trim()}
            data-testid={`handle-target-${nodeId}-${port.key}`}
          />
        ))
      ) : (
        <Handle
          key={`${nodeId}-target-default`}
          type="target"
          position={Position.Left}
          id="default_input"
          isConnectable={false}
          style={{ top: 48 }}
          className={`port-handle port-${nodeType} port-handle-passive`}
          data-testid={`handle-target-${nodeId}-default`}
        />
      )}

      <div className="node-header">
        <div>
          <div className="node-title">{title}</div>
          <div className="node-subtitle">{subtitle}</div>
          {recommendationLabel ? (
            <div className="node-recommendation-pill">{recommendationLabel}</div>
          ) : null}
        </div>
        <button
          className="collapse-btn nodrag nopan"
          onPointerDown={stopCanvasControlEvent}
          onClick={(event) => {
            event.stopPropagation();
            toggleNodeCollapse(nodeId);
          }}
        >
          {collapsed ? "+" : "-"}
        </button>
      </div>

      <div className={`node-summary ${isSimplified ? "compact" : ""}`}>
        {summaryValues.map((value, index) => (
          <span key={`${nodeId}-summary-${index}`} className="summary-chip">{value}</span>
        ))}
      </div>

      {!isSimplified && !collapsed && quickFieldDefinitions.length > 0 ? (
        <div
          className="node-quick-fields nodrag nopan"
          onPointerDown={stopCanvasControlEvent}
          onClick={stopCanvasControlEvent}
        >
          {quickFieldDefinitions.map((field) => (
            <label key={field.key} className="quick-field nodrag nopan">
              <span>{field.label}</span>
              {field.type === "select" ? (
                <select
                  className="nodrag nopan"
                  value={field.value}
                  onPointerDown={stopCanvasControlEvent}
                  onClick={stopCanvasControlEvent}
                  onChange={(event) => updateNodeConfig(nodeId, field.key, event.target.value)}
                >
                  {field.options.map((option) => (
                    <option key={option.value} value={option.value}>{option.label}</option>
                  ))}
                </select>
              ) : (
                <input
                  className="nodrag nopan"
                  type={field.type === "number" ? "number" : "text"}
                  value={field.value}
                  onPointerDown={stopCanvasControlEvent}
                  onClick={stopCanvasControlEvent}
                  onChange={(event) =>
                    updateNodeConfig(
                      nodeId,
                      field.key,
                      field.type === "number"
                        ? Number(event.target.value)
                        : event.target.value
                    )
                  }
                />
              )}
            </label>
          ))}
        </div>
      ) : null}

      <div className="node-runtime">
        <div className="runtime-line">
          <span className={`runtime-dot ${runtimeStatus}`}></span>
          <span>{metricLabel}</span>
        </div>
        {!isSimplified && issueMessage ? <div className="node-issue">{issueMessage}</div> : null}
      </div>

      {outputPorts.length > 0 ? (
        outputPorts.map((port, index) => (
          <Handle
            key={port.key}
            type="source"
            position={Position.Right}
            id={port.key}
            isConnectable={handlesConnectable}
            style={{ top: 48 + index * 22 }}
            className={`port-handle port-${nodeType} ${handlesConnectable ? "" : "port-handle-passive"}`.trim()}
            data-testid={`handle-source-${nodeId}-${port.key}`}
          />
        ))
      ) : (
        <Handle
          key={`${nodeId}-source-default`}
          type="source"
          position={Position.Right}
          id="default_output"
          isConnectable={false}
          style={{ top: 48 }}
          className={`port-handle port-${nodeType} port-handle-passive`}
          data-testid={`handle-source-${nodeId}-default`}
        />
      )}
    </div>
  );
}, baseNodeCardAreEqual);

// v2.5.0 P2-5: 自定义比较器 — 当仅 latestPrice/symbol 变化时跳过重渲染
// 价格通过 PriceOverlay 订阅 store 直接更新 DOM，无需 React 参与
function baseNodeCardAreEqual(prevProps, nextProps) {
  if (prevProps.selected !== nextProps.selected) return false;

  const pd = prevProps.data || {};
  const nd = nextProps.data || {};

  return (
    pd.nodeId === nd.nodeId &&
    pd.nodeType === nd.nodeType &&
    pd.runtimeStatus === nd.runtimeStatus &&
    pd.title === nd.title &&
    pd.subtitle === nd.subtitle &&
    pd.highlighted === nd.highlighted &&
    pd.simplified === nd.simplified &&
    pd.handlesConnectable === nd.handlesConnectable &&
    pd.issueMessage === nd.issueMessage &&
    pd.metricLabel === nd.metricLabel &&
    pd.collapsed === nd.collapsed &&
    pd.dimmed === nd.dimmed &&
    pd.focusMode === nd.focusMode &&
    pd.recommendationRole === nd.recommendationRole &&
    pd.inputPorts === nd.inputPorts &&
    pd.outputPorts === nd.outputPorts &&
    pd.summaryValues === nd.summaryValues &&
    pd.quickFieldDefinitions === nd.quickFieldDefinitions
  );
  // 忽略: latestPrice, symbol — 由 PriceOverlay 通过 DOM 直写处理
}

export default BaseNodeCard;
