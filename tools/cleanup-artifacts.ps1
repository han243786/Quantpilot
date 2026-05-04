[CmdletBinding()]
param(
  [string]$StorageRoot = "",
  [ValidateSet("dry-run", "execute")]
  [string]$Mode = "dry-run",
  [int]$OlderThanDays = 7,
  [switch]$IncludeLogs,
  [switch]$IncludeRuntimeArtifacts
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-NormalizedPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  $resolved = Resolve-Path -LiteralPath $Path
  return [System.IO.Path]::GetFullPath($resolved.Path)
}

if ([string]::IsNullOrWhiteSpace($StorageRoot)) {
  $scriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
  $StorageRoot = Join-Path $scriptDirectory "..\storage"
}

function Test-IsUnderRoot {
  param(
    [Parameter(Mandatory = $true)][string]$RootPath,
    [Parameter(Mandatory = $true)][string]$CandidatePath
  )

  $normalizedRoot = [System.IO.Path]::GetFullPath($RootPath).TrimEnd('\') + '\'
  $normalizedCandidate = [System.IO.Path]::GetFullPath($CandidatePath)
  return $normalizedCandidate.StartsWith($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase)
}

function New-CleanupEntry {
  param(
    [Parameter(Mandatory = $true)]$Item,
    [Parameter(Mandatory = $true)][string]$Category
  )

  [PSCustomObject]@{
    Category = $Category
    Path = [System.IO.Path]::GetFullPath($Item.FullName)
    LastWriteTime = $Item.LastWriteTime
  }
}

$storageRootPath = Resolve-NormalizedPath -Path $StorageRoot
$cutoff = (Get-Date).AddDays(-$OlderThanDays)
$targets = New-Object System.Collections.Generic.List[object]

$testArtifactDirs = Get-ChildItem -LiteralPath $storageRootPath -Directory |
  Where-Object {
    $_.Name -like "test-*"
  } |
  Where-Object { $_.LastWriteTime -lt $cutoff }

foreach ($dir in $testArtifactDirs) {
  $targets.Add((New-CleanupEntry -Item $dir -Category "test-artifact-dir"))
}

if ($IncludeLogs) {
  $logFiles = Get-ChildItem -LiteralPath $storageRootPath -File |
    Where-Object { $_.Extension -eq ".log" -or $_.Name -like "*.err.log" -or $_.Name -like "*.out.log" } |
    Where-Object { $_.LastWriteTime -lt $cutoff }

  foreach ($file in $logFiles) {
    $targets.Add((New-CleanupEntry -Item $file -Category "log-file"))
  }
}

if ($IncludeRuntimeArtifacts) {
  foreach ($name in @("runs", "backtests", "experiments")) {
    $dirPath = Join-Path $storageRootPath $name
    if (Test-Path -LiteralPath $dirPath) {
      $dir = Get-Item -LiteralPath $dirPath
      if ($dir.LastWriteTime -lt $cutoff) {
        $targets.Add((New-CleanupEntry -Item $dir -Category "runtime-artifact-dir"))
      }
    }
  }

  $auditPath = Join-Path $storageRootPath "audit"
  if (Test-Path -LiteralPath $auditPath) {
    Get-ChildItem -LiteralPath $auditPath -File -Filter "*.json" |
      Where-Object { $_.LastWriteTime -lt $cutoff } |
      ForEach-Object {
        $targets.Add((New-CleanupEntry -Item $_ -Category "audit-artifact-file"))
      }
  }

  $graphsPath = Join-Path $storageRootPath "graphs"
  if (Test-Path -LiteralPath $graphsPath) {
    Get-ChildItem -LiteralPath $graphsPath -File |
      Where-Object { ($_.Extension -eq ".json" -or $_.Extension -eq ".qs") -and $_.LastWriteTime -lt $cutoff } |
      ForEach-Object {
        $targets.Add((New-CleanupEntry -Item $_ -Category "graph-snapshot-file"))
      }

    $versionsPath = Join-Path $graphsPath "versions"
    if (Test-Path -LiteralPath $versionsPath) {
      $versionsDir = Get-Item -LiteralPath $versionsPath
      if ($versionsDir.LastWriteTime -lt $cutoff) {
        $targets.Add((New-CleanupEntry -Item $versionsDir -Category "graph-version-dir"))
      }
    }
  }
}

foreach ($target in $targets) {
  if (-not (Test-IsUnderRoot -RootPath $storageRootPath -CandidatePath $target.Path)) {
    throw "Refusing to operate outside the storage root: $($target.Path)"
  }
}

if ($targets.Count -eq 0) {
  Write-Output "No cleanup candidates found under $storageRootPath older than $OlderThanDays day(s)."
  exit 0
}

Write-Output "Cleanup mode: $Mode"
Write-Output "Storage root: $storageRootPath"
Write-Output "Cutoff: $($cutoff.ToString("s"))"
Write-Output ""
Write-Output "Candidates:"
$targets |
  Sort-Object Category, Path |
  ForEach-Object {
    Write-Output ("- [{0}] {1} (last_write={2})" -f $_.Category, $_.Path, $_.LastWriteTime.ToString("s"))
  }

if ($Mode -eq "dry-run") {
  Write-Output ""
  Write-Output "Dry-run only. Re-run with -Mode execute to delete the candidates listed above."
  exit 0
}

foreach ($target in $targets) {
  if (Test-Path -LiteralPath $target.Path) {
    if ((Get-Item -LiteralPath $target.Path) -is [System.IO.DirectoryInfo]) {
      Remove-Item -LiteralPath $target.Path -Recurse -Force
    } else {
      Remove-Item -LiteralPath $target.Path -Force
    }
  }
}

Write-Output ""
Write-Output ("Deleted {0} artifact target(s)." -f $targets.Count)
