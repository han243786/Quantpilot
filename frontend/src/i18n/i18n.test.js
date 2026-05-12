import { describe, expect, it } from "vitest";
import { translateText, setGlobalLocale, getGlobalLocale, defineLocale, registerLocale } from "./index";

describe("i18n", () => {
  it("translateText returns baseText when key missing", () => {
    expect(translateText("不存在的key")).toBe("不存在的key");
  });

  it("translateText resolves registered keys", () => {
    registerLocale("test-xx", { "保存": "Save" });
    expect(translateText("保存", {}, "test-xx")).toBe("Save");
  });

  it("translateText falls back to baseText when locale has null value", () => {
    registerLocale("test-null", { "hello": null });
    expect(translateText("hello", {}, "test-null")).toBe("hello");
  });

  it("translateText supports interpolation with {var} syntax", () => {
    registerLocale("test-interp", { "有{count}个": "There are {count}" });
    expect(translateText("有{count}个", { count: "5" }, "test-interp")).toBe("There are 5");
  });

  it("translateText handles missing interpolation variables gracefully", () => {
    registerLocale("test-missing", { "hello{x}": "hi{x}" });
    expect(translateText("hello{x}", {}, "test-missing")).toBe("hi");
  });

  it("setGlobalLocale and getGlobalLocale work correctly", () => {
    setGlobalLocale("zh-CN");
    expect(getGlobalLocale()).toBe("zh-CN");
    setGlobalLocale("en-US");
    expect(getGlobalLocale()).toBe("en-US");
  });

  it("zh-CN self-mapping: compiled strings match key", () => {
    expect(translateText("保存", {}, "zh-CN")).toBe("保存");
    expect(translateText("编辑", {}, "zh-CN")).toBe("编辑");
  });
});
