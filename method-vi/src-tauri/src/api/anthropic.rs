use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const REQUEST_TIMEOUT_SECS: u64 = 120;

/// Anthropic API client for calling Claude
pub struct AnthropicClient {
    api_key: String,
    client: reqwest::Client,
}

/// Message content for Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Message structure for Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Request body for Claude API
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Usage information from API response
#[derive(Debug, Clone, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// API response content
#[derive(Debug, Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Response from Claude API
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ResponseContent>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

/// Error response from Claude API
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl AnthropicClient {
    /// Create a new Anthropic API client
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(AnthropicClient { api_key, client })
    }

    /// Call Claude API with system prompt and user message
    ///
    /// # Arguments
    /// * `system_prompt` - System prompt to set Claude's behavior
    /// * `user_message` - User message to send to Claude
    /// * `model` - Model to use (default: claude-sonnet-4-20250514)
    /// * `max_tokens` - Maximum tokens to generate (default: 4096)
    ///
    /// # Returns
    /// The text response from Claude
    ///
    /// # Errors
    /// Returns error if:
    /// - API key is missing or invalid
    /// - Network request fails
    /// - API returns an error (rate limiting, invalid request, etc.)
    /// - Response cannot be parsed
    pub async fn call_claude(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<&str>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String> {
        let model = model.unwrap_or(DEFAULT_MODEL);
        let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        debug!("Calling Claude API with model: {}, max_tokens: {}, temperature: {:?}",
               model, max_tokens, temperature);
        debug!("System prompt length: {} chars", system_prompt.len());
        debug!("User message length: {} chars", user_message.len());

        // Build request
        let request_body = ClaudeRequest {
            model: model.to_string(),
            max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
            system: if system_prompt.is_empty() {
                None
            } else {
                Some(system_prompt.to_string())
            },
            temperature,
        };

        // Make API request
        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        debug!("API response status: {}", status);

        if !status.is_success() {
            return self.handle_error_response(status, response).await;
        }

        // Parse successful response
        let response_body = response
            .text()
            .await
            .context("Failed to read response body")?;

        let claude_response: ClaudeResponse = serde_json::from_str(&response_body)
            .context("Failed to parse Claude API response")?;

        // Log usage for cost tracking
        self.log_api_usage(&claude_response);

        // Extract text from response
        if let Some(content) = claude_response.content.first() {
            info!(
                "Claude API call successful - model: {}, input: {}, output: {} tokens",
                claude_response.model,
                claude_response.usage.input_tokens,
                claude_response.usage.output_tokens
            );

            Ok(content.text.clone())
        } else {
            anyhow::bail!("No content in Claude API response")
        }
    }

    /// Handle error responses from the API
    async fn handle_error_response(
        &self,
        status: reqwest::StatusCode,
        response: reqwest::Response,
    ) -> Result<String> {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());

        // Try to parse as structured error
        if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_body) {
            error!(
                "Claude API error - status: {}, type: {}, message: {}",
                status, error_response.error_type, error_response.message
            );

            match status.as_u16() {
                400 => anyhow::bail!(
                    "Bad request to Claude API: {}",
                    error_response.message
                ),
                401 => anyhow::bail!(
                    "Invalid API key - please check ANTHROPIC_API_KEY: {}",
                    error_response.message
                ),
                403 => anyhow::bail!("Access forbidden: {}", error_response.message),
                404 => anyhow::bail!("API endpoint not found: {}", error_response.message),
                429 => {
                    warn!("Rate limit exceeded - consider implementing retry logic");
                    anyhow::bail!("Rate limit exceeded: {}", error_response.message)
                }
                500..=599 => anyhow::bail!(
                    "Anthropic API server error: {}",
                    error_response.message
                ),
                _ => anyhow::bail!(
                    "Claude API error ({}): {}",
                    status,
                    error_response.message
                ),
            }
        }

        // Fallback for unparseable errors
        error!("Claude API error - status: {}, body: {}", status, error_body);
        anyhow::bail!("Claude API error ({}): {}", status, error_body)
    }

    /// Log API usage for cost tracking
    fn log_api_usage(&self, response: &ClaudeResponse) {
        let input_tokens = response.usage.input_tokens;
        let output_tokens = response.usage.output_tokens;
        let total_tokens = input_tokens + output_tokens;

        // Approximate costs (as of 2025)
        // Claude Sonnet 4: $3 per 1M input tokens, $15 per 1M output tokens
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
        let total_cost = input_cost + output_cost;

        info!(
            "API Usage - Model: {}, Input: {} tokens (${:.4}), Output: {} tokens (${:.4}), Total: {} tokens (${:.4}), Stop: {:?}",
            response.model,
            input_tokens,
            input_cost,
            output_tokens,
            output_cost,
            total_tokens,
            total_cost,
            response.stop_reason
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: Some("You are a helpful assistant".to_string()),
            temperature: Some(0.0),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-sonnet-4-20250514"));
        assert!(json.contains("Hello"));
        assert!(json.contains("You are a helpful assistant"));
        assert!(json.contains("0.0"));  // Temperature should be included
    }

    #[test]
    fn test_request_without_system() {
        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            system: None,
            temperature: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("system"));
        assert!(!json.contains("temperature"));  // Should not be included when None
    }

    #[test]
    fn test_client_creation() {
        let client = AnthropicClient::new("test-key".to_string());
        assert!(client.is_ok());
    }

    // Note: Actual API tests would require a valid API key and should be integration tests
    // They are commented out to avoid hitting the API during unit tests

    /*
    #[tokio::test]
    async fn test_api_call_with_real_key() {
        // Only run if ANTHROPIC_API_KEY is set
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let client = AnthropicClient::new(api_key).unwrap();

            let response = client
                .call_claude(
                    "You are a helpful assistant.",
                    "Say hello in one word.",
                    None,
                    Some(10),
                    None,
                )
                .await;

            assert!(response.is_ok());
            let text = response.unwrap();
            assert!(!text.is_empty());
        }
    }
    */
}
