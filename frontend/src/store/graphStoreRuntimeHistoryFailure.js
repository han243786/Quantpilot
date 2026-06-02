import { buildActionFailureMessage } from "../utils/actionFailure";

export function buildRuntimeHistoryFailureMessage(kind, error) {
  const fallbackMessages = {
    run_history: "加载运行历史失败。",
    backtest_history: "加载回测历史失败。",
    experiment_history: "加载实验历史失败。",
    run_detail: "加载运行详情失败。",
    backtest_detail: "加载回测详情失败。",
    experiment_detail: "加载实验详情失败。",
    run_save: "保存运行结果失败。",
    backtest_save: "保存回测结果失败。",
    experiment_save: "保存实验结果失败。",
    run_discard: "丢弃运行结果失败。",
    backtest_discard: "丢弃回测结果失败。",
    experiment_discard: "丢弃实验结果失败。"
  };
  return buildActionFailureMessage(kind, error, fallbackMessages[kind]);
}
