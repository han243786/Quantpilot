param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$sourceHook = Join-Path $repoRoot "scripts\pre-commit"
$installedHook = (& git -C $repoRoot rev-parse --git-path hooks/pre-commit).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($installedHook)) {
    throw "Unable to resolve installed pre-commit hook path through git rev-parse."
}
if (-not [System.IO.Path]::IsPathRooted($installedHook)) {
    $installedHook = Join-Path $repoRoot $installedHook
}

function Read-NormalizedUtf8File([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Required pre-commit hook file is missing: $Path"
    }

    $content = [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8)
    return $content -replace "`r`n", "`n"
}

$expected = Read-NormalizedUtf8File $sourceHook
$actual = Read-NormalizedUtf8File $installedHook

if ($expected -ne $actual) {
    throw (
        "Installed pre-commit hook is stale. " +
        "Run: `$hook = git rev-parse --git-path hooks/pre-commit; " +
        "Copy-Item -LiteralPath scripts\pre-commit -Destination `$hook -Force"
    )
}

Write-Host "Pre-commit hook check passed: installed hook matches scripts\pre-commit" -ForegroundColor Green
