export const editorBootstrapFixtures = {
  latestGraphMissing: {
    status: 404,
    contentType: "text/plain; charset=utf-8",
    body: "not found"
  },
  emptyRunHistory: {
    status: 200,
    contentType: "application/json; charset=utf-8",
    body: "[]"
  },
  emptyBacktestHistory: {
    status: 200,
    contentType: "application/json; charset=utf-8",
    body: "[]"
  }
};
