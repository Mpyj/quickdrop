# ============================================
# QuickDrop Dev Launcher — Ultimate
# ============================================

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  QuickDrop Dev Launcher" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# ============================================
# [1/4] Close processes (verified)
# ============================================
Write-Host "[1/4] Closing processes..." -ForegroundColor Yellow

# Kill quickdrop
Get-Process -Name "quickdrop" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Kill msedgewebview2
Get-Process -Name "msedgewebview2" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Wait for them to fully close
$waitTime = 0
while ($waitTime -lt 10) {
    $remaining = Get-Process -Name "quickdrop","msedgewebview2" -ErrorAction SilentlyContinue
    if (-not $remaining) {
        break
    }
    Start-Sleep -Milliseconds 500
    $waitTime++
}

# Force kill if still there
$remaining = Get-Process -Name "quickdrop","msedgewebview2" -ErrorAction SilentlyContinue
if ($remaining) {
    Write-Host "      [WARN] Still $($remaining.Count) processes, force killing..." -ForegroundColor Yellow
    $remaining | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

Write-Host "      [OK] Processes closed" -ForegroundColor Green

# ============================================
# [2/4] Check port 8080
# ============================================
Write-Host "[2/4] Checking port 8080..." -ForegroundColor Yellow

$portInUse = Get-NetTCPConnection -LocalPort 8080 -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "      [WARN] Port 8080 in use, killing..." -ForegroundColor Yellow
    $ownerPid = $portInUse.OwningProcess | Select-Object -First 1
    if ($ownerPid) {
        Stop-Process -Id $ownerPid -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    }
}

Write-Host "      [OK] Port free" -ForegroundColor Green

# ============================================
# [3/4] Wait for file locks to release
# ============================================
Write-Host "[3/4] Waiting for file locks..." -ForegroundColor Yellow

$dllPath = "$PSScriptRoot\target\debug\deps\quickdrop_lib.dll"
$maxWait = 10
$waited = 0

while ($waited -lt $maxWait) {
    try {
        # Try to open the file exclusively
        $stream = [System.IO.File]::Open($dllPath, 'Open', 'ReadWrite', 'None')
        $stream.Close()
        break
    }
    catch {
        Start-Sleep -Milliseconds 500
        $waited++
    }
}

if ($waited -ge $maxWait) {
    Write-Host "      [WARN] File still locked after $maxWait seconds" -ForegroundColor Yellow
} else {
    Write-Host "      [OK] Files unlocked" -ForegroundColor Green
}

# ============================================
# [4/4] Run project
# ============================================
Write-Host "[4/4] Running QuickDrop..." -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Set-Location $PSScriptRoot
npm run tauri dev