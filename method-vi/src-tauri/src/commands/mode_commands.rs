use tauri::State;
use serde::{Deserialize, Serialize};
use crate::governance::{ModeDetectionResult, ModeDetector, UserPosture};
use crate::commands::step0::OrchestratorState;

/// Response for get_current_mode command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeInfo {
    pub mode: Option<String>,
    pub display_name: Option<String>,
    pub user_message: Option<String>,
    pub ci_baseline: Option<f64>,
    pub confidence: Option<f64>,
    pub is_locked: bool,
}

/// Get current detected mode for the run
#[tauri::command]
pub fn get_current_mode(state: State<OrchestratorState>) -> Result<ModeInfo, String> {
    let orchestrator_lock = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(orchestrator) = orchestrator_lock.as_ref() {
        if let Some(mode_result) = &orchestrator.mode_detection_result {
            return Ok(ModeInfo {
                mode: Some(format!("{:?}", mode_result.mode)),
                display_name: Some(mode_result.mode.display_name().to_string()),
                user_message: Some(mode_result.mode.user_message().to_string()),
                ci_baseline: Some(mode_result.ci_baseline),
                confidence: Some(mode_result.confidence),
                is_locked: orchestrator.mode_locked,
            });
        }
    }

    // No mode detected yet
    Ok(ModeInfo {
        mode: None,
        display_name: None,
        user_message: None,
        ci_baseline: None,
        confidence: None,
        is_locked: false,
    })
}

/// Manually trigger mode detection (for testing)
/// Uses legacy detection without posture (backward compatibility)
#[tauri::command]
pub fn detect_mode(ci_baseline: f64) -> ModeDetectionResult {
    ModeDetector::detect_legacy(ci_baseline)
}

/// Set user's posture selection (Build/Audit) for the current run
///
/// Should be called during Step 0 to inform mode detection.
/// Combined with CI baseline to determine Transformation mode eligibility.
#[tauri::command]
pub fn set_user_posture(
    posture: String,
    state: State<OrchestratorState>,
) -> Result<String, String> {
    let posture_enum = match posture.as_str() {
        "Build" => UserPosture::Build,
        "Audit" => UserPosture::Audit,
        _ => return Err(format!("Invalid posture: {}. Expected 'Build' or 'Audit'", posture)),
    };

    let mut orchestrator_lock = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(orchestrator) = orchestrator_lock.as_mut() {
        orchestrator.set_user_posture(posture_enum);
        Ok(format!("Posture set to {:?}", posture_enum))
    } else {
        Err("No active run. Start a run before setting posture.".to_string())
    }
}

/// Get current user posture for the run
#[tauri::command]
pub fn get_user_posture(state: State<OrchestratorState>) -> Result<String, String> {
    let orchestrator_lock = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(orchestrator) = orchestrator_lock.as_ref() {
        Ok(format!("{:?}", orchestrator.user_posture))
    } else {
        Ok("Unconfirmed".to_string())
    }
}
