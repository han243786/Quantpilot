import { describe, expect, it } from "vitest";
import { buildActionFailureMessage } from "./actionFailure";
import { COMPILE_CONTRACT } from "./compileContract";

describe("buildActionFailureMessage", () => {
  it("formats compile failures as reason plus next action", () => {
    expect(
      buildActionFailureMessage(
        "compile",
        "Runtime compile rejected the generated output.",
        "策略图编译失败。"
      )
    ).toContain(COMPILE_CONTRACT.runtimeSourceOfTruthLabel);
  });

  it("formats simulation failures as reason plus next action", () => {
    expect(
      buildActionFailureMessage(
        "simulation",
        "Capability rejected: runtime mode live is not enabled for this beta backend.",
        "启动模拟运行失败。"
      )
    ).toContain(
      "原因：Capability rejected: runtime mode live is not enabled for this beta backend 后续：检查编译结果、运行模式、执行模块和当前 capability 配置后，再重新启动模拟运行。"
    );
  });

  it("formats backtest failures as reason plus next action", () => {
    expect(
      buildActionFailureMessage(
        "backtest",
        "Capability rejected: symbol XRPUSDT is outside the current beta market-data profile.",
        "回测执行失败。"
      )
    ).toContain(
      "原因：Capability rejected: symbol XRPUSDT is outside the current beta market-data profile 后续：检查编译结果、回放来源、市场数据边界和当前 capability 配置后，再重新运行回测。"
    );
  });

  it("formats export and passive runtime failures with the same template", () => {
    expect(
      buildActionFailureMessage(
        "export_config",
        "Runtime compile rejected the generated output.",
        "运行配置导出失败。"
      )
    ).toContain(
      "原因：Runtime compile rejected the generated output 后续：检查 compile diagnostics，并确认运行时编译成功后再重新导出 runtime_config。"
    );

    expect(
      buildActionFailureMessage(
        "sse_disconnect",
        "Runtime event stream connection closed.",
        "运行时事件流连接已断开。"
      )
    ).toContain(
      "原因：Runtime event stream connection closed 后续：检查后端可用性和当前运行状态；如果事件流未恢复，请重新连接或启动新的模拟运行。"
    );
  });

  it("formats save and detail load failures with action-specific next steps", () => {
    expect(
      buildActionFailureMessage("save_graph", "backend unavailable", "保存策略图失败。")
    ).toContain("后续：检查当前策略图校验结果和后端可用性后，再重新保存策略图。");

    expect(
      buildActionFailureMessage("run_detail", "Run record missing.", "加载运行详情失败。")
    ).toContain("后续：检查后端可用性，以及所选运行记录是否仍然存在后，再重新加载运行详情。");
  });

  it("formats startup recovery failures with a recovery-focused next step", () => {
    expect(
      buildActionFailureMessage(
        "startup_recovery",
        "Latest saved graph is not runnable yet.",
        "启动恢复策略图失败。"
      )
    ).toContain(
      "原因：Latest saved graph is not runnable yet 后续：检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载编辑器。"
    );
  });
});
