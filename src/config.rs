// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Context, Result};

// Configuration settings for aiprompt
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // The default AI provider to use (e.g., "claude", "copilot")
    pub provider: Option<String>,
    // Environment variables to be exported to the underlying tools
    pub env: Option<HashMap<String, String>>,
}

impl Default for Config {
    // Sane defaults for the application
    fn default() -> Self {
        Self {
            provider: Some("claude".to_string()),
            env: None,
        }
    }
}

impl Config {
    // Determine the platform-specific path for the configuration file
    fn get_config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir().context("Could not find config directory")?;
        path.push("aiprompt");
        fs::create_dir_all(&path).context("Failed to create config directory")?;
        
        // Support both .yml and .yaml extensions
        let yaml_path = path.join("config.yaml");
        if yaml_path.exists() {
            return Ok(yaml_path);
        }
        Ok(path.join("config.yml"))
    }

    // Load configuration from disk or return defaults if missing
    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path).context("Failed to read config file")?;
        let config: Config = serde_yaml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    // Save the current configuration to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path()?;
        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(path, content).context("Failed to write config file")?;
        Ok(())
    }
}
