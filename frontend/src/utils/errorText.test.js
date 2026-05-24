import { describe, expect, it, vi } from "vitest";
import { humanizeErrorText } from "./errorText";

describe("humanizeErrorText", () => {
  it("keeps plain-text backend errors without warning noise", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});

    expect(humanizeErrorText("backend unavailable", "保存策略图失败。")).toBe(
      "backend unavailable"
    );
    expect(warn).not.toHaveBeenCalled();

    warn.mockRestore();
  });

  it("extracts structured JSON error messages when provided", () => {
    expect(
      humanizeErrorText(
        JSON.stringify({ message: "Failed to fetch" }),
        "操作失败。"
      )
    ).toBe("无法连接后端服务，请确认本地 API 已启动。");
  });
});
