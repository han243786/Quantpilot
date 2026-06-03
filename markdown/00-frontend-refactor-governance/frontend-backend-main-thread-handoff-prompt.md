# Backend Main Thread Handoff Prompt

Status: prepared for manual paste into the backend refactor main process.

Use this prompt only after the backend process is ready to receive frontend closeout context.

```text
Backend main refactor process: frontend isolated recursive refactor is closed and ready for later integration.

Current frontend state:
- Latest frontend prep record: FE-0224.
- Frontend state file: markdown/00-frontend-refactor-governance/frontend-recursive-state.json.
- Frontend state is closed: current_parent=root.frontend, current_child_queue=[], next_parent=null.
- Frontend module tree is closed locally: markdown/00-frontend-refactor-governance/frontend-module-tree.md.
- Frontend full feature supplement is closed locally: markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md.

Do not merge global governance yet unless backend closeout has explicitly opened the integration step.

When backend closeout is ready, use these frontend prep files:
- markdown/00-frontend-refactor-governance/frontend-global-merge-back-map.md
- markdown/00-frontend-refactor-governance/frontend-e2e-current-state-inventory.md
- markdown/00-frontend-refactor-governance/frontend-backend-main-thread-handoff-prompt.md
- markdown/00-frontend-refactor-governance/records/FE-0224-frontend-global-e2e-preflight-handoff.md

Frontend deferred items:
- Global governance merge-back waits for backend closeout.
- E2E spec-body cleanup waits for backend route and module ownership to stabilize.

Required integration discipline:
- Keep frontend-local docs as source evidence, not as already-merged global truth.
- Do not edit E2E spec bodies during backend-only recursion.
- Do not introduce release-transition shortcuts unless the developer explicitly opens release-transition work.
- After backend closeout, perform a dedicated global integration pass with governance checks, frontend build/test, backend checks, and selected E2E smoke.
```
