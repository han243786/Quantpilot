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

if ($IncludeRuntimeArtifacts) {
  throw "-IncludeRuntimeArtifacts has been retired. Cleanup is limited to explicit temporary test artifacts and logs; persisted runtime data must be managed by product lifecycle APIs."
}

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
