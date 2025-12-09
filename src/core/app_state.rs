//! Application state - Central state management for MultiInstance

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tracing::{error, info, warn};

use super::instance::{Instance, InstanceConfig, InstanceId, InstanceStatus};
use super::monitor::SharedResourceMonitor;
use super::process::SharedProcessManager;
use super::profile::{Profile, ProfileId};
use super::settings::Settings;
use crate::persistence::Database;

/// Central application state
pub struct AppState {
    /// All managed instances
    pub instances: Arc<RwLock<HashMap<InstanceId, Instance>>>,
    /// Saved profiles
    pub profiles: Arc<RwLock<HashMap<ProfileId, Profile>>>,
    /// Application settings
    pub settings: Arc<RwLock<Settings>>,
    /// Process manager
    pub process_manager: SharedProcessManager,
    /// Resource monitor
    pub resource_monitor: SharedResourceMonitor,
    /// Database connection
    pub database: Arc<Database>,
    /// Quick launch applications (favorites)
    pub quick_launch: Arc<RwLock<Vec<InstanceConfig>>>,
    /// Instance groups
    pub groups: Arc<RwLock<Vec<String>>>,
    /// Recently used applications
    pub recent_apps: Arc<RwLock<Vec<PathBuf>>>,
    /// Last resource update time
    last_resource_update: Arc<RwLock<Instant>>,
}

impl AppState {
    /// Create a new application state
    pub fn new(database: Database) -> Result<Self> {
        // Load settings from database
        let settings = database.load_settings()?.unwrap_or_default();
        let settings = Arc::new(RwLock::new(settings));

        // Create data directories
        let data_dir = settings
            .read()
            .map_err(|e| anyhow::anyhow!("Settings lock poisoned: {}", e))?
            .get_data_directory();
        let instances_dir = settings
            .read()
            .map_err(|e| anyhow::anyhow!("Settings lock poisoned: {}", e))?
            .get_instances_directory();
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&instances_dir)?;

        // Initialize process manager
        let process_manager = SharedProcessManager::new(instances_dir);

        // Initialize resource monitor
        let monitor_interval = settings
            .read()
            .map_err(|e| anyhow::anyhow!("Settings lock poisoned: {}", e))?
            .monitor_interval_ms;
        let resource_monitor = SharedResourceMonitor::new(monitor_interval);

        // Load profiles from database
        let profiles = database.load_all_profiles()?;
        let profiles: HashMap<ProfileId, Profile> =
            profiles.into_iter().map(|p| (p.id, p)).collect();

        // Load quick launch items
        let quick_launch = database.load_quick_launch()?;

        // Load groups
        let groups = database.load_groups()?;

        // Load recent apps
        let recent_apps = database.load_recent_apps()?;

        let database = Arc::new(database);

        Ok(Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(profiles)),
            settings,
            process_manager,
            resource_monitor,
            database,
            quick_launch: Arc::new(RwLock::new(quick_launch)),
            groups: Arc::new(RwLock::new(groups)),
            recent_apps: Arc::new(RwLock::new(recent_apps)),
            last_resource_update: Arc::new(RwLock::new(Instant::now())),
        })
    }

    /// Create a new instance and optionally start it
    pub fn create_instance(&self, config: InstanceConfig, start: bool) -> Result<InstanceId> {
        let mut instance = Instance::new(config);
        let id = instance.id;

        // Add to recent apps
        self.add_recent_app(&instance.config.executable_path);

        // Start if requested
        if start {
            self.process_manager.spawn(&mut instance)?;
        }

        // Store instance
        self.instances
            .write()
            .expect("Instances lock poisoned")
            .insert(id, instance.clone());

        // Persist to database
        self.database.save_instance(&instance)?;

        info!("Created instance {} (started: {})", id, start);
        Ok(id)
    }

    /// Start an existing instance
    pub fn start_instance(&self, id: InstanceId) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get_mut(&id).context("Instance not found")?;

        if instance.status.is_active() {
            anyhow::bail!("Instance is already running");
        }

        self.process_manager.spawn(instance)?;
        self.database.update_instance_status(id, &instance.status)?;

        Ok(())
    }

    /// Stop an instance
    pub fn stop_instance(&self, id: InstanceId) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get_mut(&id).context("Instance not found")?;

        if !instance.status.is_active() {
            return Ok(()); // Already stopped
        }

        self.process_manager.stop(instance)?;
        self.database.update_instance_status(id, &instance.status)?;

        Ok(())
    }

    /// Kill an instance forcefully
    pub fn kill_instance(&self, id: InstanceId) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get_mut(&id).context("Instance not found")?;

        self.process_manager.kill(instance)?;
        self.database.update_instance_status(id, &instance.status)?;

        Ok(())
    }

    /// Pause an instance
    pub fn pause_instance(&self, id: InstanceId) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get_mut(&id).context("Instance not found")?;

        self.process_manager.pause(instance)?;
        self.database.update_instance_status(id, &instance.status)?;

        Ok(())
    }

    /// Resume a paused instance
    pub fn resume_instance(&self, id: InstanceId) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get_mut(&id).context("Instance not found")?;

        self.process_manager.resume(instance)?;
        self.database.update_instance_status(id, &instance.status)?;

        Ok(())
    }

    /// Remove an instance (must be stopped first)
    pub fn remove_instance(&self, id: InstanceId, cleanup_data: bool) -> Result<()> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let instance = instances.get(&id).context("Instance not found")?;

        if instance.status.is_active() {
            anyhow::bail!("Cannot remove a running instance");
        }

        // Remove from database
        self.database.delete_instance(id)?;

        // Clean up data directory if requested
        if cleanup_data {
            let data_dir = self
                .settings
                .read()
                .map(|s| s.get_instances_directory())
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let instance_dir = data_dir.join(id.to_string());
            if instance_dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&instance_dir) {
                    warn!("Failed to clean up instance data: {}", e);
                }
            }
        }

        // Remove from state
        instances.remove(&id);

        info!("Removed instance {}", id);
        Ok(())
    }

    /// Restart an instance
    pub fn restart_instance(&self, id: InstanceId) -> Result<()> {
        self.stop_instance(id)?;
        std::thread::sleep(Duration::from_millis(500));
        self.start_instance(id)?;
        Ok(())
    }

    /// Start all instances in a profile
    pub fn launch_profile(&self, profile_id: ProfileId) -> Result<Vec<InstanceId>> {
        // Extract data from profile with minimal lock hold time
        let (staggered, delay, configs) = {
            let mut profiles = self
                .profiles
                .write()
                .map_err(|e| anyhow::anyhow!("Profiles lock poisoned: {}", e))?;
            let profile = profiles.get_mut(&profile_id).context("Profile not found")?;

            profile.mark_used();
            (
                profile.staggered_launch,
                Duration::from_millis(profile.launch_delay_ms as u64),
                profile.instances.clone(),
            )
        }; // Lock released here before any I/O operations

        let mut ids = Vec::new();
        for (i, config) in configs.into_iter().enumerate() {
            if staggered && i > 0 {
                std::thread::sleep(delay);
            }
            let id = self.create_instance(config, true)?;
            ids.push(id);
        }

        // Update profile in database
        if let Ok(profiles) = self.profiles.read() {
            if let Some(profile) = profiles.get(&profile_id) {
                self.database.save_profile(profile)?;
            }
        }

        Ok(ids)
    }

    /// Stop all running instances
    pub fn stop_all(&self) -> Result<()> {
        let ids: Vec<InstanceId> = self
            .instances
            .read()
            .map(|i| i.keys().copied().collect())
            .unwrap_or_default();
        for id in ids {
            if let Err(e) = self.stop_instance(id) {
                error!("Failed to stop instance {}: {}", id, e);
            }
        }
        Ok(())
    }

    /// Pause all running instances
    pub fn pause_all(&self) -> Result<()> {
        let ids: Vec<InstanceId> = self
            .instances
            .read()
            .map(|i| i.keys().copied().collect())
            .unwrap_or_default();
        for id in ids {
            if let Err(e) = self.pause_instance(id) {
                error!("Failed to pause instance {}: {}", id, e);
            }
        }
        Ok(())
    }

    /// Resume all paused instances
    pub fn resume_all(&self) -> Result<()> {
        let ids: Vec<InstanceId> = self
            .instances
            .read()
            .map(|i| i.keys().copied().collect())
            .unwrap_or_default();
        for id in ids {
            let should_resume = self
                .instances
                .read()
                .map(|instances| {
                    instances
                        .get(&id)
                        .map(|i| i.status == InstanceStatus::Paused)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if should_resume {
                if let Err(e) = self.resume_instance(id) {
                    error!("Failed to resume instance {}: {}", id, e);
                }
            }
        }
        Ok(())
    }

    /// Update resource usage for all instances
    pub fn update_resources(&self) {
        // Refresh system resources
        self.resource_monitor.refresh();

        // Update per-instance usage
        if let Ok(mut instances) = self.instances.write() {
            for instance in instances.values_mut() {
                if let Some(pid) = instance.pid {
                    // Check if process is still running
                    if !self.process_manager.check_process(instance) {
                        continue;
                    }

                    // Update resource usage
                    if let Some(usage) = self.resource_monitor.get_process_usage(pid) {
                        instance.update_resource_usage(usage);
                    }
                }
            }
        }

        if let Ok(mut last_update) = self.last_resource_update.write() {
            *last_update = Instant::now();
        }
    }

    /// Handle auto-restart for crashed instances
    pub fn handle_auto_restarts(&self) {
        let restart_candidates: Vec<InstanceId> = self
            .instances
            .read()
            .map(|instances| {
                instances
                    .iter()
                    .filter(|(_, i)| i.should_auto_restart())
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default();

        for id in restart_candidates {
            let delay = self
                .instances
                .read()
                .map(|instances| {
                    instances
                        .get(&id)
                        .map(|i| Duration::from_secs(i.config.restart_delay_secs as u64))
                        .unwrap_or(Duration::from_secs(5))
                })
                .unwrap_or(Duration::from_secs(5));

            std::thread::sleep(delay);

            if let Ok(mut instances) = self.instances.write() {
                if let Some(instance) = instances.get_mut(&id) {
                    instance.increment_restart_count();
                    info!(
                        "Auto-restarting instance {} (attempt {})",
                        id, instance.restart_count
                    );
                    if let Err(e) = self.process_manager.spawn(instance) {
                        error!("Failed to auto-restart instance {}: {}", id, e);
                    }
                }
            }
        }
    }

    /// Save current session state
    pub fn save_session(&self) -> Result<()> {
        let instances = self
            .instances
            .read()
            .map_err(|e| anyhow::anyhow!("Instances lock poisoned: {}", e))?;
        let active_instances: Vec<&Instance> = instances
            .values()
            .filter(|i| i.status.is_active())
            .collect();

        self.database.save_session(&active_instances)?;
        info!(
            "Saved session with {} active instances",
            active_instances.len()
        );
        Ok(())
    }

    /// Restore previous session
    pub fn restore_session(&self) -> Result<()> {
        let configs = self.database.load_session()?;
        info!("Restoring session with {} instances", configs.len());

        for config in configs {
            if let Err(e) = self.create_instance(config, true) {
                error!("Failed to restore instance: {}", e);
            }
        }

        Ok(())
    }

    /// Save a profile
    pub fn save_profile(&self, profile: Profile) -> Result<()> {
        self.database.save_profile(&profile)?;
        self.profiles
            .write()
            .map_err(|e| anyhow::anyhow!("Profiles lock poisoned: {}", e))?
            .insert(profile.id, profile);
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&self, id: ProfileId) -> Result<()> {
        self.database.delete_profile(id)?;
        self.profiles
            .write()
            .map_err(|e| anyhow::anyhow!("Profiles lock poisoned: {}", e))?
            .remove(&id);
        Ok(())
    }

    /// Save settings
    pub fn save_settings(&self) -> Result<()> {
        let settings = self
            .settings
            .read()
            .map_err(|e| anyhow::anyhow!("Settings lock poisoned: {}", e))?;
        self.database.save_settings(&settings)?;
        Ok(())
    }

    /// Add to quick launch
    pub fn add_quick_launch(&self, config: InstanceConfig) -> Result<()> {
        self.quick_launch
            .write()
            .map_err(|e| anyhow::anyhow!("Quick launch lock poisoned: {}", e))?
            .push(config);
        self.save_quick_launch()?;
        Ok(())
    }

    /// Remove from quick launch
    pub fn remove_quick_launch(&self, index: usize) -> Result<()> {
        {
            let mut quick_launch = self
                .quick_launch
                .write()
                .map_err(|e| anyhow::anyhow!("Quick launch lock poisoned: {}", e))?;
            if index < quick_launch.len() {
                quick_launch.remove(index);
            }
        }
        self.save_quick_launch()?;
        Ok(())
    }

    /// Save quick launch items to database
    fn save_quick_launch(&self) -> Result<()> {
        let quick_launch = self
            .quick_launch
            .read()
            .map_err(|e| anyhow::anyhow!("Quick launch lock poisoned: {}", e))?;
        self.database.save_quick_launch(&quick_launch)?;
        Ok(())
    }

    /// Add a recent app
    fn add_recent_app(&self, path: &PathBuf) {
        if let Ok(mut recent) = self.recent_apps.write() {
            // Remove if already exists
            recent.retain(|p| p != path);
            // Add to front
            recent.insert(0, path.clone());
            // Keep only last 10
            recent.truncate(10);
        }

        // Save to database (ignore errors)
        if let Ok(recent) = self.recent_apps.read() {
            let _ = self.database.save_recent_apps(&recent);
        }
    }

    /// Add a group
    pub fn add_group(&self, name: String) -> Result<()> {
        {
            let mut groups = self
                .groups
                .write()
                .map_err(|e| anyhow::anyhow!("Groups lock poisoned: {}", e))?;
            if !groups.contains(&name) {
                groups.push(name);
            }
        }
        let groups = self
            .groups
            .read()
            .map_err(|e| anyhow::anyhow!("Groups lock poisoned: {}", e))?;
        self.database.save_groups(&groups)?;
        Ok(())
    }

    /// Remove a group
    pub fn remove_group(&self, name: &str) -> Result<()> {
        {
            let mut groups = self
                .groups
                .write()
                .map_err(|e| anyhow::anyhow!("Groups lock poisoned: {}", e))?;
            groups.retain(|g| g != name);
        }
        let groups = self
            .groups
            .read()
            .map_err(|e| anyhow::anyhow!("Groups lock poisoned: {}", e))?;
        self.database.save_groups(&groups)?;
        Ok(())
    }

    /// Get count of active instances
    pub fn active_instance_count(&self) -> usize {
        self.instances
            .read()
            .map(|i| i.values().filter(|inst| inst.status.is_active()).count())
            .unwrap_or(0)
    }

    /// Get count of all instances
    pub fn total_instance_count(&self) -> usize {
        self.instances.read().map(|i| i.len()).unwrap_or(0)
    }

    /// Get count of profiles
    pub fn profile_count(&self) -> usize {
        self.profiles.read().map(|p| p.len()).unwrap_or(0)
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            instances: Arc::clone(&self.instances),
            profiles: Arc::clone(&self.profiles),
            settings: Arc::clone(&self.settings),
            process_manager: self.process_manager.clone(),
            resource_monitor: self.resource_monitor.clone(),
            database: Arc::clone(&self.database),
            quick_launch: Arc::clone(&self.quick_launch),
            groups: Arc::clone(&self.groups),
            recent_apps: Arc::clone(&self.recent_apps),
            last_resource_update: Arc::clone(&self.last_resource_update),
        }
    }
}
