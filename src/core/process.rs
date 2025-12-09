//! Process management - Spawning and controlling processes

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use tracing::{error, info, warn};

use super::instance::{Instance, InstanceConfig, InstanceId, InstanceStatus};
use super::resource::ResourceLimits;
use crate::platform;

/// Manages spawning and controlling processes
pub struct ProcessManager {
    /// Running child processes
    children: HashMap<InstanceId, Child>,
    /// Instance data directory base path
    instance_data_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(instance_data_dir: PathBuf) -> Self {
        // Ensure the data directory exists
        if let Err(e) = std::fs::create_dir_all(&instance_data_dir) {
            error!("Failed to create instance data directory: {}", e);
        }

        Self {
            children: HashMap::new(),
            instance_data_dir,
        }
    }

    /// Spawn a new instance
    pub fn spawn(&mut self, instance: &mut Instance) -> Result<()> {
        let config = &instance.config;
        info!(
            "Spawning instance '{}' from {:?}",
            config.name, config.executable_path
        );

        // Validate executable exists
        if !config.executable_path.exists() {
            anyhow::bail!("Executable not found: {}", config.executable_path.display());
        }

        // Create isolated data directory if needed
        let data_dir = self.get_or_create_instance_data_dir(instance.id, config)?;

        // Build the command
        let mut cmd = Command::new(&config.executable_path);

        // Set working directory
        if let Some(ref work_dir) = config.working_directory {
            cmd.current_dir(work_dir);
        } else if let Some(parent) = config.executable_path.parent() {
            cmd.current_dir(parent);
        }

        // Add arguments
        cmd.args(&config.arguments);

        // Set up environment for isolation (only if enabled)
        // Note: Disable this for games with anti-cheat
        if config.bypass_single_instance && config.use_environment_isolation {
            self.setup_isolation_env(&mut cmd, &data_dir, config);
        }

        // Add custom environment variables
        for (key, value) in &config.environment {
            cmd.env(key, value);
        }

        // Detach from our process group
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x00000008); // DETACHED_PROCESS
        }

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    libc::setsid();
                    Ok(())
                });
            }
        }

        // Spawn the process
        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn process")?;

        let pid = child.id();
        info!("Spawned process with PID {}", pid);

        // Apply resource limits
        if config.resource_limits.has_limits() {
            if let Err(e) = self.apply_resource_limits(pid, &config.resource_limits) {
                warn!("Failed to apply resource limits: {}", e);
            }
        }

        // Close singleton mutex/event handles to allow multiple instances
        #[cfg(windows)]
        if config.bypass_single_instance {
            info!("Closing singleton handles for multi-instance support");
            let pid_copy = pid;
            std::thread::spawn(move || {
                if let Err(e) = platform::windows::close_singleton_handles(pid_copy) {
                    warn!("Failed to close singleton handles: {}", e);
                }
            });
        }

        // Hide from taskbar if configured
        #[cfg(windows)]
        if config.hide_from_taskbar {
            info!("Hiding instance from taskbar");
            let pid_copy = pid;
            std::thread::spawn(move || {
                if let Err(e) = platform::windows::hide_process_from_taskbar(pid_copy) {
                    warn!("Failed to hide from taskbar: {}", e);
                }
            });
        }

        // Update instance state
        instance.mark_starting(pid);

        // Store child handle
        self.children.insert(instance.id, child);

        Ok(())
    }

    /// Stop an instance
    pub fn stop(&mut self, instance: &mut Instance) -> Result<()> {
        info!("Stopping instance '{}'", instance.config.name);

        if let Some(pid) = instance.pid {
            // Try graceful termination first
            if let Err(e) = platform::terminate_process(pid) {
                warn!("Graceful termination failed: {}, forcing kill", e);
                platform::kill_process(pid)?;
            }

            // Clean up job handle on Windows
            #[cfg(windows)]
            {
                platform::windows::cleanup_job_handle(pid);
            }
        }

        // Remove child handle
        self.children.remove(&instance.id);

        // Update instance state
        instance.mark_stopped();

        Ok(())
    }

    /// Force kill an instance
    pub fn kill(&mut self, instance: &mut Instance) -> Result<()> {
        info!("Killing instance '{}'", instance.config.name);

        if let Some(pid) = instance.pid {
            platform::kill_process(pid)?;

            // Clean up job handle on Windows
            #[cfg(windows)]
            {
                platform::windows::cleanup_job_handle(pid);
            }
        }

        self.children.remove(&instance.id);
        instance.mark_stopped();

        Ok(())
    }

    /// Pause/suspend an instance (if supported)
    pub fn pause(&mut self, instance: &mut Instance) -> Result<()> {
        if let Some(pid) = instance.pid {
            platform::suspend_process(pid)?;
            instance.mark_paused();
        }
        Ok(())
    }

    /// Resume a paused instance
    pub fn resume(&mut self, instance: &mut Instance) -> Result<()> {
        if let Some(pid) = instance.pid {
            platform::resume_process(pid)?;
            instance.mark_running();
        }
        Ok(())
    }

    /// Check if a child process is still running
    pub fn check_process(&mut self, instance: &mut Instance) -> bool {
        if let Some(child) = self.children.get_mut(&instance.id) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process has exited
                    if status.success() {
                        instance.mark_stopped();
                    } else {
                        let error = format!("Process exited with status: {}", status);
                        instance.mark_crashed(Some(error));
                    }
                    false
                }
                Ok(None) => {
                    // Process is still running
                    if instance.status == InstanceStatus::Starting {
                        instance.mark_running();
                    }
                    true
                }
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    instance.mark_crashed(Some(e.to_string()));
                    false
                }
            }
        } else {
            // No child handle, check by PID
            if let Some(pid) = instance.pid {
                platform::is_process_running(pid)
            } else {
                false
            }
        }
    }

    /// Apply resource limits to a process
    fn apply_resource_limits(&self, pid: u32, limits: &ResourceLimits) -> Result<()> {
        // Apply CPU affinity
        if !limits.cpu_affinity.is_empty() {
            platform::set_cpu_affinity(pid, &limits.cpu_affinity)?;
        }

        // Apply process priority
        if limits.priority != 0 {
            platform::set_process_priority(pid, limits.priority)?;
        }

        // Apply memory limit (Windows only via Job Objects)
        #[cfg(windows)]
        if limits.memory_mb > 0 {
            platform::windows::set_memory_limit(pid, limits.memory_mb)?;
        }

        // CPU and network throttling would require more advanced techniques
        // (e.g., cgroups on Linux, Job Objects on Windows)

        Ok(())
    }

    /// Set up environment variables for instance isolation
    fn setup_isolation_env(&self, cmd: &mut Command, data_dir: &Path, _config: &InstanceConfig) {
        // Set custom home/appdata directory to isolate the instance
        #[cfg(windows)]
        {
            cmd.env("APPDATA", data_dir.join("AppData").join("Roaming"));
            cmd.env("LOCALAPPDATA", data_dir.join("AppData").join("Local"));
            cmd.env("USERPROFILE", data_dir);
        }

        #[cfg(target_os = "macos")]
        {
            cmd.env("HOME", data_dir);
            cmd.env("XDG_DATA_HOME", data_dir.join("Library"));
            cmd.env(
                "XDG_CONFIG_HOME",
                data_dir.join("Library").join("Preferences"),
            );
            cmd.env("XDG_CACHE_HOME", data_dir.join("Library").join("Caches"));
        }

        #[cfg(target_os = "linux")]
        {
            cmd.env("HOME", data_dir);
            cmd.env("XDG_DATA_HOME", data_dir.join(".local").join("share"));
            cmd.env("XDG_CONFIG_HOME", data_dir.join(".config"));
            cmd.env("XDG_CACHE_HOME", data_dir.join(".cache"));
        }
    }

    /// Get or create an isolated data directory for an instance
    fn get_or_create_instance_data_dir(
        &self,
        id: InstanceId,
        config: &InstanceConfig,
    ) -> Result<PathBuf> {
        let data_dir = if let Some(ref custom_dir) = config.data_directory {
            custom_dir.clone()
        } else {
            self.instance_data_dir.join(id.to_string())
        };

        // Create the directory structure
        std::fs::create_dir_all(&data_dir)?;

        #[cfg(windows)]
        {
            std::fs::create_dir_all(data_dir.join("AppData").join("Roaming"))?;
            std::fs::create_dir_all(data_dir.join("AppData").join("Local"))?;
        }

        #[cfg(target_os = "macos")]
        {
            std::fs::create_dir_all(data_dir.join("Library").join("Preferences"))?;
            std::fs::create_dir_all(data_dir.join("Library").join("Caches"))?;
            std::fs::create_dir_all(data_dir.join("Library").join("Application Support"))?;
        }

        Ok(data_dir)
    }

    /// Clean up an instance's data directory
    pub fn cleanup_instance_data(&self, id: InstanceId) -> Result<()> {
        let data_dir = self.instance_data_dir.join(id.to_string());
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)?;
        }
        Ok(())
    }

    /// Get the number of running processes
    pub fn running_count(&self) -> usize {
        self.children.len()
    }

    /// Check if we have a child handle for an instance
    pub fn has_child(&self, id: InstanceId) -> bool {
        self.children.contains_key(&id)
    }

    /// Remove a child handle (when process is no longer managed)
    pub fn remove_child(&mut self, id: InstanceId) {
        self.children.remove(&id);
    }
}

/// Thread-safe wrapper for ProcessManager
pub struct SharedProcessManager {
    inner: Arc<RwLock<ProcessManager>>,
}

impl SharedProcessManager {
    pub fn new(instance_data_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ProcessManager::new(instance_data_dir))),
        }
    }

    pub fn spawn(&self, instance: &mut Instance) -> Result<()> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .spawn(instance)
    }

    pub fn stop(&self, instance: &mut Instance) -> Result<()> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .stop(instance)
    }

    pub fn kill(&self, instance: &mut Instance) -> Result<()> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .kill(instance)
    }

    pub fn pause(&self, instance: &mut Instance) -> Result<()> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .pause(instance)
    }

    pub fn resume(&self, instance: &mut Instance) -> Result<()> {
        self.inner
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .resume(instance)
    }

    pub fn check_process(&self, instance: &mut Instance) -> bool {
        self.inner
            .write()
            .map(|mut m| m.check_process(instance))
            .unwrap_or(false)
    }

    pub fn running_count(&self) -> usize {
        self.inner.read().map(|m| m.running_count()).unwrap_or(0)
    }

    pub fn clone_inner(&self) -> Arc<RwLock<ProcessManager>> {
        Arc::clone(&self.inner)
    }
}

impl Clone for SharedProcessManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
