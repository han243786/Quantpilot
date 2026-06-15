# check-topology-runtime-governance.ps1
# Validates QuantPilot's lightweight L4 topology-governance runtime layer.

$ErrorActionPreference = "Stop"

$RootPath = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
    param([string] $Message)
    $script:Failures.Add($Message) | Out-Null
}

function Join-Root {
    param([string] $Path)
    return Join-Path $RootPath $Path
}

function Read-RepoText {
    param([string] $Path)
    return Get-Content -LiteralPath (Join-Root $Path) -Raw -Encoding UTF8
}

function Assert-FileExists {
    param([string] $Path)
    if (-not (Test-Path -LiteralPath (Join-Root $Path) -PathType Leaf)) {
        Add-Failure "Missing required file: $Path"
    }
}

function Assert-TextContains {
    param(
        [string] $Path,
        [string] $Needle,
        [string] $Description
    )
    if (-not (Test-Path -LiteralPath (Join-Root $Path) -PathType Leaf)) {
        Add-Failure "Cannot check missing file: $Path"
        return
    }
    $content = Read-RepoText $Path
    if (-not $content.Contains($Needle)) {
        Add-Failure "$Path missing $Description"
    }
}

function Get-YamlScalar {
    param(
        [string] $Content,
        [string] $Name
    )
    $match = [regex]::Match($Content, "(?m)^\s*$([regex]::Escape($Name)):\s*(?<value>.+?)\s*$")
    if (-not $match.Success) {
        return ""
    }
    return $match.Groups["value"].Value.Trim().Trim('"')
}

function Assert-YamlField {
    param(
        [string] $Content,
        [string] $Field,
        [string] $Path
    )
    if (-not ([regex]::IsMatch($Content, "(?m)^\s*$([regex]::Escape($Field))\s*:"))) {
        Add-Failure "$Path missing YAML field: $Field"
    }
}

function Assert-Ledger {
    param([string] $Path)
    $fullPath = Join-Root $Path
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        Add-Failure "Missing ledger: $Path"
        return
    }

    $lines = @(Get-Content -Encoding UTF8 -LiteralPath $fullPath | Where-Object { $_.Trim() -ne "" })
    if ($lines.Count -eq 0) {
        Add-Failure "$Path must contain at least one ledger row"
        return
    }

    $lineNo = 0
    foreach ($line in $lines) {
        $lineNo += 1
        try {
            $record = $line | ConvertFrom-Json
        } catch {
            Add-Failure "$Path line $lineNo is not valid JSON: $($_.Exception.Message)"
            continue
        }

        foreach ($field in @("cursor_id", "mode", "parent", "nodes_changed", "edges_changed", "gates", "result", "next")) {
            if (-not ($record.PSObject.Properties.Name -contains $field)) {
                Add-Failure "$Path line $lineNo missing field: $field"
            }
        }

        if (($record.PSObject.Properties.Name -contains "mode") -and $record.mode -notin @("refactor", "advance", "aspect_polish", "doc_debt_cleanup")) {
            Add-Failure "$Path line $lineNo invalid mode: $($record.mode)"
        }
        if (($record.PSObject.Properties.Name -contains "result") -and $record.result -notin @("active", "closed", "closed_with_debt", "blocked", "needs_mode_jump")) {
            Add-Failure "$Path line $lineNo invalid result: $($record.result)"
        }
    }
}

$requiredFiles = @(
    "markdown/00-matrix-governance/work-mode-routing.md",
    "markdown/00-matrix-governance/governance-quality-speed-guard.md",
    "markdown/00-matrix-governance/current-work-cursor.yaml",
    "markdown/00-matrix-governance/topology-ledger.ndjson",
    "markdown/00-matrix-governance/aspect-cutover-record.md",
    "markdown/00-matrix-governance/release-transition-exception.md",
    "markdown/00-matrix-governance/aspect-polish-protocol.md",
    "markdown/00-matrix-governance/release-transition-protocol.md"
)

foreach ($file in $requiredFiles) {
    Assert-FileExists $file
}

Assert-TextContains "markdown/00-matrix-governance/work-mode-routing.md" "work-mode-routing-v1.2" "four-entry mode routing version"
Assert-TextContains "markdown/00-matrix-governance/work-mode-routing.md" "doc_debt_cleanup" "doc debt cleanup work entry"
Assert-TextContains "markdown/00-matrix-governance/governance-quality-speed-guard.md" "doc_debt_cleanup" "quality guard work mode enum"
Assert-TextContains "markdown/00-matrix-governance/aspect-polish-protocol.md" "aspect-cutover-record.md" "aspect cutover runtime record link"
Assert-TextContains "markdown/00-matrix-governance/release-transition-protocol.md" "release-transition-exception.md" "release exception runtime record link"

$cursorPath = "markdown/00-matrix-governance/current-work-cursor.yaml"
if (Test-Path -LiteralPath (Join-Root $cursorPath) -PathType Leaf) {
    $cursor = Read-RepoText $cursorPath
    foreach ($field in @("cursor_id", "status", "work_mode_stack", "topology_slice", "parent_node", "allowed_workset", "forbidden_operations", "gates", "closeout_required", "next_action")) {
        Assert-YamlField $cursor $field $cursorPath
    }
    $status = Get-YamlScalar $cursor "status"
    if ($status -notin @("active", "blocked", "closed", "superseded")) {
        Add-Failure "$cursorPath status must be active / blocked / closed / superseded"
    }
    $parent = Get-YamlScalar $cursor "parent_node"
    if ($parent -eq "") {
        Add-Failure "$cursorPath parent_node must be filled"
    }
    if (-not $cursor.Contains("root.docs.matrix_governance")) {
        Add-Failure "$cursorPath must anchor this patch to root.docs.matrix_governance"
    }
}

Assert-Ledger "markdown/00-matrix-governance/topology-ledger.ndjson"

$exceptionPath = "markdown/00-matrix-governance/release-transition-exception.md"
if (Test-Path -LiteralPath (Join-Root $exceptionPath) -PathType Leaf) {
    $exception = Read-RepoText $exceptionPath
    foreach ($needle in @("exception_id", "approved_by", "performance_evidence", "direct_edge_added", "rollback", "review_date", "status: none")) {
        if (-not $exception.Contains($needle)) {
            Add-Failure "$exceptionPath missing release exception field or default: $needle"
        }
    }
}

$cutoverPath = "markdown/00-matrix-governance/aspect-cutover-record.md"
if (Test-Path -LiteralPath (Join-Root $cutoverPath) -PathType Leaf) {
    $cutover = Read-RepoText $cutoverPath
    foreach ($needle in @("source_slice", "mirror_slice", "frozen_interfaces", "rollback_plan", "cutover_gate", "cutover_result")) {
        if (-not $cutover.Contains($needle)) {
            Add-Failure "$cutoverPath missing cutover field: $needle"
        }
    }
}

if ($Failures.Count -gt 0) {
    Write-Host "Topology runtime governance check FAILED:"
    foreach ($failure in $Failures) {
        Write-Host " - $failure"
    }
    exit 1
}

Write-Host "Topology runtime governance check passed."
exit 0
