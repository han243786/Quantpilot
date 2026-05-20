import { useState, useEffect, useRef } from "react";

/**
 * useOrderAnimation
 *
 * React hook that provides animation CSS classes for order items.
 *
 * - `order--entering`: slides in from the right (300ms) on mount.
 * - `order--flash`: green background flash (500ms) triggered by `shouldFlash`.
 *
 * @param {boolean} shouldFlash - when truthy, triggers a green flash animation.
 *                                Set to `true` when the order transitions to a
 *                                "filled" / final state.
 * @returns {string} CSS class string combining base and animation classes.
 */
export function useOrderAnimation(shouldFlash) {
  const [isEntering, setIsEntering] = useState(true);
  const [isFlashing, setIsFlashing] = useState(false);
  const flashedRef = useRef(false);

  // Mount animation: slide in from right, removed after 300ms
  useEffect(() => {
    const timer = setTimeout(() => setIsEntering(false), 300);
    return () => clearTimeout(timer);
  }, []);

  // Flash animation: green background pulse, removed after 500ms
  // Only fires once per lifecycle per unique identity (caller must ensure
  // the component remounts or passes a new `shouldFlash` value).
  useEffect(() => {
    if (shouldFlash && !flashedRef.current) {
      flashedRef.current = true;
      setIsFlashing(true);
      const timer = setTimeout(() => setIsFlashing(false), 500);
      return () => clearTimeout(timer);
    }
  }, [shouldFlash]);

  return [isEntering ? "order--entering" : "", isFlashing ? "order--flash" : ""]
    .filter(Boolean)
    .join(" ");
}
