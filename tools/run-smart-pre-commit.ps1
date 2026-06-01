param()

$ErrorActionPreference = "Stop"

function Run-Step {
    param(
        [string] $Name,
        [scriptblock] $Command
    )

    Write-Host "=== pre-commit: $Name ==="
    & $Command
}

function Test-AnyPath {
    param(
        [string[]] $Paths,
        [scriptblock] $Predicate
    )

    foreach ($path in $Paths) {
        if (& $Predicate $path) {
            return $true
        }
    }
    return $false
}

function Test-AllPath {
    param(
        [string[]] $Paths,
        [scriptblock] $Predicate
    )

    if ($Paths.Count -eq 0) {
        return $false
    }

    foreach ($path in $Paths) {
        if (-not (& $Predicate $path)) {
            return $false
        }
    }
    return $true
}

$repoRoot = (& git rev-parse --show-toplevel).Trim()
Set-Location $repoRoot

$stagedPaths = @(& git diff --cached --name-only --diff-filter=ACMR | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$nameStatus = @(& git diff --cached --name-status | ForEach-Object { $_.Trim() } | Where-Object { $_ })

if ($stagedPaths.Count -eq 0) {
    Write-Host "=== pre-commit: no staged files ==="
    exit 0
}

$hasNewDeleteOrRename = Test-AnyPath $nameStatus { param($line) $line -match '^(A|D|R)' }
$hasRust = Test-AnyPath $stagedPaths {
    param($path)
    return (
        $path -match '\.rs$' -or
        $path -match '(^|/)Cargo\.toml$' -or
        $path -eq 'Cargo.lock'
    )
}
$hasFrontend = Test-AnyPath $stagedPaths {
    param($path)
    return (
        $path -like 'frontend/*' -or
        $path -like 'frontend-executor/*' -or
        $path -match '(^|/)package(-lock)?\.json$' -or
        $path -match '(^|/)vite\.config\.[cm]?[jt]s$'
    )
}
$hasGovernance = Test-AnyPath $stagedPaths {
    param($path)
    return (
        $path -like 'markdown/00-matrix-governance/*' -or
        $path -like 'markdown/06-milestones/*' -or
        $path -like 'markdown/10-overview/*' -or
        $path -eq 'tools/check-matrix-governance.ps1' -or
        $path -eq 'tools/check-full-feature-tree.ps1' -or
        $path -eq 'tools/check-utf8.ps1' -or
        $path -eq 'tools/run-smart-pre-commit.ps1' -or
        $path -eq 'tools/update-recursive-governance.ps1' -or
        $path -eq 'scripts/pre-commit'
    )
}
$hasHookOrTooling = Test-AnyPath $stagedPaths {
    param($path)
    return (
        $path -like 'scripts/*' -or
        $path -like 'tools/*' -or
        $path -like '.github/*'
    )
}
$docsOnly = Test-AllPath $stagedPaths {
    param($path)
    return (
        $path -like 'markdown/*' -or
        $path -match '\.md$' -or
        $path -match '\.txt$'
    )
}

$forceFull = $env:QUANTPILOT_PRECOMMIT_FULL -eq '1'
$skipFrontend = $env:QUANTPILOT_PRECOMMIT_SKIP_FRONTEND -eq '1'
$extraRustTest = $env:QUANTPILOT_PRECOMMIT_RUST_TEST

$mode = 'custom'
if ($forceFull) {
    $mode = 'full'
} elseif ($docsOnly) {
    $mode = 'docs-only'
} elseif ($hasRust -and -not $hasFrontend -and -not $hasHookOrTooling) {
    $mode = 'rust-only'
} elseif ($hasFrontend -and -not $hasRust -and -not $hasHookOrTooling) {
    $mode = 'frontend-only'
} elseif ($hasRust -and $hasFrontend) {
    $mode = 'mixed'
} elseif ($hasHookOrTooling) {
    $mode = 'tooling'
}

Write-Host "=== pre-commit: smart mode $mode ==="
Write-Host "Staged files: $($stagedPaths.Count)"

Run-Step "diff whitespace check" { git diff --cached --check }
Run-Step "UTF-8 check" { powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-utf8.ps1 }

$runFullFeatureTree = $hasGovernance -or $hasNewDeleteOrRename -or $docsOnly -or $forceFull
$runMatrixGovernance = $hasGovernance -or $docsOnly -or $forceFull
$runHookSync = $hasHookOrTooling -or $forceFull

if ($runFullFeatureTree) {
    Run-Step "full feature tree check" { powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1 }
}

if ($runMatrixGovernance) {
    Run-Step "matrix governance check" { powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-matrix-governance.ps1 }
}

if ($runHookSync) {
    Run-Step "pre-commit hook sync check" { powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-pre-commit-hook.ps1 }
}

if ($forceFull -or $hasRust -or $hasHookOrTooling) {
    Run-Step "cargo fmt --check" { cargo fmt --check }

    if ($forceFull -or $hasHookOrTooling -or (Test-AnyPath $stagedPaths { param($path) $path -match '(^|/)Cargo\.toml$' -or $path -eq 'Cargo.lock' })) {
        Run-Step "cargo check --workspace" { cargo check --workspace }
    } else {
        Run-Step "cargo check -p quantpilot" { cargo check -p quantpilot }
    }

    if ($forceFull) {
        Run-Step "cargo test --workspace --no-run" { ./scripts/test.sh test --workspace --no-run }
    } elseif ($extraRustTest) {
        Run-Step "targeted rust test" { Invoke-Expression $extraRustTest }
    }
}

if (($forceFull -or $hasFrontend -or ($mode -eq 'mixed')) -and -not $skipFrontend) {
    Run-Step "frontend build" { Push-Location frontend; try { npx vite build } finally { Pop-Location } }
    Run-Step "vitest" { Push-Location frontend; try { npx vitest run } finally { Pop-Location } }
}

Write-Host "=== pre-commit: done ($mode) ==="
