use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::agents::orchestrator::Orchestrator;
use crate::agents::scope_pattern::{IntentSummary, ScopePatternAgent};
use crate::api::AnthropicClient;
use crate::config::AppConfig;

/// Global orchestrator state
/// In a real application, this would be a map of run_id -> Orchestrator
pub struct OrchestratorState(pub Mutex<Option<Orchestrator>>);

/// Response structure for Step 0 that matches the frontend expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step0Response {
    pub intent_summary: IntentSummaryForFrontend,
    pub clarification_questions: Vec<ClarificationQuestion>,
    pub pattern_recommendations: Vec<PatternRecommendation>,
}

/// Simplified intent summary for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSummaryForFrontend {
    pub user_intent: String,
    pub normalized_goal: String,
    pub success_criteria: Vec<String>,
    pub scope_boundaries: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub question: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecommendation {
    pub pattern_id: String,
    pub intent_category: String,
    pub description: String,
    pub relevance_score: f64,
}

impl From<IntentSummary> for IntentSummaryForFrontend {
    fn from(summary: IntentSummary) -> Self {
        IntentSummaryForFrontend {
            user_intent: summary.user_request.clone(),
            normalized_goal: summary.primary_goal.clone(),
            success_criteria: summary.likely_in_scope.clone(),
            scope_boundaries: summary.likely_out_of_scope.clone(),
            assumptions: summary.edge_cases.clone(),
        }
    }
}

/// Convert questions_for_clarification to ClarificationQuestion list
fn convert_questions(questions: &[String]) -> Vec<ClarificationQuestion> {
    questions
        .iter()
        .filter(|q| !q.contains("None - intent is clear"))
        .map(|q| ClarificationQuestion {
            question: q.clone(),
            context: String::new(),
        })
        .collect()
}

/// Start Step 0: Intent Capture
///
/// This command is called when the user submits their intent in the UI.
/// It creates a new orchestrator (or reuses existing) and executes Step 0.
#[tauri::command]
pub async fn start_step_0(
    run_id: String,
    user_intent: String,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step0Response, String> {
    info!("=== START_STEP_0 command called ===");
    info!("Run ID: {}", run_id);
    info!("User Intent length: {} chars", user_intent.len());

    // Get API key from config
    let api_key = {
        let config = config_state.lock().unwrap();
        config
            .get_api_key()
            .map_err(|e| format!("API key not configured: {}. Please set it in Settings or via ANTHROPIC_API_KEY environment variable.", e))?
    };

    info!("API key found: {}...", &api_key[..15]);

    // Create Anthropic client
    let claude_client = AnthropicClient::new(api_key)
        .map_err(|e| format!("Failed to create Anthropic client: {}", e))?;

    // Create Scope & Pattern Agent
    let scope_agent = ScopePatternAgent::new(claude_client);

    // Create new orchestrator with the scope agent
    let label = run_id
        .split('-')
        .skip(3) // Skip YYYY-MM-DD parts
        .collect::<Vec<_>>()
        .join("-");

    info!("Creating new orchestrator with label: {}", label);
    let mut orchestrator = Orchestrator::new(&label).with_scope_agent(scope_agent);

    // Execute Step 0
    info!("Executing Step 0...");
    let intent_summary = orchestrator
        .execute_step_0(&user_intent)
        .await
        .map_err(|e| format!("Failed to execute Step 0: {}", e))?;

    info!("Step 0 completed successfully");
    info!("Intent: {}", intent_summary.primary_goal);

    // Convert to frontend format
    let frontend_summary = IntentSummaryForFrontend::from(intent_summary.clone());

    // Extract clarification questions
    let clarification_questions = convert_questions(&intent_summary.questions_for_clarification);

    // For now, pattern recommendations are empty (would query Learning Plane in future)
    let pattern_recommendations = vec![];

    // Store orchestrator in state for future gate approval
    {
        let mut guard = state.0.lock().unwrap();
        *guard = Some(orchestrator);
        info!("Orchestrator stored in state successfully");
        info!("State contains orchestrator: {}", guard.is_some());
    }

    Ok(Step0Response {
        intent_summary: frontend_summary,
        clarification_questions,
        pattern_recommendations,
    })
}

/// Approve the gate and proceed to Step 1
///
/// This command is called when the user clicks "Approve & Continue" in the UI.
#[tauri::command]
pub async fn approve_gate(
    approver: String,
    state: State<'_, OrchestratorState>,
) -> Result<(), String> {
    info!("=== APPROVE_GATE command called ===");
    info!("Approver: {}", approver);

    let mut orch_guard = state.0.lock().unwrap();
    info!("State lock acquired");
    info!("State contains orchestrator: {}", orch_guard.is_some());

    let orchestrator = orch_guard
        .as_mut()
        .ok_or_else(|| {
            let err = "No active run found in approve_gate".to_string();
            log::error!("{}", err);
            err
        })?;

    info!("Orchestrator found, current state: {:?}", orchestrator.state);

    orchestrator
        .approve_gate(&approver)
        .map_err(|e| {
            let err = format!("Failed to approve gate: {}", e);
            log::error!("{}", err);
            err
        })?;

    info!("Gate approved successfully");
    info!("New orchestrator state: {:?}", orchestrator.state);
    info!("State still contains orchestrator: {}", orch_guard.is_some());

    Ok(())
}

/// Reject the gate (user wants to adjust intent)
///
/// This command is called when the user clicks "Adjust Intent" in the UI.
#[tauri::command]
pub async fn reject_gate(
    rejector: String,
    reason: String,
    state: State<'_, OrchestratorState>,
) -> Result<(), String> {
    info!("=== REJECT_GATE command called ===");
    info!("Rejector: {}", rejector);

    let mut orch_guard = state.0.lock().unwrap();
    let orchestrator = orch_guard
        .as_mut()
        .ok_or_else(|| "No active run found".to_string())?;

    orchestrator
        .reject_gate(&rejector, &reason)
        .map_err(|e| format!("Failed to reject gate: {}", e))?;

    info!("Gate rejected");
    Ok(())
}

/// Submit clarification answers
///
/// This command handles clarification questions if the agent asks for more details.
/// For MVP, we'll just re-run Step 0 with the updated context.
#[tauri::command]
pub async fn submit_clarifications(
    run_id: String,
    answers: Vec<String>,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step0Response, String> {
    info!("=== SUBMIT_CLARIFICATIONS command called ===");
    info!("Run ID: {}", run_id);
    info!("Answers: {:?}", answers);

    // Get the original intent from the orchestrator
    let original_intent = {
        let orch_guard = state.0.lock().unwrap();
        let orchestrator = orch_guard
            .as_ref()
            .ok_or_else(|| "No active run found".to_string())?;

        // Get the original user request from the intent summary
        orchestrator
            .intent_summary
            .as_ref()
            .ok_or_else(|| "No intent summary found".to_string())?
            .user_request
            .clone()
    };

    // Combine original intent with answers
    let updated_intent = format!(
        "{}\n\nAdditional context:\n{}",
        original_intent,
        answers.join("\n")
    );

    // Re-run Step 0 with the updated intent
    start_step_0(run_id, updated_intent, state, config_state).await
}
