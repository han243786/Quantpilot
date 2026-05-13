import { useState, useCallback, useEffect } from "react";
import { useI18n } from "../i18n";

function getElementRect(selector) {
  const el = document.querySelector(selector);
  if (!el) return null;
  return el.getBoundingClientRect();
}

export default function TutorialOverlay({ steps = [], onClose }) {
  const { t } = useI18n();
  const [current, setCurrent] = useState(0);
  const [targetRect, setTargetRect] = useState(null);
  const [targetMissing, setTargetMissing] = useState(false);

  const step = steps[current];
  const isFirst = current === 0;
  const isLast = current === steps.length - 1;

  const updateRect = useCallback(() => {
    if (step?.target) {
      const rect = getElementRect(step.target);
      setTargetRect(rect);
      setTargetMissing(!rect);
    }
  }, [step]);

  useEffect(() => {
    updateRect();
    const handle = setInterval(updateRect, 300);
    window.addEventListener("resize", updateRect);
    window.addEventListener("scroll", updateRect, true);
    return () => {
      clearInterval(handle);
      window.removeEventListener("resize", updateRect);
      window.removeEventListener("scroll", updateRect, true);
    };
  }, [updateRect]);

  const goNext = () => { if (!isLast) setCurrent((c) => c + 1); };
  const goPrev = () => { if (!isFirst) setCurrent((c) => c - 1); };
  const handleClose = () => { setCurrent(0); onClose?.(); };

  if (!step || steps.length === 0) return null;

  return (
    <div className="tutorial-overlay" data-testid="tutorial-overlay">
      {targetRect && (
        <div
          className="tutorial-highlight"
          style={{
            left: targetRect.left - 6,
            top: targetRect.top - 6,
            width: targetRect.width + 12,
            height: targetRect.height + 12,
          }}
        />
      )}

      <div className="tutorial-bubble" data-testid="tutorial-bubble">
        <div className="tutorial-bubble-header">
          <span className="tutorial-step-indicator">
            {current + 1} / {steps.length}
          </span>
        </div>
        <h3 className="tutorial-step-title">{step.title}</h3>
        <p className="tutorial-step-desc">{step.description}</p>
        {targetMissing && (
          <p className="tutorial-step-hint" style={{color:"var(--ad-warning)",fontSize:"0.85rem",marginTop:8}}>
            目标元素未就绪，请等待界面完全加载后再继续。
          </p>
        )}
        <div className="tutorial-bubble-actions">
          <button
            className="ghost-btn"
            onClick={handleClose}
            data-testid="tutorial-close"
          >
            {t("退出教程")}
          </button>
          <div className="tutorial-nav">
            <button
              className="ghost-btn"
              onClick={goPrev}
              disabled={isFirst}
              data-testid="tutorial-prev"
            >
              {t("上一步")}
            </button>
            {isLast ? (
              <button
                className="primary-btn"
                onClick={handleClose}
                data-testid="tutorial-finish"
              >
                {t("完成")}
              </button>
            ) : (
              <button
                className="primary-btn"
                onClick={goNext}
                data-testid="tutorial-next"
              >
                {t("下一步")}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
