import { humanizeErrorText } from "./errorText";
import { COMPILE_CONTRACT } from "./compileContract";
import { translateText } from "../i18n";

const ACTION_FAILURE_FALLBACKS = {
  compile: "策略图编译失败。",
  export_config: "运行配置导出失败。",
  save_graph: "保存策略图失败。",
  load_latest: "加载最新保存的策略图失败。",
  startup_recovery: "启动恢复策略图失败。",
  simulation: "启动模拟运行失败。",
  backtest: "回测执行失败。",
  run_detail: "加载运行详情失败。",
  backtest_detail: "加载回测详情失败。",
  run_history: "加载运行历史失败。",
  backtest_history: "加载回测历史失败。",
  sse_disconnect: "运行时事件流连接已断开。"
};

const ACTION_FAILURE_NEXT_STEPS = {
  compile:
    `检查 structured compile diagnostics，并确认 Strategy IR 仅作语义预检、最终可运行结果仍${COMPILE_CONTRACT.runtimeSourceOfTruthLabel}。`,
  export_config:
    "检查 compile diagnostics，并确认运行时编译成功后再重新导出 runtime_config。",
  save_graph:
    "检查当前策略图校验结果和后端可用性后，再重新保存策略图。",
  load_latest:
    "检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载最新图。",
  startup_recovery:
    "检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载编辑器。",
  simulation:
    "检查编译结果、运行模式、执行模块和当前 capability 配置后，再重新启动模拟运行。",
  backtest:
    "检查编译结果、回放来源、市场数据边界和当前 capability 配置后，再重新运行回测。",
  run_detail:
    "检查后端可用性，以及所选运行记录是否仍然存在后，再重新加载运行详情。",
  backtest_detail:
    "检查后端可用性，以及所选回测记录是否仍然存在后，再重新加载回测详情。",
  run_history:
    "检查后端可用性，并在运行时 API 可访问后重新刷新运行历史。",
  backtest_history:
    "检查后端可用性，并在运行时 API 可访问后重新刷新回测历史。",
  sse_disconnect:
    "检查后端可用性和当前运行状态；如果事件流未恢复，请重新连接或启动新的模拟运行。"
};

export function buildActionFailureMessage(action, reasonLike, fallbackReason = "") {
  const fallback = fallbackReason || ACTION_FAILURE_FALLBACKS[action] || "操作失败。";
  const reason = humanizeErrorText(reasonLike, fallback);
  const nextStep = ACTION_FAILURE_NEXT_STEPS[action] || "检查最新错误详情后重试。";
  return translateText("原因：{reason} 后续：{nextStep}", { reason, nextStep });
}

export function getActionFailureNextStep(action) {
  return ACTION_FAILURE_NEXT_STEPS[action] || "检查最新错误详情后重试。";
}
