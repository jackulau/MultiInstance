//! Instances panel - View and manage all instances

use egui::Ui;

use crate::core::{AppState, InstanceId};
use crate::ui::components::instance_card::{CardAction, InstanceCard};
use crate::ui::dialogs::DialogState;
use crate::ui::theme::{Icons, Theme};

pub fn render(
    ui: &mut Ui,
    state: &mut AppState,
    search_query: &str,
    selected_instance: &mut Option<InstanceId>,
    dialog: &mut DialogState,
) {
    let settings = state.settings.read().unwrap();
    let view_mode = settings.view_mode;
    drop(settings);

    // Filter instances based on search - clone to avoid borrow issues
    let filtered_count = {
        let instances = state.instances.read().unwrap();
        instances
            .values()
            .filter(|i| {
                if search_query.is_empty() {
                    true
                } else {
                    let query = search_query.to_lowercase();
                    i.display_name().to_lowercase().contains(&query)
                        || i.config
                            .executable_path
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&query)
                        || i.config
                            .group
                            .as_ref()
                            .map(|g| g.to_lowercase().contains(&query))
                            .unwrap_or(false)
                }
            })
            .count()
    };

    // View mode toggle
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} instances", filtered_count))
                .color(Theme::TEXT_SECONDARY),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut settings = state.settings.write().unwrap();

            if ui
                .selectable_label(
                    settings.view_mode == crate::core::settings::ViewMode::Grid,
                    Icons::GRID,
                )
                .clicked()
            {
                settings.view_mode = crate::core::settings::ViewMode::Grid;
            }
            if ui
                .selectable_label(
                    settings.view_mode == crate::core::settings::ViewMode::List,
                    Icons::LIST,
                )
                .clicked()
            {
                settings.view_mode = crate::core::settings::ViewMode::List;
            }
            if ui
                .selectable_label(
                    settings.view_mode == crate::core::settings::ViewMode::Compact,
                    Icons::COMPACT,
                )
                .clicked()
            {
                settings.view_mode = crate::core::settings::ViewMode::Compact;
            }
        });
    });

    ui.add_space(8.0);

    if filtered_count == 0 {
        egui::Frame::none()
            .fill(Theme::BG_SECONDARY)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(32.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("📦").size(48.0));
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(if search_query.is_empty() {
                            "No instances yet"
                        } else {
                            "No instances match your search"
                        })
                        .size(16.0)
                        .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    if search_query.is_empty() {
                        ui.label(
                            egui::RichText::new("Click '+ New Instance' to create one")
                                .color(Theme::TEXT_MUTED),
                        );
                    }
                });
            });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| match view_mode {
        crate::core::settings::ViewMode::Grid => {
            render_grid_view(ui, state, selected_instance, dialog);
        }
        crate::core::settings::ViewMode::List => {
            render_list_view(ui, state, selected_instance, dialog);
        }
        crate::core::settings::ViewMode::Compact => {
            render_compact_view(ui, state, selected_instance, dialog);
        }
    });
}

fn render_grid_view(
    ui: &mut Ui,
    state: &mut AppState,
    selected_instance: &mut Option<InstanceId>,
    dialog: &mut DialogState,
) {
    ui.horizontal_wrapped(|ui| {
        let instances = state.instances.read().unwrap();
        let ids: Vec<_> = instances.keys().copied().collect();
        drop(instances);

        for id in ids {
            let instances = state.instances.read().unwrap();
            if let Some(instance) = instances.get(&id) {
                let instance = instance.clone();
                drop(instances);

                let response = InstanceCard::grid(ui, &instance);
                handle_card_action(response.action, id, state, selected_instance, dialog);
            }
        }
    });
}

fn render_list_view(
    ui: &mut Ui,
    state: &mut AppState,
    selected_instance: &mut Option<InstanceId>,
    dialog: &mut DialogState,
) {
    let instances = state.instances.read().unwrap();
    let ids: Vec<_> = instances.keys().copied().collect();
    drop(instances);

    for id in ids {
        let instances = state.instances.read().unwrap();
        if let Some(instance) = instances.get(&id) {
            let instance = instance.clone();
            drop(instances);

            let response = InstanceCard::list(ui, &instance);
            handle_card_action(response.action, id, state, selected_instance, dialog);

            ui.add_space(4.0);
        }
    }
}

fn render_compact_view(
    ui: &mut Ui,
    state: &mut AppState,
    selected_instance: &mut Option<InstanceId>,
    dialog: &mut DialogState,
) {
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            let instances = state.instances.read().unwrap();
            let ids: Vec<_> = instances.keys().copied().collect();
            drop(instances);

            for id in ids {
                let instances = state.instances.read().unwrap();
                if let Some(instance) = instances.get(&id) {
                    let instance = instance.clone();
                    drop(instances);

                    let response = InstanceCard::compact(ui, &instance);
                    handle_card_action(response.action, id, state, selected_instance, dialog);

                    ui.separator();
                }
            }
        });
}

fn handle_card_action(
    action: Option<CardAction>,
    id: InstanceId,
    state: &mut AppState,
    selected_instance: &mut Option<InstanceId>,
    dialog: &mut DialogState,
) {
    if let Some(action) = action {
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
            CardAction::Configure => {
                *dialog = DialogState::EditInstance(id);
            }
            CardAction::Select => {
                *selected_instance = Some(id);
                *dialog = DialogState::InstanceDetails(id);
            }
            CardAction::Delete => {
                // Would show confirmation dialog
                *dialog = DialogState::Confirm {
                    title: "Delete Instance".to_string(),
                    message: "Are you sure you want to delete this instance?".to_string(),
                    on_confirm: std::sync::Arc::new({
                        let state = state.clone();
                        move || {
                            if let Err(e) = state.remove_instance(id, true) {
                                tracing::error!("Failed to delete instance: {}", e);
                            }
                        }
                    }),
                };
            }
        }
    }
}
