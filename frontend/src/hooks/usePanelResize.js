import { useState, useCallback, useEffect } from "react";

/**
 * 可拖拽面板大小调整 hook
 * 用法: const { width, startResize } = usePanelResize(260, 160, 500);
 */
export default function usePanelResize(initialWidth, minWidth = 120, maxWidth = 600) {
  const [width, setWidth] = useState(initialWidth);
  const [dragging, setDragging] = useState(false);

  const startResize = useCallback((e) => {
    e.preventDefault();
    setDragging(true);
  }, []);

  useEffect(() => {
    if (!dragging) return;

    const handleMouseMove = (e) => {
      setWidth((prev) => Math.min(maxWidth, Math.max(minWidth, prev + e.movementX)));
    };

    const handleMouseUp = () => setDragging(false);

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [dragging, minWidth, maxWidth]);

  return { width, setWidth, dragging, startResize };
}
