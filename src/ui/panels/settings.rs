//! Settings panel

use egui::{Color32, Context, Ui, Vec2};

use crate::core::settings::{NotificationLevel, Theme as SettingsTheme, ViewMode};
use crate::core::AppState;
use crate::ui::theme::Theme;

/// Custom toggle switch widget for better UX
fn toggle_switch(ui: &mut Ui, on: &mut bool) -> egui::Response {
    let desired_size = Vec2::new(44.0, 24.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let _visuals = ui.style().interact_selectable(&response, *on);

        // Track background
        let track_color = if *on {
            Theme::SUCCESS.linear_multiply(0.9 + 0.1 * how_on)
        } else {
            Theme::BG_TERTIARY
        };

        let track_rect = rect;
        ui.painter().rect(
            track_rect,
            egui::Rounding::same(12.0),
            track_color,
            egui::Stroke::new(1.0, if *on { Theme::SUCCESS } else { Theme::BORDER }),
        );

        // Sliding circle
        let circle_x = egui::lerp((rect.left() + 12.0)..=(rect.right() - 12.0), how_on);
        let circle_center = egui::pos2(circle_x, rect.center().y);
        let circle_radius = 9.0;

        // Circle shadow
        ui.painter().circle(
            circle_center + Vec2::new(0.0, 1.0),
            circle_radius,
            Color32::from_black_alpha(30),
            egui::Stroke::NONE,
        );

        // Circle
        ui.painter().circle(
            circle_center,
            circle_radius,
            Color32::WHITE,
            egui::Stroke::NONE,
        );
    }

    response
}

/// Helper to render a styled section header
fn section_header(ui: &mut Ui, icon: &str, title: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(icon)
                .size(20.0)
                .color(Theme::PRIMARY_LIGHT),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(title)
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );
    });
    ui.add_space(12.0);
}

/// Helper to render a toggle setting with label and description
fn toggle_setting(ui: &mut Ui, value: &mut bool, label: &str, description: &str) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.label(egui::RichText::new(label).size(14.0).color(Color32::WHITE));
                ui.label(
                    egui::RichText::new(description)
                        .size(12.0)
                        .color(Theme::TEXT_SECONDARY),
                );
            });
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            toggle_switch(ui, value);
        });
    });
    ui.add_space(14.0);
}

/// Helper to render a setting row with label, description, and custom widget
fn setting_row(ui: &mut Ui, label: &str, description: &str, add_widget: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.label(egui::RichText::new(label).size(14.0).color(Color32::WHITE));
                ui.label(
                    egui::RichText::new(description)
                        .size(12.0)
                        .color(Theme::TEXT_SECONDARY),
                );
            });
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_widget(ui);
        });
    });
    ui.add_space(14.0);
}

/// Styled section frame with better visual design
fn section_frame(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(egui::Rounding::same(12.0))
        .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
        .inner_margin(egui::Margin::same(20.0))
        .outer_margin(egui::Margin::symmetric(0.0, 4.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn render(ui: &mut Ui, state: &mut AppState, ctx: &Context) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Center content with max width
            ui.vertical_centered(|ui| {
                ui.set_max_width(680.0);

                let mut settings = state.settings.write().unwrap();

                // Page header
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Settings")
                            .size(26.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                });
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Configure application behavior and preferences")
                        .size(14.0)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(24.0);

                // General Settings
                section_header(ui, "\u{2699}", "General");
                section_frame(ui, |ui| {
                    toggle_setting(
                        ui,
                        &mut settings.start_with_system,
                        "Start with system",
                        "Launch MultiInstance automatically when you log in",
                    );

                    toggle_setting(
                        ui,
                        &mut settings.minimize_to_tray,
                        "Minimize to tray",
                        "Keep running in system tray when window is closed",
                    );

                    toggle_setting(
                        ui,
                        &mut settings.auto_restore_sessions,
                        "Auto-restore sessions",
                        "Restore previous instances when starting the application",
                    );

                    toggle_setting(
                        ui,
                        &mut settings.show_system_resources,
                        "Show system resources",
                        "Display CPU, memory, and network usage on dashboard",
                    );
                });

                ui.add_space(20.0);

                // Appearance
                section_header(ui, "\u{1F3A8}", "Appearance");
                section_frame(ui, |ui| {
                    setting_row(ui, "Theme", "Choose your preferred color scheme", |ui| {
                        egui::ComboBox::from_id_salt("theme_select")
                            .width(130.0)
                            .selected_text(settings.theme.label())
                            .show_ui(ui, |ui| {
                                for theme in SettingsTheme::all() {
                                    let selected = settings.theme == *theme;
                                    if ui.selectable_label(selected, theme.label()).clicked() {
                                        settings.theme = *theme;
                                        match theme {
                                            SettingsTheme::Dark => {
                                                crate::ui::theme::Theme::apply_dark(ctx)
                                            }
                                            SettingsTheme::Light => {
                                                crate::ui::theme::Theme::apply_light(ctx)
                                            }
                                            SettingsTheme::System => {
                                                crate::ui::theme::Theme::apply_dark(ctx)
                                            }
                                        }
                                    }
                                }
                            });
                    });

                    setting_row(
                        ui,
                        "Default view",
                        "How instances are displayed by default",
                        |ui| {
                            egui::ComboBox::from_id_salt("view_mode_select")
                                .width(130.0)
                                .selected_text(settings.view_mode.label())
                                .show_ui(ui, |ui| {
                                    for mode in ViewMode::all() {
                                        let selected = settings.view_mode == *mode;
                                        if ui.selectable_label(selected, mode.label()).clicked() {
                                            settings.view_mode = *mode;
                                        }
                                    }
                                });
                        },
                    );
                });

                ui.add_space(20.0);

                // Resource Limits
                section_header(ui, "\u{26A1}", "Default Resource Limits");
                section_frame(ui, |ui| {
                    // CPU Limit
                    let cpu_desc = if settings.default_cpu_limit == 0 {
                        "No limit on CPU usage".to_string()
                    } else {
                        format!("Limit to {}% CPU usage", settings.default_cpu_limit)
                    };
                    setting_row(ui, "CPU Limit", &cpu_desc, |ui| {
                        ui.add(
                            egui::Slider::new(&mut settings.default_cpu_limit, 0..=100)
                                .suffix("%")
                                .show_value(true)
                                .custom_formatter(|n, _| {
                                    if n == 0.0 {
                                        "Off".to_string()
                                    } else {
                                        format!("{:.0}%", n)
                                    }
                                }),
                        );
                    });

                    // RAM Limit
                    let ram_desc = if settings.default_ram_limit == 0 {
                        "No limit on memory usage".to_string()
                    } else {
                        format!("Limit to {} MB", settings.default_ram_limit)
                    };
                    setting_row(ui, "Memory Limit", &ram_desc, |ui| {
                        let mut ram_val = settings.default_ram_limit as i64;
                        ui.add(
                            egui::DragValue::new(&mut ram_val)
                                .range(0..=65536)
                                .suffix(" MB")
                                .speed(10.0),
                        );
                        settings.default_ram_limit = ram_val as u64;
                    });

                    // Network Limit
                    let net_desc = if settings.default_network_limit == 0 {
                        "No limit on network usage".to_string()
                    } else {
                        format!("Limit to {} KB/s", settings.default_network_limit)
                    };
                    setting_row(ui, "Network Limit", &net_desc, |ui| {
                        let mut net_val = settings.default_network_limit as i64;
                        ui.add(
                            egui::DragValue::new(&mut net_val)
                                .range(0..=1000000)
                                .suffix(" KB/s")
                                .speed(100.0),
                        );
                        settings.default_network_limit = net_val as u64;
                    });

                    // Priority
                    let priority_label = match settings.default_priority {
                        p if p <= -15 => "Realtime",
                        p if p <= -10 => "High",
                        p if p <= -5 => "Above Normal",
                        p if p <= 5 => "Normal",
                        p if p <= 10 => "Below Normal",
                        _ => "Idle",
                    };
                    setting_row(
                        ui,
                        "Process Priority",
                        &format!("Currently set to: {}", priority_label),
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut settings.default_priority, -20..=19)
                                    .show_value(false)
                                    .custom_formatter(|n, _| match n as i32 {
                                        p if p <= -15 => "Realtime".to_string(),
                                        p if p <= -10 => "High".to_string(),
                                        p if p <= -5 => "Above Normal".to_string(),
                                        p if p <= 5 => "Normal".to_string(),
                                        p if p <= 10 => "Below Normal".to_string(),
                                        _ => "Idle".to_string(),
                                    }),
                            );
                        },
                    );
                });

                ui.add_space(20.0);

                // Automation
                section_header(ui, "\u{1F504}", "Automation");
                section_frame(ui, |ui| {
                    toggle_setting(
                        ui,
                        &mut settings.default_auto_restart,
                        "Auto-restart on crash",
                        "Automatically restart instances when they crash unexpectedly",
                    );

                    setting_row(
                        ui,
                        "Restart delay",
                        "Time to wait before restarting a crashed instance",
                        |ui| {
                            let mut delay = settings.default_restart_delay_secs as i32;
                            ui.add(
                                egui::DragValue::new(&mut delay)
                                    .range(0..=300)
                                    .suffix(" sec")
                                    .speed(1.0),
                            );
                            settings.default_restart_delay_secs = delay as u32;
                        },
                    );

                    setting_row(
                        ui,
                        "Staggered launch delay",
                        "Delay between launching multiple instances",
                        |ui| {
                            let mut delay = settings.staggered_launch_delay_ms as i32;
                            ui.add(
                                egui::DragValue::new(&mut delay)
                                    .range(0..=60000)
                                    .suffix(" ms")
                                    .speed(100.0),
                            );
                            settings.staggered_launch_delay_ms = delay as u32;
                        },
                    );

                    toggle_setting(
                        ui,
                        &mut settings.enable_health_checks,
                        "Enable health checks",
                        "Periodically check if instances are responding correctly",
                    );
                });

                ui.add_space(20.0);

                // Notifications
                section_header(ui, "\u{1F514}", "Notifications");
                section_frame(ui, |ui| {
                    setting_row(
                        ui,
                        "Notification level",
                        "Control which events trigger notifications",
                        |ui| {
                            egui::ComboBox::from_id_salt("notification_level")
                                .width(130.0)
                                .selected_text(settings.notification_level.label())
                                .show_ui(ui, |ui| {
                                    for level in NotificationLevel::all() {
                                        let selected = settings.notification_level == *level;
                                        if ui.selectable_label(selected, level.label()).clicked() {
                                            settings.notification_level = *level;
                                        }
                                    }
                                });
                        },
                    );

                    toggle_setting(
                        ui,
                        &mut settings.notification_sound,
                        "Play sound",
                        "Play an audio alert when notifications appear",
                    );
                });

                ui.add_space(20.0);

                // Advanced
                section_header(ui, "\u{1F527}", "Advanced");
                section_frame(ui, |ui| {
                    setting_row(
                        ui,
                        "Monitor interval",
                        "How often to check instance status",
                        |ui| {
                            let mut interval = settings.monitor_interval_ms as i32;
                            ui.add(
                                egui::DragValue::new(&mut interval)
                                    .range(100..=10000)
                                    .suffix(" ms")
                                    .speed(10.0),
                            );
                            settings.monitor_interval_ms = interval as u32;
                        },
                    );

                    let max_desc = if settings.max_instances == 0 {
                        "No limit on concurrent instances".to_string()
                    } else {
                        format!("Maximum {} concurrent instances", settings.max_instances)
                    };
                    setting_row(ui, "Max instances", &max_desc, |ui| {
                        let mut max = settings.max_instances as i32;
                        ui.add(egui::DragValue::new(&mut max).range(0..=1000).speed(1.0));
                        settings.max_instances = max as u32;
                    });

                    let retention_desc = if settings.history_retention_days == 0 {
                        "Keep history forever".to_string()
                    } else {
                        format!("Keep {} days of history", settings.history_retention_days)
                    };
                    setting_row(ui, "History retention", &retention_desc, |ui| {
                        let mut days = settings.history_retention_days as i32;
                        ui.add(
                            egui::DragValue::new(&mut days)
                                .range(0..=365)
                                .suffix(" days")
                                .speed(1.0),
                        );
                        settings.history_retention_days = days as u32;
                    });

                    toggle_setting(
                        ui,
                        &mut settings.debug_logging,
                        "Debug logging",
                        "Enable verbose logging for troubleshooting",
                    );
                });

                ui.add_space(20.0);

                // Data
                section_header(ui, "\u{1F4C1}", "Data");
                section_frame(ui, |ui| {
                    let data_dir = settings.get_data_directory();
                    setting_row(ui, "Data directory", &data_dir.to_string_lossy(), |ui| {
                        if ui
                            .add(
                                egui::Button::new("Open Folder")
                                    .fill(Theme::BG_TERTIARY)
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(100.0, 28.0)),
                            )
                            .clicked()
                        {
                            let _ = open::that(&data_dir);
                        }
                    });
                });

                ui.add_space(32.0);

                drop(settings);

                // Action buttons
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new("Save Settings")
                        .fill(Theme::PRIMARY)
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(140.0, 40.0));

                    if ui.add(save_btn).clicked() {
                        if let Err(e) = state.save_settings() {
                            tracing::error!("Failed to save settings: {}", e);
                        }
                    }

                    ui.add_space(12.0);

                    let reset_btn = egui::Button::new("Reset to Defaults")
                        .fill(Theme::BG_TERTIARY)
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(140.0, 40.0));

                    if ui.add(reset_btn).clicked() {
                        *state.settings.write().unwrap() = crate::core::Settings::default();
                    }
                });

                ui.add_space(32.0);

                // About section
                egui::Frame::none()
                    .fill(Theme::BG_TERTIARY.linear_multiply(0.4))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "MultiInstance v{}",
                                    crate::APP_VERSION
                                ))
                                .size(15.0)
                                .strong()
                                .color(Color32::WHITE),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(
                                    "Run multiple instances of single-instance applications",
                                )
                                .size(13.0)
                                .color(Theme::TEXT_SECONDARY),
                            );
                        });
                    });

                ui.add_space(24.0);
            });
        });
}
