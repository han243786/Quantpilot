import { describe, expect, it } from "vitest";
import { getAuthHeaders, resolveApiBase } from "./apiBase";

describe("apiBase", () => {
  it("uses an explicit API base and trims trailing slashes", () => {
    expect(
      resolveApiBase({
        rawBase: "  http://localhost:7897/api/// ",
        hasWindow: true,
      })
    ).toBe("http://localhost:7897/api");
  });

  it("uses relative API path in the browser without an explicit base", () => {
    expect(resolveApiBase({ rawBase: "", hasWindow: true })).toBe("/api");
  });

  it("uses local API fallback outside the browser", () => {
    expect(resolveApiBase({ rawBase: "", hasWindow: false })).toBe(
      "http://127.0.0.1:3000/api"
    );
  });

  it("keeps auth headers as an extension point", () => {
    expect(getAuthHeaders()).toEqual({});
  });
});
