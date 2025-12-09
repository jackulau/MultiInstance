//! Resource usage bar component

use egui::{Color32, Rect, Response, Rounding, Ui, Vec2};

use crate::ui::theme::Theme;

pub struct ResourceBar;

impl ResourceBar {
    /// Render a horizontal resource bar with label
    pub fn horizontal(
        ui: &mut Ui,
        value: f32, // 0.0 - 1.0
        label: &str,
        width: f32,
        show_percentage: bool,
    ) -> Response {
        let height = 22.0;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let clamped_value = value.clamp(0.0, 1.0);

            // Background with subtle border
            painter.rect_filled(rect, Rounding::same(6.0), Theme::BG_TERTIARY);
            painter.rect_stroke(
                rect,
                Rounding::same(6.0),
                egui::Stroke::new(1.0, Theme::BORDER_LIGHT),
            );

            // Fill with gradient effect
            let fill_width = rect.width() * clamped_value;
            if fill_width > 0.0 {
                let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, height));
                let fill_color = Self::color_for_value(clamped_value);

                // Main fill
                painter.rect_filled(
                    fill_rect,
                    Rounding::same(6.0),
                    fill_color.linear_multiply(0.8),
                );

                // Top highlight for depth
                let highlight_rect =
                    Rect::from_min_size(rect.min, Vec2::new(fill_width, height * 0.4));
                painter.rect_filled(
                    highlight_rect,
                    Rounding {
                        nw: 6.0,
                        ne: 6.0,
                        sw: 0.0,
                        se: 0.0,
                    },
                    fill_color.linear_multiply(1.1),
                );
            }

            // Label with shadow for readability
            let text = if show_percentage {
                format!("{}: {:.0}%", label, clamped_value * 100.0)
            } else {
                label.to_string()
            };

            // Text shadow
            painter.text(
                rect.center() + Vec2::new(1.0, 1.0),
                egui::Align2::CENTER_CENTER,
                &text,
                egui::FontId::proportional(11.0),
                Color32::from_black_alpha(100),
            );

            // Text
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(11.0),
                Theme::TEXT_PRIMARY,
            );
        }

        response
    }

    /// Render a vertical resource bar (for CPU cores, etc.)
    pub fn vertical(ui: &mut Ui, value: f32, height: f32) -> Response {
        let width = 10.0;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let clamped_value = value.clamp(0.0, 1.0);

            // Background
            painter.rect_filled(rect, Rounding::same(3.0), Theme::BG_TERTIARY);

            // Fill from bottom
            let fill_height = rect.height() * clamped_value;
            if fill_height > 0.0 {
                let fill_rect =
                    Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - fill_height), rect.max);

                let fill_color = Self::color_for_value(clamped_value);
                painter.rect_filled(fill_rect, Rounding::same(3.0), fill_color);
            }
        }

        response.on_hover_text(format!("{:.0}%", value * 100.0))
    }

    /// Render a mini inline bar
    pub fn mini(ui: &mut Ui, value: f32) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(48.0, 10.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let clamped_value = value.clamp(0.0, 1.0);

            // Background
            painter.rect_filled(rect, Rounding::same(3.0), Theme::BG_TERTIARY);

            // Fill
            let fill_width = rect.width() * clamped_value;
            if fill_width > 0.0 {
                let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
                let fill_color = Self::color_for_value(clamped_value);
                painter.rect_filled(fill_rect, Rounding::same(3.0), fill_color);
            }
        }

        response.on_hover_text(format!("{:.0}%", value * 100.0))
    }

    /// Render a circular progress indicator
    pub fn circular(ui: &mut Ui, value: f32, size: f32) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let radius = size * 0.4;
            let stroke_width = size * 0.12;
            let clamped_value = value.clamp(0.0, 1.0);

            // Background circle
            painter.circle_stroke(
                center,
                radius,
                egui::Stroke::new(stroke_width, Theme::BG_TERTIARY),
            );

            // Progress arc
            if clamped_value > 0.0 {
                let fill_color = Self::color_for_value(clamped_value);
                let n_points = (32.0 * clamped_value).max(2.0) as usize;
                let start_angle = -std::f32::consts::FRAC_PI_2;
                let end_angle = start_angle + std::f32::consts::TAU * clamped_value;

                let points: Vec<egui::Pos2> = (0..=n_points)
                    .map(|i| {
                        let t = i as f32 / n_points as f32;
                        let angle = start_angle + (end_angle - start_angle) * t;
                        center + Vec2::new(angle.cos(), angle.sin()) * radius
                    })
                    .collect();

                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(stroke_width, fill_color),
                ));
            }

            // Center text
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{:.0}", clamped_value * 100.0),
                egui::FontId::proportional(size * 0.25),
                Theme::TEXT_PRIMARY,
            );
        }

        response.on_hover_text(format!("{:.1}%", value * 100.0))
    }

    /// Get color based on resource usage value with smooth gradient
    fn color_for_value(value: f32) -> Color32 {
        if value < 0.5 {
            // Green zone
            Theme::SUCCESS
        } else if value < 0.75 {
            // Transition to warning
            let t = (value - 0.5) / 0.25;
            Self::lerp_color(Theme::SUCCESS, Theme::WARNING, t)
        } else if value < 0.9 {
            // Warning zone
            Theme::WARNING
        } else {
            // Critical zone
            let t = (value - 0.9) / 0.1;
            Self::lerp_color(Theme::WARNING, Theme::ERROR, t.min(1.0))
        }
    }

    /// Linear interpolation between two colors
    fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        Color32::from_rgba_unmultiplied(
            (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
            (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
            (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
            (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
        )
    }
}
