use serde::Serialize;

/// High-level OCR output structure returned by this AU.
/// This does not need to mirror the deepseek-ocr engine types exactly;
/// it is the stable contract for AGEniX pipelines.
#[derive(Debug, Serialize)]
pub struct OcrRegion {
    pub text: String,
    pub confidence: f32,
    /// [x1, y1, x2, y2] in image coordinates
    pub bbox: [f32; 4],
}

#[derive(Debug, Serialize)]
pub struct OcrResult {
    pub text: String,
    pub regions: Vec<OcrRegion>,
    pub model: String,
}
