import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import DesktopTitleBar from "./DesktopTitleBar";

vi.mock("../i18n", () => ({
  useI18n: () => ({ t: (text) => text })
}));

describe("DesktopTitleBar", () => {
  it("does not render without a desktop window", () => {
    const { container } = render(<DesktopTitleBar appWindow={null} isMaximized={false} />);

    expect(container).toBeEmptyDOMElement();
  });

  it("delegates titlebar controls to the desktop window", () => {
    const appWindow = {
      minimize: vi.fn(),
      toggleMaximize: vi.fn(),
      close: vi.fn(),
    };

    render(<DesktopTitleBar appWindow={appWindow} isMaximized={false} />);

    screen.getByRole("button", { name: "最小化" }).click();
    screen.getByRole("button", { name: "最大化" }).click();
    screen.getByRole("button", { name: "关闭" }).click();

    expect(appWindow.minimize).toHaveBeenCalledTimes(1);
    expect(appWindow.toggleMaximize).toHaveBeenCalledTimes(1);
    expect(appWindow.close).toHaveBeenCalledTimes(1);
  });
});
