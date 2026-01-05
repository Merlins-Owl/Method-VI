# Method-VI Architecture Documentation
**Date:** 2026-01-05
**Tag:** Phase5-Progression
**Status:** Production-Ready with Progression Architecture
**Branch:** feature/progression-architecture

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Overview](#system-overview)
3. [Progression Architecture Philosophy](#progression-architecture-philosophy)
4. [Technology Stack](#technology-stack)
5. [Architecture Layers](#architecture-layers)
6. [Core Components](#core-components)
7. [Agent System](#agent-system)
8. [Callout System](#callout-system)
9. [Mode Detection System](#mode-detection-system)
10. [Database Architecture](#database-architecture)
11. [Metrics & Governance System](#metrics--governance-system)
12. [Frontend Integration](#frontend-integration)
13. [Data Flow Patterns](#data-flow-patterns)
14. [File Structure & Relationships](#file-structure--relationships)
15. [Testing Infrastructure](#testing-infrastructure)
16. [Key Patterns & Interactions](#key-patterns--interactions)
17. [Complete File Reference](#complete-file-reference)
18. [Recent Changes](#recent-changes)

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

**Production Status:**
- ✅ All 68 governance tests passing
- ✅ All 187 total tests passing
- ✅ Phase 5 frontend integration complete
- ✅ Progression architecture fully implemented
- ✅ Release build successful
- ✅ Application launch confirmed

**Phase 5 Highlights:**
- ✅ Callout system with 4-tier severity model
- ✅ Mode detection with CI baseline analysis
- ✅ Frontend StatusBar with real-time callout monitoring
- ✅ Gate blocking only on Critical callouts
- ✅ 66 manual test cases for UX validation

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

**Enforcement Philosophy (Post-Phase 5):**
- **Critical Callouts**: Block gate progression until acknowledged
- **Warning/Attention Callouts**: Inform but don't block
- **Info Callouts**: FYI, no action required
- **No callouts**: Progression allowed immediately

---

## Progression Architecture Philosophy

### Core Philosophy Shift

Method-VI's progression architecture represents a fundamental shift from **absolute gatekeeping** to **relative progression enablement**:

| Dimension | Old (Gatekeeper) | New (Progression Engine) |
|-----------|------------------|--------------------------|
| **Measurement** | Absolute thresholds | Relative improvement |
| **Starting Point** | Must meet CI ≥ 0.70 | Accept any CI, measure delta |
| **Enforcement** | HALT on failure | Callouts with Critical-only blocking |
| **Philosophy** | Block low-quality work | Guide toward improvement |
| **Entry Point** | High-structure required | Accept any structure level |

### The 4 Non-Negotiable Constraints

Even with adaptive thresholds, these 4 constraints are **immutable**:

#### Constraint 1: Transparency Mandate
**Rule**: Mode detection MUST be logged and visible to user

**Why**: Users must know which ruleset governs their run

**Implementation**:
- Mode detected at Step 2 (after CI baseline measured)
- Logged to steno-ledger with timestamp
- Displayed in UI via ModeBadge
- Mode locked for duration of run (cannot change mid-run)

**Location**: `governance/mode_detector.rs:122`, UI: `components/ModeBadge.tsx`

#### Constraint 2: Delta Baseline Rule
**Rule**: Step 3 CI MUST use Step 4 CI as baseline (not Step 1)

**Why**: Step 3 → 4 is critical transition (diagnostic → synthesis). Ensures quality of synthesis, not just diagnostic.

**Implementation**:
- Step 3 CI stored as `ci_step3`
- Step 4 gate checks: `CI_step4 ≥ CI_step3` (no backsliding)
- Triggers callout if regression detected

**Location**: `agents/orchestrator.rs:1456`, `governance/callouts.rs:245`

#### Constraint 3: Noise Filter
**Rule**: In Architecting mode, Warning → Attention (demote severity)

**Why**: Early-stage work naturally has gaps. Warning severity would create false alarms.

**Implementation**:
```rust
fn apply_noise_filter(tier: CalloutTier, mode: StructureMode) -> CalloutTier {
    if mode == StructureMode::Architecting && tier == CalloutTier::Warning {
        CalloutTier::Attention  // Demote
    } else {
        tier  // Keep original
    }
}
```

**Location**: `governance/callouts.rs:389`

#### Constraint 4: Critical-Only Blocking
**Rule**: ONLY Critical callouts block gate progression

**Why**: Allows progress even with minor issues. Forces prioritization of truly blocking problems.

**Implementation**:
- `can_proceed()` checks: `pending_critical_acknowledgments == 0`
- Warning/Attention callouts visible but don't block
- Gate approval checks: `if !calloutApi.canProceed() { alert(...); return; }`

**Location**: `governance/callouts.rs:156`, `pages/RunView.tsx:49`

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

### Frontend (React/TypeScript)
- **React 19.1.0** - UI framework
- **TypeScript 5.x** - Type-safe JavaScript
- **Vite 7.3.0** - Build tool and dev server
- **Tailwind CSS 4.1.18** - Utility-first styling
- **Lucide React** - Icon library
- **@tauri-apps/api** - Tauri IPC bindings

### AI Integration
- **Anthropic Claude API** - LLM for agent reasoning
- **Primary Model:** claude-sonnet-4-5-20250929
- **Features:** Streaming, structured output, temperature control
- **Rate Limiting:** Handled at API client level

### Development Tools
- **Cargo** - Rust package manager
- **npm** - Node package manager
- **Git** - Version control
- **cargo test** - Rust testing framework
- **vitest** - Frontend testing (planned)

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
│                  │    Panel   │                              │
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
- `check_progression_gates()` - Evaluate if step can proceed (checks `can_proceed()`)
- `calculate_step_metrics()` - Trigger metric calculations
- `generate_callouts_for_step()` - Create callouts based on metrics
- `emit_signal()` - Broadcast state changes
- `detect_and_lock_mode()` - Trigger mode detection at Step 2

**Callout Integration Points:**
```rust
// After metrics calculated
let callouts = self.callout_manager.generate_callouts(
    &metrics,
    current_step,
    self.mode_detection_result.as_ref()
)?;

// Before gate approval
if !self.callout_manager.can_proceed() {
    return Err("Critical callouts pending acknowledgment");
}
```

**Dependencies:**
- CalloutManager (progression control)
- ModeDetector (threshold adaptation)
- GovernanceTelemetryAgent (metrics)
- All specialized agents (step execution)
- LedgerManager (decision tracking)
- SpineManager (knowledge graph)

**Location:** `src-tauri/src/agents/orchestrator.rs` (2558 lines)

### 2. Governance Telemetry Agent (`agents/governance_telemetry.rs`)
**Role:** Metric calculation and threshold evaluation

**Responsibilities:**
- Calculate all 6 metrics (CI, EV, IAS, EFI, SEC, PCI)
- Apply step-semantic weighting
- Mode-aware threshold resolution
- Temperature control (0.0 for deterministic metrics)
- Metric explainability (inputs, method, interpretation)

**Key Methods:**
- `calculate_metrics()` - Main entry point, calculates all metrics
- `calculate_ci()` - Coherence Index with step-semantic weights
- `calculate_ias()` - Intent Alignment Score with soft/hard gates
- `calculate_efi()` - Evidence Fidelity Index (Step 6 only)
- `resolve_threshold()` - Mode-aware threshold adjustment
- `evaluate_metric_status()` - Compare value to thresholds

**Mode Integration:**
```rust
pub fn resolve_threshold(&self, base_threshold: f64, mode: &StructureMode) -> f64 {
    match mode {
        StructureMode::Architecting => base_threshold * 0.71,  // Lower bar
        StructureMode::Builder => base_threshold,              // Standard
        StructureMode::Refining => base_threshold * 1.10,      // Higher bar
    }
}
```

**Dependencies:**
- AnthropicClient (LLM calls)
- ModeDetectionResult (threshold adjustment)
- LedgerManager (baseline retrieval)

**Location:** `src-tauri/src/agents/governance_telemetry.rs` (1789 lines)

### 3. Callout Manager (`governance/callouts.rs`)
**Role:** Callout lifecycle management

**Responsibilities:**
- Generate callouts from metrics
- Track acknowledgment status
- Determine if progression can proceed
- Apply noise filter (mode-aware tier demotion)
- Calculate callout summaries
- Persist callouts across steps

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
    pub recommendation: String,
    pub requires_acknowledgment: bool,
    pub acknowledged: bool,
}
```

**Key Methods:**
- `generate_callouts()` - Create callouts from metric results
- `acknowledge_callout()` - Mark callout as acknowledged
- `can_proceed()` - Check if all Critical callouts acknowledged
- `get_summary()` - Get callout count by tier
- `apply_noise_filter()` - Demote Warning → Attention in Architecting mode

**Callout Generation Logic:**
```rust
fn determine_tier(status: MetricStatus, metric_name: &str, step: u8) -> CalloutTier {
    match status {
        MetricStatus::Pass => CalloutTier::Info,
        MetricStatus::Warning => {
            // Soft gates (IAS 0.30-0.70) are Attention, not Warning
            if metric_name == "IAS" && value >= 0.30 && value < 0.70 {
                CalloutTier::Attention
            } else {
                CalloutTier::Warning
            }
        }
        MetricStatus::Fail => {
            // Hard gates (IAS < 0.30, EFI < 0.50, CI < mode_threshold)
            if is_hard_gate(metric_name, value, step) {
                CalloutTier::Critical
            } else {
                CalloutTier::Warning
            }
        }
    }
}
```

**Location:** `src-tauri/src/governance/callouts.rs` (763 lines)

### 4. Mode Detector (`governance/mode_detector.rs`)
**Role:** Detect user's starting structure level

**Responsibilities:**
- Analyze CI baseline (Step 1 CI)
- Classify into Architecting/Builder/Refining
- Calculate confidence score
- Lock mode for run duration
- Log detection to ledger

**Mode Classification:**
```rust
pub enum StructureMode {
    Architecting,  // CI < 0.50 (low structure, exploration phase)
    Builder,       // 0.50 ≤ CI < 0.80 (moderate structure, building phase)
    Refining,      // CI ≥ 0.80 (high structure, polish phase)
}

impl ModeDetector {
    pub fn detect(&self, ci_baseline: f64) -> ModeDetectionResult {
        let mode = if ci_baseline < 0.50 {
            StructureMode::Architecting
        } else if ci_baseline < 0.80 {
            StructureMode::Builder
        } else {
            StructureMode::Refining
        };

        let confidence = self.calculate_confidence(ci_baseline, &mode);

        ModeDetectionResult {
            mode,
            ci_baseline,
            confidence,
            detected_at: chrono::Utc::now(),
        }
    }

    fn calculate_confidence(&self, ci: f64, mode: &StructureMode) -> f64 {
        // Distance from mode boundaries → higher confidence
        match mode {
            StructureMode::Architecting => {
                let dist_from_boundary = (0.50 - ci).abs();
                (1.0 - dist_from_boundary).max(0.80)
            }
            StructureMode::Builder => {
                let dist_from_lower = (ci - 0.50).abs();
                let dist_from_upper = (0.80 - ci).abs();
                let min_dist = dist_from_lower.min(dist_from_upper);
                (1.0 - min_dist * 2.0).max(0.85)
            }
            StructureMode::Refining => {
                let dist_from_boundary = (ci - 0.80).abs();
                (1.0 - dist_from_boundary).max(0.90)
            }
        }
    }
}
```

**User Messages:**
```rust
impl StructureMode {
    pub fn user_message(&self) -> &'static str {
        match self {
            StructureMode::Architecting =>
                "Early-stage exploration: Lower thresholds active. Focus on ideation and structure building.",
            StructureMode::Builder =>
                "Moderate structure detected: Standard thresholds active. Balance expansion with coherence.",
            StructureMode::Refining =>
                "High structure detected: Elevated thresholds active. Focus on polish and evidence.",
        }
    }
}
```

**Location:** `src-tauri/src/governance/types.rs:96-187` (included in types file)

---

## Callout System

### Overview

The callout system replaces binary HALT conditions with a **tiered feedback model**:

- **Info** (Green): Everything nominal, FYI only
- **Attention** (Yellow): Minor concern, no action required
- **Warning** (Orange): Important issue, review suggested
- **Critical** (Red): Blocks progression until acknowledged

### Architecture

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
│                   Callout Storage                         │
│  HashMap<String, Callout> - In-memory for run duration   │
└──────────────────────────────────────────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                Frontend Display (React)                   │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐         │
│  │  Callout   │  │  Callout   │  │   Status   │         │
│  │   Badge    │→│   Panel    │  │    Bar     │         │
│  │ (summary)  │  │ (details)  │  │ (header)   │         │
│  └────────────┘  └────────────┘  └────────────┘         │
└──────────────────────────────────────────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                  Gate Approval Check                      │
│        if !can_proceed() { block_gate_approval() }       │
└──────────────────────────────────────────────────────────┘
```

### Tier Determination Logic

**Rule-Based Tier Assignment:**

1. **Pass → Info** (always)
   - All metrics in healthy range
   - No action needed

2. **Warning → Attention or Warning**
   - IAS soft gate (0.30-0.70): **Attention**
   - Other warnings: **Warning**
   - Mode filter: Warning → Attention in Architecting mode

3. **Fail → Warning or Critical**
   - Hard gates (IAS < 0.30, EFI < 0.50, CI < mode threshold): **Critical**
   - Soft failures: **Warning**

**Code:**
```rust
pub fn generate_callouts(
    &mut self,
    metrics: &CriticalMetrics,
    step: u8,
    mode_result: Option<&ModeDetectionResult>,
) -> Result<Vec<Callout>> {
    let mut callouts = Vec::new();

    for metric_result in metrics.all_results() {
        let tier = self.determine_tier(&metric_result, step);

        // Apply noise filter (mode-aware demotion)
        let final_tier = if let Some(mode) = mode_result {
            self.apply_noise_filter(tier, &mode.mode)
        } else {
            tier
        };

        let callout = Callout {
            id: Uuid::new_v4().to_string(),
            tier: final_tier,
            original_tier: tier,
            metric_name: metric_result.metric_name.clone(),
            current_value: metric_result.value,
            previous_value: self.get_previous_value(&metric_result.metric_name),
            delta: self.calculate_delta(&metric_result),
            threshold_context: format!(
                "Mode: {:?}, Threshold: {:.2}",
                mode.map(|m| &m.mode),
                metric_result.threshold.pass
            ),
            explanation: self.generate_explanation(&metric_result, final_tier),
            recommendation: self.generate_recommendation(&metric_result, final_tier),
            requires_acknowledgment: final_tier == CalloutTier::Critical,
            acknowledged: false,
        };

        callouts.push(callout);
    }

    // Store callouts
    for callout in &callouts {
        self.callouts.insert(callout.id.clone(), callout.clone());
    }

    Ok(callouts)
}
```

### Acknowledgment Flow

**User Workflow:**
1. User sees callout badge in header (shows count, highest tier color)
2. User clicks badge → CalloutPanel modal opens
3. User reviews callouts grouped by tier (Must Review / Important / Minor / Info)
4. For Critical callouts:
   - User checks "I understand" checkbox (intentional friction)
   - User clicks "Acknowledge" button
   - Callout marked as acknowledged
5. Gate approval now allowed (`can_proceed() → true`)

**Backend Logic:**
```rust
pub fn acknowledge_callout(&mut self, callout_id: &str, user_confirmation: String) -> Result<()> {
    let callout = self.callouts.get_mut(callout_id)
        .ok_or_else(|| anyhow!("Callout not found: {}", callout_id))?;

    callout.acknowledged = true;
    callout.acknowledged_at = Some(chrono::Utc::now());
    callout.user_confirmation = Some(user_confirmation);

    Ok(())
}

pub fn can_proceed(&self) -> bool {
    self.callouts.values()
        .filter(|c| c.tier == CalloutTier::Critical)
        .all(|c| c.acknowledged)
}
```

### Noise Filter

**Purpose:** Prevent false alarms in early-stage work

**Rule:** In Architecting mode (CI < 0.50), demote Warning → Attention

**Rationale:**
- Early exploration naturally has gaps, inconsistencies
- Warning severity would create alert fatigue
- Attention still visible but less alarming

**Implementation:**
```rust
fn apply_noise_filter(&self, tier: CalloutTier, mode: &StructureMode) -> CalloutTier {
    if *mode == StructureMode::Architecting && tier == CalloutTier::Warning {
        CalloutTier::Attention
    } else {
        tier
    }
}
```

**Example:**
- **Without filter:** CI = 0.55 in Architecting mode → Warning (orange)
- **With filter:** CI = 0.55 in Architecting mode → Attention (yellow)

### Callout Tauri Commands

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
    user_confirmation: String
) -> Result<(), String>

#[tauri::command]
pub fn acknowledge_all_callouts(
    state: State<OrchestratorState>,
    user_confirmation: String
) -> Result<(), String>

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
  acknowledgeCallout: (calloutId: string, userConfirmation: string) =>
    invoke('acknowledge_callout', { calloutId, userConfirmation }),
  acknowledgeAllCallouts: (userConfirmation: string) =>
    invoke('acknowledge_all_callouts', { userConfirmation }),
};
```

---

## Mode Detection System

### Overview

Mode detection analyzes the user's **starting structure level** and adapts thresholds accordingly:

- **Architecting** (CI < 0.50): Early exploration, lower thresholds
- **Builder** (0.50 ≤ CI < 0.80): Moderate structure, standard thresholds
- **Refining** (CI ≥ 0.80): High structure, elevated thresholds

### Detection Timing

**When:** After Step 1 baseline freeze (CI_baseline measured)

**Process:**
1. Step 1 completes → CI_baseline calculated
2. Orchestrator calls `mode_detector.detect(ci_baseline)`
3. ModeDetectionResult stored in orchestrator
4. Mode logged to steno-ledger (Transparency Mandate)
5. Mode locked for duration of run (immutable)
6. Frontend polls `get_current_mode()` every 5 seconds

### Mode Classification

```rust
pub enum StructureMode {
    /// CI < 0.50: High expansion expected, gaps normal
    Architecting,

    /// 0.50 ≤ CI < 0.80: Gap filling focus, moderate expansion
    Builder,

    /// CI ≥ 0.80: Refinement focus, stability expected
    Refining,
}
```

**Mode Characteristics:**

| Mode | CI Range | Philosophy | Threshold Adjustment | Noise Filter |
|------|----------|------------|---------------------|--------------|
| Architecting | < 0.50 | Ideation, exploration | -29% (×0.71) | Warning → Attention |
| Builder | 0.50-0.79 | Structure building | 0% (×1.0) | None |
| Refining | ≥ 0.80 | Polish, evidence | +10% (×1.10) | None |

### Threshold Adjustment

**Method:**
```rust
impl GovernanceTelemetryAgent {
    pub fn resolve_threshold(&self, base_threshold: f64, mode: &StructureMode) -> f64 {
        match mode {
            StructureMode::Architecting => base_threshold * 0.71,  // Lower
            StructureMode::Builder => base_threshold,              // Standard
            StructureMode::Refining => base_threshold * 1.10,      // Higher
        }
    }
}
```

**Example (CI thresholds):**

| Threshold | Base | Architecting (×0.71) | Builder (×1.0) | Refining (×1.10) |
|-----------|------|---------------------|----------------|------------------|
| Pass      | 0.70 | 0.497 | 0.70 | 0.77 |
| Warning   | 0.50 | 0.355 | 0.50 | 0.55 |
| HALT      | 0.30 | 0.213 | 0.30 | 0.33 |

**Impact:**
- Architecting: User can proceed with CI = 0.50 (would be Warning in Builder mode)
- Refining: User needs CI = 0.77 to pass (higher bar than Builder's 0.70)

### Confidence Scoring

**Algorithm:**
```rust
fn calculate_confidence(&self, ci: f64, mode: &StructureMode) -> f64 {
    match mode {
        StructureMode::Architecting => {
            // Far from 0.50 boundary → high confidence
            let dist_from_boundary = (0.50 - ci).abs();
            (1.0 - dist_from_boundary).max(0.80)
        }
        StructureMode::Builder => {
            // Far from both boundaries → high confidence
            let dist_from_lower = (ci - 0.50).abs();
            let dist_from_upper = (0.80 - ci).abs();
            let min_dist = dist_from_lower.min(dist_from_upper);
            (1.0 - min_dist * 2.0).max(0.85)
        }
        StructureMode::Refining => {
            // Far from 0.80 boundary → high confidence
            let dist_from_boundary = (ci - 0.80).abs();
            (1.0 - dist_from_boundary).max(0.90)
        }
    }
}
```

**Examples:**
- CI = 0.30 → Architecting, confidence = 80% (low, near boundary)
- CI = 0.65 → Builder, confidence = 85% (medium, near center)
- CI = 0.95 → Refining, confidence = 95% (high, far from boundary)

### Mode Tauri Commands

**Backend Commands** (`commands/mode_commands.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeInfo {
    pub mode: Option<String>,              // "Architecting" | "Builder" | "Refining"
    pub display_name: Option<String>,      // "Architecting Mode"
    pub user_message: Option<String>,      // Explanation text
    pub ci_baseline: Option<f64>,          // CI value that triggered detection
    pub confidence: Option<f64>,           // 0.0-1.0
    pub is_locked: bool,                   // Always true after detection
}

#[tauri::command]
pub fn get_current_mode(state: State<OrchestratorState>) -> Result<ModeInfo, String> {
    // Returns mode info if detected, else nulls
}
```

**Frontend API** (`utils/calloutApi.ts`):
```typescript
export const calloutApi = {
  getCurrentMode: () => invoke<ModeInfo>('get_current_mode'),
  // ... other methods
};
```

### User Experience

**Before Step 2** (mode not detected):
- ModeBadge shows: "Mode: Detecting..."
- Tooltip: "Mode is determined after Step 2 baseline analysis. Keep working!"
- Gray styling

**After Step 2** (mode detected and locked):
- ModeBadge shows: "Architecting (85%)" (mode name + confidence)
- Click badge → Details popover opens:
  - Mode: Architecting Mode
  - CI Baseline: 0.35
  - Confidence: 85%
  - Status: Locked for run
  - User Message: "Early-stage exploration: Lower thresholds active..."
- Color-coded: purple (Architecting), blue (Builder), green (Refining)
- Lock icon 🔒 with tooltip: "Mode is fixed for this run. You can continue editing."

---

## Frontend Integration

### Overview

Phase 5 adds 6 new React components that connect the Rust callout/mode backend to the UI:

1. **StatusBar** - Container for ModeBadge + CalloutBadge
2. **ModeBadge** - Mode display with details popover
3. **CalloutBadge** - Summary badge with click to open panel
4. **CalloutPanel** - Modal for reviewing and acknowledging callouts
5. **callouts.ts** - TypeScript types (FFI contract)
6. **calloutApi.ts** - Type-safe Tauri command wrappers

### Component Hierarchy

```
Header (layout/Header.tsx)
├─ Logo + Run Info (left)
├─ StatusBar (center) ← NEW
│  ├─ ModeBadge ← NEW
│  ├─ Divider
│  └─ CalloutBadge ← NEW
│     └─ CalloutPanel (modal) ← NEW
└─ Navigation (right)
```

### StatusBar Component

**Purpose:** Container that integrates badges and handles auto-refresh

**Location:** `src/components/StatusBar.tsx`

**Features:**
- Polls `getCalloutSummary()` every 5 seconds
- Opens/closes CalloutPanel on badge click
- Refreshes after acknowledgment

**Code:**
```typescript
export const StatusBar: React.FC = () => {
  const [summary, setSummary] = useState<CalloutSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [panelOpen, setPanelOpen] = useState(false);

  const fetchSummary = async () => {
    const data = await calloutApi.getCalloutSummary();
    setSummary(data);
    setLoading(false);
  };

  useEffect(() => {
    fetchSummary();
    const interval = setInterval(fetchSummary, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <>
      <div className="flex items-center gap-3">
        <ModeBadge showDetails />
        <div className="w-px h-6 bg-gray-700" />
        <CalloutBadge
          summary={summary}
          loading={loading}
          onClick={() => setPanelOpen(true)}
        />
      </div>

      <CalloutPanel
        isOpen={panelOpen}
        onClose={() => setPanelOpen(false)}
        onAcknowledge={fetchSummary}
      />
    </>
  );
};
```

### ModeBadge Component

**Purpose:** Display current mode with details popover

**Location:** `src/components/ModeBadge.tsx`

**States:**
1. **Loading** - "Loading..." (gray)
2. **Detecting** - "Mode: Detecting..." (gray) + tooltip
3. **Detected** - "Architecting (85%)" (color-coded) + lock icon

**Features:**
- Auto-refresh every 5 seconds
- Click to expand details popover (if `showDetails={true}`)
- Null-safe (all `.toFixed()` calls guarded)
- Color-coded by mode (purple/blue/green)
- Lock icon with tooltip after detection

**UX Guardrails:**
- ✅ "Detecting..." not "Pending"
- ✅ Lock icon tooltip: "Mode is fixed for this run..."
- ✅ Helpful tooltip before detection

**Code:**
```typescript
export const ModeBadge: React.FC<ModeBadgeProps> = ({ showDetails = false }) => {
  const [modeInfo, setModeInfo] = useState<ModeInfo | null>(null);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    const fetchMode = async () => {
      const info = await calloutApi.getCurrentMode();
      setModeInfo(info);
    };
    fetchMode();
    const interval = setInterval(fetchMode, 5000);
    return () => clearInterval(interval);
  }, []);

  if (!modeInfo?.mode) {
    return (
      <div title="Mode is determined after Step 2 baseline analysis. Keep working!">
        <span>Mode: Detecting...</span>
      </div>
    );
  }

  const colors = MODE_COLORS[modeInfo.mode];
  const confidencePercent = modeInfo.confidence !== null
    ? Math.round(modeInfo.confidence * 100)
    : null;

  return (
    <div className={`${colors.bg} border ${colors.border}`} onClick={() => setExpanded(!expanded)}>
      <span className={colors.text}>{modeInfo.mode}</span>
      {confidencePercent && <span>({confidencePercent}%)</span>}
      {modeInfo.is_locked && (
        <span title="Mode is fixed for this run. You can continue editing.">
          🔒
        </span>
      )}
      {expanded && <DetailsPopover modeInfo={modeInfo} />}
    </div>
  );
};
```

### CalloutBadge Component

**Purpose:** Summary badge showing callout count and highest severity

**Location:** `src/components/CalloutBadge.tsx`

**States:**
1. **Loading** - "Loading..." with pulsing dot
2. **No callouts** - "No callouts" with green dot
3. **Has callouts** - "X callouts" with tier-colored styling + pending count

**Visual Design:**
- Color matches highest tier (Critical=red, Warning=orange, Attention=yellow, Info=blue)
- Shows total count: "3 callouts"
- Shows pending count if Critical unacknowledged: "2 pending" (red badge)
- Click to open CalloutPanel

**Code:**
```typescript
export const CalloutBadge: React.FC<CalloutBadgeProps> = ({ summary, loading, onClick }) => {
  if (loading) {
    return <div className="bg-gray-800"><span>Loading...</span></div>;
  }

  if (!summary || summary.total === 0) {
    return (
      <div className="bg-gray-800" onClick={onClick}>
        <div className="w-2 h-2 rounded-full bg-green-500" />
        <span>No callouts</span>
      </div>
    );
  }

  const highestTier: CalloutTier = summary.by_tier.critical > 0 ? 'Critical'
    : summary.by_tier.warning > 0 ? 'Warning'
    : summary.by_tier.attention > 0 ? 'Attention'
    : 'Info';

  const colors = TIER_COLORS[highestTier];

  return (
    <div className={`${colors.bg} border ${colors.border}`} onClick={onClick}>
      <div className={`${colors.text.replace('text-', 'bg-')}`} />
      <span className={colors.text}>
        {summary.total} callout{summary.total !== 1 ? 's' : ''}
      </span>
      {summary.pending_acknowledgments > 0 && (
        <span className="bg-red-600 text-white">
          {summary.pending_acknowledgments} pending
        </span>
      )}
    </div>
  );
};
```

### CalloutPanel Component

**Purpose:** Modal for reviewing and acknowledging callouts

**Location:** `src/components/CalloutPanel.tsx`

**Layout:**
```
┌────────────────────────────────────────────────────┐
│ Callouts (3)                                    ✕ │ ← Header
├────────────────────────────────────────────────────┤
│                                                    │
│ Must Review (1)                    ← Tier section │
│ ┌────────────────────────────────────────────────┐│
│ │ CI                                            ││
│ │ Value below threshold for Architecting mode   ││
│ │ 💡 Review and improve logical flow           ││
│ │ Value: 0.45  Change: -0.05                    ││
│ │                           [ ] I understand    ││
│ │                           [Acknowledge]       ││
│ └────────────────────────────────────────────────┘│
│                                                    │
│ Important (1)                      ← Tier section │
│ ┌────────────────────────────────────────────────┐│
│ │ IAS                                           ││
│ │ Moderate alignment drift detected             ││
│ │ 💡 Revisit charter and realign content       ││
│ │ Value: 0.65  Change: +0.05                    ││
│ └────────────────────────────────────────────────┘│
│                                                    │
│ Minor (1)                          ← Tier section │
│ ┌────────────────────────────────────────────────┐│
│ │ EV                            ✓ Acknowledged  ││
│ │ Content growth within acceptable range        ││
│ │ Value: 8.5%                                   ││
│ └────────────────────────────────────────────────┘│
│                                                    │
├────────────────────────────────────────────────────┤
│ 1 callout pending acknowledgment        ← Footer │
│ [ ] I have reviewed these concerns...            │
│ [        Acknowledge All        ]                │
└────────────────────────────────────────────────────┘
```

**Features:**
- Groups callouts by tier (Must Review / Important / Minor / Info)
- Individual acknowledgment per Critical callout
- Bulk acknowledgment for all pending
- Checkbox confirmation required (intentional friction)
- "Change:" label instead of "Delta:" (UX guardrail)
- ESC key and backdrop click to close
- Auto-refresh after acknowledgment

**UX Guardrails:**
- ✅ Human-friendly tier labels (not raw tier names)
- ✅ Checkbox friction for acknowledgment
- ✅ "Change:" instead of "Delta:"
- ✅ Disabled button until checkbox checked

**Individual Acknowledgment:**
```typescript
const CalloutItem: React.FC<CalloutItemProps> = ({ callout, onAcknowledge }) => {
  const [confirmed, setConfirmed] = useState(false);
  const needsAcknowledge = callout.requires_acknowledgment && !callout.acknowledged;

  return (
    <div className={`${colors.bg} border ${colors.border}`}>
      <div>
        <span>{callout.metric_name}</span>
        <p>{callout.explanation}</p>
        {callout.recommendation && <p>💡 {callout.recommendation}</p>}
        <div>
          <span>Value: {callout.current_value.toFixed(2)}</span>
          {callout.delta !== null && (
            <span>Change: {callout.delta >= 0 ? '+' : ''}{callout.delta.toFixed(2)}</span>
          )}
        </div>
      </div>

      {needsAcknowledge && (
        <div>
          <label>
            <input
              type="checkbox"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
            />
            I understand
          </label>
          <button
            onClick={onAcknowledge}
            disabled={!confirmed}
          >
            Acknowledge
          </button>
        </div>
      )}

      {callout.acknowledged && <span>✓ Acknowledged</span>}
    </div>
  );
};
```

**Bulk Acknowledgment Footer:**
```typescript
const AcknowledgeAllFooter: React.FC = ({ pendingCount, onAcknowledgeAll }) => {
  const [confirmed, setConfirmed] = useState(false);

  return (
    <div className="p-4 border-t bg-gray-800/50">
      <span>{pendingCount} callout{pendingCount !== 1 ? 's' : ''} pending acknowledgment</span>

      <label>
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
        />
        I have reviewed these concerns and understand the risks
      </label>

      <button
        onClick={onAcknowledgeAll}
        disabled={!confirmed}
      >
        Acknowledge All
      </button>
    </div>
  );
};
```

### TypeScript Types

**Location:** `src/types/callouts.ts`

**FFI Contract (verified via Task 0 discovery):**
- Enums serialize as **PascalCase strings** (e.g., `"Architecting"`, `"Critical"`)
- Struct fields serialize as **snake_case** (e.g., `metric_name`, `ci_baseline`)
- No `rename_all` or `rename` attributes in Rust

**Types:**
```typescript
export type CalloutTier = 'Info' | 'Attention' | 'Warning' | 'Critical';
export type StructureMode = 'Architecting' | 'Builder' | 'Refining';

export interface Callout {
  id: string;
  tier: CalloutTier;
  original_tier: CalloutTier | null;
  metric_name: string;
  current_value: number;
  previous_value: number | null;
  delta: number | null;
  threshold_context: string;
  explanation: string;
  recommendation: string;
  requires_acknowledgment: boolean;
  acknowledged: boolean;
}

export interface CalloutCountByTier {
  info: number;
  attention: number;
  warning: number;
  critical: number;
}

export interface CalloutSummary {
  total: number;
  by_tier: CalloutCountByTier;
  pending_acknowledgments: number;
  can_proceed: boolean;
}

export interface ModeInfo {
  mode: StructureMode | null;
  display_name: string | null;
  user_message: string | null;
  ci_baseline: number | null;
  confidence: number | null;
  is_locked: boolean;
}

export const TIER_COLORS: Record<CalloutTier, {
  bg: string;
  border: string;
  text: string;
  label: string;
}> = {
  Info: { bg: 'bg-blue-900/30', border: 'border-blue-500', text: 'text-blue-400', label: 'Info' },
  Attention: { bg: 'bg-yellow-900/30', border: 'border-yellow-500', text: 'text-yellow-400', label: 'Minor' },
  Warning: { bg: 'bg-orange-900/30', border: 'border-orange-500', text: 'text-orange-400', label: 'Important' },
  Critical: { bg: 'bg-red-900/30', border: 'border-red-500', text: 'text-red-400', label: 'Must Review' },
};

export const MODE_COLORS: Record<StructureMode, { bg: string; text: string }> = {
  Architecting: { bg: 'bg-purple-900/30', text: 'text-purple-400' },
  Builder: { bg: 'bg-blue-900/30', text: 'text-blue-400' },
  Refining: { bg: 'bg-green-900/30', text: 'text-green-400' },
};
```

### Gate Blocking Integration

**Location:** `src/pages/RunView.tsx` (lines 44-57)

**Purpose:** Prevent gate approval if Critical callouts pending

**Implementation:**
```typescript
const handleGateReached = async (summary: IntentSummary) => {
  console.log('[RunView] Gate reached with summary:', summary);

  // Check if critical callouts have been acknowledged
  try {
    const canProceed = await calloutApi.canProceed();
    if (!canProceed) {
      alert('Please acknowledge Critical callouts before proceeding');
      return;  // Block gate approval
    }
  } catch (error) {
    console.error('[RunView] Failed to check can_proceed:', error);
    // Continue anyway if check fails (fail-open to prevent blocking)
  }

  // Show confirmation dialog (existing logic)
  const confirmed = window.confirm(
    `Ready to proceed to Step 1?\n\nNormalized Goal: ${summary.normalized_goal}\n\nClick OK to approve.`
  );

  if (confirmed) {
    await invoke('approve_gate', { approver: 'User' });
    setCurrentStep(1);
  }
};
```

**Behavior:**
- **Before acknowledgment:** Alert shown, gate stays closed
- **After acknowledgment:** Normal confirmation dialog, gate opens
- **On error:** Fail-open (allow progression to prevent deadlock)

### Auto-Refresh Strategy

**StatusBar:** Polls `getCalloutSummary()` every 5 seconds
- Updates badge count and colors
- Re-fetches after acknowledgment

**ModeBadge:** Polls `getCurrentMode()` every 5 seconds
- Shows "Detecting..." until mode locked
- Updates confidence percentage

**CalloutPanel:** Fetches on open + after acknowledgment
- Always shows current state
- No stale data

**Implementation Pattern:**
```typescript
useEffect(() => {
  fetchData();
  const interval = setInterval(fetchData, 5000);
  return () => clearInterval(interval);
}, []);
```

---

## Database Architecture

*(No changes from previous version - database not involved in callout/mode system)*

**Database Path:** `~/AppData/Roaming/com.ryanb.method-vi/method-vi.db` (Windows)

**Schema:** 7 tables
- `runs` - Run metadata
- `artifacts` - Step outputs
- `spine_edges` - Knowledge graph
- `patterns` - Reusable patterns
- `ledger` - Decision audit trail
- `flaws` - Persistent issues
- `runs_audit` - Run history snapshots

**Note:** Callouts and mode detection are stored **in-memory only** (not persisted to database). They exist for the duration of the run and are discarded at closure.

---

## Metrics & Governance System

### Overview

The governance system calculates 6 metrics at each step and generates callouts based on performance:

1. **CI (Coherence Index)** - Logical flow, term consistency, clarity, structure
2. **EV (Expansion Variance)** - Content growth stability (informational only)
3. **IAS (Intent Alignment Score)** - Charter alignment (soft/hard gates)
4. **EFI (Evidence Fidelity Index)** - Claim substantiation (Step 6 only)
5. **SEC (Scope Expansion Count)** - Scope drift detection (informational)
6. **PCI (Process Compliance Index)** - Workflow adherence (Step 6 only)

### Metric Flow

```
┌────────────────────────────────────────────────────┐
│     Step Execution (Orchestrator)                  │
│  - User provides input                             │
│  - Agent generates output                          │
│  - Content stored in artifacts                     │
└────────────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────────────┐
│  Metric Calculation (GovernanceTelemetryAgent)     │
│  ┌──────────────────────────────────────────────┐  │
│  │ For each metric:                             │  │
│  │ 1. Retrieve mode (if detected)               │  │
│  │ 2. Resolve threshold (mode-aware adjustment) │  │
│  │ 3. Call LLM with strict JSON prompt          │  │
│  │ 4. Parse result                               │  │
│  │ 5. Compare to threshold                       │  │
│  │ 6. Return MetricResult                        │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────────────┐
│    Callout Generation (CalloutManager)             │
│  - Determine tier for each metric                  │
│  - Apply noise filter (mode-aware)                 │
│  - Generate explanation & recommendation           │
│  - Store callouts                                  │
└────────────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────────────┐
│     Gate Check (Orchestrator)                      │
│  - Check: can_proceed() → all Critical ack'd?      │
│  - If yes: allow gate approval                     │
│  - If no: block until acknowledged                 │
└────────────────────────────────────────────────────┘
```

### Metric Details

#### CI (Coherence Index)
**Purpose:** Measure logical flow, term consistency, clarity, and structure

**Calculation:**
1. LLM evaluates 4 components (0.0-1.0 each):
   - Logical Flow (40-50% weight)
   - Term Consistency (15% weight)
   - Sentence Clarity (15-30% weight)
   - Structural Organization (5-30% weight)
2. Step-semantic weighted average
3. Temperature = 0.0 (deterministic)
4. Return final score (0.0-1.0)

**Step-Semantic Weights:**
- **Steps 0-4:** Flow 50%, Terms 15%, Clarity 30%, Structure 5%
- **Step 5+:** Flow 40%, Terms 15%, Clarity 15%, Structure 30%

**Thresholds (Builder mode baseline):**
- Pass: ≥ 0.70
- Warning: 0.50-0.69
- Fail/HALT: < 0.50

**Mode Adjustment:**
- Architecting: 0.70 × 0.71 = 0.497 (pass)
- Builder: 0.70 × 1.0 = 0.70 (pass)
- Refining: 0.70 × 1.10 = 0.77 (pass)

**Enforcement:** All steps, triggers Critical callout on hard fail

**Recent Changes:**
- FIX-021: Made deterministic (0.0000 variance)
- FIX-022: Implemented step-semantic weighting
- Phase 5: Mode-aware threshold adjustment

#### EV (Expansion Variance)
**Purpose:** Detect content growth instability (informational only)

**Calculation:**
1. Calculate entropy of current content (E_current)
2. Compare to E_baseline (locked at Step 1)
3. Return percentage variance: `|(E_current - E_baseline) / E_baseline| * 100`

**Thresholds:**
- Pass: ≤ 10%
- Warning: 10-20%
- "Fail": > 20% (but never enforced)

**Enforcement:** NONE - Always Pass status, never triggers callouts

**Recent Changes:**
- FIX-027: Status always Pass, never triggers HALT
- Phase 5: Info-tier callout only (never Warning/Critical)

#### IAS (Intent Alignment Score)
**Purpose:** Measure alignment with charter intent

**Calculation:**
1. LLM compares content to charter
2. Returns score 0.0-1.0 (alignment strength)
3. Temperature = 0.3 (slight variance allowed for interpretation)

**Thresholds:**
- Pass: ≥ 0.70
- Soft Gate (Attention): 0.30-0.69
- Hard Gate (Critical): < 0.30

**Gate Behavior:**
- **0.70-1.00:** Pass → Info callout
- **0.30-0.69:** Soft gate → Attention callout (drift detected, no blocking)
- **< 0.30:** Hard gate → Critical callout (extreme drift, blocks progression)

**Enforcement:** All steps, triggers Critical callout on hard gate

**Recent Changes:**
- FIX-024: Implemented soft gate (0.30-0.70 warning range)
- Phase 5: Soft gate → Attention tier (not Warning)

#### EFI (Evidence Fidelity Index)
**Purpose:** Measure claim substantiation quality (Step 6 only)

**Calculation:**
1. LLM identifies all claims (factual/predictive/prescriptive)
2. Ignores instructional/procedural/observational statements
3. Evaluates evidence for each claim
4. Returns: `substantiated_claims / total_claims`
5. Temperature = 0.0 (deterministic)

**Claim Taxonomy:**
- **Scored (require evidence):**
  - Factual: "X is Y"
  - Predictive: "X will cause Y"
  - Prescriptive: "Always do X" (normative)
- **Ignored (no evidence needed):**
  - Instructional: "Run command X"
  - Procedural: "Step 1, Step 2..."
  - Observational: "We see X"

**Thresholds:**
- Pass: ≥ 0.80
- Warning: 0.50-0.79
- Fail/Critical: < 0.50

**Enforcement:** Step 6 ONLY, triggers Critical callout on hard fail

**Recent Changes:**
- FIX-025: Implemented claim taxonomy
- FIX-005: Made calculation consistent across steps 2-6
- Phase 5: Mode-aware threshold adjustment

#### SEC (Scope Expansion Count)
**Purpose:** Detect scope drift (informational only)

**Calculation:** Placeholder (always returns 0.0)

**Thresholds:** Not defined (informational metric)

**Enforcement:** NONE - Informational only

**Status:** Placeholder implementation, planned for future

#### PCI (Process Compliance Index)
**Purpose:** Measure workflow adherence (Step 6 only)

**Calculation:** Deterministic checklist evaluation

**Thresholds:**
- Pass: All items checked
- Fail: Any item unchecked

**Enforcement:** Step 6 ONLY, triggers Critical callout on failure

**Recent Changes:**
- FIX-026: Converted to deterministic checklist

### Enforcement Matrix

| Metric | Steps | Enforced? | Critical Trigger | Attention Trigger | Info |
|--------|-------|-----------|-----------------|-------------------|------|
| CI     | 0-6   | Yes       | < mode threshold | 0.50-0.70 (mode-adjusted) | ≥ 0.70 |
| EV     | 2-6   | No        | Never           | Never             | Always |
| IAS    | 0-6   | Yes       | < 0.30          | 0.30-0.70         | ≥ 0.70 |
| EFI    | 6     | Yes       | < 0.50          | 0.50-0.80         | ≥ 0.80 |
| SEC    | 2-6   | No        | Never           | Never             | Always |
| PCI    | 6     | Yes       | Checklist fail  | N/A               | Pass |

---

## Data Flow Patterns

### Gate Approval Flow (with Callouts)

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
    │   ├─ Yes → return false
    │   └─ No → return true
    ↓
Frontend: if (!canProceed) { alert(...); return; }
    ↓
    ├─ Blocked → User must acknowledge Critical callouts
    │   ↓
    │   User clicks CalloutBadge → CalloutPanel opens
    │   ↓
    │   User reviews callout, checks "I understand"
    │   ↓
    │   User clicks "Acknowledge" → calloutApi.acknowledgeCallout()
    │   ↓
    │   Backend: callout.acknowledged = true
    │   ↓
    │   User tries gate approval again → canProceed = true
    ↓
    └─ Allowed → Show confirmation dialog
        ↓
        User confirms → invoke('approve_gate')
        ↓
        Backend: Orchestrator approves gate
        ↓
        Transition to next step
```

### Mode Detection Flow

```
Step 1 completes
    ↓
Orchestrator: calculate_step_metrics(step=1)
    ↓
GovernanceTelemetryAgent: CI baseline calculated
    ↓
Orchestrator: detect_and_lock_mode(ci_baseline)
    ↓
ModeDetector: detect(ci_baseline)
    ↓
    ├─ CI < 0.50 → Architecting
    ├─ 0.50 ≤ CI < 0.80 → Builder
    └─ CI ≥ 0.80 → Refining
    ↓
Calculate confidence (distance from boundaries)
    ↓
Create ModeDetectionResult
    ↓
Orchestrator: Store result, lock mode for run
    ↓
LedgerManager: Log mode detection (Transparency Mandate)
    ↓
Frontend: ModeBadge polls getCurrentMode() every 5s
    ↓
UI updates: "Architecting (85%)" with color + lock icon
```

### Metric Calculation Flow (with Mode Awareness)

```
Step completes
    ↓
Orchestrator: calculate_step_metrics(step)
    ↓
GovernanceTelemetryAgent: calculate_metrics()
    ↓
For each metric (CI, EV, IAS, EFI, SEC, PCI):
    ↓
    1. Retrieve mode_detection_result (if available)
    ↓
    2. Resolve threshold
       ├─ Architecting → base × 0.71
       ├─ Builder → base × 1.0
       └─ Refining → base × 1.10
    ↓
    3. Build LLM prompt (strict JSON, temperature control)
    ↓
    4. Call Anthropic API
    ↓
    5. Parse JSON response
    ↓
    6. Compare value to threshold
    ↓
    7. Return MetricResult {
         metric_name, value, threshold, status,
         inputs_used, calculation_method,
         interpretation, recommendation
       }
    ↓
Return CriticalMetrics (all 6 results)
    ↓
Orchestrator: generate_callouts_for_step(metrics, mode)
    ↓
CalloutManager: generate_callouts()
    ↓
For each metric result:
    ↓
    1. Determine tier (Pass→Info, Warning→Warning/Attention, Fail→Critical/Warning)
    ↓
    2. Apply noise filter
       ├─ Architecting mode + Warning → Attention
       └─ Other → Keep original tier
    ↓
    3. Generate explanation & recommendation
    ↓
    4. Create Callout struct
    ↓
    5. Store in HashMap<String, Callout>
    ↓
Return Vec<Callout>
    ↓
Frontend: StatusBar auto-refreshes → see new callouts in badge
```

---

## File Structure & Relationships

### Project Root
```
method-vi-app/
├── method-vi/                           # Tauri frontend
│   ├── src/
│   │   ├── components/                  # React components
│   │   │   ├── layout/
│   │   │   │   └── Header.tsx           # Header with StatusBar integration
│   │   │   ├── steps/                   # Step-specific views
│   │   │   ├── metrics/                 # Metric display components
│   │   │   ├── CalloutBadge.tsx         # NEW: Callout summary badge
│   │   │   ├── CalloutPanel.tsx         # NEW: Callout review modal
│   │   │   ├── ModeBadge.tsx            # NEW: Mode display badge
│   │   │   └── StatusBar.tsx            # NEW: Container for badges
│   │   ├── pages/
│   │   │   └── RunView.tsx              # Modified: Gate blocking logic
│   │   ├── types/
│   │   │   ├── index.ts                 # Modified: Export callout types
│   │   │   ├── callouts.ts              # NEW: Callout/Mode TypeScript types
│   │   │   └── metrics.ts               # Existing: Metric types
│   │   ├── utils/
│   │   │   └── calloutApi.ts            # NEW: Tauri command wrappers
│   │   └── App.tsx
│   ├── src-tauri/                       # Rust backend
│   │   ├── src/
│   │   │   ├── agents/
│   │   │   │   ├── orchestrator.rs      # Modified: Callout integration
│   │   │   │   ├── governance_telemetry.rs  # Modified: Mode-aware thresholds
│   │   │   │   ├── analysis_synthesis.rs
│   │   │   │   ├── baseline_report.rs
│   │   │   │   ├── charter_scope.rs
│   │   │   │   ├── structure_redesign.rs
│   │   │   │   └── validation_learning.rs
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs               # Modified: Register callout/mode commands
│   │   │   │   ├── callout_commands.rs  # NEW: 6 callout Tauri commands
│   │   │   │   ├── mode_commands.rs     # NEW: 2 mode Tauri commands
│   │   │   │   ├── step0.rs
│   │   │   │   ├── step1.rs
│   │   │   │   ├── step2.rs
│   │   │   │   ├── step3.rs
│   │   │   │   ├── step4.rs
│   │   │   │   ├── step5.rs
│   │   │   │   ├── step6.rs
│   │   │   │   ├── step6_5.rs
│   │   │   │   ├── closure.rs
│   │   │   │   └── validate.rs
│   │   │   ├── governance/              # NEW: Progression architecture
│   │   │   │   ├── mod.rs               # Module exports
│   │   │   │   ├── callouts.rs          # CalloutManager, tier determination
│   │   │   │   ├── types.rs             # StructureMode, ModeDetectionResult
│   │   │   │   └── integration_tests.rs # 68 governance tests
│   │   │   ├── api/
│   │   │   │   └── anthropic.rs         # Claude API client
│   │   │   ├── database/
│   │   │   ├── ledger/
│   │   │   ├── spine/
│   │   │   ├── signals/
│   │   │   └── lib.rs                   # Modified: Register commands
│   │   └── Cargo.toml
│   └── package.json
├── ARCHITECTURE-2026-01-05-Phase5-Progression.md  # THIS FILE
├── ARCHITECTURE-2026-01-01-Post_Test_Run_8.md     # Previous version
├── PHASE-5-TESTING-CHECKLIST.md                   # Manual test cases
└── .gitignore                                      # Updated: Secret files
```

### Key File Relationships

**Orchestrator → Callout System:**
```
orchestrator.rs
├─ Calls: callout_manager.generate_callouts(metrics, step, mode)
├─ Calls: callout_manager.can_proceed()
└─ Stores: callouts in OrchestratorState
```

**Governance Agent → Mode Detection:**
```
governance_telemetry.rs
├─ Receives: mode_detection_result from orchestrator
├─ Calls: resolve_threshold(base, mode)
└─ Returns: MetricResult with mode-adjusted thresholds
```

**Frontend → Backend:**
```
RunView.tsx
├─ Imports: calloutApi from utils/calloutApi.ts
├─ Calls: calloutApi.canProceed() before gate approval
└─ Blocks: gate if Critical callouts pending

StatusBar.tsx
├─ Imports: calloutApi from utils/calloutApi.ts
├─ Polls: getCalloutSummary() every 5s
└─ Renders: CalloutBadge + ModeBadge

calloutApi.ts
├─ Imports: invoke from @tauri-apps/api/core
├─ Wraps: 7 Tauri commands (6 callout + 1 mode)
└─ Returns: Typed responses (Callout[], CalloutSummary, ModeInfo)

callout_commands.rs
├─ Receives: Tauri IPC calls from frontend
├─ Accesses: OrchestratorState.callout_manager
└─ Returns: Serialized JSON responses
```

---

## Testing Infrastructure

### Backend Tests

**Location:** `src-tauri/src/governance/integration_tests.rs`

**Governance Tests:** 68 tests passing

**Test Categories:**
1. **Mode Detection** (8 tests)
   - `test_mode_detection_architecting()` - CI < 0.50 → Architecting
   - `test_mode_detection_builder()` - 0.50 ≤ CI < 0.80 → Builder
   - `test_mode_detection_refining()` - CI ≥ 0.80 → Refining
   - `test_confidence_scoring()` - Boundary distance → confidence
   - `test_mode_locking()` - Mode immutable after detection
   - `test_mode_user_messages()` - Correct message per mode
   - `test_mode_display_names()` - Correct display name per mode
   - `test_mode_threshold_adjustment()` - Correct multiplier applied

2. **Callout Generation** (12 tests)
   - `test_callout_tier_determination()` - Pass/Warning/Fail → Info/Warning/Critical
   - `test_noise_filter_architecting()` - Warning → Attention in Architecting
   - `test_noise_filter_builder()` - No demotion in Builder
   - `test_noise_filter_refining()` - No demotion in Refining
   - `test_callout_explanation_generation()` - Meaningful explanations
   - `test_callout_recommendation_generation()` - Actionable recommendations
   - `test_callout_delta_calculation()` - Correct delta from previous step
   - `test_callout_threshold_context()` - Mode + threshold logged
   - `test_callout_original_tier_preserved()` - Audit trail for noise filter
   - `test_callout_acknowledgment_requirement()` - Critical requires ack
   - `test_callout_id_uniqueness()` - UUIDs unique per callout
   - `test_callout_storage()` - HashMap persistence during run

3. **Acknowledgment** (8 tests)
   - `test_acknowledge_callout()` - Mark callout as acknowledged
   - `test_acknowledge_all_callouts()` - Bulk acknowledgment
   - `test_can_proceed_no_critical()` - True if no Critical
   - `test_can_proceed_critical_unacknowledged()` - False if Critical pending
   - `test_can_proceed_critical_acknowledged()` - True after acknowledged
   - `test_acknowledgment_timestamp()` - Timestamp recorded
   - `test_acknowledgment_user_confirmation()` - User message stored
   - `test_pending_acknowledgments_count()` - Correct count returned

4. **Soft Gates** (6 tests)
   - `test_ias_soft_gate_attention()` - IAS 0.30-0.70 → Attention
   - `test_ias_hard_gate_critical()` - IAS < 0.30 → Critical
   - `test_ias_pass_info()` - IAS ≥ 0.70 → Info
   - `test_soft_gate_no_blocking()` - Attention callouts don't block
   - `test_hard_gate_blocking()` - Critical callouts block
   - `test_soft_gate_noise_filter_interaction()` - Attention unaffected by filter

5. **Threshold Resolution** (10 tests)
   - `test_threshold_architecting_multiplier()` - ×0.71 applied
   - `test_threshold_builder_no_change()` - ×1.0 applied
   - `test_threshold_refining_multiplier()` - ×1.10 applied
   - `test_ci_threshold_architecting()` - 0.70 → 0.497
   - `test_ci_threshold_builder()` - 0.70 → 0.70
   - `test_ci_threshold_refining()` - 0.70 → 0.77
   - `test_ias_threshold_architecting()` - 0.70 → 0.497
   - `test_ias_threshold_builder()` - 0.70 → 0.70
   - `test_ias_threshold_refining()` - 0.70 → 0.77
   - `test_efi_threshold_architecting()` - 0.80 → 0.568

6. **Callout Summary** (6 tests)
   - `test_summary_total_count()` - Correct total
   - `test_summary_by_tier_counts()` - Correct per-tier breakdown
   - `test_summary_pending_count()` - Correct pending count
   - `test_summary_can_proceed_flag()` - Correct boolean
   - `test_summary_empty()` - Empty summary if no callouts
   - `test_summary_all_acknowledged()` - Pending = 0 after ack all

7. **Mode Constraints** (8 tests)
   - `test_transparency_mandate_logging()` - Mode logged to ledger
   - `test_transparency_mandate_ui_visibility()` - Mode in ModeInfo
   - `test_delta_baseline_rule_step3_to_4()` - CI_step4 ≥ CI_step3
   - `test_delta_baseline_rule_regression_callout()` - Callout on backslide
   - `test_noise_filter_constraint()` - Warning → Attention in Architecting
   - `test_noise_filter_critical_unaffected()` - Critical never demoted
   - `test_critical_only_blocking()` - Only Critical blocks gates
   - `test_non_critical_progression_allowed()` - Warning/Attention don't block

8. **Integration** (10 tests)
   - `test_full_architecting_flow()` - E2E: CI 0.35 → Architecting → adjusted thresholds
   - `test_full_builder_flow()` - E2E: CI 0.65 → Builder → standard thresholds
   - `test_full_refining_flow()` - E2E: CI 0.85 → Refining → elevated thresholds
   - `test_mode_change_blocked_mid_run()` - Mode immutable after lock
   - `test_callouts_persist_across_steps()` - Callouts visible in later steps
   - `test_acknowledgment_persists()` - Acknowledged state preserved
   - `test_gate_blocking_e2e()` - Critical callout → gate blocked → ack → gate opens
   - `test_multiple_critical_callouts()` - All must be acknowledged
   - `test_mixed_tiers_can_proceed()` - Only Critical matters
   - `test_orchestrator_callout_integration()` - Orchestrator generates callouts correctly

**Running Backend Tests:**
```bash
cd method-vi/src-tauri
cargo test governance:: --lib
# Output: 68 tests passing
```

### Frontend Tests

**Status:** Manual testing via checklist (automated tests planned)

**Location:** `PHASE-5-TESTING-CHECKLIST.md`

**Test Categories:** 66 manual test cases
1. Basic Integration (2 tests)
2. ModeBadge UX Guardrails (7 tests)
3. CalloutPanel UX Guardrails (10 tests)
4. Gate Integration (6 tests)
5. Auto-Refresh Behavior (2 tests)
6. Visual/UI Tests (4 tests)
7. Edge Cases (4 tests)
8. Console Checks (3 tests)

**Running Manual Tests:**
```bash
cd method-vi
npm run tauri dev
# Open PHASE-5-TESTING-CHECKLIST.md
# Follow test steps and check boxes
```

### Test Coverage

**Backend:**
- ✅ 68/68 governance tests passing (100%)
- ✅ 187/187 total tests passing (100%)
- ✅ All callout tier determination logic covered
- ✅ All mode detection logic covered
- ✅ All threshold adjustment logic covered
- ✅ All acknowledgment flow covered

**Frontend:**
- ✅ 66 manual test cases defined
- ⏳ Automated tests planned (Vitest + React Testing Library)
- ⏳ E2E tests planned (Playwright)

---

## Key Patterns & Interactions

### Pattern 1: Mode-Aware Threshold Resolution

**When:** Metric calculated, threshold needed for comparison

**Pattern:**
```rust
// In governance_telemetry.rs
pub fn calculate_metrics(&self, content: &str, mode: Option<&ModeDetectionResult>) -> CriticalMetrics {
    // Calculate raw CI value
    let ci_value = self.calculate_ci(content)?;

    // Resolve threshold (mode-aware)
    let ci_threshold = if let Some(mode) = mode {
        self.resolve_threshold(0.70, &mode.mode)  // 0.70 = base threshold
    } else {
        0.70  // Fallback if mode not detected yet
    };

    // Compare and determine status
    let status = if ci_value >= ci_threshold {
        MetricStatus::Pass
    } else if ci_value >= ci_threshold * 0.714 {  // Warning band
        MetricStatus::Warning
    } else {
        MetricStatus::Fail
    };

    MetricResult { value: ci_value, threshold: ci_threshold, status, ... }
}
```

**Why:** Enables adaptive governance without changing metric calculation logic

### Pattern 2: Noise Filter Application

**When:** Callout tier determined, mode available

**Pattern:**
```rust
// In callouts.rs
pub fn generate_callouts(&mut self, metrics: &CriticalMetrics, mode: Option<&ModeDetectionResult>) -> Vec<Callout> {
    for metric_result in metrics.all_results() {
        // Determine initial tier
        let tier = self.determine_tier(&metric_result);

        // Apply noise filter (mode-aware demotion)
        let final_tier = if let Some(mode) = mode {
            self.apply_noise_filter(tier, &mode.mode)
        } else {
            tier  // No filter if mode unknown
        };

        // Create callout with final tier
        let callout = Callout {
            tier: final_tier,
            original_tier: tier,  // Preserve for audit
            ...
        };
    }
}

fn apply_noise_filter(&self, tier: CalloutTier, mode: &StructureMode) -> CalloutTier {
    match (tier, mode) {
        (CalloutTier::Warning, StructureMode::Architecting) => CalloutTier::Attention,
        _ => tier,
    }
}
```

**Why:** Prevents false alarms in early-stage work, preserves original tier for audit

### Pattern 3: Critical-Only Gate Blocking

**When:** User attempts gate approval

**Pattern:**
```rust
// Backend: callouts.rs
pub fn can_proceed(&self) -> bool {
    self.callouts.values()
        .filter(|c| c.tier == CalloutTier::Critical)
        .all(|c| c.acknowledged)
}

// Frontend: RunView.tsx
const handleGateReached = async () => {
    const canProceed = await calloutApi.canProceed();
    if (!canProceed) {
        alert('Please acknowledge Critical callouts before proceeding');
        return;  // Block gate
    }
    // Continue with normal gate approval...
};
```

**Why:** Allows progress despite minor issues, forces focus on blocking problems

### Pattern 4: Fail-Open Error Handling

**When:** Frontend callout check fails (network error, backend crash, etc.)

**Pattern:**
```typescript
// Frontend: RunView.tsx
try {
    const canProceed = await calloutApi.canProceed();
    if (!canProceed) {
        alert('Please acknowledge Critical callouts');
        return;
    }
} catch (error) {
    console.error('Failed to check can_proceed:', error);
    // FAIL OPEN: Continue anyway to prevent deadlock
}
```

**Why:** Prevents system from locking user out due to backend errors

### Pattern 5: Auto-Refresh with Polling

**When:** StatusBar needs real-time callout updates

**Pattern:**
```typescript
// Frontend: StatusBar.tsx
const fetchSummary = async () => {
    const data = await calloutApi.getCalloutSummary();
    setSummary(data);
};

useEffect(() => {
    fetchSummary();  // Initial fetch
    const interval = setInterval(fetchSummary, 5000);  // Poll every 5s
    return () => clearInterval(interval);  // Cleanup on unmount
}, []);
```

**Why:** Ensures UI always shows current state without manual refresh

### Pattern 6: Intentional Friction for Acknowledgment

**When:** User attempts to acknowledge Critical callout

**Pattern:**
```typescript
// Frontend: CalloutPanel.tsx
const [confirmed, setConfirmed] = useState(false);

<label>
  <input
    type="checkbox"
    checked={confirmed}
    onChange={(e) => setConfirmed(e.target.checked)}
  />
  I understand
</label>

<button
  onClick={onAcknowledge}
  disabled={!confirmed}  // Button disabled until checked
>
  Acknowledge
</button>
```

**Why:** Forces deliberate action, prevents accidental dismissal of serious issues

### Pattern 7: Mode Locking

**When:** Mode detected at Step 2

**Pattern:**
```rust
// In orchestrator.rs
pub fn detect_and_lock_mode(&mut self, ci_baseline: f64) -> Result<()> {
    if self.mode_detection_result.is_some() {
        return Err(anyhow!("Mode already locked for this run"));
    }

    let result = self.mode_detector.detect(ci_baseline);

    // Log to ledger (Transparency Mandate)
    self.ledger.log_mode_detection(&result)?;

    // Lock mode (immutable for run duration)
    self.mode_detection_result = Some(result);
    self.mode_locked = true;

    Ok(())
}
```

**Why:** Ensures consistency, prevents mode gaming mid-run

### Pattern 8: FFI Type Safety

**When:** Passing data between Rust and TypeScript

**Pattern:**
```rust
// Backend: callouts.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalloutTier {
    Info,       // Serializes as "Info"
    Attention,  // Serializes as "Attention"
    Warning,    // Serializes as "Warning"
    Critical,   // Serializes as "Critical"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Callout {
    pub id: String,
    pub tier: CalloutTier,
    pub metric_name: String,  // snake_case
    // ...
}

// Frontend: callouts.ts
export type CalloutTier = 'Info' | 'Attention' | 'Warning' | 'Critical';  // PascalCase strings

export interface Callout {
  id: string;
  tier: CalloutTier;
  metric_name: string;  // snake_case matches Rust
  // ...
}
```

**Why:** Ensures type safety across FFI boundary, caught at compile time

---

## Complete File Reference

### Backend (Rust)

| File | Lines | Purpose | Phase |
|------|-------|---------|-------|
| `agents/orchestrator.rs` | 2558 | Workflow coordinator, callout integration | 1-5 |
| `agents/governance_telemetry.rs` | 1789 | Metric calculation, mode-aware thresholds | 1-5 |
| `agents/analysis_synthesis.rs` | 856 | Multi-lens analysis | 1-3 |
| `agents/baseline_report.rs` | 423 | Charter and baseline | 1-3 |
| `agents/charter_scope.rs` | 678 | Intent capture and scoping | 1-3 |
| `agents/structure_redesign.rs` | 645 | Framework generation | 1-3 |
| `agents/validation_learning.rs` | 789 | Evidence audit and learning | 1-3 |
| `commands/mod.rs` | 68 | Command registration | 1-5 |
| `commands/callout_commands.rs` | 58 | 6 callout Tauri commands | 5 |
| `commands/mode_commands.rs` | 50 | 2 mode Tauri commands | 5 |
| `commands/step0.rs` | 145 | Intent capture command | 1-3 |
| `commands/step1.rs` | 178 | Baseline command | 1-3 |
| `commands/step2.rs` | 256 | Governance command | 1-3 |
| `commands/step3.rs` | 189 | Analysis command | 1-3 |
| `commands/step4.rs` | 234 | Synthesis command | 1-3 |
| `commands/step5.rs` | 201 | Structure command | 1-3 |
| `commands/step6.rs` | 312 | Validation command | 1-3 |
| `commands/step6_5.rs` | 167 | Learning command | 1-3 |
| `commands/closure.rs` | 245 | Closure command | 1-3 |
| `governance/mod.rs` | 47 | Governance module exports | 4-5 |
| `governance/callouts.rs` | 763 | CalloutManager, tier logic | 4-5 |
| `governance/types.rs` | 452 | StructureMode, ModeDetector | 4-5 |
| `governance/integration_tests.rs` | 671 | 68 governance tests | 4-5 |
| `api/anthropic.rs` | 234 | Claude API client | 1-3 |
| `database/mod.rs` | 123 | Database initialization | 1-3 |
| `ledger/mod.rs` | 456 | Steno-Ledger implementation | 1-3 |
| `spine/mod.rs` | 389 | Knowledge graph tracking | 1-3 |
| `signals/mod.rs` | 178 | Signal router | 1-3 |
| `lib.rs` | 145 | Library root, command registration | 1-5 |
| `Cargo.toml` | 89 | Dependency manifest | 1-5 |

**Total Backend:** ~12,000 lines across 29 files

### Frontend (React/TypeScript)

| File | Lines | Purpose | Phase |
|------|-------|---------|-------|
| `components/layout/Header.tsx` | 54 | Header with StatusBar integration | 1-5 |
| `components/layout/Sidebar.tsx` | 123 | Navigation sidebar | 1-3 |
| `components/layout/MainLayout.tsx` | 42 | Main app layout | 1-3 |
| `components/CalloutBadge.tsx` | 62 | Callout summary badge | 5 |
| `components/CalloutPanel.tsx` | 262 | Callout review modal | 5 |
| `components/ModeBadge.tsx` | 124 | Mode display badge | 5 |
| `components/StatusBar.tsx` | 62 | StatusBar container | 5 |
| `components/ChatInterface.tsx` | 187 | Chat UI | 1-3 |
| `components/GateDialog.tsx` | 201 | Gate approval dialog | 1-3 |
| `components/MetricsBar.tsx` | 234 | Metrics display | 1-3 |
| `components/steps/Step0View.tsx` | 156 | Intent capture view | 1-3 |
| `components/steps/Step1View.tsx` | 189 | Baseline view | 1-3 |
| `components/steps/Step2View.tsx` | 178 | Governance view | 1-3 |
| `components/steps/Step3View.tsx` | 145 | Analysis view | 1-3 |
| `components/steps/Step4View.tsx` | 167 | Synthesis view | 1-3 |
| `components/steps/Step5View.tsx` | 134 | Structure view | 1-3 |
| `components/steps/Step6View.tsx` | 198 | Validation view | 1-3 |
| `components/steps/Step6_5View.tsx` | 143 | Learning view | 1-3 |
| `components/steps/ClosureView.tsx` | 176 | Closure view | 1-3 |
| `pages/Home.tsx` | 89 | Home page | 1-3 |
| `pages/RunView.tsx` | 245 | Run view with gate blocking | 1-5 |
| `pages/Settings.tsx` | 123 | Settings page | 1-3 |
| `types/index.ts` | 214 | Type exports | 1-5 |
| `types/callouts.ts` | 98 | Callout/Mode types | 5 |
| `types/metrics.ts` | 234 | Metric types | 1-3 |
| `utils/calloutApi.ts` | 22 | Callout API wrappers | 5 |
| `App.tsx` | 67 | App root | 1-3 |
| `main.tsx` | 12 | Entry point | 1-3 |
| `package.json` | 45 | Dependency manifest | 1-5 |
| `vite.config.ts` | 23 | Vite configuration | 1-3 |

**Total Frontend:** ~4,000 lines across 30 files

### Documentation

| File | Lines | Purpose | Phase |
|------|-------|---------|-------|
| `ARCHITECTURE-2026-01-05-Phase5-Progression.md` | ~3000 | THIS FILE | 5 |
| `ARCHITECTURE-2026-01-01-Post_Test_Run_8.md` | 2165 | Previous architecture doc | 1-4 |
| `PHASE-5-TESTING-CHECKLIST.md` | 239 | Manual test cases | 5 |
| `README.md` | 145 | Project overview | 1-3 |
| `.gitignore` | 6 | Secret file exclusions | 5 |

**Total Project:** ~16,000 lines across 64 files

---

## Recent Changes

### Phase 5: Progression Architecture (2026-01-05)

**Summary:** Complete frontend integration of callout and mode detection systems, establishing Method-VI as a progression-focused governance engine rather than a binary gatekeeper.

**Philosophy Shift:**
- **From:** Absolute thresholds → Binary pass/fail → HALT on failure
- **To:** Relative improvement → Tiered feedback → Critical-only blocking

**Backend Changes:**

1. **Callout System** (3 new files, 1886 lines)
   - `governance/mod.rs` - Module exports
   - `governance/callouts.rs` - CalloutManager, tier determination, acknowledgment
   - `governance/types.rs` - StructureMode, ModeDetector, confidence scoring
   - `governance/integration_tests.rs` - 68 governance tests

2. **Tauri Commands** (2 new files, 108 lines)
   - `commands/callout_commands.rs` - 6 callout commands
   - `commands/mode_commands.rs` - 2 mode commands

3. **Orchestrator Integration** (orchestrator.rs)
   - Added `callout_manager: CalloutManager`
   - Added `mode_detector: ModeDetector`
   - Added `mode_detection_result: Option<ModeDetectionResult>`
   - Added `generate_callouts_for_step()` method
   - Modified gate approval to check `can_proceed()`

4. **Governance Agent Updates** (governance_telemetry.rs)
   - Added `resolve_threshold()` for mode-aware adjustment
   - Modified all metric calculations to use mode-adjusted thresholds
   - Added mode parameter to `calculate_metrics()`

**Frontend Changes:**

1. **New Components** (6 files, 632 lines)
   - `components/StatusBar.tsx` - Container for badges
   - `components/ModeBadge.tsx` - Mode display with popover
   - `components/CalloutBadge.tsx` - Callout summary badge
   - `components/CalloutPanel.tsx` - Callout review modal
   - `types/callouts.ts` - TypeScript FFI types
   - `utils/calloutApi.ts` - Tauri command wrappers

2. **Integration Points**
   - `layout/Header.tsx` - Added StatusBar (line 34)
   - `pages/RunView.tsx` - Added gate blocking check (lines 47-57)
   - `types/index.ts` - Export callout types

**Testing:**

1. **Backend Tests**
   - Added 68 governance integration tests
   - All 187 total tests passing
   - Covers: mode detection, callout generation, acknowledgment, threshold resolution

2. **Frontend Tests**
   - Created 66 manual test cases
   - Automated tests planned (Vitest)

**UX Guardrails:**

1. **ModeBadge**
   - ✅ "Detecting..." not "Pending"
   - ✅ Lock icon tooltip
   - ✅ Helpful tooltip before detection

2. **CalloutPanel**
   - ✅ Human-friendly tier labels ("Must Review" not "Critical")
   - ✅ Checkbox friction for acknowledgment
   - ✅ "Change:" instead of "Delta:"

3. **Gate Blocking**
   - ✅ Critical-only blocking
   - ✅ Fail-open on error

**Breaking Changes:** None (additive only)

**Migration Notes:** Existing runs not affected. New mode detection only applies to runs started after Phase 5 deployment.

---

### Phase 4: Metrics Redesign (2026-01-01)

**Summary:** Completed FIX-021 through FIX-027, establishing deterministic metrics, claim taxonomy, and soft gates.

**Key Changes:**

1. **FIX-021:** CI deterministic (temperature = 0.0, variance = 0.0000)
2. **FIX-022:** Step-semantic CI weights (Flow 50%→40% at Step 5+)
3. **FIX-023:** EV triggered metrics filter (only on entropy anomalies)
4. **FIX-024:** IAS soft gate (0.30-0.70 warning, < 0.30 HALT)
5. **FIX-025:** EFI claim taxonomy (factual/predictive/prescriptive only)
6. **FIX-026:** PCI deterministic checklist
7. **FIX-027:** EV status always Pass (informational only)

**Testing:** All 116 tests passing

**Documentation:** ARCHITECTURE-2026-01-01-Post_Test_Run_8.md created

---

### Phases 1-3: Foundation (2025-12-15 to 2025-12-31)

**Phase 1:** Core agent system, workflow orchestration, ledger/spine

**Phase 2:** Metrics calculation, threshold evaluation, HALT conditions

**Phase 3:** Frontend UI, step views, metrics display, gate dialogs

**Testing:** 48 backend tests, 0 frontend tests (manual only)

---

## Conclusion

Method-VI's Phase 5 progression architecture transforms the system from a binary gatekeeper into an adaptive progression engine. By measuring relative improvement from any starting point, applying mode-aware threshold adjustments, and using tiered callouts instead of HALT conditions, Method-VI now enables users to make progress while maintaining quality guardrails.

The four non-negotiable constraints (Transparency Mandate, Delta Baseline Rule, Noise Filter, Critical-Only Blocking) ensure that the system remains rigorous and auditable while supporting users at all structure levels—from early exploration (Architecting) to final polish (Refining).

With 68 governance tests passing and 66 manual test cases defined, Phase 5 is production-ready and marks a significant milestone in Method-VI's evolution toward intelligent, adaptive quality governance.

**Next Steps:**
1. Automated frontend tests (Vitest + React Testing Library)
2. E2E tests for full user workflows (Playwright)
3. User acceptance testing with live callout generation
4. Mode detection refinement based on user feedback
5. Additional metrics (SEC placeholder, new governance dimensions)

---

**End of Architecture Documentation**
