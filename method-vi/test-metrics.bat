@echo off
REM Method-VI Metrics Test Runner with API Key
REM This script properly sets the ANTHROPIC_API_KEY environment variable before running tests

echo ========================================
echo Method-VI Metrics Test
echo ========================================
echo.

REM Set your Anthropic API key here
set ANTHROPIC_API_KEY=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

echo API Key configured: %ANTHROPIC_API_KEY:~0,15%...
echo.
echo Running metrics test...
echo.

cd src-tauri
cargo test --test test_metrics -- --nocapture
