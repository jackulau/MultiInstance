//! Instance management - Represents a single application instance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::resource::{ResourceLimits, ResourceUsage};

/// Unique identifier for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(pub Uuid);

impl InstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceStatus {
    /// Instance is starting up
    Starting,
    /// Instance is running normally
    Running,
    /// Instance is paused/suspended
    Paused,
    /// Instance is stopping
    Stopping,
    /// Instance has stopped
    Stopped,
    /// Instance has crashed
    Crashed,
    /// Instance status is unknown
    Unknown,
}

impl InstanceStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Paused)
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Starting => egui::Color32::from_rgb(251, 191, 36), // Yellow
            Self::Running => egui::Color32::from_rgb(34, 197, 94),   // Green
            Self::Paused => egui::Color32::from_rgb(59, 130, 246),   // Blue
            Self::Stopping => egui::Color32::from_rgb(251, 146, 60), // Orange
            Self::Stopped => egui::Color32::from_rgb(156, 163, 175), // Gray
            Self::Crashed => egui::Color32::from_rgb(239, 68, 68),   // Red
            Self::Unknown => egui::Color32::from_rgb(107, 114, 128), // Dark gray
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Stopping => "Stopping",
            Self::Stopped => "Stopped",
            Self::Crashed => "Crashed",
            Self::Unknown => "Unknown",
        }
    }
}

/// Configuration for launching an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    /// Display name for the instance
    pub name: String,
    /// Path to the executable
    pub executable_path: PathBuf,
    /// Command line arguments
    pub arguments: Vec<String>,
    /// Working directory (defaults to executable's directory)
    pub working_directory: Option<PathBuf>,
    /// Environment variables to set
    pub environment: Vec<(String, String)>,
    /// Resource limits for this instance
    pub resource_limits: ResourceLimits,
    /// Custom data directory for isolation
    pub data_directory: Option<PathBuf>,
    /// Whether to bypass single-instance checks
    pub bypass_single_instance: bool,
    /// Whether to use environment isolation (set custom APPDATA, etc.)
    /// Disable this for games with anti-cheat
    #[serde(default)]
    pub use_environment_isolation: bool,
    /// Group/category for organization
    pub group: Option<String>,
    /// Custom icon path
    pub icon_path: Option<PathBuf>,
    /// Notes/description
    pub notes: String,
    /// Auto-restart on crash
    pub auto_restart: bool,
    /// Restart delay in seconds
    pub restart_delay_secs: u32,
    /// Hide instance window from taskbar
    #[serde(default)]
    pub hide_from_taskbar: bool,
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            executable_path: PathBuf::new(),
            arguments: Vec::new(),
            working_directory: None,
            environment: Vec::new(),
            resource_limits: ResourceLimits::default(),
            data_directory: None,
            bypass_single_instance: true,
            use_environment_isolation: false, // Default OFF for compatibility with anti-cheat
            group: None,
            icon_path: None,
            notes: String::new(),
            auto_restart: false,
            restart_delay_secs: 5,
            hide_from_taskbar: false,
        }
    }
}

impl InstanceConfig {
    pub fn new(name: impl Into<String>, executable_path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            executable_path: executable_path.into(),
            ..Default::default()
        }
    }

    pub fn with_arguments(mut self, args: Vec<String>) -> Self {
        self.arguments = args;
        self
    }

    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn with_data_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_directory = Some(path.into());
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// Represents a managed application instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// Unique identifier
    pub id: InstanceId,
    /// Configuration used to launch this instance
    pub config: InstanceConfig,
    /// Current status
    pub status: InstanceStatus,
    /// Operating system process ID
    pub pid: Option<u32>,
    /// When the instance was created
    pub created_at: DateTime<Utc>,
    /// When the instance was last started
    pub started_at: Option<DateTime<Utc>>,
    /// When the instance stopped
    pub stopped_at: Option<DateTime<Utc>>,
    /// Current resource usage
    #[serde(skip)]
    pub resource_usage: ResourceUsage,
    /// Number of restarts
    pub restart_count: u32,
    /// Last error message if crashed
    pub last_error: Option<String>,
}

impl Instance {
    pub fn new(config: InstanceConfig) -> Self {
        Self {
            id: InstanceId::new(),
            config,
            status: InstanceStatus::Stopped,
            pid: None,
            created_at: Utc::now(),
            started_at: None,
            stopped_at: None,
            resource_usage: ResourceUsage::default(),
            restart_count: 0,
            last_error: None,
        }
    }

    /// Get the display name, falling back to executable name
    pub fn display_name(&self) -> &str {
        if self.config.name.is_empty() {
            self.config
                .executable_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
        } else {
            &self.config.name
        }
    }

    /// Get uptime duration if running
    pub fn uptime(&self) -> Option<chrono::Duration> {
        self.started_at.map(|started| Utc::now() - started)
    }

    /// Format uptime as human-readable string
    pub fn uptime_string(&self) -> String {
        match self.uptime() {
            Some(duration) => {
                let secs = duration.num_seconds();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else if secs < 86400 {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                } else {
                    format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
                }
            }
            None => "-".to_string(),
        }
    }

    /// Check if instance should be auto-restarted
    pub fn should_auto_restart(&self) -> bool {
        self.config.auto_restart && matches!(self.status, InstanceStatus::Crashed)
    }

    /// Mark instance as starting
    pub fn mark_starting(&mut self, pid: u32) {
        self.status = InstanceStatus::Starting;
        self.pid = Some(pid);
        self.started_at = Some(Utc::now());
        self.stopped_at = None;
        self.last_error = None;
    }

    /// Mark instance as running
    pub fn mark_running(&mut self) {
        self.status = InstanceStatus::Running;
    }

    /// Mark instance as stopped
    pub fn mark_stopped(&mut self) {
        self.status = InstanceStatus::Stopped;
        self.stopped_at = Some(Utc::now());
        self.resource_usage = ResourceUsage::default();
    }

    /// Mark instance as crashed
    pub fn mark_crashed(&mut self, error: Option<String>) {
        self.status = InstanceStatus::Crashed;
        self.stopped_at = Some(Utc::now());
        self.last_error = error;
        self.resource_usage = ResourceUsage::default();
    }

    /// Mark instance as paused
    pub fn mark_paused(&mut self) {
        self.status = InstanceStatus::Paused;
    }

    /// Increment restart counter
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }

    /// Update resource usage
    pub fn update_resource_usage(&mut self, usage: ResourceUsage) {
        self.resource_usage = usage;
    }
}
