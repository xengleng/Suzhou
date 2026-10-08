//! First run: sign in, pick your apps, wait for the computer, meet a Bot.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, Context, FontWeight, Hsla, IntoElement, SharedString, Window, div,
    prelude::*, px, svg,
};

use super::{
    app::{AppView, Stage},
    kit::{self, icon},
};
use crate::{
    anim, data,
    model::Shape,
    theme::{Theme, bot_color, hex, theme},
};

/// The black face from the sign-in screen, drawn in the text colour.
fn logo(size: f32, t: &Theme) -> impl IntoElement + use<> {
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .child(
            svg()
                .absolute()
                .size(px(size))
                .path("avatars/circle.svg")
                .text_color(t.text),
        )
        .child(
            svg()
                .absolute()
                .size(px(size))
                .path("avatars/eyes-circle.svg")
                .text_color(t.bg),
        )
}

/// Fades and lifts a screen into place when the stage changes.
fn entrance(stage: Stage, child: gpui::Div) -> impl IntoElement + use<> {
    child.with_animation(
        SharedString::from(format!("stage-{stage:?}")),
        Animation::new(Duration::from_millis(560)).with_easing(anim::ease_out_quint),
        |this, t| this.opacity(t).top(px(14. * (1. - t))),
    )
}

fn title(text: &str, size: f32, t: &Theme) -> gpui::Div {
    div()
        .text_size(px(size))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(t.text)
        .child(text.to_string())
}

fn subtitle(text: &str, t: &Theme) -> gpui::Div {
    div()
        .max_w(px(460.))
        .text_center()
        .text_size(px(17.))
        .line_height(px(26.))
        .text_color(t.text_muted)
        .child(text.to_string())
}

fn pill(id: &'static str, label: &str, t: &Theme) -> gpui::Stateful<gpui::Div> {
    let hover = kit::mix(t.primary, t.bg, 0.18);
    div()
        .id(id)
        .flex()
        .items_center()
        .gap_2()
        .h(px(48.))
        .px(px(30.))
        .rounded_full()
        .bg(t.primary)
        .text_color(t.primary_text)
        .text_size(px(19.))
        .font_weight(FontWeight::MEDIUM)
        .border_2()
        .border_color(t.accent)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(label.to_string())
        .child(icon("icons/arrow-right.svg", 17., t.primary_text))
}

impl AppView {
    pub fn render_onboarding(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let body = match self.stage {
            Stage::SignIn => self.sign_in(&t, cx),
            Stage::Authorizing => self.authorizing(&t, cx),
            Stage::Tools => self.tools(&t, cx),
            Stage::SettingUp => self.setting_up(&t, window),
            Stage::Meet => self.meet(&t, cx),
            Stage::Main => div(),
        };
        div()
            .id("onboarding")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(t.bg)
            .child(div().h(px(36.)).w_full().flex_none())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(entrance(self.stage, body)),
            )
            .child(div().h(px(36.)).flex_none())
    }

    fn sign_in(&mut self, t: &Theme, cx: &mut Context<Self>) -> gpui::Div {
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(26.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(22.))
                    .child(logo(88., t))
                    .child(
                        div()
                            .text_size(px(76.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.text)
                            .child("Suzhou"),
                    ),
            )
            .child(
                subtitle(
                    "Your team of always-on Bots that you can give real work to.",
                    t,
                )
                .max_w(px(420.))
                .text_size(px(22.))
                .line_height(px(32.))
                .text_color(t.text),
            )
            .child(div().h(px(8.)))
            .child(pill("sign-in", "Sign in", t).on_click(
                cx.listener(|this, _, window, cx| this.go(Stage::Authorizing, window, cx)),
            ))
    }

    fn authorizing(&mut self, t: &Theme, cx: &mut Context<Self>) -> gpui::Div {
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(18.))
            .child(logo(56., t))
            .child(title("Finish signing in in your browser", 28., t))
            .child(subtitle("We opened a page in your browser. Approve the sign-in there and you will come straight back here.", t))
            .child(div().flex().items_center().gap_2().text_color(t.text_muted).child(kit::spinner("auth-spin", 16., t.text_muted)).child("Waiting for your browser\u{2026}"))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .pt(px(8.))
                    .child(kit::secondary_button("auth-cancel", "Cancel", t).on_click(cx.listener(|this, _, window, cx| this.go(Stage::SignIn, window, cx))))
                    .child(kit::primary_button("auth-done", "I've signed in", t, true).on_click(cx.listener(|this, _, window, cx| this.go(Stage::Tools, window, cx)))),
            )
    }

    fn tools(&mut self, t: &Theme, cx: &mut Context<Self>) -> gpui::Div {
        let tiles = data::WORK_TOOLS
            .iter()
            .map(|(id, name)| {
                let id = *id;
                let picked = self.tools_picked.contains(&id);
                let plugin = self.plugins.iter().find(|p| p.id == id);
                let (glyph, color) = plugin
                    .map(|p| (p.glyph, p.color))
                    .unwrap_or(("?", 0x888888));
                let hover = t.hover;
                div()
                    .id(SharedString::from(format!("tool-{id}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .w(px(214.))
                    .h(px(52.))
                    .px(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(if picked { t.accent } else { t.border })
                    .bg(if picked {
                        t.accent.opacity(0.07)
                    } else {
                        t.surface
                    })
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .child(glyph_tile(glyph, color, 28.))
                    .child(div().flex_1().text_size(px(15.)).child(name.to_string()))
                    .child(if picked {
                        div()
                            .size(px(20.))
                            .rounded_full()
                            .bg(t.accent)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(icon("icons/check.svg", 13., gpui::white()))
                    } else {
                        div()
                            .size(px(20.))
                            .rounded_full()
                            .border_1()
                            .border_color(t.border_strong)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(at) = this.tools_picked.iter().position(|p| *p == id) {
                            this.tools_picked.remove(at);
                        } else {
                            this.tools_picked.push(id);
                        }
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.))
            .child(title("What do you use for work?", 30., t))
            .child(subtitle("Your Bots can use these apps on your behalf. You can change this any time in Plugins.", t))
            .child(div().pt(px(8.)).w(px(440.)).flex().flex_wrap().gap(px(12.)).children(tiles))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .pt(px(10.))
                    .child(kit::secondary_button("tools-skip", "Skip", t).on_click(cx.listener(|this, _, window, cx| {
                        this.tools_picked.clear();
                        this.go(Stage::SettingUp, window, cx)
                    })))
                    .child(kit::primary_button("tools-next", "Continue", t, true).on_click(cx.listener(|this, _, window, cx| this.go(Stage::SettingUp, window, cx)))),
            )
    }

    fn setting_up(&mut self, t: &Theme, window: &mut Window) -> gpui::Div {
        let progress = self.setup.tick(window);
        let step = if progress < 0.34 {
            "Starting your cloud computer"
        } else if progress < 0.68 {
            "Installing a browser and terminal"
        } else {
            "Syncing your connected apps"
        };
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(18.))
            .child(computer_glyph(t))
            .child(title("Setting up your computer", 28., t))
            .child(subtitle("Every Bot gets its own screen on one cloud computer. Files, sign-ins and apps are shared between them.", t))
            .child(
                div()
                    .mt(px(10.))
                    .w(px(360.))
                    .h(px(6.))
                    .rounded_full()
                    .bg(t.field)
                    .child(div().h_full().rounded_full().bg(t.accent).w(px(360. * progress))),
            )
            .child(div().text_size(px(14.)).text_color(t.text_muted).child(format!("{step}\u{2026}")))
    }

    fn meet(&mut self, t: &Theme, cx: &mut Context<Self>) -> gpui::Div {
        div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(18.))
            .child(div().h(px(140.)).flex().items_end().child(kit::bobbing("meet-bob", kit::avatar(bot_color(6), Shape::Circle, 112., false), 18.)))
            .child(title("Meet a future teammate", 34., t))
            .child(subtitle("Bots work on their own computer, remember what you teach them, and ask before anything risky. Give one a name and a job.", t))
            .child(div().h(px(6.)))
            .child(kit::primary_button("meet-go", "Create your first Bot", t, true).h(px(42.)).px(px(22.)).on_click(cx.listener(|this, _, window, cx| this.go(Stage::Main, window, cx))))
    }
}

fn computer_glyph(t: &Theme) -> impl IntoElement + use<> {
    div()
        .size(px(72.))
        .rounded(px(18.))
        .bg(t.field)
        .flex()
        .items_center()
        .justify_center()
        .child(icon("icons/monitor.svg", 34., t.text))
        .with_animation(
            "setup-pulse",
            Animation::new(Duration::from_millis(1400))
                .repeat()
                .with_easing(gpui::pulsating_between(0.55, 1.0)),
            |this, t| this.opacity(t),
        )
}

/// A coloured square with a short label, standing in for an app's logo.
pub fn glyph_tile(glyph: &str, color: u32, size: f32) -> impl IntoElement + use<> {
    let fg: Hsla = gpui::white();
    div()
        .flex_none()
        .size(px(size))
        .rounded(px(size * 0.24))
        .bg(hex(color))
        .flex()
        .items_center()
        .justify_center()
        .text_color(fg)
        .text_size(px(if glyph.len() > 1 {
            size * 0.36
        } else {
            size * 0.5
        }))
        .font_weight(FontWeight::SEMIBOLD)
        .child(glyph.to_string())
}
