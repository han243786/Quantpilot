param(
  [string]$OutputPath = "frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json"
)

$ErrorActionPreference = "Stop"

$startMarker = "__CAPABILITY_FIXTURE_START__"
$endMarker = "__CAPABILITY_FIXTURE_END__"

$previousErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$commandOutput = & cargo test export_capability_fixture_json_snapshot -- --ignored --nocapture 2>&1
$ErrorActionPreference = $previousErrorActionPreference
$text = ($commandOutput | Out-String)

$startIndex = $text.IndexOf($startMarker)
$endIndex = $text.IndexOf($endMarker)

if ($startIndex -lt 0 -or $endIndex -lt 0 -or $endIndex -le $startIndex) {
  throw "Failed to extract capability fixture JSON from cargo test output."
}

$jsonStart = $startIndex + $startMarker.Length
$hex = $text.Substring($jsonStart, $endIndex - $jsonStart).Trim()
if (($hex.Length % 2) -ne 0) {
  throw "Capability fixture payload is not valid hex."
}

$bytes = [byte[]]::new($hex.Length / 2)
for ($i = 0; $i -lt $hex.Length; $i += 2) {
  $bytes[$i / 2] = [Convert]::ToByte($hex.Substring($i, 2), 16)
}

$json = [System.Text.Encoding]::UTF8.GetString($bytes)
$parsed = $json | ConvertFrom-Json
$normalizedJson = $parsed | ConvertTo-Json -Depth 10

$output = Join-Path (Get-Location) $OutputPath
$outputDir = Split-Path -Parent $output
if (-not (Test-Path $outputDir)) {
  New-Item -ItemType Directory -Path $outputDir | Out-Null
}

[System.IO.File]::WriteAllText(
  $output,
  $normalizedJson + [Environment]::NewLine,
  [System.Text.UTF8Encoding]::new($false)
)

Write-Host "Capability fixture written to $output"
