use anyhow::Result;
use std::io::{self, Write};
use crate::models::ModelManager;
use crate::planner::{CandleBackend, CandleConfig, ModelRole, ModelBackend, PlanContext};

// Default Echo model
const DEFAULT_REPO: &str = "hugging-quants/Meta-Llama-3.1-8B-Instruct-AWQ-INT4";
const DEFAULT_FILENAME: &str = "model.safetensors"; // Placeholder for actual GGUF filename when we switch to GGUF repo

pub async fn run() -> Result<()> {
    println!("Welcome to Agenix Echo (Chat)");
    println!("Initializing Model Manager...");

    let manager = ModelManager::new()?;
    // Using TinyLlama-1.1B-Chat-v1.0-GGUF for testing
    let repo = "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF";
    let file = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";
    
    println!("Ensuring model is available: {}/{}", repo, file);
    let model_path = manager.ensure_model(repo, file).await?;
    println!("Model loaded from: {}", model_path.display());

    // Also ensure tokenizer.json is available
    // Note: GGUF repos often don't have tokenizer.json, so we use the base model repo
    // Fallback to raw download since hf-hub is having issues with this repo
    let tokenizer_url = "https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0/resolve/main/tokenizer.json";
    let tokenizer_file = "tokenizer.json";
    println!("Ensuring tokenizer is available from: {}", tokenizer_url);
    
    // We need to tell Candle where the tokenizer is. 
    // CandleBackend expects it next to the model file OR we can pass it explicitly.
    // But CandleBackend::new takes a config with model_path and derives tokenizer path.
    // We might need to hack this by copying the tokenizer to the model directory.
    
    let raw_tokenizer_path = manager.download_file_raw(tokenizer_url, tokenizer_file).await?;
    
    // Copy tokenizer to model directory so Candle finds it
    let model_dir = model_path.parent().unwrap();
    let dest_tokenizer_path = model_dir.join("tokenizer.json");
    
    if !dest_tokenizer_path.exists() {
        println!("Copying tokenizer to model directory: {}", dest_tokenizer_path.display());
        tokio::fs::copy(&raw_tokenizer_path, &dest_tokenizer_path).await?;
    }

    // Initialize Candle Backend
    let config = CandleConfig {
        model_path: model_path.clone(),
        model_role: ModelRole::Echo,
        ..CandleConfig::default()
    };
    
    println!("Initializing inference engine...");
    let backend = CandleBackend::new(config).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize backend: {:?}", e))?;

    println!("---------------------------------------");
    println!("Type '/exit' to quit, '/reset' to clear history");
    
    let mut history: Vec<String> = Vec::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "/exit" {
            break;
        } else if input == "/reset" {
            history.clear();
            println!("History cleared.");
            continue;
        }

        history.push(format!("User: {}", input));
        
        // Construct a simple context for now
        let context = PlanContext::default();
        
        print!("Echo: Thinking...");
        stdout.flush()?;
        
        // Run inference
        // Note: generate_plan expects an instruction and context. 
        // For Echo, we treat the input as the instruction.
        // The backend's build_echo_prompt handles the formatting.
        match backend.generate_plan(input, &context).await {
            Ok(plan) => {
                // Clear "Thinking..." line
                print!("\r\x1b[K"); 
                
                // For Echo, we might want just the raw text, but generate_plan returns a structured plan.
                // However, CandleBackend::generate_plan decodes the output tokens.
                // Let's see if we can get the raw text or if we need to adjust the backend.
                // The backend parses the response into tasks. 
                // If the model just chats, parsing might fail or return empty tasks.
                // For a true Chat REPL, we probably want a generate_text method on the backend.
                // But for now, let's just print what we got.
                
                println!("Response (Tasks): {:?}", plan.tasks);
                history.push(format!("Assistant: {:?}", plan.tasks));
            }
            Err(e) => {
                print!("\r\x1b[K");
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}
