param(
    [string[]]$Paths = @("frontend/src", "frontend/index.html", "src/main.rs", "markdown"),
    [string[]]$Extensions = @(".js", ".jsx", ".ts", ".tsx", ".css", ".md", ".html", ".rs"),
    [string[]]$PositiveClaimAuditPaths = @()
)

$ErrorActionPreference = "Stop"

function Get-CapabilityTextGatePolicy {
    $renderScript = Join-Path $PSScriptRoot "render-capability-governance.mjs"

    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        throw "Node.js is required to load capability text-gate policy."
    }

    $json = & node --experimental-specifier-resolution=node $renderScript --text-gates-json
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to load capability text-gate policy from capability governance registry."
    }

    return $json | ConvertFrom-Json
}

function Get-TargetFiles {
    param(
        [string[]]$InputPaths,
        [string[]]$WantedExtensions
    )

    $result = New-Object System.Collections.Generic.List[System.IO.FileInfo]
    foreach ($path in $InputPaths) {
        if (-not (Test-Path -LiteralPath $path)) {
            continue
        }

        $item = Get-Item -LiteralPath $path
        if ($item.PSIsContainer) {
            Get-ChildItem -LiteralPath $item.FullName -Recurse -File | Where-Object {
                $WantedExtensions -contains $_.Extension.ToLowerInvariant()
            } | ForEach-Object {
                $result.Add($_)
            }
        } elseif ($WantedExtensions -contains $item.Extension.ToLowerInvariant()) {
            $result.Add($item)
        }
    }

    return $result | Sort-Object FullName -Unique
}

function Add-Finding {
    param(
        [System.Collections.Generic.List[object]]$Findings,
        [string]$Path,
        [int]$Line,
        [string]$Kind,
        [string]$Detail
    )

    $Findings.Add([pscustomobject]@{
            Path   = $Path
            Line   = $Line
            Kind   = $Kind
            Detail = $Detail
        })
}

function Normalize-RepoRelativePath {
    param(
        [string]$RepoRoot,
        [string]$FullPath
    )

    $normalizedRoot = [System.IO.Path]::GetFullPath($RepoRoot)
    $normalizedPath = [System.IO.Path]::GetFullPath($FullPath)

    if ($normalizedPath.StartsWith($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        $relative = $normalizedPath.Substring($normalizedRoot.Length).TrimStart('\', '/')
        return ($relative -replace '\\', '/')
    }

    return ($normalizedPath -replace '\\', '/')
}

function New-UnicodeFragment {
    param(
        [int[]]$CodePoints
    )

    return [string]::Concat(($CodePoints | ForEach-Object { [char]$_ }))
}

$findings = New-Object System.Collections.Generic.List[object]
$repoRoot = Split-Path -Parent $PSScriptRoot
$files = Get-TargetFiles -InputPaths $Paths -WantedExtensions $Extensions
$utf8 = [System.Text.UTF8Encoding]::new($false, $true)
$replacementChar = [char]0xFFFD
$knownMojibakeFragments = @(
    (New-UnicodeFragment @(0x9365, 0x70B4, 0x7974)),
    (New-UnicodeFragment @(0x7EDB, 0x682B, 0x6690)),
    (New-UnicodeFragment @(0x6769, 0x612F, 0xE511)),
    (New-UnicodeFragment @(0x7F02, 0x682C, 0x7627)),
    (New-UnicodeFragment @(0x5A34, 0x5B2D, 0x762F)),
    (New-UnicodeFragment @(0x7487, 0xFE3D, 0x510F)),
    (New-UnicodeFragment @(0x6769, 0x65BF, 0x6D16)),
    (New-UnicodeFragment @(0x5BEE, 0x20AC, 0x6FEE)),
    (New-UnicodeFragment @(0x7F01, 0x64B4, 0x6F6B)),
    (New-UnicodeFragment @(0x95B0, 0x5D87, 0x7586, 0x935D, 0x581D, 0x7B07)),
    (New-UnicodeFragment @(0x93C8, 0x20AC, 0x7F01, 0x581F, 0x6F48, 0x9429)),
    (New-UnicodeFragment @(0x9359, 0x5C7D, 0x6F4E, 0x7EFE)),
    (New-UnicodeFragment @(0x59DD, 0xFF45, 0x6E6A, 0x9354, 0x72BA, 0x6D47)),
    (New-UnicodeFragment @(0x9429, 0xE1BC, 0x7D8D))
)
$capabilityTextGatePolicy = Get-CapabilityTextGatePolicy
$allowedClaims = @($capabilityTextGatePolicy.allowedClaims | ForEach-Object { $_.value })
$allowedClaimPhrases = @($capabilityTextGatePolicy.allowedClaims | ForEach-Object { $_.approvedPhrase })
$forbiddenOverstatementChecks = @(
    $capabilityTextGatePolicy.disallowedClaims | ForEach-Object {
        @{
            Pattern               = $_.forbiddenPattern
            Detail                = $_.detail
            AllowedContextPattern = if ([string]::IsNullOrWhiteSpace($_.allowedContextPattern)) {
                'must not|must not appear|disallowedClaims'
            } else {
                $_.allowedContextPattern
            }
        }
    }
)
$positiveClaimAuditConfig = $capabilityTextGatePolicy.positiveClaimAudit
$positiveClaimAuditPathSet = New-Object System.Collections.Generic.HashSet[string]([System.StringComparer]::OrdinalIgnoreCase)
if ($PositiveClaimAuditPaths.Count -gt 0) {
    foreach ($auditPath in $PositiveClaimAuditPaths) {
        $positiveClaimAuditPathSet.Add((Normalize-RepoRelativePath -RepoRoot $repoRoot -FullPath $auditPath)) | Out-Null
    }
} else {
    foreach ($auditPath in $positiveClaimAuditConfig.scopedPaths) {
        $positiveClaimAuditPathSet.Add(($auditPath -replace '\\', '/')) | Out-Null
    }
}

if ($allowedClaims.Count -eq 0) {
    throw "Capability governance registry returned no allowed claims for the user-facing text gate."
}

if ($allowedClaimPhrases.Count -eq 0) {
    throw "Capability governance registry returned no approved phrases for allowed claims."
}

if ($forbiddenOverstatementChecks.Count -eq 0) {
    throw "Capability governance registry returned no disallowed claims for the user-facing text gate."
}

if ($positiveClaimAuditPathSet.Count -eq 0) {
    throw "Capability governance registry returned no scoped paths for the positive-claim audit."
}

foreach ($file in $files) {
    try {
        $content = $utf8.GetString([System.IO.File]::ReadAllBytes($file.FullName))
    } catch {
        Add-Finding -Findings $findings -Path $file.FullName -Line 1 -Kind "invalid_utf8" -Detail $_.Exception.Message
        continue
    }

    $lines = $content -split "`r?`n"
    $relativePath = Normalize-RepoRelativePath -RepoRoot $repoRoot -FullPath $file.FullName
    $isPositiveClaimAuditTarget = $positiveClaimAuditPathSet.Contains($relativePath)
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $lineNo = $i + 1

        if ($line.Contains($replacementChar)) {
            Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "replacement_char" -Detail "Line contains U+FFFD replacement character."
        }

        foreach ($fragment in $knownMojibakeFragments) {
            if ($line.Contains($fragment)) {
                Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "known_mojibake" -Detail ("Line contains known mojibake fragment: {0}" -f $fragment)
            }
        }

        if ($line.Contains("window.alert")) {
            Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "window_alert" -Detail "User-facing alert calls should not be used."
        }

        if ($line -match 'HTTP\s+\d{3}') {
            Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "raw_http_status" -Detail "Raw HTTP status text may leak into user-facing content."
        }

        if ($file.Extension.ToLowerInvariant() -ne ".html" -and $line -match '<html|<!doctype html') {
            Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "raw_html" -Detail "Raw HTML should not appear in user-facing text paths."
        }

        foreach ($check in $forbiddenOverstatementChecks) {
            $isPatternPresent = $line -match [regex]::Escape($check.Pattern)
            $isQuotedGuardrail = $line -match $check.AllowedContextPattern

            if ($isPatternPresent -and -not $isQuotedGuardrail) {
                Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "overstated_capability" -Detail $check.Detail
            }
        }

        if ($isPositiveClaimAuditTarget) {
            $hasPositiveSupportPhrase = $false
            foreach ($pattern in $positiveClaimAuditConfig.positiveStatementPatterns) {
                if ($line -match $pattern) {
                    $hasPositiveSupportPhrase = $true
                    break
                }
            }

            if ($hasPositiveSupportPhrase) {
                $hasAllowedPhrase = $false
                foreach ($phrase in $allowedClaimPhrases) {
                    if ($line -match [regex]::Escape($phrase)) {
                        $hasAllowedPhrase = $true
                        break
                    }
                }

                $isAllowedContext = $line -match $positiveClaimAuditConfig.allowedContextPattern

                if (-not $hasAllowedPhrase -and -not $isAllowedContext) {
                    Add-Finding -Findings $findings -Path $file.FullName -Line $lineNo -Kind "positive_claim_not_whitelisted" -Detail "Positive support wording in README or core pages must stay within the allowed_claim whitelist."
                }
            }
        }
    }
}

if ($findings.Count -gt 0) {
    Write-Host "User-facing text check failed:" -ForegroundColor Red
    $findings | Sort-Object Path, Line, Kind | ForEach-Object {
        Write-Host ("- {0}:{1} [{2}] {3}" -f $_.Path, $_.Line, $_.Kind, $_.Detail)
    }
    exit 1
}

Write-Host ("User-facing text check passed for {0} files." -f $files.Count) -ForegroundColor Green
