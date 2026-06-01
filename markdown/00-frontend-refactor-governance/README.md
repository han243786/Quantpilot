# Frontend Refactor Governance

Status: parallel isolated governance line.

This directory is the frontend-only governance root for the parallel refactor. It is intentionally separated from `markdown/00-matrix-governance`, `markdown/10-overview`, and backend milestone logs so frontend work can progress without colliding with backend recursion.

## Isolation Contract

- Frontend recursion writes frontend code and files under this directory first.
- Do not update global module tree, global full feature tree, global roadmap, or backend milestone logs during frontend isolated recursion.
- The files in this directory start from frontend-local truth, not copied content. They are empty-copy replacements for high-conflict global documents.
- Global tree merge happens only after frontend refactor closeout, with an explicit merge-back record.
- Release-transition shortcuts still require an explicit developer decision. AI must not proactively propose release-transition coupling.

## Local File Map

| File | Purpose |
| --- | --- |
| `frontend-process-matrix.md` | Frontend-specific recursive workflow and gates. |
| `frontend-standard-matrix.md` | Frontend hard rules, whitebox node contract, and split rules. |
| `frontend-guidance-matrix.md` | Frontend navigation map and candidate parent modules. |
| `frontend-module-tree.md` | Frontend-only module tree, initialized from blank frontend state. |
| `frontend-full-feature-tree.md` | Frontend-only full feature tree, initialized from blank frontend state. |
| `frontend-recursive-state.json` | Local recursion cursor and forbidden global hot files. |
| `frontend-recursive-speed-protocol.md` | Speed rules derived from backend experience. |
| `frontend-proposal-flow.md` | Frontend proposal, fit check, refinement, and design landing flow. |
| `frontend-closeout-and-merge-protocol.md` | Rules for final merge back into global governance. |

## Merge-Back Principle

When frontend refactor is fully closed, freeze the frontend-local module tree and full feature tree, generate a delta report, then merge into the global trees in a dedicated integration step.
