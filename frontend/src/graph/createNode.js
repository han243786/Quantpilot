import { createNodePositionAllocator } from "./nodeFactoryLayout";

let sequence = 1;

const nextPosition = createNodePositionAllocator();

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
