//! The roster: your Bots, newest work first, with their faces and status.

use gpui::{
    Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString, Window, div,
    point, prelude::*, px,
};

use super::{
    app::{AppView, Modal, Page, Popover},
    kit::{self, icon},
};
use crate::{
    model::*,
    theme::{Theme, bot_color, theme},
};

pub const SIDEBAR_WIDTH: f32 = 300.;

impl AppView {
    pub fn render_sidebar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let open = self.sidebar.tick(window);
        let roster = self.roster();
        let hidden_count = self.bots.iter().filter(|b| b.hidden).count();
        let empty = self.bots.is_empty();
        let searching = !self.search_cache.is_empty();

        let mut list = div().flex().flex_col().gap(px(2.)).px(px(10.));
        if self.page == Page::NewBot || empty {
            list = list.child(self.draft_row(&t, empty, cx));
        }
        for (ix, id) in roster.iter().enumerate() {
            if let Some(bot) = self.bot(*id).cloned() {
                list = list.child(self.roster_row(&bot, ix, &t, cx));
            }
        }

        let body = if empty {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .child(div().pt(px(6.)).child(list))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(t.text_muted)
                        .text_size(px(17.))
                        .child("No chats yet"),
                )
        } else if roster.is_empty() && searching {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(t.text_muted)
                .child("No matches")
        } else {
            div().flex_1().min_h_0().child(
                div()
                    .id("roster")
                    .size_full()
                    .overflow_y_scroll()
                    .child(list),
            )
        };

        let inner = div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex()
            .flex_col()
            .bg(t.sidebar)
            .child(
                // Title bar: room for the window buttons, then "+".
                div()
                    .h(px(52.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px(px(12.))
                    .child(if empty {
                        div()
                    } else {
                        div().child(kit::icon_button("new-bot", "icons/plus.svg", &t).on_click(
                            cx.listener(|this, _, window, cx| this.start_new_bot(window, cx)),
                        ))
                    }),
            )
            .child(if empty {
                div()
            } else {
                div().px(px(10.)).pb(px(10.)).child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .h(px(38.))
                        .px(px(12.))
                        .rounded(px(10.))
                        .bg(t.field)
                        .child(icon("icons/search.svg", 17., t.text_muted))
                        .child(div().flex_1().child(self.inputs.search.clone())),
                )
            })
            .child(body)
            .child(if hidden_count > 0 {
                let label = if self.show_hidden {
                    "Hide hidden Bots".to_string()
                } else {
                    format!("{hidden_count} hidden \u{00b7} Show")
                };
                div().px(px(20.)).py(px(6.)).child(
                    div()
                        .id("show-hidden")
                        .text_size(px(13.))
                        .text_color(t.text_muted)
                        .cursor_pointer()
                        .hover(|s| s.underline())
                        .child(label)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_hidden = !this.show_hidden;
                            cx.notify();
                        })),
                )
            } else {
                div()
            })
            .child(
                div()
                    .flex_none()
                    .px(px(10.))
                    .pb(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        footer_row("plugins", &t)
                            .child(
                                div()
                                    .size(px(32.))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(t.border)
                                    .bg(t.surface)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(icon("icons/plug.svg", 16., t.text)),
                            )
                            .child("Plugins")
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open_modal(Modal::Plugins, cx)),
                            ),
                    )
                    .child(
                        footer_row("profile", &t)
                            .child(user_avatar(&self.profile.name, 32., &t))
                            .child(self.profile.name.clone())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                                    cx.stop_propagation();
                                    this.open_popover(
                                        Popover::Profile {
                                            at: point(e.position.x, e.position.y),
                                        },
                                        cx,
                                    )
                                }),
                            ),
                    ),
            );

        div()
            .flex_none()
            .h_full()
            .w(px(SIDEBAR_WIDTH * open))
            .overflow_hidden()
            .border_r_1()
            .border_color(t.border)
            .child(inner.opacity(0.3 + 0.7 * open))
    }

    fn draft_row(
        &self,
        t: &Theme,
        first: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let name = self.inputs.bot_name.read(cx).text().trim().to_string();
        let label = if first && name.is_empty() {
            "Create your first Bot".to_string()
        } else if name.is_empty() {
            "New Bot".to_string()
        } else {
            name
        };
        let selected = self.page == Page::NewBot;
        div()
            .id("draft-row")
            .flex()
            .items_center()
            .gap(px(12.))
            .h(px(72.))
            .px(px(12.))
            .rounded(px(14.))
            .when(selected, |d| d.bg(t.sidebar_selected))
            .cursor_pointer()
            .child(kit::avatar(
                bot_color(self.draft_color),
                self.draft_shape,
                44.,
                false,
            ))
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label),
            )
            .on_click(cx.listener(|this, _, window, cx| {
                if this.page != Page::NewBot {
                    this.start_new_bot(window, cx);
                }
            }))
    }

    fn roster_row(
        &self,
        bot: &Bot,
        ix: usize,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = bot.id;
        let selected = self.page == Page::Bot(id);
        let hover = t.hover;
        let preview = match bot.status {
            BotStatus::Working => bot
                .messages
                .iter()
                .rev()
                .find_map(|m| match &m.kind {
                    MessageKind::Activity { label, done: false } => {
                        Some(format!("{label}\u{2026}"))
                    }
                    _ => None,
                })
                .unwrap_or_else(|| "Working\u{2026}".into()),
            _ => bot.preview(),
        };
        let face = kit::avatar(bot_color(bot.color), bot.shape, 44., self.eyes_closed(ix));
        let face = div()
            .relative()
            .size(px(44.))
            .flex_none()
            .child(if bot.status == BotStatus::Working {
                kit::bobbing(SharedString::from(format!("bob-{id}")), face, 4.).into_any_element()
            } else {
                face.into_any_element()
            })
            .child(match bot.status {
                BotStatus::Idle => status_dot(t.online, t, selected),
                BotStatus::NeedsYou => status_dot(t.warning, t, selected),
                BotStatus::Working => div(),
            });
        div()
            .id(SharedString::from(format!("bot-{id}")))
            .flex()
            .items_center()
            .gap(px(12.))
            .h(px(72.))
            .px(px(12.))
            .rounded(px(14.))
            .cursor_pointer()
            .when(selected, |d| d.bg(t.sidebar_selected))
            .when(!selected, move |d| d.hover(move |s| s.bg(hover)))
            .when(bot.hidden, |d| d.opacity(0.55))
            .child(face)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(17.))
                                    .font_weight(if bot.unread {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::MEDIUM
                                    })
                                    .child(bot.name.clone()),
                            )
                            .when(bot.pinned, |d| {
                                d.child(icon("icons/pin.svg", 13., t.text_faint))
                            })
                            .child(
                                div()
                                    .flex_none()
                                    .text_size(px(14.))
                                    .text_color(t.text_muted)
                                    .child(bot.last_time()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(15.))
                                    .text_color(if bot.unread { t.text } else { t.text_muted })
                                    .child(preview),
                            )
                            .when(bot.unread, |d| {
                                d.child(div().flex_none().size(px(9.)).rounded_full().bg(t.accent))
                            }),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| this.open_bot(id, window, cx)))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.open_popover(
                        Popover::BotMenu {
                            bot: id,
                            at: e.position,
                        },
                        cx,
                    );
                }),
            )
    }
}

fn status_dot(color: gpui::Hsla, t: &Theme, selected: bool) -> gpui::Div {
    div()
        .absolute()
        .right(px(-1.))
        .bottom(px(1.))
        .size(px(13.))
        .rounded_full()
        .bg(color)
        .border_2()
        .border_color(if selected {
            t.sidebar_selected
        } else {
            t.sidebar
        })
}

fn footer_row(id: &'static str, t: &Theme) -> gpui::Stateful<gpui::Div> {
    let hover = t.hover;
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(12.))
        .h(px(46.))
        .px(px(12.))
        .rounded(px(12.))
        .text_size(px(17.))
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
}

/// Your own avatar: a circle with your initial.
pub fn user_avatar(name: &str, size: f32, t: &Theme) -> impl IntoElement + use<> {
    let initial = name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    div()
        .flex_none()
        .size(px(size))
        .rounded_full()
        .bg(kit::mix(t.accent, t.text, 0.25))
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white())
        .text_size(px(size * 0.44))
        .font_weight(FontWeight::SEMIBOLD)
        .child(initial)
}
