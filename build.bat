@echo off
taskkill /F /IM tinywhispr.exe >nul 2>&1
timeout /t 1 /nobreak >nul
npm run tauri build
pause