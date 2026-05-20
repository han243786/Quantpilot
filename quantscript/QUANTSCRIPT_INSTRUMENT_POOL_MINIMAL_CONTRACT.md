# QuantScript Instrument Pool Minimal Contract

This document defines the next-phase design direction for instrument-pool
authoring in QuantScript.

It does **not** describe landed `V1` behavior.

Current retained `V1` behavior still follows the existing `universe(...)`,
`filter(...)`, `sort_by(...)`, and `top(...)` helper path, and dynamic
factor-ranked universe construction still fails at the current `QPQSLOW011`
boundary.

This contract exists to define the replacement direction that better matches
real strategy authoring goals.

## 1. Purpose

The purpose of this contract is to replace the old helper-centered mental model:

- `universe(...)`
- `filter(...)`
- `sort_by(...)`
- `top(...)`

with a first-class instrument-pool pipeline that can support:

- readable strategy authoring
- frontend pool-flow visualization
- metadata-based selection
- dynamic per-symbol feature computation
- factor-ranked selection
- retained weighting and rebalance flow

The design target is:

- source-first authoring
- explicit pool semantics
- frontend-readable strategy structure
- compatibility with `quantscript_authoring_view`
- no new duplicate truth source

## 2. Why the old boundary is not enough

`QPQSLOW011` currently means:

- dynamic factor-ranked universe construction is rejected

That boundary is consistent with current `V1`, but it is not a good long-term
design center for real strategy authoring.

The real authoring goal is not "metadata helpers only".
The real goal is:

- define a candidate pool
- apply basic eligibility filters
- compute per-symbol features
- select instruments from those features
- assign weights
- rebalance

So the next phase should not just weaken or remove `QPQSLOW011`.
It should replace the old model that made `QPQSLOW011` necessary.

## 3. Source-of-truth rule

The truth-source rule remains unchanged:

1. formal QuantScript source remains the only editable truth source
2. semantic analysis defines executable meaning
3. lowering defines runtime/config meaning
4. any instrument-pool artifact is derived, not editable truth

This contract does **not** authorize:

- a second editable pool language
- direct editing of a derived pool artifact
- frontend-owned selection semantics

Any future pool artifact must remain a derived view or derived lowered summary.

## 4. Minimal semantic pipeline

The next-phase instrument pool model should be expressed as a stable pipeline:

1. `source`
2. `eligibility`
3. `features`
4. `selection`
5. `weighting`
6. `rebalance`

These are semantic stages, not necessarily final syntax blocks.

## 5. Stage definitions

### `source`

Defines where the candidate instrument set comes from.

Examples:

- exchange / market / quote-based universe
- explicit symbol list
- future retained static group helpers

Minimal retained direction:

- `universe(...)`
- `symbols(...)`

### `eligibility`

Applies stable, mostly structural constraints before ranking/selection.

Examples:

- `quote == "USDT"`
- `volume_24h >= threshold`
- `listing_age_days >= threshold`
- `enabled == true`

This stage is where current metadata filters naturally belong.

### `features`

Computes per-symbol values used for later selection or weighting.

Examples:

- `momentum_20`
- `sma_fast`
- `sma_slow`
- `rsi14`
- `volatility_20`
- future composite `factor_score`

This stage is the key design change.
Dynamic per-symbol computation must become a first-class semantic stage, not an
unsupported accident.

### `selection`

Chooses which instruments survive into the tradable set.

This stage must support two retained shapes:

- metadata-ranked selection
- feature-ranked selection

Examples:

- top by `market_cap`
- top by `volume_24h`
- top by `factor_score`
- threshold-based inclusion
- signal-gated inclusion

### `weighting`

Defines how selected instruments are weighted.

Minimal retained target:

- `equal_weight`
- `rank_weight`
- `score_weight`
- `fixed_weights`

### `rebalance`

Defines rebalance timing and retained agent behavior.

Examples:

- daily
- weekly

This stage remains attached to the existing retained rebalance/agent model.

## 6. Minimal next-phase capability target

The first meaningful post-`V1` pool target should support:

- metadata-based source and eligibility
- per-symbol feature computation
- feature-ranked selection
- retained weighting
- retained rebalance

Concretely, the design target is to make this class of strategy legal:

- choose a broad metadata-constrained pool
- compute a feature or factor score for each symbol
- select top `N` by that score
- rebalance using a retained weighting helper

## 7. Explicit non-goals

This contract does **not** aim to introduce:

- SQL-like pool queries
- arbitrary joins
- panel-wide dataframe semantics
- unrestricted custom optimizer DSL
- unconstrained portfolio language design
- free-form research notebook semantics

The target is real strategy pool authoring, not a general data-processing
language.

## 8. Current-to-next-phase transition rule

Current state:

- metadata-ranked cross-sectional rotation is supported
- metadata-ranked selection plus per-symbol signal gating is supported
- dynamic factor-ranked universe construction is rejected by `QPQSLOW011`

Next-phase direction:

- `QPQSLOW011` should stop being the long-term semantic boundary
- the old helper-only restriction should be replaced by an instrument-pool model

Transition rule:

- do not reinterpret current `QPQSLOW011` behavior as already solved
- do not patch the old helper path ad hoc
- first define the new pool semantics
- then rework lowering/admission around that semantic model

## 9. Relationship to `quantscript_authoring_view`

The instrument-pool model should be visible in authoring and frontend flow.

That means future `quantscript_authoring_view` evolution should be able to show:

- pool source
- eligibility filters
- feature computation
- selection rule
- weighting rule
- rebalance stage

This does **not** mean `quantscript_authoring_view` becomes the semantic truth
source.

It means:

- pool semantics are defined by analysis/lowering
- authoring view renders those semantics in a source-first way

## 10. Minimal future artifact shape

If a derived pool artifact is added later, it should look like a lowered
summary, not a new authoring language.

Minimal shape suggestion:

```json
{
  "kind": "instrument_pool_view",
  "source": {
    "kind": "universe",
    "constraints": {
      "exchange": "binance",
      "market": "spot",
      "quote": "USDT"
    }
  },
  "eligibility_rules": [
    {"field": "volume_24h", "op": ">=", "value": 1000000000},
    {"field": "listing_age_days", "op": ">=", "value": 180}
  ],
  "feature_defs": [
    {"name": "momentum_20", "kind": "momentum", "window": 20},
    {"name": "rsi14", "kind": "rsi", "window": 14}
  ],
  "selection_rule": {
    "kind": "top_n",
    "key": "factor_score",
    "order": "desc",
    "count": 5
  },
  "weighting_rule": {
    "kind": "rank_weight",
    "method": "linear"
  },
  "rebalance_rule": {
    "every": "weekly"
  }
}
```

This is only a shape target for future design.
It is not landed behavior.

## 11. Authoring-surface rule

This contract intentionally does **not** lock the final source syntax yet.

Allowed future directions:

- evolve the current helper family into a pool-aware semantic path
- introduce a clearer pool-oriented authoring block
- support both via a retained compatibility layer

But the semantic model must come first.
Syntax should follow semantics, not the reverse.

## 12. Frontend rule

Frontend should eventually be able to show the pool pipeline as:

- `Source -> Eligibility -> Features -> Selection -> Weighting -> Rebalance`

This is the pool-specific complement to the broader strategy pipeline:

- `Data -> Intent -> Agent -> Risk -> Execution`

The pool pipeline should be readable, inspectable, and patch-oriented.

It should not require the frontend to reconstruct pool semantics from raw
strings.

## 13. Implementation rule

The next implementation phase should follow this order:

1. define pool semantics in analysis/lowering terms
2. decide admission boundary
3. update diagnostics and lowering contracts
4. then evolve frontend and authoring view
5. only after that, revise outward wording and sample strategies

Do not start by weakening diagnostics alone.
Do not start by changing frontend wording alone.
Do not start by inventing final syntax alone.

## 14. Phase-1 next-step acceptance rule

The first real step under this contract is acceptable only if it:

- replaces helper-only thinking with explicit pool semantics
- preserves formal source as the only editable truth source
- does not over-expand into a general data/query DSL
- keeps retained weighting/rebalance semantics narrow
- gives frontend enough structure to explain pool flow honestly

## 15. Current status

Current status is:

- this contract is now documented
- first internal lowering-side pool semantics are now beginning to exist as a
  derived internal model
- `quantscript_authoring_view` now exposes that internal model as a read-only
  `pool_pipeline` block for frontend display
- failed formal compile responses now best-effort emit
  `partial_artifacts.quantscript_authoring_view.pool_pipeline` when extraction
  succeeds before the outward error is returned
- current code still enforces the old retained `V1` boundary
- `QPQSLOW011` is still a real current diagnostic
- the design direction has changed, but outward admission has not yet been updated

That means:

- current metadata-ranked strategies remain valid
- current signal-gated degraded strategies remain valid
- dynamic factor-ranked pool construction is still deferred

until a later implementation phase lands this contract.

---
> 文档版本: v3.7.0 | 最后更新: 2026-05-21 | QuantPilot v3.7.0
