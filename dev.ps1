# ============================================
# QuickDrop Dev Launcher
# ============================================
# 
# این اسکریپت:
# 1. پروسه‌های قبلی WebView2 رو می‌بنده
# 2. پوشه‌ی داده‌ی WebView2 رو پاک می‌کنه
# 3. پروژه رو اجرا می‌کنه
#
# استفاده: .\dev.ps1
# ============================================

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  QuickDrop Dev Launcher" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# ============================================
# 1. بستن پروسه‌های قبلی
# ============================================
Write-Host "[1/3] Closing previous processes..." -ForegroundColor Yellow
taskkill /f /im msedgewebview2.exe 2>$null | Out-Null
taskkill /f /im quickdrop.exe 2>$null | Out-Null
Start-Sleep -Milliseconds 800
Write-Host "      [OK] Processes closed" -ForegroundColor Green

# ============================================
# 2. پاک کردن پوشه‌های داده‌ی WebView2
# ============================================
Write-Host "[2/3] Clearing WebView2 data..." -ForegroundColor Yellow

$pathsToClear = @(
    "$env:LOCALAPPDATA\com.parsa.quickdrop",
    "$env:LOCALAPPDATA\QuickDropWebView",
    "$env:LOCALAPPDATA\Microsoft\EdgeWebView\EBWebView"
)

foreach ($path in $pathsToClear) {
    if (Test-Path $path) {
        Remove-Item -Path $path -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "      [DEL] $path" -ForegroundColor Gray
    }
}
Write-Host "      [OK] Cleanup done" -ForegroundColor Green

# ============================================
# 3. اجرای پروژه
# ============================================
Write-Host "[3/3] Running QuickDrop..." -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Set-Location $PSScriptRoot
npm run tauri dev