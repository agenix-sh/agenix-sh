// src/llm.rs
//
// Ollama LLM client for sending prompts and receiving responses.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for interacting with Ollama API
#[derive(Debug, Clone)]
pub struct OllamaClient {
    endpoint: String,
    model: String,
    temperature: f32,
    max_tokens: usize,
    client: reqwest::Client,
}

/// Request payload for Ollama /api/generate endpoint
#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: GenerateOptions,
}

/// Options for generation
#[derive(Debug, Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: usize,
}

/// Response from Ollama /api/generate endpoint
#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
    #[allow(dead_code)]
    model: Option<String>,
    #[allow(dead_code)]
    done: Option<bool>,
}

impl OllamaClient {
    /// Create a new OllamaClient
    ///
    /// # Arguments
    /// - `endpoint`: Ollama API endpoint (e.g., "http://localhost:11434")
    /// - `model`: Model name (e.g., "qwen2.5:1.5b")
    /// - `temperature`: Sampling temperature (0.0-1.0)
    /// - `max_tokens`: Maximum tokens to generate
    ///
    /// # Errors
    /// Returns error if:
    /// - Temperature is not in valid range [0.0, 1.0]
    /// - HTTP client cannot be built
    pub fn new(endpoint: &str, model: &str, temperature: f32, max_tokens: usize) -> Result<Self> {
        // Validate temperature range
        if !(0.0..=1.0).contains(&temperature) {
            anyhow::bail!(
                "Temperature must be between 0.0 and 1.0, got {}",
                temperature
            );
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
            client,
        })
    }

    /// Create a new OllamaClient with custom timeout
    ///
    /// # Errors
    /// Returns error if:
    /// - Temperature is not in valid range [0.0, 1.0]
    /// - HTTP client cannot be built
    #[allow(dead_code)] // Part of public API, used in tests
    pub fn with_timeout(
        endpoint: &str,
        model: &str,
        temperature: f32,
        max_tokens: usize,
        timeout_secs: u64,
    ) -> Result<Self> {
        // Validate temperature range
        if !(0.0..=1.0).contains(&temperature) {
            anyhow::bail!(
                "Temperature must be between 0.0 and 1.0, got {}",
                temperature
            );
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
            client,
        })
    }

    /// Generate a response from the LLM for the given prompt
    ///
    /// # Errors
    /// Returns error if:
    /// - Connection to Ollama fails
    /// - Request times out
    /// - Response is malformed
    /// - Response missing required fields
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: GenerateOptions {
                temperature: self.temperature,
                num_predict: self.max_tokens,
            },
        };

        let url = format!("{}/api/generate", self.endpoint);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context(format!(
                "Failed to connect to Ollama at {}. Is Ollama running?",
                self.endpoint
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Ollama API returned error status {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            );
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response as JSON")?;

        Ok(generate_response.response)
    }

    /// Get the configured endpoint
    #[allow(dead_code)] // Part of public API, used in tests
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get the configured model
    #[allow(dead_code)] // Part of public API, used in tests
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// Get Ollama endpoint from environment or use default
pub fn get_ollama_endpoint() -> String {
    std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", 0.1, 500)
            .expect("Failed to create client");

        assert_eq!(client.endpoint(), "http://localhost:11434");
        assert_eq!(client.model(), "qwen2.5:1.5b");
    }

    #[test]
    fn test_new_client_strips_trailing_slash() {
        let client = OllamaClient::new("http://localhost:11434/", "qwen2.5:1.5b", 0.1, 500)
            .expect("Failed to create client");

        assert_eq!(client.endpoint(), "http://localhost:11434");
    }

    #[test]
    fn test_get_ollama_endpoint_default() {
        // Test default when env var is not set
        // Don't modify env vars to avoid test interference
        let default_endpoint = "http://localhost:11434";

        // If OLLAMA_ENDPOINT is not set, this should return default
        match std::env::var("OLLAMA_ENDPOINT") {
            Ok(_) => {
                // Env var is set by another test or environment, skip this assertion
                // Just verify the function works
                let _ = get_ollama_endpoint();
            }
            Err(_) => {
                let endpoint = get_ollama_endpoint();
                assert_eq!(endpoint, default_endpoint);
            }
        }
    }

    #[test]
    fn test_get_ollama_endpoint_logic() {
        // Test the logic without relying on global env state
        // Verify that when var exists it's used, otherwise default
        let original = std::env::var("OLLAMA_ENDPOINT").ok();

        // Test with custom value
        std::env::set_var("OLLAMA_ENDPOINT", "http://custom:8080");
        let endpoint = get_ollama_endpoint();
        assert_eq!(endpoint, "http://custom:8080");

        // Restore original state
        match original {
            Some(val) => std::env::set_var("OLLAMA_ENDPOINT", val),
            None => std::env::remove_var("OLLAMA_ENDPOINT"),
        }
    }

    #[test]
    fn test_client_with_custom_timeout() {
        let client =
            OllamaClient::with_timeout("http://localhost:11434", "qwen2.5:1.5b", 0.1, 500, 60)
                .expect("Failed to create client");

        assert_eq!(client.endpoint(), "http://localhost:11434");
        assert_eq!(client.model(), "qwen2.5:1.5b");
    }

    #[test]
    fn test_temperature_validation_too_low() {
        let result = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", -0.1, 500);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between"));
    }

    #[test]
    fn test_temperature_validation_too_high() {
        let result = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", 1.1, 500);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between"));
    }

    #[test]
    fn test_temperature_validation_edge_cases() {
        // Test valid edge cases
        let client_zero = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", 0.0, 500);
        assert!(client_zero.is_ok());

        let client_one = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", 1.0, 500);
        assert!(client_one.is_ok());
    }

    // Note: Integration tests with real Ollama would go in tests/ directory
    // These are kept as unit tests with documentation for manual testing

    #[tokio::test]
    #[ignore] // Only run manually when Ollama is running
    async fn test_generate_with_real_ollama() {
        let client = OllamaClient::new("http://localhost:11434", "qwen2.5:1.5b", 0.1, 50)
            .expect("Failed to create client");

        let prompt = "Say hello in exactly 3 words.";
        let response = client.generate(prompt).await;

        assert!(response.is_ok());
        let text = response.unwrap();
        assert!(!text.is_empty());
        println!("Response: {}", text);
    }

    #[tokio::test]
    async fn test_generate_connection_error() {
        // Use an endpoint that won't respond
        let client = OllamaClient::with_timeout(
            "http://localhost:9999",
            "qwen2.5:1.5b",
            0.1,
            500,
            1, // 1 second timeout
        )
        .expect("Failed to create client");

        let result = client.generate("test").await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to connect") || err_msg.contains("Connection refused"),
            "Expected connection error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_generate_request_serialization() {
        let request = GenerateRequest {
            model: "qwen2.5:1.5b".to_string(),
            prompt: "Test prompt".to_string(),
            stream: false,
            options: GenerateOptions {
                temperature: 0.1,
                num_predict: 500,
            },
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["model"], "qwen2.5:1.5b");
        assert_eq!(json["prompt"], "Test prompt");
        assert_eq!(json["stream"], false);
        // Floating point comparison - check approximate equality
        let temp = json["options"]["temperature"].as_f64().unwrap();
        assert!(
            (temp - 0.1).abs() < 0.001,
            "Temperature should be approximately 0.1"
        );
        assert_eq!(json["options"]["num_predict"], 500);
    }

    #[test]
    fn test_generate_response_deserialization() {
        let json = r#"{
            "model": "qwen2.5:1.5b",
            "response": "Hello, world!",
            "done": true
        }"#;

        let response: GenerateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response, "Hello, world!");
    }

    #[test]
    fn test_generate_response_minimal() {
        // Ollama might return minimal response
        let json = r#"{"response": "Hello"}"#;

        let response: GenerateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response, "Hello");
    }
}
