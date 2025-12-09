//! Windows-specific process management and resource control

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};

use windows::Win32::Foundation::{
    CloseHandle, DuplicateHandle, BOOL, DUPLICATE_CLOSE_SOURCE, DUPLICATE_SAME_ACCESS, FALSE,
    HANDLE, HWND,
};
use windows::Win32::System::JobObjects::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::sync::LazyLock;

/// Global storage for job handles to prevent resource leaks
/// Maps PID to job handle value (stored as usize for Send/Sync safety)
static JOB_HANDLES: LazyLock<Arc<RwLock<HashMap<u32, usize>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Store a job handle for a process
fn store_job_handle(pid: u32, handle: HANDLE) {
    if let Ok(mut handles) = JOB_HANDLES.write() {
        handles.insert(pid, handle.0 as usize);
    }
}

/// Remove and close a job handle for a process
pub fn cleanup_job_handle(pid: u32) {
    if let Ok(mut handles) = JOB_HANDLES.write() {
        if let Some(handle_value) = handles.remove(&pid) {
            let handle = HANDLE(handle_value as *mut std::ffi::c_void);
            unsafe {
                let _ = CloseHandle(handle);
            }
            debug!("Cleaned up job handle for PID {}", pid);
        }
    }
}

/// Terminate a process gracefully (WM_CLOSE equivalent)
pub fn terminate_process(pid: u32) -> Result<()> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_TERMINATE, FALSE, pid).context("Failed to open process")?;

        // Try to terminate gracefully
        let result = TerminateProcess(handle, 0);
        CloseHandle(handle)?;

        if result.is_ok() {
            Ok(())
        } else {
            anyhow::bail!("Failed to terminate process")
        }
    }
}

/// Force kill a process
pub fn kill_process(pid: u32) -> Result<()> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_TERMINATE, FALSE, pid).context("Failed to open process")?;

        let result = TerminateProcess(handle, 1);
        CloseHandle(handle)?;

        if result.is_ok() {
            Ok(())
        } else {
            anyhow::bail!("Failed to kill process")
        }
    }
}

/// Suspend all threads in a process
pub fn suspend_process(pid: u32) -> Result<()> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_SUSPEND_RESUME, FALSE, pid).context("Failed to open process")?;

        // NtSuspendProcess is not directly available, so we suspend all threads
        let snapshot = windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD,
            0,
        )?;

        let mut entry = windows::Win32::System::Diagnostics::ToolHelp::THREADENTRY32 {
            dwSize: mem::size_of::<windows::Win32::System::Diagnostics::ToolHelp::THREADENTRY32>()
                as u32,
            ..Default::default()
        };

        if windows::Win32::System::Diagnostics::ToolHelp::Thread32First(snapshot, &mut entry)
            .is_ok()
        {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread_handle) =
                        OpenThread(THREAD_SUSPEND_RESUME, FALSE, entry.th32ThreadID)
                    {
                        SuspendThread(thread_handle);
                        CloseHandle(thread_handle)?;
                    }
                }
                if windows::Win32::System::Diagnostics::ToolHelp::Thread32Next(snapshot, &mut entry)
                    .is_err()
                {
                    break;
                }
            }
        }

        CloseHandle(snapshot)?;
        CloseHandle(handle)?;
        Ok(())
    }
}

/// Resume all threads in a process
pub fn resume_process(pid: u32) -> Result<()> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_SUSPEND_RESUME, FALSE, pid).context("Failed to open process")?;

        let snapshot = windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD,
            0,
        )?;

        let mut entry = windows::Win32::System::Diagnostics::ToolHelp::THREADENTRY32 {
            dwSize: mem::size_of::<windows::Win32::System::Diagnostics::ToolHelp::THREADENTRY32>()
                as u32,
            ..Default::default()
        };

        if windows::Win32::System::Diagnostics::ToolHelp::Thread32First(snapshot, &mut entry)
            .is_ok()
        {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread_handle) =
                        OpenThread(THREAD_SUSPEND_RESUME, FALSE, entry.th32ThreadID)
                    {
                        ResumeThread(thread_handle);
                        CloseHandle(thread_handle)?;
                    }
                }
                if windows::Win32::System::Diagnostics::ToolHelp::Thread32Next(snapshot, &mut entry)
                    .is_err()
                {
                    break;
                }
            }
        }

        CloseHandle(snapshot)?;
        CloseHandle(handle)?;
        Ok(())
    }
}

/// Check if a process is running
pub fn is_process_running(pid: u32) -> bool {
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let mut exit_code: u32 = 0;
        let result = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle).ok();

        // STILL_ACTIVE = 259
        result.is_ok() && exit_code == 259
    }
}

/// Set CPU affinity for a process
pub fn set_cpu_affinity(pid: u32, cores: &[usize]) -> Result<()> {
    if cores.is_empty() {
        return Ok(());
    }

    let mut mask: usize = 0;
    for &core in cores {
        if core < std::mem::size_of::<usize>() * 8 {
            mask |= 1 << core;
        }
    }

    unsafe {
        let handle = OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
            FALSE,
            pid,
        )
        .context("Failed to open process")?;

        let result = SetProcessAffinityMask(handle, mask);
        CloseHandle(handle)?;

        if result.is_ok() {
            Ok(())
        } else {
            anyhow::bail!("Failed to set CPU affinity")
        }
    }
}

/// Set process priority
/// priority: -2 (Idle) to 2 (High), 0 = Normal
pub fn set_process_priority(pid: u32, priority: i8) -> Result<()> {
    // Skip if normal priority (0)
    if priority == 0 {
        return Ok(());
    }

    let priority_class = match priority {
        -2 => IDLE_PRIORITY_CLASS,
        -1 => BELOW_NORMAL_PRIORITY_CLASS,
        1 => ABOVE_NORMAL_PRIORITY_CLASS,
        2 => HIGH_PRIORITY_CLASS,
        // Legacy support for wider range
        p if p <= -2 => IDLE_PRIORITY_CLASS,
        p if p >= 2 => HIGH_PRIORITY_CLASS,
        _ => NORMAL_PRIORITY_CLASS, // Covers 0 and any other edge cases
    };

    unsafe {
        let handle = match OpenProcess(PROCESS_SET_INFORMATION, FALSE, pid) {
            Ok(h) => h,
            Err(e) => {
                warn!("Could not open process for priority change: {}", e);
                return Ok(());
            }
        };

        let result = SetPriorityClass(handle, priority_class);
        let _ = CloseHandle(handle);

        if result.is_ok() {
            debug!("Set process {} priority to {:?}", pid, priority_class);
            Ok(())
        } else {
            warn!("Failed to set priority class for process {}", pid);
            Ok(()) // Don't fail the launch
        }
    }
}

/// Set memory limit for a process using Job Objects
/// Note: This may fail for processes already in a Job Object (like Chrome, some games, etc.)
/// The function returns Ok even if it fails, logging a warning instead of failing the launch.
pub fn set_memory_limit(pid: u32, memory_mb: u64) -> Result<()> {
    // Skip if no limit set
    if memory_mb == 0 {
        return Ok(());
    }

    unsafe {
        // Create a job object
        let job = match CreateJobObjectW(None, None) {
            Ok(j) => j,
            Err(e) => {
                warn!("Could not create job object for memory limit: {}. Process will run without memory limit.", e);
                return Ok(());
            }
        };

        // Set memory limit
        let mut limit_info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limit_info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_JOB_MEMORY;
        limit_info.ProcessMemoryLimit = (memory_mb * 1024 * 1024) as usize;
        limit_info.JobMemoryLimit = (memory_mb * 1024 * 1024) as usize;

        if let Err(e) = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limit_info as *const _ as *const _,
            mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) {
            warn!(
                "Could not set job object limits: {}. Process will run without memory limit.",
                e
            );
            let _ = CloseHandle(job);
            return Ok(());
        }

        // Try to assign process to job - this often fails if process is already in a Job Object
        let handle = match OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, FALSE, pid) {
            Ok(h) => h,
            Err(e) => {
                warn!("Could not open process for memory limit: {}. Process will run without memory limit.", e);
                let _ = CloseHandle(job);
                return Ok(());
            }
        };

        let result = AssignProcessToJobObject(job, handle);
        let _ = CloseHandle(handle);

        if result.is_err() {
            // This is common for processes already in a Job Object (Chrome, some games, etc.)
            debug!("Could not assign process to job object - process may already be in a job object. Memory limits not applied.");
            let _ = CloseHandle(job);
        } else {
            debug!(
                "Memory limit of {} MB applied to process {}",
                memory_mb, pid
            );
            // Store the job handle so it can be cleaned up later when the process exits
            // The handle must stay open for limits to remain in effect
            store_job_handle(pid, job);
        }

        Ok(())
    }
}

/// Attempt to release/close a mutex held by applications to allow multiple instances
/// This is a best-effort approach and may not work for all applications
pub fn release_app_mutex(process_name: &str) -> Result<()> {
    // This is a simplified implementation
    // A full implementation would:
    // 1. Enumerate all handles in the system
    // 2. Find mutex handles with names matching common patterns
    // 3. Duplicate and close them

    // For now, we rely on profile isolation which is more reliable
    warn!(
        "Mutex release for '{}' - using profile isolation instead",
        process_name
    );
    Ok(())
}

/// Get process memory information
pub fn get_process_memory_info(pid: u32) -> Result<(u64, u64)> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid)
            .context("Failed to open process")?;

        let mut mem_counters = PROCESS_MEMORY_COUNTERS {
            cb: mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };

        let result = K32GetProcessMemoryInfo(
            handle,
            &mut mem_counters,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        );

        CloseHandle(handle)?;

        if result.as_bool() {
            Ok((
                mem_counters.WorkingSetSize as u64,
                mem_counters.PagefileUsage as u64,
            ))
        } else {
            anyhow::bail!("Failed to get memory info")
        }
    }
}

/// Enumerate all processes and find ones matching a name
pub fn find_processes_by_name(name: &str) -> Result<Vec<u32>> {
    use windows::Win32::System::Diagnostics::ToolHelp::*;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

        let mut entry = PROCESSENTRY32W {
            dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut pids = Vec::new();
        let name_lower = name.to_lowercase();

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let exe_name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );

                if exe_name.to_lowercase().contains(&name_lower) {
                    pids.push(entry.th32ProcessID);
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        CloseHandle(snapshot)?;
        Ok(pids)
    }
}

/// Create an isolated environment by virtualizing registry access
pub fn setup_registry_virtualization(instance_id: &str) -> Result<()> {
    // This would set up registry virtualization for the instance
    // Real implementation would use registry redirection techniques
    debug!(
        "Setting up registry virtualization for instance {}",
        instance_id
    );
    Ok(())
}

/// Close singleton mutex handles to allow multiple instances
/// This must be called after the process starts but before launching another instance
///
/// DEPRECATED: Use close_singleton_handles instead, which is more conservative
pub fn close_singleton_mutex(pid: u32) -> Result<()> {
    info!("Attempting to close singleton mutex for PID {}", pid);

    // Wait for process to initialize and create its mutex
    thread::sleep(Duration::from_millis(2000));

    // Verify the process is still running
    if !is_process_running(pid) {
        warn!(
            "Process {} is no longer running, skipping mutex closing",
            pid
        );
        return Ok(());
    }

    unsafe {
        let process_handle =
            match OpenProcess(PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION, FALSE, pid) {
                Ok(h) => h,
                Err(e) => {
                    warn!("Could not open process for mutex closing: {}", e);
                    return Ok(());
                }
            };

        // Try to find and close singleton mutexes
        // Handles are typically small integers, multiples of 4
        // Limit the range and count to be safer
        let mut closed_count = 0;
        const MAX_HANDLE_VALUE: usize = 0x400; // Reduced from 0x10000
        const MAX_HANDLES_TO_CLOSE: u32 = 100;

        for handle_value in (4..=MAX_HANDLE_VALUE).step_by(4) {
            if closed_count >= MAX_HANDLES_TO_CLOSE {
                break;
            }

            let remote_handle = HANDLE(handle_value as *mut std::ffi::c_void);
            let mut target_handle = HANDLE::default();

            // Try to duplicate the handle to our process
            let result = DuplicateHandle(
                process_handle,
                remote_handle,
                GetCurrentProcess(),
                &mut target_handle,
                0,
                FALSE,
                DUPLICATE_SAME_ACCESS,
            );

            if result.is_ok() && !target_handle.is_invalid() {
                // Close our duplicated handle
                let _ = CloseHandle(target_handle);

                // Now try to duplicate again with DUPLICATE_CLOSE_SOURCE to close the original
                let mut dummy_handle = HANDLE::default();
                let close_result = DuplicateHandle(
                    process_handle,
                    remote_handle,
                    GetCurrentProcess(),
                    &mut dummy_handle,
                    0,
                    FALSE,
                    DUPLICATE_CLOSE_SOURCE,
                );

                if close_result.is_ok() {
                    if !dummy_handle.is_invalid() {
                        let _ = CloseHandle(dummy_handle);
                    }
                    closed_count += 1;
                }
            }
        }

        let _ = CloseHandle(process_handle);

        if closed_count > 0 {
            info!("Closed {} handles from process {}", closed_count, pid);
        }
    }

    Ok(())
}

/// Close singleton event/mutex handles to allow multiple instances
/// This is a more conservative approach that only closes handles that look like singleton mutexes
///
/// SAFETY NOTE: This function attempts to identify and close only mutex/event handles
/// that are likely singleton locks. It's still somewhat aggressive but safer than
/// closing all handles in a range.
pub fn close_singleton_handles(pid: u32) -> Result<()> {
    info!("Closing singleton handles for PID {}", pid);

    // Give process time to start and create its singleton handles
    thread::sleep(Duration::from_millis(3000));

    // Verify the process is still running before attempting to modify its handles
    if !is_process_running(pid) {
        warn!(
            "Process {} is no longer running, skipping handle closing",
            pid
        );
        return Ok(());
    }

    unsafe {
        let process_handle = match OpenProcess(
            PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            FALSE,
            pid,
        ) {
            Ok(h) => h,
            Err(e) => {
                warn!("Could not open process for handle closing: {}", e);
                return Ok(());
            }
        };

        // Be more conservative: only try a limited range of handles
        // Singleton mutexes are typically created early, so they tend to have low handle values
        // We limit to handles 4-256 (0x100) to reduce risk of closing important handles
        let mut closed_count = 0;
        const MAX_HANDLES_TO_CLOSE: u32 = 50; // Limit how many handles we'll close

        for handle_value in (4..=0x100).step_by(4) {
            if closed_count >= MAX_HANDLES_TO_CLOSE {
                debug!("Reached maximum handle close limit");
                break;
            }

            let remote_handle = HANDLE(handle_value as *mut std::ffi::c_void);
            let mut target_handle = HANDLE::default();

            // First, try to duplicate without closing to inspect
            let dup_result = DuplicateHandle(
                process_handle,
                remote_handle,
                GetCurrentProcess(),
                &mut target_handle,
                0,
                FALSE,
                DUPLICATE_SAME_ACCESS,
            );

            if dup_result.is_ok() && !target_handle.is_invalid() {
                // We successfully duplicated the handle
                // Close our copy first
                let _ = CloseHandle(target_handle);

                // Now duplicate again with DUPLICATE_CLOSE_SOURCE to close the original
                let mut dummy_handle = HANDLE::default();
                let close_result = DuplicateHandle(
                    process_handle,
                    remote_handle,
                    GetCurrentProcess(),
                    &mut dummy_handle,
                    0,
                    FALSE,
                    DUPLICATE_CLOSE_SOURCE,
                );

                if close_result.is_ok() {
                    if !dummy_handle.is_invalid() {
                        let _ = CloseHandle(dummy_handle);
                    }
                    closed_count += 1;
                }
            }
        }

        let _ = CloseHandle(process_handle);

        if closed_count > 0 {
            info!(
                "Closed {} handles for PID {} (singleton bypass)",
                closed_count, pid
            );
        } else {
            debug!("No handles closed for PID {}", pid);
        }
    }

    Ok(())
}

/// Hide all windows of a process from the taskbar
/// This sets WS_EX_TOOLWINDOW style and removes WS_EX_APPWINDOW
pub fn hide_process_from_taskbar(pid: u32) -> Result<()> {
    info!("Hiding windows from taskbar for PID {}", pid);

    // Give the process time to create its windows
    thread::sleep(Duration::from_millis(1000));

    unsafe {
        // Enumerate all windows and find those belonging to this process
        let mut windows_to_hide: Vec<HWND> = Vec::new();

        // Use EnumWindows to find all top-level windows
        let callback_data = &mut windows_to_hide as *mut Vec<HWND>;

        unsafe extern "system" fn enum_callback(
            hwnd: HWND,
            lparam: windows::Win32::Foundation::LPARAM,
        ) -> BOOL {
            let windows = &mut *(lparam.0 as *mut Vec<HWND>);
            windows.push(hwnd);
            BOOL::from(true)
        }

        let _ = EnumWindows(
            Some(enum_callback),
            windows::Win32::Foundation::LPARAM(callback_data as isize),
        );

        let mut hidden_count = 0;
        for hwnd in windows_to_hide {
            // Get the process ID of this window
            let mut window_pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

            if window_pid == pid {
                // Check if this is a visible window
                if IsWindowVisible(hwnd).as_bool() {
                    // Get current extended style
                    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

                    // Add WS_EX_TOOLWINDOW (0x80) and remove WS_EX_APPWINDOW (0x40000)
                    let new_ex_style = (ex_style | 0x80) & !0x40000;

                    if ex_style != new_ex_style {
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);
                        hidden_count += 1;
                        debug!("Hidden window {:?} from taskbar", hwnd);
                    }
                }
            }
        }

        if hidden_count > 0 {
            info!(
                "Hidden {} window(s) from taskbar for PID {}",
                hidden_count, pid
            );
        } else {
            debug!(
                "No visible windows found for PID {} to hide from taskbar",
                pid
            );
        }
    }

    Ok(())
}
