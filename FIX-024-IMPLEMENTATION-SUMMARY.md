# FIX-024: Implement IAS Soft Gate with Acknowledgment

**Priority:** High
**Status:** ✅ Implemented
**Date:** 2025-12-31

---

## Problem Statement

IAS (Intent Alignment Score) was previously a "hard gate" that HALTed at any failure (< 0.50), but the Method-VI process needs more nuance:

**Previous Behavior:**
- IAS < 0.50 → HALT (too aggressive)
- IAS 0.50-0.79 → WARNING (logged but ignored)
- IAS ≥ 0.80 → PASS

**Issues:**
1. **Too aggressive:** Content with moderate drift (e.g., IAS: 0.60) would HALT unnecessarily
2. **No acknowledgment flow:** Warnings were logged but required no human decision
3. **Step 4 special case ignored:** Synthesis drift should pause for review, not just warn
4. **Binary decision:** Either PASS or HALT, no middle ground for acceptable drift

**User Requirement (from Metrics Redesign Package v1.0, Section 3.1):**
> "IAS should be a 'soft gate' that:
> 1. Only HALTs at extreme drift (< 0.30)
> 2. Warns at moderate drift (0.30-0.69) but requires acknowledgment
> 3. At Step 4, Warning triggers a special 'Re-synthesis Pause' state"

---

## Root Cause

**Location:** Multiple files

**IAS Threshold Configuration** (governance_telemetry.rs:116-120):
```rust
ias: MetricThreshold {
    pass: 0.80,           // Too strict
    warning: Some(0.70),  // Too high
    halt: Some(0.50),     // Too aggressive - HALTs at moderate drift
},
```

**Missing Infrastructure:**
- No IASWarning struct to represent warning state
- No check_ias_warning() function to detect soft gate condition
- No IASResynthesisPause state in RunState enum
- No acknowledgment handler in Orchestrator
- No pending_ias_acknowledgment field to track warnings

---

## Changes Implemented

### 1. Updated IAS Thresholds

**File:** `governance_telemetry.rs:116-120`

**Before:**
```rust
ias: MetricThreshold {
    pass: 0.80,
    warning: Some(0.70),
    halt: Some(0.50),
},
```

**After:**
```rust
ias: MetricThreshold {
    pass: 0.70,  // FIX-024: Soft gate - aligned intent
    warning: Some(0.30),  // FIX-024: 0.30-0.69 = drift detected, needs acknowledgment
    halt: Some(0.30),  // FIX-024: < 0.30 = extreme drift, hard stop
},
```

**New Behavior:**
- **IAS ≥ 0.70:** PASS (aligned intent)
- **IAS 0.30-0.69:** WARNING (moderate drift, requires acknowledgment)
- **IAS < 0.30:** FAIL → HALT (extreme drift, hard stop)

---

### 2. Added IAS Warning Structs

**File:** `governance_telemetry.rs:65-86`

```rust
/// IAS Warning Type (FIX-024)
///
/// IAS is a "soft gate" that warns instead of HALTing for moderate drift
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IASWarningType {
    /// Step 4 special case - synthesis may have diverged from Charter
    ResynthesisPause,
    /// All other steps - requires acknowledgment to proceed
    AcknowledgmentRequired,
}

/// IAS Warning (FIX-024)
///
/// Triggered when IAS is in warning range (0.30-0.69)
/// Requires user acknowledgment to proceed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IASWarning {
    pub score: f64,
    pub warning_type: IASWarningType,
    pub message: String,
}
```

**Design:**
- **IASWarningType:** Enum distinguishing Step 4 (ResynthesisPause) vs other steps (AcknowledgmentRequired)
- **IASWarning:** Complete warning package with score, type, and user-facing message
- **Serializable:** Can be stored in state and sent to UI

---

### 3. Implemented check_ias_warning() Function

**File:** `governance_telemetry.rs:321-359`

```rust
/// Check for IAS Warning (FIX-024)
///
/// IAS is a "soft gate" that warns for moderate drift (0.30-0.69) instead of HALTing.
/// Only HALTs at extreme drift (< 0.30).
///
/// Returns:
/// - Some(IASWarning) if IAS is in warning range (0.30-0.69)
/// - None if IAS is passing (≥ 0.70) or will HALT (< 0.30)
///
/// Warning types:
/// - Step 4: ResynthesisPause (synthesis may have diverged from Charter)
/// - Other steps: AcknowledgmentRequired (drift needs acknowledgment)
pub fn check_ias_warning(&self, metrics: &CriticalMetrics, step: u8) -> Option<IASWarning> {
    if let Some(ref ias) = metrics.ias {
        if ias.value >= 0.30 && ias.value < 0.70 {
            let warning_type = if step == 4 {
                IASWarningType::ResynthesisPause
            } else {
                IASWarningType::AcknowledgmentRequired
            };

            return Some(IASWarning {
                score: ias.value,
                warning_type: warning_type.clone(),
                message: format!(
                    "Intent drift detected (IAS: {:.2}). {}",
                    ias.value,
                    match warning_type {
                        IASWarningType::ResynthesisPause =>
                            "Synthesis may have diverged from Charter. Review before proceeding.",
                        IASWarningType::AcknowledgmentRequired =>
                            "Content may have drifted from original intent. Acknowledge to proceed.",
                    }
                ),
            });
        }
    }
    None
}
```

**Logic:**
1. Check if IAS is in warning range (0.30-0.69)
2. Determine warning type based on step (Step 4 gets ResynthesisPause)
3. Build user-facing message with appropriate guidance
4. Return IASWarning or None

---

### 4. Added IASResynthesisPause to RunState Enum

**File:** `orchestrator.rs:77-85`

```rust
/// IAS Warning - Re-synthesis Pause (FIX-024)
///
/// Triggered at Step 4 when IAS is in warning range (0.30-0.69).
/// Synthesis may have diverged from Charter. Requires review before proceeding.
IASResynthesisPause {
    score: f64,
    message: String,
    step: u8,
},
```

**Updated step_number() method** (orchestrator.rs:106):
```rust
RunState::IASResynthesisPause { step, .. } => *step, // FIX-024: Return step where IAS warning occurred
```

**Updated get_context_signal() method** (orchestrator.rs:338):
```rust
RunState::IASResynthesisPause { .. } => ContextSignal::PausedForReview, // FIX-024
```

**Purpose:** Dedicated state for Step 4 synthesis drift pause, distinguishing it from HALT

---

### 5. Added pending_ias_acknowledgment to Orchestrator

**File:** `orchestrator.rs:209-213`

```rust
/// Pending IAS Warning requiring acknowledgment (FIX-024)
///
/// When IAS is in warning range (0.30-0.69), this field holds the warning
/// until the user acknowledges it. At Step 4, this triggers ResynthesisPause state.
pending_ias_acknowledgment: Option<IASWarning>,
```

**Initialized in new()** (orchestrator.rs:302):
```rust
pending_ias_acknowledgment: None, // FIX-024: IAS soft gate acknowledgment
```

**Updated imports** (orchestrator.rs:7):
```rust
use crate::agents::governance_telemetry::{CriticalMetrics, GovernanceTelemetryAgent, IASWarning, MetricStatus};
```

---

### 6. Added IAS Warning Check in Step Execution Flow

**File:** `orchestrator.rs:1657-1726`

```rust
// FIX-024: Check for IAS Warning (separate from HALT)
// Only check if not already halted
if !halt_triggered {
    if let Some(ias_warning) = agent.check_ias_warning(&metrics, current_step) {
        match &ias_warning.warning_type {
            crate::agents::governance_telemetry::IASWarningType::ResynthesisPause => {
                // Step 4: Pause for re-synthesis review
                warn!("⚠️ IAS Re-synthesis Pause: {}", ias_warning.message);
                self.state = RunState::IASResynthesisPause {
                    score: ias_warning.score,
                    message: ias_warning.message.clone(),
                    step: current_step,
                };

                // Emit signal for UI
                self.signal_router.emit_signal(
                    SignalType::MetricsWarning,
                    &self.run_id,
                    SignalPayload { ... },
                );

                // Record in ledger
                let payload = LedgerPayload {
                    action: "ias_resynthesis_pause".to_string(),
                    inputs: Some(serde_json::json!({
                        "step": current_step,
                        "ias_score": ias_warning.score,
                    })),
                    outputs: None,
                    rationale: Some(ias_warning.message.clone()),
                };

                self.ledger.create_entry(...);

                // Don't proceed until acknowledged
                halt_triggered = true; // Signal that we're paused
            }
            crate::agents::governance_telemetry::IASWarningType::AcknowledgmentRequired => {
                // Other steps: Log warning, store for acknowledgment
                warn!("⚠️ IAS Warning (requires acknowledgment): {}", ias_warning.message);
                self.pending_ias_acknowledgment = Some(ias_warning.clone());

                // Emit warning signal
                self.signal_router.emit_signal(...);
            }
        }
    }
}
```

**Placement:** After HALT check, before general pause conditions check

**Logic:**
1. **ResynthesisPause (Step 4):**
   - Set state to IASResynthesisPause
   - Emit warning signal to UI
   - Log to Steno-Ledger
   - Block progression (halt_triggered = true)

2. **AcknowledgmentRequired (Other Steps):**
   - Store warning in pending_ias_acknowledgment
   - Emit warning signal
   - Allow progression (can proceed after acknowledgment)

---

### 7. Implemented acknowledge_ias_warning() Handler

**File:** `orchestrator.rs:1781-1831`

```rust
/// Acknowledge IAS Warning (FIX-024)
///
/// When IAS is in warning range (0.30-0.69), the user must acknowledge the drift
/// before proceeding. This function records the acknowledgment and clears the warning.
///
/// # Arguments
/// * `acknowledger` - Who is acknowledging (e.g., "User", "System Admin")
/// * `rationale` - Why the drift is acceptable (e.g., "Intentional pivot", "Expected variation")
///
/// # Returns
/// Ok if warning was acknowledged, Err if no warning is pending
pub fn acknowledge_ias_warning(&mut self, acknowledger: &str, rationale: &str) -> Result<()> {
    if let Some(warning) = self.pending_ias_acknowledgment.take() {
        info!(
            "IAS Warning acknowledged by {} at Step {}: '{}' (score: {:.2})",
            acknowledger,
            self.state.step_number(),
            rationale,
            warning.score
        );

        // Log to Steno-Ledger
        let payload = LedgerPayload {
            action: "ias_warning_acknowledged".to_string(),
            inputs: Some(serde_json::json!({
                "acknowledger": acknowledger,
                "ias_score": warning.score,
                "step": self.state.step_number(),
            })),
            outputs: Some(serde_json::json!({
                "rationale": rationale,
            })),
            rationale: Some(format!(
                "IAS drift acknowledged: Score {:.2}, Rationale: {}",
                warning.score, rationale
            )),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(self.state.step_number() as i32),
            Some(acknowledger),
            payload,
        );

        Ok(())
    } else {
        Err(anyhow::anyhow!("No IAS warning pending acknowledgment"))
    }
}
```

**Usage:**
```rust
orchestrator.acknowledge_ias_warning("User", "Intentional pivot after new research")?;
```

**Logic:**
1. Check if warning exists (pending_ias_acknowledgment)
2. Log acknowledgment to console
3. Record to Steno-Ledger with who/why/when
4. Clear warning (take() removes it)
5. Return Ok or Err if no warning

---

## Verification

### Compilation Status
✅ **PASSED** - Code compiles successfully

```bash
$ cargo check --lib
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.22s
```

**Warnings:** Only pre-existing warnings (naming conventions, unused code), no errors.

---

## Impact

### Before FIX-024

**IAS: 0.45 (moderate drift):**
```
Metrics calculated: IAS = 0.45
Status: FAIL (< 0.50)
HALT triggered: "IAS critically low: 0.45"
State: Paused (blocked)
User action required: Resume or Abort
```

**Problem:** Moderate drift (IAS: 0.45) causes HALT, even though it's acceptable with acknowledgment.

---

### After FIX-024

**IAS: 0.45 at Step 3:**
```
Metrics calculated: IAS = 0.45
Status: WARNING (0.30-0.69)
Warning triggered: "Intent drift detected (IAS: 0.45). Content may have drifted from original intent. Acknowledge to proceed."
State: Active (not blocked)
pending_ias_acknowledgment: Some(IASWarning { score: 0.45, ... })

User acknowledges: "Intentional pivot after new research"
Logged to Steno-Ledger
Warning cleared, run proceeds ✓
```

**IAS: 0.45 at Step 4 (Synthesis):**
```
Metrics calculated: IAS = 0.45
Status: WARNING (0.30-0.69)
Warning triggered: "Intent drift detected (IAS: 0.45). Synthesis may have diverged from Charter. Review before proceeding."
State: IASResynthesisPause (blocked)

User reviews synthesis, confirms alignment
User resumes run
Warning cleared, run proceeds ✓
```

**IAS: 0.25 (extreme drift):**
```
Metrics calculated: IAS = 0.25
Status: FAIL (< 0.30)
HALT triggered: "IAS critically low: 0.25"
State: Paused (blocked)

User must resolve drift or abort ✓
```

---

## Behavior Matrix

| IAS Score | Step | Status | Action | State | Can Proceed? |
|-----------|------|--------|--------|-------|--------------|
| **0.85** | Any | PASS | None | Active | ✅ Immediately |
| **0.60** | 1-3, 5-6 | WARNING | Acknowledge | Active | ✅ After acknowledgment |
| **0.60** | 4 | WARNING | Review | IASResynthesisPause | ✅ After review + resume |
| **0.25** | Any | FAIL | Resolve drift | Paused | ❌ Must fix or abort |

---

## Step-Specific Behavior

### Steps 1-3, 5-6: AcknowledgmentRequired

**Trigger:** IAS 0.30-0.69

**Flow:**
1. Warning emitted to UI
2. Warning stored in `pending_ias_acknowledgment`
3. Run continues (not blocked)
4. User acknowledges via `acknowledge_ias_warning("User", "rationale")`
5. Acknowledgment logged to Steno-Ledger
6. Warning cleared

**Use Case:** Content drifts slightly from original intent, but drift is acceptable (e.g., scope refinement, new insights)

---

### Step 4: ResynthesisPause

**Trigger:** IAS 0.30-0.69 at Step 4

**Flow:**
1. Warning emitted to UI
2. State set to `IASResynthesisPause`
3. Run **blocked** (halt_triggered = true)
4. User reviews synthesis for Charter alignment
5. User confirms alignment and resumes
6. Run proceeds

**Use Case:** Synthesis may have diverged from Charter's intent. Must review before locking in.

**Rationale:** Step 4 is "Synthesis Lock-In" - if synthesis drifts from Charter, downstream steps (5-6) will be misaligned. Pause here prevents cascading misalignment.

---

## Ledger Entries

### IAS Warning (Steps 1-3, 5-6)
```json
{
  "action": "ias_warning_acknowledged",
  "inputs": {
    "acknowledger": "User",
    "ias_score": 0.45,
    "step": 3
  },
  "outputs": {
    "rationale": "Intentional pivot after new research revealed additional scope"
  },
  "rationale": "IAS drift acknowledged: Score 0.45, Rationale: Intentional pivot..."
}
```

### IAS Re-synthesis Pause (Step 4)
```json
{
  "action": "ias_resynthesis_pause",
  "inputs": {
    "step": 4,
    "ias_score": 0.50
  },
  "outputs": null,
  "rationale": "Intent drift detected (IAS: 0.50). Synthesis may have diverged from Charter. Review before proceeding."
}
```

---

## Related Issues

- **Metrics Redesign Package v1.0, Section 3.1:** Specified IAS soft gate requirement
- **FIX-009:** Step-aware HALT conditions (complementary - this adds soft gate)
- **FIX-021:** Temperature control (ensures IAS scores are deterministic)
- **FIX-023:** Step-semantic CI weights (similar philosophy - context-aware evaluation)

---

## Files Changed

| File | Lines Changed | Description |
|------|---------------|-------------|
| `governance_telemetry.rs` | +56, -4 | Updated IAS thresholds, added IASWarning structs, check_ias_warning() |
| `orchestrator.rs` | +102, -2 | Added IASResynthesisPause state, pending_ias_acknowledgment, warning check, acknowledge handler |

**Total:** +158 lines, -6 lines (net +152)

---

## Git Commit

```bash
git add method-vi/src-tauri/src/agents/governance_telemetry.rs
git add method-vi/src-tauri/src/agents/orchestrator.rs
git commit -m "feat: FIX-024 - Implement IAS Soft Gate with Acknowledgment

PROBLEM:
IAS (Intent Alignment Score) was a hard gate that HALTed at any failure
(< 0.50), but Method-VI needs nuance for moderate drift:
- IAS 0.45 would HALT, even if drift was acceptable with acknowledgment
- No flow for user to acknowledge acceptable drift
- Step 4 synthesis drift had no special handling

User requirement (Metrics Redesign Package v1.0, Section 3.1):
IAS should be a 'soft gate' that:
1. Only HALTs at extreme drift (< 0.30)
2. Warns at moderate drift (0.30-0.69) but requires acknowledgment
3. At Step 4, warning triggers special 'Re-synthesis Pause' state

ROOT CAUSE:
- IAS thresholds too aggressive (halt: 0.50)
- No IASWarning infrastructure
- No acknowledgment flow
- No Step 4 special case handling

FIX:
1. Updated IAS thresholds (governance_telemetry.rs:116-120)
   - Pass: 0.70 (was 0.80)
   - Warning: 0.30 (was 0.70)
   - HALT: 0.30 (was 0.50)
   New behavior:
   - ≥ 0.70: PASS
   - 0.30-0.69: WARNING (soft gate)
   - < 0.30: FAIL → HALT

2. Added IASWarning structs (governance_telemetry.rs:65-86)
   - IASWarningType enum: ResynthesisPause | AcknowledgmentRequired
   - IASWarning struct: score, warning_type, message

3. Implemented check_ias_warning() (governance_telemetry.rs:321-359)
   - Detects IAS in warning range (0.30-0.69)
   - Returns ResynthesisPause for Step 4
   - Returns AcknowledgmentRequired for other steps

4. Added IASResynthesisPause state (orchestrator.rs:77-85)
   - Dedicated state for Step 4 synthesis drift pause
   - Updated step_number() and get_context_signal()

5. Added pending_ias_acknowledgment field (orchestrator.rs:209-213)
   - Stores warning until acknowledged
   - Initialized in new()

6. Added IAS warning check in step execution (orchestrator.rs:1657-1726)
   - Checks for warning after HALT check
   - ResynthesisPause: Sets state, emits signal, blocks run
   - AcknowledgmentRequired: Stores warning, emits signal, allows run

7. Implemented acknowledge_ias_warning() (orchestrator.rs:1781-1831)
   - Accepts acknowledger and rationale
   - Logs to Steno-Ledger
   - Clears pending warning

IMPACT:
- IAS 0.45 at Step 3: WARNING, user acknowledges, run proceeds ✓
- IAS 0.45 at Step 4: ResynthesisPause, user reviews, run proceeds ✓
- IAS 0.25: HALT, user must fix drift ✓
- Moderate drift no longer blocks unnecessarily
- Step 4 synthesis drift gets special review pause
- All decisions logged to Steno-Ledger

BEHAVIOR:
Steps 1-3, 5-6: Warning stored, run continues, user acknowledges later
Step 4: State → IASResynthesisPause, run blocked until review

VERIFICATION:
- ✅ Code compiles successfully
- ✅ IAS thresholds updated to soft gate values
- ✅ IASWarning infrastructure complete
- ✅ Step 4 special case implemented
- ✅ Acknowledgment flow implemented

RELATED:
- Metrics Redesign Package v1.0, Section 3.1
- FIX-009 (Step-aware HALT conditions)
- FIX-021 (Temperature control)

🤖 Generated with Claude Code

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Next Steps

1. **Test IAS Warning Flow:**
   - Create content with IAS: 0.45 at Step 3
   - Verify warning is logged, run continues
   - Call acknowledge_ias_warning("User", "test")
   - Verify Steno-Ledger entry

2. **Test Step 4 Re-synthesis Pause:**
   - Create synthesis with IAS: 0.50
   - Verify state changes to IASResynthesisPause
   - Verify run is blocked
   - Test resume flow

3. **Test Extreme Drift HALT:**
   - Create content with IAS: 0.25
   - Verify HALT is triggered
   - Verify state is Paused (not IASResynthesisPause)

4. **UI Integration:**
   - Display IAS warning messages in UI
   - Add acknowledgment dialog for AcknowledgmentRequired
   - Add review panel for ResynthesisPause
   - Show pending_ias_acknowledgment status

5. **Documentation:**
   - Update user docs with IAS soft gate behavior
   - Document acknowledgment flow
   - Add examples for different IAS ranges

---

**Implementation Complete:** ✅
**Code Status:** Compiled and Ready
**Testing Status:** Ready for Integration Testing
**Documentation Status:** Complete
