use anyhow::{Context, Result};
use image::DynamicImage;

use crate::model::ModelConfig;
use crate::types::OcrResult;

// DeepSeek OCR engine imports
use candle_core::{DType, Device};
use deepseek_ocr_core::inference::{
    DecodeParameters, ModelKind, ModelLoadArgs, VisionSettings,
};
use deepseek_ocr_infer_deepseek::load_model;
use tokenizers::Tokenizer;

/// Default prompt used when no custom prompt is provided
const DEFAULT_PROMPT: &str = "<image>\nExtract all text from this image.";

pub fn run_ocr(image_bytes: &[u8], cfg: &ModelConfig, custom_prompt: Option<&str>) -> Result<OcrResult> {
    // Decode image from bytes
    let img = image::load_from_memory(image_bytes)
        .context("Failed to decode image bytes from stdin")?;

    // Delegate to DeepSeek engine with custom prompt if provided
    let text = run_engine(&img, &cfg.model_path, custom_prompt)?;

    // For now, we only return the full OCR text without region-level details
    // The DeepSeek engine doesn't expose bounding boxes in its current API
    Ok(OcrResult {
        text,
        regions: vec![], // TODO: Add region detection if needed
        model: format!("deepseek-ocr ({})", cfg.model_path.display()),
    })
}

/// Runs the DeepSeek OCR engine on the provided image.
///
/// The model_path should point to a directory containing:
/// - config.json: Model configuration
/// - model.safetensors (or model.gguf): Model weights
/// - tokenizer.json: Tokenizer configuration
///
/// The custom_prompt parameter allows specifying task-specific instructions.
/// Use <image> token to denote where the image should be placed in the prompt.
fn run_engine(img: &DynamicImage, model_path: &std::path::Path, custom_prompt: Option<&str>) -> Result<String> {
    // Validate that model_path is a directory
    anyhow::ensure!(
        model_path.is_dir(),
        "Model path must be a directory containing config.json, weights, and tokenizer.json"
    );

    // Construct paths to required files
    let config_path = model_path.join("config.json");
    let tokenizer_path = model_path.join("tokenizer.json");

    // Try to find weights file (safetensors or gguf)
    let weights_path = if model_path.join("model.safetensors").exists() {
        model_path.join("model.safetensors")
    } else if model_path.join("model.gguf").exists() {
        model_path.join("model.gguf")
    } else {
        anyhow::bail!(
            "No model weights found in {}. Expected model.safetensors or model.gguf",
            model_path.display()
        );
    };

    // Validate all required files exist
    anyhow::ensure!(
        config_path.exists(),
        "Config file not found: {}",
        config_path.display()
    );
    anyhow::ensure!(
        tokenizer_path.exists(),
        "Tokenizer file not found: {}",
        tokenizer_path.display()
    );

    // Select device (prefer Metal on macOS, fallback to CPU)
    let device = Device::new_metal(0).unwrap_or(Device::Cpu);

    // Select dtype based on device
    let dtype = match &device {
        Device::Cpu => DType::BF16,
        Device::Metal(_) => DType::F16,
        _ => DType::F16,
    };

    // Load the model
    let load_args = ModelLoadArgs {
        kind: ModelKind::Deepseek,
        config_path: Some(&config_path),
        weights_path: Some(&weights_path),
        snapshot_path: None, // No quantized snapshot for now
        device: device.clone(),
        dtype,
    };

    let model = load_model(load_args)
        .context("Failed to load DeepSeek OCR model")?;

    // Load tokenizer
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer from {}: {}", tokenizer_path.display(), e))?;

    // Prepare vision settings (using defaults from DeepSeek OCR CLI)
    let vision_settings = VisionSettings {
        base_size: 2,
        image_size: 640,
        crop_mode: false,
    };

    // Prepare decode parameters (conservative defaults)
    let decode_params = DecodeParameters {
        max_new_tokens: 4096,
        do_sample: false,
        temperature: 0.0,
        top_p: None,
        top_k: None,
        repetition_penalty: 1.0,
        no_repeat_ngram_size: None,
        seed: None,
        use_cache: true,
    };

    // Use custom prompt if provided, otherwise use default
    let prompt = custom_prompt.unwrap_or(DEFAULT_PROMPT);

    // Ensure prompt contains <image> token
    anyhow::ensure!(
        prompt.contains("<image>"),
        "Prompt must contain <image> token to indicate image placement. Got: {}",
        prompt
    );

    // Run OCR inference
    let outcome = model
        .decode(
            &tokenizer,
            prompt,
            &[img.clone()],
            vision_settings,
            &decode_params,
            None, // No streaming callback
        )
        .context("OCR inference failed")?;

    Ok(outcome.text)
}
