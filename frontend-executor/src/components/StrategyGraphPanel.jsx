/// v3.4.0: 锁定策略图渲染 — 拓扑不可编辑, 节点显示实时价格涌动

import { memo, useCallback, useEffect, useRef, useState } from "react";
import { ReactFlow, ReactFlowProvider, Background } from "@xyflow/react";
import "@xyflow/react/dist/style.css";

const API = "/api/executor";

const StrategyGraphPanel = memo(function StrategyGraphPanel({ strategy }) {
  const [nodes, setNodes] = useState([]);
  const [edges, setEdges] = useState([]);
  const [pulsingNodes, setPulsingNodes] = useState(new Set());

  // 从策略图 JSON 构建 React Flow 节点 (锁定模式)
  useEffect(() => {
    if (!strategy?.graph_json) return;
    const g = strategy.graph_json;
    const ns = (g.nodes || []).map((n, i) => ({
      id: n.id || `node_${i}`,
      type: "default",
      position: n.position || { x: 120 + i * 300, y: 200 },
      data: {
        label: `${n.type || "节点"}\n${n.module_key?.split(".").pop() || ""}`,
      },
      style: { background: "var(--exec-card)", border: "1px solid var(--exec-border)", color: "var(--exec-text)", borderRadius: 4, padding: "8px 16px", fontSize: 12 },
      draggable: false,
      selectable: false,
    }));
    const es = (g.edges || []).map((e, i) => ({
      id: e.id || `edge_${i}`,
      source: e.source_node_id || e.source,
      target: e.target_node_id || e.target,
      style: { stroke: "var(--exec-border)" },
    }));
    setNodes(ns);
    setEdges(es);
  }, [strategy]);

  // v3.5.0: 价格DOM overlay — useRef直接DOM更新, 避免每ticker触发React重渲染
  const priceOverlaysRef = useRef({});

  useEffect(() => {
    if (!strategy?.strategy_id) return;
    const es = new EventSource(`${API}/strategies/${strategy.strategy_id}/events`);
    const handleStreamEvent = (event) => {
      try {
        const data = JSON.parse(event.data);
        const eventType = event.type === "message" ? data.type : event.type;
        if (eventType === "ticker") {
          const el = priceOverlaysRef.current[data.symbol];
          if (el) el.textContent = `$${Number(data.price).toFixed(2)}`;
        }
        if (eventType === "trigger") {
          setPulsingNodes(prev => new Set([...prev, data.node_id]));
          setTimeout(() => setPulsingNodes(prev => {
            const next = new Set(prev); next.delete(data.node_id); return next;
          }), 600);
        }
      } catch (e) { console.warn("[StrategyGraph] SSE parse error:", e.message); }
    };
    es.addEventListener("trigger", handleStreamEvent);
    es.addEventListener("ticker", handleStreamEvent);
    es.onmessage = handleStreamEvent;
    es.onerror = () => console.warn("[StrategyGraph] SSE 连接错误, 将自动重连");
    return () => {
      es.removeEventListener("trigger", handleStreamEvent);
      es.removeEventListener("ticker", handleStreamEvent);
      es.close();
    };
  }, [strategy?.strategy_id]);

  const nodeTypes = {}; // 使用默认节点类型

  return (
    <ReactFlowProvider>
      {/* v3.5.0: 实时价格条 — DOM直接更新, 无React重渲染 */}
      <div className="exec-price-ticker" ref={(el) => { priceOverlaysRef.current["_ticker"] = el; }}>
        {strategy?.subscribed_symbols?.map(sym =>
          <span key={sym} className="exec-price-item" id={`price-${sym}`}>--</span>
        )}
      </div>
      <ReactFlow
        nodes={nodes.map(n => ({
          ...n,
          className: pulsingNodes.has(n.id) ? "exec-graph-node-pulse" : "",
          data: {
            ...n.data,
            // v3.5.0: 价格由DOM overlay渲染, 节点仅显示名称
            label: n.data.label,
          },
        }))}
        edges={edges}
        nodeTypes={nodeTypes}
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        panOnDrag={false}
        zoomOnScroll={false}
        fitView
      >
        <Background color="var(--exec-border)" gap={20} />
      </ReactFlow>
    </ReactFlowProvider>
  );
});
export default StrategyGraphPanel;
