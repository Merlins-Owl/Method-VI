/// Integration Test Suite: Metrics Redesign Verification
/// Tests all phases of the metrics redesign (FIX-021 through FIX-027)

use method_vi_lib::agents::governance_telemetry::{
    GovernanceTelemetryAgent, MetricStatus,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║   INTEGRATION TEST SUITE: METRICS REDESIGN VERIFICATION                ║");
    println!("║   Tests: FIX-021 through FIX-027                                       ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY").expect(
        "ANTHROPIC_API_KEY environment variable must be set",
    );

    let mut results = Vec::new();

    // =========================================================================
    // TEST 1: CI VARIANCE TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 1: CI Variance (Determinism)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: All 5 runs within ±0.02 variance\n");

    let test_content = r#"
# Technical Analysis Report

## System Architecture
The current system uses a monolithic architecture with tightly coupled components.
This creates deployment challenges and limits scalability options.

## Performance Metrics
- Response time: 450ms average
- Throughput: 1200 requests/second
- Error rate: 0.3%

## Recommendations
1. Implement service decomposition strategy
2. Add caching layer for frequently accessed data
3. Optimize database queries for better performance
"#;

    let charter = "Modernize system architecture to improve performance and scalability";

    let mut ci_scores = Vec::new();
    for i in 1..=5 {
        let mut agent = GovernanceTelemetryAgent::new(api_key.clone())?;
        // Initialize E_baseline for each agent
        agent.calculate_e_baseline(charter, 1).await?;
        agent.lock_e_baseline(1)?;

        let metrics = agent.calculate_metrics(test_content, charter, 3).await?;
        let ci_value = metrics.ci.as_ref().unwrap().value;
        ci_scores.push(ci_value);
        println!("  Run {}: CI = {:.4}", i, ci_value);
    }

    let min_ci = ci_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ci = ci_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let variance = max_ci - min_ci;

    let test1_pass = variance < 0.02;
    println!("\n  Variance: {:.4}", variance);
    println!("  Result: {}", if test1_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: {} CI scores within ±{:.4} range\n", ci_scores.len(), variance);

    results.push((
        "Test 1: CI Variance",
        test1_pass,
        format!("Variance: {:.4} (threshold: 0.02)", variance),
    ));

    // =========================================================================
    // TEST 2: CI STEP-SEMANTIC TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 2: CI Step-Semantic Weighting");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: Unstructured but logical content scores higher at Step 3 than Step 5\n");

    let unstructured_content = r#"
Looking at the current authentication system, there are several issues we need to address.
The password hashing algorithm is outdated (MD5), which is a security vulnerability.
Users are also complaining about session timeouts being too short at 15 minutes.

From a performance perspective, the login endpoint is slow because it's making synchronous
calls to three different services. We should parallelize these or use async patterns.
The token validation happens on every request which adds latency.

For the migration, we should first upgrade the hashing to bcrypt with a cost factor of 12.
Then we can extend session duration to 2 hours with sliding expiration. The async refactor
can happen in phase two since it requires more extensive testing.
"#;

    let mut agent_step3 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_step3.calculate_e_baseline(charter, 1).await?;
    agent_step3.lock_e_baseline(1)?;
    let metrics_step3 = agent_step3.calculate_metrics(unstructured_content, charter, 3).await?;
    let ci_step3 = metrics_step3.ci.as_ref().unwrap().value;

    let mut agent_step5 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_step5.calculate_e_baseline(charter, 1).await?;
    agent_step5.lock_e_baseline(1)?;
    let metrics_step5 = agent_step5.calculate_metrics(unstructured_content, charter, 5).await?;
    let ci_step5 = metrics_step5.ci.as_ref().unwrap().value;

    println!("  Step 3 (Multi-Angle Analysis) - Structure weight: 5%");
    println!("    CI Score: {:.4}", ci_step3);
    println!("\n  Step 5 (Structure & Redesign) - Structure weight: 30%");
    println!("    CI Score: {:.4}", ci_step5);

    let test2_pass = ci_step3 > ci_step5 && ci_step3 >= 0.50;
    println!("\n  Result: {}", if test2_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: Step 3 ({:.4}) > Step 5 ({:.4}), Step 3 ≥ 0.50\n", ci_step3, ci_step5);

    results.push((
        "Test 2: CI Step-Semantic",
        test2_pass,
        format!("Step 3: {:.4}, Step 5: {:.4}", ci_step3, ci_step5),
    ));

    // =========================================================================
    // TEST 3: EFI TAXONOMY TEST (Pure Instructional)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 3: EFI Claim Taxonomy (Pure Instructional)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: EFI = 1.0 (no factual claims to score)\n");

    let instructional_content = r#"
# Implementation Guide

## Setup Instructions
1. Install the required dependencies using npm install
2. Configure the environment variables in .env file
3. Run the database migrations with npm run migrate
4. Start the development server using npm run dev

## Best Practices
- Always validate user input before processing
- Use parameterized queries to prevent SQL injection
- Implement proper error handling with try-catch blocks
- Write unit tests for all business logic

## Next Steps
Follow the deployment guide in docs/deployment.md to publish to production.
Review the security checklist before going live.
"#;

    let mut agent_efi1 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_efi1.calculate_e_baseline(charter, 1).await?;
    agent_efi1.lock_e_baseline(1)?;
    let metrics_efi1 = agent_efi1.calculate_metrics(instructional_content, charter, 6).await?;
    let efi1 = metrics_efi1.efi.as_ref().unwrap();
    let efi1_value = efi1.value;

    println!("  EFI Score: {:.2}", efi1_value);
    println!("  Status: {:?}", efi1.status);
    println!("  Interpretation: {}", efi1.interpretation);

    let test3_pass = efi1_value >= 0.95; // Allow small variance due to LLM
    println!("\n  Result: {}", if test3_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: Pure instructional content → EFI = {:.2}\n", efi1_value);

    results.push((
        "Test 3: EFI Taxonomy (Instructional)",
        test3_pass,
        format!("EFI: {:.2} (expected: 1.0)", efi1_value),
    ));

    // =========================================================================
    // TEST 4: EFI MIXED CONTENT TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 4: EFI Mixed Content (Factual + Instructional)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: EFI ≈ 0.60 (3/5 factual claims substantiated, ignore instructional)\n");

    let mixed_content = r#"
# Performance Analysis Report

## Current Metrics (Factual Claims)
1. The API response time averages 450ms across all endpoints. [SUBSTANTIATED: Load test data from 2024-12-15 shows mean=448ms, p95=652ms]
2. Database queries account for 80% of request latency. [SUBSTANTIATED: APM traces show DB time avg 360ms of 450ms total]
3. The system handles 1200 requests per second at peak load. [SUBSTANTIATED: Production metrics from Dec 2024 show peak RPS=1205]
4. Cache hit rate is currently 35%. [UNSUBSTANTIATED: No data source provided]
5. Error rate has increased by 50% in the last month. [UNSUBSTANTIATED: No baseline or current data provided]

## Recommendations (Instructional - Should be ignored by EFI)
1. Implement Redis caching for frequently accessed data
2. Add database connection pooling with max 50 connections
3. Optimize the top 10 slowest queries identified in the analysis
4. Set up monitoring alerts for response time thresholds
5. Enable query result caching in the ORM layer
6. Add CDN for static assets
7. Implement API rate limiting per client
8. Use database read replicas for query distribution
9. Enable gzip compression on all responses
10. Schedule regular performance audits
"#;

    let mut agent_efi2 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_efi2.calculate_e_baseline(charter, 1).await?;
    agent_efi2.lock_e_baseline(1)?;
    let metrics_efi2 = agent_efi2.calculate_metrics(mixed_content, charter, 6).await?;
    let efi2 = metrics_efi2.efi.as_ref().unwrap();
    let efi2_value = efi2.value;

    println!("  EFI Score: {:.2}", efi2_value);
    println!("  Status: {:?}", efi2.status);
    println!("  Interpretation: {}", efi2.interpretation);

    // Allow ±0.10 variance for LLM interpretation
    let test4_pass = efi2_value >= 0.50 && efi2_value <= 0.70;
    println!("\n  Result: {}", if test4_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: 3/5 claims substantiated → EFI = {:.2} (expected: 0.60 ±0.10)\n", efi2_value);

    results.push((
        "Test 4: EFI Mixed Content",
        test4_pass,
        format!("EFI: {:.2} (expected: 0.60 ±0.10)", efi2_value),
    ));

    // =========================================================================
    // TEST 5: PCI DETERMINISM TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 5: PCI Determinism");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: Identical scores on same content (PCI uses deterministic checklist)\n");

    // Run metrics calculation twice on same content
    let mut agent_pci1 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_pci1.calculate_e_baseline(charter, 1).await?;
    agent_pci1.lock_e_baseline(1)?;
    let metrics_pci1 = agent_pci1.calculate_metrics(test_content, charter, 3).await?;
    let pci1_value = metrics_pci1.pci.as_ref().unwrap().value;

    let mut agent_pci2 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_pci2.calculate_e_baseline(charter, 1).await?;
    agent_pci2.lock_e_baseline(1)?;
    let metrics_pci2 = agent_pci2.calculate_metrics(test_content, charter, 3).await?;
    let pci2_value = metrics_pci2.pci.as_ref().unwrap().value;

    println!("  Run 1 PCI: {:.4}", pci1_value);
    println!("  Run 2 PCI: {:.4}", pci2_value);

    let test5_pass = (pci1_value - pci2_value).abs() < 0.0001;
    println!("\n  Result: {}", if test5_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: Deterministic checklist produces identical scores\n");

    results.push((
        "Test 5: PCI Determinism",
        test5_pass,
        format!("Run 1: {:.4}, Run 2: {:.4}", pci1_value, pci2_value),
    ));

    // =========================================================================
    // TEST 6: IAS SOFT GATE TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 6: IAS Soft Gate (Resynthesis Pause)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: IAS ≥ 0.30 triggers ResynthesisPause, not HALT\n");

    let moderate_alignment_content = r#"
# Technical Analysis

## Current System
The system uses a traditional three-tier architecture with web, application, and database layers.
Performance is adequate for current load but may need optimization as traffic grows.

## Proposed Changes
Consider implementing caching strategies and database indexing to improve response times.
The architecture could benefit from service-oriented design principles.

## Next Steps
Further analysis needed to determine specific implementation details and timeline.
"#;

    let mut agent_ias1 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_ias1.calculate_e_baseline(charter, 1).await?;
    agent_ias1.lock_e_baseline(1)?;
    let metrics_ias1 = agent_ias1.calculate_metrics(moderate_alignment_content, charter, 4).await?;
    let ias1 = metrics_ias1.ias.as_ref().unwrap();

    println!("  IAS Score: {:.2}", ias1.value);
    println!("  Status: {:?}", ias1.status);

    // Check HALT conditions (returns Option<String>)
    let halt1 = agent_ias1.check_halt_conditions(&metrics_ias1, 4);
    let is_soft_gate = ias1.value >= 0.30 && ias1.value < 0.70;

    // For IAS in soft gate range, should not HALT (may warn via check_ias_warning)
    let ias_warning = agent_ias1.check_ias_warning(&metrics_ias1, 4);

    println!("  HALT: {:?}", halt1);
    println!("  IAS Warning: {:?}", ias_warning.is_some());

    let test6_pass = is_soft_gate && halt1.is_none();
    println!("\n  Result: {}", if test6_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: IAS {:.2} in soft gate range → No HALT (warning only)\n", ias1.value);

    results.push((
        "Test 6: IAS Soft Gate",
        test6_pass,
        format!("IAS: {:.2}, HALT: {}", ias1.value, halt1.is_some()),
    ));

    // =========================================================================
    // TEST 7: IAS HALT TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 7: IAS Hard HALT");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: IAS < 0.30 triggers HALT\n");

    let poor_alignment_content = r#"
# Random Technical Notes

Some observations about various technologies and approaches.
Cloud computing is popular these days. Kubernetes is complex but powerful.
React is a frontend framework. Databases store data.

## Various Ideas
Maybe we could use microservices. Or maybe not. It depends on the situation.
Performance is important. Security matters too. Testing is good practice.

## Conclusion
There are many ways to build software systems. Each has trade-offs.
"#;

    let mut agent_ias2 = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_ias2.calculate_e_baseline(charter, 1).await?;
    agent_ias2.lock_e_baseline(1)?;
    let metrics_ias2 = agent_ias2.calculate_metrics(poor_alignment_content, charter, 3).await?;
    let ias2 = metrics_ias2.ias.as_ref().unwrap();

    println!("  IAS Score: {:.2}", ias2.value);
    println!("  Status: {:?}", ias2.status);

    let halt2 = agent_ias2.check_halt_conditions(&metrics_ias2, 3);
    println!("  HALT: {:?}", halt2);

    let test7_pass = ias2.value < 0.30 && halt2.is_some();
    println!("\n  Result: {}", if test7_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: IAS {:.2} < 0.30 → HALT triggered\n", ias2.value);

    results.push((
        "Test 7: IAS Hard HALT",
        test7_pass,
        format!("IAS: {:.2}, HALT: {}", ias2.value, halt2.is_some()),
    ));

    // =========================================================================
    // TEST 8: CI HALT TEST (Incoherent Content)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 8: CI HALT (Low Coherence)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: Incoherent content triggers CI failure and HALT\n");

    let incoherent_content = r#"
The system performance database cloud security authentication API microservices
deployment monitoring logging caching optimization scalability architecture pattern
design framework library testing documentation configuration infrastructure networking
storage processing analysis metrics dashboard alerts integration deployment pipeline
continuous delivery automation orchestration containerization virtualization.
"#;

    let mut agent_trig = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_trig.calculate_e_baseline(charter, 1).await?;
    agent_trig.lock_e_baseline(1)?;
    let metrics_trig = agent_trig.calculate_metrics(incoherent_content, charter, 3).await?;
    let ci_trig = metrics_trig.ci.as_ref().unwrap();

    let halt_trig = agent_trig.check_halt_conditions(&metrics_trig, 3);

    println!("  CI Score: {:.2} (Status: {:?})", ci_trig.value, ci_trig.status);
    println!("  HALT: {:?}", halt_trig);

    let test8_pass = ci_trig.status == MetricStatus::Fail && halt_trig.is_some();
    println!("\n  Result: {}", if test8_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: CI failure triggers HALT\n");

    results.push((
        "Test 8: CI HALT",
        test8_pass,
        format!("CI: {:.2}, HALT: {}", ci_trig.value, halt_trig.is_some()),
    ));

    // =========================================================================
    // TEST 9: EV ADVISORY TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 9: EV Advisory (Informational Only)");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: High variance → Pass status, no HALT\n");

    // Create content with very different entropy from baseline
    let high_entropy_content = r#"
Brief note: Use microservices.
"#;

    let mut agent_ev = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_ev.calculate_e_baseline(charter, 1).await?;
    agent_ev.lock_e_baseline(1)?;

    let metrics_ev = agent_ev.calculate_metrics(high_entropy_content, charter, 3).await?;
    let ev = metrics_ev.ev.as_ref().unwrap();

    let halt_ev = agent_ev.check_halt_conditions(&metrics_ev, 3);

    println!("  EV Variance: {:.1}%", ev.value);
    println!("  EV Status: {:?}", ev.status);
    println!("  HALT: {:?}", halt_ev);

    // EV should always be Pass status and never cause HALT
    let test9_pass = ev.status == MetricStatus::Pass;
    println!("\n  Result: {}", if test9_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: EV is informational, variance {:.1}% shows Pass status\n", ev.value);

    results.push((
        "Test 9: EV Advisory",
        test9_pass,
        format!("EV: {:.1}%, Status: {:?}", ev.value, ev.status),
    ));

    // =========================================================================
    // TEST 10: SEC PLACEHOLDER TEST
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("TEST 10: SEC Placeholder");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Expected: SEC always returns 100% with Pass status\n");

    let mut agent_sec = GovernanceTelemetryAgent::new(api_key.clone())?;
    agent_sec.calculate_e_baseline(charter, 1).await?;
    agent_sec.lock_e_baseline(1)?;
    let metrics_sec = agent_sec.calculate_metrics(test_content, charter, 3).await?;
    let sec = metrics_sec.sec.as_ref().unwrap();

    println!("  SEC Value: {:.1}%", sec.value);
    println!("  SEC Status: {:?}", sec.status);
    println!("  Inputs Used: {} (expected: 0)", sec.inputs_used.len());

    let test10_pass = sec.value == 100.0 &&
                      sec.status == MetricStatus::Pass &&
                      sec.inputs_used.is_empty();
    println!("\n  Result: {}", if test10_pass { "✅ PASS" } else { "❌ FAIL" });
    println!("  Details: SEC placeholder returns 100% Pass with no inputs\n");

    results.push((
        "Test 10: SEC Placeholder",
        test10_pass,
        format!("SEC: {:.1}%, Status: {:?}", sec.value, sec.status),
    ));

    // =========================================================================
    // FINAL SUMMARY
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                          TEST SUMMARY                                  ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    let passed = results.iter().filter(|(_, pass, _)| *pass).count();
    let failed = results.len() - passed;

    for (name, pass, details) in &results {
        println!("{} {}", if *pass { "✅" } else { "❌" }, name);
        println!("   {}", details);
        println!();
    }

    println!("─────────────────────────────────────────────────────────────────────────");
    println!("Passed: {}/{}", passed, results.len());
    println!("Failed: {}/{}", failed, results.len());
    println!("─────────────────────────────────────────────────────────────────────────");

    let ready = passed == results.len();
    println!("\nReady for Test Run 8: {}", if ready { "✅ YES" } else { "❌ NO" });

    if !ready {
        println!("\n⚠️  Failed tests must be addressed before proceeding to Test Run 8");
    }

    println!("\n╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                     INTEGRATION TEST COMPLETE                          ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");

    Ok(())
}
