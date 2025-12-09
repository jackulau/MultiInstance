//! MultiInstance - Run multiple instances of single-instance applications
//!
//! A cross-platform desktop application for launching and managing multiple
//! instances of applications that traditionally restrict themselves to a single instance.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)] // Many API methods are part of a comprehensive public API

mod core;
mod persistence;
mod platform;
mod ui;

use anyhow::Result;
use single_instance::SingleInstance;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::core::AppState;
use crate::persistence::Database;
use crate::ui::MultiInstanceApp;

/// Application name constant
pub const APP_NAME: &str = "MultiInstance";

/// Application version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    info!("{} v{} starting...", APP_NAME, APP_VERSION);

    // Ensure only one instance of MultiInstance itself is running
    let instance = SingleInstance::new(APP_NAME).expect("Failed to create single instance lock");
    if !instance.is_single() {
        error!("Another instance of {} is already running!", APP_NAME);
        show_already_running_dialog();
        return Ok(());
    }

    // Initialize database
    let db = Database::new()?;
    db.initialize()?;
    info!("Database initialized");

    // Create application state
    let app_state = AppState::new(db)?;
    info!("Application state initialized");

    // Restore previous session if configured
    if app_state.settings.read().unwrap().auto_restore_sessions {
        if let Err(e) = app_state.restore_session() {
            error!("Failed to restore previous session: {}", e);
        }
    }

    // Run the GUI application
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    info!("Starting GUI...");
    eframe::run_native(
        &format!("{} v{}", APP_NAME, APP_VERSION),
        native_options,
        Box::new(|cc| Ok(Box::new(MultiInstanceApp::new(cc, app_state)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    info!("{} shutting down", APP_NAME);
    Ok(())
}

/// Initialize the logging system
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("multiinstance=info,eframe=warn,egui=warn,wgpu=error"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load the application icon
fn load_app_icon() -> egui::IconData {
    // Default icon - blue gradient hexagon concept
    // In production, this would load from an embedded resource
    let size = 64;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let cx = x as f32 - size as f32 / 2.0;
            let cy = y as f32 - size as f32 / 2.0;
            let dist = (cx * cx + cy * cy).sqrt();

            if dist < size as f32 / 2.0 - 2.0 {
                // Blue gradient
                let t = dist / (size as f32 / 2.0);
                rgba[idx] = (37.0 + t * 30.0) as u8; // R
                rgba[idx + 1] = (99.0 - t * 35.0) as u8; // G
                rgba[idx + 2] = (235.0 - t * 60.0) as u8; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

/// Show dialog when another instance is already running
fn show_already_running_dialog() {
    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};

        let title: Vec<u16> = format!("{}\0", APP_NAME).encode_utf16().collect();
        let msg: Vec<u16> = format!("{} is already running!\0", APP_NAME)
            .encode_utf16()
            .collect();

        unsafe {
            MessageBoxW(
                None,
                PCWSTR::from_raw(msg.as_ptr()),
                PCWSTR::from_raw(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, just print to stderr - the system will show the dock icon bounce
        eprintln!("{} is already running!", APP_NAME);
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        eprintln!("{} is already running!", APP_NAME);
    }
}
