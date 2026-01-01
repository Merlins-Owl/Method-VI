/// Test to verify temperature=0.0 produces deterministic metric results
///
/// This test validates FIX-021: Temperature Control for LLM-based metrics
///
/// Expected behavior:
/// - Same content should produce identical CI scores (±0.02) across multiple runs
/// - Temperature=0.0 eliminates the variance seen in Test Run 5 vs Test Run 7
use method_vi_lib::agents::governance_telemetry::GovernanceTelemetryAgent;
use std::env;

#[tokio::test]
#[ignore] // Only run with: cargo test test_ci_determinism -- --ignored
async fn test_ci_determinism() -> anyhow::Result<()> {
    // Requires ANTHROPIC_API_KEY environment variable
    let api_key = env::var("ANTHROPIC_API_KEY").expect(
        "ANTHROPIC_API_KEY required for temperature determinism test. \
        Run: set ANTHROPIC_API_KEY=your-key-here",
    );

    let agent = GovernanceTelemetryAgent::new(api_key)?;

    // Test content - same as used in Test Runs 5 & 7
    let test_content = r#"
# Integrated Diagnostic Summary

## Structural Analysis
The framework demonstrates clear hierarchical organization with 11 distinct sections.
Each section builds upon the previous, creating a logical progression from problem
identification through solution implementation.

## Thematic Analysis
Central themes include AI adoption readiness, leadership alignment, and practical
implementation pathways. The FLAME acronym (Foundation, Leadership, Alignment,
Measurement, Enablement) provides consistent thematic structure.

## Logic Analysis
The logical flow follows a diagnostic-prescriptive pattern: assess current state,
identify gaps, propose solutions, and provide implementation guidance. Each section
contains clear cause-effect relationships.
"#;

    println!("Testing CI calculation determinism with temperature=0.0");
    println!("Running 3 iterations with identical content...\n");

    let mut scores = Vec::new();

    for i in 1..=3 {
        let metrics = agent.calculate_metrics(test_content, "test objectives", 3).await?;

        let ci = metrics.ci.expect("CI metric should be calculated");
        let score = ci.value;

        println!("Iteration {}: CI = {:.4}", i, score);
        scores.push(score);
    }

    // Verify determinism: all scores should be identical or within ±0.02
    let first_score = scores[0];
    let max_variance = scores.iter()
        .map(|s| (s - first_score).abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    println!("\nResults:");
    println!("  First score: {:.4}", first_score);
    println!("  Max variance: {:.4}", max_variance);

    assert!(
        max_variance <= 0.02,
        "CI variance {:.4} exceeds threshold 0.02. \
        Temperature=0.0 should produce deterministic results.",
        max_variance
    );

    println!("\n✅ PASS: CI calculation is deterministic (variance: {:.4})", max_variance);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_all_metrics_determinism() -> anyhow::Result<()> {
    let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required");
    let agent = GovernanceTelemetryAgent::new(api_key)?;

    let test_content = "Test framework content for metrics";
    let charter = "Test charter objectives";

    println!("Testing determinism for all LLM-based metrics");
    println!("Running 2 iterations...\n");

    let metrics1 = agent.calculate_metrics(test_content, charter, 6).await?;
    let metrics2 = agent.calculate_metrics(test_content, charter, 6).await?;

    // Check CI
    let ci1 = metrics1.ci.unwrap().value;
    let ci2 = metrics2.ci.unwrap().value;
    let ci_variance = (ci1 - ci2).abs();
    println!("CI:  Run1={:.4}, Run2={:.4}, Variance={:.4}", ci1, ci2, ci_variance);
    assert!(ci_variance <= 0.02, "CI variance too high");

    // Check IAS
    let ias1 = metrics1.ias.unwrap().value;
    let ias2 = metrics2.ias.unwrap().value;
    let ias_variance = (ias1 - ias2).abs();
    println!("IAS: Run1={:.4}, Run2={:.4}, Variance={:.4}", ias1, ias2, ias_variance);
    assert!(ias_variance <= 0.02, "IAS variance too high");

    // Check EFI
    let efi1 = metrics1.efi.unwrap().value;
    let efi2 = metrics2.efi.unwrap().value;
    let efi_variance = (efi1 - efi2).abs();
    println!("EFI: Run1={:.2}, Run2={:.2}, Variance={:.2}", efi1, efi2, efi_variance);
    assert!(efi_variance <= 2.0, "EFI variance too high");

    println!("\n✅ PASS: All metrics are deterministic");

    Ok(())
}
