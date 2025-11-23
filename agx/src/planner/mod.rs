// Core backend abstraction
pub mod backend;
pub mod types;

// Device selection
pub mod device;

// Backend implementations
pub mod candle;
pub mod ollama;
pub mod openai;

// High-level wrapper (backward compatible API)
pub mod wrapper;

pub mod prompts;

pub use backend::ModelBackend;
pub use candle::{CandleBackend, CandleConfig, ModelRole};
pub use ollama::{OllamaBackend, OllamaConfig};
pub use openai::OpenAIBackend;
pub use types::{ChatMessage, PlanContext, ToolInfo};
pub use wrapper::{Planner, PlannerConfig, BackendKind};
