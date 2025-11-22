use anyhow::{Context, Result};
use hf_hub::{api::tokio::Api, Repo, RepoType};
use std::path::PathBuf;

pub struct ModelManager {
    api: Api,
}

use hf_hub::api::tokio::ApiBuilder;

impl ModelManager {
    pub fn new() -> Result<Self> {
        let api = ApiBuilder::new()
            .with_progress(true)
            .build()
            .context("Failed to initialize Hugging Face API client")?;
        Ok(Self { api })
    }

    /// Ensures a model file exists locally, downloading it if necessary.
    /// Returns the path to the local file.
    pub async fn ensure_model(&self, repo_id: &str, filename: &str) -> Result<PathBuf> {
        println!("Checking for model: {}/{}", repo_id, filename);
        
        let repo = self.api.repo(Repo::new(repo_id.to_string(), RepoType::Model));
        
        // download method automatically checks cache and downloads if missing
        let path = repo.download(filename).await
            .map_err(|e| {
                println!("Error downloading {} from {}: {:?}", filename, repo_id, e);
                e
            })
            .context(format!("Failed to download model {} from {}", filename, repo_id))?;
            
        println!("Model available at: {}", path.display());
        Ok(path)
    }

    /// Manually download a file from a URL to the local cache
    pub async fn download_file_raw(&self, url: &str, filename: &str) -> Result<PathBuf> {
        println!("Downloading raw file: {}", url);
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?
            .join(".cache/agenix/models/raw");
            
        tokio::fs::create_dir_all(&cache_dir).await?;
        let path = cache_dir.join(filename);
        
        if path.exists() {
            println!("File already exists: {}", path.display());
            return Ok(path);
        }
        
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download file: {}", response.status()));
        }
        
        let content = response.bytes().await?;
        tokio::fs::write(&path, content).await?;
        
        println!("Downloaded to: {}", path.display());
        Ok(path)
    }
}
