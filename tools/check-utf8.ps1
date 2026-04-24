param(
    [string[]]$Paths = @("frontend/src", "frontend/index.html", "src/main.rs", "markdown"),
    [string[]]$Extensions = @(".js", ".jsx", ".ts", ".tsx", ".css", ".md", ".json", ".html", ".rs")
)

$ErrorActionPreference = "Stop"

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
        [string]$Issue,
        [string]$Detail
    )

    $Findings.Add([pscustomobject]@{
            Path   = $Path
            Issue  = $Issue
            Detail = $Detail
        })
}

$utf8 = [System.Text.UTF8Encoding]::new($false, $true)
$replacementChar = [char]0xFFFD
$findings = New-Object System.Collections.Generic.List[object]
$files = Get-TargetFiles -InputPaths $Paths -WantedExtensions $Extensions

foreach ($file in $files) {
    $bytes = [System.IO.File]::ReadAllBytes($file.FullName)

    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        Add-Finding -Findings $findings -Path $file.FullName -Issue "utf8_bom" -Detail "File starts with UTF-8 BOM."
    }

    if ($bytes -contains 0x00) {
        Add-Finding -Findings $findings -Path $file.FullName -Issue "nul_byte" -Detail "File contains NUL byte."
        continue
    }

    try {
        $content = $utf8.GetString($bytes)
    } catch {
        Add-Finding -Findings $findings -Path $file.FullName -Issue "invalid_utf8" -Detail $_.Exception.Message
        continue
    }

    if ($content.Contains($replacementChar)) {
        Add-Finding -Findings $findings -Path $file.FullName -Issue "replacement_char" -Detail "Decoded text contains U+FFFD replacement character."
    }
}

if ($findings.Count -gt 0) {
    Write-Host "UTF-8 check failed:" -ForegroundColor Red
    $findings | Sort-Object Path, Issue | ForEach-Object {
        Write-Host ("- {0} [{1}] {2}" -f $_.Path, $_.Issue, $_.Detail)
    }
    exit 1
}

Write-Host ("UTF-8 check passed for {0} files." -f $files.Count) -ForegroundColor Green
