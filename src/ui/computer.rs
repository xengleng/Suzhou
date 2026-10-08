//! A picture of a Bot's screen on the Agent Computer: a browser window on a
//! desktop with a dock, and the pointer gliding to whatever it is doing.

use std::time::Duration;

use gpui::{Animation, AnimationExt, Div, FontWeight, SharedString, div, prelude::*, px};

use super::kit::icon;
use crate::{
    model::ComputerState,
    theme::{MONO, Theme, hex},
};

/// Draws the screen `width` pixels wide at 16:10. `cursor` is the pointer
/// position for this frame as fractions of the screen.
pub fn screen(
    key: &str,
    state: &ComputerState,
    cursor: (f32, f32),
    width: f32,
    live: bool,
    t: &Theme,
) -> Div {
    let height = width * 0.625;
    let s = width / 900.; // everything below is drawn for a 900px screen
    let page_dark = state.url.contains("datacamp");
    let (page_bg, page_fg, page_muted) = if page_dark {
        (hex(0x05192D), hex(0xFFFFFF), hex(0x9AA7B4))
    } else {
        (hex(0xFFFFFF), hex(0x1B1B1B), hex(0x8A8A8A))
    };
    let host = state
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string();
    let blank = state.url == "about:blank";

    let page = if blank {
        div()
            .flex_1()
            .bg(page_bg)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10. * s))
            .child(icon("icons/globe.svg", 40. * s, page_muted))
            .child(
                div()
                    .text_size(px(15. * s))
                    .text_color(page_muted)
                    .child("Nothing open yet"),
            )
    } else {
        let bar = |w: f32, h: f32, c: gpui::Hsla| {
            div().w(px(w * s)).h(px(h * s)).rounded(px(3. * s)).bg(c)
        };
        div()
            .flex_1()
            .bg(page_bg)
            .flex()
            .flex_col()
            .gap(px(14. * s))
            .p(px(28. * s))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(15. * s))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(page_fg)
                            .child(host.split('/').next().unwrap_or_default().to_string()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10. * s))
                            .child(bar(46., 10., page_muted.opacity(0.4)))
                            .child(bar(46., 10., page_muted.opacity(0.4)))
                            .child(
                                div()
                                    .px(px(10. * s))
                                    .py(px(5. * s))
                                    .rounded(px(5. * s))
                                    .border_1()
                                    .border_color(page_muted)
                                    .text_size(px(11. * s))
                                    .text_color(page_fg)
                                    .child("Sign in"),
                            ),
                    ),
            )
            .child(
                div()
                    .pt(px(18. * s))
                    .text_size(px(30. * s))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(page_fg)
                    .child(state.page_title.clone()),
            )
            .child(bar(520., 10., page_muted.opacity(0.45)))
            .child(bar(460., 10., page_muted.opacity(0.45)))
            .child(bar(380., 10., page_muted.opacity(0.45)))
            .child(
                div()
                    .pt(px(10. * s))
                    .flex()
                    .gap(px(14. * s))
                    .children((0..3).map(|i| {
                        div()
                            .w(px(180. * s))
                            .h(px(92. * s))
                            .rounded(px(8. * s))
                            .bg(if page_dark {
                                hex(0x0F2A44)
                            } else {
                                hex(0xF2F2F2)
                            })
                            .p(px(10. * s))
                            .flex()
                            .flex_col()
                            .gap(px(6. * s))
                            .child(bar(120. - i as f32 * 20., 8., page_muted.opacity(0.5)))
                            .child(bar(90., 8., page_muted.opacity(0.35)))
                    })),
            )
    };

    let browser = div()
        .absolute()
        .left(px(60. * s))
        .top(px(28. * s))
        .w(px(780. * s))
        .h(px(height - 90. * s))
        .rounded(px(8. * s))
        .overflow_hidden()
        .bg(hex(0xDEE1E6))
        .flex()
        .flex_col()
        .shadow(super::kit::shadow(t, 10. * s, 30. * s))
        .child(
            // Tab strip
            div()
                .h(px(30. * s))
                .flex()
                .items_end()
                .px(px(10. * s))
                .gap(px(6. * s))
                .child(
                    div()
                        .h(px(24. * s))
                        .w(px(200. * s))
                        .px(px(10. * s))
                        .rounded_t(px(7. * s))
                        .bg(hex(0xFFFFFF))
                        .flex()
                        .items_center()
                        .gap(px(6. * s))
                        .child(
                            div()
                                .size(px(10. * s))
                                .rounded(px(2. * s))
                                .bg(hex(0x10B981)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .truncate()
                                .text_size(px(10. * s))
                                .text_color(hex(0x333333))
                                .child(state.page_title.clone()),
                        ),
                )
                .child(
                    div()
                        .pb(px(6. * s))
                        .text_size(px(14. * s))
                        .text_color(hex(0x555555))
                        .child("+"),
                ),
        )
        .child(
            // Address bar
            div()
                .h(px(30. * s))
                .bg(hex(0xFFFFFF))
                .flex()
                .items_center()
                .gap(px(8. * s))
                .px(px(10. * s))
                .child(icon("icons/chevron-left.svg", 12. * s, hex(0x777777)))
                .child(icon("icons/chevron-right.svg", 12. * s, hex(0xBBBBBB)))
                .child(icon("icons/refresh.svg", 11. * s, hex(0x777777)))
                .child(
                    div()
                        .flex_1()
                        .h(px(20. * s))
                        .rounded_full()
                        .bg(hex(0xF1F3F4))
                        .flex()
                        .items_center()
                        .gap(px(6. * s))
                        .px(px(10. * s))
                        .child(icon("icons/lock.svg", 10. * s, hex(0x777777)))
                        .child(
                            div()
                                .truncate()
                                .text_size(px(10.5 * s))
                                .text_color(hex(0x333333))
                                .font_family(MONO)
                                .child(if blank {
                                    "about:blank".to_string()
                                } else {
                                    host.clone()
                                }),
                        ),
                ),
        )
        .child(page);

    let dock = div()
        .absolute()
        .bottom(px(10. * s))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .flex()
                .gap(px(10. * s))
                .px(px(12. * s))
                .py(px(7. * s))
                .rounded(px(12. * s))
                .bg(hex(0xFFFFFF).opacity(0.55))
                .children(
                    [
                        ("icons/globe.svg", 0x1A73E8u32),
                        ("icons/folder.svg", 0xF59E0B),
                        ("icons/terminal.svg", 0x111111),
                    ]
                    .map(|(path, color)| {
                        div()
                            .size(px(30. * s))
                            .rounded(px(8. * s))
                            .bg(hex(color))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(icon(path, 16. * s, gpui::white()))
                    }),
                ),
        );

    let pointer_x = cursor.0 * width;
    let pointer_y = cursor.1 * height;
    let mut pointer = div()
        .absolute()
        .left(px(pointer_x))
        .top(px(pointer_y))
        .child(icon("icons/pointer.svg", 20. * s.max(0.6), hex(0x111111)));
    if let Some(typing) = &state.typing
        && live
    {
        pointer = pointer.child(
            div()
                .absolute()
                .left(px(16.))
                .top(px(16.))
                .px(px(8.))
                .py(px(3.))
                .rounded(px(6.))
                .bg(hex(0x111111))
                .text_color(gpui::white())
                .text_size(px(11.))
                .whitespace_nowrap()
                .child(format!("\u{2328} {typing}")),
        );
    }

    let mut screen = div()
        .relative()
        .w(px(width))
        .h(px(height))
        .overflow_hidden()
        .bg(gpui::linear_gradient(
            160.,
            gpui::linear_color_stop(hex(0xC9CED6), 0.),
            gpui::linear_color_stop(hex(0x8E97A5), 1.),
        ))
        .child(browser)
        .child(dock)
        .child(pointer);

    if live && !state.status.is_empty() && state.status != "Idle" {
        screen = screen.child(
            div()
                .absolute()
                .top(px(10.))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(10.))
                        .py(px(4.))
                        .rounded_full()
                        .bg(hex(0x111111).opacity(0.78))
                        .text_color(gpui::white())
                        .text_size(px(12.))
                        .child(
                            div()
                                .size(px(6.))
                                .rounded_full()
                                .bg(hex(0x34C759))
                                .with_animation(
                                    SharedString::from(format!("{key}-live")),
                                    Animation::new(Duration::from_millis(1000))
                                        .repeat()
                                        .with_easing(gpui::pulsating_between(0.3, 1.0)),
                                    |this, t| this.opacity(t),
                                ),
                        )
                        .child(state.status.clone()),
                ),
        );
    }
    screen
}
