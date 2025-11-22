// src/parser.rs
//
// Response parser and validator for LLM evaluation results.
// Extracts JSON from markdown-wrapped responses and validates structure.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Evaluation result from LLM
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    /// Decision field (LLM may use "decision" or "result")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,

    /// Result field (alternative to decision)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,

    /// Reasoning/explanation for the evaluation
    pub reasoning: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Evidence supporting the decision
    #[serde(default)]
    pub evidence: Vec<String>,
}

impl EvaluationResult {
    /// Get the decision or result value (whichever is present)
    pub fn get_decision(&self) -> Option<&str> {
        self.decision.as_deref().or(self.result.as_deref())
    }

    /// Validate that the result has required fields and valid values
    fn validate(&self) -> Result<()> {
        // Reasoning is required and should not be empty
        if self.reasoning.trim().is_empty() {
            anyhow::bail!("Reasoning field is required and cannot be empty");
        }

        // At least one of decision or result should be present and non-empty
        let has_valid_decision = self
            .decision
            .as_ref()
            .map(|d| !d.trim().is_empty())
            .unwrap_or(false);
        let has_valid_result = self
            .result
            .as_ref()
            .map(|r| !r.trim().is_empty())
            .unwrap_or(false);

        if !has_valid_decision && !has_valid_result {
            anyhow::bail!("Either 'decision' or 'result' field is required and must be non-empty");
        }

        // Confidence must be in valid range
        if !(0.0..=1.0).contains(&self.confidence) {
            anyhow::bail!(
                "Confidence must be between 0.0 and 1.0, got {}",
                self.confidence
            );
        }

        Ok(())
    }
}

/// Parse LLM response into EvaluationResult
///
/// Handles responses in multiple formats:
/// 1. JSON wrapped in markdown code blocks: ```json ... ```
/// 2. Raw JSON without wrapper
///
/// # Errors
/// Returns error if:
/// - Response is too large (>100KB)
/// - Response is not valid JSON
/// - Required fields are missing
/// - Field values are invalid
pub fn parse_llm_response(raw: &str) -> Result<EvaluationResult> {
    // Security: Validate input size to prevent DoS attacks (CLAUDE.md §5.2)
    const MAX_RESPONSE_SIZE: usize = 100 * 1024; // 100KB
    if raw.len() > MAX_RESPONSE_SIZE {
        anyhow::bail!(
            "Response too large: {} bytes (max {} bytes)",
            raw.len(),
            MAX_RESPONSE_SIZE
        );
    }

    let json_str = extract_json_from_markdown(raw)?;

    let result: EvaluationResult =
        serde_json::from_str(&json_str).context("Failed to parse JSON response from LLM")?;

    result.validate()?;

    Ok(result)
}

/// Extract JSON from markdown code blocks or return raw string
///
/// Looks for patterns like:
/// - ```json\n{ ... }\n```
/// - ```\n{ ... }\n```
///
/// If no markdown wrapper found, returns trimmed input
fn extract_json_from_markdown(raw: &str) -> Result<String> {
    let trimmed = raw.trim();

    // Try to find ```json ... ``` block
    if let Some(start) = trimmed.find("```json") {
        if let Some(end_idx) = trimmed[start + 7..].find("```") {
            let json_start = start + 7; // len("```json")
            let json_end = start + 7 + end_idx;
            return Ok(trimmed[json_start..json_end].trim().to_string());
        }
    } else if let Some(start) = trimmed.find("```") {
        // Try to find ``` ... ``` block (no language specified)
        // Use else if to prevent fallthrough after checking ```json
        let after_first = start + 3;
        // Skip the language identifier line if present
        let json_start = if let Some(newline) = trimmed[after_first..].find('\n') {
            after_first + newline + 1
        } else {
            after_first
        };

        if let Some(end_idx) = trimmed[json_start..].find("```") {
            let json_end = json_start + end_idx;
            return Ok(trimmed[json_start..json_end].trim().to_string());
        }
    }

    // No markdown wrapper, assume entire response is JSON
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_with_markdown_wrapper() {
        let raw = r#"
Here is my evaluation:

```json
{
  "decision": "accept",
  "reasoning": "Candidate meets all requirements",
  "confidence": 0.9,
  "evidence": ["5 years Rust", "Distributed systems"]
}
```

Hope this helps!
"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.decision, Some("accept".to_string()));
        assert_eq!(result.reasoning, "Candidate meets all requirements");
        assert_eq!(result.confidence, 0.9);
        assert_eq!(result.evidence.len(), 2);
    }

    #[test]
    fn test_parse_raw_json_without_wrapper() {
        let raw = r#"{
  "result": "compliant",
  "reasoning": "All expenses within policy limits",
  "confidence": 0.85,
  "evidence": ["All receipts attached"]
}"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.result, Some("compliant".to_string()));
        assert_eq!(result.reasoning, "All expenses within policy limits");
        assert_eq!(result.confidence, 0.85);
    }

    #[test]
    fn test_parse_markdown_without_json_tag() {
        let raw = r#"
```
{
  "decision": "reject",
  "reasoning": "Missing required skills",
  "confidence": 0.95,
  "evidence": []
}
```
"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.decision, Some("reject".to_string()));
        assert_eq!(result.confidence, 0.95);
    }

    #[test]
    fn test_missing_reasoning_fails() {
        let raw = r#"{
  "decision": "accept",
  "reasoning": "",
  "confidence": 0.9,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Reasoning field is required"));
    }

    #[test]
    fn test_missing_decision_and_result_fails() {
        let raw = r#"{
  "reasoning": "Some reasoning",
  "confidence": 0.9,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either 'decision' or 'result' field is required"));
    }

    #[test]
    fn test_invalid_confidence_too_low() {
        let raw = r#"{
  "decision": "accept",
  "reasoning": "Good candidate",
  "confidence": -0.1,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Confidence must be between"));
    }

    #[test]
    fn test_invalid_confidence_too_high() {
        let raw = r#"{
  "decision": "accept",
  "reasoning": "Good candidate",
  "confidence": 1.5,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Confidence must be between"));
    }

    #[test]
    fn test_confidence_edge_cases() {
        // Test 0.0
        let raw = r#"{
  "decision": "uncertain",
  "reasoning": "Not enough information",
  "confidence": 0.0,
  "evidence": []
}"#;
        assert!(parse_llm_response(raw).is_ok());

        // Test 1.0
        let raw = r#"{
  "decision": "certain",
  "reasoning": "Absolutely sure",
  "confidence": 1.0,
  "evidence": []
}"#;
        assert!(parse_llm_response(raw).is_ok());
    }

    #[test]
    fn test_malformed_json() {
        let raw = r#"{
  "decision": "accept"
  "reasoning": "Missing comma"
  "confidence": 0.9
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse JSON"));
    }

    #[test]
    fn test_evidence_optional() {
        let raw = r#"{
  "decision": "accept",
  "reasoning": "Good candidate",
  "confidence": 0.9
}"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.evidence.len(), 0);
    }

    #[test]
    fn test_get_decision_prefers_decision_field() {
        let result = EvaluationResult {
            decision: Some("accept".to_string()),
            result: Some("approved".to_string()),
            reasoning: "test".to_string(),
            confidence: 0.9,
            evidence: vec![],
        };

        assert_eq!(result.get_decision(), Some("accept"));
    }

    #[test]
    fn test_get_decision_falls_back_to_result() {
        let result = EvaluationResult {
            decision: None,
            result: Some("approved".to_string()),
            reasoning: "test".to_string(),
            confidence: 0.9,
            evidence: vec![],
        };

        assert_eq!(result.get_decision(), Some("approved"));
    }

    #[test]
    fn test_extract_json_with_extra_text() {
        let raw = r#"
Sure, I can help with that evaluation.

```json
{
  "decision": "accept",
  "reasoning": "Meets requirements",
  "confidence": 0.8,
  "evidence": []
}
```

Let me know if you need anything else!
"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.decision, Some("accept".to_string()));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = EvaluationResult {
            decision: Some("accept".to_string()),
            result: None,
            reasoning: "Test reasoning".to_string(),
            confidence: 0.75,
            evidence: vec!["evidence1".to_string(), "evidence2".to_string()],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EvaluationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    // Security tests

    #[test]
    fn test_response_size_limit() {
        let large_response = "x".repeat(101 * 1024); // > 100KB

        let result = parse_llm_response(&large_response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Response too large"));
    }

    #[test]
    fn test_empty_decision_string_fails() {
        let raw = r#"{
  "decision": "   ",
  "reasoning": "Some reasoning",
  "confidence": 0.9,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either 'decision' or 'result' field is required and must be non-empty"));
    }

    #[test]
    fn test_empty_result_string_fails() {
        let raw = r#"{
  "result": "",
  "reasoning": "Some reasoning",
  "confidence": 0.9,
  "evidence": []
}"#;

        let result = parse_llm_response(raw);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either 'decision' or 'result' field is required and must be non-empty"));
    }

    #[test]
    fn test_unicode_in_decision() {
        let raw = r#"{
  "decision": "✅ accept",
  "reasoning": "Candidate 很好",
  "confidence": 0.9,
  "evidence": ["Speaks 日本語", "Expert in Rust 🦀"]
}"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.decision, Some("✅ accept".to_string()));
        assert_eq!(result.reasoning, "Candidate 很好");
        assert_eq!(result.evidence[0], "Speaks 日本語");
        assert_eq!(result.evidence[1], "Expert in Rust 🦀");
    }

    #[test]
    fn test_unicode_emoji_in_all_fields() {
        let raw = r#"```json
{
  "result": "🎉 approved",
  "reasoning": "Perfect fit 💯",
  "confidence": 1.0,
  "evidence": ["Great skills 🚀", "Team player 👍"]
}
```"#;

        let result = parse_llm_response(raw).unwrap();
        assert_eq!(result.result, Some("🎉 approved".to_string()));
        assert!(result.reasoning.contains("💯"));
        assert_eq!(result.evidence.len(), 2);
    }
}
