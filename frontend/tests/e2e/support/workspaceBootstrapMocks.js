import { buildWorkspaceGraphFixture } from "./workspaceGraphFixture";

function buildGraphIndexFixture(graphFixture) {
  return [
    {
      graph_id: graphFixture.metadata?.graph_id || "draft_graph",
      name: graphFixture.metadata?.name || "Draft graph",
      updated_at: graphFixture.metadata?.updated_at || Date.now(),
      path: `storage/graphs/${graphFixture.metadata?.graph_id || "draft_graph"}.json`
    }
  ];
}

export async function installWorkspaceBootstrapMocks(
  api,
  {
    graphFixture = buildWorkspaceGraphFixture(),
    latestGraphResponse = graphFixture,
    runHistory = [],
    backtestHistory = [],
    experiments = []
  } = {}
) {
  const graphId = graphFixture.metadata?.graph_id || "draft_graph";

  if (
    latestGraphResponse &&
    typeof latestGraphResponse === "object" &&
    ("status" in latestGraphResponse || "body" in latestGraphResponse)
  ) {
    await api.fulfill("**/api/graphs/latest", latestGraphResponse);
  } else {
    await api.json("**/api/graphs/latest", latestGraphResponse);
  }
  await api.json("**/api/graphs", buildGraphIndexFixture(graphFixture));
  await api.json(`**/api/graphs/${graphId}`, graphFixture);
  await api.json("**/api/runtime/runs", runHistory);
  await api.json("**/api/runtime/backtests", backtestHistory);
  await api.json("**/api/runtime/mutations**", []);
  await api.json("**/api/runtime/reports**", []);
  await api.json("**/api/runtime/experiments", experiments);
  await api.json("**/api/runtime/experiments/*", {
    experiment_id: "",
    graph_id: graphId,
    variants: []
  });
  await api.json("**/api/graphs/*/versions", []);
  await api.json("**/api/graphs/*/audit", []);

  return graphFixture;
}
