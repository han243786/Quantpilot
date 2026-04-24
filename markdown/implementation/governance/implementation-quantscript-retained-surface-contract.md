# QuantScript Retained-Surface Contract

This file is the active wording boundary for `CL-P1-005`.

## Goal

Keep formal QuantScript `V1` strict, honest, and limited to the retained
executable trunk.

## Retained source categories

- positive retained examples live in `quantscript/authoring_samples/`
- intentional retained-boundary failures live in `quantscript/boundary_samples/`
- compatibility-only parser examples inside crate tests must not be described as
  release-facing authoring samples
- parser compatibility tests must name themselves as compatibility-only when
  they cover syntax outside the retained executable trunk

## Retained executable truth

- parser acceptance is not product support
- supported product entrypoints remain:
  - `analyze_formal_quant_script(...)`
  - `parse_formal_quant_script_config(...)`
  - `/api/quantscript/formal/compile`
- unsupported constructs should fail through stable `QS06xx` or `QPQSLOWxxx`
  diagnostics instead of falling through to vague later-stage errors

## Closeout rules

- do not keep negative boundary fixtures inside active authoring-sample folders
- do not describe parser-only legacy syntax as part of the admitted executable
  trunk
- do not use parser acceptance, test pass status, or fixture presence as
  shorthand for release-facing authoring support
- do not widen the retained surface without a written contract update in the
  active docs

## Current implementation anchors

- `quantscript/QUANTSCRIPT_SUPPORTED_SURFACE.md`
- `quantscript/QUANTSCRIPT_REAL_STRATEGY_AUTHORING_TRIAL.md`
- `markdown/guides/quantscript/guide-formal-quantscript-syntax.md`
- `markdown/guides/quantscript/guide-v1-freeze-descope-checklist.md`
- `tests/quantscript_real_strategy_authoring.rs`
