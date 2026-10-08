//! The conversation: header, messages and the composer.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Context, Focusable, FontWeight, IntoElement, MouseButton,
    MouseDownEvent, PathPromptOptions, ScrollWheelEvent, SharedString, Window, div, point,
    prelude::*, px,
};

use super::{
    app::{AppView, Modal, Page, Panel, Popover, SuggestIcon, SuggestKind},
    computer,
    kit::{self, icon},
    onboarding::glyph_tile,
    rich::{self, Style},
};
use crate::{
    anim,
    model::*,
    theme::{MONO, Theme, bot_color, theme},
};

const COLUMN: f32 = 860.;

impl AppView {
    pub fn render_main(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let body = match self.page {
            Page::NewBot => self.render_new_bot(window, cx).into_any_element(),
            Page::Bot(_) => self.render_chat(window, cx).into_any_element(),
        };
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(t.bg)
            .child(body)
    }

    fn render_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let Some(bot) = self.current_bot().cloned() else {
            return div();
        };
        let viewport = window.viewport_size();
        let main_width = f32::from(viewport.width)
            - super::sidebar::SIDEBAR_WIDTH * self.sidebar.value()
            - super::panels::PANEL_WIDTH * self.panel.t.value();
        self.bubble_max = ((main_width.min(COLUMN) - 48.) * 0.78).max(200.);

        // Follow the bottom smoothly while new text streams in.
        let handle = self.chat_scroll.clone();
        let max = handle.max_offset().height;
        let current = -handle.offset().y;
        let gap = max - current;
        if self.stick_bottom && gap > px(0.5) {
            let step = (gap * 0.28).max(px(1.)).min(gap);
            handle.set_offset(point(px(0.), -(current + step)));
            window.request_animation_frame();
        }
        let show_jump = !self.stick_bottom && gap > px(240.);

        let mut column = div()
            .w_full()
            .max_w(px(COLUMN))
            .mx_auto()
            .px(px(24.))
            .pt(px(18.))
            .pb(px(18.))
            .flex()
            .flex_col();
        let mut previous: Option<&Message> = None;
        for msg in &bot.messages {
            let gap = spacing(previous, msg);
            column = column.child(
                div()
                    .pt(px(gap))
                    .child(self.render_message(&bot, msg, &t, window, cx)),
            );
            previous = Some(msg);
        }
        if bot.status == BotStatus::Working {
            column = column.child(div().pt(px(12.)).h(px(46.)).flex().items_end().child(
                kit::bobbing(
                    SharedString::from(format!("working-{}", bot.id)),
                    kit::avatar(bot_color(bot.color), bot.shape, 30., false),
                    6.,
                ),
            ));
        }

        let list = div()
            .id("chat-scroll")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.chat_scroll)
            .on_scroll_wheel(cx.listener(|this, e: &ScrollWheelEvent, _, cx| {
                let dy = e.delta.pixel_delta(px(20.)).y;
                if dy > px(0.) {
                    this.stick_bottom = false;
                } else {
                    let gap = this.chat_scroll.max_offset().height + this.chat_scroll.offset().y;
                    if gap < px(60.) {
                        this.stick_bottom = true;
                    }
                }
                cx.notify();
            }))
            .child(column);

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.chat_header(&bot, &t, cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(list)
                    .when(show_jump, |d| {
                        d.child(
                            div()
                                .absolute()
                                .bottom(px(12.))
                                .left_0()
                                .right_0()
                                .flex()
                                .justify_center()
                                .child(
                                    div()
                                        .id("jump")
                                        .size(px(38.))
                                        .rounded_full()
                                        .bg(t.surface)
                                        .border_1()
                                        .border_color(t.border)
                                        .shadow(kit::shadow(&t, 4., 12.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .child(icon("icons/arrow-down.svg", 18., t.text))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.stick_bottom = true;
                                            cx.notify();
                                        }))
                                        .with_animation(
                                            "jump-in",
                                            Animation::new(Duration::from_millis(260))
                                                .with_easing(anim::ease_out_quint),
                                            |this, t| this.opacity(t).mt(px(10. * (1. - t))),
                                        ),
                                ),
                        )
                    }),
            )
            .child(self.render_composer(&bot, &t, window, cx))
    }

    fn chat_header(
        &self,
        bot: &Bot,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let computer_on = self.panel.is(&Panel::Computer);
        let details_on = self.panel.is(&Panel::Details);
        div()
            .h(px(52.))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px(px(18.))
            .border_b_1()
            .border_color(t.border)
            .child(
                div()
                    .id("header-bot")
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(6.))
                    .py(px(4.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(kit::avatar(bot_color(bot.color), bot.shape, 26., false))
                    .child(
                        div()
                            .text_size(px(17.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(bot.name.clone()),
                    )
                    .when(!bot.title.is_empty(), |d| {
                        d.child(
                            div()
                                .text_size(px(14.))
                                .text_color(t.text_faint)
                                .child(bot.title.clone()),
                        )
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_panel(Panel::Details, cx))),
            )
            .child(div().flex_1())
            .child(
                div()
                    .relative()
                    .child(
                        kit::icon_button("toggle-computer", "icons/monitor.svg", t)
                            .when(computer_on, |d| d.bg(t.hover))
                            .on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.toggle_panel(Panel::Computer, cx)
                                }),
                            ),
                    )
                    .when(
                        bot.computer.active && bot.status == BotStatus::Working,
                        |d| {
                            d.child(
                                div()
                                    .absolute()
                                    .top(px(5.))
                                    .right(px(5.))
                                    .size(px(7.))
                                    .rounded_full()
                                    .bg(t.online)
                                    .with_animation(
                                        "computer-live",
                                        Animation::new(Duration::from_millis(1200))
                                            .repeat()
                                            .with_easing(gpui::pulsating_between(0.3, 1.0)),
                                        |this, t| this.opacity(t),
                                    ),
                            )
                        },
                    ),
            )
            .child(
                kit::icon_button("toggle-details", "icons/sidebar.svg", t)
                    .when(details_on, |d| d.bg(t.hover))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_panel(Panel::Details, cx))),
            )
    }

    fn render_message(
        &self,
        bot: &Bot,
        msg: &Message,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = msg.id;
        let body: AnyElement = match &msg.kind {
            MessageKind::Stamp(text) => div()
                .flex()
                .justify_center()
                .py(px(10.))
                .text_size(px(14.))
                .text_color(t.text_muted)
                .child(text.clone())
                .into_any_element(),
            MessageKind::Event {
                text,
                icon: glyph,
                link,
                routine,
            } => {
                let routine = *routine;
                let mut row = div()
                    .flex()
                    .justify_center()
                    .items_center()
                    .gap(px(6.))
                    .py(px(6.))
                    .text_size(px(14.))
                    .text_color(t.text_muted)
                    .child(text.clone());
                if glyph.is_some() || link.is_some() {
                    let mut chip = div()
                        .id(SharedString::from(format!("event-{id}")))
                        .flex()
                        .items_center()
                        .gap(px(5.));
                    if let Some(path) = glyph {
                        chip = chip.child(
                            gpui::svg()
                                .path(SharedString::from(path.clone()))
                                .size(px(14.))
                                .text_color(t.text_muted),
                        );
                    }
                    if let Some(link) = link {
                        chip =
                            chip.child(div().font_weight(FontWeight::MEDIUM).child(link.clone()));
                    }
                    if let Some(rid) = routine {
                        chip =
                            chip.cursor_pointer()
                                .hover(|s| s.underline())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_panel(Panel::Routine(rid), cx)
                                }));
                    }
                    row = row.child(chip);
                }
                row.into_any_element()
            }
            MessageKind::User { text, attachments } => {
                self.user_bubble(msg, text, attachments, t, cx)
            }
            MessageKind::Bot { text, streaming } => self.bot_bubble(msg, text, *streaming, t, cx),
            MessageKind::Activity { label, done } => div()
                .flex()
                .items_center()
                .gap(px(8.))
                .py(px(2.))
                .pl(px(4.))
                .text_size(px(14.))
                .text_color(t.text_muted)
                .child(if *done {
                    icon("icons/check.svg", 14., t.online).into_any_element()
                } else {
                    kit::spinner(SharedString::from(format!("act-{id}")), 14., t.text_muted)
                        .into_any_element()
                })
                .child(rich::inline(
                    SharedString::from(format!("act-text-{id}")).into(),
                    label,
                    &Style::new(14., 20., t.text_muted),
                    t,
                ))
                .into_any_element(),
            MessageKind::File {
                name,
                size,
                content,
            } => self.file_card(id, name, size, content, t, cx),
            MessageKind::Question {
                prompt,
                placeholder,
                answer,
            } => self.question_card(
                bot,
                id,
                prompt,
                placeholder,
                answer.as_deref(),
                t,
                window,
                cx,
            ),
            MessageKind::Approval {
                action,
                detail,
                state,
            } => self.approval_card(id, action, detail, *state, t, cx),
            MessageKind::Snapshot { title, url } => self.snapshot_card(bot, id, title, url, t, cx),
        };
        if msg.fresh {
            let from_right = matches!(msg.kind, MessageKind::User { .. });
            div()
                .relative()
                .child(body)
                .with_animation(
                    SharedString::from(format!("in-{id}")),
                    Animation::new(Duration::from_millis(380)).with_easing(anim::ease_out_quint),
                    move |this, t| {
                        let this = this.opacity(t).top(px(10. * (1. - t)));
                        if from_right {
                            this.left(px(8. * (1. - t)))
                        } else {
                            this
                        }
                    },
                )
                .into_any_element()
        } else {
            body
        }
    }

    fn hover_actions(
        &self,
        msg: &Message,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = msg.id;
        let quote = match &msg.kind {
            MessageKind::User { text, .. } | MessageKind::Bot { text, .. } => text.clone(),
            _ => String::new(),
        };
        let visible = self.hovered_msg == Some(id)
            || matches!(self.popover.target, Some(Popover::React { msg, .. } | Popover::MessageMore { msg, .. }) if msg == id);
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .px(px(6.))
            .when(!visible, |d| d.invisible())
            .child(
                kit::icon_button(
                    SharedString::from(format!("react-{id}")),
                    "icons/smile.svg",
                    t,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                        cx.stop_propagation();
                        this.open_popover(
                            Popover::React {
                                msg: id,
                                at: e.position,
                            },
                            cx,
                        )
                    }),
                ),
            )
            .child(
                kit::icon_button(
                    SharedString::from(format!("reply-{id}")),
                    "icons/reply.svg",
                    t,
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    let line: String = quote
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .chars()
                        .take(120)
                        .collect();
                    this.reply_to = Some((id, line));
                    window.focus(&this.inputs.composer.focus_handle(cx));
                    cx.notify();
                })),
            )
            .child(
                kit::icon_button(
                    SharedString::from(format!("more-{id}")),
                    "icons/more.svg",
                    t,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                        cx.stop_propagation();
                        this.open_popover(
                            Popover::MessageMore {
                                msg: id,
                                at: e.position,
                            },
                            cx,
                        )
                    }),
                ),
            )
    }

    fn reactions(
        &self,
        msg: &Message,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = msg.id;
        div()
            .flex()
            .gap(px(4.))
            .pt(px(4.))
            .children(msg.reactions.iter().map(|r| {
                let r = *r;
                div()
                    .id(SharedString::from(format!("reaction-{id}-{r:?}")))
                    .flex()
                    .items_center()
                    .h(px(24.))
                    .px(px(8.))
                    .rounded_full()
                    .bg(t.field)
                    .border_1()
                    .border_color(t.border)
                    .cursor_pointer()
                    .child(icon(
                        r.icon(),
                        13.,
                        if r == Reaction::Heart {
                            t.danger
                        } else {
                            t.text
                        },
                    ))
                    .on_click(cx.listener(move |this, _, _, cx| this.toggle_reaction(id, r, cx)))
                    .with_animation(
                        SharedString::from(format!("pop-{id}-{r:?}")),
                        Animation::new(Duration::from_millis(300))
                            .with_easing(anim::ease_out_quint),
                        |this, t| {
                            let s = anim::ease_out_back(t);
                            this.opacity(t).w(px(36. * s.max(0.3)))
                        },
                    )
            }))
    }

    fn user_bubble(
        &self,
        msg: &Message,
        text: &str,
        attachments: &[String],
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = msg.id;
        let mut bubble = div()
            .max_w(px(self.bubble_max * 1.03))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(6.));
        if let Some(quote) = &msg.reply_to {
            bubble = bubble.child(
                div()
                    .max_w(px(420.))
                    .truncate()
                    .text_size(px(13.))
                    .text_color(t.text_muted)
                    .pl(px(10.))
                    .border_l_2()
                    .border_color(t.border_strong)
                    .child(quote.clone()),
            );
        }
        for file in attachments {
            bubble = bubble.child(attachment_chip(file, t));
        }
        if !text.is_empty() {
            bubble = bubble.child(
                div()
                    .px(px(18.))
                    .py(px(12.))
                    .rounded(px(22.))
                    .bg(t.user_bubble)
                    .child(rich::markdown(
                        &format!("u{id}"),
                        text,
                        &Style::new(16., 26., t.user_text),
                        t,
                    )),
            );
        }
        div()
            .id(SharedString::from(format!("msg-{id}")))
            .flex()
            .justify_end()
            .items_center()
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    this.hovered_msg = Some(id);
                } else if this.hovered_msg == Some(id) {
                    this.hovered_msg = None;
                }
                cx.notify();
            }))
            .child(self.hover_actions(msg, t, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .child(bubble)
                    .child(self.reactions(msg, t, cx)),
            )
            .into_any_element()
    }

    fn bot_bubble(
        &self,
        msg: &Message,
        text: &str,
        streaming: bool,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = msg.id;
        let content = if text.is_empty() && streaming {
            kit::typing_dots(&format!("dots-{id}"), t.text_muted).into_any_element()
        } else {
            rich::markdown(&format!("b{id}"), text, &Style::new(16., 26., t.text), t)
                .into_any_element()
        };
        div()
            .id(SharedString::from(format!("msg-{id}")))
            .flex()
            .items_center()
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if *hovered {
                    this.hovered_msg = Some(id);
                } else if this.hovered_msg == Some(id) {
                    this.hovered_msg = None;
                }
                cx.notify();
            }))
            .child(
                div()
                    .max_w(px(self.bubble_max))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(18.))
                            .py(px(12.))
                            .rounded(px(22.))
                            .bg(t.bot_bubble)
                            .child(content),
                    )
                    .child(self.reactions(msg, t, cx)),
            )
            .child(self.hover_actions(msg, t, cx))
            .into_any_element()
    }

    fn file_card(
        &self,
        id: MsgId,
        name: &str,
        size: &str,
        content: &str,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hover = t.hover;
        let (n1, c1, n2, c2) = (
            name.to_string(),
            content.to_string(),
            name.to_string(),
            content.to_string(),
        );
        div()
            .flex()
            .child(
                div()
                    .id(SharedString::from(format!("file-{id}")))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .w(px(320.))
                    .p(px(12.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(t.border)
                    .bg(t.surface)
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .child(
                        div()
                            .size(px(42.))
                            .rounded(px(9.))
                            .bg(t.accent.opacity(0.12))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(t.accent)
                            .font_family(MONO)
                            .child(if name.ends_with(".md") {
                                "M\u{2193}"
                            } else {
                                "FILE"
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(name.to_string()),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(t.text_muted)
                                    .child(size.to_string()),
                            ),
                    )
                    .child(
                        kit::icon_button(
                            SharedString::from(format!("dl-{id}")),
                            "icons/download.svg",
                            t,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.download_file(&n1, &c1, cx)
                        })),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_modal(
                            Modal::File {
                                name: n2.clone(),
                                content: c2.clone(),
                            },
                            cx,
                        )
                    })),
            )
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn question_card(
        &self,
        bot: &Bot,
        id: MsgId,
        prompt: &str,
        placeholder: &str,
        answer: Option<&str>,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let latest_open = bot
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.kind, MessageKind::Question { answer: None, .. }))
            .map(|m| m.id)
            == Some(id);
        if latest_open {
            let placeholder = placeholder.to_string();
            self.inputs
                .answer
                .update(cx, |i, cx| i.set_placeholder(placeholder, cx));
        }
        let focused = self.inputs.answer.read(cx).is_focused(window);
        let footer: AnyElement = match answer {
            Some(answer) => div()
                .flex()
                .items_center()
                .gap(px(8.))
                .text_size(px(15.))
                .text_color(t.text_muted)
                .child(icon("icons/check.svg", 15., t.online))
                .child(answer.to_string())
                .into_any_element(),
            None if latest_open => div()
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(46.))
                .pl(px(12.))
                .pr(px(6.))
                .rounded(px(12.))
                .border_1()
                .border_color(if focused { t.accent } else { t.border_strong })
                .bg(t.bg)
                .child(
                    div()
                        .size(px(22.))
                        .rounded(px(5.))
                        .bg(t.field)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.))
                        .text_color(t.text_muted)
                        .child("A"),
                )
                .child(div().flex_1().child(self.inputs.answer.clone()))
                .child(
                    kit::icon_button(
                        SharedString::from(format!("answer-{id}")),
                        "icons/check.svg",
                        t,
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.answer_question(Some(id), window, cx)
                    })),
                )
                .into_any_element(),
            None => div()
                .text_size(px(14.))
                .text_color(t.text_faint)
                .child("Waiting for an answer")
                .into_any_element(),
        };
        div()
            .max_w(px(self.bubble_max))
            .flex()
            .flex_col()
            .gap(px(12.))
            .px(px(18.))
            .py(px(14.))
            .rounded(px(22.))
            .bg(t.bot_bubble)
            .child(rich::markdown(
                &format!("q{id}"),
                prompt,
                &Style::new(16., 26., t.text),
                t,
            ))
            .child(footer)
            .into_any_element()
    }

    fn approval_card(
        &self,
        id: MsgId,
        action: &str,
        detail: &str,
        state: ApprovalState,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let footer: AnyElement = match state {
            ApprovalState::Pending => div()
                .flex()
                .gap_2()
                .child(
                    kit::secondary_button(SharedString::from(format!("deny-{id}")), "Deny", t)
                        .on_click(cx.listener(move |this, _, _, cx| this.decide(id, false, cx))),
                )
                .child(
                    kit::primary_button(
                        SharedString::from(format!("approve-{id}")),
                        "Approve",
                        t,
                        true,
                    )
                    .h(px(34.))
                    .on_click(cx.listener(move |this, _, _, cx| this.decide(id, true, cx))),
                )
                .into_any_element(),
            ApprovalState::Approved => status_line("icons/check.svg", "Approved", t.online, t),
            ApprovalState::Denied => status_line("icons/x.svg", "Denied", t.danger, t),
        };
        div()
            .w(px(440f32.min(self.bubble_max)))
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(16.))
            .rounded(px(18.))
            .border_1()
            .border_color(if state == ApprovalState::Pending {
                t.warning.opacity(0.6)
            } else {
                t.border
            })
            .bg(t.surface)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(13.))
                    .text_color(t.text_muted)
                    .child(icon("icons/shield.svg", 15., t.warning))
                    .child("Needs your approval"),
            )
            .child(
                div()
                    .w_full()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(action.to_string()),
            )
            .child(
                div()
                    .w_full()
                    .text_size(px(15.))
                    .line_height(px(22.))
                    .text_color(t.text_muted)
                    .child(detail.to_string()),
            )
            .child(footer)
            .into_any_element()
    }

    fn snapshot_card(
        &self,
        bot: &Bot,
        id: MsgId,
        title: &str,
        url: &str,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut state = bot.computer.clone();
        state.url = format!("https://{url}");
        state.page_title = title.to_string();
        state.status = String::new();
        div()
            .id(SharedString::from(format!("snap-{id}")))
            .w(px(420.))
            .rounded(px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(t.border)
            .cursor_pointer()
            .child(computer::screen(
                &format!("snap{id}"),
                &state,
                (0.5, 0.5),
                420.,
                false,
                t,
            ))
            .on_click(cx.listener(|this, _, _, cx| this.open_panel(Panel::Computer, cx)))
            .into_any_element()
    }

    fn render_composer(
        &mut self,
        bot: &Bot,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let text_empty = self.inputs.composer.read(cx).text().is_empty();
        let working = bot.status == BotStatus::Working;
        let long = {
            let text = self.inputs.composer.read(cx).text();
            text.contains('\n') || text.len() > 70
        };
        let suggest = self.render_suggest(t, cx);
        let plus = div()
            .id("composer-plus")
            .flex_none()
            .size(px(34.))
            .rounded_full()
            .bg(t.field)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.opacity(0.75))
            .child(icon("icons/plus.svg", 18., t.text))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.open_popover(Popover::Plus { at: e.position }, cx)
                }),
            );
        let mic_dark = text_empty && !working;
        let mic = div()
            .id("composer-mic")
            .flex_none()
            .size(px(38.))
            .rounded_full()
            .bg(if mic_dark { t.primary } else { t.field })
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.opacity(0.85))
            .child(icon(
                "icons/mic.svg",
                18.,
                if mic_dark { t.primary_text } else { t.text },
            ))
            .on_click(cx.listener(|this, _, window, cx| this.toggle_voice(window, cx)));
        let send = div()
            .id("composer-send")
            .flex_none()
            .size(px(38.))
            .rounded_full()
            .bg(t.primary)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.opacity(0.85))
            .child(icon("icons/arrow-up.svg", 19., t.primary_text))
            .on_click(cx.listener(|this, _, window, cx| this.send_message(window, cx)))
            .with_animation(
                "send-in",
                Animation::new(Duration::from_millis(220)).with_easing(anim::ease_out_quint),
                |this, t| {
                    let s = anim::ease_out_back(t);
                    this.opacity(t).size(px(38. * (0.6 + 0.4 * s)))
                },
            );
        let stop = div()
            .id("composer-stop")
            .flex_none()
            .size(px(38.))
            .rounded_full()
            .bg(t.primary)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .child(div().size(px(12.)).rounded(px(3.)).bg(t.primary_text))
            .on_click(cx.listener(|this, _, _, cx| this.stop_bot(cx)));

        let buttons = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .when(text_empty && working, |d| d.child(stop))
            .when(!(text_empty && working), |d| d.child(mic))
            .when(!text_empty, |d| d.child(send));

        let mut extras = div().flex().flex_col().gap(px(8.));
        let mut has_extras = false;
        if let Some((_, quote)) = &self.reply_to {
            has_extras = true;
            extras = extras.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(14.))
                    .text_color(t.text_muted)
                    .child(icon("icons/reply.svg", 14., t.text_muted))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(format!("Replying to \u{201c}{quote}\u{201d}")),
                    )
                    .child(kit::icon_button("clear-reply", "icons/x.svg", t).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.reply_to = None;
                            cx.notify();
                        }),
                    )),
            );
        }
        if !self.attachments.is_empty() {
            has_extras = true;
            extras = extras.child(div().flex().flex_wrap().gap(px(6.)).children(
                self.attachments.iter().enumerate().map(|(i, f)| {
                    div()
                        .id(SharedString::from(format!("att-{i}")))
                        .child(attachment_chip(f, t))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.attachments.remove(i);
                            cx.notify();
                        }))
                }),
            ));
        }

        let inner = if let Some(started) = self.voice {
            window.request_animation_frame();
            self.voice_row(started.elapsed().as_secs(), t, cx)
                .into_any_element()
        } else if long {
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .px(px(8.))
                        .pt(px(6.))
                        .child(self.inputs.composer.clone()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(plus)
                        .child(div().flex_1())
                        .child(buttons),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(plus)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .py(px(4.))
                        .child(self.inputs.composer.clone()),
                )
                .child(buttons)
                .into_any_element()
        };

        let focused = self.inputs.composer.read(cx).is_focused(window);
        div()
            .flex_none()
            .w_full()
            .max_w(px(COLUMN))
            .mx_auto()
            .px(px(20.))
            .pb(px(18.))
            .relative()
            .child(suggest)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(8.))
                    .rounded(px(28.))
                    .border_1()
                    .border_color(if focused {
                        t.border_strong
                    } else {
                        t.field_border
                    })
                    .bg(t.surface)
                    .shadow(kit::shadow(t, 2., 10.))
                    .when(has_extras, |d| {
                        d.child(div().px(px(10.)).pt(px(4.)).child(extras))
                    })
                    .child(inner),
            )
    }

    fn voice_row(&self, secs: u64, t: &Theme, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let bars = (0..28).map(|i| {
            div().w(px(3.)).rounded_full().bg(t.text).with_animation(
                SharedString::from(format!("wave-{i}")),
                Animation::new(Duration::from_millis(700 + (i * 53 % 400) as u64))
                    .repeat()
                    .with_easing(gpui::bounce(anim::ease_in_out_cubic)),
                move |this, v| {
                    let seed = ((i * 37) % 11) as f32 / 11.;
                    this.h(px(4. + 18. * (0.25 + 0.75 * seed) * v))
                },
            )
        });
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .h(px(38.))
            .pl(px(10.))
            .child(
                div()
                    .size(px(10.))
                    .rounded_full()
                    .bg(t.danger)
                    .with_animation(
                        "rec-dot",
                        Animation::new(Duration::from_millis(1000))
                            .repeat()
                            .with_easing(gpui::pulsating_between(0.35, 1.0)),
                        |this, t| this.opacity(t),
                    ),
            )
            .child(
                div()
                    .text_size(px(15.))
                    .text_color(t.text_muted)
                    .font_family(MONO)
                    .child(format!("{}:{:02}", secs / 60, secs % 60)),
            )
            .child(
                div()
                    .flex_1()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(3.))
                    .children(bars),
            )
            .child(
                kit::icon_button("voice-cancel", "icons/x.svg", t)
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_voice(cx))),
            )
            .child(
                div()
                    .id("voice-done")
                    .size(px(38.))
                    .rounded_full()
                    .bg(t.primary)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child(icon("icons/check.svg", 18., t.primary_text))
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_voice(window, cx))),
            )
    }

    fn render_suggest(&self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let Some(s) = self.suggest.clone() else {
            return div().into_any_element();
        };
        let items = self.suggest_items(&s);
        if items.is_empty() {
            return div().into_any_element();
        }
        let mut list = kit::popover_card(t).w(px(420.)).max_h(px(360.));
        let mut last_section = "";
        for (i, item) in items.iter().enumerate() {
            if item.section != last_section {
                last_section = item.section;
                list = list.child(
                    div()
                        .px(px(10.))
                        .pt(px(8.))
                        .pb(px(4.))
                        .text_size(px(12.))
                        .text_color(t.text_faint)
                        .child(item.section),
                );
            }
            let selected = i == s.ix;
            let lead: AnyElement = match &item.icon {
                SuggestIcon::Bot(color, shape) => {
                    kit::avatar(bot_color(*color), *shape, 24., false).into_any_element()
                }
                SuggestIcon::Path(path) => div()
                    .size(px(24.))
                    .rounded(px(6.))
                    .bg(t.field)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon(path, 14., t.text_muted))
                    .into_any_element(),
                SuggestIcon::Glyph(g, c) => glyph_tile(g, *c, 24.).into_any_element(),
            };
            let hover = t.hover;
            list = list.child(
                div()
                    .id(SharedString::from(format!("sugg-{i}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(40.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .when(selected, |d| d.bg(t.hover))
                    .hover(move |st| st.bg(hover))
                    .child(lead)
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(15.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(item.label.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(13.))
                            .text_color(t.text_muted)
                            .child(item.detail.clone()),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.accept_suggest(Some(i), window, cx)
                    })),
            );
        }
        let hint = match s.kind {
            SuggestKind::Mention => "Mention a Bot, routine or connector",
            SuggestKind::Skill => "Use a skill",
        };
        div()
            .absolute()
            .bottom(px(84.))
            .left(px(24.))
            .child(
                list.child(
                    div()
                        .px(px(10.))
                        .py(px(6.))
                        .text_size(px(12.))
                        .text_color(t.text_faint)
                        .child(format!(
                            "{hint} \u{00b7} \u{2191}\u{2193} to move, Enter to pick"
                        )),
                )
                .with_animation(
                    SharedString::from(format!("suggest-{:?}-{}", s.kind, s.start)),
                    Animation::new(Duration::from_millis(200)).with_easing(anim::ease_out_quint),
                    |this, t| this.opacity(t).mt(px(8. * (1. - t))),
                ),
            )
            .into_any_element()
    }

    pub fn pick_files(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: None,
        });
        cx.spawn(async move |this, cx| {
            let picked = receiver.await.ok().and_then(|r| r.ok()).flatten();
            let _ = this.update(cx, |this, cx| {
                match picked {
                    Some(paths) => {
                        for p in paths {
                            if let Some(name) = p.file_name() {
                                this.attachments.push(name.to_string_lossy().into_owned());
                            }
                        }
                    }
                    None => {
                        let n = this.attachments.len() + 1;
                        this.attachments.push(format!("notes-{n}.pdf"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn spacing(previous: Option<&Message>, msg: &Message) -> f32 {
    let Some(prev) = previous else { return 0. };
    let same_side = matches!(
        (&prev.kind, &msg.kind),
        (MessageKind::Bot { .. }, MessageKind::Bot { .. })
            | (MessageKind::User { .. }, MessageKind::User { .. })
            | (MessageKind::Activity { .. }, MessageKind::Activity { .. })
    );
    if same_side { 6. } else { 14. }
}

fn status_line(path: &'static str, label: &str, color: gpui::Hsla, t: &Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_size(px(14.))
        .text_color(t.text_muted)
        .child(icon(path, 14., color))
        .child(label.to_string())
        .into_any_element()
}

pub fn attachment_chip(name: &str, t: &Theme) -> impl IntoElement + use<> {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .h(px(30.))
        .px(px(10.))
        .rounded(px(8.))
        .bg(t.field)
        .text_size(px(13.))
        .child(icon("icons/paperclip.svg", 13., t.text_muted))
        .child(name.to_string())
}
