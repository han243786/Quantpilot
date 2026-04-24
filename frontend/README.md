# QuantPilot Frontend

React + JavaScript first-version strategy editor for browser runtime.

## Start

Normal dev mode:

```bash
cd frontend
npm install
npm run dev
```

If your browser or environment blocks `eval` because of strict Content Security Policy, use CSP-safe preview mode instead:

Terminal 1:

```bash
cd frontend
npm run build:csp:watch
```

Terminal 2:

```bash
cd frontend
npm run preview:csp
```

Then open:

- [http://localhost:4173](http://localhost:4173)

This mode avoids dev-time HMR injection and is much more stable under strict CSP.

## Included in v1

- strategy graph editor with constrained chain
- builtin node templates
- real-time graph validation
- runtime config export
- backend-connected runtime event stream
- node runtime status write-back
- backend graph save/load

## Main folders

- `src/components`: page panels and editor layout
- `src/nodes`: node card renderer
- `src/modules`: builtin module registry and schemas
- `src/graph`: graph creation, validation, compilation
- `src/store`: zustand graph store
- `src/runtime`: runtime integration helpers
