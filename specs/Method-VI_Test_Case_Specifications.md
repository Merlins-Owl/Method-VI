# Method-VI Test Case Specifications

**Version:** 1.0.0  
**Status:** SPECIFICATION - Ready for Test Implementation  
**Parent Document:** module-plan-method-vi.md  
**Purpose:** Validation tests for Coherence Spine, Knowledge Repository, Metrics, Gate Protocol, and all infrastructure components

---

## Overview

### Test Categories

| Category | Component | Priority | Phase |
|----------|-----------|----------|-------|
| **TC-CS** | Coherence Spine | Critical | MVP |
| **TC-KR** | Knowledge Repository | Critical | MVP |
| **TC-LM** | Ledger Manager | Critical | MVP |
| **TC-MT** | Metrics System | Critical | MVP |
| **TC-GP** | Gate Protocol | Critical | MVP |
| **TC-SR** | Signal Router | High | MVP |
| **TC-CM** | Context Manager | High | MVP |
| **TC-AE** | Artifact Envelope | High | MVP |
| **TC-TH** | Threshold Canon | High | MVP |
| **TC-CE** | Cost Estimation | Medium | MVP |
| **TC-NM** | Novice Mode | Medium | MVP |
| **TC-INT** | Integration Tests | Critical | MVP |
| **TC-E2E** | End-to-End Tests | Critical | MVP |

### Test Naming Convention

```
TC-{Category}-{Number}-{Brief Description}

Example: TC-CS-001-GetDependencies
```

### Test Result Schema

```yaml
TestResult:
  test_id: string
  test_name: string
  status: "pass | fail | skip | error"
  execution_time_ms: integer
  expected: any
  actual: any
  error_message: string | null
  timestamp: datetime
```

---

## TC-CS: Coherence Spine Tests

### TC-CS-001: Get Dependencies

**Purpose:** Verify `get_dependencies(artifact_id)` returns correct dependency list

**Preconditions:**
- Database initialized with test artifacts
- Spine edges created between artifacts

**Test Data:**
```sql
-- Setup
INSERT INTO artifacts VALUES ('art-001', 'run-001', 'Intent_Anchor', 1, 'hash1', 1, '/path', '2025-01-01', NULL);
INSERT INTO artifacts VALUES ('art-002', 'run-001', 'Charter', 1, 'hash2', 1, '/path', '2025-01-01', 'hash1');
INSERT INTO artifacts VALUES ('art-003', 'run-001', 'Baseline', 1, 'hash3', 1, '/path', '2025-01-01', 'hash2');
INSERT INTO spine_edges VALUES ('art-002', 'art-001', 'derived_from', '2025-01-01');
INSERT INTO spine_edges VALUES ('art-003', 'art-002', 'derived_from', '2025-01-01');
INSERT INTO spine_edges VALUES ('art-003', 'art-001', 'constrained_by', '2025-01-01');
```

**Test Cases:**

| Test ID | Input | Expected Output | Notes |
|---------|-------|-----------------|-------|
| TC-CS-001-A | `get_dependencies('art-001')` | `[]` | Root has no dependencies |
| TC-CS-001-B | `get_dependencies('art-002')` | `[{id: 'art-001', type: 'derived_from'}]` | Single dependency |
| TC-CS-001-C | `get_dependencies('art-003')` | `[{id: 'art-002', type: 'derived_from'}, {id: 'art-001', type: 'constrained_by'}]` | Multiple dependencies |
| TC-CS-001-D | `get_dependencies('nonexistent')` | `[]` or error | Handle missing artifact |

---

### TC-CS-002: Get Dependents

**Purpose:** Verify `get_dependents(artifact_id)` returns correct list of artifacts depending on target

**Test Cases:**

| Test ID | Input | Expected Output | Notes |
|---------|-------|-----------------|-------|
| TC-CS-002-A | `get_dependents('art-001')` | `[{id: 'art-002'}, {id: 'art-003'}]` | Root has multiple dependents |
| TC-CS-002-B | `get_dependents('art-002')` | `[{id: 'art-003'}]` | Single dependent |
| TC-CS-002-C | `get_dependents('art-003')` | `[]` | Leaf has no dependents |
| TC-CS-002-D | `get_dependents('nonexistent')` | `[]` or error | Handle missing artifact |

---

### TC-CS-003: Is On Critical Path

**Purpose:** Verify `is_on_critical_path(artifact_id)` correctly identifies Critical Path artifacts

**Test Data:**
```
Critical Path: Intent_Anchor → Charter → Baseline → Core_Thesis
Non-Critical: Governance_Summary, Lens_Efficacy_Report, Innovation_Notes
```

**Test Cases:**

| Test ID | Input | Expected | Notes |
|---------|-------|----------|-------|
| TC-CS-003-A | Intent_Anchor artifact | `true` | Root of Critical Path |
| TC-CS-003-B | Charter artifact | `true` | On Critical Path |
| TC-CS-003-C | Baseline artifact | `true` | On Critical Path |
| TC-CS-003-D | Core_Thesis artifact | `true` | On Critical Path |
| TC-CS-003-E | Governance_Summary artifact | `false` | Not on Critical Path |
| TC-CS-003-F | Lens_Efficacy_Report artifact | `false` | Not on Critical Path |
| TC-CS-003-G | Innovation_Notes artifact | `false` | Not on Critical Path |

---

### TC-CS-004: Validate Spine Integrity

**Purpose:** Verify `validate_spine_integrity()` detects breaks and orphans

**Test Scenarios:**

| Test ID | Scenario | Expected Result | Notes |
|---------|----------|-----------------|-------|
| TC-CS-004-A | Valid spine with all edges | `{valid: true, breaks: [], orphans: []}` | Healthy spine |
| TC-CS-004-B | Missing edge between Charter and Intent_Anchor | `{valid: false, breaks: [{from: 'Charter', to: 'Intent_Anchor'}]}` | Broken link |
| TC-CS-004-C | Artifact with no parent_hash and not Intent_Anchor | `{valid: false, orphans: ['artifact_id']}` | Orphan artifact |
| TC-CS-004-D | Circular dependency | `{valid: false, cycles: [...]}` | Cycle detected |
| TC-CS-004-E | Empty spine | `{valid: true, breaks: [], orphans: []}` | Edge case |

---

### TC-CS-005: Get Lineage

**Purpose:** Verify `get_lineage(artifact_id)` traces path to Intent Anchor

**Test Cases:**

| Test ID | Input | Expected Output | Notes |
|---------|-------|-----------------|-------|
| TC-CS-005-A | Intent_Anchor | `['Intent_Anchor']` | Root returns itself |
| TC-CS-005-B | Charter | `['Charter', 'Intent_Anchor']` | Direct child |
| TC-CS-005-C | Framework_Draft (Step 5) | `['Framework_Draft', 'Core_Thesis', 'Diagnostic_Summary', ..., 'Intent_Anchor']` | Full lineage |
| TC-CS-005-D | Orphan artifact | `[]` or error | Handle broken lineage |

---

### TC-CS-006: Immutability Enforcement

**Purpose:** Verify immutable artifacts cannot be modified

**Test Cases:**

| Test ID | Action | Expected Result | Notes |
|---------|--------|-----------------|-------|
| TC-CS-006-A | Update Intent_Anchor content | Rejection/Error | Immutable artifact |
| TC-CS-006-B | Update Charter content | Rejection/Error | Immutable artifact |
| TC-CS-006-C | Update Baseline_Report content | Rejection/Error | Immutable artifact |
| TC-CS-006-D | Update Diagnostic_Summary content | Success | Mutable artifact |
| TC-CS-006-E | Delete Intent_Anchor | Rejection/Error | Cannot delete immutable |
| TC-CS-006-F | Create superseding version of Charter | Success | Supersession allowed |

---

## TC-KR: Knowledge Repository Tests

### TC-KR-001: Run CRUD Operations

**Purpose:** Verify basic run table operations

**Test Cases:**

| Test ID | Operation | Input | Expected | Notes |
|---------|-----------|-------|----------|-------|
| TC-KR-001-A | Create | Valid run data | Success, returns run_id | New run |
| TC-KR-001-B | Create | Duplicate run_id | Error | PK violation |
| TC-KR-001-C | Read | Existing run_id | Run data | |
| TC-KR-001-D | Read | Non-existent run_id | Null/Error | |
| TC-KR-001-E | Update | Set completed_at, final_ci | Success | Run completion |
| TC-KR-001-F | Update | Change status to 'aborted' | Success | Abort run |
| TC-KR-001-G | Delete | Active run | Error | Cannot delete active |
| TC-KR-001-H | Delete | Completed run | Success | Cleanup allowed |

---

### TC-KR-002: Pattern Storage and Retrieval

**Purpose:** Verify pattern table operations and queries

**Test Cases:**

| Test ID | Operation | Expected | Notes |
|---------|-----------|----------|-------|
| TC-KR-002-A | Insert starter pattern | Success with is_starter=1 | |
| TC-KR-002-B | Insert user pattern | Success with is_starter=0, source_run_id set | |
| TC-KR-002-C | Query by intent_category | Returns matching patterns | |
| TC-KR-002-D | Query by vitality threshold | Returns patterns above threshold | |
| TC-KR-002-E | Update application_count | Increments correctly | Pattern applied |
| TC-KR-002-F | Update success_count | Increments correctly | Pattern succeeded |
| TC-KR-002-G | Calculate combined vitality | (freshness × 0.4) + (relevance × 0.6) | Vitality formula |

---

### TC-KR-003: Pattern Vitality Decay

**Purpose:** Verify vitality decay calculations

**Test Data:**
```yaml
Pattern:
  created_at: "2025-01-01"
  application_count: 5
  success_count: 4
  last_applied: "2025-03-01"
  current_date: "2025-06-01"  # 3 months since last use
```

**Test Cases:**

| Test ID | Scenario | Expected Freshness | Expected Relevance | Notes |
|---------|----------|-------------------|-------------------|-------|
| TC-KR-003-A | Never used, 1 month old | 0.9 | 1.0 | Initial decay |
| TC-KR-003-B | Never used, 6 months old | 0.4 | 1.0 | Significant decay |
| TC-KR-003-C | Used 3 months ago | 0.7 | (success/app) | Freshness decayed |
| TC-KR-003-D | Just applied | 1.0 | (success/app) | Freshness refreshed |
| TC-KR-003-E | 5 applications, 4 successes | any | 0.8 | 80% success rate |
| TC-KR-003-F | 10 applications, 2 successes | any | 0.2 | 20% success rate |
| TC-KR-003-G | Combined vitality < 0.3, 12+ months | Archive candidate | | Archive threshold |

---

### TC-KR-004: Artifact Storage

**Purpose:** Verify artifact table operations

**Test Cases:**

| Test ID | Operation | Expected | Notes |
|---------|-----------|----------|-------|
| TC-KR-004-A | Insert with valid frontmatter | Success | |
| TC-KR-004-B | Insert with missing required field | Error | Validation |
| TC-KR-004-C | Insert with duplicate artifact_id | Error | PK violation |
| TC-KR-004-D | Query artifacts by run_id | Returns all artifacts for run | |
| TC-KR-004-E | Query artifacts by step_origin | Returns step's artifacts | |
| TC-KR-004-F | Query artifacts by type | Returns typed artifacts | |
| TC-KR-004-G | Verify hash matches content | Pass if hash valid | Integrity check |

---

### TC-KR-005: Spine Edge Management

**Purpose:** Verify spine_edges table operations

**Test Cases:**

| Test ID | Operation | Expected | Notes |
|---------|-----------|----------|-------|
| TC-KR-005-A | Insert valid edge | Success | |
| TC-KR-005-B | Insert duplicate edge | Error | PK violation |
| TC-KR-005-C | Insert edge with invalid source | Error | FK violation |
| TC-KR-005-D | Insert edge with invalid target | Error | FK violation |
| TC-KR-005-E | Query edges by source | Returns outgoing edges | |
| TC-KR-005-F | Query edges by target | Returns incoming edges | |
| TC-KR-005-G | Delete edge | Success | |

---

## TC-LM: Ledger Manager Tests

### TC-LM-001: State Transition Validation

**Purpose:** Verify ledger enforces legal state transitions

**Test Cases:**

| Test ID | Current State | Action | Expected | Notes |
|---------|---------------|--------|----------|-------|
| TC-LM-001-A | Step 0 active | Intent capture | Allowed | Legal action |
| TC-LM-001-B | Step 0 active | Pattern query | Allowed | Legal action |
| TC-LM-001-C | Step 0 active | Baseline freeze | Rejected | Illegal - too early |
| TC-LM-001-D | Step 0 active | Validation | Rejected | Illegal - too early |
| TC-LM-001-E | Baseline frozen | Analysis | Allowed | Legal action |
| TC-LM-001-F | Baseline frozen | Scope change | Rejected | Illegal - baseline locked |
| TC-LM-001-G | Baseline frozen | Baseline edit | Rejected | Illegal - immutable |
| TC-LM-001-H | Gate pending | Human approve | Allowed | Legal action |
| TC-LM-001-I | Gate pending | Agent progression | Rejected | Illegal - awaiting human |
| TC-LM-001-J | HALT active | Human decision | Allowed | Only human can act |
| TC-LM-001-K | HALT active | Any automated action | Rejected | Illegal - halted |

---

### TC-LM-002: Ledger Entry Creation

**Purpose:** Verify ledger entries are created correctly

**Test Cases:**

| Test ID | Entry Type | Expected Fields | Notes |
|---------|------------|-----------------|-------|
| TC-LM-002-A | gate | run_id, step, role, payload, hash | Gate entry |
| TC-LM-002-B | intervention | domain, trigger, action, outcome | Intervention entry |
| TC-LM-002-C | signal | type, step_from, step_to, artifacts | Signal entry |
| TC-LM-002-D | decision | action, rationale | Decision entry |
| TC-LM-002-E | metric_snapshot | all Critical 6 values | Metrics entry |

---

### TC-LM-003: Hash Chain Integrity

**Purpose:** Verify ledger hash chain is maintained

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-LM-003-A | First entry | prior_hash = null, hash calculated | Chain start |
| TC-LM-003-B | Second entry | prior_hash = first entry's hash | Chain link |
| TC-LM-003-C | Nth entry | prior_hash = (N-1)th entry's hash | Chain continues |
| TC-LM-003-D | Verify chain integrity | All hashes valid, no breaks | Full verification |
| TC-LM-003-E | Tampered entry | Chain validation fails | Detect tampering |

---

### TC-LM-004: HALT/PAUSE Triggers

**Purpose:** Verify correct triggering of HALT and PAUSE

**Test Cases:**

| Test ID | Trigger Condition | Expected Action | Notes |
|---------|-------------------|-----------------|-------|
| TC-LM-004-A | CI = 0.49 | HALT_IMMEDIATE | Below 0.50 threshold |
| TC-LM-004-B | CI = 0.50 | Continue (barely passing) | At threshold |
| TC-LM-004-C | EV = +31% | HALT_IMMEDIATE | Exceeds ±30% |
| TC-LM-004-D | EV = -30% | Continue (at limit) | At threshold |
| TC-LM-004-E | SEC violation | HALT_IMMEDIATE | Undocumented expansion |
| TC-LM-004-F | Spine break detected | HALT_IMMEDIATE | Critical path broken |
| TC-LM-004-G | CI = 0.72 | PAUSE_FOR_REVIEW | Warning zone (0.70-0.80) |
| TC-LM-004-H | CI = 0.69 | PAUSE_FOR_REVIEW | Below warning |
| TC-LM-004-I | Any threshold breach | Log intervention | Minimal intervention |

---

## TC-MT: Metrics System Tests

### TC-MT-001: CI (Coherence Index) Calculation

**Purpose:** Verify Coherence Index calculation

**Test Cases:**

| Test ID | Inputs | Expected CI | Notes |
|---------|--------|-------------|-------|
| TC-MT-001-A | All coherence dimensions = 1.0 | 1.0 | Perfect score |
| TC-MT-001-B | All coherence dimensions = 0.0 | 0.0 | Zero score |
| TC-MT-001-C | Mixed dimensions | Weighted average | Standard case |
| TC-MT-001-D | structural=0.9, thematic=0.8, logic=0.85 | ~0.85 | Typical case |
| TC-MT-001-E | One dimension very low | Pulls down average | Weak link effect |

---

### TC-MT-002: EV (Expansion Variance) Calculation

**Purpose:** Verify Expansion Variance calculation

**Test Data:**
```
E_baseline = 1000 (tokens/units)
```

**Test Cases:**

| Test ID | E_current | Expected EV | Status | Notes |
|---------|-----------|-------------|--------|-------|
| TC-MT-002-A | 1000 | 0% | Pass | No change |
| TC-MT-002-B | 1050 | +5% | Pass | Minor expansion |
| TC-MT-002-C | 950 | -5% | Pass | Minor contraction |
| TC-MT-002-D | 1100 | +10% | Pass (at limit) | At threshold |
| TC-MT-002-E | 1150 | +15% | Warning | Exceeds pass |
| TC-MT-002-F | 1300 | +30% | Warning (at HALT) | At HALT threshold |
| TC-MT-002-G | 1350 | +35% | HALT | Exceeds HALT |
| TC-MT-002-H | 700 | -30% | Warning (at HALT) | Contraction at HALT |

**Formula Verification:**
```
EV = |E_current - E_baseline| / E_baseline × 100
   = |1100 - 1000| / 1000 × 100
   = 10%
```

---

### TC-MT-003: IAS (Intent Alignment Score) Calculation

**Purpose:** Verify Intent Alignment Score calculation

**Test Cases:**

| Test ID | Charter Objectives Met | Expected IAS | Notes |
|---------|------------------------|--------------|-------|
| TC-MT-003-A | All objectives fully met | 1.0 | Perfect alignment |
| TC-MT-003-B | No objectives met | 0.0 | Zero alignment |
| TC-MT-003-C | 4/5 objectives fully met | 0.8 | 80% alignment |
| TC-MT-003-D | All objectives partially met (50%) | 0.5 | Partial alignment |
| TC-MT-003-E | Mix of full, partial, none | Weighted average | Complex case |

---

### TC-MT-004: EFI (Execution Fidelity Index) Calculation

**Purpose:** Verify Execution Fidelity Index calculation

**Test Cases:**

| Test ID | Claims Substantiated | Expected EFI | Notes |
|---------|---------------------|--------------|-------|
| TC-MT-004-A | All claims with strong evidence | 100% | Perfect fidelity |
| TC-MT-004-B | No claims substantiated | 0% | No evidence |
| TC-MT-004-C | 19/20 claims substantiated | 95% | At pass threshold |
| TC-MT-004-D | 18/20 claims substantiated | 90% | At warning threshold |
| TC-MT-004-E | 16/20 claims substantiated | 80% | At HALT threshold |

---

### TC-MT-005: SEC (Scope Expansion Count) Calculation

**Purpose:** Verify Scope Expansion Count tracking

**Test Cases:**

| Test ID | Scenario | Expected SEC | Status | Notes |
|---------|----------|--------------|--------|-------|
| TC-MT-005-A | No scope expansions | 0 (100% compliant) | Pass | Baseline scope maintained |
| TC-MT-005-B | 1 approved expansion | 1 (100% documented) | Pass | Documented expansion |
| TC-MT-005-C | 1 undocumented expansion | 1 (<100% documented) | HALT | SEC violation |
| TC-MT-005-D | 3 approved expansions | 3 (100% documented) | Pass | Multiple documented |

---

### TC-MT-006: PCI (Process Coherence Index) Calculation

**Purpose:** Verify Process Coherence Index calculation

**Test Cases:**

| Test ID | Process Compliance Factors | Expected PCI | Notes |
|---------|---------------------------|--------------|-------|
| TC-MT-006-A | All steps followed, all gates passed | 1.0 | Perfect compliance |
| TC-MT-006-B | Skipped step detected | < 0.9 | Process deviation |
| TC-MT-006-C | Gate bypassed | < 0.85 | Serious deviation |
| TC-MT-006-D | Architecture Map deviation | < 0.9 | Cadence issue |

---

### TC-MT-007: Metric Explainability Output

**Purpose:** Verify metrics include required explainability fields

**Test Cases:**

| Test ID | Metric | Required Fields | Notes |
|---------|--------|-----------------|-------|
| TC-MT-007-A | Any metric | metric_name present | Identity |
| TC-MT-007-B | Any metric | value is numeric | Numeric value |
| TC-MT-007-C | Any metric | threshold from Canon | Reference threshold |
| TC-MT-007-D | Any metric | status = pass/warning/fail | Status classification |
| TC-MT-007-E | Any metric | inputs_used array | Input traceability |
| TC-MT-007-F | Any metric | calculation_method string | Method transparency |
| TC-MT-007-G | Any metric | interpretation string | Plain language |
| TC-MT-007-H | Warning/Fail metric | recommendation string | Actionable guidance |
| TC-MT-007-I | Pass metric | recommendation = null | No action needed |

---

## TC-GP: Gate Protocol Tests

### TC-GP-001: Gate Signal Recognition

**Purpose:** Verify gate signals are correctly identified

**Test Cases:**

| Test ID | Signal | Is Gate? | Notes |
|---------|--------|----------|-------|
| TC-GP-001-A | Ready_for_Step_1 | Yes | Step 0→1 gate |
| TC-GP-001-B | Baseline_Frozen | Yes | Step 1→2 gate |
| TC-GP-001-C | Ready_for_Analysis | Yes | Step 2→3 gate |
| TC-GP-001-D | Ready_for_Synthesis | Yes | Step 3→4 gate |
| TC-GP-001-E | Ready_for_Redesign | Yes | Step 4→5 gate |
| TC-GP-001-F | Ready_for_Validation | Yes | Step 5→6 gate |
| TC-GP-001-G | Validation_Complete | Yes | Step 6 completion |
| TC-GP-001-H | Learning_Harvested | No | Step 6.5 (no gate) |
| TC-GP-001-I | New_Run_Ready | No | Closure (no gate) |
| TC-GP-001-J | Metric_Update | No | Internal signal |

---

### TC-GP-002: Gate Blocking Behavior

**Purpose:** Verify gates block progression until human approval

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-GP-002-A | Gate emitted, no approval | Progression blocked | Awaiting human |
| TC-GP-002-B | Gate emitted, human approves | Progression allowed | Gate passed |
| TC-GP-002-C | Gate emitted, human rejects | Stay at current step | Gate rejected |
| TC-GP-002-D | Gate emitted, timeout | Remains blocked | No auto-approve |
| TC-GP-002-E | Gate emitted, adjust scope selected | Micro-burst triggered | Scope adjustment |

---

### TC-GP-003: Gate Density by Skill Level

**Purpose:** Verify gate density matches user skill level

**Test Cases:**

| Test ID | Skill Level | Expected Gates | Notes |
|---------|-------------|----------------|-------|
| TC-GP-003-A | Novice | All 7 gates | Maximum gates |
| TC-GP-003-B | Intermediate | Major gates (0→1, 1→2, 5→6) | Reduced gates |
| TC-GP-003-C | Expert | Critical gates only (1→2, 5→6) | Minimum gates |

---

### TC-GP-004: Gate Approval Recording

**Purpose:** Verify gate approvals are recorded in ledger

**Test Cases:**

| Test ID | Action | Expected Ledger Entry | Notes |
|---------|--------|----------------------|-------|
| TC-GP-004-A | Approve gate | entry_type='gate', payload.decision='approved' | Approval logged |
| TC-GP-004-B | Reject gate | entry_type='gate', payload.decision='rejected' | Rejection logged |
| TC-GP-004-C | Approve with comments | payload.comments populated | Comments captured |
| TC-GP-004-D | Gate timeout warning | entry_type='gate', payload.timeout_warning=true | Timeout logged |

---

## TC-SR: Signal Router Tests

### TC-SR-001: Signal Emission

**Purpose:** Verify signals are emitted with correct payload

**Test Cases:**

| Test ID | Signal Type | Required Payload Fields | Notes |
|---------|-------------|------------------------|-------|
| TC-SR-001-A | Ready_for_Step_1 | type, run_id, timestamp, step_from=0, step_to=1 | Step transition |
| TC-SR-001-B | Baseline_Frozen | artifacts_produced, metrics_snapshot | Baseline lock |
| TC-SR-001-C | Any gate signal | gate_required=true | Gate flag |
| TC-SR-001-D | Non-gate signal | gate_required=false | No gate |

---

### TC-SR-002: Signal Hash Chain

**Purpose:** Verify signal hash chain integrity

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-SR-002-A | First signal | prior_signal_hash=null | Chain start |
| TC-SR-002-B | Subsequent signal | prior_signal_hash = previous signal's hash | Chain link |
| TC-SR-002-C | Chain verification | All links valid | Full verification |

---

## TC-CM: Context Manager Tests

### TC-CM-001: Steno-Ledger Generation

**Purpose:** Verify Steno-Ledger format is correct

**Test Cases:**

| Test ID | Input State | Expected Steno-Ledger | Notes |
|---------|-------------|----------------------|-------|
| TC-CM-001-A | Step 0, Observer, CI=null | `[RUN:test-run \| S:0 \| R:OBS \| CI:- \| EV:- \| M:STD \| 🚦:Initializing]` | Initial state |
| TC-CM-001-B | Step 3, Observer, CI=0.87, EV=+3% | `[RUN:test-run \| S:3 \| R:OBS \| CI:0.87 \| EV:+3% \| M:STD \| 🚦:Ready_for_Synthesis]` | Mid-run |
| TC-CM-001-C | Step 2, Conductor | `[RUN:test-run \| S:2 \| R:COND \| ...]` | Conductor role |
| TC-CM-001-D | Step 5, Auditor | `[RUN:test-run \| S:5 \| R:AUD \| ...]` | Auditor role |
| TC-CM-001-E | Component mode | `[... \| M:COMP \| ...]` | Component mode |
| TC-CM-001-F | Surgical mode | `[... \| M:SURG \| ...]` | Surgical mode |

---

### TC-CM-002: Steno-Ledger Injection

**Purpose:** Verify Steno-Ledger is injected into agent prompts

**Test Cases:**

| Test ID | Agent | Expected | Notes |
|---------|-------|----------|-------|
| TC-CM-002-A | Any agent call | Steno-Ledger prepended to system prompt | Injection present |
| TC-CM-002-B | Orchestrator | Contains current step, role | Orchestrator context |
| TC-CM-002-C | Specialist agent | Contains calling context | Specialist context |

---

## TC-AE: Artifact Envelope Tests

### TC-AE-001: Frontmatter Validation

**Purpose:** Verify artifact frontmatter is validated

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-AE-001-A | All required fields present | Valid | Complete frontmatter |
| TC-AE-001-B | Missing artifact_id | Invalid | Required field |
| TC-AE-001-C | Missing artifact_type | Invalid | Required field |
| TC-AE-001-D | Missing run_id | Invalid | Required field |
| TC-AE-001-E | Missing hash | Invalid | Required field |
| TC-AE-001-F | Invalid artifact_type | Invalid | Not in enum |
| TC-AE-001-G | Hash mismatch | Invalid | Integrity failure |

---

### TC-AE-002: Dependency Validation

**Purpose:** Verify artifact dependencies are validated

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-AE-002-A | All dependencies exist | Valid | Dependencies found |
| TC-AE-002-B | Missing dependency | Invalid | Broken reference |
| TC-AE-002-C | Circular dependency | Invalid | Cycle detected |
| TC-AE-002-D | Intent_Anchor with no parent | Valid | Root artifact |
| TC-AE-002-E | Non-root with no parent | Invalid | Orphan artifact |

---

### TC-AE-003: Handoff Protocol

**Purpose:** Verify artifact handoff between agents

**Test Cases:**

| Test ID | Step | Expected Validation | Notes |
|---------|------|---------------------|-------|
| TC-AE-003-A | Agent receives artifact | Envelope completeness checked | Step 1 |
| TC-AE-003-B | Agent receives artifact | Hash integrity verified | Step 2 |
| TC-AE-003-C | Agent receives artifact | Dependencies exist in Spine | Step 3 |
| TC-AE-003-D | Handoff logged | Ledger entry created | Step 4 |

---

## TC-TH: Threshold Canon Tests

### TC-TH-001: Threshold Loading

**Purpose:** Verify thresholds are loaded from config

**Test Cases:**

| Test ID | Scenario | Expected | Notes |
|---------|----------|----------|-------|
| TC-TH-001-A | Valid config file | All thresholds loaded | Normal load |
| TC-TH-001-B | Missing config file | Default thresholds used | Fallback |
| TC-TH-001-C | Corrupted config file | Error + default fallback | Error handling |
| TC-TH-001-D | Partial config | Merge with defaults | Partial override |

---

### TC-TH-002: Threshold Application

**Purpose:** Verify thresholds are correctly applied to metrics

**Test Cases:**

| Test ID | Metric | Value | Expected Status | Notes |
|---------|--------|-------|-----------------|-------|
| TC-TH-002-A | CI | 0.85 | Pass | Above 0.80 |
| TC-TH-002-B | CI | 0.80 | Pass | At threshold |
| TC-TH-002-C | CI | 0.75 | Warning | Between 0.70-0.80 |
| TC-TH-002-D | CI | 0.70 | Warning | At warning |
| TC-TH-002-E | CI | 0.55 | Warning | Between 0.50-0.70 |
| TC-TH-002-F | CI | 0.50 | Warning | At HALT |
| TC-TH-002-G | CI | 0.45 | HALT | Below 0.50 |

---

## TC-CE: Cost Estimation Tests

### TC-CE-001: Token Estimation

**Purpose:** Verify token estimation by telemetry profile

**Test Cases:**

| Test ID | Profile | Expected Min | Expected Max | Notes |
|---------|---------|--------------|--------------|-------|
| TC-CE-001-A | Lite | 10,000 | 25,000 | Minimal telemetry |
| TC-CE-001-B | Standard | 25,000 | 75,000 | Normal telemetry |
| TC-CE-001-C | Full | 75,000 | 200,000 | Full telemetry |
| TC-CE-001-D | Learning | 100,000 | 300,000 | Learning mode |

---

### TC-CE-002: Cost Calculation

**Purpose:** Verify cost calculation from token estimates

**Test Data:**
```yaml
model: claude-sonnet
cost_per_1k_input_tokens: 0.003
cost_per_1k_output_tokens: 0.015
```

**Test Cases:**

| Test ID | Input Tokens | Output Tokens | Expected Cost | Notes |
|---------|--------------|---------------|---------------|-------|
| TC-CE-002-A | 10,000 | 5,000 | $0.105 | (10×0.003 + 5×0.015) |
| TC-CE-002-B | 50,000 | 25,000 | $0.525 | Standard run |
| TC-CE-002-C | 100,000 | 50,000 | $1.05 | Full run |

---

### TC-CE-003: Budget Alert

**Purpose:** Verify budget alert triggers correctly

**Test Cases:**

| Test ID | Estimated Cost | Budget Threshold | Expected | Notes |
|---------|----------------|------------------|----------|-------|
| TC-CE-003-A | $2.00 | $5.00 | No alert | Under budget |
| TC-CE-003-B | $5.00 | $5.00 | Alert | At budget |
| TC-CE-003-C | $7.00 | $5.00 | Alert | Over budget |

---

## TC-NM: Novice Mode Tests

### TC-NM-001: Skill Level Features

**Purpose:** Verify features match skill level settings

**Test Cases:**

| Test ID | Skill Level | Feature | Expected State | Notes |
|---------|-------------|---------|----------------|-------|
| TC-NM-001-A | Novice | Tooltips | Always visible | Maximum help |
| TC-NM-001-B | Intermediate | Tooltips | On hover | Reduced help |
| TC-NM-001-C | Expert | Tooltips | Disabled | Minimal help |
| TC-NM-001-D | Novice | Explanations | Verbose | Detailed |
| TC-NM-001-E | Expert | Explanations | Minimal | Concise |
| TC-NM-001-F | Novice | Warnings | Proactive | Early warnings |
| TC-NM-001-G | Expert | Warnings | Silent | No interruption |

---

### TC-NM-002: Progressive Disclosure

**Purpose:** Verify progressive disclosure behavior

**Test Cases:**

| Test ID | Action | Expected | Notes |
|---------|--------|----------|-------|
| TC-NM-002-A | Initial view | Essential controls only | Simplified |
| TC-NM-002-B | Click "Show more" | Advanced features revealed | Expanded |
| TC-NM-002-C | Expert mode | All features visible | Full access |

---

## TC-INT: Integration Tests

### TC-INT-001: Step 0→1 Integration

**Purpose:** Verify complete Step 0 to Step 1 flow

**Test Steps:**
1. User submits intent request
2. Scope & Pattern Agent creates Intent_Summary
3. Pattern_Suggestions artifact created (if patterns exist)
4. User answers clarification questions
5. Ready_for_Step_1 signal emitted
6. Gate presented to user
7. User approves gate
8. Intent_Anchor created (immutable)
9. Charter created (immutable)
10. Baseline_Report created (immutable)
11. Architecture_Map created (immutable)
12. Baseline_Frozen signal emitted

**Verification Points:**

| Checkpoint | Verification | Notes |
|------------|--------------|-------|
| After step 2 | Intent_Summary artifact exists | Artifact created |
| After step 5 | Gate signal has gate_required=true | Gate flag |
| After step 6 | System blocked awaiting approval | Gate blocking |
| After step 8 | Intent_Anchor.is_immutable=true | Immutability |
| After step 11 | All Step 1 artifacts in Coherence Spine | Spine populated |
| After step 12 | Ledger contains gate approval | Audit trail |

---

### TC-INT-002: Metric Calculation Integration

**Purpose:** Verify metrics flow from calculation to display

**Test Steps:**
1. Step completes with content
2. Governance & Telemetry Agent calculates metrics
3. MetricResult objects created with explainability
4. Thresholds applied from Canon
5. Status determined (pass/warning/fail)
6. Interventions triggered if needed
7. Metrics displayed in UI
8. Ledger entry created

**Verification Points:**

| Checkpoint | Verification | Notes |
|------------|--------------|-------|
| After step 2 | All Critical 6 calculated | Complete metrics |
| After step 3 | Explainability fields populated | Transparency |
| After step 4 | Thresholds match Canon | Consistency |
| After step 6 | HALT triggered if threshold breached | Enforcement |
| After step 7 | UI shows all metric details | Display |
| After step 8 | metric_snapshot in ledger | Audit trail |

---

### TC-INT-003: Pattern Recommendation Integration

**Purpose:** Verify pattern recommendation flow

**Test Steps:**
1. Intent_Summary created with intent_category
2. Knowledge Repository queried for matching patterns
3. Patterns ranked by fit score and vitality
4. Top patterns presented in Pattern_Suggestions
5. User selects pattern
6. Pattern applied to Architecture_Map
7. Pattern application_count incremented

**Verification Points:**

| Checkpoint | Verification | Notes |
|------------|--------------|-------|
| After step 2 | Query uses correct category | Filter works |
| After step 3 | Vitality calculation correct | Ranking correct |
| After step 4 | Top 3-5 patterns shown | Limit applied |
| After step 6 | Architecture reflects pattern | Applied |
| After step 7 | application_count += 1 | Tracking |

---

## TC-E2E: End-to-End Tests

### TC-E2E-001: Complete Standard Run (Happy Path)

**Purpose:** Verify complete run from Step 0 through Closure

**Test Scenario:** Simple analytical task with clear intent

**Expected Flow:**
```
Step 0: Intent capture → Pattern suggestions → Ready_for_Step_1 🚧
Step 1: Charter → Baseline → Architecture → Baseline_Frozen 🚧
Step 2: Governance monitoring → Ready_for_Analysis 🚧
Step 3: Six-lens analysis → Diagnostic → Ready_for_Synthesis 🚧
Step 4: Core thesis → Causal spine → Ready_for_Redesign 🚧
Step 5: Framework draft → Ready_for_Validation 🚧
Step 6: Validation → CI ≥ 0.85 → Validation_Complete 🚧
Step 6.5: Pattern extraction → Learning_Harvested
Closure: Audit trail → New_Run_Ready
```

**Success Criteria:**
- All gates passed
- Final CI ≥ 0.85
- EV ≤ ±10%
- All artifacts in Coherence Spine
- Pattern extracted to repository
- Audit trail complete

---

### TC-E2E-002: Run with Warning Recovery

**Purpose:** Verify run recovers from warning state

**Test Scenario:** CI drops to 0.75 at Step 3, recovers by Step 6

**Expected Behavior:**
1. Step 3 completes with CI = 0.75
2. PAUSE_FOR_REVIEW triggered
3. User reviews and continues
4. Steps 4-5 improve coherence
5. Step 6 completes with CI = 0.82
6. Run completes successfully (no Step 6.5)

**Success Criteria:**
- Warning logged in ledger
- User decision recorded
- Recovery achieved
- Final CI in pass range (0.80-0.85)

---

### TC-E2E-003: Run with HALT Recovery

**Purpose:** Verify run recovers from HALT state

**Test Scenario:** EV exceeds 30% at Step 4

**Expected Behavior:**
1. Step 4 produces content with EV = 32%
2. HALT_IMMEDIATE triggered
3. All automated actions blocked
4. User reviews situation
5. User decides to adjust scope
6. Micro-burst creates new Intent_Anchor version
7. Run resumes from appropriate step

**Success Criteria:**
- HALT logged with reason
- System blocked until human action
- Recovery path executed
- Run completes or intentionally aborted

---

### TC-E2E-004: Run Abort and Resume

**Purpose:** Verify session save/resume functionality

**Test Scenario:** User pauses at Step 3, resumes later

**Expected Behavior:**
1. Steps 0-2 complete normally
2. Step 3 begins analysis
3. User clicks "Pause Session"
4. Session state saved
5. Application closed
6. Application reopened
7. User clicks "Resume Session"
8. Step 3 resumes from checkpoint
9. Run completes normally

**Success Criteria:**
- Session file created with full state
- All artifacts preserved
- Metrics history intact
- Resume continues from correct point
- No data loss

---

### TC-E2E-005: First-Time User Experience

**Purpose:** Verify novice onboarding flow

**Test Scenario:** New user first launch

**Expected Behavior:**
1. Application detects first launch
2. Initialize-method-vi workflow triggers
3. Welcome screen displayed
4. User enters name
5. User selects skill level (Novice)
6. User sets storage path
7. User configures first API key
8. API connection tested
9. Tutorial offered
10. First run with maximum guidance

**Success Criteria:**
- All config saved correctly
- API key encrypted
- Novice mode features active
- Tutorial accessible
- Help tooltips visible

---

## Test Execution Plan

### Phase 1: Unit Tests (MVP)

| Priority | Category | Test Count | Automation |
|----------|----------|------------|------------|
| 1 | TC-CS (Coherence Spine) | 24 | Automated |
| 2 | TC-KR (Knowledge Repository) | 28 | Automated |
| 3 | TC-LM (Ledger Manager) | 20 | Automated |
| 4 | TC-MT (Metrics System) | 35 | Automated |
| 5 | TC-GP (Gate Protocol) | 16 | Automated |
| 6 | TC-SR (Signal Router) | 8 | Automated |
| 7 | TC-CM (Context Manager) | 8 | Automated |
| 8 | TC-AE (Artifact Envelope) | 14 | Automated |
| 9 | TC-TH (Threshold Canon) | 12 | Automated |

### Phase 2: Integration Tests (MVP)

| Priority | Category | Test Count | Automation |
|----------|----------|------------|------------|
| 1 | TC-INT (Integration) | 12 | Automated |
| 2 | TC-CE (Cost Estimation) | 9 | Automated |
| 3 | TC-NM (Novice Mode) | 10 | Semi-automated |

### Phase 3: End-to-End Tests (MVP)

| Priority | Category | Test Count | Automation |
|----------|----------|------------|------------|
| 1 | TC-E2E (End-to-End) | 5 | Manual + Automated |

### Total Test Coverage

| Category | Tests | Priority |
|----------|-------|----------|
| Unit Tests | 165 | Critical |
| Integration Tests | 31 | High |
| End-to-End Tests | 5 | Critical |
| **Total** | **201** | |

---

## Test Data Requirements

### Seed Data for Testing

```sql
-- Test Runs
INSERT INTO runs VALUES ('test-run-001', 'hash1', '2025-01-01', NULL, NULL, NULL, 'active');
INSERT INTO runs VALUES ('test-run-002', 'hash2', '2025-01-01', '2025-01-02', 0.87, 5.0, 'completed');

-- Test Artifacts (Critical Path)
INSERT INTO artifacts VALUES ('test-anchor-001', 'test-run-001', 'Intent_Anchor', 1, 'anchor-hash', 1, '/path', '2025-01-01', NULL);
INSERT INTO artifacts VALUES ('test-charter-001', 'test-run-001', 'Charter', 1, 'charter-hash', 1, '/path', '2025-01-01', 'anchor-hash');
INSERT INTO artifacts VALUES ('test-baseline-001', 'test-run-001', 'Baseline_Report', 1, 'baseline-hash', 1, '/path', '2025-01-01', 'charter-hash');

-- Test Patterns (Starters)
INSERT INTO patterns VALUES ('starter-analytical-001', 'Analytical', 0.87, 5.0, '{}', '{}', '{}', '{}', '{}', '{}', 1.0, 1.0, 0, 0, '2025-01-01', NULL, NULL, 1);

-- Test Spine Edges
INSERT INTO spine_edges VALUES ('test-charter-001', 'test-anchor-001', 'derived_from', '2025-01-01');
INSERT INTO spine_edges VALUES ('test-baseline-001', 'test-charter-001', 'derived_from', '2025-01-01');
```

### Mock API Responses

```yaml
# Mock LLM response for metric calculation
mock_metric_response:
  CI: 0.85
  EV: 5.0
  IAS: 0.88
  EFI: 96
  SEC: 0
  PCI: 0.92

# Mock LLM response for intent extraction
mock_intent_response:
  primary_goal: "Test goal"
  audience: "Test audience"
  expected_outcome: "Test outcome"
  confidence: 85
```

---

## External Documentation Status

| Item | Status |
|------|--------|
| Test categories defined | ✅ Complete |
| Unit test cases | ✅ Complete (165 tests) |
| Integration test cases | ✅ Complete (31 tests) |
| End-to-end test cases | ✅ Complete (5 tests) |
| Test data requirements | ✅ Complete |
| Execution plan | ✅ Complete |

**Next Steps:**
1. Implement test framework (pytest recommended)
2. Create test fixtures from seed data
3. Implement unit tests in priority order
4. Set up CI/CD pipeline for automated testing
5. Create test reporting dashboard

---

**Document Created:** 2025-12-17  
**Aligned With:** module-plan-method-vi.md (Architecture Hardened)  
**Status:** Ready for Test Implementation
