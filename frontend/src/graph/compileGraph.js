import { DEFAULT_CAPABILITIES, normalizeCapabilities } from "../modules/builtinModules";
import { attachQuantScriptArtifacts, generateGraphQuantScript } from "./quantscript";
import { buildCoreIr } from "./compileGraphCoreIr";
import {
  agentUsesPortfolioRebalance,
  buildLocalCompileDiagnostics,
  capabilityEntryStatus,
  capabilityReason,
  capabilitySet,
  normalizeRebalanceAllocationKind,
  normalizeRebalanceRankMethod,
  normalizeRebalanceSchedule,
  normalizeRebalanceScoreNormalize,
  parseCsvNumbers,
  parseCsvStrings,
  supportMap
} from "./compileGraphSupport";

function buildTopology(graph) {
  const indegree = new Map(graph.nodes.map((node) => [node.id, 0]));
  const outgoing = new Map(graph.nodes.map((node) => [node.id, []]));

  graph.edges.forEach((edge) => {
    if (!indegree.has(edge.target_node_id) || !outgoing.has(edge.source_node_id)) return;
    indegree.set(edge.target_node_id, indegree.get(edge.target_node_id) + 1);
    outgoing.get(edge.source_node_id).push(edge.target_node_id);
  });

  const queue = graph.nodes
    .filter((node) => indegree.get(node.id) === 0)
    .map((node) => node.id);
  const order = [];

  while (queue.length > 0) {
    const nodeId = queue.shift();
    order.push(nodeId);
    for (const nextId of outgoing.get(nodeId) || []) {
      indegree.set(nextId, indegree.get(nextId) - 1);
      if (indegree.get(nextId) === 0) queue.push(nextId);
    }
  }

  return {
    topologyOrder: order,
    hasCycle: order.length !== graph.nodes.length
  };
}

export function compileGraph(graph, registry = null) {
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

  const compileId = `compile_${Date.now()}`;
  const errors = [];
  const warnings = [];
  const output = {
    metadata: {
      graph_id: graph.metadata.graph_id,
      compile_id: compileId,
      name: graph.metadata.name,
      version: graph.metadata.version,
      mode: "paper"
    },
    data_sources: [],
    intent_generators: [],
    agents: [],
    risk_controls: [],
    executions: [],
    runtime_control: null,
    mappings: {
      source_id_to_node_id: {}
    }
  };

  const idMap = {};

  graph.nodes.forEach((node) => {
    const compiledId = `${node.type}_${node.id}`;
    idMap[node.id] = compiledId;
    output.mappings.source_id_to_node_id[compiledId] = node.id;
  });

  const runtimeNodes = graph.nodes.filter((node) => node.type === "runtime");
  if (runtimeNodes.length !== 1) {
    errors.push("策略图必须且只能包含一个运行控制节点。");
  }

  graph.nodes.forEach((node) => {
    const moduleDef = registry?.getByKey(node.module_key);
    const base = {
      id: idMap[node.id],
      module_key: node.module_key,
      name: node.name,
      config: node.config
    };

    if (moduleDef?.availability?.status === "unsupported") {
      const moduleSupportEntry = frontendModuleSupport.get(node.module_key);
      errors.push(
        `节点 ${node.name} 使用了当前未开放的模块 ${node.module_key}：${capabilityReason(moduleSupportEntry, moduleDef.availability.reason)}`
      );
    }

    if (node.type === "data") {
      const exchangeEntry = exchangeSupport.get(node.config.exchange);
      if (!capabilityEntryStatus(exchangeEntry, supportedExchanges, node.config.exchange)) {
        errors.push(
          `数据节点 ${node.name} 使用了未支持的交易所 ${node.config.exchange || "-"}。 ${capabilityReason(exchangeEntry, "")}`.trim()
        );
      }

      const symbolEntry = symbolSupport.get(node.config.instrument);
      if (!capabilityEntryStatus(symbolEntry, supportedSymbols, node.config.instrument)) {
        errors.push(
          `数据节点 ${node.name} 使用了未支持的交易对 ${node.config.instrument || "-"}。 ${capabilityReason(symbolEntry, "")}`.trim()
        );
      }

      output.data_sources.push(base);
    }

    if (node.type === "intent") {
      const inputRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => ({
          source_id: idMap[edge.source_node_id],
          source_port: edge.source_port,
          target_port: edge.target_port
        }));
      if (inputRefs.length === 0) {
        errors.push(`意图节点 ${node.name} 缺少数据输入。`);
      }
      output.intent_generators.push({ ...base, input_refs: inputRefs });
    }

    if (node.type === "agent") {
      const intentRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => idMap[edge.source_node_id]);
      if (intentRefs.length === 0) {
        errors.push(`代理节点 ${node.name} 缺少意图输入。`);
      }
      const isPortfolioRebalance =
        node.module_key === "builtin.agent.weighted" && agentUsesPortfolioRebalance(node.config);
      if (isPortfolioRebalance) {
        const rebalanceSymbols = parseCsvStrings(node.config.rebalance_symbols);
        const unsupportedSymbols = rebalanceSymbols.filter(
          (symbol) => !supportedSymbols.has(symbol)
        );
        if (unsupportedSymbols.length > 0) {
          errors.push(
            `代理节点 ${node.name} 使用了未支持的再平衡交易对：${unsupportedSymbols.join(", ")}。`
          );
        }

        if (normalizeRebalanceSchedule(node.config.rebalance_schedule) === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 rebalance cadence 值不合法。`);
        }
        const allocationKind = normalizeRebalanceAllocationKind(
          node.config.rebalance_allocation_kind
        );
        if (allocationKind === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 allocation rule 值不合法。`);
        }
        if (normalizeRebalanceRankMethod(node.config.rebalance_rank_method) === "__invalid__") {
          errors.push(`代理节点 ${node.name} 的 rank method 值不合法。`);
        }
        if (
          normalizeRebalanceScoreNormalize(node.config.rebalance_score_normalize) === "__invalid__"
        ) {
          errors.push(`代理节点 ${node.name} 的 score normalize 值不合法。`);
        }

        const weightsRaw = parseCsvStrings(node.config.rebalance_target_weights);
        const weights = parseCsvNumbers(node.config.rebalance_target_weights);
        if (weightsRaw.length !== weights.length) {
          errors.push(
            `代理节点 ${node.name} 的 target weights 必须是由逗号分隔的数字。`
          );
        }
        if (
          allocationKind === "fixed_weights" &&
          rebalanceSymbols.length > 0 &&
          weights.length !== rebalanceSymbols.length
        ) {
          errors.push(
            `代理节点 ${node.name} 的 fixed_weights 数量必须与 rebalance symbols 一致。`
          );
        }
      }
      output.agents.push({ ...base, intent_refs: intentRefs });
    }

    if (node.type === "risk") {
      const agentRefs = graph.edges
        .filter((edge) => edge.target_node_id === node.id)
        .map((edge) => idMap[edge.source_node_id]);
      if (agentRefs.length === 0) {
        errors.push(`风控节点 ${node.name} 缺少代理输入。`);
      }
      output.risk_controls.push({ ...base, agent_refs: agentRefs });
    }

    if (node.type === "execution") {
      const executionEntry = executionModuleSupport.get(node.module_key);
      if (!capabilityEntryStatus(executionEntry, supportedExecutionModules, node.module_key)) {
        errors.push(
          `执行节点 ${node.name} 使用了当前后端未支持的模块 ${node.module_key}。 ${capabilityReason(executionEntry, "")}`.trim()
        );
      }
      const riskEdge = graph.edges.find((edge) => edge.target_node_id === node.id);
      if (!riskEdge) {
        errors.push(`执行节点 ${node.name} 缺少风控输入。`);
      }
      output.executions.push({
        ...base,
        risk_ref: riskEdge ? idMap[riskEdge.source_node_id] : null
      });
    }

    if (node.type === "runtime") {
      output.runtime_control = base;
      output.metadata.mode = node.config.mode || "paper";
      const runtimeModeEntry = runtimeModeSupport.get(output.metadata.mode);
      if (!capabilityEntryStatus(runtimeModeEntry, supportedRuntimeModes, output.metadata.mode)) {
        errors.push(
          `当前仅支持这些运行模式：${[...supportedRuntimeModes].join(", ")}。 ${capabilityReason(runtimeModeEntry, "")}`.trim()
        );
      }
    }
  });

  if (output.executions.length !== 1) {
    errors.push("当前 beta 仅支持一个执行节点。");
  }

  const topology = buildTopology(graph);
  if (topology.hasCycle) {
    errors.push("策略图存在循环依赖，无法编译。");
  }

  if ((graph.validation_state?.issue_counts?.error || 0) > 0) {
    errors.push("策略图校验未通过，无法编译。");
  }

  const graphWithArtifacts = attachQuantScriptArtifacts(graph);
  const coreIr = buildCoreIr(graph, output);
  graphWithArtifacts.metadata.artifacts = {
    ...(graphWithArtifacts.metadata.artifacts || {}),
    core_ir: coreIr
  };
  const quantscript =
    graphWithArtifacts.metadata.artifacts.quantscript.graph_source ||
    generateGraphQuantScript(graph);
  const compilable = errors.length === 0;
  const diagnostics = buildLocalCompileDiagnostics(errors, warnings);

  return {
    compile_id: compileId,
    runtime_config: output,
    core_ir: coreIr,
    quantscript,
    graph: graphWithArtifacts,
    compile_summary: {
      compilable,
      last_compile_id: compileId,
      last_compile_at: Date.now(),
      topology_order: topology.topologyOrder,
      outputs: {
        data_sources: output.data_sources.length,
        intent_generators: output.intent_generators.length,
        agents: output.agents.length,
        risk_controls: output.risk_controls.length,
        executions: output.executions.length
      },
      diagnostics,
      warnings,
      errors
    }
  };
}
