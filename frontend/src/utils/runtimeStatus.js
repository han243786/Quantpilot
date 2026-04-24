const RUNTIME_STATUS_META = {
  idle: { label: "空闲", tone: "muted" },
  connecting: { label: "连接中", tone: "warning" },
  running: { label: "运行中", tone: "success" },
  waiting: { label: "等待中", tone: "warning" },
  completed: { label: "已完成", tone: "success" },
  error: { label: "错误", tone: "danger" },
  stopped: { label: "已停止", tone: "muted" }
};

export function getRuntimeStatusMeta(status) {
  return RUNTIME_STATUS_META[status] || { label: status || "-", tone: "muted" };
}

export function runtimeStatusLabel(status) {
  return getRuntimeStatusMeta(status).label;
}
