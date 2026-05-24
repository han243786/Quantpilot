param(
    [int]$MaxWarnings = 0
)

$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

$output = & cargo check --bin executor 2>&1 | ForEach-Object {
    if ($_ -is [System.Management.Automation.ErrorRecord]) {
        $_.Exception.Message
    } else {
        $_.ToString()
    }
}
$exitCode = $LASTEXITCODE
$warningCount = (
    $output |
        Where-Object { $_ -match '^warning:' -and $_ -notmatch ' generated \d+ warnings' } |
        Measure-Object
).Count

$output | ForEach-Object { Write-Host $_ }

if ($exitCode -ne 0) {
    Write-Host "executor check failed with exit code $exitCode" -ForegroundColor Red
    exit $exitCode
}

if ($warningCount -gt $MaxWarnings) {
    Write-Host "executor warning budget exceeded: $warningCount > $MaxWarnings" -ForegroundColor Red
    exit 1
}

Write-Host "executor warning budget: $warningCount/$MaxWarnings" -ForegroundColor Green
exit 0
