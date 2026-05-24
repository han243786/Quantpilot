import {
  CAPABILITY_ACTION_MAP,
  WORKSPACE_SURFACE_MAP,
  getCapabilityActionBlockReason
} from "./supportMatrix";

const UNSUPPORTED_STATUS = "unsupported";
const DECLARED_ONLY_STATUS = "declared_only";
const SUPPORTED_STATUS = "supported";

function cleanText(value, fallback = "") {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : fallback;
}

function normalizeStatus(status) {
  if (status === SUPPORTED_STATUS) return SUPPORTED_STATUS;
  if (status === DECLARED_ONLY_STATUS) return DECLARED_ONLY_STATUS;
  return UNSUPPORTED_STATUS;
}

function entryMap(entries = []) {
  return new Map(
    (Array.isArray(entries) ? entries : [])
      .filter((entry) => entry && typeof entry === "object" && entry.key)
      .map((entry) => [entry.key, entry])
  );
}

function projectEntry({ key, definition, backendEntry, missingReason }) {
  const status = normalizeStatus(backendEntry?.status);
  const reason =
    status === SUPPORTED_STATUS
      ? ""
      : cleanText(
          backendEntry?.reason,
          status === DECLARED_ONLY_STATUS ? "后端已声明，但当前版本未开放。" : missingReason
        );

  return {
    key,
    label: definition?.label || key,
    apiPaths: definition?.apiPaths || [],
    notes: definition?.notes || [],
    status,
    reason,
    source: cleanText(backendEntry?.source, definition?.sourceOfTruth || ""),
    visible: status !== UNSUPPORTED_STATUS,
    enabled: status === SUPPORTED_STATUS
  };
}

export function projectWorkspaceSurfaces(capabilities) {
  const backendEntries = entryMap(capabilities?.workspace?.surfaces);

  return Object.fromEntries(
    Object.entries(WORKSPACE_SURFACE_MAP).map(([key, definition]) => [
      key,
      projectEntry({
        key,
        definition,
        backendEntry: backendEntries.get(key),
        missingReason: "后端能力快照未声明该工作区入口。"
      })
    ])
  );
}

export function projectUiActions({
  capabilities,
  capabilityStatus = "ready",
  capabilitySource = "remote",
  capabilityMessage = ""
} = {}) {
  const backendEntries = entryMap(capabilities?.ui_actions?.actions);

  return Object.fromEntries(
    Object.entries(CAPABILITY_ACTION_MAP).map(([key, definition]) => {
      const projected = projectEntry({
        key,
        definition,
        backendEntry: backendEntries.get(key),
        missingReason: "后端能力快照未声明该操作。"
      });
      const blockReason = getCapabilityActionBlockReason({
        actionKey: key,
        capabilityStatus,
        capabilitySource,
        capabilityMessage,
        capabilities
      });
      const hardBlockedByCapabilitySync =
        definition.blockedDuringCapabilitySync &&
        (capabilityStatus === "loading" ||
          capabilitySource === "safe_fallback" ||
          (capabilityStatus === "error" && capabilitySource !== "cache"));

      return [
        key,
        {
          ...projected,
          blockReason,
          enabled: projected.enabled && !hardBlockedByCapabilitySync
        }
      ];
    })
  );
}

export function projectCapabilityView({
  capabilities,
  capabilityStatus = "ready",
  capabilitySource = "remote",
  capabilityMessage = ""
} = {}) {
  return {
    workspace: {
      surfaces: projectWorkspaceSurfaces(capabilities)
    },
    uiActions: {
      actions: projectUiActions({
        capabilities,
        capabilityStatus,
        capabilitySource,
        capabilityMessage
      })
    }
  };
}

export function isProjectedEntryEnabled(entry) {
  return Boolean(entry?.enabled);
}
