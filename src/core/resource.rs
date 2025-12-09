//! Resource management - Limits and usage tracking

use serde::{Deserialize, Serialize};

/// Resource limits that can be applied to an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage (0-100, 0 = unlimited)
    #[serde(default)]
    pub cpu_percent: u8,
    /// CPU affinity - specific cores to run on (empty = all cores)
    #[serde(default)]
    pub cpu_affinity: Vec<usize>,
    /// Maximum memory in MB (0 = unlimited)
    #[serde(default)]
    pub memory_mb: u64,
    /// Maximum network bandwidth in KB/s (0 = unlimited)
    #[serde(default)]
    pub network_kbps: u64,
    /// Process priority (-20 to 19 on Unix, -2 to 2 on Windows)
    #[serde(default)]
    pub priority: i8,
    /// Maximum GPU memory in MB (0 = unlimited)
    #[serde(default)]
    pub gpu_memory_mb: u64,
}

impl ResourceLimits {
    /// Create limits with CPU percentage cap
    pub fn with_cpu_limit(mut self, percent: u8) -> Self {
        self.cpu_percent = percent.min(100);
        self
    }

    /// Create limits with memory cap
    pub fn with_memory_limit(mut self, mb: u64) -> Self {
        self.memory_mb = mb;
        self
    }

    /// Create limits with network bandwidth cap
    pub fn with_network_limit(mut self, kbps: u64) -> Self {
        self.network_kbps = kbps;
        self
    }

    /// Create limits with process priority
    pub fn with_priority(mut self, priority: i8) -> Self {
        self.priority = priority.clamp(-20, 19);
        self
    }

    /// Set CPU affinity to specific cores
    pub fn with_cpu_affinity(mut self, cores: Vec<usize>) -> Self {
        self.cpu_affinity = cores;
        self
    }

    /// Check if any limits are set
    pub fn has_limits(&self) -> bool {
        self.cpu_percent > 0
            || !self.cpu_affinity.is_empty()
            || self.memory_mb > 0
            || self.network_kbps > 0
            || self.priority != 0
            || self.gpu_memory_mb > 0
    }

    /// Get Windows priority class value
    #[cfg(windows)]
    pub fn windows_priority_class(&self) -> u32 {
        use windows::Win32::System::Threading::*;
        match self.priority {
            p if p <= -15 => REALTIME_PRIORITY_CLASS.0,
            p if p <= -10 => HIGH_PRIORITY_CLASS.0,
            p if p <= -5 => ABOVE_NORMAL_PRIORITY_CLASS.0,
            p if p <= 5 => NORMAL_PRIORITY_CLASS.0,
            p if p <= 10 => BELOW_NORMAL_PRIORITY_CLASS.0,
            _ => IDLE_PRIORITY_CLASS.0,
        }
    }

    /// Get Unix nice value
    #[cfg(unix)]
    pub fn unix_nice_value(&self) -> i32 {
        self.priority as i32
    }
}

/// Current resource usage for an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_percent: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Virtual memory usage in bytes
    pub virtual_memory_bytes: u64,
    /// Network bytes received since start
    pub network_rx_bytes: u64,
    /// Network bytes transmitted since start
    pub network_tx_bytes: u64,
    /// Current network receive rate in bytes/sec
    pub network_rx_rate: u64,
    /// Current network transmit rate in bytes/sec
    pub network_tx_rate: u64,
    /// Disk read bytes since start
    pub disk_read_bytes: u64,
    /// Disk write bytes since start
    pub disk_write_bytes: u64,
    /// Number of open file handles
    pub open_files: u32,
    /// Number of threads
    pub thread_count: u32,
    /// GPU usage percentage (if applicable)
    pub gpu_percent: f32,
    /// GPU memory usage in bytes
    pub gpu_memory_bytes: u64,
}

impl ResourceUsage {
    /// Format memory as human-readable string
    pub fn memory_string(&self) -> String {
        format_bytes(self.memory_bytes)
    }

    /// Format network RX rate as human-readable string
    pub fn rx_rate_string(&self) -> String {
        format!("{}/s", format_bytes(self.network_rx_rate))
    }

    /// Format network TX rate as human-readable string
    pub fn tx_rate_string(&self) -> String {
        format!("{}/s", format_bytes(self.network_tx_rate))
    }

    /// Format CPU percentage
    pub fn cpu_string(&self) -> String {
        format!("{:.1}%", self.cpu_percent)
    }
}

/// System-wide resource information
#[derive(Debug, Clone, Default)]
pub struct SystemResources {
    /// Total CPU usage percentage
    pub cpu_percent: f32,
    /// Per-core CPU usage
    pub cpu_per_core: Vec<f32>,
    /// Total physical memory in bytes
    pub total_memory: u64,
    /// Used physical memory in bytes
    pub used_memory: u64,
    /// Available physical memory in bytes
    pub available_memory: u64,
    /// Total swap/page file in bytes
    pub total_swap: u64,
    /// Used swap/page file in bytes
    pub used_swap: u64,
    /// Network interfaces with usage
    pub network_interfaces: Vec<NetworkInterface>,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// CPU brand/model name
    pub cpu_name: String,
    /// System uptime in seconds
    pub uptime_secs: u64,
}

impl SystemResources {
    /// Memory usage percentage
    pub fn memory_percent(&self) -> f32 {
        if self.total_memory > 0 {
            (self.used_memory as f32 / self.total_memory as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Swap usage percentage
    pub fn swap_percent(&self) -> f32 {
        if self.total_swap > 0 {
            (self.used_swap as f32 / self.total_swap as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Format total memory
    pub fn total_memory_string(&self) -> String {
        format_bytes(self.total_memory)
    }

    /// Format used memory
    pub fn used_memory_string(&self) -> String {
        format_bytes(self.used_memory)
    }

    /// Format available memory
    pub fn available_memory_string(&self) -> String {
        format_bytes(self.available_memory)
    }
}

/// Network interface information
#[derive(Debug, Clone, Default)]
pub struct NetworkInterface {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
