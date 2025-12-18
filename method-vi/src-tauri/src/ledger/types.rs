use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of ledger entries in Method-VI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    /// Gate entry - requires human acknowledgment
    Gate,

    /// Intervention - corrective action taken
    Intervention,

    /// Signal - state transition or notification
    Signal,

    /// Decision - human or automated decision point
    Decision,

    /// Metric snapshot - Critical 6 metrics captured
    MetricSnapshot,
}

/// Current state of the ledger/run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerState {
    /// Step 0 active - intent capture and pattern query allowed
    Step0Active,

    /// Baseline frozen - analysis and synthesis allowed
    BaselineFrozen,

    /// Gate pending - awaiting human approval
    GatePending,

    /// HALT active - only human decisions allowed
    HaltActive,

    /// Normal operation - no special restrictions
    Normal,
}

/// HALT/PAUSE status based on metrics and conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaltStatus {
    /// Continue normal operation
    Continue,

    /// Pause for review (CI 0.70-0.80 or similar warning conditions)
    PauseForReview { reason: String },

    /// Immediate halt required
    HaltImmediate { reason: String },
}

/// Metrics snapshot for threshold checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Coherence Index (0.0 - 1.0)
    pub ci: f64,

    /// Expansion Velocity (-100% to +∞)
    pub ev: f64,

    /// Structural Envelope Compliance (true/false)
    pub sec: bool,

    /// Optional: other Critical 6 metrics can be added here
    pub timestamp: DateTime<Utc>,
}

/// Payload structure for ledger entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerPayload {
    /// What action was taken
    pub action: String,

    /// What was considered (inputs)
    pub inputs: Option<serde_json::Value>,

    /// What was produced (outputs)
    pub outputs: Option<serde_json::Value>,

    /// Why this action was taken (for explainability)
    pub rationale: Option<String>,
}

/// Ledger entry representing a single event in the run
///
/// Each entry forms a link in the hash chain, ensuring
/// immutability and tamper detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Auto-increment ID (None for new entries)
    pub id: Option<i64>,

    /// Run ID this entry belongs to
    pub run_id: String,

    /// Type of entry
    pub entry_type: EntryType,

    /// Step number (0-6.5 in Method-VI)
    pub step: Option<i32>,

    /// Active governance role
    pub role: Option<String>,

    /// Entry payload (action, inputs, outputs, rationale)
    pub payload: LedgerPayload,

    /// Hash of previous entry (null for first entry)
    pub prior_hash: Option<String>,

    /// SHA-256 hash of this entry
    pub hash: String,

    /// When this entry was created
    pub created_at: DateTime<Utc>,
}

/// Result of action validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionValidationResult {
    /// Whether the action is allowed
    pub allowed: bool,

    /// Reason for rejection (if not allowed)
    pub reason: Option<String>,
}

impl ActionValidationResult {
    pub fn allowed() -> Self {
        ActionValidationResult {
            allowed: true,
            reason: None,
        }
    }

    pub fn rejected(reason: &str) -> Self {
        ActionValidationResult {
            allowed: false,
            reason: Some(reason.to_string()),
        }
    }
}
