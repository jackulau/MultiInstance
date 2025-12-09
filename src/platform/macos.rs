//! macOS-specific process management and resource control

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, warn};

/// Terminate a process gracefully (SIGTERM)
pub fn terminate_process(pid: u32) -> Result<()> {
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGTERM);
        if result == 0 {
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to terminate process: {}",
                std::io::Error::last_os_error()
            )
        }
    }
}

/// Force kill a process (SIGKILL)
pub fn kill_process(pid: u32) -> Result<()> {
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGKILL);
        if result == 0 {
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to kill process: {}",
                std::io::Error::last_os_error()
            )
        }
    }
}

/// Suspend a process (SIGSTOP)
pub fn suspend_process(pid: u32) -> Result<()> {
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGSTOP);
        if result == 0 {
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to suspend process: {}",
                std::io::Error::last_os_error()
            )
        }
    }
}

/// Resume a suspended process (SIGCONT)
pub fn resume_process(pid: u32) -> Result<()> {
    unsafe {
        let result = libc::kill(pid as i32, libc::SIGCONT);
        if result == 0 {
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to resume process: {}",
                std::io::Error::last_os_error()
            )
        }
    }
}

/// Check if a process is running
pub fn is_process_running(pid: u32) -> bool {
    unsafe {
        // kill with signal 0 checks if process exists without sending a signal
        libc::kill(pid as i32, 0) == 0
    }
}

/// Set CPU affinity for a process
/// Note: macOS doesn't have direct CPU affinity APIs like Linux/Windows
/// We use thread affinity tags as a hint to the scheduler
pub fn set_cpu_affinity(pid: u32, cores: &[usize]) -> Result<()> {
    if cores.is_empty() {
        return Ok(());
    }

    // macOS doesn't support process-level CPU affinity directly
    // The best we can do is set thread affinity tags which are hints
    // For now, we'll log a warning and continue
    warn!(
        "CPU affinity is not fully supported on macOS. Cores {:?} requested for PID {}",
        cores, pid
    );

    // Could potentially use thread_policy_set with THREAD_AFFINITY_POLICY
    // but it requires the thread port, not just the process ID

    Ok(())
}

/// Set process priority (nice value)
pub fn set_process_priority(pid: u32, priority: i8) -> Result<()> {
    // Convert our -20 to 19 range to nice value
    let nice_value = priority as i32;

    unsafe {
        let result = libc::setpriority(libc::PRIO_PROCESS, pid as u32, nice_value);
        if result == 0 {
            Ok(())
        } else {
            // setpriority can legitimately return -1 for nice value -1
            // Check errno to be sure
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(0) {
                Ok(())
            } else {
                anyhow::bail!("Failed to set priority: {}", err)
            }
        }
    }
}

/// Set resource limits for the current process (before exec)
/// This should be called from a child process before exec
pub fn set_resource_limits(memory_mb: u64, cpu_percent: u8) -> Result<()> {
    unsafe {
        // Set memory limit (RLIMIT_AS - address space)
        if memory_mb > 0 {
            let limit = libc::rlimit {
                rlim_cur: memory_mb * 1024 * 1024,
                rlim_max: memory_mb * 1024 * 1024,
            };
            let result = libc::setrlimit(libc::RLIMIT_AS, &limit);
            if result != 0 {
                warn!(
                    "Failed to set memory limit: {}",
                    std::io::Error::last_os_error()
                );
            }
        }

        // Set CPU time limit (RLIMIT_CPU) - this is cumulative time, not percentage
        // For percentage, we'd need a more sophisticated approach
        if cpu_percent > 0 && cpu_percent < 100 {
            debug!("CPU percentage limiting not directly supported, would need cgroups or similar");
        }
    }

    Ok(())
}

/// Get file locks held by a process using lsof
pub fn get_process_locks(pid: u32) -> Result<Vec<PathBuf>> {
    let output = Command::new("lsof")
        .args(["-p", &pid.to_string(), "-Fn"])
        .output()
        .context("Failed to run lsof")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut locks = Vec::new();

    for line in stdout.lines() {
        if line.starts_with('n') && line.len() > 1 {
            let path = &line[1..];
            if !path.starts_with("/dev") && !path.starts_with("pipe") {
                locks.push(PathBuf::from(path));
            }
        }
    }

    Ok(locks)
}

/// Check if an application bundle has a single-instance lock file
pub fn find_app_lock_file(app_path: &std::path::Path) -> Option<PathBuf> {
    // Common locations for app lock files on macOS
    let app_name = app_path.file_stem()?.to_str()?;

    let potential_locations = vec![
        dirs::cache_dir()?.join(format!("{}.lock", app_name)),
        dirs::data_local_dir()?.join(format!("{}/{}.lock", app_name, app_name)),
        PathBuf::from("/tmp").join(format!("{}.lock", app_name)),
        PathBuf::from("/tmp").join(format!("{}-lock", app_name)),
    ];

    for path in potential_locations {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Remove a lock file if it exists (to allow multiple instances)
pub fn remove_lock_file(lock_path: &std::path::Path) -> Result<()> {
    if lock_path.exists() {
        std::fs::remove_file(lock_path)
            .context(format!("Failed to remove lock file: {:?}", lock_path))?;
        info!("Removed lock file: {:?}", lock_path);
    }
    Ok(())
}

/// Launch an app bundle (.app) with environment modifications for isolation
pub fn launch_app_isolated(
    app_path: &std::path::Path,
    data_dir: &std::path::Path,
    args: &[String],
) -> Result<u32> {
    // For .app bundles, we use the 'open' command with --new-instance
    // Combined with environment variable modifications for isolation

    let executable = if app_path.extension().map(|e| e == "app").unwrap_or(false) {
        // It's an app bundle, find the actual executable
        let contents = app_path.join("Contents").join("MacOS");
        if contents.exists() {
            // Find the executable in the MacOS folder
            std::fs::read_dir(&contents)?
                .filter_map(|e| e.ok())
                .find(|e| {
                    e.file_type().map(|t| t.is_file()).unwrap_or(false)
                        && e.path().extension().is_none()
                })
                .map(|e| e.path())
                .unwrap_or_else(|| app_path.to_path_buf())
        } else {
            app_path.to_path_buf()
        }
    } else {
        app_path.to_path_buf()
    };

    let mut cmd = Command::new(&executable);

    // Set isolated environment
    cmd.env("HOME", data_dir);
    cmd.env("XDG_DATA_HOME", data_dir.join("Library"));
    cmd.env(
        "XDG_CONFIG_HOME",
        data_dir.join("Library").join("Preferences"),
    );
    cmd.env("XDG_CACHE_HOME", data_dir.join("Library").join("Caches"));
    cmd.env(
        "TMPDIR",
        data_dir.join("Library").join("Caches").join("tmp"),
    );

    // Add any custom arguments
    cmd.args(args);

    // Spawn the process
    let child = cmd.spawn().context("Failed to launch application")?;
    Ok(child.id())
}

/// Get the Info.plist path from an app bundle
pub fn get_info_plist_path(app_path: &std::path::Path) -> Option<PathBuf> {
    let plist_path = app_path.join("Contents").join("Info.plist");
    if plist_path.exists() {
        Some(plist_path)
    } else {
        None
    }
}

/// Read bundle identifier from Info.plist
pub fn get_bundle_identifier(app_path: &std::path::Path) -> Option<String> {
    // Use /usr/libexec/PlistBuddy to read the bundle identifier
    let plist_path = get_info_plist_path(app_path)?;

    let output = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleIdentifier", plist_path.to_str()?])
        .output()
        .ok()?;

    if output.status.success() {
        let identifier = String::from_utf8_lossy(&output.stdout);
        Some(identifier.trim().to_string())
    } else {
        None
    }
}

/// Get running instances of an application by bundle identifier
pub fn get_running_instances_by_bundle(bundle_id: &str) -> Result<Vec<u32>> {
    let output = Command::new("pgrep")
        .args(["-f", bundle_id])
        .output()
        .context("Failed to run pgrep")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(pids)
}

/// Use launchctl to set up a launch agent for auto-start
pub fn setup_launch_agent(app_name: &str, executable_path: &str) -> Result<PathBuf> {
    let launch_agents_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join("Library")
        .join("LaunchAgents");

    std::fs::create_dir_all(&launch_agents_dir)?;

    let plist_name = format!("com.multiinstance.{}.plist", app_name.to_lowercase());
    let plist_path = launch_agents_dir.join(&plist_name);

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.multiinstance.{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>"#,
        app_name.to_lowercase(),
        executable_path
    );

    std::fs::write(&plist_path, plist_content)?;

    // Load the launch agent
    Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .output()
        .context("Failed to load launch agent")?;

    Ok(plist_path)
}

/// Remove a launch agent
pub fn remove_launch_agent(app_name: &str) -> Result<()> {
    let plist_name = format!("com.multiinstance.{}.plist", app_name.to_lowercase());
    let plist_path = dirs::home_dir()
        .context("Failed to get home directory")?
        .join("Library")
        .join("LaunchAgents")
        .join(&plist_name);

    if plist_path.exists() {
        // Unload the launch agent
        Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap()])
            .output()
            .ok();

        // Remove the plist file
        std::fs::remove_file(&plist_path)?;
    }

    Ok(())
}
