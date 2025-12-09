//! New profile dialog

use egui::Context;

use crate::core::{AppState, Profile};
use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

pub fn render(ctx: &Context, state: &mut AppState, dialog: &mut DialogState) {
    let mut profile = Profile::new("");
    let mut open = true;

    egui::Window::new("New Profile")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(450.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
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
            ui.label(
                egui::RichText::new("Launch instances one by one with a delay")
                    .small()
                    .color(Theme::TEXT_MUTED),
            );

            if profile.staggered_launch {
                ui.horizontal(|ui| {
                    ui.label("Delay (ms):");
                    let mut delay = profile.launch_delay_ms as i32;
                    ui.add(egui::DragValue::new(&mut delay).range(0..=60000));
                    profile.launch_delay_ms = delay as u32;
                });
            }

            ui.add_space(16.0);

            // Note about instances
            egui::Frame::none()
                .fill(Theme::BG_TERTIARY)
                .rounding(egui::Rounding::same(4.0))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "After creating the profile, you can add instances to it from the Instances view.",
                        )
                        .small()
                        .color(Theme::TEXT_SECONDARY),
                    );
                });

            ui.add_space(16.0);

            // Action buttons
            ui.horizontal(|ui| {
                let can_create = !profile.name.is_empty();

                if ui
                    .add_enabled(can_create, egui::Button::new("Create"))
                    .clicked()
                {
                    if let Err(e) = state.save_profile(profile) {
                        tracing::error!("Failed to create profile: {}", e);
                    }
                    *dialog = DialogState::None;
                }

                if ui.button("Cancel").clicked() {
                    *dialog = DialogState::None;
                }
            });
        });

    if !open {
        *dialog = DialogState::None;
    }
}
