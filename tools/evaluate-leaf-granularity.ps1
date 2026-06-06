param(
    [Parameter(Mandatory = $true)]
    [string[]] $Path,

    [string] $LeafId = "",
    [string] $ParentId = "",
    [int] $ParentLoc = 0,
    [int] $Depth = 0,
    [int] $ExpectedMilestoneCount = 2,
    [string[]] $RiskTag = @(),
    [switch] $SameParentWaveCandidate,
    [switch] $AsJson,
    [int] $SplitBenefitOverride = -1,
    [int] $LeafSizeFitOverride = -1,
    [int] $RiskPenaltyOverride = -1,
    [int] $GovernanceCostOverride = -1,
    [int] $SystemEfficiencyPenaltyOverride = -1
)

$ErrorActionPreference = "Stop"

function Clamp-Score {
    param([double] $Value)
    return [int][Math]::Min(100, [Math]::Max(0, [Math]::Round($Value)))
}

function Resolve-RepoPath {
    param([string] $InputPath)
    $resolved = Resolve-Path -LiteralPath $InputPath -ErrorAction Stop
    return $resolved.Path
}

function Count-Matches {
    param(
        [string[]] $Lines,
        [string] $Pattern
    )
    return @($Lines | Where-Object { $_ -match $Pattern }).Count
}

function Get-CodeLines {
    param([string[]] $Lines)
    return @(
        $Lines | Where-Object {
            $trim = $_.Trim()
            if ($trim.Length -eq 0) { return $false }
            if ($trim.StartsWith("//")) { return $false }
            if ($trim.StartsWith("#")) { return $false }
            if ($trim.StartsWith("/*")) { return $false }
            if ($trim.StartsWith("*")) { return $false }
            return $true
        }
    )
}

function Get-LeafSizeFitScore {
    param([int] $Loc)
    if ($Loc -lt 100) { return 0 }
    if ($Loc -lt 150) { return 15 }
    if ($Loc -le 600) { return 35 }
    if ($Loc -le 800) { return 65 }
    return 90
}

function Get-SplitBenefitScore {
    param(
        [int] $Loc,
        [int] $FunctionCount,
        [int] $BranchCount,
        [int] $DomainCount,
        [int] $PublicSurface,
        [int] $ParentLoc
    )

    $score = 0
    if ($FunctionCount -ge 20) { $score += 25 }
    elseif ($FunctionCount -ge 10) { $score += 15 }
    elseif ($FunctionCount -ge 5) { $score += 8 }

    $branchDensity = 0.0
    if ($Loc -gt 0) {
        $branchDensity = $BranchCount / [double]$Loc
    }
    if ($branchDensity -ge 0.12) { $score += 25 }
    elseif ($branchDensity -ge 0.06) { $score += 15 }
    elseif ($branchDensity -ge 0.03) { $score += 8 }

    if ($DomainCount -ge 4) { $score += 20 }
    elseif ($DomainCount -ge 3) { $score += 12 }
    elseif ($DomainCount -ge 2) { $score += 6 }

    if ($PublicSurface -ge 10) { $score += 15 }
    elseif ($PublicSurface -ge 5) { $score += 8 }

    if ($ParentLoc -gt 0 -and $Loc -gt 0) {
        $ratio = $Loc / [double]$ParentLoc
        if ($ratio -ge 0.30) { $score += 15 }
        elseif ($ratio -ge 0.15) { $score += 8 }
    }

    return Clamp-Score $score
}

function Get-RiskScore {
    param(
        [string] $Text,
        [string[]] $RiskTag
    )

    $riskWeights = @{
        public_api = 20
        route_schema = 25
        persistence = 25
        lock = 25
        state_machine = 25
        trading_semantics = 30
        security = 30
        live_execution = 35
        compiler_contract = 30
        cross_crate = 20
        release_transition = 40
    }

    $autoPatterns = @{
        public_api = '\bpub\b|\bpublic\b|\bexport\b|\btrait\b'
        route_schema = 'route|handler|schema|openapi|dto|request|response'
        persistence = 'persist|storage|database|file|snapshot|cache'
        lock = 'Mutex|RwLock|lock|atomic|Arc<'
        state_machine = 'state_machine|transition|phase|state'
        trading_semantics = 'order|fill|execution|risk|portfolio|position|trade'
        security = 'credential|secret|signature|token|auth|encrypt|decrypt'
        live_execution = '\b(live|actual|submit|place_order|placeOrder)\b'
        compiler_contract = '\b(compile|compiler|lower|lowering|ir|contract|schema)\b|CoreIr|CoreStrategyIr'
        cross_crate = 'qrpc_core|qrpc_core_ir|crate::|pub\(crate\)'
        release_transition = 'release transition|release-transition'
    }

    $found = New-Object System.Collections.Generic.HashSet[string]
    foreach ($tag in $RiskTag) {
        if ($riskWeights.ContainsKey($tag)) {
            [void]$found.Add($tag)
        }
    }
    foreach ($key in $autoPatterns.Keys) {
        if ($Text -match $autoPatterns[$key]) {
            [void]$found.Add($key)
        }
    }

    $score = 0
    foreach ($key in $found) {
        $score += $riskWeights[$key]
    }
    return @{
        score = Clamp-Score $score
        tags = @($found | Sort-Object)
    }
}

function Get-GovernanceCostScore {
    param(
        [int] $Loc,
        [int] $FunctionCount,
        [int] $PublicSurface,
        [int] $Depth,
        [int] $ExpectedMilestoneCount
    )

    $score = 20
    if ($Loc -lt 100) { $score += 25 }
    elseif ($Loc -lt 200) { $score += 15 }
    if ($FunctionCount -le 2) { $score += 15 }
    elseif ($FunctionCount -le 4) { $score += 8 }
    if ($PublicSurface -ge 10) { $score += 15 }
    elseif ($PublicSurface -ge 5) { $score += 8 }
    if ($Depth -ge 5) { $score += 15 }
    elseif ($Depth -ge 3) { $score += 8 }
    $score += [Math]::Min(20, [Math]::Max(0, $ExpectedMilestoneCount) * 5)
    return Clamp-Score $score
}

function Get-SystemEfficiencyPenalty {
    param(
        [int] $Loc,
        [int] $FunctionCount,
        [int] $PublicSurface,
        [int] $ReexportCount,
        [int] $ForwardingCount
    )

    $score = 0
    if ($Loc -lt 120 -and $PublicSurface -gt 2) { $score += 25 }
    if ($FunctionCount -le 2 -and $Loc -lt 80) { $score += 25 }
    if ($ReexportCount -gt $FunctionCount -and $ReexportCount -gt 0) { $score += 25 }
    if ($ForwardingCount -ge 3 -and $Loc -lt 200) { $score += 15 }
    return Clamp-Score $score
}

$resolvedPaths = @($Path | ForEach-Object { Resolve-RepoPath $_ })
$allLines = New-Object System.Collections.Generic.List[string]
foreach ($file in $resolvedPaths) {
    foreach ($line in (Get-Content -LiteralPath $file -Encoding UTF8)) {
        $allLines.Add($line) | Out-Null
    }
}

$codeLines = Get-CodeLines @($allLines)
$text = [string]::Join("`n", $codeLines)
$loc = $codeLines.Count
$functionCount = Count-Matches $codeLines '^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn\s+|^\s*(export\s+)?function\s+|^\s*def\s+|^\s*(const|let|var)\s+\w+\s*=\s*(async\s*)?(\([^)]*\)|[A-Za-z0-9_]+)\s*=>'
$publicSurface = Count-Matches $codeLines '\bpub(\([^)]*\))?\b|\bpublic\b|\bexport\b'
$branchCount = Count-Matches $codeLines '\b(if|else\s+if|match|for|while|loop|switch|case|try|catch|when)\b|&&|\|\|'
$reexportCount = Count-Matches $codeLines '^\s*pub\s+use\b|^\s*export\s+\*|::\*'
$forwardingCount = Count-Matches $codeLines '^\s*(return\s+)?[A-Za-z0-9_:]+::[A-Za-z0-9_]+\(|=>\s*[A-Za-z0-9_.:]+\('

$domainPatterns = @{
    route = 'route|handler|endpoint'
    schema = 'schema|dto|contract'
    cache = 'cache'
    persistence = 'persist|storage|file|snapshot'
    lock = 'Mutex|RwLock|lock|atomic'
    transport = 'http|fetch|request|response'
    parsing = 'parse|parser|payload'
    normalization = 'normalize|normalization'
    mock = 'mock|fixture|test'
    config = 'config'
    state = 'state|machine|transition'
    compiler = 'compile|lower|ir'
}
$domains = New-Object System.Collections.Generic.List[string]
foreach ($key in $domainPatterns.Keys) {
    if ($text -match $domainPatterns[$key]) {
        $domains.Add($key) | Out-Null
    }
}

$risk = Get-RiskScore -Text $text -RiskTag $RiskTag
$splitBenefit = Get-SplitBenefitScore -Loc $loc -FunctionCount $functionCount -BranchCount $branchCount -DomainCount $domains.Count -PublicSurface $publicSurface -ParentLoc $ParentLoc
$leafSizeFit = Get-LeafSizeFitScore -Loc $loc
$riskPenalty = [int]$risk.score
$governanceCost = Get-GovernanceCostScore -Loc $loc -FunctionCount $functionCount -PublicSurface $publicSurface -Depth $Depth -ExpectedMilestoneCount $ExpectedMilestoneCount
$systemEfficiencyPenalty = Get-SystemEfficiencyPenalty -Loc $loc -FunctionCount $functionCount -PublicSurface $publicSurface -ReexportCount $reexportCount -ForwardingCount $forwardingCount

if ($SplitBenefitOverride -ge 0) { $splitBenefit = Clamp-Score $SplitBenefitOverride }
if ($LeafSizeFitOverride -ge 0) { $leafSizeFit = Clamp-Score $LeafSizeFitOverride }
if ($RiskPenaltyOverride -ge 0) { $riskPenalty = Clamp-Score $RiskPenaltyOverride }
if ($GovernanceCostOverride -ge 0) { $governanceCost = Clamp-Score $GovernanceCostOverride }
if ($SystemEfficiencyPenaltyOverride -ge 0) { $systemEfficiencyPenalty = Clamp-Score $SystemEfficiencyPenaltyOverride }

$weightedDelta =
    (0.40 * $splitBenefit) +
    (0.20 * $leafSizeFit) -
    (0.20 * $riskPenalty) -
    (0.15 * $governanceCost) -
    (0.05 * $systemEfficiencyPenalty)
$normalizedScore = Clamp-Score (40 + $weightedDelta)

$highRisk = $riskPenalty -ge 60
$helperOnly = ($loc -lt 100 -and $functionCount -le 2 -and $publicSurface -le 2)

$decision = "STOP"
$reason = "low split score or terminal-sized cohesive leaf"
if ($helperOnly -and $splitBenefit -lt 80) {
    $decision = "STOP"
    $reason = "helper-only or micro leaf; governance and communication cost exceed split value"
} elseif ($normalizedScore -ge 65 -and $highRisk) {
    $decision = "PRECISION"
    $reason = "strong split pressure with high-risk surface"
} elseif ($normalizedScore -ge 65) {
    $decision = "SPLIT"
    $reason = "strong split pressure and risk is not high enough to force precision mode"
} elseif ($normalizedScore -ge 40 -and $SameParentWaveCandidate) {
    $decision = "WAVE"
    $reason = "medium split pressure; handle only as same-parent wave"
} elseif ($normalizedScore -ge 40) {
    $decision = "STOP"
    $reason = "medium score without same-parent wave candidate; avoid standalone governance"
}

$action = switch ($decision) {
    "STOP" { "Set stop_split: true unless developer supplies stronger ownership evidence." }
    "WAVE" { "Batch with same-parent homogeneous children; keep independent child white-box rows." }
    "SPLIT" { "Continue splitting, preferably as a standard same-parent wave." }
    "PRECISION" { "Use precision single-leaf governance and do not batch away high-risk evidence." }
}

$result = [ordered]@{
    leaf_id = $LeafId
    parent_id = $ParentId
    paths = $resolvedPaths
    metrics = [ordered]@{
        loc = $loc
        function_count = $functionCount
        public_surface = $publicSurface
        branch_count = $branchCount
        reexport_count = $reexportCount
        forwarding_count = $forwardingCount
        domain_count = $domains.Count
        domains = @($domains | Sort-Object)
        risk_tags = @($risk.tags)
    }
    scores = [ordered]@{
        split_benefit = $splitBenefit
        leaf_size_fit = $leafSizeFit
        risk_penalty = $riskPenalty
        governance_cost = $governanceCost
        system_efficiency_penalty = $systemEfficiencyPenalty
        weighted_delta = [Math]::Round($weightedDelta, 2)
        normalized_split_score = $normalizedScore
    }
    decision = $decision
    reason = $reason
    recommended_action = $action
}

if ($AsJson) {
    $result | ConvertTo-Json -Depth 8
    exit 0
}

Write-Host "Leaf granularity evaluation"
Write-Host "Leaf: $LeafId"
Write-Host "Decision: $decision"
Write-Host "Score: $normalizedScore"
Write-Host "Reason: $reason"
Write-Host ""
Write-Host "Metrics:"
Write-Host ("  loc={0}, functions={1}, public_surface={2}, branches={3}, domains={4}" -f $loc, $functionCount, $publicSurface, $branchCount, $domains.Count)
Write-Host ("  risk_tags={0}" -f ([string]::Join(",", @($risk.tags))))
Write-Host ""
Write-Host "Scores:"
Write-Host ("  split_benefit={0}, leaf_size_fit={1}, risk_penalty={2}, governance_cost={3}, system_efficiency_penalty={4}" -f $splitBenefit, $leafSizeFit, $riskPenalty, $governanceCost, $systemEfficiencyPenalty)
Write-Host ("  weighted_delta={0}, normalized_split_score={1}" -f ([Math]::Round($weightedDelta, 2)), $normalizedScore)
Write-Host ""
Write-Host "Recommended action:"
Write-Host "  $action"
