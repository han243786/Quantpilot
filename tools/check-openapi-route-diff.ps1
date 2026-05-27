param(
    [switch]$FailOnDiff
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$openApiPath = Join-Path $repoRoot "contracts/openapi/root.yaml"

function Normalize-RoutePath {
    param([string]$Path)
    return ($Path -replace ":([A-Za-z_][A-Za-z0-9_]*)", '{$1}')
}

function Get-RustRoutePaths {
    param([string[]]$Roots)

    $paths = New-Object System.Collections.Generic.HashSet[string]
    foreach ($root in $Roots) {
        $fullRoot = Join-Path $repoRoot $root
        if (-not (Test-Path $fullRoot)) { continue }
        Get-ChildItem -LiteralPath $fullRoot -Recurse -Filter *.rs | ForEach-Object {
            $content = Get-Content -LiteralPath $_.FullName -Raw -Encoding UTF8
            [regex]::Matches($content, '\.route\(\s*"([^"]+)"') | ForEach-Object {
                $route = $_.Groups[1].Value
                if ($route.StartsWith("/api/")) {
                    $normalized = Normalize-RoutePath $route
                    if ($normalized -ne "/api/*path" -and $normalized -ne "/api/test") {
                        [void]$paths.Add($normalized)
                    }
                }
            }
        }
    }
    return $paths
}

function Get-OpenApiPaths {
    param([string]$Path)

    $paths = New-Object System.Collections.Generic.HashSet[string]
    $content = Get-Content -LiteralPath $Path -Raw -Encoding UTF8
    [regex]::Matches($content, '(?m)^\s{2}(/api[^:]+):\s*$') | ForEach-Object {
        [void]$paths.Add($_.Groups[1].Value.Trim())
    }
    return $paths
}

$codePaths = Get-RustRoutePaths @("src", "src-executor")
$specPaths = Get-OpenApiPaths $openApiPath

$missingInOpenApi = @($codePaths | Where-Object { -not $specPaths.Contains($_) } | Sort-Object)
$missingInCode = @($specPaths | Where-Object { -not $codePaths.Contains($_) } | Sort-Object)

Write-Host "OpenAPI route diff baseline"
Write-Host "  code paths: $($codePaths.Count)"
Write-Host "  spec paths: $($specPaths.Count)"

if ($missingInOpenApi.Count -gt 0) {
    Write-Host ""
    Write-Host "Implemented but not in OpenAPI:"
    $missingInOpenApi | ForEach-Object { Write-Host "  - $_" }
}

if ($missingInCode.Count -gt 0) {
    Write-Host ""
    Write-Host "OpenAPI paths not found in route registration:"
    $missingInCode | ForEach-Object { Write-Host "  - $_" }
}

if ($missingInOpenApi.Count -eq 0 -and $missingInCode.Count -eq 0) {
    Write-Host "  diff: clean"
}

if ($FailOnDiff -and ($missingInOpenApi.Count -gt 0 -or $missingInCode.Count -gt 0)) {
    throw "OpenAPI route diff is not clean"
}
