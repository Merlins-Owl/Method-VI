use serde::{Deserialize, Serialize};

/// Governance roles in Method-VI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Observer - Observes and reports without altering
    Observer,

    /// Conductor - Orchestrates workflow
    Conductor,

    /// Auditor - Reviews for compliance
    Auditor,

    /// Patcher - Applies targeted fixes
    Patcher,

    /// Fabricator - Constructs new artifacts
    Fabricator,

    /// Examiner - Tests and validates
    Examiner,

    /// Curator - Maintains knowledge repository
    Curator,

    /// Archivist - Manages long-term storage
    Archivist,
}

/// Operational modes in Method-VI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Standard mode - normal operation
    Standard,

    /// Component mode - focused on specific component
    Component,

    /// Surgical mode - precise, minimal-scope intervention
    Surgical,
}

/// Signal states for workflow control
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// System initializing
    Initializing,

    /// Ready to proceed with synthesis
    ReadyForSynthesis,

    /// Awaiting human approval at gate
    AwaitingGate,

    /// System halted
    Halted,

    /// Paused for review
    PausedForReview,

    /// Run completed successfully
    Completed,

    /// Generic active state
    Active,
}

/// Context data for generating Steno-Ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContext {
    /// Run identifier
    pub run_id: String,

    /// Current step (0-6)
    pub step: i32,

    /// Active governance role
    pub role: Role,

    /// Coherence Index (0.0 - 1.0)
    pub ci: Option<f64>,

    /// Expansion Velocity (-100% to +∞)
    pub ev: Option<f64>,

    /// Current operational mode
    pub mode: Mode,

    /// Current signal state
    pub signal: Signal,
}
