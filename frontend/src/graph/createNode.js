let sequence = 1;

const laneX = {
  data: 120,
  intent: 420,
  agent: 720,
  risk: 1020,
  execution: 1320,
  runtime: 40
};

const laneYCounter = {
  data: 120,
  intent: 120,
  agent: 120,
  risk: 120,
  execution: 120,
  runtime: 24
};

function nextPosition(category) {
  const x = laneX[category] || 120;
  const y = laneYCounter[category] || 120;
  laneYCounter[category] = y + 180;
  return { x, y };
}

export function createNodeFromModule(moduleDef) {
  const id = `node_${moduleDef.category}_${sequence++}`;
  const config = {};

  (moduleDef.config_schema?.fields || []).forEach((field) => {
    config[field.key] = field.default;
  });

  return {
    id,
    type: moduleDef.category,
    module_key: moduleDef.module_key,
    name: moduleDef.node.default_name,
    position: nextPosition(moduleDef.category),
    config,
    input_ports: moduleDef.ports.inputs || [],
    output_ports: moduleDef.ports.outputs || [],
    ui_state: {
      collapsed: false
    },
    runtime_state: {
      status: "idle",
      last_event_type: null,
      last_event_time: null,
      last_message: "",
      metrics: {},
      error: null
    }
  };
}
