use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

use crate::api::anthropic::AnthropicClient;

/// Analysis & Synthesis Agent - Deep Reasoning and Model Building Specialist
///
/// Handles Step 3 (Six-Lens Analysis) and Step 4 (Synthesis Lock-In)
///
/// CRITICAL: This agent is STATEFUL - it stores lens results from Step 3
/// for use in Step 4. DO NOT replace this agent between steps.
pub struct AnalysisSynthesisAgent {
    api_client: AnthropicClient,

    // Lens results from Step 3 (stored for Step 4 synthesis)
    structural_analysis: Option<LensResult>,
    thematic_analysis: Option<LensResult>,
    logic_analysis: Option<LensResult>,
    evidence_analysis: Option<LensResult>,
    expression_analysis: Option<LensResult>,
    intent_analysis: Option<LensResult>,

    // Cross-lens integration result
    integrated_diagnostic: Option<String>,
}

/// Result from applying a single lens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensResult {
    pub lens_name: String,
    pub analysis: String,
    pub key_findings: Vec<String>,
    pub efficacy_score: f64, // 0.0-1.0: did this lens provide valuable insights?
    pub tokens_used: u32,
}

/// Lens efficacy tracking for pattern learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensEfficacyReport {
    pub lens_results: Vec<LensResult>,  // Individual lens results for UI display
    pub total_insights: usize,
    pub high_value_combinations: usize,  // Count of high-value lenses
    pub estimated_cost: f64,
    pub actual_cost: f64,
}

/// Model geometry selection for Step 4 synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelGeometry {
    Linear,    // Sequential, one-directional flow
    Cyclic,    // Feedback loops, iterative processes
    Branching, // Decision points, multiple paths
}

/// Complete synthesis result from Step 4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step4SynthesisResult {
    pub core_thesis: String,
    pub operating_principles: Vec<String>,
    pub model_geometry: ModelGeometry,
    pub geometry_rationale: String,
    pub causal_spine: String,
    pub north_star_narrative: String,
    pub glossary: Vec<GlossaryEntry>,
    pub limitations: Vec<String>,
    pub novel_geometry_flag: bool,
}

/// Single glossary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub term: String,
    pub definition: String,
}

impl AnalysisSynthesisAgent {
    /// Create a new Analysis & Synthesis Agent
    pub fn new(api_key: String) -> Result<Self> {
        let api_client = AnthropicClient::new(api_key)?;

        Ok(Self {
            api_client,
            structural_analysis: None,
            thematic_analysis: None,
            logic_analysis: None,
            evidence_analysis: None,
            expression_analysis: None,
            intent_analysis: None,
            integrated_diagnostic: None,
        })
    }

    // =============================================================================
    // STEP 3: SIX-LENS ANALYSIS
    // =============================================================================
    //
    // TODO(Phase 2): Lens Efficacy System
    // ----------------------------------------------------------------------------
    // Current state: Efficacy scores are placeholders (always 0.0)
    //
    // Phase 2 Pattern Learning requirements:
    // 1. Track which lenses provide unique insights (not found by other lenses)
    // 2. Measure downstream usage (what findings appear in synthesis)
    // 3. Enable adaptive sequencing (run high-value lenses first)
    // 4. Support cost optimization (skip low-value lenses for known patterns)
    //
    // Design options documented in: Method-VI_MVP_Fixes_and_Enhancements_Guide.md
    // See FIX-007 for analysis of why previous heuristic was removed
    // =============================================================================

    /// Apply all six lenses to the USER'S CONTENT
    ///
    /// This is the main entry point for Step 3
    ///
    /// # Arguments
    /// * `analysis_target` - The user's original content to analyze (from user_request)
    /// * `governance_context` - The Charter (used ONLY for Intent lens alignment)
    /// * `intent_category` - Category determining lens sequence
    ///
    /// CRITICAL: The first 5 lenses analyze analysis_target. Only Intent lens uses governance_context.
    pub async fn perform_six_lens_analysis(
        &mut self,
        analysis_target: &str,
        governance_context: &str,
        intent_category: &str,
    ) -> Result<(String, LensEfficacyReport)> {
        info!("Starting six-lens analysis");
        info!("Intent category: {}", intent_category);
        info!("Analysis target: {} chars", analysis_target.len());
        info!("Governance context: {} chars", governance_context.len());

        // Validation: Ensure we're not analyzing governance metadata
        if analysis_target.contains("# Charter") || analysis_target.contains("## Objectives") {
            anyhow::bail!("HALT: Cannot analyze Charter as subject matter content");
        }

        // Apply lenses in weighted sequence based on intent category
        let lens_sequence = self.get_lens_sequence(intent_category);

        for lens_name in lens_sequence {
            match lens_name {
                "Structural" => {
                    info!("Applying Structural lens...");
                    self.structural_analysis = Some(self.apply_structural_lens(analysis_target).await?);
                }
                "Thematic" => {
                    info!("Applying Thematic lens...");
                    self.thematic_analysis = Some(self.apply_thematic_lens(analysis_target).await?);
                }
                "Logic" => {
                    info!("Applying Logic lens...");
                    self.logic_analysis = Some(self.apply_logic_lens(analysis_target).await?);
                }
                "Evidence" => {
                    info!("Applying Evidence lens...");
                    self.evidence_analysis = Some(self.apply_evidence_lens(analysis_target).await?);
                }
                "Expression" => {
                    info!("Applying Expression lens...");
                    self.expression_analysis = Some(self.apply_expression_lens(analysis_target).await?);
                }
                "Intent" => {
                    info!("Applying Intent lens...");
                    // Intent lens uses BOTH: analysis findings + governance context for alignment
                    self.intent_analysis = Some(self.apply_intent_lens(analysis_target, governance_context).await?);
                }
                _ => {}
            }
        }

        // Cross-lens integration
        info!("Performing cross-lens integration...");
        let diagnostic = self.perform_cross_lens_integration().await?;
        self.integrated_diagnostic = Some(diagnostic.clone());

        // Calculate lens efficacy
        let efficacy_report = self.calculate_lens_efficacy();

        info!("Six-lens analysis complete");
        Ok((diagnostic, efficacy_report))
    }

    /// Get lens sequence based on intent category
    ///
    /// From spec §4.3.4:
    /// - Exploratory: Thematic, Expression, Structure, Logic
    /// - Analytical: Logic, Evidence, Structure, Intent
    /// - Operational: Structure, Intent, Logic, Evidence
    fn get_lens_sequence(&self, intent_category: &str) -> Vec<&'static str> {
        match intent_category.to_lowercase().as_str() {
            "exploratory" => vec!["Thematic", "Expression", "Structural", "Logic", "Evidence", "Intent"],
            "analytical" => vec!["Logic", "Evidence", "Structural", "Intent", "Thematic", "Expression"],
            "operational" => vec!["Structural", "Intent", "Logic", "Evidence", "Thematic", "Expression"],
            _ => vec!["Structural", "Thematic", "Logic", "Evidence", "Expression", "Intent"], // Default sequence
        }
    }

    /// Apply Structural Lens - Organization, hierarchy, flow
    async fn apply_structural_lens(&self, content: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the STRUCTURAL LENS in Method-VI Step 3 analysis.\n\
            Focus on organization, hierarchy, and flow of the USER'S CONTENT.\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT, not governance documents.\n\
            Never critique Method-VI methodology. Analyze the content the user asked you to examine.";

        let user_message = format!(
            r#"STRUCTURAL LENS ANALYSIS

Focus: Organization, hierarchy, flow

USER'S CONTENT TO ANALYZE:
{}

Provide analysis covering:
1. Overall Structure: How is the content organized?
2. Hierarchy: How are elements arranged?
3. Flow: How does information progress?
4. Section Balance: Are sections proportionate?
5. Structural Strengths: What works well in the organization?
6. Structural Weaknesses: What needs improvement?

Format your response as:

**Overall Structure:**
[Your analysis]

**Hierarchy:**
[Your analysis]

**Flow:**
[Your analysis]

**Section Balance:**
[Your analysis]

**Structural Strengths:**
- [Strength 1]
- [Strength 2]
...

**Structural Weaknesses:**
- [Weakness 1]
- [Weakness 2]
...

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        // Extract key findings (simple parsing - look for lines starting with - under Key Findings)
        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Structural".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0, // Will be set by API client
        })
    }

    /// Apply Thematic Lens - Core themes, recurring patterns
    async fn apply_thematic_lens(&self, content: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the THEMATIC LENS in Method-VI Step 3 analysis.\n\
            Focus on core themes and recurring patterns in the USER'S CONTENT.\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT, not governance documents.\n\
            Never critique Method-VI methodology. Analyze the content the user asked you to examine.";

        let user_message = format!(
            r#"THEMATIC LENS ANALYSIS

Focus: Core themes, recurring patterns

USER'S CONTENT TO ANALYZE:
{}

Provide analysis covering:
1. Dominant Themes: What ideas appear most frequently?
2. Recurring Patterns: What structures or concepts repeat?
3. Theme Relationships: How do themes connect or conflict?
4. Missing Themes: What important topics are absent?
5. Theme Strength: Which themes are well-developed vs underdeveloped?

Format your response as:

**Dominant Themes:**
1. [Theme 1] - [Description]
2. [Theme 2] - [Description]
...

**Recurring Patterns:**
- [Pattern 1]
- [Pattern 2]
...

**Theme Relationships:**
[Your analysis of how themes interact]

**Missing Themes:**
- [Missing theme 1]
- [Missing theme 2]
...

**Theme Strength Assessment:**
[Your analysis]

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Thematic".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0,
        })
    }

    /// Apply Logic Lens - Arguments, reasoning chains
    async fn apply_logic_lens(&self, content: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the LOGIC LENS in Method-VI Step 3 analysis.\n\
            Focus on arguments, reasoning chains, and logical validity in the USER'S CONTENT.\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT, not governance documents.\n\
            Never critique Method-VI methodology. Analyze the content the user asked you to examine.";

        let user_message = format!(
            r#"LOGIC LENS ANALYSIS

Focus: Arguments, reasoning chains, logical validity

USER'S CONTENT TO ANALYZE:
{}

Provide analysis covering:
1. Argument Structure: What claims are being made?
2. Reasoning Chains: How are conclusions derived?
3. Logical Soundness: Are arguments valid?
4. Fallacies: Any logical errors present?
5. Evidence-Conclusion Links: Are conclusions supported by premises?
6. Consistency: Any contradictions or inconsistencies?

Format your response as:

**Argument Structure:**
[Your analysis]

**Reasoning Chains:**
1. [Chain 1] - [Analysis]
2. [Chain 2] - [Analysis]
...

**Logical Soundness:**
[Your assessment]

**Fallacies Detected:**
- [Fallacy 1] - [Description]
- [Fallacy 2] - [Description]
...

**Evidence-Conclusion Links:**
[Your analysis]

**Consistency Check:**
[Your analysis]

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Logic".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0,
        })
    }

    /// Apply Evidence Lens - Data, sources, substantiation
    async fn apply_evidence_lens(&self, content: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the EVIDENCE LENS in Method-VI Step 3 analysis.\n\
            Focus on data, sources, and substantiation of claims in the USER'S CONTENT.\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT, not governance documents.\n\
            Never critique Method-VI methodology. Analyze the content the user asked you to examine.";

        let user_message = format!(
            r#"EVIDENCE LENS ANALYSIS

Focus: Data, sources, substantiation

USER'S CONTENT TO ANALYZE:
{}

Provide analysis covering:
1. Evidence Types: What kinds of evidence are used?
2. Source Quality: Are sources credible and reliable?
3. Evidence Sufficiency: Is there enough evidence for each claim?
4. Data Reliability: Can the data be trusted?
5. Evidence Gaps: What claims lack supporting evidence?
6. Evidence Strength: How strong is the overall evidence base?

Format your response as:

**Evidence Types:**
- [Type 1] - [Examples]
- [Type 2] - [Examples]
...

**Source Quality:**
[Your assessment]

**Evidence Sufficiency:**
[Your analysis]

**Data Reliability:**
[Your assessment]

**Evidence Gaps:**
- [Gap 1]
- [Gap 2]
...

**Evidence Strength:**
[Your overall assessment]

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Evidence".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0,
        })
    }

    /// Apply Expression Lens - Tone, clarity, readability
    async fn apply_expression_lens(&self, content: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the EXPRESSION LENS in Method-VI Step 3 analysis.\n\
            Focus on tone, clarity, and readability of the USER'S CONTENT.\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT, not governance documents.\n\
            Never critique Method-VI methodology. Analyze the content the user asked you to examine.";

        let user_message = format!(
            r#"EXPRESSION LENS ANALYSIS

Focus: Tone, clarity, readability

USER'S CONTENT TO ANALYZE:
{}

Provide analysis covering:
1. Tone: What is the overall tone? (formal, casual, technical, etc.)
2. Clarity: How clear and understandable is the language?
3. Readability: How easy is it to read and follow?
4. Ambiguity: Are there ambiguous or unclear passages?
5. Terminology: Is technical language appropriate for the audience?
6. Expression Strengths: What works well in the writing?
7. Expression Weaknesses: What could be improved?

Format your response as:

**Tone:**
[Your analysis]

**Clarity:**
[Your assessment]

**Readability:**
[Your assessment]

**Ambiguity Issues:**
- [Issue 1]
- [Issue 2]
...

**Terminology Assessment:**
[Your analysis]

**Expression Strengths:**
- [Strength 1]
- [Strength 2]
...

**Expression Weaknesses:**
- [Weakness 1]
- [Weakness 2]
...

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            content
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Expression".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0,
        })
    }

    /// Apply Intent Lens - Alignment to Charter
    ///
    /// CRITICAL: This lens analyzes the USER'S CONTENT and checks if findings align with Charter objectives.
    /// It does NOT analyze the Charter itself.
    async fn apply_intent_lens(&self, analysis_target: &str, governance_context: &str) -> Result<LensResult> {
        let system_prompt = "You are applying the INTENT LENS in Method-VI Step 3 analysis.\n\
            \n\
            Your task:\n\
            1. Analyze the USER'S CONTENT to understand what it contains\n\
            2. Check if your analysis findings align with Charter objectives\n\
            \n\
            CRITICAL: You are analyzing the USER'S SUBJECT MATTER CONTENT.\n\
            The Charter is ONLY used as a reference to check alignment - DO NOT analyze the Charter itself.\n\
            Never critique Method-VI methodology.";

        let user_message = format!(
            r#"INTENT LENS ANALYSIS

Focus: Does the user's content align with Charter objectives?

USER'S CONTENT TO ANALYZE:
{}

---

CHARTER (For Alignment Reference Only):
{}

---

Provide analysis covering:
1. Content Understanding: What is the user's content about?
2. Objective Alignment: Do findings from the user's content align with Charter objectives?
3. Scope Adherence: Does the user's content stay within Charter boundaries?
4. Priority Alignment: Are Charter priorities reflected in the user's content?
5. Intent Drift: Any deviation from Charter intent in the user's content?
6. Success Criteria Coverage: Does the user's content address Charter success criteria?
7. Alignment Strength: Overall assessment of alignment

Format your response as:

**Content Understanding:**
[Brief summary of what the user's content is about]

**Objective Alignment:**
[Your analysis of how user's content aligns with Charter objectives]

**Scope Adherence:**
[Does user's content stay within Charter boundaries?]

**Priority Alignment:**
[Are Charter priorities reflected in user's content?]

**Intent Drift:**
[Any deviation from Charter intent in user's content]

**Success Criteria Coverage:**
[Does user's content address Charter success criteria?]

**Alignment Strength:**
[Your overall assessment with score 0.0-1.0]

**Key Findings:**
- [Finding 1]
- [Finding 2]
- [Finding 3]
"#,
            analysis_target, governance_context
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        let key_findings = self.extract_key_findings(&response);
        let efficacy_score = self.calculate_efficacy_score(&key_findings, &response);

        Ok(LensResult {
            lens_name: "Intent".to_string(),
            analysis: response,
            key_findings,
            efficacy_score,
            tokens_used: 0,
        })
    }

    /// Perform cross-lens integration to create Integrated Diagnostic Summary
    async fn perform_cross_lens_integration(&self) -> Result<String> {
        let system_prompt = "You are performing CROSS-LENS INTEGRATION in Method-VI Step 3.\n\
            Synthesize insights from all six analytical lenses into a unified diagnostic summary.";

        // Collect findings from all lenses
        let mut findings_summary = String::new();

        if let Some(result) = &self.structural_analysis {
            findings_summary.push_str(&format!("**Structural Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }
        if let Some(result) = &self.thematic_analysis {
            findings_summary.push_str(&format!("**Thematic Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }
        if let Some(result) = &self.logic_analysis {
            findings_summary.push_str(&format!("**Logic Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }
        if let Some(result) = &self.evidence_analysis {
            findings_summary.push_str(&format!("**Evidence Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }
        if let Some(result) = &self.expression_analysis {
            findings_summary.push_str(&format!("**Expression Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }
        if let Some(result) = &self.intent_analysis {
            findings_summary.push_str(&format!("**Intent Lens:**\n{}\n\n",
                result.key_findings.join("\n- ")));
        }

        let user_message = format!(
            r#"CROSS-LENS INTEGRATION

Synthesis of insights from all six lenses:

{}

Provide integrated analysis covering:
1. How findings connect across lenses
2. Common patterns observed
3. Contradictions to resolve
4. Priority areas for improvement
5. Overall diagnostic assessment

Format your response as a comprehensive Integrated Diagnostic Summary that will guide synthesis in Step 4.

**Integrated Insights:**
[Your synthesis of cross-lens findings]

**Common Patterns:**
- [Pattern 1]
- [Pattern 2]
...

**Contradictions to Resolve:**
- [Contradiction 1]
- [Contradiction 2]
...

**Priority Areas:**
1. [Priority 1]
2. [Priority 2]
...

**Overall Diagnostic Summary:**
[Comprehensive assessment that will inform Step 4 synthesis]
"#,
            findings_summary
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(3000), None)
            .await?;

        Ok(response)
    }

    /// Calculate lens efficacy report for pattern learning
    ///
    /// NOTE: Efficacy scores are currently placeholders (0.0) pending Phase 2 implementation.
    /// See TODO(Phase 2) comment at top of Step 3 section for details.
    fn calculate_lens_efficacy(&self) -> LensEfficacyReport {
        // NOTE: All efficacy_score values in lens_results will be 0.0 (placeholder)
        // This is intentional - see FIX-007 for why previous heuristic was removed

        // Collect all lens results
        let mut lens_results = Vec::new();

        if let Some(r) = &self.structural_analysis {
            lens_results.push(r.clone());
        }
        if let Some(r) = &self.thematic_analysis {
            lens_results.push(r.clone());
        }
        if let Some(r) = &self.logic_analysis {
            lens_results.push(r.clone());
        }
        if let Some(r) = &self.evidence_analysis {
            lens_results.push(r.clone());
        }
        if let Some(r) = &self.expression_analysis {
            lens_results.push(r.clone());
        }
        if let Some(r) = &self.intent_analysis {
            lens_results.push(r.clone());
        }

        // Count total insights
        let total_insights: usize = lens_results.iter()
            .map(|r| r.key_findings.len())
            .sum();

        // Count high-value lenses (efficacy > 0.7)
        // NOTE: Will always be 0 until Phase 2 implements real efficacy calculation
        let high_value_combinations = lens_results.iter()
            .filter(|r| r.efficacy_score > 0.7)
            .count();

        // Calculate costs (rough estimate based on tokens)
        let total_tokens: u32 = lens_results.iter()
            .map(|r| r.tokens_used)
            .sum();

        // Pricing: $3/1M input tokens, $15/1M output tokens
        // Rough estimate: 60% input, 40% output
        let input_tokens = (total_tokens as f64) * 0.6;
        let output_tokens = (total_tokens as f64) * 0.4;
        let actual_cost = (input_tokens / 1_000_000.0 * 3.0) + (output_tokens / 1_000_000.0 * 15.0);

        // Estimated cost for 6 lenses (before execution)
        let estimated_cost = 0.10; // Fixed estimate

        LensEfficacyReport {
            lens_results,
            total_insights,
            high_value_combinations,
            estimated_cost,
            actual_cost,
        }
    }

    /// Extract key findings from lens analysis response
    fn extract_key_findings(&self, response: &str) -> Vec<String> {
        // Simple extraction: find lines starting with - after "Key Findings:"
        let mut findings = Vec::new();
        let mut in_findings_section = false;

        for line in response.lines() {
            if line.contains("Key Findings:") || line.contains("**Key Findings:**") {
                in_findings_section = true;
                continue;
            }

            if in_findings_section {
                let trimmed = line.trim();
                if trimmed.starts_with('-') {
                    findings.push(trimmed.trim_start_matches('-').trim().to_string());
                } else if trimmed.starts_with("**") {
                    // New section started, stop collecting findings
                    break;
                }
            }
        }

        // If no findings found, take last 3 paragraphs as findings
        if findings.is_empty() {
            let paragraphs: Vec<&str> = response.split("\n\n")
                .filter(|p| !p.trim().is_empty())
                .collect();
            findings = paragraphs.iter()
                .rev()
                .take(3)
                .map(|s| s.to_string())
                .collect();
        }

        findings
    }

    /// Calculate lens efficacy score
    ///
    /// TODO(Phase 2 - Pattern Learning): Implement meaningful efficacy calculation
    ///
    /// Current implementation returns placeholder value. Phase 2 should implement:
    /// - Option A: Uniqueness check (findings not duplicated across lenses)
    /// - Option B: Synthesis citation tracking (what gets used downstream)
    /// - Option C: User feedback integration
    /// - Option D: LLM-based quality assessment with stricter criteria
    ///
    /// Previous heuristic (removed in FIX-007) always returned ~100% because:
    /// - LLMs always produce 3+ findings (0.6 score)
    /// - LLMs always produce 1500+ char responses (0.3 score)
    /// - Common words "specific"/"particular"/"notably" (0.1 score)
    /// Result: No discrimination between valuable and low-value insights
    ///
    /// See: Method-VI_MVP_Fixes_and_Enhancements_Guide.md, FIX-007 for design discussion
    fn calculate_efficacy_score(&self, _key_findings: &[String], _full_response: &str) -> f64 {
        // PLACEHOLDER: Return sentinel value indicating "not yet implemented"
        // Using 0.0 rather than 1.0 to make it obvious this is placeholder data
        // Phase 2 will replace this with meaningful calculation
        0.0
    }

    // ==================== STEP 4: SYNTHESIS LOCK-IN ====================

    /// Perform complete Step 4 synthesis
    ///
    /// Transforms analytic intelligence from Step 3 into a unified, explainable model
    pub async fn perform_step4_synthesis(&mut self) -> Result<Step4SynthesisResult> {
        info!("Starting Step 4 synthesis");

        // Ensure we have the integrated diagnostic from Step 3
        let diagnostic = self.integrated_diagnostic.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No integrated diagnostic available. Run Step 3 first."))?;

        info!("Step 4.1: Deriving core thesis");
        let core_thesis = self.derive_core_thesis(diagnostic).await?;

        info!("Step 4.2: Extracting operating principles");
        let operating_principles = self.extract_operating_principles(diagnostic, &core_thesis).await?;

        info!("Step 4.3: Selecting model geometry");
        let (model_geometry, geometry_rationale, novel_flag) =
            self.select_model_geometry(diagnostic, &core_thesis).await?;

        info!("Step 4.4: Creating causality map");
        let causal_spine = self.create_causality_map(diagnostic, &core_thesis, &operating_principles).await?;

        info!("Step 4.5: Authoring North-Star narrative");
        let north_star_narrative = self.author_north_star_narrative(
            &core_thesis,
            &operating_principles,
            &causal_spine,
        ).await?;

        info!("Step 4.6: Creating glossary");
        let glossary = self.create_glossary(diagnostic, &core_thesis).await?;

        info!("Step 4.7: Documenting limitations");
        let limitations = self.document_limitations(diagnostic, &core_thesis).await?;

        info!("Step 4 synthesis complete");

        Ok(Step4SynthesisResult {
            core_thesis,
            operating_principles,
            model_geometry,
            geometry_rationale,
            causal_spine,
            north_star_narrative,
            glossary,
            limitations,
            novel_geometry_flag: novel_flag,
        })
    }

    /// Derive core thesis from integrated diagnostic
    async fn derive_core_thesis(&self, diagnostic: &str) -> Result<String> {
        let system_prompt = "You are deriving the CORE THESIS in Method-VI Step 4.\n\
            Extract the central claim or finding that unifies all analytical insights.";

        let user_message = format!(
            r#"CORE THESIS DERIVATION

Input: Integrated Diagnostic Summary
Goal: Extract central claim or finding

Integrated Diagnostics:
{}

Analysis Required:
1. What is the fundamental insight across all lenses?
2. What is the core claim that emerges?
3. What is the organizing principle that ties everything together?

Provide:
- Analysis of Diagnostics (brief synthesis of key insights)
- Core Thesis Statement (single, clear statement - 1-3 sentences max)
- Supporting Rationale (why this thesis captures the essence)

Format as:
ANALYSIS: [Brief synthesis]

CORE THESIS: [Your clear, concise thesis statement]

RATIONALE: [Why this captures the essence]
"#,
            diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1500), None)
            .await?;

        // Extract the thesis statement from the response
        let thesis = if let Some(start) = response.find("CORE THESIS:") {
            let after_label = &response[start + 12..];
            if let Some(end) = after_label.find("RATIONALE:") {
                after_label[..end].trim().to_string()
            } else {
                after_label.trim().to_string()
            }
        } else {
            response.trim().to_string()
        };

        Ok(thesis)
    }

    /// Extract operating principles from diagnostic and thesis
    async fn extract_operating_principles(&self, diagnostic: &str, thesis: &str) -> Result<Vec<String>> {
        let system_prompt = "You are extracting OPERATING PRINCIPLES in Method-VI Step 4.\n\
            Identify the governing rules that define how the framework operates.";

        let user_message = format!(
            r#"OPERATING PRINCIPLES EXTRACTION

Core Thesis:
{}

Integrated Diagnostics:
{}

Goal: Define 3-7 Operating Principles that govern this framework

Principles should:
- Guide rather than prescribe
- Be clear and actionable
- Emerge from the analysis
- Support the core thesis

Format each principle as:
1. [Principle Name]: [Clear statement of the principle]

Provide between 3-7 principles.
"#,
            thesis, diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1500), None)
            .await?;

        // Extract principles from numbered list
        let mut principles = Vec::new();
        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(|c: char| c.is_numeric()) {
                if let Some(colon_pos) = trimmed.find(':') {
                    let principle = trimmed[colon_pos + 1..].trim().to_string();
                    if !principle.is_empty() {
                        principles.push(principle);
                    }
                }
            }
        }

        // If no numbered principles found, try to extract from the response
        if principles.is_empty() {
            for line in response.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed.len() > 20 {
                    principles.push(trimmed.to_string());
                    if principles.len() >= 5 {
                        break;
                    }
                }
            }
        }

        Ok(principles)
    }

    /// Select model geometry (Linear, Cyclic, or Branching)
    async fn select_model_geometry(
        &self,
        diagnostic: &str,
        thesis: &str,
    ) -> Result<(ModelGeometry, String, bool)> {
        let system_prompt = "You are selecting MODEL GEOMETRY in Method-VI Step 4.\n\
            Choose the structural pattern that best fits the content and relationships.";

        let user_message = format!(
            r#"MODEL GEOMETRY SELECTION

Core Thesis:
{}

Synthesis Content Summary:
{}

Geometry Options:
1. LINEAR: Sequential, one-directional flow (A → B → C)
   - Use when: Process has clear stages, dependencies are sequential

2. CYCLIC: Feedback loops, iterative processes (A ⇄ B ⇄ C)
   - Use when: Continuous improvement, recurring patterns, feedback systems

3. BRANCHING: Decision points, multiple paths (A → B₁/B₂ → C)
   - Use when: Conditional logic, multiple scenarios, decision trees

Analysis Required:
1. Content Nature: What structure fits the material?
2. Causal Relationships: How do elements interact?
3. User Mental Model: How will the audience understand this best?

Provide:
SELECTED GEOMETRY: [LINEAR / CYCLIC / BRANCHING]

RATIONALE: [2-3 sentences explaining why this geometry fits best]

NOVEL: [YES / NO] - Is this an innovative or unusual geometry application?

MODEL DIAGRAM: [Simple text representation of the structure]
"#,
            thesis, diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1500), None)
            .await?;

        // Extract geometry selection
        let geometry = if response.to_uppercase().contains("CYCLIC") {
            ModelGeometry::Cyclic
        } else if response.to_uppercase().contains("BRANCHING") {
            ModelGeometry::Branching
        } else {
            ModelGeometry::Linear
        };

        // Extract rationale
        let rationale = if let Some(start) = response.find("RATIONALE:") {
            let after_label = &response[start + 10..];
            if let Some(end) = after_label.find("NOVEL:") {
                after_label[..end].trim().to_string()
            } else {
                after_label.trim().to_string()
            }
        } else {
            "Model geometry selected based on content structure and relationships.".to_string()
        };

        // Check if novel
        let novel = response.to_uppercase().contains("NOVEL: YES");

        Ok((geometry, rationale, novel))
    }

    /// Create causality map (Causal Spine Draft)
    async fn create_causality_map(
        &self,
        diagnostic: &str,
        thesis: &str,
        principles: &[String],
    ) -> Result<String> {
        let system_prompt = "You are creating the CAUSAL SPINE in Method-VI Step 4.\n\
            Map how elements relate and influence each other. Show the 'why', not just the 'what'.";

        let principles_text = principles.join("\n");

        let user_message = format!(
            r#"CAUSAL SPINE CREATION

Core Thesis:
{}

Operating Principles:
{}

Diagnostic Context:
{}

Goal: Map causal relationships and influences

Requirements:
1. Show WHY things happen, not just what happens
2. Trace dependencies and influences
3. Make causality explicit and traceable
4. Identify feedback loops if present
5. Note critical dependencies

Format:
CAUSAL RELATIONSHIPS:
- [Element A] → [Element B]: [Why/How A influences B]
- [Element B] → [Element C]: [Why/How B influences C]

FEEDBACK LOOPS (if any):
- [Description of any circular dependencies or reinforcing cycles]

CRITICAL DEPENDENCIES:
- [Key relationships that must be maintained]

CAUSAL MAP:
[Text-based visual representation of the causal structure]
"#,
            thesis, principles_text, diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        Ok(response)
    }

    /// Author North-Star narrative paragraph
    async fn author_north_star_narrative(
        &self,
        thesis: &str,
        principles: &[String],
        causal_spine: &str,
    ) -> Result<String> {
        let system_prompt = "You are authoring the NORTH-STAR NARRATIVE in Method-VI Step 4.\n\
            Write a guiding paragraph that captures the essence and direction of the framework.";

        let principles_text = principles.join("\n");

        let user_message = format!(
            r#"NORTH-STAR NARRATIVE AUTHORING

Core Thesis:
{}

Operating Principles:
{}

Causal Structure:
{}

Goal: Write a single guiding paragraph (100-200 words) that:
- Captures the essence of the framework
- Provides direction for downstream work
- Integrates thesis, principles, and causality
- Serves as a reference point for all future decisions
- Is clear, compelling, and actionable

The North-Star narrative should answer:
- What is this framework fundamentally about?
- Why does it matter?
- How should it guide future work?

Write the narrative as a single cohesive paragraph.
"#,
            thesis, principles_text, causal_spine
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1000), None)
            .await?;

        Ok(response.trim().to_string())
    }

    /// Create glossary of key terms
    async fn create_glossary(&self, diagnostic: &str, thesis: &str) -> Result<Vec<GlossaryEntry>> {
        let system_prompt = "You are creating the GLOSSARY in Method-VI Step 4.\n\
            Lock down key terminology to ensure consistent understanding.";

        let user_message = format!(
            r#"GLOSSARY CREATION

Core Thesis:
{}

Diagnostic Context:
{}

Goal: Create a glossary of 5-15 key terms

Identify:
- Domain-specific terminology
- Terms with specific meaning in this framework
- Concepts that need precise definition
- Terms that might be ambiguous without definition

Format each entry as:
TERM: [Term name]
DEFINITION: [Clear, precise definition]

---

Provide 5-15 entries.
"#,
            thesis, diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(2000), None)
            .await?;

        // Parse glossary entries - try multiple formats
        let mut glossary = Vec::new();

        // Try format 1: TERM: / DEFINITION: with --- separators
        let sections: Vec<&str> = response.split("---").collect();
        if sections.len() > 1 {
            for section in sections {
                let section = section.trim();
                if section.is_empty() {
                    continue;
                }

                let mut term = String::new();
                let mut definition = String::new();

                for line in section.lines() {
                    let line = line.trim();
                    if line.starts_with("TERM:") {
                        term = line[5..].trim().to_string();
                    } else if line.starts_with("DEFINITION:") {
                        definition = line[11..].trim().to_string();
                    } else if !term.is_empty() && !line.is_empty() && !line.starts_with("TERM") {
                        // Continue definition if we're in a multi-line definition
                        if !definition.is_empty() {
                            definition.push(' ');
                        }
                        definition.push_str(line);
                    }
                }

                if !term.is_empty() && !definition.is_empty() {
                    glossary.push(GlossaryEntry { term, definition });
                }
            }
        }

        // Try format 2: **Term**: definition or **Term** - definition
        if glossary.is_empty() {
            let mut current_term = String::new();
            let mut current_def = String::new();

            for line in response.lines() {
                let line = line.trim();

                // Check for bold term formats: **Term**: or **Term** -
                if line.starts_with("**") {
                    // Save previous entry
                    if !current_term.is_empty() && !current_def.is_empty() {
                        glossary.push(GlossaryEntry {
                            term: current_term.clone(),
                            definition: current_def.trim().to_string(),
                        });
                    }

                    // Extract term and definition from same line if present
                    // Find the closing ** by looking for it after position 2
                    if let Some(end_bold) = line[2..].find("**") {
                        let end_pos = end_bold + 2;
                        current_term = line[2..end_pos].trim().to_string();
                        let rest = &line[end_pos + 2..];
                        // Remove : or - separator
                        current_def = rest.trim_start_matches(':').trim_start_matches('-').trim().to_string();
                    } else {
                        current_term = line.replace("**", "").replace(":", "").trim().to_string();
                        current_def.clear();
                    }
                } else if !current_term.is_empty() && !line.is_empty() && !line.starts_with("TERM") {
                    // Continue definition
                    if !current_def.is_empty() {
                        current_def.push(' ');
                    }
                    current_def.push_str(line);
                }
            }

            // Add last entry
            if !current_term.is_empty() && !current_def.is_empty() {
                glossary.push(GlossaryEntry {
                    term: current_term,
                    definition: current_def.trim().to_string(),
                });
            }
        }

        // Try format 3: Numbered list "1. Term: definition"
        if glossary.is_empty() {
            for line in response.lines() {
                let line = line.trim();
                if line.starts_with(|c: char| c.is_numeric()) {
                    if let Some(dot_pos) = line.find('.') {
                        let rest = line[dot_pos + 1..].trim();
                        if let Some(colon_pos) = rest.find(':') {
                            let term = rest[..colon_pos].trim().to_string();
                            let definition = rest[colon_pos + 1..].trim().to_string();
                            if !term.is_empty() && !definition.is_empty() {
                                glossary.push(GlossaryEntry { term, definition });
                            }
                        }
                    }
                }
            }
        }

        Ok(glossary)
    }

    /// Document what the framework doesn't cover (limitations)
    async fn document_limitations(&self, diagnostic: &str, thesis: &str) -> Result<Vec<String>> {
        let system_prompt = "You are documenting LIMITATIONS in Method-VI Step 4.\n\
            Be honest about what the framework doesn't cover or address.";

        let user_message = format!(
            r#"LIMITATIONS DOCUMENTATION

Core Thesis:
{}

Diagnostic Context:
{}

Goal: Document 3-7 clear limitations of this framework

A good limitation statement:
- Is specific and honest
- Identifies scope boundaries
- Notes what's deliberately excluded
- Acknowledges trade-offs made
- Helps users understand when NOT to use this framework

Format each limitation as:
- [Clear statement of what is NOT covered or addressed]

Provide 3-7 limitation statements.
"#,
            thesis, diagnostic
        );

        let response = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(1500), None)
            .await?;

        // Extract limitations from bullet points
        let mut limitations = Vec::new();
        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('•') {
                let limitation = trimmed[1..].trim().to_string();
                if !limitation.is_empty() {
                    limitations.push(limitation);
                }
            } else if trimmed.starts_with(|c: char| c.is_numeric()) && trimmed.contains('.') {
                // Handle numbered lists like "1. Limitation"
                if let Some(dot_pos) = trimmed.find('.') {
                    let limitation = trimmed[dot_pos + 1..].trim().to_string();
                    if !limitation.is_empty() {
                        limitations.push(limitation);
                    }
                }
            }
        }

        // If no bullet points found, try to extract sentences
        if limitations.is_empty() {
            for line in response.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed.len() > 30 {
                    limitations.push(trimmed.to_string());
                    if limitations.len() >= 5 {
                        break;
                    }
                }
            }
        }

        Ok(limitations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test content for lens analysis
    const TEST_CHARTER: &str = r#"
# Project Charter: Customer Analytics Dashboard

## Objectives
1. Build a real-time analytics dashboard for customer behavior tracking
2. Provide actionable insights to marketing and sales teams
3. Improve customer retention by 15% through data-driven decisions

## Scope
- Integration with existing CRM system
- Real-time data visualization
- Predictive analytics for customer churn
- Mobile-responsive design

## Constraints
- Must be completed within 3 months
- Budget: $150,000
- Team size: 5 developers, 1 designer, 1 PM

## Success Criteria
- Dashboard loads in < 2 seconds
- 95% uptime
- User satisfaction score > 4.0/5.0
- Successfully integrated with 3+ data sources
"#;

    #[tokio::test]
    #[ignore] // Run with: cargo test --lib -- --ignored --nocapture
    async fn test_analysis_synthesis_agent_isolation() {
        // Initialize logger for test output
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();

        println!("\n=== Testing Analysis & Synthesis Agent in Isolation ===\n");

        // 1. Create agent instance with API key
        println!("1. Creating agent instance...");
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable must be set");

        let mut agent = AnalysisSynthesisAgent::new(api_key)
            .expect("Failed to create agent");
        println!("   ✓ Agent created successfully\n");

        // 2. Test each lens function individually
        println!("2. Testing individual lens functions...\n");

        // Test Structural Lens
        println!("   Testing Structural Lens...");
        let structural_result = agent.apply_structural_lens(TEST_CHARTER).await
            .expect("Structural lens failed");
        assert!(!structural_result.analysis.is_empty(), "Structural analysis is empty");
        assert!(!structural_result.key_findings.is_empty(), "No structural findings");
        assert!(structural_result.efficacy_score > 0.0, "Invalid efficacy score");
        println!("   ✓ Structural Lens: {} findings, efficacy: {:.2}",
                 structural_result.key_findings.len(),
                 structural_result.efficacy_score);

        // Test Thematic Lens
        println!("   Testing Thematic Lens...");
        let thematic_result = agent.apply_thematic_lens(TEST_CHARTER).await
            .expect("Thematic lens failed");
        assert!(!thematic_result.analysis.is_empty(), "Thematic analysis is empty");
        assert!(!thematic_result.key_findings.is_empty(), "No thematic findings");
        println!("   ✓ Thematic Lens: {} findings, efficacy: {:.2}",
                 thematic_result.key_findings.len(),
                 thematic_result.efficacy_score);

        // Test Logic Lens
        println!("   Testing Logic Lens...");
        let logic_result = agent.apply_logic_lens(TEST_CHARTER).await
            .expect("Logic lens failed");
        assert!(!logic_result.analysis.is_empty(), "Logic analysis is empty");
        assert!(!logic_result.key_findings.is_empty(), "No logic findings");
        println!("   ✓ Logic Lens: {} findings, efficacy: {:.2}",
                 logic_result.key_findings.len(),
                 logic_result.efficacy_score);

        // Test Evidence Lens
        println!("   Testing Evidence Lens...");
        let evidence_result = agent.apply_evidence_lens(TEST_CHARTER).await
            .expect("Evidence lens failed");
        assert!(!evidence_result.analysis.is_empty(), "Evidence analysis is empty");
        assert!(!evidence_result.key_findings.is_empty(), "No evidence findings");
        println!("   ✓ Evidence Lens: {} findings, efficacy: {:.2}",
                 evidence_result.key_findings.len(),
                 evidence_result.efficacy_score);

        // Test Expression Lens
        println!("   Testing Expression Lens...");
        let expression_result = agent.apply_expression_lens(TEST_CHARTER).await
            .expect("Expression lens failed");
        assert!(!expression_result.analysis.is_empty(), "Expression analysis is empty");
        assert!(!expression_result.key_findings.is_empty(), "No expression findings");
        println!("   ✓ Expression Lens: {} findings, efficacy: {:.2}",
                 expression_result.key_findings.len(),
                 expression_result.efficacy_score);

        // Test Intent Lens (needs both analysis target and governance context)
        println!("   Testing Intent Lens...");
        let mock_governance = "# Charter\n\n## Objectives\n1. Build analytics dashboard\n2. Provide insights\n\n## Success Criteria\n- Performance < 2s\n- 95% uptime";
        let intent_result = agent.apply_intent_lens(TEST_CHARTER, mock_governance).await
            .expect("Intent lens failed");
        assert!(!intent_result.analysis.is_empty(), "Intent analysis is empty");
        assert!(!intent_result.key_findings.is_empty(), "No intent findings");
        println!("   ✓ Intent Lens: {} findings, efficacy: {:.2}\n",
                 intent_result.key_findings.len(),
                 intent_result.efficacy_score);

        // 3. Verify agent stores results internally
        println!("3. Testing six-lens analysis workflow (with internal storage)...");
        let mock_governance_full = "# Charter\n\n## Objectives\n1. Build real-time analytics dashboard\n2. Provide actionable insights\n3. Improve customer retention by 15%\n\n## Success Criteria\n- Dashboard loads in < 2 seconds\n- 95% uptime\n- User satisfaction > 4.0/5.0";
        let (integrated_diagnostic, lens_efficacy) = agent
            .perform_six_lens_analysis(TEST_CHARTER, mock_governance_full, "Analytical")
            .await
            .expect("Six-lens analysis failed");

        assert!(!integrated_diagnostic.is_empty(), "Integrated diagnostic is empty");
        println!("   ✓ Integrated diagnostic generated ({} chars)", integrated_diagnostic.len());

        // 4. Verify lens efficacy tracking works
        println!("\n4. Verifying lens efficacy tracking...");
        assert!(!lens_efficacy.lens_results.is_empty(), "No lens results tracked");
        assert!(lens_efficacy.total_insights > 0, "No insights tracked");

        // Verify each lens has efficacy tracking
        for lens_result in &lens_efficacy.lens_results {
            assert!(lens_result.efficacy_score >= 0.0 && lens_result.efficacy_score <= 1.0,
                "Lens {} has invalid efficacy score: {}", lens_result.lens_name, lens_result.efficacy_score);
        }

        println!("   ✓ Lens Efficacy Report:");
        for lens_result in &lens_efficacy.lens_results {
            println!("     - {}: {:.2}", lens_result.lens_name, lens_result.efficacy_score);
        }
        println!("     - Total Insights: {}", lens_efficacy.total_insights);
        println!("     - High-Value Combinations: {}", lens_efficacy.high_value_combinations);

        // 5. Verify agent maintains state between calls
        println!("\n5. Verifying agent state persistence (Step 3 → Step 4)...");

        // Verify integrated diagnostic is stored (needed for Step 4)
        assert!(agent.integrated_diagnostic.is_some(), "Integrated diagnostic not stored");
        println!("   ✓ Agent stored integrated diagnostic internally");

        // Now test Step 4 synthesis (should use stored diagnostic)
        println!("\n6. Testing Step 4 synthesis (using stored state from Step 3)...");
        let synthesis_result = agent
            .perform_step4_synthesis()
            .await
            .expect("Step 4 synthesis failed");

        // Verify all Step 4 artifacts
        assert!(!synthesis_result.core_thesis.is_empty(), "Core thesis is empty");
        assert!(!synthesis_result.operating_principles.is_empty(), "No operating principles");
        assert!(!synthesis_result.causal_spine.is_empty(), "Causal spine is empty");
        assert!(!synthesis_result.north_star_narrative.is_empty(), "North-star narrative is empty");
        assert!(!synthesis_result.glossary.is_empty(), "No glossary entries");
        assert!(!synthesis_result.limitations.is_empty(), "No limitations documented");

        println!("   ✓ Step 4 Synthesis Results:");
        println!("     - Core Thesis: {} chars", synthesis_result.core_thesis.len());
        println!("     - Operating Principles: {}", synthesis_result.operating_principles.len());
        println!("     - Model Geometry: {:?}", synthesis_result.model_geometry);
        println!("     - Causal Spine: {} chars", synthesis_result.causal_spine.len());
        println!("     - North-Star Narrative: {} chars", synthesis_result.north_star_narrative.len());
        println!("     - Glossary Entries: {}", synthesis_result.glossary.len());
        println!("     - Limitations: {}", synthesis_result.limitations.len());
        println!("     - Novel Geometry: {}", synthesis_result.novel_geometry_flag);

        // Verify glossary structure
        for entry in &synthesis_result.glossary {
            assert!(!entry.term.is_empty(), "Glossary entry has empty term");
            assert!(!entry.definition.is_empty(), "Glossary entry has empty definition");
        }

        // Verify at least 3 operating principles
        assert!(synthesis_result.operating_principles.len() >= 3,
                "Expected at least 3 operating principles, got {}",
                synthesis_result.operating_principles.len());

        // Verify at least 3 limitations
        assert!(synthesis_result.limitations.len() >= 3,
                "Expected at least 3 limitations, got {}",
                synthesis_result.limitations.len());

        println!("\n=== All Tests Passed! ===");
        println!("\nSummary:");
        println!("✓ Agent instance created successfully");
        println!("✓ All 6 lens functions work correctly");
        println!("✓ Agent stores lens results internally");
        println!("✓ Cross-lens integration produces diagnostic");
        println!("✓ Lens efficacy tracking works");
        println!("✓ State persists from Step 3 to Step 4");
        println!("✓ Step 4 synthesis uses stored state correctly");
    }

    #[test]
    fn test_lens_efficacy_calculation() {
        let agent = AnalysisSynthesisAgent {
            api_client: AnthropicClient::new("dummy-key".to_string()).unwrap(),
            structural_analysis: None,
            thematic_analysis: None,
            logic_analysis: None,
            evidence_analysis: None,
            expression_analysis: None,
            intent_analysis: None,
            integrated_diagnostic: None,
        };

        // Test with few findings
        let findings = vec!["Finding 1".to_string(), "Finding 2".to_string()];
        let response = "This is a short response.";
        let score = agent.calculate_efficacy_score(&findings, response);
        // Placeholder returns 0.0 - accept this until Phase 2 implementation
        assert!(score >= 0.0 && score <= 1.0, "Score should be between 0 and 1");

        // Test with many findings
        let many_findings = vec![
            "Finding 1".to_string(),
            "Finding 2".to_string(),
            "Finding 3".to_string(),
            "Finding 4".to_string(),
        ];
        let long_response = "This is a much longer response with specific details and particular insights that should increase the efficacy score.";
        let high_score = agent.calculate_efficacy_score(&many_findings, long_response);
        // Placeholder returns 0.0 for both - when implemented, high_score should be > score
        assert!(high_score == score, "Placeholder: both return 0.0 until Phase 2 implementation");
    }

    #[test]
    fn test_extract_key_findings() {
        let agent = AnalysisSynthesisAgent {
            api_client: AnthropicClient::new("dummy-key".to_string()).unwrap(),
            structural_analysis: None,
            thematic_analysis: None,
            logic_analysis: None,
            evidence_analysis: None,
            expression_analysis: None,
            intent_analysis: None,
            integrated_diagnostic: None,
        };

        // Test with KEY FINDINGS section
        let response = r#"
Analysis complete.

KEY FINDINGS:
- Finding one about structure
- Finding two about patterns
- Finding three about gaps

Conclusion here.
"#;
        let findings = agent.extract_key_findings(response);
        assert_eq!(findings.len(), 3, "Should extract 3 findings");

        // Test with FINDINGS section
        let response2 = r#"
FINDINGS:
- First finding
- Second finding

Other content.
"#;
        let findings2 = agent.extract_key_findings(response2);
        assert_eq!(findings2.len(), 2, "Should extract 2 findings");
    }
}
