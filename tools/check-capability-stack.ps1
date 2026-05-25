param()

$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

$repoRoot = Split-Path -Parent $PSScriptRoot
$failures = New-Object System.Collections.Generic.List[string]

function Read-Text {
    param([string]$Path)
    return [System.IO.File]::ReadAllText((Join-Path $repoRoot $Path), [System.Text.UTF8Encoding]::new($false))
}

function Read-JsonFile {
    param([string]$Path)
    return (Read-Text $Path) | ConvertFrom-Json
}

function Get-RustStringArray {
    param(
        [string]$Content,
        [string]$ConstName
    )

    $pattern = "const\s+$ConstName\s*:\s*\[&str;\s*(?<count>\d+)\]\s*=\s*\[(?<body>.*?)\];"
    $match = [regex]::Match($Content, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
    if (-not $match.Success) {
        $failures.Add("missing Rust const array: $ConstName")
        return @()
    }

    $expectedCount = [int]$match.Groups["count"].Value
    $values = @([regex]::Matches($match.Groups["body"].Value, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })
    if ($values.Count -ne $expectedCount) {
        $failures.Add("$ConstName declares $expectedCount entries but contains $($values.Count)")
    }

    return $values
}

function Get-JsStringArray {
    param(
        [string]$Content,
        [string]$ConstName
    )

    $pattern = "export\s+const\s+$ConstName\s*=\s*\[(?<body>.*?)\];"
    $match = [regex]::Match($Content, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
    if (-not $match.Success) {
        $failures.Add("missing JS const array: $ConstName")
        return @()
    }

    return @([regex]::Matches($match.Groups["body"].Value, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })
}

function Get-DefaultCapabilitySchemaHash {
    param([string]$Content)

    $match = [regex]::Match($Content, 'schema_hash:\s*"([^"]+)"')
    if (-not $match.Success) {
        $failures.Add("missing DEFAULT_CAPABILITIES.schema_hash")
        return ""
    }

    return $match.Groups[1].Value
}

function Assert-Equal {
    param(
        [string]$Name,
        [object]$Actual,
        [object]$Expected
    )

    if ([string]$Actual -ne [string]$Expected) {
        $failures.Add("$Name mismatch: actual=[$Actual], expected=[$Expected]")
    }
}

function Assert-ArrayEqual {
    param(
        [string]$Name,
        [string[]]$Actual,
        [string[]]$Expected
    )

    if ($Actual.Count -ne $Expected.Count) {
        $failures.Add("$Name count mismatch: actual=$($Actual.Count), expected=$($Expected.Count)")
        return
    }

    for ($i = 0; $i -lt $Expected.Count; $i++) {
        if ($Actual[$i] -ne $Expected[$i]) {
            $failures.Add("$Name entry[$i] mismatch: actual=[$($Actual[$i])], expected=[$($Expected[$i])]")
            return
        }
    }
}

$trackGateMetrics = Join-Path $PSScriptRoot "track-gate-metrics.ps1"
& powershell -NoProfile -ExecutionPolicy Bypass -File $trackGateMetrics -DryRun -OutputDir (Join-Path $repoRoot "target\capability-stack-dryrun")
if ($LASTEXITCODE -ne 0) {
    $failures.Add("track-gate-metrics.ps1 DryRun failed")
}

$backendFixture = Read-JsonFile "frontend\src\test\fixtures\capabilities\backend-capabilities-v1.json"
$runtimeFixture = Read-JsonFile "tests\fixtures\runtime\minimal_runtime_request.json"
$rustLib = Read-Text "src\lib.rs"
$supportMatrix = Read-Text "frontend\src\capabilities\supportMatrix.js"
$builtinModules = Read-Text "frontend\src\modules\builtinModules.js"

$fixtureDeclaredKeys = [string[]]@($backendFixture.frontend.declared_module_keys)
$fixtureSupportedKeys = [string[]]@($backendFixture.frontend.supported_module_keys)
$fixtureModuleSupportKeys = [string[]]@($backendFixture.frontend.module_support | ForEach-Object { $_.module_key })
$rustDeclaredKeys = [string[]](Get-RustStringArray -Content $rustLib -ConstName "DECLARED_FRONTEND_MODULE_KEYS")
$rustSupportedKeys = [string[]](Get-RustStringArray -Content $rustLib -ConstName "SUPPORTED_FRONTEND_MODULE_KEYS")
$frontendSupportedKeys = [string[]](Get-JsStringArray -Content $supportMatrix -ConstName "SUPPORTED_FRONTEND_MODULE_KEYS")
$defaultSchemaHash = Get-DefaultCapabilitySchemaHash -Content $builtinModules

Assert-Equal "backend fixture schema_hash vs runtime fixture capability_context.schema_hash" `
    $runtimeFixture.capability_context.schema_hash `
    $backendFixture.schema_hash
Assert-Equal "backend fixture schema_hash vs DEFAULT_CAPABILITIES.schema_hash" `
    $defaultSchemaHash `
    $backendFixture.schema_hash

Assert-ArrayEqual "backend declared_module_keys vs supported_module_keys" $fixtureDeclaredKeys $fixtureSupportedKeys
Assert-ArrayEqual "backend module_support keys vs supported_module_keys" $fixtureModuleSupportKeys $fixtureSupportedKeys
Assert-ArrayEqual "src/lib.rs DECLARED_FRONTEND_MODULE_KEYS vs backend declared_module_keys" $rustDeclaredKeys $fixtureDeclaredKeys
Assert-ArrayEqual "src/lib.rs SUPPORTED_FRONTEND_MODULE_KEYS vs backend supported_module_keys" $rustSupportedKeys $fixtureSupportedKeys
Assert-ArrayEqual "frontend supportMatrix SUPPORTED_FRONTEND_MODULE_KEYS vs backend supported_module_keys" $frontendSupportedKeys $fixtureSupportedKeys

if ($failures.Count -gt 0) {
    Write-Host "Capability stack check failed:" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}

Write-Host "Capability stack check passed: schema hash, module keys, backend fixture, frontend projection, and meta DryRun are aligned." -ForegroundColor Green
