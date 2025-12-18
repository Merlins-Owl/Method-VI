# Method-VI: Getting Started Guide for Claude Code

**Version:** 1.0.0  
**Purpose:** Step-by-step guide for building Method-VI using AI coding assistants  
**Primary Tool:** Claude Code  
**Supporting Tools:** Claude Chat (architecture questions), ChatGPT/Gemini (second opinions)  
**Audience:** Non-developer with basic technical understanding

---

## How to Use This Guide

This document is designed to be **shared directly with Claude Code**. Each section contains:

1. **Context** - What we're building and why
2. **Prompt** - Exact text to give Claude Code
3. **Checkpoint** - How to verify it worked
4. **Troubleshooting** - Common issues and fixes

**Golden Rules:**
- Never skip checkpoints—verify each piece works before moving on
- If something breaks, share the error with Claude Code before trying to fix it yourself
- Save your work frequently (git commits recommended)
- When in doubt, ask Claude Chat for architectural guidance

---

## Project Overview

### What We're Building

Method-VI is a desktop application that:
- Guides users through a 7-step structured reasoning process
- Enforces human-in-the-loop governance at critical gates
- Tracks artifacts in a dependency graph (Coherence Spine)
- Calculates metrics to ensure quality
- Learns patterns from successful runs

### Technology Stack

| Layer | Technology | Why |
|-------|------------|-----|
| Desktop Framework | Tauri | Small, fast, secure |
| Frontend | React + TypeScript | Modern, well-supported |
| Backend | Rust (via Tauri) | Performance, safety |
| Database | SQLite | Portable, no server needed |
| AI APIs | Anthropic Claude | Primary reasoning engine |

### Project Structure (Target)

```
method-vi-app/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # React hooks
│   ├── services/           # API clients
│   └── App.tsx             # Main app
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── database/       # SQLite operations
│   │   ├── spine/          # Coherence Spine Manager
│   │   ├── ledger/         # Ledger Manager
│   │   ├── agents/         # Agent orchestration
│   │   └── api/            # Tauri commands
│   └── Cargo.toml          # Rust dependencies
├── specs/                  # Your specification documents
│   ├── module-plan-method-vi.md
│   ├── Method-VI_Starter_Pattern_Library.md
│   ├── Method-VI_Artifact_Templates.md
│   └── Method-VI_Test_Case_Specifications.md
└── README.md
```

---

## Phase 0: Environment Setup

### Prerequisites Checklist

Before starting with Claude Code, ensure you have:

- [ ] **Node.js** (v18 or later) - [Download](https://nodejs.org/)
- [ ] **Rust** - [Install via rustup](https://rustup.rs/)
- [ ] **VS Code** or similar editor
- [ ] **Git** - For version control
- [ ] **Claude Code** - Installed and authenticated
- [ ] **Anthropic API Key** - For AI calls (get from console.anthropic.com)

### Verify Prerequisites

Open a terminal and run:

```bash
node --version    # Should show v18.x.x or higher
npm --version     # Should show 9.x.x or higher
rustc --version   # Should show 1.70.x or higher
cargo --version   # Should show 1.70.x or higher
git --version     # Should show 2.x.x or higher
```

**If any are missing:** Ask Claude Code to help you install them for your operating system.

---

## Phase 1: Project Initialization

### Session 1.1: Create Project Folder and Add Specs

**What we're doing:** Setting up the project folder with all specification documents.

**Manual Steps (do this yourself):**

```bash
# 1. Create project folder
mkdir method-vi-app
cd method-vi-app

# 2. Create specs folder
mkdir specs

# 3. Copy your specification files into specs/
# - module-plan-method-vi.md
# - Method-VI_Starter_Pattern_Library.md
# - Method-VI_Artifact_Templates.md
# - Method-VI_Test_Case_Specifications.md
# - This file (Method-VI_Getting_Started_Guide.md)
```

**Checkpoint:**
```bash
ls specs/
# Should show all 5 .md files
```

---

### Session 1.2: Initialize Tauri + React Project

**What we're doing:** Creating the desktop application skeleton with Tauri and React.

**Open Claude Code in the method-vi-app folder and use this prompt:**

```
I'm building Method-VI, a desktop application with complete specifications in the specs/ folder.

Please help me initialize a Tauri + React + TypeScript project:

1. Use `npm create tauri-app@latest` with React and TypeScript template
2. Set the app name to "method-vi"
3. After initialization, verify the project structure is correct
4. Add SQLite support to the Rust backend (rusqlite crate)

Please walk me through each step and explain what's happening. I'm not a developer, 
so clear explanations help me understand the codebase.

After setup, show me how to run the app to verify it works.
```

**Checkpoint:**

After Claude Code finishes, you should be able to run:

```bash
npm run tauri dev
```

And see a window open with the default Tauri + React template.

**Troubleshooting:**

| Problem | Solution |
|---------|----------|
| "npm not found" | Install Node.js |
| "cargo not found" | Install Rust via rustup |
| Build errors on Windows | May need Visual Studio Build Tools |
| Build errors on Mac | May need Xcode Command Line Tools: `xcode-select --install` |

---

### Session 1.3: Create Database Schema

**What we're doing:** Setting up the SQLite database with the Knowledge Repository schema.

**Prompt for Claude Code:**

```
Now let's create the SQLite database layer for Method-VI.

Please read the Knowledge Repository schema from specs/module-plan-method-vi.md 
(search for "CREATE TABLE" around line 2670-2760).

Create:
1. A database initialization module in src-tauri/src/database/
2. The schema with all 6 tables: runs, artifacts, spine_edges, patterns, ledger_entries, persistent_flaws
3. All indexes as specified
4. A function to initialize the database on first run
5. A function to check if database exists

The database file should be stored at the user's app data directory 
(use Tauri's app_data_dir).

Please also create basic CRUD operations for the 'runs' table as a starting example.
```

**Checkpoint:**

Ask Claude Code to create a simple test:

```
Create a test that:
1. Initializes the database
2. Creates a test run
3. Reads it back
4. Prints success

Show me how to run this test.
```

**Troubleshooting:**

| Problem | Solution |
|---------|----------|
| "rusqlite not found" | Ensure Cargo.toml has rusqlite dependency |
| Permission errors | Check app_data_dir path is writable |
| Schema errors | Compare against spec exactly |

---

## Phase 2: Infrastructure Services

### Session 2.1: Coherence Spine Manager

**What we're doing:** Building the artifact dependency tracking system.

**Prompt for Claude Code:**

```
Let's implement the Coherence Spine Manager.

Read the specification from specs/module-plan-method-vi.md (search for 
"Coherence Spine Manager" around line 2620-2660).

Create a module at src-tauri/src/spine/ that implements:

1. Data structures for Artifact nodes and Dependency edges (matching the spec schemas)

2. These 5 required queries:
   - get_dependencies(artifact_id) → List of artifacts this depends on
   - get_dependents(artifact_id) → List of artifacts depending on this
   - is_on_critical_path(artifact_id) → Boolean
   - validate_spine_integrity() → List of breaks/orphans
   - get_lineage(artifact_id) → Trace to Intent Anchor

3. Critical Path rules:
   - Intent_Anchor → Charter → Baseline → Core_Thesis form the Critical Path
   - is_on_critical_path should check if artifact is one of these types

4. Functions to add artifacts and edges to the spine

Include detailed comments explaining each function. Create unit tests for each 
of the 5 queries using the test cases from specs/Method-VI_Test_Case_Specifications.md 
(search for "TC-CS" tests).
```

**Checkpoint:**

```
Run the Coherence Spine tests and show me the results.
All TC-CS tests should pass.
```

**Troubleshooting:**

| Problem | Solution |
|---------|----------|
| Circular dependency detected incorrectly | Check get_lineage recursion |
| Critical Path check failing | Verify artifact types match exactly |
| Tests failing | Compare test data with spec |

---

### Session 2.2: Ledger Manager

**What we're doing:** Building the active state management system.

**Prompt for Claude Code:**

```
Now let's implement the Ledger Manager.

Read the specification from specs/module-plan-method-vi.md (search for 
"Ledger Manager" around line 2768-2812).

Create a module at src-tauri/src/ledger/ that implements:

1. LedgerEntry struct matching the spec:
   - id, run_id, entry_type, step, role, payload, prior_hash, hash, created_at

2. State transition validation:
   - Step 0 active: Allow intent capture, pattern query; Block baseline freeze, validation
   - Baseline frozen: Allow analysis, synthesis; Block scope changes, baseline edits
   - Gate pending: Allow human approve/reject; Block agent progression
   - HALT active: Allow human decision only; Block all automated actions

3. Hash chain integrity:
   - Each entry's hash = SHA-256 of entry content
   - Each entry's prior_hash = previous entry's hash
   - First entry has prior_hash = null

4. HALT/PAUSE trigger detection:
   - CI < 0.50 → HALT_IMMEDIATE
   - EV > ±30% → HALT_IMMEDIATE
   - SEC violation → HALT_IMMEDIATE
   - Spine break → HALT_IMMEDIATE
   - CI 0.70-0.80 → PAUSE_FOR_REVIEW

5. Functions:
   - create_entry(run_id, entry_type, step, role, payload) → LedgerEntry
   - validate_action(current_state, proposed_action) → bool
   - check_thresholds(metrics) → HaltStatus
   - verify_chain_integrity(run_id) → bool

Include unit tests from TC-LM test cases.
```

**Checkpoint:**

```
Run the Ledger Manager tests.
Specifically verify:
1. State transitions are enforced (TC-LM-001)
2. Hash chain builds correctly (TC-LM-003)
3. HALT triggers fire at correct thresholds (TC-LM-004)
```

---

### Session 2.3: Context Manager (Steno-Ledger)

**What we're doing:** Building the context injection system for agent prompts.

**Prompt for Claude Code:**

```
Now let's implement the Context Manager with Steno-Ledger generation.

Read the specification from specs/module-plan-method-vi.md (search for 
"Context Manager" around line 2845-2866).

Create a module at src-tauri/src/context/ that implements:

1. Steno-Ledger format:
   [RUN:{id} | S:{step} | R:{role} | CI:{value} | EV:{±value}% | M:{mode} | 🚦:{signal}]

2. Role abbreviations:
   Observer=OBS, Conductor=COND, Auditor=AUD, Patcher=PATCH, 
   Fabricator=FAB, Examiner=EXAM, Curator=CUR, Archivist=ARCH

3. Mode abbreviations:
   Standard=STD, Component=COMP, Surgical=SURG

4. Functions:
   - generate_steno_ledger(run_context) → String
   - get_role_abbreviation(role) → String
   - get_mode_abbreviation(mode) → String

The Steno-Ledger should be prepended to every agent prompt to give the AI 
context about the current run state without re-reading full history.

Include unit tests from TC-CM test cases.
```

**Checkpoint:**

```
Test the Steno-Ledger generation with this sample context:
- run_id: "2025-12-17-TestRun"
- step: 3
- role: "Observer"
- ci: 0.87
- ev: +5%
- mode: "Standard"
- signal: "Ready_for_Synthesis"

Expected output:
[RUN:2025-12-17-TestRun | S:3 | R:OBS | CI:0.87 | EV:+5% | M:STD | 🚦:Ready_for_Synthesis]
```

---

### Session 2.4: Signal Router

**What we're doing:** Building the signal emission and gate recognition system.

**Prompt for Claude Code:**

```
Now let's implement the Signal Router.

Read the specification from specs/module-plan-method-vi.md (search for 
"Signal Router" around line 2815-2843).

Create a module at src-tauri/src/signals/ that implements:

1. Signal struct:
   - type: String (e.g., "Ready_for_Step_1", "Baseline_Frozen")
   - run_id: String
   - timestamp: DateTime
   - prior_signal_hash: Option<String>
   - payload: SignalPayload (step_from, step_to, artifacts_produced, metrics_snapshot, gate_required)

2. Gate signal recognition - these signals require human approval:
   - Ready_for_Step_1 (Step 0→1)
   - Baseline_Frozen (Step 1→2)
   - Ready_for_Analysis (Step 2→3)
   - Ready_for_Synthesis (Step 3→4)
   - Ready_for_Redesign (Step 4→5)
   - Ready_for_Validation (Step 5→6)
   - Validation_Complete (Step 6)

3. Non-gate signals (no approval needed):
   - Learning_Harvested
   - New_Run_Ready
   - Metric_Update (internal)

4. Functions:
   - emit_signal(signal_type, run_id, payload) → Signal
   - is_gate_signal(signal_type) → bool
   - get_signal_chain(run_id) → Vec<Signal>

Include unit tests from TC-SR test cases.
```

**Checkpoint:**

```
Verify:
1. emit_signal creates valid signal with hash chain
2. is_gate_signal returns true for all 7 gate signals
3. is_gate_signal returns false for non-gate signals
```

---

## Phase 3: First Working Agent

### Session 3.1: API Client Setup

**What we're doing:** Creating the connection to Claude API.

**Prompt for Claude Code:**

```
Let's set up the Anthropic API client for calling Claude.

Create a module at src-tauri/src/api/anthropic.rs that:

1. Reads API key from environment variable ANTHROPIC_API_KEY or config file

2. Implements a function to call Claude:
   async fn call_claude(
       system_prompt: &str,
       user_message: &str,
       model: &str,  // default: "claude-sonnet-4-20250514"
       max_tokens: u32  // default: 4096
   ) -> Result<String, Error>

3. Handles errors gracefully:
   - API key missing
   - Network errors
   - Rate limiting
   - Invalid responses

4. Logs API calls for cost tracking (input tokens, output tokens)

For now, use a simple HTTP client (reqwest). We can optimize later.

Also create a config module that reads from {app_data_dir}/config/settings.json
including the API key (encrypted) as specified in the module plan.
```

**Checkpoint:**

```
Create a simple test that:
1. Calls Claude with "Hello, respond with just 'Hello human!'"
2. Prints the response
3. Shows token usage

Note: This will use real API credits, so keep the test short.
```

**Troubleshooting:**

| Problem | Solution |
|---------|----------|
| "API key not found" | Set ANTHROPIC_API_KEY environment variable |
| "401 Unauthorized" | Check API key is valid |
| "429 Rate Limited" | Wait and retry, or check usage limits |

---

### Session 3.2: Orchestrator Agent (Minimal Version)

**What we're doing:** Building the first agent that can run a simplified Step 0.

**Prompt for Claude Code:**

```
Let's build a minimal Orchestrator that can run Step 0.

Read the Orchestrator specification from specs/module-plan-method-vi.md 
(search for "Agent 1: Orchestrator" around line 817).

For this minimal version, create src-tauri/src/agents/orchestrator.rs that:

1. Manages run state:
   - run_id: String
   - current_step: u8 (0-6)
   - active_role: String
   - mode: String (always "Standard" for MVP)

2. Implements Step 0 flow:
   a. Generate run_id: "{YYYY-MM-DD}-{user-provided-label}"
   b. Call Scope & Pattern Agent to capture intent (we'll stub this)
   c. Emit Ready_for_Step_1 signal
   d. Wait for human gate approval
   e. Transition to Step 1

3. Gate enforcement:
   - When gate signal emitted, block progression
   - Expose function for UI to call when human approves
   - Record approval in ledger

4. Uses Context Manager to generate Steno-Ledger for prompts

For now, STUB the Scope & Pattern Agent call - just return a mock Intent_Summary.
We'll implement the real agent next.

Create a simple state machine that tracks the current state and validates transitions.
```

**Checkpoint:**

```
Create a test that:
1. Starts a new run with label "Test-Run"
2. Runs Step 0 (with stubbed agent)
3. Verifies Ready_for_Step_1 signal was emitted
4. Simulates human approval
5. Verifies state transitioned correctly
6. Checks ledger has the gate approval entry
```

---

### Session 3.3: Scope & Pattern Agent (Real Implementation)

**What we're doing:** Building the first real AI-powered agent.

**Prompt for Claude Code:**

```
Now let's implement the real Scope & Pattern Agent.

Read the specification from specs/module-plan-method-vi.md 
(search for "Agent 2: Scope & Pattern" around line 1098).

Create src-tauri/src/agents/scope_pattern.rs that:

1. Implements intent interpretation (Step 0):
   
   Input: User's request text
   Output: Intent_Summary artifact
   
   Uses this prompt template:
   ```
   {steno_ledger}
   
   You are operating as the Scope & Pattern Agent under the OBSERVER stance.
   PERMITTED: Data collection, pattern matching, drift detection.
   FORBIDDEN: Active intervention, scope changes.
   
   INTENT INTERPRETATION
   
   User Request: {user_request}
   
   Please extract:
   - Primary Goal: [What user wants to accomplish]
   - Audience: [Who will use this]
   - Expected Outcome: [What success looks like]
   - Intent Category: [Exploratory / Analytical / Operational]
   - Initial Confidence: [0-100]
   
   Questions for Clarity:
   [List any ambiguities, or "None - intent is clear"]
   
   Preliminary Scope:
   IN SCOPE:
   - [items]
   
   OUT OF SCOPE:
   - [items]
   
   Respond in the exact format above.
   ```

2. Parses Claude's response into an Intent_Summary artifact
   (use the template from specs/Method-VI_Artifact_Templates.md §1)

3. Saves the artifact to the database and registers in Coherence Spine

4. Implements pattern recommendation (Step 0):
   - Query Knowledge Repository for patterns matching intent_category
   - Rank by vitality (freshness × 0.4 + relevance × 0.6)
   - Return top 3-5 patterns
   - Format as Pattern_Suggestions artifact

Connect this to the Orchestrator so Step 0 uses the real agent instead of the stub.
```

**Checkpoint:**

```
Run an end-to-end test:
1. Start new run
2. Provide intent: "Create a project plan for launching a new mobile app"
3. Verify Intent_Summary artifact is created with correct structure
4. Verify artifact is in Coherence Spine
5. Verify Pattern_Suggestions includes relevant patterns (assuming starter patterns are loaded)
6. Verify Ready_for_Step_1 signal emitted
```

---

## Phase 4: Basic UI

### Session 4.1: React Shell and Routing

**What we're doing:** Creating the basic UI structure.

**Prompt for Claude Code:**

```
Let's create the basic React UI for Method-VI.

The UI should have:

1. Main layout with:
   - Header: App name, current run status, settings button
   - Sidebar: Step navigator (Steps 0-6.5 + Closure)
   - Main content area: Current step's interface
   - Footer: Metrics bar (CI, EV, IAS, EFI, SEC, PCI)

2. Pages/Routes:
   - /: Home (new run or resume)
   - /run/:runId: Active run view
   - /settings: Configuration
   - /sessions: Past sessions list

3. Components needed:
   - StepNavigator: Shows 7 steps with current highlighted
   - MetricsBar: Shows Critical 6 metrics with color coding
   - GateDialog: Modal for gate approval decisions
   - ChatInterface: For user input during steps

Use Tailwind CSS for styling (should already be in the Tauri template).
Keep it simple and functional - we can polish the design later.

Start with just the shell and navigation - we'll add step-specific content next.
```

**Checkpoint:**

```
Run the app with `npm run tauri dev` and verify:
1. Window opens with layout visible
2. Navigation between routes works
3. Step navigator shows all steps
4. Metrics bar displays (with placeholder values)
```

---

### Session 4.2: Step 0 Interface

**What we're doing:** Creating the UI for intent capture.

**Prompt for Claude Code:**

```
Let's create the Step 0 (Intent Capture) interface.

Create a component for the Step 0 view that:

1. Shows a welcome message explaining what will happen

2. Has a text area for the user to describe their intent/goal
   - Placeholder: "Describe what you want to accomplish..."
   - Large, comfortable input area

3. Has a "Begin Analysis" button that:
   - Calls the Tauri backend to start a new run
   - Passes the user's intent text
   - Shows loading state while processing

4. After processing, shows:
   - The Intent_Summary artifact (formatted nicely)
   - Any clarification questions from the agent
   - Pattern recommendations (if any)
   - Ability to answer questions or refine intent

5. Shows the gate dialog when Ready_for_Step_1 is emitted:
   - Summary of what was captured
   - "Approve & Continue" button
   - "Adjust Intent" button (goes back to input)

Connect this to the real backend - when user clicks Begin Analysis, 
it should call the Orchestrator which calls the Scope & Pattern Agent.
```

**Checkpoint:**

```
Full user flow test:
1. Open app
2. Enter intent: "I want to create a marketing strategy for my small business"
3. Click "Begin Analysis"
4. See loading indicator
5. See Intent_Summary displayed
6. See Pattern_Suggestions (if patterns exist)
7. See gate dialog
8. Click "Approve & Continue"
9. Verify state moves to Step 1
```

---

### Session 4.3: Metrics Display

**What we're doing:** Making metrics visible and meaningful.

**Prompt for Claude Code:**

```
Let's implement proper metrics display following the Metric Explainability Contract.

Read the specification from specs/module-plan-method-vi.md 
(search for "Metric Explainability Contract" around line 2971).

Create components:

1. MetricCard component showing:
   - Metric name (CI, EV, etc.)
   - Current value with color coding:
     - Green: Pass (meets threshold)
     - Yellow: Warning (approaching threshold)
     - Red: Fail (below threshold)
   - Threshold indicator
   - "Why this score?" expandable section showing:
     - Inputs used
     - Calculation method
     - Interpretation
     - Recommendation (if not passing)

2. MetricsBar component (footer) showing all 6 metrics in compact form:
   - Click any metric to expand full MetricCard

3. MetricsDashboard component (accessible from menu) showing:
   - Radar chart of all 6 metrics
   - History graph (values across steps)
   - Threshold lines on graphs

Use the Threshold Canon values from specs/module-plan-method-vi.md 
(search for "Threshold Canon" around line 3119).

For now, use mock metric values - we'll connect to real calculation later.
```

**Checkpoint:**

```
Verify:
1. Metrics bar shows 6 metrics
2. Color coding works:
   - CI=0.85 → Green
   - CI=0.75 → Yellow
   - CI=0.45 → Red
3. "Why this score?" expands with explanation
4. Dashboard shows radar chart
```

---

## Phase 5: Core Flow Completion

### Session 5.1: Remaining Infrastructure

**What we're doing:** Adding missing pieces before more agents.

**Prompt for Claude Code:**

```
Before building more agents, let's ensure all infrastructure is solid.

Please review what we've built and add any missing pieces:

1. Artifact Envelope validation:
   - Validate frontmatter matches specs/Method-VI_Artifact_Templates.md
   - Verify hash calculation
   - Check dependencies exist

2. Threshold Canon loading:
   - Load thresholds from config file
   - Fall back to defaults if missing
   - Make thresholds available to metrics calculation

3. Cost tracking:
   - Track tokens used per API call
   - Aggregate by run
   - Display in UI

4. Session persistence:
   - Save run state to database
   - Auto-save every 5 minutes
   - Resume capability

5. Error handling:
   - Graceful handling of API failures
   - User-friendly error messages
   - Recovery options

Review the test cases in specs/Method-VI_Test_Case_Specifications.md and 
ensure we have coverage for TC-AE (Artifact Envelope) and TC-TH (Threshold Canon).
```

**Checkpoint:**

```
Run all infrastructure tests:
- TC-CS (Coherence Spine): All pass
- TC-KR (Knowledge Repository): All pass
- TC-LM (Ledger Manager): All pass
- TC-SR (Signal Router): All pass
- TC-CM (Context Manager): All pass
- TC-AE (Artifact Envelope): All pass
- TC-TH (Threshold Canon): All pass
```

---

### Session 5.2: Governance & Telemetry Agent

**What we're doing:** Building the metrics calculation agent.

**Prompt for Claude Code:**

```
Let's implement the Governance & Telemetry Agent.

Read the specification from specs/module-plan-method-vi.md 
(search for "Agent 3: Governance & Telemetry" around line 1310).

Create src-tauri/src/agents/governance_telemetry.rs that:

1. Calculates Critical 6 metrics at step completion:

   CI (Coherence Index):
   - Analyze content for structural coherence, term consistency, logical flow
   - Return 0.0-1.0 score
   
   EV (Expansion Variance):
   - Compare current content size to E_baseline
   - Formula: |E_current - E_baseline| / E_baseline × 100
   - Return percentage
   
   IAS (Intent Alignment Score):
   - Compare current content against Charter objectives
   - Return 0.0-1.0 score
   
   EFI (Execution Fidelity Index):
   - Audit claims for evidence support
   - Return percentage of substantiated claims
   
   SEC (Scope Expansion Count):
   - Count approved vs undocumented scope changes
   - Return compliance percentage
   
   PCI (Process Coherence Index):
   - Check adherence to Architecture Map
   - Return 0.0-1.0 score

2. Uses Claude API for coherence/alignment calculations:
   - Send content + criteria
   - Parse numeric scores from response
   - Include explainability in output

3. Returns MetricResult objects following the Explainability Contract

4. Triggers HALT/PAUSE based on thresholds

5. Implements E_baseline calculation and locking at Step 1

Connect to Orchestrator so metrics are calculated after each step.
```

**Checkpoint:**

```
Test metric calculation:
1. Create sample content for each metric
2. Calculate all 6 metrics
3. Verify output includes:
   - Numeric value
   - Status (pass/warning/fail)
   - Inputs used
   - Interpretation
   - Recommendation (if needed)
```

---

### Session 5.3: Step 1 Flow

**What we're doing:** Implementing the complete Step 1 (Baseline Establishment).

**Prompt for Claude Code:**

```
Let's implement Step 1 (Baseline Establishment).

At Step 1, we need to create 4 immutable artifacts:
- Intent_Anchor (from Intent_Summary)
- Charter
- Baseline_Report
- Architecture_Map

Implement in the Orchestrator:

1. When Step 1 begins (after Ready_for_Step_1 gate approved):
   
   a. Scope & Pattern Agent finalizes Intent_Anchor:
      - Lock the intent (make immutable)
      - This becomes root of Coherence Spine
   
   b. Scope & Pattern Agent creates Charter:
      - Objectives from intent
      - Scope boundaries (confirmed)
      - Success criteria
      - Use template from specs/Method-VI_Artifact_Templates.md §4
   
   c. Governance & Telemetry Agent creates Baseline_Report:
      - Calculate E_baseline from input materials
      - Lock baseline values
      - Register governance checkpoints
      - Use template from specs/Method-VI_Artifact_Templates.md §5
   
   d. Structure & Redesign Agent creates Architecture_Map:
      - Define process flow geometry
      - Set reflection cadence
      - Configure telemetry anchors
      - Use template from specs/Method-VI_Artifact_Templates.md §6

2. All 4 artifacts marked is_immutable=true

3. All 4 artifacts added to Coherence Spine with proper dependencies

4. Emit Baseline_Frozen signal

5. Present gate to user with baseline summary

For Structure & Redesign Agent, create a new file:
src-tauri/src/agents/structure_redesign.rs

Use the prompt templates from the module plan for each agent's work.
```

**Checkpoint:**

```
Full Step 1 test:
1. Complete Step 0 with intent
2. Approve gate
3. Watch Step 1 execute
4. Verify 4 artifacts created:
   - Intent_Anchor (is_immutable=true)
   - Charter (is_immutable=true)
   - Baseline_Report (is_immutable=true, E_baseline set)
   - Architecture_Map (is_immutable=true)
5. Verify all are in Coherence Spine
6. Verify Baseline_Frozen signal emitted
7. Verify gate presented
```

---

## Collaboration Protocol

### When to Use Each Tool

| Situation | Use | Why |
|-----------|-----|-----|
| Writing code, creating files | Claude Code | Direct file manipulation |
| Architecture questions | Claude Chat | Broader context, discussion |
| Second opinion on approach | ChatGPT or Gemini | Different perspective |
| Debugging errors | Claude Code first | Has full codebase context |
| Understanding spec intent | Claude Chat | Can discuss nuance |

### Sharing Context Between Tools

When switching tools, provide context:

**To Claude Chat:**
```
I'm building Method-VI with Claude Code. We just completed [X].
Now I need architectural guidance on [Y].

Key files:
- [relevant file contents or summaries]

My question: [specific question]
```

**To ChatGPT/Gemini for second opinion:**
```
I'm building a desktop app with this specification: [brief summary]

Claude Code suggested [approach A].

What's your assessment? Would you suggest a different approach?
```

### Red Flags to Watch For

Stop and ask Claude Chat if:

- ❌ Claude Code wants to rewrite core architecture
- ❌ Tests that were passing start failing
- ❌ Code seems to deviate from the specification
- ❌ You don't understand why something is being done
- ❌ The same error keeps recurring

### Commit Checkpoints

After each session, commit your code:

```bash
git add .
git commit -m "Session X.Y: [what was accomplished]"
```

This lets you roll back if something goes wrong.

---

## Quick Reference

### Key Specification Locations

| Topic | File | Search Term |
|-------|------|-------------|
| Coherence Spine | module-plan-method-vi.md | "Coherence Spine Manager" |
| Knowledge Repository Schema | module-plan-method-vi.md | "CREATE TABLE" |
| Ledger Manager | module-plan-method-vi.md | "Ledger Manager" |
| Context Manager | module-plan-method-vi.md | "Context Manager" |
| Threshold Canon | module-plan-method-vi.md | "Threshold Canon Storage" |
| Agent Specifications | module-plan-method-vi.md | "Agent 1:", "Agent 2:", etc. |
| Artifact Templates | Method-VI_Artifact_Templates.md | "§1", "§2", etc. |
| Test Cases | Method-VI_Test_Case_Specifications.md | "TC-CS", "TC-KR", etc. |
| Starter Patterns | Method-VI_Starter_Pattern_Library.md | "Pattern 1:", "Pattern 2:", etc. |

### Critical Thresholds

| Metric | Pass | Warning | HALT |
|--------|------|---------|------|
| CI | ≥ 0.80 | 0.70-0.80 | < 0.50 |
| EV | ≤ ±10% | ±10-20% | > ±30% |
| IAS | ≥ 0.80 | 0.70-0.80 | < 0.50 |
| EFI | ≥ 95% | 90-95% | < 80% |
| SEC | 100% | - | < 100% |
| PCI | ≥ 0.90 | 0.85-0.90 | < 0.70 |

### Gate Signals

| Signal | Transition | Required |
|--------|------------|----------|
| Ready_for_Step_1 | 0→1 | Yes |
| Baseline_Frozen | 1→2 | Yes |
| Ready_for_Analysis | 2→3 | Yes |
| Ready_for_Synthesis | 3→4 | Yes |
| Ready_for_Redesign | 4→5 | Yes |
| Ready_for_Validation | 5→6 | Yes |
| Validation_Complete | 6 done | Yes |

---

## Estimated Timeline

| Phase | Sessions | Hours | Outcome |
|-------|----------|-------|---------|
| 0: Setup | 1 | 1-2 | Prerequisites ready |
| 1: Project Init | 3 | 4-6 | Tauri + DB working |
| 2: Infrastructure | 4 | 8-12 | All services working |
| 3: First Agent | 3 | 6-9 | Step 0 working E2E |
| 4: Basic UI | 3 | 6-9 | Usable interface |
| 5: Core Flow | 3 | 8-12 | Steps 0-1 complete |
| **Total MVP Foundation** | **17** | **33-50** | **Working prototype** |

This gets you to a working prototype that can:
- Capture intent
- Create immutable baseline artifacts
- Track in Coherence Spine
- Display metrics
- Enforce gates

From there, adding remaining steps follows the same pattern.

---

## Next Steps After This Guide

Once you complete this guide, you'll have the foundation for Method-VI. Next phases:

1. **Steps 2-6:** Follow same pattern—implement agent, connect to orchestrator, build UI
2. **Step 6.5:** Pattern extraction to Knowledge Repository
3. **Polish:** Error handling, edge cases, UI refinement
4. **Testing:** Run full test suite from TC-E2E
5. **Distribution:** Build installers for Windows/Mac

Each of these can be a separate Claude Code collaboration session.

---

**Document Created:** 2025-12-17  
**For Use With:** Claude Code, Claude Chat, ChatGPT, Gemini  
**Status:** Ready to Begin Development
