use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::step0::OrchestratorState;

/// Response from execute_step_2 command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step2Response {
    pub governance_summary_id: String,
    pub domain_snapshots_id: String,
    pub governance_summary: String,
    pub domain_snapshots: String,
    pub metrics: Option<MetricsSnapshot>,
}

/// Metrics snapshot for Step 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub ci: Option<f64>,
    pub ev: Option<f64>,
    pub ias: Option<f64>,
    pub efi: Option<f64>,
    pub sec: Option<f64>,
    pub pci: Option<f64>,
}

/// Execute Step 2: Governance Calibration
///
/// This command:
/// 1. Configures five control domains (Entropy, Objective, Process, Reflective, Termination)
/// 2. Creates Governance_Summary and Domain_Snapshots artifacts
/// 3. Calculates initial metrics from baseline
/// 4. Emits Ready_for_Analysis signal (GATE)
/// 5. Presents gate to user for approval
#[tauri::command]
pub async fn execute_step_2(
    run_id: String,
    state: State<'_, OrchestratorState>,
) -> Result<Step2Response, String> {
    info!("=== EXECUTE_STEP_2 command called ===");
    info!("Run ID: {}", run_id);
    info!("Timestamp: {}", chrono::Utc::now().to_rfc3339());

    // Get the orchestrator from state (governance agent should already be attached from Step 1)
    info!("Acquiring state lock...");
    let mut orchestrator = {
        let mut orch_guard = state.0.lock().unwrap();
        info!("State lock acquired");
        info!("State contains orchestrator: {}", orch_guard.is_some());

        // Take ownership of the orchestrator temporarily
        let orch = orch_guard
            .take()
            .ok_or_else(|| {
                let err = "No active run found. Please complete Steps 0 and 1 first.".to_string();
                log::error!("[EXECUTE_STEP_2] {}", err);
                log::error!("[EXECUTE_STEP_2] State was empty when trying to take orchestrator");
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);
        info!("State contains orchestrator after take: {}", orch_guard.is_some());

        orch
    }; // Lock is released here

    // Execute Step 2 (now without holding the lock)
    info!("Executing Step 2 workflow...");
    let (governance_summary_id, domain_snapshots_id) = orchestrator
        .execute_step_2()
        .await
        .map_err(|e| {
            let err = format!("Failed to execute Step 2: {}", e);
            log::error!("[EXECUTE_STEP_2] {}", err);
            err
        })?;

    info!("Step 2 completed successfully");

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
        .ok_or_else(|| "Orchestrator not found after Step 2 execution".to_string())?;

    // Extract artifacts
    let governance_summary = orchestrator
        .governance_summary
        .as_ref()
        .ok_or_else(|| "Governance Summary not generated".to_string())?
        .clone();

    let domain_snapshots = orchestrator
        .domain_snapshots
        .as_ref()
        .ok_or_else(|| "Domain Snapshots not generated".to_string())?
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

    info!("Step 2 complete - artifacts and metrics ready");

    Ok(Step2Response {
        governance_summary_id,
        domain_snapshots_id,
        governance_summary,
        domain_snapshots,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step2_command_structure() {
        // This test validates the command structure
        // Actual execution test would require a valid API key
        let response = Step2Response {
            governance_summary_id: "test-governance-summary".to_string(),
            domain_snapshots_id: "test-domain-snapshots".to_string(),
            governance_summary: "test content".to_string(),
            domain_snapshots: "test content".to_string(),
            metrics: Some(MetricsSnapshot {
                ci: Some(0.85),
                ev: Some(5.0),
                ias: Some(0.90),
                efi: Some(95.0),
                sec: Some(100.0),
                pci: Some(0.92),
            }),
        };

        assert_eq!(response.governance_summary_id, "test-governance-summary");
        assert!(response.metrics.is_some());
    }
}
