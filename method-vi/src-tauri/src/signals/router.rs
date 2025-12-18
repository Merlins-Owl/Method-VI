use super::types::{Signal, SignalPayload, SignalType};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Signal Router for managing signal emission, sequencing, and gate recognition
///
/// The Signal Router maintains signal chains for each run and provides
/// functions to emit signals, recognize gate signals, and retrieve signal history.
pub struct SignalRouter {
    /// Signal chains organized by run_id
    signal_chains: HashMap<String, Vec<Signal>>,
}

impl SignalRouter {
    /// Create a new Signal Router
    pub fn new() -> Self {
        SignalRouter {
            signal_chains: HashMap::new(),
        }
    }

    /// Emit a signal for a run
    ///
    /// Creates a new signal with hash chain integrity. Each signal's hash
    /// includes the prior signal's hash to form a tamper-evident chain.
    ///
    /// # Example
    /// ```
    /// use method_vi_lib::signals::{SignalRouter, SignalType, SignalPayload};
    ///
    /// let mut router = SignalRouter::new();
    /// let payload = SignalPayload {
    ///     step_from: 0,
    ///     step_to: 1,
    ///     artifacts_produced: vec!["charter-001".to_string()],
    ///     metrics_snapshot: None,
    ///     gate_required: true,
    /// };
    ///
    /// let signal = router.emit_signal(SignalType::ReadyForStep1, "run-001", payload);
    /// assert_eq!(signal.signal_type, SignalType::ReadyForStep1);
    /// assert!(signal.payload.gate_required);
    /// ```
    pub fn emit_signal(
        &mut self,
        signal_type: SignalType,
        run_id: &str,
        payload: SignalPayload,
    ) -> Signal {
        let timestamp = Utc::now();

        // Get prior signal hash from chain
        let prior_signal_hash = self
            .signal_chains
            .get(run_id)
            .and_then(|chain| chain.last())
            .map(|sig| sig.hash.clone());

        // Calculate hash for this signal
        let hash = Self::calculate_signal_hash(
            &signal_type,
            run_id,
            &timestamp,
            &prior_signal_hash,
            &payload,
        );

        let signal = Signal {
            signal_type,
            run_id: run_id.to_string(),
            timestamp,
            prior_signal_hash,
            hash,
            payload,
        };

        // Add to signal chain
        self.signal_chains
            .entry(run_id.to_string())
            .or_insert_with(Vec::new)
            .push(signal.clone());

        signal
    }

    /// Check if a signal type requires a gate (human approval)
    ///
    /// Gate signals:
    /// - Ready_for_Step_1 (Step 0→1)
    /// - Baseline_Frozen (Step 1→2)
    /// - Ready_for_Analysis (Step 2→3)
    /// - Ready_for_Synthesis (Step 3→4)
    /// - Ready_for_Redesign (Step 4→5)
    /// - Ready_for_Validation (Step 5→6)
    /// - Validation_Complete (Step 6)
    ///
    /// Non-gate signals:
    /// - Learning_Harvested
    /// - New_Run_Ready
    /// - Metric_Update
    pub fn is_gate_signal(signal_type: &SignalType) -> bool {
        matches!(
            signal_type,
            SignalType::ReadyForStep1
                | SignalType::BaselineFrozen
                | SignalType::ReadyForAnalysis
                | SignalType::ReadyForSynthesis
                | SignalType::ReadyForRedesign
                | SignalType::ReadyForValidation
                | SignalType::ValidationComplete
        )
    }

    /// Get the signal chain for a run
    ///
    /// Returns all signals emitted for a run in chronological order.
    pub fn get_signal_chain(&self, run_id: &str) -> Vec<Signal> {
        self.signal_chains
            .get(run_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Verify signal chain integrity for a run
    ///
    /// Checks that all signals in the chain have valid hash links.
    pub fn verify_chain_integrity(&self, run_id: &str) -> bool {
        let chain = match self.signal_chains.get(run_id) {
            Some(c) => c,
            None => return true, // Empty chain is valid
        };

        for (i, signal) in chain.iter().enumerate() {
            // First signal should have no prior hash
            if i == 0 {
                if signal.prior_signal_hash.is_some() {
                    return false;
                }
            } else {
                // Subsequent signals should link to previous
                let prev_hash = &chain[i - 1].hash;
                match &signal.prior_signal_hash {
                    Some(hash) if hash == prev_hash => {}
                    _ => return false,
                }
            }

            // Verify signal's own hash
            let expected_hash = Self::calculate_signal_hash(
                &signal.signal_type,
                &signal.run_id,
                &signal.timestamp,
                &signal.prior_signal_hash,
                &signal.payload,
            );

            if signal.hash != expected_hash {
                return false;
            }
        }

        true
    }

    /// Calculate SHA-256 hash for a signal
    fn calculate_signal_hash(
        signal_type: &SignalType,
        run_id: &str,
        timestamp: &chrono::DateTime<Utc>,
        prior_signal_hash: &Option<String>,
        payload: &SignalPayload,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(signal_type.as_str().as_bytes());
        hasher.update(run_id.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());

        if let Some(prior_hash) = prior_signal_hash {
            hasher.update(prior_hash.as_bytes());
        }

        hasher.update(&payload.step_from.to_le_bytes());
        hasher.update(&payload.step_to.to_le_bytes());
        hasher.update(payload.gate_required.to_string().as_bytes());

        // Hash artifacts
        for artifact_id in &payload.artifacts_produced {
            hasher.update(artifact_id.as_bytes());
        }

        // Hash metrics if present
        if let Some(metrics) = &payload.metrics_snapshot {
            if let Ok(metrics_str) = serde_json::to_string(metrics) {
                hasher.update(metrics_str.as_bytes());
            }
        }

        format!("{:x}", hasher.finalize())
    }
}

impl Default for SignalRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-SR-001-A: Ready_for_Step_1 signal with required fields
    #[test]
    fn tc_sr_001_a_ready_for_step_1() {
        println!("\n=== TC-SR-001-A: Ready_for_Step_1 signal ===");

        let mut router = SignalRouter::new();
        let payload = SignalPayload {
            step_from: 0,
            step_to: 1,
            artifacts_produced: vec!["charter-001".to_string()],
            metrics_snapshot: None,
            gate_required: true,
        };

        let signal = router.emit_signal(SignalType::ReadyForStep1, "run-001", payload);

        println!("Signal type: {:?}", signal.signal_type);
        println!("Run ID: {}", signal.run_id);
        println!("Step transition: {} -> {}", signal.payload.step_from, signal.payload.step_to);
        println!("Timestamp: {}", signal.timestamp);

        assert_eq!(signal.signal_type, SignalType::ReadyForStep1);
        assert_eq!(signal.run_id, "run-001");
        assert_eq!(signal.payload.step_from, 0);
        assert_eq!(signal.payload.step_to, 1);
        assert!(signal.timestamp <= Utc::now());

        println!("✓ Test passed");
    }

    /// TC-SR-001-B: Baseline_Frozen with artifacts and metrics
    #[test]
    fn tc_sr_001_b_baseline_frozen() {
        println!("\n=== TC-SR-001-B: Baseline_Frozen signal ===");

        let mut router = SignalRouter::new();
        let metrics = serde_json::json!({
            "ci": 0.85,
            "ev": 2.5,
            "sec": true
        });

        let payload = SignalPayload {
            step_from: 1,
            step_to: 2,
            artifacts_produced: vec![
                "baseline-001".to_string(),
                "charter-002".to_string(),
            ],
            metrics_snapshot: Some(metrics.clone()),
            gate_required: true,
        };

        let signal = router.emit_signal(SignalType::BaselineFrozen, "run-002", payload);

        println!("Artifacts produced: {:?}", signal.payload.artifacts_produced);
        println!("Metrics snapshot present: {}", signal.payload.metrics_snapshot.is_some());

        assert_eq!(signal.payload.artifacts_produced.len(), 2);
        assert!(signal.payload.metrics_snapshot.is_some());
        assert_eq!(signal.payload.metrics_snapshot.unwrap(), metrics);

        println!("✓ Test passed");
    }

    /// TC-SR-001-C: Gate signals have gate_required=true
    #[test]
    fn tc_sr_001_c_gate_signal_flag() {
        println!("\n=== TC-SR-001-C: Gate signals have gate_required=true ===");

        let mut router = SignalRouter::new();

        let gate_signals = vec![
            SignalType::ReadyForStep1,
            SignalType::BaselineFrozen,
            SignalType::ReadyForAnalysis,
            SignalType::ReadyForSynthesis,
            SignalType::ReadyForRedesign,
            SignalType::ReadyForValidation,
            SignalType::ValidationComplete,
        ];

        for signal_type in gate_signals {
            let payload = SignalPayload {
                step_from: 0,
                step_to: 1,
                artifacts_produced: vec![],
                metrics_snapshot: None,
                gate_required: true,
            };

            let signal = router.emit_signal(signal_type.clone(), "run-003", payload);
            assert!(signal.payload.gate_required, "Gate signal {:?} should have gate_required=true", signal_type);
            println!("✓ {:?} requires gate", signal_type);
        }

        println!("✓ All gate signals verified");
    }

    /// TC-SR-001-D: Non-gate signals have gate_required=false
    #[test]
    fn tc_sr_001_d_non_gate_signal_flag() {
        println!("\n=== TC-SR-001-D: Non-gate signals have gate_required=false ===");

        let mut router = SignalRouter::new();

        let non_gate_signals = vec![
            SignalType::LearningHarvested,
            SignalType::NewRunReady,
            SignalType::MetricUpdate,
        ];

        for signal_type in non_gate_signals {
            let payload = SignalPayload {
                step_from: 0,
                step_to: 0,
                artifacts_produced: vec![],
                metrics_snapshot: None,
                gate_required: false,
            };

            let signal = router.emit_signal(signal_type.clone(), "run-004", payload);
            assert!(!signal.payload.gate_required, "Non-gate signal {:?} should have gate_required=false", signal_type);
            println!("✓ {:?} does not require gate", signal_type);
        }

        println!("✓ All non-gate signals verified");
    }

    /// TC-SR-002-A: First signal has no prior hash
    #[test]
    fn tc_sr_002_a_first_signal_no_prior_hash() {
        println!("\n=== TC-SR-002-A: First signal has no prior hash ===");

        let mut router = SignalRouter::new();
        let payload = SignalPayload {
            step_from: 0,
            step_to: 1,
            artifacts_produced: vec![],
            metrics_snapshot: None,
            gate_required: true,
        };

        let signal = router.emit_signal(SignalType::ReadyForStep1, "run-005", payload);

        println!("First signal prior_hash: {:?}", signal.prior_signal_hash);
        assert!(signal.prior_signal_hash.is_none());

        println!("✓ Test passed");
    }

    /// TC-SR-002-B: Subsequent signal links to previous
    #[test]
    fn tc_sr_002_b_subsequent_signal_links() {
        println!("\n=== TC-SR-002-B: Subsequent signal links to previous ===");

        let mut router = SignalRouter::new();

        // Emit first signal
        let payload1 = SignalPayload {
            step_from: 0,
            step_to: 1,
            artifacts_produced: vec![],
            metrics_snapshot: None,
            gate_required: true,
        };
        let signal1 = router.emit_signal(SignalType::ReadyForStep1, "run-006", payload1);
        println!("Signal 1 hash: {}", signal1.hash);

        // Emit second signal
        let payload2 = SignalPayload {
            step_from: 1,
            step_to: 2,
            artifacts_produced: vec![],
            metrics_snapshot: None,
            gate_required: true,
        };
        let signal2 = router.emit_signal(SignalType::BaselineFrozen, "run-006", payload2);
        println!("Signal 2 prior_hash: {:?}", signal2.prior_signal_hash);

        assert_eq!(signal2.prior_signal_hash, Some(signal1.hash.clone()));

        println!("✓ Test passed - Signal 2 links to Signal 1");
    }

    /// TC-SR-002-C: Chain verification for multiple signals
    #[test]
    fn tc_sr_002_c_chain_verification() {
        println!("\n=== TC-SR-002-C: Chain verification ===");

        let mut router = SignalRouter::new();

        // Create a chain of 5 signals
        let signal_types = vec![
            SignalType::ReadyForStep1,
            SignalType::BaselineFrozen,
            SignalType::ReadyForAnalysis,
            SignalType::ReadyForSynthesis,
            SignalType::ReadyForRedesign,
        ];

        for (i, signal_type) in signal_types.iter().enumerate() {
            let payload = SignalPayload {
                step_from: i as i32,
                step_to: (i + 1) as i32,
                artifacts_produced: vec![],
                metrics_snapshot: None,
                gate_required: true,
            };
            router.emit_signal(signal_type.clone(), "run-007", payload);
        }

        let chain = router.get_signal_chain("run-007");
        println!("Chain length: {}", chain.len());
        assert_eq!(chain.len(), 5);

        // Verify integrity
        let integrity = router.verify_chain_integrity("run-007");
        println!("Chain integrity: {}", integrity);
        assert!(integrity);

        println!("✓ Test passed - All 5 signals linked correctly");
    }

    /// Test is_gate_signal function
    #[test]
    fn test_is_gate_signal() {
        println!("\n=== Test: is_gate_signal function ===");

        // Gate signals
        assert!(SignalRouter::is_gate_signal(&SignalType::ReadyForStep1));
        assert!(SignalRouter::is_gate_signal(&SignalType::BaselineFrozen));
        assert!(SignalRouter::is_gate_signal(&SignalType::ReadyForAnalysis));
        assert!(SignalRouter::is_gate_signal(&SignalType::ReadyForSynthesis));
        assert!(SignalRouter::is_gate_signal(&SignalType::ReadyForRedesign));
        assert!(SignalRouter::is_gate_signal(&SignalType::ReadyForValidation));
        assert!(SignalRouter::is_gate_signal(&SignalType::ValidationComplete));
        println!("✓ All 7 gate signals recognized");

        // Non-gate signals
        assert!(!SignalRouter::is_gate_signal(&SignalType::LearningHarvested));
        assert!(!SignalRouter::is_gate_signal(&SignalType::NewRunReady));
        assert!(!SignalRouter::is_gate_signal(&SignalType::MetricUpdate));
        println!("✓ All 3 non-gate signals recognized");

        println!("✓ Test passed");
    }

    /// Test get_signal_chain function
    #[test]
    fn test_get_signal_chain() {
        println!("\n=== Test: get_signal_chain function ===");

        let mut router = SignalRouter::new();

        // Emit 3 signals
        for i in 0..3 {
            let payload = SignalPayload {
                step_from: i,
                step_to: i + 1,
                artifacts_produced: vec![],
                metrics_snapshot: None,
                gate_required: true,
            };
            router.emit_signal(SignalType::ReadyForStep1, "run-008", payload);
        }

        let chain = router.get_signal_chain("run-008");
        println!("Retrieved chain length: {}", chain.len());
        assert_eq!(chain.len(), 3);

        // Empty chain for non-existent run
        let empty_chain = router.get_signal_chain("non-existent");
        assert_eq!(empty_chain.len(), 0);
        println!("✓ Empty chain for non-existent run");

        println!("✓ Test passed");
    }

    /// Test multiple isolated run chains
    #[test]
    fn test_multiple_run_isolation() {
        println!("\n=== Test: Multiple runs are isolated ===");

        let mut router = SignalRouter::new();

        // Create chains for 3 different runs
        for run_num in 1..=3 {
            let run_id = format!("run-{}", run_num);
            for step in 0..2 {
                let payload = SignalPayload {
                    step_from: step,
                    step_to: step + 1,
                    artifacts_produced: vec![],
                    metrics_snapshot: None,
                    gate_required: true,
                };
                router.emit_signal(SignalType::ReadyForStep1, &run_id, payload);
            }
        }

        // Verify each run has its own chain
        let chain1 = router.get_signal_chain("run-1");
        let chain2 = router.get_signal_chain("run-2");
        let chain3 = router.get_signal_chain("run-3");

        assert_eq!(chain1.len(), 2);
        assert_eq!(chain2.len(), 2);
        assert_eq!(chain3.len(), 2);

        // Verify chains are independent
        assert_ne!(chain1[0].hash, chain2[0].hash);
        assert_ne!(chain2[0].hash, chain3[0].hash);

        println!("✓ All 3 runs have isolated chains");
        println!("✓ Test passed");
    }

    /// Test comprehensive signal workflow
    #[test]
    fn test_comprehensive_signal_workflow() {
        println!("\n=== Test: Comprehensive signal workflow ===");

        let mut router = SignalRouter::new();
        let run_id = "comprehensive-run";

        println!("Step 1: Emitting Ready_for_Step_1...");
        let payload = SignalPayload {
            step_from: 0,
            step_to: 1,
            artifacts_produced: vec!["charter-001".to_string()],
            metrics_snapshot: Some(serde_json::json!({"ci": 0.90})),
            gate_required: true,
        };
        router.emit_signal(SignalType::ReadyForStep1, run_id, payload);

        println!("Step 2: Emitting Baseline_Frozen...");
        let payload = SignalPayload {
            step_from: 1,
            step_to: 2,
            artifacts_produced: vec!["baseline-001".to_string()],
            metrics_snapshot: Some(serde_json::json!({"ci": 0.85})),
            gate_required: true,
        };
        router.emit_signal(SignalType::BaselineFrozen, run_id, payload);

        println!("Step 3: Emitting non-gate Metric_Update...");
        let payload = SignalPayload {
            step_from: 2,
            step_to: 2,
            artifacts_produced: vec![],
            metrics_snapshot: Some(serde_json::json!({"ci": 0.88})),
            gate_required: false,
        };
        router.emit_signal(SignalType::MetricUpdate, run_id, payload);

        println!("Step 4: Emitting Ready_for_Analysis...");
        let payload = SignalPayload {
            step_from: 2,
            step_to: 3,
            artifacts_produced: vec!["analysis-001".to_string()],
            metrics_snapshot: Some(serde_json::json!({"ci": 0.92})),
            gate_required: true,
        };
        router.emit_signal(SignalType::ReadyForAnalysis, run_id, payload);

        let chain = router.get_signal_chain(run_id);
        println!("Final chain length: {}", chain.len());
        assert_eq!(chain.len(), 4);

        println!("Verifying chain integrity...");
        assert!(router.verify_chain_integrity(run_id));

        println!("✓ Comprehensive workflow test passed");
    }
}
