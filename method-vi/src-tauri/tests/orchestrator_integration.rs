/// Integration test for Orchestrator Step 0 workflow
///
/// This test verifies the complete Step 0 flow:
/// 1. Create new run
/// 2. Execute Step 0 (intent capture)
/// 3. Verify gate signal emission
/// 4. Simulate human gate approval
/// 5. Verify state transitions
/// 6. Verify ledger entries

use method_vi_lib::agents::Orchestrator;

#[tokio::test]
async fn test_complete_step_0_workflow() {
    println!("\n=== Integration Test: Complete Step 0 Workflow ===\n");

    // Step 1: Start a new run with label "Test-Run"
    println!("Step 1: Creating new run with label 'Test-Run'");
    let mut orch = Orchestrator::new("Test-Run");

    println!("  ✓ Run created");
    println!("  Run ID: {}", orch.run_id);
    println!("  Initial state: {:?}", orch.state);
    println!("  Active role: {:?}", orch.active_role);

    // Verify initial state
    assert!(orch.run_id.contains("Test-Run"));
    assert!(orch.run_id.contains("-")); // Should have date prefix
    assert!(matches!(
        orch.state,
        method_vi_lib::agents::orchestrator::RunState::Step0Active
    ));
    assert!(matches!(
        orch.active_role,
        method_vi_lib::context::Role::Observer
    ));

    // Step 2: Run Step 0 (with stubbed agent)
    println!("\nStep 2: Executing Step 0 - Intent Capture");

    let user_intent = "Build a secure user authentication system with OAuth2 support";
    println!("  User intent: {}", user_intent);

    let intent_result = orch.execute_step_0(user_intent).await;
    assert!(
        intent_result.is_ok(),
        "Step 0 execution failed: {:?}",
        intent_result.err()
    );

    let intent = intent_result.unwrap();
    println!("  ✓ Intent captured");
    println!("  Intent: {}", intent.user_request);
    println!("  Primary Goal: {}", intent.primary_goal);
    println!("  Confidence: {}", intent.confidence_score);

    // Verify intent was captured
    assert_eq!(intent.user_request, user_intent);
    assert!(!intent.primary_goal.is_empty());
    assert!(intent.confidence_score > 0);

    // Step 3: Verify Ready_for_Step_1 signal was emitted
    println!("\nStep 3: Verifying Ready_for_Step_1 signal emission");

    let signal_chain = orch.get_signal_router().get_signal_chain(&orch.run_id);
    println!("  Signal chain length: {}", signal_chain.len());

    assert!(
        !signal_chain.is_empty(),
        "No signals emitted - signal chain is empty"
    );

    let signal = signal_chain.last().unwrap();
    println!("  Latest signal type: {:?}", signal.signal_type);
    println!("  Gate required: {}", signal.payload.gate_required);
    println!("  Step transition: {} → {}", signal.payload.step_from, signal.payload.step_to);
    println!("  Signal hash: {}", signal.hash);

    // Verify signal details
    assert!(matches!(
        signal.signal_type,
        method_vi_lib::signals::SignalType::ReadyForStep1
    ));
    assert!(signal.payload.gate_required, "Gate should be required");
    assert_eq!(signal.payload.step_from, 0);
    assert_eq!(signal.payload.step_to, 1);
    assert!(!signal.hash.is_empty());

    println!("  ✓ Ready_for_Step_1 signal verified");

    // Verify state is now gate pending
    println!("\nStep 3b: Verifying state is now gate pending");
    println!("  Current state: {:?}", orch.state);

    assert!(matches!(
        orch.state,
        method_vi_lib::agents::orchestrator::RunState::Step0GatePending
    ));
    assert!(orch.state.is_gate_pending());
    println!("  ✓ State is Step0GatePending");

    // Step 4: Simulate human approval
    println!("\nStep 4: Simulating human gate approval");

    let approver = "Jane Smith (QA Lead)";
    println!("  Approver: {}", approver);

    let approval_result = orch.approve_gate(approver);
    assert!(
        approval_result.is_ok(),
        "Gate approval failed: {:?}",
        approval_result.err()
    );

    let approved = approval_result.unwrap();
    assert!(approved, "Gate approval returned false");
    println!("  ✓ Gate approved successfully");

    // Step 5: Verify state transitioned correctly
    println!("\nStep 5: Verifying state transition");
    println!("  New state: {:?}", orch.state);
    println!("  New role: {:?}", orch.active_role);

    assert!(
        matches!(
            orch.state,
            method_vi_lib::agents::orchestrator::RunState::Step1Active
        ),
        "State should be Step1Active, got: {:?}",
        orch.state
    );

    assert!(
        matches!(
            orch.active_role,
            method_vi_lib::context::Role::Conductor
        ),
        "Role should be Conductor, got: {:?}",
        orch.active_role
    );

    assert_eq!(orch.state.step_number(), 1, "Should be on step 1");
    assert!(!orch.state.is_gate_pending(), "Should not be gate pending");

    println!("  ✓ State transitioned: Step0GatePending → Step1Active");
    println!("  ✓ Role transitioned: Observer → Conductor");

    // Step 6: Check ledger has the gate approval entry
    println!("\nStep 6: Verifying ledger entries");

    let ledger_entries = orch.get_ledger().get_entries(&orch.run_id);
    println!("  Total ledger entries: {}", ledger_entries.len());

    // We should have at least:
    // 1. run_start
    // 2. intent_captured
    // 3. gate_signal_emitted
    // 4. gate_approved
    assert!(
        ledger_entries.len() >= 4,
        "Expected at least 4 ledger entries, got {}",
        ledger_entries.len()
    );

    // Find the gate approval entry
    let gate_approval = ledger_entries
        .iter()
        .find(|e| e.payload.action == "gate_approved");

    assert!(
        gate_approval.is_some(),
        "No 'gate_approved' entry found in ledger"
    );

    let gate_approval = gate_approval.unwrap();
    println!("\n  Gate approval ledger entry found:");
    println!("    Action: {}", gate_approval.payload.action);
    println!("    Entry type: {:?}", gate_approval.entry_type);
    println!("    Step: {:?}", gate_approval.step);
    println!("    Hash: {}", gate_approval.hash);

    // Verify the approval entry details
    assert!(matches!(
        gate_approval.entry_type,
        method_vi_lib::ledger::EntryType::Decision
    ));

    if let Some(inputs) = &gate_approval.payload.inputs {
        // Verify approver is recorded
        let gate_field = inputs.get("gate");
        let approver_field = inputs.get("approver");

        assert!(gate_field.is_some(), "Gate field missing in approval entry");
        assert!(approver_field.is_some(), "Approver field missing in approval entry");

        assert_eq!(
            gate_field.unwrap().as_str().unwrap(),
            "Ready_for_Step_1"
        );
        assert_eq!(approver_field.unwrap().as_str().unwrap(), approver);
    } else {
        panic!("Gate approval entry has no inputs");
    }

    println!("  ✓ Gate approval entry verified");

    // Verify hash chain integrity
    println!("\nStep 6b: Verifying ledger hash chain integrity");
    let chain_valid = orch.get_ledger().verify_chain_integrity(&orch.run_id);
    println!("  Chain integrity: {}", chain_valid);
    assert!(chain_valid, "Ledger hash chain is broken");
    println!("  ✓ Ledger hash chain is valid");

    // Print summary
    println!("\n=== Test Summary ===");
    println!("✓ Run created: {}", orch.run_id);
    println!("✓ Step 0 executed successfully");
    println!("✓ Ready_for_Step_1 signal emitted");
    println!("✓ Gate approval recorded");
    println!("✓ State transitioned: Step0Active → Step0GatePending → Step1Active");
    println!("✓ Role transitioned: Observer → Conductor");
    println!("✓ {} ledger entries created", ledger_entries.len());
    println!("✓ Ledger hash chain valid");
    println!("\n=== Integration Test PASSED ===\n");
}

#[tokio::test]
async fn test_gate_rejection_workflow() {
    println!("\n=== Integration Test: Gate Rejection Workflow ===\n");

    let mut orch = Orchestrator::new("Test-Reject");

    // Execute Step 0
    println!("Executing Step 0...");
    orch.execute_step_0("Test intent for rejection").await.unwrap();
    println!("  ✓ Step 0 complete - gate pending");

    // Verify gate is pending
    assert!(orch.state.is_gate_pending());

    // Reject the gate
    println!("\nRejecting gate...");
    let rejection_result = orch.reject_gate("John Doe", "Scope too broad");
    assert!(rejection_result.is_ok());
    println!("  ✓ Gate rejected");

    // Verify state is halted
    println!("\nVerifying halted state...");
    assert!(matches!(
        orch.state,
        method_vi_lib::agents::orchestrator::RunState::Halted { .. }
    ));
    println!("  ✓ State is Halted");

    // Verify ledger has rejection entry
    println!("\nVerifying ledger entries...");
    let ledger_entries = orch.get_ledger().get_entries(&orch.run_id);
    let rejection_entry = ledger_entries
        .iter()
        .find(|e| e.payload.action == "gate_rejected");

    assert!(rejection_entry.is_some(), "No rejection entry in ledger");

    let rejection_entry = rejection_entry.unwrap();
    println!("  Gate rejection entry found:");
    println!("    Action: {}", rejection_entry.payload.action);

    if let Some(inputs) = &rejection_entry.payload.inputs {
        let rejector = inputs.get("rejector").unwrap().as_str().unwrap();
        let reason = inputs.get("reason").unwrap().as_str().unwrap();

        assert_eq!(rejector, "John Doe");
        assert_eq!(reason, "Scope too broad");
    }

    println!("  ✓ Gate rejection recorded correctly");
    println!("\n=== Gate Rejection Test PASSED ===\n");
}

#[tokio::test]
async fn test_steno_ledger_updates_during_workflow() {
    println!("\n=== Integration Test: Steno-Ledger Updates ===\n");

    let mut orch = Orchestrator::new("Steno-Test");

    // Check initial Steno-Ledger
    println!("Initial Steno-Ledger:");
    let steno_initial = orch.generate_steno_ledger();
    println!("  {}", steno_initial);

    assert!(steno_initial.contains("S:0"));
    assert!(steno_initial.contains("R:OBS"));
    assert!(steno_initial.contains("🚦:Initializing"));

    // Execute Step 0
    println!("\nExecuting Step 0...");
    orch.execute_step_0("Test intent").await.unwrap();

    // Check Steno-Ledger after Step 0 (gate pending)
    println!("\nSteno-Ledger after Step 0 (gate pending):");
    let steno_gate_pending = orch.generate_steno_ledger();
    println!("  {}", steno_gate_pending);

    assert!(steno_gate_pending.contains("S:0"));
    assert!(steno_gate_pending.contains("R:OBS"));
    assert!(steno_gate_pending.contains("🚦:Awaiting_Gate"));

    // Approve gate
    println!("\nApproving gate...");
    orch.approve_gate("Test User").unwrap();

    // Check Steno-Ledger after gate approval (Step 1 active)
    println!("\nSteno-Ledger after gate approval (Step 1):");
    let steno_step1 = orch.generate_steno_ledger();
    println!("  {}", steno_step1);

    assert!(steno_step1.contains("S:1"));
    assert!(steno_step1.contains("R:COND"));  // Role changed to Conductor
    assert!(steno_step1.contains("🚦:Active"));

    println!("\n✓ Steno-Ledger correctly updates throughout workflow");
    println!("\n=== Steno-Ledger Test PASSED ===\n");
}
