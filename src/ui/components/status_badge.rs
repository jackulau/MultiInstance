//! Status badge component

use egui::{Color32, Response, Rounding, Ui, Vec2};

use crate::core::InstanceStatus;
use crate::ui::theme::Theme;

pub struct StatusBadge;

impl StatusBadge {
    /// Render a full status badge with text
    pub fn show(ui: &mut Ui, status: &InstanceStatus) -> Response {
        let color = Theme::status_color(status);
        let label = status.label();

        let (rect, response) = ui.allocate_exact_size(Vec2::new(90.0, 26.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background with subtle gradient effect
            painter.rect_filled(rect, Rounding::same(13.0), color.linear_multiply(0.15));
            painter.rect_stroke(
                rect,
                Rounding::same(13.0),
                egui::Stroke::new(1.0, color.linear_multiply(0.3)),
            );

            // Animated dot indicator with glow
            let dot_center = rect.left_center() + Vec2::new(14.0, 0.0);

            // Outer glow for active statuses
            if status.is_active() {
                painter.circle_filled(dot_center, 6.0, color.linear_multiply(0.3));
            }

            // Inner dot
            painter.circle_filled(dot_center, 4.0, color);

            // Text
            painter.text(
                rect.center() + Vec2::new(8.0, 0.0),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(12.0),
                color,
            );
        }

        response
    }

    /// Render a small dot indicator with glow effect for active states
    pub fn dot(ui: &mut Ui, status: &InstanceStatus) -> Response {
        let color = Theme::status_color(status);
        let is_active = status.is_active();

        let size = if is_active { 16.0 } else { 14.0 };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();

            if is_active {
                // Outer glow ring for active statuses
                painter.circle_filled(center, 7.0, color.linear_multiply(0.25));
                painter.circle_stroke(
                    center,
                    6.0,
                    egui::Stroke::new(1.0, color.linear_multiply(0.4)),
                );
            }

            // Main dot
            painter.circle_filled(center, 5.0, color);

            // Inner highlight
            painter.circle_filled(
                center + Vec2::new(-1.0, -1.0),
                2.0,
                Color32::from_white_alpha(40),
            );
        }

        response.on_hover_text(status.label())
    }

    /// Render an inline status indicator with text
    pub fn inline(ui: &mut Ui, status: &InstanceStatus) -> Response {
        let color = Theme::status_color(status);

        let response = ui.horizontal(|ui| {
            // Small dot
            let (rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().circle_filled(rect.center(), 3.5, color);
            }

            ui.add_space(6.0);

            ui.label(egui::RichText::new(status.label()).size(12.0).color(color));
        });

        response.response
    }
}
