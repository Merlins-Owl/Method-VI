use super::types::*;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Ledger Manager
///
/// Manages the ledger as active state, not passive logging.
/// The ledger state determines what actions are legal.
///
/// Before any action:
/// 1. Query ledger for current state
/// 2. Validate action is legal given state
/// 3. Execute action
/// 4. Record result to ledger
pub struct LedgerManager {
    /// Entries organized by run_id
    entries: HashMap<String, Vec<LedgerEntry>>,
}

impl LedgerManager {
    /// Creates a new empty LedgerManager
    pub fn new() -> Self {
        LedgerManager {
            entries: HashMap::new(),
        }
    }

    /// Creates a new ledger entry with hash chain integrity
    ///
    /// # Arguments
    /// * `run_id` - ID of the run
    /// * `entry_type` - Type of entry
    /// * `step` - Current step (0-6.5)
    /// * `role` - Active governance role
    /// * `payload` - Entry payload
    ///
    /// # Returns
    /// A new LedgerEntry with hash calculated and prior_hash set
    ///
    /// # Example
    /// ```
    /// use method_vi::ledger::{LedgerManager, EntryType, LedgerPayload};
    ///
    /// let mut manager = LedgerManager::new();
    /// let payload = LedgerPayload {
    ///     action: "capture_intent".to_string(),
    ///     inputs: None,
    ///     outputs: None,
    ///     rationale: Some("User initiated run".to_string()),
    /// };
    ///
    /// let entry = manager.create_entry(
    ///     "run-001",
    ///     EntryType::Signal,
    ///     Some(0),
    ///     Some("Orchestrator"),
    ///     payload,
    /// );
    /// ```
    pub fn create_entry(
        &mut self,
        run_id: &str,
        entry_type: EntryType,
        step: Option<i32>,
        role: Option<&str>,
        payload: LedgerPayload,
    ) -> LedgerEntry {
        // Get prior hash from last entry in this run
        let prior_hash = self
            .entries
            .get(run_id)
            .and_then(|entries| entries.last())
            .map(|entry| entry.hash.clone());

        let entry = LedgerEntry {
            id: None, // Will be set by database
            run_id: run_id.to_string(),
            entry_type,
            step,
            role: role.map(|s| s.to_string()),
            payload,
            prior_hash,
            hash: String::new(), // Will be calculated
            created_at: Utc::now(),
        };

        // Calculate hash for this entry
        let hash = self.calculate_entry_hash(&entry);
        let mut entry = entry;
        entry.hash = hash;

        // Add to entries
        self.entries
            .entry(run_id.to_string())
            .or_insert_with(Vec::new)
            .push(entry.clone());

        entry
    }

    /// Validates if an action is legal given the current state
    ///
    /// State Transition Rules:
    /// - Step 0 active: Allow intent capture, pattern query; Block baseline freeze, validation
    /// - Baseline frozen: Allow analysis, synthesis; Block scope changes, baseline edits
    /// - Gate pending: Allow human approve/reject; Block agent progression
    /// - HALT active: Allow human decision only; Block all automated actions
    ///
    /// # Arguments
    /// * `current_state` - Current ledger state
    /// * `proposed_action` - Action to validate
    ///
    /// # Returns
    /// ActionValidationResult with allowed flag and reason
    ///
    /// # Example
    /// ```
    /// use method_vi::ledger::{LedgerManager, LedgerState};
    ///
    /// let manager = LedgerManager::new();
    /// let result = manager.validate_action(
    ///     &LedgerState::Step0Active,
    ///     "intent_capture"
    /// );
    /// assert!(result.allowed);
    /// ```
    pub fn validate_action(
        &self,
        current_state: &LedgerState,
        proposed_action: &str,
    ) -> ActionValidationResult {
        match current_state {
            LedgerState::Step0Active => {
                // Legal: intent capture, pattern query
                // Illegal: baseline freeze, validation
                match proposed_action {
                    "intent_capture" | "pattern_query" => ActionValidationResult::allowed(),
                    "baseline_freeze" => {
                        ActionValidationResult::rejected("Cannot freeze baseline in Step 0")
                    }
                    "validation" => {
                        ActionValidationResult::rejected("Cannot validate in Step 0 - too early")
                    }
                    _ => ActionValidationResult::allowed(), // Other actions allowed by default
                }
            }

            LedgerState::BaselineFrozen => {
                // Legal: analysis, synthesis
                // Illegal: scope changes, baseline edits
                match proposed_action {
                    "analysis" | "synthesis" => ActionValidationResult::allowed(),
                    "scope_change" => ActionValidationResult::rejected(
                        "Cannot change scope - baseline is frozen",
                    ),
                    "baseline_edit" => {
                        ActionValidationResult::rejected("Cannot edit baseline - it is immutable")
                    }
                    _ => ActionValidationResult::allowed(),
                }
            }

            LedgerState::GatePending => {
                // Legal: human approve/reject
                // Illegal: agent progression
                match proposed_action {
                    "human_approve" | "human_reject" => ActionValidationResult::allowed(),
                    "agent_progression" => ActionValidationResult::rejected(
                        "Cannot progress - awaiting human decision at gate",
                    ),
                    _ => {
                        // Most other automated actions should be blocked
                        if proposed_action.starts_with("agent_") {
                            ActionValidationResult::rejected("Gate pending - human approval required")
                        } else {
                            ActionValidationResult::allowed()
                        }
                    }
                }
            }

            LedgerState::HaltActive => {
                // Legal: human decision only
                // Illegal: any automated action
                if proposed_action.starts_with("human_") {
                    ActionValidationResult::allowed()
                } else {
                    ActionValidationResult::rejected(
                        "System is HALTED - only human decisions allowed",
                    )
                }
            }

            LedgerState::Normal => {
                // Normal operation - most actions allowed
                ActionValidationResult::allowed()
            }
        }
    }

    /// Checks metrics against thresholds to determine HALT/PAUSE status
    ///
    /// HALT/PAUSE Triggers:
    /// - CI < 0.50 → HALT_IMMEDIATE
    /// - EV > ±30% → HALT_IMMEDIATE
    /// - SEC violation → HALT_IMMEDIATE
    /// - CI 0.70-0.80 → PAUSE_FOR_REVIEW
    ///
    /// # Arguments
    /// * `metrics` - Current metrics snapshot
    ///
    /// # Returns
    /// HaltStatus indicating whether to continue, pause, or halt
    ///
    /// # Example
    /// ```
    /// use method_vi::ledger::{LedgerManager, MetricsSnapshot};
    /// use chrono::Utc;
    ///
    /// let manager = LedgerManager::new();
    /// let metrics = MetricsSnapshot {
    ///     ci: 0.95,
    ///     ev: 10.0,
    ///     sec: true,
    ///     timestamp: Utc::now(),
    /// };
    ///
    /// let status = manager.check_thresholds(&metrics);
    /// // Should return Continue
    /// ```
    pub fn check_thresholds(&self, metrics: &MetricsSnapshot) -> HaltStatus {
        // Check CI < 0.50 → HALT_IMMEDIATE
        if metrics.ci < 0.50 {
            return HaltStatus::HaltImmediate {
                reason: format!("CI below threshold: {:.2} < 0.50", metrics.ci),
            };
        }

        // Check EV > ±30% → HALT_IMMEDIATE
        if metrics.ev.abs() > 30.0 {
            return HaltStatus::HaltImmediate {
                reason: format!("EV exceeds threshold: {:.2}% > ±30%", metrics.ev),
            };
        }

        // Check SEC violation → HALT_IMMEDIATE
        if !metrics.sec {
            return HaltStatus::HaltImmediate {
                reason: "SEC violation: Undocumented expansion detected".to_string(),
            };
        }

        // Check CI 0.70-0.80 → PAUSE_FOR_REVIEW
        // Note: TC-LM-004-H says CI = 0.69 should NOT pause (below 0.70)
        if metrics.ci >= 0.70 && metrics.ci <= 0.80 {
            return HaltStatus::PauseForReview {
                reason: format!("CI in warning zone: {:.2} (0.70-0.80)", metrics.ci),
            };
        }

        HaltStatus::Continue
    }

    /// Verifies the hash chain integrity for a run
    ///
    /// Checks:
    /// 1. First entry has prior_hash = null
    /// 2. Each subsequent entry's prior_hash matches previous entry's hash
    /// 3. Each entry's hash is correctly calculated
    ///
    /// # Arguments
    /// * `run_id` - ID of the run to verify
    ///
    /// # Returns
    /// true if chain is intact, false if broken or tampered
    ///
    /// # Example
    /// ```
    /// use method_vi::ledger::LedgerManager;
    ///
    /// let manager = LedgerManager::new();
    /// assert!(manager.verify_chain_integrity("run-001"));
    /// ```
    pub fn verify_chain_integrity(&self, run_id: &str) -> bool {
        let entries = match self.entries.get(run_id) {
            Some(e) => e,
            None => return true, // Empty chain is valid
        };

        if entries.is_empty() {
            return true;
        }

        // First entry should have prior_hash = None
        if entries[0].prior_hash.is_some() {
            return false;
        }

        // Verify first entry's hash
        if entries[0].hash != self.calculate_entry_hash(&entries[0]) {
            return false;
        }

        // Verify chain links
        for i in 1..entries.len() {
            let prev_entry = &entries[i - 1];
            let current_entry = &entries[i];

            // Check prior_hash matches previous hash
            if current_entry.prior_hash.as_ref() != Some(&prev_entry.hash) {
                return false;
            }

            // Verify current entry's hash
            if current_entry.hash != self.calculate_entry_hash(current_entry) {
                return false;
            }
        }

        true
    }

    /// Gets the current state for a run based on its ledger
    ///
    /// Analyzes the ledger entries to determine the current state
    pub fn get_current_state(&self, run_id: &str) -> LedgerState {
        let entries = match self.entries.get(run_id) {
            Some(e) => e,
            None => return LedgerState::Step0Active, // New run starts at Step 0
        };

        if entries.is_empty() {
            return LedgerState::Step0Active;
        }

        // Check last entry for state indicators
        let last_entry = &entries[entries.len() - 1];

        // Check for HALT status
        if last_entry.payload.action.contains("HALT") {
            return LedgerState::HaltActive;
        }

        // Check for gate pending
        if last_entry.entry_type == EntryType::Gate {
            // Check if gate has been resolved
            if entries.len() > 1 {
                let next_entries: Vec<_> = entries.iter().skip(entries.len() - 1).collect();
                if !next_entries
                    .iter()
                    .any(|e| e.payload.action.contains("approve") || e.payload.action.contains("reject"))
                {
                    return LedgerState::GatePending;
                }
            } else {
                return LedgerState::GatePending;
            }
        }

        // Check for baseline frozen
        if last_entry.payload.action.contains("baseline_freeze") {
            return LedgerState::BaselineFrozen;
        }

        // Check step number
        if let Some(step) = last_entry.step {
            if step == 0 {
                return LedgerState::Step0Active;
            }
        }

        LedgerState::Normal
    }

    /// Calculates SHA-256 hash for a ledger entry
    ///
    /// Hash includes: run_id, entry_type, step, role, payload, prior_hash, created_at
    fn calculate_entry_hash(&self, entry: &LedgerEntry) -> String {
        let mut hasher = Sha256::new();

        // Add entry fields to hash
        hasher.update(entry.run_id.as_bytes());
        hasher.update(format!("{:?}", entry.entry_type).as_bytes());

        if let Some(step) = entry.step {
            hasher.update(step.to_string().as_bytes());
        }

        if let Some(role) = &entry.role {
            hasher.update(role.as_bytes());
        }

        hasher.update(entry.payload.action.as_bytes());

        if let Some(inputs) = &entry.payload.inputs {
            hasher.update(inputs.to_string().as_bytes());
        }

        if let Some(outputs) = &entry.payload.outputs {
            hasher.update(outputs.to_string().as_bytes());
        }

        if let Some(rationale) = &entry.payload.rationale {
            hasher.update(rationale.as_bytes());
        }

        if let Some(prior_hash) = &entry.prior_hash {
            hasher.update(prior_hash.as_bytes());
        }

        hasher.update(entry.created_at.to_rfc3339().as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Gets all entries for a run (for testing/debugging)
    pub fn get_entries(&self, run_id: &str) -> Vec<LedgerEntry> {
        self.entries
            .get(run_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for LedgerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test payload
    fn create_payload(action: &str) -> LedgerPayload {
        LedgerPayload {
            action: action.to_string(),
            inputs: None,
            outputs: None,
            rationale: Some(format!("Test: {}", action)),
        }
    }

    // ===== TC-LM-001: State Transition Validation Tests =====

    #[test]
    fn tc_lm_001_a_step0_intent_capture_allowed() {
        println!("\n=== TC-LM-001-A: Step 0 - Intent capture allowed ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::Step0Active, "intent_capture");
        println!("Result: {:?}", result);
        assert!(result.allowed, "Intent capture should be allowed in Step 0");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_b_step0_pattern_query_allowed() {
        println!("\n=== TC-LM-001-B: Step 0 - Pattern query allowed ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::Step0Active, "pattern_query");
        println!("Result: {:?}", result);
        assert!(result.allowed, "Pattern query should be allowed in Step 0");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_c_step0_baseline_freeze_rejected() {
        println!("\n=== TC-LM-001-C: Step 0 - Baseline freeze rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::Step0Active, "baseline_freeze");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Baseline freeze should be rejected in Step 0");
        assert!(result.reason.is_some());
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_d_step0_validation_rejected() {
        println!("\n=== TC-LM-001-D: Step 0 - Validation rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::Step0Active, "validation");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Validation should be rejected in Step 0 - too early");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_e_baseline_frozen_analysis_allowed() {
        println!("\n=== TC-LM-001-E: Baseline frozen - Analysis allowed ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::BaselineFrozen, "analysis");
        println!("Result: {:?}", result);
        assert!(result.allowed, "Analysis should be allowed when baseline is frozen");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_f_baseline_frozen_scope_change_rejected() {
        println!("\n=== TC-LM-001-F: Baseline frozen - Scope change rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::BaselineFrozen, "scope_change");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Scope change should be rejected - baseline locked");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_g_baseline_frozen_baseline_edit_rejected() {
        println!("\n=== TC-LM-001-G: Baseline frozen - Baseline edit rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::BaselineFrozen, "baseline_edit");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Baseline edit should be rejected - immutable");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_h_gate_pending_human_approve_allowed() {
        println!("\n=== TC-LM-001-H: Gate pending - Human approve allowed ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::GatePending, "human_approve");
        println!("Result: {:?}", result);
        assert!(result.allowed, "Human approve should be allowed at gate");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_i_gate_pending_agent_progression_rejected() {
        println!("\n=== TC-LM-001-I: Gate pending - Agent progression rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::GatePending, "agent_progression");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Agent progression should be rejected - awaiting human");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_j_halt_active_human_decision_allowed() {
        println!("\n=== TC-LM-001-J: HALT active - Human decision allowed ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::HaltActive, "human_decision");
        println!("Result: {:?}", result);
        assert!(result.allowed, "Human decision should be allowed when HALTED");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_001_k_halt_active_automated_action_rejected() {
        println!("\n=== TC-LM-001-K: HALT active - Automated action rejected ===");
        let manager = LedgerManager::new();

        let result = manager.validate_action(&LedgerState::HaltActive, "automated_action");
        println!("Result: {:?}", result);
        assert!(!result.allowed, "Automated actions should be rejected when HALTED");
        println!("✓ Test passed\n");
    }

    // ===== TC-LM-002: Ledger Entry Creation Tests =====

    #[test]
    fn tc_lm_002_a_create_gate_entry() {
        println!("\n=== TC-LM-002-A: Create gate entry ===");
        let mut manager = LedgerManager::new();

        let payload = create_payload("gate_checkpoint");
        let entry = manager.create_entry(
            "run-001",
            EntryType::Gate,
            Some(2),
            Some("Orchestrator"),
            payload,
        );

        println!("Created entry: {:?}", entry);
        assert_eq!(entry.run_id, "run-001");
        assert_eq!(entry.entry_type, EntryType::Gate);
        assert_eq!(entry.step, Some(2));
        assert_eq!(entry.role, Some("Orchestrator".to_string()));
        assert!(!entry.hash.is_empty(), "Hash should be calculated");
        assert!(entry.prior_hash.is_none(), "First entry should have no prior hash");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_002_multiple_entry_types() {
        println!("\n=== TC-LM-002: Create various entry types ===");
        let mut manager = LedgerManager::new();

        // Intervention entry
        let entry = manager.create_entry(
            "run-001",
            EntryType::Intervention,
            Some(3),
            Some("Circuit Breaker"),
            create_payload("ci_threshold_breach"),
        );
        assert_eq!(entry.entry_type, EntryType::Intervention);

        // Signal entry
        let entry = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(4),
            Some("Signal Router"),
            create_payload("step_transition"),
        );
        assert_eq!(entry.entry_type, EntryType::Signal);

        // Decision entry
        let entry = manager.create_entry(
            "run-001",
            EntryType::Decision,
            Some(5),
            Some("Human"),
            create_payload("approve_progression"),
        );
        assert_eq!(entry.entry_type, EntryType::Decision);

        // Metric snapshot entry
        let entry = manager.create_entry(
            "run-001",
            EntryType::MetricSnapshot,
            Some(6),
            Some("Metrics System"),
            create_payload("metrics_captured"),
        );
        assert_eq!(entry.entry_type, EntryType::MetricSnapshot);

        println!("✓ All entry types created successfully\n");
    }

    // ===== TC-LM-003: Hash Chain Integrity Tests =====

    #[test]
    fn tc_lm_003_a_first_entry_no_prior_hash() {
        println!("\n=== TC-LM-003-A: First entry has no prior hash ===");
        let mut manager = LedgerManager::new();

        let entry = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("run_started"),
        );

        println!("First entry prior_hash: {:?}", entry.prior_hash);
        assert!(entry.prior_hash.is_none(), "First entry should have no prior hash");
        assert!(!entry.hash.is_empty(), "Hash should be calculated");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_003_b_second_entry_links_to_first() {
        println!("\n=== TC-LM-003-B: Second entry links to first ===");
        let mut manager = LedgerManager::new();

        let entry1 = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("run_started"),
        );
        let hash1 = entry1.hash.clone();

        let entry2 = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(1),
            Some("Orchestrator"),
            create_payload("step_1_started"),
        );

        println!("Entry 1 hash: {}", hash1);
        println!("Entry 2 prior_hash: {:?}", entry2.prior_hash);
        assert_eq!(entry2.prior_hash, Some(hash1), "Second entry's prior_hash should match first entry's hash");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_003_c_nth_entry_links_to_previous() {
        println!("\n=== TC-LM-003-C: Nth entry links to (N-1)th ===");
        let mut manager = LedgerManager::new();

        // Create chain of 5 entries
        let mut prev_hash = None;
        for i in 0..5 {
            let entry = manager.create_entry(
                "run-001",
                EntryType::Signal,
                Some(i),
                Some("Orchestrator"),
                create_payload(&format!("step_{}", i)),
            );

            if i > 0 {
                assert_eq!(entry.prior_hash, prev_hash, "Entry {} should link to entry {}", i, i - 1);
            }

            prev_hash = Some(entry.hash);
        }

        println!("✓ Chain of 5 entries linked correctly\n");
    }

    #[test]
    fn tc_lm_003_d_verify_chain_integrity_valid() {
        println!("\n=== TC-LM-003-D: Verify valid chain integrity ===");
        let mut manager = LedgerManager::new();

        // Create chain
        for i in 0..5 {
            manager.create_entry(
                "run-001",
                EntryType::Signal,
                Some(i),
                Some("Orchestrator"),
                create_payload(&format!("step_{}", i)),
            );
        }

        let valid = manager.verify_chain_integrity("run-001");
        println!("Chain integrity: {}", valid);
        assert!(valid, "Chain should be valid");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_003_e_detect_tampered_entry() {
        println!("\n=== TC-LM-003-E: Detect tampered entry ===");
        let mut manager = LedgerManager::new();

        // Create chain
        manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("entry_1"),
        );
        manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(1),
            Some("Orchestrator"),
            create_payload("entry_2"),
        );

        // Tamper with the chain by modifying an entry's hash
        if let Some(entries) = manager.entries.get_mut("run-001") {
            entries[1].hash = "tampered_hash".to_string();
        }

        let valid = manager.verify_chain_integrity("run-001");
        println!("Chain integrity after tampering: {}", valid);
        assert!(!valid, "Chain should be invalid after tampering");
        println!("✓ Test passed\n");
    }

    // ===== TC-LM-004: HALT/PAUSE Trigger Tests =====

    #[test]
    fn tc_lm_004_a_ci_below_threshold_halts() {
        println!("\n=== TC-LM-004-A: CI < 0.50 → HALT_IMMEDIATE ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.49,
            ev: 10.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::HaltImmediate { .. }), "Should HALT when CI < 0.50");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_b_ci_at_threshold_continues() {
        println!("\n=== TC-LM-004-B: CI = 0.50 → Continue ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.50,
            ev: 10.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::Continue), "Should continue when CI = 0.50");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_c_ev_exceeds_threshold_halts() {
        println!("\n=== TC-LM-004-C: EV > ±30% → HALT_IMMEDIATE ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.95,
            ev: 31.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::HaltImmediate { .. }), "Should HALT when EV > +30%");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_d_ev_at_limit_continues() {
        println!("\n=== TC-LM-004-D: EV = -30% → Continue ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.95,
            ev: -30.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::Continue), "Should continue when EV = -30%");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_e_sec_violation_halts() {
        println!("\n=== TC-LM-004-E: SEC violation → HALT_IMMEDIATE ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.95,
            ev: 10.0,
            sec: false, // SEC violation
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::HaltImmediate { .. }), "Should HALT on SEC violation");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_g_ci_warning_zone_pauses() {
        println!("\n=== TC-LM-004-G: CI = 0.72 → PAUSE_FOR_REVIEW ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.72,
            ev: 10.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::PauseForReview { .. }), "Should PAUSE when CI in warning zone (0.70-0.80)");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_lm_004_h_ci_below_warning_continues() {
        println!("\n=== TC-LM-004-H: CI = 0.69 → Continue (no pause) ===");
        let manager = LedgerManager::new();

        let metrics = MetricsSnapshot {
            ci: 0.69,
            ev: 10.0,
            sec: true,
            timestamp: Utc::now(),
        };

        let status = manager.check_thresholds(&metrics);
        println!("Status: {:?}", status);
        assert!(matches!(status, HaltStatus::Continue), "Should continue when CI < 0.70 (below warning zone)");
        println!("✓ Test passed\n");
    }

    // ===== Additional Comprehensive Tests =====

    #[test]
    fn test_comprehensive_ledger_workflow() {
        println!("\n=== Test: Comprehensive Ledger Workflow ===");
        let mut manager = LedgerManager::new();

        // Step 1: Create run start entry
        println!("Step 1: Creating run start entry...");
        let entry1 = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("run_started"),
        );
        assert!(entry1.prior_hash.is_none());

        // Step 2: Add intent capture
        println!("Step 2: Adding intent capture...");
        let entry2 = manager.create_entry(
            "run-001",
            EntryType::Decision,
            Some(0),
            Some("Orchestrator"),
            create_payload("intent_captured"),
        );
        assert_eq!(entry2.prior_hash, Some(entry1.hash.clone()));

        // Step 3: Add baseline freeze
        println!("Step 3: Adding baseline freeze...");
        let entry3 = manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(2),
            Some("Orchestrator"),
            create_payload("baseline_freeze"),
        );
        assert_eq!(entry3.prior_hash, Some(entry2.hash.clone()));

        // Step 4: Add gate entry
        println!("Step 4: Adding gate entry...");
        let entry4 = manager.create_entry(
            "run-001",
            EntryType::Gate,
            Some(3),
            Some("Orchestrator"),
            create_payload("gate_checkpoint"),
        );

        // Step 5: Add metric snapshot
        println!("Step 5: Adding metric snapshot...");
        manager.create_entry(
            "run-001",
            EntryType::MetricSnapshot,
            Some(4),
            Some("Metrics System"),
            create_payload("metrics_captured"),
        );

        // Verify chain integrity
        println!("Verifying chain integrity...");
        assert!(manager.verify_chain_integrity("run-001"));

        // Verify entry count
        let entries = manager.get_entries("run-001");
        assert_eq!(entries.len(), 5);

        println!("✓ Comprehensive workflow test passed\n");
    }

    #[test]
    fn test_multiple_runs_isolated() {
        println!("\n=== Test: Multiple runs are isolated ===");
        let mut manager = LedgerManager::new();

        // Create entries for run-001
        manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("run_1_entry_1"),
        );
        manager.create_entry(
            "run-001",
            EntryType::Signal,
            Some(1),
            Some("Orchestrator"),
            create_payload("run_1_entry_2"),
        );

        // Create entries for run-002
        let run2_entry1 = manager.create_entry(
            "run-002",
            EntryType::Signal,
            Some(0),
            Some("Orchestrator"),
            create_payload("run_2_entry_1"),
        );

        // run-002's first entry should have no prior hash (independent chain)
        assert!(run2_entry1.prior_hash.is_none());

        // Verify both chains are valid
        assert!(manager.verify_chain_integrity("run-001"));
        assert!(manager.verify_chain_integrity("run-002"));

        // Verify entry counts
        assert_eq!(manager.get_entries("run-001").len(), 2);
        assert_eq!(manager.get_entries("run-002").len(), 1);

        println!("✓ Multiple runs are properly isolated\n");
    }
}
