param(
    [string]$SnapshotPath = "markdown\implementation\governance\implementation-capability-governance-registry.generated.md",
    [switch]$WriteSnapshot
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$renderScript = Join-Path $PSScriptRoot "render-capability-governance.mjs"
$resolvedSnapshotPath = if ([System.IO.Path]::IsPathRooted($SnapshotPath)) {
    $SnapshotPath
} else {
    Join-Path $repoRoot $SnapshotPath
}

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    throw "Node.js is required to render capability governance snapshots."
}

if ($WriteSnapshot) {
    & node --experimental-specifier-resolution=node $renderScript --write $resolvedSnapshotPath
} else {
    & node --experimental-specifier-resolution=node $renderScript --check $resolvedSnapshotPath
}

if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

if ($WriteSnapshot) {
    Write-Host ("Capability governance snapshot updated: {0}" -f $resolvedSnapshotPath) -ForegroundColor Green
} else {
    Write-Host ("Capability governance snapshot check passed: {0}" -f $resolvedSnapshotPath) -ForegroundColor Green
}

