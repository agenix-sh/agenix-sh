// agx-eval: Generic LLM evaluation Agentic Unit
//
// Main orchestration: stdin → prompt → LLM → parse → stdout

mod llm;
mod parser;
mod prompt;

use anyhow::{Context, Result};
use clap::Parser;
use llm::{get_ollama_endpoint, OllamaClient};
use parser::{parse_llm_response, EvaluationResult};
use prompt::PromptBuilder;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::time::Instant;

#[derive(Parser, Debug, Clone)]
#[command(name = "agx-eval")]
#[command(about = "Generic LLM evaluation Agentic Unit", long_about = None)]
struct Cli {
    /// Context: background information, criteria, domain knowledge
    #[arg(long, required = true)]
    context: String,

    /// Prompt: evaluation question/instruction
    #[arg(long, required = true)]
    prompt: String,

    /// LLM model to use
    #[arg(long, default_value = "qwen2.5:1.5b")]
    model: String,

    /// Sampling temperature (0.0-1.0)
    #[arg(long, default_value = "0.1")]
    temperature: f32,

    /// Maximum tokens to generate
    #[arg(long, default_value = "500")]
    max_tokens: usize,

    /// Output format (json or text)
    #[arg(long, default_value = "json")]
    format: String,
}

/// Output structure for evaluation results
#[derive(Debug, Serialize, Deserialize)]
struct Output {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<EvaluationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorInfo>,
}

/// Metadata about the evaluation
#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    model: String,
    backend: String,
    latency_ms: u128,
}

/// Error information
#[derive(Debug, Serialize, Deserialize)]
struct ErrorInfo {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

/// Read data from stdin with size limit
fn read_stdin() -> Result<String> {
    const MAX_STDIN_SIZE: usize = 1024 * 1024; // 1MB

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.len() > MAX_STDIN_SIZE {
        anyhow::bail!(
            "Stdin data too large: {} bytes (max {} bytes)",
            buffer.len(),
            MAX_STDIN_SIZE
        );
    }

    Ok(buffer)
}

/// Main evaluation pipeline
async fn evaluate(args: Cli) -> Result<Output> {
    let start = Instant::now();

    // 1. Read stdin data
    tracing::debug!("Reading stdin data");
    let data = read_stdin().context("Failed to read input data")?;
    tracing::debug!("Read {} bytes from stdin", data.len());

    // 2. Build prompt
    tracing::debug!("Building evaluation prompt");
    let prompt_text = PromptBuilder::new()
        .with_context(&args.context)
        .with_data(&data)
        .with_instruction(&args.prompt)
        .build()
        .context("Failed to build prompt")?;

    tracing::debug!("Prompt built: {} chars", prompt_text.len());

    // 3. Call LLM
    tracing::info!("Calling LLM: model={}", args.model);
    let endpoint = get_ollama_endpoint();
    let client = OllamaClient::new(&endpoint, &args.model, args.temperature, args.max_tokens)
        .context("Failed to create LLM client")?;

    let llm_response = client
        .generate(&prompt_text)
        .await
        .context("LLM inference failed")?;

    tracing::debug!("LLM response: {} chars", llm_response.len());

    // 4. Parse response
    tracing::debug!("Parsing LLM response");
    let result = parse_llm_response(&llm_response).context("Failed to parse LLM response")?;

    let latency = start.elapsed().as_millis();
    tracing::info!("Evaluation complete in {}ms", latency);

    // 5. Build output
    Ok(Output {
        status: "success".to_string(),
        result: Some(result),
        metadata: Some(Metadata {
            model: args.model.clone(),
            backend: "ollama".to_string(),
            latency_ms: latency,
        }),
        error: None,
    })
}

/// Format output based on requested format
fn format_output(output: &Output, format: &str) -> Result<String> {
    match format {
        "json" => serde_json::to_string_pretty(output).context("Failed to serialize output"),
        "text" => {
            if let Some(ref result) = output.result {
                let decision = result.get_decision().unwrap_or("N/A");
                Ok(format!(
                    "Decision: {}\nReasoning: {}\nConfidence: {:.2}",
                    decision, result.reasoning, result.confidence
                ))
            } else if let Some(ref error) = output.error {
                Ok(format!("Error: {}", error.message))
            } else {
                Ok("Unknown output".to_string())
            }
        }
        _ => anyhow::bail!("Unsupported output format: {}", format),
    }
}

/// Convert error to structured output
fn error_to_output(error: anyhow::Error) -> Output {
    // Determine error code based on error message
    let error_msg = error.to_string();
    let code = if error_msg.contains("required") || error_msg.contains("cannot be empty") {
        "invalid_arguments"
    } else if error_msg.contains("Failed to read") || error_msg.contains("too large") {
        "input_error"
    } else if error_msg.contains("Failed to build prompt") {
        "prompt_error"
    } else if error_msg.contains("Failed to create LLM client") {
        "llm_client_error"
    } else if error_msg.contains("LLM inference failed") || error_msg.contains("connect") {
        "llm_connection_failed"
    } else if error_msg.contains("Failed to parse") {
        "parse_error"
    } else {
        "unknown_error"
    };

    Output {
        status: "error".to_string(),
        result: None,
        metadata: None,
        error: Some(ErrorInfo {
            code: code.to_string(),
            message: error_msg.clone(),
            details: Some(format!("{:#}", error)),
        }),
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    // Initialize tracing (logs to stderr)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("agx-eval v0.1.0 starting");
    tracing::debug!(
        "Arguments: model={}, temperature={}, max_tokens={}",
        args.model,
        args.temperature,
        args.max_tokens
    );

    // Extract format before moving args
    let format = args.format.clone();

    // Run evaluation and handle errors
    let output = match evaluate(args).await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!("Evaluation failed: {:#}", error);
            error_to_output(error)
        }
    };

    // Format and print output (only to stdout)
    match format_output(&output, &format) {
        Ok(formatted) => {
            println!("{}", formatted);
            // Exit with appropriate code
            if output.status == "success" {
                std::process::exit(0);
            } else if output
                .error
                .as_ref()
                .map(|e| e.code == "invalid_arguments")
                .unwrap_or(false)
            {
                std::process::exit(2);
            } else {
                std::process::exit(1);
            }
        }
        Err(error) => {
            // Fallback: output raw JSON if formatting fails
            eprintln!("Failed to format output: {}", error);
            if let Ok(json) = serde_json::to_string_pretty(&output) {
                println!("{}", json);
            }
            std::process::exit(1);
        }
    }
}
