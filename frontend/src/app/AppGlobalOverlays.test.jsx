import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import AppGlobalOverlays from "./AppGlobalOverlays";

const tutorialState = vi.hoisted(() => ({
  closeTutorial: vi.fn(),
  createTutorialSteps: vi.fn(() => [{ id: "welcome" }]),
  tutorialOpen: true,
}));

vi.mock("../components/CommandPalette", () => ({
  default: ({ open, onClose }) => (
    <section data-open={open ? "true" : "false"} data-testid="command-palette">
      <button type="button" onClick={onClose}>
        close command
      </button>
    </section>
  ),
}));

vi.mock("../components/ToastContainer", () => ({
  default: () => <div data-testid="toast-container" />,
}));

vi.mock("../components/TutorialOverlay", () => ({
  default: ({ onClose, steps }) => (
    <section data-step-count={steps.length} data-testid="tutorial-overlay">
      <button type="button" onClick={onClose}>
        close tutorial
      </button>
    </section>
  ),
}));

vi.mock("../data/tutorialSteps", () => ({
  createTutorialSteps: tutorialState.createTutorialSteps,
}));

vi.mock("../hooks/useTutorial", () => ({
  useTutorial: () => ({
    closeTutorial: tutorialState.closeTutorial,
    tutorialOpen: tutorialState.tutorialOpen,
  }),
}));

vi.mock("../i18n", () => ({
  useI18n: () => ({ t: (text) => text }),
}));

describe("AppGlobalOverlays", () => {
  beforeEach(() => {
    tutorialState.closeTutorial.mockReset();
    tutorialState.createTutorialSteps.mockClear();
    tutorialState.tutorialOpen = true;
  });

  it("hosts the tutorial, command palette, and toast container", () => {
    const onCloseCommandPalette = vi.fn();

    render(
      <AppGlobalOverlays
        commandPaletteOpen
        onCloseCommandPalette={onCloseCommandPalette}
      />
    );

    expect(screen.getByTestId("tutorial-overlay")).toHaveAttribute(
      "data-step-count",
      "1"
    );
    expect(screen.getByTestId("command-palette")).toHaveAttribute(
      "data-open",
      "true"
    );
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();

    screen.getByRole("button", { name: "close tutorial" }).click();
    screen.getByRole("button", { name: "close command" }).click();

    expect(tutorialState.closeTutorial).toHaveBeenCalledTimes(1);
    expect(onCloseCommandPalette).toHaveBeenCalledTimes(1);
    expect(tutorialState.createTutorialSteps).toHaveBeenCalledTimes(1);
  });

  it("keeps the command palette and toast host when the tutorial is closed", () => {
    tutorialState.tutorialOpen = false;

    render(
      <AppGlobalOverlays
        commandPaletteOpen={false}
        onCloseCommandPalette={() => {}}
      />
    );

    expect(screen.queryByTestId("tutorial-overlay")).not.toBeInTheDocument();
    expect(screen.getByTestId("command-palette")).toHaveAttribute(
      "data-open",
      "false"
    );
    expect(screen.getByTestId("toast-container")).toBeInTheDocument();
  });
});
