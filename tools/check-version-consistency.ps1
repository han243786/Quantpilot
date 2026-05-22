$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

function Read-JsonFile($path) {
    Get-Content -Raw -Encoding UTF8 $path | ConvertFrom-Json
}

function Read-JsonTopLevelVersion($path) {
    $content = Get-Content -Raw -Encoding UTF8 $path
    $match = [regex]::Match($content, '(?m)^\s*"version"\s*:\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "top-level version not found in $path"
    }
    $match.Groups[1].Value
}

function Read-PackageLockPackageVersion($path) {
    $content = Get-Content -Raw -Encoding UTF8 $path
    $match = [regex]::Match($content, '(?s)"packages"\s*:\s*\{\s*""\s*:\s*\{.*?"version"\s*:\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "root package version not found in $path"
    }
    $match.Groups[1].Value
}

function Read-TomlPackageVersion($path) {
    $content = Get-Content -Raw -Encoding UTF8 $path
    $match = [regex]::Match($content, '(?m)^\s*version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "version not found in $path"
    }
    $match.Groups[1].Value
}

$expected = Read-TomlPackageVersion "Cargo.toml"
$checks = @(
    @{ Name = "Cargo.toml"; Value = $expected },
    @{ Name = "Cargo.lock quantpilot"; Value = ((Select-String -Path "Cargo.lock" -Pattern 'name = "quantpilot"' -Context 0,1).Context.PostContext[0] -replace 'version = "|"', '').Trim() },
    @{ Name = "src-tauri/Cargo.toml"; Value = Read-TomlPackageVersion "src-tauri/Cargo.toml" },
    @{ Name = "Cargo.lock quantpilot-tauri"; Value = ((Select-String -Path "Cargo.lock" -Pattern 'name = "quantpilot-tauri"' -Context 0,1).Context.PostContext[0] -replace 'version = "|"', '').Trim() },
    @{ Name = "src-tauri/tauri.conf.json"; Value = (Read-JsonFile "src-tauri/tauri.conf.json").version },
    @{ Name = "frontend/package.json"; Value = (Read-JsonFile "frontend/package.json").version },
    @{ Name = "frontend/package-lock.json"; Value = Read-JsonTopLevelVersion "frontend/package-lock.json" },
    @{ Name = "frontend/package-lock.json packages['']"; Value = Read-PackageLockPackageVersion "frontend/package-lock.json" },
    @{ Name = "frontend-executor/package.json"; Value = (Read-JsonFile "frontend-executor/package.json").version },
    @{ Name = "frontend-executor/package-lock.json"; Value = Read-JsonTopLevelVersion "frontend-executor/package-lock.json" },
    @{ Name = "frontend-executor/package-lock.json packages['']"; Value = Read-PackageLockPackageVersion "frontend-executor/package-lock.json" }
)

$failed = 0
foreach ($check in $checks) {
    if ($check.Value -ne $expected) {
        Write-Host "FAIL: $($check.Name) version '$($check.Value)' != '$expected'" -ForegroundColor Red
        $failed += 1
    } else {
        Write-Host "PASS: $($check.Name) = $expected" -ForegroundColor Green
    }
}

$textFiles = @(
    "CHANGELOG.md",
    "README.md",
    "frontend-executor/index.html",
    "frontend-executor/dist/index.html",
    "markdown/README.md",
    "markdown/01-principles/README.md",
    "markdown/01-principles/principles-super-standardization.md",
    "markdown/06-milestones/README.md",
    "markdown/10-overview/overview-current-status-and-roadmap.md"
)

foreach ($file in $textFiles) {
    $content = Get-Content -Raw -Encoding UTF8 $file
    if ($content -notmatch [regex]::Escape("v$expected")) {
        Write-Host "FAIL: $file does not mention v$expected" -ForegroundColor Red
        $failed += 1
    } else {
        Write-Host "PASS: $file mentions v$expected" -ForegroundColor Green
    }
}

if ($failed -gt 0) {
    Write-Host "Version consistency failed: $failed issue(s)" -ForegroundColor Red
    exit 1
}

Write-Host "Version consistency passed: $expected" -ForegroundColor Green
