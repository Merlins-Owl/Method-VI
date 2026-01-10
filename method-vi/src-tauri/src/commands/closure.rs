use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::step0::OrchestratorState;
use crate::agents::orchestrator::{
    ClosureResult, ClosureStatus, AuditEntry, ArchivedArtifact, RunStatistics,
};

/// Response from execute_closure command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureResponse {
    pub run_id: String,
    pub final_ledger: String,
    pub audit_trail: Vec<AuditEntry>,
    pub archived_artifacts: Vec<ArchivedArtifact>,
    pub statistics: RunStatistics,
    pub final_metrics: std::collections::HashMap<String, f64>,
    /// Final CI value for quick reference
    pub final_ci: f64,
    /// Closure status indicating run outcome
    pub closure_status: ClosureStatus,
    pub success: bool,
    pub completed_at: String,
}

impl From<ClosureResult> for ClosureResponse {
    fn from(result: ClosureResult) -> Self {
        ClosureResponse {
            run_id: result.run_id,
            final_ledger: result.final_ledger,
            audit_trail: result.audit_trail,
            archived_artifacts: result.archived_artifacts,
            statistics: result.statistics,
            final_metrics: result.final_metrics,
            final_ci: result.final_ci,
            closure_status: result.closure_status,
            success: result.success,
            completed_at: result.completed_at,
        }
    }
}

/// Execute Closure: Final ledger, archival, and export
///
/// CRITICAL: This command executes after Step 6 (normal) or Step 6.5 (exceptional).
/// State must be RunState::Completed.
#[tauri::command]
pub async fn execute_closure(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<ClosureResponse, String> {
    info!("=== EXECUTE_CLOSURE command called ===");
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
                log::error!("[EXECUTE_CLOSURE] {}", err);
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);

        orch
    }; // Lock is released here

    // Execute Closure workflow (now without holding the lock)
    info!("Executing Closure workflow...");
    let closure_result = orchestrator
        .execute_closure()
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to execute Closure: {}", e);
            log::error!("[EXECUTE_CLOSURE] {}", err_msg);
            err_msg
        })?;

    info!("✓ Closure execution complete");
    info!("Success: {}", closure_result.success);
    info!("Artifacts archived: {}", closure_result.archived_artifacts.len());

    // Return orchestrator to state
    {
        let mut orch_guard = state.0.lock().unwrap();
        *orch_guard = Some(orchestrator);
    }

    info!("Orchestrator returned to state");

    // Convert to response
    let response = ClosureResponse::from(closure_result);

    info!("=== EXECUTE_CLOSURE complete ===");

    Ok(response)
}

/// Export run artifacts as Markdown report
#[tauri::command]
pub async fn export_markdown(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<String, String> {
    info!("=== EXPORT_MARKDOWN command called ===");
    info!("Run ID: {}", run_id);

    // Generate report data while holding the lock
    let (run_id_str, ledger, steps_completed, exceptional) = {
        let orch_guard = state.0.lock().unwrap();
        let orchestrator = orch_guard.as_ref()
            .ok_or_else(|| "No active run found".to_string())?;

        (
            orchestrator.run_id.clone(),
            orchestrator.generate_steno_ledger(),
            orchestrator.state.step_number(),
            orchestrator.exceptional_flag,
        )
    }; // Lock released here

    // Generate Markdown report
    let mut markdown = String::new();
    markdown.push_str(&format!("# Method-VI Run Report\n\n"));
    markdown.push_str(&format!("**Run ID:** {}\n\n", run_id_str));
    markdown.push_str(&format!("**Completed:** {}\n\n", chrono::Utc::now().to_rfc3339()));
    markdown.push_str(&format!("---\n\n"));
    markdown.push_str(&format!("## Final Ledger\n\n```\n{}\n```\n\n", ledger));
    markdown.push_str(&format!("## Statistics\n\n"));
    markdown.push_str(&format!("- **Steps Completed:** {}\n", steps_completed));
    markdown.push_str(&format!("- **Exceptional Run:** {}\n", exceptional));
    markdown.push_str(&format!("\n---\n\n*Generated by Method-VI*\n"));

    info!("✓ Markdown report generated ({} bytes)", markdown.len());

    Ok(markdown)
}

/// Export run artifacts as JSON bundle
#[tauri::command]
pub async fn export_json(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<String, String> {
    info!("=== EXPORT_JSON command called ===");
    info!("Run ID: {}", run_id);

    // Generate export data while holding the lock
    let (run_id_str, state_str, exceptional, ledger) = {
        let orch_guard = state.0.lock().unwrap();
        let orchestrator = orch_guard.as_ref()
            .ok_or_else(|| "No active run found".to_string())?;

        (
            orchestrator.run_id.clone(),
            format!("{:?}", orchestrator.state),
            orchestrator.exceptional_flag,
            orchestrator.generate_steno_ledger(),
        )
    }; // Lock released here

    // Create JSON export bundle
    let export = serde_json::json!({
        "run_id": run_id_str,
        "completed_at": chrono::Utc::now().to_rfc3339(),
        "state": state_str,
        "exceptional_flag": exceptional,
        "final_ledger": ledger,
    });

    let json_string = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    info!("✓ JSON export generated ({} bytes)", json_string.len());

    Ok(json_string)
}
