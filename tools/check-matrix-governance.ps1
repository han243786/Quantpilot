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
    @("markdown/06-milestones/v4.16.0/09-*.md", "v4.16 system entry extraction record")
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
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/03-", "module tree v4.16 backend register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/04-", "module tree v4.16 frontend register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/05-", "module tree v4.16 test asset register"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/06-", "module tree v4.16 backend interface boundary plan"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/07-", "module tree v4.16 top module statistics"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/08-", "module tree v4.16 system module split statistics"),
    @("markdown/00-matrix-governance/module-tree.md", "markdown/06-milestones/v4.16.0/09-", "module tree v4.16 system entry extraction record"),
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
    "system.entry",
    "system.entry.backend_process",
    "backend.router",
    "backend.capability",
    "backend.strategy_config",
    "backend.interface_boundary",
    "frontend.workspace",
    "backend.runtime",
    "backend.graph_compile",
    "backend.storage_security",
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
