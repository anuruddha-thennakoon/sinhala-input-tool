@echo off
rem Double-click to install. Asks for administrator rights if needed.
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo Requesting administrator rights...
    powershell -NoProfile -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
if %errorlevel% neq 0 (
    echo.
    echo *** Installation FAILED. See the message above. ***
) else (
    echo.
    echo Done.
)
pause
