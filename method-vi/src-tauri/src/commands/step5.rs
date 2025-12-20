use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::step0::OrchestratorState;

/// Response from execute_step_5 command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step5Response {
    pub framework_architecture_id: String,
    pub framework_architecture: String,
    pub metrics: Option<MetricsSnapshot>,
}

/// Metrics snapshot for Step 5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub ci: Option<f64>,
    pub ev: Option<f64>,
    pub ias: Option<f64>,
    pub efi: Option<f64>,
    pub sec: Option<f64>,
    pub pci: Option<f64>,
}

/// Execute Step 5: Structure & Redesign
///
/// This command:
/// 1. Reuses Structure & Redesign Agent (attached in Step 1)
/// 2. Uses Core Thesis from Step 4 (immutable)
/// 3. Creates framework architecture with section map and transitions
/// 4. Performs governance coherence audit
/// 5. Calculates metrics
/// 6. Emits Ready_for_Validation signal (GATE)
/// 7. Presents gate to user for approval
#[tauri::command]
pub async fn execute_step_5(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<Step5Response, String> {
    info!("=== EXECUTE_STEP_5 command called ===");
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
                let err = "No active run found. Please complete Steps 0-4 first.".to_string();
                log::error!("[EXECUTE_STEP_5] {}", err);
                log::error!("[EXECUTE_STEP_5] State was empty when trying to take orchestrator");
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);
        info!("State contains orchestrator after take: {}", orch_guard.is_some());

        orch
    }; // Lock is released here

    // Execute Step 5 (now without holding the lock)
    info!("Executing Step 5 workflow...");
    let framework_architecture_id = orchestrator
        .execute_step_5()
        .await
        .map_err(|e| {
            let err = format!("Failed to execute Step 5: {}", e);
            log::error!("[EXECUTE_STEP_5] {}", err);
            err
        })?;

    info!("Step 5 completed successfully");

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
        .ok_or_else(|| "Orchestrator not found after Step 5 execution".to_string())?;

    // Extract artifacts
    let framework_architecture = orchestrator
        .framework_architecture
        .as_ref()
        .ok_or_else(|| "Framework Architecture not generated".to_string())?
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

    info!("Step 5 complete - framework architecture and metrics ready");

    Ok(Step5Response {
        framework_architecture_id,
        framework_architecture,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step5_command_structure() {
        // This test validates the command structure
        // Actual execution test would require a valid API key
        let response = Step5Response {
            framework_architecture_id: "test-framework-architecture".to_string(),
            framework_architecture: "test framework content".to_string(),
            metrics: Some(MetricsSnapshot {
                ci: Some(0.85),
                ev: Some(5.0),
                ias: Some(0.90),
                efi: Some(95.0),
                sec: Some(100.0),
                pci: Some(0.92),
            }),
        };

        assert_eq!(response.framework_architecture_id, "test-framework-architecture");
        assert!(response.metrics.is_some());
    }
}
