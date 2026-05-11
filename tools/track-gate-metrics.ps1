# QuantPilot 元流水线 — 门禁耗时追踪 (§7.2)
# 记录每次 CI 门禁的耗时、通过/失败状态

param(
    [string]$OutputDir = "storage\audit"
)

$ErrorActionPreference = "Continue"
$now = Get-Date -Format "yyyy-MM-ddTHH:mm:ss"
$outputFile = Join-Path $OutputDir "gate-metrics.json"
$metrics = @{
    recorded_at = $now
    gates = @()
}

function Measure-Gate {
    param(
        [string]$Name,
        [string]$Command,
        [string]$WorkingDir = $null
    )
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $success = $false
    $errorMsg = ""
    try {
        if ($WorkingDir) {
            Push-Location $WorkingDir
        }
        $result = Invoke-Expression $Command 2>&1
        $exitCode = $LASTEXITCODE
        if ($exitCode -eq 0) {
            $success = $true
        } else {
            $errorMsg = "$result" | Out-String
        }
        if ($WorkingDir) {
            Pop-Location
        }
    } catch {
        $errorMsg = $_.Exception.Message
    }
    $sw.Stop()

    $gate = @{
        name = $Name
        elapsed_ms = $sw.ElapsedMilliseconds
        success = $success
        error = if ($errorMsg.Length -gt 200) { $errorMsg.Substring(0, 200) + "..." } else { $errorMsg }
    }
    Write-Host "  ${Name}: $($sw.ElapsedMilliseconds)ms $($if($success){'PASS'}else{'FAIL'})" -ForegroundColor $(if($success){'Green'}else{'Red'})
    return $gate
}

# 仅记录模式: 运行收口门禁但不阻断
$allGates = @()
$allGates += Measure-Gate "utf8-check" "powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1"
$allGates += Measure-Gate "cargo-check" "cargo check --workspace"
$allGates += Measure-Gate "cargo-test-no-run" "cargo test --workspace --no-run"
$allGates += Measure-Gate "frontend-build" "npm run build" "frontend"
$allGates += Measure-Gate "frontend-test" "npx vitest run" "frontend"
$allGates += Measure-Gate "npm-audit" "npm audit --audit-level=moderate" "frontend"

$metrics.gates = $allGates
$metrics.total_elapsed_ms = ($allGates | Measure-Object -Property elapsed_ms -Sum).Sum
$passed = ($allGates | Where-Object { $_.success }).Count
$failed = ($allGates | Where-Object { -not $_.success }).Count
$metrics.summary = @{
    passed = $passed
    failed = $failed
    total = $allGates.Count
}

# 确保输出目录存在
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# 追加到 JSON 数组 (简单追加, 不完整 JSON 解析)
$record = $metrics | ConvertTo-Json -Compress
Add-Content -Path $outputFile -Value $record

Write-Host ""
Write-Host "门禁追踪: ${passed}/${($allGates.Count)} 通过, $($metrics.total_elapsed_ms)ms 总耗时" -ForegroundColor $(if($failed -eq 0){'Green'}else{'Red'})
Write-Host "记录写入: $outputFile"
