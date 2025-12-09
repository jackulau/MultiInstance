//! Profile card component

use egui::Ui;

use crate::core::Profile;
use crate::ui::theme::{Icons, Theme};

pub struct ProfileCard;

impl ProfileCard {
    /// Render profile as a card
    pub fn show(ui: &mut Ui, profile: &Profile) -> ProfileCardResponse {
        let mut response = ProfileCardResponse::default();

        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(8.0))
            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(250.0);

                // Header: Name and favorite
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&profile.name).strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let star_icon = if profile.is_favorite {
                            Icons::STAR
                        } else {
                            Icons::STAR_EMPTY
                        };
                        if ui
                            .button(egui::RichText::new(star_icon).color(Theme::WARNING))
                            .clicked()
                        {
                            response.action = Some(ProfileAction::ToggleFavorite);
                        }
                    });
                });

                ui.add_space(4.0);

                // Description
                if !profile.description.is_empty() {
                    ui.label(
                        egui::RichText::new(&profile.description)
                            .small()
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);
                }

                // Instance count
                ui.label(
                    egui::RichText::new(format!("{} instances", profile.instance_count()))
                        .small()
                        .color(Theme::TEXT_MUTED),
                );

                // Category/tags
                if let Some(ref category) = profile.category {
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(Theme::PRIMARY.linear_multiply(0.2))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(category)
                                        .small()
                                        .color(Theme::PRIMARY_LIGHT),
                                );
                            });
                    });
                }

                ui.add_space(8.0);

                // Launch stats
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Launched {} times", profile.launch_count))
                            .small()
                            .color(Theme::TEXT_MUTED),
                    );
                });

                ui.add_space(8.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button(format!("{} Launch", Icons::PLAY)).clicked() {
                        response.action = Some(ProfileAction::Launch);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(Icons::TRASH).clicked() {
                            response.action = Some(ProfileAction::Delete);
                        }
                        if ui.small_button(Icons::EDIT).clicked() {
                            response.action = Some(ProfileAction::Edit);
                        }
                        if ui.small_button(Icons::EXPORT).clicked() {
                            response.action = Some(ProfileAction::Export);
                        }
                    });
                });
            });

        response
    }

    /// Render profile as a list row
    pub fn list_row(ui: &mut Ui, profile: &Profile) -> ProfileCardResponse {
        let mut response = ProfileCardResponse::default();

        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Favorite star
                    let star_color = if profile.is_favorite {
                        Theme::WARNING
                    } else {
                        Theme::TEXT_MUTED
                    };
                    let star_icon = if profile.is_favorite {
                        Icons::STAR
                    } else {
                        Icons::STAR_EMPTY
                    };
                    if ui
                        .button(egui::RichText::new(star_icon).color(star_color))
                        .clicked()
                    {
                        response.action = Some(ProfileAction::ToggleFavorite);
                    }

                    ui.add_space(8.0);

                    // Name
                    ui.label(egui::RichText::new(&profile.name).strong());

                    ui.add_space(16.0);

                    // Instance count
                    ui.label(
                        egui::RichText::new(format!("{} instances", profile.instance_count()))
                            .color(Theme::TEXT_SECONDARY),
                    );

                    // Category
                    if let Some(ref category) = profile.category {
                        ui.add_space(8.0);
                        egui::Frame::none()
                            .fill(Theme::PRIMARY.linear_multiply(0.2))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(category)
                                        .small()
                                        .color(Theme::PRIMARY_LIGHT),
                                );
                            });
                    }

                    // Right-aligned actions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(Icons::TRASH).clicked() {
                            response.action = Some(ProfileAction::Delete);
                        }
                        if ui.small_button(Icons::EDIT).clicked() {
                            response.action = Some(ProfileAction::Edit);
                        }
                        if ui.button(format!("{} Launch", Icons::PLAY)).clicked() {
                            response.action = Some(ProfileAction::Launch);
                        }
                    });
                });
            });

        response
    }
}

/// Response from profile card interaction
#[derive(Default)]
pub struct ProfileCardResponse {
    pub action: Option<ProfileAction>,
}

/// Actions that can be triggered from a profile card
#[derive(Debug, Clone, Copy)]
pub enum ProfileAction {
    Launch,
    Edit,
    Delete,
    Export,
    ToggleFavorite,
}
