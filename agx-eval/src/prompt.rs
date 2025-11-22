// src/prompt.rs
//
// Generic prompt builder for LLM evaluation.
// Combines user context, data, and instruction into a structured prompt.

use anyhow::Result;

/// Builder for constructing evaluation prompts
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    context: String,
    data: String,
    instruction: String,
}

impl PromptBuilder {
    /// Create a new PromptBuilder with empty fields
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the context (background info, criteria, domain knowledge)
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = context.to_string();
        self
    }

    /// Set the data to evaluate (typically from stdin)
    pub fn with_data(mut self, data: &str) -> Self {
        self.data = data.to_string();
        self
    }

    /// Set the evaluation instruction (user's prompt/question)
    pub fn with_instruction(mut self, instruction: &str) -> Self {
        self.instruction = instruction.to_string();
        self
    }

    /// Build the final prompt string
    pub fn build(self) -> Result<String> {
        // Validate that all required fields are provided
        if self.context.trim().is_empty() {
            anyhow::bail!("Context cannot be empty");
        }
        if self.data.trim().is_empty() {
            anyhow::bail!("Data cannot be empty");
        }
        if self.instruction.trim().is_empty() {
            anyhow::bail!("Instruction cannot be empty");
        }

        // Security: Validate input sizes (CLAUDE.md Section 5.2)
        const MAX_CONTEXT_SIZE: usize = 10 * 1024; // 10KB
        const MAX_INSTRUCTION_SIZE: usize = 1024; // 1KB
        const MAX_DATA_SIZE: usize = 1024 * 1024; // 1MB

        if self.context.len() > MAX_CONTEXT_SIZE {
            anyhow::bail!(
                "Context too large: {} bytes (max {} bytes)",
                self.context.len(),
                MAX_CONTEXT_SIZE
            );
        }
        if self.instruction.len() > MAX_INSTRUCTION_SIZE {
            anyhow::bail!(
                "Instruction too large: {} bytes (max {} bytes)",
                self.instruction.len(),
                MAX_INSTRUCTION_SIZE
            );
        }
        if self.data.len() > MAX_DATA_SIZE {
            anyhow::bail!(
                "Data too large: {} bytes (max {} bytes)",
                self.data.len(),
                MAX_DATA_SIZE
            );
        }

        // Security: Validate no null bytes (CLAUDE.md Section 5.1)
        if self.context.contains('\0') {
            anyhow::bail!("Context contains null bytes");
        }
        if self.data.contains('\0') {
            anyhow::bail!("Data contains null bytes");
        }
        if self.instruction.contains('\0') {
            anyhow::bail!("Instruction contains null bytes");
        }

        // Construct the generic prompt template
        let prompt = format!(
            r#"# Context
{}

# Data to Evaluate
{}

# Task
{}

Provide your response in JSON format with:
- "decision" or "result": Your evaluation
- "reasoning": Explain step-by-step
- "confidence": 0-1 score
- "evidence": Key facts supporting your decision

Response:"#,
            self.context.trim(),
            self.data.trim(),
            self.instruction.trim()
        );

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_basic() {
        let prompt = PromptBuilder::new()
            .with_context("Job: Senior Rust developer")
            .with_data("Candidate has 5 years Rust experience")
            .with_instruction("Does candidate meet requirements?")
            .build()
            .unwrap();

        assert!(prompt.contains("# Context"));
        assert!(prompt.contains("Job: Senior Rust developer"));
        assert!(prompt.contains("# Data to Evaluate"));
        assert!(prompt.contains("Candidate has 5 years Rust experience"));
        assert!(prompt.contains("# Task"));
        assert!(prompt.contains("Does candidate meet requirements?"));
        assert!(prompt.contains("Provide your response in JSON format"));
    }

    #[test]
    fn test_prompt_builder_all_components_present() {
        let context = "Test context";
        let data = "Test data";
        let instruction = "Test instruction";

        let prompt = PromptBuilder::new()
            .with_context(context)
            .with_data(data)
            .with_instruction(instruction)
            .build()
            .unwrap();

        // Verify all three components appear in the output
        assert!(prompt.contains(context));
        assert!(prompt.contains(data));
        assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_prompt_builder_empty_context_fails() {
        let result = PromptBuilder::new()
            .with_context("")
            .with_data("Some data")
            .with_instruction("Some instruction")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Context cannot be empty"));
    }

    #[test]
    fn test_prompt_builder_empty_data_fails() {
        let result = PromptBuilder::new()
            .with_context("Some context")
            .with_data("")
            .with_instruction("Some instruction")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Data cannot be empty"));
    }

    #[test]
    fn test_prompt_builder_empty_instruction_fails() {
        let result = PromptBuilder::new()
            .with_context("Some context")
            .with_data("Some data")
            .with_instruction("")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instruction cannot be empty"));
    }

    #[test]
    fn test_prompt_builder_whitespace_only_fails() {
        let result = PromptBuilder::new()
            .with_context("   \n\t   ")
            .with_data("Some data")
            .with_instruction("Some instruction")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_builder_trims_whitespace() {
        let prompt = PromptBuilder::new()
            .with_context("  Context with spaces  \n")
            .with_data("\n  Data with spaces  ")
            .with_instruction("  Instruction with spaces  \n")
            .build()
            .unwrap();

        // Should contain trimmed versions
        assert!(prompt.contains("Context with spaces"));
        assert!(prompt.contains("Data with spaces"));
        assert!(prompt.contains("Instruction with spaces"));

        // Should not have leading/trailing whitespace in the actual content
        assert!(!prompt.contains("  Context with spaces  "));
    }

    #[test]
    fn test_prompt_builder_generic_structure() {
        // This test ensures the prompt structure is generic and task-agnostic
        let prompt = PromptBuilder::new()
            .with_context("Any context")
            .with_data("Any data")
            .with_instruction("Any instruction")
            .build()
            .unwrap();

        // Verify generic sections are present
        assert!(prompt.contains("# Context"));
        assert!(prompt.contains("# Data to Evaluate"));
        assert!(prompt.contains("# Task"));
        assert!(prompt.contains("Response:"));

        // Verify NO hardcoded task-specific keywords
        assert!(!prompt.contains("cv_screening"));
        assert!(!prompt.contains("compliance"));
        assert!(!prompt.contains("anomaly_detection"));
    }

    #[test]
    fn test_prompt_builder_json_instructions() {
        let prompt = PromptBuilder::new()
            .with_context("Context")
            .with_data("Data")
            .with_instruction("Instruction")
            .build()
            .unwrap();

        // Verify JSON response format instructions are present
        assert!(prompt.contains("Provide your response in JSON format"));
        assert!(prompt.contains("decision"));
        assert!(prompt.contains("reasoning"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("evidence"));
    }

    #[test]
    fn test_prompt_builder_handles_special_characters() {
        let prompt = PromptBuilder::new()
            .with_context("Context with \"quotes\" and 'apostrophes'")
            .with_data("Data with $pecial ch@rs & symbols!")
            .with_instruction("Instruction with newlines\nand tabs\t")
            .build()
            .unwrap();

        // Should preserve special characters
        assert!(prompt.contains("\"quotes\""));
        assert!(prompt.contains("$pecial ch@rs"));
        assert!(prompt.contains("newlines\nand tabs"));
    }

    #[test]
    fn test_prompt_builder_handles_json_data() {
        let json_data = r#"{"user_id": 123, "amount": 9999, "suspicious": true}"#;

        let prompt = PromptBuilder::new()
            .with_context("Fraud detection rules")
            .with_data(json_data)
            .with_instruction("Is this fraudulent?")
            .build()
            .unwrap();

        // Should preserve JSON structure
        assert!(prompt.contains(json_data));
    }

    #[test]
    fn test_prompt_builder_multiline_context() {
        let multiline_context = r#"Rule 1: Age must be 0-120
Rule 2: Email must be valid
Rule 3: Phone must match E.164"#;

        let prompt = PromptBuilder::new()
            .with_context(multiline_context)
            .with_data("Some data")
            .with_instruction("Validate")
            .build()
            .unwrap();

        assert!(prompt.contains("Rule 1"));
        assert!(prompt.contains("Rule 2"));
        assert!(prompt.contains("Rule 3"));
    }

    // Security tests

    #[test]
    fn test_prompt_builder_context_size_limit() {
        let large_context = "x".repeat(11 * 1024); // 11KB > 10KB limit

        let result = PromptBuilder::new()
            .with_context(&large_context)
            .with_data("data")
            .with_instruction("instruction")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Context too large"));
    }

    #[test]
    fn test_prompt_builder_instruction_size_limit() {
        let large_instruction = "x".repeat(1025); // > 1KB limit

        let result = PromptBuilder::new()
            .with_context("context")
            .with_data("data")
            .with_instruction(&large_instruction)
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instruction too large"));
    }

    #[test]
    fn test_prompt_builder_data_size_limit() {
        let large_data = "x".repeat(1024 * 1024 + 1); // > 1MB limit

        let result = PromptBuilder::new()
            .with_context("context")
            .with_data(&large_data)
            .with_instruction("instruction")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Data too large"));
    }

    #[test]
    fn test_prompt_builder_null_byte_in_context() {
        let context_with_null = "context\0with null";

        let result = PromptBuilder::new()
            .with_context(context_with_null)
            .with_data("data")
            .with_instruction("instruction")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Context contains null bytes"));
    }

    #[test]
    fn test_prompt_builder_null_byte_in_data() {
        let data_with_null = "data\0with null";

        let result = PromptBuilder::new()
            .with_context("context")
            .with_data(data_with_null)
            .with_instruction("instruction")
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Data contains null bytes"));
    }

    #[test]
    fn test_prompt_builder_null_byte_in_instruction() {
        let instruction_with_null = "instruction\0with null";

        let result = PromptBuilder::new()
            .with_context("context")
            .with_data("data")
            .with_instruction(instruction_with_null)
            .build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instruction contains null bytes"));
    }

    #[test]
    fn test_prompt_builder_max_size_allowed() {
        // Test that exact max sizes are allowed
        let max_context = "x".repeat(10 * 1024); // Exactly 10KB
        let max_instruction = "y".repeat(1024); // Exactly 1KB
        let max_data = "z".repeat(1024 * 1024); // Exactly 1MB

        let result = PromptBuilder::new()
            .with_context(&max_context)
            .with_data(&max_data)
            .with_instruction(&max_instruction)
            .build();

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_any_non_empty_inputs_produce_valid_prompt(
            context in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,199}",
            data in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,199}",
            instruction in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,199}",
        ) {
            let result = PromptBuilder::new()
                .with_context(&context)
                .with_data(&data)
                .with_instruction(&instruction)
                .build();

            // Should always succeed with non-empty inputs
            prop_assert!(result.is_ok());

            let prompt = result.unwrap();

            // All components should be present in output
            prop_assert!(prompt.contains(&context.trim()));
            prop_assert!(prompt.contains(&data.trim()));
            prop_assert!(prompt.contains(&instruction.trim()));

            // Generic structure should always be present
            prop_assert!(prompt.contains("# Context"));
            prop_assert!(prompt.contains("# Data to Evaluate"));
            prop_assert!(prompt.contains("# Task"));
            prop_assert!(prompt.contains("Response:"));
        }

        #[test]
        fn test_whitespace_only_inputs_fail(
            whitespace in r"[ \n\t\r]{1,20}",
        ) {
            // Context is whitespace-only
            let result1 = PromptBuilder::new()
                .with_context(&whitespace)
                .with_data("valid data")
                .with_instruction("valid instruction")
                .build();
            prop_assert!(result1.is_err());

            // Data is whitespace-only
            let result2 = PromptBuilder::new()
                .with_context("valid context")
                .with_data(&whitespace)
                .with_instruction("valid instruction")
                .build();
            prop_assert!(result2.is_err());

            // Instruction is whitespace-only
            let result3 = PromptBuilder::new()
                .with_context("valid context")
                .with_data("valid data")
                .with_instruction(&whitespace)
                .build();
            prop_assert!(result3.is_err());
        }

        #[test]
        fn test_special_characters_preserved(
            context in r"[a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/|-][a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/| -]{0,99}",
            data in r"[a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/|-][a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/| -]{0,99}",
            instruction in r"[a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/|-][a-zA-Z0-9!@#$%^&*()_+=\{\}\[\]:;<>,.?/| -]{0,99}",
        ) {
            let prompt = PromptBuilder::new()
                .with_context(&context)
                .with_data(&data)
                .with_instruction(&instruction)
                .build()
                .unwrap();

            // Special characters should be preserved
            prop_assert!(prompt.contains(&context.trim()));
            prop_assert!(prompt.contains(&data.trim()));
            prop_assert!(prompt.contains(&instruction.trim()));
        }

        #[test]
        fn test_no_task_specific_keywords_in_generic_structure(
            context in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
            data in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
            instruction in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
        ) {
            let prompt = PromptBuilder::new()
                .with_context(&context)
                .with_data(&data)
                .with_instruction(&instruction)
                .build()
                .unwrap();

            // Extract only the template structure (not user content)
            // Store trimmed values first to avoid lifetime issues
            let context_trimmed = context.trim().to_string();
            let data_trimmed = data.trim().to_string();
            let instruction_trimmed = instruction.trim().to_string();

            let template_parts: Vec<&str> = prompt
                .split(&context_trimmed)
                .flat_map(|s| s.split(&data_trimmed))
                .flat_map(|s| s.split(&instruction_trimmed))
                .collect();

            let template_text = template_parts.join("");

            // Verify NO hardcoded task types in template structure
            prop_assert!(!template_text.contains("cv_screening"));
            prop_assert!(!template_text.contains("compliance"));
            prop_assert!(!template_text.contains("anomaly_detection"));
            prop_assert!(!template_text.contains("fraud"));
            prop_assert!(!template_text.contains("sentiment"));
        }

        #[test]
        fn test_output_is_deterministic(
            context in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
            data in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
            instruction in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
        ) {
            // Build the same prompt twice
            let prompt1 = PromptBuilder::new()
                .with_context(&context)
                .with_data(&data)
                .with_instruction(&instruction)
                .build()
                .unwrap();

            let prompt2 = PromptBuilder::new()
                .with_context(&context)
                .with_data(&data)
                .with_instruction(&instruction)
                .build()
                .unwrap();

            // Should produce identical results
            prop_assert_eq!(prompt1, prompt2);
        }
    }
}
