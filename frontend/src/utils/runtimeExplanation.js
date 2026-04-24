const DETAIL_ROWS_BY_KIND = {
  risk: (detail) => detail.risk_detail_rows || [],
  order: (detail) => detail.order_detail_rows || [],
  dataQuality: (detail) => detail.data_quality_rows || []
};

export function getEventExplanationSummary(event) {
  const summary = event?.payload?.explanation_summary || event?.payload?.reason_text || null;
  if (!summary || summary === event?.summary) return null;
  return summary;
}

export function buildDiagnosticsExplanationEntries(graph, diagnostics, kind = "risk") {
  const selectRows = DETAIL_ROWS_BY_KIND[kind];
  if (!selectRows || !diagnostics?.node_details) return [];

  const nodeMap = new Map((graph?.nodes || []).map((node) => [node.id, node]));
  return Object.values(diagnostics.node_details)
    .map((detail) => {
      const rows = selectRows(detail);
      if (rows.length === 0) return null;
      const node = nodeMap.get(detail.node_id);
      return {
        nodeId: detail.node_id,
        nodeName: node?.name || detail.node_id,
        explanationSummary: detail.explanation_summary || null,
        rows
      };
    })
    .filter(Boolean);
}
