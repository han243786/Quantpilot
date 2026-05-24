$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

$status = git status --short
if ($LASTEXITCODE -ne 0) {
    Write-Host "git status failed" -ForegroundColor Red
    exit $LASTEXITCODE
}

if ($status) {
    Write-Host "worktree is not clean after closeout gates:" -ForegroundColor Red
    $status | ForEach-Object { Write-Host $_ }
    exit 1
}

Write-Host "worktree clean"
exit 0
