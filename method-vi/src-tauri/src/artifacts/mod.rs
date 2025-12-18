/// Artifact validation and handling
/// Implements validation rules from specs/Method-VI_Artifact_Templates.md
pub mod validation;

pub use validation::{
    Artifact, ArtifactFrontmatter, ArtifactType, GovernanceRole, ValidationError,
    calculate_content_hash, detect_circular_dependency, is_immutable_type, parse_artifact,
    validate_artifact, validate_dependencies, validate_frontmatter, validate_hash,
    validate_immutability, validate_parent, validate_uniqueness,
};
