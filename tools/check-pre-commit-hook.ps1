param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$sourceHook = Join-Path $repoRoot "scripts\pre-commit"
$installedHook = Join-Path $repoRoot ".git\hooks\pre-commit"

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
        "Run: Copy-Item -LiteralPath scripts\pre-commit -Destination .git\hooks\pre-commit -Force"
    )
}

Write-Host "Pre-commit hook check passed: .git\hooks\pre-commit matches scripts\pre-commit" -ForegroundColor Green
