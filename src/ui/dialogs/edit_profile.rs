//! Edit profile dialog

use egui::Context;

use crate::core::{AppState, ProfileId};
use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

pub fn render(ctx: &Context, id: ProfileId, state: &mut AppState, dialog: &mut DialogState) {
    let profiles = state.profiles.read().unwrap();
    let Some(profile) = profiles.get(&id).cloned() else {
        *dialog = DialogState::None;
        return;
    };
    drop(profiles);

    let mut profile = profile;
    let mut open = true;

    egui::Window::new(format!("Edit Profile: {}", profile.name))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(550.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Name
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut profile.name);
                });

                ui.add_space(8.0);

                // Description
                ui.label("Description:");
                ui.text_edit_multiline(&mut profile.description);

                ui.add_space(8.0);

                // Category
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    let mut category = profile.category.clone().unwrap_or_default();
                    ui.text_edit_singleline(&mut category);
                    profile.category = if category.is_empty() {
                        None
                    } else {
                        Some(category)
                    };
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Launch options
                ui.label(egui::RichText::new("Launch Options").strong());
                ui.add_space(8.0);

                ui.checkbox(&mut profile.staggered_launch, "Staggered launch");

                if profile.staggered_launch {
                    ui.horizontal(|ui| {
                        ui.label("Delay (ms):");
                        let mut delay = profile.launch_delay_ms as i32;
                        ui.add(egui::DragValue::new(&mut delay).range(0..=60000));
                        profile.launch_delay_ms = delay as u32;
                    });
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Instances in profile
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Instances").strong());
                    ui.label(
                        egui::RichText::new(format!("({})", profile.instances.len()))
                            .color(Theme::TEXT_MUTED),
                    );
                });
                ui.add_space(8.0);

                if profile.instances.is_empty() {
                    egui::Frame::none()
                        .fill(Theme::BG_TERTIARY)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("No instances in this profile")
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                } else {
                    let mut to_remove = None;

                    for (idx, config) in profile.instances.iter().enumerate() {
                        egui::Frame::none()
                            .fill(Theme::BG_TERTIARY)
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&config.name).strong());
                                    ui.label(
                                        egui::RichText::new(
                                            config
                                                .executable_path
                                                .file_name()
                                                .map(|s| s.to_string_lossy().to_string())
                                                .unwrap_or_default(),
                                        )
                                        .small()
                                        .color(Theme::TEXT_SECONDARY),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.small_button("✕").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(4.0);
                    }

                    if let Some(idx) = to_remove {
                        profile.remove_instance(idx);
                    }
                }

                ui.add_space(8.0);

                // Add instance from existing
                egui::CollapsingHeader::new("Add Existing Instance")
                    .default_open(false)
                    .show(ui, |ui| {
                        let instances = state.instances.read().unwrap();
                        for instance in instances.values() {
                            ui.horizontal(|ui| {
                                ui.label(&instance.config.name);
                                if ui.small_button("+").clicked() {
                                    profile.add_instance(instance.config.clone());
                                }
                            });
                        }
                    });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Stats
                ui.label(
                    egui::RichText::new(format!("Launched {} times", profile.launch_count))
                        .small()
                        .color(Theme::TEXT_MUTED),
                );
                if let Some(last_used) = profile.last_used_at {
                    ui.label(
                        egui::RichText::new(format!(
                            "Last used: {}",
                            last_used.format("%Y-%m-%d %H:%M")
                        ))
                        .small()
                        .color(Theme::TEXT_MUTED),
                    );
                }

                ui.add_space(16.0);

                // Action buttons
                let profile_name = profile.name.clone();
                let mut should_save = false;
                let mut should_delete = false;

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        should_save = true;
                    }

                    if ui.button("Cancel").clicked() {
                        *dialog = DialogState::None;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(Theme::ERROR))
                            .clicked()
                        {
                            should_delete = true;
                        }
                    });
                });

                if should_save {
                    profile.mark_modified();
                    if let Err(e) = state.save_profile(profile) {
                        tracing::error!("Failed to save profile: {}", e);
                    }
                    *dialog = DialogState::None;
                } else if should_delete {
                    *dialog = DialogState::Confirm {
                        title: "Delete Profile".to_string(),
                        message: format!("Are you sure you want to delete '{}'?", profile_name),
                        on_confirm: std::sync::Arc::new({
                            let state = state.clone();
                            move || {
                                let _ = state.delete_profile(id);
                            }
                        }),
                    };
                }
            });
        });

    if !open {
        *dialog = DialogState::None;
    }
}
