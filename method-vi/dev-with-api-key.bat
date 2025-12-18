@echo off
REM Method-VI Development Server Launcher with API Key
REM This script properly sets the ANTHROPIC_API_KEY environment variable before launching

echo ========================================
echo Method-VI Development Server
echo ========================================
echo.

REM Set your Anthropic API key here
set ANTHROPIC_API_KEY=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

echo API Key configured: %ANTHROPIC_API_KEY:~0,15%...
echo.
echo Starting Tauri dev server...
echo.

npm run tauri dev
