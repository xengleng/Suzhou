//! Everything you see.

pub mod app;
mod chat;
mod computer;
pub mod kit;
mod modals;
mod new_bot;
mod onboarding;
mod overlays;
pub mod panels;
pub mod rich;
pub mod sidebar;
pub mod text_input;

use gpui::{App, KeyBinding};

use crate::theme::{ActiveTheme, ThemeMode};

pub fn init(cx: &mut App) {
    cx.set_global(ActiveTheme {
        mode: ThemeMode::System,
        system_dark: false,
    });
    text_input::bind_keys(cx);
    use app::*;
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewBot, None),
        KeyBinding::new("secondary-k", FocusSearch, None),
        KeyBinding::new("secondary-d", ToggleVoice, None),
        KeyBinding::new("secondary-shift-l", ToggleTheme, None),
        KeyBinding::new("secondary-,", OpenSettings, None),
        KeyBinding::new("secondary-shift-p", OpenPlugins, None),
        KeyBinding::new("secondary-shift-c", ToggleComputer, None),
        KeyBinding::new("secondary-i", ToggleDetails, None),
        KeyBinding::new("secondary-\\", ToggleSidebar, None),
        KeyBinding::new("secondary-.", StopBot, None),
        KeyBinding::new("escape", Dismiss, None),
        KeyBinding::new("secondary-q", Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
}
