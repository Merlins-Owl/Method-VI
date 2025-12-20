use log::info;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::commands::step0::OrchestratorState;
use crate::config::AppConfig;
use crate::agents::AnalysisSynthesisAgent;

/// Response from execute_step_3 command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step3Response {
    pub integrated_diagnostic_id: String,
    pub lens_efficacy_report_id: String,
    pub integrated_diagnostic: String,
    pub lens_efficacy_report: String,
    pub metrics: Option<MetricsSnapshot>,
}

/// Metrics snapshot for Step 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub ci: Option<f64>,
    pub ev: Option<f64>,
    pub ias: Option<f64>,
    pub efi: Option<f64>,
    pub sec: Option<f64>,
    pub pci: Option<f64>,
}

/// Execute Step 3: Multi-Angle Analysis
///
/// This command:
/// 1. CREATES and ATTACHES the Analysis & Synthesis Agent (OBSERVER role)
/// 2. Applies six analytical lenses (Structural, Thematic, Logic, Evidence, Expression, Intent)
/// 3. Uses weighted lens sequencing based on intent category
/// 4. Creates Integrated_Diagnostic and Lens_Efficacy_Report artifacts
/// 5. Calculates metrics
/// 6. Emits Ready_for_Synthesis signal (GATE)
/// 7. Presents gate to user for approval
///
/// CRITICAL: The Analysis agent created here is STATEFUL and will be REUSED in Step 4.
#[tauri::command]
pub async fn execute_step_3(
    run_id: String,
    state: State<'_, OrchestratorState>,
    config_state: State<'_, Mutex<AppConfig>>,
) -> Result<Step3Response, String> {
    info!("=== EXECUTE_STEP_3 command called ===");
    info!("Run ID: {}", run_id);
    info!("Timestamp: {}", chrono::Utc::now().to_rfc3339());

    // Get API key from config (checks env var first, then config file)
    info!("Retrieving API key from config...");
    let api_key = {
        let config = config_state.lock().unwrap();
        config
            .get_api_key()
            .map_err(|e| {
                let err = format!("API key not configured: {}. Please set it in Settings or via ANTHROPIC_API_KEY environment variable.", e);
                log::error!("[EXECUTE_STEP_3] {}", err);
                err
            })?
    };
    info!("API key retrieved successfully");

    // Get the orchestrator from state
    info!("Acquiring state lock...");
    let mut orchestrator = {
        let mut orch_guard = state.0.lock().unwrap();
        info!("State lock acquired");
        info!("State contains orchestrator: {}", orch_guard.is_some());

        // Take ownership of the orchestrator temporarily
        let mut orch = orch_guard
            .take()
            .ok_or_else(|| {
                let err = "No active run found. Please complete Steps 0, 1, and 2 first.".to_string();
                log::error!("[EXECUTE_STEP_3] {}", err);
                log::error!("[EXECUTE_STEP_3] State was empty when trying to take orchestrator");
                err
            })?;

        info!("Orchestrator taken from state");
        info!("Orchestrator run_id: {}", orch.run_id);
        info!("Orchestrator state: {:?}", orch.state);
        info!("State contains orchestrator after take: {}", orch_guard.is_some());

        // CREATE and ATTACH Analysis & Synthesis Agent (OBSERVER role)
        info!("Creating Analysis & Synthesis Agent...");
        let analysis_agent = AnalysisSynthesisAgent::new(api_key)
            .map_err(|e| {
                let err = format!("Failed to create Analysis & Synthesis Agent: {}", e);
                log::error!("[EXECUTE_STEP_3] {}", err);
                err
            })?;
        info!("Analysis & Synthesis Agent created successfully");

        info!("Attaching Analysis & Synthesis Agent to Orchestrator...");
        orch = orch.with_analysis_synthesis_agent(analysis_agent);
        info!("Analysis & Synthesis Agent attached - will be REUSED in Step 4");

        orch
    }; // Lock is released here

    // Execute Step 3 (now without holding the lock)
    info!("Executing Step 3 workflow...");
    let (integrated_diagnostic_id, lens_efficacy_report_id) = orchestrator
        .execute_step_3()
        .await
        .map_err(|e| {
            let err = format!("Failed to execute Step 3: {}", e);
            log::error!("[EXECUTE_STEP_3] {}", err);
            err
        })?;

    info!("Step 3 completed successfully");

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
        .ok_or_else(|| "Orchestrator not found after Step 3 execution".to_string())?;

    // Extract artifacts
    let integrated_diagnostic = orchestrator
        .integrated_diagnostic
        .as_ref()
        .ok_or_else(|| "Integrated Diagnostic not generated".to_string())?
        .clone();

    let lens_efficacy_report = orchestrator
        .lens_efficacy_report
        .as_ref()
        .ok_or_else(|| "Lens Efficacy Report not generated".to_string())?
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

    info!("Step 3 complete - artifacts and metrics ready");

    Ok(Step3Response {
        integrated_diagnostic_id,
        lens_efficacy_report_id,
        integrated_diagnostic,
        lens_efficacy_report,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step3_command_structure() {
        // This test validates the command structure
        // Actual execution test would require a valid API key
        let response = Step3Response {
            integrated_diagnostic_id: "test-integrated-diagnostic".to_string(),
            lens_efficacy_report_id: "test-lens-efficacy-report".to_string(),
            integrated_diagnostic: "test content".to_string(),
            lens_efficacy_report: "test content".to_string(),
            metrics: Some(MetricsSnapshot {
                ci: Some(0.85),
                ev: Some(5.0),
                ias: Some(0.90),
                efi: Some(95.0),
                sec: Some(100.0),
                pci: Some(0.92),
            }),
        };

        assert_eq!(response.integrated_diagnostic_id, "test-integrated-diagnostic");
        assert!(response.metrics.is_some());
    }
}
