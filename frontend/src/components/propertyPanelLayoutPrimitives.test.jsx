import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  PropertyPanelShell,
  PropertySection,
  WorkspaceInspectorShell,
  renderFieldInput
} from "./propertyPanelLayoutPrimitives";

describe("propertyPanelLayoutPrimitives", () => {
  it("renders property shell and section hierarchy", () => {
    render(
      <PropertyPanelShell title="Inspector" subtitle="Shared shell">
        <PropertySection
          kicker="Graph"
          title="Configuration"
          summary="Editable defaults"
          testId="config-section"
        >
          <span>section body</span>
        </PropertySection>
      </PropertyPanelShell>
    );

    expect(screen.getByText("Inspector")).toBeInTheDocument();
    expect(screen.getByTestId("config-section")).toHaveAttribute(
      "aria-label",
      "Configuration"
    );
    expect(screen.getByText("Graph")).toBeInTheDocument();
    expect(screen.getByText("Editable defaults")).toBeInTheDocument();
    expect(screen.getByText("section body")).toBeInTheDocument();
  });

  it("renders workspace inspector meta, actions, and context notice", () => {
    render(
      <WorkspaceInspectorShell
        title="Code"
        subtitle="Source lane"
        contextNotice="Using selected source lane"
        actions={<button type="button">Run</button>}
        summaryItems={[
          { label: "Errors", value: "0", note: "clean", tone: "success" },
          { label: "Warnings", value: "2" }
        ]}
      >
        <div>workspace body</div>
      </WorkspaceInspectorShell>
    );

    expect(screen.getByText("Code")).toBeInTheDocument();
    expect(screen.getByText("Using selected source lane")).toBeInTheDocument();
    expect(screen.getByText("Errors")).toBeInTheDocument();
    expect(screen.getByText("0")).toBeInTheDocument();
    expect(screen.getByText("clean")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Run" })).toBeInTheDocument();
    expect(screen.getByText("workspace body")).toBeInTheDocument();
  });

  it("normalizes field input change payloads", () => {
    const onSelect = vi.fn();
    const onBoolean = vi.fn();
    const onNumber = vi.fn();

    render(
      <div>
        {renderFieldInput(
          {
            key: "mode",
            type: "select",
            options: [
              { value: "paper", label: "Paper" },
              { value: "live", label: "Live" }
            ]
          },
          "paper",
          onSelect
        )}
        {renderFieldInput({ key: "enabled", type: "boolean" }, true, onBoolean)}
        {renderFieldInput({ key: "limit", type: "number" }, 3, onNumber)}
      </div>
    );

    fireEvent.change(screen.getByTestId("prop-input-mode"), {
      target: { value: "live" }
    });
    fireEvent.click(screen.getByTestId("prop-input-enabled"));
    fireEvent.change(screen.getByTestId("prop-input-limit"), {
      target: { value: "7" }
    });

    expect(onSelect).toHaveBeenCalledWith("live");
    expect(onBoolean).toHaveBeenCalledWith(false);
    expect(onNumber).toHaveBeenCalledWith(7);
  });
});
