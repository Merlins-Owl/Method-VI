use super::types::{Mode, Role, RunContext, Signal};

/// Context Manager for generating Steno-Ledger strings
///
/// The Steno-Ledger provides agents with current run context without
/// re-reading full history. It's prepended to every agent prompt.
pub struct ContextManager;

impl ContextManager {
    /// Generate Steno-Ledger string from run context
    ///
    /// Format: [RUN:{id} | S:{step} | R:{role} | CI:{value} | EV:{±value}% | M:{mode} | 🚦:{signal}]
    ///
    /// # Example
    /// ```
    /// use method_vi_lib::context::{RunContext, Role, Mode, Signal, ContextManager};
    ///
    /// let context = RunContext {
    ///     run_id: "test-run".to_string(),
    ///     step: 3,
    ///     role: Role::Observer,
    ///     ci: Some(0.87),
    ///     ev: Some(3.0),
    ///     mode: Mode::Standard,
    ///     signal: Signal::ReadyForSynthesis,
    /// };
    ///
    /// let steno = ContextManager::generate_steno_ledger(&context);
    /// assert_eq!(steno, "[RUN:test-run | S:3 | R:OBS | CI:0.87 | EV:+3% | M:STD | 🚦:Ready_for_Synthesis]");
    /// ```
    pub fn generate_steno_ledger(context: &RunContext) -> String {
        let role_abbr = Self::get_role_abbreviation(&context.role);
        let mode_abbr = Self::get_mode_abbreviation(&context.mode);
        let signal_str = Self::get_signal_string(&context.signal);

        // Format CI value
        let ci_str = match context.ci {
            Some(value) => format!("{:.2}", value),
            None => "-".to_string(),
        };

        // Format EV value with sign and percentage
        let ev_str = match context.ev {
            Some(value) => {
                if value >= 0.0 {
                    format!("+{}%", value as i32)
                } else {
                    format!("{}%", value as i32)
                }
            }
            None => "-".to_string(),
        };

        format!(
            "[RUN:{} | S:{} | R:{} | CI:{} | EV:{} | M:{} | 🚦:{}]",
            context.run_id, context.step, role_abbr, ci_str, ev_str, mode_abbr, signal_str
        )
    }

    /// Get role abbreviation
    ///
    /// # Abbreviations
    /// - Observer → OBS
    /// - Conductor → COND
    /// - Auditor → AUD
    /// - Patcher → PATCH
    /// - Fabricator → FAB
    /// - Examiner → EXAM
    /// - Curator → CUR
    /// - Archivist → ARCH
    pub fn get_role_abbreviation(role: &Role) -> String {
        match role {
            Role::Observer => "OBS",
            Role::Conductor => "COND",
            Role::Auditor => "AUD",
            Role::Patcher => "PATCH",
            Role::Fabricator => "FAB",
            Role::Examiner => "EXAM",
            Role::Curator => "CUR",
            Role::Archivist => "ARCH",
        }
        .to_string()
    }

    /// Get mode abbreviation
    ///
    /// # Abbreviations
    /// - Standard → STD
    /// - Component → COMP
    /// - Surgical → SURG
    pub fn get_mode_abbreviation(mode: &Mode) -> String {
        match mode {
            Mode::Standard => "STD",
            Mode::Component => "COMP",
            Mode::Surgical => "SURG",
        }
        .to_string()
    }

    /// Get signal string representation
    fn get_signal_string(signal: &Signal) -> String {
        match signal {
            Signal::Initializing => "Initializing",
            Signal::ReadyForSynthesis => "Ready_for_Synthesis",
            Signal::AwaitingGate => "Awaiting_Gate",
            Signal::Halted => "Halted",
            Signal::PausedForReview => "Paused_for_Review",
            Signal::Completed => "Completed",
            Signal::Active => "Active",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-CM-001-A: Initial state with null CI/EV
    #[test]
    fn tc_cm_001_a_initial_state() {
        println!("\n=== TC-CM-001-A: Initial state with null CI/EV ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 0,
            role: Role::Observer,
            ci: None,
            ev: None,
            mode: Mode::Standard,
            signal: Signal::Initializing,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        let expected = "[RUN:test-run | S:0 | R:OBS | CI:- | EV:- | M:STD | 🚦:Initializing]";
        assert_eq!(steno, expected);
        println!("✓ Test passed");
    }

    /// TC-CM-001-B: Mid-run with CI=0.87, EV=+3%
    #[test]
    fn tc_cm_001_b_mid_run() {
        println!("\n=== TC-CM-001-B: Mid-run with CI=0.87, EV=+3% ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 3,
            role: Role::Observer,
            ci: Some(0.87),
            ev: Some(3.0),
            mode: Mode::Standard,
            signal: Signal::ReadyForSynthesis,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        let expected =
            "[RUN:test-run | S:3 | R:OBS | CI:0.87 | EV:+3% | M:STD | 🚦:Ready_for_Synthesis]";
        assert_eq!(steno, expected);
        println!("✓ Test passed");
    }

    /// TC-CM-001-C: Conductor role
    #[test]
    fn tc_cm_001_c_conductor_role() {
        println!("\n=== TC-CM-001-C: Conductor role ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 2,
            role: Role::Conductor,
            ci: Some(0.75),
            ev: Some(5.0),
            mode: Mode::Standard,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("R:COND"));
        println!("✓ Test passed - Contains R:COND");
    }

    /// TC-CM-001-D: Auditor role
    #[test]
    fn tc_cm_001_d_auditor_role() {
        println!("\n=== TC-CM-001-D: Auditor role ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 5,
            role: Role::Auditor,
            ci: Some(0.92),
            ev: Some(-2.0),
            mode: Mode::Standard,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("R:AUD"));
        println!("✓ Test passed - Contains R:AUD");
    }

    /// TC-CM-001-E: Component mode
    #[test]
    fn tc_cm_001_e_component_mode() {
        println!("\n=== TC-CM-001-E: Component mode ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 3,
            role: Role::Fabricator,
            ci: Some(0.80),
            ev: Some(1.0),
            mode: Mode::Component,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("M:COMP"));
        println!("✓ Test passed - Contains M:COMP");
    }

    /// TC-CM-001-F: Surgical mode
    #[test]
    fn tc_cm_001_f_surgical_mode() {
        println!("\n=== TC-CM-001-F: Surgical mode ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 4,
            role: Role::Patcher,
            ci: Some(0.68),
            ev: Some(-5.0),
            mode: Mode::Surgical,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("M:SURG"));
        println!("✓ Test passed - Contains M:SURG");
    }

    /// Test all role abbreviations
    #[test]
    fn test_all_role_abbreviations() {
        println!("\n=== Test: All role abbreviations ===");

        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Observer),
            "OBS"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Conductor),
            "COND"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Auditor),
            "AUD"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Patcher),
            "PATCH"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Fabricator),
            "FAB"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Examiner),
            "EXAM"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Curator),
            "CUR"
        );
        assert_eq!(
            ContextManager::get_role_abbreviation(&Role::Archivist),
            "ARCH"
        );

        println!("✓ All 8 role abbreviations correct");
    }

    /// Test all mode abbreviations
    #[test]
    fn test_all_mode_abbreviations() {
        println!("\n=== Test: All mode abbreviations ===");

        assert_eq!(
            ContextManager::get_mode_abbreviation(&Mode::Standard),
            "STD"
        );
        assert_eq!(
            ContextManager::get_mode_abbreviation(&Mode::Component),
            "COMP"
        );
        assert_eq!(
            ContextManager::get_mode_abbreviation(&Mode::Surgical),
            "SURG"
        );

        println!("✓ All 3 mode abbreviations correct");
    }

    /// Test negative EV formatting
    #[test]
    fn test_negative_ev_formatting() {
        println!("\n=== Test: Negative EV formatting ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 2,
            role: Role::Observer,
            ci: Some(0.75),
            ev: Some(-15.0),
            mode: Mode::Standard,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("EV:-15%"));
        println!("✓ Negative EV formatted correctly");
    }

    /// Test positive EV with plus sign
    #[test]
    fn test_positive_ev_with_plus() {
        println!("\n=== Test: Positive EV with plus sign ===");

        let context = RunContext {
            run_id: "test-run".to_string(),
            step: 3,
            role: Role::Observer,
            ci: Some(0.90),
            ev: Some(25.0),
            mode: Mode::Standard,
            signal: Signal::Active,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        assert!(steno.contains("EV:+25%"));
        println!("✓ Positive EV formatted with + sign");
    }

    /// Test all signal states
    #[test]
    fn test_all_signal_states() {
        println!("\n=== Test: All signal states ===");

        let signals = vec![
            (Signal::Initializing, "Initializing"),
            (Signal::ReadyForSynthesis, "Ready_for_Synthesis"),
            (Signal::AwaitingGate, "Awaiting_Gate"),
            (Signal::Halted, "Halted"),
            (Signal::PausedForReview, "Paused_for_Review"),
            (Signal::Completed, "Completed"),
            (Signal::Active, "Active"),
        ];

        for (signal, expected_str) in signals {
            let context = RunContext {
                run_id: "test-run".to_string(),
                step: 1,
                role: Role::Observer,
                ci: Some(0.80),
                ev: Some(0.0),
                mode: Mode::Standard,
                signal,
            };

            let steno = ContextManager::generate_steno_ledger(&context);
            assert!(steno.contains(&format!("🚦:{}", expected_str)));
            println!("✓ Signal {:?} formatted correctly", expected_str);
        }

        println!("✓ All 7 signal states correct");
    }

    /// Test comprehensive steno-ledger generation
    #[test]
    fn test_comprehensive_steno_ledger() {
        println!("\n=== Test: Comprehensive Steno-Ledger generation ===");

        // Test with all fields populated
        let context = RunContext {
            run_id: "2025-12-17-Analysis".to_string(),
            step: 3,
            role: Role::Observer,
            ci: Some(0.87),
            ev: Some(3.0),
            mode: Mode::Standard,
            signal: Signal::ReadyForSynthesis,
        };

        let steno = ContextManager::generate_steno_ledger(&context);
        println!("Generated: {}", steno);

        // Verify all components are present
        assert!(steno.starts_with("[RUN:"));
        assert!(steno.contains("2025-12-17-Analysis"));
        assert!(steno.contains("S:3"));
        assert!(steno.contains("R:OBS"));
        assert!(steno.contains("CI:0.87"));
        assert!(steno.contains("EV:+3%"));
        assert!(steno.contains("M:STD"));
        assert!(steno.contains("🚦:Ready_for_Synthesis"));
        assert!(steno.ends_with("]"));

        println!("✓ Comprehensive test passed");
    }
}
