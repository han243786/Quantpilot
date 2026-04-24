const FOCUS_NODE_WIDTH = 250;
const FOCUS_NODE_HEIGHT = 140;
const FOCUS_PADDING_X = 120;
const FOCUS_PADDING_Y = 84;

export const CANVAS_FOCUS_MODES = ["selected", "issues", "recent"];
const DIAGNOSTIC_RECOMMENDATION_TYPES = [
  "execution",
  "risk",
  "runtime",
  "agent",
  "data",
  "intent"
];

function graphNodeIds(graph) {
  return new Set((graph?.nodes || []).map((node) => node.id));
}

function graphNodeMap(graph) {
  return new Map((graph?.nodes || []).map((node) => [node.id, node]));
}

function shortestPathNodeIds(graph, startId, targetId) {
  if (!startId || !targetId || startId === targetId) {
    return startId && targetId ? [startId] : null;
  }

  const validNodeIds = graphNodeIds(graph);
  if (!validNodeIds.has(startId) || !validNodeIds.has(targetId)) {
    return null;
  }

  const adjacency = new Map();
  (graph?.edges || []).forEach((edge) => {
    if (!adjacency.has(edge.source_node_id)) {
      adjacency.set(edge.source_node_id, []);
    }
    adjacency.get(edge.source_node_id).push(edge.target_node_id);
  });

  const queue = [[startId]];
  const visited = new Set([startId]);

  while (queue.length > 0) {
    const path = queue.shift();
    const currentId = path[path.length - 1];
    const nextIds = adjacency.get(currentId) || [];

    for (const nextId of nextIds) {
      if (visited.has(nextId)) continue;
      const nextPath = [...path, nextId];
      if (nextId === targetId) {
        return nextPath;
      }
      visited.add(nextId);
      queue.push(nextPath);
    }
  }

  return null;
}

function uniqueNodeIds(ids = []) {
  return ids.filter((id, index) => ids.indexOf(id) === index);
}

function diagnosticRecommendationRank(nodeType) {
  const index = DIAGNOSTIC_RECOMMENDATION_TYPES.indexOf(nodeType);
  return index >= 0 ? index : DIAGNOSTIC_RECOMMENDATION_TYPES.length;
}

export function resolveCanvasRecommendations(graph, selectedNodeId, laneId = null) {
  if (!selectedNodeId || laneId !== "diagnostics") {
    return {
      recommendedNodeIds: [],
      pathNodeIds: [],
      pathEdgeIds: [],
      issueNodeIds: []
    };
  }

  const nodeMap = graphNodeMap(graph);
  const selectedNode = nodeMap.get(selectedNodeId);
  if (!selectedNode) {
    return {
      recommendedNodeIds: [],
      pathNodeIds: [],
      pathEdgeIds: [],
      issueNodeIds: []
    };
  }

  const issueCandidates = collectIssueNodeIds(graph)
    .filter((nodeId) => nodeId !== selectedNodeId)
    .map((nodeId) => ({
      nodeId,
      node: nodeMap.get(nodeId) || null,
      pathFromSelection: shortestPathNodeIds(graph, selectedNodeId, nodeId)
    }))
    .filter((candidate) => candidate.node);

  const orderedIssueCandidates = issueCandidates.sort((left, right) => {
    const typeRankDelta =
      diagnosticRecommendationRank(left.node.type) - diagnosticRecommendationRank(right.node.type);
    if (typeRankDelta !== 0) return typeRankDelta;

    const pathAvailabilityDelta =
      Number(Boolean(left.pathFromSelection)) - Number(Boolean(right.pathFromSelection));
    if (pathAvailabilityDelta !== 0) return pathAvailabilityDelta * -1;

    const pathLengthDelta =
      (left.pathFromSelection?.length || Number.POSITIVE_INFINITY) -
      (right.pathFromSelection?.length || Number.POSITIVE_INFINITY);
    if (pathLengthDelta !== 0) return pathLengthDelta;

    return (left.node.name || left.node.id).localeCompare(right.node.name || right.node.id);
  });

  const stitchedPath = [selectedNodeId];
  let currentAnchorId = selectedNodeId;

  orderedIssueCandidates.forEach((candidate) => {
    const path =
      shortestPathNodeIds(graph, currentAnchorId, candidate.nodeId) || candidate.pathFromSelection;
    if (!path || path.length <= 1) return;

    path.slice(1).forEach((nodeId) => {
      if (!stitchedPath.includes(nodeId)) {
        stitchedPath.push(nodeId);
      }
    });
    currentAnchorId = candidate.nodeId;
  });

  const recommendedNodeIds = uniqueNodeIds([
    selectedNodeId,
    ...stitchedPath.slice(1),
    ...orderedIssueCandidates.map((candidate) => candidate.nodeId)
  ]).slice(0, 4);
  const pathEdgeIds = [];

  for (let index = 0; index < stitchedPath.length - 1; index += 1) {
    const sourceId = stitchedPath[index];
    const targetId = stitchedPath[index + 1];
    const matchedEdge = (graph?.edges || []).find(
      (edge) => edge.source_node_id === sourceId && edge.target_node_id === targetId
    );
    if (matchedEdge?.id) {
      pathEdgeIds.push(matchedEdge.id);
    }
  }

  return {
    recommendedNodeIds,
    pathNodeIds: stitchedPath.length > 1 ? stitchedPath : [],
    pathEdgeIds,
    issueNodeIds: orderedIssueCandidates.map((candidate) => candidate.nodeId)
  };
}

export function collectIssueNodeIds(graph) {
  const nodeIssues = graph?.validation_state?.node_issues || {};

  return (graph?.nodes || [])
    .filter((node) => Array.isArray(nodeIssues[node.id]) && nodeIssues[node.id].length > 0)
    .map((node) => node.id);
}

export function collectRecentNodeIds(graph) {
  const validNodeIds = graphNodeIds(graph);
  const recentNodeIds = Array.isArray(graph?.metadata?.editor?.recent_node_ids)
    ? graph.metadata.editor.recent_node_ids
    : [];

  return recentNodeIds.filter((nodeId) => validNodeIds.has(nodeId));
}

export function resolveCanvasFocusTargetIds(graph, selectedNodeId, focusMode) {
  if (focusMode === "issues") {
    return collectIssueNodeIds(graph);
  }

  if (focusMode === "recent") {
    return collectRecentNodeIds(graph);
  }

  if (selectedNodeId && graphNodeIds(graph).has(selectedNodeId)) {
    return [selectedNodeId];
  }

  return [];
}

export function resolveCanvasFocusAnchorId(targetIds = [], selectedNodeId = null, focusMode) {
  if (focusMode === "selected") {
    return targetIds[0] || null;
  }

  return targetIds.includes(selectedNodeId) ? selectedNodeId : null;
}

export function resolveCanvasActiveTargetId(targetIds = [], selectedNodeId = null) {
  if (targetIds.length === 0) return null;
  if (selectedNodeId && targetIds.includes(selectedNodeId)) return selectedNodeId;
  return targetIds[0];
}

export function cycleCanvasFocusTarget(targetIds = [], currentTargetId = null, direction = 1) {
  if (targetIds.length === 0) return null;
  if (targetIds.length === 1) return targetIds[0];

  const currentIndex = currentTargetId ? targetIds.indexOf(currentTargetId) : -1;
  const normalizedIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = (normalizedIndex + direction + targetIds.length) % targetIds.length;
  return targetIds[nextIndex];
}

export function buildCanvasFocusBounds(nodes = [], targetIds = []) {
  const targetSet = new Set(targetIds);
  const matchedNodes = nodes.filter((node) => targetSet.has(node.id));
  if (matchedNodes.length === 0) return null;

  const bounds = matchedNodes.reduce(
    (accumulator, node) => ({
      minX: Math.min(accumulator.minX, node.position.x),
      minY: Math.min(accumulator.minY, node.position.y),
      maxX: Math.max(accumulator.maxX, node.position.x + FOCUS_NODE_WIDTH),
      maxY: Math.max(accumulator.maxY, node.position.y + FOCUS_NODE_HEIGHT)
    }),
    {
      minX: Number.POSITIVE_INFINITY,
      minY: Number.POSITIVE_INFINITY,
      maxX: Number.NEGATIVE_INFINITY,
      maxY: Number.NEGATIVE_INFINITY
    }
  );

  return {
    x: bounds.minX - FOCUS_PADDING_X,
    y: bounds.minY - FOCUS_PADDING_Y,
    width: bounds.maxX - bounds.minX + FOCUS_PADDING_X * 2,
    height: bounds.maxY - bounds.minY + FOCUS_PADDING_Y * 2
  };
}
