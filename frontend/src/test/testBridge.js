import { useGraphStore } from "../store/graphStore";
import { navigateTo, parseRoute } from "../router";

/**
 * Test bridge exposed on window.__QUANTPILOT_TEST__
 * Only mounted in DEV mode to prevent exposure in production.
 */
export function installTestBridge() {
  if (!import.meta.env.DEV) return;

  const api = {
    // ── Navigation ──
    navigateTo(path) {
      navigateTo(path);
    },

    getCurrentRoute() {
      return parseRoute(window.location.pathname, window.location.search);
    },

    // ── Canvas State ──
    getNodeCount() {
      return useGraphStore.getState().graph.nodes?.length ?? 0;
    },

    getEdgeCount() {
      return useGraphStore.getState().graph.edges?.length ?? 0;
    },

    getNodeById(id) {
      const nodes = useGraphStore.getState().graph.nodes ?? [];
      return nodes.find((n) => n.id === id) ?? null;
    },

    getSelectedNodeId() {
      return useGraphStore.getState().selectedNodeId;
    },

    getSelectedEdgeId() {
      return useGraphStore.getState().selectedEdgeId;
    },

    // ── Graph Meta ──
    getGraphId() {
      return useGraphStore.getState().graph.metadata?.graph_id ?? null;
    },

    getGraphName() {
      return useGraphStore.getState().graph.metadata?.name ?? null;
    },

    // ── Compile Status ──
    getCompileStatus() {
      const summary = useGraphStore.getState().graph.compile_summary;
      return {
        compilable: summary?.compilable ?? null,
        protocol_name: summary?.protocol_name ?? null,
        config_hash: summary?.config_hash ?? null,
        diagnostics_count: summary?.diagnostics?.length ?? 0,
      };
    },

    // ── Runtime Status ──
    getRuntimeStatus() {
      return useGraphStore.getState().runtime.status;
    },

    getRuntimeLabel() {
      const runtime = useGraphStore.getState().runtime;
      const status = runtime?.status ?? "idle";
      return status;
    },

    // ── Capability Status ──
    getCapabilityAlert() {
      return useGraphStore.getState().capabilityAlert ?? null;
    },

    // ── Workspace Tab ──
    getActiveTab() {
      // The active tab is stored in a separate UI store
      // We look for the tab button with the --active class
      const activeTab = document.querySelector(".ad-tab--active, .workspace-tab--active");
      if (activeTab) {
        const testId = activeTab.getAttribute("data-testid");
        return testId?.replace("workspace-tab-", "") ?? null;
      }
      return null;
    },

    // ── Hub / Roster ──
    getRosterItems() {
      const rows = document.querySelectorAll("[data-testid='strategy-hub-roster-table-body'] tr, [data-testid='strategy-hub-roster-table-body'] > div");
      return Array.from(rows).map((row) => {
        const cells = row.querySelectorAll("td, [data-testid*='roster-action']");
        return {
          text: row.textContent?.trim().slice(0, 100),
          actions: Array.from(row.querySelectorAll("button")).map((b) => b.textContent?.trim()),
        };
      });
    },

    // ── Layout Snapshot ──
    getLayoutSnapshot() {
      const areas = [".top-toolbar", ".strategy-workspace-tabbar", ".react-flow", ".property-panel",
        ".module-sidebar", ".event-stream-panel", ".strategy-hub-hero",
        ".strategy-hub-roster-table"];
      const snapshot = {};
      for (const selector of areas) {
        const el = document.querySelector(selector);
        if (el) {
          const rect = el.getBoundingClientRect();
          snapshot[selector] = {
            x: Math.round(rect.x),
            y: Math.round(rect.y),
            width: Math.round(rect.width),
            height: Math.round(rect.height),
            visible: rect.width > 0 && rect.height > 0,
          };
        }
      }
      return snapshot;
    },

    // ── Highlight Element (for screenshot debugging) ──
    highlightElement(selector) {
      const el = document.querySelector(selector);
      if (!el) return false;
      const orig = el.style.outline;
      el.style.outline = "3px solid magenta";
      setTimeout(() => {
        el.style.outline = orig;
      }, 3000);
      return true;
    },

    // ── Raw Store Access ──
    getRawState() {
      return JSON.parse(JSON.stringify(useGraphStore.getState()));
    },
  };

  window.__QUANTPILOT_TEST__ = api;
}
