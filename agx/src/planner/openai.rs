use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

use super::backend::ModelBackend;
use super::types::{ChatMessage, GeneratedPlan, ModelError, PlanContext};

pub struct OpenAIBackend {
    client: Client,
    model: String,
    api_key: String,
}

impl OpenAIBackend {
    pub fn new(model: String) -> Self {
        let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
        Self {
            client: Client::new(),
            model,
            api_key,
        }
    }
}

#[async_trait]
impl ModelBackend for OpenAIBackend {
    async fn generate_plan(
        &self,
        instruction: &str,
        context: &PlanContext,
    ) -> Result<GeneratedPlan, ModelError> {
        // 1. Build the prompt using shared logic
        let system_prompt = super::prompts::build_system_prompt(context);
        let user_prompt = super::prompts::build_user_prompt(instruction, context);

        let history = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_prompt),
        ];

        // 2. Call Chat API
        let response_text = self.chat(&history, context).await?;

        // 3. Parse JSON
        // Clean up markdown code blocks if present
        let clean_json = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let plan: GeneratedPlan = serde_json::from_str(clean_json).map_err(|e| {
            ModelError::ParseError(format!("Failed to parse OpenAI response: {}. Response: {}", e, clean_json))
        })?;

        Ok(plan)
    }

    fn backend_type(&self) -> &'static str {
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn health_check(&self) -> Result<(), ModelError> {
        if self.api_key.is_empty() {
            return Err(ModelError::ConfigError("OPENAI_API_KEY not set".to_string()));
        }
        Ok(())
    }

    async fn chat(
        &self,
        history: &[ChatMessage],
        _context: &PlanContext,
    ) -> Result<String, ModelError> {
        if self.api_key.is_empty() {
            return Err(ModelError::ConfigError("OPENAI_API_KEY not set".to_string()));
        }

        let messages: Vec<Value> = history
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content
                })
            })
            .collect();

        let body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7
        });

        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::InferenceError(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::InferenceError(format!(
                "OpenAI API error: {} - {}",
                status, text
            )));
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| ModelError::InferenceError(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| ModelError::ParseError("Invalid response format from OpenAI".to_string()))?;

        Ok(content.to_string())
    }
}
