import { useCallback, useEffect, useState } from "react";

// v3.6.0 U7: Simple event-based toast system
// Components call: window.dispatchEvent(new CustomEvent('qp-toast', { detail: { type: 'success'|'error'|'info', message: '...' } }))

let toastId = 0;

export function useNotification() {
  const [toasts, setToasts] = useState([]);

  const addToast = useCallback((type, message) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, type, message }]);
    return id;
  }, []);

  const removeToast = useCallback((id) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  useEffect(() => {
    const handler = (event) => {
      const { type, message } = event.detail || {};
      if (type && message) {
        addToast(type, message);
      }
    };
    window.addEventListener("qp-toast", handler);
    return () => window.removeEventListener("qp-toast", handler);
  }, [addToast]);

  return { toasts, addToast, removeToast };
}
