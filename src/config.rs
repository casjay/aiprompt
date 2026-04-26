// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt; // For setting file permissions
use nix::unistd; // For checking root user

/// Represents a single permission rule, which can be to either "allow" or "deny" access.
/// The rule is generic and can be applied to files, directories, commands, or web access.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Rule {
    Allow { allow: String },
    Deny { deny: String },
}

/// Defines the set of permissions for the AI tool, including access controls for
/// files, directories, commands, and web URLs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Permissions {
    #[serde(default)]
    pub files: Vec<Rule>,
    #[serde(default)]
    pub directories: Vec<Rule>,
    #[serde(default)]
    pub commands: Vec<Rule>,
    #[serde(default)]
    pub web: Vec<Rule>,
}

/// Contains the behavioral context for the AI, specifying rules that must always be
/// followed and behaviors that are never allowed.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Context {
    #[serde(default)]
    pub always: Vec<String>,
    #[serde(default)]
    pub never: Vec<String>,
}

/// Stores the resolved application directories for configuration, data, and logs.
#[derive(Debug, Clone)]
pub struct AppDirs {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppDirs {
    /// Determines and creates the platform-specific application directories.
    /// Handles Linux, macOS, Windows, and the special case for the root user.
    pub fn get_app_dirs() -> Result<Self> {
        let home_dir = dirs::home_dir().with_context(|| "Could not find home directory")?;

        let (mut config_base, mut data_base, mut log_base);

        if cfg!(target_os = "macos") {
            config_base = home_dir.join("Library/Application Support");
            data_base = home_dir.join("Library/Application Support");
            log_base = home_dir.join("Library/Logs");
        } else if cfg!(target_os = "windows") {
            config_base = dirs::data_dir().with_context(|| "Could not find AppData directory")?;
            data_base = dirs::data_local_dir().with_context(|| "Could not find LocalAppData directory")?;
            log_base = dirs::data_local_dir().with_context(|| "Could not find LocalAppData directory")?;
        } else {
            // Linux, BSD, Solaris
            config_base = dirs::config_dir().with_context(|| "Could not find config directory")?;
            data_base = dirs::data_local_dir().with_context(|| "Could not find local data directory")?;
            log_base = dirs::data_local_dir().with_context(|| "Could not find local data directory")?;

            // Adjust for root user on Unix-like systems as per AI.md specification
            #[cfg(target_family = "unix")]
            {
                if unistd::getuid().is_root() {
                    let root_home = PathBuf::from("/root");
                    config_base = root_home.join(".config");
                    data_base = root_home.join(".local/share");
                    log_base = root_home.join(".local/log");
                }
            }
        };

        let config_dir = config_base.join("aiprompt");
        let data_dir = data_base.join("aiprompt");
        let log_dir = log_base.join("aiprompt");

        // Create directories and set permissions
        for dir in [&config_dir, &data_dir, &log_dir] {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {:?}", dir))?;
            
            #[cfg(target_family = "unix")]
            {
                use std::fs::Permissions;
                fs::set_permissions(dir, Permissions::from_mode(0o700))
                    .with_context(|| format!("Failed to set permissions for directory: {:?}", dir))?;
            }
        }
        
        Ok(Self {
            config_dir,
            data_dir,
            log_dir,
        })
    }
}

/// Main configuration struct for the `aiprompt` application.
/// Holds all settings, including the default provider, environment variables,
/// permissions, and behavioral context.
#[derive(Serialize, Deserialize, Debug, Clone, Default)] // Re-added Default here
pub struct Config {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)] // Re-added serde(default) for context
    pub context: Context, 
}

impl Config {
    /// Determines the platform-specific path for the configuration file.
    /// It checks for `aiprompt/config.yaml` and `aiprompt/config.yml` in the default config directory.
    fn get_config_path() -> Result<PathBuf> {
        if let Ok(config_path_env) = std::env::var("AIPROMPT_CONFIG") {
            return Ok(PathBuf::from(config_path_env));
        }

        let app_dirs = AppDirs::get_app_dirs()?;
        let path = app_dirs.config_dir;
        
        let yaml_path = path.join("config.yaml");
        if yaml_path.exists() {
            return Ok(yaml_path);
        }
        Ok(path.join("config.yml"))
    }

    /// Loads the configuration from the user's config directory.
    /// If no configuration file is found, it returns a default configuration.
    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            // If config file doesn't exist, create it with defaults before loading.
            let default_config = Config::default();
            default_config.save()?; 
            return Ok(default_config);
        }

        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read config file from {:?}", path))?;
        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file from {:?}. Ensure it's valid YAML.", path))?;
        Ok(config)
    }

    /// Saves the current configuration to the user's config directory.
    /// The configuration is serialized to YAML format.
    pub fn save(&self) -> Result<()> {
        let app_dirs = AppDirs::get_app_dirs()?;
        let path = app_dirs.config_dir.join("config.yaml"); // Always save as config.yaml

        let content = serde_yaml::to_string(self).with_context(|| "Failed to serialize config")?;
        fs::write(&path, content).with_context(|| format!("Failed to write config file to {:?}", path))?;
        
        #[cfg(target_family = "unix")]
        {
            use std::fs::Permissions;
            fs::set_permissions(&path, Permissions::from_mode(0o600))
                .with_context(|| format!("Failed to set permissions for config file: {:?}", path))?;
        }
        Ok(())
    }
}