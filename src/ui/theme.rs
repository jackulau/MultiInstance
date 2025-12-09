//! Theme and styling for the UI

use egui::{Color32, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};

/// Application color palette
pub struct Theme;

impl Theme {
    // Primary colors - refined indigo/violet accent
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Indigo-500
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(79, 70, 229); // Indigo-600
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(165, 180, 252); // Indigo-300
    pub const PRIMARY_DARK: Color32 = Color32::from_rgb(67, 56, 202); // Indigo-700

    // Status colors - balanced and harmonious
    pub const SUCCESS: Color32 = Color32::from_rgb(16, 185, 129); // Emerald-500
    pub const SUCCESS_LIGHT: Color32 = Color32::from_rgb(52, 211, 153); // Emerald-400
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amber-500
    pub const WARNING_LIGHT: Color32 = Color32::from_rgb(251, 191, 36); // Amber-400
    pub const ERROR: Color32 = Color32::from_rgb(244, 63, 94); // Rose-500
    pub const ERROR_LIGHT: Color32 = Color32::from_rgb(251, 113, 133); // Rose-400
    pub const INFO: Color32 = Color32::from_rgb(6, 182, 212); // Cyan-500

    // Neutral colors (dark theme) - modern charcoal palette
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(17, 17, 27); // Deep charcoal
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(24, 24, 37); // Card background
    pub const BG_TERTIARY: Color32 = Color32::from_rgb(35, 35, 52); // Elevated elements
    pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 65); // Hover state
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(30, 30, 45); // Modals/dropdowns

    // Text colors - crisp contrast
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 255); // Near white
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 180); // Gray-400
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 132); // Gray-500

    // Border colors - subtle definition
    pub const BORDER: Color32 = Color32::from_rgb(50, 50, 70); // Subtle border
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(38, 38, 55); // Lighter border
    pub const BORDER_ACCENT: Color32 = Color32::from_rgb(99, 102, 241); // Primary accent

    // Instance status colors
    pub const STATUS_RUNNING: Color32 = Self::SUCCESS;
    pub const STATUS_STARTING: Color32 = Self::WARNING;
    pub const STATUS_PAUSED: Color32 = Self::INFO;
    pub const STATUS_STOPPED: Color32 = Self::TEXT_MUTED;
    pub const STATUS_CRASHED: Color32 = Self::ERROR;

    /// Apply dark theme to egui
    pub fn apply_dark(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Set up visuals
        let mut visuals = Visuals::dark();

        visuals.panel_fill = Self::BG_PRIMARY;
        visuals.window_fill = Self::BG_ELEVATED;
        visuals.extreme_bg_color = Self::BG_PRIMARY;
        visuals.faint_bg_color = Self::BG_TERTIARY;

        // Non-interactive widgets (labels, etc.)
        visuals.widgets.noninteractive.bg_fill = Self::BG_SECONDARY;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Self::BORDER_LIGHT);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        // Inactive interactive widgets (buttons at rest)
        visuals.widgets.inactive.bg_fill = Self::BG_TERTIARY;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);
        visuals.widgets.inactive.bg_stroke = Stroke::new(0.5, Self::BORDER);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);

        // Hovered widgets - smooth visual feedback
        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::PRIMARY.linear_multiply(0.6));
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.expansion = 1.0;

        // Active/pressed widgets
        visuals.widgets.active.bg_fill = Self::PRIMARY;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::PRIMARY_DARK);
        visuals.widgets.active.rounding = Rounding::same(6.0);

        // Open widgets (like ComboBox when open)
        visuals.widgets.open.bg_fill = Self::BG_ELEVATED;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::PRIMARY.linear_multiply(0.5));
        visuals.widgets.open.rounding = Rounding::same(6.0);

        // Selection colors
        visuals.selection.bg_fill = Self::PRIMARY.linear_multiply(0.25);
        visuals.selection.stroke = Stroke::new(1.0, Self::PRIMARY);

        // Window styling
        visuals.window_rounding = Rounding::same(10.0);
        visuals.window_stroke = Stroke::new(0.5, Self::BORDER);
        visuals.window_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 10.0),
            blur: 30.0,
            spread: 8.0,
            color: Color32::from_black_alpha(120),
        };

        // Popup shadow
        visuals.popup_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 6.0),
            blur: 16.0,
            spread: 4.0,
            color: Color32::from_black_alpha(100),
        };

        // Menu rounding
        visuals.menu_rounding = Rounding::same(8.0);

        // Striped backgrounds for tables
        visuals.striped = true;

        style.visuals = visuals;

        // Set up text styles with good readability
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();

        // Set spacing for comfortable touch/click targets
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.indent = 20.0;
        style.spacing.slider_width = 160.0;
        style.spacing.combo_width = 120.0;
        style.spacing.icon_width = 18.0;
        style.spacing.icon_spacing = 6.0;

        // Interaction settings
        style.interaction.tooltip_delay = 0.3;

        ctx.set_style(style);
    }

    /// Apply light theme to egui
    pub fn apply_light(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = Visuals::light();

        // Light theme background colors - clean and modern
        let bg_primary = Color32::from_rgb(249, 250, 251); // Gray-50
        let bg_secondary = Color32::from_rgb(243, 244, 246); // Gray-100
        let bg_tertiary = Color32::from_rgb(229, 231, 235); // Gray-200
        let bg_hover = Color32::from_rgb(209, 213, 219); // Gray-300
        let text_primary = Color32::from_rgb(17, 24, 39); // Gray-900
        let text_secondary = Color32::from_rgb(75, 85, 99); // Gray-600
        let border = Color32::from_rgb(209, 213, 219); // Gray-300

        visuals.panel_fill = bg_primary;
        visuals.window_fill = Color32::WHITE;
        visuals.extreme_bg_color = Color32::WHITE;
        visuals.faint_bg_color = bg_secondary;

        visuals.widgets.noninteractive.bg_fill = bg_secondary;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_primary);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, border);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        visuals.widgets.inactive.bg_fill = bg_tertiary;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_secondary);
        visuals.widgets.inactive.bg_stroke = Stroke::new(0.5, border);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = bg_hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_primary);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::PRIMARY.linear_multiply(0.7));
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.expansion = 1.0;

        visuals.widgets.active.bg_fill = Self::PRIMARY;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::PRIMARY_DARK);
        visuals.widgets.active.rounding = Rounding::same(6.0);

        visuals.widgets.open.bg_fill = Color32::WHITE;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, text_primary);
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::PRIMARY.linear_multiply(0.6));
        visuals.widgets.open.rounding = Rounding::same(6.0);

        visuals.selection.bg_fill = Self::PRIMARY.linear_multiply(0.15);
        visuals.selection.stroke = Stroke::new(1.0, Self::PRIMARY);

        visuals.window_rounding = Rounding::same(10.0);
        visuals.window_stroke = Stroke::new(0.5, border);
        visuals.window_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 4.0,
            color: Color32::from_black_alpha(20),
        };

        visuals.popup_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 12.0,
            spread: 2.0,
            color: Color32::from_black_alpha(15),
        };

        visuals.menu_rounding = Rounding::same(8.0);
        visuals.striped = true;

        style.visuals = visuals;

        // Set up text styles - match dark theme
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();

        // Match dark theme spacing
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.indent = 20.0;
        style.spacing.slider_width = 160.0;
        style.spacing.combo_width = 120.0;
        style.spacing.icon_width = 18.0;
        style.spacing.icon_spacing = 6.0;

        style.interaction.tooltip_delay = 0.3;

        ctx.set_style(style);
    }

    /// Get color for instance status
    pub fn status_color(status: &crate::core::InstanceStatus) -> Color32 {
        use crate::core::InstanceStatus;
        match status {
            InstanceStatus::Running => Self::STATUS_RUNNING,
            InstanceStatus::Starting => Self::STATUS_STARTING,
            InstanceStatus::Paused => Self::STATUS_PAUSED,
            InstanceStatus::Stopping => Self::WARNING,
            InstanceStatus::Stopped => Self::STATUS_STOPPED,
            InstanceStatus::Crashed => Self::STATUS_CRASHED,
            InstanceStatus::Unknown => Self::TEXT_MUTED,
        }
    }
}

/// Icon characters (using Unicode symbols)
pub struct Icons;

impl Icons {
    pub const PLAY: &'static str = "▶";
    pub const PAUSE: &'static str = "⏸";
    pub const STOP: &'static str = "⏹";
    pub const RESTART: &'static str = "↻";
    pub const CLOSE: &'static str = "✕";
    pub const ADD: &'static str = "+";
    pub const SETTINGS: &'static str = "⚙";
    pub const FOLDER: &'static str = "📁";
    pub const APP: &'static str = "📦";
    pub const PROFILE: &'static str = "📋";
    pub const CHART: &'static str = "📊";
    pub const HISTORY: &'static str = "📜";
    pub const CPU: &'static str = "⚡";
    pub const MEMORY: &'static str = "💾";
    pub const NETWORK: &'static str = "🌐";
    pub const STAR: &'static str = "★";
    pub const STAR_EMPTY: &'static str = "☆";
    pub const SEARCH: &'static str = "🔍";
    pub const FILTER: &'static str = "⚖";
    pub const GRID: &'static str = "▦";
    pub const LIST: &'static str = "☰";
    pub const COMPACT: &'static str = "▤";
    pub const EXPAND: &'static str = "⬚";
    pub const COLLAPSE: &'static str = "▣";
    pub const WARNING: &'static str = "⚠";
    pub const ERROR: &'static str = "⛔";
    pub const INFO: &'static str = "ℹ";
    pub const SUCCESS: &'static str = "✓";
    pub const COPY: &'static str = "📋";
    pub const TRASH: &'static str = "🗑";
    pub const EDIT: &'static str = "✎";
    pub const SAVE: &'static str = "💾";
    pub const EXPORT: &'static str = "📤";
    pub const IMPORT: &'static str = "📥";
}
