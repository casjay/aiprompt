// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use crate::session::{Session, Role};
use anyhow::{Context, Result, anyhow};
use std::process::Command;
use async_trait::async_trait;

#[async_trait]
pub trait Provider {
    async fn send(&self, session: &Session) -> Result<String>;
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

    async fn send(&self, session: &Session) -> Result<String> {
        // Construct the full prompt from history
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

        // The claude CLI uses -p for prompt.
        // We assume the prompt is the *entire* context we want it to see.
        // Or, more likely, we just want to send the *last* user message if the tool manages context?
        // But the user said *we* manage context. "Wrapper architecture".
        // If we send full history to `claude -p`, it might treat it as a single big prompt.
        // This is acceptable for a "stateless wrapper" approach.

        let output = Command::new("claude")
            .arg("-p")
            .arg(&full_prompt)
            .output()
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

    async fn send(&self, session: &Session) -> Result<String> {
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

        // Standalone copilot binary
        let output = Command::new("copilot")
            .arg("explain") // Using explain as a placeholder for prompting
            .arg(&full_prompt)
            .output()
            .context("Failed to execute 'copilot' command. Is the GitHub Copilot CLI installed?")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(anyhow!("Copilot CLI failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}
