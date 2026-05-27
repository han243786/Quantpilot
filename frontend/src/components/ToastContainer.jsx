import { useEffect, useRef } from "react";
import { useNotification } from "../hooks/useNotification";
import { useI18n } from "../i18n";

// v3.6.0 U7: Global toast notification container
// Listens for 'qp-toast' custom events and renders toast notifications

const AUTO_DISMISS_MS = 3000;

function ToastItem({ toast, onDismiss, dismissLabel }) {
  const toastType = ["success", "error", "info"].includes(toast.type) ? toast.type : "info";
  const isPersistent = toast.type === "error";

  return (
    <div
      role="alert"
      onClick={() => isPersistent && onDismiss(toast.id)}
      className={`toast-item toast-item--${toastType}${isPersistent ? " toast-item--persistent" : ""}`}
    >
      {toast.message}
      {isPersistent && (
        <span className="toast-item__dismiss-hint">
          {dismissLabel}
        </span>
      )}
    </div>
  );
}

export default function ToastContainer() {
  const { t } = useI18n();
  const { toasts, removeToast } = useNotification();
  const timersRef = useRef({});

  // Auto-dismiss non-error toasts after 3 seconds
  useEffect(() => {
    toasts.forEach((toast) => {
      if (toast.type === "error") return;
      if (timersRef.current[toast.id]) return;
      timersRef.current[toast.id] = setTimeout(() => {
        delete timersRef.current[toast.id];
        removeToast(toast.id);
      }, AUTO_DISMISS_MS);
    });

    // Clean up timers for removed toasts
    const activeIds = new Set(toasts.map((t) => t.id));
    Object.keys(timersRef.current).forEach((id) => {
      const numId = Number(id);
      if (!activeIds.has(numId)) {
        clearTimeout(timersRef.current[numId]);
        delete timersRef.current[numId];
      }
    });
  }, [toasts, removeToast]);

  // Cleanup all timers on unmount
  useEffect(() => {
    return () => {
      Object.values(timersRef.current).forEach(clearTimeout);
      timersRef.current = {};
    };
  }, []);

  return (
    <div className="toast-container">
      {toasts.map((toast) => (
        <ToastItem
          key={toast.id}
          toast={toast}
          onDismiss={removeToast}
          dismissLabel={t("(点击关闭)")}
        />
      ))}
    </div>
  );
}
