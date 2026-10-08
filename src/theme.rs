//! Colours, type and the light/dark switch.
//!
//! The greys are sampled from the reference screenshots: a near-white chat pane,
//! a slightly darker sidebar, black user bubbles and soft grey Bot bubbles.

use gpui::{App, Global, Hsla, Rgba, WindowAppearance, rgb};

pub const FONT: &str = "Inter";
pub const MONO: &str = "DejaVu Sans Mono";

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

/// Which palette is live. Stored as a global so any render function can reach it.
pub struct ActiveTheme {
    pub mode: ThemeMode,
    pub system_dark: bool,
}

impl Global for ActiveTheme {}

impl ActiveTheme {
    pub fn is_dark(&self) -> bool {
        match self.mode {
            ThemeMode::System => self.system_dark,
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
        }
    }
}

pub fn appearance_is_dark(appearance: WindowAppearance) -> bool {
    matches!(
        appearance,
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub bg: Hsla,
    pub sidebar: Hsla,
    pub sidebar_selected: Hsla,
    pub hover: Hsla,
    pub border: Hsla,
    pub border_strong: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub text_faint: Hsla,
    pub user_bubble: Hsla,
    pub user_text: Hsla,
    pub bot_bubble: Hsla,
    pub field: Hsla,
    pub field_border: Hsla,
    pub surface: Hsla,
    pub surface_raised: Hsla,
    pub primary: Hsla,
    pub primary_text: Hsla,
    pub disabled: Hsla,
    pub disabled_text: Hsla,
    pub accent: Hsla,
    pub link: Hsla,
    pub code: Hsla,
    pub code_bg: Hsla,
    pub online: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
    pub scrim: Hsla,
    pub shadow: Hsla,
    pub selection: Hsla,
    pub caret: Hsla,
}

fn c(hex: u32) -> Hsla {
    let rgba: Rgba = rgb(hex);
    rgba.into()
}

fn ca(hex: u32, alpha: f32) -> Hsla {
    let mut color = c(hex);
    color.a = alpha;
    color
}

impl Theme {
    pub fn light() -> Self {
        Theme {
            dark: false,
            bg: c(0xFBFBFB),
            sidebar: c(0xF6F6F6),
            sidebar_selected: c(0xE4E4E4),
            hover: ca(0x000000, 0.045),
            border: c(0xEAEAEA),
            border_strong: c(0xDADADA),
            text: c(0x161616),
            text_muted: c(0x6B6B6B),
            text_faint: c(0x9C9C9C),
            user_bubble: c(0x0B0B0B),
            user_text: c(0xFAFAFA),
            bot_bubble: c(0xEEEEEE),
            field: c(0xECECEC),
            field_border: c(0xDCDCDC),
            surface: c(0xFFFFFF),
            surface_raised: c(0xFDFDFD),
            primary: c(0x0B0B0B),
            primary_text: c(0xFFFFFF),
            disabled: c(0xDDDDDD),
            disabled_text: c(0x9E9E9E),
            accent: c(0x3B82F6),
            link: c(0x2D6BD6),
            code: c(0xC0283E),
            code_bg: c(0xF4F4F4),
            online: c(0x34C759),
            warning: c(0xF5A33C),
            danger: c(0xE5484D),
            scrim: ca(0x000000, 0.42),
            shadow: ca(0x000000, 0.16),
            selection: ca(0x3B82F6, 0.25),
            caret: c(0x3B82F6),
        }
    }

    pub fn dark() -> Self {
        Theme {
            dark: true,
            bg: c(0x0E0E0E),
            sidebar: c(0x151515),
            sidebar_selected: c(0x2A2A2A),
            hover: ca(0xFFFFFF, 0.06),
            border: c(0x232323),
            border_strong: c(0x333333),
            text: c(0xEDEDED),
            text_muted: c(0x9A9A9A),
            text_faint: c(0x6A6A6A),
            user_bubble: c(0x4A4A4A),
            user_text: c(0xF5F5F5),
            bot_bubble: c(0x242424),
            field: c(0x1F1F1F),
            field_border: c(0x353535),
            surface: c(0x1A1A1A),
            surface_raised: c(0x202020),
            primary: c(0xF2F2F2),
            primary_text: c(0x0B0B0B),
            disabled: c(0x2C2C2C),
            disabled_text: c(0x6A6A6A),
            accent: c(0x4C8DF6),
            link: c(0x5B9BFF),
            code: c(0xF2627A),
            code_bg: c(0x1C1C1C),
            online: c(0x30D158),
            warning: c(0xF5A33C),
            danger: c(0xF0585D),
            scrim: ca(0x000000, 0.6),
            shadow: ca(0x000000, 0.5),
            selection: ca(0x4C8DF6, 0.35),
            caret: c(0x4C8DF6),
        }
    }
}

/// The palette for the current frame.
pub fn theme(cx: &App) -> Theme {
    if cx.has_global::<ActiveTheme>() && cx.global::<ActiveTheme>().is_dark() {
        Theme::dark()
    } else {
        Theme::light()
    }
}

/// The ten Bot colours from the avatar picker, in the order shown there.
pub const BOT_COLORS: [u32; 10] = [
    0x8D5B37, 0xE8423F, 0xEF7430, 0xF2A23A, 0x4CC16E, 0x4DB6A6, 0x3B82F6, 0x8B5CF6, 0xEC4899,
    0x6F6F6F,
];

pub fn bot_color(ix: usize) -> Hsla {
    c(BOT_COLORS[ix % BOT_COLORS.len()])
}

pub fn hex(hex_value: u32) -> Hsla {
    c(hex_value)
}
