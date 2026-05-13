# QuantPilot Desktop v1.0.7
# PowerShell 启动脚本 (UTF-8 native)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

$env:QUANTPILOT_DEV = "true"

Write-Host "============================================"
Write-Host "  QuantPilot Desktop v1.0.7"
Write-Host "  QUANTPILOT_DEV = $env:QUANTPILOT_DEV"
Write-Host "============================================"

Write-Host "  Stopping old processes..."
Get-Process -Name "quantpilot" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "quantpilot-tauri" -ErrorAction SilentlyContinue | Stop-Process -Force

# Kill process on port 5173
$port5173 = Get-NetTCPConnection -LocalPort 5173 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess
if ($port5173) { Stop-Process -Id $port5173 -Force -ErrorAction SilentlyContinue }

Write-Host ""
Write-Host "============================================"
Write-Host "  Step 1/3: Building backend..."
Write-Host "============================================"

cargo build --bin quantpilot
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Backend build failed!"
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "============================================"
Write-Host "  Step 2/3: Starting backend..."
Write-Host "============================================"

$backend = Start-Process -FilePath "target\debug\quantpilot.exe" -WindowStyle Minimized -PassThru

# Wait for backend
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Seconds 1
    try {
        $null = (New-Object Net.Sockets.TcpClient).Connect("127.0.0.1", 3000)
        Write-Host "  Backend is ready on port 3000!"
        break
    } catch {
        if ($i -eq 29) {
            Write-Host "  [WARN] Backend did not start. Proceeding anyway..."
        }
    }
}

Write-Host ""
Write-Host "============================================"
Write-Host "  Step 3/3: Starting Tauri Desktop..."
Write-Host "============================================"

Set-Location "$root\src-tauri"
cargo tauri dev

Read-Host "Press Enter to exit"
