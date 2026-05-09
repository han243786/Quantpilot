# QuantPilot i18n check — scans Rust source for English user-facing strings
# Usage: powershell -File tools/check-i18n.ps1
# Exit 0 = clean, Exit 1 = violations found

$ErrorActionPreference = "Stop"
$violations = 0

$srcDirs = @("src", "quantscript/src", "qrpc_core/src", "qrpc_core_ir/src", "qrpc_compiler/src", "qrpc_runtime/src")

# Patterns that are English user-facing messages
# Only flag bail!/anyhow!/Err( with English content — these are the user-visible strings
$englishPatterns = @(
    'bail!\("[A-Za-z][a-z\s]{10,}',
    'anyhow!\("[A-Za-z][a-z\s]{10,}',
    'Err\("[A-Za-z][a-z\s]{10,}'
)

# Exceptions: diagnostic codes, identifiers, file paths
$exceptions = @(
    "QS0001", "QS0002", "QS0003", "QS0004", "QS0005", "QS0006",
    "QS0401", "QS0402", "QS0501", "QS0601", "QS0602", "QS0603",
    "QS0604", "QS0605", "QS0606", "QS0607", "QS0608", "QS0609",
    "QPQSLOW001", "QPQSLOW002", "QPQSLOW003", "QPQSLOW004",
    "QPQSLOW005", "QPQSLOW006", "QPQSLOW007", "QPQSLOW008",
    "QPQSLOW009", "QPQSLOW010", "QPQSLOW011", "QPQSLOW012",
    "QPQSLOW013", "QPQSLOW014", "QPQSLOW015", "QPQSLOW016",
    "QPQSLOW017", "QPQSLOW018", "QPQSLOW019", "QPQSLOW020",
    "QPQSLOW021", "QPQSLOW022", "QPQSLOW023", "QPQSLOW024",
    "QPQSLOW025", "QPQSLOW026", "QPQSLOW027", "QPQSLOW028",
    "CUSTOM001", "CUSTOM002", "CUSTOM003", "CUSTOM004", "CUSTOM005",
    "QPSTRATSPREAD001", "QPSTRATSPREAD004",
    "QPSTRATJSON001",
    "capability_gated", "runtime_compile_failed", "strategy_ir_compile_failed",
    "quantscript_compile_failed", "quantscript_lowering_failed",
    "sha256:", "trace-", "backtest_", "quantpilot/", "builtin."
)

Write-Host "=== QuantPilot i18n Check ===" -ForegroundColor Cyan

foreach ($dir in $srcDirs) {
    if (-not (Test-Path $dir)) { continue }
    $files = Get-ChildItem -Path $dir -Filter "*.rs" -Recurse -File

    foreach ($file in $files) {
        $lines = Get-Content $file.FullName -Raw
        if (-not $lines) { continue }

        foreach ($pattern in $englishPatterns) {
            $matches = [regex]::Matches($lines, $pattern)
            foreach ($m in $matches) {
                $text = $m.Value
                $isException = $false
                foreach ($ex in $exceptions) {
                    if ($text -match $ex) { $isException = $true; break }
                }
                if (-not $isException) {
                    $lineNo = ($lines.Substring(0, $m.Index).Split("`n").Count)
                    Write-Host "  VIOLATION: $($file.FullName):$lineNo" -ForegroundColor Red
                    Write-Host "    $text" -ForegroundColor Yellow
                    $violations++
                }
            }
        }
    }
}

Write-Host ""
if ($violations -eq 0) {
    Write-Host "PASS: No English user-facing strings found." -ForegroundColor Green
    exit 0
} else {
    Write-Host "FAIL: $violations English string(s) found." -ForegroundColor Red
    exit 1
}
