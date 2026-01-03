use tauri::State;
use std::sync::Mutex;

use crate::governance::{Callout, CalloutManager, CalloutSummary, AcknowledgmentRecord};

/// State wrapper for CalloutManager
pub struct CalloutState(pub Mutex<CalloutManager>);

/// Get all callouts for the current run
#[tauri::command]
pub fn get_all_callouts(state: State<CalloutState>) -> Result<Vec<Callout>, String> {
    let manager = state.0.lock().map_err(|e| e.to_string())?;
    Ok(manager.all().to_vec())
}

/// Get callouts requiring acknowledgment
#[tauri::command]
pub fn get_pending_callouts(state: State<CalloutState>) -> Result<Vec<Callout>, String> {
    let manager = state.0.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_pending_acknowledgments().into_iter().cloned().collect())
}

/// Get callout summary (counts by tier, can_proceed status)
#[tauri::command]
pub fn get_callout_summary(state: State<CalloutState>) -> Result<CalloutSummary, String> {
    let manager = state.0.lock().map_err(|e| e.to_string())?;
    Ok(manager.summary())
}

/// Check if we can proceed (no unacknowledged Critical callouts)
#[tauri::command]
pub fn can_proceed(state: State<CalloutState>) -> Result<bool, String> {
    let manager = state.0.lock().map_err(|e| e.to_string())?;
    Ok(manager.can_proceed())
}

/// Acknowledge a specific callout
#[tauri::command]
pub fn acknowledge_callout(
    state: State<CalloutState>,
    callout_id: String,
    confirmation: String,
) -> Result<AcknowledgmentRecord, String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    manager.acknowledge_with_confirmation(&callout_id, confirmation)
    // TODO: Log to ledger in Session 1.4 integration
}

/// Acknowledge all pending Critical callouts
#[tauri::command]
pub fn acknowledge_all_callouts(
    state: State<CalloutState>,
    confirmation: String,
) -> Result<Vec<AcknowledgmentRecord>, String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    Ok(manager.acknowledge_all_pending(confirmation))
    // TODO: Log to ledger in Session 1.4 integration
}
