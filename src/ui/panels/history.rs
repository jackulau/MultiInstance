//! History panel - View instance history and events

use egui::Ui;

use crate::core::AppState;
use crate::ui::theme::Theme;

pub fn render(ui: &mut Ui, state: &AppState) {
    ui.heading("Instance History");
    ui.add_space(8.0);

    // This is a placeholder - would need to implement actual history tracking
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Recent Activity")
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.add_space(16.0);

            // Get instances and show recent activity
            let instances = state.instances.read().unwrap();

            if instances.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("No history yet").color(Theme::TEXT_MUTED));
                    ui.label(
                        egui::RichText::new("Instance events will appear here")
                            .small()
                            .color(Theme::TEXT_MUTED),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        // Sort instances by last activity
                        let mut sorted: Vec<_> = instances.values().collect();
                        sorted.sort_by(|a, b| {
                            let a_time = a.started_at.or(a.stopped_at);
                            let b_time = b.started_at.or(b.stopped_at);
                            b_time.cmp(&a_time)
                        });

                        for instance in sorted.iter().take(20) {
                            egui::Frame::none()
                                .fill(Theme::BG_TERTIARY)
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Status indicator
                                        let color = Theme::status_color(&instance.status);
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(8.0, 8.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(rect.center(), 4.0, color);

                                        ui.add_space(8.0);

                                        // Instance name
                                        ui.label(
                                            egui::RichText::new(instance.display_name()).strong(),
                                        );

                                        ui.add_space(8.0);

                                        // Status
                                        ui.label(
                                            egui::RichText::new(instance.status.label())
                                                .small()
                                                .color(color),
                                        );

                                        // Timestamp
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let time =
                                                    instance.started_at.or(instance.stopped_at);
                                                if let Some(time) = time {
                                                    ui.label(
                                                        egui::RichText::new(
                                                            time.format("%Y-%m-%d %H:%M")
                                                                .to_string(),
                                                        )
                                                        .small()
                                                        .color(Theme::TEXT_MUTED),
                                                    );
                                                }
                                            },
                                        );
                                    });

                                    // Additional details
                                    if instance.restart_count > 0 {
                                        ui.horizontal(|ui| {
                                            ui.add_space(16.0);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "Restarted {} times",
                                                    instance.restart_count
                                                ))
                                                .small()
                                                .color(Theme::TEXT_MUTED),
                                            );
                                        });
                                    }

                                    if let Some(ref error) = instance.last_error {
                                        ui.horizontal(|ui| {
                                            ui.add_space(16.0);
                                            ui.label(
                                                egui::RichText::new(error)
                                                    .small()
                                                    .color(Theme::ERROR),
                                            );
                                        });
                                    }
                                });

                            ui.add_space(4.0);
                        }
                    });
            }
        });

    ui.add_space(16.0);

    // Statistics
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Statistics")
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.add_space(16.0);

            let instances = state.instances.read().unwrap();
            let total = instances.len();
            let active = instances.values().filter(|i| i.status.is_active()).count();
            let crashed = instances
                .values()
                .filter(|i| i.status == crate::core::InstanceStatus::Crashed)
                .count();
            let total_restarts: u32 = instances.values().map(|i| i.restart_count).sum();

            ui.horizontal(|ui| {
                stat_item(ui, "Total Instances", &total.to_string());
                stat_item(ui, "Active", &active.to_string());
                stat_item(ui, "Crashed", &crashed.to_string());
                stat_item(ui, "Total Restarts", &total_restarts.to_string());
            });

            let profiles = state.profiles.read().unwrap();
            let total_launches: u32 = profiles.values().map(|p| p.launch_count).sum();
            drop(profiles);

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                stat_item(ui, "Profiles", &state.profile_count().to_string());
                stat_item(ui, "Profile Launches", &total_launches.to_string());
            });
        });

    ui.add_space(16.0);

    // Clear history button
    ui.horizontal(|ui| {
        if ui.button("Clear History").clicked() {
            // Would clear history from database
            tracing::info!("Clear history requested");
        }

        if ui.button("Export History").clicked() {
            // Would export history to file
            tracing::info!("Export history requested");
        }
    });
}

fn stat_item(ui: &mut Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(Theme::BG_TERTIARY)
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(16.0, 8.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(value)
                        .size(24.0)
                        .strong()
                        .color(Theme::PRIMARY_LIGHT),
                );
                ui.label(egui::RichText::new(label).small().color(Theme::TEXT_MUTED));
            });
        });
}
