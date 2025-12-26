use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::step0::OrchestratorState;
use crate::agents::validation_learning::PatternCard;

/// Response from execute_step_6_5 command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step6_5Response {
    pub knowledge_update: String,
    pub pattern_cards: Vec<PatternCardForFrontend>,
    pub success_count: usize,
    pub failure_count: usize,
    pub optimization_count: usize,
}

/// Pattern card simplified for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCardForFrontend {
    pub pattern_id: String,
    pub pattern_name: String,
    pub category: String,           // "Success" / "Failure" / "Optimization"
    pub context: String,
    pub mechanics: String,
    pub efficacy: f64,              // 0.0-1.0
    pub reusability: String,        // "High" / "Medium" / "Low"
    pub recommendation: String,
}

impl From<PatternCard> for PatternCardForFrontend {
    fn from(card: PatternCard) -> Self {
        PatternCardForFrontend {
            pattern_id: card.pattern_id,
            pattern_name: card.pattern_name,
            category: card.category,
            context: card.context,
            mechanics: card.mechanics,
            efficacy: card.efficacy,
            reusability: card.reusability,
            recommendation: card.recommendation,
        }
    }
}

/// Execute Step 6.5: Learning Harvest
///
/// CRITICAL: This command REUSES the validation agent from Step 6.
/// DO NOT create a new agent or pass config_state.
#[tauri::command]
pub async fn execute_step_6_5(
    run_id: String,
    state: State<'_, OrchestratorState>,
    // NO config_state - agent already attached in Step 6
) -> Result<Step6_5Response, String> {
    info!("=== EXECUTE_STEP_6_5 command called ===");
    info!("Run ID: {}", run_id);
    info!("Timestamp: {}", chrono::Utc::now().to_rfc3339());

    // Get the orchestrator from state (take() → modify → return pattern)
    info!("Acquiring state lock...");
    let mut orchestrator = {
        let mut orch_guard = state.0.lock().unwrap();
        info!("State lock acquired");
        info!("State contains orchestrator: {}", orch_guard.is_some());

        // Take ownership of the orchestrator temporarily
        let orch = orch_guard
            .take()
            .ok_or_else(|| {
                let err = "No active run found. Please complete Steps 0-6 first.".to_string();
                log::error!("[EXECUTE_STEP_6_5] {}", err);
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);

        // DO NOT create or attach new agent - agent was attached in Step 6
        orch
    }; // Lock is released here

    // Execute Step 6.5 (now without holding the lock)
    info!("Executing Step 6.5 workflow...");
    let harvest_result = orchestrator
        .execute_step_6_5()
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to execute Step 6.5: {}", e);
            log::error!("[EXECUTE_STEP_6_5] {}", err_msg);
            err_msg
        })?;

    info!("✓ Step 6.5 execution complete");

    // Convert pattern cards for frontend
    let pattern_cards: Vec<PatternCardForFrontend> = harvest_result
        .pattern_cards
        .into_iter()
        .map(PatternCardForFrontend::from)
        .collect();

    info!("Extracted {} pattern cards for frontend", pattern_cards.len());

    // Return orchestrator to state
    {
        let mut orch_guard = state.0.lock().unwrap();
        *orch_guard = Some(orchestrator);
    }

    info!("Orchestrator returned to state");

    // Build response
    let response = Step6_5Response {
        knowledge_update: harvest_result.knowledge_update,
        pattern_cards,
        success_count: harvest_result.success_count,
        failure_count: harvest_result.failure_count,
        optimization_count: harvest_result.optimization_count,
    };

    info!("=== EXECUTE_STEP_6_5 complete ===");

    Ok(response)
}
