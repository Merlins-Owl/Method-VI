use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Process steps in Method-VI workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Step0_Intent,
    Step1_Baseline,
    Step2_Governance,
    Step3_Diagnostic,
    Step4_Synthesis,
    Step5_Redesign,
    Step6_Validation,
}

impl Step {
    /// Convert Step to numeric value for compatibility with existing code
    pub fn as_u8(&self) -> u8 {
        match self {
            Step::Step0_Intent => 0,
            Step::Step1_Baseline => 1,
            Step::Step2_Governance => 2,
            Step::Step3_Diagnostic => 3,
            Step::Step4_Synthesis => 4,
            Step::Step5_Redesign => 5,
            Step::Step6_Validation => 6,
        }
    }

    /// Convert numeric value to Step (for migration from existing code)
    pub fn from_u8(step: u8) -> Option<Self> {
        match step {
            0 => Some(Step::Step0_Intent),
            1 => Some(Step::Step1_Baseline),
            2 => Some(Step::Step2_Governance),
            3 => Some(Step::Step3_Diagnostic),
            4 => Some(Step::Step4_Synthesis),
            5 => Some(Step::Step5_Redesign),
            6 => Some(Step::Step6_Validation),
            _ => None,
        }
    }
}

/// User's posture selection from Step 0
/// Combined with CI baseline to determine mode (especially Transformation eligibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UserPosture {
    /// User hasn't confirmed posture yet
    #[default]
    Unconfirmed,
    /// User selected Build/Partner mode - creation focus
    Build,
    /// User selected Audit/Review mode - evaluation focus
    Audit,
}

// =============================================================================
// ARTIFACT FIDELITY TYPES (Phase 6: Charter-Driven Validation)
// =============================================================================

/// Fidelity level of an artifact - how complete/ready it is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArtifactFidelity {
    /// Placeholder/outline only - structure exists but no real content
    #[default]
    Draft,
    /// Structure complete, content partial - work in progress
    Placeholder,
    /// Complete and ready for validation
    Final,
}

/// An artifact expected to be produced during the run
/// Defined in Charter, validated at Step 6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedArtifact {
    /// Unique key for this artifact (e.g., "workshop_guide", "api_spec")
    pub artifact_key: String,
    /// Human-readable name (e.g., "Workshop Facilitator Guide")
    pub display_name: String,
    /// Minimum fidelity required for validation pass
    pub required_fidelity: ArtifactFidelity,
    /// If true, absence blocks Step 6 validation
    pub required: bool,
}

/// Structured Charter data with artifact tracking
/// Stores structured fields instead of raw markdown for better programmatic access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterData {
    /// SHA-256 hash of the charter content (for integrity verification)
    pub hash: String,
    /// Primary goal from IntentSummary
    pub primary_goal: String,
    /// Objectives extracted from Charter/IntentSummary
    pub objectives: Vec<String>,
    /// Artifacts expected to be produced (parsed from Success Criteria)
    pub expected_artifacts: Vec<ExpectedArtifact>,
    /// Success criteria clarity from IntentSummary ("Defined" | "Implied" | "Missing")
    pub success_criteria_state: String,
    /// When the Charter was created
    pub created_at: DateTime<Utc>,
}

impl CharterData {
    /// Get only the required artifacts (those that block validation if missing)
    pub fn get_required_artifacts(&self) -> Vec<&ExpectedArtifact> {
        self.expected_artifacts.iter().filter(|a| a.required).collect()
    }

    /// Get artifacts by fidelity level
    pub fn get_artifacts_by_fidelity(&self, fidelity: ArtifactFidelity) -> Vec<&ExpectedArtifact> {
        self.expected_artifacts.iter().filter(|a| a.required_fidelity == fidelity).collect()
    }

    /// Check if all required artifacts are defined
    pub fn has_all_required(&self) -> bool {
        !self.expected_artifacts.is_empty() &&
            self.expected_artifacts.iter().any(|a| a.required)
    }

    /// Generate display markdown from structured data
    pub fn to_display_markdown(&self) -> String {
        format!(
            "# Charter\n\n## Primary Goal\n{}\n\n## Objectives\n{}\n\n## Expected Deliverables\n{}\n\n## Success Criteria State\n{}\n\n---\n*Created: {}*",
            self.primary_goal,
            self.objectives.iter().map(|o| format!("- {}", o)).collect::<Vec<_>>().join("\n"),
            self.expected_artifacts.iter().map(|a| format!("- {} ({})", a.display_name, if a.required { "required" } else { "optional" })).collect::<Vec<_>>().join("\n"),
            self.success_criteria_state,
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

/// Auto-detected structure level of input content
/// Mode is determined by CI baseline at Step 1 and remains fixed for the run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureMode {
    /// CI baseline ≤ 0.35: High expansion expected, gaps and contradictions normal
    Architecting,
    /// CI baseline 0.36-0.69: Gap filling focus, moderate expansion
    Builder,
    /// Intent=Operational + Low Input CI + Build Posture.
    /// Content generation focus. Applies relaxed thresholds during Steps 3-5.
    Transformation,
    /// CI baseline ≥ 0.70: Polish and validation, tightening expected
    Refining,
}

impl StructureMode {
    /// Detect mode from CI baseline value (called at Step 1)
    pub fn from_ci_baseline(ci: f64) -> Self {
        if ci <= 0.35 {
            StructureMode::Architecting
        } else if ci >= 0.70 {
            StructureMode::Refining
        } else {
            StructureMode::Builder
        }
    }

    /// Returns true if Orange callouts should be downgraded to Yellow
    /// Implements Constraint 3: Noise Filter for Architecting Mode
    pub fn should_downgrade_orange(&self) -> bool {
        matches!(self, StructureMode::Architecting)
    }

    /// Human-readable name for logging and UI
    pub fn display_name(&self) -> &'static str {
        match self {
            StructureMode::Architecting => "Architecting",
            StructureMode::Builder => "Builder",
            StructureMode::Transformation => "Transformation",
            StructureMode::Refining => "Refining",
        }
    }

    /// User-facing message explaining what this mode means
    pub fn user_message(&self) -> &'static str {
        match self {
            StructureMode::Architecting =>
                "Engaging exploratory processing. Method-VI will focus on organizing and structuring your ideas.",
            StructureMode::Builder =>
                "Engaging builder processing. Method-VI will focus on filling gaps and strengthening your framework.",
            StructureMode::Transformation =>
                "Transformation mode: Converting rough input into structured deliverables. Relaxed thresholds during content generation.",
            StructureMode::Refining =>
                "Engaging refinement processing. Method-VI will focus on validation and polish.",
        }
    }
}

// =============================================================================
// BASELINE TYPES (Constraint 2: Delta Baseline Rule)
// =============================================================================

/// Baseline recorded at Step 1 (immutable anchor for the run)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RunAnchor {
    pub ci: f64,
}

/// Baseline recorded at Step 3 (informational, for Step 4+ delta calculation)
/// Step 3 CI is recorded but NOT enforced - diagnostic content is exploratory
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticBaseline {
    pub ci: f64,
}

/// Tracks which baseline is available for delta calculations
/// Prefer DiagnosticBaseline when available (more recent), fall back to RunAnchor
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Baseline {
    /// Preferred for Step 4+ deltas (recorded at Step 3)
    Diagnostic(DiagnosticBaseline),
    /// Fallback if Step 3 not yet complete (recorded at Step 1)
    Anchor(RunAnchor),
}

impl Baseline {
    /// Extract the CI value regardless of baseline type
    pub fn ci(&self) -> f64 {
        match self {
            Baseline::Diagnostic(d) => d.ci,
            Baseline::Anchor(a) => a.ci,
        }
    }

    /// Returns true if this is the preferred diagnostic baseline
    pub fn is_diagnostic(&self) -> bool {
        matches!(self, Baseline::Diagnostic(_))
    }
}

// =============================================================================
// THRESHOLD TYPES (Constraint 3: Noise Filter via Mode-Adjusted Thresholds)
// =============================================================================

/// Mode-adjusted thresholds for governance metrics
/// These values change based on detected StructureMode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Thresholds {
    // CI (Coherence Index) thresholds
    pub ci_pass: f64,
    pub ci_warn: f64,
    pub ci_critical: f64,
    // IAS (Intent Alignment Score) thresholds - also mode-adjusted
    pub ias_pass: f64,
    pub ias_warn: f64,
    pub ias_critical: f64,
}

/// Resolves thresholds based on detected mode
/// This is the engine for Constraint 3: Noise Filter for Architecting Mode
pub struct ThresholdResolver;

impl ThresholdResolver {
    /// Get thresholds appropriate for the given mode
    /// Step parameter is reserved for future step-specific adjustments
    pub fn resolve(mode: StructureMode, _step: Step) -> Thresholds {
        match mode {
            StructureMode::Architecting => Thresholds {
                // Relaxed thresholds - exploratory content expected
                ci_pass: 0.50,
                ci_warn: 0.35,
                ci_critical: 0.20,
                ias_pass: 0.50,
                ias_warn: 0.35,
                ias_critical: 0.20,
            },
            StructureMode::Builder => Thresholds {
                // Moderate thresholds - structure emerging
                ci_pass: 0.65,
                ci_warn: 0.50,
                ci_critical: 0.35,
                ias_pass: 0.65,
                ias_warn: 0.50,
                ias_critical: 0.35,
            },
            StructureMode::Transformation => Thresholds {
                // Same as Builder - content generation focus
                ci_pass: 0.65,
                ci_warn: 0.50,
                ci_critical: 0.35,
                ias_pass: 0.65,
                ias_warn: 0.50,
                ias_critical: 0.35,
            },
            StructureMode::Refining => Thresholds {
                // Strict thresholds - polished content expected
                ci_pass: 0.80,
                ci_warn: 0.70,
                ci_critical: 0.50,
                ias_pass: 0.80,
                ias_warn: 0.70,
                ias_critical: 0.50,
            },
        }
    }
}

// =============================================================================
// METRIC ENFORCEMENT (Constraint 2: Delta Baseline Rule)
// =============================================================================

/// Controls whether metrics generate callouts or are recorded only
/// Step 3 (Diagnostic) uses Informational mode - metrics are logged but no callouts generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricEnforcement {
    /// Normal: metrics generate callouts based on thresholds
    Enforced,
    /// Record only: metrics are logged but no callouts generated
    /// Used at Step 3 (Diagnostic) where low coherence is expected
    Informational,
}

// =============================================================================
// MODE DETECTION SERVICE (Constraint 1: Transparency Mandate)
// =============================================================================

/// Result of mode detection with metadata for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeDetectionResult {
    pub mode: StructureMode,
    pub ci_baseline: f64,
    pub confidence: f64,
    pub signals: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

pub struct ModeDetector;

impl ModeDetector {
    /// Detect mode with full metadata (Constraint 1: Transparency)
    pub fn detect(ci_baseline: f64) -> ModeDetectionResult {
        let mode = StructureMode::from_ci_baseline(ci_baseline);
        let confidence = Self::calculate_confidence(ci_baseline, mode);
        let signals = Self::generate_signals(ci_baseline, mode);

        let result = ModeDetectionResult {
            mode,
            ci_baseline,
            confidence,
            signals,
            detected_at: Utc::now(),
        };

        Self::log_detection(&result);
        result
    }

    fn calculate_confidence(ci: f64, mode: StructureMode) -> f64 {
        // Higher confidence when farther from boundaries (0.35, 0.70)
        match mode {
            StructureMode::Architecting => ((0.35 - ci) / 0.35).clamp(0.5, 1.0),
            StructureMode::Builder | StructureMode::Transformation => {
                let dist = (ci - 0.35).min(0.70 - ci);
                (dist / 0.175 + 0.5).clamp(0.5, 1.0)
            }
            StructureMode::Refining => ((ci - 0.70) / 0.30 + 0.5).clamp(0.5, 1.0),
        }
    }

    fn generate_signals(ci: f64, mode: StructureMode) -> Vec<String> {
        let mut signals = vec![format!("CI baseline: {:.2}", ci)];
        match mode {
            StructureMode::Architecting => {
                signals.push("Low initial structure detected".to_string());
                signals.push("Expecting significant content expansion".to_string());
            }
            StructureMode::Builder => {
                signals.push("Medium structure detected".to_string());
                signals.push("Expecting gap filling and reinforcement".to_string());
            }
            StructureMode::Transformation => {
                signals.push("Transformation mode activated".to_string());
                signals.push("Converting rough input into structured deliverables".to_string());
            }
            StructureMode::Refining => {
                signals.push("High structure detected".to_string());
                signals.push("Expecting polish and validation focus".to_string());
            }
        }
        signals
    }

    fn log_detection(result: &ModeDetectionResult) {
        let level = match result.mode {
            StructureMode::Architecting => "Low",
            StructureMode::Builder => "Medium",
            StructureMode::Transformation => "Transformation",
            StructureMode::Refining => "High",
        };
        // Constraint 1: Transparency Mandate
        log::info!(
            "Detected Structure: {}. Engaging {} Mode. (CI: {:.2}, Confidence: {:.0}%)",
            level, result.mode.display_name(), result.ci_baseline, result.confidence * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_as_u8() {
        assert_eq!(Step::Step0_Intent.as_u8(), 0);
        assert_eq!(Step::Step3_Diagnostic.as_u8(), 3);
        assert_eq!(Step::Step6_Validation.as_u8(), 6);
    }

    #[test]
    fn test_step_from_u8() {
        assert_eq!(Step::from_u8(0), Some(Step::Step0_Intent));
        assert_eq!(Step::from_u8(4), Some(Step::Step4_Synthesis));
        assert_eq!(Step::from_u8(6), Some(Step::Step6_Validation));
        assert_eq!(Step::from_u8(7), None);
        assert_eq!(Step::from_u8(99), None);
    }

    #[test]
    fn test_mode_from_ci_baseline_architecting() {
        assert_eq!(StructureMode::from_ci_baseline(0.20), StructureMode::Architecting);
        assert_eq!(StructureMode::from_ci_baseline(0.35), StructureMode::Architecting);
    }

    #[test]
    fn test_mode_from_ci_baseline_builder() {
        assert_eq!(StructureMode::from_ci_baseline(0.36), StructureMode::Builder);
        assert_eq!(StructureMode::from_ci_baseline(0.50), StructureMode::Builder);
        assert_eq!(StructureMode::from_ci_baseline(0.69), StructureMode::Builder);
    }

    #[test]
    fn test_mode_from_ci_baseline_refining() {
        assert_eq!(StructureMode::from_ci_baseline(0.70), StructureMode::Refining);
        assert_eq!(StructureMode::from_ci_baseline(0.85), StructureMode::Refining);
    }

    #[test]
    fn test_noise_filter_flag() {
        assert!(StructureMode::Architecting.should_downgrade_orange());
        assert!(!StructureMode::Builder.should_downgrade_orange());
        assert!(!StructureMode::Refining.should_downgrade_orange());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(StructureMode::Architecting.display_name(), "Architecting");
        assert_eq!(StructureMode::Builder.display_name(), "Builder");
        assert_eq!(StructureMode::Refining.display_name(), "Refining");
    }

    // =========================================================================
    // BASELINE TESTS
    // =========================================================================

    #[test]
    fn test_baseline_ci_extraction() {
        let diag = Baseline::Diagnostic(DiagnosticBaseline { ci: 0.42 });
        let anchor = Baseline::Anchor(RunAnchor { ci: 0.35 });

        assert_eq!(diag.ci(), 0.42);
        assert_eq!(anchor.ci(), 0.35);
    }

    #[test]
    fn test_baseline_is_diagnostic() {
        let diag = Baseline::Diagnostic(DiagnosticBaseline { ci: 0.42 });
        let anchor = Baseline::Anchor(RunAnchor { ci: 0.35 });

        assert!(diag.is_diagnostic());
        assert!(!anchor.is_diagnostic());
    }

    // =========================================================================
    // THRESHOLD RESOLVER TESTS
    // =========================================================================

    #[test]
    fn test_threshold_resolver_architecting() {
        let thresholds = ThresholdResolver::resolve(
            StructureMode::Architecting,
            Step::Step4_Synthesis
        );
        assert_eq!(thresholds.ci_pass, 0.50);
        assert_eq!(thresholds.ci_warn, 0.35);
        assert_eq!(thresholds.ci_critical, 0.20);
        assert_eq!(thresholds.ias_pass, 0.50);
    }

    #[test]
    fn test_threshold_resolver_builder() {
        let thresholds = ThresholdResolver::resolve(
            StructureMode::Builder,
            Step::Step4_Synthesis
        );
        assert_eq!(thresholds.ci_pass, 0.65);
        assert_eq!(thresholds.ci_warn, 0.50);
        assert_eq!(thresholds.ci_critical, 0.35);
    }

    #[test]
    fn test_threshold_resolver_refining() {
        let thresholds = ThresholdResolver::resolve(
            StructureMode::Refining,
            Step::Step6_Validation
        );
        assert_eq!(thresholds.ci_pass, 0.80);
        assert_eq!(thresholds.ci_warn, 0.70);
        assert_eq!(thresholds.ci_critical, 0.50);
    }

    #[test]
    fn test_threshold_resolver_ias_included() {
        let thresholds = ThresholdResolver::resolve(
            StructureMode::Refining,
            Step::Step6_Validation
        );
        // Verify IAS thresholds are also set
        assert_eq!(thresholds.ias_pass, 0.80);
        assert_eq!(thresholds.ias_warn, 0.70);
        assert_eq!(thresholds.ias_critical, 0.50);
    }

    // =========================================================================
    // MODE DETECTION TESTS
    // =========================================================================

    #[test]
    fn test_mode_detection_result_architecting() {
        let result = ModeDetector::detect(0.25);
        assert_eq!(result.mode, StructureMode::Architecting);
        assert!(result.confidence >= 0.5);
    }

    #[test]
    fn test_mode_detection_result_builder() {
        let result = ModeDetector::detect(0.50);
        assert_eq!(result.mode, StructureMode::Builder);
    }

    #[test]
    fn test_mode_detection_result_refining() {
        let result = ModeDetector::detect(0.80);
        assert_eq!(result.mode, StructureMode::Refining);
    }

    #[test]
    fn test_confidence_higher_away_from_boundaries() {
        let far = ModeDetector::detect(0.10);
        let near = ModeDetector::detect(0.33);
        assert!(far.confidence > near.confidence);
    }

    #[test]
    fn test_signals_contain_ci() {
        let result = ModeDetector::detect(0.50);
        assert!(result.signals.iter().any(|s| s.contains("0.50")));
    }
}
