param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

if (-not $CargoArgs -or $CargoArgs.Count -eq 0) {
    $CargoArgs = @("test", "--workspace")
}

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

$processNames = @(
    "quantpilot",
    "quantpilot-tauri",
    "executor"
)

foreach ($name in $processNames) {
    Get-Process -Name $name -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -like "*\target\*" -or $_.Path -like "*\quantpilot*" } |
        Stop-Process -Force -ErrorAction SilentlyContinue
}

Start-Sleep -Milliseconds 500

Write-Host "cargo $($CargoArgs -join ' ')" -ForegroundColor Cyan
& cargo @CargoArgs
exit $LASTEXITCODE
