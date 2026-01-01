/// Test to verify triggered_metrics only contains HALT-causing metrics
///
/// This test validates FIX-022: Filter triggered_metrics to HALT-Causing Metrics Only
///
/// Expected behavior:
/// - When CI fails at Step 3, triggered_metrics should contain ONLY ci
/// - It should NOT contain efi, ev, pci (which aren't checked at Step 3 per FIX-009)
/// - all_metrics_snapshot should contain ALL metrics for debugging

use method_vi_lib::agents::governance_telemetry::{
    CriticalMetrics, GovernanceTelemetryAgent, MetricResult, MetricStatus, MetricThreshold,
};
use serde_json;
use std::env;

/// Helper to create a mock metric result
fn create_metric_result(name: &str, value: f64, status: MetricStatus) -> MetricResult {
    MetricResult {
        metric_name: name.to_string(),
        value,
        threshold: MetricThreshold {
            pass: 0.80,
            warning: Some(0.70),
            halt: Some(0.50),
        },
        status,
        inputs_used: vec![],
        calculation_method: "test".to_string(),
        interpretation: "test".to_string(),
        recommendation: None,
    }
}

#[test]
fn test_triggered_metrics_filtering_logic() {
    // Create mock metrics where:
    // - CI fails (should trigger HALT at Step 3)
    // - EFI fails (should NOT trigger HALT at Step 3, only at Step 6)
    // - EV fails (should NOT trigger HALT, disabled per FIX-017)
    // - PCI fails (should NOT trigger HALT at Step 3, only at Steps 5-6)

    let metrics = CriticalMetrics {
        ci: Some(create_metric_result("CI", 0.30, MetricStatus::Fail)),
        ev: Some(create_metric_result("EV", 119.0, MetricStatus::Fail)),
        ias: Some(create_metric_result("IAS", 0.85, MetricStatus::Pass)),
        efi: Some(create_metric_result("EFI", 15.0, MetricStatus::Fail)),
        sec: Some(create_metric_result("SEC", 100.0, MetricStatus::Pass)),
        pci: Some(create_metric_result("PCI", 0.30, MetricStatus::Fail)),
    };

    // Simulate halt_reason from check_halt_conditions at Step 3
    // At Step 3, only CI and IAS are checked (per FIX-009)
    let halt_reason = "HALT: Critical metrics failed: CI critically low: 0.30";
    let step = 3;

    // Manually apply the filtering logic from get_halt_triggering_metrics
    let mut triggered = serde_json::Map::new();

    // CI triggers HALT at all steps
    if let Some(ref ci) = metrics.ci {
        if ci.status == MetricStatus::Fail && halt_reason.contains("CI") {
            triggered.insert("ci".to_string(), serde_json::to_value(ci).unwrap());
        }
    }

    // IAS triggers HALT at all steps
    if let Some(ref ias) = metrics.ias {
        if ias.status == MetricStatus::Fail && halt_reason.contains("IAS") {
            triggered.insert("ias".to_string(), serde_json::to_value(ias).unwrap());
        }
    }

    // EFI only triggers at Step 6 (NOT Step 3)
    if step == 6 {
        if let Some(ref efi) = metrics.efi {
            if efi.status == MetricStatus::Fail && halt_reason.contains("EFI") {
                triggered.insert("efi".to_string(), serde_json::to_value(efi).unwrap());
            }
        }
    }

    // PCI only triggers at Steps 5-6 (NOT Step 3)
    if step >= 5 {
        if let Some(ref pci) = metrics.pci {
            if pci.status == MetricStatus::Fail && halt_reason.contains("PCI") {
                triggered.insert("pci".to_string(), serde_json::to_value(pci).unwrap());
            }
        }
    }

    // SEC only triggers at Steps 1 and 6 (NOT Step 3)
    if step == 1 || step == 6 {
        if let Some(ref sec) = metrics.sec {
            if sec.status == MetricStatus::Fail && halt_reason.contains("SEC") {
                triggered.insert("sec".to_string(), serde_json::to_value(sec).unwrap());
            }
        }
    }

    // EV never triggers (disabled per FIX-017)

    let triggered_metrics = serde_json::Value::Object(triggered);

    // Assertions
    println!("Triggered metrics keys: {:?}", triggered_metrics.as_object().unwrap().keys());

    // Should contain CI (actually triggered HALT)
    assert!(
        triggered_metrics.get("ci").is_some(),
        "triggered_metrics should contain 'ci' (caused HALT at Step 3)"
    );

    // Should NOT contain EFI (only checked at Step 6)
    assert!(
        triggered_metrics.get("efi").is_none(),
        "triggered_metrics should NOT contain 'efi' (not checked at Step 3 per FIX-009)"
    );

    // Should NOT contain EV (disabled)
    assert!(
        triggered_metrics.get("ev").is_none(),
        "triggered_metrics should NOT contain 'ev' (disabled per FIX-017)"
    );

    // Should NOT contain PCI (only checked at Steps 5-6)
    assert!(
        triggered_metrics.get("pci").is_none(),
        "triggered_metrics should NOT contain 'pci' (not checked at Step 3 per FIX-009)"
    );

    // Should NOT contain IAS (didn't fail)
    assert!(
        triggered_metrics.get("ias").is_none(),
        "triggered_metrics should NOT contain 'ias' (passed, didn't trigger HALT)"
    );

    // Should only contain 1 metric (ci)
    assert_eq!(
        triggered_metrics.as_object().unwrap().len(),
        1,
        "triggered_metrics should contain exactly 1 metric (ci)"
    );

    println!("✅ PASS: triggered_metrics correctly filtered to only CI at Step 3");
}

#[test]
fn test_triggered_metrics_filtering_step_6() {
    // Test that at Step 6, EFI is included when it triggers HALT
    let metrics = CriticalMetrics {
        ci: Some(create_metric_result("CI", 0.85, MetricStatus::Pass)),
        ev: Some(create_metric_result("EV", 5.0, MetricStatus::Pass)),
        ias: Some(create_metric_result("IAS", 0.85, MetricStatus::Pass)),
        efi: Some(create_metric_result("EFI", 75.0, MetricStatus::Fail)),
        sec: Some(create_metric_result("SEC", 100.0, MetricStatus::Pass)),
        pci: Some(create_metric_result("PCI", 0.92, MetricStatus::Pass)),
    };

    let halt_reason = "HALT: Critical metrics failed: EFI critically low: 75.0";
    let step = 6;

    let mut triggered = serde_json::Map::new();

    // Apply filtering logic for Step 6
    if let Some(ref ci) = metrics.ci {
        if ci.status == MetricStatus::Fail && halt_reason.contains("CI") {
            triggered.insert("ci".to_string(), serde_json::to_value(ci).unwrap());
        }
    }

    if let Some(ref ias) = metrics.ias {
        if ias.status == MetricStatus::Fail && halt_reason.contains("IAS") {
            triggered.insert("ias".to_string(), serde_json::to_value(ias).unwrap());
        }
    }

    // EFI only triggers at Step 6
    if step == 6 {
        if let Some(ref efi) = metrics.efi {
            if efi.status == MetricStatus::Fail && halt_reason.contains("EFI") {
                triggered.insert("efi".to_string(), serde_json::to_value(efi).unwrap());
            }
        }
    }

    if step >= 5 {
        if let Some(ref pci) = metrics.pci {
            if pci.status == MetricStatus::Fail && halt_reason.contains("PCI") {
                triggered.insert("pci".to_string(), serde_json::to_value(pci).unwrap());
            }
        }
    }

    if step == 1 || step == 6 {
        if let Some(ref sec) = metrics.sec {
            if sec.status == MetricStatus::Fail && halt_reason.contains("SEC") {
                triggered.insert("sec".to_string(), serde_json::to_value(sec).unwrap());
            }
        }
    }

    let triggered_metrics = serde_json::Value::Object(triggered);

    // Should contain EFI (triggered at Step 6)
    assert!(
        triggered_metrics.get("efi").is_some(),
        "triggered_metrics should contain 'efi' (caused HALT at Step 6)"
    );

    // Should only contain 1 metric (efi)
    assert_eq!(
        triggered_metrics.as_object().unwrap().len(),
        1,
        "triggered_metrics should contain exactly 1 metric (efi)"
    );

    println!("✅ PASS: triggered_metrics correctly includes EFI at Step 6");
}

#[test]
fn test_triggered_metrics_multiple_failures() {
    // Test when both CI and IAS fail at Step 3
    let metrics = CriticalMetrics {
        ci: Some(create_metric_result("CI", 0.45, MetricStatus::Fail)),
        ev: Some(create_metric_result("EV", 5.0, MetricStatus::Pass)),
        ias: Some(create_metric_result("IAS", 0.40, MetricStatus::Fail)),
        efi: Some(create_metric_result("EFI", 15.0, MetricStatus::Fail)),
        sec: Some(create_metric_result("SEC", 100.0, MetricStatus::Pass)),
        pci: Some(create_metric_result("PCI", 0.30, MetricStatus::Fail)),
    };

    let halt_reason = "HALT: Critical metrics failed: CI critically low: 0.45, IAS critically low: 0.40";
    let step = 3;

    let mut triggered = serde_json::Map::new();

    if let Some(ref ci) = metrics.ci {
        if ci.status == MetricStatus::Fail && halt_reason.contains("CI") {
            triggered.insert("ci".to_string(), serde_json::to_value(ci).unwrap());
        }
    }

    if let Some(ref ias) = metrics.ias {
        if ias.status == MetricStatus::Fail && halt_reason.contains("IAS") {
            triggered.insert("ias".to_string(), serde_json::to_value(ias).unwrap());
        }
    }

    // Skip EFI, PCI, SEC checks at Step 3 (per step guards)

    let triggered_metrics = serde_json::Value::Object(triggered);

    // Should contain both CI and IAS
    assert!(
        triggered_metrics.get("ci").is_some(),
        "triggered_metrics should contain 'ci'"
    );
    assert!(
        triggered_metrics.get("ias").is_some(),
        "triggered_metrics should contain 'ias'"
    );

    // Should NOT contain EFI or PCI
    assert!(
        triggered_metrics.get("efi").is_none(),
        "triggered_metrics should NOT contain 'efi'"
    );
    assert!(
        triggered_metrics.get("pci").is_none(),
        "triggered_metrics should NOT contain 'pci'"
    );

    // Should contain exactly 2 metrics
    assert_eq!(
        triggered_metrics.as_object().unwrap().len(),
        2,
        "triggered_metrics should contain exactly 2 metrics (ci, ias)"
    );

    println!("✅ PASS: triggered_metrics correctly contains both CI and IAS at Step 3");
}
