/// Integration test for Scope & Pattern Agent with real Claude API
///
/// This example demonstrates the complete Step 0 workflow with actual Claude API calls:
/// 1. Create Orchestrator with real Scope & Pattern Agent
/// 2. Execute Step 0 with user intent
/// 3. Claude interprets the intent and returns Intent_Summary artifact
/// 4. Display the artifact contents
/// 5. Show the Steno-Ledger context
///
/// To run this example:
/// 1. Set your API key: set ANTHROPIC_API_KEY=your-key-here
/// 2. Run: cargo run --example test_scope_pattern_agent
///
/// Note: This will use real API credits!

use method_vi_lib::agents::{Orchestrator, ScopePatternAgent};
use method_vi_lib::api::AnthropicClient;

#[tokio::main]
async fn main() {
    // Initialize logger to see detailed output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    println!("\n=== Testing Scope & Pattern Agent with Real Claude API ===\n");

    // Get API key from environment
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("❌ Error: ANTHROPIC_API_KEY environment variable not set");
            eprintln!("\nPlease set your API key:");
            eprintln!("  Windows: set ANTHROPIC_API_KEY=your-key-here");
            eprintln!("  Linux/Mac: export ANTHROPIC_API_KEY=your-key-here");
            std::process::exit(1);
        }
    };

    println!("✓ API key found");

    // Create Claude API client
    let claude_client = match AnthropicClient::new(api_key) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to create Claude client: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Claude API client created");

    // Create Scope & Pattern Agent
    let scope_agent = ScopePatternAgent::new(claude_client);
    println!("✓ Scope & Pattern Agent created");

    // Create Orchestrator with the real agent
    let mut orch = Orchestrator::new("API-Test").with_scope_agent(scope_agent);
    println!("✓ Orchestrator created with run_id: {}", orch.run_id);

    // Display initial Steno-Ledger
    println!("\n--- Initial Steno-Ledger ---");
    println!("{}", orch.generate_steno_ledger());

    // Define user intent
    let user_intent = "Build a secure REST API for managing user accounts with authentication, authorization, and audit logging capabilities. The API should support CRUD operations on user profiles and maintain a complete audit trail of all changes.";

    println!("\n--- Executing Step 0: Intent Capture ---");
    println!("User Intent:");
    println!("  {}", user_intent);
    println!("\nCalling Claude API...");

    // Execute Step 0 (this will call the real Claude API)
    match orch.execute_step_0(user_intent).await {
        Ok(intent_summary) => {
            println!("\n✓ Step 0 completed successfully!");
            println!("\n=== Intent Summary Artifact ===");
            println!("\nArtifact ID: {}", intent_summary.artifact_id);
            println!("Content Hash: {}", intent_summary.hash);
            println!("\n--- Primary Goal ---");
            println!("{}", intent_summary.primary_goal);
            println!("\n--- Audience ---");
            println!("{}", intent_summary.audience);
            println!("\n--- Expected Outcome ---");
            println!("{}", intent_summary.expected_outcome);
            println!("\n--- Intent Category ---");
            println!("{}", intent_summary.intent_category);
            println!("\n--- Confidence Assessment ---");
            println!("Score: {}/100", intent_summary.confidence_score);
            println!("Explanation: {}", intent_summary.confidence_explanation);
            println!("\n--- Clarity Indicators ---");
            println!("Request Specificity: {}", intent_summary.request_specificity);
            println!("Scope Definition: {}", intent_summary.scope_definition_clarity);
            println!("Success Criteria: {}", intent_summary.success_criteria_state);
            println!("\n--- Questions for Clarification ---");
            for (i, q) in intent_summary.questions_for_clarification.iter().enumerate() {
                println!("{}. {}", i + 1, q);
            }
            println!("\n--- Preliminary Scope Boundaries ---");
            println!("\nLikely In Scope:");
            for item in &intent_summary.likely_in_scope {
                println!("  • {}", item);
            }
            println!("\nLikely Out of Scope:");
            for item in &intent_summary.likely_out_of_scope {
                println!("  • {}", item);
            }
            if !intent_summary.edge_cases.is_empty() {
                println!("\nEdge Cases (Need Confirmation):");
                for item in &intent_summary.edge_cases {
                    println!("  • {}", item);
                }
            }

            // Display full markdown artifact
            println!("\n\n=== Complete Artifact (Markdown) ===");
            println!("{}", intent_summary.to_markdown());

            // Display current state
            println!("\n=== Current State ===");
            println!("Orchestrator State: {:?}", orch.state);
            println!("Active Role: {:?}", orch.active_role);
            println!("\n--- Updated Steno-Ledger ---");
            println!("{}", orch.generate_steno_ledger());

            // Display signal chain
            println!("\n=== Signal Chain ===");
            let signals = orch.get_signal_router().get_signal_chain(&orch.run_id);
            println!("Total signals emitted: {}", signals.len());
            for (i, signal) in signals.iter().enumerate() {
                println!("\nSignal {}: {:?}", i + 1, signal.signal_type);
                println!("  Hash: {}", signal.hash);
                println!("  Gate Required: {}", signal.payload.gate_required);
                println!("  Step Transition: {} → {}", signal.payload.step_from, signal.payload.step_to);
            }

            // Display ledger entries
            println!("\n=== Ledger Entries ===");
            let ledger_entries = orch.get_ledger().get_entries(&orch.run_id);
            println!("Total ledger entries: {}", ledger_entries.len());
            for (i, entry) in ledger_entries.iter().enumerate() {
                println!("\nEntry {}: {}", i + 1, entry.payload.action);
                println!("  Type: {:?}", entry.entry_type);
                println!("  Hash: {}", entry.hash);
            }

            // Verify hash chain
            println!("\n=== Hash Chain Verification ===");
            let chain_valid = orch.get_ledger().verify_chain_integrity(&orch.run_id);
            println!("Ledger chain integrity: {}", if chain_valid { "✓ Valid" } else { "✗ Broken" });

            let signal_chain_valid = orch.get_signal_router().verify_chain_integrity(&orch.run_id);
            println!("Signal chain integrity: {}", if signal_chain_valid { "✓ Valid" } else { "✗ Broken" });

            println!("\n=== Test Complete ===");
            println!("✓ Scope & Pattern Agent successfully interpreted user intent");
            println!("✓ Intent_Summary artifact created with hash: {}", intent_summary.hash);
            println!("✓ Ready for human gate approval to proceed to Step 1");
            println!("\nNext step: Call orch.approve_gate(approver) to transition to Step 1");
        }
        Err(e) => {
            eprintln!("\n❌ Step 0 execution failed: {}", e);
            eprintln!("\nPossible causes:");
            eprintln!("  • Invalid or expired API key");
            eprintln!("  • Network connectivity issues");
            eprintln!("  • API rate limiting");
            eprintln!("  • Insufficient API credits");
            std::process::exit(1);
        }
    }
}
