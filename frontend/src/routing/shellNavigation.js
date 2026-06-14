import {
  alertsPath,
  approvalsPath,
  chaosPath,
  quantscriptPath,
  runbookPath,
  settingsPath,
  snapshotsPath,
  strategiesPath,
} from "./routeContract";

export const SHELL_NAV_SECTIONS = [
  [
    {
      id: "strategies",
      path: strategiesPath(),
      labelKey: "策略",
      iconKey: "chart",
    },
    {
      id: "quantscript",
      path: quantscriptPath(),
      labelKey: "QuantScript",
      iconKey: "code",
    },
  ],
  [
    {
      id: "approvals",
      path: approvalsPath(),
      labelKey: "审批",
      iconKey: "check",
    },
    {
      id: "alerts",
      path: alertsPath(),
      labelKey: "告警",
      iconKey: "alert",
    },
    {
      id: "snapshots",
      path: snapshotsPath(),
      labelKey: "快照",
      iconKey: "camera",
    },
    {
      id: "runbook",
      path: runbookPath(),
      labelKey: "故障手册",
      iconKey: "book",
    },
    {
      id: "chaos",
      path: chaosPath(),
      labelKey: "混沌",
      iconKey: "flask",
    },
    {
      id: "settings",
      path: settingsPath(),
      labelKey: "设置",
      iconKey: "settings",
    },
  ],
];

export const COMMAND_NAVIGATION_DEFS = [
  {
    id: "strategies",
    labelKey: "策略中心",
    keys: [strategiesPath()],
    sectionKey: "导航",
  },
  {
    id: "quantscript",
    labelKey: "QuantScript 编辑器",
    keys: [quantscriptPath()],
    sectionKey: "导航",
  },
  {
    id: "approvals",
    labelKey: "审批队列",
    keys: [approvalsPath()],
    sectionKey: "运维",
  },
  {
    id: "alerts",
    labelKey: "告警面板",
    keys: [alertsPath()],
    sectionKey: "运维",
  },
  {
    id: "snapshots",
    labelKey: "签名快照",
    keys: [snapshotsPath()],
    sectionKey: "运维",
  },
  {
    id: "runbook",
    labelKey: "故障手册",
    keys: [runbookPath()],
    sectionKey: "运维",
  },
  {
    id: "chaos",
    labelKey: "混沌实验",
    keys: [chaosPath()],
    sectionKey: "运维",
  },
];

export function isShellNavPathActive(currentPath, itemPath) {
  return (
    currentPath === itemPath ||
    currentPath.startsWith(`${itemPath}/`) ||
    currentPath.startsWith(`${itemPath}?`)
  );
}
