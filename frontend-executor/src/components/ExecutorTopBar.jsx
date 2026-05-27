/// v3.4.0: 执行端顶部工具栏 — 品牌、状态、紧急停止
/// v4.8.0: 双模拟盘模式边界

import { memo } from "react";
import { useI18n } from "../i18n";

const ExecutorTopBar = memo(function ExecutorTopBar({
  status,
  statusMsg,
  strategyCount,
  onEmergencyStop,
  mode,
  modeError,
  runtimeKind,
  onModeSwitch,
}) {
  const { t } = useI18n();
  const statusLabel = status === "running" ? t("运行中") : status === "error" ? t("错误") : t("空闲");
  const statusClass = status === "running" ? "running" : status === "error" ? "error" : "stopped";
  const normalizedMode = mode || "paper_simulated";

  return (
    <div className="exec-topbar">
      <div className="exec-topbar-brand">{t("QuantPilot 实时执行端")}</div>
      <div className="exec-topbar-actions">
        <div className="exec-status">
          <span className={`exec-status-dot ${statusClass}`} />
          <span>{statusLabel}</span>
          {strategyCount > 0 && <span>· {strategyCount} {t("策略")}</span>}
          {statusMsg && <span className="exec-status-msg" title={statusMsg}>{statusMsg}</span>}
          {modeError && <span className="exec-status-msg exec-status-err" title={modeError}>{modeError}</span>}
        </div>

        <span className={`exec-runtime-badge ${(runtimeKind || "v3").toLowerCase()}`}>
          {(runtimeKind || "v3").toUpperCase()}
        </span>

        <div className="exec-mode-segment" role="group" aria-label={t("执行模式")}>
          <button
            className={`exec-btn exec-mode-btn ${normalizedMode === "paper_simulated" ? "active" : ""}`}
            onClick={() => onModeSwitch?.("paper_simulated")}
            title={t("实时模拟盘 / 本地撮合 / 无 provider 下单")}
          >
            {t("实时模拟盘")}
          </button>
          <button
            className={`exec-btn exec-mode-btn ${normalizedMode === "paper_actual" ? "active" : ""}`}
            onClick={() => onModeSwitch?.("paper_actual")}
            title={t("OKX 模拟盘 / 非真实资金 / provider 回执")}
          >
            {t("OKX 模拟盘")}
          </button>
        </div>

        <button
          className="exec-btn danger"
          onClick={onEmergencyStop}
          disabled={strategyCount === 0}
        >
          ⏹ {t("紧急停止")}
        </button>
      </div>

    </div>
  );
});
export default ExecutorTopBar;
