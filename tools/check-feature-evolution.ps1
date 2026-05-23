$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

function Read-Utf8($path) {
    Get-Content -Raw -Encoding UTF8 $path
}

# Keep this script ASCII-only so Windows PowerShell 5.1 can parse it without a UTF-8 BOM.
function U($base64) {
    [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($base64))
}

function Add-Failure {
    param(
        [System.Collections.Generic.List[string]]$Failures,
        [string]$Message
    )

    $Failures.Add($Message) | Out-Null
}

function Assert-FileContains {
    param(
        [System.Collections.Generic.List[string]]$Failures,
        [string]$Path,
        [string]$Pattern,
        [string]$Description
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        Add-Failure $Failures "$Description missing: $Path"
        return
    }

    $content = Read-Utf8 $Path
    if (-not ([regex]::IsMatch($content, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline))) {
        Add-Failure $Failures "$Description not found in $Path"
    }
}

function Get-MarkdownSection {
    param(
        [string]$Content,
        [string]$HeadingPattern
    )

    $lines = $Content -split "`r?`n"
    $capture = $false
    $section = New-Object System.Collections.Generic.List[string]
    foreach ($line in $lines) {
        if (-not $capture) {
            if ([regex]::IsMatch($line, $HeadingPattern)) {
                $capture = $true
            }
            continue
        }
        if ([regex]::IsMatch($line, '^##\s+')) {
            break
        }
        $section.Add($line) | Out-Null
    }
    [string]::Join("`n", $section)
}

function Read-CargoVersion {
    $content = Read-Utf8 "Cargo.toml"
    $match = [regex]::Match($content, '(?m)^\s*version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "version not found in Cargo.toml"
    }
    $match.Groups[1].Value
}

$failures = New-Object System.Collections.Generic.List[string]

$contract = "markdown/03-implementation/governance/implementation-feature-evolution-contract.md"
Assert-FileContains $failures $contract (U "XiMjIOS4ieOAgeWKn+iDvea8lOi/m+eZu+iusA==") "feature evolution registration section"
Assert-FileContains $failures $contract (U "XiMjIOWbm+OAgeWbnuW9kuS/neaKpOefqemYtQ==") "regression protection matrix section"
Assert-FileContains $failures $contract (U "XiMjIOWFreOAgemYsuWbnumAgOinhOWImQ==") "anti-regression rules section"
Assert-FileContains $failures $contract 'tools/check-feature-evolution\.ps1' "tooling reference"

$requiredScenarios = @(
    "tests/scenarios/scenario_01_btc_dual_ma.qs",
    "tests/scenarios/scenario_02_backtest_compare.qs",
    "tests/scenarios/scenario_08_multi_symbol.qs"
)

foreach ($scenario in $requiredScenarios) {
    if (-not (Test-Path -LiteralPath $scenario)) {
        Add-Failure $failures "core regression scenario missing: $scenario"
    }
}

$version = Read-CargoVersion
$milestoneDir = "markdown/06-milestones/v$version"
if (-not (Test-Path -LiteralPath $milestoneDir)) {
    Add-Failure $failures "current milestone directory missing: $milestoneDir"
} else {
    $planPath = Join-Path $milestoneDir (U "MDEt6KeE5YiS5pa55qGILm1k")
    if (-not (Test-Path -LiteralPath $planPath)) {
        Add-Failure $failures "current milestone plan missing: $planPath"
    } else {
        $plan = Read-Utf8 $planPath
        $nonGoals = Get-MarkdownSection $plan (U "XiMjIOmdnuebruaghw==")
        $declaresNoFeatureExpansion = $nonGoals -match (U "5LiN5paw5aKe5Yqf6IO9fOS4jeaWsOWinuWKn+iDveeJueaAp3zkuI3mianlpKflip/og73ojIPlm7R85LiN5omp5aSn6IO95Yqb6IyD5Zu0")

        if ($declaresNoFeatureExpansion) {
            Write-Host "PASS: v$version declares no feature expansion." -ForegroundColor Green
        } else {
            $requiredSections = @(
                @{ Pattern = (U "XiMjIOWKn+iDvea8lOi/m+eZu+iusA=="); Name = "feature evolution registration" },
                @{ Pattern = (U "XiMjIOWbnuW9kuS/neaKpOefqemYtQ=="); Name = "regression protection matrix" },
                @{ Pattern = (U "XiMjIOWFvOWuueaAp+S4jui/geenuw=="); Name = "compatibility and migration" },
                @{ Pattern = (U "XiMjIOmdnuebruaghw=="); Name = "non-goals" }
            )

            foreach ($section in $requiredSections) {
                if (-not ([regex]::IsMatch($plan, $section.Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline))) {
                    Add-Failure $failures "feature milestone v$version missing section: $($section.Name)"
                }
            }
        }
    }
}

$superSpec = "markdown/01-principles/principles-super-standardization.md"
Assert-FileContains $failures $superSpec (U "5Yqf6IO95ryU6L+b6YCa6YGT") "super-standardization feature evolution rule"
Assert-FileContains $failures $superSpec 'check-feature-evolution\.ps1' "super-standardization gate entry"

$gp = "markdown/General_Policy.md"
Assert-FileContains $failures $gp (U "5Yqf6IO95ryU6L+b5b+F6aG75YWI55m76K6w") "General_Policy feature evolution rule"
Assert-FileContains $failures $gp (U "5Yqf6IO95ryU6L+b55m76K6w") "General_Policy feature registry wording"

if ($failures.Count -gt 0) {
    Write-Host "Feature evolution check failed:" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "- $_" }
    exit 1
}

Write-Host "Feature evolution check passed for v$version." -ForegroundColor Green
exit 0
