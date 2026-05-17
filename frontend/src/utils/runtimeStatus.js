import { translateText } from "../i18n";

const RUNTIME_TONES = {
  idle: "muted",
  connecting: "warning",
  running: "success",
  waiting: "warning",
  completed: "success",
  error: "danger",
  stopped: "muted"
};

export function getRuntimeStatusMeta(status) {
  const label = translateText(status === "idle" ? "空闲" :
    status === "connecting" ? "连接中" :
    status === "running" ? "运行中" :
    status === "waiting" ? "等待中" :
    status === "completed" ? "已完成" :
    status === "error" ? "错误" :
    status === "stopped" ? "已停止" : (status || "-"));
  const tone = RUNTIME_TONES[status] || "muted";
  return { label, tone };
}

export function runtimeStatusLabel(status) {
  return getRuntimeStatusMeta(status).label;
}
