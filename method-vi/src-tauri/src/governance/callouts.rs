use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{Step, StructureMode, Thresholds, ThresholdResolver};

/// Callout severity tiers - replaces binary HALT/PASS with graduated responses
/// Ordered by severity: Info < Attention < Warning < Critical
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CalloutTier {
    /// Green - Normal progression, no concerns
    Info,
    /// Yellow - Minor concern, no action required
    Attention,
    /// Orange - Significant concern, acknowledgment recommended
    /// Note: Auto-downgraded to Attention in Architecting mode (Noise Filter)
    Warning,
    /// Red - Fundamental issue, acknowledgment REQUIRED before proceeding
    /// Never downgraded regardless of mode
    Critical,
}

impl CalloutTier {
    /// Returns true if this tier requires user acknowledgment
    pub fn requires_acknowledgment(&self) -> bool {
        matches!(self, CalloutTier::Critical)
    }

    /// Returns the display color for UI
    pub fn color(&self) -> &'static str {
        match self {
            CalloutTier::Info => "green",
            CalloutTier::Attention => "yellow",
            CalloutTier::Warning => "orange",
            CalloutTier::Critical => "red",
        }
    }

    /// Apply Noise Filter: downgrade Warning to Attention in Architecting mode
    /// Critical is NEVER downgraded
    pub fn apply_noise_filter(self, mode: StructureMode) -> Self {
        if mode.should_downgrade_orange() && self == CalloutTier::Warning {
            CalloutTier::Attention
        } else {
            self
        }
    }
}

/// A callout represents a governance concern at a specific step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Callout {
    /// Unique identifier for this callout
    pub id: String,
    /// Severity tier (after noise filter applied)
    pub tier: CalloutTier,
    /// Original tier before noise filter (for audit trail)
    pub original_tier: CalloutTier,
    /// Which metric triggered this callout
    pub metric_name: String,
    /// Current metric value
    pub current_value: f64,
    /// Previous step's value (for delta display)
    pub previous_value: Option<f64>,
    /// Calculated delta (current - previous)
    pub delta: Option<f64>,
    /// Context about thresholds (e.g., "Architecting mode: pass=0.50, warn=0.35")
    pub threshold_context: String,
    /// Human-readable explanation of the concern
    pub explanation: String,
    /// Recommended action
    pub recommendation: String,
    /// Whether user must acknowledge before proceeding
    pub requires_acknowledgment: bool,
    /// Whether user has acknowledged
    pub acknowledged: bool,
    /// When acknowledged (if applicable)
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Which step generated this callout
    pub step: Step,
    /// Active mode when callout was generated
    pub mode: StructureMode,
    /// When the callout was created
    pub created_at: DateTime<Utc>,
}

impl Callout {
    /// Create a new callout (noise filter applied automatically)
    pub fn new(
        tier: CalloutTier,
        metric_name: impl Into<String>,
        current_value: f64,
        previous_value: Option<f64>,
        threshold_context: impl Into<String>,
        explanation: impl Into<String>,
        recommendation: impl Into<String>,
        step: Step,
        mode: StructureMode,
    ) -> Self {
        let filtered_tier = tier.apply_noise_filter(mode);
        let delta = previous_value.map(|prev| current_value - prev);

        Self {
            id: Uuid::new_v4().to_string(),
            tier: filtered_tier,
            original_tier: tier,
            metric_name: metric_name.into(),
            current_value,
            previous_value,
            delta,
            threshold_context: threshold_context.into(),
            explanation: explanation.into(),
            recommendation: recommendation.into(),
            requires_acknowledgment: filtered_tier.requires_acknowledgment(),
            acknowledged: false,
            acknowledged_at: None,
            step,
            mode,
            created_at: Utc::now(),
        }
    }

    /// Check if this callout was downgraded by noise filter
    pub fn was_downgraded(&self) -> bool {
        self.original_tier != self.tier
    }
}

/// Acknowledgment record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgmentRecord {
    pub callout_id: String,
    pub tier: CalloutTier,
    pub metric_name: String,
    pub value: f64,
    pub user_confirmation: String,
    pub acknowledged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutCountByTier {
    pub info: usize,
    pub attention: usize,
    pub warning: usize,
    pub critical: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutSummary {
    pub total: usize,
    pub by_tier: CalloutCountByTier,
    pub pending_acknowledgments: usize,
    pub can_proceed: bool,
}

/// Manages callouts for a run
#[derive(Debug, Default)]
pub struct CalloutManager {
    callouts: Vec<Callout>,
}

impl CalloutManager {
    pub fn new() -> Self {
        Self { callouts: Vec::new() }
    }

    /// Add a callout to the manager
    pub fn add(&mut self, callout: Callout) {
        self.callouts.push(callout);
    }

    /// Get all callouts requiring acknowledgment that haven't been acknowledged
    pub fn get_pending_acknowledgments(&self) -> Vec<&Callout> {
        self.callouts
            .iter()
            .filter(|c| c.requires_acknowledgment && !c.acknowledged)
            .collect()
    }

    /// Check if we can proceed (no unacknowledged Critical callouts)
    pub fn can_proceed(&self) -> bool {
        self.get_pending_acknowledgments().is_empty()
    }

    /// Acknowledge a callout by ID
    pub fn acknowledge(&mut self, callout_id: &str) -> Result<(), String> {
        let callout = self.callouts
            .iter_mut()
            .find(|c| c.id == callout_id)
            .ok_or_else(|| format!("Callout not found: {}", callout_id))?;

        callout.acknowledged = true;
        callout.acknowledged_at = Some(Utc::now());
        Ok(())
    }

    /// Get all callouts for a specific step
    pub fn get_callouts_for_step(&self, step: Step) -> Vec<&Callout> {
        self.callouts
            .iter()
            .filter(|c| c.step == step)
            .collect()
    }

    /// Get all callouts
    pub fn all(&self) -> &[Callout] {
        &self.callouts
    }

    /// Get count by tier
    pub fn count_by_tier(&self, tier: CalloutTier) -> usize {
        self.callouts.iter().filter(|c| c.tier == tier).count()
    }

    /// Acknowledge a callout with user confirmation message
    /// Returns an AcknowledgmentRecord for ledger logging
    pub fn acknowledge_with_confirmation(
        &mut self,
        callout_id: &str,
        user_confirmation: impl Into<String>
    ) -> Result<AcknowledgmentRecord, String> {
        let confirmation = user_confirmation.into();
        let callout = self.callouts
            .iter_mut()
            .find(|c| c.id == callout_id)
            .ok_or_else(|| format!("Callout not found: {}", callout_id))?;

        let now = Utc::now();
        callout.acknowledged = true;
        callout.acknowledged_at = Some(now);

        Ok(AcknowledgmentRecord {
            callout_id: callout_id.to_string(),
            tier: callout.tier,
            metric_name: callout.metric_name.clone(),
            value: callout.current_value,
            user_confirmation: confirmation,
            acknowledged_at: now,
        })
    }

    /// Acknowledge all pending Critical callouts
    /// Returns records for all acknowledged callouts
    pub fn acknowledge_all_pending(
        &mut self,
        user_confirmation: impl Into<String>
    ) -> Vec<AcknowledgmentRecord> {
        let confirmation = user_confirmation.into();
        let now = Utc::now();
        let mut records = Vec::new();

        for callout in self.callouts.iter_mut() {
            if callout.requires_acknowledgment && !callout.acknowledged {
                callout.acknowledged = true;
                callout.acknowledged_at = Some(now);

                records.push(AcknowledgmentRecord {
                    callout_id: callout.id.clone(),
                    tier: callout.tier,
                    metric_name: callout.metric_name.clone(),
                    value: callout.current_value,
                    user_confirmation: confirmation.clone(),
                    acknowledged_at: now,
                });
            }
        }

        records
    }

    /// Get a summary of current callout state
    pub fn summary(&self) -> CalloutSummary {
        CalloutSummary {
            total: self.callouts.len(),
            by_tier: CalloutCountByTier {
                info: self.count_by_tier(CalloutTier::Info),
                attention: self.count_by_tier(CalloutTier::Attention),
                warning: self.count_by_tier(CalloutTier::Warning),
                critical: self.count_by_tier(CalloutTier::Critical),
            },
            pending_acknowledgments: self.get_pending_acknowledgments().len(),
            can_proceed: self.can_proceed(),
        }
    }
}

/// Determines callout tier based on metric results
/// Uses mode-adjusted thresholds from ThresholdResolver
pub struct CalloutTrigger;

impl CalloutTrigger {
    /// Determine tier for CI (Coherence Index) - DELTA-BASED
    /// CI uses delta from previous step, not absolute value
    pub fn ci_tier(
        current: f64,
        previous: Option<f64>,
        thresholds: &Thresholds,
    ) -> CalloutTier {
        // If no previous value, use absolute thresholds
        let Some(prev) = previous else {
            return Self::absolute_tier(current, thresholds.ci_pass, thresholds.ci_warn, thresholds.ci_critical);
        };

        let delta = current - prev;

        // Delta-based tiers (negative delta = regression)
        if delta >= 0.0 || delta > -0.10 {
            CalloutTier::Info
        } else if delta > -0.20 {
            CalloutTier::Attention
        } else if delta > -0.30 {
            CalloutTier::Warning
        } else {
            CalloutTier::Critical
        }
    }

    /// Determine tier for IAS (Intent Alignment Score) - ABSOLUTE
    /// IAS uses mode-adjusted absolute thresholds
    pub fn ias_tier(value: f64, thresholds: &Thresholds) -> CalloutTier {
        Self::absolute_tier(value, thresholds.ias_pass, thresholds.ias_warn, thresholds.ias_critical)
    }

    /// Determine tier for EFI (Evidence Fidelity Index) - STEP-AWARE
    /// EFI is informational before Step 4, enforced at Step 6 only
    pub fn efi_tier(value: f64, step: Step) -> CalloutTier {
        // Steps 0-3: Always informational (diagnostic phase)
        if step.as_u8() < 4 {
            return CalloutTier::Info;
        }

        // Step 4-5: Soft awareness
        if step.as_u8() < 6 {
            if value >= 0.80 {
                CalloutTier::Info
            } else if value >= 0.60 {
                CalloutTier::Attention
            } else {
                CalloutTier::Warning
            }
        } else {
            // Step 6: Full enforcement
            if value >= 0.80 {
                CalloutTier::Info
            } else if value >= 0.60 {
                CalloutTier::Attention
            } else if value >= 0.50 {
                CalloutTier::Warning
            } else {
                CalloutTier::Critical
            }
        }
    }

    /// Determine tier for PCI (Process Compliance Index) - STEP-AWARE
    /// PCI is monitored throughout, enforced strictly at Step 6
    pub fn pci_tier(value: f64, step: Step) -> CalloutTier {
        if step.as_u8() < 6 {
            // Pre-validation: softer thresholds
            if value >= 0.85 {
                CalloutTier::Info
            } else if value >= 0.70 {
                CalloutTier::Attention
            } else {
                CalloutTier::Warning
            }
        } else {
            // Step 6: Strict validation
            if value >= 0.95 {
                CalloutTier::Info
            } else if value >= 0.85 {
                CalloutTier::Attention
            } else if value >= 0.70 {
                CalloutTier::Warning
            } else {
                CalloutTier::Critical
            }
        }
    }

    /// Determine tier for EV (Expansion Variance) - ALWAYS INFORMATIONAL
    /// EV is never enforced, only logged for calibration
    pub fn ev_tier(_value: f64) -> CalloutTier {
        CalloutTier::Info
    }

    /// Determine tier for SEC (Scope Expansion Count) - PLACEHOLDER
    /// SEC is always pass in MVP
    pub fn sec_tier(value: f64) -> CalloutTier {
        if value >= 100.0 {
            CalloutTier::Info
        } else if value >= 90.0 {
            CalloutTier::Attention
        } else {
            CalloutTier::Warning
        }
    }

    /// Helper for absolute threshold comparison
    fn absolute_tier(value: f64, pass: f64, warn: f64, critical: f64) -> CalloutTier {
        if value >= pass {
            CalloutTier::Info
        } else if value >= warn {
            CalloutTier::Attention
        } else if value >= critical {
            CalloutTier::Warning
        } else {
            CalloutTier::Critical
        }
    }

    /// Main entry point: determine tier for any metric
    /// Applies noise filter automatically based on mode
    pub fn determine_tier(
        metric_name: &str,
        current_value: f64,
        previous_value: Option<f64>,
        step: Step,
        mode: StructureMode,
    ) -> CalloutTier {
        let thresholds = ThresholdResolver::resolve(mode, step);

        let raw_tier = match metric_name {
            "CI" => Self::ci_tier(current_value, previous_value, &thresholds),
            "IAS" => Self::ias_tier(current_value, &thresholds),
            "EFI" => Self::efi_tier(current_value, step),
            "PCI" => Self::pci_tier(current_value, step),
            "EV" => Self::ev_tier(current_value),
            "SEC" => Self::sec_tier(current_value),
            _ => CalloutTier::Info, // Unknown metrics default to Info
        };

        // Apply noise filter (Constraint 3)
        raw_tier.apply_noise_filter(mode)
    }

    /// Determine tier with enforcement mode check (Session 3.1)
    /// Returns Info for all metrics when enforcement is Informational
    /// This allows Step 3 (Diagnostic) to record metrics without generating callouts
    pub fn determine_tier_with_enforcement(
        metric_name: &str,
        current_value: f64,
        previous_value: Option<f64>,
        step: Step,
        mode: StructureMode,
        enforcement: super::types::MetricEnforcement,
    ) -> CalloutTier {
        // If enforcement is Informational, always return Info (no callouts)
        if enforcement == super::types::MetricEnforcement::Informational {
            return CalloutTier::Info;
        }

        // Otherwise, use normal tier determination
        Self::determine_tier(metric_name, current_value, previous_value, step, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callout_tier_requires_acknowledgment() {
        assert!(!CalloutTier::Info.requires_acknowledgment());
        assert!(!CalloutTier::Attention.requires_acknowledgment());
        assert!(!CalloutTier::Warning.requires_acknowledgment());
        assert!(CalloutTier::Critical.requires_acknowledgment());
    }

    #[test]
    fn test_noise_filter_downgrades_warning_in_architecting() {
        let warning = CalloutTier::Warning;
        let filtered = warning.apply_noise_filter(StructureMode::Architecting);
        assert_eq!(filtered, CalloutTier::Attention);
    }

    #[test]
    fn test_noise_filter_preserves_warning_in_other_modes() {
        let warning = CalloutTier::Warning;
        assert_eq!(warning.apply_noise_filter(StructureMode::Builder), CalloutTier::Warning);
        assert_eq!(warning.apply_noise_filter(StructureMode::Refining), CalloutTier::Warning);
    }

    #[test]
    fn test_noise_filter_never_downgrades_critical() {
        let critical = CalloutTier::Critical;
        assert_eq!(critical.apply_noise_filter(StructureMode::Architecting), CalloutTier::Critical);
        assert_eq!(critical.apply_noise_filter(StructureMode::Builder), CalloutTier::Critical);
    }

    #[test]
    fn test_callout_creation_applies_noise_filter() {
        let callout = Callout::new(
            CalloutTier::Warning,
            "CI",
            0.45,
            Some(0.50),
            "Architecting mode",
            "CI dropped",
            "Review content",
            Step::Step4_Synthesis,
            StructureMode::Architecting,
        );

        assert_eq!(callout.tier, CalloutTier::Attention); // Downgraded
        assert_eq!(callout.original_tier, CalloutTier::Warning); // Original preserved
        assert!(callout.was_downgraded());
        assert!(!callout.requires_acknowledgment); // Attention doesn't require ack
    }

    #[test]
    fn test_callout_delta_calculation() {
        let callout = Callout::new(
            CalloutTier::Info,
            "CI",
            0.65,
            Some(0.60),
            "Builder mode",
            "CI improved",
            "Continue",
            Step::Step4_Synthesis,
            StructureMode::Builder,
        );

        // Use approximate comparison for floating point
        assert!(callout.delta.is_some());
        let delta = callout.delta.unwrap();
        assert!((delta - 0.05).abs() < 0.0001);
    }

    #[test]
    fn test_callout_manager_can_proceed() {
        let mut manager = CalloutManager::new();

        // Info callout - can proceed
        manager.add(Callout::new(
            CalloutTier::Info, "CI", 0.80, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        assert!(manager.can_proceed());

        // Add Critical - cannot proceed
        manager.add(Callout::new(
            CalloutTier::Critical, "IAS", 0.25, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        assert!(!manager.can_proceed());
    }

    #[test]
    fn test_callout_manager_acknowledge() {
        let mut manager = CalloutManager::new();

        let callout = Callout::new(
            CalloutTier::Critical, "CI", 0.15, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        );
        let callout_id = callout.id.clone();
        manager.add(callout);

        assert!(!manager.can_proceed());

        manager.acknowledge(&callout_id).unwrap();

        assert!(manager.can_proceed());
    }

    #[test]
    fn test_callout_manager_get_by_step() {
        let mut manager = CalloutManager::new();

        manager.add(Callout::new(
            CalloutTier::Info, "CI", 0.80, None, "", "", "",
            Step::Step3_Diagnostic, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Attention, "IAS", 0.65, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Info, "CI", 0.85, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));

        assert_eq!(manager.get_callouts_for_step(Step::Step3_Diagnostic).len(), 1);
        assert_eq!(manager.get_callouts_for_step(Step::Step4_Synthesis).len(), 2);
    }

    // =========================================================================
    // CALLOUT TRIGGER TESTS
    // =========================================================================

    #[test]
    fn test_ci_delta_tiers() {
        let thresholds = Thresholds {
            ci_pass: 0.65, ci_warn: 0.50, ci_critical: 0.35,
            ias_pass: 0.65, ias_warn: 0.50, ias_critical: 0.35,
        };

        // Improvement or small drop -> Info
        assert_eq!(CalloutTrigger::ci_tier(0.70, Some(0.65), &thresholds), CalloutTier::Info);
        assert_eq!(CalloutTrigger::ci_tier(0.60, Some(0.65), &thresholds), CalloutTier::Info);

        // Moderate drop -> Attention
        assert_eq!(CalloutTrigger::ci_tier(0.50, Some(0.65), &thresholds), CalloutTier::Attention);

        // Significant drop -> Warning
        assert_eq!(CalloutTrigger::ci_tier(0.40, Some(0.65), &thresholds), CalloutTier::Warning);

        // Severe drop -> Critical
        assert_eq!(CalloutTrigger::ci_tier(0.30, Some(0.65), &thresholds), CalloutTier::Critical);
    }

    #[test]
    fn test_ias_absolute_tiers() {
        let thresholds = Thresholds {
            ci_pass: 0.65, ci_warn: 0.50, ci_critical: 0.35,
            ias_pass: 0.65, ias_warn: 0.50, ias_critical: 0.35,
        };

        assert_eq!(CalloutTrigger::ias_tier(0.70, &thresholds), CalloutTier::Info);
        assert_eq!(CalloutTrigger::ias_tier(0.55, &thresholds), CalloutTier::Attention);
        assert_eq!(CalloutTrigger::ias_tier(0.40, &thresholds), CalloutTier::Warning);
        assert_eq!(CalloutTrigger::ias_tier(0.20, &thresholds), CalloutTier::Critical);
    }

    #[test]
    fn test_efi_step_aware() {
        // Step 3: Always Info (diagnostic phase)
        assert_eq!(CalloutTrigger::efi_tier(0.30, Step::Step3_Diagnostic), CalloutTier::Info);

        // Step 5: Warning but not Critical
        assert_eq!(CalloutTrigger::efi_tier(0.30, Step::Step5_Redesign), CalloutTier::Warning);

        // Step 6: Full enforcement
        assert_eq!(CalloutTrigger::efi_tier(0.30, Step::Step6_Validation), CalloutTier::Critical);
    }

    #[test]
    fn test_pci_step_aware() {
        // Pre-Step 6: Softer thresholds
        assert_eq!(CalloutTrigger::pci_tier(0.80, Step::Step4_Synthesis), CalloutTier::Attention);

        // Step 6: Strict thresholds
        assert_eq!(CalloutTrigger::pci_tier(0.80, Step::Step6_Validation), CalloutTier::Warning);
        assert_eq!(CalloutTrigger::pci_tier(0.60, Step::Step6_Validation), CalloutTier::Critical);
    }

    #[test]
    fn test_ev_always_info() {
        assert_eq!(CalloutTrigger::ev_tier(500.0), CalloutTier::Info);
        assert_eq!(CalloutTrigger::ev_tier(-100.0), CalloutTier::Info);
    }

    #[test]
    fn test_determine_tier_with_noise_filter() {
        // Warning in Architecting -> downgraded to Attention
        let tier = CalloutTrigger::determine_tier(
            "IAS",
            0.40, // Would be Warning
            None,
            Step::Step4_Synthesis,
            StructureMode::Architecting,
        );
        assert_eq!(tier, CalloutTier::Attention);

        // Warning in Builder -> stays Warning
        let tier = CalloutTrigger::determine_tier(
            "IAS",
            0.40,
            None,
            Step::Step4_Synthesis,
            StructureMode::Builder,
        );
        assert_eq!(tier, CalloutTier::Warning);

        // Critical in Architecting -> stays Critical (never downgraded)
        let tier = CalloutTrigger::determine_tier(
            "IAS",
            0.15,
            None,
            Step::Step4_Synthesis,
            StructureMode::Architecting,
        );
        assert_eq!(tier, CalloutTier::Critical);
    }

    #[test]
    fn test_acknowledge_with_confirmation() {
        let mut manager = CalloutManager::new();

        let callout = Callout::new(
            CalloutTier::Critical, "CI", 0.15, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        );
        let callout_id = callout.id.clone();
        manager.add(callout);

        let record = manager.acknowledge_with_confirmation(
            &callout_id,
            "I understand the CI is critically low"
        ).unwrap();

        assert_eq!(record.callout_id, callout_id);
        assert_eq!(record.tier, CalloutTier::Critical);
        assert_eq!(record.user_confirmation, "I understand the CI is critically low");
        assert!(manager.can_proceed());
    }

    #[test]
    fn test_acknowledge_all_pending() {
        let mut manager = CalloutManager::new();

        // Add multiple Critical callouts
        manager.add(Callout::new(
            CalloutTier::Critical, "CI", 0.15, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Critical, "IAS", 0.20, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        // Add non-critical (should not be in records)
        manager.add(Callout::new(
            CalloutTier::Attention, "EV", 50.0, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));

        assert!(!manager.can_proceed());

        let records = manager.acknowledge_all_pending("Acknowledged all critical issues");

        assert_eq!(records.len(), 2); // Only Critical callouts
        assert!(manager.can_proceed());
    }

    #[test]
    fn test_callout_summary() {
        let mut manager = CalloutManager::new();

        manager.add(Callout::new(
            CalloutTier::Info, "CI", 0.80, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Attention, "IAS", 0.55, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Critical, "CI", 0.15, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));

        let summary = manager.summary();

        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_tier.info, 1);
        assert_eq!(summary.by_tier.attention, 1);
        assert_eq!(summary.by_tier.critical, 1);
        assert_eq!(summary.pending_acknowledgments, 1);
        assert!(!summary.can_proceed);
    }
}
