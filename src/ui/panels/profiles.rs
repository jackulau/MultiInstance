//! Profiles panel - View and manage profiles

use egui::Ui;

use crate::core::AppState;
use crate::ui::components::profile_card::{ProfileAction, ProfileCard};
use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

pub fn render(ui: &mut Ui, state: &mut AppState, search_query: &str, dialog: &mut DialogState) {
    // Filter profiles based on search - clone to avoid borrow issues
    let (filtered_count, favorites_count) = {
        let profiles = state.profiles.read().unwrap();
        let filtered: Vec<_> = profiles
            .values()
            .filter(|p| {
                if search_query.is_empty() {
                    true
                } else {
                    let query = search_query.to_lowercase();
                    p.name.to_lowercase().contains(&query)
                        || p.description.to_lowercase().contains(&query)
                        || p.category
                            .as_ref()
                            .map(|c| c.to_lowercase().contains(&query))
                            .unwrap_or(false)
                }
            })
            .collect();
        let favorites_count = filtered.iter().filter(|p| p.is_favorite).count();
        (filtered.len(), favorites_count)
    };

    // Header with create button
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} profiles", filtered_count))
                .color(Theme::TEXT_SECONDARY),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ New Profile").clicked() {
                *dialog = DialogState::NewProfile;
            }
        });
    });

    ui.add_space(8.0);

    if filtered_count == 0 {
        render_empty_state(ui, search_query.is_empty(), dialog);
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Favorites section
        if favorites_count > 0 {
            ui.label(egui::RichText::new("★ Favorites").strong());
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                let profiles = state.profiles.read().unwrap();
                let favorite_ids: Vec<_> = profiles
                    .iter()
                    .filter(|(_, p)| p.is_favorite)
                    .map(|(id, _)| *id)
                    .collect();
                drop(profiles);

                for id in favorite_ids {
                    let profiles = state.profiles.read().unwrap();
                    if let Some(profile) = profiles.get(&id) {
                        let profile = profile.clone();
                        drop(profiles);

                        let response = ProfileCard::show(ui, &profile);
                        handle_profile_action(response.action, profile.id, state, dialog);
                    }
                }
            });

            ui.add_space(16.0);
        }

        // All profiles
        ui.label(egui::RichText::new("All Profiles").strong());
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            let profiles = state.profiles.read().unwrap();
            let ids: Vec<_> = profiles.keys().copied().collect();
            drop(profiles);

            for id in ids {
                let profiles = state.profiles.read().unwrap();
                if let Some(profile) = profiles.get(&id) {
                    let profile = profile.clone();
                    drop(profiles);

                    let response = ProfileCard::show(ui, &profile);
                    handle_profile_action(response.action, profile.id, state, dialog);
                }
            }
        });
    });
}

fn render_empty_state(ui: &mut Ui, no_profiles: bool, dialog: &mut DialogState) {
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(32.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("📋").size(48.0));
                ui.add_space(16.0);

                if no_profiles {
                    ui.label(
                        egui::RichText::new("No profiles yet")
                            .size(16.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(
                            "Create a profile to save your instance configurations",
                        )
                        .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(16.0);

                    if ui.button("+ Create Profile").clicked() {
                        *dialog = DialogState::NewProfile;
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No profiles match your search")
                            .size(16.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                }
            });
        });
}

fn handle_profile_action(
    action: Option<ProfileAction>,
    profile_id: crate::core::ProfileId,
    state: &mut AppState,
    dialog: &mut DialogState,
) {
    if let Some(action) = action {
        match action {
            ProfileAction::Launch => {
                if let Err(e) = state.launch_profile(profile_id) {
                    tracing::error!("Failed to launch profile: {}", e);
                }
            }
            ProfileAction::Edit => {
                *dialog = DialogState::EditProfile(profile_id);
            }
            ProfileAction::Delete => {
                *dialog = DialogState::Confirm {
                    title: "Delete Profile".to_string(),
                    message: "Are you sure you want to delete this profile?".to_string(),
                    on_confirm: std::sync::Arc::new({
                        let state = state.clone();
                        move || {
                            if let Err(e) = state.delete_profile(profile_id) {
                                tracing::error!("Failed to delete profile: {}", e);
                            }
                        }
                    }),
                };
            }
            ProfileAction::Export => {
                let profiles = state.profiles.read().unwrap();
                if let Some(profile) = profiles.get(&profile_id) {
                    if let Ok(json) = profile.to_json() {
                        // In a real implementation, we'd open a file save dialog
                        tracing::info!("Profile exported: {}", json);
                    }
                }
            }
            ProfileAction::ToggleFavorite => {
                let mut profiles = state.profiles.write().unwrap();
                if let Some(profile) = profiles.get_mut(&profile_id) {
                    profile.toggle_favorite();
                    let profile = profile.clone();
                    drop(profiles);
                    if let Err(e) = state.database.save_profile(&profile) {
                        tracing::error!("Failed to save profile: {}", e);
                    }
                }
            }
        }
    }
}
