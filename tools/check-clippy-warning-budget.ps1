param(
    [int]$MaxWarnings = 58
)

$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

$output = & cargo clippy --workspace --all-targets 2>&1 | ForEach-Object {
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
    Write-Host "workspace clippy failed with exit code $exitCode" -ForegroundColor Red
    exit $exitCode
}

if ($warningCount -gt $MaxWarnings) {
    Write-Host "workspace clippy warning budget exceeded: $warningCount > $MaxWarnings" -ForegroundColor Red
    exit 1
}

Write-Host "workspace clippy warning budget: $warningCount/$MaxWarnings" -ForegroundColor Green
exit 0
