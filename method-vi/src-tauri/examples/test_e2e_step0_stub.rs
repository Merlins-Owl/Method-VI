/// End-to-end test for Step 0 workflow (using stubbed agent - no API key required)
///
/// This test verifies:
/// 1. Start new run
/// 2. Provide intent: "Create a project plan for launching a new mobile app"
/// 3. Verify Intent_Summary artifact is created with correct structure
/// 4. Verify Ready_for_Step_1 signal emitted
/// 5. Verify ledger and signal chain integrity
///
/// To run this test:
/// cargo run --example test_e2e_step0_stub

use method_vi_lib::agents::Orchestrator;
use method_vi_lib::signals::SignalType;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║      Method-VI Step 0 - End-to-End Test (Stubbed)           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ═══════════════════════════════════════════════════════════════
    // TEST 1: Start new run
    // ═══════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ TEST 1: Start new run                                      │");
    println!("└─────────────────────────────────────────────────────────────┘");

    let mut orch = Orchestrator::new("Mobile-App-Launch");

    println!("✓ New run created");
    println!("  Run ID: {}", orch.run_id);
    println!("  Initial state: {:?}", orch.state);
    println!("  Active role: {:?}", orch.active_role);
    println!("  Steno-Ledger: {}", orch.generate_steno_ledger());

    // Verify initial state
    assert!(orch.run_id.contains("Mobile-App-Launch"), "Run ID should contain label");
    assert!(matches!(
        orch.state,
        method_vi_lib::agents::orchestrator::RunState::Step0Active
    ), "Should be in Step0Active state");
    assert!(matches!(
        orch.active_role,
        method_vi_lib::context::Role::Observer
    ), "Should have Observer role");

    println!("✓ All initial state assertions passed\n");

    // ═══════════════════════════════════════════════════════════════
    // TEST 2: Provide intent and execute Step 0
    // ═══════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ TEST 2: Execute Step 0 with user intent (stubbed agent)    │");
    println!("└─────────────────────────────────────────────────────────────┘");

    let user_intent = "Create a project plan for launching a new mobile app";
    println!("User Intent: \"{}\"", user_intent);
    println!("\nExecuting Step 0 with stubbed Scope & Pattern Agent...");

    let intent_result = orch.execute_step_0(user_intent).await;

    if let Err(e) = &intent_result {
        eprintln!("\n❌ Step 0 execution failed: {}", e);
        std::process::exit(1);
    }

    let intent_summary = intent_result.unwrap();
    println!("✓ Step 0 execution completed\n");

    // ═══════════════════════════════════════════════════════════════
    // TEST 3: Verify Intent_Summary artifact structure
    // ═══════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ TEST 3: Verify Intent_Summary artifact structure           │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("Artifact Metadata:");
    println!("  artifact_id: {}", intent_summary.artifact_id);
    println!("  artifact_type: {}", intent_summary.artifact_type);
    println!("  run_id: {}", intent_summary.run_id);
    println!("  step_origin: {}", intent_summary.step_origin);
    println!("  created_at: {}", intent_summary.created_at);
    println!("  hash: {}", intent_summary.hash);
    println!("  author: {}", intent_summary.author);
    println!("  governance_role: {}", intent_summary.governance_role);
    println!("  is_immutable: {}", intent_summary.is_immutable);

    // Verify artifact structure
    assert!(intent_summary.artifact_id.contains(&orch.run_id), "Artifact ID should contain run_id");
    assert_eq!(intent_summary.artifact_type, "Intent_Summary", "Should be Intent_Summary type");
    assert_eq!(intent_summary.run_id, orch.run_id, "Run ID should match");
    assert_eq!(intent_summary.step_origin, 0, "Should originate from Step 0");
    assert!(!intent_summary.hash.is_empty(), "Hash should not be empty");
    assert_eq!(intent_summary.hash.len(), 64, "SHA-256 hash should be 64 chars");
    assert!(intent_summary.author.contains("scope-pattern-agent"), "Author should be scope-pattern-agent");
    assert_eq!(intent_summary.governance_role, "Observer", "Role should be Observer");
    assert_eq!(intent_summary.is_immutable, false, "Should not be immutable yet");

    println!("\n✓ All artifact metadata assertions passed");

    println!("\nIntent Extraction:");
    println!("  Primary Goal: {}", intent_summary.primary_goal);
    println!("  Audience: {}", intent_summary.audience);
    println!("  Expected Outcome: {}", intent_summary.expected_outcome);
    println!("  Intent Category: {}", intent_summary.intent_category);

    assert!(!intent_summary.primary_goal.is_empty(), "Primary goal should not be empty");
    assert!(!intent_summary.audience.is_empty(), "Audience should not be empty");
    assert!(!intent_summary.expected_outcome.is_empty(), "Expected outcome should not be empty");
    assert!(!intent_summary.intent_category.is_empty(), "Intent category should not be empty");

    println!("\n✓ All intent extraction assertions passed");

    println!("\nConfidence Assessment:");
    println!("  Score: {}/100", intent_summary.confidence_score);
    println!("  Explanation: {}", intent_summary.confidence_explanation);

    assert!(intent_summary.confidence_score > 0, "Confidence score should be > 0");
    assert!(intent_summary.confidence_score <= 100, "Confidence score should be <= 100");

    println!("\n✓ Confidence assessment assertions passed");

    println!("\nScope Boundaries:");
    println!("  In Scope Items: {}", intent_summary.likely_in_scope.len());
    for (i, item) in intent_summary.likely_in_scope.iter().enumerate() {
        println!("    {}. {}", i + 1, item);
    }
    println!("  Out of Scope Items: {}", intent_summary.likely_out_of_scope.len());
    for (i, item) in intent_summary.likely_out_of_scope.iter().enumerate() {
        println!("    {}. {}", i + 1, item);
    }

    assert!(!intent_summary.likely_in_scope.is_empty() || !intent_summary.likely_out_of_scope.is_empty(),
        "Should have at least some scope boundaries defined");

    println!("\n✓ Scope boundaries assertions passed");

    // Verify hash integrity
    println!("\nVerifying hash integrity...");
    let recomputed_hash = intent_summary.compute_hash();
    assert_eq!(intent_summary.hash, recomputed_hash, "Hash should match recomputed hash");
    println!("✓ Hash integrity verified");

    // Display complete artifact markdown
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Complete Artifact (Markdown with YAML Frontmatter)         │");
    println!("└─────────────────────────────────────────────────────────────┘\n");
    println!("{}", intent_summary.to_markdown());

    println!("\n✓ ALL Intent_Summary artifact structure tests PASSED\n");

    // ═══════════════════════════════════════════════════════════════
    // TEST 4: Verify Ready_for_Step_1 signal emitted
    // ═══════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ TEST 4: Verify Ready_for_Step_1 signal emitted             │");
    println!("└─────────────────────────────────────────────────────────────┘");

    let signal_chain = orch.get_signal_router().get_signal_chain(&orch.run_id);
    println!("Signal chain length: {}", signal_chain.len());

    assert!(!signal_chain.is_empty(), "Signal chain should not be empty");
    assert_eq!(signal_chain.len(), 1, "Should have exactly 1 signal");

    let signal = signal_chain.last().unwrap();
    println!("\nSignal Details:");
    println!("  Type: {:?}", signal.signal_type);
    println!("  Hash: {}", signal.hash);
    println!("  Prior Signal Hash: {:?}", signal.prior_signal_hash);
    println!("  Timestamp: {}", signal.timestamp);
    println!("  Gate Required: {}", signal.payload.gate_required);
    println!("  Step Transition: {} → {}", signal.payload.step_from, signal.payload.step_to);
    println!("  Artifacts Produced: {:?}", signal.payload.artifacts_produced);

    assert!(matches!(signal.signal_type, SignalType::ReadyForStep1),
        "Signal should be ReadyForStep1");
    assert!(signal.payload.gate_required, "Gate should be required");
    assert_eq!(signal.payload.step_from, 0, "Should transition from step 0");
    assert_eq!(signal.payload.step_to, 1, "Should transition to step 1");
    assert!(!signal.hash.is_empty(), "Signal hash should not be empty");
    assert_eq!(signal.hash.len(), 64, "Signal hash should be 64 chars (SHA-256)");

    println!("\n✓ All signal assertions passed");

    // Verify signal chain integrity
    println!("\nVerifying signal chain integrity...");
    let chain_valid = orch.get_signal_router().verify_chain_integrity(&orch.run_id);
    assert!(chain_valid, "Signal chain should be valid");
    println!("✓ Signal chain integrity verified\n");

    // ═══════════════════════════════════════════════════════════════
    // FINAL STATE VERIFICATION
    // ═══════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ FINAL STATE VERIFICATION                                    │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("Current State:");
    println!("  Orchestrator State: {:?}", orch.state);
    println!("  Active Role: {:?}", orch.active_role);
    println!("  Step Number: {}", orch.state.step_number());
    println!("  Gate Pending: {}", orch.state.is_gate_pending());

    assert!(matches!(
        orch.state,
        method_vi_lib::agents::orchestrator::RunState::Step0GatePending
    ), "Should be in Step0GatePending state");
    assert_eq!(orch.state.step_number(), 0, "Should still be on step 0");
    assert!(orch.state.is_gate_pending(), "Should be waiting for gate approval");

    println!("\n✓ Final state assertions passed");

    println!("\nLedger Entries:");
    let ledger_entries = orch.get_ledger().get_entries(&orch.run_id);
    println!("  Total entries: {}", ledger_entries.len());
    for (i, entry) in ledger_entries.iter().enumerate() {
        println!("  {}. {} ({:?})", i + 1, entry.payload.action, entry.entry_type);
    }

    assert!(ledger_entries.len() >= 3, "Should have at least 3 ledger entries");

    // Verify ledger chain integrity
    println!("\nVerifying ledger chain integrity...");
    let ledger_chain_valid = orch.get_ledger().verify_chain_integrity(&orch.run_id);
    assert!(ledger_chain_valid, "Ledger chain should be valid");
    println!("✓ Ledger chain integrity verified");

    println!("\nSteno-Ledger:");
    println!("  {}", orch.generate_steno_ledger());

    // ═══════════════════════════════════════════════════════════════
    // TEST SUMMARY
    // ═══════════════════════════════════════════════════════════════
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    TEST SUMMARY                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ✓ TEST 1: New run created successfully                      ║");
    println!("║ ✓ TEST 2: Step 0 executed with stubbed agent                ║");
    println!("║ ✓ TEST 3: Intent_Summary artifact structure verified        ║");
    println!("║ ✓ TEST 4: Ready_for_Step_1 signal emitted and verified      ║");
    println!("║ ✓ Ledger chain integrity verified                           ║");
    println!("║ ✓ Signal chain integrity verified                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  Status: ✓ ALL TESTS PASSED                                 ║");
    println!("║                                                              ║");
    println!("║  Completed Functionality:                                    ║");
    println!("║  ✓ Orchestrator run creation                                ║");
    println!("║  ✓ Step 0 execution workflow                                ║");
    println!("║  ✓ Intent_Summary artifact generation                       ║");
    println!("║  ✓ SHA-256 hash computation and verification                ║");
    println!("║  ✓ Signal emission (Ready_for_Step_1)                       ║");
    println!("║  ✓ Ledger recording (run_start, intent_captured, gate)      ║");
    println!("║  ✓ Hash chain integrity (ledger + signals)                  ║");
    println!("║  ✓ State transitions (Step0Active → Step0GatePending)       ║");
    println!("║  ✓ Steno-Ledger generation                                  ║");
    println!("║                                                              ║");
    println!("║  Pending Features:                                           ║");
    println!("║  ⚠ Artifact database persistence                            ║");
    println!("║  ⚠ Coherence Spine registration                             ║");
    println!("║  ⚠ Pattern recommendation from Knowledge Repository         ║");
    println!("║                                                              ║");
    println!("║  Next Steps:                                                 ║");
    println!("║  - To test with real Claude API:                            ║");
    println!("║    set ANTHROPIC_API_KEY=your-key                           ║");
    println!("║    cargo run --example test_e2e_step0                       ║");
    println!("║  - Call approve_gate(\"approver\") to proceed to Step 1       ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Run ID: {}", orch.run_id);
    println!("Artifact ID: {}", intent_summary.artifact_id);
    println!("Artifact Hash: {}", intent_summary.hash);
}
