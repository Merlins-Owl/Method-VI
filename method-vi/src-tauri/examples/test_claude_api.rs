/// Simple integration test for Claude API
///
/// To run this example:
/// 1. Set your API key: set ANTHROPIC_API_KEY=your-key-here
/// 2. Run: cargo run --example test_claude_api
///
/// Note: This will use real API credits!

use method_vi_lib::api::AnthropicClient;

#[tokio::main]
async fn main() {
    // Initialize logger to see token usage
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    println!("\n=== Testing Claude API ===\n");

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

    // Create client
    let client = match AnthropicClient::new(api_key) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to create client: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Client created");
    println!("\n--- Calling Claude API ---");
    println!("Prompt: Hello, respond with just 'Hello human!'\n");

    // Call Claude
    match client
        .call_claude(
            "",  // Empty system prompt
            "Hello, respond with just 'Hello human!'",
            None,  // Use default model (claude-sonnet-4-20250514)
            Some(20),  // Keep max_tokens low to minimize cost
        )
        .await
    {
        Ok(response) => {
            println!("--- Response ---");
            println!("{}", response);
            println!("\n✓ API call successful!");
            println!("\nNote: Check the logs above for token usage and cost information");
        }
        Err(e) => {
            eprintln!("\n❌ API call failed: {}", e);
            std::process::exit(1);
        }
    }
}
