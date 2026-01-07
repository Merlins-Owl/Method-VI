use tauri::State;

use crate::governance::{Callout, CalloutSummary, AcknowledgmentRecord};
use crate::commands::step0::OrchestratorState;

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
