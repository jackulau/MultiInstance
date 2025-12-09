//! Main application UI

use std::time::{Duration, Instant};

use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use tracing::{error, info};

use super::dialogs::{self, DialogState};
use super::panels;
use super::theme::Theme;
use crate::core::{AppState, InstanceConfig, InstanceId};

/// Active view/tab in the main panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    #[default]
    Dashboard,
    Instances,
    Profiles,
    Settings,
    History,
}

impl ActiveView {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Instances => "Instances",
            Self::Profiles => "Profiles",
            Self::Settings => "Settings",
            Self::History => "History",
        }
    }
}

/// Main application struct
pub struct MultiInstanceApp {
    /// Application state
    state: AppState,
    /// Current active view
    active_view: ActiveView,
    /// Dialog state
    dialog: DialogState,
    /// Search filter text
    search_query: String,
    /// Selected instance for details panel
    selected_instance: Option<InstanceId>,
    /// Last resource update time
    last_update: Instant,
    /// Update interval
    update_interval: Duration,
    /// Show system resources panel
    show_system_resources: bool,
    /// Notifications queue
    notifications: Vec<Notification>,
    /// New instance config being edited
    new_instance_config: Option<InstanceConfig>,
    /// First frame flag
    first_frame: bool,
}

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl MultiInstanceApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        // Apply theme
        let settings = state.settings.read().unwrap();
        match settings.theme {
            crate::core::settings::Theme::Dark => Theme::apply_dark(&cc.egui_ctx),
            crate::core::settings::Theme::Light => Theme::apply_light(&cc.egui_ctx),
            crate::core::settings::Theme::System => {
                // Default to dark for now
                Theme::apply_dark(&cc.egui_ctx);
            }
        }
        let show_system_resources = settings.show_system_resources;
        let update_interval = Duration::from_millis(settings.monitor_interval_ms as u64);
        drop(settings);

        Self {
            state,
            active_view: ActiveView::Dashboard,
            dialog: DialogState::None,
            search_query: String::new(),
            selected_instance: None,
            last_update: Instant::now(),
            update_interval,
            show_system_resources,
            notifications: Vec::new(),
            new_instance_config: None,
            first_frame: true,
        }
    }

    /// Add a notification
    pub fn notify(&mut self, message: impl Into<String>, level: NotificationLevel) {
        self.notifications.push(Notification {
            message: message.into(),
            level,
            created_at: Instant::now(),
        });
    }

    /// Update resources if needed
    fn update_resources(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.state.update_resources();
            self.state.handle_auto_restarts();
            self.last_update = now;
        }
    }

    /// Clean up old notifications
    fn cleanup_notifications(&mut self) {
        let timeout = Duration::from_secs(5);
        self.notifications
            .retain(|n| n.created_at.elapsed() < timeout);
    }

    /// Render the sidebar navigation
    fn render_sidebar(&mut self, ctx: &Context) {
        SidePanel::left("sidebar")
            .resizable(false)
            .default_width(220.0)
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_SECONDARY)
                    .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT)),
            )
            .show(ctx, |ui| {
                ui.add_space(20.0);

                // Logo/Title with icon
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("◈").size(24.0).color(Theme::PRIMARY));
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("MultiInstance")
                            .size(18.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                });

                ui.add_space(24.0);

                // Navigation items with custom styling
                let views = [
                    (ActiveView::Dashboard, "◉", "Dashboard"),
                    (ActiveView::Instances, "▣", "Instances"),
                    (ActiveView::Profiles, "▤", "Profiles"),
                    (ActiveView::Settings, "⚙", "Settings"),
                    (ActiveView::History, "◷", "History"),
                ];

                ui.add_space(4.0);
                for (view, icon, label) in views {
                    let selected = self.active_view == view;

                    let bg_color = if selected {
                        Theme::PRIMARY.linear_multiply(0.15)
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    let text_color = if selected {
                        Theme::PRIMARY_LIGHT
                    } else {
                        Theme::TEXT_SECONDARY
                    };

                    let frame = egui::Frame::none()
                        .fill(bg_color)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 12.0));

                    let response = frame.show(ui, |ui| {
                        ui.set_width(ui.available_width() - 16.0);
                        ui.horizontal(|ui| {
                            if selected {
                                // Active indicator bar
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(3.0, 18.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    egui::Rounding::same(2.0),
                                    Theme::PRIMARY,
                                );
                                ui.add_space(8.0);
                            }
                            ui.label(egui::RichText::new(icon).size(16.0).color(text_color));
                            ui.add_space(12.0);
                            ui.label(egui::RichText::new(label).size(14.0).color(text_color));
                        });
                    });

                    if response.response.interact(egui::Sense::click()).clicked() {
                        self.active_view = view;
                    }

                    if response.response.interact(egui::Sense::hover()).hovered() && !selected {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    ui.add_space(2.0);
                }

                ui.add_space(20.0);

                // Divider line
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width() - 32.0, 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 0.0, Theme::BORDER_LIGHT);
                });

                ui.add_space(16.0);

                // Quick stats section
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("QUICK STATS")
                            .small()
                            .color(Theme::TEXT_MUTED),
                    );
                });
                ui.add_space(12.0);

                let active = self.state.active_instance_count();
                let total = self.state.total_instance_count();
                let profiles = self.state.profile_count();

                // Stats cards
                egui::Frame::none()
                    .fill(Theme::BG_TERTIARY.linear_multiply(0.5))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .outer_margin(egui::Margin::symmetric(16.0, 0.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}", active))
                                        .size(20.0)
                                        .strong()
                                        .color(Theme::SUCCESS),
                                );
                                ui.label(
                                    egui::RichText::new("Running")
                                        .small()
                                        .color(Theme::TEXT_MUTED),
                                );
                            });
                            ui.add_space(24.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}", total))
                                        .size(20.0)
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    egui::RichText::new("Total")
                                        .small()
                                        .color(Theme::TEXT_MUTED),
                                );
                            });
                            ui.add_space(24.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}", profiles))
                                        .size(20.0)
                                        .strong()
                                        .color(Theme::INFO),
                                );
                                ui.label(
                                    egui::RichText::new("Profiles")
                                        .small()
                                        .color(Theme::TEXT_MUTED),
                                );
                            });
                        });
                    });

                // Fill remaining space and show version at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(format!("v{}", crate::APP_VERSION))
                                .small()
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                    ui.add_space(8.0);
                });
            });
    }

    /// Render the top bar with actions
    fn render_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_PRIMARY)
                    .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // View title
                    ui.label(
                        egui::RichText::new(self.active_view.label())
                            .size(24.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );

                    ui.add_space(24.0);

                    // Search box (for instances/profiles views)
                    if matches!(
                        self.active_view,
                        ActiveView::Instances | ActiveView::Profiles
                    ) {
                        egui::Frame::none()
                            .fill(Theme::BG_SECONDARY)
                            .rounding(egui::Rounding::same(8.0))
                            .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("⌕")
                                            .size(14.0)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                    ui.add_space(8.0);
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.search_query)
                                            .hint_text("Search instances...")
                                            .desired_width(180.0)
                                            .frame(false),
                                    );
                                });
                            });
                    }

                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // New instance button (primary action)
                        let new_btn = egui::Button::new(
                            egui::RichText::new("+ New Instance").color(egui::Color32::WHITE),
                        )
                        .fill(Theme::PRIMARY)
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(130.0, 36.0));

                        if ui.add(new_btn).clicked() {
                            self.dialog = DialogState::NewInstance;
                            self.new_instance_config = Some(InstanceConfig::default());
                        }

                        ui.add_space(12.0);

                        // Quick actions (secondary buttons)
                        if self.state.active_instance_count() > 0 {
                            let pause_btn = egui::Button::new(
                                egui::RichText::new("⏸ Pause All").color(Theme::TEXT_PRIMARY),
                            )
                            .fill(Theme::BG_TERTIARY)
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(100.0, 36.0));

                            if ui.add(pause_btn).clicked() {
                                if let Err(e) = self.state.pause_all() {
                                    self.notify(
                                        format!("Failed to pause: {}", e),
                                        NotificationLevel::Error,
                                    );
                                }
                            }

                            ui.add_space(8.0);

                            let stop_btn = egui::Button::new(
                                egui::RichText::new("⏹ Stop All").color(Theme::TEXT_PRIMARY),
                            )
                            .fill(Theme::BG_TERTIARY)
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(100.0, 36.0));

                            if ui.add(stop_btn).clicked() {
                                if let Err(e) = self.state.stop_all() {
                                    self.notify(
                                        format!("Failed to stop: {}", e),
                                        NotificationLevel::Error,
                                    );
                                }
                            }
                        }
                    });
                });
            });
    }

    /// Render the main content area
    fn render_main_content(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| match self.active_view {
            ActiveView::Dashboard => {
                panels::dashboard::render(ui, &mut self.state, self.show_system_resources);
            }
            ActiveView::Instances => {
                panels::instances::render(
                    ui,
                    &mut self.state,
                    &self.search_query,
                    &mut self.selected_instance,
                    &mut self.dialog,
                );
            }
            ActiveView::Profiles => {
                panels::profiles::render(ui, &mut self.state, &self.search_query, &mut self.dialog);
            }
            ActiveView::Settings => {
                panels::settings::render(ui, &mut self.state, ctx);
            }
            ActiveView::History => {
                panels::history::render(ui, &self.state);
            }
        });
    }

    /// Render notifications
    fn render_notifications(&mut self, ctx: &Context) {
        if self.notifications.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("notifications"))
            .fixed_pos(egui::pos2(ctx.screen_rect().width() - 360.0, 80.0))
            .show(ctx, |ui| {
                for notification in &self.notifications {
                    let (bg_color, icon, border_color) = match notification.level {
                        NotificationLevel::Info => (Theme::BG_ELEVATED, "ℹ", Theme::INFO),
                        NotificationLevel::Success => (Theme::BG_ELEVATED, "✓", Theme::SUCCESS),
                        NotificationLevel::Warning => (Theme::BG_ELEVATED, "⚠", Theme::WARNING),
                        NotificationLevel::Error => (Theme::BG_ELEVATED, "✕", Theme::ERROR),
                    };

                    egui::Frame::none()
                        .fill(bg_color)
                        .rounding(egui::Rounding::same(10.0))
                        .stroke(egui::Stroke::new(1.0, border_color.linear_multiply(0.5)))
                        .shadow(egui::Shadow {
                            offset: egui::vec2(0.0, 4.0),
                            blur: 12.0,
                            spread: 2.0,
                            color: egui::Color32::from_black_alpha(60),
                        })
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_width(320.0);
                            ui.horizontal(|ui| {
                                // Icon with colored background
                                egui::Frame::none()
                                    .fill(border_color.linear_multiply(0.2))
                                    .rounding(egui::Rounding::same(6.0))
                                    .inner_margin(egui::Margin::same(6.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(icon)
                                                .size(14.0)
                                                .color(border_color),
                                        );
                                    });
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&notification.message)
                                            .size(13.0)
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                });
                            });
                        });

                    ui.add_space(10.0);
                }
            });
    }

    /// Render dialogs
    fn render_dialogs(&mut self, ctx: &Context) {
        match &self.dialog {
            DialogState::None => {}
            DialogState::NewInstance => {
                dialogs::new_instance::render(
                    ctx,
                    &mut self.new_instance_config,
                    &mut self.state,
                    &mut self.dialog,
                    &mut self.notifications,
                );
            }
            DialogState::EditInstance(id) => {
                let id = *id;
                dialogs::edit_instance::render(ctx, id, &mut self.state, &mut self.dialog);
            }
            DialogState::NewProfile => {
                dialogs::new_profile::render(ctx, &mut self.state, &mut self.dialog);
            }
            DialogState::EditProfile(id) => {
                let id = *id;
                dialogs::edit_profile::render(ctx, id, &mut self.state, &mut self.dialog);
            }
            DialogState::Confirm {
                title,
                message,
                on_confirm,
            } => {
                let title = title.clone();
                let message = message.clone();
                let on_confirm = on_confirm.clone();
                dialogs::confirm::render(ctx, &title, &message, on_confirm, &mut self.dialog);
            }
            DialogState::InstanceDetails(id) => {
                let id = *id;
                dialogs::instance_details::render(ctx, id, &mut self.state, &mut self.dialog);
            }
        }
    }
}

impl eframe::App for MultiInstanceApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // First frame setup
        if self.first_frame {
            self.first_frame = false;
            info!("First frame rendered");
        }

        // Update resources periodically
        self.update_resources();

        // Clean up old notifications
        self.cleanup_notifications();

        // Request repaint for animations
        ctx.request_repaint_after(Duration::from_millis(100));

        // Render UI components
        self.render_sidebar(ctx);
        self.render_top_bar(ctx);
        self.render_main_content(ctx);
        self.render_notifications(ctx);
        self.render_dialogs(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save session on exit
        if let Err(e) = self.state.save_session() {
            error!("Failed to save session: {}", e);
        }

        // Save settings
        if let Err(e) = self.state.save_settings() {
            error!("Failed to save settings: {}", e);
        }

        info!("Application exiting");
    }
}
