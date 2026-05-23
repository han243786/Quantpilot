$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

# Keep this script ASCII-only so Windows PowerShell 5.1 can parse it without a UTF-8 BOM.
function U($base64) {
    [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($base64))
}

function Read-Utf8($path) {
    Get-Content -Raw -Encoding UTF8 $path
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

$failures = New-Object System.Collections.Generic.List[string]

$gitignore = Read-Utf8 ".gitignore"
if ($gitignore -notmatch '(?m)^/markdown/learning/$') {
    Add-Failure $failures ".gitignore must ignore /markdown/learning/."
}

$trackedLearning = & git ls-files -- "markdown/learning"
if ($trackedLearning) {
    Add-Failure $failures "markdown/learning contains tracked files: $($trackedLearning -join ', ')"
}

$checkIgnored = & git check-ignore --quiet "markdown/learning/"
if ($LASTEXITCODE -ne 0) {
    Add-Failure $failures "git check-ignore does not ignore markdown/learning/."
}

$learningDoc = "markdown/03-implementation/governance/implementation-developer-learning-pipeline.md"
Assert-FileContains $failures $learningDoc 'markdown/learning/' "local learning note boundary"
Assert-FileContains $failures $learningDoc 'based_on_commit' "learning staleness metadata"
Assert-FileContains $failures $learningDoc (U "5pys54mI5pys5piv5ZCm5byV5YWl5LqG6ZyA6KaB6aG555uuIG93bmVyIOWtpuS5oOeahOaWsOaguOW/g+acuuWItu+8nw==") "major closeout learning question"
Assert-FileContains $failures $learningDoc (U "6K6w5b2V5pys6L2u5a2m5Lmg") "explicit learning write command"

$v4Plan = Join-Path "markdown/06-milestones/v4.0.0" (U "MDEt6KeE5YiS5pa55qGILm1k")
Assert-FileContains $failures $v4Plan (U "IyMg5aSn54mI5pysIGNsb3Nlb3V0IOWtpuS5oOajgOafpemhuQ==") "v4 major learning closeout section"
Assert-FileContains $failures $v4Plan 'markdown/learning/' "v4 local learning note boundary"

$superSpec = "markdown/01-principles/principles-super-standardization.md"
Assert-FileContains $failures $superSpec 'Developer Learning Closeout' "super-standardization learning closeout rule"
Assert-FileContains $failures $superSpec 'markdown/learning/' "super-standardization local learning note boundary"

$closeoutGate = "tools/run-closeout-gates.bat"
Assert-FileContains $failures $closeoutGate 'check-learning-closeout\.ps1' "closeout learning gate wiring"

if ($failures.Count -gt 0) {
    Write-Host "Learning closeout check failed:" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "- $_" }
    exit 1
}

Write-Host "Learning closeout check passed." -ForegroundColor Green
exit 0
