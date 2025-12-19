@echo off
REM Method-VI Step 1 Test Runner with API Key

echo ========================================
echo Method-VI Step 1 Full Test
echo ========================================
echo.

REM Set your Anthropic API key here
set ANTHROPIC_API_KEY=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

echo API Key configured: %ANTHROPIC_API_KEY:~0,15%...
echo.
echo Running Step 1 test...
echo.

cd src-tauri
cargo test --test test_step_1 -- --nocapture
