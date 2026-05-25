param(
    [string]$OutputDir = "storage\audit",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

$repoRoot = Split-Path -Parent $PSScriptRoot
$schemaVersion = "quantpilot/gate-metrics/v1"

$gateDefinitions = @(
    @{
        name = "utf8-check"
        command = "powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1"
        working_dir = $null
    },
    @{
        name = "cargo-check"
        command = "cargo check --workspace"
        working_dir = $null
    },
    @{
        name = "cargo-test-no-run"
        command = "cargo test --workspace --no-run"
        working_dir = $null
    },
    @{
        name = "frontend-build"
        command = "npm run build"
        working_dir = "frontend"
    },
    @{
        name = "frontend-test"
        command = "npx vitest run"
        working_dir = "frontend"
    },
    @{
        name = "npm-audit"
        command = "npm audit --audit-level=moderate"
        working_dir = "frontend"
    }
)

function Get-ShortError {
    param([string]$Text)

    if ([string]::IsNullOrWhiteSpace($Text)) {
        return ""
    }

    $normalized = ($Text -replace "\s+", " ").Trim()
    if ($normalized.Length -gt 400) {
        return $normalized.Substring(0, 400) + "..."
    }

    return $normalized
}

function Test-GateDefinition {
    param(
        [hashtable]$Gate,
        [int]$Index
    )

    $errors = @()

    if ([string]::IsNullOrWhiteSpace($Gate.name)) {
        $errors += "gate[$Index] has empty name"
    }

    if ([string]::IsNullOrWhiteSpace($Gate.command)) {
        $errors += "gate[$Index] has empty command"
    }

    if ($Gate.working_dir) {
        $resolvedWorkingDir = Join-Path $repoRoot $Gate.working_dir
        if (-not (Test-Path -LiteralPath $resolvedWorkingDir -PathType Container)) {
            $errors += "gate[$Index] working_dir does not exist: $($Gate.working_dir)"
        }
    }

    return $errors
}

function Measure-Gate {
    param(
        [hashtable]$Gate
    )

    Write-Host "=== $($Gate.name) ===" -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $success = $false
    $exitCode = 0
    $errorText = ""
    $pushed = $false

    try {
        if ($Gate.working_dir) {
            Push-Location (Join-Path $repoRoot $Gate.working_dir)
            $pushed = $true
        } else {
            Push-Location $repoRoot
            $pushed = $true
        }

        $global:LASTEXITCODE = 0
        $result = Invoke-Expression $Gate.command 2>&1
        $exitCode = if ($null -eq $global:LASTEXITCODE) { 0 } else { $global:LASTEXITCODE }
        $success = ($exitCode -eq 0)
        if (-not $success) {
            $errorText = Get-ShortError ($result | Out-String)
        }
    } catch {
        $exitCode = 1
        $errorText = Get-ShortError $_.Exception.Message
    } finally {
        if ($pushed) {
            Pop-Location
        }
        $sw.Stop()
    }

    $statusText = if ($success) { "PASS" } else { "FAIL" }
    $statusColor = if ($success) { "Green" } else { "Red" }
    Write-Host ("  {0}: {1}ms {2}" -f $Gate.name, $sw.ElapsedMilliseconds, $statusText) -ForegroundColor $statusColor

    return @{
        name = $Gate.name
        elapsed_ms = $sw.ElapsedMilliseconds
        success = $success
        exit_code = $exitCode
        error = $errorText
    }
}

$definitionErrors = @()
for ($i = 0; $i -lt $gateDefinitions.Count; $i++) {
    $definitionErrors += Test-GateDefinition -Gate $gateDefinitions[$i] -Index $i
}

if ($definitionErrors.Count -gt 0) {
    Write-Host "Gate metrics definition check failed:" -ForegroundColor Red
    $definitionErrors | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}

if ($DryRun) {
    $dryRunRecord = @{
        schema_version = $schemaVersion
        mode = "dry-run"
        recorded_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
        gates = $gateDefinitions | ForEach-Object {
            @{
                name = $_.name
                command = $_.command
                working_dir = $_.working_dir
            }
        }
        summary = @{
            total = $gateDefinitions.Count
            validated = $gateDefinitions.Count
        }
    }

    $null = $dryRunRecord | ConvertTo-Json -Depth 8 -Compress
    Write-Host "Gate metrics DryRun passed: $($gateDefinitions.Count) gate definitions validated." -ForegroundColor Green
    exit 0
}

$overallSw = [System.Diagnostics.Stopwatch]::StartNew()
$allGates = @()
foreach ($gateDefinition in $gateDefinitions) {
    $allGates += Measure-Gate -Gate $gateDefinition
}
$overallSw.Stop()

$passed = @($allGates | Where-Object { $_.success }).Count
$failed = @($allGates | Where-Object { -not $_.success }).Count
$metrics = @{
    schema_version = $schemaVersion
    mode = "record"
    recorded_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    total_elapsed_ms = $overallSw.ElapsedMilliseconds
    gates = $allGates
    summary = @{
        passed = $passed
        failed = $failed
        total = $allGates.Count
    }
}

if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

$outputFile = Join-Path $OutputDir "gate-metrics.ndjson"
$record = $metrics | ConvertTo-Json -Depth 8 -Compress
[System.IO.File]::AppendAllText(
    $outputFile,
    $record + [Environment]::NewLine,
    [System.Text.UTF8Encoding]::new($false)
)

$summaryColor = if ($failed -eq 0) { "Green" } else { "Red" }
Write-Host ""
Write-Host ("Gate metrics recorded: {0}/{1} passed, {2}ms total." -f $passed, $allGates.Count, $metrics.total_elapsed_ms) -ForegroundColor $summaryColor
Write-Host "Record appended to: $outputFile"

if ($failed -gt 0) {
    exit 1
}
