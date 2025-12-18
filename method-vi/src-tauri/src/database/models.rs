use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a run in the Knowledge Repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub intent_anchor_hash: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub final_ci: Option<f64>,
    pub final_ev: Option<f64>,
    pub status: Option<String>, // active | completed | aborted
}

/// Represents an artifact created during a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub run_id: String,
    pub r#type: String,
    pub step_origin: i32,
    pub hash: String,
    pub is_immutable: bool,
    pub content_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub parent_hash: Option<String>,
}

/// Represents an edge in the coherence spine graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String, // derived_from | constrained_by | references
    pub created_at: DateTime<Utc>,
}

/// Represents a reusable pattern with vitality tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub intent_category: String, // Exploratory | Analytical | Operational
    pub ci_achievement: Option<f64>,
    pub ev_stability: Option<f64>,
    pub architecture_pattern: Option<String>, // JSON blob
    pub analysis_pattern: Option<String>,     // JSON blob
    pub synthesis_pattern: Option<String>,    // JSON blob
    pub structure_pattern: Option<String>,    // JSON blob
    pub validation_pattern: Option<String>,   // JSON blob
    pub applicability: Option<String>,        // JSON: similar_contexts, pitfalls, adaptations
    pub vitality_freshness: f64,
    pub vitality_relevance: f64,
    pub application_count: i32,
    pub success_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_applied: Option<DateTime<Utc>>,
    pub source_run_id: Option<String>,
    pub is_starter: bool,
}

/// Represents a ledger entry for audit trail and state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Option<i64>, // AUTOINCREMENT, None for new entries
    pub run_id: String,
    pub entry_type: String, // gate | intervention | signal | decision
    pub step: Option<i32>,
    pub role: Option<String>,
    pub payload: Option<String>, // JSON blob
    pub prior_hash: Option<String>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

/// Represents a persistent flaw for tracking recurring issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentFlaw {
    pub id: Option<i64>, // AUTOINCREMENT, None for new entries
    pub flaw_description: String,
    pub occurrence_count: i32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub affected_runs: Option<String>, // JSON array of run IDs
    pub resolution_status: Option<String>, // open | resolved | escalated
    pub policy_ticket: Option<String>,
}
