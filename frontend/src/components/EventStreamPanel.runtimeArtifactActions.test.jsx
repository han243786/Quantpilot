import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { EventPanelIntro } from "./EventStreamPanel";

function buildRuntime(overrides = {}) {
  return {
    artifactPersistenceStatus: "transient",
    backendError: null,
    runId: "run-preview-1",
    runKind: "simulation",
    status: "completed",
    ...overrides
  };
}

describe("EventPanelIntro runtime artifact actions", () => {
  it("shows explicit save and discard actions for transient completed artifacts", () => {
    const handleSaveCurrentRuntimeArtifact = vi.fn();
    const handleDiscardCurrentRuntimeArtifact = vi.fn();

    render(
      <EventPanelIntro
        runtime={buildRuntime()}
        displayedEvents={[]}
        panelNotice={null}
        setPanelNotice={vi.fn()}
        handleSaveCurrentRuntimeArtifact={handleSaveCurrentRuntimeArtifact}
        handleDiscardCurrentRuntimeArtifact={handleDiscardCurrentRuntimeArtifact}
      />
    );

    fireEvent.click(screen.getByTestId("runtime-artifact-save"));
    fireEvent.click(screen.getByTestId("runtime-artifact-discard"));

    expect(handleSaveCurrentRuntimeArtifact).toHaveBeenCalledTimes(1);
    expect(handleDiscardCurrentRuntimeArtifact).toHaveBeenCalledTimes(1);
    expect(screen.getByText("未保存结果仅保留在当前会话。")).toBeInTheDocument();
  });

  it("hides transient artifact actions after the artifact is persisted", () => {
    render(
      <EventPanelIntro
        runtime={buildRuntime({ artifactPersistenceStatus: "saved" })}
        displayedEvents={[]}
        panelNotice={null}
        setPanelNotice={vi.fn()}
        handleSaveCurrentRuntimeArtifact={vi.fn()}
        handleDiscardCurrentRuntimeArtifact={vi.fn()}
      />
    );

    expect(screen.queryByTestId("runtime-artifact-save")).not.toBeInTheDocument();
    expect(screen.queryByTestId("runtime-artifact-discard")).not.toBeInTheDocument();
  });
});
