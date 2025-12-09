//! SQLite database implementation for persistent storage

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, error, info};

use crate::core::{
    Instance, InstanceConfig, InstanceId, InstanceStatus, Profile, ProfileId, Settings,
};

/// Database wrapper for SQLite operations
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create a new database connection
    pub fn new() -> Result<Self> {
        let db_path = Self::get_database_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .context(format!("Failed to open database at {:?}", db_path))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        info!("Database opened at {:?}", db_path);
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Get the database file path
    fn get_database_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("MultiInstance");
        Ok(data_dir.join("multiinstance.db"))
    }

    /// Initialize database schema
    pub fn initialize(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute_batch(
            r#"
            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Instances table
            CREATE TABLE IF NOT EXISTS instances (
                id TEXT PRIMARY KEY,
                config TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                started_at TEXT,
                stopped_at TEXT,
                restart_count INTEGER DEFAULT 0,
                last_error TEXT
            );

            -- Profiles table
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );

            -- Quick launch items
            CREATE TABLE IF NOT EXISTS quick_launch (
                idx INTEGER PRIMARY KEY,
                config TEXT NOT NULL
            );

            -- Groups
            CREATE TABLE IF NOT EXISTS groups (
                name TEXT PRIMARY KEY
            );

            -- Recent apps
            CREATE TABLE IF NOT EXISTS recent_apps (
                idx INTEGER PRIMARY KEY,
                path TEXT NOT NULL
            );

            -- Session state (for restore)
            CREATE TABLE IF NOT EXISTS session (
                id TEXT PRIMARY KEY,
                config TEXT NOT NULL
            );

            -- Instance history
            CREATE TABLE IF NOT EXISTS instance_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                instance_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_time TEXT NOT NULL,
                details TEXT
            );
            "#,
        )?;

        info!("Database schema initialized");
        Ok(())
    }

    // === Settings ===

    /// Load settings from database
    pub fn load_settings(&self) -> Result<Option<Settings>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'app_settings'")?;
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;

        match result {
            Some(json) => {
                let mut settings: Settings =
                    serde_json::from_str(&json).context("Failed to deserialize settings")?;
                // Validate and fix any invalid values after deserialization
                settings.validate();
                Ok(Some(settings))
            }
            None => Ok(None),
        }
    }

    /// Save settings to database
    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let json = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('app_settings', ?1)",
            params![json],
        )?;
        debug!("Settings saved");
        Ok(())
    }

    // === Instances ===

    /// Save an instance to database
    pub fn save_instance(&self, instance: &Instance) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let config_json = serde_json::to_string(&instance.config)?;
        let status_str = format!("{:?}", instance.status);

        conn.execute(
            r#"
            INSERT OR REPLACE INTO instances
            (id, config, status, created_at, started_at, stopped_at, restart_count, last_error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                instance.id.to_string(),
                config_json,
                status_str,
                instance.created_at.to_rfc3339(),
                instance.started_at.map(|t| t.to_rfc3339()),
                instance.stopped_at.map(|t| t.to_rfc3339()),
                instance.restart_count,
                instance.last_error,
            ],
        )?;

        debug!("Instance {} saved", instance.id);
        Ok(())
    }

    /// Update instance status
    pub fn update_instance_status(&self, id: InstanceId, status: &InstanceStatus) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let status_str = format!("{:?}", status);
        conn.execute(
            "UPDATE instances SET status = ?1 WHERE id = ?2",
            params![status_str, id.to_string()],
        )?;
        Ok(())
    }

    /// Load all instances from database
    pub fn load_all_instances(&self) -> Result<Vec<Instance>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, config, status, created_at, started_at, stopped_at, restart_count, last_error FROM instances"
        )?;

        let instances = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let config_json: String = row.get(1)?;
            let _status_str: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let started_at_str: Option<String> = row.get(4)?;
            let stopped_at_str: Option<String> = row.get(5)?;
            let restart_count: u32 = row.get(6)?;
            let last_error: Option<String> = row.get(7)?;

            Ok((
                id_str,
                config_json,
                created_at_str,
                started_at_str,
                stopped_at_str,
                restart_count,
                last_error,
            ))
        })?;

        let mut result = Vec::new();
        for row in instances {
            let (
                id_str,
                config_json,
                created_at_str,
                started_at_str,
                stopped_at_str,
                restart_count,
                last_error,
            ) = row?;

            let id = uuid::Uuid::parse_str(&id_str)
                .map(InstanceId)
                .unwrap_or_else(|_| InstanceId::new());

            let config: InstanceConfig = match serde_json::from_str(&config_json) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to deserialize instance config: {}", e);
                    continue;
                }
            };

            let mut instance = Instance::new(config);
            instance.id = id;
            instance.status = InstanceStatus::Stopped; // Always start as stopped
            instance.created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|t| t.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            instance.started_at = started_at_str
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|t| t.with_timezone(&chrono::Utc));
            instance.stopped_at = stopped_at_str
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|t| t.with_timezone(&chrono::Utc));
            instance.restart_count = restart_count;
            instance.last_error = last_error;

            result.push(instance);
        }

        Ok(result)
    }

    /// Delete an instance from database
    pub fn delete_instance(&self, id: InstanceId) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "DELETE FROM instances WHERE id = ?1",
            params![id.to_string()],
        )?;
        debug!("Instance {} deleted", id);
        Ok(())
    }

    // === Profiles ===

    /// Save a profile to database
    pub fn save_profile(&self, profile: &Profile) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let json = serde_json::to_string(profile)?;
        conn.execute(
            "INSERT OR REPLACE INTO profiles (id, data) VALUES (?1, ?2)",
            params![profile.id.to_string(), json],
        )?;
        debug!("Profile {} saved", profile.id);
        Ok(())
    }

    /// Load all profiles from database
    pub fn load_all_profiles(&self) -> Result<Vec<Profile>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT data FROM profiles")?;
        let profiles = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut result = Vec::new();
        for json in profiles {
            let json = json?;
            match serde_json::from_str::<Profile>(&json) {
                Ok(profile) => result.push(profile),
                Err(e) => error!("Failed to deserialize profile: {}", e),
            }
        }

        Ok(result)
    }

    /// Delete a profile from database
    pub fn delete_profile(&self, id: ProfileId) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "DELETE FROM profiles WHERE id = ?1",
            params![id.to_string()],
        )?;
        debug!("Profile {} deleted", id);
        Ok(())
    }

    // === Quick Launch ===

    /// Load quick launch items
    pub fn load_quick_launch(&self) -> Result<Vec<InstanceConfig>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT config FROM quick_launch ORDER BY idx")?;
        let items = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut result = Vec::new();
        for json in items {
            let json = json?;
            match serde_json::from_str::<InstanceConfig>(&json) {
                Ok(config) => result.push(config),
                Err(e) => error!("Failed to deserialize quick launch item: {}", e),
            }
        }

        Ok(result)
    }

    /// Save quick launch items
    pub fn save_quick_launch(&self, items: &[InstanceConfig]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM quick_launch", [])?;

        for (idx, config) in items.iter().enumerate() {
            let json = serde_json::to_string(config)?;
            conn.execute(
                "INSERT INTO quick_launch (idx, config) VALUES (?1, ?2)",
                params![idx as i64, json],
            )?;
        }

        debug!("Quick launch items saved");
        Ok(())
    }

    // === Groups ===

    /// Load groups
    pub fn load_groups(&self) -> Result<Vec<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT name FROM groups ORDER BY name")?;
        let groups = stmt.query_map([], |row| row.get(0))?;

        let mut result = Vec::new();
        for group in groups {
            result.push(group?);
        }

        Ok(result)
    }

    /// Save groups
    pub fn save_groups(&self, groups: &[String]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM groups", [])?;

        for group in groups {
            conn.execute("INSERT INTO groups (name) VALUES (?1)", params![group])?;
        }

        debug!("Groups saved");
        Ok(())
    }

    // === Recent Apps ===

    /// Load recent apps
    pub fn load_recent_apps(&self) -> Result<Vec<PathBuf>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT path FROM recent_apps ORDER BY idx")?;
        let apps = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            Ok(PathBuf::from(path))
        })?;

        let mut result = Vec::new();
        for app in apps {
            result.push(app?);
        }

        Ok(result)
    }

    /// Save recent apps
    pub fn save_recent_apps(&self, apps: &[PathBuf]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM recent_apps", [])?;

        for (idx, path) in apps.iter().enumerate() {
            conn.execute(
                "INSERT INTO recent_apps (idx, path) VALUES (?1, ?2)",
                params![idx as i64, path.to_string_lossy().to_string()],
            )?;
        }

        debug!("Recent apps saved");
        Ok(())
    }

    // === Session ===

    /// Save session state
    pub fn save_session(&self, instances: &[&Instance]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM session", [])?;

        for instance in instances {
            let json = serde_json::to_string(&instance.config)?;
            conn.execute(
                "INSERT INTO session (id, config) VALUES (?1, ?2)",
                params![instance.id.to_string(), json],
            )?;
        }

        debug!("Session saved with {} instances", instances.len());
        Ok(())
    }

    /// Load session state
    pub fn load_session(&self) -> Result<Vec<InstanceConfig>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT config FROM session")?;
        let configs = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut result = Vec::new();
        for json in configs {
            let json = json?;
            match serde_json::from_str::<InstanceConfig>(&json) {
                Ok(config) => result.push(config),
                Err(e) => error!("Failed to deserialize session config: {}", e),
            }
        }

        Ok(result)
    }

    /// Clear session
    pub fn clear_session(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM session", [])?;
        Ok(())
    }

    // === History ===

    /// Record an instance event
    pub fn record_instance_event(
        &self,
        instance_id: InstanceId,
        event_type: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            r#"
            INSERT INTO instance_history (instance_id, event_type, event_time, details)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                instance_id.to_string(),
                event_type,
                chrono::Utc::now().to_rfc3339(),
                details,
            ],
        )?;
        Ok(())
    }

    /// Get instance history
    pub fn get_instance_history(
        &self,
        instance_id: InstanceId,
    ) -> Result<Vec<(String, String, Option<String>)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT event_type, event_time, details FROM instance_history WHERE instance_id = ?1 ORDER BY event_time DESC"
        )?;

        let history = stmt.query_map(params![instance_id.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut result = Vec::new();
        for event in history {
            result.push(event?);
        }

        Ok(result)
    }

    /// Clean up old history entries
    pub fn cleanup_history(&self, retention_days: u32) -> Result<usize> {
        if retention_days == 0 {
            return Ok(0); // Keep forever
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let cutoff = chrono::Utc::now()
            - chrono::TimeDelta::try_days(retention_days as i64)
                .unwrap_or_else(|| chrono::TimeDelta::days(30));
        let count = conn.execute(
            "DELETE FROM instance_history WHERE event_time < ?1",
            params![cutoff.to_rfc3339()],
        )?;

        debug!("Cleaned up {} old history entries", count);
        Ok(count)
    }
}
