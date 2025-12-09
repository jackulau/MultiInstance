//! Confirmation dialog

use egui::Context;
use std::sync::Arc;

use crate::ui::dialogs::DialogState;
use crate::ui::theme::Theme;

pub fn render(
    ctx: &Context,
    title: &str,
    message: &str,
    on_confirm: Arc<dyn Fn() + Send + Sync>,
    dialog: &mut DialogState,
) {
    let mut open = true;

    egui::Window::new(title)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(350.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(message);

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("Confirm").color(Theme::ERROR))
                    .clicked()
                {
                    on_confirm();
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
