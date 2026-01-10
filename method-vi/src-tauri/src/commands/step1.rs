use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::agents::{GovernanceTelemetryAgent, StructureRedesignAgent};
use crate::commands::step0::OrchestratorState;
use crate::config::AppConfig;

/// Response structure for Step 1 that matches the frontend expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step1Response {
    pub intent_anchor: Step1Artifact,
    pub charter: Step1Artifact,
    pub baseline_report: Step1Artifact,
    pub architecture_map: Step1Artifact,
    pub e_baseline: f64,
}

/// Artifact information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step1Artifact {
    pub artifact_id: String,
    pub artifact_type: String,
    pub hash: String,
    pub is_immutable: bool,
    pub content_preview: String,
}

/// Extract artifact metadata from the full artifact markdown
fn extract_artifact_info(artifact: &str, artifact_type: &str) -> Step1Artifact {
    let mut artifact_id = String::new();
    let mut hash = String::new();

    // Extract from frontmatter
    for line in artifact.lines() {
        if line.starts_with("artifact_id:") {
            artifact_id = line
                .trim_start_matches("artifact_id:")
                .trim()
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("hash:") {
            hash = line
                .trim_start_matches("hash:")
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }

    // Extract first 200 characters of content as preview
    let content_preview = if let Some(start) = artifact.find("\n\n") {
        let content = &artifact[start + 2..];
        content.chars().take(200).collect::<String>()
    } else {
        artifact.chars().take(200).collect::<String>()
    };

    Step1Artifact {
        artifact_id,
        artifact_type: artifact_type.to_string(),
        hash,
        is_immutable: true,
        content_preview,
    }
}

/// Execute Step 1: Baseline Establishment
///
/// This command creates the 4 immutable baseline artifacts:
/// - Intent_Anchor
/// - Charter
/// - Baseline_Report
/// - Architecture_Map
#[tauri::command]
pub async fn execute_step_1(
    run_id: String,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step1Response, String> {
    info!("=== EXECUTE_STEP_1 command called ===");
    info!("Run ID: {}", run_id);
    info!("Timestamp: {}", chrono::Utc::now().to_rfc3339());

    // Get API key from config
    let api_key = {
        let config = config_state.lock().unwrap();
        config
            .get_api_key()
            .map_err(|e| format!("API key not configured: {}", e))?
    };
    info!("API key retrieved: {}...", &api_key[..15]);

    // Create agents
    info!("Creating agents...");
    let governance_agent = GovernanceTelemetryAgent::new(api_key.clone())
        .map_err(|e| format!("Failed to create Governance Agent: {}", e))?;
    info!("Governance agent created");

    let structure_agent = StructureRedesignAgent::new(api_key.clone())
        .map_err(|e| format!("Failed to create Structure Agent: {}", e))?;
    info!("Structure agent created");

    // Get the orchestrator from state and add agents
    info!("Acquiring state lock...");
    let mut orchestrator = {
        let mut orch_guard = state.0.lock().unwrap();
        info!("State lock acquired");
        info!("State contains orchestrator: {}", orch_guard.is_some());

        // Take ownership of the orchestrator temporarily
        let mut orch = orch_guard
            .take()
            .ok_or_else(|| {
                let err = "No active run found. Please complete Step 0 first.".to_string();
                log::error!("[EXECUTE_STEP_1] {}", err);
                log::error!("[EXECUTE_STEP_1] State was empty when trying to take orchestrator");
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);
        info!("State contains orchestrator after take: {}", orch_guard.is_some());

        // Add agents if not already present
        info!("Adding governance and structure agents...");
        orch = orch
            .with_governance_agent(governance_agent)
            .with_structure_agent(structure_agent);

        info!("Agents added to orchestrator");
        orch
    }; // Lock is released here

    // Execute Step 1 (now without holding the lock)
    info!("Executing Step 1 workflow...");

    // Execute and ensure orchestrator is always put back, even on error
    let step1_result = orchestrator.execute_step_1().await;

    // Always put orchestrator back into state, even if there was an error
    info!("Putting orchestrator back into state...");
    {
        let mut orch_guard = state.0.lock().unwrap();
        *orch_guard = Some(orchestrator);
        info!("Orchestrator restored to state");
    }

    // Now check if step1 succeeded
    let (_intent_anchor_id, _charter_id, _baseline_id, _architecture_id) = step1_result
        .map_err(|e| {
            let err = format!("Failed to execute Step 1: {}", e);
            log::error!("[EXECUTE_STEP_1] {}", err);
            err
        })?;

    info!("Step 1 completed successfully");

    // Get orchestrator back to extract artifact info
    let orch_guard = state.0.lock().unwrap();
    let orchestrator = orch_guard.as_ref()
        .ok_or_else(|| "Orchestrator not found after Step 1 execution".to_string())?;

    // Extract artifact information for frontend
    let intent_anchor = orchestrator
        .intent_anchor
        .as_ref()
        .ok_or_else(|| "Intent_Anchor not created".to_string())?;
    let charter = orchestrator
        .charter
        .as_ref()
        .ok_or_else(|| "Charter not created".to_string())?;
    let baseline_report = orchestrator
        .baseline_report
        .as_ref()
        .ok_or_else(|| "Baseline_Report not created".to_string())?;
    let architecture_map = orchestrator
        .architecture_map
        .as_ref()
        .ok_or_else(|| "Architecture_Map not created".to_string())?;

    let e_baseline = orchestrator
        .get_e_baseline()
        .ok_or_else(|| "E_baseline not set".to_string())?;

    // Build response
    let response = Step1Response {
        intent_anchor: extract_artifact_info(intent_anchor, "Intent_Anchor"),
        charter: extract_artifact_info(&charter.to_display_markdown(), "Charter"),
        baseline_report: extract_artifact_info(baseline_report, "Baseline_Report"),
        architecture_map: extract_artifact_info(architecture_map, "Architecture_Map"),
        e_baseline,
    };

    Ok(response)
}
