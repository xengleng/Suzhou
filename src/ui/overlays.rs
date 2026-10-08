//! Things drawn above everything else: the modal layer, popovers and toasts.

use gpui::{
    AnyElement, Context, Focusable, FontWeight, IntoElement, MouseButton, MouseDownEvent,
    SharedString, Window, div, prelude::*, px,
};

use super::{
    app::{AppView, Modal, Panel, Popover},
    kit::{self, icon},
};
use crate::{
    model::*,
    theme::{ActiveTheme, ThemeMode, bot_color, theme},
};

impl AppView {
    pub fn render_modal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let Some((modal, progress)) = self.modal.visible(window) else {
            return div().id("modal-none");
        };
        let body = self.modal_body(&modal, window, cx);
        let takeover = modal == Modal::Takeover;
        let card = div()
            .id("modal-card")
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .relative()
            .top(px(18. * (1. - progress)))
            .opacity(progress)
            .rounded(px(18.))
            .overflow_hidden()
            .when(!takeover, |d| {
                d.bg(t.surface)
                    .border_1()
                    .border_color(t.border)
                    .shadow(kit::shadow(&t, 24., 60.))
            })
            .when(takeover, |d| {
                d.bg(t.surface).shadow(kit::shadow(&t, 24., 60.))
            })
            .child(body);
        div()
            .id("modal-layer")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .occlude()
            .bg(gpui::Hsla {
                a: t.scrim.a * progress * if takeover { 1.6 } else { 1.0 },
                ..t.scrim
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.modal.target != Some(Modal::Takeover)
                        && this.modal.target != Some(Modal::Teach)
                    {
                        this.close_modal(window, cx);
                    }
                }),
            )
            .child(card)
    }

    pub fn render_popover(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let Some((popover, progress)) = self.popover.visible(window) else {
            return div().id("popover-none");
        };
        let viewport = window.viewport_size();
        let (card, at, upward, width): (AnyElement, gpui::Point<gpui::Pixels>, bool, f32) =
            match popover {
                Popover::BotMenu { bot, at } => {
                    let Some(b) = self.bot(bot).cloned() else {
                        return div().id("popover-none");
                    };
                    let card = kit::popover_card(&t)
                        .w(px(220.))
                        .child(
                            kit::menu_item(
                                "m-open",
                                "icons/sidebar.svg",
                                "Edit profile",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.close_popover(cx);
                                    this.open_bot(bot, window, cx);
                                    this.editing_profile = true;
                                    this.open_panel(Panel::Details, cx);
                                    this.sync_panel_inputs_public(cx);
                                },
                            )),
                        )
                        .child(
                            kit::menu_item(
                                "m-pin",
                                "icons/pin.svg",
                                if b.pinned { "Unpin" } else { "Pin to top" },
                                &t,
                                false,
                            )
                            .on_click(cx.listener(move |this, _, _, cx| this.toggle_pin(bot, cx))),
                        )
                        .child(
                            kit::menu_item("m-dup", "icons/copy.svg", "Duplicate", &t, false)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.duplicate_bot(bot, window, cx)
                                })),
                        )
                        .child(
                            kit::menu_item(
                                "m-hide",
                                "icons/eye-off.svg",
                                if b.hidden { "Unhide" } else { "Hide" },
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                move |this, _, window, cx| this.toggle_hidden(bot, window, cx),
                            )),
                        )
                        .child(div().my(px(4.)).child(kit::divider(&t)))
                        .child(
                            kit::menu_item("m-del", "icons/trash.svg", "Delete\u{2026}", &t, true)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_modal(Modal::ConfirmDelete(bot), cx)
                                })),
                        )
                        .into_any_element();
                    (card, at, false, 220.)
                }
                Popover::Profile { at } => {
                    let dark = cx.global::<ActiveTheme>().is_dark();
                    let card = kit::popover_card(&t)
                        .w(px(240.))
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(8.))
                                .text_size(px(13.))
                                .text_color(t.text_muted)
                                .child(format!("Signed in as {}", self.profile.name)),
                        )
                        .child(
                            kit::menu_item(
                                "p-settings",
                                "icons/settings.svg",
                                "Settings",
                                &t,
                                false,
                            )
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open_modal(Modal::Settings, cx)),
                            ),
                        )
                        .child(
                            kit::menu_item("p-plugins", "icons/plug.svg", "Plugins", &t, false)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.open_modal(Modal::Plugins, cx)
                                })),
                        )
                        .child(
                            kit::menu_item(
                                "p-theme",
                                if dark {
                                    "icons/sun.svg"
                                } else {
                                    "icons/moon.svg"
                                },
                                if dark { "Light mode" } else { "Dark mode" },
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    cx.global_mut::<ActiveTheme>().mode = if dark {
                                        ThemeMode::Light
                                    } else {
                                        ThemeMode::Dark
                                    };
                                    this.close_popover(cx);
                                    this.save(cx);
                                },
                            )),
                        )
                        .child(div().my(px(4.)).child(kit::divider(&t)))
                        .child(
                            kit::menu_item("p-signout", "icons/logout.svg", "Sign out", &t, false)
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.sign_out(window, cx)),
                                ),
                        )
                        .into_any_element();
                    (card, at, true, 240.)
                }
                Popover::Plus { at } => {
                    let card = kit::popover_card(&t)
                        .w(px(250.))
                        .child(
                            kit::menu_item(
                                "plus-file",
                                "icons/paperclip.svg",
                                "Attach files",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_popover(cx);
                                this.pick_files(cx);
                            })),
                        )
                        .child(
                            kit::menu_item(
                                "plus-teach",
                                "icons/sparkles.svg",
                                "Teach a task",
                                &t,
                                false,
                            )
                            .on_click(
                                cx.listener(|this, _, _, cx| this.open_modal(Modal::Teach, cx)),
                            ),
                        )
                        .child(
                            kit::menu_item(
                                "plus-computer",
                                "icons/monitor.svg",
                                "Agent Computer",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_popover(cx);
                                this.open_panel(Panel::Computer, cx);
                            })),
                        )
                        .child(
                            kit::menu_item(
                                "plus-skill",
                                "icons/slash.svg",
                                "Use a skill",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.close_popover(cx);
                                    this.inputs.composer.update(cx, |i, cx| i.set_text("/", cx));
                                    window.focus(&this.inputs.composer.focus_handle(cx));
                                },
                            )),
                        )
                        .child(
                            kit::menu_item(
                                "plus-mention",
                                "icons/at.svg",
                                "Mention a Bot or app",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.close_popover(cx);
                                    this.inputs.composer.update(cx, |i, cx| {
                                        let mut text = i.text().to_string();
                                        if !text.is_empty() && !text.ends_with(' ') {
                                            text.push(' ');
                                        }
                                        text.push('@');
                                        i.set_text(text, cx)
                                    });
                                    window.focus(&this.inputs.composer.focus_handle(cx));
                                },
                            )),
                        )
                        .child(
                            kit::menu_item(
                                "plus-voice",
                                "icons/mic.svg",
                                "Start voice input",
                                &t,
                                false,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.close_popover(cx);
                                    this.toggle_voice(window, cx);
                                },
                            )),
                        )
                        .child(div().my(px(4.)).child(kit::divider(&t)))
                        .child(
                            kit::menu_item("plus-plugins", "icons/plug.svg", "Plugins", &t, false)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.open_modal(Modal::Plugins, cx)
                                })),
                        )
                        .into_any_element();
                    (card, at, true, 250.)
                }
                Popover::MessageMore { msg, at } => {
                    let text = self
                        .current_bot()
                        .and_then(|b| b.messages.iter().find(|m| m.id == msg))
                        .and_then(|m| match &m.kind {
                            MessageKind::User { text, .. } | MessageKind::Bot { text, .. } => {
                                Some(text.clone())
                            }
                            _ => None,
                        })
                        .unwrap_or_default();
                    let card = kit::popover_card(&t)
                        .w(px(220.))
                        .child(
                            kit::menu_item("mm-copy", "icons/copy.svg", "Copy text", &t, false)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                        text.clone(),
                                    ));
                                    this.close_popover(cx);
                                })),
                        )
                        .child(
                            kit::menu_item(
                                "mm-skill",
                                "icons/book.svg",
                                "Save the process as skill",
                                &t,
                                false,
                            )
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.save_as_skill(msg, cx)),
                            ),
                        )
                        .child(div().my(px(4.)).child(kit::divider(&t)))
                        .child(
                            kit::menu_item("mm-del", "icons/trash.svg", "Delete", &t, true)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.delete_message(msg, cx)),
                                ),
                        )
                        .into_any_element();
                    (card, at, false, 220.)
                }
                Popover::React { msg, at } => {
                    let card = kit::popover_card(&t)
                        .flex_row()
                        .gap(px(2.))
                        .children(Reaction::ALL.iter().map(|r| {
                            let r = *r;
                            kit::icon_button(SharedString::from(format!("r-{r:?}")), r.icon(), &t)
                                .size(px(36.))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.toggle_reaction(msg, r, cx)
                                }))
                        }))
                        .into_any_element();
                    (card, at, true, 160.)
                }
            };

        let x = (f32::from(at.x) - if upward { 10. } else { 4. })
            .clamp(8., (f32::from(viewport.width) - width - 8.).max(8.));
        let mut positioned = div().absolute().left(px(x)).opacity(progress);
        positioned = if upward {
            positioned.bottom(px(
                f32::from(viewport.height) - f32::from(at.y) + 10. - 6. * (1. - progress)
            ))
        } else {
            positioned.top(px(f32::from(at.y) + 6. + 6. * (1. - progress)))
        };
        div()
            .id("popover-layer")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .when(self.popover.target.is_some(), |d| d.occlude())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, _, cx| this.close_popover(cx)),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _: &MouseDownEvent, _, cx| this.close_popover(cx)),
            )
            .child(
                positioned.child(
                    div()
                        .id("popover-card")
                        .occlude()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(card),
                ),
            )
    }

    pub fn render_toasts(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        if self.toasts.is_empty() {
            return div();
        }
        let mut stack = div()
            .absolute()
            .top(px(14.))
            .right(px(14.))
            .w(px(340.))
            .flex()
            .flex_col()
            .gap(px(10.));
        for toast in &self.toasts {
            let progress = toast.t.tick(window);
            let (id, bot) = (toast.id, toast.bot);
            let face = self
                .bot(bot)
                .map(|b| kit::avatar(bot_color(b.color), b.shape, 34., false).into_any_element())
                .unwrap_or_else(|| div().into_any_element());
            stack = stack.child(
                div()
                    .id(SharedString::from(format!("toast-{id}")))
                    .occlude()
                    .relative()
                    .left(px(40. * (1. - progress)))
                    .opacity(progress.clamp(0., 1.))
                    .flex()
                    .gap(px(12.))
                    .p(px(14.))
                    .rounded(px(16.))
                    .bg(t.surface)
                    .border_1()
                    .border_color(t.border)
                    .shadow(kit::shadow(&t, 10., 30.))
                    .cursor_pointer()
                    .child(face)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(toast.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .line_height(px(20.))
                                    .text_color(t.text_muted)
                                    .line_clamp(2)
                                    .child(toast.body.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("toast-x-{id}")))
                            .child(icon("icons/x.svg", 14., t.text_faint))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.dismiss_toast(id, cx)
                            })),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.dismiss_toast(id, cx);
                        this.open_bot(bot, window, cx);
                    })),
            );
        }
        stack
    }
}
