export function buildRepairPathInsight(target, graph, repairPathState) {
  const pathNodeIds = repairPathState?.pathNodeIds || [];
  const pathEdgeIds = repairPathState?.pathEdgeIds || [];
  if (!graph || !target || (pathNodeIds.length === 0 && pathEdgeIds.length === 0)) {
    return null;
  }

  const nodeMap = new Map((graph.nodes || []).map((node) => [node.id, node]));

  if (target.scope === "node" && target.node_id) {
    const nodeIndex = pathNodeIds.indexOf(target.node_id);
    if (nodeIndex < 0) return null;

    const currentLabel = nodeMap.get(target.node_id)?.name || target.label || target.node_id;
    if (nodeIndex === 0) {
      return {
        chip: "当前选中",
        segment: currentLabel,
        note: "该项位于当前激活修复路径的起点，并与当前选中对象对齐。"
      };
    }

    const previousId = pathNodeIds[nodeIndex - 1];
    const previousLabel = nodeMap.get(previousId)?.name || previousId;
    const isEndpoint = nodeIndex === pathNodeIds.length - 1;

    return {
      chip: isEndpoint ? "下一步修复" : "修复路径",
      segment: `${previousLabel} -> ${currentLabel}`,
      note: isEndpoint
        ? "该项是当前激活修复路径的终点。"
        : "该项位于画布中高亮显示的激活修复路径片段上。"
    };
  }

  if (target.scope === "edge" && target.edge_id && pathEdgeIds.includes(target.edge_id)) {
    const edge = (graph.edges || []).find((item) => item.id === target.edge_id);
    if (!edge) return null;

    const sourceNode = nodeMap.get(edge.source_node_id);
    const targetNode = nodeMap.get(edge.target_node_id);
    return {
      chip: "修复路径连线",
      segment: `${sourceNode?.name || edge.source_node_id} -> ${
        targetNode?.name || edge.target_node_id
      }`,
      note: "该项对应的连线已经作为激活修复路径的一部分在画布中高亮显示。"
    };
  }

  return null;
}

