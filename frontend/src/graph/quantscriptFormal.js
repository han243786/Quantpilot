function toIdentifier(value, fallback = "node") {
  const normalized = String(value || fallback)
    .trim()
    .replace(/[^a-zA-Z0-9_]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_+/g, "_")
    .toLowerCase();
  if (!normalized) return fallback;
  return /^[a-z_]/.test(normalized) ? normalized : `n_${normalized}`;
}

function formalNodeIdentifier(node) {
  return toIdentifier(node?.id, node?.name || "node");
}

export function formalDataRuntimeId(node) {
  return `data_${formalNodeIdentifier(node)}`;
}

export function formalDataBindingName(node) {
  return `${formalDataRuntimeId(node)}_series`;
}

export function formalIntentRuntimeId(node) {
  return `intent_${formalNodeIdentifier(node)}`;
}

export function formalIntentBindingBase(node) {
  return formalIntentRuntimeId(node);
}

export function formalIntentSignalBindingName(node) {
  return `${formalIntentBindingBase(node)}_signal`;
}

export function formalDataNodes(graph) {
  return (graph.nodes || []).filter((node) => node.type === "data");
}

function formalInputDataNode(graph, node) {
  const edge = (graph.edges || []).find((item) => item.target_node_id === node.id);
  if (!edge) return null;
  return (graph.nodes || []).find((candidate) => candidate.id === edge.source_node_id) || null;
}

function formalInstrument(node) {
  return node?.config?.instrument || "BTCUSDT";
}

function formalExchange(node) {
  return node?.config?.exchange || "binance";
}

function formalDataStatements(node) {
  if (node.module_key !== "builtin.data.kline") return [];
  const binding = formalDataBindingName(node);
  return [
    `    let ${binding} = fetch(${JSON.stringify(formalInstrument(node))}, exchange=${JSON.stringify(formalExchange(node))}, interval=${JSON.stringify(node.config?.timeframe || "1d")}, lookback=${Number(node.config?.window_size || 200)})?`
  ];
}

function formalAgentStatement(node) {
  const strategy = node.config?.strategy || "weighted";
  const threshold = node.config?.decision_threshold ?? 0.05;
  return [`    agent(${JSON.stringify(strategy)}, decision_threshold=${threshold})`];
}

function formalRiskStatement(node) {
  const profile = node.config?.profile_name || "global";
  const maxPos = node.config?.max_position ?? 0.2;
  const maxLev = node.config?.max_total_leverage ?? 3.0;
  return [`    risk.profile(${JSON.stringify(profile)}, max_position=${maxPos}, max_total_leverage=${maxLev})`];
}

function formalExecutionStatement(node) {
  const profile = node.config?.profile_name || "paper";
  const fee = node.config?.fee_bps ?? 10;
  const slip = node.config?.slippage_bps ?? 5;
  return [`    execution.profile(${JSON.stringify(profile)}, fee_bps=${fee}, slippage_bps=${slip})`];
}

function formalRuntimeStatement(node) {
  const mode = node.config?.mode || "paper";
  return [`    runtime.mode(${JSON.stringify(mode)})`];
}

export function canGenerateFormalQuantScript(graph) {
  const dataNodes = formalDataNodes(graph);
  const supportedIntentKeys = new Set([
    "builtin.intent.double_ma",
    "builtin.intent.ma_deviation",
    "builtin.intent.rsi",
    "builtin.intent.macd",
    "builtin.intent.momentum",
    "builtin.intent.zscore",
    "builtin.intent.spread_observer"
  ]);

  if (dataNodes.some((node) => node.module_key !== "builtin.data.kline")) return false;
  return (graph.nodes || [])
    .filter((node) => node.type === "intent")
    .every((node) => supportedIntentKeys.has(node.module_key) && formalInputDataNode(graph, node));
}

function formalIntentStatements(graph, node) {
  const dataNode = formalInputDataNode(graph, node);
  if (!dataNode || dataNode.module_key !== "builtin.data.kline") return [];

  const series = formalDataBindingName(dataNode);
  const instrument = JSON.stringify(formalInstrument(dataNode));
  const signal = formalIntentSignalBindingName(node);

  switch (node.module_key) {
    case "builtin.intent.double_ma":
      return [
        `    let ${signal} = sma(${series}, ${Number(node.config?.fast_period || 50)}) / sma(${series}, ${Number(node.config?.slow_period || 150)})`,
        `    if ${signal} > ${Number(node.config?.entry_ratio || 1)} {`,
        `        emit Intent("BUY", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.ma_deviation":
      return [
        `    let ${signal} = sma(${series}, ${Number(node.config?.lookback || 15)}) / sma(${series}, ${Number(node.config?.baseline_period || 150)})`,
        `    if ${signal} > ${Number(node.config?.threshold_ratio || 1)} {`,
        `        emit Intent("SELL", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.rsi":
      return [
        `    let ${signal} = rsi(${series}, ${Number(node.config?.period || 14)})`,
        `    if ${signal} < ${Number(node.config?.oversold_threshold || 30)} {`,
        `        emit Intent("BUY", instrument=${instrument}, quantity=1.0)`,
        `    } else if ${signal} > ${Number(node.config?.overbought_threshold || 70)} {`,
        `        emit Intent("SELL", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.macd":
      return [
        `    let ${signal} = macd(${series}, ${Number(node.config?.fast_period || 12)}, ${Number(node.config?.slow_period || 26)}, ${Number(node.config?.signal_period || 9)})`,
        `    if ${signal} > ${Number(node.config?.histogram_threshold || 0)} {`,
        `        emit Intent("BUY", instrument=${instrument}, quantity=1.0)`,
        `    } else if ${signal} < ${-Math.abs(Number(node.config?.histogram_threshold || 0))} {`,
        `        emit Intent("SELL", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.momentum":
      return [
        `    let ${signal} = momentum(${series}, ${Number(node.config?.lookback || 10)})`,
        `    if ${signal} > ${Number(node.config?.threshold_ratio || 0.02)} {`,
        `        emit Intent("BUY", instrument=${instrument}, quantity=1.0)`,
        `    } else if ${signal} < ${-Math.abs(Number(node.config?.threshold_ratio || 0.02))} {`,
        `        emit Intent("SELL", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.zscore":
      return [
        `    let ${signal} = zscore(${series}, ${Number(node.config?.window || 20)})`,
        `    if ${signal} > ${Number(node.config?.entry_z || 2)} {`,
        `        emit Intent("SELL", instrument=${instrument}, quantity=1.0)`,
        `    } else if ${signal} < ${-Math.abs(Number(node.config?.entry_z || 2))} {`,
        `        emit Intent("BUY", instrument=${instrument}, quantity=1.0)`,
        "    }"
      ];
    case "builtin.intent.spread_observer":
      return [
        `    let ${signal} = spread_observe(${series}, ` +
          `field_code=${node.config?.field_code ?? 0}, ` +
          `align_direction_code=${node.config?.align_direction_code ?? 0}, ` +
          `spread_output_code=${node.config?.spread_output_code ?? 0}, ` +
          `max_time_diff_ms=${node.config?.max_time_diff_ms ?? 5000})`,
        `    let _ = ${signal}`,
      ];
    default:
      return [];
  }
}

export function generateFormalQuantScript(graph) {
  if (!canGenerateFormalQuantScript(graph)) return "";

  const lines = ["fn strategy() {"];

  formalDataNodes(graph).forEach((node) => {
    lines.push(...formalDataStatements(node));
  });

  // Agent/Risk/Execution/Runtime configuration statements
  (graph.nodes || [])
    .filter((node) => ["agent", "risk", "execution", "runtime", "runtime_control"].includes(node.type))
    .forEach((node) => {
      let stmts = [];
      switch (node.type) {
        case "agent":
          stmts = formalAgentStatement(node);
          break;
        case "risk":
          stmts = formalRiskStatement(node);
          break;
        case "execution":
          stmts = formalExecutionStatement(node);
          break;
        case "runtime":
        case "runtime_control":
          stmts = formalRuntimeStatement(node);
          break;
      }
      if (stmts.length > 0) lines.push(...stmts);
    });

  graph.nodes
    .filter((node) => node.type === "intent")
    .forEach((node) => {
      const statements = formalIntentStatements(graph, node);
      if (statements.length > 0) {
        if (lines.length > 1) lines.push("");
        lines.push(...statements);
      }
    });

  lines.push("}");
  return lines.join("\n");
}

