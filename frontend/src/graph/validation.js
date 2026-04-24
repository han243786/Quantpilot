import { DEFAULT_CAPABILITIES, normalizeCapabilities } from "../modules/builtinModules";

const allowedChain = {
  data: ["intent"],
  intent: ["agent"],
  agent: ["risk"],
  risk: ["execution"],
  execution: [],
  runtime: []
};

const typeLabels = {
  data: "数据",
  intent: "意图",
  agent: "代理",
  risk: "风控",
  execution: "执行",
  runtime: "运行"
};

function capabilitySet(values, fallback) {
  return new Set(Array.isArray(values) && values.length > 0 ? values : fallback);
}

function supportMap(entries, keyField = "key") {
  return new Map(
    (Array.isArray(entries) ? entries : [])
      .filter((entry) => entry && typeof entry === "object" && entry[keyField])
      .map((entry) => [entry[keyField], entry])
  );
}

function capabilityEntryStatus(entry, fallbackSet, key) {
  if (entry) return entry.status === "supported";
  return fallbackSet.has(key);
}

function capabilityReason(entry, fallback = "") {
  return entry?.reason || fallback;
}

function compareValues(leftValue, operator, rightValue) {
  if (operator === "<") return leftValue < rightValue;
  if (operator === "<=") return leftValue <= rightValue;
  if (operator === ">") return leftValue > rightValue;
  if (operator === ">=") return leftValue >= rightValue;
  if (operator === "===") return leftValue === rightValue;
  return true;
}

function buildIssue(level, scope, targetId, code, message, hint = "") {
  return {
    id: `${scope}_${targetId}_${code}`,
    level,
    scope,
    target_id: targetId,
    code,
    message,
    hint
  };
}

export function isValidConnection(graph, registry, connection) {
  const sourceNode = graph.nodes.find((node) => node.id === connection.source);
  const targetNode = graph.nodes.find((node) => node.id === connection.target);

  if (!sourceNode || !targetNode) return { valid: false, reason: "节点不存在。" };
  if (sourceNode.type === "runtime" || targetNode.type === "runtime") {
    return { valid: false, reason: "运行控制节点不参与数据流连线。" };
  }
  if (!(allowedChain[sourceNode.type] || []).includes(targetNode.type)) {
    return {
      valid: false,
      reason: `${typeLabels[sourceNode.type] || sourceNode.type}节点只能连接到合法的下游层。`
    };
  }

  const sourcePort = (sourceNode.output_ports || []).find(
    (port) => port.key === connection.sourceHandle
  );
  const targetPort = (targetNode.input_ports || []).find(
    (port) => port.key === connection.targetHandle
  );

  if (!sourcePort || !targetPort) {
    return { valid: false, reason: "端口不存在，或端口类型不匹配。" };
  }
  if (targetPort.accepts && !targetPort.accepts.includes(sourcePort.provides)) {
    return { valid: false, reason: "端口数据类型不兼容。" };
  }

  if (sourceNode.type === "agent") {
    const riskTargetIds = new Set(
      graph.edges
        .filter((edge) => edge.source_node_id === sourceNode.id)
        .filter((edge) => {
          const node = graph.nodes.find((item) => item.id === edge.target_node_id);
          return node?.type === "risk";
        })
        .map((edge) => edge.target_node_id)
    );
    if (riskTargetIds.size >= 1 && !riskTargetIds.has(targetNode.id)) {
      return { valid: false, reason: "一个代理节点最多只能连接一个风控节点。" };
    }
  }

  if (targetNode.type === "execution") {
    const executionSourceIds = new Set(
      graph.edges
        .filter((edge) => edge.target_node_id === targetNode.id)
        .map((edge) => edge.source_node_id)
    );
    if (executionSourceIds.size >= 1 && !executionSourceIds.has(sourceNode.id)) {
      return { valid: false, reason: "一个执行节点只能接收一个风控输入。" };
    }
  }

  return { valid: true, reason: "" };
}

export function validateGraph(graph, registry) {
  const capabilities = normalizeCapabilities(registry?.capabilities || DEFAULT_CAPABILITIES);
  const supportedRuntimeModes = capabilitySet(
    capabilities.runtime?.supported_modes,
    DEFAULT_CAPABILITIES.runtime.supported_modes
  );
  const supportedExecutionModules = capabilitySet(
    capabilities.runtime?.supported_execution_modules,
    DEFAULT_CAPABILITIES.runtime.supported_execution_modules
  );
  const supportedSymbols = capabilitySet(
    capabilities.market_data?.supported_symbols,
    DEFAULT_CAPABILITIES.market_data.supported_symbols
  );
  const supportedExchanges = capabilitySet(
    capabilities.market_data?.supported_exchanges,
    DEFAULT_CAPABILITIES.market_data.supported_exchanges
  );
  const runtimeModeSupport = supportMap(capabilities.runtime?.mode_support);
  const executionModuleSupport = supportMap(capabilities.runtime?.execution_module_support);
  const exchangeSupport = supportMap(capabilities.market_data?.exchange_support);
  const symbolSupport = supportMap(capabilities.market_data?.symbol_support);
  const frontendModuleSupport = supportMap(capabilities.frontend?.module_support, "module_key");

  const nodeIssues = {};
  const edgeIssues = {};
  const graphIssues = [];
  const counts = { error: 0, warning: 0, info: 0 };

  const addNodeIssue = (nodeId, issue) => {
    nodeIssues[nodeId] = nodeIssues[nodeId] || [];
    nodeIssues[nodeId].push(issue);
    counts[issue.level] += 1;
  };

  const addEdgeIssue = (edgeId, issue) => {
    edgeIssues[edgeId] = edgeIssues[edgeId] || [];
    edgeIssues[edgeId].push(issue);
    counts[issue.level] += 1;
  };

  const addGraphIssue = (issue) => {
    graphIssues.push(issue);
    counts[issue.level] += 1;
  };

  graph.nodes.forEach((node) => {
    const incoming = graph.edges.filter((edge) => edge.target_node_id === node.id);
    const outgoing = graph.edges.filter((edge) => edge.source_node_id === node.id);

    const moduleDef = registry.getByKey(node.module_key);
    if (!moduleDef) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "MODULE_NOT_FOUND", "节点对应的模块未注册。")
      );
      return;
    }

    if (moduleDef.availability?.status === "unsupported") {
      const moduleSupportEntry = frontendModuleSupport.get(node.module_key);
      addNodeIssue(
        node.id,
        buildIssue(
          "error",
          "node",
          node.id,
          "UNSUPPORTED_MODULE",
          `模块“${moduleDef.display_name}”当前未开放。`,
          capabilityReason(moduleSupportEntry, moduleDef.availability.reason || "当前版本未开放该模块。")
        )
      );
    }

    (moduleDef.config_schema?.fields || []).forEach((field) => {
      const value = node.config[field.key];
      if (field.required && (value === "" || value === null || value === undefined)) {
        addNodeIssue(
          node.id,
          buildIssue("error", "node", node.id, "FIELD_REQUIRED", `必须填写“${field.label}”。`)
        );
      }
      if (field.type === "number" && value !== null && value !== undefined && value !== "") {
        if (typeof value !== "number" || Number.isNaN(value)) {
          addNodeIssue(
            node.id,
            buildIssue("error", "node", node.id, "FIELD_TYPE", `“${field.label}”必须是数字。`)
          );
        }
        if (field.min !== undefined && value < field.min) {
          addNodeIssue(
            node.id,
            buildIssue("error", "node", node.id, "FIELD_MIN", `“${field.label}”必须 >= ${field.min}。`)
          );
        }
        if (field.max !== undefined && value > field.max) {
          addNodeIssue(
            node.id,
            buildIssue("error", "node", node.id, "FIELD_MAX", `“${field.label}”必须 <= ${field.max}。`)
          );
        }
      }
    });

    (moduleDef.constraints?.node_rules || []).forEach((rule) => {
      if (rule.rule === "compare") {
        const leftValue = node.config[rule.left];
        const rightValue = node.config[rule.right];
        if (!compareValues(leftValue, rule.operator, rightValue)) {
          addNodeIssue(
            node.id,
            buildIssue("error", "node", node.id, "FIELD_COMPARE", rule.message)
          );
        }
      }
    });

    if (node.type === "data") {
      const exchangeEntry = exchangeSupport.get(node.config.exchange);
      if (
        node.config.exchange &&
        !capabilityEntryStatus(exchangeEntry, supportedExchanges, node.config.exchange)
      ) {
        addNodeIssue(
          node.id,
          buildIssue(
            "error",
            "node",
            node.id,
            "UNSUPPORTED_EXCHANGE",
            `当前不支持交易所“${node.config.exchange}”。`,
            capabilityReason(exchangeEntry, "")
          )
        );
      }

      const symbolEntry = symbolSupport.get(node.config.instrument);
      if (
        node.config.instrument &&
        !capabilityEntryStatus(symbolEntry, supportedSymbols, node.config.instrument)
      ) {
        addNodeIssue(
          node.id,
          buildIssue(
            "error",
            "node",
            node.id,
            "UNSUPPORTED_SYMBOL",
            `当前不支持交易对“${node.config.instrument}”。`,
            capabilityReason(symbolEntry, "")
          )
        );
      }
    }

    const executionEntry = executionModuleSupport.get(node.module_key);
    if (
      node.type === "execution" &&
      !capabilityEntryStatus(executionEntry, supportedExecutionModules, node.module_key)
    ) {
      addNodeIssue(
        node.id,
        buildIssue(
          "error",
          "node",
          node.id,
          "UNSUPPORTED_EXECUTION",
          "当前 beta 仅支持模拟执行模块。",
          capabilityReason(executionEntry, "")
        )
      );
    }

    const runtimeModeEntry = runtimeModeSupport.get(node.config.mode);
    if (
      node.type === "runtime" &&
      node.config.mode &&
      !capabilityEntryStatus(runtimeModeEntry, supportedRuntimeModes, node.config.mode)
    ) {
      addNodeIssue(
        node.id,
        buildIssue(
          "error",
          "node",
          node.id,
          "UNSUPPORTED_RUNTIME_MODE",
          "当前 beta 仅支持 paper 运行模式。",
          capabilityReason(runtimeModeEntry, "")
        )
      );
    }

    if (node.type === "intent" && incoming.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "INTENT_NO_INPUT", "意图节点必须连接上游市场数据。")
      );
    }

    if (node.module_key === "builtin.intent.spread_observer" && incoming.length < 2) {
      addNodeIssue(
        node.id,
        buildIssue(
          "error",
          "node",
          node.id,
          "SPREAD_INPUT_COUNT",
          "Spread observer must connect to at least two quote sources."
        )
      );
    }

    if (node.module_key === "builtin.intent.spread_observer" && incoming.length >= 2) {
      const sourceNodes = incoming
        .map((edge) => graph.nodes.find((item) => item.id === edge.source_node_id))
        .filter(Boolean);
      const uniqueSources = new Set(sourceNodes.map((item) => item.id));
      const nonDataSources = sourceNodes.filter((item) => item.type !== "data");
      if (uniqueSources.size < 2) {
        addNodeIssue(
          node.id,
          buildIssue(
            "error",
            "node",
            node.id,
            "SPREAD_DUPLICATE_SOURCE",
            "Spread observer must consume two distinct upstream data sources."
          )
        );
      }
      if (nonDataSources.length > 0) {
        addNodeIssue(
          node.id,
          buildIssue(
            "error",
            "node",
            node.id,
            "SPREAD_NON_DATA_INPUT",
            "Spread observer only accepts upstream market data sources."
          )
        );
      }
    }

    if (node.type === "agent" && incoming.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "AGENT_NO_INPUT", "代理节点必须连接至少一个意图输入。")
      );
    }

    if (node.type === "agent" && outgoing.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "AGENT_NO_OUTPUT", "代理节点必须连接到风控节点。")
      );
    }

    if (node.module_key === "builtin.agent.arbitrage") {
      const spreadInputs = incoming.filter((edge) => {
        const sourceNode = graph.nodes.find((item) => item.id === edge.source_node_id);
        return sourceNode?.module_key === "builtin.intent.spread_observer";
      });
      if (spreadInputs.length === 0) {
        addNodeIssue(
          node.id,
          buildIssue(
            "error",
            "node",
            node.id,
            "ARBITRAGE_INPUT_KIND",
            "Arbitrage agent must consume a spread observer intent."
          )
        );
      }
    }

    if (node.type === "risk" && incoming.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "RISK_NO_INPUT", "风控节点必须连接代理输入。")
      );
    }

    if (node.type === "risk" && outgoing.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "RISK_NO_OUTPUT", "风控节点必须连接到执行节点。")
      );
    }

    if (node.type === "execution" && incoming.length === 0) {
      addNodeIssue(
        node.id,
        buildIssue("error", "node", node.id, "EXECUTION_NO_INPUT", "执行节点必须连接风控输入。")
      );
    }
  });

  graph.edges.forEach((edge) => {
    const result = isValidConnection(graph, registry, {
      source: edge.source_node_id,
      sourceHandle: edge.source_port,
      target: edge.target_node_id,
      targetHandle: edge.target_port
    });
    if (!result.valid) {
      addEdgeIssue(edge.id, buildIssue("error", "edge", edge.id, "INVALID_EDGE", result.reason));
    }
  });

  const runtimeCount = graph.nodes.filter((node) => node.type === "runtime").length;
  if (runtimeCount === 0) {
    addGraphIssue(buildIssue("error", "graph", "graph", "MISSING_RUNTIME", "缺少运行控制节点。"));
  }
  if (runtimeCount > 1) {
    addGraphIssue(
      buildIssue("error", "graph", "graph", "MULTIPLE_RUNTIME", "当前仅支持一个运行控制节点。")
    );
  }

  const hasExecution = graph.nodes.some((node) => node.type === "execution");
  const hasRisk = graph.nodes.some((node) => node.type === "risk");
  const hasAgent = graph.nodes.some((node) => node.type === "agent");
  const hasIntent = graph.nodes.some((node) => node.type === "intent");
  const hasData = graph.nodes.some((node) => node.type === "data");

  if (hasExecution && !hasRisk) {
    addGraphIssue(buildIssue("error", "graph", "graph", "MISSING_RISK", "执行节点上游必须连接风控节点。"));
  }
  if (hasRisk && !hasAgent) {
    addGraphIssue(buildIssue("error", "graph", "graph", "MISSING_AGENT", "风控节点上游必须连接代理节点。"));
  }
  if (hasAgent && !hasIntent) {
    addGraphIssue(buildIssue("error", "graph", "graph", "MISSING_INTENT", "代理节点上游必须连接意图节点。"));
  }
  if (hasIntent && !hasData) {
    addGraphIssue(buildIssue("error", "graph", "graph", "MISSING_DATA", "意图节点上游必须连接数据节点。"));
  }

  graph.nodes.forEach((node) => {
    if (node.type === "intent") {
      const hasOutput = graph.edges.some((edge) => edge.source_node_id === node.id);
      if (!hasOutput) {
        addNodeIssue(
          node.id,
          buildIssue("warning", "node", node.id, "INTENT_NO_OUTPUT", "意图节点尚未连接到代理节点。")
        );
      }
    }
  });

  const hasCompleteChain =
    hasData &&
    hasIntent &&
    hasAgent &&
    hasRisk &&
    hasExecution &&
    graph.nodes.filter((node) => node.type === "execution").length === 1 &&
    counts.error === 0;

  return {
    is_valid: counts.error === 0,
    is_runnable: hasCompleteChain && counts.error === 0,
    node_issues: nodeIssues,
    edge_issues: edgeIssues,
    graph_issues: graphIssues,
    issue_counts: counts,
    last_validated_at: Date.now()
  };
}
