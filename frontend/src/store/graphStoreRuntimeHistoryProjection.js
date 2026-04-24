import {
  applyEventsToGraphNodes,
  collectHighlightedNodeIds,
  resolveBacktestEvents,
  withRuntimeBinding
} from "./graphStoreHelpers";

export function projectRunDetailGraph(graph, detail) {
  const highlightedNodeIds = collectHighlightedNodeIds(detail.events);
  const nextGraph = withRuntimeBinding(
    {
      ...graph,
      nodes: applyEventsToGraphNodes(graph.nodes, detail.events)
    },
    { current_run_id: detail.run_id }
  );

  return {
    nextGraph,
    highlightedNodeIds
  };
}

export function projectBacktestDetailGraph(graph, detail) {
  const events = resolveBacktestEvents(detail);
  const highlightedNodeIds = collectHighlightedNodeIds(events);
  const nextGraph = withRuntimeBinding(
    {
      ...graph,
      nodes: applyEventsToGraphNodes(graph.nodes, events)
    },
    {
      current_run_id: null,
      last_compile_id: detail.compile_id
    }
  );

  return {
    nextGraph,
    events,
    highlightedNodeIds
  };
}
