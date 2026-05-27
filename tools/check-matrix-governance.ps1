# check-matrix-governance.ps1
# Validates the v4.12+ matrix-governance control plane.

$ErrorActionPreference = "Stop"

$RootPath = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Failures = New-Object System.Collections.Generic.List[string]

function U {
    param([string] $Base64)
    return [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($Base64))
}

function Add-Failure {
    param([string] $Message)
    $script:Failures.Add($Message) | Out-Null
}

function Join-Root {
    param([string] $Path)
    return (Join-Path $RootPath $Path)
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

function Assert-FilePattern {
    param(
        [string] $Pattern,
        [string] $Description
    )
    $matches = @(Get-ChildItem -Path (Join-Root $Pattern) -File -ErrorAction SilentlyContinue)
    if ($matches.Count -eq 0) {
        Add-Failure "Missing required file pattern: $Pattern ($Description)"
    }
}

function Assert-TextContains {
    param(
        [string] $Path,
        [string] $Needle,
        [string] $Description
    )
    $content = Read-RepoText $Path
    if (-not $content.Contains($Needle)) {
        Add-Failure "$Path missing $Description"
    }
}

function Get-RelativePath {
    param([string] $FullName)
    return $FullName.Substring($RootPath.Length + 1).Replace("\", "/")
}

function Get-ModuleSection {
    param(
        [string] $Content,
        [string] $ModuleId
    )

    $lines = $Content -split "`r?`n"
    $items = New-Object System.Collections.Generic.List[string]
    $capture = $false
    $headingPattern = '^###\s+.*`' + [regex]::Escape($ModuleId) + '`'

    foreach ($line in $lines) {
        if (-not $capture) {
            if ($line -match $headingPattern) {
                $capture = $true
                $items.Add($line) | Out-Null
            }
            continue
        }

        if (($line -match '^#{2,3}\s+') -and ($line -notmatch [regex]::Escape($ModuleId))) {
            break
        }

        $items.Add($line) | Out-Null
    }

    return [string]::Join("`n", $items)
}

$T = @{
    ProcessMatrix = U "5rWB56iL55+p6Zi1"
    StandardMatrix = U "6KeE6IyD55+p6Zi1"
    GuidanceMatrix = U "5byV5a+855+p6Zi1"
    ModuleTree = U "5qih5Z2X5qCR"
    ExecutionTier = U "5omn6KGM5qGj5L2N"
    ImpactMatrixStatement = U "5b2x5ZON55+p6Zi15aOw5piO"
    ThreeMatrixImpactStatement = U "5LiJ55+p6Zi15b2x5ZON5aOw5piO"
    GuidanceCoordinate = U "5byV5a+85Z2Q5qCH"
    GuidanceCoordinateStatement = U "5byV5a+85Z2Q5qCH5aOw5piO"
    FullFeatureTreeNode = U "5YWo6YeP5qCR6IqC54K5"
    ModuleTreeNode = U "5qih5Z2X5qCR6IqC54K5"
    RealFile = U "55yf5a6e5paH5Lu2"
    TestGate = U "5rWL6K+VL+mXqOemgQ=="
    Test = U "5rWL6K+V"
    Gate = U "6Zeo56aB"
    CompatibilityCheck = U "6YCC6YWN5oCn5qCh6aqM"
    PlanOptimization = U "5pa55qGI5LyY5YyW"
    NonGoal = U "6Z2e55uu5qCH"
    ParentCommunicationRule = U "54i257qn6YCa5L+h6KeE5YiZ"
    RegressionProtection = U "5Zue5b2S5L+d5oqk"
    ReleaseTransition = U "5Y+R5biD6L+H5rih"
    NoProactive = U "5LiN5b6X5Li75Yqo"
    HorizontalLink = U "5qiq5ZCR6L+e5o6l"
    ReleasePerformanceEdge = U "5Y+R5biD5oCB5oCn6IO96L65"
    DeveloperExplicitStatement = U "5byA5Y+R6ICF5piO56Gu5aOw5piO"
    Standard = U "5qCH5YeG"
    Heavy = U "6YeN5Z6L"
}

$requiredCoreDocs = @(
    "markdown/00-matrix-governance/README.md",
    "markdown/00-matrix-governance/process-matrix.md",
    "markdown/00-matrix-governance/standard-matrix.md",
    "markdown/00-matrix-governance/guidance-matrix.md",
    "markdown/00-matrix-governance/module-tree.md",
    "markdown/00-matrix-governance/proposal-flow.md",
    "markdown/00-matrix-governance/proposal-examples.md",
    "markdown/00-matrix-governance/release-transition-protocol.md",
    "markdown/00-matrix-governance/landing-roadmap.md"
)

foreach ($path in $requiredCoreDocs) {
    Assert-FileExists $path
}

$requiredMilestonePatterns = @(
    @("markdown/06-milestones/v4.12.0/01-*.md", "v4.12 plan"),
    @("markdown/06-milestones/v4.12.0/02-*.md", "v4.12 landing record"),
    @("markdown/06-milestones/v4.13.0/01-*.md", "v4.13 plan"),
    @("markdown/06-milestones/v4.13.0/02-*.md", "v4.13 landing record"),
    @("markdown/06-milestones/v4.14.0/01-*.md", "v4.14 plan"),
    @("markdown/06-milestones/v4.14.0/02-*.md", "v4.14 landing record"),
    @("markdown/06-milestones/v4.15.0/01-*.md", "v4.15 plan"),
    @("markdown/06-milestones/v4.15.0/02-*.md", "v4.15 governance closeout"),
    @("markdown/06-milestones/v4.16.0/01-*.md", "v4.16 extraction plan"),
    @("markdown/06-milestones/v4.16.0/02-*.md", "v4.16 landing record"),
    @("markdown/06-milestones/v4.16.0/03-*.md", "v4.16 backend extraction register"),
    @("markdown/06-milestones/v4.16.0/04-*.md", "v4.16 frontend extraction register"),
    @("markdown/06-milestones/v4.16.0/05-*.md", "v4.16 test asset register"),
    @("markdown/06-milestones/v4.16.0/06-*.md", "v4.16 backend interface boundary plan"),
    @("markdown/06-milestones/v4.16.0/07-*.md", "v4.16 top module statistics"),
    @("markdown/06-milestones/v4.16.0/08-*.md", "v4.16 system module split statistics"),
    @("markdown/06-milestones/v4.16.0/09-*.md", "v4.16 system entry extraction record"),
    @("markdown/06-milestones/v4.16.0/10-*.md", "v4.16 system extraction completion record"),
    @("markdown/06-milestones/v4.16.0/11-*.md", "v4.16 system extraction lessons backfill"),
    @("markdown/06-milestones/v4.16.0/12-*.md", "v4.16 system ten leaf equivalence baseline"),
    @("markdown/06-milestones/v4.16.0/13-*.md", "v4.16 recursive modularization global root flow"),
    @("markdown/06-milestones/v4.16.0/14-*.md", "v4.16 system launch scripts leaf closeout"),
    @("markdown/06-milestones/v4.16.0/15-*.md", "v4.16 system tauri config leaf closeout"),
    @("markdown/06-milestones/v4.16.0/16-*.md", "v4.16 system runtime profile config examples closeout"),
    @("markdown/06-milestones/v4.16.0/17-*.md", "v4.16 system tauri runtime readiness equivalence check"),
    @("markdown/06-milestones/v4.16.0/18-*.md", "v4.16 system tauri runtime leaf closeout"),
    @("markdown/06-milestones/v4.16.0/19-*.md", "v4.16 system desktop build scripts leaf closeout"),
    @("markdown/06-milestones/v4.16.0/20-*.md", "v4.16 system backend process leaf closeout"),
    @("markdown/06-milestones/v4.16.0/21-*.md", "v4.16 system assets schema leaf closeout"),
    @("markdown/06-milestones/v4.16.0/22-*.md", "v4.16 system container proxy leaf closeout"),
    @("markdown/06-milestones/v4.16.0/23-*.md", "v4.16 system workspace manifest and ci release pause record"),
    @("markdown/06-milestones/v4.16.0/24-*.md", "v4.16 system top stage closeout"),
    @("markdown/06-milestones/v4.16.0/25-*.md", "v4.16 system s6 s9 resume proposal"),
    @("markdown/06-milestones/v4.16.0/26-*.md", "v4.16 system workspace manifest closeout"),
    @("markdown/06-milestones/v4.16.0/27-*.md", "v4.16 system ci release closeout"),
    @("markdown/06-milestones/v4.16.0/28-*.md", "v4.16 backend module split statistics"),
    @("markdown/06-milestones/v4.16.0/29-*.md", "v4.16 backend interface boundary equivalence baseline"),
    @("markdown/06-milestones/v4.16.0/30-*.md", "v4.16 backend nine leaf facade extraction record"),
    @("markdown/06-milestones/v4.16.0/31-*.md", "v4.16 backend interface boundary leaf closeout"),
    @("markdown/06-milestones/v4.16.0/32-*.md", "v4.16 backend capability leaf closeout"),
    @("markdown/06-milestones/v4.16.0/33-*.md", "v4.16 backend strategy config leaf closeout"),
    @("markdown/06-milestones/v4.16.0/34-*.md", "v4.16 backend runtime leaf closeout"),
    @("markdown/06-milestones/v4.16.0/35-*.md", "v4.16 backend graph compile leaf closeout"),
    @("markdown/06-milestones/v4.16.0/36-*.md", "v4.16 backend storage security leaf closeout"),
    @("markdown/06-milestones/v4.16.0/37-*.md", "v4.16 backend ops governance leaf closeout"),
    @("markdown/06-milestones/v4.16.0/38-*.md", "v4.16 backend app state wiring leaf closeout"),
    @("markdown/06-milestones/v4.16.0/39-*.md", "v4.16 backend test support leaf closeout")
)

foreach ($entry in $requiredMilestonePatterns) {
    Assert-FilePattern $entry[0] $entry[1]
}

$indexChecks = @(
    @("README.md", "markdown/00-matrix-governance/README.md", "root matrix governance entry"),
    @("markdown/README.md", "00-matrix-governance/README.md", "markdown matrix governance entry"),
    @("markdown/General_Policy.md", "00-matrix-governance/README.md", "general policy matrix governance handoff"),
    @("markdown/01-principles/principles-super-standardization.md", "00-matrix-governance/README.md", "super standardization matrix governance handoff"),
    @("markdown/00-matrix-governance/README.md", "proposal-examples.md", "matrix proposal examples entry"),
    @("markdown/10-overview/README.md", "../00-matrix-governance/README.md", "overview matrix governance entry"),
    @("markdown/10-overview/overview-docs-index.md", "../00-matrix-governance/README.md", "docs index matrix governance entry"),
    @("markdown/10-overview/overview-docs-index.md", "proposal-examples.md", "docs index proposal examples"),
    @("markdown/10-overview/overview-current-status-and-roadmap.md", "../00-matrix-governance/landing-roadmap.md", "current roadmap matrix landing roadmap"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/00-matrix-governance/", "full feature tree matrix directory"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/00-matrix-governance/proposal-examples.md", "full feature tree proposal examples"),
    @("markdown/10-overview/overview-full-feature-tree.md", "tools/check-matrix-governance.ps1", "full feature tree matrix gate"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.14.0/02-", "full feature tree v4.14 record"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.15.0/02-", "full feature tree v4.15 closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/01-", "full feature tree v4.16 plan"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/02-", "full feature tree v4.16 landing"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/03-", "full feature tree v4.16 backend register"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/04-", "full feature tree v4.16 frontend register"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/05-", "full feature tree v4.16 test asset register"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/06-", "full feature tree v4.16 backend interface boundary plan"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/07-", "full feature tree v4.16 top module statistics"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/08-", "full feature tree v4.16 system module split statistics"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/09-", "full feature tree v4.16 system entry extraction record"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/10-", "full feature tree v4.16 system extraction completion record"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/11-", "full feature tree v4.16 system extraction lessons backfill"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/12-", "full feature tree v4.16 system ten leaf baseline"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/13-", "full feature tree v4.16 recursive modularization flow"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/14-", "full feature tree v4.16 system launch scripts closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/15-", "full feature tree v4.16 system tauri config closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/16-", "full feature tree v4.16 system runtime profile closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/17-", "full feature tree v4.16 system tauri runtime readiness check"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/18-", "full feature tree v4.16 system tauri runtime closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/19-", "full feature tree v4.16 system desktop build scripts closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/20-", "full feature tree v4.16 system backend process closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/21-", "full feature tree v4.16 system assets schema closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/22-", "full feature tree v4.16 system container proxy closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/23-", "full feature tree v4.16 system s6 s9 pause record"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/24-", "full feature tree v4.16 system top stage closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/25-", "full feature tree v4.16 system s6 s9 resume proposal"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/26-", "full feature tree v4.16 system workspace manifest closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/27-", "full feature tree v4.16 system ci release closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/28-", "full feature tree v4.16 backend module split statistics"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/29-", "full feature tree v4.16 backend interface baseline"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/30-", "full feature tree v4.16 backend nine leaf facade extraction"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/31-", "full feature tree v4.16 backend interface closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/32-", "full feature tree v4.16 backend capability closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/33-", "full feature tree v4.16 backend strategy config closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/34-", "full feature tree v4.16 backend runtime closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/35-", "full feature tree v4.16 backend graph compile closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/36-", "full feature tree v4.16 backend storage security closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/37-", "full feature tree v4.16 backend ops governance closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/38-", "full feature tree v4.16 backend app state wiring closeout"),
    @("markdown/10-overview/overview-full-feature-tree.md", "markdown/06-milestones/v4.16.0/39-", "full feature tree v4.16 backend test support closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/03-", "module tree v4.16 backend register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/04-", "module tree v4.16 frontend register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/05-", "module tree v4.16 test asset register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/06-", "module tree v4.16 backend interface boundary plan"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/07-", "module tree v4.16 top module statistics"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/08-", "module tree v4.16 system module split statistics"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/09-", "module tree v4.16 system entry extraction record"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/10-", "module tree v4.16 system extraction completion record"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/11-", "module tree v4.16 system extraction lessons backfill"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/12-", "module tree v4.16 system ten leaf baseline"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/13-", "module tree v4.16 recursive modularization flow"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/14-", "module tree v4.16 system launch scripts closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/15-", "module tree v4.16 system tauri config closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/16-", "module tree v4.16 system runtime profile closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/17-", "module tree v4.16 system tauri runtime readiness check"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/18-", "module tree v4.16 system tauri runtime closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/19-", "module tree v4.16 system desktop build scripts closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/20-", "module tree v4.16 system backend process closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/21-", "module tree v4.16 system assets schema closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/22-", "module tree v4.16 system container proxy closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/23-", "module tree v4.16 system s6 s9 pause record"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/24-", "module tree v4.16 system top stage closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/25-", "module tree v4.16 system s6 s9 resume proposal"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/26-", "module tree v4.16 system workspace manifest closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/27-", "module tree v4.16 system ci release closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/28-", "module tree v4.16 backend module split statistics"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/29-", "module tree v4.16 backend interface baseline"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/30-", "module tree v4.16 backend nine leaf facade extraction"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/31-", "module tree v4.16 backend interface closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/32-", "module tree v4.16 backend capability closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/33-", "module tree v4.16 backend strategy config closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/34-", "module tree v4.16 backend runtime closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/35-", "module tree v4.16 backend graph compile closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/36-", "module tree v4.16 backend storage security closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/37-", "module tree v4.16 backend ops governance closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/38-", "module tree v4.16 backend app state wiring closeout"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/39-", "module tree v4.16 backend test support closeout"),
    @("markdown/10-overview/overview-docs-index.md", "v4.16.0", "docs index v4.16 plan"),
    @("markdown/10-overview/overview-current-status-and-roadmap.md", "v4.16.0", "current roadmap v4.16 plan")
)

foreach ($check in $indexChecks) {
    Assert-TextContains $check[0] $check[1] $check[2]
}

$v416PlanFiles = @(Get-ChildItem -LiteralPath (Join-Root "markdown/06-milestones/v4.16.0") -Filter "01-*.md" -File -ErrorAction SilentlyContinue)
if ($v416PlanFiles.Count -eq 0) {
    Add-Failure "Missing v4.16 extraction plan file"
} else {
    $v416Plan = Get-RelativePath $v416PlanFiles[0].FullName
    $v416Tokens = @(
        @((U "5Y2B5LiH6KGM57qn6YeN5aSn5bel56iL5L+d5oqk5qCP"), "large-scale extraction guardrails"),
        @((U "5ZCO56uv5oq956a7"), "backend extraction"),
        @((U "5YmN56uv5oq956a7"), "frontend extraction"),
        @((U "RTJFIOaVtOeQhuW7tuWQjg=="), "deferred E2E cleanup"),
        @((U "5rWL6K+V6LWE5Lqn5rGw5o2i"), "test asset replacement"),
        @((U "5oq956a7"), "extract phase"),
        @((U "5pW055CG"), "organize phase"),
        @((U "6YeN5p6E"), "refactor phase"),
        @((U "5Yaz562W5pqC5YGc"), "decision pause"),
        @((U "5om55qyh5YiH54mH"), "batch slicing")
    )

    foreach ($check in $v416Tokens) {
        Assert-TextContains $v416Plan $check[0] $check[1]
    }
}

$v416LandingFiles = @(
    @("markdown/06-milestones/v4.16.0/02-*.md", @(
        @((U "5oq956a75o6n5Yi26Z2i"), "extraction control plane"),
        @((U "5ZCO56uv5oq956a7"), "backend extraction"),
        @((U "5YmN56uv5oq956a7"), "frontend extraction"),
        @((U "5rWL6K+V6LWE5Lqn5rGw5o2i"), "test asset replacement"),
        @((U "RTJFIOaVtOeQhuW7tuWQjg=="), "deferred E2E cleanup")
    )),
    @("markdown/06-milestones/v4.16.0/03-*.md", @(
        @((U "5ZCO56uv5oq956a755m76K6w"), "backend extraction register"),
        @((U "5LiN5YiH5o2i5Li7IEFQSQ=="), "no main API switch"),
        @((U "5pu/5Luj6K+B5o2u"), "replacement evidence")
    )),
    @("markdown/06-milestones/v4.16.0/04-*.md", @(
        @((U "5YmN56uv5oq956a755m76K6w"), "frontend extraction register"),
        @((U "5LiN5YCf5oq956a75YGaIFVYIOmHjeaehA=="), "no UX refactor"),
        @((U "5pu/5Luj6K+B5o2u"), "replacement evidence")
    )),
    @("markdown/06-milestones/v4.16.0/05-*.md", @(
        @((U "5rWL6K+V6LWE5Lqn5rGw5o2i55m76K6w"), "test asset replacement register"),
        @((U "5LiN5Yig6Zmk5pen5rWL6K+V56iL5bqP"), "do not delete old test programs"),
        @((U "5pu/5Luj6K+B5o2u"), "replacement evidence"),
        @((U "6aOO6Zmp56qX5Y+j"), "risk window")
    )),
    @("markdown/06-milestones/v4.16.0/06-*.md", @(
        @("BE-001", "BE-001 batch marker"),
        @((U "5ZCO56uv5o6l5Y+j6L6555WM"), "backend interface boundary"),
        @("build_app_router", "backend router parent entry"),
        @("register_runtime_routes", "runtime route registration"),
        @((U "5LiN6L+B56e754q25oCB5omA5pyJ5p2D"), "no state ownership migration"),
        @((U "5LiN5Yig6Zmk5penIGhhbmRsZXI="), "do not delete old handler"),
        @((U "5YW85a655qGl"), "compatibility bridge"),
        @((U "5Zue6YCA54K5"), "rollback point")
    )),
    @("markdown/06-milestones/v4.16.0/07-*.md", @(
        @((U "6aG25bGC5aSn5qih5Z2X57uf6K6h"), "top module statistics"),
        @((U "6YC76L6R6aG25bGC5aSn5qih5Z2XIDYg5Liq"), "six logical top modules"),
        @((U "5bey55m76K6w55m9566x5a2Q6IqC54K5IDE2IOS4qg=="), "sixteen whitebox child nodes"),
        @((U "dHJhY2tlZCDmlofku7YgMTAwNyDkuKo="), "tracked file count"),
        @("backend.interface_boundary", "backend interface boundary selection"),
        @((U "Y29udHJhY3RzIOmhtuWxguaaguaXoOeZveeuseWtkOiKgueCuQ=="), "contracts coverage gap")
    )),
    @("markdown/06-milestones/v4.16.0/08-*.md", @(
        @((U "c3lzdGVtIOWkp+aooeWdl+WIhuWxgue7n+iuoQ=="), "system module split statistics"),
        @((U "YHN5c3RlbWAg5bu66K6u5YiGIDMg5bGC77yMMTAg5Liq5Y+25a2Q5qih5Z2X"), "system three layers and ten leaf modules"),
        @((U "c3lzdGVtLmVudHJ5LmxhdW5jaF9zY3JpcHRz"), "system entry launch scripts"),
        @((U "c3lzdGVtLmVudHJ5LmJhY2tlbmRfcHJvY2Vzcw=="), "system entry backend process"),
        @((U "c3lzdGVtLmRlc2t0b3Bfc2hlbGwudGF1cmlfcnVudGltZQ=="), "system desktop shell tauri runtime"),
        @((U "c3lzdGVtLmJ1aWxkX2RlbGl2ZXJ5LmNpX3JlbGVhc2U="), "system build delivery ci release"),
        @((U "c3lzdGVtLnJ1bnRpbWVfcHJvZmlsZS5jb25maWdfZXhhbXBsZXM="), "system runtime profile config examples"),
        @((U "5LiN5oqK5ZCv5Yqo57yW5o6S6K+v5Yik5Li65Lia5Yqh6IO95Yqb55yf5rqQ"), "do not mistake startup orchestration as capability truth source"),
        @("root.system", "root system node"),
        @("backend.interface_boundary", "backend interface boundary relation")
    )),
    @("markdown/06-milestones/v4.16.0/09-*.md", @(
        @("system.entry.backend_process", "system entry backend process module"),
        @("src/system/entry/backend_process.rs", "system entry backend process file"),
        @("src/system/mod.rs", "system module root file"),
        @("src/system/entry/mod.rs", "system entry module file"),
        @("run_server", "run server public method"),
        @("quantpilot::run_server", "compatible crate root entry"),
        @("build_app_router", "backend interface boundary bridge"),
        @("cargo check -p quantpilot", "rust check gate")
    )),
    @("markdown/06-milestones/v4.16.0/10-*.md", @(
        @("system.entry.backend_process", "system entry backend process module"),
        @("src/system/entry/backend_process.rs", "system entry backend process file"),
        @("run_server", "run server public method"),
        @("run_api_server", "run api server startup boundary"),
        @("quantpilot::run_server", "compatible crate root entry"),
        @("new_app_state", "state factory boundary"),
        @("build_app_router", "backend interface boundary bridge"),
        @("cargo check -p quantpilot", "rust check gate")
    )),
    @("markdown/06-milestones/v4.16.0/11-*.md", @(
        @("system.entry.backend_process", "system entry backend process module"),
        @("public", "public method classification"),
        @("run_api_server", "internal startup implementation example"),
        @("new_app_state", "state factory retained boundary"),
        @("build_app_router", "backend interface retained boundary"),
        @("BE-001", "backend interface boundary reuse"),
        @("owner", "owner review rule"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/12-*.md", @(
        @("system.entry.launch_scripts", "system launch scripts leaf"),
        @("system.entry.backend_process", "system backend process leaf"),
        @("system.desktop_shell.tauri_runtime", "system tauri runtime leaf"),
        @("system.desktop_shell.tauri_config", "system tauri config leaf"),
        @("system.desktop_shell.assets_schema", "system assets schema leaf"),
        @("system.build_delivery.workspace_manifest", "system workspace manifest leaf"),
        @("system.build_delivery.desktop_build_scripts", "system desktop build scripts leaf"),
        @("system.build_delivery.container_proxy", "system container proxy leaf"),
        @("system.build_delivery.ci_release", "system ci release leaf"),
        @("system.runtime_profile.config_examples", "system runtime profile config examples leaf"),
        @("public", "public entry marker"),
        @("owner", "owner baseline marker"),
        @("cargo check -p quantpilot", "backend check gate"),
        @("cargo check -p quantpilot-tauri", "tauri check gate")
    )),
    @("markdown/06-milestones/v4.16.0/13-*.md", @(
        @("root.system", "root system module"),
        @("root.backend", "root backend module"),
        @("root.frontend", "root frontend module"),
        @("root.executor", "root executor module"),
        @("root.contracts", "root contracts module"),
        @("root.docs", "root docs module"),
        @("public", "public entry marker"),
        @("owner", "owner review marker"),
        @((U "6YCS5b2S"), "recursive marker"),
        @((U "5YWo5bGA5qC5"), "global root marker"),
        @((U "57uG5YiG5Lu35YC8"), "split value marker"),
        @("tools/check-full-feature-tree.ps1", "full feature tree gate")
    )),
    @("markdown/06-milestones/v4.16.0/14-*.md", @(
        @("system.entry.launch_scripts", "system launch scripts leaf"),
        @("start.bat", "batch launch script"),
        @("start.ps1", "powershell launch script"),
        @("QUANTPILOT_DEV=true", "dev mode environment"),
        @("cargo build --bin quantpilot", "backend build command"),
        @("target\debug\quantpilot.exe", "backend executable"),
        @("cargo tauri dev", "tauri dev command"),
        @("3000", "backend readiness port"),
        @("5173", "frontend dev port cleanup"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/15-*.md", @(
        @("system.desktop_shell.tauri_config", "system tauri config leaf"),
        @("src-tauri/tauri.conf.json", "tauri config file"),
        @("src-tauri/capabilities/default.json", "tauri capability file"),
        @("QuantPilot", "product name"),
        @("com.quantpilot.app", "app identifier"),
        @("http://localhost:5173", "tauri dev url"),
        @("http://127.0.0.1:3000", "backend api csp target"),
        @("core:default", "default capability permission"),
        @("shell:allow-open", "shell open permission"),
        @("cargo check -p quantpilot-tauri", "tauri check gate")
    )),
    @("markdown/06-milestones/v4.16.0/16-*.md", @(
        @("system.runtime_profile.config_examples", "system runtime profile config examples leaf"),
        @(".env.example", "env example file"),
        @("config/runtime_protocol.example.yaml", "runtime protocol example file"),
        @("config/strategy_ir.v0.schema.json", "strategy ir schema file"),
        @("config/strategy_ir.v0.example.json", "strategy ir example file"),
        @("QUANTPILOT_DEV", "dev env example"),
        @("QUANTPILOT_LOG_FORMAT", "log env example"),
        @("generators", "runtime protocol generators"),
        @("global_risk", "runtime protocol risk"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/17-*.md", @(
        @("system.desktop_shell.tauri_runtime", "system tauri runtime leaf"),
        @("src-tauri/src/main.rs", "tauri runtime main file"),
        @("src-tauri/Cargo.toml", "tauri cargo manifest"),
        @("src-tauri/tauri.conf.json", "tauri config relation"),
        @("main", "tauri main public entry"),
        @("wait_for_backend", "backend readiness wait"),
        @("TcpStream::connect_timeout", "tcp readiness check"),
        @("BACKEND_PORT", "backend port constant"),
        @("MAX_WAIT_SECS", "max wait constant"),
        @("127.0.0.1:3000", "backend readiness endpoint"),
        @("tauri::Builder::default", "tauri builder startup"),
        @("tauri_plugin_shell::init", "tauri shell plugin"),
        @("cargo check -p quantpilot-tauri", "tauri check gate")
    )),
    @("markdown/06-milestones/v4.16.0/18-*.md", @(
        @("system.desktop_shell.tauri_runtime", "system tauri runtime leaf"),
        @("src-tauri/src/main.rs", "tauri runtime main file"),
        @("src-tauri/Cargo.toml", "tauri cargo manifest"),
        @("src-tauri/tauri.conf.json", "tauri config relation"),
        @("main", "tauri main public entry"),
        @("wait_for_backend", "backend readiness wait"),
        @("127.0.0.1:3000", "backend readiness endpoint"),
        @("5173", "frontend dev server port"),
        @("cargo build --bin quantpilot", "backend build smoke"),
        @("cargo tauri dev --no-watch", "tauri desktop smoke command"),
        @("CloseMainWindow", "window lifecycle close check"),
        @("quantpilot-tauri.exe", "tauri window process"),
        @("cargo check -p quantpilot-tauri", "tauri check gate"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/19-*.md", @(
        @("system.build_delivery.desktop_build_scripts", "system desktop build scripts leaf"),
        @("src-tauri/build.rs", "tauri build rs file"),
        @("src-tauri/build.bat", "tauri build bat file"),
        @("src-tauri/dev.bat", "tauri dev bat file"),
        @("src-tauri/tauri.conf.json", "tauri config relation"),
        @("frontend/package.json", "frontend package scripts"),
        @("tauri_build::build()", "tauri build script call"),
        @("npm run build", "frontend build command"),
        @("npm run dev -- --strictPort", "frontend dev strict port command"),
        @("5173", "frontend dev server port"),
        @("cmd /c src-tauri\build.bat", "build bat smoke"),
        @("cargo check -p quantpilot-tauri", "tauri check gate"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/20-*.md", @(
        @("system.entry.backend_process", "system backend process leaf"),
        @("src/system/entry/backend_process.rs", "system backend process file"),
        @("src/app_runtime_helpers.rs", "app state retained file"),
        @("src/app_router.rs", "router retained file"),
        @("run_server", "run server public entry"),
        @("quantpilot::run_server", "compatible crate root entry"),
        @("run_api_server", "run api server internal startup"),
        @("new_app_state", "state factory retained boundary"),
        @("build_app_router", "router retained boundary"),
        @("cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided", "default server behavior test")
    )),
    @("markdown/06-milestones/v4.16.0/21-*.md", @(
        @("system.desktop_shell.assets_schema", "system assets schema leaf"),
        @("src-tauri/icons/32x32.png", "tauri icon file"),
        @("src-tauri/icons/128x128.png", "tauri icon file"),
        @("src-tauri/icons/icon.ico", "tauri ico file"),
        @("src-tauri/gen/schemas/acl-manifests.json", "tauri acl schema"),
        @("src-tauri/gen/schemas/capabilities.json", "tauri capabilities schema"),
        @("src-tauri/gen/schemas/desktop-schema.json", "tauri desktop schema"),
        @("src-tauri/gen/schemas/windows-schema.json", "tauri windows schema"),
        @("ConvertFrom-Json", "json parse evidence")
    )),
    @("markdown/06-milestones/v4.16.0/22-*.md", @(
        @("system.build_delivery.container_proxy", "system container proxy leaf"),
        @("Dockerfile", "dockerfile"),
        @("docker-compose.yml", "docker compose file"),
        @("nginx.conf", "nginx config file"),
        @("3000", "backend port"),
        @("5173", "frontend dev port"),
        @("443", "https proxy port"),
        @("docker compose config", "optional docker compose config evidence"),
        @("Docker runtime smoke requires developer release decision", "docker runtime release decision gate")
    )),
    @("markdown/06-milestones/v4.16.0/23-*.md", @(
        @("system.build_delivery.workspace_manifest", "system workspace manifest leaf"),
        @("system.build_delivery.ci_release", "system ci release leaf"),
        @("Cargo.toml", "cargo manifest"),
        @("Cargo.lock", "cargo lockfile"),
        @("src-tauri/Cargo.toml", "tauri manifest"),
        @(".github/workflows/ci.yml", "ci workflow"),
        @(".github/workflows/release.yml", "release workflow"),
        @(".github/workflows/scenario-test.yml", "scenario workflow"),
        @("cargo metadata --format-version 1 --no-deps", "cargo metadata gate"),
        @("S6/S9 pause is not closeout", "pause is not closeout marker")
    )),
    @("markdown/06-milestones/v4.16.0/24-*.md", @(
        @("root.system", "root system module"),
        @("system top stage closeout is not full final completion", "system top stage closeout marker"),
        @("S1-S10", "completed system leaf set"),
        @("system.entry.backend_process", "system backend process leaf"),
        @("system.desktop_shell.assets_schema", "system assets schema leaf"),
        @("system.build_delivery.container_proxy", "system container proxy leaf"),
        @("Docker runtime smoke", "docker runtime boundary"),
        @("tools/check-matrix-governance.ps1", "matrix governance gate")
    )),
    @("markdown/06-milestones/v4.16.0/25-*.md", @(
        @("S6/S9 resume proposal passed", "s6 s9 resume proposal marker"),
        @("system.build_delivery.workspace_manifest", "system workspace manifest leaf"),
        @("system.build_delivery.ci_release", "system ci release leaf"),
        @("proposal compatibility optimization continue design", "proposal flow marker"),
        @("git diff -- Cargo.toml Cargo.lock src-tauri/Cargo.toml", "target diff clean command"),
        @("no real manifest workflow edits", "no real manifest edits")
    )),
    @("markdown/06-milestones/v4.16.0/26-*.md", @(
        @("S6 workspace manifest closeout complete", "s6 closeout marker"),
        @("system.build_delivery.workspace_manifest", "system workspace manifest leaf"),
        @("Cargo.toml", "cargo manifest"),
        @("Cargo.lock", "cargo lockfile"),
        @("src-tauri/Cargo.toml", "tauri manifest"),
        @("cargo metadata --format-version 1 --no-deps", "cargo metadata gate"),
        @("cargo check --workspace", "cargo workspace check"),
        @("git diff -- Cargo.toml Cargo.lock src-tauri/Cargo.toml", "manifest diff clean")
    )),
    @("markdown/06-milestones/v4.16.0/27-*.md", @(
        @("S9 ci release closeout complete", "s9 closeout marker"),
        @("system.build_delivery.ci_release", "system ci release leaf"),
        @(".github/workflows/ci.yml", "ci workflow"),
        @(".github/workflows/release.yml", "release workflow"),
        @(".github/workflows/scenario-test.yml", "scenario workflow"),
        @("packaging/windows/installer.nsi", "windows installer script"),
        @("release/release-manifest.yaml", "release manifest"),
        @("release dry-run", "release dry run boundary"),
        @("not release approval", "not release approval marker")
    )),
    @("markdown/06-milestones/v4.16.0/28-*.md", @(
        @("root.backend", "backend root module"),
        @("backend.interface_boundary", "backend interface boundary"),
        @("backend.runtime", "backend runtime leaf"),
        @("backend.graph_compile", "backend graph compile leaf"),
        @("backend.storage_security", "backend storage security leaf"),
        @("backend.app_state_wiring", "backend app state wiring candidate"),
        @("backend.test_support", "backend test support candidate"),
        @("9", "backend leaf candidate count"),
        @("build_app_router", "backend router parent entry"),
        @("register_runtime_routes", "runtime route registration"),
        @("cargo check -p quantpilot", "backend check gate")
    )),
    @("markdown/06-milestones/v4.16.0/29-*.md", @(
        @("BE-001A", "backend interface baseline marker"),
        @("backend.interface_boundary", "backend interface boundary"),
        @("route owner", "route owner baseline"),
        @("build_app_router", "backend router parent entry"),
        @("get_capabilities", "capability entry"),
        @("register_strategy_config_routes", "strategy config route registration"),
        @("register_runtime_routes", "runtime route registration"),
        @("register_graph_routes", "graph route registration"),
        @("register_graph_quantscript_routes", "quantscript graph route registration"),
        @("register_compile_routes", "compile route registration"),
        @("existing handler", "handler retention marker"),
        @("existing state owner", "state owner retention marker"),
        @("cargo test -p quantpilot --test api_run", "api run test gate")
    )),
    @("markdown/06-milestones/v4.16.0/30-*.md", @(
        @("BE-001B", "backend facade extraction marker"),
        @("src/backend/mod.rs", "backend module root file"),
        @("src/backend/interface_boundary.rs", "backend interface facade file"),
        @("src/backend/capability.rs", "backend capability facade file"),
        @("src/backend/strategy_config.rs", "backend strategy config facade file"),
        @("src/backend/runtime.rs", "backend runtime facade file"),
        @("src/backend/graph_compile.rs", "backend graph compile facade file"),
        @("src/backend/storage_security.rs", "backend storage security facade file"),
        @("src/backend/ops_governance.rs", "backend ops governance facade file"),
        @("src/backend/app_state_wiring.rs", "backend app state wiring facade file"),
        @("src/backend/test_support.rs", "backend test support facade file"),
        @("existing handler", "handler retention marker"),
        @("existing state owner", "state owner retention marker"),
        @("cargo check -p quantpilot", "backend check gate")
    )),
    @("markdown/06-milestones/v4.16.0/31-*.md", @(
        @("BE-001C-01", "backend interface closeout marker"),
        @("backend.interface_boundary", "backend interface boundary"),
        @("src/backend/interface_boundary.rs", "backend interface facade file"),
        @("src/app_router.rs", "app router file"),
        @("build_app_router", "backend router parent entry"),
        @("attach_state", "state attach facade"),
        @((U "5LiN57un57ut5ouG5YiG"), "stop split decision")
    )),
    @("markdown/06-milestones/v4.16.0/32-*.md", @(
        @("BE-001C-02", "backend capability closeout marker"),
        @("backend.capability", "backend capability leaf"),
        @("src/backend/capability.rs", "backend capability facade file"),
        @("src/capability_api.rs", "capability api file"),
        @("get_capabilities", "capability entry"),
        @((U "5LiN57un57ut5ouG5YiG"), "stop split decision")
    )),
    @("markdown/06-milestones/v4.16.0/33-*.md", @(
        @("BE-001C-03", "backend strategy config closeout marker"),
        @("backend.strategy_config", "backend strategy config leaf"),
        @("src/backend/strategy_config.rs", "backend strategy config facade file"),
        @("register_strategy_config_routes", "strategy config route registration"),
        @("backend.strategy_config.artifact", "strategy config artifact l3"),
        @("backend.strategy_config.preflight", "strategy config preflight l3"),
        @((U "5YC85b6X57un57ut5ouG5YiG"), "worth split decision")
    )),
    @("markdown/06-milestones/v4.16.0/34-*.md", @(
        @("BE-001C-04", "backend runtime closeout marker"),
        @("backend.runtime", "backend runtime leaf"),
        @("src/backend/runtime.rs", "backend runtime facade file"),
        @("register_runtime_routes", "runtime route registration"),
        @("backend.runtime.run", "runtime run l3"),
        @("backend.runtime.backtest", "runtime backtest l3"),
        @((U "5YC85b6X57un57ut5ouG5YiG"), "worth split decision")
    )),
    @("markdown/06-milestones/v4.16.0/35-*.md", @(
        @("BE-001C-05", "backend graph compile closeout marker"),
        @("backend.graph_compile", "backend graph compile leaf"),
        @("src/backend/graph_compile.rs", "backend graph compile facade file"),
        @("register_graph_routes", "graph route registration"),
        @("register_compile_routes", "compile route registration"),
        @("backend.graph_compile.graph_api", "graph api l3"),
        @((U "5YC85b6X57un57ut5ouG5YiG"), "worth split decision")
    )),
    @("markdown/06-milestones/v4.16.0/36-*.md", @(
        @("BE-001C-06", "backend storage security closeout marker"),
        @("backend.storage_security", "backend storage security leaf"),
        @("src/backend/storage_security.rs", "backend storage security facade file"),
        @("register_credential_routes", "credential route registration"),
        @("backend.storage_security.credential_vault", "credential vault l3"),
        @((U "5a6J5YWo5Yaz562W5pqC5YGc"), "security decision pause"),
        @((U "5YC85b6X57un57ut5ouG5YiG"), "worth split decision")
    )),
    @("markdown/06-milestones/v4.16.0/37-*.md", @(
        @("BE-001C-07", "backend ops governance closeout marker"),
        @("backend.ops_governance", "backend ops governance leaf"),
        @("src/backend/ops_governance.rs", "backend ops governance facade file"),
        @("register_alert_routes", "alert route registration"),
        @("backend.ops_governance.sandbox", "ops sandbox l3"),
        @("backend.ops_governance.hotswap", "ops hotswap l3"),
        @((U "5YC85b6X57un57ut5ouG5YiG"), "worth split decision")
    )),
    @("markdown/06-milestones/v4.16.0/38-*.md", @(
        @("BE-001C-08", "backend app state wiring closeout marker"),
        @("backend.app_state_wiring", "backend app state wiring leaf"),
        @("src/backend/app_state_wiring.rs", "backend app state wiring facade file"),
        @("new_app_state", "app state factory"),
        @("attach_state", "state attach facade"),
        @((U "5LiN57un57ut5ouG5YiG"), "stop split decision")
    )),
    @("markdown/06-milestones/v4.16.0/39-*.md", @(
        @("BE-001C-09", "backend test support closeout marker"),
        @("backend.test_support", "backend test support leaf"),
        @("src/backend/test_support.rs", "backend test support facade file"),
        @("register_test_scenario_routes", "test scenario route registration"),
        @((U "MDUt5rWL6K+V6LWE5Lqn5rGw5o2i55m76K6wLm1k"), "test asset replacement register"),
        @((U "5LiN57un57ut5ouG5YiG"), "stop split decision")
    ))
)

foreach ($entry in $v416LandingFiles) {
    $files = @(Get-ChildItem -Path (Join-Root $entry[0]) -File -ErrorAction SilentlyContinue)
    if ($files.Count -eq 0) {
        Add-Failure "Missing required v4.16 landing file pattern: $($entry[0])"
        continue
    }

    $relative = Get-RelativePath $files[0].FullName
    foreach ($check in $entry[1]) {
        Assert-TextContains $relative $check[0] $check[1]
    }
}

$proposalFlow = "markdown/00-matrix-governance/proposal-flow.md"
$proposalTemplateTokens = @(
    @($T.ExecutionTier, "execution tier"),
    @($T.ImpactMatrixStatement, "impact matrix statement"),
    @($T.ProcessMatrix, "process matrix"),
    @($T.StandardMatrix, "standard matrix"),
    @($T.GuidanceMatrix, "guidance matrix"),
    @($T.ModuleTree, "module tree"),
    @($T.GuidanceCoordinateStatement, "guidance coordinate statement"),
    @($T.FullFeatureTreeNode, "full feature tree node"),
    @($T.ModuleTreeNode, "module tree node"),
    @($T.RealFile, "real file"),
    @("public", "public method marker"),
    @($T.TestGate, "test/gate coordinate"),
    @($T.CompatibilityCheck, "compatibility check"),
    @($T.PlanOptimization, "plan optimization"),
    @($T.NonGoal, "non-goal")
)

foreach ($check in $proposalTemplateTokens) {
    Assert-TextContains $proposalFlow $check[0] $check[1]
}

$releaseProtocol = "markdown/00-matrix-governance/release-transition-protocol.md"
$releaseTokens = @(
    @($T.NoProactive, "no proactive transition rule"),
    @($T.ReleaseTransition, "release transition"),
    @($T.HorizontalLink, "horizontal link"),
    @($T.ReleasePerformanceEdge, "release performance edge"),
    @($T.DeveloperExplicitStatement, "developer explicit statement"),
    @("AI", "AI actor rule")
)

foreach ($check in $releaseTokens) {
    Assert-TextContains $releaseProtocol $check[0] $check[1]
}

$moduleTreePath = "markdown/00-matrix-governance/module-tree.md"
$moduleTreeContent = Read-RepoText $moduleTreePath
$requiredModules = @(
    "system",
    "system.entry",
    "system.entry.launch_scripts",
    "system.entry.backend_process",
    "system.desktop_shell.tauri_runtime",
    "system.desktop_shell.tauri_config",
    "system.desktop_shell.assets_schema",
    "system.runtime_profile.config_examples",
    "system.build_delivery.workspace_manifest",
    "system.build_delivery.desktop_build_scripts",
    "system.build_delivery.container_proxy",
    "system.build_delivery.ci_release",
    "backend",
    "backend.router",
    "backend.capability",
    "backend.strategy_config",
    "backend.interface_boundary",
    "frontend.workspace",
    "backend.runtime",
    "backend.graph_compile",
    "backend.storage_security",
    "backend.ops_governance",
    "backend.app_state_wiring",
    "backend.test_support",
    "frontend.strategy_hub",
    "frontend.capability_projection",
    "frontend.runtime_panels",
    "executor.state",
    "executor.runner",
    "docs.matrix_governance",
    "docs.feature_tree"
)

foreach ($moduleId in $requiredModules) {
    $section = Get-ModuleSection $moduleTreeContent $moduleId
    if ([string]::IsNullOrWhiteSpace($section)) {
        Add-Failure "$moduleTreePath missing module node: $moduleId"
        continue
    }

    $moduleChecks = @(
        @($T.RealFile, "real files"),
        @("public", "public methods"),
        @($T.ParentCommunicationRule, "parent communication rule"),
        @($T.RegressionProtection, "regression protection")
    )
    foreach ($check in $moduleChecks) {
        if (-not $section.Contains($check[0])) {
            Add-Failure "$moduleTreePath module $moduleId missing $($check[1])"
        }
    }
}

$pathPattern = "``([^``\s]+\.(rs|js|jsx|ts|tsx|md|ps1|bat|toml|json|yaml|yml|html|css))``"
$pathMatches = [regex]::Matches($moduleTreeContent, $pathPattern)
$seenPaths = @{}
foreach ($match in $pathMatches) {
    $rawPath = $match.Groups[1].Value
    if ($rawPath.Contains("*")) {
        continue
    }

    $candidate = $rawPath.Replace("\", "/")
    if ($candidate.StartsWith("./")) {
        $candidate = $candidate.Substring(2)
    }
    if ($candidate.StartsWith("path/to/")) {
        continue
    }
    if ($candidate -match "^[a-zA-Z]+://") {
        continue
    }
    if ($candidate.StartsWith("../")) {
        continue
    }
    if ($seenPaths.ContainsKey($candidate)) {
        continue
    }
    $seenPaths[$candidate] = $true

    if (-not (Test-Path -LiteralPath (Join-Root $candidate) -PathType Leaf)) {
        Add-Failure "$moduleTreePath references missing file: $candidate"
    }
}

$proposalDirs = @(
    "markdown/06-milestones/v4.12.0",
    "markdown/06-milestones/v4.13.0",
    "markdown/06-milestones/v4.14.0",
    "markdown/06-milestones/v4.15.0",
    "markdown/06-milestones/v4.16.0"
)

foreach ($dir in $proposalDirs) {
    $files = @(Get-ChildItem -LiteralPath (Join-Root $dir) -Filter "*.md" -File -ErrorAction SilentlyContinue)
    foreach ($file in $files) {
        $relative = Get-RelativePath $file.FullName
        $content = Get-Content -LiteralPath $file.FullName -Raw -Encoding UTF8

        if (-not $content.Contains($T.ExecutionTier)) {
            continue
        }

        $tierLine = ($content -split "`r?`n" | Where-Object { $_.Contains($T.ExecutionTier) } | Select-Object -First 1)
        $isStandardOrHeavy = $tierLine.Contains($T.Standard) -or $tierLine.Contains($T.Heavy)
        $isHeavy = $tierLine.Contains($T.Heavy)

        if ($isStandardOrHeavy -and
            (-not ($content.Contains($T.ThreeMatrixImpactStatement) -or $content.Contains($T.ImpactMatrixStatement)))) {
            Add-Failure "$relative missing matrix impact statement for non-lightweight proposal"
        }

        if ($isHeavy) {
            $requiredCoordinates = @(
                @($T.GuidanceCoordinate, "guidance coordinate"),
                @($T.FullFeatureTreeNode, "full feature tree node"),
                @($T.ModuleTreeNode, "module tree node"),
                @($T.RealFile, "real file"),
                @("public", "public method"),
                @($T.Test, "test")
            )
            foreach ($check in $requiredCoordinates) {
                if (-not $content.Contains($check[0])) {
                    Add-Failure "$relative missing heavy-tier $($check[1])"
                }
            }
        }
    }
}

if ($Failures.Count -gt 0) {
    Write-Host "Matrix governance check FAILED:"
    foreach ($failure in $Failures) {
        Write-Host " - $failure"
    }
    exit 1
}

Write-Host "Matrix governance check passed."
exit 0
