use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::api::anthropic::AnthropicClient;

/// Metric input - a value that contributed to the metric calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInput {
    pub name: String,
    pub value: MetricInputValue,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricInputValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Threshold values for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    pub pass: f64,
    pub warning: Option<f64>,
    pub halt: Option<f64>,
}

/// Metric status based on threshold comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    Pass,
    Warning,
    Fail,
}

/// Complete metric result with explainability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub value: f64,
    pub threshold: MetricThreshold,
    pub status: MetricStatus,
    pub inputs_used: Vec<MetricInput>,
    pub calculation_method: String,
    pub interpretation: String,
    pub recommendation: Option<String>,
}

/// All 6 critical metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalMetrics {
    pub ci: Option<MetricResult>,
    pub ev: Option<MetricResult>,
    pub ias: Option<MetricResult>,
    pub efi: Option<MetricResult>,
    pub sec: Option<MetricResult>,
    pub pci: Option<MetricResult>,
}

/// E_baseline state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EBaseline {
    pub value: f64,
    pub locked: bool,
    pub locked_at_step: Option<u8>,
    pub source: String,
}

/// Governance & Telemetry Agent
///
/// Responsible for:
/// - Calculating Critical 6 metrics at step completion
/// - E_baseline management and locking
/// - Threshold monitoring and intervention triggers
/// - Metrics explainability following the contract
pub struct GovernanceTelemetryAgent {
    /// Claude API client for metric calculations
    api_client: AnthropicClient,

    /// E_baseline (locked after Step 1)
    e_baseline: Option<EBaseline>,

    /// Threshold configuration
    thresholds: ThresholdsConfig,
}

/// Threshold configuration for all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    pub ci: MetricThreshold,
    pub ev: MetricThreshold,
    pub ias: MetricThreshold,
    pub efi: MetricThreshold,
    pub sec: MetricThreshold,
    pub pci: MetricThreshold,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            ci: MetricThreshold {
                pass: 0.80,
                warning: Some(0.70),
                halt: Some(0.50),
            },
            ev: MetricThreshold {
                pass: 10.0,
                warning: Some(20.0),
                halt: Some(30.0),
            },
            ias: MetricThreshold {
                pass: 0.80,
                warning: Some(0.70),
                halt: Some(0.50),
            },
            efi: MetricThreshold {
                pass: 95.0,
                warning: Some(90.0),
                halt: Some(80.0),
            },
            sec: MetricThreshold {
                pass: 100.0,
                warning: None,
                halt: None,
            },
            pci: MetricThreshold {
                pass: 0.90,
                warning: Some(0.85),
                halt: Some(0.70),
            },
        }
    }
}

impl GovernanceTelemetryAgent {
    /// Create a new Governance & Telemetry Agent
    pub fn new(api_key: String) -> Result<Self> {
        let api_client = AnthropicClient::new(api_key)?;

        Ok(Self {
            api_client,
            e_baseline: None,
            thresholds: ThresholdsConfig::default(),
        })
    }

    /// Calculate E_baseline from Baseline Report (Step 1)
    ///
    /// This calculates the baseline content size that will be used
    /// for EV (Expansion Variance) calculations throughout the run.
    pub fn calculate_e_baseline(&mut self, baseline_content: &str, step: u8) -> Result<f64> {
        if self.e_baseline.is_some() && self.e_baseline.as_ref().unwrap().locked {
            return Err(anyhow::anyhow!("E_baseline is already locked and cannot be recalculated"));
        }

        // Simple calculation: word count (can be enhanced later)
        let word_count = baseline_content
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .count() as f64;

        info!("Calculated E_baseline: {} words from baseline content", word_count);

        self.e_baseline = Some(EBaseline {
            value: word_count,
            locked: false,
            locked_at_step: None,
            source: "Baseline Report".to_string(),
        });

        Ok(word_count)
    }

    /// Lock E_baseline (Step 1 completion)
    ///
    /// Once locked, E_baseline becomes immutable for the rest of the run.
    pub fn lock_e_baseline(&mut self, step: u8) -> Result<()> {
        if let Some(ref mut baseline) = self.e_baseline {
            if baseline.locked {
                warn!("E_baseline already locked at step {}", baseline.locked_at_step.unwrap_or(0));
                return Ok(());
            }

            baseline.locked = true;
            baseline.locked_at_step = Some(step);
            info!("E_baseline locked at step {} with value: {}", step, baseline.value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot lock E_baseline: not yet calculated"))
        }
    }

    /// Get current E_baseline value
    pub fn get_e_baseline(&self) -> Option<f64> {
        self.e_baseline.as_ref().map(|b| b.value)
    }

    /// Calculate all 6 critical metrics for step completion
    pub async fn calculate_metrics(
        &self,
        content: &str,
        charter_objectives: &str,
        step: u8,
    ) -> Result<CriticalMetrics> {
        info!("Calculating Critical 6 metrics for step {}", step);

        // Calculate each metric
        let ci = self.calculate_ci(content).await?;
        let ev = self.calculate_ev(content)?;
        let ias = self.calculate_ias(content, charter_objectives).await?;
        let efi = self.calculate_efi(content, step).await?;
        let sec = self.calculate_sec()?;
        let pci = self.calculate_pci(content, step).await?;

        Ok(CriticalMetrics {
            ci: Some(ci),
            ev: Some(ev),
            ias: Some(ias),
            efi: Some(efi),
            sec: Some(sec),
            pci: Some(pci),
        })
    }

    /// Calculate CI (Coherence Index)
    ///
    /// Analyzes content for structural coherence, term consistency, and logical flow.
    /// Returns 0.0-1.0 score.
    async fn calculate_ci(&self, content: &str) -> Result<MetricResult> {
        debug!("Calculating CI (Coherence Index)");

        let system_prompt = "You are a coherence analysis expert for Method-VI governance. \
            Analyze content for structural coherence, term consistency, and logical flow. \
            Return ONLY a JSON object with this exact structure: \
            {\"score\": <number 0-1>, \"reasoning\": \"<explanation>\"}";

        let user_message = format!(
            "Analyze this content for coherence:\n\n{}\n\n\
            Evaluate:\n\
            1. Structural coherence (organization, flow)\n\
            2. Term consistency (uniform terminology)\n\
            3. Logical flow (ideas connect properly)\n\n\
            Return score 0.0-1.0 and brief reasoning.",
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024))
            .await?;

        // Parse JSON response - extract JSON if embedded in text
        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse CI response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let score = parsed["score"]
            .as_f64()
            .context("Missing or invalid 'score' field in CI response")?;

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let status = self.evaluate_status(score, &self.thresholds.ci, false);

        Ok(MetricResult {
            metric_name: "CI".to_string(),
            value: score,
            threshold: self.thresholds.ci.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Content Length".to_string(),
                    value: MetricInputValue::Number(content.len() as f64),
                    source: "Current Content".to_string(),
                },
            ],
            calculation_method: "LLM-based analysis of structural coherence, term consistency, and logical flow".to_string(),
            interpretation: reasoning,
            recommendation: if status != MetricStatus::Pass {
                Some("Review content structure and ensure consistent terminology throughout.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate EV (Expansion Variance)
    ///
    /// Compares current content size to E_baseline.
    /// Formula: |E_current - E_baseline| / E_baseline × 100
    fn calculate_ev(&self, content: &str) -> Result<MetricResult> {
        debug!("Calculating EV (Expansion Variance)");

        let e_current = content
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .count() as f64;

        let e_baseline = self.e_baseline
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("E_baseline not set"))?
            .value;

        // Formula: |E_current - E_baseline| / E_baseline × 100
        let variance = ((e_current - e_baseline).abs() / e_baseline) * 100.0;

        let status = self.evaluate_status(variance, &self.thresholds.ev, true); // Inverse: lower is better

        Ok(MetricResult {
            metric_name: "EV".to_string(),
            value: variance,
            threshold: self.thresholds.ev.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "E_current".to_string(),
                    value: MetricInputValue::Number(e_current),
                    source: "Current Content".to_string(),
                },
                MetricInput {
                    name: "E_baseline".to_string(),
                    value: MetricInputValue::Number(e_baseline),
                    source: "Baseline Report".to_string(),
                },
            ],
            calculation_method: format!(
                "|E_current - E_baseline| / E_baseline × 100 = |{} - {}| / {} × 100 = {:.2}%",
                e_current, e_baseline, e_baseline, variance
            ),
            interpretation: format!(
                "Content has {}% variance from baseline ({}±10% is target). Current: {} words, Baseline: {} words.",
                variance.round(),
                if variance < 10.0 { "PASS" } else if variance < 20.0 { "WARNING" } else { "FAIL" },
                e_current.round(),
                e_baseline.round()
            ),
            recommendation: if status != MetricStatus::Pass {
                Some("Content size deviates significantly from baseline. Review scope creep or compression.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate IAS (Intent Alignment Score)
    ///
    /// Compares current content against Charter objectives.
    /// Returns 0.0-1.0 score.
    async fn calculate_ias(&self, content: &str, charter_objectives: &str) -> Result<MetricResult> {
        debug!("Calculating IAS (Intent Alignment Score)");

        let system_prompt = "You are an intent alignment analyzer for Method-VI governance. \
            Compare content against Charter objectives to measure alignment. \
            Return ONLY a JSON object with this exact structure: \
            {\"score\": <number 0-1>, \"reasoning\": \"<explanation>\"}";

        let user_message = format!(
            "Compare this content against Charter objectives:\n\n\
            CHARTER OBJECTIVES:\n{}\n\n\
            CURRENT CONTENT:\n{}\n\n\
            Return alignment score 0.0-1.0 and brief reasoning.",
            charter_objectives,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024))
            .await?;

        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse IAS response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let score = parsed["score"]
            .as_f64()
            .context("Missing or invalid 'score' field in IAS response")?;

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let status = self.evaluate_status(score, &self.thresholds.ias, false);

        Ok(MetricResult {
            metric_name: "IAS".to_string(),
            value: score,
            threshold: self.thresholds.ias.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Charter Objectives".to_string(),
                    value: MetricInputValue::String(charter_objectives.to_string()),
                    source: "Charter".to_string(),
                },
            ],
            calculation_method: "LLM-based comparison of content against Charter objectives".to_string(),
            interpretation: reasoning,
            recommendation: if status != MetricStatus::Pass {
                Some("Review content alignment with Charter objectives. Consider refocusing on original intent.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate EFI (Execution Fidelity Index)
    ///
    /// Audits claims for evidence support.
    /// Returns percentage of substantiated claims.
    async fn calculate_efi(&self, content: &str, step: u8) -> Result<MetricResult> {
        debug!("Calculating EFI (Execution Fidelity Index)");

        let system_prompt = "You are an execution fidelity auditor for Method-VI governance. \
            Count claims in content and determine what percentage have supporting evidence. \
            Return ONLY a JSON object with this exact structure: \
            {\"percentage\": <number 0-100>, \"total_claims\": <number>, \"substantiated_claims\": <number>, \"reasoning\": \"<explanation>\"}";

        let user_message = format!(
            "Audit this content for execution fidelity:\n\n{}\n\n\
            Count:\n\
            1. Total claims made\n\
            2. Claims with supporting evidence\n\
            Return percentage substantiated (0-100).",
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024))
            .await?;

        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse EFI response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let percentage = parsed["percentage"]
            .as_f64()
            .context("Missing or invalid 'percentage' field in EFI response")?;

        let total_claims = parsed["total_claims"]
            .as_u64()
            .unwrap_or(0);

        let substantiated = parsed["substantiated_claims"]
            .as_u64()
            .unwrap_or(0);

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let status = self.evaluate_status(percentage, &self.thresholds.efi, false);

        Ok(MetricResult {
            metric_name: "EFI".to_string(),
            value: percentage,
            threshold: self.thresholds.efi.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Total Claims".to_string(),
                    value: MetricInputValue::Number(total_claims as f64),
                    source: "Current Content".to_string(),
                },
                MetricInput {
                    name: "Substantiated Claims".to_string(),
                    value: MetricInputValue::Number(substantiated as f64),
                    source: "Current Content".to_string(),
                },
            ],
            calculation_method: format!(
                "{} substantiated / {} total claims × 100 = {:.1}%",
                substantiated, total_claims, percentage
            ),
            interpretation: reasoning,
            recommendation: if status != MetricStatus::Pass {
                Some("Add evidence and citations to support claims. Increase rigor of substantiation.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate SEC (Scope Expansion Count)
    ///
    /// Counts approved vs undocumented scope changes.
    /// Returns compliance percentage.
    fn calculate_sec(&self) -> Result<MetricResult> {
        debug!("Calculating SEC (Scope Expansion Count)");

        // MVP: Always returns 100 (no scope expansions yet)
        // TODO: Track scope expansions in ledger and count them here
        let approved_expansions = 0;
        let undocumented_expansions = 0;
        let total_expansions = approved_expansions + undocumented_expansions;

        let compliance_percentage = if total_expansions == 0 {
            100.0 // No expansions = perfect compliance
        } else {
            (approved_expansions as f64 / total_expansions as f64) * 100.0
        };

        let status = self.evaluate_status(compliance_percentage, &self.thresholds.sec, false);

        Ok(MetricResult {
            metric_name: "SEC".to_string(),
            value: compliance_percentage,
            threshold: self.thresholds.sec.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Approved Scope Expansions".to_string(),
                    value: MetricInputValue::Number(approved_expansions as f64),
                    source: "Ledger".to_string(),
                },
                MetricInput {
                    name: "Undocumented Scope Expansions".to_string(),
                    value: MetricInputValue::Number(undocumented_expansions as f64),
                    source: "Ledger".to_string(),
                },
            ],
            calculation_method: format!(
                "{} approved / {} total scope changes × 100 = {:.0}%",
                approved_expansions, total_expansions, compliance_percentage
            ),
            interpretation: format!(
                "Scope compliance: {}%. {} approved, {} undocumented expansions.",
                compliance_percentage.round(),
                approved_expansions,
                undocumented_expansions
            ),
            recommendation: if status != MetricStatus::Pass {
                Some("Document all scope expansions and seek approval before proceeding.".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate PCI (Pattern Consistency Index)
    ///
    /// Checks adherence to Architecture Map.
    /// Returns 0.0-1.0 score.
    async fn calculate_pci(&self, content: &str, step: u8) -> Result<MetricResult> {
        debug!("Calculating PCI (Pattern Consistency Index)");

        let system_prompt = "You are a process compliance auditor for Method-VI governance. \
            Check if content follows Method-VI Architecture Map and process rules. \
            Return ONLY a JSON object with this exact structure: \
            {\"score\": <number 0-1>, \"reasoning\": \"<explanation>\"}";

        let user_message = format!(
            "Audit this Step {} content for process compliance:\n\n{}\n\n\
            Check:\n\
            1. Follows Method-VI structure\n\
            2. Contains required sections\n\
            3. Adheres to governance rules\n\
            Return compliance score 0.0-1.0 and brief reasoning.",
            step,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1024))
            .await?;

        let parsed: serde_json::Value = self.extract_json(&response)
            .context(format!("Failed to parse PCI response as JSON. Raw response: {}", &response[..response.len().min(200)]))?;

        let score = parsed["score"]
            .as_f64()
            .context("Missing or invalid 'score' field in PCI response")?;

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let status = self.evaluate_status(score, &self.thresholds.pci, false);

        Ok(MetricResult {
            metric_name: "PCI".to_string(),
            value: score,
            threshold: self.thresholds.pci.clone(),
            status: status.clone(),
            inputs_used: vec![
                MetricInput {
                    name: "Current Step".to_string(),
                    value: MetricInputValue::Number(step as f64),
                    source: "Orchestrator".to_string(),
                },
            ],
            calculation_method: "LLM-based audit of adherence to Method-VI Architecture Map".to_string(),
            interpretation: reasoning,
            recommendation: if status != MetricStatus::Pass {
                Some("Review Architecture Map and ensure all process rules are followed.".to_string())
            } else {
                None
            },
        })
    }

    /// Evaluate metric status based on threshold
    ///
    /// # Arguments
    /// * `value` - The metric value
    /// * `threshold` - The threshold configuration
    /// * `inverse_scale` - If true, lower values are better (like EV)
    fn evaluate_status(
        &self,
        value: f64,
        threshold: &MetricThreshold,
        inverse_scale: bool,
    ) -> MetricStatus {
        if inverse_scale {
            // Lower is better (e.g., EV)
            if value <= threshold.pass {
                MetricStatus::Pass
            } else if let Some(warning) = threshold.warning {
                if value <= warning {
                    MetricStatus::Warning
                } else {
                    MetricStatus::Fail
                }
            } else {
                MetricStatus::Fail
            }
        } else {
            // Higher is better (e.g., CI, IAS, EFI, PCI)
            if value >= threshold.pass {
                MetricStatus::Pass
            } else if let Some(warning) = threshold.warning {
                if value >= warning {
                    MetricStatus::Warning
                } else {
                    MetricStatus::Fail
                }
            } else {
                MetricStatus::Fail
            }
        }
    }

    /// Check if any metrics require HALT
    ///
    /// # Arguments
    /// * `metrics` - The metrics to check
    /// * `step` - Current step (0-6) - EFI only enforced at Step 6 per spec §9.1.4
    pub fn check_halt_conditions(&self, metrics: &CriticalMetrics, step: u8) -> Option<String> {
        let mut halt_reasons = Vec::new();

        if let Some(ref ci) = metrics.ci {
            if ci.status == MetricStatus::Fail {
                halt_reasons.push(format!("CI critically low: {:.2}", ci.value));
            }
        }

        if let Some(ref ev) = metrics.ev {
            if ev.status == MetricStatus::Fail {
                halt_reasons.push(format!("EV exceeded limits: {:.1}%", ev.value));
            }
        }

        if let Some(ref ias) = metrics.ias {
            if ias.status == MetricStatus::Fail {
                halt_reasons.push(format!("IAS critically low: {:.2}", ias.value));
            }
        }

        // FIX-008: EFI should only trigger HALT at Step 6 (per spec §9.1.4)
        // Early steps (0-5) may have low EFI (e.g., Charter is governance, not evidence)
        // but this shouldn't block progression before validation
        if step == 6 {
            if let Some(ref efi) = metrics.efi {
                if efi.status == MetricStatus::Fail {
                    halt_reasons.push(format!("EFI critically low: {:.1}%", efi.value));
                }
            }
        }

        if let Some(ref pci) = metrics.pci {
            if pci.status == MetricStatus::Fail {
                halt_reasons.push(format!("PCI critically low: {:.2}", pci.value));
            }
        }

        if !halt_reasons.is_empty() {
            Some(format!("HALT: Critical metrics failed: {}", halt_reasons.join(", ")))
        } else {
            None
        }
    }

    /// Check if any metrics require PAUSE (warning status)
    pub fn check_pause_conditions(&self, metrics: &CriticalMetrics) -> Option<String> {
        let mut warning_reasons = Vec::new();

        if let Some(ref ci) = metrics.ci {
            if ci.status == MetricStatus::Warning {
                warning_reasons.push(format!("CI below target: {:.2}", ci.value));
            }
        }

        if let Some(ref ev) = metrics.ev {
            if ev.status == MetricStatus::Warning {
                warning_reasons.push(format!("EV approaching limit: {:.1}%", ev.value));
            }
        }

        if let Some(ref ias) = metrics.ias {
            if ias.status == MetricStatus::Warning {
                warning_reasons.push(format!("IAS below target: {:.2}", ias.value));
            }
        }

        if let Some(ref efi) = metrics.efi {
            if efi.status == MetricStatus::Warning {
                warning_reasons.push(format!("EFI below target: {:.1}%", efi.value));
            }
        }

        if let Some(ref pci) = metrics.pci {
            if pci.status == MetricStatus::Warning {
                warning_reasons.push(format!("PCI below target: {:.2}", pci.value));
            }
        }

        if !warning_reasons.is_empty() {
            Some(format!("PAUSE: Metrics need attention: {}", warning_reasons.join(", ")))
        } else {
            None
        }
    }

    /// Extract JSON from Claude's response, handling cases where JSON is embedded in text
    ///
    /// Claude sometimes returns JSON with explanatory text before/after.
    /// This method tries to find and extract just the JSON portion.
    fn extract_json(&self, response: &str) -> Result<serde_json::Value> {
        // First try direct parsing
        if let Ok(parsed) = serde_json::from_str(response) {
            return Ok(parsed);
        }

        // Try to find JSON object markers
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str(json_str) {
                    return Ok(parsed);
                }
            }
        }

        // If all else fails, return original error
        Err(anyhow::anyhow!("Could not extract valid JSON from response"))
    }

    /// Create Baseline_Report artifact (Step 1)
    ///
    /// Creates the immutable Baseline Report that locks E_baseline and defines
    /// governance checkpoints for the run.
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `charter_content` - Content from the Charter artifact
    /// * `charter_id` - ID of the Charter artifact
    /// * `charter_hash` - Hash of the Charter artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `e_baseline` - The calculated E_baseline value
    /// * `telemetry_profile` - Telemetry profile (Lite / Standard / Full / Learning)
    pub fn create_baseline_report(
        &self,
        run_id: &str,
        charter_content: &str,
        charter_id: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
        telemetry_profile: &str,
    ) -> Result<String> {
        info!("Creating Baseline_Report for run {}", run_id);

        // Extract charter objectives and success criteria counts
        let objectives_count = charter_content
            .lines()
            .filter(|line| line.trim().starts_with("###") || line.trim().starts_with("1.") || line.trim().starts_with("2."))
            .count();

        let success_criteria_count = charter_content
            .lines()
            .filter(|line| line.contains("Criterion"))
            .count()
            .max(2); // At least 2 (CI and EV are always included)

        // Build the Baseline Report content
        let content_body = format!(
            r#"# Baseline Report

> 📊 **IMMUTABLE ARTIFACT** - E_baseline is locked at this point.
> All future EV calculations reference these values.

## Baseline Freeze Confirmation

| Parameter | Value | Locked |
|-----------|-------|--------|
| E_baseline | {} | ✓ |
| Charter Objectives | {} | ✓ |
| Success Criteria | {} | ✓ |

## E_baseline Calculation

### Input Materials

| Material | Type | Size/Scope |
|----------|------|------------|
| Charter | Governance Document | {} words |

### Calculation Method

```
E_baseline = Word count of Charter content
           = {}
```

### Baseline Components

| Component | Weight | Value | Contribution |
|-----------|--------|-------|--------------|
| Charter Content | 100% | {} | **{}** |

## Threshold Canon Alignment

### Critical 6 Targets

| Metric | Target | Warning | HALT |
|--------|--------|---------|------|
| CI | ≥ 0.80 | 0.70 | 0.50 |
| EV | ≤ ±10% | ±20% | ±30% |
| IAS | ≥ 0.80 | 0.70 | 0.50 |
| EFI | ≥ 95% | 90% | 80% |
| SEC | 100% | - | - |
| PCI | ≥ 0.90 | 0.85 | 0.70 |

### Telemetry Profile Configuration

**Selected Profile:** {}

| Setting | Value |
|---------|-------|
| Metric Frequency | per step |
| Domain Monitoring | enabled |
| Learning Capture | {} |

## Governance Checkpoint Registry

| Checkpoint | Step | Gate Required |
|------------|------|---------------|
| Intent Confirmed | 0→1 | ✓ |
| Baseline Frozen | 1→2 | ✓ |
| Analysis Ready | 2→3 | ✓ |
| Synthesis Ready | 3→4 | ✓ |
| Redesign Ready | 4→5 | ✓ |
| Validation Ready | 5→6 | ✓ |
| Completion | 6→Close | ✓ |

## Human Approval

- **Baseline Frozen By:** System
- **Approval Timestamp:** {}
- **Gate:** Baseline_Frozen

---
📊 **Baseline Hash:** `{}`
*E_baseline = {} is now immutable*"#,
            e_baseline,
            objectives_count,
            success_criteria_count,
            e_baseline,
            e_baseline,
            e_baseline,
            e_baseline,
            telemetry_profile,
            if telemetry_profile == "Learning" { "enabled" } else { "disabled" },
            chrono::Utc::now().to_rfc3339(),
            charter_hash,
            e_baseline
        );

        // Calculate hash of content body
        let content_hash = self.compute_content_hash(&content_body);

        // Build complete artifact with frontmatter
        let artifact_id = format!("{}-baseline-report", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Baseline_Report\"\n\
            run_id: \"{}\"\n\
            step_origin: 1\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"derived_from\"\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: true\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            charter_id,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Baseline_Report created successfully: {}", artifact_id);
        Ok(artifact)
    }

    /// Compute SHA-256 hash of content
    fn compute_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Perform governance calibration for Step 2
    ///
    /// Reviews Charter objectives and configures five control domains:
    /// - Entropy Control: Set acceptable EV bounds
    /// - Objective Control: Align metrics to Charter goals
    /// - Process Control: Confirm step sequencing
    /// - Reflective Control: Set reflection cadence
    /// - Termination Control: Define completion criteria
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `charter_content` - Content from the Charter artifact
    /// * `charter_hash` - Hash of the Charter artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `architecture_map_content` - Content from the Architecture Map
    /// * `e_baseline` - The locked E_baseline value
    ///
    /// # Returns
    /// Tuple of (Governance_Summary artifact, Domain_Snapshots artifact)
    pub async fn perform_governance_calibration(
        &self,
        run_id: &str,
        charter_content: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        architecture_map_content: &str,
        e_baseline: f64,
    ) -> Result<(String, String)> {
        info!("Performing governance calibration for run {}", run_id);

        let system_prompt = "You are the Governance & Telemetry Agent under the CONDUCTOR role. \
            Your task is to review the Charter and configure active governance controls for this Method-VI run. \
            Be specific and concrete about thresholds, triggers, and intervention criteria.";

        let user_message = format!(
            r#"GOVERNANCE CALIBRATION (Step 2)

Charter Content:
{}

Architecture Map:
{}

E_baseline (locked): {} words

Your task: Configure the five control domains for active governance orchestration.

For each domain, specify:
1. Control objective (what we're monitoring)
2. Threshold settings (when to intervene)
3. Intervention triggers (what causes an alert)
4. Calibration rationale (why these settings)

## 1. ENTROPY CONTROL
Objective: Manage content expansion/contraction relative to E_baseline
Default bounds: ±10% variance acceptable
- What EV bounds are appropriate for this run's scope?
- What are the warning thresholds (yellow) and HALT thresholds (red)?
- Rationale:

## 2. OBJECTIVE CONTROL
Objective: Ensure work remains aligned with Charter objectives
Metrics: IAS (Intent Alignment Score)
- What IAS threshold indicates good alignment? (default: ≥0.82)
- When should we warn about drift from objectives?
- Rationale:

## 3. PROCESS CONTROL
Objective: Confirm adherence to Method-VI step sequencing
Metrics: PCI (Process Compliance Index)
- What PCI threshold indicates proper process adherence? (default: ≥0.90)
- What process violations should trigger intervention?
- Rationale:

## 4. REFLECTIVE CONTROL
Objective: Maintain appropriate reflection cadence per Architecture Map
Metrics: RCC (Reflection Cadence Compliance)
- What is the planned reflection schedule based on the Architecture Map?
- How do we detect deviations from this schedule?
- Rationale:

## 5. TERMINATION CONTROL
Objective: Define clear completion criteria
- What conditions signal readiness to complete this run?
- What are the minimum acceptable metric values for closure? (CI, EV, IAS, etc.)
- What are the "good enough" vs "excellent" criteria?
- Rationale:

Return your calibration settings in a structured format."#,
            charter_content,
            architecture_map_content,
            e_baseline
        );

        // Call Claude API for governance calibration
        let calibration_response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(3000))
            .await
            .context("Failed to generate governance calibration")?;

        info!("Governance calibration response received");

        // Create Governance_Summary artifact
        let governance_summary = self.create_governance_summary_artifact(
            run_id,
            &calibration_response,
            charter_hash,
            intent_anchor_id,
            e_baseline,
        )?;

        // Create Domain_Snapshots artifact (baseline readings)
        let domain_snapshots = self.create_domain_snapshots_artifact(
            run_id,
            charter_hash,
            intent_anchor_id,
            e_baseline,
        )?;

        info!("Governance calibration complete");

        Ok((governance_summary, domain_snapshots))
    }

    /// Create Governance_Summary artifact
    fn create_governance_summary_artifact(
        &self,
        run_id: &str,
        calibration_content: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
    ) -> Result<String> {
        info!("Creating Governance_Summary artifact");

        let content_body = format!(
            r#"# Governance Summary

> 📊 **GOVERNANCE CALIBRATION** - Active governance controls configured for this run.
> These settings guide continuous monitoring and minimal intervention.

## Control Domain Configuration

{}

## E_baseline Reference

| Parameter | Value | Status |
|-----------|-------|--------|
| E_baseline | {} words | Locked ✓ |
| Acceptable EV | ±10% default | Configured ✓ |

## Threshold Canon Application

All thresholds aligned with Method-VI Threshold Canon:

| Metric | Pass | Warning | HALT |
|--------|------|---------|------|
| CI | ≥ 0.80 | 0.70 | 0.50 |
| EV | ≤ ±10% | ±20% | ±30% |
| IAS | ≥ 0.80 | 0.70 | 0.50 |
| EFI | ≥ 95% | 90% | 80% |
| SEC | 100% | - | - |
| PCI | ≥ 0.90 | 0.85 | 0.70 |

## Active Governance Mode

**Status:** ENABLED
**Monitoring Frequency:** Per step completion
**Intervention Strategy:** Minimal intervention (proportional response)

## Governance Role

**Active Role:** Conductor
**Activation:** Step 2 (Governance Calibration)
**Deactivation:** End of Step 2 (→ Observer role reactivated)

---
📊 **Calibration Complete**
*Governance controls active for duration of run*"#,
            calibration_content,
            e_baseline
        );

        let content_hash = self.compute_content_hash(&content_body);
        let artifact_id = format!("{}-governance-summary", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Governance_Summary\"\n\
            run_id: \"{}\"\n\
            step_origin: 2\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: false\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Governance_Summary created: {}", artifact_id);
        Ok(artifact)
    }

    /// Create Domain_Snapshots artifact (baseline readings for five domains)
    fn create_domain_snapshots_artifact(
        &self,
        run_id: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        e_baseline: f64,
    ) -> Result<String> {
        info!("Creating Domain_Snapshots artifact");

        let content_body = format!(
            r#"# Domain Snapshots

> 📸 **BASELINE READINGS** - Initial state of five control domains at Step 2.
> These snapshots provide reference points for detecting drift and triggering interventions.

## Domain Baseline Readings

### 1. Clarity Domain
- **Metric:** CI (Coherence Index)
- **Baseline Reading:** Not yet measured (Step 3+)
- **Target:** ≥ 0.82
- **Status:** Monitoring configured ✓

### 2. Entropy Domain
- **Metric:** EV (Expansion Variance)
- **Baseline Reading:** 0.0% (at E_baseline = {} words)
- **Target:** ≤ ±10%
- **Status:** Monitoring configured ✓

### 3. Alignment Domain
- **Metric:** IAS (Intent Alignment Score)
- **Baseline Reading:** Not yet measured (Step 3+)
- **Target:** ≥ 0.82
- **Status:** Monitoring configured ✓

### 4. Cadence Domain
- **Metric:** RCC (Reflection Cadence Compliance)
- **Baseline Reading:** On schedule (Step 2 complete)
- **Target:** Per Architecture Map
- **Status:** Monitoring configured ✓

### 5. Overhead Domain
- **Metric:** GLR (Governance Latency Ratio)
- **Baseline Reading:** Not yet measured (Phase 2 metric)
- **Target:** ≤ 15%
- **Status:** Deferred to Phase 2

## Snapshot Metadata

| Field | Value |
|-------|-------|
| Snapshot Time | {} |
| Run ID | {} |
| Step | 2 (Governance Calibration) |
| E_baseline | {} words |

## Monitoring Status

All five domains are configured for continuous monitoring throughout the run.
Snapshots will be updated at each step completion.

---
📸 **Domain Monitoring Active**
*Baseline snapshots recorded for reference*"#,
            e_baseline,
            chrono::Utc::now().to_rfc3339(),
            run_id,
            e_baseline
        );

        let content_hash = self.compute_content_hash(&content_body);
        let artifact_id = format!("{}-domain-snapshots", run_id);
        let created_at = chrono::Utc::now().to_rfc3339();

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Domain_Snapshots\"\n\
            run_id: \"{}\"\n\
            step_origin: 2\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"constrained_by\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: false\n\
            author: \"governance-telemetry-agent\"\n\
            governance_role: \"Conductor\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Domain_Snapshots created: {}", artifact_id);
        Ok(artifact)
    }

    /// Check synthesis relevance before Step 4 synthesis
    ///
    /// Validates that the integrated diagnostic from Step 3 actually relates to
    /// the Charter objectives. This prevents synthesizing based on wrong analysis target
    /// (e.g., analyzing Charter methodology instead of user's content).
    ///
    /// Returns a relevance score from 0.0 to 1.0:
    /// - 1.0 = Findings directly address all objectives
    /// - 0.7 = Findings mostly relevant with some tangential content
    /// - 0.5 = Findings partially relevant
    /// - 0.3 = Findings mostly unrelated
    /// - 0.0 = Findings completely unrelated
    pub async fn check_synthesis_relevance(
        &self,
        diagnostic: &str,
        charter_objectives: &[String],
    ) -> Result<f64> {
        info!("Checking synthesis relevance...");

        let system_prompt = "You are a quality assurance checker for Method-VI. \
            Assess whether analysis findings relate to the stated objectives.";

        let objectives_text = charter_objectives.join("\n- ");

        let user_message = format!(
            r#"RELEVANCE CHECK

Charter Objectives:
- {}

Analysis Findings (from Integrated Diagnostic):
{}

Question: Do these analysis findings directly address or inform these objectives?

Score from 0.0 to 1.0:
- 1.0 = Findings directly address all objectives
- 0.7 = Findings mostly relevant with some tangential content
- 0.5 = Findings partially relevant
- 0.3 = Findings mostly unrelated
- 0.0 = Findings completely unrelated (e.g., analyzing methodology instead of content)

WARNING SIGNS of wrong analysis target:
- Findings critique "project management methodology" or "governance approach"
- References to "category error", "sophistication paradox", "planning primacy"
- Meta-analysis of the Charter document itself rather than user's content
- Findings discuss Method-VI framework instead of user's subject matter

Respond with ONLY a JSON object:
{{"score": 0.XX, "rationale": "brief explanation"}}"#,
            objectives_text,
            &diagnostic[..diagnostic.len().min(3000)]  // Truncate if too long
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(300))
            .await?;

        // Parse score from response using existing extract_json helper
        let json = self.extract_json(&response)?;

        let score = json["score"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse relevance score from response"))?;

        let rationale = json["rationale"]
            .as_str()
            .unwrap_or("No rationale provided");

        info!("Synthesis relevance score: {:.2} - {}", score, rationale);

        Ok(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_evaluation() {
        let thresholds = ThresholdsConfig::default();
        let agent = GovernanceTelemetryAgent {
            api_client: AnthropicClient::new("test-key".to_string()).unwrap(),
            e_baseline: None,
            thresholds,
        };

        // Test CI (higher is better)
        assert_eq!(
            agent.evaluate_status(0.85, &agent.thresholds.ci, false),
            MetricStatus::Pass
        );
        assert_eq!(
            agent.evaluate_status(0.75, &agent.thresholds.ci, false),
            MetricStatus::Warning
        );
        assert_eq!(
            agent.evaluate_status(0.45, &agent.thresholds.ci, false),
            MetricStatus::Fail
        );

        // Test EV (lower is better - inverse scale)
        assert_eq!(
            agent.evaluate_status(8.0, &agent.thresholds.ev, true),
            MetricStatus::Pass
        );
        assert_eq!(
            agent.evaluate_status(15.0, &agent.thresholds.ev, true),
            MetricStatus::Warning
        );
        assert_eq!(
            agent.evaluate_status(35.0, &agent.thresholds.ev, true),
            MetricStatus::Fail
        );
    }

    #[test]
    fn test_e_baseline_locking() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "test-key".to_string());
        let mut agent = GovernanceTelemetryAgent::new(api_key).unwrap();

        // Calculate baseline
        let baseline_content = "This is a test baseline with some words in it.";
        let result = agent.calculate_e_baseline(baseline_content, 1);
        assert!(result.is_ok());

        let e_baseline = agent.get_e_baseline();
        assert!(e_baseline.is_some());
        assert!(e_baseline.unwrap() > 0.0);

        // Lock baseline
        let lock_result = agent.lock_e_baseline(1);
        assert!(lock_result.is_ok());

        // Verify locked
        assert!(agent.e_baseline.as_ref().unwrap().locked);

        // Try to recalculate - should fail
        let recalc_result = agent.calculate_e_baseline("New content", 2);
        assert!(recalc_result.is_err());
    }
}
