# Step 0 (Intent Capture) Implementation Summary

**Date**: 2025-12-17
**Status**: ✅ COMPLETE - Ready for Testing

---

## Overview

Successfully implemented the complete Step 0 (Intent Capture) interface connecting the React frontend to the Rust backend via Tauri, integrating with the Orchestrator and Scope & Pattern Agent to call Claude API for real intent interpretation.

---

## Components Created

### Frontend (React + TypeScript)

#### 1. **Step0View Component** (`src/components/steps/Step0View.tsx`)
A comprehensive multi-state component handling the entire Step 0 workflow:

**States**:
- **Input**: Welcome screen with large text area for user intent
- **Processing**: Loading spinner while Claude API processes intent
- **Clarifying**: Follow-up questions interface (if agent needs clarification)
- **Review**: Display of captured intent with gate approval options

**Features**:
- Clean, intuitive UI with Tailwind CSS dark theme
- Real-time error handling and display
- Intent summary display with formatted sections:
  - User's original intent
  - Normalized goal
  - Success criteria
  - Scope boundaries
  - Assumptions
- Pattern recommendations (future: from Learning Plane)
- Built-in gate approval flow

#### 2. **Updated RunView** (`src/pages/RunView.tsx`)
Refactored to support multiple steps:
- Step-based routing (currently supports Step 0 and placeholder for Step 1)
- Gate approval callback handling
- Integration with Tauri backend via `invoke()`
- Seamless transition between steps

---

### Backend (Rust + Tauri)

#### 3. **Tauri Commands Module** (`src-tauri/src/commands/`)

**New Files**:
- `commands/mod.rs` - Module exports
- `commands/step0.rs` - Step 0 command implementations

**Commands Created**:

##### `start_step_0`
```rust
pub async fn start_step_0(
    run_id: String,
    user_intent: String,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step0Response, String>
```

**Flow**:
1. Retrieves API key from config or environment variable
2. Creates AnthropicClient
3. Creates ScopePatternAgent with Claude client
4. Initializes new Orchestrator with scope agent
5. Executes `orchestrator.execute_step_0()`
6. Returns formatted response for frontend

**Returns**: `Step0Response`
```rust
struct Step0Response {
    intent_summary: IntentSummaryForFrontend,
    clarification_questions: Vec<ClarificationQuestion>,
    pattern_recommendations: Vec<PatternRecommendation>,
}
```

##### `approve_gate`
```rust
pub async fn approve_gate(
    approver: String,
    state: State<'_, OrchestratorState>,
) -> Result<(), String>
```

Approves the gate and transitions orchestrator state from `Step0GatePending` to `Step1Active`.

##### `reject_gate`
```rust
pub async fn reject_gate(
    rejector: String,
    reason: String,
    state: State<'_, OrchestratorState>,
) -> Result<(), String>
```

Rejects the gate and halts the run with the provided reason.

##### `submit_clarifications`
```rust
pub async fn submit_clarifications(
    run_id: String,
    answers: Vec<String>,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step0Response, String>
```

Re-runs Step 0 with user's clarification answers appended to original intent.

---

#### 4. **State Management**

**Added to `lib.rs`**:
- `OrchestratorState`: Global state holding current orchestrator instance
- `AppConfig` state: Application configuration (API key, model, settings)

**Initialization** (in `lib.rs:setup()`):
```rust
let config = AppConfig::load(&app_handle)?;
app.manage(OrchestratorState(Mutex::new(None)));
app.manage(Mutex::new(config));
```

---

#### 5. **Updated Files**

**lib.rs**:
- Added `commands` module
- Registered 4 new Tauri commands
- Added state management for Orchestrator and AppConfig
- Imports `tauri::Manager` trait

**agents/orchestrator.rs**:
- Made `intent_summary` field public for command access

---

## Data Flow

### Step 0 Execution Flow

```
User Input (React)
    ↓
Step0View → invoke('start_step_0', { runId, userIntent })
    ↓
Tauri Command: start_step_0
    ↓
Get API Key from AppConfig or ENV
    ↓
Create AnthropicClient
    ↓
Create ScopePatternAgent(claude_client)
    ↓
Create Orchestrator.with_scope_agent(scope_agent)
    ↓
orchestrator.execute_step_0(user_intent)
    ↓
    ├─ Record run_start in Ledger
    ├─ Generate Steno-Ledger context
    ├─ Call scope_agent.interpret_intent()
    │     ├─ Build system prompt with Steno-Ledger
    │     ├─ Call Claude API
    │     ├─ Parse structured response
    │     └─ Return IntentSummary artifact
    ├─ Store intent_summary in Orchestrator
    ├─ Emit Ready_for_Step_1 signal
    ├─ Record gate signal in Ledger
    └─ Transition to Step0GatePending state
    ↓
Convert IntentSummary → Step0Response
    ↓
Store Orchestrator in state
    ↓
Return Step0Response to Frontend
    ↓
Step0View displays results in Review state
    ↓
User clicks "Approve & Continue"
    ↓
invoke('approve_gate', { approver: 'User' })
    ↓
Tauri Command: approve_gate
    ↓
orchestrator.approve_gate(approver)
    ↓
    ├─ Record gate_approved in Ledger
    ├─ Transition to Step1Active
    └─ Update role: Observer → Conductor
    ↓
RunView → setCurrentStep(1)
    ↓
Display Step 1 placeholder
```

---

## Integration Points

### Real Backend Integration ✅

The implementation connects to **real backend services**:

1. **Orchestrator** (`agents/orchestrator.rs`):
   - Manages Method-VI state machine
   - Controls step transitions
   - Enforces Gate Protocol
   - Records all actions in Ledger

2. **Scope & Pattern Agent** (`agents/scope_pattern.rs`):
   - Calls real Claude API via AnthropicClient
   - Uses Steno-Ledger prepended to system prompt
   - Parses structured intent summary from Claude's response
   - Generates content-addressable hash for artifact

3. **Anthropic API** (`api/anthropic.rs`):
   - Real HTTP calls to `api.anthropic.com`
   - Model: `claude-sonnet-4-20250514` (default)
   - Max tokens: Configurable (default 2000 for Step 0)
   - Timeout: 120 seconds

4. **Ledger Manager** (`ledger/manager.rs`):
   - Records all state transitions
   - Maintains hash chain for audit trail
   - Validates actions against current state

5. **Signal Router** (`signals/mod.rs`):
   - Emits Ready_for_Step_1 signal
   - Records signal hash
   - Tracks signal chain per run

---

## API Key Configuration

The system supports multiple methods for API key configuration:

### Method 1: Environment Variable (Highest Priority)
```bash
set ANTHROPIC_API_KEY=sk-ant-api03-...
npm run tauri dev
```

### Method 2: Config File (Settings UI - Future)
Stored in: `{app_data_dir}/config/settings.json`
- API key stored as base64 encoded
- Accessible via Settings page (to be connected)

### Method 3: Direct Set in Code (Development Only)
Temporarily modify config file for testing.

---

## User Interface Screens

### 1. Input Screen
```
┌─────────────────────────────────────────────┐
│ Step 0: Intent Capture                      │
├─────────────────────────────────────────────┤
│ Welcome to Method-VI                        │
│                                             │
│ This first step helps you clearly define... │
│ • Understand your goal and normalize it     │
│ • Identify success criteria...              │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Describe your intent or goal            │ │
│ │                                          │ │
│ │ [Large text area]                        │ │
│ │                                          │ │
│ │                                          │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│          [Begin Analysis Button]            │
└─────────────────────────────────────────────┘
```

### 2. Processing Screen
```
┌─────────────────────────────────────────────┐
│                                             │
│              [Spinning loader]              │
│         Processing your intent...           │
│   The Scope & Pattern Agent is analyzing    │
│               your goal                     │
│                                             │
└─────────────────────────────────────────────┘
```

### 3. Review Screen (Gate)
```
┌─────────────────────────────────────────────┐
│ 📋 Captured Intent                          │
├─────────────────────────────────────────────┤
│ Original Intent:                            │
│ > [User's input]                            │
│                                             │
│ Normalized Goal:                            │
│ [Claude's interpretation]                   │
│                                             │
│ Success Criteria:                           │
│ 1. [Criterion 1]                            │
│ 2. [Criterion 2]                            │
│                                             │
│ Scope Boundaries:                           │
│ 1. [Out of scope item 1]                    │
│                                             │
│ 💡 Recommended Patterns                     │
│ [Pattern cards would appear here]           │
│                                             │
│ 🚦 Gate: Ready for Step 1                   │
│ Review the summary above and decide:        │
│                                             │
│ [← Adjust Intent]  [✓ Approve & Continue → │
└─────────────────────────────────────────────┘
```

---

## Testing Checklist

### ✅ Compilation
- [x] Rust backend compiles without errors
- [x] React frontend builds successfully
- [x] All TypeScript types resolve correctly

### ⏳ Runtime Testing (To Be Performed)

**Step 0 Flow**:
1. [ ] Start new run from Home page
2. [ ] Enter intent in text area
3. [ ] Click "Begin Analysis"
4. [ ] Verify loading state appears
5. [ ] Verify Claude API is called (check logs)
6. [ ] Verify intent summary displays correctly
7. [ ] Click "Approve & Continue"
8. [ ] Verify gate approval is recorded in ledger
9. [ ] Verify transition to Step 1 placeholder

**Error Handling**:
1. [ ] Test with no API key configured
2. [ ] Test with invalid API key
3. [ ] Test with network error (disconnect)
4. [ ] Test with malformed intent (empty string)

**Clarification Flow** (if Claude asks questions):
1. [ ] Verify clarification questions display
2. [ ] Answer questions in text areas
3. [ ] Submit answers
4. [ ] Verify re-processing with updated context

---

## Known Limitations (MVP Scope)

1. **Pattern Recommendations**: Currently empty array
   - Future: Query Learning Plane for similar successful runs
   - Future: Display pattern cards with relevance scores

2. **Single Run at a Time**: OrchestratorState holds only one orchestrator
   - Future: Use HashMap<RunId, Orchestrator> for concurrent runs

3. **Simple Gate UI**: Uses browser confirm dialog
   - Future: Rich gate dialog with signature, timestamp, rationale

4. **No Artifact Storage**: IntentSummary not saved to database yet
   - Future: Save to `artifacts` table with hash

5. **No Steno-Ledger Display**: Context string not shown in UI
   - Future: Optional display in header or debug panel

---

## Files Modified/Created

### Created Files (5)
1. `method-vi/src/components/steps/Step0View.tsx` - 400+ lines
2. `src-tauri/src/commands/mod.rs` - Module exports
3. `src-tauri/src/commands/step0.rs` - 230+ lines
4. `STEP0_IMPLEMENTATION.md` - This document

### Modified Files (3)
1. `method-vi/src/pages/RunView.tsx` - Refactored to use Step0View
2. `src-tauri/src/lib.rs` - Added state management and command registration
3. `src-tauri/src/agents/orchestrator.rs` - Made `intent_summary` public

---

## Next Steps

### Immediate (Testing)
1. Set API key environment variable
2. Run `npm run tauri dev`
3. Create new run and test Step 0 flow
4. Verify Claude API calls in logs
5. Test gate approval and rejection paths

### Short-term (Complete Step 0)
1. Connect Settings page to AppConfig
   - Add UI for setting API key
   - Add save button that calls Tauri command
2. Save IntentSummary artifact to database
3. Display Steno-Ledger in UI (optional toggle)
4. Implement pattern recommendation query

### Medium-term (Step 1 and Beyond)
1. Create Step1View component (Charter & Baseline)
2. Implement Charter Agent
3. Add baseline freeze functionality
4. Connect remaining steps (2-6.5)

---

## API Usage Example

### Calling from React:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Start Step 0
const result = await invoke<Step0Response>('start_step_0', {
  runId: '2025-12-17-MyProject',
  userIntent: 'Build a mobile app for tracking fitness goals...'
});

console.log('Intent Summary:', result.intent_summary);
console.log('Questions:', result.clarification_questions);

// Approve gate
await invoke('approve_gate', {
  approver: 'John Doe'
});
```

---

## Logging

The backend produces detailed logs for debugging:

```
[INFO] === START_STEP_0 command called ===
[INFO] Run ID: 2025-12-17-MyProject
[INFO] User Intent length: 245 chars
[INFO] API key found: sk-ant-api03-JX...
[INFO] Creating new orchestrator with label: MyProject
[INFO] Executing Step 0...
[INFO] === Executing Step 0: Intent Capture ===
[INFO] Using real Scope & Pattern Agent
[INFO] Calling Claude API for intent interpretation...
[INFO] Claude response received: [response length]
[INFO] Parsing Claude response...
[INFO] Intent interpretation complete
[INFO]   Primary Goal: [goal]
[INFO]   Confidence: 85
[INFO]   Category: Operational
[INFO] Emitting Ready_for_Step_1 signal (GATE)
[INFO] Step 0 complete - awaiting gate approval
[INFO] State: Step0GatePending
```

---

## Success Criteria ✅

All implementation goals achieved:

1. ✅ Shows welcome message explaining Step 0
2. ✅ Large text area for intent input with placeholder
3. ✅ "Begin Analysis" button with loading state
4. ✅ Displays Intent_Summary artifact (formatted nicely)
5. ✅ Shows clarification questions (if any)
6. ✅ Pattern recommendations placeholder
7. ✅ Ability to answer questions and refine intent
8. ✅ Gate dialog with approval/adjustment options
9. ✅ Real backend connection to Orchestrator
10. ✅ Real Scope & Pattern Agent integration
11. ✅ Real Claude API calls

**Status**: Implementation complete and ready for end-to-end testing with live API key.
