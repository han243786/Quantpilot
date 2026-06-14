# Frontend Closeout And Merge Protocol

Status: frontend-only finalization protocol.

## Leaf Closeout

Each leaf closeout must record:

- Final owner node.
- Files owned or intentionally left behind.
- Public inputs and outputs.
- Behavior equivalence anchor.
- Tests or verification command.
- Residual risks.
- Further-split decision.

## Parent Closeout

A parent closes only when:

- All child leaves are closed or explicitly deferred.
- Parent boundary is stable.
- No sibling development coupling remains.
- Local module tree and full feature tree are updated.
- Residuals are listed for the next parent or merge-back phase.

## Merge Back To Global Governance

Merge-back is a separate integration phase after frontend refactor closes.

Required steps:

1. Freeze frontend-local module tree and full feature tree.
2. Produce a frontend-to-global delta report.
3. Reconcile node ids with global naming.
4. Update global module tree and global full feature tree once.
5. Update global docs index and roadmap if needed.
6. Mark frontend isolated governance as merged or superseded.

Do not perform merge-back without explicit user direction.
