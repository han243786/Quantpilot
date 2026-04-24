import { Handle, Position } from "@xyflow/react";
import { useGraphStore } from "../store/graphStore";

export default function BaseNodeCard({ data, selected }) {
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
      onClick={() => setSelectedNode(nodeId)}
    >
      {inputPorts.map((port, index) => (
        <Handle
          key={port.key}
          type="target"
          position={Position.Left}
          id={port.key}
          isConnectable={handlesConnectable}
          style={{ top: 48 + index * 22 }}
          className={`port-handle port-${nodeType} ${handlesConnectable ? "" : "port-handle-passive"}`.trim()}
        />
      ))}

      <div className="node-header">
        <div>
          <div className="node-title">{title}</div>
          <div className="node-subtitle">{subtitle}</div>
          {recommendationLabel ? (
            <div className="node-recommendation-pill">{recommendationLabel}</div>
          ) : null}
        </div>
        <button
          className="collapse-btn"
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
        <div className="node-quick-fields">
          {quickFieldDefinitions.map((field) => (
            <label key={field.key} className="quick-field">
              <span>{field.label}</span>
              {field.type === "select" ? (
                <select
                  value={field.value}
                  onChange={(event) => updateNodeConfig(nodeId, field.key, event.target.value)}
                >
                  {field.options.map((option) => (
                    <option key={option.value} value={option.value}>{option.label}</option>
                  ))}
                </select>
              ) : (
                <input
                  type={field.type === "number" ? "number" : "text"}
                  value={field.value}
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

      {outputPorts.map((port, index) => (
        <Handle
          key={port.key}
          type="source"
          position={Position.Right}
          id={port.key}
          isConnectable={handlesConnectable}
          style={{ top: 48 + index * 22 }}
          className={`port-handle port-${nodeType} ${handlesConnectable ? "" : "port-handle-passive"}`.trim()}
        />
      ))}
    </div>
  );
}
