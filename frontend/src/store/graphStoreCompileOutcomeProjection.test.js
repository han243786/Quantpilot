import { describe, expect, it } from "vitest";
import { inferCompileFailureSource } from "./graphStoreCompileOutcomeProjection";

describe("graphStoreCompileOutcomeProjection", () => {
  it("prefers the explicit compile_source on the error", () => {
    expect(
      inferCompileFailureSource(
        {
          hasStrategyIrArtifact: true,
          runtimeCompileSource: "formal_quantscript"
        },
        null,
        { compile_source: "runtime_fallback" }
      )
    ).toBe("runtime_fallback");
  });

  it("labels Strategy IR preflight failures before runtime fallback branches", () => {
    expect(
      inferCompileFailureSource(
        {
          hasStrategyIrArtifact: true,
          runtimeCompileSource: "formal_quantscript"
        },
        null,
        {}
      )
    ).toBe("strategy_ir");
  });

  it("labels formal QuantScript failures when no Strategy IR preflight is pending", () => {
    expect(
      inferCompileFailureSource(
        {
          hasStrategyIrArtifact: false,
          runtimeCompileSource: "formal_quantscript"
        },
        { compilable: true },
        {}
      )
    ).toBe("formal_quantscript");
  });
});
