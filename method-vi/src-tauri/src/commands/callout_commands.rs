use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::agents::orchestrator::ArtifactSummary;
use crate::governance::{Callout, CalloutSummary, AcknowledgmentRecord};
use crate::commands::step0::OrchestratorState;

// Re-export types for external use
pub use crate::agents::orchestrator::ArtifactSummary as GateArtifactSummary;

/// Gate preview data - shows what was created and what's missing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatePreview {
    pub step: i32,
    pub artifacts_created: Vec<ArtifactSummary>,
    pub missing_required: Vec<String>,
    pub has_hard_blocks: bool,
}

/// Gate decision types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateDecision {
    #[serde(rename = "approve")]
    Approve,
    #[serde(rename = "request_changes")]
    RequestChanges,
    #[serde(rename = "start_over")]
    StartOver,
}

/// Get all callouts for the current run
#[tauri::command]
pub fn get_all_callouts(state: State<OrchestratorState>) -> Result<Vec<Callout>, String> {
    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_ref()
        .ok_or_else(|| "No active run".to_string())?;
    Ok(orchestrator.callout_manager.all().to_vec())
}

/// Get callouts requiring acknowledgment
#[tauri::command]
pub fn get_pending_callouts(state: State<OrchestratorState>) -> Result<Vec<Callout>, String> {
    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_ref()
        .ok_or_else(|| "No active run".to_string())?;
    Ok(orchestrator.callout_manager.get_pending_acknowledgments().into_iter().cloned().collect())
}

/// Get callout summary (counts by tier, can_proceed status)
#[tauri::command]
pub fn get_callout_summary(state: State<OrchestratorState>) -> Result<CalloutSummary, String> {
    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;

    // If no active run, return empty summary (graceful degradation)
    if let Some(orchestrator) = orch_lock.as_ref() {
        Ok(orchestrator.callout_manager.summary())
    } else {
        Ok(CalloutSummary::default())
    }
}

/// Check if we can proceed (no unacknowledged Critical callouts)
#[tauri::command]
pub fn can_proceed(state: State<OrchestratorState>) -> Result<bool, String> {
    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;

    // If no active run, allow proceeding (no callouts to block)
    if let Some(orchestrator) = orch_lock.as_ref() {
        Ok(orchestrator.callout_manager.can_proceed())
    } else {
        Ok(true)
    }
}

/// Acknowledge a specific callout
#[tauri::command]
pub fn acknowledge_callout(
    state: State<OrchestratorState>,
    callout_id: String,
    confirmation: String,
) -> Result<AcknowledgmentRecord, String> {
    let mut orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_mut()
        .ok_or_else(|| "No active run".to_string())?;
    orchestrator.callout_manager.acknowledge_with_confirmation(&callout_id, confirmation)
    // TODO: Log to ledger in Session 1.4 integration
}

/// Acknowledge all pending Critical callouts
#[tauri::command]
pub fn acknowledge_all_callouts(
    state: State<OrchestratorState>,
    confirmation: String,
) -> Result<Vec<AcknowledgmentRecord>, String> {
    let mut orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_mut()
        .ok_or_else(|| "No active run".to_string())?;
    Ok(orchestrator.callout_manager.acknowledge_all_pending(confirmation))
    // TODO: Log to ledger in Session 1.4 integration
}

/// Get gate preview - shows artifacts created and missing deliverables
#[tauri::command]
pub fn get_gate_preview(step: i32, state: State<OrchestratorState>) -> Result<GatePreview, String> {
    info!("=== GET_GATE_PREVIEW called for step {} ===", step);

    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_ref()
        .ok_or_else(|| "No active run".to_string())?;

    // Get artifacts created
    let artifacts = orchestrator.get_artifacts_summary();
    info!("Artifacts found: {}", artifacts.len());

    // Check for missing required deliverables
    let missing = match orchestrator.check_required_deliverables() {
        Ok(()) => vec![],
        Err(missing_list) => missing_list,
    };
    info!("Missing deliverables: {:?}", missing);

    // Check hard blocks
    let has_hard_blocks = orchestrator.callout_manager.has_hard_blocks();
    info!("Has hard blocks: {}", has_hard_blocks);

    Ok(GatePreview {
        step,
        artifacts_created: artifacts,
        missing_required: missing,
        has_hard_blocks,
    })
}

/// Get all hard-block callouts (cannot be cleared by acknowledgment)
#[tauri::command]
pub fn get_hard_blocks(state: State<OrchestratorState>) -> Result<Vec<Callout>, String> {
    let orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_ref()
        .ok_or_else(|| "No active run".to_string())?;
    Ok(orchestrator.callout_manager.get_hard_blocks().into_iter().cloned().collect())
}

/// Submit a gate decision (approve, request_changes, or start_over)
#[tauri::command]
pub fn submit_gate_decision(
    decision: GateDecision,
    feedback: Option<String>,
    state: State<OrchestratorState>,
) -> Result<(), String> {
    info!("=== SUBMIT_GATE_DECISION called ===");
    info!("Decision: {:?}", decision);
    if let Some(ref fb) = feedback {
        info!("Feedback: {}", fb);
    }

    let mut orch_lock = state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = orch_lock.as_mut()
        .ok_or_else(|| "No active run".to_string())?;

    match decision {
        GateDecision::Approve => {
            // Check if we can actually approve (no hard blocks)
            if orchestrator.callout_manager.has_hard_blocks() {
                return Err("Cannot approve: hard-block callouts exist".to_string());
            }
            info!("Gate decision: APPROVE");
            // The actual gate approval is handled by approve_gate command
            Ok(())
        }
        GateDecision::RequestChanges => {
            info!("Gate decision: REQUEST_CHANGES");
            // Log feedback for future implementation
            if let Some(fb) = feedback {
                info!("User feedback for changes: {}", fb);
                // TODO: Store feedback in orchestrator for re-processing
            }
            Ok(())
        }
        GateDecision::StartOver => {
            info!("Gate decision: START_OVER");
            // TODO: Implement run reset logic
            Ok(())
        }
    }
}
