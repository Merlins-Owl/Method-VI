use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};

use crate::agents::analysis_synthesis::AnalysisSynthesisAgent;
use crate::agents::governance_telemetry::{CriticalMetrics, GovernanceTelemetryAgent};
use crate::agents::scope_pattern::{IntentSummary, ScopePatternAgent};
use crate::agents::structure_redesign::StructureRedesignAgent;
use crate::context::{ContextManager, Mode, Role, RunContext, Signal as ContextSignal};
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

    /// Step 6: Future steps (not yet implemented)
    FutureStep(u8),

    /// Run completed
    Completed,

    /// Run halted
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
            RunState::FutureStep(n) => *n,
            RunState::Completed => 7,
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

    /// Scope & Pattern Agent (optional - if None, uses stub)
    scope_agent: Option<ScopePatternAgent>,

    /// Governance & Telemetry Agent (optional - if None, metrics are not calculated)
    governance_agent: Option<GovernanceTelemetryAgent>,

    /// Structure & Redesign Agent (optional - if None, uses stub)
    structure_agent: Option<StructureRedesignAgent>,

    /// Analysis & Synthesis Agent (optional - if None, uses stub)
    analysis_synthesis_agent: Option<AnalysisSynthesisAgent>,

    /// Latest calculated metrics from Governance Agent
    pub latest_metrics: Option<CriticalMetrics>,
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
            scope_agent: None,            // Will be set via with_scope_agent()
            governance_agent: None,       // Will be set via with_governance_agent()
            structure_agent: None,        // Will be set via with_structure_agent()
            analysis_synthesis_agent: None, // Will be set via with_analysis_synthesis_agent()
            latest_metrics: None,
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
            RunState::Completed => ContextSignal::Completed,
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

                // Transition to Step 6 (future implementation)
                self.state = RunState::FutureStep(6);

                info!("✓ Framework gate approved - transitioning to Step 6");
                info!("Active role: {:?}", self.active_role);

                Ok(true)
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
        let e_baseline = self.calculate_and_lock_e_baseline(&charter_content)?;
        info!("✓ E_baseline locked: {} words", e_baseline);

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
        let initial_metrics = self.governance_agent.as_ref().unwrap()
            .calculate_metrics(&charter_content, &charter_content, 2)
            .await?;

        // Store metrics
        self.latest_metrics = Some(initial_metrics.clone());

        info!("✓ Initial metrics calculated:");
        if let Some(ci) = &initial_metrics.ci {
            info!("  CI: {:.2} ({})", ci.value, match ci.status {
                crate::agents::governance_telemetry::MetricStatus::Pass => "PASS",
                crate::agents::governance_telemetry::MetricStatus::Warning => "WARNING",
                crate::agents::governance_telemetry::MetricStatus::Fail => "FAIL",
            });
        }
        if let Some(ev) = &initial_metrics.ev {
            info!("  EV: {:.1}% ({})", ev.value, match ev.status {
                crate::agents::governance_telemetry::MetricStatus::Pass => "PASS",
                crate::agents::governance_telemetry::MetricStatus::Warning => "WARNING",
                crate::agents::governance_telemetry::MetricStatus::Fail => "FAIL",
            });
        }

        // Record Step 2 completion in ledger
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

    /// Validate an action is allowed in the current state
    ///
    /// Uses the Ledger Manager to validate state transitions
    pub fn validate_action(&self, action: &str) -> Result<bool> {
        let ledger_state = match &self.state {
            RunState::Step0Active => LedgerState::Step0Active,
            RunState::Step0GatePending | RunState::Step1GatePending => LedgerState::GatePending,
            RunState::Step1Active => LedgerState::Normal,
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
    /// The calculated metrics, or None if governance agent is not available
    pub async fn calculate_metrics(
        &mut self,
        content: &str,
        charter_objectives: &str,
    ) -> Result<Option<CriticalMetrics>> {
        if let Some(ref agent) = self.governance_agent {
            info!("Calculating metrics for step {}", self.state.step_number());

            let metrics = agent
                .calculate_metrics(content, charter_objectives, self.state.step_number())
                .await?;

            // Check for HALT conditions
            if let Some(halt_reason) = agent.check_halt_conditions(&metrics) {
                warn!("HALT condition detected: {}", halt_reason);
                self.state = RunState::Halted {
                    reason: halt_reason.clone(),
                };

                // Emit HALT signal
                self.signal_router.emit_signal(
                    SignalType::Halt,
                    &self.run_id,
                    SignalPayload {
                        step_from: self.state.step_number() as i32,
                        step_to: self.state.step_number() as i32,
                        artifacts_produced: vec![],
                        metrics_snapshot: Some(serde_json::to_value(&metrics)?),
                        gate_required: false,
                    },
                );
            }
            // Check for PAUSE (warning) conditions
            else if let Some(pause_reason) = agent.check_pause_conditions(&metrics) {
                warn!("PAUSE condition detected: {}", pause_reason);

                // Emit warning signal (not a hard stop)
                self.signal_router.emit_signal(
                    SignalType::MetricsWarning,
                    &self.run_id,
                    SignalPayload {
                        step_from: self.state.step_number() as i32,
                        step_to: self.state.step_number() as i32,
                        artifacts_produced: vec![],
                        metrics_snapshot: Some(serde_json::to_value(&metrics)?),
                        gate_required: false,
                    },
                );
            }

            // Store latest metrics
            self.latest_metrics = Some(metrics.clone());

            Ok(Some(metrics))
        } else {
            debug!("Governance agent not available - skipping metrics calculation");
            Ok(None)
        }
    }

    /// Calculate and lock E_baseline (Step 1)
    ///
    /// This should be called after the Baseline Report is generated.
    pub fn calculate_and_lock_e_baseline(&mut self, baseline_content: &str) -> Result<f64> {
        if let Some(ref mut agent) = self.governance_agent {
            let baseline = agent.calculate_e_baseline(baseline_content, 1)?;
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

    /// Execute Step 3: Multi-Angle Analysis
    ///
    /// Performs six-lens analysis on the Charter by:
    /// - Applying six analytical lenses (Structural, Thematic, Logic, Evidence, Expression, Intent)
    /// - Using weighted lens sequencing based on intent category
    /// - Tracking lens efficacy for pattern learning
    /// - Creating Integrated_Diagnostic artifact
    /// - Emitting Ready_for_Synthesis signal
    ///
    /// # Returns
    /// A tuple of (integrated_diagnostic_id, lens_efficacy_report_id)
    pub async fn execute_step_3(&mut self) -> Result<(String, String)> {
        info!("=== Executing Step 3: Multi-Angle Analysis ===");

        // Validate state
        if !matches!(self.state, RunState::Step3Active) {
            anyhow::bail!("Cannot execute Step 3 - current state: {:?}", self.state);
        }

        // Ensure we have required artifacts from Step 1
        let charter = self.charter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Charter available - Step 1 must be completed first"))?;

        // Validate analysis_synthesis_agent is configured
        if self.analysis_synthesis_agent.is_none() {
            anyhow::bail!("Analysis & Synthesis Agent not configured");
        }

        // Extract Charter content
        let charter_content = self.extract_content_from_artifact(charter)?;

        // Get intent category from intent summary (clone to avoid borrow issues)
        let intent_category = self.intent_summary.as_ref()
            .map(|intent| intent.intent_category.clone())
            .unwrap_or_else(|| "Analytical".to_string()); // Default to Analytical if not available

        info!("Step 3: Performing six-lens analysis...");
        info!("  Intent category: {}", intent_category);

        // Perform six-lens analysis (this takes ownership temporarily)
        let agent = self.analysis_synthesis_agent.as_mut().unwrap();
        let (integrated_diagnostic, lens_efficacy) = agent
            .perform_six_lens_analysis(&charter_content, &intent_category)
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
        let metrics = self.calculate_metrics(&charter_content, &integrated_diagnostic).await?;

        // Record Step 3 completion in ledger
        let payload = LedgerPayload {
            action: "step_3_complete".to_string(),
            inputs: Some(serde_json::json!({
                "intent_category": intent_category,
                "charter_size": charter_content.len(),
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
        let metrics = self.calculate_metrics(&charter_content, &synthesis_result.north_star_narrative).await?;

        // Record Step 4 completion in ledger
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
        let metrics = self.calculate_metrics(&charter_content, &framework_architecture).await?;

        // Record Step 5 completion in ledger
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
