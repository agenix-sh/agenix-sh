use agx::planner::{OllamaBackend, ModelBackend, PlanContext, ToolInfo};
use agx::registry::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrainingExample {
    messages: Vec<ChatMessage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Initializing Synthetic Data Generator...");

    // 1. Setup
    let registry = ToolRegistry::new();
    // ... (rest of setup) ...
    let tools = registry.tools();
    let tools_desc = registry.describe_for_planner();
    
    let provider = std::env::var("AGX_TEACHER_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    let teacher_model = std::env::var("AGX_TEACHER_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string());
    
    println!("Using Teacher Provider: {}", provider);
    println!("Using Teacher Model: {}", teacher_model);
    
    let backend: Box<dyn ModelBackend> = match provider.as_str() {
        "openai" => Box::new(agx::planner::OpenAIBackend::new(teacher_model)),
        _ => Box::new(OllamaBackend::new(teacher_model)),
    };

    let categories = vec![
        "File manipulation (sorting, deduplicating, counting)",
        "Data extraction (grep, cut, tr)",
        "JSON processing (jq)",
        "Complex pipelines (chaining multiple tools)",
    ];

    let mut examples = Vec::new();

    for category in categories {
        // ... (generation loop) ...
        println!("Generating scenarios for: {}", category);
        
        let prompt = format!(
            "You are a synthetic data generator. \
             Generate 5 diverse, realistic user instructions for a CLI agent that can use these tools:\n\
             {}\n\
             \n\
             The instructions should be related to: {}\n\
             \n\
             Output ONLY a JSON array of strings. Example: [\"Sort file.txt\", \"Count lines in data.log\"]",
            tools_desc, category
        );

        let context = PlanContext::default();
        let history = vec![agx::planner::ChatMessage::user(prompt)];
        let response = backend.chat(&history, &context).await?;
        
        let clean_json = response.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let instructions: Vec<String> = serde_json::from_str(clean_json)
            .unwrap_or_else(|e| {
                println!("Failed to parse scenarios: {}", e);
                vec![]
            });

        for instruction in instructions {
            println!("  Processing: {}", instruction);
            
            let context = PlanContext {
                tool_registry: registry.tools().iter().map(|t| ToolInfo::new(t.id, t.description)).collect(),
                ..PlanContext::default()
            };
            
            let system_prompt = agx::planner::prompts::build_system_prompt(&context);
            let user_prompt = agx::planner::prompts::build_user_prompt(&instruction, &context);
            
            let plan_prompt = format!("{}\n\n{}", system_prompt, user_prompt);
            
            let history = vec![agx::planner::ChatMessage::user(plan_prompt)];
            let plan_response = backend.chat(&history, &context).await?;
            
            if let Ok(_) = serde_json::from_str::<serde_json::Value>(&plan_response) {
                let example = TrainingExample {
                    messages: vec![
                        ChatMessage { role: "system".to_string(), content: system_prompt },
                        ChatMessage { role: "user".to_string(), content: instruction },
                        ChatMessage { role: "assistant".to_string(), content: plan_response },
                    ],
                };
                examples.push(example);
            }
        }
    }


    // 5. Save to file
    let mut file = std::fs::File::create("dataset.jsonl")?;
    for example in examples {
        let json = serde_json::to_string(&example)?;
        writeln!(file, "{}", json)?;
    }

    println!("Generated {} examples in dataset.jsonl", file.metadata()?.len());
    Ok(())
}
