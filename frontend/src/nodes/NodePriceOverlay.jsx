import { memo, useEffect, useRef } from "react";
import { useGraphStore } from "../store/graphStore";

const NodePriceOverlay = memo(function NodePriceOverlay({ nodeId }) {
  const ref = useRef(null);

  useEffect(() => {
    const initialPrice = useGraphStore
      .getState()
      .graph.nodes.find((node) => node.id === nodeId)?.runtime_state?.metrics
      ?.latest_price;
    if (initialPrice != null && ref.current) {
      ref.current.textContent = String(initialPrice);
    }

    const unsub = useGraphStore.subscribe((state, prevState) => {
      if (state.graph.nodes === prevState.graph.nodes) return;

      const node = state.graph.nodes.find((item) => item.id === nodeId);
      if (!node || node.type !== "data") return;

      const price = node.runtime_state?.metrics?.latest_price;
      if (price != null && ref.current) {
        ref.current.textContent = String(price);
      }
    });

    return unsub;
  }, [nodeId]);

  return (
    <span
      ref={ref}
      className="ticker-price-overlay"
      data-testid={`ticker-price-${nodeId}`}
    >
      --
    </span>
  );
}, () => true);

export default NodePriceOverlay;
