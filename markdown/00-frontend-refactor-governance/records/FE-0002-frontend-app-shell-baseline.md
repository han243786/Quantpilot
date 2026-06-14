# FE-0002 Frontend App Shell Baseline

Status: parent baseline established. No code extraction in this record.

## Parent Node

`frontend.app_shell`

## Current Owned Files

- `frontend/src/main.jsx`
- `frontend/src/App.jsx`

## Public Inputs

- Root DOM element with id `root`.
- Browser `window`, `document`, `navigator`, `location`, `history`, and lifecycle events.
- Optional Tauri window runtime from `@tauri-apps/api/window`.
- `useGraphStore.initialize`, `useGraphStore.capabilityStatus`, and `refreshGraphIndex`.
- Router functions: `parseRoute`, `navigateTo`, and `strategiesPath`.
- I18n provider and `useI18n`.
- Tutorial state from `useTutorial`.
- Lazy-loaded page modules and shell components.

## Public Outputs

- React app mounted under `React.StrictMode`.
- `I18nProvider` wrapped app tree.
- Global unhandled rejection logging.
- Loading shell until initialization finishes or user skips.
- App titlebar when Tauri runtime exists.
- Left sidebar, offline banner, storage quota banner, skip link, main content region, tutorial overlay, command palette, and toast container.
- Route-driven page content wrapped in `ErrorBoundary`.
- Browser navigation updates for root-to-strategies redirect and popstate changes.

## Child Leaf Queue

| Leaf | Current Responsibility | First Extraction Bias |
| --- | --- | --- |
| `frontend.app_shell.bootstrap_root` | `main.jsx`, providers, test bridge, global style imports, unhandled rejection hook. | Extract root bootstrap wrapper while keeping entrypoint stable. |
| `frontend.app_shell.startup_readiness` | Store initialization, loading fallback, force-ready behavior. | Move fallback/readiness into app shell leaf. |
| `frontend.app_shell.environment_events` | Theme, online/offline, storage quota, visibility refresh, beforeunload, command palette shortcut. | Group global browser event wiring behind a hook. |
| `frontend.app_shell.desktop_window_chrome` | Tauri window resolution, maximized state, titlebar controls. | Isolate desktop runtime boundary. |
| `frontend.app_shell.route_host` | Route state, route content selection, lazy page loading, per-route error boundary. | Prepare handoff to `frontend.routing`. |
| `frontend.app_shell.global_overlays` | Tutorial overlay, command palette, toast container. | Keep overlays as shell children with narrow props. |

## Boundary Handoffs

- `frontend.routing` owns route parsing, path builders, and navigation primitives.
- Page parents own page internals and lazy-loaded feature content.
- `frontend.store` owns graph store actions and persistence semantics.
- `frontend.design_system_styles` owns global style policy after it is extracted.

## Equivalence Anchor

- `npm.cmd test -- router.test.js`: passed, 5 tests.
- `npm.cmd run build`: passed, 991 modules transformed.

## Split Decision

Further split is required. `App.jsx` currently mixes bootstrapping, route hosting, desktop runtime, browser event wiring, loading readiness, and overlay orchestration. The first code leaf should be `frontend.app_shell.bootstrap_root` because it is small, stable, and gives the frontend refactor a concrete code home without changing route/page behavior.
