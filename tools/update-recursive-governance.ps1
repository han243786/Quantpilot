param(
    [Parameter(Mandatory = $true)]
    [int] $Number,

    [Parameter(Mandatory = $true)]
    [string] $FileSlug,

    [Parameter(Mandatory = $true)]
    [string] $BatchId,

    [Parameter(Mandatory = $true)]
    [string] $NodeId,

    [Parameter(Mandatory = $true)]
    [ValidateSet("baseline_plan", "extract_closeout", "single_leaf_closeout", "parent_residual_judgment", "governance")]
    [string] $StageType,

    [Parameter(Mandatory = $true)]
    [string] $Summary,

    [Parameter(Mandatory = $true)]
    [string] $NextStep,

    [string] $ParentNode = "",
    [string[]] $RealFiles = @(),
    [string[]] $Markers = @(),
    [string[]] $GateCommands = @(),
    [switch] $Apply
)

$ErrorActionPreference = "Stop"

function U {
    param([string] $Base64)
    return [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($Base64))
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$milestoneDir = Join-Path $repoRoot "markdown/06-milestones/v4.16.0"
$landingRecordName = U "MDIt6JC95Zyw6K6w5b2VLm1k"
$landingRecordPath = Join-Path $milestoneDir $landingRecordName
$milestoneReadmePath = Join-Path $repoRoot "markdown/06-milestones/README.md"
$docsIndexPath = Join-Path $repoRoot "markdown/10-overview/overview-docs-index.md"
$currentStatusPath = Join-Path $repoRoot "markdown/10-overview/overview-current-status-and-roadmap.md"
$fullTreePath = Join-Path $repoRoot "markdown/10-overview/overview-full-feature-tree.md"
$moduleTreePath = Join-Path $repoRoot "markdown/00-matrix-governance/module-tree.md"

function Write-Utf8NoBom {
    param(
        [string] $Path,
        [string] $Content
    )
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $encoding)
}

function Append-Line {
    param(
        [string] $Path,
        [string] $Line
    )
    $content = [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8)
    if (-not $content.EndsWith("`n")) {
        $content += "`n"
    }
    $content += $Line + "`n"
    Write-Utf8NoBom $Path $content
}

function Get-RelativePath {
    param([string] $Path)
    return $Path.Substring($repoRoot.Length + 1).Replace("\", "/")
}

function Get-NextLandingNumber {
    $content = [System.IO.File]::ReadAllText($landingRecordPath, [System.Text.Encoding]::UTF8)
    $matches = [regex]::Matches($content, '(?m)^(\d+)\.\s+')
    if ($matches.Count -eq 0) {
        return 1
    }
    return ([int]$matches[$matches.Count - 1].Groups[1].Value) + 1
}

function New-DecisionGateBlock {
    if ($StageType -ne "single_leaf_closeout" -and $StageType -ne "parent_residual_judgment") {
        return ""
    }

    return @"
## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | TBD | TBD |
| parent_child_communication_kept | TBD | TBD |
| equivalence_baseline_freezable | TBD | TBD |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | TBD | TBD |
| state_machine_phase | TBD | TBD |
| strategy_branch | TBD | TBD |
| independent_failure_mode | TBD | TBD |
| reuse_pressure | TBD | TBD |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | TBD | TBD |
| communication_cost_rises | TBD | TBD |
| local_proof_missing | TBD | TBD |
| line_count_only | TBD | TBD |

leaf_split_decision_result

TBD

next_recursive_step

$NextStep

"@
}

function New-MilestoneContent {
    $realFileLines = if ($RealFiles.Count -gt 0) {
        ($RealFiles | ForEach-Object { "- ``$_``" }) -join "`n"
    } else {
        "- TBD"
    }
    $markerLines = if ($Markers.Count -gt 0) {
        ($Markers | ForEach-Object { "- ``$_``" }) -join "`n"
    } else {
        "- ``$BatchId``"
    }
    $gateLines = if ($GateCommands.Count -gt 0) {
        ($GateCommands | ForEach-Object { "- ``$_``" }) -join "`n"
    } else {
        @(
            "- ``git diff --check``",
            "- ``powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1``",
            "- ``powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1``",
            "- ``powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1``"
        ) -join "`n"
    }
    $decisionGate = New-DecisionGateBlock
    $parentLine = if ($ParentNode) { $ParentNode } else { "TBD" }

    return @"
# v4.16.0 $Summary

> Batch: $BatchId
> Node: ``$NodeId``
> Parent: ``$parentLine``
> Stage: ``$StageType``
> Movement: TBD.

---

## Summary

$Summary

$decisionGate## Boundary

**Real files**:
$realFileLines

**Markers**:
$markerLines

**Next step**:
$NextStep

---

## Gates

$gateLines
"@
}

$fileName = "$Number-$FileSlug.md"
$targetPath = Join-Path $milestoneDir $fileName
$relativeTarget = Get-RelativePath $targetPath

$plannedUpdates = @(
    $relativeTarget,
    ("markdown/06-milestones/v4.16.0/" + $landingRecordName),
    "markdown/06-milestones/README.md",
    "markdown/10-overview/overview-docs-index.md",
    "markdown/10-overview/overview-current-status-and-roadmap.md",
    "markdown/10-overview/overview-full-feature-tree.md",
    "markdown/00-matrix-governance/module-tree.md"
)

if (-not $Apply) {
    Write-Host "Recursive governance update preview:"
    foreach ($path in $plannedUpdates) {
        Write-Host " - $path"
    }
    Write-Host "Pass -Apply to write files."
    exit 0
}

if (Test-Path -LiteralPath $targetPath) {
    throw "Target milestone file already exists: $relativeTarget"
}

Write-Utf8NoBom $targetPath (New-MilestoneContent)

$landingNumber = Get-NextLandingNumber
$backtick = [char]96
$nodeCode = "{0}{1}{0}" -f $backtick, $NodeId
$targetCode = "{0}{1}{0}" -f $backtick, $relativeTarget
$addText = U "5paw5aKe"
$nextText = U "5LiL5LiA5q2l"
$boundaryText = U "6YCS5b2S6L6555WM6KGl5YWF"
$latestStatusText = U "5pyA5paw54q25oCB6KGl5YWF"
$semiText = U "77yb"
$periodText = U "44CC"
Append-Line $landingRecordPath ("{0}. {1} {2} {3}{4}{5}: {6}{7}" -f $landingNumber, $BatchId, $nodeCode, $Summary, $semiText, $nextText, $NextStep, $periodText)
Append-Line $milestoneReadmePath ("| v4.16.0 / {0} | {1}: {2} {3} |" -f $BatchId, $addText, $nodeCode, $Summary)
Append-Line $docsIndexPath ("| v4.16.0 / {0} | {1}: {2} {3} |" -f $BatchId, $addText, $nodeCode, $Summary)
Append-Line $currentStatusPath ("- {0} {1} {2}{3}{4}: {5}{6}" -f $BatchId, $nodeCode, $Summary, $semiText, $nextText, $NextStep, $periodText)
Append-Line $fullTreePath ("{0}: {1} {2} {3}{4}{5}: {6}{7}" -f $boundaryText, $BatchId, $nodeCode, $Summary, $semiText, $nextText, $NextStep, $periodText)
Append-Line $fullTreePath ("- {0} - v4.16.0 {1} {2}" -f $targetCode, $BatchId, $Summary)
Append-Line $moduleTreePath ("**{0}({1})**: {2} {3}{4}{5}: {6}{7}" -f $latestStatusText, $BatchId, $nodeCode, $Summary, $semiText, $nextText, $NextStep, $periodText)

Write-Host "Recursive governance update written:"
foreach ($path in $plannedUpdates) {
    Write-Host " - $path"
}
