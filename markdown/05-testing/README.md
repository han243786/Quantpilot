# Testing Surface

> Scope: active testing documents only.
> Updated: 2026-06-13.

This directory keeps only testing documents that still participate in current governance, recursive refactor baselines, or active test evidence collection.

Active files:

| File | Current role |
|------|--------------|
| `meta-pipeline-log.md` | Meta-pipeline evidence log referenced by the governance process. |
| `手动全量实机测试检查单.md` | Manual smoke coverage still referenced by v4.16 runtime/report baselines. |

Historical audit reports, old testing plans, retired matrices, and obsolete latest-style reports live under `markdown/09-archive/testing-retired/`.

Old ignored screenshot and JSON report artifacts from the retired testing artifact surface are covered by `markdown/09-archive/testing-artifacts-retired/README.md`. Those local artifact directories may be cleaned when they are outside the active evidence surface.

Deadwood cleanup record:

- 2026-06-13: removed 68 tracked PNG files from `markdown/05-testing/screenshots/` because they were retired generated evidence, had no active Markdown/code references, and are covered by the retired testing artifact policy.
- Future screenshots for ad-hoc local checks must stay ignored local artifacts unless a current milestone explicitly promotes one into a tracked evidence document.
