use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::step0::OrchestratorState;

/// Response from execute_step_4 command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step4Response {
    pub core_thesis_id: String,
    pub north_star_narrative_id: String,
    pub core_thesis: String,
    pub operating_principles: String,
    pub model_geometry: String,
    pub causal_spine: String,
    pub north_star_narrative: String,
    pub glossary: String,
    pub limitations: String,
    pub metrics: Option<MetricsSnapshot>,
}

/// Metrics snapshot for Step 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub ci: Option<f64>,
    pub ev: Option<f64>,
    pub ias: Option<f64>,
    pub efi: Option<f64>,
    pub sec: Option<f64>,
    pub pci: Option<f64>,
}

/// Execute Step 4: Synthesis Lock-In
///
/// This command:
/// 1. Derives core thesis from integrated diagnostic
/// 2. Extracts operating principles
/// 3. Selects model geometry (Linear/Cyclic/Branching)
/// 4. Maps causality (Causal Spine)
/// 5. Authors North-Star narrative
/// 6. Creates glossary
/// 7. Documents limitations
/// 8. Calculates metrics
/// 9. Emits Ready_for_Redesign signal (GATE)
/// 10. Presents gate to user for approval
#[tauri::command]
pub async fn execute_step_4(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<Step4Response, String> {
    info!("=== EXECUTE_STEP_4 command called ===");
    info!("Run ID: {}", run_id);
    info!("Timestamp: {}", chrono::Utc::now().to_rfc3339());

    // Get the orchestrator from state
    info!("Acquiring state lock...");
    let mut orchestrator = {
        let mut orch_guard = state.0.lock().unwrap();
        info!("State lock acquired");
        info!("State contains orchestrator: {}", orch_guard.is_some());

        // Take ownership of the orchestrator temporarily
        let orch = orch_guard
            .take()
            .ok_or_else(|| {
                let err = "No active run found. Please complete Steps 0, 1, 2, and 3 first.".to_string();
                log::error!("[EXECUTE_STEP_4] {}", err);
                log::error!("[EXECUTE_STEP_4] State was empty when trying to take orchestrator");
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);
        info!("State contains orchestrator after take: {}", orch_guard.is_some());

        orch
    }; // Lock is released here

    // Execute Step 4 (now without holding the lock)
    info!("Executing Step 4 workflow...");
    let (core_thesis_id, north_star_narrative_id) = orchestrator
        .execute_step_4()
        .await
        .map_err(|e| {
            let err = format!("Failed to execute Step 4: {}", e);
            log::error!("[EXECUTE_STEP_4] {}", err);
            err
        })?;

    info!("Step 4 completed successfully");

    // Put orchestrator back into state
    info!("Putting orchestrator back into state...");
    {
        let mut orch_guard = state.0.lock().unwrap();
        *orch_guard = Some(orchestrator);
        info!("Orchestrator restored to state");
    }

    // Get orchestrator back to extract artifact info
    let orch_guard = state.0.lock().unwrap();
    let orchestrator = orch_guard.as_ref()
        .ok_or_else(|| "Orchestrator not found after Step 4 execution".to_string())?;

    // Extract artifacts
    let core_thesis = orchestrator
        .core_thesis
        .as_ref()
        .ok_or_else(|| "Core Thesis not generated".to_string())?
        .clone();

    let operating_principles = orchestrator
        .operating_principles
        .as_ref()
        .ok_or_else(|| "Operating Principles not generated".to_string())?
        .clone();

    let model_geometry = orchestrator
        .model_geometry
        .as_ref()
        .ok_or_else(|| "Model Geometry not generated".to_string())?
        .clone();

    let causal_spine = orchestrator
        .causal_spine
        .as_ref()
        .ok_or_else(|| "Causal Spine not generated".to_string())?
        .clone();

    let north_star_narrative = orchestrator
        .north_star_narrative
        .as_ref()
        .ok_or_else(|| "North-Star Narrative not generated".to_string())?
        .clone();

    let glossary = orchestrator
        .glossary
        .as_ref()
        .ok_or_else(|| "Glossary not generated".to_string())?
        .clone();

    let limitations = orchestrator
        .limitations
        .as_ref()
        .ok_or_else(|| "Limitations not documented".to_string())?
        .clone();

    // Extract metrics snapshot
    let metrics = orchestrator.latest_metrics.as_ref().map(|m| {
        MetricsSnapshot {
            ci: m.ci.as_ref().map(|metric| metric.value),
            ev: m.ev.as_ref().map(|metric| metric.value),
            ias: m.ias.as_ref().map(|metric| metric.value),
            efi: m.efi.as_ref().map(|metric| metric.value),
            sec: m.sec.as_ref().map(|metric| metric.value),
            pci: m.pci.as_ref().map(|metric| metric.value),
        }
    });

    info!("Step 4 complete - all synthesis artifacts and metrics ready");

    Ok(Step4Response {
        core_thesis_id,
        north_star_narrative_id,
        core_thesis,
        operating_principles,
        model_geometry,
        causal_spine,
        north_star_narrative,
        glossary,
        limitations,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step4_command_structure() {
        // This test validates the command structure
        // Actual execution test would require a valid API key
        let response = Step4Response {
            core_thesis_id: "test-core-thesis".to_string(),
            north_star_narrative_id: "test-north-star-narrative".to_string(),
            core_thesis: "test thesis content".to_string(),
            operating_principles: "test principles".to_string(),
            model_geometry: "Linear: Sequential flow".to_string(),
            causal_spine: "test causal spine".to_string(),
            north_star_narrative: "test narrative".to_string(),
            glossary: "test glossary".to_string(),
            limitations: "test limitations".to_string(),
            metrics: Some(MetricsSnapshot {
                ci: Some(0.85),
                ev: Some(5.0),
                ias: Some(0.90),
                efi: Some(95.0),
                sec: Some(100.0),
                pci: Some(0.92),
            }),
        };

        assert_eq!(response.core_thesis_id, "test-core-thesis");
        assert!(response.metrics.is_some());
    }
}
