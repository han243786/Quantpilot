import { useEffect, useId, useRef, useState } from "react";

export default function StrategyHubInlineNote({ title, content, triggerLabel, triggerText }) {
  const popupId = useId();
  const [isOpen, setIsOpen] = useState(false);
  const [isPinned, setIsPinned] = useState(false);
  const [isPopupHovered, setIsPopupHovered] = useState(false);
  const [progress, setProgress] = useState(0);
  const [dragPosition, setDragPosition] = useState(null);
  const [isDragging, setIsDragging] = useState(false);
  const rootRef = useRef(null);
  const popupRef = useRef(null);
  const progressIntervalRef = useRef(null);
  const pinTimeoutRef = useRef(null);
  const dragStateRef = useRef(null);

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

  useEffect(() => {
    function handlePointerMove(event) {
      if (!dragStateRef.current) return;

      const deltaX = event.clientX - dragStateRef.current.pointerStartX;
      const deltaY = event.clientY - dragStateRef.current.pointerStartY;

      setDragPosition({
        left: dragStateRef.current.popupStartLeft + deltaX,
        top: dragStateRef.current.popupStartTop + deltaY
      });
    }

    function stopDragging() {
      dragStateRef.current = null;
      setIsDragging(false);
    }

    window.addEventListener("mousemove", handlePointerMove);
    window.addEventListener("mouseup", stopDragging);

    return () => {
      window.removeEventListener("mousemove", handlePointerMove);
      window.removeEventListener("mouseup", stopDragging);
    };
  }, []);

  function startPinCountdown() {
    if (isPinned) return;

    clearPinCountdown();
    setIsPopupHovered(true);
    setProgress(0);

    const startedAt = Date.now();
    progressIntervalRef.current = window.setInterval(() => {
      const elapsed = Date.now() - startedAt;
      setProgress(Math.min(elapsed / 3000, 1));
    }, 50);

    pinTimeoutRef.current = window.setTimeout(() => {
      clearPinCountdown({ resetProgress: false });
      setProgress(1);
      setIsPinned(true);
      setIsPopupHovered(false);
    }, 3000);
  }

  function closePopup() {
    clearPinCountdown();
    dragStateRef.current = null;
    setIsPinned(false);
    setIsPopupHovered(false);
    setIsDragging(false);
    setDragPosition(null);
    setIsOpen(false);
  }

  function startDragging(event) {
    if (!isPinned) return;
    if (event.target.closest(".strategy-inline-note__close")) return;

    const rootRect = rootRef.current?.getBoundingClientRect();
    const popupRect = popupRef.current?.getBoundingClientRect();
    if (!rootRect || !popupRect) return;

    dragStateRef.current = {
      pointerStartX: event.clientX,
      pointerStartY: event.clientY,
      popupStartLeft: popupRect.left - rootRect.left,
      popupStartTop: popupRect.top - rootRect.top
    };
    setDragPosition({
      left: popupRect.left - rootRect.left,
      top: popupRect.top - rootRect.top
    });
    setIsDragging(true);
    event.preventDefault();
  }

  return (
    <span
      ref={rootRef}
      className={`strategy-inline-note${isOpen ? " strategy-inline-note--open" : ""}${
        isPinned ? " strategy-inline-note--pinned" : ""
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
      <h1 className="strategy-inline-note__heading">
        <button
          type="button"
          className="strategy-inline-note__trigger-text"
          aria-label={triggerLabel}
          aria-describedby={isOpen ? popupId : undefined}
        >
          {triggerText}
        </button>
      </h1>

      {isOpen ? (
        <>
          <span
            className="strategy-inline-note__bridge"
            aria-hidden="true"
            data-testid="strategy-inline-note-bridge"
          />
          <span
            id={popupId}
            role="tooltip"
            ref={popupRef}
            className={`strategy-inline-note__popup${
              isPinned ? " strategy-inline-note__popup--pinned" : ""
            }${isDragging ? " strategy-inline-note__popup--dragging" : ""}`}
            style={dragPosition ? { left: `${dragPosition.left}px`, top: `${dragPosition.top}px` } : undefined}
            onMouseEnter={startPinCountdown}
            onMouseLeave={() => {
              setIsPopupHovered(false);
              if (!isPinned) {
                clearPinCountdown();
              }
            }}
            onMouseDown={startDragging}
          >
            <span className="strategy-inline-note__popup-header">
              <span className="strategy-inline-note__popup-title">{title}</span>
              {isPinned ? (
                <button
                  type="button"
                  className="strategy-inline-note__close"
                  aria-label={`关闭${title}`}
                  onClick={closePopup}
                >
                  x
                </button>
              ) : isPopupHovered ? (
                <span
                  className="strategy-inline-note__pin-progress"
                  style={{ "--progress-deg": `${progress * 360}deg` }}
                  aria-hidden="true"
                />
              ) : null}
            </span>
            <span className="strategy-inline-note__popup-body">{content}</span>
          </span>
        </>
      ) : null}
    </span>
  );
}
