export const nodeLaneX = {
  data: 120,
  intent: 420,
  agent: 720,
  risk: 1020,
  execution: 1320,
  runtime: 40
};

export const initialNodeLaneY = {
  data: 120,
  intent: 120,
  agent: 120,
  risk: 120,
  execution: 120,
  runtime: 24
};

export function createNodePositionAllocator() {
  const laneYCounter = { ...initialNodeLaneY };

  return function nextPosition(category) {
    const x = nodeLaneX[category] || 120;
    const y = laneYCounter[category] || 120;
    laneYCounter[category] = y + 180;
    return { x, y };
  };
}
