use anyhow::{Context as AnyhropicContext, Result};
use chrono::Utc;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::api::AnthropicClient;

/// Intent Summary artifact produced by Scope & Pattern Agent
///
/// This is the structured output from Step 0 intent interpretation.
/// It will be refined into an Intent_Anchor artifact in Step 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSummary {
    // Artifact metadata
    pub artifact_id: String,
    pub artifact_type: String,
    pub run_id: String,
    pub step_origin: u8,
    pub created_at: String,
    pub hash: String,
    pub parent_hash: Option<String>,
    pub dependencies: Vec<String>,
    pub intent_anchor_link: Option<String>,
    pub is_immutable: bool,
    pub author: String,
    pub governance_role: String,

    // Intent extraction
    pub user_request: String,
    pub primary_goal: String,
    pub audience: String,
    pub expected_outcome: String,
    pub intent_category: String,

    // Initial assessment
    pub confidence_score: u8,
    pub confidence_explanation: String,
    pub request_specificity: String,
    pub scope_definition_clarity: String,
    pub success_criteria_state: String,

    // Clarification
    pub questions_for_clarification: Vec<String>,

    // Preliminary scope
    pub likely_in_scope: Vec<String>,
    pub likely_out_of_scope: Vec<String>,
    pub edge_cases: Vec<String>,
}

impl IntentSummary {
    /// Generate the artifact content body (without frontmatter) for hashing
    pub fn generate_content_body(&self) -> String {
        format!(
            r#"# Intent Summary

## User Request

> {}

## Intent Extraction

### Primary Goal
{}

### Audience
{}

### Expected Outcome
{}

### Intent Category
{}

## Initial Assessment

### Confidence Score
{} - {}

### Clarity Indicators
- Request specificity: {}
- Scope definition: {}
- Success criteria: {}

## Questions for Clarification

{}

## Preliminary Scope Boundaries

### Likely In Scope
{}

### Likely Out of Scope
{}

### Edge Cases (Need Confirmation)
{}

---
*Preliminary artifact - will be refined into Intent_Anchor at Step 1*
"#,
            self.user_request,
            self.primary_goal,
            self.audience,
            self.expected_outcome,
            self.intent_category,
            self.confidence_score,
            self.confidence_explanation,
            self.request_specificity,
            self.scope_definition_clarity,
            self.success_criteria_state,
            format_list(&self.questions_for_clarification, "None - intent is clear"),
            format_list(&self.likely_in_scope, "(None)"),
            format_list(&self.likely_out_of_scope, "(None)"),
            format_list(&self.edge_cases, "(None)")
        )
    }

    /// Generate SHA-256 hash of the content body
    pub fn compute_hash(&self) -> String {
        let content = self.generate_content_body();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate the complete artifact as markdown with YAML frontmatter
    pub fn to_markdown(&self) -> String {
        format!(
            r#"---
artifact_id: "{}"
artifact_type: "{}"
run_id: "{}"
step_origin: {}
created_at: "{}"
hash: "{}"
parent_hash: {}
dependencies: {}
intent_anchor_link: {}
is_immutable: {}
author: "{}"
governance_role: "{}"
---

{}
"#,
            self.artifact_id,
            self.artifact_type,
            self.run_id,
            self.step_origin,
            self.created_at,
            self.hash,
            format_option_string(&self.parent_hash),
            format_vec_string(&self.dependencies),
            format_option_string(&self.intent_anchor_link),
            self.is_immutable,
            self.author,
            self.governance_role,
            self.generate_content_body()
        )
    }
}

/// Helper: Format a list as numbered markdown list or fallback text
fn format_list(items: &[String], fallback: &str) -> String {
    if items.is_empty() {
        fallback.to_string()
    } else {
        items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{}. {}", i + 1, item))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Helper: Format Option<String> for YAML
fn format_option_string(opt: &Option<String>) -> String {
    match opt {
        Some(s) => format!("\"{}\"", s),
        None => "null".to_string(),
    }
}

/// Helper: Format Vec<String> for YAML
fn format_vec_string(vec: &Vec<String>) -> String {
    if vec.is_empty() {
        "[]".to_string()
    } else {
        let items = vec
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}]", items)
    }
}

/// Scope & Pattern Agent
///
/// Responsible for:
/// - Intent interpretation (Step 0)
/// - Scope boundary definition
/// - Pattern recommendation (Learning Plane queries)
/// - Drift detection (continuous monitoring)
pub struct ScopePatternAgent {
    claude_client: AnthropicClient,
}

impl ScopePatternAgent {
    /// Create a new Scope & Pattern Agent
    pub fn new(claude_client: AnthropicClient) -> Self {
        ScopePatternAgent { claude_client }
    }

    /// Interpret user intent and create Intent_Summary artifact
    ///
    /// This is invoked during Step 0 of the Method-VI process.
    ///
    /// # Arguments
    /// * `run_id` - The current run identifier
    /// * `user_request` - The user's original request
    /// * `steno_ledger` - The Steno-Ledger context string
    ///
    /// # Returns
    /// A complete Intent_Summary artifact
    pub async fn interpret_intent(
        &self,
        run_id: &str,
        user_request: &str,
        steno_ledger: &str,
    ) -> Result<IntentSummary> {
        info!("Scope & Pattern Agent: Interpreting user intent");
        debug!("Run ID: {}", run_id);
        debug!("Steno-Ledger: {}", steno_ledger);

        // Build the system prompt with Steno-Ledger prepended
        let system_prompt = format!(
            r#"{steno_ledger}

You are operating as the Scope & Pattern Agent under the OBSERVER stance.
PERMITTED: Data collection, pattern matching, drift detection.
FORBIDDEN: Active intervention, scope changes.

You MUST respond in a structured format that will be parsed. Follow the format EXACTLY."#
        );

        // Build the user prompt with intent interpretation template
        let user_prompt = format!(
            r#"INTENT INTERPRETATION
User Request: {user_request}

Please extract:

Primary Goal: [What user wants to accomplish - single clear statement]
Audience: [Who will use this]
Expected Outcome: [What success looks like]
Intent Category: [Exploratory | Analytical | Operational]
Initial Confidence: [0-100]

Confidence Explanation: [Why this confidence level?]

Clarity Indicators:
- Request specificity: [High | Medium | Low]
- Scope definition: [Clear | Partial | Unclear]
- Success criteria: [Defined | Implied | Missing]

Questions for Clarity:
[List any ambiguities that need resolution, or "None - intent is clear"]

Preliminary Scope:
IN SCOPE:
- [items]

OUT OF SCOPE:
- [items]

EDGE CASES (need confirmation):
- [items, or "None"]

Respond in the EXACT format above, preserving the section headers."#,
            user_request = user_request
        );

        // Call Claude API
        info!("Calling Claude API for intent interpretation...");
        let response = self
            .claude_client
            .call_claude(&system_prompt, &user_prompt, None, Some(2000))
            .await
            .context("Failed to call Claude API for intent interpretation")?;

        debug!("Claude response received:\n{}", response);

        // Parse the response into IntentSummary
        info!("Parsing Claude response...");
        let parsed = Self::parse_intent_response(run_id, user_request, &response)?;

        info!("Intent interpretation complete");
        info!("  Primary Goal: {}", parsed.primary_goal);
        info!("  Confidence: {}", parsed.confidence_score);
        info!("  Category: {}", parsed.intent_category);

        Ok(parsed)
    }

    /// Parse Claude's structured response into an IntentSummary artifact
    fn parse_intent_response(
        run_id: &str,
        user_request: &str,
        response: &str,
    ) -> Result<IntentSummary> {
        debug!("Parsing intent response...");

        // Extract sections using simple string parsing
        let primary_goal = extract_field(response, "Primary Goal:")
            .unwrap_or_else(|| "Unable to determine primary goal".to_string());

        let audience = extract_field(response, "Audience:")
            .unwrap_or_else(|| "General users".to_string());

        let expected_outcome = extract_field(response, "Expected Outcome:")
            .unwrap_or_else(|| "Outcome to be defined".to_string());

        let intent_category = extract_field(response, "Intent Category:")
            .unwrap_or_else(|| "Operational".to_string())
            .trim()
            .to_string();

        let confidence_score = extract_field(response, "Initial Confidence:")
            .and_then(|s| s.trim().parse::<u8>().ok())
            .unwrap_or(50);

        let confidence_explanation = extract_field(response, "Confidence Explanation:")
            .unwrap_or_else(|| "Confidence level not specified".to_string());

        let request_specificity = extract_clarity_indicator(response, "Request specificity:")
            .unwrap_or_else(|| "Medium".to_string());

        let scope_definition_clarity =
            extract_clarity_indicator(response, "Scope definition:")
                .unwrap_or_else(|| "Partial".to_string());

        let success_criteria_state = extract_clarity_indicator(response, "Success criteria:")
            .unwrap_or_else(|| "Implied".to_string());

        let questions_for_clarification = extract_list_section(response, "Questions for Clarity:");

        let likely_in_scope = extract_list_section(response, "IN SCOPE:");
        let likely_out_of_scope = extract_list_section(response, "OUT OF SCOPE:");
        let edge_cases = extract_list_section(response, "EDGE CASES");

        // Generate artifact metadata
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let artifact_id = format!("{}-intent-summary-{}", run_id, timestamp);
        let created_at = Utc::now().to_rfc3339();

        // Create the artifact
        let mut artifact = IntentSummary {
            artifact_id,
            artifact_type: "Intent_Summary".to_string(),
            run_id: run_id.to_string(),
            step_origin: 0,
            created_at,
            hash: String::new(), // Will be computed below
            parent_hash: None,
            dependencies: vec![],
            intent_anchor_link: None,
            is_immutable: false,
            author: "scope-pattern-agent".to_string(),
            governance_role: "Observer".to_string(),
            user_request: user_request.to_string(),
            primary_goal,
            audience,
            expected_outcome,
            intent_category,
            confidence_score,
            confidence_explanation,
            request_specificity,
            scope_definition_clarity,
            success_criteria_state,
            questions_for_clarification,
            likely_in_scope,
            likely_out_of_scope,
            edge_cases,
        };

        // Compute hash of content body
        artifact.hash = artifact.compute_hash();

        debug!("Intent artifact created: {}", artifact.artifact_id);
        debug!("Content hash: {}", artifact.hash);

        Ok(artifact)
    }

    /// Create Intent_Anchor artifact (Step 1)
    ///
    /// Finalizes and locks the intent from the Intent_Summary.
    /// This becomes the immutable root of the Coherence Spine.
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `intent_summary_content` - Content from the Intent_Summary artifact
    /// * `intent_summary_hash` - Hash of the Intent_Summary artifact
    pub async fn create_intent_anchor(
        &self,
        run_id: &str,
        intent_summary_content: &str,
        intent_summary_hash: &str,
    ) -> Result<String> {
        info!("Creating Intent_Anchor for run {}", run_id);

        let system_prompt = "You are finalizing the Intent Anchor for Method-VI. \
            This is an IMMUTABLE artifact that locks the project intent and scope. \
            Review the Intent Summary and create the final, locked version with all \
            clarifications resolved and scope boundaries confirmed.";

        let user_message = format!(
            "INTENT ANCHOR FINALIZATION\n\n\
            Intent Summary:\n{}\n\n\
            Create the final Intent_Anchor document including:\n\
            1. Canonical Intent Statement (single, clear, unambiguous)\n\
            2. Locked Parameters (primary goal, audience, success criteria, intent category)\n\
            3. Scope Boundaries (confirmed in scope, out of scope, resolved edge cases)\n\
            4. Pattern Selection (if any pattern was applied)\n\
            5. Anchor Metadata (questions resolved, confidence score, human approval)\n\n\
            Use the Method-VI Intent_Anchor template format. \
            Return ONLY the content body (without YAML frontmatter).",
            intent_summary_content
        );

        let anchor_content = self.claude_client
            .call_claude(&system_prompt, &user_message, None, Some(4096))
            .await
            .context("Failed to generate Intent_Anchor content")?;

        // Build complete artifact with frontmatter
        let artifact_id = format!("{}-intent-anchor", run_id);
        let created_at = Utc::now().to_rfc3339();
        let content_hash = self.compute_content_hash(&anchor_content);

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Intent_Anchor\"\n\
            run_id: \"{}\"\n\
            step_origin: 1\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"derived_from\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: true\n\
            author: \"scope-pattern-agent\"\n\
            governance_role: \"Observer\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            intent_summary_hash,
            intent_summary_hash,
            artifact_id, // Self-referencing: this IS the anchor
            anchor_content.trim()
        );

        info!("Intent_Anchor created successfully: {}", artifact_id);
        Ok(artifact)
    }

    /// Create Charter artifact (Step 1)
    ///
    /// Creates the governing Charter document from the Intent_Anchor.
    /// The Charter defines objectives, scope, success criteria, stakeholders,
    /// deliverables, assumptions, and risks.
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `intent_anchor_content` - Content from the Intent_Anchor artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `intent_anchor_hash` - Hash of the Intent_Anchor artifact
    /// * `execution_mode` - Execution mode (Standard / Component / Surgical)
    /// * `telemetry_profile` - Telemetry profile (Lite / Standard / Full / Learning)
    pub async fn create_charter(
        &self,
        run_id: &str,
        intent_anchor_content: &str,
        intent_anchor_id: &str,
        intent_anchor_hash: &str,
        execution_mode: &str,
        telemetry_profile: &str,
    ) -> Result<String> {
        info!("Creating Charter for run {}", run_id);

        let system_prompt = "You are creating the Charter document for Method-VI. \
            The Charter is an IMMUTABLE governing document that defines objectives, scope, \
            success criteria, stakeholders, deliverables, assumptions, and risks. \
            It must be comprehensive and serve as the reference for the entire run.";

        let user_message = format!(
            "CHARTER CREATION\n\n\
            Intent Anchor:\n{}\n\n\
            Execution Mode: {}\n\
            Telemetry Profile: {}\n\n\
            Create a complete Charter document including:\n\
            1. Run Identity (Run ID, Intent Category, Telemetry Profile, Execution Mode)\n\
            2. Objectives (Primary and Secondary)\n\
            3. Scope Definition (Inclusions, Exclusions, Constraints)\n\
            4. Success Criteria (measurable targets including CI ≥ 0.80, EV ≤ ±10%)\n\
            5. Stakeholders (roles, interests, engagement levels)\n\
            6. Deliverables (what will be produced)\n\
            7. Assumptions (what we're assuming is true)\n\
            8. Risks (potential risks with mitigation strategies)\n\n\
            Use the Method-VI Charter template format. \
            Return ONLY the content body (without YAML frontmatter).",
            intent_anchor_content,
            execution_mode,
            telemetry_profile
        );

        let charter_content = self.claude_client
            .call_claude(&system_prompt, &user_message, None, Some(4096))
            .await
            .context("Failed to generate Charter content")?;

        // Build complete artifact with frontmatter
        let artifact_id = format!("{}-charter", run_id);
        let created_at = Utc::now().to_rfc3339();
        let content_hash = self.compute_content_hash(&charter_content);

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Charter\"\n\
            run_id: \"{}\"\n\
            step_origin: 1\n\
            created_at: \"{}\"\n\
            hash: \"{}\"\n\
            parent_hash: \"{}\"\n\
            dependencies:\n\
              - artifact_id: \"{}\"\n\
                relationship: \"derived_from\"\n\
            intent_anchor_link: \"{}\"\n\
            is_immutable: true\n\
            author: \"scope-pattern-agent\"\n\
            governance_role: \"Observer\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            intent_anchor_hash,
            intent_anchor_hash,
            intent_anchor_id,
            charter_content.trim()
        );

        info!("Charter created successfully: {}", artifact_id);
        Ok(artifact)
    }

    /// Compute SHA-256 hash of content
    fn compute_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Extract a single-line field value from the response
fn extract_field(response: &str, field_name: &str) -> Option<String> {
    response
        .lines()
        .find(|line| line.contains(field_name))
        .and_then(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty() && !s.starts_with('['))
}

/// Extract a clarity indicator value (High/Medium/Low or Clear/Partial/Unclear, etc.)
fn extract_clarity_indicator(response: &str, indicator_name: &str) -> Option<String> {
    response
        .lines()
        .find(|line| line.contains(indicator_name))
        .and_then(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let value = parts[1].trim();
                // Extract just the first word (High, Medium, Low, etc.)
                Some(
                    value
                        .split_whitespace()
                        .next()
                        .unwrap_or("Medium")
                        .to_string(),
                )
            } else {
                None
            }
        })
}

/// Extract a list section (multi-line items starting with - or numbers)
fn extract_list_section(response: &str, section_header: &str) -> Vec<String> {
    let mut in_section = false;
    let mut items = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();

        // Check if we're entering the section
        if trimmed.contains(section_header) {
            in_section = true;
            continue;
        }

        // Stop if we hit another major section header
        if in_section {
            if trimmed.is_empty() {
                continue; // Skip empty lines within section
            }

            // Check if we've hit a new section header (ends with : and doesn't start with -)
            if trimmed.ends_with(':') && !trimmed.starts_with('-') {
                // This is a new section header, stop parsing this section
                break;
            }

            // Extract list items (lines starting with -, *, or numbers)
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                let item = trimmed.trim_start_matches('-').trim_start_matches('*').trim();
                if !item.is_empty() && !item.starts_with('[') {
                    items.push(item.to_string());
                }
            } else if trimmed
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_digit())
            {
                // Handle numbered lists (1. Item)
                if let Some(pos) = trimmed.find('.') {
                    let item = trimmed[pos + 1..].trim();
                    if !item.is_empty() && !item.starts_with('[') {
                        items.push(item.to_string());
                    }
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_field() {
        let response = r#"
Primary Goal: Build a user authentication system
Audience: End users of the application
Expected Outcome: Secure login functionality
"#;

        assert_eq!(
            extract_field(response, "Primary Goal:"),
            Some("Build a user authentication system".to_string())
        );
        assert_eq!(
            extract_field(response, "Audience:"),
            Some("End users of the application".to_string())
        );
    }

    #[test]
    fn test_extract_list_section() {
        let response = r#"
IN SCOPE:
- User registration
- Password hashing
- Session management

OUT OF SCOPE:
- OAuth integration
- Two-factor authentication
"#;

        let in_scope = extract_list_section(response, "IN SCOPE:");
        assert_eq!(in_scope.len(), 3);
        assert!(in_scope.contains(&"User registration".to_string()));

        let out_scope = extract_list_section(response, "OUT OF SCOPE:");
        assert_eq!(out_scope.len(), 2);
        assert!(out_scope.contains(&"OAuth integration".to_string()));
    }

    #[test]
    fn test_intent_summary_hash() {
        let summary = IntentSummary {
            artifact_id: "test-id".to_string(),
            artifact_type: "Intent_Summary".to_string(),
            run_id: "2025-12-17-Test".to_string(),
            step_origin: 0,
            created_at: "2025-12-17T10:00:00Z".to_string(),
            hash: String::new(),
            parent_hash: None,
            dependencies: vec![],
            intent_anchor_link: None,
            is_immutable: false,
            author: "scope-pattern-agent".to_string(),
            governance_role: "Observer".to_string(),
            user_request: "Test request".to_string(),
            primary_goal: "Test goal".to_string(),
            audience: "Test audience".to_string(),
            expected_outcome: "Test outcome".to_string(),
            intent_category: "Operational".to_string(),
            confidence_score: 85,
            confidence_explanation: "High confidence".to_string(),
            request_specificity: "High".to_string(),
            scope_definition_clarity: "Clear".to_string(),
            success_criteria_state: "Defined".to_string(),
            questions_for_clarification: vec![],
            likely_in_scope: vec!["Item 1".to_string()],
            likely_out_of_scope: vec!["Item 2".to_string()],
            edge_cases: vec![],
        };

        let hash = summary.compute_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_intent_summary_markdown_generation() {
        let summary = IntentSummary {
            artifact_id: "test-id".to_string(),
            artifact_type: "Intent_Summary".to_string(),
            run_id: "2025-12-17-Test".to_string(),
            step_origin: 0,
            created_at: "2025-12-17T10:00:00Z".to_string(),
            hash: "abc123".to_string(),
            parent_hash: None,
            dependencies: vec![],
            intent_anchor_link: None,
            is_immutable: false,
            author: "scope-pattern-agent".to_string(),
            governance_role: "Observer".to_string(),
            user_request: "Test request".to_string(),
            primary_goal: "Test goal".to_string(),
            audience: "Test audience".to_string(),
            expected_outcome: "Test outcome".to_string(),
            intent_category: "Operational".to_string(),
            confidence_score: 85,
            confidence_explanation: "High confidence".to_string(),
            request_specificity: "High".to_string(),
            scope_definition_clarity: "Clear".to_string(),
            success_criteria_state: "Defined".to_string(),
            questions_for_clarification: vec!["Question 1".to_string()],
            likely_in_scope: vec!["Item 1".to_string()],
            likely_out_of_scope: vec!["Item 2".to_string()],
            edge_cases: vec![],
        };

        let markdown = summary.to_markdown();
        assert!(markdown.contains("---"));
        assert!(markdown.contains("artifact_id: \"test-id\""));
        assert!(markdown.contains("# Intent Summary"));
        assert!(markdown.contains("Test request"));
    }
}
