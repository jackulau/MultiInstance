//! Profile management - Saved launch configurations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::instance::InstanceConfig;

/// Unique identifier for a profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(pub Uuid);

impl ProfileId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProfileId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A saved profile containing one or more instance configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier
    pub id: ProfileId,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category/group
    pub category: Option<String>,
    /// Instance configurations in this profile
    pub instances: Vec<InstanceConfig>,
    /// Launch instances in sequence with delay
    pub staggered_launch: bool,
    /// Delay between instance launches in ms
    pub launch_delay_ms: u32,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// When the profile was last modified
    pub modified_at: DateTime<Utc>,
    /// When the profile was last used
    pub last_used_at: Option<DateTime<Utc>>,
    /// Number of times this profile has been launched
    pub launch_count: u32,
    /// Whether this is a favorite profile
    pub is_favorite: bool,
    /// Custom icon path
    pub icon_path: Option<std::path::PathBuf>,
    /// Tags for organization
    pub tags: Vec<String>,
}

impl Profile {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: ProfileId::new(),
            name: name.into(),
            description: String::new(),
            category: None,
            instances: Vec::new(),
            staggered_launch: false,
            launch_delay_ms: 1000,
            created_at: now,
            modified_at: now,
            last_used_at: None,
            launch_count: 0,
            is_favorite: false,
            icon_path: None,
            tags: Vec::new(),
        }
    }

    /// Add an instance configuration to this profile
    pub fn add_instance(&mut self, config: InstanceConfig) {
        self.instances.push(config);
        self.modified_at = Utc::now();
    }

    /// Remove an instance configuration by index
    pub fn remove_instance(&mut self, index: usize) -> Option<InstanceConfig> {
        if index < self.instances.len() {
            self.modified_at = Utc::now();
            Some(self.instances.remove(index))
        } else {
            None
        }
    }

    /// Mark profile as used
    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.launch_count += 1;
    }

    /// Mark profile as modified
    pub fn mark_modified(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        self.modified_at = Utc::now();
    }

    /// Get the number of instances in this profile
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Check if profile has any instances
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.modified_at = Utc::now();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.modified_at = Utc::now();
        }
    }

    /// Export profile to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import profile from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
