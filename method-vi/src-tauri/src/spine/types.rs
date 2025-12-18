use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Artifact types in the Method-VI workflow
///
/// The Critical Path consists of: Intent_Anchor → Charter → Baseline → Core_Thesis
/// These artifacts are immutable after creation and form the backbone of the coherence spine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Root artifact defining the intent (Step 0)
    /// Immutable, forms the foundation of the Critical Path
    Intent_Anchor,

    /// Defines scope and constraints (Step 1)
    /// Immutable, on Critical Path
    Charter,

    /// Frozen snapshot of existing state (Step 2)
    /// Immutable, on Critical Path
    Baseline,

    /// Main analytical outcome (Step 3-4)
    /// Immutable, on Critical Path
    Core_Thesis,

    /// Governance and meta-analysis artifacts
    Governance_Summary,

    /// Lens efficacy reports
    Lens_Efficacy_Report,

    /// Innovation and discovery notes
    Innovation_Notes,

    /// Diagnostic summaries
    Diagnostic_Summary,

    /// Framework drafts
    Framework_Draft,

    /// Individual sections of work
    Section,

    /// Patches and updates
    Patch,

    /// Other artifact types
    Other(String),
}

impl ArtifactType {
    /// Returns true if this artifact type is on the Critical Path
    ///
    /// Critical Path: Intent_Anchor → Charter → Baseline → Core_Thesis
    /// Artifacts on the Critical Path are immutable and cannot be targeted
    /// by Surgical Mode in Phase 2.
    pub fn is_on_critical_path(&self) -> bool {
        matches!(
            self,
            ArtifactType::Intent_Anchor
                | ArtifactType::Charter
                | ArtifactType::Baseline
                | ArtifactType::Core_Thesis
        )
    }
}

/// Dependency edge types between artifacts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Artifact is derived from another (e.g., Charter derived from Intent_Anchor)
    DerivedFrom,

    /// Artifact is constrained by another (e.g., Solution constrained by Charter)
    ConstrainedBy,

    /// Artifact references another for context
    References,
}

/// Represents an artifact node in the Coherence Spine DAG
///
/// Each artifact represents a deliverable or checkpoint in the Method-VI workflow.
/// Artifacts are connected via Dependency edges to form a directed acyclic graph (DAG).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier (e.g., "charter-v1-abc123")
    pub id: String,

    /// Type of artifact (determines if on Critical Path)
    pub artifact_type: ArtifactType,

    /// Step where artifact was created (0-6.5 in Method-VI)
    pub step_origin: i32,

    /// SHA-256 hash of artifact content for integrity verification
    pub hash: String,

    /// True for Intent_Anchor, Baseline, and locked artifacts
    /// Immutable artifacts cannot be modified after creation
    pub is_immutable: bool,

    /// Timestamp when artifact was created
    pub created_at: DateTime<Utc>,

    /// Hash of immediate predecessor for lineage tracking
    /// None for Intent_Anchor (root of the tree)
    pub parent_hash: Option<String>,
}

/// Represents a dependency edge in the Coherence Spine DAG
///
/// Dependencies create the structure of the spine, showing how artifacts
/// relate to each other. The spine must remain acyclic (DAG) to ensure
/// logical consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// ID of the artifact that depends (the source)
    pub source_id: String,

    /// ID of the artifact being depended upon (the target)
    pub target_id: String,

    /// Type of dependency relationship
    pub dependency_type: DependencyType,

    /// Timestamp when dependency was created
    pub created_at: DateTime<Utc>,
}

/// Result of spine integrity validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineIntegrityReport {
    /// True if spine has no breaks, orphans, or cycles
    pub valid: bool,

    /// List of broken edges (dependencies pointing to non-existent artifacts)
    pub breaks: Vec<BrokenEdge>,

    /// List of orphaned artifacts (no path to Intent_Anchor)
    pub orphans: Vec<String>,

    /// List of cycles detected (should never exist in a valid DAG)
    pub cycles: Vec<Vec<String>>,
}

/// Represents a broken edge in the spine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenEdge {
    /// Source artifact ID
    pub from: String,

    /// Target artifact ID (may not exist)
    pub to: String,

    /// Type of dependency
    pub dependency_type: DependencyType,
}

/// Dependency information returned by get_dependencies query
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyInfo {
    /// ID of the artifact this depends on
    pub id: String,

    /// Type of dependency relationship
    pub dependency_type: DependencyType,
}
