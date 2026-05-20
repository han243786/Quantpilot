import { useEffect, useRef, useState } from "react";
import { useNotification } from "../hooks/useNotification";

// v3.6.0 U7: Global toast notification container
// Listens for 'qp-toast' custom events and renders toast notifications

const TOAST_STYLES = {
  success: { background: "#16a34a", color: "#fff" },
  error: { background: "#dc2626", color: "#fff" },
  info: { background: "#2563eb", color: "#fff" },
};

const AUTO_DISMISS_MS = 3000;

function ToastItem({ toast, onDismiss }) {
  const style = TOAST_STYLES[toast.type] || TOAST_STYLES.info;
  const isPersistent = toast.type === "error";

  return (
    <div
      role="alert"
      onClick={() => isPersistent && onDismiss(toast.id)}
      style={{
        ...style,
        padding: "10px 16px",
        borderRadius: 6,
        marginBottom: 8,
        cursor: isPersistent ? "pointer" : "default",
        boxShadow: "0 2px 8px rgba(0,0,0,0.2)",
        fontSize: 14,
        lineHeight: 1.4,
        minWidth: 260,
        maxWidth: 400,
        wordBreak: "break-word",
        transition: "opacity 0.2s",
        pointerEvents: "auto",
      }}
    >
      {toast.message}
      {isPersistent && (
        <span style={{ marginLeft: 8, opacity: 0.7, fontSize: 12 }}>
          (点击关闭)
        </span>
      )}
    </div>
  );
}

export default function ToastContainer() {
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
    <div
      style={{
        position: "fixed",
        top: 48,
        right: 16,
        zIndex: 9999,
        display: "flex",
        flexDirection: "column",
        alignItems: "flex-end",
        pointerEvents: "none",
      }}
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={removeToast} />
      ))}
    </div>
  );
}
