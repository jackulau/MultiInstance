//! Instance card component for grid/list views

use egui::{Color32, Ui};

use crate::core::{Instance, InstanceStatus};
use crate::ui::theme::{Icons, Theme};

use super::resource_bar::ResourceBar;
use super::status_badge::StatusBadge;

pub struct InstanceCard;

impl InstanceCard {
    /// Styled action button for cards
    fn action_button(ui: &mut Ui, icon: &str, tooltip: &str, color: Color32) -> bool {
        let btn = egui::Button::new(egui::RichText::new(icon).size(13.0).color(color))
            .fill(Theme::BG_TERTIARY)
            .rounding(egui::Rounding::same(6.0))
            .min_size(egui::vec2(32.0, 28.0));

        ui.add(btn).on_hover_text(tooltip).clicked()
    }

    /// Render instance as a grid card
    pub fn grid(ui: &mut Ui, instance: &Instance) -> CardResponse {
        let mut response = CardResponse::default();

        let status_color = Theme::status_color(&instance.status);
        let is_active = instance.status.is_active();

        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(
                1.0,
                if is_active {
                    status_color.linear_multiply(0.4)
                } else {
                    Theme::BORDER_LIGHT
                },
            ))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(240.0);

                // Header: Status indicator and name
                ui.horizontal(|ui| {
                    StatusBadge::dot(ui, &instance.status);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(instance.display_name())
                                .strong()
                                .size(15.0)
                                .color(Theme::TEXT_PRIMARY),
                        );
                        if let Some(path) = instance.config.executable_path.file_name() {
                            ui.label(
                                egui::RichText::new(path.to_string_lossy())
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        }
                    });
                });

                ui.add_space(14.0);

                // Resource usage (if running)
                if is_active {
                    // CPU bar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("CPU")
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                        ui.add_space(8.0);
                        ResourceBar::mini(ui, instance.resource_usage.cpu_percent / 100.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.0}%",
                                instance.resource_usage.cpu_percent
                            ))
                            .size(11.0)
                            .color(Theme::TEXT_SECONDARY),
                        );
                    });

                    ui.add_space(6.0);

                    // Memory bar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("MEM")
                                .size(11.0)
                                .color(Theme::TEXT_MUTED),
                        );
                        ui.add_space(4.0);
                        ResourceBar::mini(ui, 0.3); // Placeholder ratio
                        ui.label(
                            egui::RichText::new(instance.resource_usage.memory_string())
                                .size(11.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                    });

                    ui.add_space(10.0);

                    // Uptime badge
                    egui::Frame::none()
                        .fill(Theme::BG_TERTIARY.linear_multiply(0.6))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("⏱ {}", instance.uptime_string()))
                                    .size(11.0)
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        });
                } else if instance.status == InstanceStatus::Crashed {
                    if let Some(ref error) = instance.last_error {
                        egui::Frame::none()
                            .fill(Theme::ERROR.linear_multiply(0.15))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(error)
                                        .size(11.0)
                                        .color(Theme::ERROR_LIGHT),
                                );
                            });
                    }
                } else {
                    // Show placeholder for stopped instances
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Instance stopped")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);
                }

                ui.add_space(12.0);

                // Action buttons row
                ui.horizontal(|ui| {
                    match instance.status {
                        InstanceStatus::Running => {
                            if Self::action_button(ui, Icons::PAUSE, "Pause", Theme::WARNING) {
                                response.action = Some(CardAction::Pause);
                            }
                            ui.add_space(4.0);
                            if Self::action_button(ui, Icons::STOP, "Stop", Theme::ERROR_LIGHT) {
                                response.action = Some(CardAction::Stop);
                            }
                            ui.add_space(4.0);
                            if Self::action_button(ui, Icons::RESTART, "Restart", Theme::INFO) {
                                response.action = Some(CardAction::Restart);
                            }
                        }
                        InstanceStatus::Paused => {
                            if Self::action_button(ui, Icons::PLAY, "Resume", Theme::SUCCESS) {
                                response.action = Some(CardAction::Resume);
                            }
                            ui.add_space(4.0);
                            if Self::action_button(ui, Icons::STOP, "Stop", Theme::ERROR_LIGHT) {
                                response.action = Some(CardAction::Stop);
                            }
                        }
                        InstanceStatus::Stopped | InstanceStatus::Crashed => {
                            if Self::action_button(ui, Icons::PLAY, "Start", Theme::SUCCESS) {
                                response.action = Some(CardAction::Start);
                            }
                        }
                        _ => {}
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Self::action_button(ui, Icons::SETTINGS, "Configure", Theme::TEXT_MUTED)
                        {
                            response.action = Some(CardAction::Configure);
                        }
                    });
                });
            });

        response
    }

    /// Render instance as a list row
    pub fn list(ui: &mut Ui, instance: &Instance) -> CardResponse {
        let mut response = CardResponse::default();

        let status_color = Theme::status_color(&instance.status);
        let is_active = instance.status.is_active();

        let row_response = egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(10.0))
            .stroke(egui::Stroke::new(
                1.0,
                if is_active {
                    status_color.linear_multiply(0.3)
                } else {
                    Theme::BORDER_LIGHT
                },
            ))
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Status dot with glow effect for active
                    StatusBadge::dot(ui, &instance.status);

                    ui.add_space(12.0);

                    // Name and executable in a column
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(instance.display_name())
                                .strong()
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY),
                        );
                        if let Some(path) = instance.config.executable_path.file_name() {
                            ui.label(
                                egui::RichText::new(path.to_string_lossy())
                                    .size(12.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        }
                    });

                    ui.add_space(24.0);

                    // Resource usage badges
                    if is_active {
                        // CPU badge
                        egui::Frame::none()
                            .fill(Theme::BG_TERTIARY.linear_multiply(0.6))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "CPU {:.0}%",
                                        instance.resource_usage.cpu_percent
                                    ))
                                    .size(11.0)
                                    .color(Theme::TEXT_SECONDARY),
                                );
                            });

                        ui.add_space(8.0);

                        // RAM badge
                        egui::Frame::none()
                            .fill(Theme::BG_TERTIARY.linear_multiply(0.6))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(instance.resource_usage.memory_string())
                                        .size(11.0)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });

                        ui.add_space(8.0);

                        // Uptime badge
                        egui::Frame::none()
                            .fill(Theme::BG_TERTIARY.linear_multiply(0.6))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("⏱ {}", instance.uptime_string()))
                                        .size(11.0)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });
                    }

                    // Right-aligned actions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Self::action_button(ui, Icons::SETTINGS, "Configure", Theme::TEXT_MUTED)
                        {
                            response.action = Some(CardAction::Configure);
                        }

                        ui.add_space(6.0);

                        match instance.status {
                            InstanceStatus::Running => {
                                if Self::action_button(ui, Icons::RESTART, "Restart", Theme::INFO) {
                                    response.action = Some(CardAction::Restart);
                                }
                                ui.add_space(4.0);
                                if Self::action_button(ui, Icons::STOP, "Stop", Theme::ERROR_LIGHT)
                                {
                                    response.action = Some(CardAction::Stop);
                                }
                                ui.add_space(4.0);
                                if Self::action_button(ui, Icons::PAUSE, "Pause", Theme::WARNING) {
                                    response.action = Some(CardAction::Pause);
                                }
                            }
                            InstanceStatus::Paused => {
                                if Self::action_button(ui, Icons::STOP, "Stop", Theme::ERROR_LIGHT)
                                {
                                    response.action = Some(CardAction::Stop);
                                }
                                ui.add_space(4.0);
                                if Self::action_button(ui, Icons::PLAY, "Resume", Theme::SUCCESS) {
                                    response.action = Some(CardAction::Resume);
                                }
                            }
                            InstanceStatus::Stopped | InstanceStatus::Crashed => {
                                if Self::action_button(ui, Icons::PLAY, "Start", Theme::SUCCESS) {
                                    response.action = Some(CardAction::Start);
                                }
                            }
                            _ => {}
                        }
                    });
                });
            });

        if row_response.response.clicked() {
            response.action = Some(CardAction::Select);
        }

        response
    }

    /// Render instance as a compact row
    pub fn compact(ui: &mut Ui, instance: &Instance) -> CardResponse {
        let mut response = CardResponse::default();

        let is_active = instance.status.is_active();

        egui::Frame::none()
            .fill(Color32::TRANSPARENT)
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    StatusBadge::dot(ui, &instance.status);
                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new(instance.display_name())
                            .size(13.0)
                            .color(Theme::TEXT_PRIMARY),
                    );

                    if is_active {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.0}%",
                                instance.resource_usage.cpu_percent
                            ))
                            .size(11.0)
                            .color(Theme::TEXT_MUTED),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_active {
                            if Self::action_button(ui, Icons::STOP, "Stop", Theme::ERROR_LIGHT) {
                                response.action = Some(CardAction::Stop);
                            }
                        } else if Self::action_button(ui, Icons::PLAY, "Start", Theme::SUCCESS) {
                            response.action = Some(CardAction::Start);
                        }
                    });
                });
            });

        response
    }
}

/// Response from instance card interaction
#[derive(Default)]
pub struct CardResponse {
    pub action: Option<CardAction>,
}

/// Actions that can be triggered from a card
#[derive(Debug, Clone, Copy)]
pub enum CardAction {
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
    Configure,
    Select,
    Delete,
}
