# Method-VI Step 0 Testing Guide

**Date**: 2025-12-17
**Status**: ✅ Ready for Testing
**Issue Resolved**: API Key Configuration

---

## What Happened

### Initial Test Results (Failed)

You tested the Step 0 flow earlier today (2025-12-18 00:06:11Z - 00:07:14Z) and encountered an error. Analysis of the logs revealed:

**Evidence**:
```
[2025-12-18T00:06:11Z INFO] === START_STEP_0 command called ===
[2025-12-18T00:06:11Z INFO] Run ID: 2025-12-18-Marketing Strategy
[2025-12-18T00:06:11Z INFO] User Intent length: 61 chars
[Logs stopped here - no "API key found" message]
```

**What went wrong**:
- The `ANTHROPIC_API_KEY` environment variable set via `set` command in CMD was not inherited by the Tauri/Rust subprocess
- On Windows, environment variables set in the parent shell don't automatically propagate to child processes started by npm
- The backend's `config.get_api_key()` failed with error: "ANTHROPIC_API_KEY not found in environment or config file"
- You saw this error message in the UI when clicking "Begin Analysis"
- You tried again with a longer intent (108 chars) but got the same error

---

## The Solution

Created a Windows batch script (`dev-with-api-key.bat`) that properly sets the environment variable before launching the dev server.

**How it works**:
```batch
set ANTHROPIC_API_KEY=sk-ant-api03-...
npm run tauri dev
```

This ensures the API key is available in the same process context as the npm/Tauri execution.

---

## How to Test Now

### Step 1: Ensure Dev Server is Running

The dev server should already be running from the batch script. Verify by checking:
- Terminal shows: "Configuration loaded successfully"
- No errors in the console
- Vite is running on http://localhost:1420/

**If not running**, start it:
```bash
cd method-vi
./dev-with-api-key.bat
```

Or alternatively on Windows PowerShell:
```powershell
cd method-vi
.\dev-with-api-key.bat
```

---

### Step 2: Open the Application

1. Open your browser
2. Navigate to: **http://localhost:1420/**
3. You should see the Method-VI home page

---

### Step 3: Create a New Run

1. Click **"New Run"** button from home page
2. Enter a Run ID (e.g., "Marketing-Strategy-Test")
3. Click "Start Run"
4. You should be taken to the Step 0 view

---

### Step 4: Complete Step 0 Flow

#### 4a. Input View
**What you should see**:
- Welcome message explaining Step 0
- Large text area with placeholder: "Describe what you want to accomplish..."
- "Begin Analysis" button (blue)

**Action**:
1. Enter your intent. Example:
   ```
   I want to create a marketing strategy for my small business that includes
   social media campaigns, email newsletters, and local partnerships to grow
   our customer base by 25% in the next 6 months.
   ```

2. Click **"Begin Analysis"** button

---

#### 4b. Processing View
**What you should see**:
- Loading spinner
- Message: "Processing your intent..."
- Sub-message: "The Scope & Pattern Agent is analyzing your goal"

**What's happening in the backend** (check terminal):
```
[INFO] === START_STEP_0 command called ===
[INFO] Run ID: 2025-12-18-Marketing-Strategy-Test
[INFO] User Intent length: XXX chars
[INFO] API key found: sk-ant-api03-JX...  ← This should now appear!
[INFO] Creating new orchestrator with label: Marketing-Strategy-Test
[INFO] Executing Step 0...
[INFO] === Executing Step 0: Intent Capture ===
[INFO] Using real Scope & Pattern Agent
[INFO] Calling Claude API for intent interpretation...
[INFO] Claude response received: [response length]
[INFO] Parsing Claude response...
[INFO] Intent interpretation complete
[INFO]   Primary Goal: [your normalized goal]
[INFO]   Confidence: 85
[INFO]   Category: Strategic
[INFO] Emitting Ready_for_Step_1 signal (GATE)
[INFO] Step 0 complete - awaiting gate approval
```

**Expected duration**: 5-15 seconds (depending on Claude API response time)

---

#### 4c. Review View
**What you should see**:
- **📋 Captured Intent** header
- **Original Intent**: Your exact input text
- **Normalized Goal**: Claude's interpretation of your primary goal
- **Success Criteria**: Bulleted list of what's likely in scope
- **Scope Boundaries**: Bulleted list of what's likely out of scope
- **Assumptions**: Any edge cases or assumptions Claude identified
- **💡 Recommended Patterns** section (empty for MVP)
- **🚦 Gate: Ready for Step 1** section with two buttons:
  - "← Adjust Intent" (gray) - returns to input view
  - "✓ Approve & Continue →" (green) - proceeds to Step 1

**Action**:
- Review the captured intent summary
- Verify it matches your intent
- Click **"Approve & Continue"** button

---

#### 4d. Gate Approval
**What you should see**:
- Browser confirmation dialog:
  ```
  Ready to proceed to Step 1?

  Normalized Goal: [your goal]

  Click OK to approve, Cancel to go back.
  ```

**Backend logs** (check terminal):
```
[INFO] === APPROVE_GATE command called ===
[INFO] Approver: User
[INFO] Gate approved successfully
```

**Action**:
- Click **OK** to approve

---

#### 4e. Step 1 Transition
**What you should see**:
- Step 1 placeholder page
- Header: "Step 1: Charter & Baseline"
- Message: "This step is not yet implemented."
- Sub-message: "Coming soon: Charter creation and baseline freezing."

**This confirms**:
- ✅ Step 0 completed successfully
- ✅ Gate approval worked
- ✅ State transitioned to Step 1
- ✅ Orchestrator state machine advanced

---

## Expected Results Summary

### ✅ Success Indicators

If everything works correctly, you should observe:

**Frontend**:
1. ✅ Input view renders with text area
2. ✅ Loading spinner appears after clicking "Begin Analysis"
3. ✅ Intent summary displays with all sections populated
4. ✅ Gate approval buttons visible
5. ✅ Transition to Step 1 placeholder after approval

**Backend Logs**:
1. ✅ `START_STEP_0` command logged
2. ✅ `API key found` logged (KEY INDICATOR!)
3. ✅ `Calling Claude API` logged
4. ✅ `Intent interpretation complete` logged
5. ✅ `Ready_for_Step_1 signal` logged
6. ✅ `APPROVE_GATE` command logged
7. ✅ `Gate approved successfully` logged

**Database** (optional check):
```bash
# Open SQLite database
cd C:\Users\ryanb\AppData\Roaming\com.ryanb.method-vi
sqlite3 method-vi.db

# Check ledger entries
SELECT entry_type, timestamp FROM ledger_entries ORDER BY id;
```

Expected entries:
- `run_start`
- `gate_signal` (Ready_for_Step_1)
- `gate_approved`

---

## Troubleshooting

### Issue: "API key not configured" error still appears

**Check**:
1. Verify batch script shows: `API key configured: sk-ant-api03-JX...`
2. Check terminal logs for `[INFO] API key found: sk-ant-api03-JX...`

**If still failing**:
- Stop the dev server (Ctrl+C)
- Restart using the batch script: `./dev-with-api-key.bat`

---

### Issue: "Failed to create Anthropic client" error

**Possible causes**:
- Invalid API key format
- Network connectivity issues

**Check**:
- Verify API key starts with `sk-ant-api03-`
- Test network connectivity
- Check firewall isn't blocking `api.anthropic.com`

---

### Issue: Timeout after 120 seconds

**Possible causes**:
- Slow network connection
- Claude API service issues

**Check**:
- Monitor terminal for API call start/completion logs
- Check https://status.anthropic.com/ for service status

---

### Issue: "Failed to parse Claude response"

**Possible causes**:
- Claude returned unexpected format
- API version mismatch

**Check**:
- Terminal logs show full error message
- Report issue with logs

---

## Alternative: Set API Key in Config File

If you prefer not to use environment variables, you can set the API key directly in the config file:

### Option 1: Via Settings UI (Future)
Once the Settings page is connected, you'll be able to:
1. Navigate to Settings
2. Enter API key in the "Anthropic API Key" field
3. Click Save
4. Restart the application

### Option 2: Manually Edit Config File

1. **Locate config file**:
   ```
   C:\Users\ryanb\AppData\Roaming\com.ryanb.method-vi\config\settings.json
   ```

2. **Encode your API key** (Base64):
   ```javascript
   // In browser console or Node.js:
   btoa("sk-ant-api03-your-key-here")
   ```

3. **Edit settings.json**:
   ```json
   {
     "anthropic_api_key": "c2stYW50LWFwaTAzLXlvdXIta2V5LWhlcmU=",
     "default_model": "claude-sonnet-4-20250514",
     "default_max_tokens": 4096,
     "enable_api_logging": true
   }
   ```

4. **Restart the application**

---

## Next Steps After Successful Test

Once Step 0 works end-to-end:

### Immediate
1. ✅ Verify ledger entries in database
2. ✅ Test "Adjust Intent" button (gate rejection)
3. ✅ Test with ambiguous intent to check clarification flow
4. ✅ Test error handling (empty intent, network errors)

### Short-term
1. Connect Settings page to AppConfig for UI-based API key management
2. Save IntentSummary artifact to database
3. Add pattern recommendation query to Learning Plane
4. Display Steno-Ledger context in UI (optional toggle)

### Medium-term
1. Implement Step 1 (Charter & Baseline)
2. Create Charter Agent
3. Add baseline freeze functionality
4. Continue with Steps 2-6.5

---

## Success Criteria Checklist

**MVP Requirements**:
- [ ] Application window opens
- [ ] Step 0 view renders
- [ ] Can enter intent text
- [ ] "Begin Analysis" triggers backend call
- [ ] Loading state displays
- [ ] **API key is found and used** (KEY FIX!)
- [ ] Claude API call succeeds
- [ ] Intent summary displays with all fields
- [ ] Gate approval dialog appears
- [ ] Can approve gate
- [ ] Transitions to Step 1 placeholder
- [ ] Backend logs show complete flow
- [ ] Ledger records all events

---

## Test Data Suggestions

Try these different types of intents to verify Claude's interpretation:

### 1. Concrete Intent
```
Build a mobile app for tracking daily water intake with reminders and achievement badges.
```

### 2. Abstract Intent
```
Make our company more innovative and agile.
```
**Expected**: Claude asks clarification questions

### 3. Technical Intent
```
Migrate our PostgreSQL database to AWS Aurora with zero downtime and implement read replicas for scaling.
```

### 4. Business Intent
```
Launch a new product line targeting millennials through Instagram and TikTok campaigns with influencer partnerships.
```

### 5. Ambiguous Intent
```
Improve things.
```
**Expected**: Claude asks many clarification questions

---

## Reporting Issues

If you encounter any issues during testing, please capture:

1. **Screenshots** of each view state
2. **Full backend logs** from terminal
3. **Browser console** errors (F12 → Console tab)
4. **Error messages** displayed in UI
5. **Steps to reproduce** the issue
6. **Intent text** used during test

---

## Files Reference

**Batch Script**:
- `method-vi/dev-with-api-key.bat` - Start dev server with API key

**Implementation Docs**:
- `STEP0_IMPLEMENTATION.md` - Complete Step 0 implementation details
- `STEP0_TEST_REPORT.md` - Test plan and results documentation

**Frontend**:
- `method-vi/src/components/steps/Step0View.tsx` - Step 0 UI component
- `method-vi/src/pages/RunView.tsx` - Run view with step routing

**Backend**:
- `src-tauri/src/commands/step0.rs` - Step 0 Tauri commands
- `src-tauri/src/agents/orchestrator.rs` - Orchestrator state machine
- `src-tauri/src/agents/scope_pattern.rs` - Scope & Pattern Agent (Claude integration)

**Config**:
- `C:\Users\ryanb\AppData\Roaming\com.ryanb.method-vi\config\settings.json` - App configuration
- `C:\Users\ryanb\AppData\Roaming\com.ryanb.method-vi\method-vi.db` - SQLite database

---

## Summary

✅ **Issue Identified**: Environment variable not passed to Rust subprocess
✅ **Solution Created**: Batch script that properly sets API key
✅ **Dev Server Running**: Application ready for testing
🎯 **Ready to Test**: Complete Step 0 flow from input to gate approval

**Your Turn**: Follow Step 3 above to test the full user flow with your marketing strategy intent (or any other intent)!

---

**Good luck with testing! 🚀**
