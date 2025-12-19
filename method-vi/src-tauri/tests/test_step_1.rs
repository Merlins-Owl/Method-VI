use method_vi_lib::agents::{
    GovernanceTelemetryAgent, Orchestrator, ScopePatternAgent, StructureRedesignAgent,
};
use method_vi_lib::api::AnthropicClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    println!("=== Method-VI Step 1 Full Test ===\n");

    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY").expect(
        "ANTHROPIC_API_KEY environment variable must be set. \
        Run: set ANTHROPIC_API_KEY=your-key-here",
    );

    // 1. Create orchestrator with all agents
    println!("1. Initializing Orchestrator with all agents...");

    let claude_client = AnthropicClient::new(api_key.clone())?;
    let scope_agent = ScopePatternAgent::new(claude_client);
    let governance_agent = GovernanceTelemetryAgent::new(api_key.clone())?;
    let structure_agent = StructureRedesignAgent::new(api_key.clone())?;

    let mut orchestrator = Orchestrator::new("Step1-Test")
        .with_scope_agent(scope_agent)
        .with_governance_agent(governance_agent)
        .with_structure_agent(structure_agent);

    println!("   ✓ Orchestrator initialized with all agents\n");

    // 2. Execute Step 0
    println!("2. Executing Step 0 (Intent Capture)...");
    let user_intent = "Design a REST API for a customer management system. \
        Include CRUD operations, authentication, and proper error handling.";

    let intent_summary = orchestrator.execute_step_0(user_intent).await?;

    println!("   ✓ Step 0 complete");
    println!("   Primary Goal: {}", intent_summary.primary_goal);
    println!("   Confidence: {}", intent_summary.confidence_score);
    println!("   Category: {}", intent_summary.intent_category);
    println!("   State: {:?}\n", orchestrator.state);

    // Verify Step 0 gate pending
    assert!(orchestrator.state.is_gate_pending(), "Step 0 should be in gate pending state");
    println!("   ✓ Step 0 gate is pending (as expected)\n");

    // 3. Approve gate to proceed to Step 1
    println!("3. Approving Ready_for_Step_1 gate...");
    let approved = orchestrator.approve_gate("Test User")?;
    assert!(approved, "Gate approval should succeed");

    println!("   ✓ Gate approved");
    println!("   State: {:?}", orchestrator.state);
    println!("   Active Role: {:?}\n", orchestrator.active_role);

    // Verify transitioned to Step 1
    assert!(
        matches!(orchestrator.state, method_vi_lib::agents::orchestrator::RunState::Step1Active),
        "Should transition to Step1Active after gate approval"
    );
    println!("   ✓ Transitioned to Step1Active (as expected)\n");

    // 4. Execute Step 1
    println!("4. Executing Step 1 (Baseline Establishment)...");
    println!("   This will create 4 immutable artifacts:\n");

    let (intent_anchor_id, charter_id, baseline_id, architecture_id) =
        orchestrator.execute_step_1().await?;

    println!("\n   ✓ Step 1 complete");
    println!("   Created artifacts:");
    println!("      - Intent_Anchor: {}", intent_anchor_id);
    println!("      - Charter: {}", charter_id);
    println!("      - Baseline_Report: {}", baseline_id);
    println!("      - Architecture_Map: {}", architecture_id);
    println!("   State: {:?}\n", orchestrator.state);

    // 5. Verify all 4 artifacts were created
    println!("5. Verifying artifact creation...");

    assert!(
        orchestrator.intent_anchor.is_some(),
        "Intent_Anchor should be created"
    );
    assert!(orchestrator.charter.is_some(), "Charter should be created");
    assert!(
        orchestrator.baseline_report.is_some(),
        "Baseline_Report should be created"
    );
    assert!(
        orchestrator.architecture_map.is_some(),
        "Architecture_Map should be created"
    );

    println!("   ✓ All 4 artifacts exist\n");

    // 6. Verify artifacts are marked as immutable
    println!("6. Verifying artifacts are immutable...");

    let intent_anchor = orchestrator.intent_anchor.as_ref().unwrap();
    let charter = orchestrator.charter.as_ref().unwrap();
    let baseline_report = orchestrator.baseline_report.as_ref().unwrap();
    let architecture_map = orchestrator.architecture_map.as_ref().unwrap();

    // Check frontmatter for is_immutable: true
    assert!(
        intent_anchor.contains("is_immutable: true"),
        "Intent_Anchor should be immutable"
    );
    assert!(
        charter.contains("is_immutable: true"),
        "Charter should be immutable"
    );
    assert!(
        baseline_report.contains("is_immutable: true"),
        "Baseline_Report should be immutable"
    );
    assert!(
        architecture_map.contains("is_immutable: true"),
        "Architecture_Map should be immutable"
    );

    println!("   ✓ All artifacts marked as immutable\n");

    // 7. Verify E_baseline is set
    println!("7. Verifying E_baseline...");

    let e_baseline = orchestrator.get_e_baseline();
    assert!(e_baseline.is_some(), "E_baseline should be set");

    let e_baseline_value = e_baseline.unwrap();
    println!("   E_baseline: {} words", e_baseline_value);
    assert!(e_baseline_value > 0.0, "E_baseline should be greater than 0");

    println!("   ✓ E_baseline is set and locked\n");

    // 8. Verify artifacts contain required sections
    println!("8. Verifying artifact content structure...");

    // Intent_Anchor should have canonical intent statement
    assert!(
        intent_anchor.contains("Intent Anchor") || intent_anchor.contains("# Intent"),
        "Intent_Anchor should have proper headers"
    );

    // Charter should have objectives
    assert!(
        charter.contains("Objective") || charter.contains("Charter"),
        "Charter should have objectives section"
    );

    // Baseline_Report should have E_baseline
    assert!(
        baseline_report.contains("E_baseline") || baseline_report.contains("Baseline"),
        "Baseline_Report should reference E_baseline"
    );

    // Architecture_Map should have architecture
    assert!(
        architecture_map.contains("Architecture") || architecture_map.contains("Process"),
        "Architecture_Map should have architecture description"
    );

    println!("   ✓ All artifacts have expected content structure\n");

    // 9. Verify Baseline_Frozen signal was emitted
    println!("9. Verifying signal emission...");

    let signal_router = orchestrator.get_signal_router();
    let signals = signal_router.get_signal_chain(&orchestrator.run_id);

    // Find Baseline_Frozen signal
    let baseline_frozen_signal = signals.iter().find(|s| {
        matches!(
            s.signal_type,
            method_vi_lib::signals::SignalType::BaselineFrozen
        )
    });

    assert!(
        baseline_frozen_signal.is_some(),
        "Baseline_Frozen signal should be emitted"
    );

    let signal = baseline_frozen_signal.unwrap();
    println!("   ✓ Baseline_Frozen signal emitted");
    println!("      Signal hash: {}", signal.hash);
    println!("      Timestamp: {}", signal.timestamp);
    println!("      Gate required: {}", signal.payload.gate_required);
    println!("      Artifacts produced: {:?}\n", signal.payload.artifacts_produced);

    // Verify signal payload
    assert_eq!(
        signal.payload.step_from, 1,
        "Signal should be from step 1"
    );
    assert_eq!(signal.payload.step_to, 2, "Signal should be to step 2");
    assert!(signal.payload.gate_required, "Gate should be required");
    assert_eq!(
        signal.payload.artifacts_produced.len(),
        4,
        "Should have 4 artifacts produced"
    );

    println!("   ✓ Signal payload validated\n");

    // 10. Verify gate is pending
    println!("10. Verifying gate state...");

    assert!(
        orchestrator.state.is_gate_pending(),
        "Orchestrator should be in gate pending state"
    );
    assert!(
        matches!(
            orchestrator.state,
            method_vi_lib::agents::orchestrator::RunState::Step1GatePending
        ),
        "Should be in Step1GatePending state"
    );

    println!("   ✓ Step 1 gate is pending (awaiting baseline approval)\n");

    // 11. Verify ledger entries
    println!("11. Verifying ledger entries...");

    let ledger = orchestrator.get_ledger();
    let entries = ledger.get_entries(&orchestrator.run_id);

    // Count entry types
    let gate_entries = entries
        .iter()
        .filter(|e| matches!(e.entry_type, method_vi_lib::ledger::EntryType::Gate))
        .count();
    let decision_entries = entries
        .iter()
        .filter(|e| matches!(e.entry_type, method_vi_lib::ledger::EntryType::Decision))
        .count();

    println!("   Ledger summary:");
    println!("      Total entries: {}", entries.len());
    println!("      Gate entries: {}", gate_entries);
    println!("      Decision entries: {}", decision_entries);

    assert!(
        gate_entries >= 2,
        "Should have at least 2 gate entries (Step 0 and Step 1)"
    );
    assert!(
        decision_entries >= 2,
        "Should have at least 2 decision entries"
    );

    println!("   ✓ Ledger entries validated\n");

    // 12. Approve Step 1 gate (baseline frozen)
    println!("12. Approving Baseline_Frozen gate...");

    let approved = orchestrator.approve_gate("Test User")?;
    assert!(approved, "Baseline gate approval should succeed");

    println!("   ✓ Baseline gate approved");
    println!("   State: {:?}", orchestrator.state);
    println!("   Ready for Step 2 (future implementation)\n");

    // Summary
    println!("{}", "=".repeat(80));
    println!("✅ STEP 1 FULL TEST COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nAll verifications passed:");
    println!("  ✓ Step 0 executed successfully");
    println!("  ✓ Ready_for_Step_1 gate approved");
    println!("  ✓ Step 1 executed successfully");
    println!("  ✓ 4 immutable artifacts created:");
    println!("     - Intent_Anchor");
    println!("     - Charter");
    println!("     - Baseline_Report");
    println!("     - Architecture_Map");
    println!("  ✓ E_baseline calculated and locked");
    println!("  ✓ Baseline_Frozen signal emitted");
    println!("  ✓ Step 1 gate pending");
    println!("  ✓ Baseline_Frozen gate approved");
    println!("  ✓ Ledger entries recorded");
    println!("{}", "=".repeat(80));

    Ok(())
}
