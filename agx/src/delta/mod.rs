use anyhow::Result;
use crate::models::ModelManager;
use crate::planner::{CandleBackend, CandleConfig, ModelRole, ModelBackend, PlanContext};


pub async fn run(goal: String) -> Result<()> {
    println!("Agenix Delta (Planner)");
    println!("Goal: {}", goal);
    println!("---------------------------------------");

    // Load configuration to determine backend
    let config = crate::planner::PlannerConfig::from_env();
    println!("Backend: {:?}", config.backend);

    let backend: Box<dyn ModelBackend> = match config.backend {
        crate::planner::BackendKind::Candle => {
            println!("Initializing Model Manager...");
            let manager = ModelManager::new()?;

            // Use Qwen 2.5 Coder 1.5B for fast local testing
            let repo = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF";
            let file = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf";
            
            println!("Ensuring model is available: {}/{}", repo, file);
            let model_path = manager.ensure_model(repo, file).await?;
            println!("Model loaded from: {}", model_path.display());

            // Ensure tokenizer is available (from base repo)
            let tokenizer_repo = "Qwen/Qwen2.5-Coder-1.5B-Instruct";
            let tokenizer_url = format!("https://huggingface.co/{}/resolve/main/tokenizer.json", tokenizer_repo);
            let tokenizer_file = "tokenizer.json";
            
            println!("Ensuring tokenizer is available from: {}", tokenizer_url);
            let raw_tokenizer_path = manager.download_file_raw(&tokenizer_url, tokenizer_file).await?;
            
            // Copy tokenizer to model directory
            let model_dir = model_path.parent().unwrap();
            let dest_tokenizer_path = model_dir.join("tokenizer.json");
            if !dest_tokenizer_path.exists() {
                println!("Copying tokenizer to model directory: {}", dest_tokenizer_path.display());
                tokio::fs::copy(&raw_tokenizer_path, &dest_tokenizer_path).await?;
            }

            // Initialize Candle Backend
            let candle_config = CandleConfig {
                model_path: model_path.clone(),
                model_role: ModelRole::Delta,
                ..CandleConfig::default()
            };
            
            println!("Initializing inference engine (Candle)...");
            let backend = CandleBackend::new(candle_config).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize backend: {:?}", e))?;
                
            Box::new(backend)
        }
        crate::planner::BackendKind::Ollama => {
            println!("Initializing inference engine (Ollama)...");
            let ollama_config = crate::planner::ollama::OllamaConfig::default();
            let backend = crate::planner::OllamaBackend::from_config(ollama_config);
            
            // Verify Ollama connection
            if let Err(e) = backend.health_check().await {
                println!("Warning: Ollama health check failed: {:?}", e);
                println!("Make sure Ollama is running and the model is pulled.");
            }
            
            Box::new(backend)
        }
    };

    println!("Planning...");
    
    // Construct context (TODO: populate with actual tools)
    let context = PlanContext::default();
    
    // Generate plan
    let plan = backend.generate_plan(&goal, &context).await
        .map_err(|e| anyhow::anyhow!("Failed to generate plan: {:?}", e))?;

    println!("Plan generated!");
    println!("---------------------------------------");
    println!("{}", serde_json::to_string_pretty(&plan.tasks)?);
    println!("---------------------------------------");
    
    // Submit to AGQ
    println!("Submitting plan to AGQ...");
    
    // Construct Plan JSON
    let plan_id = uuid::Uuid::new_v4().to_string();
    let plan_payload = serde_json::json!({
        "plan_id": plan_id,
        "plan_description": goal,
        "tasks": plan.tasks.iter().map(|t| {
            serde_json::json!({
                "task_number": t.task_number,
                "command": t.command,
                "args": t.args,
                "timeout_secs": t.timeout_secs,
                "input_from_task": t.input_from_task
            })
        }).collect::<Vec<_>>()
    });
    
    let plan_json = serde_json::to_string(&plan_payload)?;
    
    // Connect to AGQ
    let agq_addr = std::env::var("AGQ_ADDR").unwrap_or_else(|_| "127.0.0.1:6379".to_string());
    let mut client = crate::client::AgqClient::connect(&agq_addr).await?;
    
    match client.submit_plan(&plan_json).await {
        Ok(returned_id) => {
            println!("Plan submitted successfully!");
            println!("Plan ID: {}", returned_id);
            println!("Use 'agx list' or 'agq' to monitor progress.");
        }
        Err(e) => {
            println!("Failed to submit plan: {:?}", e);
        }
    }

    Ok(())
}
