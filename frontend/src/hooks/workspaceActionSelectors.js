import { useMemo } from "react";
import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import { getRuntimeStatusMeta } from "../utils/runtimeStatus";
import { resolveWorkspaceActionState } from "./workspaceActionBarShared";

export function useWorkspaceActionSelectors() {
  const { t } = useI18n();
  const graph = useGraphStore((state) => state.graph);
  const runtime = useGraphStore((state) => state.runtime);
  const capabilityStatus = useGraphStore((state) => state.capabilityStatus);
  const capabilitySource = useGraphStore((state) => state.capabilitySource);
  const capabilityMessage = useGraphStore((state) => state.capabilityMessage);
  const formalQuantScriptOverride = useGraphStore((state) => state.formalQuantScriptOverride);

  return useMemo(
    () => ({
      ...resolveWorkspaceActionState({
        graph,
        runtime,
        capabilityStatus,
        capabilitySource,
        capabilityMessage,
        formalQuantScriptOverride,
        t
      }),
      runtimeMeta: getRuntimeStatusMeta(runtime.status)
    }),
    [capabilityMessage, capabilitySource, capabilityStatus, formalQuantScriptOverride, graph, runtime, t]
  );
}
