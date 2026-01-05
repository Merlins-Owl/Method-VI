//! # Governance Module - Phase 4 Complete ✅
//!
//! ## Phase 1: Callout System (Sessions 1.0-1.5)
//! - **CalloutTier**: Four severity levels (Info, Attention, Warning, Critical)
//! - **Callout**: Full metadata tracking with noise filter support
//! - **CalloutManager**: Add, acknowledge, can_proceed, summary methods
//! - **CalloutTrigger**: Metric-specific tier determination with step-aware logic
//! - **AcknowledgmentRecord**: Audit trail for Critical callout acknowledgments
//!
//! ## Phase 2: Mode Detection (Sessions 2.1-2.3)
//! - **ModeDetectionResult**: Metadata with mode, ci_baseline, confidence, signals, timestamp
//! - **ModeDetector**: Automatic detection with transparency logging (Constraint 1)
//! - **StructureMode**: Three modes (Architecting/Builder/Refining) based on CI baseline
//! - **Orchestrator integration**: Mode detection at Step 2, locked for run
//! - **Tauri commands**: get_current_mode, detect_mode for frontend integration
//!
//! ## Phase 3: Diagnostic Tolerance (Sessions 3.1-3.3, Constraint 2)
//! - **MetricEnforcement**: Enum controlling callout generation (Enforced/Informational)
//! - **determine_tier_with_enforcement()**: Mode-aware tier determination with enforcement
//! - **Step 3 baseline recording**: CI recorded without generating callouts
//! - **diagnostic_ci_baseline**: Stored in Orchestrator for Step 4+ delta calculation
//! - **Informational mode**: Step 3 metrics logged but no callouts generated
//! - **Delta calculation**: Step 4+ uses Step 3 baseline for CI delta measurement
//!
//! ## Phase 4: Orchestrator Integration (Sessions 4.1-4.4) ✅
//! - **CalloutManager in Orchestrator**: Added to orchestrator struct (Session 4.1)
//! - **Step 2 integration**: generate_callouts() called with Enforced mode (Session 4.1)
//! - **Steps 4-6 integration**: generate_callouts() called with diagnostic baseline delta (Session 4.2)
//! - **Delta from baseline**: Steps 4-6 use diagnostic_ci_baseline for CI delta calculation
//! - **HALT deprecation**: Old HALT logic deprecated, always returns None (Session 4.3)
//! - **End-to-end verification**: Full workflow tests validate all 4 constraints (Session 4.4)
//!
//! ## Core Constraints Implemented ✅
//! 1. **Transparency Mandate**: Mode detection logs structure level and mode
//! 2. **Delta Baseline Rule**: Step 3 CI recorded informationally for Step 4+ delta calculation
//! 3. **Noise Filter**: Warning callouts auto-downgrade to Attention in Architecting mode
//! 4. **Critical-Only Blocking**: Only Critical callouts require acknowledgment to proceed

pub mod callouts;
pub mod types;

#[cfg(test)]
mod integration_tests;

pub use callouts::*;
pub use types::*;
pub use types::{ModeDetectionResult, ModeDetector, MetricEnforcement};
