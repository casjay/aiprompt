// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use std::collections::HashMap;
use crate::config::{Config, Rule};

/// The NormalizationEngine is responsible for translating generic permission
/// rules from the Config into provider-specific flags or commands.
pub struct NormalizationEngine;

impl NormalizationEngine {
    /// Translates the application's generic permission rules into a format
    /// understood by a specific AI provider.
    ///
    /// # Arguments
    /// * `config` - A reference to the application's configuration.
    /// * `provider_name` - The name of the AI provider (e.g., "claude", "copilot").
    ///
    /// # Returns
    /// A `HashMap` where keys are environment variable names or categories
    /// and values are lists of provider-specific translated rules (CLI args or env var values).
    pub fn translate_permissions(config: &Config, provider_name: &str) -> HashMap<String, Vec<String>> {
        let mut translated_rules = HashMap::new();

        match provider_name {
            "claude" => {
                // Check if all permissions are set to "allow: all"
                let all_files_allowed = config.permissions.files.len() == 1
                    && matches!(&config.permissions.files[0], Rule::Allow { allow } if allow == "all");
                let all_dirs_allowed = config.permissions.directories.len() == 1
                    && matches!(&config.permissions.directories[0], Rule::Allow { allow } if allow == "all");
                let all_commands_allowed = config.permissions.commands.len() == 1
                    && matches!(&config.permissions.commands[0], Rule::Allow { allow } if allow == "all");
                let all_web_allowed = config.permissions.web.len() == 1
                    && matches!(&config.permissions.web[0], Rule::Allow { allow } if allow == "all");

                if all_files_allowed && all_dirs_allowed && all_commands_allowed && all_web_allowed {
                    translated_rules.insert("CLAUDE_CODE_ACTION".to_string(), vec!["bypassPermissions".to_string()]);
                }
                // If not "allow all" for all, we return an empty map for Claude permissions.
                // Claude will then handle granular permissions via its internal mechanisms (e.g., settings.json, interactive prompts).
            },
            "copilot" => {
                // Copilot CLI does not seem to have direct CLI arguments for permissions.
                // It relies on its own interactive prompts or settings.
                // Therefore, we return an empty map for permissions.
            },
            _ => {
                // Default behavior for other providers: translate to CLI arguments
                // Translate file permissions
                let translated_files = config.permissions.files.iter()
                    .map(|rule| Self::translate_file_rule(rule, provider_name))
                    .collect();
                translated_rules.insert("files".to_string(), translated_files);

                // Translate directory permissions
                let translated_directories = config.permissions.directories.iter()
                    .map(|rule| Self::translate_directory_rule(rule, provider_name))
                    .collect();
                translated_rules.insert("directories".to_string(), translated_directories);

                // Translate command permissions
                let translated_commands = config.permissions.commands.iter()
                    .map(|rule| Self::translate_command_rule(rule, provider_name))
                    .collect();
                translated_rules.insert("commands".to_string(), translated_commands);

                // Translate web permissions
                let translated_web = config.permissions.web.iter()
                    .map(|rule| Self::translate_web_rule(rule, provider_name))
                    .collect();
                translated_rules.insert("web".to_string(), translated_web);
            }
        }
        translated_rules
    }

    // Helper to translate file access rules
    fn translate_file_rule(rule: &Rule, provider_name: &str) -> String {
        match rule {
            Rule::Allow { allow } => format!("--allow-file '{}'", allow),
            Rule::Deny { deny } => format!("--deny-file '{}'", deny),
        }
    }

    // Helper to translate directory access rules
    fn translate_directory_rule(rule: &Rule, provider_name: &str) -> String {
        match rule {
            Rule::Allow { allow } => format!("--allow-dir '{}'", allow),
            Rule::Deny { deny } => format!("--deny-dir '{}'", deny),
        }
    }

    // Helper to translate command execution rules
    fn translate_command_rule(rule: &Rule, provider_name: &str) -> String {
        match rule {
            Rule::Allow { allow } => format!("--allow-command '{}'", allow),
            Rule::Deny { deny } => format!("--deny-command '{}'", deny),
        }
    }

    // Helper to translate web access rules
    fn translate_web_rule(rule: &Rule, provider_name: &str) -> String {
        match rule {
            Rule::Allow { allow } => format!("--allow-url '{}'", allow),
            Rule::Deny { deny } => format!("--deny-url '{}'", deny),
        }
    }
}

