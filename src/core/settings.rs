//! Application settings management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

impl Theme {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::System => "System",
        }
    }

    pub fn all() -> &'static [Theme] {
        &[Theme::Dark, Theme::Light, Theme::System]
    }
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationLevel {
    All,
    #[default]
    Important,
    None,
}

impl NotificationLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Important => "Important Only",
            Self::None => "None",
        }
    }

    pub fn all() -> &'static [NotificationLevel] {
        &[
            NotificationLevel::All,
            NotificationLevel::Important,
            NotificationLevel::None,
        ]
    }
}

/// View mode for instance list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
    Compact,
}

impl ViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Grid => "Grid",
            Self::List => "List",
            Self::Compact => "Compact",
        }
    }

    pub fn all() -> &'static [ViewMode] {
        &[ViewMode::Grid, ViewMode::List, ViewMode::Compact]
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // General
    /// Start MultiInstance with system
    pub start_with_system: bool,
    /// Minimize to system tray instead of closing
    pub minimize_to_tray: bool,
    /// Automatically restore previous session
    pub auto_restore_sessions: bool,
    /// Application theme
    pub theme: Theme,
    /// Default view mode
    pub view_mode: ViewMode,
    /// Show system resource overview
    pub show_system_resources: bool,

    // Default Resource Limits
    /// Default CPU limit for new instances (0 = unlimited)
    pub default_cpu_limit: u8,
    /// Default RAM limit in MB for new instances (0 = unlimited)
    pub default_ram_limit: u64,
    /// Default network limit in KB/s (0 = unlimited)
    pub default_network_limit: u64,
    /// Default process priority
    pub default_priority: i8,

    // Automation
    /// Delay between staggered instance launches (ms)
    pub staggered_launch_delay_ms: u32,
    /// Auto-restart crashed instances by default
    pub default_auto_restart: bool,
    /// Restart delay in seconds
    pub default_restart_delay_secs: u32,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u32,

    // Notifications
    /// Notification level
    pub notification_level: NotificationLevel,
    /// Play sound on notifications
    pub notification_sound: bool,

    // Advanced
    /// Custom data directory
    pub data_directory: Option<PathBuf>,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Maximum instances allowed (0 = unlimited)
    pub max_instances: u32,
    /// Resource monitor update interval in ms
    pub monitor_interval_ms: u32,
    /// Keep instance history for N days (0 = forever)
    pub history_retention_days: u32,

    // UI State (not user-configurable, just persisted)
    /// Sidebar collapsed state
    pub sidebar_collapsed: bool,
    /// Last selected application path
    pub last_app_path: Option<PathBuf>,
    /// Window position
    pub window_position: Option<(i32, i32)>,
    /// Window size
    pub window_size: Option<(u32, u32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // General
            start_with_system: false,
            minimize_to_tray: true,
            auto_restore_sessions: false,
            theme: Theme::Dark,
            view_mode: ViewMode::Grid,
            show_system_resources: true,

            // Default Resource Limits
            default_cpu_limit: 0,
            default_ram_limit: 0,
            default_network_limit: 0,
            default_priority: 0,

            // Automation
            staggered_launch_delay_ms: 1000,
            default_auto_restart: false,
            default_restart_delay_secs: 5,
            enable_health_checks: false,
            health_check_interval_secs: 30,

            // Notifications
            notification_level: NotificationLevel::Important,
            notification_sound: true,

            // Advanced
            data_directory: None,
            debug_logging: false,
            max_instances: 0,
            monitor_interval_ms: 1000,
            history_retention_days: 30,

            // UI State
            sidebar_collapsed: false,
            last_app_path: None,
            window_position: None,
            window_size: None,
        }
    }
}

impl Settings {
    /// Get the data directory, using default if not set
    pub fn get_data_directory(&self) -> PathBuf {
        self.data_directory.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("MultiInstance")
        })
    }

    /// Get the instances data directory
    pub fn get_instances_directory(&self) -> PathBuf {
        self.get_data_directory().join("instances")
    }

    /// Get the profiles directory
    pub fn get_profiles_directory(&self) -> PathBuf {
        self.get_data_directory().join("profiles")
    }

    /// Get the logs directory
    pub fn get_logs_directory(&self) -> PathBuf {
        self.get_data_directory().join("logs")
    }

    /// Validate settings and fix any invalid values
    pub fn validate(&mut self) {
        self.default_cpu_limit = self.default_cpu_limit.min(100);
        self.default_priority = self.default_priority.clamp(-20, 19);
        self.monitor_interval_ms = self.monitor_interval_ms.max(100);
        self.health_check_interval_secs = self.health_check_interval_secs.max(5);
    }

    /// Create default resource limits from settings
    pub fn default_resource_limits(&self) -> super::ResourceLimits {
        super::ResourceLimits {
            cpu_percent: self.default_cpu_limit,
            memory_mb: self.default_ram_limit,
            network_kbps: self.default_network_limit,
            priority: self.default_priority,
            ..Default::default()
        }
    }
}
