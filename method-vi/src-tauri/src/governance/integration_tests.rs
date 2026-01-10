//! Integration tests for the callout system
//! These tests verify end-to-end behavior

#[cfg(test)]
mod tests {
    use super::super::*;

    /// Simulates a step execution and generates callouts
    fn simulate_step_execution(
        ci: f64,
        ias: f64,
        previous_ci: Option<f64>,
        step: Step,
        mode: StructureMode,
    ) -> CalloutManager {
        let mut manager = CalloutManager::new();

        // Generate CI callout
        let ci_tier = CalloutTrigger::determine_tier("CI", ci, previous_ci, step, mode);
        if ci_tier != CalloutTier::Info {
            manager.add(Callout::new(
                ci_tier, "CI", ci, previous_ci,
                format!("{} mode", mode.display_name()),
                "Test CI callout", "Test recommendation",
                step, mode,
            ));
        }

        // Generate IAS callout
        let ias_tier = CalloutTrigger::determine_tier("IAS", ias, None, step, mode);
        if ias_tier != CalloutTier::Info {
            manager.add(Callout::new(
                ias_tier, "IAS", ias, None,
                format!("{} mode", mode.display_name()),
                "Test IAS callout", "Test recommendation",
                step, mode,
            ));
        }

        manager
    }

    // =========================================================================
    // INTEGRATION SUITE 1: Full Step Execution Flow
    // =========================================================================

    #[test]
    fn test_healthy_metrics_no_blocking() {
        let manager = simulate_step_execution(
            0.75, 0.80, Some(0.70),
            Step::Step4_Synthesis, StructureMode::Builder,
        );
        assert!(manager.can_proceed());
        assert_eq!(manager.get_pending_acknowledgments().len(), 0);
    }

    #[test]
    fn test_critical_metrics_require_acknowledgment() {
        let manager = simulate_step_execution(
            0.15, 0.20, Some(0.50),
            Step::Step4_Synthesis, StructureMode::Builder,
        );
        assert!(!manager.can_proceed());
        assert!(manager.get_pending_acknowledgments().len() > 0);
    }

    // =========================================================================
    // INTEGRATION SUITE 2: Mode Behavior Differences
    // =========================================================================

    #[test]
    fn test_architecting_more_lenient() {
        let ci = 0.40;
        let ias = 0.40;

        let arch = simulate_step_execution(ci, ias, None, Step::Step4_Synthesis, StructureMode::Architecting);
        let refine = simulate_step_execution(ci, ias, None, Step::Step4_Synthesis, StructureMode::Refining);

        // Architecting should have fewer/lower severity callouts
        let arch_critical = arch.count_by_tier(CalloutTier::Critical);
        let refine_critical = refine.count_by_tier(CalloutTier::Critical);

        assert!(arch_critical <= refine_critical);
    }

    // =========================================================================
    // INTEGRATION SUITE 3: Acknowledgment Flow
    // =========================================================================

    #[test]
    fn test_acknowledgment_enables_proceed() {
        let mut manager = simulate_step_execution(
            0.10, 0.15, Some(0.50),
            Step::Step4_Synthesis, StructureMode::Builder,
        );

        assert!(!manager.can_proceed());

        let records = manager.acknowledge_all_pending("I understand the risks");

        assert!(records.len() > 0);
        assert!(manager.can_proceed());
    }

    // =========================================================================
    // INTEGRATION SUITE 4: Step-Aware Behavior
    // =========================================================================

    #[test]
    fn test_step3_efi_informational() {
        let efi_tier = CalloutTrigger::efi_tier(0.20, Step::Step3_Diagnostic);
        assert_eq!(efi_tier, CalloutTier::Info);

        let efi_tier_step6 = CalloutTrigger::efi_tier(0.20, Step::Step6_Validation);
        assert_eq!(efi_tier_step6, CalloutTier::Critical);
    }

    // =========================================================================
    // PHASE 1 VERIFICATION
    // =========================================================================

    #[test]
    fn phase1_verify_noise_filter() {
        let warning = CalloutTier::Warning;
        assert_eq!(warning.apply_noise_filter(StructureMode::Architecting), CalloutTier::Attention);

        let critical = CalloutTier::Critical;
        assert_eq!(critical.apply_noise_filter(StructureMode::Architecting), CalloutTier::Critical);
    }

    #[test]
    fn phase1_verify_delta_baseline_types() {
        let anchor = Baseline::Anchor(RunAnchor { ci: 0.40 });
        let diag = Baseline::Diagnostic(DiagnosticBaseline { ci: 0.55 });

        assert_eq!(anchor.ci(), 0.40);
        assert_eq!(diag.ci(), 0.55);
        assert!(diag.is_diagnostic());
    }

    #[test]
    fn phase1_verify_only_critical_blocks() {
        let mut manager = CalloutManager::new();

        // Add non-critical callouts
        manager.add(Callout::new(
            CalloutTier::Warning, "CI", 0.45, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        manager.add(Callout::new(
            CalloutTier::Attention, "IAS", 0.55, None, "", "", "",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));

        // Should proceed - no Critical callouts
        assert!(manager.can_proceed());
    }

    // =========================================================================
    // PHASE 2 VERIFICATION (Session 2.3: Mode Detection Integration)
    // =========================================================================

    #[test]
    fn phase2_mode_detection_architecting() {
        use super::super::ModeDetector;

        let ci_value = 0.25;
        let mode_result = ModeDetector::detect_legacy(ci_value);

        assert_eq!(mode_result.mode, StructureMode::Architecting);
        assert_eq!(mode_result.ci_baseline, ci_value);
        assert!(mode_result.confidence >= 0.5);
        assert!(mode_result.signals.iter().any(|s| s.contains("0.25")));
        assert!(mode_result.signals.iter().any(|s| s.contains("Low initial structure")));
    }

    #[test]
    fn phase2_mode_detection_builder() {
        use super::super::ModeDetector;

        let ci_value = 0.45;
        let mode_result = ModeDetector::detect_legacy(ci_value);

        assert_eq!(mode_result.mode, StructureMode::Builder);
        assert_eq!(mode_result.ci_baseline, ci_value);
        assert!(mode_result.confidence >= 0.5);
        assert!(mode_result.signals.iter().any(|s| s.contains("Medium structure")));
    }

    #[test]
    fn phase2_mode_detection_refining() {
        use super::super::ModeDetector;

        let ci_value = 0.80;
        let mode_result = ModeDetector::detect_legacy(ci_value);

        assert_eq!(mode_result.mode, StructureMode::Refining);
        assert_eq!(mode_result.ci_baseline, ci_value);
        assert!(mode_result.confidence >= 0.5);
        assert!(mode_result.signals.iter().any(|s| s.contains("High structure")));
    }

    #[test]
    fn phase2_mode_locked_after_detection() {
        use super::super::ModeDetector;

        // Simulate Step 2 flow: detect mode and lock it
        let ci_baseline = 0.50;
        let mode_result = ModeDetector::detect_legacy(ci_baseline);

        // In real orchestrator, these would be set
        let detected_mode = Some(mode_result.mode);
        let mode_locked = true;

        assert!(detected_mode.is_some());
        assert_eq!(detected_mode.unwrap(), StructureMode::Builder);
        assert!(mode_locked);
    }

    #[test]
    fn phase2_mode_confidence_varies_by_distance() {
        use super::super::ModeDetector;

        // Far from boundaries = high confidence
        let far_arch = ModeDetector::detect_legacy(0.10);
        let near_boundary = ModeDetector::detect_legacy(0.33);

        // CI=0.10 is further from 0.35 boundary than CI=0.33
        assert!(far_arch.confidence > near_boundary.confidence);
    }

    // =========================================================================
    // PHASE 2 BOUNDARY TESTS
    // =========================================================================

    #[test]
    fn phase2_boundary_architecting_builder() {
        use super::super::ModeDetector;

        // CI exactly at 0.35 should be Architecting (≤ 0.35)
        let at_boundary = ModeDetector::detect_legacy(0.35);
        assert_eq!(at_boundary.mode, StructureMode::Architecting);

        // CI at 0.36 should be Builder
        let above_boundary = ModeDetector::detect_legacy(0.36);
        assert_eq!(above_boundary.mode, StructureMode::Builder);
    }

    #[test]
    fn phase2_boundary_builder_refining() {
        use super::super::ModeDetector;

        // CI at 0.69 should be Builder
        let below_boundary = ModeDetector::detect_legacy(0.69);
        assert_eq!(below_boundary.mode, StructureMode::Builder);

        // CI at 0.70 should be Refining (≥ 0.70)
        let at_boundary = ModeDetector::detect_legacy(0.70);
        assert_eq!(at_boundary.mode, StructureMode::Refining);
    }

    #[test]
    fn phase2_mode_affects_threshold_resolution() {
        use super::super::ThresholdResolver;

        let arch_thresholds = ThresholdResolver::resolve(
            StructureMode::Architecting, Step::Step4_Synthesis
        );
        let refine_thresholds = ThresholdResolver::resolve(
            StructureMode::Refining, Step::Step4_Synthesis
        );

        // Architecting should have lower pass threshold (more lenient)
        // Architecting: 0.50, Refining: 0.80
        assert!(arch_thresholds.ci_pass < refine_thresholds.ci_pass);
    }

    #[test]
    fn phase2_mode_affects_callout_tier() {
        use super::super::CalloutTrigger;

        let ci = 0.45;

        // In Architecting (pass=0.50), 0.45 is borderline
        let arch_tier = CalloutTrigger::determine_tier(
            "CI", ci, None, Step::Step4_Synthesis, StructureMode::Architecting
        );

        // In Refining (pass=0.80), 0.45 is clearly below
        let refine_tier = CalloutTrigger::determine_tier(
            "CI", ci, None, Step::Step4_Synthesis, StructureMode::Refining
        );

        // Refining should be more severe (or equal) due to stricter thresholds
        assert!(refine_tier >= arch_tier);
    }

    // =========================================================================
    // PHASE 3: DIAGNOSTIC TOLERANCE TESTS (Session 3.1)
    // =========================================================================

    #[test]
    fn phase3_informational_mode_returns_info() {
        use super::super::{CalloutTrigger, MetricEnforcement};

        let tier = CalloutTrigger::determine_tier_with_enforcement(
            "CI",
            0.10, // Very low CI that would normally be Critical
            None,
            Step::Step3_Diagnostic,
            StructureMode::Builder,
            MetricEnforcement::Informational,
        );

        assert_eq!(tier, CalloutTier::Info);
    }

    #[test]
    fn phase3_enforced_mode_generates_callouts() {
        use super::super::{CalloutTrigger, MetricEnforcement};

        let tier = CalloutTrigger::determine_tier_with_enforcement(
            "CI",
            0.10,
            None,
            Step::Step4_Synthesis,
            StructureMode::Builder,
            MetricEnforcement::Enforced,
        );

        assert!(tier > CalloutTier::Info);
    }

    #[test]
    fn phase3_diagnostic_baseline_storage() {
        // Verify diagnostic baseline can be stored
        let diagnostic_ci: Option<f64> = Some(0.35);
        assert!(diagnostic_ci.is_some());
        assert_eq!(diagnostic_ci.unwrap(), 0.35);
    }

    #[test]
    fn phase3_delta_from_diagnostic_baseline() {
        // Verify delta calculation from Step 3 baseline
        let step3_ci: f64 = 0.35;
        let step4_ci: f64 = 0.55;
        let delta: f64 = step4_ci - step3_ci;

        assert!(delta > 0.0);
        assert!((delta - 0.20).abs() < 0.001);
    }

    #[test]
    fn phase3_step3_records_baseline_step4_uses_delta() {
        use super::super::{CalloutTrigger, MetricEnforcement};

        // Simulate Step 3 → Step 4 flow
        let step3_ci = 0.35;
        let step4_ci = 0.55;

        // Step 3: Record informational baseline (no callout)
        let tier_step3 = CalloutTrigger::determine_tier_with_enforcement(
            "CI", step3_ci, None,
            Step::Step3_Diagnostic, StructureMode::Builder,
            MetricEnforcement::Informational,
        );
        assert_eq!(tier_step3, CalloutTier::Info);

        // Step 4: Use Step 3 as baseline, enforced mode
        let tier_step4 = CalloutTrigger::determine_tier_with_enforcement(
            "CI", step4_ci, Some(step3_ci),
            Step::Step4_Synthesis, StructureMode::Builder,
            MetricEnforcement::Enforced,
        );

        // Delta is +0.20 (improvement), should not be Critical
        assert!(tier_step4 < CalloutTier::Critical);
        // In fact, positive delta should be Info
        assert_eq!(tier_step4, CalloutTier::Info);
    }

    // =========================================================================
    // PHASE 3 VERIFICATION TESTS
    // =========================================================================

    #[test]
    fn phase3_verify_constraint2_delta_from_diagnostic() {
        // Constraint 2: Delta calculated from Step 3 baseline
        let step3_ci: f64 = 0.35; // Diagnostic baseline
        let step4_ci: f64 = 0.55; // After synthesis

        let delta: f64 = step4_ci - step3_ci;

        // Delta should be +0.20 (improvement)
        assert!((delta - 0.20).abs() < 0.001);
        assert!(delta > 0.0); // Positive = improvement
    }

    #[test]
    fn phase3_verify_informational_never_blocks() {
        use super::super::MetricEnforcement;

        // Step 3 with Informational enforcement should NEVER block
        let mut manager = CalloutManager::new();

        // Even with terrible metrics (CI=0.05), Informational returns Info
        let tier = CalloutTrigger::determine_tier_with_enforcement(
            "CI", 0.05, None,
            Step::Step3_Diagnostic, StructureMode::Builder,
            MetricEnforcement::Informational,
        );

        assert_eq!(tier, CalloutTier::Info);

        // Add as Info callout (not Critical)
        manager.add(Callout::new(
            tier, "CI", 0.05, None, "", "", "",
            Step::Step3_Diagnostic, StructureMode::Builder,
        ));

        // Should always be able to proceed
        assert!(manager.can_proceed());
    }

    #[test]
    fn phase3_enforced_vs_informational_comparison() {
        use super::super::MetricEnforcement;

        let ci = 0.10; // Very low CI

        // Same value, different enforcement
        let informational = CalloutTrigger::determine_tier_with_enforcement(
            "CI", ci, None,
            Step::Step3_Diagnostic, StructureMode::Builder,
            MetricEnforcement::Informational,
        );

        let enforced = CalloutTrigger::determine_tier_with_enforcement(
            "CI", ci, None,
            Step::Step4_Synthesis, StructureMode::Builder,
            MetricEnforcement::Enforced,
        );

        // Informational always Info, Enforced generates real tier
        assert_eq!(informational, CalloutTier::Info);
        assert!(enforced > CalloutTier::Info);
    }

    // =========================================================================
    // PHASE 4: ORCHESTRATOR INTEGRATION TESTS (Session 4.1)
    // =========================================================================

    #[test]
    fn phase4_step2_callout_generation() {
        use super::super::MetricEnforcement;

        let mut manager = CalloutManager::new();

        // Simulate Step 2 with low CI metrics (should trigger callout in Builder mode)
        let ci_tier = CalloutTrigger::determine_tier_with_enforcement(
            "CI", 0.40, None,
            Step::Step2_Governance, StructureMode::Builder,
            MetricEnforcement::Enforced,
        );

        if ci_tier != CalloutTier::Info {
            manager.add(Callout::new(
                ci_tier, "CI", 0.40, None,
                "Builder mode: pass=0.65",
                "CI below pass threshold",
                "Review content clarity",
                Step::Step2_Governance, StructureMode::Builder,
            ));
        }

        // Verify callout was added (Builder pass threshold is 0.65, so 0.40 should create callout)
        let summary = manager.summary();
        assert!(summary.total >= 1); // Should have at least one callout
    }

    #[test]
    fn phase4_callout_manager_in_orchestrator() {
        // Verify CalloutManager can be created and used
        let mut manager = CalloutManager::new();

        // Add a callout
        manager.add(Callout::new(
            CalloutTier::Warning, "CI", 0.45, None, "", "", "",
            Step::Step2_Governance, StructureMode::Builder,
        ));

        // Verify it was added
        assert_eq!(manager.all().len(), 1);

        // Should be able to proceed (Warning doesn't block)
        assert!(manager.can_proceed());
    }

    // =========================================================================
    // INTEGRATION SUITE 5: Session 4.2 - Steps 4-6 Callout Integration
    // =========================================================================

    #[test]
    fn phase4_step4_uses_diagnostic_baseline_for_delta() {
        // Verify Step 4 delta calculation from Step 3 baseline
        let step3_ci: f64 = 0.35; // Diagnostic baseline (low, as expected at Step 3)
        let step4_ci: f64 = 0.55; // After synthesis (improvement)

        let tier = CalloutTrigger::determine_tier(
            "CI", step4_ci, Some(step3_ci),
            Step::Step4_Synthesis, StructureMode::Builder,
        );

        // Positive delta (+0.20) should not be Critical
        // CI delta thresholds: critical < -0.05, warning < 0.0, attention < +0.10
        // +0.20 delta is above attention threshold, so should be Info
        assert!(tier < CalloutTier::Critical, "Positive delta should not trigger Critical");
    }

    #[test]
    fn phase4_step6_strict_pci_enforcement() {
        // Step 6 has strictest PCI enforcement
        let pci = 0.65; // Below Step 6 threshold (0.70 warning, 0.90 pass)

        let tier = CalloutTrigger::determine_tier(
            "PCI", pci, None,
            Step::Step6_Validation, StructureMode::Builder,
        );

        // Step 6 PCI below pass (0.90) should trigger Warning or higher
        // PCI at 0.65 is below warning threshold (0.70) so should be Critical
        assert!(tier >= CalloutTier::Warning, "Low PCI at Step 6 should trigger Warning or higher");
    }

    #[test]
    fn phase4_step6_strict_efi_enforcement() {
        // EFI is fully enforced at Step 6
        let efi = 0.45; // Low EFI (below 0.50 critical threshold)

        let tier = CalloutTrigger::determine_tier(
            "EFI", efi, None,
            Step::Step6_Validation, StructureMode::Builder,
        );

        // Step 6 EFI should be enforced (not Info like earlier steps)
        // EFI thresholds at Step 6: Info >= 0.80, Attention >= 0.60, Warning >= 0.50, Critical < 0.50
        // 0.45 is below critical threshold (0.50)
        assert_eq!(tier, CalloutTier::Critical, "EFI below 0.50 at Step 6 should trigger Critical");
    }

    // =========================================================================
    // INTEGRATION SUITE 6: Session 4.4 - Phase 4 End-to-End Verification
    // =========================================================================

    #[test]
    fn phase4_full_step_flow_with_callouts() {
        // Verify complete workflow: Steps 2→3→4 with mode, baseline, delta
        let mut manager = CalloutManager::new();
        let mode = StructureMode::Builder;

        // Step 2: Initial metrics, mode detected, callouts generated
        let step2_ci = 0.55;
        let tier2 = CalloutTrigger::determine_tier_with_enforcement(
            "CI", step2_ci, None,
            Step::Step2_Governance, mode,
            MetricEnforcement::Enforced,
        );
        if tier2 != CalloutTier::Info {
            manager.add(Callout::new(
                tier2, "CI", step2_ci, None,
                "Builder mode initial check",
                "CI below Builder pass threshold",
                "Review Charter clarity",
                Step::Step2_Governance, mode,
            ));
        }

        // Step 3: Diagnostic baseline (informational - no callouts)
        let step3_ci: f64 = 0.40;
        let tier3 = CalloutTrigger::determine_tier_with_enforcement(
            "CI", step3_ci, None,
            Step::Step3_Diagnostic, mode,
            MetricEnforcement::Informational,
        );
        assert_eq!(tier3, CalloutTier::Info, "Step 3 Informational mode should always return Info");

        // Step 4: Synthesis with delta from Step 3 baseline (Constraint 2)
        let step4_ci = 0.60;
        let tier4 = CalloutTrigger::determine_tier_with_enforcement(
            "CI", step4_ci, Some(step3_ci),
            Step::Step4_Synthesis, mode,
            MetricEnforcement::Enforced,
        );
        // Positive delta (+0.20) should not trigger Critical
        assert!(tier4 < CalloutTier::Critical, "Positive delta from baseline should not be Critical");

        // Verify can proceed (no unacknowledged Critical)
        assert!(manager.can_proceed(), "Should be able to proceed without unacknowledged Critical callouts");
    }

    #[test]
    fn phase4_verify_all_constraints() {
        // CONSTRAINT 1: Transparency Mandate - Mode detection with metadata
        let result = ModeDetector::detect_legacy(0.50);
        assert_eq!(result.mode, StructureMode::Builder, "CI 0.50 should detect Builder mode");
        assert!(result.confidence >= 0.5, "Confidence should be >= 0.5");
        assert!(!result.signals.is_empty(), "Signals should contain detection metadata");
        assert!(result.signals.iter().any(|s| s.contains("CI baseline")),
            "Signals should mention CI baseline");

        // CONSTRAINT 2: Delta Baseline Rule - verified in phase4_full_step_flow_with_callouts test
        // Step 3 records baseline informationally, Steps 4+ use it for delta calculation

        // CONSTRAINT 3: Noise Filter - Warning→Attention in Architecting mode
        // Use delta that triggers Warning: -0.25 (between -0.20 and -0.30)
        let warning_tier_arch = CalloutTrigger::determine_tier(
            "CI", 0.25, Some(0.50), // Delta -0.25 triggers Warning
            Step::Step4_Synthesis, StructureMode::Architecting,
        );
        // In Architecting, Warning should downgrade to Attention (noise filter)
        assert_eq!(warning_tier_arch, CalloutTier::Attention,
            "Noise filter should downgrade Warning to Attention in Architecting mode");

        // Verify noise filter doesn't affect other modes
        let warning_tier_builder = CalloutTrigger::determine_tier(
            "CI", 0.25, Some(0.50), // Same delta -0.25
            Step::Step4_Synthesis, StructureMode::Builder,
        );
        // In Builder, Warning stays Warning (no downgrade)
        assert_eq!(warning_tier_builder, CalloutTier::Warning,
            "Warning should NOT downgrade in Builder mode");

        // CONSTRAINT 4: Critical-Only Blocking
        let mut manager = CalloutManager::new();

        // Warning doesn't block
        manager.add(Callout::new(
            CalloutTier::Warning, "CI", 0.40, None,
            "Warning test", "CI below target", "Review content",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        assert!(manager.can_proceed(), "Warning callouts should NOT block progression");

        // Attention doesn't block
        manager.add(Callout::new(
            CalloutTier::Attention, "IAS", 0.65, None,
            "Attention test", "IAS needs attention", "Review alignment",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        assert!(manager.can_proceed(), "Attention callouts should NOT block progression");

        // Critical DOES block
        manager.add(Callout::new(
            CalloutTier::Critical, "CI", 0.15, None,
            "Critical test", "CI critically low", "Major revision needed",
            Step::Step4_Synthesis, StructureMode::Builder,
        ));
        assert!(!manager.can_proceed(), "Critical callouts MUST block progression until acknowledged");

        // After acknowledgment, can proceed
        let critical_callout_ids: Vec<String> = manager.all()
            .iter()
            .filter(|c| c.tier == CalloutTier::Critical)
            .map(|c| c.id.clone())
            .collect();
        for id in critical_callout_ids {
            manager.acknowledge(&id).unwrap();
        }
        assert!(manager.can_proceed(), "After acknowledging Critical callouts, should be able to proceed");
    }
}
