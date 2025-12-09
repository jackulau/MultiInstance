//! Instance details dialog

use egui::Context;

use crate::core::{resource::format_bytes, AppState, InstanceId};
use crate::ui::components::ResourceBar;
use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

pub fn render(ctx: &Context, id: InstanceId, state: &mut AppState, dialog: &mut DialogState) {
    let instances = state.instances.read().unwrap();
    let Some(instance) = instances.get(&id).cloned() else {
        *dialog = DialogState::None;
        return;
    };
    drop(instances);

    let mut open = true;

    egui::Window::new(format!("Details: {}", instance.display_name()))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(550.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Status header
                ui.horizontal(|ui| {
                    let color = Theme::status_color(&instance.status);
                    ui.label(
                        egui::RichText::new(instance.status.label())
                            .size(18.0)
                            .color(color),
                    );

                    if instance.status.is_active() {
                        ui.label(
                            egui::RichText::new(format!("Uptime: {}", instance.uptime_string()))
                                .color(Theme::TEXT_SECONDARY),
                        );
                    }
                });

                ui.add_space(16.0);

                // Basic info
                egui::Frame::none()
                    .fill(Theme::BG_SECONDARY)
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Instance Info").strong());
                        ui.add_space(8.0);

                        egui::Grid::new("instance_info_grid")
                            .num_columns(2)
                            .spacing([16.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("ID:");
                                ui.label(
                                    egui::RichText::new(instance.id.to_string())
                                        .small()
                                        .color(Theme::TEXT_MUTED),
                                );
                                ui.end_row();

                                ui.label("Executable:");
                                ui.label(
                                    egui::RichText::new(
                                        instance.config.executable_path.to_string_lossy(),
                                    )
                                    .small(),
                                );
                                ui.end_row();

                                if let Some(pid) = instance.pid {
                                    ui.label("PID:");
                                    ui.label(egui::RichText::new(pid.to_string()));
                                    ui.end_row();
                                }

                                if !instance.config.arguments.is_empty() {
                                    ui.label("Arguments:");
                                    ui.label(
                                        egui::RichText::new(instance.config.arguments.join(" "))
                                            .small(),
                                    );
                                    ui.end_row();
                                }

                                if let Some(ref group) = instance.config.group {
                                    ui.label("Group:");
                                    ui.label(group);
                                    ui.end_row();
                                }

                                ui.label("Created:");
                                ui.label(
                                    egui::RichText::new(
                                        instance.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                    )
                                    .small(),
                                );
                                ui.end_row();

                                ui.label("Restarts:");
                                ui.label(instance.restart_count.to_string());
                                ui.end_row();
                            });
                    });

                ui.add_space(16.0);

                // Resource usage (if active)
                if instance.status.is_active() {
                    egui::Frame::none()
                        .fill(Theme::BG_SECONDARY)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Resource Usage").strong());
                            ui.add_space(8.0);

                            let usage = &instance.resource_usage;

                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("CPU");
                                    ResourceBar::horizontal(
                                        ui,
                                        usage.cpu_percent / 100.0,
                                        "",
                                        150.0,
                                        true,
                                    );
                                });

                                ui.add_space(16.0);

                                ui.vertical(|ui| {
                                    ui.label("Memory");
                                    ui.label(
                                        egui::RichText::new(format_bytes(usage.memory_bytes))
                                            .size(18.0)
                                            .color(Theme::PRIMARY_LIGHT),
                                    );
                                });
                            });

                            ui.add_space(8.0);

                            egui::Grid::new("resource_grid")
                                .num_columns(2)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Virtual Memory:");
                                    ui.label(format_bytes(usage.virtual_memory_bytes));
                                    ui.end_row();

                                    ui.label("Network RX:");
                                    ui.label(format_bytes(usage.network_rx_bytes));
                                    ui.end_row();

                                    ui.label("Network TX:");
                                    ui.label(format_bytes(usage.network_tx_bytes));
                                    ui.end_row();

                                    ui.label("Disk Read:");
                                    ui.label(format_bytes(usage.disk_read_bytes));
                                    ui.end_row();

                                    ui.label("Disk Write:");
                                    ui.label(format_bytes(usage.disk_write_bytes));
                                    ui.end_row();
                                });
                        });

                    ui.add_space(16.0);
                }

                // Resource limits
                egui::Frame::none()
                    .fill(Theme::BG_SECONDARY)
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Resource Limits").strong());
                        ui.add_space(8.0);

                        let limits = &instance.config.resource_limits;

                        egui::Grid::new("limits_grid")
                            .num_columns(2)
                            .spacing([16.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("CPU Limit:");
                                ui.label(if limits.cpu_percent == 0 {
                                    "Unlimited".to_string()
                                } else {
                                    format!("{}%", limits.cpu_percent)
                                });
                                ui.end_row();

                                ui.label("Memory Limit:");
                                ui.label(if limits.memory_mb == 0 {
                                    "Unlimited".to_string()
                                } else {
                                    format!("{} MB", limits.memory_mb)
                                });
                                ui.end_row();

                                ui.label("Network Limit:");
                                ui.label(if limits.network_kbps == 0 {
                                    "Unlimited".to_string()
                                } else {
                                    format!("{} KB/s", limits.network_kbps)
                                });
                                ui.end_row();

                                ui.label("Priority:");
                                ui.label(limits.priority.to_string());
                                ui.end_row();
                            });
                    });

                ui.add_space(16.0);

                // Error info
                if let Some(ref error) = instance.last_error {
                    egui::Frame::none()
                        .fill(Theme::ERROR.linear_multiply(0.2))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Last Error")
                                    .strong()
                                    .color(Theme::ERROR),
                            );
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(error).color(Theme::TEXT_PRIMARY));
                        });

                    ui.add_space(16.0);
                }

                // Action buttons
                ui.horizontal(|ui| {
                    match instance.status {
                        crate::core::InstanceStatus::Running => {
                            if ui.button("Pause").clicked() {
                                let _ = state.pause_instance(id);
                            }
                            if ui.button("Stop").clicked() {
                                let _ = state.stop_instance(id);
                            }
                            if ui.button("Restart").clicked() {
                                let _ = state.restart_instance(id);
                            }
                        }
                        crate::core::InstanceStatus::Paused => {
                            if ui.button("Resume").clicked() {
                                let _ = state.resume_instance(id);
                            }
                            if ui.button("Stop").clicked() {
                                let _ = state.stop_instance(id);
                            }
                        }
                        crate::core::InstanceStatus::Stopped
                        | crate::core::InstanceStatus::Crashed => {
                            if ui.button("Start").clicked() {
                                let _ = state.start_instance(id);
                            }
                        }
                        _ => {}
                    }

                    if ui.button("Edit").clicked() {
                        *dialog = DialogState::EditInstance(id);
                    }

                    if ui.button("Close").clicked() {
                        *dialog = DialogState::None;
                    }
                });
            });
        });

    if !open {
        *dialog = DialogState::None;
    }
}
