// Integration tests for agx-eval
//
// Tests end-to-end evaluation pipeline

use agx_eval::parser::{parse_llm_response, EvaluationResult};
use agx_eval::prompt::PromptBuilder;
use serde_json::Value;

#[test]
fn test_prompt_builder_integration() {
    // Test that PromptBuilder works end-to-end
    let prompt = PromptBuilder::new()
        .with_context("Test context")
        .with_data("Test data")
        .with_instruction("Test instruction")
        .build();

    assert!(prompt.is_ok());
    let prompt_text = prompt.unwrap();
    assert!(prompt_text.contains("Test context"));
    assert!(prompt_text.contains("Test data"));
    assert!(prompt_text.contains("Test instruction"));
}

#[test]
fn test_evaluation_result_serialization() {
    // Test that EvaluationResult can be serialized to JSON
    let result = EvaluationResult {
        decision: Some("accept".to_string()),
        result: None,
        reasoning: "Test reasoning".to_string(),
        confidence: 0.9,
        evidence: vec!["evidence1".to_string()],
    };

    let json = serde_json::to_string(&result);
    assert!(json.is_ok());

    // Verify it can be deserialized back
    let deserialized: Result<EvaluationResult, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
}

#[test]
fn test_output_json_structure() {
    // Test the expected output JSON structure
    let json_str = r#"{
        "status": "success",
        "result": {
            "decision": "accept",
            "reasoning": "Good candidate",
            "confidence": 0.85,
            "evidence": ["5 years experience"]
        },
        "metadata": {
            "model": "qwen2.5:1.5b",
            "backend": "ollama",
            "latency_ms": 1234
        }
    }"#;

    let parsed: Result<Value, _> = serde_json::from_str(json_str);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["status"], "success");
    assert_eq!(value["result"]["decision"], "accept");
    assert_eq!(value["metadata"]["backend"], "ollama");
}

#[test]
fn test_error_output_json_structure() {
    // Test error output structure
    let json_str = r#"{
        "status": "error",
        "error": {
            "code": "llm_connection_failed",
            "message": "Failed to connect to Ollama",
            "details": "Connection refused"
        }
    }"#;

    let parsed: Result<Value, _> = serde_json::from_str(json_str);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["code"], "llm_connection_failed");
}

// Integration test: CV Screening workflow
#[test]
fn test_cv_screening_workflow() {
    // Simulates a CV screening use case
    let context = "Job requirements: Senior backend engineer, 3+ years Rust, distributed systems experience";
    let data = r#"{
        "name": "Jane Doe",
        "experience": "5 years Rust development",
        "projects": ["Built distributed database", "Contributed to Tokio"],
        "skills": ["Rust", "async/await", "distributed systems"]
    }"#;
    let instruction = "Evaluate candidate fit. Provide decision (accept/reject), confidence, and reasoning.";

    // Build prompt
    let prompt = PromptBuilder::new()
        .with_context(context)
        .with_data(data)
        .with_instruction(instruction)
        .build();

    assert!(prompt.is_ok());
    let prompt_text = prompt.unwrap();

    // Verify all components are present
    assert!(prompt_text.contains("Senior backend engineer"));
    assert!(prompt_text.contains("Jane Doe"));
    assert!(prompt_text.contains("Evaluate candidate fit"));

    // Simulate LLM response
    let mock_response = r#"```json
    {
        "decision": "accept",
        "reasoning": "Candidate exceeds requirements with 5 years of Rust experience and proven distributed systems work",
        "confidence": 0.92,
        "evidence": ["5 years Rust development", "Built distributed database", "Contributed to Tokio"]
    }
    ```"#;

    let result = parse_llm_response(mock_response);
    assert!(result.is_ok());

    let eval = result.unwrap();
    assert_eq!(eval.decision, Some("accept".to_string()));
    assert!(eval.confidence >= 0.9);
    assert!(eval.reasoning.contains("5 years"));
}

// Integration test: Data quality check workflow
#[test]
fn test_data_quality_check_workflow() {
    let context = "Data validation rules: age 0-120, email must contain @, phone 10 digits";
    let data = r#"{
        "user_id": 123,
        "age": -5,
        "email": "invalid-email",
        "phone": "123"
    }"#;
    let instruction = "List all validation failures with severity (high/medium/low).";

    let prompt = PromptBuilder::new()
        .with_context(context)
        .with_data(data)
        .with_instruction(instruction)
        .build();

    assert!(prompt.is_ok());
    let prompt_text = prompt.unwrap();
    assert!(prompt_text.contains("age 0-120"));
    assert!(prompt_text.contains("age\": -5"));

    // Simulate LLM response identifying validation failures
    let mock_response = r#"{
        "result": "invalid",
        "reasoning": "Multiple validation failures detected: age is negative, email missing @, phone too short",
        "confidence": 0.95,
        "evidence": ["age: -5 (must be 0-120)", "email: no @ symbol", "phone: 3 digits (need 10)"]
    }"#;

    let result = parse_llm_response(mock_response);
    assert!(result.is_ok());

    let eval = result.unwrap();
    assert_eq!(eval.result, Some("invalid".to_string()));
    assert_eq!(eval.evidence.len(), 3);
}

// Integration test: Anomaly detection workflow
#[test]
fn test_anomaly_detection_workflow() {
    let context = "Baseline metrics: API latency 50-200ms, error rate <0.1%, throughput 100-1000 RPS";
    let data = r#"{
        "timestamp": "2025-11-19T10:00:00Z",
        "latency_p50": 850,
        "latency_p99": 2500,
        "error_rate": 0.05,
        "throughput": 450
    }"#;
    let instruction = "Detect anomalies. Classify severity (low/medium/high) and recommend actions.";

    let prompt = PromptBuilder::new()
        .with_context(context)
        .with_data(data)
        .with_instruction(instruction)
        .build();

    assert!(prompt.is_ok());

    // Simulate LLM response detecting anomaly
    let mock_response = r#"{
        "decision": "anomaly_detected",
        "reasoning": "Latency significantly elevated: p50 at 850ms (baseline 50-200ms), p99 at 2.5s. Error rate acceptable.",
        "confidence": 0.88,
        "evidence": ["p50 latency 4x baseline", "p99 latency 12x baseline", "error rate within limits"]
    }"#;

    let result = parse_llm_response(mock_response);
    assert!(result.is_ok());

    let eval = result.unwrap();
    assert!(eval.reasoning.contains("elevated"));
    assert!(eval.confidence >= 0.8);
}

// Integration test: Malformed LLM response handling
#[test]
fn test_malformed_llm_response_handling() {
    // Missing required field (reasoning)
    let malformed1 = r#"{
        "decision": "accept",
        "confidence": 0.9,
        "evidence": []
    }"#;

    let result1 = parse_llm_response(malformed1);
    // Should fail due to missing reasoning field
    assert!(result1.is_err());

    // Invalid confidence (out of range)
    let malformed2 = r#"{
        "decision": "accept",
        "reasoning": "Good",
        "confidence": 1.5,
        "evidence": []
    }"#;

    let result2 = parse_llm_response(malformed2);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Confidence"));

    // Missing both decision and result
    let malformed3 = r#"{
        "reasoning": "Some reasoning",
        "confidence": 0.8,
        "evidence": []
    }"#;

    let result3 = parse_llm_response(malformed3);
    assert!(result3.is_err());
}

// Integration test: Unicode handling across pipeline
#[test]
fn test_unicode_handling_integration() {
    let context = "Evaluate international candidate: 日本語能力を確認";
    let data = "Candidate: 张三, speaks 日本語, Email: 用户@例え.jp";
    let instruction = "Does candidate meet language requirements? 🌍";

    let prompt = PromptBuilder::new()
        .with_context(context)
        .with_data(data)
        .with_instruction(instruction)
        .build();

    assert!(prompt.is_ok());
    let prompt_text = prompt.unwrap();

    // Verify unicode preserved
    assert!(prompt_text.contains("日本語"));
    assert!(prompt_text.contains("张三"));
    assert!(prompt_text.contains("🌍"));

    // Simulate LLM response with unicode
    let mock_response = r#"{
        "decision": "accept",
        "reasoning": "候補者は日本語が堪能です",
        "confidence": 0.9,
        "evidence": ["speaks 日本語", "Email: 用户@例え.jp"]
    }"#;

    let result = parse_llm_response(mock_response);
    assert!(result.is_ok());

    let eval = result.unwrap();
    assert!(eval.reasoning.contains("日本語"));
    assert!(eval.evidence.iter().any(|e| e.contains("日本語")));
}

// Integration test: Large input at size limits
#[test]
fn test_large_input_at_limits() {
    // Test with context at 10KB limit
    let large_context = "Job requirements: ".to_string() + &"X".repeat(10_240 - 19);

    // Should succeed at exactly 10KB
    let result = PromptBuilder::new()
        .with_context(&large_context)
        .with_data("test data")
        .with_instruction("test instruction")
        .build();

    assert!(result.is_ok());

    // Test with data at 1MB limit (1,048,576 bytes)
    let large_data = "Data: ".to_string() + &"Y".repeat(1_048_576 - 6);

    let result = PromptBuilder::new()
        .with_context("context")
        .with_data(&large_data)
        .with_instruction("instruction")
        .build();

    assert!(result.is_ok());
}

// Integration test: Empty and whitespace inputs
#[test]
fn test_empty_inputs_error_handling() {
    // Empty context should fail
    let result1 = PromptBuilder::new()
        .with_context("")
        .with_data("data")
        .with_instruction("instruction")
        .build();
    assert!(result1.is_err());

    // Whitespace-only context should fail
    let result2 = PromptBuilder::new()
        .with_context("   \t\n  ")
        .with_data("data")
        .with_instruction("instruction")
        .build();
    assert!(result2.is_err());

    // Empty data should fail
    let result3 = PromptBuilder::new()
        .with_context("context")
        .with_data("")
        .with_instruction("instruction")
        .build();
    assert!(result3.is_err());

    // Empty instruction should fail
    let result4 = PromptBuilder::new()
        .with_context("context")
        .with_data("data")
        .with_instruction("")
        .build();
    assert!(result4.is_err());
}

// Integration test: Special characters in all fields
#[test]
fn test_special_characters_handling() {
    let context = r#"Rules: "quotes", 'apostrophes', \backslashes\, $pecial ch@rs!"#;
    let data = r#"{"key": "value with \"escaped\" quotes"}"#;
    let instruction = "Evaluate with <angle> brackets & ampersands";

    let result = PromptBuilder::new()
        .with_context(context)
        .with_data(data)
        .with_instruction(instruction)
        .build();

    assert!(result.is_ok());
    let prompt = result.unwrap();

    // Verify special characters are preserved
    assert!(prompt.contains("\"quotes\""));
    assert!(prompt.contains("'apostrophes'"));
    assert!(prompt.contains("\\backslashes\\"));
    assert!(prompt.contains("<angle>"));
    assert!(prompt.contains("&"));
}

// Integration test: JSON response with extra text before/after
#[test]
fn test_llm_response_with_extra_text() {
    let response_with_prefix = r#"
    Here's my analysis of the candidate:

    ```json
    {
        "decision": "accept",
        "reasoning": "Strong technical background",
        "confidence": 0.85,
        "evidence": ["5 years experience"]
    }
    ```

    I hope this helps!
    "#;

    let result = parse_llm_response(response_with_prefix);
    assert!(result.is_ok());

    let eval = result.unwrap();
    assert_eq!(eval.decision, Some("accept".to_string()));
    assert_eq!(eval.confidence, 0.85);
}

// Note: Full end-to-end tests with real Ollama are ignored
// These should be run manually when Ollama is available
#[test]
#[ignore]
fn test_end_to_end_with_ollama() {
    // This test would require Ollama to be running
    // Run with: cargo test --ignored
    todo!("Implement end-to-end test with mock Ollama server");
}
