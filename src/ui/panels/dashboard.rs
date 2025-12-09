//! Dashboard panel - Overview of system resources and instances

use egui::{Color32, Ui};

use crate::core::resource::format_bytes;
use crate::core::AppState;
use crate::ui::components::{InstanceCard, ResourceBar};
use crate::ui::theme::Theme;

/// Section header helper
fn section_header(ui: &mut Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(icon)
                .size(18.0)
                .color(Theme::PRIMARY_LIGHT),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
    });
    ui.add_space(14.0);
}

pub fn render(ui: &mut Ui, state: &mut AppState, show_system_resources: bool) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);

            // System Resources Overview
            if show_system_resources {
                render_system_resources(ui, state);
                ui.add_space(24.0);
            }

            // Quick Launch Bar
            render_quick_launch(ui, state);
            ui.add_space(24.0);

            // Active Instances Grid
            render_active_instances(ui, state);

            ui.add_space(20.0);
        });
}

fn render_system_resources(ui: &mut Ui, state: &AppState) {
    let resources = state.resource_monitor.get_system_resources();

    section_header(ui, "📊", "System Resources");

    // Resource cards in a horizontal layout
    ui.horizontal(|ui| {
        // CPU Card
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(280.0);

                ui.horizontal(|ui| {
                    // Circular progress indicator
                    ResourceBar::circular(ui, resources.cpu_percent / 100.0, 70.0);

                    ui.add_space(16.0);

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("CPU")
                                .size(16.0)
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&resources.cpu_name)
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("{} cores", resources.cpu_cores))
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                    });
                });

                ui.add_space(16.0);

                // Per-core bars
                ui.horizontal_wrapped(|ui| {
                    for (i, &usage) in resources.cpu_per_core.iter().take(12).enumerate() {
                        ResourceBar::vertical(ui, usage / 100.0, 32.0)
                            .on_hover_text(format!("Core {}: {:.0}%", i, usage));
                    }
                    if resources.cpu_per_core.len() > 12 {
                        ui.label(
                            egui::RichText::new(format!("+{}", resources.cpu_per_core.len() - 12))
                                .size(10.0)
                                .color(Theme::TEXT_MUTED),
                        );
                    }
                });
            });

        ui.add_space(16.0);

        // Memory Card
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(260.0);

                ui.horizontal(|ui| {
                    let mem_percent = resources.memory_percent() / 100.0;
                    ResourceBar::circular(ui, mem_percent, 70.0);

                    ui.add_space(16.0);

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Memory")
                                .size(16.0)
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} / {}",
                                resources.used_memory_string(),
                                resources.total_memory_string()
                            ))
                            .size(12.0)
                            .color(Theme::TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} available",
                                resources.available_memory_string()
                            ))
                            .size(11.0)
                            .color(Theme::TEXT_MUTED),
                        );
                    });
                });

                // Swap usage if present
                if resources.total_swap > 0 {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Swap:")
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                        ui.add_space(8.0);
                        ResourceBar::mini(ui, resources.swap_percent() / 100.0);
                    });
                }
            });

        ui.add_space(16.0);

        // Network Card
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_min_width(200.0);

                ui.label(
                    egui::RichText::new("Network")
                        .size(16.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.add_space(12.0);

                for iface in resources.network_interfaces.iter().take(3) {
                    ui.label(
                        egui::RichText::new(&iface.name)
                            .size(12.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("↓").size(12.0).color(Theme::SUCCESS));
                        ui.label(
                            egui::RichText::new(format!("{}/s", format_bytes(iface.rx_rate)))
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("↑").size(12.0).color(Theme::INFO));
                        ui.label(
                            egui::RichText::new(format!("{}/s", format_bytes(iface.tx_rate)))
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                    ui.add_space(8.0);
                }

                ui.add_space(4.0);

                // System uptime
                let uptime_secs = resources.uptime_secs;
                let days = uptime_secs / 86400;
                let hours = (uptime_secs % 86400) / 3600;
                let minutes = (uptime_secs % 3600) / 60;
                let uptime_str = if days > 0 {
                    format!("{}d {}h {}m", days, hours, minutes)
                } else {
                    format!("{}h {}m", hours, minutes)
                };
                ui.label(
                    egui::RichText::new(format!("Uptime: {}", uptime_str))
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
                );
            });
    });
}

fn render_quick_launch(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("⚡")
                .size(18.0)
                .color(Theme::PRIMARY_LIGHT),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("Quick Launch")
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let add_btn = egui::Button::new("+ Add")
                .fill(Theme::BG_TERTIARY)
                .rounding(egui::Rounding::same(6.0));
            if ui.add(add_btn).clicked() {
                // Would open file picker
            }
        });
    });
    ui.add_space(14.0);

    let quick_launch_items: Vec<_> = {
        let quick_launch = state.quick_launch.read().unwrap();
        quick_launch.iter().cloned().collect()
    };

    if quick_launch_items.is_empty() {
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(32.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("⚡")
                            .size(32.0)
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("No quick launch items")
                            .size(14.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new("Add your favorite apps for one-click launching")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });
    } else {
        let mut launch_idx = None;
        ui.horizontal_wrapped(|ui| {
            for (idx, config) in quick_launch_items.iter().enumerate() {
                egui::Frame::none()
                    .fill(Theme::BG_SECONDARY)
                    .rounding(egui::Rounding::same(10.0))
                    .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&config.name)
                                    .size(14.0)
                                    .strong()
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.add_space(12.0);
                            let launch_btn =
                                egui::Button::new(egui::RichText::new("▶").color(Theme::SUCCESS))
                                    .fill(Theme::SUCCESS.linear_multiply(0.15))
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(32.0, 28.0));
                            if ui.add(launch_btn).on_hover_text("Launch").clicked() {
                                launch_idx = Some(idx);
                            }
                        });
                    });
                ui.add_space(8.0);
            }
        });

        if let Some(idx) = launch_idx {
            let config = quick_launch_items[idx].clone();
            if let Err(e) = state.create_instance(config, true) {
                tracing::error!("Failed to launch: {}", e);
            }
        }
    }
}

fn render_active_instances(ui: &mut Ui, state: &mut AppState) {
    let active_count = {
        let instances = state.instances.read().unwrap();
        instances.values().filter(|i| i.status.is_active()).count()
    };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("▣")
                .size(18.0)
                .color(Theme::PRIMARY_LIGHT),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("Active Instances")
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(8.0);

        // Count badge
        egui::Frame::none()
            .fill(Theme::PRIMARY.linear_multiply(0.2))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("{}", active_count))
                        .size(13.0)
                        .color(Theme::PRIMARY_LIGHT),
                );
            });
    });
    ui.add_space(14.0);

    if active_count == 0 {
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(40.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("📦").size(40.0));
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("No active instances")
                            .size(16.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Create a new instance to get started")
                            .size(13.0)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });
    } else {
        // Grid layout - collect IDs first to avoid borrow issues
        let mut pending_action: Option<(
            crate::core::InstanceId,
            crate::ui::components::CardAction,
        )> = None;

        ui.horizontal_wrapped(|ui| {
            let instances = state.instances.read().unwrap();
            let active_ids: Vec<_> = instances
                .iter()
                .filter(|(_, i)| i.status.is_active())
                .map(|(id, _)| *id)
                .collect();
            drop(instances);

            for id in active_ids {
                let instances = state.instances.read().unwrap();
                if let Some(instance) = instances.get(&id) {
                    let instance = instance.clone();
                    drop(instances);

                    let card_response = InstanceCard::grid(ui, &instance);

                    if let Some(action) = card_response.action {
                        pending_action = Some((id, action));
                        break;
                    }
                }
                ui.add_space(8.0);
            }
        });

        if let Some((id, action)) = pending_action {
            use crate::ui::components::CardAction;
            match action {
                CardAction::Start => {
                    if let Err(e) = state.start_instance(id) {
                        tracing::error!("Failed to start instance: {}", e);
                    }
                }
                CardAction::Stop => {
                    if let Err(e) = state.stop_instance(id) {
                        tracing::error!("Failed to stop instance: {}", e);
                    }
                }
                CardAction::Pause => {
                    if let Err(e) = state.pause_instance(id) {
                        tracing::error!("Failed to pause instance: {}", e);
                    }
                }
                CardAction::Resume => {
                    if let Err(e) = state.resume_instance(id) {
                        tracing::error!("Failed to resume instance: {}", e);
                    }
                }
                CardAction::Restart => {
                    if let Err(e) = state.restart_instance(id) {
                        tracing::error!("Failed to restart instance: {}", e);
                    }
                }
                _ => {}
            }
        }

        ui.add_space(16.0);

        // Aggregate resource usage summary
        let instances = state.instances.read().unwrap();
        let total_cpu: f32 = instances
            .values()
            .filter(|i| i.status.is_active())
            .map(|i| i.resource_usage.cpu_percent)
            .sum();
        let total_memory: u64 = instances
            .values()
            .filter(|i| i.status.is_active())
            .map(|i| i.resource_usage.memory_bytes)
            .sum();
        drop(instances);

        // Summary bar
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY.linear_multiply(0.6))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Total Resource Usage:")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(16.0);

                    // CPU badge
                    egui::Frame::none()
                        .fill(Theme::BG_TERTIARY)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("CPU {:.1}%", total_cpu))
                                    .size(11.0)
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        });

                    ui.add_space(8.0);

                    // Memory badge
                    egui::Frame::none()
                        .fill(Theme::BG_TERTIARY)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Memory {}",
                                    format_bytes(total_memory)
                                ))
                                .size(11.0)
                                .color(Theme::TEXT_SECONDARY),
                            );
                        });
                });
            });
    }
}
