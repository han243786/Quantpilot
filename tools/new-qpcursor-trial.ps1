param(
    [Parameter(Mandatory = $true)]
    [string] $TrialId,

    [Parameter(Mandatory = $true)]
    [string] $Scope,

    [Parameter(Mandatory = $true)]
    [string] $LeafPath,

    [string] $Heat = "G4",
    [string] $ModeStack = "refactor: RM.R1_old_behavior_freeze, refactor: RM.R2_code_module_dependency_inventory",
    [string] $NextAction = "",
    [string] $StopIf = "Any need to change public API, route/schema, persistence, lock owner, runtime semantics, release transition, or cross-child direct calls.",
    [string] $OutputPath = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = (& git rev-parse --show-toplevel).Trim()
Set-Location $repoRoot

$statePath = "markdown/00-matrix-governance/recursive-state.json"
if (-not (Test-Path -LiteralPath $statePath)) {
    throw "recursive state not found: $statePath"
}

$state = Get-Content -LiteralPath $statePath -Encoding UTF8 -Raw | ConvertFrom-Json
$head = (& git rev-parse --short HEAD).Trim()

if ([string]::IsNullOrWhiteSpace($NextAction)) {
    $NextAction = "Continue legacy step $($state.current_step) $($state.current_phase)."
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $safeScope = ($Scope -replace '[^A-Za-z0-9_.-]+', '-').Trim('-')
    $OutputPath = "governance-next/trials/$TrialId-$safeScope-qpcursor.md"
}

$dir = Split-Path -Parent $OutputPath
if ($dir -and -not (Test-Path -LiteralPath $dir)) {
    New-Item -ItemType Directory -Path $dir | Out-Null
}

$shortCursor = "QPC://$($state.version)/RM:R1+R2/GH:$Heat/FFT:root.contracts.runtime_support/MT:$Scope/LEAF:$LeafPath"

$content = @(
    "# QPCursor Trial ${TrialId}: $Scope",
    "",
    "> governance_next_trial: true",
    "> legacy_governance_authority: preserved",
    "> status: draft",
    "> created_from_commit: ``$head``",
    "",
    "This trial was generated from the legacy recursive cursor.",
    "",
    "## Authority Boundary",
    "",
    "| Layer | Status | Rule |",
    "| --- | --- | --- |",
    "| Legacy governance | authoritative | ``markdown/00-matrix-governance/recursive-state.json`` remains the source of truth. |",
    "| Governance next | trial wrapper | This file adds a QPCursor handoff view only. |",
    "| Conflict resolution | legacy wins | Any conflict falls back to the legacy recursive protocol. |",
    "",
    "## Short Cursor",
    "",
    "```text",
    $shortCursor,
    "```",
    "",
    "## Long Cursor",
    "",
    "| Field | Value |",
    "| --- | --- |",
    "| ``cursor_version`` | ``qpcursor-trial-v0`` |",
    "| ``cursor_id`` | ``QPC-TRIAL-$TrialId`` |",
    "| ``status`` | ``draft`` |",
    "| ``repo_baseline`` | ``master @ $head``, milestone ``$($state.version)``, legacy batch ``$($state.current_step)`` |",
    "| ``mode_stack`` | $ModeStack |",
    "| ``super_pipeline`` | legacy recursive modularization, ``$($state.protocol)``, ``$($state.latest_governance_batch)`` |",
    "| ``scope`` | ``MT=$Scope``, ``LEAF=$LeafPath`` |",
    "| ``interface_freeze`` | No public API, route, schema, persistence owner, lock owner, release transition, or sibling horizontal link change unless explicitly authorized. |",
    "| ``allowed_workset`` | Fill from legacy baseline before execution. |",
    "| ``next_action`` | $NextAction |",
    "| ``stop_if`` | $StopIf |",
    "",
    "## Heat Trigger",
    "",
    "| Signal | Result |",
    "| --- | --- |",
    "| Declared heat | $Heat |",
    "| Release transition | not active; AI must not propose |",
    "",
    "## Legacy State Mapping",
    "",
    "| Legacy field | Value |",
    "| --- | --- |",
    "| ``current_parent`` | ``$($state.current_parent)`` |",
    "| ``current_step`` | ``$($state.current_step)`` |",
    "| ``current_phase`` | ``$($state.current_phase)`` |",
    "| ``next_recommended_child`` | ``$($state.next_recommended_child)`` |",
    "",
    "## Evidence Captured",
    "",
    "| Gate | Result |",
    "| --- | --- |",
    "| Required gates | Pending |",
    "",
    "## Trial Judgment",
    "",
    "Draft generated. A developer or agent must complete allowed workset, evidence, and trial judgment before declaring handoff_ready."
)

Set-Content -LiteralPath $OutputPath -Encoding UTF8 -Value $content
Write-Host "QPCursor trial generated: $OutputPath"
