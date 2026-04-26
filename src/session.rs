// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::config::AppDirs; // Import AppDirs

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Manages the mapping between current working directories and session IDs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DirectoryMap {
    pub mappings: HashMap<PathBuf, Uuid>,
}

impl DirectoryMap {
    /// Determines the platform-specific path for the directory map file.
    fn get_map_path() -> Result<PathBuf> {
        let app_dirs = AppDirs::get_app_dirs()?;
        let path = app_dirs.data_dir.join("directory_map.json");
        Ok(path)
    }

    /// Loads the directory map from disk. If the file doesn't exist, returns an empty map.
    pub fn load() -> Result<Self> {
        let path = Self::get_map_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).context("Failed to read directory map file")?;
        let map: DirectoryMap = serde_json::from_str(&content).context("Failed to parse directory map file")?;
        Ok(map)
    }

    /// Saves the current directory map to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_map_path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).context("Failed to write directory map file")?;
        Ok(())
    }

    /// Associates a session ID with a given directory.
    pub fn set_session_for_dir(&mut self, dir: PathBuf, session_id: Uuid) {
        self.mappings.insert(dir, session_id);
    }

    /// Retrieves the session ID for a given directory, if it exists.
    pub fn get_session_for_dir(&self, dir: &Path) -> Option<&Uuid> {
        self.mappings.get(dir)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub provider: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(provider: String, system_message: Option<String>) -> Result<Self> {
        let now = Utc::now();
        let mut messages = Vec::new();

        if let Some(msg) = system_message {
            messages.push(Message {
                role: Role::System,
                content: msg,
                timestamp: now,
            });
        }

        let session = Self {
            id: Uuid::new_v4(),
            provider,
            messages,
            created_at: now,
            updated_at: now,
        };

        // Update DirectoryMap with the new session
        let mut dir_map = DirectoryMap::load()?;
        let current_dir = std::env::current_dir().context("Failed to get current working directory")?;
        dir_map.set_session_for_dir(current_dir, session.id);
        dir_map.save()?;

        Ok(session)
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    /// Determines the platform-specific storage path for session files.
    fn get_storage_path() -> Result<PathBuf> {
        let app_dirs = AppDirs::get_app_dirs()?;
        let path = app_dirs.data_dir.join("sessions");
        fs::create_dir_all(&path).context("Failed to create session directory")?;
        Ok(path)
    }

    pub fn save(&self) -> Result<()> {
        let mut path = Self::get_storage_path()?;
        path.push(format!("{}.json", self.id));
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).context("Failed to write session file")?;

        // Update DirectoryMap after saving the session
        let mut dir_map = DirectoryMap::load()?;
        let current_dir = std::env::current_dir().context("Failed to get current working directory")?;
        dir_map.set_session_for_dir(current_dir, self.id);
        dir_map.save()?;

        Ok(())
    }

    pub fn load(id: Uuid) -> Result<Self> {
        let mut path = Self::get_storage_path()?;
        path.push(format!("{}.json", id));

        let content = fs::read_to_string(path).context("Failed to read session file")?;
        let session: Session = serde_json::from_str(&content).context("Failed to parse session file")?;
        Ok(session)
    }

    /// Retrieves the session associated with the current working directory, if one exists.
    pub fn get_session_for_cwd() -> Result<Option<Self>> {
        let dir_map = DirectoryMap::load()?;
        let current_dir = std::env::current_dir().context("Failed to get current working directory")?;

        if let Some(session_id) = dir_map.get_session_for_dir(&current_dir) {
            match Session::load(*session_id) {
                Ok(session) => Ok(Some(session)),
                Err(_) => {
                    // If the session file doesn't exist, remove it from the map
                    let mut updated_map = dir_map;
                    updated_map.mappings.remove(&current_dir);
                    updated_map.save()?;
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn list_sessions() -> Result<Vec<Session>> {
        let path = Self::get_storage_path()?;
        let mut sessions = Vec::new();

        if path.exists() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }
        // Sort by updated_at descending
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }
    
    pub fn get_last_session() -> Result<Option<Session>> {
        let sessions = Self::list_sessions()?;
        Ok(sessions.into_iter().next())
    }
}

