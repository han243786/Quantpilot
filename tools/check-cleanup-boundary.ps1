[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $scriptRoot ".."))
$cleanupScript = Join-Path $scriptRoot "cleanup-artifacts.ps1"

if (-not (Test-Path -LiteralPath $cleanupScript)) {
  throw "cleanup-artifacts.ps1 not found."
}

$scriptText = Get-Content -Raw -Encoding UTF8 -LiteralPath $cleanupScript
$forbiddenPatterns = @(
  'Join-Path\s+\$storageRootPath\s+["'']runs["'']',
  'Join-Path\s+\$storageRootPath\s+["'']backtests["'']',
  'Join-Path\s+\$storageRootPath\s+["'']experiments["'']',
  'Join-Path\s+\$storageRootPath\s+["'']graphs["'']',
  'Category\s*=\s*["'']runtime-artifact-dir["'']',
  'Category\s*=\s*["'']graph-snapshot-file["'']',
  'Category\s*=\s*["'']graph-version-dir["'']'
)

$violations = New-Object System.Collections.Generic.List[string]
foreach ($pattern in $forbiddenPatterns) {
  if ($scriptText -match $pattern) {
    $violations.Add("cleanup script still contains protected artifact enumeration pattern: $pattern")
  }
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("quantpilot_cleanup_boundary_" + [System.Guid]::NewGuid().ToString("N"))
$tmpFull = [System.IO.Path]::GetFullPath($tmpRoot)
$tempBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\') + '\'
if (-not $tmpFull.StartsWith($tempBase, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Refusing to create cleanup boundary fixture outside temp: $tmpFull"
}

try {
  New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null
  foreach ($name in @("test-old", "runs", "backtests", "experiments", "graphs", "graphs\versions")) {
    New-Item -ItemType Directory -Force -Path (Join-Path $tmpRoot $name) | Out-Null
  }
  foreach ($file in @("graphs\latest.json", "graphs\strategy.qs", "graphs\versions\1.json", "runs\run.json", "backtests\bt.json", "experiments\exp.json")) {
    Set-Content -LiteralPath (Join-Path $tmpRoot $file) -Encoding UTF8 -NoNewline -Value "{}"
  }
  $old = (Get-Date).AddDays(-30)
  Get-ChildItem -LiteralPath $tmpRoot -Recurse -Force | ForEach-Object {
    $_.LastWriteTime = $old
  }

  $dryRunOutput = & $cleanupScript -StorageRoot $tmpRoot -OlderThanDays 7 -Mode dry-run
  if ($LASTEXITCODE -ne 0) {
    throw "cleanup dry-run exited with code $LASTEXITCODE"
  }
  $dryRunText = ($dryRunOutput | Out-String)
  if ($dryRunText -notmatch '\[test-artifact-dir\].*test-old') {
    $violations.Add("cleanup dry-run did not include the expected test-old artifact.")
  }
  foreach ($protected in @("runs", "backtests", "experiments", "graphs")) {
    if ($dryRunText -match [regex]::Escape((Join-Path $tmpRoot $protected))) {
      $violations.Add("cleanup dry-run listed protected artifact path: $protected")
    }
  }

  $flagRejected = $false
  try {
    & $cleanupScript -StorageRoot $tmpRoot -OlderThanDays 7 -Mode dry-run -IncludeRuntimeArtifacts | Out-Null
  } catch {
    $flagRejected = $true
  }
  if (-not $flagRejected) {
    $violations.Add("-IncludeRuntimeArtifacts was not rejected.")
  }

  & $cleanupScript -StorageRoot $tmpRoot -OlderThanDays 7 -Mode execute | Out-Null
  foreach ($protected in @("runs", "backtests", "experiments", "graphs", "graphs\versions", "graphs\latest.json")) {
    if (-not (Test-Path -LiteralPath (Join-Path $tmpRoot $protected))) {
      $violations.Add("cleanup execute removed protected artifact path: $protected")
    }
  }
  if (Test-Path -LiteralPath (Join-Path $tmpRoot "test-old")) {
    $violations.Add("cleanup execute did not remove the old test artifact directory.")
  }
} finally {
  if ($tmpFull.StartsWith($tempBase, [System.StringComparison]::OrdinalIgnoreCase) -and (Test-Path -LiteralPath $tmpRoot)) {
    Remove-Item -LiteralPath $tmpRoot -Recurse -Force
  }
}

if ($violations.Count -gt 0) {
  Write-Host "FAIL: cleanup boundary check found $($violations.Count) issue(s)." -ForegroundColor Red
  foreach ($violation in $violations) {
    Write-Host " - $violation" -ForegroundColor Red
  }
  exit 1
}

Write-Host "PASS: cleanup-artifacts.ps1 only targets temporary test artifacts/logs and rejects retired runtime cleanup." -ForegroundColor Green
