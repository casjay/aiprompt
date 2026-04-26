// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use crate::session::{Session, Role};
use crate::config::Config; // Import Config
use crate::normalization::NormalizationEngine; // Import NormalizationEngine
use anyhow::{Context, Result, anyhow};
use std::process::Command;
use async_trait::async_trait;
use reqwest;
use serde::{Deserialize, Serialize}; // Added serde imports

#[async_trait]
pub trait Provider {
    async fn send(&self, session: &Session, config: &Config) -> Result<String>; // Added config: &Config
    fn name(&self) -> &str;
}

pub struct ClaudeWrapper;

impl ClaudeWrapper {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for ClaudeWrapper {
    fn name(&self) -> &str {
        "claude"
    }

    async fn send(&self, session: &Session, config: &Config) -> Result<String> {
        let mut full_prompt = String::new();
        for msg in &session.messages {
            let prefix = match msg.role {
                Role::User => "User: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
            };
            full_prompt.push_str(prefix);
            full_prompt.push_str(&msg.content);
            full_prompt.push_str("

");
        }

        let mut command = Command::new("claude");
        command.arg("-p").arg(&full_prompt);
        command.envs(std::env::vars()); // Inherit environment variables from parent process

        // Translate permissions and add them as arguments or environment variables
        let translated_permissions = NormalizationEngine::translate_permissions(config, self.name());
        for (key, flags) in translated_permissions {
            if key == "CLAUDE_CODE_ACTION" {
                // Assuming CLAUDE_CODE_ACTION will have a single value
                if let Some(value) = flags.get(0) {
                    command.env(key, value);
                }
            } else {
                // Assume other keys represent categories for CLI arguments
                for flag in flags {
                    command.arg(flag);
                }
            }
        }
        
        let output = command
            .output()
            .await
            .context("Failed to execute 'claude' command. Is @anthropic-ai/claude-code installed?")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(anyhow!("Claude CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}

pub struct CopilotWrapper;

impl CopilotWrapper {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CopilotWrapper {
    fn name(&self) -> &str {
        "copilot"
    }

    async fn send(&self, session: &Session, config: &Config) -> Result<String> {
        let mut full_prompt = String::new();
        for msg in &session.messages {
             let prefix = match msg.role {
                Role::User => "User: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
            };
            full_prompt.push_str(prefix);
            full_prompt.push_str(&msg.content);
            full_prompt.push_str("\n\n");
        }

        let mut command = Command::new("copilot");
        command.arg("-p").arg(&full_prompt); // Use -p or --prompt for non-interactive mode
        command.envs(std::env::vars()); // Inherit environment variables from parent process

        // Translate permissions and add them as arguments or environment variables
        let translated_permissions = NormalizationEngine::translate_permissions(config, self.name());
        for (key, flags) in translated_permissions {
            if key == "CLAUDE_CODE_ACTION" { // Currently, only Claude uses this, but good to keep consistent
                // Assuming CLAUDE_CODE_ACTION will have a single value
                if let Some(value) = flags.get(0) {
                    command.env(key, value);
                }
            } else {
                // Assume other keys represent categories for CLI arguments
                for flag in flags {
                    command.arg(flag);
                }
            }
        }
        
        let output = command
            .output()
            .await
            .context("Failed to execute 'copilot' command. Is the GitHub Copilot CLI installed?")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(anyhow!("Copilot CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}

pub struct CodexWrapper;

impl CodexWrapper {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CodexWrapper {
    fn name(&self) -> &str {
        "codex"
    }

    async fn send(&self, session: &Session, config: &Config) -> Result<String> {
        let mut full_prompt = String::new();
        for msg in &session.messages {
             let prefix = match msg.role {
                Role::User => "User: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
            };
            full_prompt.push_str(prefix);
            full_prompt.push_str(&msg.content);
            full_prompt.push_str("\n\n");
        }

        let mut command = Command::new("codex");
        command.arg("-p").arg(&full_prompt); // Assuming -p for prompt, can be adjusted
        command.envs(std::env::vars()); // Inherit environment variables from parent process

        // Translate permissions and add them as arguments or environment variables
        // Codex specific permission handling can be added here if needed
        let translated_permissions = NormalizationEngine::translate_permissions(config, self.name());
        for (key, flags) in translated_permissions {
            // If Codex uses specific environment variables or CLI flags for permissions,
            // they would be handled here. For now, assuming it doesn't have unique ones
            // like CLAUDE_CODE_ACTION, but will pass generic CLI flags if present.
            for flag in flags {
                command.arg(flag);
            }
        }
        
        let output = command
            .output()
            .await
            .context("Failed to execute 'codex' command. Is the Codex CLI installed?")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(anyhow!("Codex CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}

// Structs for Ollama API
#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String, // TODO: Make configurable
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

pub struct OllamaWrapper {
    client: reqwest::Client,
}

impl OllamaWrapper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for OllamaWrapper {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn send(&self, session: &Session, config: &Config) -> Result<String> {
        let ollama_host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let url = format!("{}/api/chat", ollama_host);

        let ollama_messages: Vec<OllamaMessage> = session.messages.iter().map(|msg| {
            OllamaMessage {
                role: format!("{:?}", msg.role).to_lowercase(),
                content: msg.content.clone(),
            }
        }).collect();

        // TODO: Make model configurable via config.yaml
        let request_body = OllamaRequest {
            model: "llama2".to_string(), // Default model, make configurable
            messages: ollama_messages,
            stream: false, // For simplicity, don't stream for now
        };

        let response = self.client.post(&url)
            .json(&request_body)
            .send()
            .await
            .context(format!("Failed to send request to Ollama API at {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Ollama API request failed with status: {} - {}", status, text);
        }

        let ollama_response: OllamaResponse = response.json().await
            .context("Failed to parse Ollama API response")?;

        Ok(ollama_response.message.content.trim().to_string())
    }
}

pub struct OpencodeWrapper;

impl OpencodeWrapper {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for OpencodeWrapper {
    fn name(&self) -> &str {
        "opencode"
    }

    async fn send(&self, session: &Session, config: &Config) -> Result<String> {
        let mut full_prompt = String::new();
        for msg in &session.messages {
             let prefix = match msg.role {
                Role::User => "User: ",
                Role::Assistant => "Assistant: ",
                Role::System => "System: ",
            };
            full_prompt.push_str(prefix);
            full_prompt.push_str(&msg.content);
            full_prompt.push_str("\n\n");
        }

        let mut command = Command::new("opencode");
        command.arg("-p").arg(&full_prompt); // Assuming -p for prompt, can be adjusted
        command.envs(std::env::vars()); // Inherit environment variables from parent process

        // Translate permissions and add them as arguments or environment variables
        // Opencode specific permission handling can be added here if needed
        let translated_permissions = NormalizationEngine::translate_permissions(config, self.name());
        for (key, flags) in translated_permissions {
            // If Opencode uses specific environment variables or CLI flags for permissions,
            // they would be handled here. For now, assuming it doesn't have unique ones
            // like CLAUDE_CODE_ACTION, but will pass generic CLI flags if present.
            for flag in flags {
                command.arg(flag);
            }
        }
        
        let output = command
            .output()
            .await
            .context("Failed to execute 'opencode' command. Is the Opencode CLI installed?")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(anyhow!("Opencode CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}