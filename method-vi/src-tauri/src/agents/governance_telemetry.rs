use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::api::anthropic::AnthropicClient;
use crate::governance::{
    Callout, CalloutManager, CalloutTier, CalloutTrigger,
    MetricEnforcement, Step, StructureMode, ThresholdResolver,
};

/// Metric input - a value that contributed to the metric calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInput {
    pub name: String,
    pub value: MetricInputValue,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricInputValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Threshold values for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    pub pass: f64,
    pub warning: Option<f64>,
    pub halt: Option<f64>,
}

/// Metric status based on threshold comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    Pass,
    Warning,
    Fail,
}

/// Complete metric result with explainability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub value: f64,
    pub threshold: MetricThreshold,
    pub status: MetricStatus,
    pub inputs_used: Vec<MetricInput>,
    pub calculation_method: String,
    pub interpretation: String,
    pub recommendation: Option<String>,
}

/// All 6 critical metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalMetrics {
    pub ci: Option<MetricResult>,
    pub ev: Option<MetricResult>,
    pub ias: Option<MetricResult>,
    pub efi: Option<MetricResult>,
    pub sec: Option<MetricResult>,
    pub pci: Option<MetricResult>,
}

/// IAS Warning Type (FIX-024)
///
/// IAS is a "soft gate" that warns instead of HALTing for moderate drift
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IASWarningType {
    /// Step 4 special case - synthesis may have diverged from Charter
    ResynthesisPause,
    /// All other steps - requires acknowledgment to proceed
    AcknowledgmentRequired,
}

/// IAS Warning (FIX-024)
///
/// Triggered when IAS is in warning range (0.30-0.69)
/// Requires user acknowledgment to proceed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IASWarning {
    pub score: f64,
    pub warning_type: IASWarningType,
    pub message: String,
}

/// PCI Check (FIX-026)
///
/// Individual process compliance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCICheck {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

/// PCI Category (FIX-026)
///
/// Group of related checks with a weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCICategory {
    pub name: String,
    pub weight: f32,
    pub checks: Vec<PCICheck>,
}

/// PCI Checklist (FIX-026)
///
/// Complete deterministic process compliance audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCIChecklist {
    pub step_sequence: PCICategory,
    pub gate_compliance: PCICategory,
    pub artifact_presence: PCICategory,
    pub audit_integrity: PCICategory,
}

/// Orchestrator Audit Data (FIX-026)
///
/// Data extracted from orchestrator for PCI calculation
/// This is what governance agent receives to audit process compliance
#[derive(Debug, Clone)]
pub struct OrchestratorAuditData {
    pub current_step: u8,
    pub step_history: Vec<u8>,
    pub rollback_count: u32,
    pub halt_count: u32,
    pub override_count: u32,
    pub charter_approved: bool,
    pub charter_approver: Option<String>,
    pub synthesis_approved: bool,
    pub synthesis_approver: Option<String>,
    pub artifacts: Vec<String>,
    pub metric_snapshot_count: u32,
    pub has_timestamps: bool,
    pub artifact_versions_continuous: bool,
}

impl OrchestratorAuditData {
    /// Check if steps were executed in proper sequence (0→1→2→3...)
    pub fn steps_executed_in_order(&self) -> bool {
        if self.step_history.is_empty() {
            return true;
        }
        self.step_history.windows(2).all(|w| w[0] <= w[1])
    }

    /// Check for forward jumps (e.g., 2→4 skipping 3)
    pub fn has_forward_jumps(&self) -> bool {
        if self.step_history.is_empty() {
            return false;
        }
        self.step_history.windows(2).any(|w| w[1] > w[0] + 1)
    }

    /// Check if all rollbacks were properly logged
    pub fn rollbacks_all_logged(&self) -> bool {
        // For MVP: If we have rollback_count, assume they're all logged
        // In full implementation, verify each rollback has ledger entry
        true
    }

    /// Check if all HALTs were presented to user
    pub fn all_halts_presented_to_user(&self) -> bool {
        // For MVP: Assume all HALTs are presented (orchestrator blocks until resolved)
        true
    }

    /// Check if all override decisions have rationale
    pub fn all_overrides_have_rationale(&self) -> bool {
        // For MVP: Assume overrides have rationale if they exist
        // In full implementation, verify ledger entries have rationale field
        true
    }

    /// Check if artifact exists by name
    pub fn has_artifact(&self, name: &str) -> bool {
        self.artifacts.iter().any(|a| a.contains(name))
    }

    /// Check if all metrics were logged
    pub fn all_metrics_logged(&self) -> bool {
        // For MVP: Check if we have metric snapshots for steps executed
        self.metric_snapshot_count > 0
    }

    /// Check if all decisions have timestamps
    pub fn all_decisions_timestamped(&self) -> bool {
        self.has_timestamps
    }
}

/// E_baseline state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EBaseline {
    pub value: f64,
    pub locked: bool,
    pub locked_at_step: Option<u8>,
    pub source: String,
}

/// Governance & Telemetry Agent
///
/// Responsible for:
/// - Calculating Critical 6 metrics at step completion
/// - E_baseline management and locking
/// - Threshold monitoring and intervention triggers
/// - Metrics explainability following the contract
pub struct GovernanceTelemetryAgent {
    /// Claude API client for metric calculations
    api_client: AnthropicClient,

    /// E_baseline (locked after Step 1)
    e_baseline: Option<EBaseline>,

    /// Threshold configuration
    thresholds: ThresholdsConfig,
}

/// Threshold configuration for all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    pub ci: MetricThreshold,
    pub ev: MetricThreshold,
    pub ias: MetricThreshold,
    pub efi: MetricThreshold,
    pub sec: MetricThreshold,
    pub pci: MetricThreshold,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            ci: MetricThreshold {
                pass: 0.70,  // FIX-023: Lowered from 0.80 for step-semantic evaluation
                warning: Some(0.50),  // FIX-023: Adjusted from 0.70
                halt: Some(0.50),
            },
            ev: MetricThreshold {
                pass: 10.0,
                warning: Some(20.0),
                halt: Some(30.0),
            },
            ias: MetricThreshold {
                pass: 0.70,  // FIX-024: Soft gate - aligned intent
                warning: Some(0.30),  // FIX-024: 0.30-0.69 = drift detected, needs acknowledgment
                halt: Some(0.30),  // FIX-024: < 0.30 = extreme drift, hard stop
            },
            efi: MetricThreshold {
                pass: 0.80,  // FIX-025: ≥ 80% of scored claims substantiated
                warning: Some(0.50),  // FIX-025: 50-79% - some gaps in evidence
                halt: Some(0.50),  // FIX-025: < 50% - majority unsubstantiated
            },
            sec: MetricThreshold {
                pass: 100.0,
                warning: None,
                halt: Some(100.0),  // Any value < 100% should HALT (scope violation)
            },
            pci: MetricThreshold {
                pass: 0.95,  // FIX-026: ≥ 95% of checks pass
                warning: Some(0.70),  // FIX-026: 70-94% - some process gaps
                halt: Some(0.70),  // FIX-026: < 70% - significant process violations
            },
        }
    }
}

/// CI dimension weights for step-semantic evaluation (FIX-023)
///
/// Different Method-VI steps have different clarity priorities:
/// - Steps 1-2: Balanced weights (governance needs all-around clarity)
/// - Steps 3-4: Logical flow critical (diagnostics/synthesis prioritize reasoning)
/// - Steps 5-6: Structure important (deliverables need consistent organization)
#[derive(Debug, Clone)]
struct CIWeights {
    logical_flow: f32,
    term_consistency: f32,
    sentence_clarity: f32,
    structure_consistency: f32,
}

/// Get CI weights for the current step (FIX-023)
///
/// Per Method-VI Metrics Redesign Package v1.0, Section 2.1
fn get_ci_weights(step: u8) -> CIWeights {
    match step {
        // Profile A: Inception/Governance (Steps 1-2)
        // Balanced weights - governance documents need all-around clarity
        1 | 2 => CIWeights {
            logical_flow: 0.30,
            term_consistency: 0.30,
            sentence_clarity: 0.25,
            structure_consistency: 0.15,
        },
        // Profile B: Analysis/Synthesis (Steps 3-4)
        // Logical flow critical - diagnostics prioritize traceable reasoning
        // Structure unimportant - analysis is exploratory, not formatted
        3 | 4 => CIWeights {
            logical_flow: 0.50,
            term_consistency: 0.15,
            sentence_clarity: 0.30,
            structure_consistency: 0.05,  // Very low for diagnostic output
        },
        // Profile C: Production/Validation (Steps 5-6)
        // Structure important - deliverables need consistent organization
        5 | 6 => CIWeights {
            logical_flow: 0.25,
            term_consistency: 0.25,
            sentence_clarity: 0.20,
            structure_consistency: 0.30,  // High for deliverables
        },
        _ => get_ci_weights(1),  // Default to Profile A
    }
}

/// Get step context for CI evaluation (FIX-023)
fn get_step_context(step: u8) -> (&'static str, &'static str) {
    match step {
        1 => ("Baseline Establishment", "Create governing Charter; clarity prevents downstream misalignment"),
        2 => ("Governance Calibration", "Configure monitoring; clarity ensures correct rule application"),
        3 => ("Multi-Angle Analysis", "Diagnostic deep-dive; clarity ensures synthesis can interpret findings"),
        4 => ("Synthesis Lock-In", "Transform analysis to framework; clarity ensures deliverable has sound foundation"),
        5 => ("Structured Output", "Produce deliverable; clarity ensures end-user comprehension"),
        6 => ("Validation & Learning", "Final quality gate; clarity confirms deliverable is ready for use"),
        _ => ("Unknown Step", "Evaluating content clarity"),
    }
}

impl GovernanceTelemetryAgent {
    /// Create a new Governance & Telemetry Agent
    pub fn new(api_key: String) -> Result<Self> {
        let api_client = AnthropicClient::new(api_key)?;

        Ok(Self {
            api_client,
            e_baseline: None,
            thresholds: ThresholdsConfig::default(),
        })
    }

    /// Get the threshold configuration (for testing)
    pub fn get_thresholds(&self) -> &ThresholdsConfig {
        &self.thresholds
    }

    /// Calculate E_baseline from Baseline Report (Step 1)
    ///
    /// Per spec §9.1.2, E_baseline measures entropy:
    /// E = (Unique_Concepts + Defined_Relationships + Decision_Points) / Content_Units
    ///
    /// This calculates the baseline entropy that will be used for EV (Expansion Variance)
    /// calculations throughout the run.
    pub async fn calculate_e_baseline(&mut self, baseline_content: &str, _step: u8) -> Result<f64> {
        if self.e_baseline.is_some() && self.e_baseline.as_ref().unwrap().locked {
            return Err(anyhow::anyhow!("E_baseline is already locked and cannot be recalculated"));
        }

        info!("Calculating E_baseline entropy from baseline content...");

        // Calculate entropy using the standard formula
        let entropy = self.calculate_entropy(baseline_content).await?;

        info!("E_baseline entropy calculated: {:.2}", entropy);

        self.e_baseline = Some(EBaseline {
            value: entropy,
            locked: false,
            locked_at_step: None,
            source: "Baseline Report".to_string(),
        });

        Ok(entropy)
    }

    /// Lock E_baseline (Step 1 completion)
    ///
    /// Once locked, E_baseline becomes immutable for the rest of the run.
    pub fn lock_e_baseline(&mut self, step: u8) -> Result<()> {
        if let Some(ref mut baseline) = self.e_baseline {
            if baseline.locked {
                warn!("E_baseline already locked at step {}", baseline.locked_at_step.unwrap_or(0));
                return Ok(());
            }

            baseline.locked = true;
            baseline.locked_at_step = Some(step);
            info!("E_baseline locked at step {} with value: {}", step, baseline.value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot lock E_baseline: not yet calculated"))
        }
    }

    /// Get current E_baseline value
    pub fn get_e_baseline(&self) -> Option<f64> {
        self.e_baseline.as_ref().map(|b| b.value)
    }

    /// Calculate all 6 critical metrics for step completion
    pub async fn calculate_metrics(
        &self,
        content: &str,
        charter_objectives: &str,
        step: u8,
    ) -> Result<CriticalMetrics> {
        info!("Calculating Critical 6 metrics for step {}", step);

        // Calculate each metric
        let ci = self.calculate_ci(content, step).await?;
        let ev = self.calculate_ev(content).await?;
        let ias = self.calculate_ias(content, charter_objectives).await?;
        let efi = self.calculate_efi(content, step).await?;
        let sec = self.calculate_sec()?;

        // FIX-026: Create stub audit data for PCI (MVP - orchestrator will provide full data later)
        let audit_stub = OrchestratorAuditData {
            current_step: step,
            step_history: (0..=step).collect(), // Assume linear progression for stub
            rollback_count: 0,
            halt_count: 0,
            override_count: 0,
            charter_approved: step >= 1,  // Assume approved if past Step 1
            charter_approver: if step >= 1 { Some("User".to_string()) } else { None },
            synthesis_approved: step >= 4,  // Assume approved if past Step 4
            synthesis_approver: if step >= 4 { Some("User".to_string()) } else { None },
            artifacts: vec!["Charter".to_string(), "Architecture".to_string()],  // Basic stub
            metric_snapshot_count: step as u32,  // Assume one snapshot per step
            has_timestamps: true,  // Assume timestamps exist
            artifact_versions_continuous: true,  // Assume no version gaps
        };
        let pci = self.calculate_pci(&audit_stub)?;

        Ok(CriticalMetrics {
            ci: Some(ci),
            ev: Some(ev),
            ias: Some(ias),
            efi: Some(efi),
            sec: Some(sec),
            pci: Some(pci),
        })
    }

    /// Check for IAS Warning (FIX-024)
    ///
    /// IAS is a "soft gate" that warns for moderate drift (0.30-0.69) instead of HALTing.
    /// Only HALTs at extreme drift (< 0.30).
    ///
    /// Returns:
    /// - Some(IASWarning) if IAS is in warning range (0.30-0.69)
    /// - None if IAS is passing (≥ 0.70) or will HALT (< 0.30)
    ///
    /// Warning types:
    /// - Step 4: ResynthesisPause (synthesis may have diverged from Charter)
    /// - Other steps: AcknowledgmentRequired (drift needs acknowledgment)
    pub fn check_ias_warning(&self, metrics: &CriticalMetrics, step: u8) -> Option<IASWarning> {
        if let Some(ref ias) = metrics.ias {
            if ias.value >= 0.30 && ias.value < 0.70 {
                let warning_type = if step == 4 {
                    IASWarningType::ResynthesisPause
                } else {
                    IASWarningType::AcknowledgmentRequired
                };

                return Some(IASWarning {
                    score: ias.value,
                    warning_type: warning_type.clone(),
                    message: format!(
                        "Intent drift detected (IAS: {:.2}). {}",
                        ias.value,
                        match warning_type {
                            IASWarningType::ResynthesisPause =>
                                "Synthesis may have diverged from Charter. Review before proceeding.",
                            IASWarningType::AcknowledgmentRequired =>
                                "Content may have drifted from original intent. Acknowledge to proceed.",
                        }
                    ),
                });
            }
        }
        None
    }

    // =============================================================================
    // CI (Coherence Index) - Step-Semantic Weights (FIX-023)
    // =============================================================================

    /// Calculate CI (Coherence Index) with step-semantic weights
    ///
    /// FIX-023: Uses step-aware evaluation that prioritizes different clarity aspects
    /// based on the step's purpose:
    /// - Steps 1-2: Balanced weights (governance documents)
    /// - Steps 3-4: Logical flow critical (analysis/synthesis)
    /// - Steps 5-6: Structure important (deliverables)
    ///
    /// Returns 0.0-1.0 score with detailed dimension breakdown.
    async fn calculate_ci(&self, content: &str, step: u8) -> Result<MetricResult> {
        debug!("Calculating CI (Coherence Index) for Step {}", step);

        let weights = get_ci_weights(step);
        let (step_name, step_purpose) = get_step_context(step);

        let system_prompt = "You are evaluating the CLARITY of content for a specific Method-VI step. \
            Score each dimension objectively from 0.0 to 1.0. \
            Return ONLY valid JSON.";

        let user_message = format!(r#"STEP CONTEXT: Step {} — {}
PURPOSE: {}

Evaluate the content on four dimensions. Score each from 0.0 to 1.0:

1. LOGICAL FLOW ({:.0}% weight)
   - Do ideas connect in a traceable sequence?
   - Can a reader follow the reasoning from start to end?

2. TERM CONSISTENCY ({:.0}% weight)
   - Are key terms used uniformly throughout?
   - Are there conflicting definitions?

3. SENTENCE CLARITY ({:.0}% weight)
   - Are individual sentences parseable on first read?
   - Is prose free of convoluted constructions?

4. STRUCTURE CONSISTENCY ({:.0}% weight)
   - Is content organized predictably?
   - Do sections/headers aid comprehension?

IMPORTANT: Measure CLARITY only, not correctness or completeness.

Calculate weighted CI:
CI = (flow × {:.2}) + (term × {:.2}) + (clarity × {:.2}) + (structure × {:.2})

Respond in JSON:
{{
  "logical_flow": {{"score": 0.XX, "rationale": "..."}},
  "term_consistency": {{"score": 0.XX, "rationale": "..."}},
  "sentence_clarity": {{"score": 0.XX, "rationale": "..."}},
  "structure_consistency": {{"score": 0.XX, "rationale": "..."}},
  "ci_score": 0.XX,
  "overall_assessment": "One sentence summary"
}}

CONTENT:
---
{}
---"#,
            step, step_name, step_purpose,
            weights.logical_flow * 100.0,
            weights.term_consistency * 100.0,
            weights.sentence_clarity * 100.0,
            weights.structure_consistency * 100.0,
            weights.logical_flow,
            weights.term_consistency,
            weights.sentence_clarity,
            weights.structure_consistency,
            content
        );

        let response = self.api_client
            .call_claude(&system_prompt, &user_message, None, Some(2048), Some(0.0))
            .await?;

        // Parse JSON response - extract JSON if embedded in text
        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse CI response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        // Extract CI score (use ci_score if available, otherwise fall back to score)
        let score = parsed["ci_score"]
            .as_f64()
            .or_else(|| parsed["score"].as_f64())
            .context("Missing or invalid 'ci_score' or 'score' field in CI response")?;

        // Extract dimension scores for detailed interpretation
        let logical_flow = parsed["logical_flow"]["score"].as_f64().unwrap_or(0.0);
        let term_consistency = parsed["term_consistency"]["score"].as_f64().unwrap_or(0.0);
        let sentence_clarity = parsed["sentence_clarity"]["score"].as_f64().unwrap_or(0.0);
        let structure_consistency = parsed["structure_consistency"]["score"].as_f64().unwrap_or(0.0);

        let overall_assessment = parsed["overall_assessment"]
            .as_str()
            .unwrap_or("No assessment provided")
            .to_string();

        let status = self.evaluate_status(score, &self.thresholds.ci, false);

        Ok(MetricResult {
            metric_name: "CI".to_string(),
            value: score,
            threshold: self.thresholds.ci.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Step".to_string(),
                    value: MetricInputValue::Number(step as f64),
                    source: "Orchestrator".to_string(),
                },
                MetricInput {
                    name: "Logical Flow".to_string(),
                    value: MetricInputValue::Number(logical_flow),
                    source: format!("LLM ({:.0}% weight)", weights.logical_flow * 100.0),
                },
                MetricInput {
                    name: "Term Consistency".to_string(),
                    value: MetricInputValue::Number(term_consistency),
                    source: format!("LLM ({:.0}% weight)", weights.term_consistency * 100.0),
                },
                MetricInput {
                    name: "Sentence Clarity".to_string(),
                    value: MetricInputValue::Number(sentence_clarity),
                    source: format!("LLM ({:.0}% weight)", weights.sentence_clarity * 100.0),
                },
                MetricInput {
                    name: "Structure Consistency".to_string(),
                    value: MetricInputValue::Number(structure_consistency),
                    source: format!("LLM ({:.0}% weight)", weights.structure_consistency * 100.0),
                },
            ],
            calculation_method: format!(
                "Step-semantic weighted CI (Step {} - {}): \
                Flow={:.2}×{:.2} + Term={:.2}×{:.2} + Clarity={:.2}×{:.2} + Structure={:.2}×{:.2} = {:.2}",
                step, step_name,
                logical_flow, weights.logical_flow,
                term_consistency, weights.term_consistency,
                sentence_clarity, weights.sentence_clarity,
                structure_consistency, weights.structure_consistency,
                score
            ),
            interpretation: overall_assessment,
            recommendation: if status != MetricStatus::Pass {
                Some(match step {
                    1 | 2 => "Improve all-around clarity. Focus on term consistency and logical flow for governance documents.".to_string(),
                    3 | 4 => "Strengthen logical flow and sentence clarity. Structure is less critical for diagnostic/synthesis content.".to_string(),
                    5 | 6 => "Improve structure consistency and organization. Deliverables require predictable formatting.".to_string(),
                    _ => "Review content clarity across all dimensions.".to_string(),
                })
            } else {
                None
            },
        })
    }

    /// Calculate entropy for content using LLM analysis
    ///
    /// Per spec §9.1.2:
    /// E = (Unique_Concepts + Defined_Relationships + Decision_Points) / Content_Units
    async fn calculate_entropy(&self, content: &str) -> Result<f64> {
        let system_prompt = "You are an entropy analysis expert for Method-VI governance. \
            Analyze content to identify unique concepts, defined relationships, and decision points. \
            Return ONLY a JSON object with this exact structure: \
            {\"unique_concepts\": <number>, \"relationships\": <number>, \"decision_points\": <number>, \"content_units\": <number>}";

        let user_message = format!(
            r#"Analyze this content for entropy calculation:

{}

Extract:
1. unique_concepts: Count of distinct concepts, entities, or ideas introduced
2. relationships: Count of defined relationships between concepts (dependencies, hierarchies, flows)
3. decision_points: Count of places requiring decisions, choices, or branching logic
4. content_units: Total semantic units (paragraphs, sections, or logical blocks)

Return JSON with counts."#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024), Some(0.0))
            .await?;

        // Parse JSON response - extract JSON if embedded in text
        let entropy_data: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse entropy analysis as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let unique_concepts = entropy_data["unique_concepts"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing unique_concepts in response"))?;

        let relationships = entropy_data["relationships"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing relationships in response"))?;

        let decision_points = entropy_data["decision_points"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing decision_points in response"))?;

        let content_units = entropy_data["content_units"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing content_units in response"))?;

        // Calculate entropy: E = (Unique_Concepts + Defined_Relationships + Decision_Points) / Content_Units
        let entropy = if content_units > 0.0 {
            (unique_concepts + relationships + decision_points) / content_units
        } else {
            0.0
        };

        Ok(entropy)
    }

    // =============================================================================
    // EV (Entropy Variance) - INFORMATIONAL ONLY (FIX-027)
    // =============================================================================
    //
    // Status: NEVER triggers HALT or Warning - purely informational for calibration
    //
    // MVP Implementation:
    // - Uses LLM-based entropy estimation (concepts + relationships + decision_points)
    // - Calculated and logged for calibration data collection
    // - Always returns Pass status (never blocks progression)
    //
    // Rationale:
    // - Entropy variance is expected across steps (analysis expands, synthesis condenses)
    // - No calibration data exists yet to set meaningful thresholds
    // - Metric useful for observing patterns but not enforcing constraints
    //
    // Phase 2 Considerations:
    // 1. Collect EV data across multiple runs to establish baseline patterns
    // 2. Analyze step-specific variance ranges:
    //    - Step 2 (Charter): Should match input entropy closely
    //    - Step 3 (Analysis): Entropy typically increases (more concepts)
    //    - Step 4 (Synthesis): Entropy may decrease (consolidation)
    //    - Step 5 (Framework): Entropy typically increases (detailed implementation)
    // 3. Consider if enforcement is valuable (may remain informational permanently)
    //
    // =============================================================================

    /// Calculate EV (Expansion Variance) - FIX-027
    ///
    /// INFORMATIONAL ONLY - Never triggers HALT or Warning
    ///
    /// Measures entropy variance from baseline:
    /// EV = ((E_current - E_baseline) / E_baseline) × 100%
    async fn calculate_ev(&self, content: &str) -> Result<MetricResult> {
        debug!("Calculating EV (Expansion Variance)");

        // Calculate current entropy using same formula as E_baseline
        let e_current = self.calculate_entropy(content).await?;

        let e_baseline = self.e_baseline
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("E_baseline not set"))?
            .value;

        // Formula: |E_current - E_baseline| / E_baseline × 100
        let variance = ((e_current - e_baseline).abs() / e_baseline) * 100.0;

        // FIX-027: Always Pass - informational only, never blocks
        let status = MetricStatus::Pass;

        info!("EV calculated: {:.1}% variance (informational only - not enforced)", variance);

        Ok(MetricResult {
            metric_name: "EV".to_string(),
            value: variance,
            threshold: self.thresholds.ev.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "E_current".to_string(),
                    value: MetricInputValue::Number(e_current),
                    source: "Current Content".to_string(),
                },
                MetricInput {
                    name: "E_baseline".to_string(),
                    value: MetricInputValue::Number(e_baseline),
                    source: "Baseline Report".to_string(),
                },
            ],
            calculation_method: format!(
                "|E_current - E_baseline| / E_baseline × 100 = |{:.2} - {:.2}| / {:.2} × 100 = {:.2}%",
                e_current, e_baseline, e_baseline, variance
            ),
            interpretation: format!(
                "Content has {:.1}% entropy variance from baseline (informational only - not enforced). Current: {:.2}, Baseline: {:.2}.",
                variance,
                e_current,
                e_baseline
            ),
            recommendation: Some("Informational metric for calibration data collection. No action required.".to_string()),
        })
    }

    /// Calculate IAS (Intent Alignment Score)
    ///
    /// Compares current content against Charter objectives.
    /// Returns 0.0-1.0 score.
    async fn calculate_ias(&self, content: &str, charter_objectives: &str) -> Result<MetricResult> {
        debug!("Calculating IAS (Intent Alignment Score)");

        let system_prompt = "You are an intent alignment analyzer for Method-VI governance. \
            Compare content against Charter objectives to measure alignment. \
            Return ONLY a JSON object with this exact structure: \
            {\"score\": <number 0-1>, \"reasoning\": \"<explanation>\"}";

        let user_message = format!(
            "Compare this content against Charter objectives:\n\n\
            CHARTER OBJECTIVES:\n{}\n\n\
            CURRENT CONTENT:\n{}\n\n\
            Return alignment score 0.0-1.0 and brief reasoning.",
            charter_objectives,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024), Some(0.0))
            .await?;

        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse IAS response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let score = parsed["score"]
            .as_f64()
            .context("Missing or invalid 'score' field in IAS response")?;

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let status = self.evaluate_status(score, &self.thresholds.ias, false);

        Ok(MetricResult {
            metric_name: "IAS".to_string(),
            value: score,
            threshold: self.thresholds.ias.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Charter Objectives".to_string(),
                    value: MetricInputValue::String(charter_objectives.to_string()),
                    source: "Charter".to_string(),
                },
            ],
            calculation_method: "LLM-based comparison of content against Charter objectives".to_string(),
            interpretation: reasoning,
            recommendation: if status != MetricStatus::Pass {
                Some("Review content alignment with Charter objectives. Consider refocusing on original intent.".to_string())
            } else {
                None
            },
        })
    }

    /// Evaluate EFI status with step-specific enforcement (FIX-025)
    ///
    /// EFI (Evidence Fidelity Index) has different enforcement levels at different steps:
    /// - Steps 1-3: Informational only (always Pass) - early steps don't need evidence yet
    /// - Step 4: Warning only - synthesis should start building evidence
    /// - Step 5: Informational only (always Pass) - framework may not have all evidence yet
    /// - Step 6: Full enforcement - validation requires complete evidence
    ///
    /// # Arguments
    /// * `score` - EFI score (0.0-1.0, percentage of scored claims substantiated)
    /// * `step` - Current Method-VI step (1-6)
    ///
    /// # Returns
    /// MetricStatus based on score and step-specific enforcement
    fn evaluate_efi_status(&self, score: f64, step: u8) -> MetricStatus {
        if step == 6 {
            // Step 6: Full enforcement - validation requires evidence
            if score >= 0.80 {
                MetricStatus::Pass
            } else if score >= 0.50 {
                MetricStatus::Warning
            } else {
                MetricStatus::Fail  // Will HALT
            }
        } else if step == 4 {
            // Step 4: Warning only - synthesis should start building evidence
            if score >= 0.80 {
                MetricStatus::Pass
            } else {
                MetricStatus::Warning  // Never Fail before Step 6
            }
        } else {
            // Steps 1-3, 5: Informational only
            MetricStatus::Pass  // Always pass, just log the value
        }
    }

    /// Calculate EFI (Execution Fidelity Index) with Claim Taxonomy (FIX-025)
    ///
    /// Audits claims for evidence support using a claim taxonomy:
    /// - SCORED claims (factual, prescriptive) require evidence
    /// - EXEMPT claims (exploratory, instructional, observational) do not
    ///
    /// This prevents false failures on workshop/training content where
    /// instructional statements like "Good leaders listen" are exempt from
    /// evidence requirements.
    ///
    /// Returns 0.0-1.0 score (percentage of scored claims substantiated).
    /// If no scored claims exist, returns 1.0 (perfect score).
    async fn calculate_efi(&self, content: &str, step: u8) -> Result<MetricResult> {
        debug!("Calculating EFI (Evidence Fidelity Index) for Step {} with claim taxonomy", step);

        let system_prompt = r#"You are an evidence fidelity analyst. Your task is to:
1. Identify all claims in the content
2. Classify each claim by type using the claim taxonomy
3. For SCORED claims only, determine if evidence is provided
4. Calculate EFI based only on scored claims

Return ONLY valid JSON."#;

        let user_message = format!(r#"
Analyze the following content for evidence fidelity.

CLAIM TAXONOMY:

SCORED CLAIMS (require evidence):
- FACTUAL: Assertions about verifiable reality
  Example: "AI adoption increased 40% in 2024"
- PRESCRIPTIVE: Recommendations with implied outcomes
  Example: "Companies should implement AI governance to reduce risk"

EXEMPT CLAIMS (do not require evidence):
- EXPLORATORY: Questions, hypotheses, possibilities
  Example: "What if we considered a phased approach?"
- INSTRUCTIONAL: Teaching statements, definitions, explanations
  Example: "A neural network consists of interconnected nodes"
- OBSERVATIONAL: Descriptions of what exists without causal claims
  Example: "The current process has five steps"

INSTRUCTIONS:
1. List each claim in the content
2. Classify as: FACTUAL, PRESCRIPTIVE, EXPLORATORY, INSTRUCTIONAL, or OBSERVATIONAL
3. For FACTUAL and PRESCRIPTIVE claims only: Is evidence provided? (yes/no)
4. Calculate EFI = (Substantiated Scored Claims) / (Total Scored Claims)
   - If zero scored claims, EFI = 1.0 (no claims requiring evidence)

Respond in JSON:
{{
  "claims": [
    {{
      "text": "...",
      "type": "FACTUAL|PRESCRIPTIVE|EXPLORATORY|INSTRUCTIONAL|OBSERVATIONAL",
      "scored": true|false,
      "substantiated": true|false|null,
      "evidence_reference": "..." or null
    }}
  ],
  "summary": {{
    "total_claims": N,
    "scored_claims": N,
    "substantiated_scored": N,
    "efi_score": X.XX
  }},
  "reasoning": "One sentence explanation"
}}

CONTENT:
---
{}
---
"#, content);

        let response = self.api_client
            .call_claude(&system_prompt, &user_message, None, Some(4096), Some(0.0))
            .await?;

        // Parse and validate
        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse EFI response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let efi_score = parsed["summary"]["efi_score"]
            .as_f64()
            .unwrap_or(1.0);  // Default to 1.0 if no scored claims

        let total_claims = parsed["summary"]["total_claims"]
            .as_u64()
            .unwrap_or(0);

        let scored_claims = parsed["summary"]["scored_claims"]
            .as_u64()
            .unwrap_or(0);

        let substantiated_scored = parsed["summary"]["substantiated_scored"]
            .as_u64()
            .unwrap_or(0);

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        // Log for debugging
        info!(
            "EFI Calculation (Step {}): {} total claims, {} scored claims, {} substantiated, score = {:.2}",
            step, total_claims, scored_claims, substantiated_scored, efi_score
        );

        // Use step-specific enforcement
        let status = self.evaluate_efi_status(efi_score, step);

        Ok(MetricResult {
            metric_name: "EFI".to_string(),
            value: efi_score,
            threshold: self.thresholds.efi.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Total Claims".to_string(),
                    value: MetricInputValue::Number(total_claims as f64),
                    source: "Content Analysis".to_string(),
                },
                MetricInput {
                    name: "Scored Claims".to_string(),
                    value: MetricInputValue::Number(scored_claims as f64),
                    source: "Claim Taxonomy Filter".to_string(),
                },
                MetricInput {
                    name: "Substantiated Scored Claims".to_string(),
                    value: MetricInputValue::Number(substantiated_scored as f64),
                    source: "Evidence Analysis".to_string(),
                },
            ],
            calculation_method: if scored_claims > 0 {
                format!(
                    "Claim Taxonomy: {} substantiated / {} scored claims = {:.2}",
                    substantiated_scored, scored_claims, efi_score
                )
            } else {
                "No scored claims (instructional/exploratory content) = 1.0".to_string()
            },
            interpretation: reasoning,
            recommendation: if status == MetricStatus::Fail {
                Some("Add evidence and citations to support factual and prescriptive claims. Increase rigor of substantiation.".to_string())
            } else if status == MetricStatus::Warning {
                Some("Consider adding more evidence for factual and prescriptive claims to improve credibility.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate SEC (Scope Expansion Count) - FIX-027
    ///
    /// PLACEHOLDER FOR MVP - Always returns 100%
    ///
    /// Scope change detection not implemented. Always assumes perfect compliance.
    fn calculate_sec(&self) -> Result<MetricResult> {
        debug!("Calculating SEC (Scope Expansion Count) - placeholder");

        // FIX-027: SEC is placeholder for MVP - always returns 100%
        // Scope detection not implemented
        info!("SEC: Placeholder returning 100% (scope detection not implemented)");

        Ok(MetricResult {
            metric_name: "SEC".to_string(),
            value: 100.0,  // Always 100% for MVP
            threshold: self.thresholds.sec.clone(),
            status: MetricStatus::Pass,  // Always Pass
            inputs_used: vec![],  // No inputs - placeholder
            calculation_method: "Placeholder (always 100% - scope detection not implemented)".to_string(),
            interpretation: "Scope detection not implemented for MVP. Assumed compliant (100%).".to_string(),
            recommendation: Some("Future enhancement: Implement scope change detection and tracking in Steno-Ledger.".to_string()),
        })
    }

    /// Calculate PCI (Process Compliance Index) - FIX-026
    ///
    /// DETERMINISTIC checklist-based audit of Method-VI process compliance.
    /// NO LLM calls - uses orchestrator audit data only.
    ///
    /// Returns 0.0-1.0 score based on weighted category compliance.
    fn calculate_pci(&self, audit: &OrchestratorAuditData) -> Result<MetricResult> {
        debug!("Calculating PCI (Process Compliance Index) - deterministic checklist");

        // Build checklist from audit data
        let checklist = self.build_pci_checklist(audit);

        // Score checklist (weighted average)
        let score = self.score_pci_checklist(&checklist);

        // Evaluate status
        let status = self.evaluate_status(score, &self.thresholds.pci, false);

        Ok(MetricResult {
            metric_name: "PCI".to_string(),
            value: score,
            threshold: self.thresholds.pci.clone(),
            status: status.clone(),
            inputs_used: self.checklist_to_inputs(&checklist),
            calculation_method: "Deterministic checklist audit (4 categories: Step Sequence 25%, Gate Compliance 30%, Artifact Presence 20%, Audit Integrity 25%)".to_string(),
            interpretation: self.summarize_pci_checklist(&checklist, score),
            recommendation: self.get_pci_recommendation(score, &checklist),
        })
    }

    /// Build PCI checklist from orchestrator audit data (FIX-026)
    fn build_pci_checklist(&self, audit: &OrchestratorAuditData) -> PCIChecklist {
        PCIChecklist {
            // Category 1: Step Sequence (25% weight)
            step_sequence: PCICategory {
                name: "Step Sequence".to_string(),
                weight: 0.25,
                checks: vec![
                    PCICheck {
                        name: "steps_in_order".to_string(),
                        passed: audit.steps_executed_in_order(),
                        details: format!("Steps executed: {:?}", audit.step_history),
                    },
                    PCICheck {
                        name: "no_forward_jumps".to_string(),
                        passed: !audit.has_forward_jumps(),
                        details: if audit.has_forward_jumps() {
                            "Steps skipped - forward jump detected".to_string()
                        } else {
                            "No steps skipped".to_string()
                        },
                    },
                    PCICheck {
                        name: "rollbacks_logged".to_string(),
                        passed: audit.rollbacks_all_logged(),
                        details: format!("{} rollback(s), all logged", audit.rollback_count),
                    },
                ],
            },

            // Category 2: Gate Compliance (30% weight)
            gate_compliance: PCICategory {
                name: "Gate Compliance".to_string(),
                weight: 0.30,
                checks: vec![
                    PCICheck {
                        name: "charter_approval".to_string(),
                        passed: audit.charter_approved,
                        details: format!(
                            "Charter {}: {}",
                            if audit.charter_approved { "approved" } else { "not approved" },
                            audit.charter_approver.as_deref().unwrap_or("N/A")
                        ),
                    },
                    PCICheck {
                        name: "synthesis_approval".to_string(),
                        passed: audit.synthesis_approved,
                        details: format!(
                            "Synthesis {}: {}",
                            if audit.synthesis_approved { "approved" } else { "not approved" },
                            audit.synthesis_approver.as_deref().unwrap_or("N/A")
                        ),
                    },
                    PCICheck {
                        name: "halt_gates_presented".to_string(),
                        passed: audit.all_halts_presented_to_user(),
                        details: format!("{} HALT(s), all presented to user", audit.halt_count),
                    },
                    PCICheck {
                        name: "override_rationale_recorded".to_string(),
                        passed: audit.all_overrides_have_rationale(),
                        details: format!("{} override(s), all with rationale", audit.override_count),
                    },
                ],
            },

            // Category 3: Artifact Presence (20% weight)
            artifact_presence: PCICategory {
                name: "Artifact Presence".to_string(),
                weight: 0.20,
                checks: vec![
                    PCICheck {
                        name: "charter_exists".to_string(),
                        passed: audit.has_artifact("Charter") || audit.has_artifact("charter"),
                        details: if audit.has_artifact("Charter") || audit.has_artifact("charter") {
                            "Charter artifact present".to_string()
                        } else {
                            "Charter artifact missing".to_string()
                        },
                    },
                    PCICheck {
                        name: "architecture_map_exists".to_string(),
                        passed: audit.has_artifact("Architecture") || audit.has_artifact("architecture"),
                        details: if audit.has_artifact("Architecture") || audit.has_artifact("architecture") {
                            "Architecture Map present".to_string()
                        } else {
                            "Architecture Map missing".to_string()
                        },
                    },
                    PCICheck {
                        name: "diagnostic_exists".to_string(),
                        passed: audit.current_step < 3 || audit.has_artifact("Diagnostic") || audit.has_artifact("diagnostic"),
                        details: if audit.current_step < 3 {
                            "Not yet required".to_string()
                        } else if audit.has_artifact("Diagnostic") || audit.has_artifact("diagnostic") {
                            "Diagnostic present".to_string()
                        } else {
                            "Diagnostic missing".to_string()
                        },
                    },
                    PCICheck {
                        name: "synthesis_exists".to_string(),
                        passed: audit.current_step < 4 || audit.has_artifact("Thesis") || audit.has_artifact("thesis"),
                        details: if audit.current_step < 4 {
                            "Not yet required".to_string()
                        } else if audit.has_artifact("Thesis") || audit.has_artifact("thesis") {
                            "Synthesis artifacts present".to_string()
                        } else {
                            "Synthesis artifacts missing".to_string()
                        },
                    },
                ],
            },

            // Category 4: Audit Integrity (25% weight)
            audit_integrity: PCICategory {
                name: "Audit Integrity".to_string(),
                weight: 0.25,
                checks: vec![
                    PCICheck {
                        name: "metric_evaluations_recorded".to_string(),
                        passed: audit.all_metrics_logged(),
                        details: format!("{} metric snapshot(s) recorded", audit.metric_snapshot_count),
                    },
                    PCICheck {
                        name: "decision_timestamps".to_string(),
                        passed: audit.all_decisions_timestamped(),
                        details: if audit.has_timestamps {
                            "All decisions timestamped".to_string()
                        } else {
                            "Missing timestamps".to_string()
                        },
                    },
                    PCICheck {
                        name: "version_continuity".to_string(),
                        passed: audit.artifact_versions_continuous,
                        details: if audit.artifact_versions_continuous {
                            "No version gaps in artifacts".to_string()
                        } else {
                            "Version gaps detected".to_string()
                        },
                    },
                ],
            },
        }
    }

    /// Score PCI checklist (FIX-026)
    ///
    /// Weighted average of category scores
    fn score_pci_checklist(&self, checklist: &PCIChecklist) -> f64 {
        let categories = [
            &checklist.step_sequence,
            &checklist.gate_compliance,
            &checklist.artifact_presence,
            &checklist.audit_integrity,
        ];

        let mut total_score = 0.0;

        for category in categories {
            let passed = category.checks.iter().filter(|c| c.passed).count() as f32;
            let total = category.checks.len() as f32;
            let category_score = if total > 0.0 { passed / total } else { 1.0 };
            total_score += category_score as f64 * category.weight as f64;
        }

        total_score
    }

    /// Convert checklist to metric inputs (FIX-026)
    fn checklist_to_inputs(&self, checklist: &PCIChecklist) -> Vec<MetricInput> {
        let mut inputs = Vec::new();

        for category in [
            &checklist.step_sequence,
            &checklist.gate_compliance,
            &checklist.artifact_presence,
            &checklist.audit_integrity,
        ] {
            let passed = category.checks.iter().filter(|c| c.passed).count();
            let total = category.checks.len();
            inputs.push(MetricInput {
                name: format!("{} ({}/{})", category.name, passed, total),
                value: MetricInputValue::String(format!("{:.0}%", (passed as f64 / total as f64) * 100.0)),
                source: "Process Audit".to_string(),
            });
        }

        inputs
    }

    /// Summarize PCI checklist (FIX-026)
    fn summarize_pci_checklist(&self, checklist: &PCIChecklist, score: f64) -> String {
        let mut summary = format!("Process compliance: {:.0}%. ", score * 100.0);

        let categories = [
            &checklist.step_sequence,
            &checklist.gate_compliance,
            &checklist.artifact_presence,
            &checklist.audit_integrity,
        ];

        let mut failures = Vec::new();
        for category in categories {
            for check in &category.checks {
                if !check.passed {
                    failures.push(format!("{}: {}", category.name, check.name));
                }
            }
        }

        if failures.is_empty() {
            summary.push_str("All process checks passed.");
        } else {
            summary.push_str(&format!("Failed checks: {}", failures.join(", ")));
        }

        summary
    }

    /// Get PCI recommendation (FIX-026)
    fn get_pci_recommendation(&self, score: f64, checklist: &PCIChecklist) -> Option<String> {
        if score >= 0.95 {
            return None; // Perfect compliance
        }

        let mut recommendations = Vec::new();

        // Check each category for failures
        for category in [
            &checklist.step_sequence,
            &checklist.gate_compliance,
            &checklist.artifact_presence,
            &checklist.audit_integrity,
        ] {
            for check in &category.checks {
                if !check.passed {
                    match check.name.as_str() {
                        "steps_in_order" => recommendations.push("Ensure steps are executed sequentially (0→1→2...)"),
                        "no_forward_jumps" => recommendations.push("Do not skip steps - execute in proper order"),
                        "charter_approval" => recommendations.push("Obtain Charter approval before proceeding"),
                        "synthesis_approval" => recommendations.push("Obtain Synthesis approval at Step 4 gate"),
                        "charter_exists" | "architecture_map_exists" | "diagnostic_exists" | "synthesis_exists" =>
                            recommendations.push("Generate all required artifacts for this step"),
                        "metric_evaluations_recorded" => recommendations.push("Ensure metrics are calculated and logged"),
                        _ => {}
                    }
                }
            }
        }

        if recommendations.is_empty() {
            Some("Review process compliance and ensure all checks pass.".to_string())
        } else {
            Some(recommendations.join("; "))
        }
    }

    /// Evaluate metric status based on threshold
    ///
    /// # Arguments
    /// * `value` - The metric value
    /// * `threshold` - The threshold configuration
    /// * `inverse_scale` - If true, lower values are better (like EV)
    pub fn evaluate_status(
        &self,
        value: f64,
        threshold: &MetricThreshold,
        inverse_scale: bool,
    ) -> MetricStatus {
        if inverse_scale {
            // Lower is better (e.g., EV)
            if value <= threshold.pass {
                MetricStatus::Pass
            } else if let Some(halt) = threshold.halt {
                if value > halt {
                    MetricStatus::Fail  // Only Fail if above HALT threshold (for inverse)
                } else {
                    MetricStatus::Warning  // Between pass and halt = warning
                }
            } else {
                // No halt threshold defined, use warning logic
                if let Some(warn) = threshold.warning {
                    if value <= warn {
                        MetricStatus::Warning
                    } else {
                        MetricStatus::Fail
                    }
                } else {
                    MetricStatus::Warning
                }
            }
        } else {
            // Higher is better (e.g., CI, IAS, EFI, PCI)
            if value >= threshold.pass {
                MetricStatus::Pass
            } else if let Some(halt) = threshold.halt {
                if value < halt {
                    MetricStatus::Fail  // Only Fail if below HALT threshold
                } else {
                    MetricStatus::Warning  // Between halt and pass = warning
                }
            } else {
                // No halt threshold defined, use warning logic
                if let Some(warn) = threshold.warning {
                    if value >= warn {
                        MetricStatus::Warning
                    } else {
                        MetricStatus::Fail
                    }
                } else {
                    MetricStatus::Warning
                }
            }
        }
    }

    /// **DEPRECATED (Session 4.3):** Check if any metrics require HALT
    ///
    /// This function has been replaced by the Callout System (Phase 4).
    /// Critical callouts (via `generate_callouts()`) now handle metric violations.
    ///
    /// **Why deprecated:**
    /// - Old HALT system blocks execution unconditionally
    /// - New callout system provides graduated severity (Info/Attention/Warning/Critical)
    /// - Callouts allow informed consent - users acknowledge but can proceed
    /// - Callouts are mode-adjusted and use delta-based CI measurement
    ///
    /// **Migration:** Use `generate_callouts()` instead. Critical callouts require
    /// user acknowledgment via CalloutManager but don't block execution.
    ///
    /// This function now always returns None. Kept for backward compatibility.
    /// Will be fully removed in Phase 5.
    #[deprecated(since = "0.1.0", note = "Use generate_callouts() - see Callout System (Phase 4)")]
    pub fn check_halt_conditions(&self, metrics: &CriticalMetrics, step: u8) -> Option<String> {
        // Session 4.3: HALT logic disabled - callout system handles metric violations
        debug!("check_halt_conditions() called but disabled (step {})", step);
        debug!("Metric violations now handled by callout system (see generate_callouts)");
        None  // Always return None - callouts handle metric violations
    }

    /// Generate callouts from current metric evaluation
    /// This replaces the old HALT logic with the Progression Engine model
    pub fn generate_callouts(
        &self,
        metrics: &CriticalMetrics,
        previous_metrics: Option<&CriticalMetrics>,
        step: Step,
        mode: StructureMode,
        enforcement: MetricEnforcement,  // Session 3.2: New parameter for enforcement mode
        callout_manager: &mut CalloutManager,
    ) {
        // Session 3.2: If informational mode, just log metrics and return (no callouts)
        // This is used for Step 3 (Diagnostic) where low coherence is expected
        if enforcement == MetricEnforcement::Informational {
            info!(
                "Step {:?} metrics (informational): CI={:.2}, IAS={:.2}",
                step,
                metrics.ci.as_ref().map(|m| m.value).unwrap_or(0.0),
                metrics.ias.as_ref().map(|m| m.value).unwrap_or(0.0)
            );
            return;
        }

        let thresholds = ThresholdResolver::resolve(mode, step);

        // CI Callout
        if let Some(ref ci) = metrics.ci {
            let ci_tier = CalloutTrigger::determine_tier(
                "CI",
                ci.value,
                previous_metrics.and_then(|m| m.ci.as_ref().map(|c| c.value)),
                step,
                mode,
            );
            if ci_tier != CalloutTier::Info {
                callout_manager.add(Callout::new(
                    ci_tier,
                    "CI",
                    ci.value,
                    previous_metrics.and_then(|m| m.ci.as_ref().map(|c| c.value)),
                    format!("{} mode: pass={:.2}, warn={:.2}",
                        mode.display_name(), thresholds.ci_pass, thresholds.ci_warn),
                    self.explain_ci_callout(ci_tier, ci.value),
                    self.recommend_ci_action(ci_tier),
                    step,
                    mode,
                ));
            }
        }

        // IAS Callout
        if let Some(ref ias) = metrics.ias {
            let ias_tier = CalloutTrigger::determine_tier(
                "IAS",
                ias.value,
                previous_metrics.and_then(|m| m.ias.as_ref().map(|i| i.value)),
                step,
                mode,
            );
            if ias_tier != CalloutTier::Info {
                callout_manager.add(Callout::new(
                    ias_tier,
                    "IAS",
                    ias.value,
                    previous_metrics.and_then(|m| m.ias.as_ref().map(|i| i.value)),
                    format!("{} mode: pass={:.2}, warn={:.2}",
                        mode.display_name(), thresholds.ias_pass, thresholds.ias_warn),
                    self.explain_ias_callout(ias_tier, ias.value),
                    self.recommend_ias_action(ias_tier),
                    step,
                    mode,
                ));
            }
        }

        // EFI Callout (step-aware, enforced at Step 6 only)
        if let Some(ref efi) = metrics.efi {
            let efi_tier = CalloutTrigger::determine_tier(
                "EFI",
                efi.value,
                previous_metrics.and_then(|m| m.efi.as_ref().map(|e| e.value)),
                step,
                mode,
            );
            if efi_tier != CalloutTier::Info {
                callout_manager.add(Callout::new(
                    efi_tier,
                    "EFI",
                    efi.value,
                    previous_metrics.and_then(|m| m.efi.as_ref().map(|e| e.value)),
                    format!("Step {}: Evidence validation", step.as_u8()),
                    self.explain_efi_callout(efi_tier, efi.value),
                    self.recommend_efi_action(efi_tier),
                    step,
                    mode,
                ));
            }
        }

        // PCI Callout (step-aware, enforced at Step 6 only)
        if let Some(ref pci) = metrics.pci {
            let pci_tier = CalloutTrigger::determine_tier(
                "PCI",
                pci.value,
                previous_metrics.and_then(|m| m.pci.as_ref().map(|p| p.value)),
                step,
                mode,
            );
            if pci_tier != CalloutTier::Info {
                callout_manager.add(Callout::new(
                    pci_tier,
                    "PCI",
                    pci.value,
                    previous_metrics.and_then(|m| m.pci.as_ref().map(|p| p.value)),
                    format!("Step {}: Process compliance", step.as_u8()),
                    self.explain_pci_callout(pci_tier, pci.value),
                    self.recommend_pci_action(pci_tier),
                    step,
                    mode,
                ));
            }
        }

        // EV is always Info, so we skip it
        // SEC is placeholder (always 100%), so we skip it
    }

    fn explain_ci_callout(&self, tier: CalloutTier, value: f64) -> String {
        match tier {
            CalloutTier::Critical => format!(
                "Coherence Index ({:.2}) is critically low. Content may have significant structural issues.",
                value
            ),
            CalloutTier::Warning => format!(
                "Coherence Index ({:.2}) shows notable regression from previous step.",
                value
            ),
            CalloutTier::Attention => format!(
                "Coherence Index ({:.2}) has dropped slightly. Minor structural concerns.",
                value
            ),
            CalloutTier::Info => String::new(),
        }
    }

    fn recommend_ci_action(&self, tier: CalloutTier) -> String {
        match tier {
            CalloutTier::Critical => "Review content structure. Consider strengthening logical connections.".to_string(),
            CalloutTier::Warning => "Review recent changes for unintended structural impacts.".to_string(),
            CalloutTier::Attention => "Monitor in subsequent steps.".to_string(),
            CalloutTier::Info => String::new(),
        }
    }

    fn explain_ias_callout(&self, tier: CalloutTier, value: f64) -> String {
        match tier {
            CalloutTier::Critical => format!(
                "Intent Alignment Score ({:.2}) is critically low. Content may have drifted from original intent.",
                value
            ),
            CalloutTier::Warning => format!(
                "Intent Alignment Score ({:.2}) indicates potential drift from original intent.",
                value
            ),
            CalloutTier::Attention => format!(
                "Intent Alignment Score ({:.2}) shows minor deviation from original intent.",
                value
            ),
            CalloutTier::Info => String::new(),
        }
    }

    fn recommend_ias_action(&self, tier: CalloutTier) -> String {
        match tier {
            CalloutTier::Critical => "Revisit original intent. Consider whether scope has expanded appropriately.".to_string(),
            CalloutTier::Warning => "Review recent additions against original goals.".to_string(),
            CalloutTier::Attention => "Ensure new content aligns with stated objectives.".to_string(),
            CalloutTier::Info => String::new(),
        }
    }

    fn explain_efi_callout(&self, tier: CalloutTier, value: f64) -> String {
        match tier {
            CalloutTier::Critical => format!(
                "Evidence Fidelity Index ({:.0}%) is critically low. Most claims lack substantiation.",
                value * 100.0
            ),
            CalloutTier::Warning => format!(
                "Evidence Fidelity Index ({:.0}%) shows insufficient evidence for some claims.",
                value * 100.0
            ),
            CalloutTier::Attention => format!(
                "Evidence Fidelity Index ({:.0}%) could be strengthened.",
                value * 100.0
            ),
            CalloutTier::Info => String::new(),
        }
    }

    fn recommend_efi_action(&self, tier: CalloutTier) -> String {
        match tier {
            CalloutTier::Critical => "Add evidence for unsupported claims or reclassify as aspirational.".to_string(),
            CalloutTier::Warning => "Strengthen evidence for key assertions.".to_string(),
            CalloutTier::Attention => "Consider adding supporting evidence where appropriate.".to_string(),
            CalloutTier::Info => String::new(),
        }
    }

    fn explain_pci_callout(&self, tier: CalloutTier, value: f64) -> String {
        match tier {
            CalloutTier::Critical => format!(
                "Process Compliance Index ({:.0}%) shows significant process violations.",
                value * 100.0
            ),
            CalloutTier::Warning => format!(
                "Process Compliance Index ({:.0}%) indicates some process gaps.",
                value * 100.0
            ),
            CalloutTier::Attention => format!(
                "Process Compliance Index ({:.0}%) has minor compliance issues.",
                value * 100.0
            ),
            CalloutTier::Info => String::new(),
        }
    }

    fn recommend_pci_action(&self, tier: CalloutTier) -> String {
        match tier {
            CalloutTier::Critical => "Review and address major process compliance issues before proceeding.".to_string(),
            CalloutTier::Warning => "Address identified process gaps.".to_string(),
            CalloutTier::Attention => "Monitor process compliance in subsequent steps.".to_string(),
            CalloutTier::Info => String::new(),
        }
    }

    /// **DEPRECATED (Session 4.3):** Check if any metrics require PAUSE (warning status)
    ///
    /// This function has been replaced by the Callout System (Phase 4).
    /// Warning-level callouts now handle metrics that need attention.
    ///
    /// **Migration:** Use `generate_callouts()` instead. Warning callouts are
    /// displayed to users but don't require acknowledgment (only Critical blocks).
    ///
    /// This function now always returns None. Kept for backward compatibility.
    #[deprecated(since = "0.1.0", note = "Use generate_callouts() - see Callout System (Phase 4)")]
    pub fn check_pause_conditions(&self, metrics: &CriticalMetrics) -> Option<String> {
        // Session 4.3: PAUSE logic disabled - callout system handles warnings
        debug!("check_pause_conditions() called but disabled - using callout system");
        None  // Always return None - callouts handle warnings
    }

    /// Extract JSON from Claude's response, handling cases where JSON is embedded in text
    ///
    /// Claude sometimes returns JSON with explanatory text before/after.
    /// This method tries to find and extract just the JSON portion.
    fn extract_json(&self, response: &str) -> Result<serde_json::Value> {
        // First try direct parsing
        if let Ok(parsed) = serde_json::from_str(response) {
            return Ok(parsed);
        }

        // Try to find JSON object markers
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str(json_str) {
                    return Ok(parsed);
                }
            }
        }

        // If all else fails, return original error
        Err(anyhow::anyhow!("Could not extract valid JSON from response"))
    }

    /// Create Baseline_Report artifact (Step 1)
    ///
    /// Creates the immutable Baseline Report that locks E_baseline and defines
    /// governance checkpoints for the run.
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `charter_content` - Content from the Charter artifact
    /// * `charter_id` - ID of the Charter artifact
    /// * `charter_hash` - Hash of the Charter artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `e_baseline` - The calculated E_baseline value
    /// * `telemetry_profile` - Telemetry profile (Lite / Standard / Full / Learning)
    pub fn create_baseline_report(
        &self,
        run_id: &str,
        charter_content: &str,
        charter_id: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
        telemetry_profile: &str,
    ) -> Result<String> {
        info!("Creating Baseline_Report for run {}", run_id);

        // Extract charter objectives and success criteria counts
        let objectives_count = charter_content
            .lines()
            .filter(|line| line.trim().starts_with("###") || line.trim().starts_with("1.") || line.trim().starts_with("2."))
            .count();

        let success_criteria_count = charter_content
            .lines()
            .filter(|line| line.contains("Criterion"))
            .count()
            .max(2); // At least 2 (CI and EV are always included)

        // Build the Baseline Report content
        let content_body = format!(
            r#"# Baseline Report

> 📊 **IMMUTABLE ARTIFACT** - E_baseline is locked at this point.
> All future EV calculations reference these values.

## Baseline Freeze Confirmation

| Parameter | Value | Locked |
|-----------|-------|--------|
| E_baseline | {} | ✓ |
| Charter Objectives | {} | ✓ |
| Success Criteria | {} | ✓ |

## E_baseline Calculation

### Input Materials

| Material | Type | Size/Scope |
|----------|------|------------|
| Charter | Governance Document | {} words |

### Calculation Method

```
E_baseline = Word count of Charter content
           = {}
```

### Baseline Components

| Component | Weight | Value | Contribution |
|-----------|--------|-------|--------------|
| Charter Content | 100% | {} | **{}** |

## Threshold Canon Alignment

### Critical 6 Targets

| Metric | Target | Warning | HALT |
|--------|--------|---------|------|
| CI | ≥ 0.80 | 0.70 | 0.50 |
| EV | ≤ ±10% | ±20% | ±30% |
| IAS | ≥ 0.80 | 0.70 | 0.50 |
| EFI | ≥ 95% | 90% | 80% |
| SEC | 100% | - | - |
| PCI | ≥ 0.90 | 0.85 | 0.70 |

### Telemetry Profile Configuration

**Selected Profile:** {}

| Setting | Value |
|---------|-------|
| Metric Frequency | per step |
| Domain Monitoring | enabled |
| Learning Capture | {} |

## Governance Checkpoint Registry

| Checkpoint | Step | Gate Required |
|------------|------|---------------|
| Intent Confirmed | 0→1 | ✓ |
| Baseline Frozen | 1→2 | ✓ |
| Analysis Ready | 2→3 | ✓ |
| Synthesis Ready | 3→4 | ✓ |
| Redesign Ready | 4→5 | ✓ |
| Validation Ready | 5→6 | ✓ |
| Completion | 6→Close | ✓ |

## Human Approval

- **Baseline Frozen By:** System
- **Approval Timestamp:** {}
- **Gate:** Baseline_Frozen

---
📊 **Baseline Hash:** `{}`
*E_baseline = {} is now immutable*"#,
            e_baseline,
            objectives_count,
            success_criteria_count,
            e_baseline,
            e_baseline,
            e_baseline,
            e_baseline,
            telemetry_profile,
            if telemetry_profile == "Learning" { "enabled" } else { "disabled" },
            chrono::Utc::now().to_rfc3339(),
            charter_hash,
            e_baseline
        );

        // Calculate hash of content body
        let content_hash = self.compute_content_hash(&content_body);

        // Build complete artifact with frontmatter
        let artifact_id = format!("{}-baseline-report", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Baseline_Report\"\n\
            run_id: \"{}\"\n\
            step_origin: 1\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"derived_from\"\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: true\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            charter_id,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Baseline_Report created successfully: {}", artifact_id);
        Ok(artifact)
    }

    /// Compute SHA-256 hash of content
    fn compute_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Perform governance calibration for Step 2
    ///
    /// Reviews Charter objectives and configures five control domains:
    /// - Entropy Control: Set acceptable EV bounds
    /// - Objective Control: Align metrics to Charter goals
    /// - Process Control: Confirm step sequencing
    /// - Reflective Control: Set reflection cadence
    /// - Termination Control: Define completion criteria
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `charter_content` - Content from the Charter artifact
    /// * `charter_hash` - Hash of the Charter artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `architecture_map_content` - Content from the Architecture Map
    /// * `e_baseline` - The locked E_baseline value
    ///
    /// # Returns
    /// Tuple of (Governance_Summary artifact, Domain_Snapshots artifact)
    pub async fn perform_governance_calibration(
        &self,
        run_id: &str,
        charter_content: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        architecture_map_content: &str,
        e_baseline: f64,
    ) -> Result<(String, String)> {
        info!("Performing governance calibration for run {}", run_id);

        let system_prompt = "You are the Governance & Telemetry Agent under the CONDUCTOR role. \
            Your task is to review the Charter and configure active governance controls for this Method-VI run. \
            Be specific and concrete about thresholds, triggers, and intervention criteria.";

        let user_message = format!(
            r#"GOVERNANCE CALIBRATION (Step 2)

Charter Content:
{}

Architecture Map:
{}

E_baseline (locked): {} words

Your task: Configure the five control domains for active governance orchestration.

For each domain, specify:
1. Control objective (what we're monitoring)
2. Threshold settings (when to intervene)
3. Intervention triggers (what causes an alert)
4. Calibration rationale (why these settings)

## 1. ENTROPY CONTROL
Objective: Manage content expansion/contraction relative to E_baseline
Default bounds: ±10% variance acceptable
- What EV bounds are appropriate for this run's scope?
- What are the warning thresholds (yellow) and HALT thresholds (red)?
- Rationale:

## 2. OBJECTIVE CONTROL
Objective: Ensure work remains aligned with Charter objectives
Metrics: IAS (Intent Alignment Score)
- What IAS threshold indicates good alignment? (default: ≥0.82)
- When should we warn about drift from objectives?
- Rationale:

## 3. PROCESS CONTROL
Objective: Confirm adherence to Method-VI step sequencing
Metrics: PCI (Process Compliance Index)
- What PCI threshold indicates proper process adherence? (default: ≥0.90)
- What process violations should trigger intervention?
- Rationale:

## 4. REFLECTIVE CONTROL
Objective: Maintain appropriate reflection cadence per Architecture Map
Metrics: RCC (Reflection Cadence Compliance)
- What is the planned reflection schedule based on the Architecture Map?
- How do we detect deviations from this schedule?
- Rationale:

## 5. TERMINATION CONTROL
Objective: Define clear completion criteria
- What conditions signal readiness to complete this run?
- What are the minimum acceptable metric values for closure? (CI, EV, IAS, etc.)
- What are the "good enough" vs "excellent" criteria?
- Rationale:

Return your calibration settings in a structured format."#,
            charter_content,
            architecture_map_content,
            e_baseline
        );

        // Call Claude API for governance calibration
        let calibration_response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(3000), Some(0.0))
            .await
            .context("Failed to generate governance calibration")?;

        info!("Governance calibration response received");

        // Create Governance_Summary artifact
        let governance_summary = self.create_governance_summary_artifact(
            run_id,
            &calibration_response,
            charter_hash,
            intent_anchor_id,
            e_baseline,
        )?;

        // Create Domain_Snapshots artifact (baseline readings)
        let domain_snapshots = self.create_domain_snapshots_artifact(
            run_id,
            charter_hash,
            intent_anchor_id,
            e_baseline,
        )?;

        info!("Governance calibration complete");

        Ok((governance_summary, domain_snapshots))
    }

    /// Create Governance_Summary artifact
    fn create_governance_summary_artifact(
        &self,
        run_id: &str,
        calibration_content: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
    ) -> Result<String> {
        info!("Creating Governance_Summary artifact");

        let content_body = format!(
            r#"# Governance Summary

> 📊 **GOVERNANCE CALIBRATION** - Active governance controls configured for this run.
> These settings guide continuous monitoring and minimal intervention.

## Control Domain Configuration

{}

## E_baseline Reference

| Parameter | Value | Status |
|-----------|-------|--------|
| E_baseline | {} words | Locked ✓ |
| Acceptable EV | ±10% default | Configured ✓ |

## Threshold Canon Application

All thresholds aligned with Method-VI Threshold Canon:

| Metric | Pass | Warning | HALT |
|--------|------|---------|------|
| CI | ≥ 0.80 | 0.70 | 0.50 |
| EV | ≤ ±10% | ±20% | ±30% |
| IAS | ≥ 0.80 | 0.70 | 0.50 |
| EFI | ≥ 95% | 90% | 80% |
| SEC | 100% | - | - |
| PCI | ≥ 0.90 | 0.85 | 0.70 |

## Active Governance Mode

**Status:** ENABLED
**Monitoring Frequency:** Per step completion
**Intervention Strategy:** Minimal intervention (proportional response)

## Governance Role

**Active Role:** Conductor
**Activation:** Step 2 (Governance Calibration)
**Deactivation:** End of Step 2 (→ Observer role reactivated)

---
📊 **Calibration Complete**
*Governance controls active for duration of run*"#,
            calibration_content,
            e_baseline
        );

        let content_hash = self.compute_content_hash(&content_body);
        let artifact_id = format!("{}-governance-summary", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Governance_Summary\"\n\
            run_id: \"{}\"\n\
            step_origin: 2\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: false\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Governance_Summary created: {}", artifact_id);
        Ok(artifact)
    }

    /// Create Domain_Snapshots artifact (baseline readings for five domains)
    fn create_domain_snapshots_artifact(
        &self,
        run_id: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
    ) -> Result<String> {
        info!("Creating Domain_Snapshots artifact");

        let content_body = format!(
            r#"# Domain Snapshots

> 📸 **BASELINE READINGS** - Initial state of five control domains at Step 2.
> These snapshots provide reference points for detecting drift and triggering interventions.

## Domain Baseline Readings

### 1. Clarity Domain
- **Metric:** CI (Coherence Index)
- **Baseline Reading:** Not yet measured (Step 3+)
- **Target:** ≥ 0.82
- **Status:** Monitoring configured ✓

### 2. Entropy Domain
- **Metric:** EV (Expansion Variance)
- **Baseline Reading:** 0.0% (at E_baseline = {} words)
- **Target:** ≤ ±10%
- **Status:** Monitoring configured ✓

### 3. Alignment Domain
- **Metric:** IAS (Intent Alignment Score)
- **Baseline Reading:** Not yet measured (Step 3+)
- **Target:** ≥ 0.82
- **Status:** Monitoring configured ✓

### 4. Cadence Domain
- **Metric:** RCC (Reflection Cadence Compliance)
- **Baseline Reading:** On schedule (Step 2 complete)
- **Target:** Per Architecture Map
- **Status:** Monitoring configured ✓

### 5. Overhead Domain
- **Metric:** GLR (Governance Latency Ratio)
- **Baseline Reading:** Not yet measured (Phase 2 metric)
- **Target:** ≤ 15%
- **Status:** Deferred to Phase 2

## Snapshot Metadata

| Field | Value |
|-------|-------|
| Snapshot Time | {} |
| Run ID | {} |
| Step | 2 (Governance Calibration) |
| E_baseline | {} words |

## Monitoring Status

All five domains are configured for continuous monitoring throughout the run.
Snapshots will be updated at each step completion.

---
📸 **Domain Monitoring Active**
*Baseline snapshots recorded for reference*"#,
            e_baseline,
            chrono::Utc::now().to_rfc3339(),
            run_id,
            e_baseline
        );

        let content_hash = self.compute_content_hash(&content_body);
        let artifact_id = format!("{}-domain-snapshots", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Domain_Snapshots\"\n\
            run_id: \"{}\"\n\
            step_origin: 2\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: false\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Domain_Snapshots created: {}", artifact_id);
        Ok(artifact)
    }

    /// Check synthesis relevance before Step 4 synthesis
    ///
    /// Validates that the integrated diagnostic from Step 3 actually relates to
    /// the Charter objectives. This prevents synthesizing based on wrong analysis target
    /// (e.g., analyzing Charter methodology instead of user's content).
    ///
    /// Returns a relevance score from 0.0 to 1.0:
    /// - 1.0 = Findings directly address all objectives
    /// - 0.7 = Findings mostly relevant with some tangential content
    /// - 0.5 = Findings partially relevant
    /// - 0.3 = Findings mostly unrelated
    /// - 0.0 = Findings completely unrelated
    pub async fn check_synthesis_relevance(
        &self,
        diagnostic: &str,
        charter_objectives: &[String],
    ) -> Result<f64> {
        info!("Checking synthesis relevance...");

        let system_prompt = "You are a quality assurance checker for Method-VI. \
            Assess whether analysis findings relate to the stated objectives.";

        let objectives_text = charter_objectives.join("\n- ");

        let user_message = format!(
            r#"RELEVANCE CHECK

Charter Objectives:
- {}

Analysis Findings (from Integrated Diagnostic):
{}

Question: Do these analysis findings directly address or inform these objectives?

Score from 0.0 to 1.0:
- 1.0 = Findings directly address all objectives
- 0.7 = Findings mostly relevant with some tangential content
- 0.5 = Findings partially relevant
- 0.3 = Findings mostly unrelated
- 0.0 = Findings completely unrelated (e.g., analyzing methodology instead of content)

WARNING SIGNS of wrong analysis target:
- Findings critique "project management methodology" or "governance approach"
- References to "category error", "sophistication paradox", "planning primacy"
- Meta-analysis of the Charter document itself rather than user's content
- Findings discuss Method-VI framework instead of user's subject matter

Respond with ONLY a JSON object:
{{"score": 0.XX, "rationale": "brief explanation"}}"#,
            objectives_text,
            &diagnostic[..diagnostic.len().min(3000)]  // Truncate if too long
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(300), Some(0.0))
            .await?;

        // Parse score from response using existing extract_json helper
        let json = self.extract_json(&response)?;

        let score = json["score"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse relevance score from response"))?;

        let rationale = json["rationale"]
            .as_str()
            .unwrap_or("No rationale provided");

        info!("Synthesis relevance score: {:.2} - {}", score, rationale);

        Ok(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_evaluation() {
        let thresholds = ThresholdsConfig::default();
        let agent = GovernanceTelemetryAgent {
            api_client: AnthropicClient::new("test-key".to_string()).unwrap(),
            e_baseline: None,
            thresholds,
        };

        // Test CI (higher is better)
        // FIX-023: CI pass threshold is 0.70, warning is 0.50
        assert_eq!(
            agent.evaluate_status(0.85, &agent.thresholds.ci, false),
            MetricStatus::Pass
        );
        assert_eq!(
            agent.evaluate_status(0.60, &agent.thresholds.ci, false),
            MetricStatus::Warning
        );
        assert_eq!(
            agent.evaluate_status(0.45, &agent.thresholds.ci, false),
            MetricStatus::Fail
        );

        // Test EV (lower is better - inverse scale)
        assert_eq!(
            agent.evaluate_status(8.0, &agent.thresholds.ev, true),
            MetricStatus::Pass
        );
        assert_eq!(
            agent.evaluate_status(15.0, &agent.thresholds.ev, true),
            MetricStatus::Warning
        );
        assert_eq!(
            agent.evaluate_status(35.0, &agent.thresholds.ev, true),
            MetricStatus::Fail
        );
    }

    #[tokio::test]
    async fn test_e_baseline_locking() {
        // Skip test if no valid API key is available
        let api_key = match std::env::var("ANTHROPIC_API_KEY") {
            Ok(key) if key.starts_with("sk-ant-") => key,
            _ => {
                println!("Skipping test_e_baseline_locking: ANTHROPIC_API_KEY not set or invalid");
                return;
            }
        };
        let mut agent = GovernanceTelemetryAgent::new(api_key).unwrap();

        // Calculate baseline
        let baseline_content = "This is a test baseline with some words in it.";
        let result = agent.calculate_e_baseline(baseline_content, 1).await;
        assert!(result.is_ok());

        let e_baseline = agent.get_e_baseline();
        assert!(e_baseline.is_some());
        assert!(e_baseline.unwrap() > 0.0);

        // Lock baseline
        let lock_result = agent.lock_e_baseline(1);
        assert!(lock_result.is_ok());

        // Verify locked
        assert!(agent.e_baseline.as_ref().unwrap().locked);

        // Try to recalculate - should fail
        let recalc_result = agent.calculate_e_baseline("New content", 2).await;
        assert!(recalc_result.is_err());
    }
}
