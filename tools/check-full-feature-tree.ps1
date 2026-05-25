# check-full-feature-tree.ps1
# QuantPilot full feature tree gate.
# Checks: stale placeholders, version header, path existence, active file coverage,
# required sections, path format.

param(
    [string]$TreeFile = "markdown/10-overview/overview-full-feature-tree.md",
    [string]$ExcludeFile = "tools/full-feature-tree-excludes.txt",
    [string]$RepoRoot = "."
)

$ErrorActionPreference = "Stop"
$script:Failures = @()

function Add-Failure {
    param([string]$Message)
    $script:Failures += $Message
    Write-Host "  [FAIL] $Message" -ForegroundColor Red
}

function Add-Pass {
    param([string]$Message)
    Write-Host "  [PASS] $Message" -ForegroundColor Green
}

function Normalize-RepoPath {
    param([string]$Path)
    return ($Path -replace "\\", "/").TrimStart("./")
}

function Get-RepoRelativePath {
    param([System.IO.FileInfo]$File)
    $rootPath = (Resolve-Path $RepoRoot).Path
    return (Normalize-RepoPath ($File.FullName.Substring($rootPath.Length + 1)))
}

function Test-Excluded {
    param([string]$Path, [string[]]$Patterns)
    foreach ($pattern in $Patterns) {
        if ($Path -like $pattern) {
            return $true
        }
    }
    return $false
}

function Add-Files {
    param(
        [string]$Root,
        [string[]]$Extensions,
        [ref]$Collector
    )

    $fullRoot = Join-Path $RepoRoot $Root
    if (-not (Test-Path $fullRoot)) {
        return
    }

    Get-ChildItem -LiteralPath $fullRoot -Recurse -ErrorAction SilentlyContinue |
        Where-Object { -not $_.PSIsContainer -and ($Extensions -contains $_.Extension.ToLowerInvariant()) } |
        ForEach-Object { $Collector.Value += (Get-RepoRelativePath $_) }
}

function Assert-NoMatch {
    param([string]$Description, [string]$Pattern, [string[]]$Lines)

    $matches = $Lines | Select-String -Pattern $Pattern
    if ($matches) {
        Add-Failure $Description
        foreach ($match in $matches) {
            Write-Host "    $($match.Line.Trim())" -ForegroundColor DarkYellow
        }
    }
    else {
        Add-Pass $Description
    }
}

$repoResolved = (Resolve-Path $RepoRoot).Path
$treePath = Join-Path $RepoRoot $TreeFile
if (-not (Test-Path -LiteralPath $treePath)) {
    Add-Failure "tree file missing: $TreeFile"
    exit 1
}

$treeContent = Get-Content -LiteralPath $treePath -Encoding UTF8
$treeText = $treeContent -join "`n"
Write-Host "[INFO] tree file: $TreeFile ($($treeContent.Count) lines)" -ForegroundColor Cyan

$excludes = @()
$excludePath = Join-Path $RepoRoot $ExcludeFile
if (Test-Path -LiteralPath $excludePath) {
    $excludes = Get-Content -LiteralPath $excludePath -Encoding UTF8 |
        Where-Object { $_ -notmatch '^\s*(#|$)' } |
        ForEach-Object { Normalize-RepoPath $_ }
}
Write-Host "[INFO] exclude rules: $($excludes.Count)" -ForegroundColor Cyan

Write-Host "`n=== Step 1: stale placeholder check ===" -ForegroundColor Yellow
Assert-NoMatch "no TODO token" '\bTODO\b' $treeContent
Assert-NoMatch "no FIXME token" '\bFIXME\b' $treeContent
$pendingShort = -join ([char[]](24453, 34917))
$pendingLong = -join ([char[]](24453, 23436, 21892))
Assert-NoMatch "no Chinese pending marker: dai-bu" $pendingShort $treeContent
Assert-NoMatch "no Chinese pending marker: dai-wan-shan" $pendingLong $treeContent

$cargoToml = Get-Content -LiteralPath (Join-Path $RepoRoot "Cargo.toml") -Encoding UTF8 -Raw
$cargoVersionMatch = [regex]::Match($cargoToml, '(?m)^\s*version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
    Add-Failure "Cargo.toml package version missing"
}
$expectedTreeVersion = $cargoVersionMatch.Groups[1].Value

Write-Host "`n=== Step 2: version header check ===" -ForegroundColor Yellow
if ($treeContent.Count -eq 0 -or $treeContent[0] -notmatch [regex]::Escape("v$expectedTreeVersion")) {
    Add-Failure "tree title must include v$expectedTreeVersion"
}
else {
    Add-Pass "tree title includes v$expectedTreeVersion"
}

Write-Host "`n=== Step 3: explicit path existence check ===" -ForegroundColor Yellow
$pathPattern = "``([^``]+\.(rs|js|jsx|ts|tsx|md|ps1|bat|toml|json|yaml|yml|html|css))``"
$pathMatches = [regex]::Matches($treeText, $pathPattern)
$explicitPaths = @()
foreach ($match in $pathMatches) {
    $path = Normalize-RepoPath $match.Groups[1].Value
    if ($path -match '\*') {
        continue
    }
    if ($path -match '^(src|src-executor|frontend|frontend-executor|qrpc_|quantscript|tools|scripts|tests|contracts|config|release|plugins|src-tauri|markdown|Cargo\.|start\.|Dockerfile|docker-compose\.|nginx\.|\.env|\.git)') {
        $explicitPaths += $path
    }
}
$explicitPaths = $explicitPaths | Sort-Object -Unique

$missingPaths = @()
foreach ($path in $explicitPaths) {
    if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot ($path -replace "/", "\")))) {
        $missingPaths += $path
    }
}
if ($missingPaths.Count -gt 0) {
    Add-Failure "explicit path references missing: $($missingPaths.Count)"
    $missingPaths | Sort-Object | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkYellow }
}
else {
    Add-Pass "all explicit path references exist ($($explicitPaths.Count))"
}

Write-Host "`n=== Step 4: active file coverage check ===" -ForegroundColor Yellow
$activeFiles = @()
Add-Files "src" @(".rs") ([ref]$activeFiles)
Add-Files "src-executor" @(".rs") ([ref]$activeFiles)
Add-Files "qrpc_core/src" @(".rs") ([ref]$activeFiles)
Add-Files "qrpc_core_ir/src" @(".rs") ([ref]$activeFiles)
Add-Files "qrpc_compiler/src" @(".rs") ([ref]$activeFiles)
Add-Files "qrpc_runtime/src" @(".rs") ([ref]$activeFiles)
Add-Files "qrpc_session/src" @(".rs") ([ref]$activeFiles)
Add-Files "quantscript/src" @(".rs") ([ref]$activeFiles)
Add-Files "src-tauri" @(".rs", ".json", ".toml", ".bat") ([ref]$activeFiles)
Add-Files "frontend/src" @(".js", ".jsx", ".css", ".json") ([ref]$activeFiles)
Add-Files "frontend/tests" @(".js", ".jsx") ([ref]$activeFiles)
Add-Files "frontend-executor/src" @(".js", ".jsx", ".css") ([ref]$activeFiles)
Add-Files "tools" @(".ps1", ".bat", ".js") ([ref]$activeFiles)
Add-Files "scripts" @(".ps1", ".bat", ".js") ([ref]$activeFiles)
Add-Files "tests" @(".rs", ".json") ([ref]$activeFiles)
Add-Files "contracts" @(".yaml", ".yml") ([ref]$activeFiles)
Add-Files "config" @(".yaml", ".yml", ".json") ([ref]$activeFiles)
Add-Files "release" @(".yaml", ".yml") ([ref]$activeFiles)
Add-Files "plugins" @(".json") ([ref]$activeFiles)

$rootFiles = @(
    "Cargo.toml",
    "start.bat",
    "start.ps1",
    "Dockerfile",
    "docker-compose.yml",
    "nginx.conf",
    ".env.example",
    ".gitignore",
    ".gitattributes"
)
foreach ($rootFile in $rootFiles) {
    if (Test-Path -LiteralPath (Join-Path $RepoRoot $rootFile)) {
        $activeFiles += $rootFile
    }
}

$activeFiles = $activeFiles |
    Where-Object { $_ -notmatch '(^|/)node_modules/|(^|/)dist/|(^|/)target/' } |
    Sort-Object -Unique
$excludedFiles = @()
$uncoveredFiles = @()
foreach ($file in $activeFiles) {
    if (Test-Excluded $file $excludes) {
        $excludedFiles += $file
        continue
    }

    $covered = ($treeText -match [regex]::Escape($file))
    if (-not $covered) {
        $uncoveredFiles += $file
    }
}

Write-Host "[INFO] active files: $($activeFiles.Count)" -ForegroundColor Cyan
Write-Host "[INFO] excluded active files: $($excludedFiles.Count)" -ForegroundColor Cyan
if ($uncoveredFiles.Count -gt 0) {
    Add-Failure "active files not covered by exact repo-relative path: $($uncoveredFiles.Count)"
    $uncoveredFiles | Sort-Object | Select-Object -First 260 | ForEach-Object {
        Write-Host "    $_" -ForegroundColor DarkYellow
    }
    if ($uncoveredFiles.Count -gt 260) {
        Write-Host "    ... plus $($uncoveredFiles.Count - 260) more" -ForegroundColor DarkYellow
    }
}
else {
    Add-Pass "all non-excluded active files are covered by exact repo-relative path"
}

Write-Host "`n=== Step 5: required section check ===" -ForegroundColor Yellow
$requiredSections = @(
    @{ Name = "chapter 0"; Pattern = "(?m)^## 0\." },
    @{ Name = "coverage scope"; Pattern = "(?m)^### 0\.3" },
    @{ Name = "maintenance rules"; Pattern = "(?m)^### 0\.4" },
    @{ Name = "node label spec"; Pattern = "(?m)^### 0\.5" },
    @{ Name = "root 1"; Pattern = "(?m)^## .+1:" },
    @{ Name = "root 2"; Pattern = "(?m)^## .+2:" },
    @{ Name = "root 3"; Pattern = "(?m)^## .+3:" },
    @{ Name = "root 4"; Pattern = "(?m)^## .+4:" },
    @{ Name = "root 5"; Pattern = "(?m)^## .+5:" },
    @{ Name = "root 6"; Pattern = "(?m)^## .+6:" },
    @{ Name = "root 7"; Pattern = "(?m)^## .+7:" },
    @{ Name = "appendix E"; Pattern = "(?m)^## .+ E:" },
    @{ Name = "appendix F"; Pattern = "(?m)^## .+ F:" }
)
foreach ($section in $requiredSections) {
    if ($treeText -match $section.Pattern) {
        Add-Pass $section.Name
    }
    else {
        Add-Failure "required section missing: $($section.Name)"
    }
}

Write-Host "`n=== Step 6: path format check ===" -ForegroundColor Yellow
$badPathLines = $treeContent | Select-String -Pattern '^- `[^`]+\.(rs|js|jsx|ps1|bat|toml|json|yaml|yml|html|css)`' |
    Where-Object {
        $_.Line -notmatch '^- `(src|src-executor|frontend|frontend-executor|qrpc_|quantscript|tools|scripts|tests|contracts|config|release|plugins|src-tauri|Cargo\.|start\.|Dockerfile|docker-compose\.|nginx\.|\.env|\.git|markdown)'
    }
if ($badPathLines) {
    Add-Failure "possible context-relative file paths found: $($badPathLines.Count)"
    $badPathLines | Select-Object -First 30 | ForEach-Object {
        Write-Host "    $($_.Line.Trim())" -ForegroundColor DarkYellow
    }
}
else {
    Add-Pass "file path bullets use repo-relative prefixes"
}

Write-Host "`n=== Step 7: hard-coded line count check ===" -ForegroundColor Yellow
$hanLine = [char]34892
$hanOrdinalPrefix = [char]31532
$lineCountPattern = "([0-9]+|~[0-9]+)\s*$hanLine|$hanOrdinalPrefix\s*[0-9]+\s*$hanLine"
$lineCountMatches = $treeContent | Select-String -Pattern $lineCountPattern
if ($lineCountMatches) {
    Add-Failure "hard-coded line counts are not allowed: $($lineCountMatches.Count)"
    $lineCountMatches | Select-Object -First 40 | ForEach-Object {
        Write-Host "    $($_.Line.Trim())" -ForegroundColor DarkYellow
    }
}
else {
    Add-Pass "no hard-coded line counts"
}

Write-Host "`n========================================" -ForegroundColor Cyan
if ($script:Failures.Count -eq 0) {
    Write-Host "Full feature tree check: PASS" -ForegroundColor Green
    exit 0
}

Write-Host "Full feature tree check: FAIL ($($script:Failures.Count))" -ForegroundColor Red
foreach ($failure in $script:Failures) {
    Write-Host "  $failure" -ForegroundColor Red
}
exit 1
