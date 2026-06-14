import { afterEach, describe, expect, it } from "vitest";
import { act, render, screen } from "@testing-library/react";
import NodePriceOverlay from "./NodePriceOverlay";
import { useGraphStore } from "../store/graphStore";

describe("NodePriceOverlay", () => {
  const initialState = useGraphStore.getState();

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("hydrates the current price and updates from graph node changes without a parent render", () => {
    act(() => {
      useGraphStore.setState({
        ...initialState,
        graph: {
          ...initialState.graph,
          nodes: [
            {
              id: "node_data",
              type: "data",
              runtime_state: {
                metrics: {
                  latest_price: 123.45
                }
              }
            }
          ]
        }
      });
    });

    render(<NodePriceOverlay nodeId="node_data" />);

    const overlay = screen.getByTestId("ticker-price-node_data");
    expect(overlay).toHaveTextContent("123.45");

    act(() => {
      useGraphStore.setState({
        graph: {
          ...useGraphStore.getState().graph,
          nodes: [
            {
              id: "node_data",
              type: "data",
              runtime_state: {
                metrics: {
                  latest_price: 456.78
                }
              }
            }
          ]
        }
      });
    });

    expect(overlay).toHaveTextContent("456.78");
  });
});
