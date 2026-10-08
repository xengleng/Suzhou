//! Dialogs: Plugins, Settings, a file, taking over the computer, teaching a
//! task, and "are you sure?".

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Context, FontWeight, IntoElement, MouseButton,
    MouseDownEvent, SharedString, Window, div, prelude::*, px,
};

use super::{
    app::{AppView, Modal},
    computer,
    kit::{self, icon},
    onboarding::glyph_tile,
    rich::{self, Style},
};
use crate::{
    anim, data,
    model::*,
    theme::{ActiveTheme, MONO, Theme, ThemeMode, theme},
};

impl AppView {
    pub fn modal_body(
        &mut self,
        modal: &Modal,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let t = theme(cx);
        match modal {
            Modal::Plugins => self.plugins_modal(&t, window, cx).into_any_element(),
            Modal::Settings => self.settings_modal(&t, cx).into_any_element(),
            Modal::File { name, content } => self.file_modal(name, content, &t, cx).into_any_element(),
            Modal::Takeover => self.takeover_modal(&t, window, cx).into_any_element(),
            Modal::Teach => self.teach_modal(&t, window, cx).into_any_element(),
            Modal::ConfirmDelete(id) => {
                let id = *id;
                let name = self.bot(id).map(|b| b.name.clone()).unwrap_or_default();
                confirm(
                    &format!("Delete {name}?"),
                    "Its conversation, routines and skills are removed. Files it saved on the computer stay.",
                    "Delete",
                    &t,
                    cx,
                    move |this, window, cx| this.delete_bot(id, window, cx),
                )
                .into_any_element()
            }
            Modal::ConfirmReset => confirm(
                "Reset the computer?",
                "This throws away every Bot's screen, open pages and anything not yet synced. Try Recover first.",
                "Reset",
                &t,
                cx,
                |this, window, cx| this.reset_computer(window, cx),
            )
            .into_any_element(),
        }
    }

    fn plugins_modal(
        &mut self,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let query = self.inputs.plugin_search.read(cx).text().to_lowercase();
        let category = data::PLUGIN_CATEGORIES[self.plugin_category];
        let installed: Vec<Plugin> = self
            .plugins
            .iter()
            .filter(|p| p.installed)
            .cloned()
            .collect();
        let matches = |p: &&Plugin| {
            (query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query))
                && match category {
                    "All" => true,
                    "Featured" => p.featured,
                    "Team plugins" => p.team,
                    c => p.categories.contains(&c),
                }
        };
        let featured: Vec<Plugin> = self
            .plugins
            .iter()
            .filter(|p| p.featured)
            .filter(matches)
            .cloned()
            .collect();
        let team: Vec<Plugin> = self
            .plugins
            .iter()
            .filter(|p| p.team)
            .filter(matches)
            .cloned()
            .collect();
        let rest: Vec<Plugin> = self
            .plugins
            .iter()
            .filter(|p| !p.featured && !p.team)
            .filter(matches)
            .cloned()
            .collect();
        let focused = self.inputs.plugin_search.read(cx).is_focused(window);

        let chips = data::PLUGIN_CATEGORIES
            .iter()
            .enumerate()
            .map(|(ix, name)| {
                kit::chip(
                    SharedString::from(format!("cat-{ix}")),
                    *name,
                    ix == self.plugin_category,
                    t,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.plugin_category = ix;
                    cx.notify();
                }))
            })
            .collect::<Vec<_>>();

        let mut sections = div().flex().flex_col();
        for (title, list) in [
            ("Featured", featured),
            ("Team plugins", team),
            ("More", rest),
        ] {
            if list.is_empty() {
                continue;
            }
            sections = sections
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .pt(px(26.))
                        .pb(px(12.))
                        .px(px(14.))
                        .text_size(px(15.))
                        .text_color(t.text_muted)
                        .child(title)
                        .when(title == "Featured", |d| d.child("View all")),
                )
                .child(
                    div().flex().flex_wrap().children(
                        list.into_iter()
                            .map(|p| self.plugin_card(p, t, cx))
                            .collect::<Vec<_>>(),
                    ),
                );
        }
        if self.plugins.iter().filter(|p| matches(p)).count() == 0 {
            sections = sections.child(
                div()
                    .py(px(40.))
                    .flex()
                    .justify_center()
                    .text_color(t.text_muted)
                    .child("No plugins match"),
            );
        }

        div()
            .w(px(940f32.min(self.vp.0 * 0.94)))
            .h(px(640f32.min(self.vp.1 * 0.9)))
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_none()
                    .px(px(36.))
                    .pt(px(30.))
                    .flex()
                    .flex_col()
                    .gap(px(18.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(24.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Plugins"),
                            )
                            .child(
                                kit::icon_button("plugins-close", "icons/x.svg", t).on_click(
                                    cx.listener(|this, _, window, cx| this.close_modal(window, cx)),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                div()
                                    .flex()
                                    .children(installed.iter().take(4).enumerate().map(
                                        |(i, p)| {
                                            div()
                                                .ml(px(if i == 0 { 0. } else { -6. }))
                                                .rounded(px(9.))
                                                .border_2()
                                                .border_color(t.surface)
                                                .child(glyph_tile(p.glyph, p.color, 32.))
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .text_color(t.text_muted)
                                    .child(format!("{} installed", installed.len())),
                            )
                            .child(icon("icons/chevron-right.svg", 16., t.text_muted)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .h(px(46.))
                            .px(px(14.))
                            .rounded(px(10.))
                            .bg(t.field)
                            .border_1()
                            .border_color(if focused { t.accent } else { t.field })
                            .child(icon("icons/search.svg", 18., t.text_muted))
                            .child(div().flex_1().child(self.inputs.plugin_search.clone())),
                    )
                    .child(div().flex().flex_wrap().gap(px(10.)).children(chips)),
            )
            .child(
                div()
                    .id("plugins-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(22.))
                    .pb(px(24.))
                    .child(sections),
            )
    }

    fn plugin_card(
        &self,
        p: Plugin,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = p.id;
        let connecting = self.connecting.contains(&id);
        let button: AnyElement = if connecting {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(36.))
                .px(px(14.))
                .rounded_full()
                .bg(t.field)
                .text_size(px(14.))
                .text_color(t.text_muted)
                .child(kit::spinner(
                    SharedString::from(format!("conn-{id}")),
                    14.,
                    t.text_muted,
                ))
                .child("Connecting")
                .into_any_element()
        } else if p.installed {
            div()
                .id(SharedString::from(format!("added-{id}")))
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(36.))
                .px(px(14.))
                .rounded_full()
                .text_size(px(14.))
                .text_color(t.text_muted)
                .cursor_pointer()
                .child(icon("icons/check.svg", 14., t.online))
                .child("Added")
                .hover(|s| s.opacity(0.7))
                .on_click(cx.listener(move |this, _, _, cx| this.uninstall_plugin(id, cx)))
                .with_animation(
                    SharedString::from(format!("added-in-{id}")),
                    Animation::new(Duration::from_millis(320)).with_easing(anim::ease_out_quint),
                    |this, t| this.opacity(t),
                )
                .into_any_element()
        } else {
            let hover = t.sidebar_selected;
            div()
                .id(SharedString::from(format!("add-{id}")))
                .flex()
                .items_center()
                .h(px(36.))
                .px(px(18.))
                .rounded_full()
                .bg(t.field)
                .text_size(px(15.))
                .font_weight(FontWeight::MEDIUM)
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .child("Add")
                .on_click(cx.listener(move |this, _, _, cx| this.install_plugin(id, cx)))
                .into_any_element()
        };
        let tile: AnyElement = if p.team {
            div()
                .size(px(52.))
                .rounded(px(12.))
                .bg(t.field)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(18.))
                .text_color(t.text_muted)
                .child(p.glyph)
                .into_any_element()
        } else {
            glyph_tile(p.glyph, p.color, 52.).into_any_element()
        };
        div()
            .w(px(((940f32.min(self.vp.0 * 0.94)) - 46.) / 2.))
            .flex()
            .items_center()
            .gap(px(16.))
            .px(px(14.))
            .py(px(12.))
            .child(tile)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(17.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(p.name),
                            )
                            .when(p.team, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(4.))
                                        .px(px(7.))
                                        .py(px(2.))
                                        .rounded_full()
                                        .bg(t.field)
                                        .text_size(px(13.))
                                        .text_color(t.text_muted)
                                        .child(icon("icons/users.svg", 12., t.text_muted))
                                        .child("Team"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(15.))
                            .text_color(t.text_muted)
                            .child(p.description),
                    ),
            )
            .child(button)
    }

    fn settings_modal(&mut self, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sections = [
            "General",
            "Notifications",
            "Approvals",
            "Computer",
            "Account",
        ];
        let hover = t.hover;
        let nav = sections
            .iter()
            .enumerate()
            .map(|(ix, name)| {
                let selected = ix == self.settings_section;
                div()
                    .id(SharedString::from(format!("set-{ix}")))
                    .h(px(34.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .text_size(px(15.))
                    .cursor_pointer()
                    .when(selected, |d| d.bg(t.sidebar_selected))
                    .when(!selected, |d| d.hover(move |s| s.bg(hover)))
                    .child(*name)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.settings_section = ix;
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();

        let row = |title: &str, detail: &str, control: AnyElement| {
            div()
                .flex()
                .items_center()
                .gap(px(16.))
                .py(px(14.))
                .border_b_1()
                .border_color(t.border)
                .child(
                    div()
                        .flex_1()
                        .child(div().text_size(px(15.)).child(title.to_string()))
                        .child(
                            div()
                                .text_size(px(13.))
                                .line_height(px(19.))
                                .text_color(t.text_muted)
                                .child(detail.to_string()),
                        ),
                )
                .child(control)
        };
        macro_rules! switch {
            ($id:literal, $field:ident) => {
                kit::toggle($id, self.settings.$field, t)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings.$field = !this.settings.$field;
                        this.save(cx);
                        cx.notify();
                    }))
                    .into_any_element()
            };
        }

        let body: AnyElement = match self.settings_section {
            0 => {
                let mode = cx.global::<ActiveTheme>().mode;
                let segmented = div().flex().p(px(3.)).rounded(px(9.)).bg(t.field).children([ThemeMode::System, ThemeMode::Light, ThemeMode::Dark].map(|m| {
                    let selected = m == mode;
                    div()
                        .id(SharedString::from(format!("mode-{}", m.label())))
                        .px(px(12.))
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .rounded(px(7.))
                        .text_size(px(14.))
                        .cursor_pointer()
                        .when(selected, |d| d.bg(t.surface).shadow(kit::shadow(t, 1., 3.)))
                        .child(m.label())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.global_mut::<ActiveTheme>().mode = m;
                            this.save(cx);
                            cx.notify();
                        }))
                }));
                div()
                    .child(row("Appearance", "Match the system, or pick one.", segmented.into_any_element()))
                    .child(row("Backend", &format!("Who does the work. Set SUZHOU_BACKEND=pi to use Pi. Now: {}.", self.backend.name()), div().text_size(px(14.)).text_color(t.text_muted).child(self.backend.name()).into_any_element()))
                    .child(row(
                        "Keyboard shortcuts",
                        "New Bot \u{2318}/Ctrl+N \u{00b7} Search \u{2318}/Ctrl+K \u{00b7} Voice \u{2318}/Ctrl+D \u{00b7} Computer \u{2318}/Ctrl+\u{21e7}+C \u{00b7} Details \u{2318}/Ctrl+I \u{00b7} Sidebar \u{2318}/Ctrl+\\ \u{00b7} Dark mode \u{2318}/Ctrl+\u{21e7}+L",
                        div().into_any_element(),
                    ))
                    .into_any_element()
            }
            1 => div()
                .child(row("Desktop notifications", "A card in the corner when a Bot finishes, asks something or needs approval.", switch!("s-notif", notifications)))
                .child(row("Sounds", "Play a soft sound with each notification.", switch!("s-sounds", sounds)))
                .child(row("Only when a Bot needs me", "Skip \u{201c}finished\u{201d} notifications; keep questions and approvals.", switch!("s-needed", only_when_needed)))
                .into_any_element(),
            2 => div()
                .child(row("Auto Review", "Let a reviewer model approve low-risk actions. Risky ones still come to you.", switch!("s-auto", auto_review)))
                .child(row("Ask before sending messages", "Email, chat and invitations.", switch!("s-send", ask_before_sending)))
                .child(row("Ask before purchases", "Anything that spends money.", switch!("s-buy", ask_before_purchases)))
                .child(row("Ask before publishing", "Posts, pushes, deploys and public changes.", switch!("s-pub", ask_before_publishing)))
                .into_any_element(),
            3 => div()
                .child(row("Status", "One cloud computer for your account; every Bot has its own screen.", div().flex().items_center().gap(px(6.)).text_size(px(14.)).child(div().size(px(8.)).rounded_full().bg(t.online)).child("Running").into_any_element()))
                .child(row("Recover", "Restarts the computer's apps and keeps your files. Try this first if a screen is stuck.", kit::secondary_button("recover", "Recover", t).on_click(cx.listener(|this, _, window, cx| this.reset_computer(window, cx))).into_any_element()))
                .child(row("Reset", "Last resort. Discards work that has not synced.", kit::secondary_button("reset", "Reset\u{2026}", t).text_color(t.danger).on_click(cx.listener(|this, _, _, cx| this.open_modal(Modal::ConfirmReset, cx))).into_any_element()))
                .into_any_element(),
            _ => div()
                .child(row("Name", "Shown to your Bots and in the sidebar.", div().text_size(px(15.)).child(self.profile.name.clone()).into_any_element()))
                .child(row("Data", &format!("Saved in {}", crate::store::data_dir().display()), div().into_any_element()))
                .child(row("Sign out", "Your Bots keep working while you are away.", kit::secondary_button("signout", "Sign out", t).on_click(cx.listener(|this, _, window, cx| this.sign_out(window, cx))).into_any_element()))
                .into_any_element(),
        };

        div()
            .w(px(780f32.min(self.vp.0 * 0.94)))
            .h(px(520f32.min(self.vp.1 * 0.9)))
            .flex()
            .child(
                div()
                    .w(px(200.))
                    .flex_none()
                    .p(px(14.))
                    .border_r_1()
                    .border_color(t.border)
                    .bg(t.sidebar)
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .px(px(12.))
                            .pt(px(6.))
                            .pb(px(12.))
                            .text_size(px(20.))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Settings"),
                    )
                    .children(nav),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .px(px(24.))
                            .pt(px(18.))
                            .child(
                                div()
                                    .text_size(px(18.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(sections[self.settings_section]),
                            )
                            .child(
                                kit::icon_button("settings-close", "icons/x.svg", t).on_click(
                                    cx.listener(|this, _, window, cx| this.close_modal(window, cx)),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .id("settings-scroll")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .px(px(24.))
                            .pb(px(20.))
                            .child(body),
                    ),
            )
    }

    fn file_modal(
        &mut self,
        name: &str,
        content: &str,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let (n, c) = (name.to_string(), content.to_string());
        div()
            .w(px(1080f32.min(self.vp.0 * 0.94)))
            .h(px(self.vp.1 * 0.92))
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(52.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(20.))
                    .border_b_1()
                    .border_color(t.border)
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(16.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(name.to_string()),
                    )
                    .child(
                        kit::icon_button("file-download", "icons/download.svg", t).on_click(
                            cx.listener(move |this, _, _, cx| this.download_file(&n, &c, cx)),
                        ),
                    )
                    .child(
                        kit::icon_button("file-close", "icons/x.svg", t).on_click(
                            cx.listener(|this, _, window, cx| this.close_modal(window, cx)),
                        ),
                    ),
            )
            .child(
                div()
                    .id("file-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(720.))
                            .mx_auto()
                            .px(px(32.))
                            .py(px(40.))
                            .child(if name.ends_with(".md") {
                                rich::markdown("file", content, &Style::new(17., 28., t.text), t)
                                    .into_any_element()
                            } else {
                                div()
                                    .font_family(MONO)
                                    .text_size(px(14.))
                                    .child(content.to_string())
                                    .into_any_element()
                            }),
                    ),
            )
    }

    fn takeover_modal(
        &mut self,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let Some(bot) = self.current_bot().cloned() else {
            return div();
        };
        let viewport = window.viewport_size();
        let width = (f32::from(viewport.width) * 0.86).clamp(400., 1280.);
        let width = width.min((f32::from(viewport.height) - 150.) / 0.625);
        let cursor = if self.cursor_bot == Some(bot.id) {
            (self.cursor.0.tick(window), self.cursor.1.tick(window))
        } else {
            bot.computer.cursor
        };
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(54.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(16.))
                    .child(kit::avatar(
                        crate::theme::bot_color(bot.color),
                        bot.shape,
                        24.,
                        false,
                    ))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!("{}'s computer", bot.name)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .px(px(10.))
                            .py(px(4.))
                            .rounded_full()
                            .bg(t.warning.opacity(0.15))
                            .text_size(px(13.))
                            .text_color(t.text)
                            .child(icon("icons/hand.svg", 13., t.warning))
                            .child(format!("You're in control \u{00b7} {} is paused", bot.name)),
                    )
                    .child(div().flex_1())
                    .child(
                        kit::secondary_button("teach-from-takeover", "Teach a task", t).on_click(
                            cx.listener(|this, _, _, cx| this.open_modal(Modal::Teach, cx)),
                        ),
                    )
                    .child(
                        kit::primary_button("hand-back", "Hand back", t, true).on_click(
                            cx.listener(|this, _, window, cx| this.hand_back(window, cx)),
                        ),
                    ),
            )
            .child(
                div()
                    .id("takeover-screen")
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                            // Clicking the screen moves the pointer there: you are driving.
                            let viewport = window.viewport_size();
                            let left = (f32::from(viewport.width) - width) / 2.;
                            let top = (f32::from(viewport.height) - width * 0.625) / 2. + 27.;
                            let x = ((f32::from(e.position.x) - left) / width).clamp(0., 1.);
                            let y =
                                ((f32::from(e.position.y) - top) / (width * 0.625)).clamp(0., 1.);
                            this.cursor.0.animate_to(x, 260, anim::ease_out_cubic);
                            this.cursor.1.animate_to(y, 260, anim::ease_out_cubic);
                            if let Some(b) = this.current_bot_mut() {
                                b.computer.cursor = (x, y);
                            }
                            cx.notify();
                        }),
                    )
                    .child(computer::screen(
                        "takeover",
                        &bot.computer,
                        cursor,
                        width,
                        false,
                        t,
                    )),
            )
    }

    fn teach_modal(
        &mut self,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        window.request_animation_frame();
        let secs = self.teach.map(|s| s.elapsed().as_secs()).unwrap_or(0);
        let name = self
            .current_bot()
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "your Bot".into());
        div()
            .w(px(460.))
            .p(px(28.))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.))
            .child(
                div().size(px(64.)).rounded_full().bg(t.danger.opacity(0.12)).flex().items_center().justify_center().child(
                    div().size(px(22.)).rounded_full().bg(t.danger).with_animation(
                        "teach-rec",
                        Animation::new(Duration::from_millis(1100)).repeat().with_easing(gpui::pulsating_between(0.45, 1.0)),
                        |this, t| this.opacity(t),
                    ),
                ),
            )
            .child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).child("Teach a task"))
            .child(
                div()
                    .text_center()
                    .text_size(px(15.))
                    .line_height(px(23.))
                    .text_color(t.text_muted)
                    .child(format!("Do the task once on the computer while {name} watches. It saves the steps as a skill it can repeat. Up to 10 minutes; the microphone is off.")),
            )
            .child(div().font_family(MONO).text_size(px(28.)).child(format!("{}:{:02} / 10:00", secs / 60, secs % 60)))
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .pt(px(6.))
                    .child(kit::secondary_button("teach-cancel", "Cancel", t).on_click(cx.listener(|this, _, window, cx| this.close_modal(window, cx))))
                    .child(kit::primary_button("teach-save", "Stop and save skill", t, true).on_click(cx.listener(|this, _, _, cx| this.finish_teach(cx)))),
            )
    }
}

fn confirm(
    title: &str,
    detail: &str,
    action: &str,
    t: &Theme,
    cx: &mut Context<AppView>,
    on_confirm: impl Fn(&mut AppView, &mut Window, &mut Context<AppView>) + 'static,
) -> gpui::Div {
    div()
        .w(px(420.))
        .p(px(24.))
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(
            div()
                .text_size(px(19.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(15.))
                .line_height(px(22.))
                .text_color(t.text_muted)
                .child(detail.to_string()),
        )
        .child(
            div()
                .pt(px(12.))
                .flex()
                .justify_end()
                .gap(px(8.))
                .child(
                    kit::secondary_button("confirm-cancel", "Cancel", t)
                        .on_click(cx.listener(|this, _, window, cx| this.close_modal(window, cx))),
                )
                .child(
                    kit::primary_button("confirm-ok", action.to_string(), t, true)
                        .bg(t.danger)
                        .text_color(gpui::white())
                        .on_click(
                            cx.listener(move |this, _, window, cx| on_confirm(this, window, cx)),
                        ),
                ),
        )
}
