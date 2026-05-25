/// v3.4.0: 执行端顶部工具栏 — 品牌、状态、紧急停止
/// v3.5.0: 新增 Paper/Live 模式切换

import { memo, useState, useEffect, useRef } from "react";

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
  const statusLabel = status === "running" ? "运行中" : status === "error" ? "错误" : "空闲";
  const statusClass = status === "running" ? "running" : status === "error" ? "error" : "stopped";
  const isLive = mode === "live";
  const [showConfirm, setShowConfirm] = useState(false);
  const cancelRef = useRef(null);

  // v3.5.1: ESC 关闭 + 焦点管理 (无障碍)
  useEffect(() => {
    if (!showConfirm) return;
    const handleKey = (e) => { if (e.key === "Escape") setShowConfirm(false); };
    document.addEventListener("keydown", handleKey);
    cancelRef.current?.focus();
    return () => document.removeEventListener("keydown", handleKey);
  }, [showConfirm]);

  const handleModeToggle = () => {
    if (isLive) {
      // Live → Paper 无需确认
      onModeSwitch?.("paper");
    } else {
      // Paper → Live 需确认
      setShowConfirm(true);
    }
  };

  return (
    <div className="exec-topbar">
      <div className="exec-topbar-brand">QuantPilot 实时执行端</div>
      <div className="exec-topbar-actions">
        <div className="exec-status">
          <span className={`exec-status-dot ${statusClass}`} />
          <span>{statusLabel}</span>
          {strategyCount > 0 && <span>· {strategyCount} 策略</span>}
          {statusMsg && <span className="exec-status-msg" title={statusMsg}>{statusMsg}</span>}
          {modeError && <span className="exec-status-msg exec-status-err" title={modeError}>{modeError}</span>}
        </div>

        <span className={`exec-runtime-badge ${(runtimeKind || "v3").toLowerCase()}`}>
          {(runtimeKind || "v3").toUpperCase()}
        </span>

        {/* v3.5.0: Paper/Live 模式切换 */}
        <button
          className={`exec-btn exec-mode-btn ${isLive ? "live" : "paper"}`}
          onClick={handleModeToggle}
          title={isLive ? "切换到模拟盘模式" : "切换到实盘模式"}
        >
          {isLive ? "🔴 Live" : "🟢 Paper"}
        </button>

        <button
          className="exec-btn danger"
          onClick={onEmergencyStop}
          disabled={strategyCount === 0}
        >
          ⏹ 紧急停止
        </button>
      </div>

      {/* v3.5.0: 实盘模式确认对话框 */}
      {showConfirm && (
        <div className="exec-confirm-overlay" onClick={() => setShowConfirm(false)}>
          <div className="exec-confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="exec-confirm-title" onClick={(e) => e.stopPropagation()}>
            <h3 id="exec-confirm-title">切换到实盘模式</h3>
            <p>切换到实盘模式将向真实交易所下单。</p>
            <p className="exec-confirm-warning">请确认已完成以下检查：</p>
            <ul>
              <li>已配置交易所 API 凭证</li>
              <li>策略参数已经过模拟盘验证</li>
              <li>了解实盘交易的风险</li>
            </ul>
            <div className="exec-confirm-actions">
              <button className="exec-btn" ref={cancelRef} onClick={() => setShowConfirm(false)}>取消</button>
              <button
                className="exec-btn danger"
                onClick={() => { setShowConfirm(false); onModeSwitch?.("live"); }}
              >
                确认切换
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
});
export default ExecutorTopBar;
