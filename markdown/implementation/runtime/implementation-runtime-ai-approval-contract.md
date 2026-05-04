# Runtime AI Approval Contract

This is the active contract source of truth for AI-assisted runtime proposals
and the approval chain. The v0.2.0 worklist tracks delivery status; this
document owns the stable safety boundary and the fields that already exist.

## Scope

Current implemented scope is Block 4 P0:

- AI proposal candidate intake
- source evidence identity
- model, prompt, and evidence hashes
- capability and permission-boundary validation
- static validation pass/fail
- append-only AI proposal ledger
- proposal read APIs
- governed AI proposal events
- frontend normalization for proposal/static-check state

Sandbox replay, human approval, conversion into Block 3 mutation proposals, and
approval-chain reporting are not implemented yet. They remain P1/P2 work.

## Safety Boundary

- AI can create candidates only when capability context is current and
  `ai_write_policy=proposal_only`.
- AI proposal creation never writes the Block 3 mutation ledger.
- AI proposal creation never schedules activation, rollback, or active
  parameter-version changes.
- Static-check failure is auditable but cannot move into sandbox replay or
  approval because those routes are not available in P0.
- Any future approval or conversion path must still pass through the Runtime
  Mutation Contract safe-window and activation boundary rules.

## Field Ownership

| Surface | Owner | Source of truth | Update rule |
|---|---|---|---|
| AI proposal request/record shape | Backend | `src/frontend_api_types.rs` | Add fields as typed structs/enums first, then update tests, frontend reader, and docs. |
| AI proposal ledger persistence | Backend | `src/runtime_persistence.rs` | AI proposal records stay separate from Block 3 mutation records. |
| AI proposal API behavior | Backend | `src/runtime_api.rs` | Candidate intake must validate capability context, AI write policy, target, actor, model identity, prompt hash, and evidence hash. |
| AI proposal event classification | Backend | `src/runtime_event_projection.rs` | Every `AIProposal*` P0 event is `system` + `key` and must pass governed envelope validation. |
| Timeline/replay projection | Backend | `src/runtime_response_mapping.rs` | AI proposal key events must stay visible in run detail and replay evidence. |
| Frontend reader contract | Frontend | `frontend/src/utils/runtimeAiProposal.js` | UI consumes normalized proposal/static-check states and disabled reasons. |
| Contract tests | Backend + frontend | `tests/api_ai_proposal.rs`, `frontend/src/utils/runtimeAiProposal.test.js` | Tests must cover allowed intake, denial, static-check failure, event envelopes, replay visibility, and reader fallback behavior. |

## Stable Status Values

- `draft`
- `submitted`
- `static_check_failed`
- `static_check_passed`
- `denied`
- `expired`

P0 create requests return either `static_check_passed` or
`static_check_failed`. `draft`, `submitted`, `denied`, and `expired` are
reserved contract states for lifecycle and future approval work.

## Governed Events

P0 recognizes these events as retained key evidence:

- `AIProposalCreated`
- `AIProposalDenied`
- `AIProposalStaticCheckPassed`
- `AIProposalStaticCheckFailed`

Each event payload must include AI proposal identity, source identity, source
evidence, target parameter, old/proposed parameter versions, model identity,
prompt hash, evidence hash, actor, reason, static-check state, and governance.

## Update Checklist

When changing AI proposal or approval behavior:

- update typed backend contracts first
- update event envelope classification for any new event type
- keep AI proposal ledger separate from mutation ledger unless an approved
  conversion explicitly creates a Block 3 mutation proposal
- update frontend reader normalization and disabled/actionable state
- update API and frontend tests for denial, static-check failure, timeline, and
  replay visibility
- update this document and the v0.2.0 worklist
- verify changed Markdown files decode as UTF-8
