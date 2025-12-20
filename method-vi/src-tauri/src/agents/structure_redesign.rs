use anyhow::{Context, Result};
use chrono::Utc;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::api::anthropic::AnthropicClient;

/// Structure & Redesign Agent
///
/// Responsible for:
/// - Architecture Map design (Step 1)
/// - Framework architecture design (Step 5)
/// - Section isolation and revision (Step 5.5)
/// - Structural coherence validation
pub struct StructureRedesignAgent {
    /// Claude API client for architecture design
    api_client: AnthropicClient,
}

impl StructureRedesignAgent {
    /// Create a new Structure & Redesign Agent
    pub fn new(api_key: String) -> Result<Self> {
        let api_client = AnthropicClient::new(api_key)
            .context("Failed to create Anthropic API client")?;

        Ok(Self { api_client })
    }

    /// Create Architecture Map artifact (Step 1)
    ///
    /// Designs the process architecture for the run based on Charter and Mode Profile.
    ///
    /// # Arguments
    /// * `run_id` - The run identifier
    /// * `charter_content` - The Charter artifact content
    /// * `charter_hash` - Hash of the Charter artifact
    /// * `intent_anchor_id` - ID of the Intent_Anchor artifact
    /// * `mode_profile` - Execution mode (Standard / Component / Surgical)
    pub async fn create_architecture_map(
        &self,
        run_id: &str,
        charter_content: &str,
        charter_hash: &str,
        intent_anchor_id: &str,
        mode_profile: &str,
    ) -> Result<String> {
        info!("Creating Architecture Map for run {}", run_id);

        let system_prompt = "You are a process architecture expert for Method-VI. \
            Design a process architecture map that defines the phases, loops, reflection points, \
            telemetry anchors, and checkpoints for this run. Follow the Architecture_Map template \
            from Method-VI specification.";

        let user_message = format!(
            "ARCHITECTURE MAP DESIGN\n\n\
            Charter:\n{}\n\n\
            Mode Profile: {}\n\n\
            Design a complete Architecture Map including:\n\
            1. Process Overview with Flow Geometry (Linear/Cyclic/Branching)\n\
            2. Phase Definitions with purpose, inputs, outputs, primary agent\n\
            3. Reflection Cadence (scheduled reflection points and triggers)\n\
            4. Telemetry Anchors (metric collection points)\n\
            5. Checkpoint Configuration (gate density and human decision points)\n\n\
            Use the Method-VI Architecture_Map template format with YAML frontmatter and markdown body.\n\
            Include clear rationale for why this geometry fits the charter objectives.",
            charter_content,
            mode_profile
        );

        let architecture_content = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(4096))
            .await
            .context("Failed to generate Architecture Map content")?;

        // Build complete artifact with frontmatter
        let artifact_id = format!("{}-architecture-map", run_id);
        let created_at = Utc::now().to_rfc3339();

        let content_body = self.extract_content_body(&architecture_content);
        let content_hash = self.calculate_hash(&content_body);

        let artifact = format!(
            "---\n\
            artifact_id: \"{}\"\n\
            artifact_type: \"Architecture_Map\"\n\
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
            author: \"structure-redesign-agent\"\n\
            governance_role: \"Observer\"\n\
            ---\n\n\
            {}",
            artifact_id,
            run_id,
            created_at,
            content_hash,
            charter_hash,
            charter_hash,
            intent_anchor_id,
            intent_anchor_id,
            content_body
        );

        info!("Architecture Map created successfully: {}", artifact_id);
        Ok(artifact)
    }

    /// Extract content body from LLM response (removes any wrapping text)
    fn extract_content_body(&self, response: &str) -> String {
        // If response already has frontmatter, extract just the body
        if let Some(start) = response.find("---\n") {
            if let Some(end) = response[start + 4..].find("\n---\n") {
                // Skip past second --- delimiter
                let body_start = start + 4 + end + 5;
                return response[body_start..].trim().to_string();
            }
        }

        // If no frontmatter found, return as-is (we'll add frontmatter)
        response.trim().to_string()
    }

    /// Calculate SHA-256 hash of content
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Create Framework Draft Architecture Outline (Step 5)
    ///
    /// Designs the framework architecture from Core Thesis and synthesis.
    pub async fn create_framework_architecture(
        &self,
        run_id: &str,
        core_thesis: &str,
        synthesis: &str,
    ) -> Result<String> {
        debug!("Creating Framework Architecture for run {}", run_id);

        let system_prompt = "You are the Structure & Redesign Agent for Method-VI, operating under the AUDITOR governance role.\n\n\
            Your task is to design a comprehensive framework architecture that transforms the synthesis artifacts \
            from Step 4 into a structured, coherent document framework.\n\n\
            Key responsibilities:\n\
            1. Create logical section boundaries with clear purposes and dependencies\n\
            2. Define transition logic that ensures smooth flow between sections\n\
            3. Maintain governance coherence (PCI, CI, EV, SEC, GLR)\n\
            4. Normalize headers and ensure term consistency with the Glossary\n\
            5. Design architecture that serves the Core Thesis (IMMUTABLE)\n\n\
            The framework you design must:\n\
            - Organize content into a clear, logical structure\n\
            - Make dependencies explicit\n\
            - Support the Model Geometry identified in synthesis\n\
            - Honor the Causal Spine's logic flow\n\
            - Use terms consistently per the Glossary\n\
            - Align with the North Star Narrative direction";

        let user_message = format!(
            "STEP 5: FRAMEWORK ARCHITECTURE DESIGN\n\
            GOVERNANCE ROLE: Auditor\n\
            EXECUTION MODE: Standard\n\n\
            You are designing the framework architecture for run: {}\n\n\
            === IMMUTABLE CORE THESIS ===\n\
            (This is locked from Step 4 - DO NOT modify or reinterpret)\n\n\
            {}\n\n\
            === SYNTHESIS ARTIFACTS FROM STEP 4 ===\n\n\
            {}\n\n\
            === YOUR TASK ===\n\n\
            Design a Framework Architecture following this structure:\n\n\
            1. FRAMEWORK OVERVIEW\n\
               - Framework Type (description)\n\
               - Model Geometry (from synthesis)\n\
               - Section Count\n\
               - Execution Mode (Standard)\n\n\
            2. SECTION FUNCTION MAP\n\
               Create a table with columns:\n\
               - Section # (numbered sequentially)\n\
               - Section Name (clear, descriptive)\n\
               - Purpose (what this section accomplishes)\n\
               - Dependencies (which sections it depends on, or \"None\")\n\n\
            3. FRAMEWORK CONTENT\n\
               For each section, provide:\n\
               - Section Name (as header)\n\
               - Purpose (what this section accomplishes)\n\
               - Content Outline (key points/structure, not full text)\n\
               - Transition to Next (how this section connects to the next)\n\n\
            4. TRANSITION LOGIC MAP\n\
               - Explain how sections connect and flow\n\
               - Justify the sequence\n\
               - Show any branching or cyclic patterns (if Model Geometry requires)\n\n\
            5. GOVERNANCE AUDIT\n\
               - Purpose Coherence Index (PCI): Are section purposes aligned with Core Thesis?\n\
               - Content Identity (CI): Does structure preserve synthesis meaning?\n\
               - Structural Coherence (SEC): Are dependencies logical and minimal?\n\
               - Glossary Respect (GLR): Are terms used consistently?\n\n\
            6. ARCHITECTURE OUTLINE\n\
               Provide a visual representation of the framework structure.\n\n\
            Your framework architecture must:\n\
            - Serve the Core Thesis (do not modify or reinterpret it)\n\
            - Follow the Causal Spine logic\n\
            - Use Glossary terms consistently\n\
            - Support the identified Model Geometry\n\
            - Create clear, logical section boundaries\n\
            - Minimize coupling (low dependencies)\n\
            - Maximize coherence (sections support each other)\n\n\
            Output your framework architecture design now.",
            run_id,
            core_thesis,
            synthesis
        );

        let architecture = self.api_client
            .call_claude(system_prompt, &user_message, None, Some(8192))
            .await
            .context("Failed to generate framework architecture")?;

        Ok(architecture)
    }

    /// Validate section isolation for Component Mode (Step 5.5)
    ///
    /// Checks circuit breaker conditions before allowing section-level revision.
    pub fn validate_section_isolation(
        &self,
        section_id: &str,
        dependencies: &[String],
        is_on_critical_path: bool,
        is_first_component_revision: bool,
    ) -> Result<()> {
        debug!("Validating section isolation for {}", section_id);

        // Circuit Breaker Check 1: Section Isolation (≤2 direct dependencies)
        if dependencies.len() > 2 {
            return Err(anyhow::anyhow!(
                "Section has {} dependencies (max 2 allowed for Component Mode). \
                This section is too coupled for isolated revision.",
                dependencies.len()
            ));
        }

        // Circuit Breaker Check 2: Coherence Spine Impact (not on Critical Path)
        if is_on_critical_path {
            return Err(anyhow::anyhow!(
                "Section is on the Critical Path in the Coherence Spine. \
                Revising it would impact dependent artifacts. Not safe for Component Mode."
            ));
        }

        // Circuit Breaker Check 3: Cumulative Change (first Component revision this run)
        if !is_first_component_revision {
            return Err(anyhow::anyhow!(
                "This is not the first Component Mode revision in this run. \
                Cumulative changes increase risk. Consider Standard Mode instead."
            ));
        }

        info!("Section {} passed all circuit breaker checks", section_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content_body() {
        let agent = StructureRedesignAgent {
            api_client: AnthropicClient::new("test-key".to_string()).unwrap(),
        };

        let response_with_frontmatter = "---\nartifact_id: test\n---\n\n# Content\nBody here";
        let body = agent.extract_content_body(response_with_frontmatter);
        assert_eq!(body, "# Content\nBody here");

        let response_without_frontmatter = "# Architecture Map\nSome content";
        let body = agent.extract_content_body(response_without_frontmatter);
        assert_eq!(body, "# Architecture Map\nSome content");
    }

    #[test]
    fn test_calculate_hash() {
        let agent = StructureRedesignAgent {
            api_client: AnthropicClient::new("test-key".to_string()).unwrap(),
        };

        let content = "Test content";
        let hash = agent.calculate_hash(content);

        // SHA-256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same content should produce same hash
        let hash2 = agent.calculate_hash(content);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_validate_section_isolation() {
        let agent = StructureRedesignAgent {
            api_client: AnthropicClient::new("test-key".to_string()).unwrap(),
        };

        // Should pass with ≤2 dependencies, not on critical path, first revision
        assert!(agent.validate_section_isolation(
            "section-1",
            &["dep1".to_string(), "dep2".to_string()],
            false,
            true
        ).is_ok());

        // Should fail with >2 dependencies
        assert!(agent.validate_section_isolation(
            "section-1",
            &["dep1".to_string(), "dep2".to_string(), "dep3".to_string()],
            false,
            true
        ).is_err());

        // Should fail if on critical path
        assert!(agent.validate_section_isolation(
            "section-1",
            &["dep1".to_string()],
            true,
            true
        ).is_err());

        // Should fail if not first revision
        assert!(agent.validate_section_isolation(
            "section-1",
            &["dep1".to_string()],
            false,
            false
        ).is_err());
    }
}
