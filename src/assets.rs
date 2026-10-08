//! Icons, avatar shapes and fonts, compiled into the binary by `build.rs`.

use std::borrow::Cow;

use anyhow::Result;
use gpui::{App, AssetSource, SharedString};

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(FILES
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

/// Registers the bundled fonts so the interface looks the same on every machine.
pub fn load_fonts(cx: &mut App) {
    let fonts = FILES
        .iter()
        .filter(|(name, _)| name.starts_with("fonts/"))
        .map(|(_, bytes)| Cow::Borrowed(*bytes))
        .collect();
    if let Err(err) = cx.text_system().add_fonts(fonts) {
        eprintln!("suzhou: could not load bundled fonts: {err}");
    }
}
