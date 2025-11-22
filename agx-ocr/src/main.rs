use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

mod ocr;
mod model;
mod describe;
mod types;

use crate::model::ModelConfig;

/// agx-ocr: DeepSeek OCR Agentic Unit
#[derive(Parser, Debug)]
#[command(name = "agx-ocr")]
#[command(about = "AGEniX OCR AU using DeepSeek GGUF models", long_about = None)]
struct Cli {
    /// Path to DeepSeek OCR GGUF model
    #[arg(long = "model-path", env = "MODEL_PATH")]
    model_path: Option<PathBuf>,

    /// Print AU model description as JSON (for --describe contract)
    #[arg(long = "describe")]
    describe: bool,

    /// Custom prompt (use <image> token for image placement)
    /// Can also be provided as first positional argument
    #[arg(long = "prompt")]
    prompt: Option<String>,

    /// Prompt as first positional argument (alternative to --prompt)
    /// Example: agx-ocr "Extract chart data as JSON" < chart.png
    #[arg(value_name = "PROMPT")]
    prompt_positional: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.describe {
        describe::print_model_card()?;
        return Ok(());
    }

    let cfg = ModelConfig::from_cli(cli.model_path)?;

    // Determine prompt: --prompt flag takes precedence, then positional arg, then default
    let prompt_str = cli.prompt.or(cli.prompt_positional);
    let prompt = prompt_str.as_deref();

    // Read binary input from stdin
    let mut buf = Vec::new();
    io::stdin()
        .read_to_end(&mut buf)
        .context("Failed to read image bytes from stdin")?;

    let result = ocr::run_ocr(&buf, &cfg, prompt)?;

    // Write structured JSON to stdout
    let json = serde_json::to_string_pretty(&result)
        .context("Failed to serialize OCR result to JSON")?;
    println!("{}", json);

    Ok(())
}
