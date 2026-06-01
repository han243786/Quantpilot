import { useEffect, useState } from "react";
import { useGraphStore } from "../store/graphStore";

export function useAppInitialization() {
  const initialize = useGraphStore((state) => state.initialize);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    let disposed = false;
    void initialize().finally(() => {
      if (!disposed) {
        setIsInitialized(true);
      }
    });
    return () => {
      disposed = true;
    };
  }, [initialize]);

  return isInitialized;
}
