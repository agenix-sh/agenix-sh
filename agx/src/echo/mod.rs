use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, EditMode};


use crate::models::ModelManager;
use crate::planner::{CandleBackend, CandleConfig, ModelRole, ModelBackend, PlanContext, ChatMessage, ToolInfo};
use crate::registry::ToolRegistry;

// UI Colors
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_USER: &str = "\x1b[1;36m"; // Bold Cyan
const COLOR_AI: &str = "\x1b[1;32m";   // Bold Green
const COLOR_SYSTEM: &str = "\x1b[1;33m"; // Bold Yellow
const COLOR_BOLD: &str = "\x1b[1m";

pub async fn run() -> Result<()> {
    print_banner();
    
    // Load configuration to determine backend
    let config = crate::planner::PlannerConfig::from_env();
    println!("{}Backend: {:?}{}", COLOR_SYSTEM, config.backend, COLOR_RESET);

    let backend: Box<dyn ModelBackend> = match config.backend {
        crate::planner::BackendKind::Candle => {
            println!("{}Initializing Model Manager...{}", COLOR_SYSTEM, COLOR_RESET);
            let manager = ModelManager::new()?;
            
            // Using Qwen 2.5 7B Instruct (GGUF)
            let repo = "Qwen/Qwen2.5-7B-Instruct-GGUF";
            let file = "qwen2.5-7b-instruct-q4_k_m.gguf";
            
            println!("{}Ensuring model is available: {}/{}{}", COLOR_SYSTEM, repo, file, COLOR_RESET);
            let model_path = manager.ensure_model(repo, file).await?;
            
            // Also ensure tokenizer.json is available
            let tokenizer_url = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct/resolve/main/tokenizer.json";
            let tokenizer_file = "tokenizer.json";
            
            let raw_tokenizer_path = manager.download_file_raw(tokenizer_url, tokenizer_file).await?;
            
            // Copy tokenizer to model directory so Candle finds it
            let model_dir = model_path.parent().unwrap();
            let dest_tokenizer_path = model_dir.join("tokenizer.json");
            
            if !dest_tokenizer_path.exists() {
                tokio::fs::copy(&raw_tokenizer_path, &dest_tokenizer_path).await?;
            }

            // Initialize Candle Backend
            let candle_config = CandleConfig {
                model_path: model_path.clone(),
                model_role: ModelRole::Echo,
                ..CandleConfig::default()
            };
            
            println!("{}Initializing inference engine (Candle)...{}", COLOR_SYSTEM, COLOR_RESET);
            let backend = CandleBackend::new(candle_config).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize backend: {:?}", e))?;
                
            Box::new(backend)
        }
        crate::planner::BackendKind::Ollama => {
            println!("{}Initializing inference engine (Ollama)...{}", COLOR_SYSTEM, COLOR_RESET);
            let ollama_config = crate::planner::ollama::OllamaConfig::default();
            let backend = crate::planner::OllamaBackend::from_config(ollama_config);
            
            // Verify Ollama connection
            if let Err(e) = backend.health_check().await {
                println!("{}Warning: Ollama health check failed: {:?}{}", COLOR_SYSTEM, e, COLOR_RESET);
                println!("Make sure Ollama is running and the model is pulled.");
            }
            
            Box::new(backend)
        }
    };

    // Initialize Rustyline Editor
    let config = Config::builder()
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .build();
    let mut editor = DefaultEditor::with_config(config)?;
    
    // Chat History
    let mut history: Vec<ChatMessage> = Vec::new();
    
    // Initial System Prompt
    let reg = ToolRegistry::new();
    let tools_desc = reg.describe_for_planner();
    
    history.push(ChatMessage::system(format!(
        "You are Echo, an intelligent assistant for the Agenix platform. \
         Your goal is to help the user clarify their intent and build a task plan. \
         Be helpful, concise, and conversational. \
         When the user asks to create a plan, help them define the steps using the available tools:\n\
         {}\n\
         If a requested action is not supported by these tools, explain that.",
         tools_desc
    )));

    println!("{}", COLOR_RESET);
    println!("Type {}/help{} for commands, {}/exit{} to quit", COLOR_BOLD, COLOR_RESET, COLOR_BOLD, COLOR_RESET);
    println!("---------------------------------------");

    loop {
        let prompt = format!("{}👤 User > {}", COLOR_USER, COLOR_RESET);
        let readline = editor.readline(&prompt);

        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                // Handle Slash Commands
                if input.starts_with('/') {
                    match handle_command(input, &mut history, &backend).await {
                        Ok(should_exit) => if should_exit { break },
                        Err(e) => println!("{}Error: {}{}", COLOR_SYSTEM, e, COLOR_RESET),
                    }
                    continue;
                }

                // User Message
                history.push(ChatMessage::user(input));

                // AI Response
                print!("{}🤖 Echo > {}Thinking...", COLOR_AI, COLOR_RESET);
                use std::io::Write;
                std::io::stdout().flush()?;

                // Build context with tools
                let reg = ToolRegistry::new();
                let tool_registry: Vec<ToolInfo> = reg.tools()
                    .iter()
                    .map(|t| ToolInfo::new(t.id, t.description))
                    .collect();
                // Get cluster status
            let status = get_cluster_status().await;

            // Build context with tools and status
            let context = PlanContext {
                tool_registry: tool_registry.clone(),
                input_summary: Some(status),
                ..PlanContext::default()
            };

            // Generate response
            let response = backend.chat(&history, &context).await;
            
            match response {
                Ok(reply) => {
                    // Clear "Thinking..."
                        print!("\r\x1b[K");
                        println!("{}🤖 Echo > {}{}", COLOR_AI, COLOR_RESET, reply);
                        history.push(ChatMessage::assistant(reply));
                    }
                    Err(e) => {
                        print!("\r\x1b[K");
                        println!("{}Error: {:?}{}", COLOR_SYSTEM, e, COLOR_RESET);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn print_banner() {
    println!("{}", COLOR_AI);
    println!("    ___    ______  __");
    println!("   /   |  / ____/ / /");
    println!("  / /| | / / __  / / ");
    println!(" / ___ |/ /_/ / /_/");
    println!("/_/  |_|\\____/ (_)   ");
    println!("{}", COLOR_RESET);
    println!("{}Agenix Echo - Conversational Planner{}", COLOR_BOLD, COLOR_RESET);
    println!();
}

async fn handle_command(
    input: &str, 
    history: &mut Vec<ChatMessage>, 
    backend: &Box<dyn ModelBackend>
) -> Result<bool> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts[0];

    match cmd {
        "/exit" | "/quit" => return Ok(true),
        "/clear" | "/reset" => {
            history.clear();
            let reg = ToolRegistry::new();
            let tools_desc = reg.describe_for_planner();
            
            history.push(ChatMessage::system(format!(
                "You are Echo, an intelligent assistant for the Agenix platform. \
                 Your goal is to help the user clarify their intent and build a task plan. \
                 Be helpful, concise, and conversational. \
                 When the user asks to create a plan, help them define the steps using the available tools:\n\
                 {}\n\
                 If a requested action is not supported by these tools, explain that.",
                 tools_desc
            )));
            println!("{}History cleared.{}", COLOR_SYSTEM, COLOR_RESET);
        }
        "/history" => {
            println!("{}Conversation History:{}", COLOR_BOLD, COLOR_RESET);
            for msg in history.iter() {
                let color = match msg.role.as_str() {
                    "user" => COLOR_USER,
                    "assistant" => COLOR_AI,
                    _ => COLOR_SYSTEM,
                };
                println!("{}{}: {}{}", color, msg.role, COLOR_RESET, msg.content);
            }
        }
        "/plan" => {
            println!("{}Generating plan from conversation...{}", COLOR_SYSTEM, COLOR_RESET);
            // Aggregate user messages for the instruction
            let instruction = history.iter()
                .filter(|m| m.role == "user")
                .map(|m| m.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            
            if instruction.is_empty() {
                println!("{}No user input to plan from.{}", COLOR_SYSTEM, COLOR_RESET);
                return Ok(false);
            }

            // Build context with tools
            let reg = ToolRegistry::new();
            let tool_registry: Vec<ToolInfo> = reg.tools()
                .iter()
                .map(|t| ToolInfo::new(t.id, t.description))
                .collect();
            
            let context = PlanContext {
                tool_registry,
                ..PlanContext::default()
            };
            
            match backend.generate_plan(&instruction, &context).await {
                Ok(plan) => {
                    println!("{}Validating plan with Delta...{}", COLOR_SYSTEM, COLOR_RESET);
                    
                    // Create context for Delta with the initial plan
                    let delta_context = PlanContext {
                        tool_registry: context.tool_registry.clone(),
                        existing_tasks: plan.tasks.clone(),
                        input_summary: context.input_summary.clone(),
                        ..PlanContext::default()
                    };

                    // Run validation pass
                    match backend.generate_plan(&instruction, &delta_context).await {
                        Ok(validated_plan) => {
                            println!("{}Plan Validated!{}", COLOR_AI, COLOR_RESET);
                            let json = serde_json::to_string_pretty(&validated_plan.tasks).unwrap();
                            println!("{}", json);
                        }
                        Err(e) => {
                            println!("{}Validation failed, using original plan: {:?}{}", COLOR_SYSTEM, e, COLOR_RESET);
                            let json = serde_json::to_string_pretty(&plan.tasks).unwrap();
                            println!("{}", json);
                        }
                    }
                }
                Err(e) => println!("{}Error generating plan: {:?}{}", COLOR_SYSTEM, e, COLOR_RESET),
            }
        }
        "/help" => {
            println!("{}Available Commands:{}", COLOR_BOLD, COLOR_RESET);
            println!("  /exit, /quit    - Exit the chat");
            println!("  /clear, /reset  - Clear conversation history");
            println!("  /history        - Show full conversation history");
            println!("  /plan           - Generate a plan from the current conversation");
            println!("  /help           - Show this help message");
        }
        _ => {
            println!("{}Unknown command: {}. Type /help for available commands.{}", COLOR_SYSTEM, cmd, COLOR_RESET);
        }
    }
    Ok(false)
}

async fn get_cluster_status() -> String {
    tokio::task::spawn_blocking(|| {
        let config = crate::agq_client::AgqConfig::from_env();
        let client = crate::agq_client::AgqClient::new(config);
        
        let mut status = String::from("Cluster Status:\n");
        
        match client.list_workers() {
            Ok(crate::agq_client::OpsResponse::Workers(w)) => {
                 status.push_str(&format!("- Workers: {} active\n", w.len()));
                 for worker in w {
                     status.push_str(&format!("  - {}\n", worker));
                 }
            }
            Err(e) => status.push_str(&format!("- Workers: Error ({})\n", e)),
            _ => status.push_str("- Workers: Unknown response\n"),
        }

        match client.list_jobs() {
            Ok(crate::agq_client::OpsResponse::Jobs(j)) => {
                 status.push_str(&format!("- Jobs: {}\n", j.len()));
                 for job in j {
                     status.push_str(&format!("  - {}\n", job));
                 }
            }
             Err(e) => status.push_str(&format!("- Jobs: Error ({})\n", e)),
            _ => status.push_str("- Jobs: Unknown response\n"),
        }
        
        status
    }).await.unwrap_or_else(|e| format!("Failed to get status: {}", e))
}
