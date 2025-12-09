//! Platform-specific implementations for Windows and macOS

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

use anyhow::Result;

/// Terminate a process gracefully
pub fn terminate_process(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        windows::terminate_process(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::terminate_process(pid)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = pid;
        anyhow::bail!("Unsupported platform")
    }
}

/// Force kill a process
pub fn kill_process(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        windows::kill_process(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::kill_process(pid)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = pid;
        anyhow::bail!("Unsupported platform")
    }
}

/// Suspend a process
pub fn suspend_process(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        windows::suspend_process(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::suspend_process(pid)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = pid;
        anyhow::bail!("Unsupported platform")
    }
}

/// Resume a suspended process
pub fn resume_process(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        windows::resume_process(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::resume_process(pid)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = pid;
        anyhow::bail!("Unsupported platform")
    }
}

/// Check if a process is running
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(windows)]
    {
        windows::is_process_running(pid)
    }
    #[cfg(target_os = "macos")]
    {
        macos::is_process_running(pid)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = pid;
        false
    }
}

/// Set CPU affinity for a process
pub fn set_cpu_affinity(pid: u32, cores: &[usize]) -> Result<()> {
    #[cfg(windows)]
    {
        windows::set_cpu_affinity(pid, cores)
    }
    #[cfg(target_os = "macos")]
    {
        macos::set_cpu_affinity(pid, cores)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (pid, cores);
        anyhow::bail!("Unsupported platform")
    }
}

/// Set process priority
pub fn set_process_priority(pid: u32, priority: i8) -> Result<()> {
    #[cfg(windows)]
    {
        windows::set_process_priority(pid, priority)
    }
    #[cfg(target_os = "macos")]
    {
        macos::set_process_priority(pid, priority)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (pid, priority);
        anyhow::bail!("Unsupported platform")
    }
}

/// Release/manipulate mutex to allow multiple instances (Windows-specific)
#[cfg(windows)]
pub fn release_app_mutex(process_name: &str) -> Result<()> {
    windows::release_app_mutex(process_name)
}

/// Get list of file locks held by a process (macOS)
#[cfg(target_os = "macos")]
pub fn get_process_locks(pid: u32) -> Result<Vec<std::path::PathBuf>> {
    macos::get_process_locks(pid)
}
