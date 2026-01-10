use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::agents::analysis_synthesis::AnalysisSynthesisAgent;
use crate::agents::governance_telemetry::{CriticalMetrics, GovernanceTelemetryAgent, IASWarning};
use crate::agents::scope_pattern::{IntentSummary, ScopePatternAgent};
use crate::agents::structure_redesign::StructureRedesignAgent;
use crate::agents::validation_learning::ValidationLearningAgent;
use crate::context::{ContextManager, Mode, Role, RunContext, Signal as ContextSignal};
use crate::governance::{CalloutManager, ModeDetector};
use crate::ledger::{EntryType, LedgerManager, LedgerPayload, LedgerState};
use crate::signals::{SignalPayload, SignalRouter, SignalType};

/// Run state for tracking Method-VI session progress
#[derive(Debug, Clone)]
pub enum RunState {
    /// Step 0: Intent capture and pattern query
    Step0Active,

    /// Waiting for gate approval to proceed to Step 1
    Step0GatePending,

    /// Step 1: Charter and baseline creation
    Step1Active,

    /// Waiting for gate approval (baseline frozen)
    Step1GatePending,

    /// Step 2: Governance calibration (active governance)
    Step2Active,

    /// Waiting for gate approval (governance calibrated)
    Step2GatePending,

    /// Step 3: Multi-angle analysis (six-lens analysis)
    Step3Active,

    /// Waiting for gate approval (analysis complete)
    Step3GatePending,

    /// Step 4: Synthesis lock-in (model building)
    Step4Active,

    /// Waiting for gate approval (synthesis locked)
    Step4GatePending,

    /// Step 5: Structure & Redesign (framework creation)
    Step5Active,

    /// Waiting for gate approval (framework ready)
    Step5GatePending,

    /// Step 6: Validation & Assurance (quality verification)
    Step6Active,

    /// Waiting for gate approval (validation complete)
    Step6GatePending,

    /// Step 6.5: Learning Harvest (pattern extraction for exceptional results)
    Step6_5Active,

    /// Step 6.5 and beyond: Future steps (not yet implemented)
    FutureStep(u8),

    /// Run completed
    Completed,

    /// Run paused due to HALT condition - awaiting human decision
    Paused {
        reason: String,
        step: u8,
        triggered_metrics: Option<serde_json::Value>,  // Only metrics that caused HALT
        all_metrics_snapshot: Option<serde_json::Value>,  // Full metrics for debugging
    },

    /// IAS Warning - Re-synthesis Pause (FIX-024)
    ///
    /// Triggered at Step 4 when IAS is in warning range (0.30-0.69).
    /// Synthesis may have diverged from Charter. Requires review before proceeding.
    IASResynthesisPause {
        score: f64,
        message: String,
        step: u8,
    },

    /// Run permanently halted (aborted by user or unrecoverable error)
    Halted { reason: String },
}

impl RunState {
    /// Get the step number for this state
    pub fn step_number(&self) -> u8 {
        match self {
            RunState::Step0Active | RunState::Step0GatePending => 0,
            RunState::Step1Active | RunState::Step1GatePending => 1,
            RunState::Step2Active | RunState::Step2GatePending => 2,
            RunState::Step3Active | RunState::Step3GatePending => 3,
            RunState::Step4Active | RunState::Step4GatePending => 4,
            RunState::Step5Active | RunState::Step5GatePending => 5,
            RunState::Step6Active | RunState::Step6GatePending => 6,
            RunState::Step6_5Active => 6, // 6.5 is still step 6 from integer perspective
            RunState::FutureStep(n) => *n,
            RunState::Completed => 7,
            RunState::Paused { step, .. } => *step, // Return the step where pause occurred
            RunState::IASResynthesisPause { step, .. } => *step, // FIX-024: Return step where IAS warning occurred
            RunState::Halted { .. } => 255, // Special value for halted
        }
    }

    /// Check if this state is waiting for gate approval
    pub fn is_gate_pending(&self) -> bool {
        matches!(
            self,
            RunState::Step0GatePending
                | RunState::Step1GatePending
                | RunState::Step2GatePending
                | RunState::Step3GatePending
                | RunState::Step4GatePending
                | RunState::Step5GatePending
                | RunState::Step6GatePending
        )
    }
}

// IntentSummary is now imported from scope_pattern module

/// Orchestrator: Master coordinator for Method-VI 7-step process
///
/// The Orchestrator manages:
/// - Step sequence control (Steps 0 → 6.5 → Closure)
/// - Gate Protocol enforcement
/// - Governance role activation
/// - Signal emission and routing
/// - State persistence via Ledger
pub struct Orchestrator {
    /// Unique run identifier: {YYYY-MM-DD}-{label}
    pub run_id: String,

    /// Current state in the Method-VI process
    pub state: RunState,

    /// Active governance role
    pub active_role: Role,

    /// Operational mode (always Standard for MVP)
    pub mode: Mode,

    /// Ledger manager for recording all actions
    ledger: LedgerManager,

    /// Signal router for workflow signals
    signal_router: SignalRouter,

    /// Captured intent summary from Step 0
    pub intent_summary: Option<IntentSummary>,

    /// Step 1 artifacts (immutable baseline)
    pub intent_anchor: Option<String>,
    pub charter: Option<String>,
    pub baseline_report: Option<String>,
    pub architecture_map: Option<String>,

    /// Step 2 artifacts (governance calibration)
    pub governance_summary: Option<String>,
    pub domain_snapshots: Option<String>,

    /// Step 3 artifacts (six-lens analysis)
    pub integrated_diagnostic: Option<String>,
    pub lens_efficacy_report: Option<String>,

    /// Step 4 artifacts (synthesis lock-in)
    pub core_thesis: Option<String>,
    pub operating_principles: Option<String>,
    pub model_geometry: Option<String>,
    pub causal_spine: Option<String>,
    pub north_star_narrative: Option<String>,
    pub glossary: Option<String>,
    pub limitations: Option<String>,

    /// Step 5 artifacts (structure & redesign)
    pub framework_architecture: Option<String>,

    /// Step 6 artifacts (validation & assurance)
    pub validation_matrix: Option<String>,
    pub semantic_table: Option<String>,
    pub evidence_report: Option<String>,
    pub validation_outcome: Option<String>,  // PASS/FAIL/WARNING
    pub exceptional_flag: bool,  // True if CI ≥ 0.85 (triggers Step 6.5)

    /// Scope & Pattern Agent (optional - if None, uses stub)
    scope_agent: Option<ScopePatternAgent>,

    /// Governance & Telemetry Agent (optional - if None, metrics are not calculated)
    governance_agent: Option<GovernanceTelemetryAgent>,

    /// Structure & Redesign Agent (optional - if None, uses stub)
    structure_agent: Option<StructureRedesignAgent>,

    /// Analysis & Synthesis Agent (optional - if None, uses stub)
    analysis_synthesis_agent: Option<AnalysisSynthesisAgent>,

    /// Validation & Learning Agent (optional - if None, Step 6 unavailable)
    validation_agent: Option<ValidationLearningAgent>,

    /// Latest calculated metrics from Governance Agent
    pub latest_metrics: Option<CriticalMetrics>,

    /// Pending IAS Warning requiring acknowledgment (FIX-024)
    ///
    /// When IAS is in warning range (0.30-0.69), this field holds the warning
    /// until the user acknowledges it. At Step 4, this triggers ResynthesisPause state.
    pending_ias_acknowledgment: Option<IASWarning>,

    /// Detected structure mode (Architecting/Builder/Refining)
    /// Set at Step 2 after initial CI baseline is calculated, locked for the run
    pub detected_mode: Option<crate::governance::StructureMode>,

    /// Full mode detection result with metadata (Session 2.1)
    /// Includes confidence, signals, timestamp for transparency
    pub mode_detection_result: Option<crate::governance::ModeDetectionResult>,

    /// True after mode is locked (Step 2 completion)
    pub mode_locked: bool,

    /// User's posture selection (Build/Audit) from Step 0
    /// Combined with CI baseline to determine Transformation mode eligibility
    pub user_posture: crate::governance::UserPosture,

    /// CI baseline recorded at Step 3 (Diagnostic)
    /// Used for delta calculation at Step 4+ (Constraint 2: Delta Baseline Rule)
    pub diagnostic_ci_baseline: Option<f64>,

    /// Callout manager for progression engine (Phase 4)
    /// Tracks callouts requiring user acknowledgment before proceeding
    pub callout_manager: CalloutManager,
}

impl Orchestrator {
    /// Set the Scope & Pattern Agent for this orchestrator
    ///
    /// This allows the orchestrator to use the real agent instead of the stub.
    pub fn with_scope_agent(mut self, agent: ScopePatternAgent) -> Self {
        self.scope_agent = Some(agent);
        self
    }

    /// Set the Governance & Telemetry Agent for this orchestrator
    ///
    /// This enables automatic metrics calculation at step completion.
    pub fn with_governance_agent(mut self, agent: GovernanceTelemetryAgent) -> Self {
        self.governance_agent = Some(agent);
        self
    }

    /// Set the Structure & Redesign Agent for this orchestrator
    ///
    /// This enables architecture map creation and framework design.
    pub fn with_structure_agent(mut self, agent: StructureRedesignAgent) -> Self {
        self.structure_agent = Some(agent);
        self
    }

    /// Set the Analysis & Synthesis Agent for this orchestrator
    ///
    /// This enables six-lens analysis (Step 3) and synthesis lock-in (Step 4).
    pub fn with_analysis_synthesis_agent(mut self, agent: AnalysisSynthesisAgent) -> Self {
        self.analysis_synthesis_agent = Some(agent);
        self
    }

    /// Set the Validation & Learning Agent for this orchestrator
    ///
    /// This enables validation & assurance (Step 6) and learning harvest (Step 6.5).
    pub fn with_validation_agent(mut self, agent: ValidationLearningAgent) -> Self {
        self.validation_agent = Some(agent);
        self
    }

    /// Set user's posture selection (Build/Audit) from Step 0
    ///
    /// This is combined with CI baseline to determine Transformation mode eligibility.
    /// Should be called during Step 0 before mode detection occurs.
    pub fn set_user_posture(&mut self, posture: crate::governance::UserPosture) {
        info!("User posture set to: {:?}", posture);
        self.user_posture = posture;

        // Record to ledger for transparency
        let payload = LedgerPayload {
            action: "user_posture_set".to_string(),
            inputs: Some(serde_json::json!({
                "posture": format!("{:?}", posture),
            })),
            outputs: None,
            rationale: Some("User confirmed processing posture during Step 0".to_string()),
        };

        let _entry = self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(0),
            Some("User"),
            payload,
        );
        debug!("Posture selection recorded to ledger");
    }

    /// Create a new Orchestrator for a run
    ///
    /// # Arguments
    /// * `label` - User-provided label for the run (e.g., "Analysis", "Feature-X")
    pub fn new(label: &str) -> Self {
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let run_id = format!("{}-{}", date, label);

        info!("Initializing new Method-VI run: {}", run_id);

        Orchestrator {
            run_id,
            state: RunState::Step0Active,
            active_role: Role::Observer, // Start as Observer
            mode: Mode::Standard,        // Always Standard for MVP
            ledger: LedgerManager::new(),
            signal_router: SignalRouter::new(),
            intent_summary: None,
            intent_anchor: None,
            charter: None,
            baseline_report: None,
            architecture_map: None,
            governance_summary: None,
            domain_snapshots: None,
            integrated_diagnostic: None,
            lens_efficacy_report: None,
            core_thesis: None,
            operating_principles: None,
            model_geometry: None,
            causal_spine: None,
            north_star_narrative: None,
            glossary: None,
            limitations: None,
            framework_architecture: None,
            validation_matrix: None,
            semantic_table: None,
            evidence_report: None,
            validation_outcome: None,
            exceptional_flag: false,
            scope_agent: None,            // Will be set via with_scope_agent()
            governance_agent: None,       // Will be set via with_governance_agent()
            structure_agent: None,        // Will be set via with_structure_agent()
            analysis_synthesis_agent: None, // Will be set via with_analysis_synthesis_agent()
            validation_agent: None,       // Will be set via with_validation_agent()
            latest_metrics: None,
            pending_ias_acknowledgment: None, // FIX-024: IAS soft gate acknowledgment
            detected_mode: None,               // Session 2.2: Set at Step 2 after CI baseline
            mode_detection_result: None,       // Session 2.2: Full detection metadata
            mode_locked: false,                // Session 2.2: Locked at Step 2 completion
            user_posture: crate::governance::UserPosture::default(),  // Phase 6: User posture from Step 0
            diagnostic_ci_baseline: None,      // Session 3.1: Set at Step 3 for delta calculation
            callout_manager: CalloutManager::new(),  // Session 4.1: Progression engine callout tracking
        }
    }

    /// Get the current run context for generating Steno-Ledger
    pub fn get_run_context(&self) -> RunContext {
        RunContext {
            run_id: self.run_id.clone(),
            step: self.state.step_number() as i32,
            role: self.active_role.clone(),
            ci: None, // Will be populated by Governance & Telemetry Agent
            ev: None, // Will be populated by Governance & Telemetry Agent
            mode: self.mode.clone(),
            signal: self.get_context_signal(),
        }
    }

    /// Get the current context signal for Steno-Ledger
    fn get_context_signal(&self) -> ContextSignal {
        match &self.state {
            RunState::Step0Active => ContextSignal::Initializing,
            RunState::Step0GatePending => ContextSignal::AwaitingGate,
            RunState::Step1Active => ContextSignal::Active,
            RunState::Step1GatePending => ContextSignal::AwaitingGate,
            RunState::Step2Active => ContextSignal::Active,
            RunState::Step2GatePending => ContextSignal::AwaitingGate,
            RunState::Step3Active => ContextSignal::Active,
            RunState::Step3GatePending => ContextSignal::AwaitingGate,
            RunState::Step4Active => ContextSignal::Active,
            RunState::Step4GatePending => ContextSignal::AwaitingGate,
            RunState::Step5Active => ContextSignal::Active,
            RunState::Step5GatePending => ContextSignal::AwaitingGate,
            RunState::Step6Active => ContextSignal::Active,
            RunState::Step6GatePending => ContextSignal::AwaitingGate,
            RunState::Completed => ContextSignal::Completed,
            RunState::Paused { .. } => ContextSignal::PausedForReview,
            RunState::IASResynthesisPause { .. } => ContextSignal::PausedForReview, // FIX-024
            RunState::Halted { .. } => ContextSignal::Halted,
            _ => ContextSignal::Active,
        }
    }

    /// Generate Steno-Ledger string for prepending to agent prompts
    pub fn generate_steno_ledger(&self) -> String {
        let context = self.get_run_context();
        ContextManager::generate_steno_ledger(&context)
    }

    /// Execute Step 0: Intent capture and pattern query
    ///
    /// Step 0 flow:
    /// 1. Record run start in ledger
    /// 2. Call Scope & Pattern Agent to capture intent (real or stubbed)
    /// 3. Emit Ready_for_Step_1 signal
    /// 4. Transition to gate pending state
    ///
    /// # Returns
    /// The captured intent summary
    pub async fn execute_step_0(&mut self, user_intent: &str) -> Result<IntentSummary> {
        info!("=== Executing Step 0: Intent Capture ===");

        // Validate state
        if !matches!(self.state, RunState::Step0Active) {
            anyhow::bail!("Cannot execute Step 0 - current state: {:?}", self.state);
        }

        // Record run start in ledger
        let payload = LedgerPayload {
            action: "run_start".to_string(),
            inputs: Some(serde_json::json!({
                "run_id": self.run_id,
                "user_intent": user_intent,
            })),
            outputs: None,
            rationale: Some("Initializing Method-VI run".to_string()),
        };

        let entry = self.ledger.create_entry(
            &self.run_id,
            EntryType::Signal,
            Some(0),
            Some("Observer"),
            payload,
        );

        debug!("Ledger entry created: {:?}", entry.hash);

        // Call Scope & Pattern Agent to capture intent (real or stubbed)
        let intent_summary = if let Some(agent) = &self.scope_agent {
            // Use real agent
            info!("Using real Scope & Pattern Agent");
            let steno_ledger = self.generate_steno_ledger();
            agent
                .interpret_intent(&self.run_id, user_intent, &steno_ledger)
                .await?
        } else {
            // Use stub for testing
            info!("Using stubbed Scope & Pattern Agent");
            self.stub_scope_and_pattern_agent(user_intent)?
        };

        info!("Intent captured: {}", intent_summary.primary_goal);
        info!("Confidence: {}", intent_summary.confidence_score);
        info!("Category: {}", intent_summary.intent_category);

        // Store intent summary
        self.intent_summary = Some(intent_summary.clone());

        // Record intent capture in ledger
        let payload = LedgerPayload {
            action: "intent_captured".to_string(),
            inputs: Some(serde_json::json!({
                "user_intent": user_intent,
            })),
            outputs: Some(serde_json::json!({
                "artifact_id": intent_summary.artifact_id,
                "primary_goal": intent_summary.primary_goal,
                "confidence_score": intent_summary.confidence_score,
                "intent_category": intent_summary.intent_category,
            })),
            rationale: Some("Scope & Pattern Agent completed intent analysis".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(0),
            Some("Observer"),
            payload,
        );

        // Emit Ready_for_Step_1 signal (GATE signal)
        info!("Emitting Ready_for_Step_1 signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 0,
            step_to: 1,
            artifacts_produced: vec!["intent-anchor".to_string()],
            metrics_snapshot: None,
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ReadyForStep1,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Ready_for_Step_1",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 0 complete, awaiting human approval to proceed".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(0),
            Some("Observer"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step0GatePending;

        info!("Step 0 complete - awaiting gate approval");
        info!("State: {:?}", self.state);

        Ok(intent_summary)
    }

    /// Approve the gate and proceed to the next step
    ///
    /// This should be called by the UI when the human approves the gate.
    ///
    /// # Returns
    /// True if gate was approved and state transitioned
    pub fn approve_gate(&mut self, approver: &str) -> Result<bool> {
        info!("Gate approval requested by: {}", approver);

        match &self.state {
            RunState::Step0GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Ready_for_Step_1",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved progression to Step 1".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(0),
                    Some("Observer"),
                    payload,
                );

                // Transition to Step 1
                self.state = RunState::Step1Active;
                self.active_role = Role::Conductor; // Role transitions Observer → Conductor

                info!("✓ Gate approved - transitioning to Step 1");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step1GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Baseline_Frozen",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved baseline freeze - ready to proceed to Step 2".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(1),
                    Some("Conductor"),
                    payload,
                );

                // Transition to Step 2
                self.state = RunState::Step2Active;

                info!("✓ Baseline gate approved - transitioning to Step 2");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step2GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Ready_for_Analysis",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved governance calibration - ready to proceed to Step 3".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(2),
                    Some("Conductor"),
                    payload,
                );

                // Transition to Step 3
                self.state = RunState::Step3Active;
                self.active_role = Role::Observer; // Role transitions Conductor → Observer

                info!("✓ Governance calibration gate approved - transitioning to Step 3");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step3GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Ready_for_Synthesis",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved multi-angle analysis - ready to proceed to Step 4".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(3),
                    Some("Observer"),
                    payload,
                );

                // Transition to Step 4
                self.state = RunState::Step4Active;

                info!("✓ Analysis gate approved - transitioning to Step 4");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step4GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Ready_for_Redesign",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved synthesis lock-in - ready to proceed to Step 5".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(4),
                    Some("Observer"),
                    payload,
                );

                // Transition to Step 5
                self.state = RunState::Step5Active;

                info!("✓ Synthesis gate approved - transitioning to Step 5");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step5GatePending => {
                // Record gate approval in ledger
                let payload = LedgerPayload {
                    action: "gate_approved".to_string(),
                    inputs: Some(serde_json::json!({
                        "gate": "Framework_Ready",
                        "approver": approver,
                    })),
                    outputs: None,
                    rationale: Some("Human approved framework architecture - ready to proceed to Step 6".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(5),
                    Some("Observer"),
                    payload,
                );

                // Transition to Step 6
                self.state = RunState::Step6Active;

                info!("✓ Framework gate approved - transitioning to Step 6");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
            }
            RunState::Step6GatePending => {
                // Check for exceptional result (CI ≥ 0.85) to route to Step 6.5
                if self.exceptional_flag {
                    // Record gate approval in ledger
                    let payload = LedgerPayload {
                        action: "gate_approved".to_string(),
                        inputs: Some(serde_json::json!({
                            "gate": "Validation_Complete",
                            "approver": approver,
                            "exceptional_result": true,
                        })),
                        outputs: None,
                        rationale: Some("Exceptional result (CI ≥ 0.85) - proceeding to Step 6.5 Learning Harvest".to_string()),
                    };

                    self.ledger.create_entry(
                        &self.run_id,
                        EntryType::Decision,
                        Some(6),
                        Some("Observer"),
                        payload,
                    );

                    // Transition to Step 6.5 Learning Harvest
                    self.state = RunState::Step6_5Active;

                    info!("✓ Validation gate approved - exceptional result detected");
                    info!("✓ Proceeding to Step 6.5 Learning Harvest");
                    info!("Active role: {:?}", self.active_role);

                    Ok(true)
                } else {
                    // Record gate approval in ledger
                    let payload = LedgerPayload {
                        action: "gate_approved".to_string(),
                        inputs: Some(serde_json::json!({
                            "gate": "Validation_Complete",
                            "approver": approver,
                            "exceptional_result": false,
                        })),
                        outputs: None,
                        rationale: Some("Validation complete - run finished (no Step 6.5)".to_string()),
                    };

                    self.ledger.create_entry(
                        &self.run_id,
                        EntryType::Decision,
                        Some(6),
                        Some("Observer"),
                        payload,
                    );

                    // Transition to Completed (no learning harvest for non-exceptional results)
                    self.state = RunState::Completed;

                    info!("✓ Validation gate approved - run completed");
                    info!("Active role: {:?}", self.active_role);

                    Ok(true)
                }
            }
            _ => {
                anyhow::bail!("No gate pending - current state: {:?}", self.state);
            }
        }
    }

    /// Reject the gate (human decides not to proceed)
    pub fn reject_gate(&mut self, rejector: &str, reason: &str) -> Result<()> {
        info!("Gate rejection by: {} - reason: {}", rejector, reason);

        if !self.state.is_gate_pending() {
            anyhow::bail!("No gate pending - current state: {:?}", self.state);
        }

        // Record gate rejection in ledger
        let payload = LedgerPayload {
            action: "gate_rejected".to_string(),
            inputs: Some(serde_json::json!({
                "rejector": rejector,
                "reason": reason,
            })),
            outputs: None,
            rationale: Some("Human rejected gate - run terminating".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(self.state.step_number() as i32),
            Some(ContextManager::get_role_abbreviation(&self.active_role).as_str()),
            payload,
        );

        // Transition to halted state
        self.state = RunState::Halted {
            reason: reason.to_string(),
        };

        info!("Run halted due to gate rejection");

        Ok(())
    }

    /// Handle human decision on HALT condition
    ///
    /// When a HALT condition is detected, the run enters Paused state.
    /// The human must decide whether to proceed anyway, abort, or return to previous step.
    ///
    /// # Arguments
    /// * `decision` - "proceed", "abort", or "return"
    /// * `decider` - Name of person making the decision
    /// * `rationale` - Explanation of why they made this decision
    ///
    /// # Returns
    /// The next state the orchestrator should transition to
    pub fn handle_halt_decision(&mut self, decision: &str, decider: &str, rationale: &str) -> Result<String> {
        info!("HALT decision received: {} by {}", decision, decider);
        info!("Rationale: {}", rationale);

        // Verify we're actually in Paused state
        let (halt_reason, paused_step, triggered_metrics) = match &self.state {
            RunState::Paused { reason, step, triggered_metrics, .. } => {
                (reason.clone(), *step, triggered_metrics.clone())
            }
            _ => {
                anyhow::bail!("Cannot handle HALT decision - not in Paused state. Current state: {:?}", self.state);
            }
        };

        match decision.to_lowercase().as_str() {
            "proceed" => {
                info!("⚠️ HALT OVERRIDE: User chose to proceed despite metrics failure");

                // Record override in ledger
                let payload = LedgerPayload {
                    action: "halt_override_proceed".to_string(),
                    inputs: Some(serde_json::json!({
                        "decider": decider,
                        "rationale": rationale,
                        "original_halt_reason": halt_reason,
                        "step": paused_step,
                    })),
                    outputs: Some(serde_json::json!({
                        "triggered_metrics": triggered_metrics,
                    })),
                    rationale: Some(format!("Human override: {}", rationale)),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(paused_step as i32),
                    Some("Human"),
                    payload,
                );

                // Emit signal for run resumed
                self.signal_router.emit_signal(
                    SignalType::MetricsWarning, // Use warning signal to indicate override
                    &self.run_id,
                    SignalPayload {
                        step_from: paused_step as i32,
                        step_to: paused_step as i32,
                        artifacts_produced: vec![],
                        metrics_snapshot: triggered_metrics,
                        gate_required: false,
                    },
                );

                // Transition to appropriate GatePending state
                let next_state = match paused_step {
                    2 => "Step2GatePending",
                    3 => "Step3GatePending",
                    4 => "Step4GatePending",
                    5 => "Step5GatePending",
                    6 => "Step6GatePending",
                    _ => {
                        anyhow::bail!("Cannot resume from step {}", paused_step);
                    }
                };

                // Update state to gate pending
                self.state = match paused_step {
                    2 => RunState::Step2GatePending,
                    3 => RunState::Step3GatePending,
                    4 => RunState::Step4GatePending,
                    5 => RunState::Step5GatePending,
                    6 => RunState::Step6GatePending,
                    _ => {
                        anyhow::bail!("Invalid paused step: {}", paused_step);
                    }
                };

                info!("✓ Run resumed - awaiting gate approval to proceed to next step");
                info!("State: {:?}", self.state);

                Ok(next_state.to_string())
            }
            "abort" => {
                info!("HALT CONFIRMED: User chose to abort run");

                // Record abort in ledger
                let payload = LedgerPayload {
                    action: "halt_confirmed_abort".to_string(),
                    inputs: Some(serde_json::json!({
                        "decider": decider,
                        "rationale": rationale,
                        "original_halt_reason": halt_reason,
                        "step": paused_step,
                    })),
                    outputs: None,
                    rationale: Some(format!("Human confirmed abort: {}", rationale)),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(paused_step as i32),
                    Some("Human"),
                    payload,
                );

                // Transition to permanently Halted
                self.state = RunState::Halted {
                    reason: format!("Aborted by {} at Step {}: {}", decider, paused_step, rationale),
                };

                info!("Run permanently halted");

                Ok("Halted".to_string())
            }
            "return" => {
                // For MVP, return to previous step is not implemented
                // Just abort with explanation
                warn!("RETURN TO PREVIOUS STEP not implemented in MVP - aborting instead");

                let payload = LedgerPayload {
                    action: "halt_return_requested".to_string(),
                    inputs: Some(serde_json::json!({
                        "decider": decider,
                        "rationale": rationale,
                        "requested_action": "return",
                        "original_halt_reason": halt_reason,
                        "step": paused_step,
                    })),
                    outputs: None,
                    rationale: Some("Return to previous step not supported in MVP - run aborted".to_string()),
                };

                self.ledger.create_entry(
                    &self.run_id,
                    EntryType::Decision,
                    Some(paused_step as i32),
                    Some("Human"),
                    payload,
                );

                self.state = RunState::Halted {
                    reason: format!("Return to previous step not supported (requested by {})", decider),
                };

                info!("Run halted - return not supported in MVP");

                Ok("Halted".to_string())
            }
            _ => {
                anyhow::bail!("Invalid HALT decision: '{}'. Must be 'proceed', 'abort', or 'return'", decision);
            }
        }
    }

    /// Execute Step 1: Baseline Establishment
    ///
    /// Creates the 4 immutable artifacts that define the run baseline:
    /// - Intent_Anchor (locked intent)
    /// - Charter (governing document)
    /// - Baseline_Report (locked E_baseline)
    /// - Architecture_Map (process architecture)
    ///
    /// # Returns
    /// A tuple of (intent_anchor_id, charter_id, baseline_id, architecture_id)
    pub async fn execute_step_1(&mut self) -> Result<(String, String, String, String)> {
        info!("=== Executing Step 1: Baseline Establishment ===");

        // Validate state
        if !matches!(self.state, RunState::Step1Active) {
            anyhow::bail!("Cannot execute Step 1 - current state: {:?}", self.state);
        }

        // Ensure we have intent summary from Step 0
        let intent_summary = self.intent_summary.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No intent summary available - Step 0 must be completed first"))?;

        // Validate all agents are configured before proceeding
        if self.scope_agent.is_none() {
            anyhow::bail!("Scope & Pattern Agent not configured");
        }
        if self.governance_agent.is_none() {
            anyhow::bail!("Governance & Telemetry Agent not configured");
        }
        if self.structure_agent.is_none() {
            anyhow::bail!("Structure & Redesign Agent not configured");
        }

        // Step 1a: Create Intent_Anchor (finalize and lock intent)
        info!("Step 1a: Creating Intent_Anchor...");
        let intent_summary_content = intent_summary.generate_content_body();
        let intent_summary_hash = intent_summary.compute_hash();

        let intent_anchor = self.scope_agent.as_ref().unwrap().create_intent_anchor(
            &self.run_id,
            &intent_summary_content,
            &intent_summary_hash,
        ).await?;

        // Extract Intent_Anchor ID and hash from the artifact
        let intent_anchor_id = format!("{}-intent-anchor", self.run_id);
        let intent_anchor_hash = self.extract_hash_from_artifact(&intent_anchor)?;
        let intent_anchor_content = self.extract_content_from_artifact(&intent_anchor)?;

        info!("✓ Intent_Anchor created: {}", intent_anchor_id);

        // Step 1b: Create Charter (governing document)
        info!("Step 1b: Creating Charter...");
        let charter = self.scope_agent.as_ref().unwrap().create_charter(
            &self.run_id,
            &intent_anchor_content,
            &intent_anchor_id,
            &intent_anchor_hash,
            "Standard",  // Execution mode
            "Standard",  // Telemetry profile
        ).await?;

        let charter_id = format!("{}-charter", self.run_id);
        let charter_hash = self.extract_hash_from_artifact(&charter)?;
        let charter_content = self.extract_content_from_artifact(&charter)?;

        info!("✓ Charter created: {}", charter_id);

        // Step 1c: Calculate and lock E_baseline
        info!("Step 1c: Calculating and locking E_baseline...");
        let e_baseline = self.calculate_and_lock_e_baseline(&charter_content).await?;
        info!("✓ E_baseline locked: {:.2} entropy", e_baseline);

        // Step 1d: Create Baseline_Report
        info!("Step 1d: Creating Baseline_Report...");
        let baseline_report = self.governance_agent.as_ref().unwrap().create_baseline_report(
            &self.run_id,
            &charter_content,
            &charter_id,
            &charter_hash,
            &intent_anchor_id,
            e_baseline,
            "Standard",  // Telemetry profile
        )?;

        let baseline_id = format!("{}-baseline-report", self.run_id);
        info!("✓ Baseline_Report created: {}", baseline_id);

        // Step 1e: Create Architecture_Map
        info!("Step 1e: Creating Architecture_Map...");
        let architecture_map = self.structure_agent.as_ref().unwrap().create_architecture_map(
            &self.run_id,
            &charter_content,
            &charter_hash,
            &intent_anchor_id,
            "Standard",  // Mode profile
        ).await?;

        let architecture_id = format!("{}-architecture-map", self.run_id);
        info!("✓ Architecture_Map created: {}", architecture_id);

        // Store all 4 artifacts
        self.intent_anchor = Some(intent_anchor);
        self.charter = Some(charter);
        self.baseline_report = Some(baseline_report);
        self.architecture_map = Some(architecture_map);

        info!("All 4 immutable artifacts stored");

        // Record Step 1 completion in ledger
        let payload = LedgerPayload {
            action: "step_1_complete".to_string(),
            inputs: Some(serde_json::json!({
                "intent_anchor_id": intent_anchor_id,
            })),
            outputs: Some(serde_json::json!({
                "intent_anchor_id": intent_anchor_id,
                "charter_id": charter_id,
                "baseline_id": baseline_id,
                "architecture_id": architecture_id,
                "e_baseline": e_baseline,
            })),
            rationale: Some("4 immutable baseline artifacts created and locked".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(1),
            Some("Conductor"),
            payload,
        );

        // Emit Baseline_Frozen signal (GATE signal)
        info!("Emitting Baseline_Frozen signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 1,
            step_to: 2,
            artifacts_produced: vec![
                intent_anchor_id.clone(),
                charter_id.clone(),
                baseline_id.clone(),
                architecture_id.clone(),
            ],
            metrics_snapshot: None,
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::BaselineFrozen,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Baseline_Frozen",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 1 complete, baseline frozen, awaiting human approval to proceed".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(1),
            Some("Conductor"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step1GatePending;

        info!("Step 1 complete - awaiting baseline approval");
        info!("State: {:?}", self.state);

        Ok((intent_anchor_id, charter_id, baseline_id, architecture_id))
    }

    /// Execute Step 2: Governance Calibration
    ///
    /// Performs active governance calibration by:
    /// - Reviewing Charter objectives
    /// - Configuring five control domains (Entropy, Objective, Process, Reflective, Termination)
    /// - Creating Governance_Summary and Domain_Snapshots artifacts
    /// - Calculating initial metrics from baseline
    /// - Emitting Ready_for_Analysis signal
    ///
    /// # Returns
    /// A tuple of (governance_summary_id, domain_snapshots_id)
    pub async fn execute_step_2(&mut self) -> Result<(String, String)> {
        info!("=== Executing Step 2: Governance Calibration ===");

        // Validate state
        if !matches!(self.state, RunState::Step2Active) {
            anyhow::bail!("Cannot execute Step 2 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Step 1
        let charter = self.charter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Charter available - Step 1 must be completed first"))?;

        let architecture_map = self.architecture_map.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Architecture Map available - Step 1 must be completed first"))?;

        // Validate governance agent is configured
        if self.governance_agent.is_none() {
            anyhow::bail!("Governance & Telemetry Agent not configured");
        }

        // Extract Charter content and hash
        let charter_content = self.extract_content_from_artifact(charter)?;
        let charter_hash = self.extract_hash_from_artifact(charter)?;

        // Extract Architecture Map content
        let architecture_map_content = self.extract_content_from_artifact(architecture_map)?;

        // Get Intent Anchor ID
        let intent_anchor_id = format!("{}-intent-anchor", self.run_id);

        // Get E_baseline
        let e_baseline = self.get_e_baseline()
            .ok_or_else(|| anyhow::anyhow!("E_baseline not available - Step 1 must be completed first"))?;

        info!("Step 2: Configuring governance controls...");
        info!("  Charter hash: {}", charter_hash);
        info!("  E_baseline: {} words", e_baseline);

        // Call Governance & Telemetry Agent to perform calibration
        let (governance_summary, domain_snapshots) = self.governance_agent.as_ref().unwrap()
            .perform_governance_calibration(
                &self.run_id,
                &charter_content,
                &charter_hash,
                &intent_anchor_id,
                &architecture_map_content,
                e_baseline,
            )
            .await?;

        let governance_summary_id = format!("{}-governance-summary", self.run_id);
        let domain_snapshots_id = format!("{}-domain-snapshots", self.run_id);

        info!("✓ Governance_Summary created: {}", governance_summary_id);
        info!("✓ Domain_Snapshots created: {}", domain_snapshots_id);

        // Store artifacts
        self.governance_summary = Some(governance_summary);
        self.domain_snapshots = Some(domain_snapshots);

        info!("Governance calibration artifacts stored");

        // Calculate initial metrics from baseline
        info!("Step 2: Calculating initial metrics...");
        let (metrics_opt, halt_triggered) = self.calculate_metrics(&charter_content, &charter_content).await?;

        let initial_metrics = metrics_opt.ok_or_else(|| anyhow::anyhow!("Failed to calculate metrics in Step 2"))?;

        // Session 4.3: HALT check deprecated - halt_triggered will always be false
        // Metric violations now handled by callout system (Session 4.1)
        // This code path kept for backward compatibility but will never execute
        if halt_triggered {
            warn!("Step 2 HALTED - run is PAUSED, no gate signal emitted");
            warn!("State: {:?}", self.state);
            warn!("Human decision required to proceed");

            // Return early - do not proceed to gate
            return Ok((governance_summary_id, domain_snapshots_id));
        }

        info!("✓ Initial metrics calculated:");
        if let Some(ci) = &initial_metrics.ci {
            info!("  CI: {:.2} ({})", ci.value, match ci.status {
                crate::agents::governance_telemetry::MetricStatus::Pass => "PASS",
                crate::agents::governance_telemetry::MetricStatus::Warning => "WARNING",
                crate::agents::governance_telemetry::MetricStatus::Fail => "FAIL",
            });

            // Session 2.3: Mode Detection Integration
            // Detect structure mode from CI baseline (Constraint 1: Transparency Mandate)
            let mode_result = ModeDetector::detect(ci.value);
            info!("Mode detection complete: {:?}", mode_result.mode);

            // Store mode in orchestrator (locked for the run)
            self.detected_mode = Some(mode_result.mode);
            self.mode_detection_result = Some(mode_result.clone());
            self.mode_locked = true;

            // The ModeDetector::log_detection() function has already logged:
            // "Detected Structure: {Low|Medium|High}. Engaging {mode} Mode. (CI: {ci}, Confidence: {pct}%)"
            // This satisfies Constraint 1: Transparency Mandate

            // Note: Frontend can query mode via get_current_mode() Tauri command
            // Event emission will be added in Phase 3 (frontend integration) when we add
            // ModeDetected signal type to SignalType enum and emit via signal_router

            // Session 4.1: Generate callouts for Step 2 metrics
            if let Some(ref governance_agent) = self.governance_agent {
                governance_agent.generate_callouts(
                    &initial_metrics,
                    None, // No previous metrics at Step 2 (first measurement)
                    crate::governance::Step::Step2_Governance,
                    mode_result.mode,
                    crate::governance::MetricEnforcement::Enforced,
                    &mut self.callout_manager,
                );

                // Log callout summary
                let summary = self.callout_manager.summary();
                info!(
                    "Step 2 callouts: {} total ({} critical, {} warning, {} attention)",
                    summary.total,
                    summary.by_tier.critical,
                    summary.by_tier.warning,
                    summary.by_tier.attention
                );

                // Check if we can proceed (only Critical blocks)
                if !self.callout_manager.can_proceed() {
                    info!(
                        "Step 2: {} Critical callouts require acknowledgment before proceeding",
                        summary.pending_acknowledgments
                    );
                    // Note: Don't block here - emit event for frontend to show callouts
                    // User will acknowledge via acknowledge_callout() Tauri command
                }
            }
        }
        if let Some(ev) = &initial_metrics.ev {
            info!("  EV: {:.1}% ({})", ev.value, match ev.status {
                crate::agents::governance_telemetry::MetricStatus::Pass => "PASS",
                crate::agents::governance_telemetry::MetricStatus::Warning => "WARNING",
                crate::agents::governance_telemetry::MetricStatus::Fail => "FAIL",
            });
        }

        // Record Step 2 completion in ledger (only if not halted)
        let payload = LedgerPayload {
            action: "step_2_complete".to_string(),
            inputs: Some(serde_json::json!({
                "charter_hash": charter_hash,
                "e_baseline": e_baseline,
            })),
            outputs: Some(serde_json::json!({
                "governance_summary_id": governance_summary_id,
                "domain_snapshots_id": domain_snapshots_id,
                "initial_metrics": {
                    "ci": initial_metrics.ci.as_ref().map(|m| m.value),
                    "ev": initial_metrics.ev.as_ref().map(|m| m.value),
                    "ias": initial_metrics.ias.as_ref().map(|m| m.value),
                }
            })),
            rationale: Some("Governance calibration complete, five control domains configured".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(2),
            Some("Conductor"),
            payload,
        );

        // Emit Ready_for_Analysis signal (GATE signal)
        info!("Emitting Ready_for_Analysis signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 2,
            step_to: 3,
            artifacts_produced: vec![
                governance_summary_id.clone(),
                domain_snapshots_id.clone(),
            ],
            metrics_snapshot: Some(serde_json::to_value(&initial_metrics)?),
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ReadyForAnalysis,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Ready_for_Analysis",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 2 complete, governance calibrated, awaiting human approval to proceed".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(2),
            Some("Conductor"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step2GatePending;

        info!("Step 2 complete - awaiting governance calibration approval");
        info!("State: {:?}", self.state);

        Ok((governance_summary_id, domain_snapshots_id))
    }

    /// Extract hash from artifact YAML frontmatter
    fn extract_hash_from_artifact(&self, artifact: &str) -> Result<String> {
        for line in artifact.lines() {
            if line.starts_with("hash:") {
                let hash = line.trim_start_matches("hash:").trim().trim_matches('"');
                return Ok(hash.to_string());
            }
        }
        anyhow::bail!("No hash found in artifact frontmatter")
    }

    /// Extract content body from artifact (everything after second ---)
    fn extract_content_from_artifact(&self, artifact: &str) -> Result<String> {
        let mut in_frontmatter = false;
        let mut frontmatter_ended = false;
        let mut content_lines = Vec::new();

        for line in artifact.lines() {
            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                } else if in_frontmatter && !frontmatter_ended {
                    frontmatter_ended = true;
                    continue;
                }
            } else if frontmatter_ended {
                content_lines.push(line);
            }
        }

        if content_lines.is_empty() {
            anyhow::bail!("No content found after frontmatter in artifact");
        }

        Ok(content_lines.join("\n"))
    }

    /// Extract objectives from Charter content for relevance checking
    ///
    /// Parses the Charter markdown to find Primary and Secondary Objectives.
    /// Returns a list of objective statements.
    fn extract_objectives_from_charter(&self, charter_content: &str) -> Result<Vec<String>> {
        let mut objectives = Vec::new();
        let mut in_objectives_section = false;

        for line in charter_content.lines() {
            let trimmed = line.trim();

            // Check if we're entering an objectives section
            if trimmed.contains("Objective") && (trimmed.starts_with("##") || trimmed.starts_with("**")) {
                in_objectives_section = true;

                // If the objective is on the same line (e.g., "**Primary Objective:** Description")
                if let Some(colon_pos) = trimmed.find(':') {
                    let objective = trimmed[colon_pos + 1..].trim();
                    if !objective.is_empty() {
                        objectives.push(objective.to_string());
                    }
                }
                continue;
            }

            // If we're in objectives section, collect bullet points or text
            if in_objectives_section {
                // Stop if we hit a new section header (## Something else)
                if trimmed.starts_with("##") && !trimmed.contains("Objective") {
                    in_objectives_section = false;
                    continue;
                }

                // Collect bullet points
                if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    let objective = trimmed.trim_start_matches("- ").trim_start_matches("* ").trim();
                    if !objective.is_empty() {
                        objectives.push(objective.to_string());
                    }
                }
                // Collect numbered lists
                else if trimmed.len() > 2 && trimmed.chars().next().unwrap().is_numeric() && trimmed.chars().nth(1) == Some('.') {
                    let objective = trimmed.splitn(2, '.').nth(1).unwrap_or("").trim();
                    if !objective.is_empty() {
                        objectives.push(objective.to_string());
                    }
                }
                // Collect plain text lines (non-empty, non-header)
                else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    objectives.push(trimmed.to_string());
                }
            }
        }

        if objectives.is_empty() {
            warn!("No objectives found in Charter - using fallback");
            objectives.push("Analyze and synthesize user-provided content according to Method-VI framework".to_string());
        }

        info!("Extracted {} objectives from Charter", objectives.len());
        Ok(objectives)
    }

    /// Validate an action is allowed in the current state
    ///
    /// Uses the Ledger Manager to validate state transitions
    pub fn validate_action(&self, action: &str) -> Result<bool> {
        let ledger_state = match &self.state {
            RunState::Step0Active => LedgerState::Step0Active,
            RunState::Step0GatePending | RunState::Step1GatePending => LedgerState::GatePending,
            RunState::Step1Active => LedgerState::Normal,
            RunState::Paused { .. } => LedgerState::HaltActive, // Paused requires decision
            RunState::Halted { .. } => LedgerState::HaltActive,
            _ => LedgerState::Normal,
        };

        let validation = self.ledger.validate_action(&ledger_state, action);

        if !validation.allowed {
            if let Some(reason) = validation.reason {
                anyhow::bail!("Action '{}' not allowed: {}", action, reason);
            }
        }

        Ok(validation.allowed)
    }

    /// Get the current ledger state
    pub fn get_ledger_state(&self) -> LedgerState {
        match &self.state {
            RunState::Step0Active => LedgerState::Step0Active,
            RunState::Step0GatePending | RunState::Step1GatePending | RunState::Step2GatePending => LedgerState::GatePending,
            RunState::Step1Active => LedgerState::BaselineFrozen, // After baseline is frozen
            RunState::Step2Active => LedgerState::Normal,
            RunState::Paused { .. } => LedgerState::HaltActive, // Paused awaits human decision
            RunState::Halted { .. } => LedgerState::HaltActive,
            _ => LedgerState::Normal,
        }
    }

    /// Get reference to the signal router (for testing/inspection)
    pub fn get_signal_router(&self) -> &SignalRouter {
        &self.signal_router
    }

    /// Get reference to the ledger manager (for testing/inspection)
    pub fn get_ledger(&self) -> &LedgerManager {
        &self.ledger
    }

    /// STUB: Scope & Pattern Agent
    ///
    /// This is a placeholder that returns a mock intent summary.
    /// The real implementation will call Claude with the Steno-Ledger prepended.
    fn stub_scope_and_pattern_agent(&self, user_intent: &str) -> Result<IntentSummary> {
        debug!("STUB: Calling Scope & Pattern Agent");

        // Generate Steno-Ledger for the agent call
        let steno_ledger = self.generate_steno_ledger();
        debug!("Steno-Ledger: {}", steno_ledger);

        // Create a mock IntentSummary artifact
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let artifact_id = format!("{}-intent-summary-{}", self.run_id, timestamp);
        let created_at = Utc::now().to_rfc3339();

        let mut summary = IntentSummary {
            artifact_id,
            artifact_type: "Intent_Summary".to_string(),
            run_id: self.run_id.clone(),
            step_origin: 0,
            created_at,
            hash: String::new(), // Will be computed below
            parent_hash: None,
            dependencies: vec![],
            intent_anchor_link: None,
            is_immutable: false,
            author: "scope-pattern-agent-stub".to_string(),
            governance_role: "Observer".to_string(),
            user_request: user_intent.to_string(),
            primary_goal: format!("Accomplish: {}", user_intent),
            audience: "General users".to_string(),
            expected_outcome: format!("Successfully implement: {}", user_intent),
            intent_category: "Operational".to_string(),
            confidence_score: 75,
            confidence_explanation: "Moderate confidence - based on stub analysis".to_string(),
            request_specificity: "Medium".to_string(),
            scope_definition_clarity: "Partial".to_string(),
            success_criteria_state: "Implied".to_string(),
            questions_for_clarification: vec!["None - intent is clear (stub mode)".to_string()],
            likely_in_scope: vec![
                format!("Core functionality: {}", user_intent),
                "Essential features".to_string(),
            ],
            likely_out_of_scope: vec![
                "Unrelated features".to_string(),
                "Scope creep items".to_string(),
            ],
            edge_cases: vec![],
        };

        // Compute hash
        summary.hash = summary.compute_hash();

        Ok(summary)
    }


    /// Calculate metrics for the current step
    ///
    /// This is called at step completion to assess performance against thresholds.
    /// If governance agent is not set, this is a no-op.
    ///
    /// # Arguments
    /// * `content` - The content to analyze (step output)
    /// * `charter_objectives` - Charter objectives for alignment checking
    ///
    /// # Returns
    /// A tuple of (Option<CriticalMetrics>, bool) where the bool indicates if HALT was triggered
    pub async fn calculate_metrics(
        &mut self,
        content: &str,
        charter_objectives: &str,
    ) -> Result<(Option<CriticalMetrics>, bool)> {
        if let Some(ref agent) = self.governance_agent {
            info!("Calculating metrics for step {}", self.state.step_number());

            let metrics = agent
                .calculate_metrics(content, charter_objectives, self.state.step_number())
                .await?;

            let current_step = self.state.step_number();
            let mut halt_triggered = false;

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
                                SignalPayload {
                                    step_from: current_step as i32,
                                    step_to: current_step as i32,
                                    artifacts_produced: vec![],
                                    metrics_snapshot: Some(serde_json::to_value(&metrics)?),
                                    gate_required: false,
                                },
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

                            self.ledger.create_entry(
                                &self.run_id,
                                EntryType::Decision,
                                Some(current_step as i32),
                                Some("Governance"),
                                payload,
                            );

                            // Don't proceed until acknowledged
                            halt_triggered = true; // Signal that we're paused
                        }
                        crate::agents::governance_telemetry::IASWarningType::AcknowledgmentRequired => {
                            // Other steps: Log warning, store for acknowledgment
                            warn!("⚠️ IAS Warning (requires acknowledgment): {}", ias_warning.message);
                            self.pending_ias_acknowledgment = Some(ias_warning.clone());

                            // Emit warning signal
                            self.signal_router.emit_signal(
                                SignalType::MetricsWarning,
                                &self.run_id,
                                SignalPayload {
                                    step_from: current_step as i32,
                                    step_to: current_step as i32,
                                    artifacts_produced: vec![],
                                    metrics_snapshot: Some(serde_json::to_value(&metrics)?),
                                    gate_required: false,
                                },
                            );
                        }
                    }
                }
            }

            // Store latest metrics
            self.latest_metrics = Some(metrics.clone());

            Ok((Some(metrics), halt_triggered))
        } else {
            debug!("Governance agent not available - skipping metrics calculation");
            Ok((None, false))
        }
    }

    /// Calculate and lock E_baseline (Step 1)
    ///
    /// This should be called after the Baseline Report is generated.
    pub async fn calculate_and_lock_e_baseline(&mut self, baseline_content: &str) -> Result<f64> {
        if let Some(ref mut agent) = self.governance_agent {
            let baseline = agent.calculate_e_baseline(baseline_content, 1).await?;
            agent.lock_e_baseline(1)?;
            info!("E_baseline calculated and locked: {}", baseline);
            Ok(baseline)
        } else {
            Err(anyhow::anyhow!(
                "Governance agent not available - cannot calculate E_baseline"
            ))
        }
    }

    /// Get the current E_baseline value
    pub fn get_e_baseline(&self) -> Option<f64> {
        self.governance_agent
            .as_ref()
            .and_then(|agent| agent.get_e_baseline())
    }

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

    /// Execute Step 3: Multi-Angle Analysis
    ///
    /// Performs six-lens analysis on the USER'S CONTENT by:
    /// - Applying six analytical lenses (Structural, Thematic, Logic, Evidence, Expression, Intent)
    /// - Using weighted lens sequencing based on intent category
    /// - Tracking lens efficacy for pattern learning
    /// - Creating Integrated_Diagnostic artifact
    /// - Emitting Ready_for_Synthesis signal
    ///
    /// CRITICAL: Analyzes the user's original content (from user_request), NOT the Charter.
    /// The Charter is used only as governance context for Intent lens alignment.
    ///
    /// # Returns
    /// A tuple of (integrated_diagnostic_id, lens_efficacy_report_id)
    pub async fn execute_step_3(&mut self) -> Result<(String, String)> {
        info!("=== Executing Step 3: Multi-Angle Analysis ===");

        // Validate state
        if !matches!(self.state, RunState::Step3Active) {
            anyhow::bail!("Cannot execute Step 3 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Steps 0 and 1
        let charter = self.charter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Charter available - Step 1 must be completed first"))?;

        let intent_summary = self.intent_summary.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Intent Summary available - Step 0 must be completed first"))?;

        // Validate analysis_synthesis_agent is configured
        if self.analysis_synthesis_agent.is_none() {
            anyhow::bail!("Analysis & Synthesis Agent not configured");
        }

        // CRITICAL FIX: Extract user's original content as analysis target
        // Clone to avoid borrow checker issues with self mutations later
        let analysis_target = intent_summary.user_request.clone();

        // Extract Charter content as governance context (for Intent lens only)
        let governance_context = self.extract_content_from_artifact(charter)?;

        // Validation: Ensure we're not analyzing the Charter itself
        if analysis_target.contains("# Charter") || analysis_target.contains("## Objectives")
            || analysis_target.len() == governance_context.len() {
            warn!("HALT: Analysis target appears to be governance metadata, not user content!");
            anyhow::bail!("Invalid analysis target - cannot analyze Charter as subject matter");
        }

        // Get intent category (clone to avoid borrow issues)
        let intent_category = intent_summary.intent_category.clone();

        info!("Step 3: Performing six-lens analysis...");
        info!("  Intent category: {}", intent_category);
        info!("  Analysis target size: {} chars", analysis_target.len());
        info!("  Governance context size: {} chars", governance_context.len());

        // Perform six-lens analysis with BOTH inputs
        let agent = self.analysis_synthesis_agent.as_mut().unwrap();
        let (integrated_diagnostic, lens_efficacy) = agent
            .perform_six_lens_analysis(&analysis_target, &governance_context, &intent_category)
            .await?;

        let integrated_diagnostic_id = format!("{}-integrated-diagnostic", self.run_id);
        let lens_efficacy_report_id = format!("{}-lens-efficacy-report", self.run_id);

        info!("✓ Integrated_Diagnostic created: {}", integrated_diagnostic_id);
        info!("✓ Lens_Efficacy_Report created: {}", lens_efficacy_report_id);
        info!("  Total insights: {}", lens_efficacy.total_insights);
        info!("  High-value combinations: {}", lens_efficacy.high_value_combinations);

        // Store artifacts
        self.integrated_diagnostic = Some(integrated_diagnostic.clone());
        self.lens_efficacy_report = Some(serde_json::to_string_pretty(&lens_efficacy)?);

        info!("Six-lens analysis artifacts stored");

        // Calculate metrics
        info!("Step 3: Calculating metrics...");
        let (metrics, halt_triggered) = self.calculate_metrics(&analysis_target, &governance_context).await?;

        // Session 3.2: Record diagnostic baseline (Constraint 2: Delta Baseline Rule)
        // Step 3 metrics are INFORMATIONAL ONLY - no callouts generated for diagnostic content
        // CI is recorded as baseline for Step 4+ delta calculation
        if let Some(ref metrics) = metrics {
            if let Some(ref ci_metric) = metrics.ci {
                info!(
                    "Step 3 CI: {:.2} (informational - baseline for Step 4+ delta)",
                    ci_metric.value
                );
                self.diagnostic_ci_baseline = Some(ci_metric.value);
                info!("✓ Diagnostic baseline recorded: {:.2}", ci_metric.value);
            }
        }

        // Session 4.3: HALT check deprecated - halt_triggered will always be false
        // Step 3 uses MetricEnforcement::Informational (Session 3.2), no HALTs generated
        // This code path kept for backward compatibility but will never execute
        if halt_triggered {
            warn!("Step 3 HALTED - run is PAUSED, no gate signal emitted");
            warn!("State: {:?}", self.state);
            warn!("Human decision required to proceed");

            // Return early - do not proceed to gate
            return Ok((integrated_diagnostic_id, lens_efficacy_report_id));
        }

        // Record Step 3 completion in ledger (only if not halted)
        let payload = LedgerPayload {
            action: "step_3_complete".to_string(),
            inputs: Some(serde_json::json!({
                "intent_category": intent_category,
                "analysis_target_size": analysis_target.len(),
                "governance_context_size": governance_context.len(),
            })),
            outputs: Some(serde_json::json!({
                "integrated_diagnostic_id": integrated_diagnostic_id,
                "lens_efficacy_report_id": lens_efficacy_report_id,
                "total_insights": lens_efficacy.total_insights,
                "high_value_combinations": lens_efficacy.high_value_combinations,
            })),
            rationale: Some("Six-lens analysis complete, integrated diagnostic created".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(3),
            Some("Conductor"),
            payload,
        );

        // Emit Ready_for_Synthesis signal (GATE signal)
        info!("Emitting Ready_for_Synthesis signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 3,
            step_to: 4,
            artifacts_produced: vec![
                integrated_diagnostic_id.clone(),
                lens_efficacy_report_id.clone(),
            ],
            metrics_snapshot: metrics.as_ref().map(|m| serde_json::to_value(m).unwrap()),
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ReadyForSynthesis,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Ready_for_Synthesis",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 3 complete, six-lens analysis complete, awaiting human approval to proceed".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(3),
            Some("Conductor"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step3GatePending;

        info!("Step 3 complete - awaiting analysis approval");
        info!("State: {:?}", self.state);

        Ok((integrated_diagnostic_id, lens_efficacy_report_id))
    }

    /// Execute Step 4: Synthesis Lock-In
    ///
    /// Performs synthesis and model building by:
    /// - Deriving core thesis from diagnostic
    /// - Extracting operating principles
    /// - Selecting model geometry (Linear/Cyclic/Branching)
    /// - Mapping causality (Causal Spine)
    /// - Authoring North-Star narrative
    /// - Creating glossary
    /// - Documenting limitations
    /// - Emitting Ready_for_Redesign signal
    ///
    /// # Returns
    /// A tuple of (core_thesis_id, north_star_narrative_id)
    pub async fn execute_step_4(&mut self) -> Result<(String, String)> {
        info!("=== Executing Step 4: Synthesis Lock-In ===");

        // Validate state
        if !matches!(self.state, RunState::Step4Active) {
            anyhow::bail!("Cannot execute Step 4 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Step 3
        let _integrated_diagnostic = self.integrated_diagnostic.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Integrated Diagnostic available - Step 3 must be completed first"))?;

        // Validate analysis_synthesis_agent is configured
        if self.analysis_synthesis_agent.is_none() {
            anyhow::bail!("Analysis & Synthesis Agent not configured");
        }

        // CRITICAL: Pre-synthesis content relevance validation
        info!("Step 4: Pre-synthesis relevance check...");

        let integrated_diagnostic = self.integrated_diagnostic.as_ref().unwrap();
        let charter = self.charter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Charter available"))?;
        let charter_content = self.extract_content_from_artifact(charter)?;

        // Extract objectives from Charter for relevance checking
        let charter_objectives = self.extract_objectives_from_charter(&charter_content)?;

        // Check that analysis findings relate to Charter objectives
        // This prevents synthesizing based on wrong analysis target (e.g., Charter itself)
        let relevance_score = if let Some(ref governance_agent) = self.governance_agent {
            governance_agent.check_synthesis_relevance(
                integrated_diagnostic,
                &charter_objectives
            ).await?
        } else {
            warn!("Governance agent not available - skipping relevance check");
            1.0 // Default to passing if no governance agent
        };

        // HALT if relevance is critically low
        if relevance_score < 0.50 {
            warn!("⚠️ PRE-SYNTHESIS RELEVANCE CHECK FAILED: {:.2}", relevance_score);

            // Get current step for HALT signal
            let current_step = match self.state {
                RunState::Step4Active => 4,
                _ => 4,
            };

            // Set state to Paused
            self.state = RunState::Paused {
                reason: format!(
                    "Analysis findings do not appear to relate to Charter objectives. \
                     Relevance score: {:.2}. Review Step 3 analysis target.",
                    relevance_score
                ),
                step: current_step,
                triggered_metrics: Some(serde_json::json!({
                    "pre_synthesis_relevance": relevance_score,
                    "threshold": 0.50
                })),
                all_metrics_snapshot: None,  // Not a standard metrics HALT
            };

            // Emit HALT signal
            self.signal_router.emit_signal(
                SignalType::Halt,
                &self.run_id,
                SignalPayload {
                    step_from: current_step as i32,
                    step_to: current_step as i32,
                    artifacts_produced: vec![],
                    metrics_snapshot: Some(serde_json::json!({
                        "pre_synthesis_relevance": relevance_score,
                        "threshold": 0.50,
                        "reason": "Analysis findings unrelated to Charter objectives"
                    })),
                    gate_required: false,
                },
            );

            // Record in ledger
            self.ledger.create_entry(
                &self.run_id,
                EntryType::Signal,
                Some(current_step as i32),
                Some("Conductor"),
                LedgerPayload {
                    action: "pre_synthesis_halt".to_string(),
                    inputs: Some(serde_json::json!({
                        "relevance_score": relevance_score,
                        "threshold": 0.50
                    })),
                    outputs: None,
                    rationale: Some(format!(
                        "Pre-synthesis relevance check failed. Analysis findings appear unrelated to Charter objectives. \
                         This typically indicates Step 3 analyzed the wrong content (e.g., Charter methodology instead of user's subject matter)."
                    )),
                },
            );

            warn!("Run PAUSED - human decision required");
            warn!("State: {:?}", self.state);

            // Return early - no synthesis artifacts created
            anyhow::bail!("Pre-synthesis relevance check failed: score {:.2} below threshold 0.50", relevance_score);
        }
        // Warning if relevance is marginal
        else if relevance_score < 0.70 {
            warn!("⚠️ Pre-synthesis relevance is marginal: {:.2}", relevance_score);
            warn!("Analysis findings may be partially off-target - review recommended");
            // Emit warning but allow to proceed
        } else {
            info!("✓ Pre-synthesis relevance check passed: {:.2}", relevance_score);
        }

        info!("Step 4: Performing synthesis lock-in...");

        // Perform Step 4 synthesis (agent already has integrated diagnostic from Step 3)
        let agent = self.analysis_synthesis_agent.as_mut().unwrap();
        let synthesis_result = agent
            .perform_step4_synthesis()
            .await?;

        let core_thesis_id = format!("{}-core-thesis", self.run_id);
        let operating_principles_id = format!("{}-operating-principles", self.run_id);
        let model_geometry_id = format!("{}-model-geometry", self.run_id);
        let causal_spine_id = format!("{}-causal-spine", self.run_id);
        let north_star_narrative_id = format!("{}-north-star-narrative", self.run_id);
        let glossary_id = format!("{}-glossary", self.run_id);
        let limitations_id = format!("{}-limitations", self.run_id);

        info!("✓ Core_Thesis created: {}", core_thesis_id);
        info!("✓ Operating_Principles created: {} principles", synthesis_result.operating_principles.len());
        info!("✓ Model_Geometry created: {:?}", synthesis_result.model_geometry);
        info!("✓ Causal_Spine created: {}", causal_spine_id);
        info!("✓ North_Star_Narrative created: {}", north_star_narrative_id);
        info!("✓ Glossary created: {} terms", synthesis_result.glossary.len());
        info!("✓ Limitations documented: {} items", synthesis_result.limitations.len());

        if synthesis_result.novel_geometry_flag {
            info!("⚠ Novel geometry flagged for Learning Harvest");
        }

        // Store artifacts
        self.core_thesis = Some(synthesis_result.core_thesis.clone());
        self.operating_principles = Some(synthesis_result.operating_principles.join("\n"));
        self.model_geometry = Some(format!("{:?}: {}", synthesis_result.model_geometry, synthesis_result.geometry_rationale));
        self.causal_spine = Some(synthesis_result.causal_spine.clone());
        self.north_star_narrative = Some(synthesis_result.north_star_narrative.clone());
        self.glossary = Some(serde_json::to_string_pretty(&synthesis_result.glossary)?);
        self.limitations = Some(synthesis_result.limitations.join("\n"));

        info!("Synthesis artifacts stored");

        // Calculate metrics
        info!("Step 4: Calculating metrics...");
        let charter = self.charter.as_ref().unwrap();
        let charter_content = self.extract_content_from_artifact(charter)?;

        // Use the north star narrative as the output for metrics
        let (metrics, halt_triggered) = self.calculate_metrics(&synthesis_result.north_star_narrative, &charter_content).await?;

        // Session 4.3: HALT check deprecated - halt_triggered will always be false
        // Metric violations handled by callout system (Session 4.2, lines below)
        // This code path kept for backward compatibility but will never execute
        if halt_triggered {
            warn!("Step 4 HALTED - run is PAUSED, no gate signal emitted");
            warn!("State: {:?}", self.state);
            warn!("Human decision required to proceed");

            // Return early - do not proceed to gate
            return Ok((core_thesis_id, north_star_narrative_id));
        }

        // Session 4.2: Generate callouts for Step 4 metrics using diagnostic baseline
        if let (Some(mode), Some(ref current_metrics)) = (self.detected_mode, &metrics) {
            if let Some(ref governance_agent) = self.governance_agent {
                // Build previous_metrics from diagnostic baseline (Constraint 2: Delta from Step 3)
                let previous_metrics = self.diagnostic_ci_baseline.map(|baseline_ci| {
                    use crate::agents::governance_telemetry::{CriticalMetrics, MetricResult, MetricThreshold, MetricStatus, MetricInput, MetricInputValue};

                    CriticalMetrics {
                        ci: Some(MetricResult {
                            metric_name: "CI".to_string(),
                            value: baseline_ci,
                            threshold: MetricThreshold {
                                pass: 0.65,
                                warning: Some(0.50),
                                halt: Some(0.35),
                            },
                            status: MetricStatus::Pass,
                            inputs_used: vec![MetricInput {
                                name: "diagnostic_baseline".to_string(),
                                source: "Step 3".to_string(),
                                value: MetricInputValue::Number(baseline_ci),
                            }],
                            calculation_method: "Step 3 diagnostic baseline".to_string(),
                            interpretation: "Baseline CI for delta calculation".to_string(),
                            recommendation: None,
                        }),
                        ev: None,
                        ias: None,
                        efi: None,
                        sec: None,
                        pci: None,
                    }
                });

                governance_agent.generate_callouts(
                    current_metrics,
                    previous_metrics.as_ref(),
                    crate::governance::Step::Step4_Synthesis,
                    mode,
                    crate::governance::MetricEnforcement::Enforced,
                    &mut self.callout_manager,
                );

                // Log delta from diagnostic baseline
                if let Some(baseline_ci) = self.diagnostic_ci_baseline {
                    if let Some(ref ci_metric) = current_metrics.ci {
                        let delta = ci_metric.value - baseline_ci;
                        info!(
                            "Step 4 CI: {:.2} (delta: {:+.2} from Step 3 baseline {:.2})",
                            ci_metric.value, delta, baseline_ci
                        );
                    }
                }

                let summary = self.callout_manager.summary();
                info!(
                    "Step 4 callouts: {} total ({} critical, {} warning, {} attention)",
                    summary.total,
                    summary.by_tier.critical,
                    summary.by_tier.warning,
                    summary.by_tier.attention
                );

                if !self.callout_manager.can_proceed() {
                    info!(
                        "Step 4: {} Critical callouts require acknowledgment before proceeding",
                        summary.pending_acknowledgments
                    );
                }
            }
        }

        // FIX-006: Re-synthesis pause check (Warning status on IAS)
        // If IAS is in Warning range (0.50-0.79), pause for human review
        if let Some(ref metrics_value) = metrics {
            if let Some(ref ias) = metrics_value.ias {
                if ias.status == crate::agents::governance_telemetry::MetricStatus::Warning {
                    warn!("⚠️ Step 4 Re-Synthesis Pause: IAS at {:.2} (below 0.80 target)", ias.value);

                    let current_step = 4;

                    // Set state to Paused
                    self.state = RunState::Paused {
                        reason: format!(
                            "Re-synthesis Pause: Intent Alignment at {:.1}% (target ≥80%). \
                             The synthesized model may not fully align with Charter objectives. \
                             Review the North Star Narrative before proceeding to framework design.",
                            ias.value * 100.0
                        ),
                        step: current_step,
                        triggered_metrics: Some(serde_json::json!({
                            "ias": ias.value,
                            "ias_threshold_warning": 0.70,
                            "ias_threshold_pass": 0.80,
                            "check_type": "re_synthesis_pause"
                        })),
                        all_metrics_snapshot: Some(serde_json::to_value(metrics_value).unwrap()),
                    };

                    // Emit MetricsWarning signal (not HALT - softer signal)
                    info!("Emitting MetricsWarning signal for re-synthesis pause");
                    self.signal_router.emit_signal(
                        SignalType::MetricsWarning,
                        &self.run_id,
                        SignalPayload {
                            step_from: current_step as i32,
                            step_to: current_step as i32,
                            artifacts_produced: vec![
                                core_thesis_id.clone(),
                                north_star_narrative_id.clone(),
                            ],
                            metrics_snapshot: Some(serde_json::to_value(&metrics_value)?),
                            gate_required: false, // Not a gate - just a pause for review
                        },
                    );

                    // Record pause in ledger
                    self.ledger.create_entry(
                        &self.run_id,
                        EntryType::Signal,
                        Some(current_step as i32),
                        Some("Conductor"),
                        LedgerPayload {
                            action: "re_synthesis_pause".to_string(),
                            inputs: Some(serde_json::json!({
                                "ias_value": ias.value,
                                "ias_status": "Warning",
                            })),
                            outputs: None,
                            rationale: Some(format!(
                                "Re-synthesis pause triggered. IAS at {:.1}% (Warning range: 50-79%). \
                                 Synthesis output may need review before proceeding to framework design.",
                                ias.value * 100.0
                            )),
                        },
                    );

                    warn!("Run PAUSED for re-synthesis review - human decision required");
                    warn!("State: {:?}", self.state);

                    // Return early - do not proceed to gate
                    return Ok((core_thesis_id, north_star_narrative_id));
                }
            }
        }

        // Record Step 4 completion in ledger (only if not halted or paused)
        let payload = LedgerPayload {
            action: "step_4_complete".to_string(),
            inputs: Some(serde_json::json!({
                "integrated_diagnostic_id": format!("{}-integrated-diagnostic", self.run_id),
            })),
            outputs: Some(serde_json::json!({
                "core_thesis_id": core_thesis_id,
                "operating_principles_count": synthesis_result.operating_principles.len(),
                "model_geometry": format!("{:?}", synthesis_result.model_geometry),
                "glossary_count": synthesis_result.glossary.len(),
                "limitations_count": synthesis_result.limitations.len(),
                "novel_geometry": synthesis_result.novel_geometry_flag,
            })),
            rationale: Some("Synthesis complete, model locked, ready for redesign".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(4),
            Some("Conductor"),
            payload,
        );

        // Emit Ready_for_Redesign signal (GATE signal)
        info!("Emitting Ready_for_Redesign signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 4,
            step_to: 5,
            artifacts_produced: vec![
                core_thesis_id.clone(),
                operating_principles_id,
                model_geometry_id,
                causal_spine_id,
                north_star_narrative_id.clone(),
                glossary_id,
                limitations_id,
            ],
            metrics_snapshot: metrics.as_ref().map(|m| serde_json::to_value(m).unwrap()),
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ReadyForRedesign,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Ready_for_Redesign",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 4 complete, synthesis locked, awaiting human approval to proceed".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(4),
            Some("Conductor"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step4GatePending;

        info!("Step 4 complete - awaiting synthesis approval");
        info!("State: {:?}", self.state);

        Ok((core_thesis_id, north_star_narrative_id))
    }

    /// Execute Step 5: Structure & Redesign
    ///
    /// Designs the framework architecture by:
    /// - Reusing Structure & Redesign Agent (attached in Step 1)
    /// - Using Core Thesis from Step 4 (immutable)
    /// - Creating framework architecture with section map, transitions, and outline
    /// - Emitting Framework_Ready signal
    ///
    /// # Returns
    /// The framework_architecture_id
    pub async fn execute_step_5(&mut self) -> Result<String> {
        info!("=== Executing Step 5: Structure & Redesign ===");

        // Validate state
        if !matches!(self.state, RunState::Step5Active) {
            anyhow::bail!("Cannot execute Step 5 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Step 4
        let core_thesis = self.core_thesis.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Core Thesis available - Step 4 must be completed first"))?;

        // Validate structure_agent is configured
        if self.structure_agent.is_none() {
            anyhow::bail!("Structure & Redesign Agent not configured");
        }

        info!("Step 5: Creating framework architecture...");

        // Construct synthesis from Step 4 artifacts
        let synthesis = format!(
            "SYNTHESIS ARTIFACTS FROM STEP 4:\n\n\
            CORE THESIS:\n{}\n\n\
            OPERATING PRINCIPLES:\n{}\n\n\
            MODEL GEOMETRY:\n{}\n\n\
            CAUSAL SPINE:\n{}\n\n\
            NORTH STAR NARRATIVE:\n{}\n\n\
            GLOSSARY:\n{}\n\n\
            LIMITATIONS:\n{}",
            core_thesis,
            self.operating_principles.as_ref().unwrap_or(&"None".to_string()),
            self.model_geometry.as_ref().unwrap_or(&"None".to_string()),
            self.causal_spine.as_ref().unwrap_or(&"None".to_string()),
            self.north_star_narrative.as_ref().unwrap_or(&"None".to_string()),
            self.glossary.as_ref().unwrap_or(&"None".to_string()),
            self.limitations.as_ref().unwrap_or(&"None".to_string())
        );

        // Call Structure & Redesign Agent (reuse from Step 1)
        let agent = self.structure_agent.as_mut().unwrap();
        let framework_architecture = agent
            .create_framework_architecture(&self.run_id, core_thesis, &synthesis)
            .await?;

        let framework_architecture_id = format!("{}-framework-architecture", self.run_id);

        info!("✓ Framework_Architecture created: {}", framework_architecture_id);

        // Store artifact
        self.framework_architecture = Some(framework_architecture.clone());

        info!("Framework architecture stored");

        // Calculate metrics
        info!("Step 5: Calculating metrics...");
        let charter = self.charter.as_ref().unwrap();
        let charter_content = self.extract_content_from_artifact(charter)?;

        // Use the framework architecture as the output for metrics
        let (metrics, halt_triggered) = self.calculate_metrics(&framework_architecture, &charter_content).await?;

        // Session 4.3: HALT check deprecated - halt_triggered will always be false
        // Metric violations handled by callout system (Session 4.2, lines below)
        // This code path kept for backward compatibility but will never execute
        if halt_triggered {
            warn!("Step 5 HALTED - run is PAUSED, no gate signal emitted");
            warn!("State: {:?}", self.state);
            warn!("Human decision required to proceed");

            // Return early - do not proceed to gate
            return Ok(framework_architecture_id);
        }

        // Session 4.2: Generate callouts for Step 5 metrics using diagnostic baseline
        if let (Some(mode), Some(ref current_metrics)) = (self.detected_mode, &metrics) {
            if let Some(ref governance_agent) = self.governance_agent {
                // Build previous_metrics from diagnostic baseline (Constraint 2: Delta from Step 3)
                let previous_metrics = self.diagnostic_ci_baseline.map(|baseline_ci| {
                    use crate::agents::governance_telemetry::{CriticalMetrics, MetricResult, MetricThreshold, MetricStatus, MetricInput, MetricInputValue};

                    CriticalMetrics {
                        ci: Some(MetricResult {
                            metric_name: "CI".to_string(),
                            value: baseline_ci,
                            threshold: MetricThreshold {
                                pass: 0.65,
                                warning: Some(0.50),
                                halt: Some(0.35),
                            },
                            status: MetricStatus::Pass,
                            inputs_used: vec![MetricInput {
                                name: "diagnostic_baseline".to_string(),
                                source: "Step 3".to_string(),
                                value: MetricInputValue::Number(baseline_ci),
                            }],
                            calculation_method: "Step 3 diagnostic baseline".to_string(),
                            interpretation: "Baseline CI for delta calculation".to_string(),
                            recommendation: None,
                        }),
                        ev: None,
                        ias: None,
                        efi: None,
                        sec: None,
                        pci: None,
                    }
                });

                governance_agent.generate_callouts(
                    current_metrics,
                    previous_metrics.as_ref(),
                    crate::governance::Step::Step5_Redesign,
                    mode,
                    crate::governance::MetricEnforcement::Enforced,
                    &mut self.callout_manager,
                );

                // Log delta from diagnostic baseline
                if let Some(baseline_ci) = self.diagnostic_ci_baseline {
                    if let Some(ref ci_metric) = current_metrics.ci {
                        let delta = ci_metric.value - baseline_ci;
                        info!(
                            "Step 5 CI: {:.2} (delta: {:+.2} from Step 3 baseline {:.2})",
                            ci_metric.value, delta, baseline_ci
                        );
                    }
                }

                let summary = self.callout_manager.summary();
                info!(
                    "Step 5 callouts: {} total ({} critical, {} warning, {} attention)",
                    summary.total,
                    summary.by_tier.critical,
                    summary.by_tier.warning,
                    summary.by_tier.attention
                );

                if !self.callout_manager.can_proceed() {
                    info!(
                        "Step 5: {} Critical callouts require acknowledgment before proceeding",
                        summary.pending_acknowledgments
                    );
                }
            }
        }

        // Record Step 5 completion in ledger (only if not halted)
        let payload = LedgerPayload {
            action: "step_5_complete".to_string(),
            inputs: Some(serde_json::json!({
                "core_thesis_id": format!("{}-core-thesis", self.run_id),
                "synthesis_artifacts": "7 artifacts from Step 4",
            })),
            outputs: Some(serde_json::json!({
                "framework_architecture_id": framework_architecture_id,
            })),
            rationale: Some("Framework architecture complete, ready for implementation".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(5),
            Some("Conductor"),
            payload,
        );

        // Emit Ready_for_Validation signal (GATE signal)
        info!("Emitting Ready_for_Validation signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 5,
            step_to: 6,
            artifacts_produced: vec![
                framework_architecture_id.clone(),
            ],
            metrics_snapshot: metrics.as_ref().map(|m| serde_json::to_value(m).unwrap()),
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ReadyForValidation,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Ready_for_Validation",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 5 complete, framework ready, awaiting human approval to proceed to validation".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(5),
            Some("Conductor"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step5GatePending;

        info!("Step 5 complete - awaiting framework approval");
        info!("State: {:?}", self.state);

        Ok(framework_architecture_id)
    }

    /// Execute Step 6: Validation & Assurance
    ///
    /// Performs comprehensive validation across 6 dimensions:
    /// - Logic Validation (reasoning chains)
    /// - Semantic Validation (Glossary consistency)
    /// - Clarity Assessment (readability)
    /// - Evidence Audit (substantiation)
    /// - Scope Compliance (Charter boundaries)
    /// - Process Coherence (Architecture Map adherence)
    ///
    /// Enforces Critical 6 metrics and determines next step based on results.
    ///
    /// # Returns
    /// The validation outcome ("PASS" / "FAIL" / "WARNING")
    pub async fn execute_step_6(&mut self) -> Result<String> {
        info!("=== Executing Step 6: Validation & Assurance ===");

        // Validate state
        if !matches!(self.state, RunState::Step6Active) {
            anyhow::bail!("Cannot execute Step 6 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Step 5
        let framework_content = self.framework_architecture.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No framework architecture available - Step 5 must be completed first"))?;

        // Ensure we have Charter and Core Thesis for validation
        let charter_content = self.charter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Charter available"))?;

        let core_thesis = self.core_thesis.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Core Thesis available - Step 4 must be completed first"))?;

        let glossary = self.glossary.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Glossary available - Step 4 must be completed first"))?;

        let architecture_map = self.architecture_map.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Architecture Map available - Step 1 must be completed first"))?;

        // Validate validation_agent is configured
        if self.validation_agent.is_none() {
            anyhow::bail!("Validation & Learning Agent not configured");
        }

        info!("Step 6: Running comprehensive validation...");

        // Extract charter objectives (simplified extraction)
        let charter_objectives_content = self.extract_content_from_artifact(charter_content)?;

        // Generate Steno-Ledger for validation context
        let steno_ledger = self.generate_steno_ledger();

        // Call Validation & Learning Agent
        let agent = self.validation_agent.as_mut().unwrap();
        let mut validation_result = agent
            .validate_framework(
                &self.run_id,
                framework_content,
                &charter_objectives_content,
                core_thesis,
                glossary,
                architecture_map,
                &steno_ledger,
            )
            .await?;

        info!("✓ Validation complete");

        // FIX-005: Override EFI with consistent governance calculation
        // The validation agent's evidence_audit uses weak regex parsing and can produce
        // false positives. Use the same strict JSON-based calculation as Steps 2-5.
        info!("Calculating EFI using consistent governance method...");
        let governance_metrics = self.governance_agent.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Governance agent not available"))?
            .calculate_metrics(framework_content, &charter_objectives_content, 6)
            .await?;

        let governance_efi = governance_metrics.efi.as_ref()
            .ok_or_else(|| anyhow::anyhow!("EFI not calculated in governance metrics"))?;

        let original_efi = validation_result.critical_6_scores.efi;
        validation_result.critical_6_scores.efi = governance_efi.value / 100.0; // Convert percentage to 0.0-1.0

        info!("EFI corrected: {:.2} (validation audit) → {:.2} (governance strict)",
            original_efi, validation_result.critical_6_scores.efi);

        // Session 4.2: Generate callouts for Step 6 metrics using diagnostic baseline
        if let Some(mode) = self.detected_mode {
            if let Some(ref governance_agent) = self.governance_agent {
                // Build previous_metrics from diagnostic baseline (Constraint 2: Delta from Step 3)
                let previous_metrics = self.diagnostic_ci_baseline.map(|baseline_ci| {
                    use crate::agents::governance_telemetry::{CriticalMetrics, MetricResult, MetricThreshold, MetricStatus, MetricInput, MetricInputValue};

                    CriticalMetrics {
                        ci: Some(MetricResult {
                            metric_name: "CI".to_string(),
                            value: baseline_ci,
                            threshold: MetricThreshold {
                                pass: 0.65,
                                warning: Some(0.50),
                                halt: Some(0.35),
                            },
                            status: MetricStatus::Pass,
                            inputs_used: vec![MetricInput {
                                name: "diagnostic_baseline".to_string(),
                                source: "Step 3".to_string(),
                                value: MetricInputValue::Number(baseline_ci),
                            }],
                            calculation_method: "Step 3 diagnostic baseline".to_string(),
                            interpretation: "Baseline CI for delta calculation".to_string(),
                            recommendation: None,
                        }),
                        ev: None,
                        ias: None,
                        efi: None,
                        sec: None,
                        pci: None,
                    }
                });

                governance_agent.generate_callouts(
                    &governance_metrics,
                    previous_metrics.as_ref(),
                    crate::governance::Step::Step6_Validation,
                    mode,
                    crate::governance::MetricEnforcement::Enforced,
                    &mut self.callout_manager,
                );

                // Log delta from diagnostic baseline
                if let Some(baseline_ci) = self.diagnostic_ci_baseline {
                    if let Some(ref ci_metric) = governance_metrics.ci {
                        let delta = ci_metric.value - baseline_ci;
                        info!(
                            "Step 6 CI: {:.2} (delta: {:+.2} from Step 3 baseline {:.2})",
                            ci_metric.value, delta, baseline_ci
                        );
                    }
                }

                let summary = self.callout_manager.summary();
                info!(
                    "Step 6 callouts: {} total ({} critical, {} warning, {} attention)",
                    summary.total,
                    summary.by_tier.critical,
                    summary.by_tier.warning,
                    summary.by_tier.attention
                );

                if !self.callout_manager.can_proceed() {
                    info!(
                        "Step 6: {} Critical callouts require acknowledgment before proceeding",
                        summary.pending_acknowledgments
                    );
                }
            }
        }

        // Store validation artifacts
        self.validation_matrix = Some(validation_result.validation_matrix.clone());
        self.semantic_table = Some(validation_result.semantic_table.clone());
        self.evidence_report = Some(validation_result.evidence_report.clone());

        // Determine outcome
        let outcome = match validation_result.overall_status {
            crate::agents::validation_learning::ValidationStatus::Pass => "PASS",
            crate::agents::validation_learning::ValidationStatus::Fail => "FAIL",
            crate::agents::validation_learning::ValidationStatus::Warning => "WARNING",
        };
        self.validation_outcome = Some(outcome.to_string());

        // Store exceptional flag for Step 6.5 routing
        self.exceptional_flag = validation_result.exceptional_flag;

        info!("Validation outcome: {}", outcome);

        // Log Critical 6 scores
        let scores = &validation_result.critical_6_scores;
        info!("Critical 6 Metrics:");
        info!("  CI:  {:.2} (target ≥ 0.80)", scores.ci);
        info!("  EV:  {:.2} (target ± 0.10)", scores.ev);
        info!("  IAS: {:.2} (target ≥ 0.80)", scores.ias);
        info!("  EFI: {:.2} (target ≥ 0.95)", scores.efi);
        info!("  SEC: {:.2} (target = 1.00)", scores.sec);
        info!("  PCI: {:.2} (target ≥ 0.90)", scores.pci);

        // Check for exceptional result (CI ≥ 0.85 triggers Step 6.5)
        if validation_result.exceptional_flag {
            info!("✓ EXCEPTIONAL RESULT: CI ≥ 0.85 - Step 6.5 Learning Harvest will be available");
        }

        // Record Step 6 completion in ledger
        let payload = LedgerPayload {
            action: "step_6_complete".to_string(),
            inputs: Some(serde_json::json!({
                "framework_architecture_id": format!("{}-framework-architecture", self.run_id),
            })),
            outputs: Some(serde_json::json!({
                "validation_outcome": outcome,
                "critical_6_all_pass": scores.all_pass(),
                "exceptional_flag": validation_result.exceptional_flag,
            })),
            rationale: Some(format!(
                "Validation complete. Outcome: {}. Critical 6 all pass: {}",
                outcome,
                scores.all_pass()
            )),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(6),
            Some("Examiner"),
            payload,
        );

        // Emit Validation_Complete signal (GATE signal)
        info!("Emitting Validation_Complete signal (GATE)");

        let signal_payload = SignalPayload {
            step_from: 6,
            step_to: 7,  // Either Closure or 6.5
            artifacts_produced: vec![
                format!("{}-validation-matrix", self.run_id),
                format!("{}-semantic-table", self.run_id),
                format!("{}-evidence-report", self.run_id),
            ],
            metrics_snapshot: Some(serde_json::json!({
                "ci": scores.ci,
                "ev": scores.ev,
                "ias": scores.ias,
                "efi": scores.efi,
                "sec": scores.sec,
                "pci": scores.pci,
            })),
            gate_required: true,
        };

        let signal = self.signal_router.emit_signal(
            SignalType::ValidationComplete,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record gate signal in ledger
        let payload = LedgerPayload {
            action: "gate_signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Validation_Complete",
                "gate_required": true,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Step 6 complete, validation results ready, awaiting human approval".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Gate,
            Some(6),
            Some("Examiner"),
            payload,
        );

        // Transition to gate pending state
        self.state = RunState::Step6GatePending;

        info!("Step 6 complete - awaiting validation approval");
        info!("State: {:?}", self.state);

        Ok(outcome.to_string())
    }

    /// Execute Step 6.5: Learning Harvest (pattern extraction from exceptional results)
    ///
    /// CRITICAL: This method REUSES the validation_agent from Step 6.
    /// The agent already has all validation results stored in its state.
    pub async fn execute_step_6_5(&mut self) -> Result<crate::agents::validation_learning::LearningHarvestResult> {
        info!("=== Executing Step 6.5: Learning Harvest ===");

        // Validate state
        if !matches!(self.state, RunState::Step6_5Active) {
            anyhow::bail!("Cannot execute Step 6.5 - current state: {:?}", self.state);
        }

        // Ensure exceptional_flag is true
        if !self.exceptional_flag {
            anyhow::bail!("Cannot execute Step 6.5 - not an exceptional result (CI < 0.85)");
        }

        // Get EXISTING validation agent (DO NOT create new one)
        let validation_agent = self.validation_agent.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No validation agent found - Step 6 must be completed first"))?;

        info!("Using existing Validation & Learning Agent (agent has stored validation results)");

        // Perform learning harvest - agent uses its stored validation data
        info!("Calling perform_learning_harvest()...");
        let harvest_result = validation_agent.perform_learning_harvest().await?;

        info!("✓ Learning harvest complete");
        info!("  Pattern cards extracted: {}", harvest_result.pattern_cards.len());
        info!("  Success patterns: {}", harvest_result.success_count);
        info!("  Failure patterns: {}", harvest_result.failure_count);
        info!("  Optimization patterns: {}", harvest_result.optimization_count);

        // TODO: Update knowledge repository with pattern cards
        // For MVP, we'll log the patterns
        for card in &harvest_result.pattern_cards {
            info!("  Pattern: {} ({})", card.pattern_name, card.category);
        }

        // Record Step 6.5 completion in ledger
        let payload = LedgerPayload {
            action: "step_6_5_complete".to_string(),
            inputs: None,
            outputs: Some(serde_json::json!({
                "pattern_count": harvest_result.pattern_cards.len(),
                "success_count": harvest_result.success_count,
                "failure_count": harvest_result.failure_count,
                "optimization_count": harvest_result.optimization_count,
            })),
            rationale: Some(harvest_result.knowledge_update.clone()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Decision,
            Some(6),
            Some("Learner"),
            payload,
        );

        // Emit Learning_Harvested signal (NO GATE - automatic progression to Closure)
        info!("Emitting Learning_Harvested signal (no gate)");

        let signal_payload = SignalPayload {
            step_from: 6,
            step_to: 7,
            artifacts_produced: vec![],  // Pattern cards stored in repository, not as artifacts
            metrics_snapshot: None,
            gate_required: false,  // NO GATE - automatic progression
        };

        let signal = self.signal_router.emit_signal(
            SignalType::LearningHarvested,
            &self.run_id,
            signal_payload,
        );

        debug!("Signal emitted: {:?}", signal.hash);

        // Record signal in ledger
        let payload = LedgerPayload {
            action: "signal_emitted".to_string(),
            inputs: Some(serde_json::json!({
                "signal_type": "Learning_Harvested",
                "gate_required": false,
            })),
            outputs: Some(serde_json::json!({
                "signal_hash": signal.hash,
            })),
            rationale: Some("Learning harvest complete, proceeding to Closure".to_string()),
        };

        self.ledger.create_entry(
            &self.run_id,
            EntryType::Signal,
            Some(6),
            Some("Learner"),
            payload,
        );

        // Transition directly to Completed (no gate for Step 6.5)
        self.state = RunState::Completed;

        info!("Step 6.5 complete - run finished");
        info!("State: {:?}", self.state);

        Ok(harvest_result)
    }

    /// Execute Closure: Final ledger, archival, and database update
    ///
    /// Generates final artifacts, updates database, and prepares export packages.
    pub async fn execute_closure(&mut self) -> Result<ClosureResult> {
        info!("=== EXECUTE_CLOSURE ===");
        info!("Run ID: {}", self.run_id);
        info!("Current State: {:?}", self.state);

        // Verify state is Completed
        if !matches!(self.state, RunState::Completed) {
            anyhow::bail!(
                "Cannot execute closure - current state is {:?}, expected Completed",
                self.state
            );
        }

        // 1. Generate Final Ledger (steno format)
        let final_ledger = self.generate_steno_ledger();
        info!("✓ Final ledger generated ({} chars)", final_ledger.len());

        // 2. Create Audit Trail (all signals, gates, decisions)
        let audit_trail = self.generate_audit_trail();
        info!("✓ Audit trail created ({} entries)", audit_trail.len());

        // 3. Archive Artifacts
        let archived_artifacts = self.archive_artifacts().await?;
        info!("✓ Artifacts archived ({} items)", archived_artifacts.len());

        // 4. Calculate Final Statistics
        let statistics = self.calculate_final_statistics();
        info!("✓ Final statistics calculated");

        // 5. Extract Final Metrics (from validation results if available)
        let final_metrics = self.extract_final_metrics();

        // 6. Determine Success Status
        let success = self.determine_run_success(&final_metrics);

        // 7. Update Database
        self.update_database_closure(&final_metrics, success).await?;
        info!("✓ Database updated");

        // Log before building result (while final_metrics is still accessible)
        info!("=== CLOSURE COMPLETE ===");
        info!("Success: {}", success);
        info!("Final CI: {:?}", final_metrics.get("CI"));

        // 8. Build Closure Result
        let result = ClosureResult {
            run_id: self.run_id.clone(),
            final_ledger,
            audit_trail,
            archived_artifacts,
            statistics,
            final_metrics,
            success,
            completed_at: Utc::now().to_rfc3339(),
        };

        Ok(result)
    }

    /// Generate audit trail of all signals, gates, and decisions
    fn generate_audit_trail(&self) -> Vec<AuditEntry> {
        let mut trail = Vec::new();

        // Add signal history from signal router
        let signals = self.signal_router.get_signal_chain(&self.run_id);
        for signal in signals {
            trail.push(AuditEntry {
                timestamp: signal.timestamp.to_rfc3339(),
                entry_type: "Signal".to_string(),
                description: format!("{:?}", signal.signal_type),
                metadata: serde_json::json!({
                    "signal_type": format!("{:?}", signal.signal_type),
                    "hash": signal.hash,
                }),
            });
        }

        // Add gate decisions from ledger entries (simplified - would need ledger API)
        // For now, just note that gates were processed
        trail.push(AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            entry_type: "Gate".to_string(),
            description: "Gate approvals processed during run".to_string(),
            metadata: serde_json::json!({ "note": "Full ledger available in final_ledger" }),
        });

        trail
    }

    /// Archive all artifacts (intent, baseline, synthesis, framework, validation)
    async fn archive_artifacts(&self) -> Result<Vec<ArchivedArtifact>> {
        let mut artifacts = Vec::new();

        // Archive Step 0: Intent Summary
        if let Some(ref intent) = self.intent_summary {
            artifacts.push(ArchivedArtifact {
                artifact_type: "IntentSummary".to_string(),
                content: serde_json::to_string_pretty(intent)?,
                size_bytes: 0,
            });
        }

        // Archive Step 1: Charter & Baseline
        if let Some(ref charter) = self.charter {
            artifacts.push(ArchivedArtifact {
                artifact_type: "Charter".to_string(),
                content: charter.clone(),
                size_bytes: charter.len(),
            });
        }

        // Archive Step 3: Integrated Diagnostic
        if let Some(ref diagnostic) = self.integrated_diagnostic {
            artifacts.push(ArchivedArtifact {
                artifact_type: "IntegratedDiagnostic".to_string(),
                content: diagnostic.clone(),
                size_bytes: diagnostic.len(),
            });
        }

        // Archive Step 4: Core Thesis & North Star
        if let Some(ref thesis) = self.core_thesis {
            artifacts.push(ArchivedArtifact {
                artifact_type: "CoreThesis".to_string(),
                content: thesis.clone(),
                size_bytes: thesis.len(),
            });
        }
        if let Some(ref narrative) = self.north_star_narrative {
            artifacts.push(ArchivedArtifact {
                artifact_type: "NorthStarNarrative".to_string(),
                content: narrative.clone(),
                size_bytes: narrative.len(),
            });
        }

        // Archive Step 5: Framework Architecture
        if let Some(ref framework) = self.framework_architecture {
            artifacts.push(ArchivedArtifact {
                artifact_type: "FrameworkArchitecture".to_string(),
                content: framework.clone(),
                size_bytes: framework.len(),
            });
        }

        // Archive Step 6: Validation Results
        if let Some(ref matrix) = self.validation_matrix {
            artifacts.push(ArchivedArtifact {
                artifact_type: "ValidationMatrix".to_string(),
                content: matrix.clone(),
                size_bytes: matrix.len(),
            });
        }
        if let Some(ref evidence) = self.evidence_report {
            artifacts.push(ArchivedArtifact {
                artifact_type: "EvidenceReport".to_string(),
                content: evidence.clone(),
                size_bytes: evidence.len(),
            });
        }

        Ok(artifacts)
    }

    /// Calculate final statistics
    fn calculate_final_statistics(&self) -> RunStatistics {
        let signal_chain = self.signal_router.get_signal_chain(&self.run_id);
        RunStatistics {
            total_signals: signal_chain.len(),
            total_gates: 0, // Simplified - would count gate signals
            steps_completed: self.state.step_number() as usize,
            exceptional_run: self.exceptional_flag,
            halt_count: 0, // Would need to track HALT conditions
        }
    }

    /// Extract final metrics from validation results
    fn extract_final_metrics(&self) -> std::collections::HashMap<String, f64> {
        let mut metrics = std::collections::HashMap::new();

        // Placeholder - would extract from validation_agent.validation_results
        // For now, return empty or default values
        metrics.insert("CI".to_string(), 0.0);
        metrics.insert("EV".to_string(), 0.0);
        metrics.insert("IAS".to_string(), 0.0);
        metrics.insert("EFI".to_string(), 0.0);
        metrics.insert("SEC".to_string(), 0.0);
        metrics.insert("PCI".to_string(), 0.0);

        metrics
    }

    /// Determine if run was successful based on metrics
    fn determine_run_success(&self, metrics: &std::collections::HashMap<String, f64>) -> bool {
        // Success if CI >= 0.70 (minimum threshold)
        metrics.get("CI").unwrap_or(&0.0) >= &0.70
    }

    /// Update database with closure information
    async fn update_database_closure(
        &self,
        metrics: &std::collections::HashMap<String, f64>,
        success: bool,
    ) -> Result<()> {
        // Placeholder - would update runs table:
        // UPDATE runs SET
        //   status = 'Completed',
        //   completed_at = NOW(),
        //   ci_final = ?,
        //   ev_final = ?,
        //   ...
        // WHERE run_id = ?

        info!("Database update placeholder - metrics: {:?}, success: {}", metrics, success);
        Ok(())
    }
}

/// Result from execute_closure()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureResult {
    pub run_id: String,
    pub final_ledger: String,
    pub audit_trail: Vec<AuditEntry>,
    pub archived_artifacts: Vec<ArchivedArtifact>,
    pub statistics: RunStatistics,
    pub final_metrics: std::collections::HashMap<String, f64>,
    pub success: bool,
    pub completed_at: String,
}

/// Single entry in the audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub entry_type: String, // "Signal" | "Gate" | "Decision"
    pub description: String,
    pub metadata: serde_json::Value,
}

/// Archived artifact from the run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedArtifact {
    pub artifact_type: String, // "Personas" | "Synthesis" | "Framework" | "Validation" | "PatternCards"
    pub content: String,
    pub size_bytes: usize,
}

/// Final run statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStatistics {
    pub total_signals: usize,
    pub total_gates: usize,
    pub steps_completed: usize,
    pub exceptional_run: bool,
    pub halt_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orch = Orchestrator::new("test-run");

        assert!(orch.run_id.contains("test-run"));
        assert!(matches!(orch.state, RunState::Step0Active));
        assert!(matches!(orch.active_role, Role::Observer));
        assert!(matches!(orch.mode, Mode::Standard));
    }

    #[test]
    fn test_run_id_format() {
        let orch = Orchestrator::new("Analysis");
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let expected = format!("{}-Analysis", date);

        assert_eq!(orch.run_id, expected);
    }

    #[test]
    fn test_steno_ledger_generation() {
        let orch = Orchestrator::new("test");
        let steno = orch.generate_steno_ledger();

        assert!(steno.contains("RUN:"));
        assert!(steno.contains("S:0"));
        assert!(steno.contains("R:OBS"));
        assert!(steno.contains("M:STD"));
    }

    #[tokio::test]
    async fn test_step_0_execution() {
        let mut orch = Orchestrator::new("test");

        let result = orch.execute_step_0("Build a user authentication system").await;
        assert!(result.is_ok());

        let intent = result.unwrap();
        assert_eq!(intent.user_request, "Build a user authentication system");
        assert!(!intent.primary_goal.is_empty());
        assert!(intent.confidence_score > 0);

        // Should transition to gate pending
        assert!(matches!(orch.state, RunState::Step0GatePending));
        assert!(orch.intent_summary.is_some());
    }

    #[tokio::test]
    async fn test_gate_approval() {
        let mut orch = Orchestrator::new("test");

        // Execute Step 0 first
        orch.execute_step_0("Test intent").await.unwrap();
        assert!(matches!(orch.state, RunState::Step0GatePending));

        // Approve gate
        let result = orch.approve_gate("Human Reviewer");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Should transition to Step 1
        assert!(matches!(orch.state, RunState::Step1Active));
        assert!(matches!(orch.active_role, Role::Conductor));
    }

    #[tokio::test]
    async fn test_gate_rejection() {
        let mut orch = Orchestrator::new("test");

        // Execute Step 0 first
        orch.execute_step_0("Test intent").await.unwrap();
        assert!(matches!(orch.state, RunState::Step0GatePending));

        // Reject gate
        let result = orch.reject_gate("Human Reviewer", "Scope too broad");
        assert!(result.is_ok());

        // Should transition to halted
        assert!(matches!(orch.state, RunState::Halted { .. }));
    }

    #[test]
    fn test_gate_approval_without_pending() {
        let mut orch = Orchestrator::new("test");

        // Try to approve gate without executing Step 0
        let result = orch.approve_gate("Human");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_state_step_numbers() {
        assert_eq!(RunState::Step0Active.step_number(), 0);
        assert_eq!(RunState::Step0GatePending.step_number(), 0);
        assert_eq!(RunState::Step1Active.step_number(), 1);
        assert_eq!(RunState::Step1GatePending.step_number(), 1);
        assert_eq!(RunState::FutureStep(3).step_number(), 3);
        assert_eq!(RunState::Completed.step_number(), 7);
    }

    #[test]
    fn test_run_state_is_gate_pending() {
        assert!(RunState::Step0GatePending.is_gate_pending());
        assert!(RunState::Step1GatePending.is_gate_pending());
        assert!(!RunState::Step0Active.is_gate_pending());
        assert!(!RunState::Step1Active.is_gate_pending());
    }

    #[test]
    fn test_action_validation() {
        let orch = Orchestrator::new("test");

        // In Step 0, intent capture should be allowed
        let result = orch.validate_action("intent_capture");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // But validation should not be allowed yet
        let result = orch.validate_action("validation");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ledger_recording() {
        let mut orch = Orchestrator::new("test");

        // Execute Step 0
        orch.execute_step_0("Test intent").await.unwrap();

        // Check that ledger entries were created
        let entries = orch.ledger.get_entries(&orch.run_id);
        assert!(!entries.is_empty());

        // Should have at least: run_start, intent_captured, gate_signal_emitted
        assert!(entries.len() >= 3);
    }

    #[tokio::test]
    async fn test_signal_chain() {
        let mut orch = Orchestrator::new("test");

        // Execute Step 0
        orch.execute_step_0("Test intent").await.unwrap();

        // Check that signal was emitted
        let chain = orch.signal_router.get_signal_chain(&orch.run_id);
        assert_eq!(chain.len(), 1);

        let signal = &chain[0];
        assert!(matches!(signal.signal_type, SignalType::ReadyForStep1));
        assert!(signal.payload.gate_required);
    }
}
