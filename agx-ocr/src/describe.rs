use anyhow::Result;
use serde::Serialize;

/// AU model card structure compatible with central describe.schema.json
#[derive(Debug, Serialize)]
struct ModelCard {
    name: String,
    version: String,
    description: String,
    capabilities: Vec<String>,
    inputs: Vec<IoFormat>,
    outputs: Vec<IoFormat>,
    config: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct IoFormat {
    media_type: String,
    description: String,
}

pub fn print_model_card() -> Result<()> {
    let card = ModelCard {
        name: "agx-ocr".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Agentic Unit for OCR using DeepSeek GGUF models. Reads image bytes from stdin and outputs structured JSON."
            .to_string(),
        capabilities: vec!["ocr".to_string(), "image-to-text".to_string()],
        inputs: vec![IoFormat {
            media_type: "image/*".to_string(),
            description: "Binary image data (PNG, JPEG) via stdin".to_string(),
        }],
        outputs: vec![IoFormat {
            media_type: "application/json".to_string(),
            description: "OCR result as structured JSON (text, regions, confidences)".to_string(),
        }],
        config: serde_json::json!({
            "model-path": {
                "type": "string",
                "description": "Filesystem path to DeepSeek GGUF model file.",
                "default": null
            }
        }),
    };

    let json = serde_json::to_string_pretty(&card)?;
    println!("{json}");
    Ok(())
}
