param(
    [int]$Port = 3000,
    [string[]]$Scenarios = @(
        "tests/scenarios/scenario_01_btc_dual_ma.qs",
        "tests/scenarios/scenario_02_backtest_compare.qs",
        "tests/scenarios/scenario_08_multi_symbol.qs"
    )
)

$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Write-Host "Building backend for scenario smoke..." -ForegroundColor Cyan
& cargo build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Get-Process -Name quantpilot -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\target\*" } |
    Stop-Process -Force -ErrorAction SilentlyContinue

$env:QUANTPILOT_PORT = [string]$Port
$env:QUANTPILOT_DEV = "true"
$env:QUANTPILOT_API_KEY = ""

$exe = Join-Path $root "target\debug\quantpilot.exe"
if (-not (Test-Path $exe)) {
    Write-Error "Backend binary not found: $exe"
}

Write-Host "Starting backend on 127.0.0.1:$Port..." -ForegroundColor Cyan
$proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden

try {
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        try {
            Invoke-RestMethod "http://127.0.0.1:$Port/api/health" -TimeoutSec 2 | Out-Null
            $ready = $true
            break
        } catch {
            Start-Sleep -Seconds 1
        }
    }

    if (-not $ready) {
        Write-Error "Backend did not become ready on port $Port"
    }

    $failed = 0
    foreach ($scenario in $Scenarios) {
        Write-Host "=== Scenario: $scenario ===" -ForegroundColor Cyan
        & node tools/run-scenario.js $scenario
        if ($LASTEXITCODE -ne 0) {
            $failed += 1
        }
    }

    if ($failed -gt 0) {
        Write-Error "$failed scenario smoke test(s) failed"
    }
} finally {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}

Write-Host "Scenario smoke passed: $($Scenarios.Count)/$($Scenarios.Count)" -ForegroundColor Green
