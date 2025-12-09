//! New instance dialog

use egui::{Color32, Context};

use crate::core::{AppState, InstanceConfig};
use crate::ui::app::{Notification, NotificationLevel};
use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

/// Helper to render a form field with label and input
fn form_field(ui: &mut egui::Ui, label: &str, add_input: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.set_width(ui.available_width());
            add_input(ui);
        });
    });
    ui.add_space(12.0);
}

/// Helper for section headers
fn section_header(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(icon)
                .size(16.0)
                .color(Theme::PRIMARY_LIGHT),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(title)
                .size(15.0)
                .strong()
                .color(Color32::WHITE),
        );
    });
    ui.add_space(12.0);
}

pub fn render(
    ctx: &Context,
    config: &mut Option<InstanceConfig>,
    state: &mut AppState,
    dialog: &mut DialogState,
    notifications: &mut Vec<Notification>,
) {
    let Some(config) = config else {
        *dialog = DialogState::None;
        return;
    };

    let mut open = true;

    egui::Window::new("New Instance")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Theme::BG_ELEVATED)
                .rounding(egui::Rounding::same(12.0))
                .stroke(egui::Stroke::new(1.0, Theme::BORDER))
                .inner_margin(egui::Margin::same(24.0)),
        )
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Create New Instance")
                        .size(20.0)
                        .strong()
                        .color(Color32::WHITE),
                );
            });
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Configure and launch a new application instance")
                    .size(13.0)
                    .color(Theme::TEXT_MUTED),
            );
            ui.add_space(20.0);

            egui::ScrollArea::vertical()
                .max_height(450.0)
                .show(ui, |ui| {
                    // Basic Info Section
                    section_header(ui, "◈", "Basic Information");

                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            // Name
                            ui.label(
                                egui::RichText::new("Instance Name")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut config.name)
                                    .hint_text("Enter a name for this instance")
                                    .desired_width(f32::INFINITY),
                            );

                            ui.add_space(16.0);

                            // Executable
                            ui.label(
                                egui::RichText::new("Executable Path")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let path_str = config.executable_path.to_string_lossy().to_string();
                                let mut path_edit = path_str.clone();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut path_edit)
                                            .hint_text("Path to executable")
                                            .desired_width(ui.available_width() - 90.0),
                                    )
                                    .changed()
                                {
                                    config.executable_path = path_edit.into();
                                }

                                let browse_btn = egui::Button::new("Browse...")
                                    .fill(Theme::BG_TERTIARY)
                                    .rounding(egui::Rounding::same(6.0));
                                if ui.add(browse_btn).clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Executable", &["exe", "app", ""])
                                        .pick_file()
                                    {
                                        config.executable_path = path;
                                        if config.name.is_empty() {
                                            config.name = config
                                                .executable_path
                                                .file_stem()
                                                .map(|s| s.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                        }
                                    }
                                }
                            });

                            ui.add_space(16.0);

                            // Arguments
                            ui.label(
                                egui::RichText::new("Arguments")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            let mut args_str = config.arguments.join(" ");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut args_str)
                                        .hint_text("Command line arguments (optional)")
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                config.arguments =
                                    args_str.split_whitespace().map(|s| s.to_string()).collect();
                            }

                            ui.add_space(16.0);

                            // Working directory
                            ui.label(
                                egui::RichText::new("Working Directory")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let mut dir_str = config
                                    .working_directory
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut dir_str)
                                            .hint_text("Working directory (optional)")
                                            .desired_width(ui.available_width() - 90.0),
                                    )
                                    .changed()
                                {
                                    config.working_directory = if dir_str.is_empty() {
                                        None
                                    } else {
                                        Some(dir_str.into())
                                    };
                                }
                                let browse_btn = egui::Button::new("Browse...")
                                    .fill(Theme::BG_TERTIARY)
                                    .rounding(egui::Rounding::same(6.0));
                                if ui.add(browse_btn).clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        config.working_directory = Some(path);
                                    }
                                }
                            });
                        });

                    // Instance Isolation Section
                    section_header(ui, "🔒", "Instance Isolation");

                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            // Styled checkbox row
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut config.bypass_single_instance, "");
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Bypass single-instance check")
                                            .size(13.0)
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "Allows running multiple instances of the same app",
                                        )
                                        .size(11.0)
                                        .color(Theme::TEXT_MUTED),
                                    );
                                });
                            });

                            ui.add_space(12.0);

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut config.use_environment_isolation, "");
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Use environment isolation")
                                            .size(13.0)
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new("Sets custom APPDATA/profile paths")
                                            .size(11.0)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                });
                            });

                            ui.add_space(12.0);

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut config.hide_from_taskbar, "");
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Hide from taskbar")
                                            .size(13.0)
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "Hides the instance window from the Windows taskbar",
                                        )
                                        .size(11.0)
                                        .color(Theme::TEXT_MUTED),
                                    );
                                });
                            });
                        });

                    ui.add_space(20.0);

                    // Resource Limits Section
                    section_header(ui, "⚡", "Resource Limits");

                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Leave at 0 for unlimited/default values")
                                    .size(11.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(12.0);

                            // CPU Limit
                            ui.label(
                                egui::RichText::new("CPU Limit")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut config.resource_limits.cpu_percent,
                                        0..=100,
                                    )
                                    .suffix("%")
                                    .custom_formatter(
                                        |n, _| {
                                            if n == 0.0 {
                                                "Unlimited".to_string()
                                            } else {
                                                format!("{:.0}%", n)
                                            }
                                        },
                                    ),
                                );
                            });

                            ui.add_space(12.0);

                            // Memory Limit
                            ui.label(
                                egui::RichText::new("Memory Limit")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            let mut mem = config.resource_limits.memory_mb.min(16384) as u32;
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut mem, 0..=16384)
                                        .suffix(" MB")
                                        .logarithmic(true)
                                        .custom_formatter(|n, _| {
                                            if n == 0.0 {
                                                "Unlimited".to_string()
                                            } else if n >= 1024.0 {
                                                format!("{:.1} GB", n / 1024.0)
                                            } else {
                                                format!("{:.0} MB", n)
                                            }
                                        }),
                                );
                            });
                            config.resource_limits.memory_mb = mem as u64;

                            ui.add_space(12.0);

                            // Priority
                            ui.label(
                                egui::RichText::new("Process Priority")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.add(
                                egui::Slider::new(&mut config.resource_limits.priority, -2..=2)
                                    .custom_formatter(|n, _| match n as i8 {
                                        -2 => "Idle".to_string(),
                                        -1 => "Below Normal".to_string(),
                                        0 => "Normal".to_string(),
                                        1 => "Above Normal".to_string(),
                                        2 => "High".to_string(),
                                        _ => format!("{}", n),
                                    }),
                            );
                        });

                    ui.add_space(20.0);

                    // Automation Section
                    section_header(ui, "↻", "Automation");

                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut config.auto_restart, "");
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Auto-restart on crash")
                                            .size(13.0)
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "Automatically restart if the instance crashes",
                                        )
                                        .size(11.0)
                                        .color(Theme::TEXT_MUTED),
                                    );
                                });
                            });

                            if config.auto_restart {
                                ui.add_space(12.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Restart delay:")
                                            .size(12.0)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                    ui.add_space(8.0);
                                    let mut delay = config.restart_delay_secs as i32;
                                    ui.add(
                                        egui::DragValue::new(&mut delay)
                                            .range(0..=300)
                                            .suffix(" sec"),
                                    );
                                    config.restart_delay_secs = delay as u32;
                                });
                            }
                        });

                    ui.add_space(20.0);

                    // Group & Notes Section
                    section_header(ui, "📋", "Organization");

                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            // Group
                            ui.label(
                                egui::RichText::new("Group")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            let groups = state.groups.read().unwrap();
                            let current = config.group.clone().unwrap_or_default();

                            egui::ComboBox::from_id_salt("group_select")
                                .width(200.0)
                                .selected_text(if current.is_empty() { "None" } else { &current })
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(config.group.is_none(), "None")
                                        .clicked()
                                    {
                                        config.group = None;
                                    }
                                    for group in groups.iter() {
                                        if ui
                                            .selectable_label(
                                                config.group.as_ref() == Some(group),
                                                group,
                                            )
                                            .clicked()
                                        {
                                            config.group = Some(group.clone());
                                        }
                                    }
                                });

                            ui.add_space(16.0);

                            // Notes
                            ui.label(
                                egui::RichText::new("Notes")
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::multiline(&mut config.notes)
                                    .hint_text("Add notes about this instance (optional)")
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(3),
                            );
                        });

                    ui.add_space(16.0);
                });

            ui.add_space(16.0);

            // Divider before action buttons
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, Theme::BORDER_LIGHT);

            ui.add_space(16.0);

            // Action buttons
            ui.horizontal(|ui| {
                let can_create = !config.executable_path.as_os_str().is_empty();

                // Primary action button
                let create_launch_btn =
                    egui::Button::new(egui::RichText::new("Create & Launch").color(Color32::WHITE))
                        .fill(if can_create {
                            Theme::PRIMARY
                        } else {
                            Theme::BG_TERTIARY
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(130.0, 38.0));

                if ui.add_enabled(can_create, create_launch_btn).clicked() {
                    match state.create_instance(config.clone(), true) {
                        Ok(_) => {
                            notifications.push(Notification {
                                message: format!("Instance '{}' created and launched", config.name),
                                level: NotificationLevel::Success,
                                created_at: std::time::Instant::now(),
                            });
                            *dialog = DialogState::None;
                        }
                        Err(e) => {
                            notifications.push(Notification {
                                message: format!("Failed to create instance: {}", e),
                                level: NotificationLevel::Error,
                                created_at: std::time::Instant::now(),
                            });
                        }
                    }
                }

                ui.add_space(8.0);

                // Secondary action button
                let create_btn = egui::Button::new("Create Only")
                    .fill(Theme::BG_TERTIARY)
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(100.0, 38.0));

                if ui.add_enabled(can_create, create_btn).clicked() {
                    match state.create_instance(config.clone(), false) {
                        Ok(_) => {
                            notifications.push(Notification {
                                message: format!("Instance '{}' created", config.name),
                                level: NotificationLevel::Success,
                                created_at: std::time::Instant::now(),
                            });
                            *dialog = DialogState::None;
                        }
                        Err(e) => {
                            notifications.push(Notification {
                                message: format!("Failed to create instance: {}", e),
                                level: NotificationLevel::Error,
                                created_at: std::time::Instant::now(),
                            });
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new("Cancel").color(Theme::TEXT_SECONDARY),
                    )
                    .fill(Color32::TRANSPARENT)
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(80.0, 38.0));

                    if ui.add(cancel_btn).clicked() {
                        *dialog = DialogState::None;
                    }
                });
            });
        });

    if !open {
        *dialog = DialogState::None;
    }
}
