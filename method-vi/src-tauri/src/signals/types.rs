use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Signal types in Method-VI workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    // Gate signals - require human approval
    /// Ready to transition from Step 0 to Step 1
    ReadyForStep1,

    /// Baseline frozen, transitioning from Step 1 to Step 2
    BaselineFrozen,

    /// Ready for analysis, transitioning from Step 2 to Step 3
    ReadyForAnalysis,

    /// Ready for synthesis, transitioning from Step 3 to Step 4
    ReadyForSynthesis,

    /// Ready for redesign, transitioning from Step 4 to Step 5
    ReadyForRedesign,

    /// Ready for validation, transitioning from Step 5 to Step 6
    ReadyForValidation,

    /// Validation complete, Step 6 complete
    ValidationComplete,

    // Non-gate signals - no approval needed
    /// Learning harvested from completed run
    LearningHarvested,

    /// New run is ready to start
    NewRunReady,

    /// Metric update (internal, non-blocking)
    MetricUpdate,
}

impl SignalType {
    /// Convert signal type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            SignalType::ReadyForStep1 => "Ready_for_Step_1",
            SignalType::BaselineFrozen => "Baseline_Frozen",
            SignalType::ReadyForAnalysis => "Ready_for_Analysis",
            SignalType::ReadyForSynthesis => "Ready_for_Synthesis",
            SignalType::ReadyForRedesign => "Ready_for_Redesign",
            SignalType::ReadyForValidation => "Ready_for_Validation",
            SignalType::ValidationComplete => "Validation_Complete",
            SignalType::LearningHarvested => "Learning_Harvested",
            SignalType::NewRunReady => "New_Run_Ready",
            SignalType::MetricUpdate => "Metric_Update",
        }
    }
}

/// Signal payload containing transition details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPayload {
    /// Step transitioning from
    pub step_from: i32,

    /// Step transitioning to
    pub step_to: i32,

    /// List of artifact IDs produced
    pub artifacts_produced: Vec<String>,

    /// Metrics snapshot at time of signal
    pub metrics_snapshot: Option<serde_json::Value>,

    /// Whether this signal requires human gate approval
    pub gate_required: bool,
}

/// Signal representing a workflow event
///
/// Signals form a hash chain to ensure integrity and sequencing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Type of signal
    pub signal_type: SignalType,

    /// Run ID this signal belongs to
    pub run_id: String,

    /// When this signal was emitted
    pub timestamp: DateTime<Utc>,

    /// Hash of previous signal in chain (null for first signal)
    pub prior_signal_hash: Option<String>,

    /// SHA-256 hash of this signal
    pub hash: String,

    /// Signal payload
    pub payload: SignalPayload,
}
