use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Artifact frontmatter structure (YAML at top of markdown file)
/// From specs/Method-VI_Artifact_Templates.md (line 35-45)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFrontmatter {
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub run_id: String,
    pub step_origin: i32,
    pub created_at: String, // ISO-8601
    pub hash: String,       // SHA-256 of content body
    pub parent_hash: Option<String>,
    pub dependencies: Vec<String>, // artifact_ids this depends on
    pub intent_anchor_link: Option<String>,
    pub is_immutable: bool,
    pub author: String,
    pub governance_role: GovernanceRole,
}

/// Valid artifact types from specs/Method-VI_Artifact_Templates.md (line 48-68)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ArtifactType {
    IntentSummary,
    PatternSuggestions,
    IntentAnchor,
    Charter,
    BaselineReport,
    ArchitectureMap,
    GovernanceSummary,
    DiagnosticSummary,
    LensEfficacyReport,
    CoreThesis,
    CausalSpineDraft,
    Glossary,
    FrameworkDraft,
    InnovationNotes,
    ValidationReport,
    FinalOutput,
    PatternCard,
}

/// Valid governance roles from specs/module-plan-method-vi.md (line 2945-2958)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum GovernanceRole {
    Observer,
    Conductor,
    Auditor,
    Patcher,
    Fabricator,
    Examiner,
    Curator,
    Archivist,
}

/// Complete artifact with frontmatter and content
#[derive(Debug, Clone)]
pub struct Artifact {
    pub frontmatter: ArtifactFrontmatter,
    pub content: String, // Markdown content (excluding frontmatter)
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    MissingField(String),
    InvalidFieldValue { field: String, reason: String },
    HashMismatch { expected: String, actual: String },
    DependencyNotFound(String),
    CircularDependency(Vec<String>),
    OrphanArtifact(String),
    ImmutableModification(String),
    UniquenessViolation(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ValidationError::InvalidFieldValue { field, reason } => {
                write!(f, "Invalid value for field '{}': {}", field, reason)
            }
            ValidationError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch - expected: {}, actual: {}", expected, actual)
            }
            ValidationError::DependencyNotFound(dep_id) => {
                write!(f, "Dependency not found: {}", dep_id)
            }
            ValidationError::CircularDependency(path) => {
                write!(f, "Circular dependency detected: {:?}", path)
            }
            ValidationError::OrphanArtifact(artifact_id) => {
                write!(f, "Orphan artifact (no parent_hash and not Intent_Anchor): {}", artifact_id)
            }
            ValidationError::ImmutableModification(artifact_id) => {
                write!(f, "Attempt to modify immutable artifact: {}", artifact_id)
            }
            ValidationError::UniquenessViolation(artifact_id) => {
                write!(f, "Artifact ID already exists in run: {}", artifact_id)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Parse artifact from markdown with YAML frontmatter
pub fn parse_artifact(markdown: &str) -> Result<Artifact> {
    // Split frontmatter and content
    let parts: Vec<&str> = markdown.splitn(3, "---").collect();

    if parts.len() < 3 {
        return Err(anyhow!("Invalid artifact format: missing YAML frontmatter"));
    }

    // Parse frontmatter (parts[1] is the YAML between ---)
    let frontmatter: ArtifactFrontmatter = serde_yaml::from_str(parts[1])
        .context("Failed to parse frontmatter YAML")?;

    // Content is everything after the second ---
    let content = parts[2].trim().to_string();

    Ok(Artifact {
        frontmatter,
        content,
    })
}

/// Calculate SHA-256 hash of content
pub fn calculate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate artifact frontmatter completeness
/// From specs/Method-VI_Artifact_Templates.md (line 39)
pub fn validate_frontmatter(frontmatter: &ArtifactFrontmatter) -> Result<(), ValidationError> {
    // All required fields are enforced by the struct definition
    // Additional validation for field values

    if frontmatter.artifact_id.is_empty() {
        return Err(ValidationError::MissingField("artifact_id".to_string()));
    }

    if frontmatter.run_id.is_empty() {
        return Err(ValidationError::MissingField("run_id".to_string()));
    }

    if frontmatter.hash.is_empty() {
        return Err(ValidationError::MissingField("hash".to_string()));
    }

    if frontmatter.author.is_empty() {
        return Err(ValidationError::MissingField("author".to_string()));
    }

    // Validate step_origin is in valid range (0-6)
    if frontmatter.step_origin < 0 || frontmatter.step_origin > 6 {
        return Err(ValidationError::InvalidFieldValue {
            field: "step_origin".to_string(),
            reason: format!("Must be 0-6, got {}", frontmatter.step_origin),
        });
    }

    Ok(())
}

/// Validate hash matches content
/// From specs/Method-VI_Artifact_Templates.md (line 41)
pub fn validate_hash(artifact: &Artifact) -> Result<(), ValidationError> {
    let calculated_hash = calculate_content_hash(&artifact.content);

    if calculated_hash != artifact.frontmatter.hash {
        return Err(ValidationError::HashMismatch {
            expected: artifact.frontmatter.hash.clone(),
            actual: calculated_hash,
        });
    }

    Ok(())
}

/// Validate parent_hash references
/// From specs/Method-VI_Artifact_Templates.md (line 42)
pub fn validate_parent(
    artifact: &ArtifactFrontmatter,
    existing_artifacts: &HashSet<String>, // Set of existing artifact hashes
) -> Result<(), ValidationError> {
    // Intent_Anchor is the root and should have no parent
    if artifact.artifact_type == ArtifactType::IntentAnchor {
        if artifact.parent_hash.is_some() {
            return Err(ValidationError::InvalidFieldValue {
                field: "parent_hash".to_string(),
                reason: "Intent_Anchor must have null parent_hash".to_string(),
            });
        }
        return Ok(());
    }

    // All other artifacts must have a parent_hash
    match &artifact.parent_hash {
        None => {
            return Err(ValidationError::OrphanArtifact(
                artifact.artifact_id.clone(),
            ));
        }
        Some(parent_hash) => {
            // Verify parent exists
            if !existing_artifacts.contains(parent_hash) {
                return Err(ValidationError::DependencyNotFound(
                    parent_hash.clone(),
                ));
            }
        }
    }

    Ok(())
}

/// Validate all dependencies exist
/// From specs/Method-VI_Artifact_Templates.md (line 35)
pub fn validate_dependencies(
    artifact: &ArtifactFrontmatter,
    existing_artifacts: &HashSet<String>, // Set of existing artifact IDs
) -> Result<(), ValidationError> {
    for dep_id in &artifact.dependencies {
        if !existing_artifacts.contains(dep_id) {
            return Err(ValidationError::DependencyNotFound(dep_id.clone()));
        }
    }
    Ok(())
}

/// Detect circular dependencies using DFS
pub fn detect_circular_dependency(
    artifact_id: &str,
    dependencies: &std::collections::HashMap<String, Vec<String>>, // artifact_id -> dependency_ids
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if path.contains(&artifact_id.to_string()) {
        // Found a cycle - return the path
        let cycle_start = path.iter().position(|id| id == artifact_id).unwrap();
        return Some(path[cycle_start..].to_vec());
    }

    if visited.contains(artifact_id) {
        return None; // Already processed this subtree
    }

    visited.insert(artifact_id.to_string());
    path.push(artifact_id.to_string());

    // Check all dependencies
    if let Some(deps) = dependencies.get(artifact_id) {
        for dep_id in deps {
            if let Some(cycle) = detect_circular_dependency(dep_id, dependencies, visited, path) {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}

/// Validate artifact uniqueness within run
/// From specs/Method-VI_Artifact_Templates.md (line 40)
pub fn validate_uniqueness(
    artifact_id: &str,
    existing_artifact_ids: &HashSet<String>,
) -> Result<(), ValidationError> {
    if existing_artifact_ids.contains(artifact_id) {
        return Err(ValidationError::UniquenessViolation(
            artifact_id.to_string(),
        ));
    }
    Ok(())
}

/// Check if artifact type is immutable
/// From specs/Method-VI_Artifact_Templates.md (line 50-68)
pub fn is_immutable_type(artifact_type: &ArtifactType) -> bool {
    matches!(
        artifact_type,
        ArtifactType::IntentAnchor
            | ArtifactType::Charter
            | ArtifactType::BaselineReport
            | ArtifactType::ArchitectureMap
    )
}

/// Validate immutability constraints
/// From specs/Method-VI_Artifact_Templates.md (line 44)
pub fn validate_immutability(
    artifact: &ArtifactFrontmatter,
    existing_immutable_ids: &HashSet<String>,
) -> Result<(), ValidationError> {
    // Check if trying to modify an immutable artifact
    if existing_immutable_ids.contains(&artifact.artifact_id) {
        return Err(ValidationError::ImmutableModification(
            artifact.artifact_id.clone(),
        ));
    }

    // Verify is_immutable flag matches artifact type
    let should_be_immutable = is_immutable_type(&artifact.artifact_type);
    if artifact.is_immutable != should_be_immutable {
        return Err(ValidationError::InvalidFieldValue {
            field: "is_immutable".to_string(),
            reason: format!(
                "Artifact type {:?} should have is_immutable={}",
                artifact.artifact_type, should_be_immutable
            ),
        });
    }

    Ok(())
}

/// Comprehensive artifact validation
pub fn validate_artifact(
    artifact: &Artifact,
    existing_artifact_ids: &HashSet<String>,
    existing_artifact_hashes: &HashSet<String>,
    existing_immutable_ids: &HashSet<String>,
    dependency_graph: &std::collections::HashMap<String, Vec<String>>,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 1. Validate frontmatter completeness
    if let Err(e) = validate_frontmatter(&artifact.frontmatter) {
        errors.push(e);
    }

    // 2. Validate uniqueness
    if let Err(e) = validate_uniqueness(&artifact.frontmatter.artifact_id, existing_artifact_ids) {
        errors.push(e);
    }

    // 3. Validate hash matches content
    if let Err(e) = validate_hash(artifact) {
        errors.push(e);
    }

    // 4. Validate parent_hash references
    if let Err(e) = validate_parent(&artifact.frontmatter, existing_artifact_hashes) {
        errors.push(e);
    }

    // 5. Validate dependencies exist
    if let Err(e) = validate_dependencies(&artifact.frontmatter, existing_artifact_ids) {
        errors.push(e);
    }

    // 6. Validate immutability
    if let Err(e) = validate_immutability(&artifact.frontmatter, existing_immutable_ids) {
        errors.push(e);
    }

    // 7. Detect circular dependencies
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    if let Some(cycle) =
        detect_circular_dependency(&artifact.frontmatter.artifact_id, dependency_graph, &mut visited, &mut path)
    {
        errors.push(ValidationError::CircularDependency(cycle));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_content_hash() {
        let content = "Test content";
        let hash = calculate_content_hash(content);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64-char hex string
    }

    #[test]
    fn test_validate_hash_success() {
        let content = "Test content".to_string();
        let hash = calculate_content_hash(&content);

        let artifact = Artifact {
            frontmatter: ArtifactFrontmatter {
                artifact_id: "test-001".to_string(),
                artifact_type: ArtifactType::IntentSummary,
                run_id: "run-001".to_string(),
                step_origin: 0,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                hash: hash.clone(),
                parent_hash: None,
                dependencies: vec![],
                intent_anchor_link: None,
                is_immutable: false,
                author: "test-agent".to_string(),
                governance_role: GovernanceRole::Observer,
            },
            content,
        };

        assert!(validate_hash(&artifact).is_ok());
    }

    #[test]
    fn test_validate_hash_mismatch() {
        let artifact = Artifact {
            frontmatter: ArtifactFrontmatter {
                artifact_id: "test-001".to_string(),
                artifact_type: ArtifactType::IntentSummary,
                run_id: "run-001".to_string(),
                step_origin: 0,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                hash: "wrong_hash".to_string(),
                parent_hash: None,
                dependencies: vec![],
                intent_anchor_link: None,
                is_immutable: false,
                author: "test-agent".to_string(),
                governance_role: GovernanceRole::Observer,
            },
            content: "Test content".to_string(),
        };

        assert!(matches!(validate_hash(&artifact), Err(ValidationError::HashMismatch { .. })));
    }

    #[test]
    fn test_intent_anchor_no_parent() {
        let mut existing = HashSet::new();

        let frontmatter = ArtifactFrontmatter {
            artifact_id: "intent-anchor-001".to_string(),
            artifact_type: ArtifactType::IntentAnchor,
            run_id: "run-001".to_string(),
            step_origin: 1,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            hash: "test_hash".to_string(),
            parent_hash: None,
            dependencies: vec![],
            intent_anchor_link: None,
            is_immutable: true,
            author: "scope-pattern-agent".to_string(),
            governance_role: GovernanceRole::Observer,
        };

        assert!(validate_parent(&frontmatter, &existing).is_ok());
    }

    #[test]
    fn test_non_intent_anchor_orphan() {
        let existing = HashSet::new();

        let frontmatter = ArtifactFrontmatter {
            artifact_id: "charter-001".to_string(),
            artifact_type: ArtifactType::Charter,
            run_id: "run-001".to_string(),
            step_origin: 1,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            hash: "test_hash".to_string(),
            parent_hash: None, // This should fail!
            dependencies: vec![],
            intent_anchor_link: Some("intent-anchor-001".to_string()),
            is_immutable: true,
            author: "scope-pattern-agent".to_string(),
            governance_role: GovernanceRole::Observer,
        };

        assert!(matches!(
            validate_parent(&frontmatter, &existing),
            Err(ValidationError::OrphanArtifact(_))
        ));
    }
}
