//! Resource monitoring - System and process resource tracking

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use super::resource::{NetworkInterface, ResourceUsage, SystemResources};
use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System,
};
use tracing::trace;

/// Resource monitor that tracks system and per-process resource usage
pub struct ResourceMonitor {
    /// System information
    system: System,
    /// Network information
    networks: Networks,
    /// Last network readings for rate calculation
    last_network: HashMap<String, (u64, u64, Instant)>,
    /// Per-process network tracking (estimated from system delta)
    process_network: HashMap<u32, (u64, u64)>,
    /// Last update time
    last_update: Instant,
    /// Update interval
    update_interval: Duration,
}

impl ResourceMonitor {
    pub fn new(update_interval_ms: u32) -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            last_network: HashMap::new(),
            process_network: HashMap::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(update_interval_ms as u64),
        }
    }

    /// Refresh all system information
    pub fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }

        self.system
            .refresh_cpu_specifics(CpuRefreshKind::everything());
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::everything());
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        self.networks.refresh();

        self.last_update = now;
        trace!("Resource monitor refreshed");
    }

    /// Get system-wide resource information
    pub fn get_system_resources(&self) -> SystemResources {
        let cpus = self.system.cpus();

        let mut network_interfaces = Vec::new();
        for (name, data) in self.networks.iter() {
            let last = self.last_network.get(name);
            let (rx_rate, tx_rate) = if let Some((last_rx, last_tx, last_time)) = last {
                let elapsed = last_time.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rx_rate = ((data.total_received() - last_rx) as f64 / elapsed) as u64;
                    let tx_rate = ((data.total_transmitted() - last_tx) as f64 / elapsed) as u64;
                    (rx_rate, tx_rate)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            network_interfaces.push(NetworkInterface {
                name: name.clone(),
                rx_bytes: data.total_received(),
                tx_bytes: data.total_transmitted(),
                rx_rate,
                tx_rate,
            });
        }

        SystemResources {
            cpu_percent: self.system.global_cpu_usage(),
            cpu_per_core: cpus.iter().map(|cpu| cpu.cpu_usage()).collect(),
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            available_memory: self.system.available_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
            network_interfaces,
            cpu_cores: cpus.len(),
            cpu_name: cpus
                .first()
                .map(|c| c.brand().to_string())
                .unwrap_or_default(),
            uptime_secs: System::uptime(),
        }
    }

    /// Get resource usage for a specific process
    pub fn get_process_usage(&self, pid: u32) -> Option<ResourceUsage> {
        let process = self.system.process(Pid::from_u32(pid))?;

        // Get network usage estimate for this process
        let (network_rx, network_tx) = self.process_network.get(&pid).copied().unwrap_or((0, 0));

        Some(ResourceUsage {
            cpu_percent: process.cpu_usage(),
            memory_bytes: process.memory(),
            virtual_memory_bytes: process.virtual_memory(),
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
            network_rx_rate: 0, // Would need per-process network tracking
            network_tx_rate: 0,
            disk_read_bytes: process.disk_usage().read_bytes,
            disk_write_bytes: process.disk_usage().written_bytes,
            open_files: 0,    // Not available in sysinfo
            thread_count: 0,  // Would need platform-specific code
            gpu_percent: 0.0, // Would need GPU-specific libraries
            gpu_memory_bytes: 0,
        })
    }

    /// Check if a process is running
    pub fn is_process_running(&self, pid: u32) -> bool {
        self.system.process(Pid::from_u32(pid)).is_some()
    }

    /// Get all running process IDs
    pub fn get_running_pids(&self) -> Vec<u32> {
        self.system
            .processes()
            .keys()
            .map(|pid| pid.as_u32())
            .collect()
    }

    /// Refresh and update network rate calculations
    pub fn update_network_rates(&mut self) {
        let now = Instant::now();
        for (name, data) in self.networks.iter() {
            self.last_network.insert(
                name.clone(),
                (data.total_received(), data.total_transmitted(), now),
            );
        }
    }

    /// Get process by name
    pub fn find_processes_by_name(&self, name: &str) -> Vec<u32> {
        self.system
            .processes()
            .iter()
            .filter(|(_, proc)| {
                proc.name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&name.to_lowercase())
            })
            .map(|(pid, _)| pid.as_u32())
            .collect()
    }

    /// Get the command line of a process
    pub fn get_process_command(&self, pid: u32) -> Option<Vec<String>> {
        self.system.process(Pid::from_u32(pid)).map(|p| {
            p.cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect()
        })
    }

    /// Get the executable path of a process
    pub fn get_process_exe(&self, pid: u32) -> Option<std::path::PathBuf> {
        self.system
            .process(Pid::from_u32(pid))
            .and_then(|p| p.exe().map(|e| e.to_path_buf()))
    }

    /// Get system uptime
    pub fn get_uptime(&self) -> Duration {
        Duration::from_secs(System::uptime())
    }

    /// Get the number of CPUs/cores
    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    /// Get total system memory in bytes
    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    /// Get available system memory in bytes
    pub fn available_memory(&self) -> u64 {
        self.system.available_memory()
    }
}

/// Thread-safe wrapper for ResourceMonitor
pub struct SharedResourceMonitor {
    inner: Arc<RwLock<ResourceMonitor>>,
}

impl SharedResourceMonitor {
    pub fn new(update_interval_ms: u32) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ResourceMonitor::new(update_interval_ms))),
        }
    }

    pub fn refresh(&self) {
        if let Ok(mut monitor) = self.inner.write() {
            monitor.refresh();
            monitor.update_network_rates();
        }
    }

    pub fn get_system_resources(&self) -> SystemResources {
        self.inner
            .read()
            .map(|m| m.get_system_resources())
            .unwrap_or_default()
    }

    pub fn get_process_usage(&self, pid: u32) -> Option<ResourceUsage> {
        self.inner.read().ok()?.get_process_usage(pid)
    }

    pub fn is_process_running(&self, pid: u32) -> bool {
        self.inner
            .read()
            .map(|m| m.is_process_running(pid))
            .unwrap_or(false)
    }

    pub fn clone_inner(&self) -> Arc<RwLock<ResourceMonitor>> {
        Arc::clone(&self.inner)
    }
}

impl Clone for SharedResourceMonitor {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
