param()

$ErrorActionPreference = "Stop"

function New-UnicodeFragment {
    param(
        [int[]]$CodePoints
    )

    return [string]::Concat(($CodePoints | ForEach-Object { [char]$_ }))
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$tmpRoot = Join-Path $repoRoot "target\quality-gate-smoke"

if (Test-Path -LiteralPath $tmpRoot) {
    Remove-Item -LiteralPath $tmpRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $tmpRoot | Out-Null

$utf8BadFile = Join-Path $tmpRoot "utf8-bom.md"
$textBadFile = Join-Path $tmpRoot "user-facing-mojibake.jsx"
$capabilitySnapshotFile = Join-Path $tmpRoot "capability-governance.generated.md"
$positiveClaimBadFile = Join-Path $tmpRoot "README.md"
$mojibake = New-UnicodeFragment @(0x9365, 0x70B4, 0x7974)

try {
    $utf8Bytes = [byte[]](0xEF, 0xBB, 0xBF) + [System.Text.Encoding]::UTF8.GetBytes("# bad bom`n")
    [System.IO.File]::WriteAllBytes($utf8BadFile, $utf8Bytes)

    [System.IO.File]::WriteAllText(
        $textBadFile,
        "export const badLabel = `"$mojibake`";`n",
        [System.Text.UTF8Encoding]::new($false)
    )

    [System.IO.File]::WriteAllText(
        $positiveClaimBadFile,
        "This beta README says live execution is supported.`n",
        [System.Text.UTF8Encoding]::new($false)
    )

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-utf8.ps1") -Paths @($utf8BadFile)
    if ($LASTEXITCODE -eq 0) {
        throw "Expected check-utf8.ps1 to fail for BOM sample."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-user-facing-text.ps1") -Paths @($textBadFile)
    if ($LASTEXITCODE -eq 0) {
        throw "Expected check-user-facing-text.ps1 to fail for mojibake sample."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-user-facing-text.ps1") -Paths @($positiveClaimBadFile) -PositiveClaimAuditPaths @($positiveClaimBadFile)
    if ($LASTEXITCODE -eq 0) {
        throw "Expected check-user-facing-text.ps1 to fail for a non-whitelisted positive support claim."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-capability-governance.ps1") -SnapshotPath $capabilitySnapshotFile -WriteSnapshot
    if ($LASTEXITCODE -ne 0) {
        throw "Expected check-capability-governance.ps1 to generate a fresh snapshot."
    }

    Add-Content -LiteralPath $capabilitySnapshotFile -Value "`n<!-- drift -->"

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-capability-governance.ps1") -SnapshotPath $capabilitySnapshotFile
    if ($LASTEXITCODE -eq 0) {
        throw "Expected check-capability-governance.ps1 to fail for a drifted snapshot."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "track-gate-metrics.ps1") -DryRun -OutputDir $tmpRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Expected track-gate-metrics.ps1 DryRun to pass."
    }

    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "check-capability-stack.ps1")
    if ($LASTEXITCODE -ne 0) {
        throw "Expected check-capability-stack.ps1 to pass for the current capability stack."
    }

    Write-Host "Quality gate smoke passed: encoding, text, capability, and meta-pipeline gates behaved as expected." -ForegroundColor Green
}
finally {
    if (Test-Path -LiteralPath $tmpRoot) {
        Remove-Item -LiteralPath $tmpRoot -Recurse -Force
    }
}
