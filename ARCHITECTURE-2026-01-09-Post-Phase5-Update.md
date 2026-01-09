# Method-VI Architecture Documentation
**Date:** 2026-01-09
**Tag:** Post-Phase5-Update
**Status:** Production-Ready with UI Refinements
**Branch:** master

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Changes Since Phase 5](#changes-since-phase-5)
3. [System Overview](#system-overview)
4. [Technology Stack](#technology-stack)
5. [Architecture Layers](#architecture-layers)
6. [Core Components](#core-components)
7. [Callout System](#callout-system)
8. [Mode Detection System](#mode-detection-system)
9. [Frontend Architecture](#frontend-architecture)
10. [Metrics & Governance System](#metrics--governance-system)
11. [Data Flow Patterns](#data-flow-patterns)
12. [Key Patterns & Interactions](#key-patterns--interactions)
13. [Known Issues & Investigations](#known-issues--investigations)
14. [Complete File Reference](#complete-file-reference)
15. [Testing Status](#testing-status)

---

## Executive Summary

Method-VI is a desktop application implementing a rigorous, AI-assisted framework development process with **adaptive progression architecture**. Built with Tauri (Rust backend) and React (TypeScript frontend), it orchestrates multiple specialized AI agents through a 7-step governance workflow with real-time quality metrics and intelligent callout system.

**Key Capabilities:**
- 7-step guided framework development (Charter → Baseline → Analysis → Synthesis → Validation → Structure → Learning)
- **Progression-focused governance** (relative improvement vs absolute thresholds)
- **Adaptive mode detection** (Architecting/Builder/Refining based on CI baseline)
- **Intelligent callout system** (Info → Attention → Warning → Critical)
- Real-time governance metrics (CI, EV, IAS, EFI, SEC, PCI)
- Multi-lens diagnostic analysis (6 specialized lenses)
- Immutable decision ledger (Steno-Ledger)
- Knowledge graph tracking (Spine)
- Learning harvest and pattern extraction

**Production Status (2026-01-09):**
- ✅ All 68 governance tests passing
- ✅ Core integration tests passing (3/3)
- ✅ Callout UI consolidated into self-contained component
- ✅ HALT/PAUSE deprecation complete - callout system handles governance
- ✅ TypeScript production build successful
- ✅ Tailwind v4 configuration corrected
- ⚠️ API-dependent tests require ANTHROPIC_API_KEY

**Post-Phase 5 Highlights:**
- ✅ CalloutBadge refactored to self-contained component (merged CalloutPanel)
- ✅ CalloutPanel.tsx deleted (functionality consolidated)
- ✅ ModeBadge backdrop dimming added
- ✅ Acknowledgment API parameter fix (frontend/backend alignment)
- ✅ Deprecated HALT/PAUSE methods removed from orchestrator
- ⚠️ Mode detection investigation documented (CI calculates on Charter, not raw input)

---

## Changes Since Phase 5

### Commit History (2026-01-05 → 2026-01-09)

| Commit | Type | Description |
|--------|------|-------------|
| `b7f7e25` | refactor | Consolidate callout UI into self-contained CalloutBadge component |
| `8260f00` | refactor | Remove deprecated HALT/PAUSE methods - callout system now handles governance |
| `a3c683e` | fix | Resolve TypeScript errors blocking production builds |

### Uncommitted Changes (Working Directory)

| File | Change | Description |
|------|--------|-------------|
| `ModeBadge.tsx` | fix | Added `bg-black/50` backdrop dimming to match CalloutBadge |
| `calloutApi.ts` | fix | Fixed parameter name mismatch (`userConfirmation` → `confirmation`) |

### Major Architecture Changes

#### 1. CalloutBadge Self-Containment

**Before (Phase 5):**
```
StatusBar
├── ModeBadge (with popover)
├── CalloutBadge (summary only)
└── CalloutPanel (separate modal component)
```

**After (Post-Phase 5):**
```
StatusBar
├── ModeBadge (with popover + backdrop)
└── CalloutBadge (self-contained with integrated panel)
```

**Rationale:** Consolidating the callout display logic into a single component:
- Eliminates prop drilling between badge and panel
- Simplifies state management (panel visibility, callout data)
- Reduces component coupling
- Makes the component reusable without external dependencies

**Location:** `components/CalloutBadge.tsx` (475 lines, increased from 62)

#### 2. HALT/PAUSE Deprecation Complete

**Removed Methods:**
- `orchestrator.rs`: `check_halt_conditions()` - replaced by callout generation
- `orchestrator.rs`: `trigger_pause()` - replaced by Critical callout blocking
- `orchestrator.rs`: `resume_from_halt()` - replaced by acknowledgment flow

**New Flow:**
```
Metric Calculation → Callout Generation → Critical Check → Gate Block/Allow
```

Instead of binary HALT/PASS, the system now uses graduated callout tiers with only Critical tier blocking progression.

#### 3. Tailwind v4 Configuration

**Before:**
```css
/* index.css - v3 syntax */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**After:**
```css
/* index.css - v4 syntax */
@import "tailwindcss";
@config "../tailwind.config.js";
```

**Impact:** CSS output increased from 9.41KB to 45.99KB - all utility classes now properly generated.

---

## System Overview

### The Method-VI Process

Method-VI guides users through structured framework creation with adaptive quality governance:

```
Step 0: Intent Capture
   ↓ (User intent captured, initial CI measured)
Step 1: Charter & Baseline
   ↓ (Context frozen, CI_baseline locked, mode detected)
Step 2: Governance Calibration
   ↓ (Thresholds adapted to mode, monitoring active)
Step 3: Multi-Angle Diagnostic
   ↓ (6 lenses analyze problem space)
Step 4: Synthesis & Integration
   ↓ (Analysis integrated, delta CI measured)
Step 5: Structure & Redesign
   ↓ (Framework architecture generated)
Step 6: Final Validation
   ↓ (Evidence audit performed, EFI calculated)
Step 6.5: Learning Harvest
   ↓ (Insights extracted, patterns captured)
Closure: Archive & Summarize
   ✓ (Run completed, knowledge preserved)
```

### Critical 6 Metrics (Governance)

Each step is evaluated against 6 quality metrics with **mode-aware thresholds**:

1. **CI (Coherence Index)**: Logical flow, term consistency, clarity, structure
2. **EV (Expansion Variance)**: Content growth stability (informational only)
3. **IAS (Intent Alignment Score)**: Charter alignment (soft/hard gates)
4. **EFI (Evidence Fidelity Index)**: Claim substantiation (Step 6 only)
5. **SEC (Scope Expansion Count)**: Scope drift detection (informational)
6. **PCI (Process Compliance Index)**: Workflow adherence (Step 6 only)

**Enforcement Philosophy:**
- **Critical Callouts**: Block gate progression until acknowledged
- **Warning/Attention Callouts**: Inform but don't block
- **Info Callouts**: FYI, no action required
- **No callouts**: Progression allowed immediately

---

## Technology Stack

### Backend (Rust/Tauri)
- **Tauri 2.9.5** - Cross-platform desktop framework
- **Rust 1.83+** - Systems programming language
- **SQLite 3.x** - Embedded database (rusqlite)
- **Tokio 1.x** - Async runtime
- **Serde 1.x** - Serialization/deserialization
- **Anyhow** - Error handling
- **UUID** - Unique identifiers
- **SHA-256** - Cryptographic hashing (ledger integrity)
- **Chrono** - Date/time handling

### Frontend (React/TypeScript)
- **React 19.1.0** - UI framework
- **TypeScript 5.x** - Type-safe JavaScript
- **Vite 7.3.0** - Build tool and dev server
- **Tailwind CSS 4.1.18** - Utility-first styling (v4 syntax)
- **Lucide React** - Icon library
- **@tauri-apps/api** - Tauri IPC bindings

### AI Integration
- **Anthropic Claude API** - LLM for agent reasoning
- **Primary Model:** claude-sonnet-4-5-20250929
- **Features:** Streaming, structured output, temperature control
- **Rate Limiting:** Handled at API client level

---

## Architecture Layers

```
┌──────────────────────────────────────────────────────────────┐
│              FRONTEND (React/TypeScript/Tailwind)             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐             │
│  │   Pages    │  │ Components │  │   Utils    │             │
│  │  - Home    │  │  - Steps   │  │ - callout- │             │
│  │  - RunView │  │  - Metrics │  │   Api.ts   │             │
│  │  - Settings│  │  - Status  │  │            │             │
│  │            │  │    Bar     │  │            │             │
│  └────────────┘  │  - Callout │  └────────────┘             │
│                  │    Badge   │                              │
│                  │  - Mode    │                              │
│                  │    Badge   │                              │
│                  └────────────┘                              │
└──────────────────────────────────────────────────────────────┘
                          ↕ Tauri IPC (Invoke/Events)
┌──────────────────────────────────────────────────────────────┐
│                  COMMAND LAYER (Tauri Commands)               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Step Commands: step0 | step1 | step2 | ... | step6_5   │ │
│  │ Callout Commands: get_all_callouts | acknowledge_...   │ │
│  │ Mode Commands: get_current_mode                         │ │
│  │ Utility Commands: get_metrics | validate | ...         │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
                          ↕
┌──────────────────────────────────────────────────────────────┐
│              GOVERNANCE LAYER (Callouts & Modes)              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │ CalloutManager │  │  ModeDetector  │  │   Threshold    │ │
│  │ - generate()   │  │  - detect()    │  │   Resolver     │ │
│  │ - acknowledge()│  │  - confidence  │  │  - mode_aware  │ │
│  │ - can_proceed()│  │  - lock_mode() │  │  - adjust()    │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
└──────────────────────────────────────────────────────────────┘
                          ↕
┌──────────────────────────────────────────────────────────────┐
│                AGENT SYSTEM (Business Logic)                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │  Orchestrator  │  │   Governance   │  │    Analysis    │ │
│  │  (Workflow)    │←→│  Telemetry     │  │   (6 Lenses)   │ │
│  └────────────────┘  │  (Metrics)     │  └────────────────┘ │
│  ┌────────────────┐  └────────────────┘  ┌────────────────┐ │
│  │ Scope/Pattern  │  ┌────────────────┐  │   Validation   │ │
│  │  (Charter)     │  │   Structure    │  │   (Learning)   │ │
│  └────────────────┘  │  (Framework)   │  └────────────────┘ │
│                      └────────────────┘                      │
└──────────────────────────────────────────────────────────────┘
                          ↕
┌──────────────────────────────────────────────────────────────┐
│             INFRASTRUCTURE (Data & External)                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐             │
│  │  Database  │  │   Ledger   │  │   Spine    │             │
│  │  (SQLite)  │  │  (Steno)   │  │  (Graph)   │             │
│  └────────────┘  └────────────┘  └────────────┘             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐             │
│  │   Signals  │  │  Artifacts │  │    API     │             │
│  │  (Router)  │  │(Validation)│  │ (Anthropic)│             │
│  └────────────┘  └────────────┘  └────────────┘             │
└──────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Orchestrator (`agents/orchestrator.rs`)
**Role:** Central workflow coordinator with callout integration

**Responsibilities:**
- Step progression control with callout awareness
- Gate management (approval requires Critical acknowledgment)
- Metrics calculation coordination
- State transitions
- Callout generation at each step
- Signal routing
- Ledger integration
- Mode detection trigger (Step 2)

**Key Methods:**
- `execute_step_X()` - Execute specific step, generate callouts
- `calculate_metrics()` - Trigger metric calculations via governance agent
- `check_progression_gates()` - Evaluate if step can proceed (checks `can_proceed()`)

**Post-Phase 5 Changes:**
- Removed deprecated `check_halt_conditions()`
- Removed deprecated `trigger_pause()`
- Removed deprecated `resume_from_halt()`
- Callout generation now sole mechanism for governance feedback

**Location:** `src-tauri/src/agents/orchestrator.rs` (~2558 lines)

### 2. CalloutManager (`governance/callouts.rs`)
**Role:** Callout lifecycle management

**Key Structures:**
```rust
pub enum CalloutTier {
    Info,       // Green - FYI, no action needed
    Attention,  // Yellow - Minor concern, no blocking
    Warning,    // Orange - Important, review suggested
    Critical,   // Red - Blocks progression until acknowledged
}

pub struct Callout {
    pub id: String,
    pub tier: CalloutTier,
    pub original_tier: CalloutTier,  // Before noise filter
    pub metric_name: String,
    pub current_value: f64,
    pub previous_value: Option<f64>,
    pub delta: Option<f64>,
    pub threshold_context: String,
    pub explanation: String,
    pub recommendation: Option<String>,
    pub requires_acknowledgment: bool,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub step: Step,
    pub mode: StructureMode,
    pub created_at: DateTime<Utc>,
}
```

**Key Methods:**
- `add()` - Add a callout to the manager
- `acknowledge_with_confirmation()` - Mark callout as acknowledged with user confirmation
- `acknowledge_all_pending()` - Bulk acknowledge all pending Critical callouts
- `can_proceed()` - Check if all Critical callouts acknowledged
- `summary()` - Get callout count by tier
- `all()` - Get all callouts
- `get_pending_acknowledgments()` - Get unacknowledged Critical callouts

**Location:** `src-tauri/src/governance/callouts.rs` (~785 lines)

### 3. Governance Telemetry Agent (`agents/governance_telemetry.rs`)
**Role:** Metric calculation and callout generation

**Key Methods:**
- `calculate_metrics()` - Main entry point, calculates all 6 metrics
- `calculate_ci()` - Coherence Index with step-semantic weights
- `calculate_ias()` - Intent Alignment Score
- `calculate_efi()` - Evidence Fidelity Index (Step 6)
- `calculate_pci()` - Process Compliance Index (deterministic)
- `generate_callouts()` - Create callouts from metric results

**Location:** `src-tauri/src/agents/governance_telemetry.rs` (~2300 lines)

---

## Callout System

### Overview

The callout system replaces binary HALT conditions with a **tiered feedback model**:

- **Info** (Green): Everything nominal, FYI only
- **Attention** (Yellow): Minor concern, no action required
- **Warning** (Orange): Important issue, review suggested
- **Critical** (Red): Blocks progression until acknowledged

### Architecture (Post-Phase 5)

```
┌──────────────────────────────────────────────────────────┐
│                  Metric Calculation                       │
│         (GovernanceTelemetryAgent calculates CI,          │
│          EV, IAS, EFI, SEC, PCI with thresholds)         │
└──────────────────────────────────────────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│              CalloutManager.generate_callouts()           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ For each metric result:                             │ │
│  │ 1. Determine tier (Info/Attention/Warning/Critical) │ │
│  │ 2. Apply noise filter (mode-aware demotion)        │ │
│  │ 3. Generate explanation & recommendation            │ │
│  │ 4. Check if acknowledgment required                │ │
│  └─────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                Frontend Display (React)                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │           CalloutBadge (Self-Contained)            │  │
│  │  ┌──────────────┐  ┌───────────────────────────┐  │  │
│  │  │    Badge     │→│      Integrated Panel      │  │  │
│  │  │  (summary)   │  │  - Grouped by tier        │  │  │
│  │  │  - count     │  │  - Acknowledgment flow    │  │  │
│  │  │  - color     │  │  - Checkbox confirmation  │  │  │
│  │  └──────────────┘  └───────────────────────────┘  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                  Gate Approval Check                      │
│        if !can_proceed() { block_gate_approval() }       │
└──────────────────────────────────────────────────────────┘
```

### Tauri Commands

**Backend Commands** (`commands/callout_commands.rs`):
```rust
#[tauri::command]
pub fn get_all_callouts(state: State<OrchestratorState>) -> Result<Vec<Callout>, String>

#[tauri::command]
pub fn get_pending_callouts(state: State<OrchestratorState>) -> Result<Vec<Callout>, String>

#[tauri::command]
pub fn get_callout_summary(state: State<OrchestratorState>) -> Result<CalloutSummary, String>

#[tauri::command]
pub fn acknowledge_callout(
    state: State<OrchestratorState>,
    callout_id: String,
    confirmation: String  // Note: parameter name is 'confirmation' not 'userConfirmation'
) -> Result<AcknowledgmentRecord, String>

#[tauri::command]
pub fn acknowledge_all_callouts(
    state: State<OrchestratorState>,
    confirmation: String  // Note: parameter name is 'confirmation' not 'userConfirmation'
) -> Result<Vec<AcknowledgmentRecord>, String>

#[tauri::command]
pub fn can_proceed(state: State<OrchestratorState>) -> Result<bool, String>
```

**Frontend API** (`utils/calloutApi.ts`):
```typescript
export const calloutApi = {
  getAllCallouts: () => invoke<Callout[]>('get_all_callouts'),
  getPendingCallouts: () => invoke<Callout[]>('get_pending_callouts'),
  getCalloutSummary: () => invoke<CalloutSummary>('get_callout_summary'),
  canProceed: () => invoke<boolean>('can_proceed'),

  // IMPORTANT: Backend expects 'confirmation' parameter, not 'userConfirmation'
  acknowledgeCallout: (calloutId: string, userConfirmation: string) =>
    invoke('acknowledge_callout', { calloutId, confirmation: userConfirmation }),
  acknowledgeAllCallouts: (userConfirmation: string) =>
    invoke('acknowledge_all_callouts', { confirmation: userConfirmation }),

  getCurrentMode: () => invoke<ModeInfo>('get_current_mode'),
};
```

---

## Mode Detection System

### Overview

Mode detection analyzes the user's **starting structure level** and adapts thresholds accordingly:

- **Architecting** (CI ≤ 0.35): Early exploration, lower thresholds
- **Builder** (CI 0.36-0.69): Moderate structure, standard thresholds
- **Refining** (CI ≥ 0.70): High structure, elevated thresholds

### Mode Classification

```rust
pub enum StructureMode {
    /// CI baseline ≤ 0.35: High expansion expected, gaps normal
    Architecting,
    /// CI baseline 0.36-0.69: Gap filling focus, moderate expansion
    Builder,
    /// CI baseline ≥ 0.70: Refinement focus, stability expected
    Refining,
}

impl StructureMode {
    pub fn from_ci_baseline(ci: f64) -> Self {
        if ci <= 0.35 {
            StructureMode::Architecting
        } else if ci >= 0.70 {
            StructureMode::Refining
        } else {
            StructureMode::Builder
        }
    }
}
```

### Mode-Adjusted Thresholds

| Mode | CI Range | CI Pass | CI Warn | CI Critical | IAS Pass | IAS Warn | IAS Critical |
|------|----------|---------|---------|-------------|----------|----------|--------------|
| Architecting | ≤ 0.35 | 0.50 | 0.35 | 0.20 | 0.50 | 0.35 | 0.20 |
| Builder | 0.36-0.69 | 0.65 | 0.50 | 0.35 | 0.65 | 0.50 | 0.35 |
| Refining | ≥ 0.70 | 0.80 | 0.70 | 0.50 | 0.80 | 0.70 | 0.50 |

### Detection Timing

**When:** After Step 2 baseline freeze (CI calculated on Charter)

**Process:**
1. Step 1 completes → Charter generated
2. Step 2 runs → CI calculated on Charter content
3. `ModeDetector::detect(ci_value)` called
4. ModeDetectionResult stored in orchestrator
5. Mode logged to steno-ledger (Transparency Mandate)
6. Mode locked for duration of run (immutable)
7. Frontend polls `get_current_mode()` every 5 seconds

**Location:** `orchestrator.rs:1247` triggers detection

---

## Frontend Architecture

### Component Hierarchy

```
App.tsx
└── MainLayout
    ├── Header
    │   └── StatusBar
    │       ├── ModeBadge (with popover)
    │       └── CalloutBadge (self-contained with panel)
    ├── Sidebar
    └── Main Content
        └── RunView
            └── Step Views (Step0View...Step6_5View, ClosureView)
```

### CalloutBadge Component (Post-Phase 5)

**File:** `components/CalloutBadge.tsx` (475 lines)

**Structure:**
```typescript
// Main component
export const CalloutBadge: React.FC<CalloutBadgeProps> = ({ summary, loading }) => {
  const [showPanel, setShowPanel] = useState(false);
  const [callouts, setCallouts] = useState<Callout[]>([]);
  const [panelLoading, setPanelLoading] = useState(false);
  const [acknowledging, setAcknowledging] = useState<string | null>(null);

  // ... handlers for fetch, acknowledge, acknowledgeAll

  return (
    <div className="relative">
      {/* Badge */}
      <div onClick={() => setShowPanel(!showPanel)}>
        {/* Badge content */}
      </div>

      {/* Panel (when open) */}
      {showPanel && (
        <>
          {/* Backdrop */}
          <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setShowPanel(false)} />

          {/* Panel */}
          <div className="absolute top-full right-0 mt-2 z-50 ...">
            {/* Panel content with callout sections */}
          </div>
        </>
      )}
    </div>
  );
};

// Helper components (internal to file)
const CalloutSection: React.FC<CalloutSectionProps> = ({ ... }) => { ... };
const CalloutCard: React.FC<CalloutCardProps> = ({ ... }) => { ... };
const AcknowledgeAllButton: React.FC<AcknowledgeAllButtonProps> = ({ ... }) => { ... };
```

**Key Features:**
- Self-contained state management
- Polls callout data when panel opens
- Groups callouts by tier (Critical → Warning → Attention → Info)
- Intentional friction for acknowledgment (checkbox + button)
- Bulk acknowledge option for multiple pending callouts
- Backdrop dimming when panel open
- Escape key closes panel

### ModeBadge Component

**File:** `components/ModeBadge.tsx` (190 lines)

**Structure:**
```typescript
export const ModeBadge: React.FC<ModeBadgeProps> = ({ showDetails, className }) => {
  const [modeInfo, setModeInfo] = useState<ModeInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [showPopover, setShowPopover] = useState(false);

  // Polls every 5 seconds
  useEffect(() => { ... }, []);

  return (
    <div className="relative">
      <div onClick={() => showDetails && setShowPopover(!showPopover)}>
        {mode} ({confidencePercent}%)
      </div>

      {showDetails && showPopover && (
        <>
          {/* Backdrop with bg-black/50 */}
          <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setShowPopover(false)} />

          {/* Popover */}
          <div className="absolute top-full left-0 mt-2 z-50 ...">
            {/* Mode details and thresholds */}
          </div>
        </>
      )}
    </div>
  );
};
```

### StatusBar Component

**File:** `components/StatusBar.tsx` (51 lines)

**Structure:**
```typescript
export const StatusBar: React.FC<StatusBarProps> = ({ className, pollInterval = 5000 }) => {
  const [summary, setSummary] = useState<CalloutSummary | null>(null);
  const [loading, setLoading] = useState(true);

  // Polls callout summary at interval
  useEffect(() => { ... }, [pollInterval]);

  return (
    <div className="flex items-center gap-3">
      <ModeBadge showDetails />
      <div className="w-px h-6 bg-gray-700" />  {/* Divider */}
      <CalloutBadge summary={summary} loading={loading} />
    </div>
  );
};
```

---

## Metrics & Governance System

### CI (Coherence Index)

**Purpose:** Measure logical flow, term consistency, clarity, and structure

**Calculation (Step-Semantic Weights):**

| Profile | Steps | Logical Flow | Term Consistency | Sentence Clarity | Structure |
|---------|-------|--------------|------------------|------------------|-----------|
| A: Inception | 1-2 | 30% | 30% | 25% | 15% |
| B: Analysis | 3-4 | 50% | 15% | 30% | 5% |
| C: Production | 5-6 | 25% | 25% | 20% | 30% |

**Formula:**
```
CI = (logical_flow × weight_flow) + (term_consistency × weight_term) +
     (sentence_clarity × weight_clarity) + (structure_consistency × weight_structure)
```

**Enforcement:** All steps, triggers Critical callout on hard fail

### IAS (Intent Alignment Score)

**Purpose:** Measure alignment with charter intent

**Gate Behavior:**
- **0.70-1.00:** Pass → Info callout
- **0.30-0.69:** Soft gate → Attention callout (drift detected, no blocking)
- **< 0.30:** Hard gate → Critical callout (extreme drift, blocks progression)

### EFI (Evidence Fidelity Index)

**Purpose:** Measure claim substantiation quality (Step 6 only)

**Claim Taxonomy:**
- **Scored:** Factual, Predictive, Prescriptive claims
- **Ignored:** Instructional, Procedural, Observational statements

**Thresholds:**
- Pass: ≥ 0.80
- Warning: 0.50-0.79
- Critical: < 0.50

### PCI (Process Compliance Index)

**Purpose:** Deterministic checklist audit of workflow adherence (Step 6 only)

**Categories (weighted):**
- Step Sequence: 25%
- Gate Compliance: 30%
- Artifact Presence: 20%
- Audit Integrity: 25%

### Enforcement Matrix

| Metric | Steps | Enforced? | Critical Trigger | Info |
|--------|-------|-----------|-----------------|------|
| CI | 0-6 | Yes | < mode threshold | ≥ pass threshold |
| EV | 2-6 | No | Never | Always |
| IAS | 0-6 | Yes | < 0.30 | ≥ 0.70 |
| EFI | 6 | Yes | < 0.50 | ≥ 0.80 |
| SEC | 2-6 | No | Never | Always |
| PCI | 6 | Yes | Checklist fail | Pass |

---

## Data Flow Patterns

### Gate Approval Flow

```
User clicks "Approve Gate"
    ↓
Frontend: RunView.handleGateReached()
    ↓
Check: canProceed = await calloutApi.canProceed()
    ↓
Backend: Orchestrator.callout_manager.can_proceed()
    ↓
    ├─ Critical callouts pending?
    │   ├─ Yes → return false → Frontend shows "Critical callouts pending"
    │   └─ No → return true → Gate approval proceeds
```

### Acknowledgment Flow

```
User opens CalloutBadge panel
    ↓
Frontend fetches calloutApi.getAllCallouts()
    ↓
User sees Critical callout, checks "I understand"
    ↓
User clicks "Acknowledge"
    ↓
Frontend: calloutApi.acknowledgeCallout(id, confirmation)
    ↓
Backend: callout_manager.acknowledge_with_confirmation(id, confirmation)
    ↓
Callout marked as acknowledged
    ↓
Frontend re-fetches callouts
    ↓
can_proceed() now returns true if no other Critical pending
```

---

## Key Patterns & Interactions

### Pattern 1: Self-Contained Components

**When:** Component needs internal state + external data

**Pattern:**
```typescript
const CalloutBadge: React.FC<CalloutBadgeProps> = ({ summary, loading }) => {
  // External data for badge display
  // Internal state for panel
  const [showPanel, setShowPanel] = useState(false);
  const [callouts, setCallouts] = useState<Callout[]>([]);

  // Fetch detailed data only when panel opens
  useEffect(() => {
    if (showPanel) {
      fetchCallouts();
    }
  }, [showPanel]);

  return (
    <div className="relative">
      {/* Badge uses external summary */}
      {/* Panel uses internal callouts */}
    </div>
  );
};
```

### Pattern 2: Relative Positioning with Backdrop

**When:** Dropdown/popover needs to appear below trigger and dim background

**Pattern:**
```typescript
<div className="relative">
  {/* Trigger */}
  <div onClick={() => setShowPopover(!showPopover)}>
    Click me
  </div>

  {showPopover && (
    <>
      {/* Backdrop - fixed full screen */}
      <div
        className="fixed inset-0 z-40 bg-black/50"
        onClick={() => setShowPopover(false)}
      />

      {/* Popover - absolute relative to wrapper */}
      <div className="absolute top-full left-0 mt-2 z-50">
        Content
      </div>
    </>
  )}
</div>
```

### Pattern 3: Tauri IPC Parameter Alignment

**Critical:** Frontend parameter names must match Rust parameter names exactly.

**Rust:**
```rust
#[tauri::command]
pub fn acknowledge_callout(
    state: State<OrchestratorState>,
    callout_id: String,
    confirmation: String,  // <- This name matters
) -> Result<AcknowledgmentRecord, String>
```

**TypeScript:**
```typescript
acknowledgeCallout: (calloutId: string, userConfirmation: string) =>
  invoke('acknowledge_callout', {
    calloutId,                      // Matches Rust: callout_id (camelCase → snake_case)
    confirmation: userConfirmation  // Must be 'confirmation', not 'userConfirmation'
  }),
```

### Pattern 4: Intentional Friction

**When:** User must acknowledge serious issues

**Pattern:**
```typescript
const [confirmed, setConfirmed] = useState(false);

<label>
  <input
    type="checkbox"
    checked={confirmed}
    onChange={(e) => setConfirmed(e.target.checked)}
  />
  I understand the risk
</label>

<button
  onClick={onAcknowledge}
  disabled={!confirmed}  // Can't click until checked
>
  Acknowledge
</button>
```

---

## Known Issues & Investigations

### MODE-001: CI Calculated on Charter, Not Raw Input

**Status:** Documented, pending architectural decision

**Issue:** Mode detection always results in "Refining (100%)" regardless of input quality.

**Root Cause Analysis:**

1. **CI calculation location:** `orchestrator.rs:1221`
   ```rust
   let (metrics_opt, halt_triggered) = self.calculate_metrics(&charter_content, &charter_content).await?;
   ```

2. **CI evaluates Charter, not user input:**
   - User provides raw input (e.g., "rough meeting notes")
   - Step 1 Scope Agent generates structured Charter
   - Step 2 calculates CI on the **Charter content**
   - Charter is always well-structured (LLM-generated)
   - CI is always high (~0.85-0.95)
   - Mode is always "Refining"

3. **CI prompt evaluates CLARITY dimensions:**
   - Logical flow
   - Term consistency
   - Sentence clarity
   - Structure consistency

4. **Why Charter CI is always high:**
   - Charter is generated by Claude following a template
   - Template ensures logical structure
   - LLM output is inherently coherent prose
   - Even messy input → clean Charter → high CI

**Potential Solutions (Not Implemented):**
- A) Calculate CI on raw user input before Charter generation
- B) Create separate "Input Structure Score" metric
- C) Evaluate Charter vs. User Input delta
- D) Add manual mode override capability

**Impact:** Users expecting "Architecting" mode for exploratory work will always land in "Refining" mode with elevated thresholds.

---

## Complete File Reference

### Backend (Rust)

| File | Lines | Purpose | Updated |
|------|-------|---------|---------|
| `agents/orchestrator.rs` | ~2558 | Workflow coordinator (HALT/PAUSE removed) | 2026-01-09 |
| `agents/governance_telemetry.rs` | ~2300 | Metric calculation, callout generation | 2026-01-05 |
| `agents/analysis_synthesis.rs` | ~856 | Multi-lens analysis | - |
| `agents/baseline_report.rs` | ~423 | Charter and baseline | - |
| `agents/charter_scope.rs` | ~678 | Intent capture and scoping | - |
| `agents/structure_redesign.rs` | ~645 | Framework generation | - |
| `agents/validation_learning.rs` | ~789 | Evidence audit and learning | - |
| `commands/callout_commands.rs` | 76 | 6 callout Tauri commands | - |
| `commands/mode_commands.rs` | 50 | Mode Tauri command | - |
| `governance/callouts.rs` | ~785 | CalloutManager, tier logic | - |
| `governance/types.rs` | ~452 | StructureMode, Thresholds | - |
| `governance/integration_tests.rs` | ~671 | 68 governance tests | - |
| `lib.rs` | ~145 | Library root, command registration | - |

**Total Backend:** ~18,992 lines across 52 files

### Frontend (React/TypeScript)

| File | Lines | Purpose | Updated |
|------|-------|---------|---------|
| `components/CalloutBadge.tsx` | 475 | Self-contained callout badge + panel | 2026-01-09 |
| `components/ModeBadge.tsx` | 190 | Mode display with popover | 2026-01-09 |
| `components/StatusBar.tsx` | 51 | StatusBar container | 2026-01-09 |
| `utils/calloutApi.ts` | 22 | Callout/Mode API wrappers | 2026-01-09 |
| `types/callouts.ts` | 98 | Callout/Mode types | - |
| `index.css` | 5 | Tailwind v4 imports | 2026-01-09 |
| `pages/RunView.tsx` | ~245 | Run view with gate blocking | - |

**Total Frontend:** ~875 lines across 33 files

### Deleted Files

| File | Reason |
|------|--------|
| `components/CalloutPanel.tsx` | Merged into CalloutBadge.tsx |

---

## Testing Status

### Unit Tests

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Running tests\orchestrator_integration.rs
test test_gate_rejection_workflow ... ok
test test_steno_ledger_updates_during_workflow ... ok
test test_complete_step_0_workflow ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### API-Dependent Tests

Requires `ANTHROPIC_API_KEY` environment variable:
- `tests/test_metrics.rs` - Full metric calculation tests

### Governance Tests

68 tests in `governance/integration_tests.rs` covering:
- Callout tier determination
- Noise filter application
- Acknowledgment flow
- Mode detection
- Threshold resolution

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-05 | Phase5-Progression | Initial Phase 5 architecture documentation |
| 2026-01-09 | Post-Phase5-Update | CalloutBadge consolidation, HALT/PAUSE removal, MODE-001 documentation |
