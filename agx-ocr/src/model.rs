use std::path::PathBuf;

use anyhow::{bail, Result};

/// Configuration for model loading.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model_path: PathBuf,
}

impl ModelConfig {
    /// Build config from CLI / env.
    /// Strict mode: model path MUST be provided via --model-path or $MODEL_PATH.
    pub fn from_cli(model_path: Option<PathBuf>) -> Result<Self> {
        match model_path {
            Some(p) => Ok(Self { model_path: p }),
            None => {
                bail!(
                    "No model path specified. Provide --model-path or set $MODEL_PATH to a GGUF file."
                );
            }
        }
    }
}
