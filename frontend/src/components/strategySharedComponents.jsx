import { useEffect, useRef, useState } from "react";

const PIN_COUNTDOWN_MS = 3000;
const PIN_COUNTDOWN_INTERVAL_MS = 50;

export function StrategyCardNote({ label, note }) {
  const [isOpen, setIsOpen] = useState(false);
  const [isPinned, setIsPinned] = useState(false);
  const [isPopupHovered, setIsPopupHovered] = useState(false);
  const [progress, setProgress] = useState(0);
  const progressIntervalRef = useRef(null);
  const pinTimeoutRef = useRef(null);

  function clearPinCountdown({ resetProgress = true } = {}) {
    if (progressIntervalRef.current) {
      window.clearInterval(progressIntervalRef.current);
      progressIntervalRef.current = null;
    }
    if (pinTimeoutRef.current) {
      window.clearTimeout(pinTimeoutRef.current);
      pinTimeoutRef.current = null;
    }
    if (resetProgress) {
      setProgress(0);
    }
  }

  useEffect(() => () => clearPinCountdown(), []);

  function startPinCountdown() {
    if (isPinned) return;

    clearPinCountdown();
    setIsPopupHovered(true);
    setProgress(0);

    const startedAt = Date.now();
    progressIntervalRef.current = window.setInterval(() => {
      const elapsed = Date.now() - startedAt;
      setProgress(Math.min(elapsed / PIN_COUNTDOWN_MS, 1));
    }, PIN_COUNTDOWN_INTERVAL_MS);

    pinTimeoutRef.current = window.setTimeout(() => {
      clearPinCountdown({ resetProgress: false });
      setProgress(1);
      setIsPinned(true);
      setIsPopupHovered(false);
    }, 3000);
  }

  function closePopup() {
    clearPinCountdown();
    setIsPinned(false);
    setIsPopupHovered(false);
    setIsOpen(false);
  }

  if (!note) return <span>{label}</span>;

  return (
    <span
      className={`strategy-card-note${isOpen ? " strategy-card-note--open" : ""}${
        isPinned ? " strategy-card-note--pinned" : ""
      }`}
      onMouseEnter={() => setIsOpen(true)}
      onMouseLeave={() => {
        clearPinCountdown();
        setIsPopupHovered(false);
        if (!isPinned) {
          setIsOpen(false);
        }
      }}
    >
      <button
        type="button"
        className="strategy-card-note__trigger"
        aria-label={`查看${label}说明`}
        onFocus={() => setIsOpen(true)}
      >
        {label}
      </button>
      {isOpen ? (
        <>
          <span className="strategy-card-note__bridge" aria-hidden="true" />
          <span
            role="tooltip"
            className={`strategy-card-note__popup${
              isPinned ? " strategy-card-note__popup--pinned" : ""
            }`}
            onMouseEnter={startPinCountdown}
            onMouseLeave={() => {
              setIsPopupHovered(false);
              if (!isPinned) {
                clearPinCountdown();
              }
            }}
          >
            <span className="strategy-card-note__popup-header">
              <span className="strategy-card-note__title">{label}</span>
              {isPinned ? (
                <button
                  type="button"
                  className="strategy-card-note__close"
                  aria-label={`关闭${label}说明`}
                  onClick={closePopup}
                >
                  x
                </button>
              ) : isPopupHovered ? (
                <span
                  className="strategy-card-note__pin-progress"
                  style={{ "--progress-deg": `${progress * 360}deg` }}
                  aria-hidden="true"
                />
              ) : null}
            </span>
            <span className="strategy-card-note__body">{note}</span>
          </span>
        </>
      ) : null}
    </span>
  );
}

export function StrategyMetricCard({ label, value, note }) {
  return (
    <div className="strategy-kpi-card">
      <div className="strategy-status-card__line">
        <StrategyCardNote label={label} note={note} />
        <strong>{value}</strong>
      </div>
    </div>
  );
}

export function StrategyOpsCard({ title, value, note, tone = "muted" }) {
  return (
    <div className={`strategy-ops-card strategy-ops-card--${tone}`}>
      <div className="strategy-status-card__line strategy-ops-card__title">
        <StrategyCardNote label={title} note={note} />
        <strong>{value}</strong>
      </div>
    </div>
  );
}

export function StrategyTaskGroup({
  label,
  tone = "muted",
  children,
  className = "",
  showLabel = true
}) {
  return (
    <div className={`strategy-task-group strategy-task-group--${tone} ${className}`.trim()}>
      {showLabel ? <span className="strategy-task-group__label">{label}</span> : null}
      <div className="strategy-task-group__actions">{children}</div>
    </div>
  );
}

export function ActivityListCard({
  title,
  subtitle,
  items,
  emptyText,
  renderMeta,
  testId
}) {
  return (
    <section className="strategy-activity-card" data-testid={testId}>
      <div className="strategy-card-header">
        <div>
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
      </div>

      <div className="strategy-activity-list">
        {items.length === 0 ? <div className="strategy-directory-empty">{emptyText}</div> : null}
        {items.map((item) => (
          <div key={`${item.kind}-${item.id}`} className="strategy-activity-item">
            <div className="strategy-activity-item__copy">
              <div className="strategy-activity-item__title">
                <strong>{item.title}</strong>
                <span className={`status-pill ${item.kind === "backtest" ? "info" : "muted"}`}>
                  {item.kind === "backtest" ? "回测" : "模拟"}
                </span>
              </div>
              <div className="strategy-activity-item__meta">
                <span>{item.graphId}</span>
                <span>{item.createdAtLabel}</span>
                <span>{item.note}</span>
              </div>
              <small>{item.detail}</small>
            </div>
            {renderMeta ? (
              <div className="strategy-activity-item__actions">{renderMeta(item)}</div>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}
