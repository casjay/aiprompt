// Copyright (c) 2026 casapps
// Licensed under the MIT License. See LICENSE.md in the project root for license information.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Context, Result};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub provider: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(provider: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            provider,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    fn get_storage_path() -> Result<PathBuf> {
        let mut path = dirs::home_dir().context("Could not find home directory")?;
        path.push(".aiprompt");
        path.push("sessions");
        fs::create_dir_all(&path).context("Failed to create session directory")?;
        Ok(path)
    }

    pub fn save(&self) -> Result<()> {
        let mut path = Self::get_storage_path()?;
        path.push(format!("{}.json", self.id));
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).context("Failed to write session file")?;
        Ok(())
    }

    pub fn load(id: Uuid) -> Result<Self> {
        let mut path = Self::get_storage_path()?;
        path.push(format!("{}.json", id));

        let content = fs::read_to_string(path).context("Failed to read session file")?;
        let session: Session = serde_json::from_str(&content).context("Failed to parse session file")?;
        Ok(session)
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
