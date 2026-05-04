# P2 Responsive Visual Review

Date: 2026-04-28  
Method: Playwright + Edge  
Viewport set: `1280`, `1024`, `768`, `560`

## Scope

- Strategy hub
- Strategy workspace
- Backtest detail page
- Backtest compare page

## Screenshot Set

### Strategy Hub

- [strategy-hub-1280.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-hub-1280.png)
- [strategy-hub-1024.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-hub-1024.png)
- [strategy-hub-768.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-hub-768.png)
- [strategy-hub-560.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-hub-560.png)

### Strategy Workspace

- [strategy-workspace-1280.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-workspace-1280.png)
- [strategy-workspace-1024.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-workspace-1024.png)
- [strategy-workspace-768.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-workspace-768.png)
- [strategy-workspace-560.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/strategy-workspace-560.png)

### Backtest Detail

- [backtest-detail-1280.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-detail-1280.png)
- [backtest-detail-1024.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-detail-1024.png)
- [backtest-detail-768.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-detail-768.png)
- [backtest-detail-560.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-detail-560.png)

### Backtest Compare

- [backtest-compare-1280.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-compare-1280.png)
- [backtest-compare-1024.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-compare-1024.png)
- [backtest-compare-768.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-compare-768.png)
- [backtest-compare-560.png](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/visual-review/p2-responsive/backtest-compare-560.png)

## Findings

- The review-only script now targets the current route model. `/strategies`
  is the strategy hub, and `/strategies/visual_review_graph` is the strategy
  workspace.
- The screenshot fixture now covers graph index, graph versions, graph audit,
  runtime history, backtest history, and experiment history requests. The
  review run completed with no unexpected API calls.
- The review script now freezes animation and uses reduced-motion media before
  capturing screenshots. This keeps review screenshots focused on layout
  truth instead of transient overlay or animation state.
- Each screenshot now runs in a fresh browser context so the compare page
  cannot inherit route, animation, or overlay state from an earlier capture.
- Backtest compare screenshots no longer capture a stale darkened transition
  state.
- Backtest compare now keeps page-level decorative pseudo-elements below the
  content stacking layer, so visual review screenshots do not show a residual
  dark mask over the page.
- Backtest detail now uses the event stream detail-mode layout, so the event
  feed flows below the chart on narrow viewports instead of overlapping it.
- Event stream kicker labels now render readable Chinese text instead of
  visible JSX Unicode escape literals.
- This review artifact is screenshot evidence, not a promoted visual-diff gate.
  The canonical closeout wrapper still treats this spec as opt-in.

## Outcome

- `VISUAL_REVIEW=1 cmd /c npx playwright test tests/e2e/visual-responsive-review.spec.js`
  passed on 2026-04-28.
- The previous stale editor-route screenshot references were removed.
- Remaining visual polish concerns, including strategy hub and strategy
  workspace density, remain review-only until explicitly pulled into an
  implementation batch.
