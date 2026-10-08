//! The panel that slides in from the right: Bot details, a routine, or the
//! Agent Computer.

use gpui::{
    AnyElement, Context, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px,
};

use super::{
    app::{AppView, Modal, Panel},
    computer,
    kit::{self, icon},
};
use crate::{
    model::*,
    theme::{Theme, bot_color, theme},
};

pub const PANEL_WIDTH: f32 = 380.;

const SCHEDULES: &[&str] = &[
    "Every day at 9:00 AM",
    "Every weekday at 9:00 AM",
    "Every Monday at 8:00 AM",
    "Every day at 10:01 AM",
    "Every hour",
    "Every Friday at 5:00 PM",
];

fn field_box(t: &Theme, focused: bool) -> gpui::Div {
    div()
        .px(px(12.))
        .py(px(9.))
        .rounded(px(10.))
        .border_1()
        .border_color(if focused { t.accent } else { t.field_border })
        .bg(t.surface)
}

fn label(text: &str, t: &Theme) -> gpui::Div {
    div()
        .pt(px(18.))
        .pb(px(8.))
        .text_size(px(14.))
        .text_color(t.text_muted)
        .child(text.to_string())
}

impl AppView {
    pub fn render_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let Some((panel, progress)) = self.panel.visible(window) else {
            return div();
        };
        let Some(bot) = self.current_bot().cloned() else {
            return div();
        };
        let content: AnyElement = match panel {
            Panel::Details => self.details_panel(&bot, &t, window, cx).into_any_element(),
            Panel::Routine(rid) => match bot.routines.iter().find(|r| r.id == rid) {
                Some(r) => self.routine_panel(r, &t, window, cx).into_any_element(),
                None => div().into_any_element(),
            },
            Panel::Computer => self.computer_panel(&bot, &t, window, cx).into_any_element(),
        };
        div()
            .flex_none()
            .h_full()
            .w(px(PANEL_WIDTH * progress))
            .overflow_hidden()
            .border_l_1()
            .border_color(t.border)
            .bg(t.bg)
            .child(
                div()
                    .w(px(PANEL_WIDTH))
                    .h_full()
                    .opacity(progress)
                    .child(content),
            )
    }

    fn panel_header(
        &self,
        title: &str,
        back: bool,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        div()
            .h(px(52.))
            .flex_none()
            .flex()
            .items_center()
            .px(px(10.))
            .border_b_1()
            .border_color(t.border)
            .child(if back {
                kit::icon_button("panel-back", "icons/chevron-left.svg", t)
                    .on_click(cx.listener(|this, _, _, cx| this.open_panel(Panel::Details, cx)))
                    .into_any_element()
            } else {
                div().w(px(30.)).into_any_element()
            })
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .text_size(px(16.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(title.to_string()),
            )
            .child(
                kit::icon_button("panel-close", "icons/chevrons-right.svg", t).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.panel.close(260);
                        cx.notify();
                    }),
                ),
            )
    }

    fn details_panel(
        &self,
        bot: &Bot,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = bot.id;
        let editing = self.editing_profile;
        let profile: AnyElement = if editing {
            let f = |input: &gpui::Entity<super::text_input::TextInput>| {
                input.read(cx).is_focused(window)
            };
            div()
                .flex()
                .flex_col()
                .child(label("Name", t))
                .child(
                    field_box(t, f(&self.inputs.profile_name))
                        .child(self.inputs.profile_name.clone()),
                )
                .child(label("Job title", t))
                .child(
                    field_box(t, f(&self.inputs.profile_title))
                        .child(self.inputs.profile_title.clone()),
                )
                .child(label("Description", t))
                .child(
                    field_box(t, f(&self.inputs.profile_description))
                        .child(self.inputs.profile_description.clone()),
                )
                .child(label("Colour", t))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.))
                        .children((0..10).map(|ix| {
                            div()
                                .id(SharedString::from(format!("p-swatch-{ix}")))
                                .size(px(26.))
                                .rounded_full()
                                .bg(bot_color(ix))
                                .border_2()
                                .border_color(if ix == bot.color {
                                    t.text
                                } else {
                                    gpui::transparent_black()
                                })
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Some(b) = this.bot_mut(id) {
                                        b.color = ix;
                                    }
                                    this.save(cx);
                                    cx.notify();
                                }))
                        })),
                )
                .child(label("Shape", t))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(4.))
                        .children(Shape::ALL.iter().map(|shape| {
                            let shape = *shape;
                            div()
                                .id(SharedString::from(format!("p-shape-{}", shape.key())))
                                .size(px(38.))
                                .rounded(px(9.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .border_2()
                                .border_color(if shape == bot.shape {
                                    t.accent
                                } else {
                                    gpui::transparent_black()
                                })
                                .cursor_pointer()
                                .child(kit::avatar(bot_color(bot.color), shape, 24., false))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Some(b) = this.bot_mut(id) {
                                        b.shape = shape;
                                    }
                                    this.save(cx);
                                    cx.notify();
                                }))
                        })),
                )
                .child(div().pt(px(16.)).child(
                    kit::primary_button("profile-done", "Done", t, true).on_click(cx.listener(
                        |this, _, _, cx| {
                            this.editing_profile = false;
                            cx.notify();
                        },
                    )),
                ))
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.))
                .pt(px(22.))
                .child(kit::avatar(bot_color(bot.color), bot.shape, 76., false))
                .child(
                    div()
                        .pt(px(8.))
                        .text_size(px(21.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(bot.name.clone()),
                )
                .child(div().text_size(px(15.)).text_color(t.text_muted).child(
                    if bot.title.is_empty() {
                        "No job title yet".to_string()
                    } else {
                        bot.title.clone()
                    },
                ))
                .when(!bot.description.is_empty(), |d| {
                    d.child(
                        div()
                            .pt(px(8.))
                            .text_center()
                            .text_size(px(14.))
                            .line_height(px(21.))
                            .text_color(t.text_muted)
                            .child(bot.description.clone()),
                    )
                })
                .child(
                    div()
                        .pt(px(14.))
                        .flex()
                        .gap(px(8.))
                        .child(
                            kit::secondary_button("edit-profile", "Edit profile", t).on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.editing_profile = true;
                                    this.sync_panel_inputs_public(cx);
                                    cx.notify();
                                }),
                            ),
                        )
                        .child(
                            kit::secondary_button("open-computer", "Computer", t).on_click(
                                cx.listener(|this, _, _, cx| this.open_panel(Panel::Computer, cx)),
                            ),
                        ),
                )
                .into_any_element()
        };

        let hover = t.hover;
        let routines = bot
            .routines
            .iter()
            .map(|r| {
                let rid = r.id;
                div()
                    .id(SharedString::from(format!("routine-row-{rid}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(48.))
                    .px(px(10.))
                    .rounded(px(10.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .child(icon(
                        "icons/clock.svg",
                        16.,
                        if r.active { t.text } else { t.text_faint },
                    ))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(div().truncate().text_size(px(15.)).child(r.name.clone()))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(13.))
                                    .text_color(t.text_muted)
                                    .child(if r.active {
                                        r.schedules.first().cloned().unwrap_or_default()
                                    } else {
                                        "Paused".into()
                                    }),
                            ),
                    )
                    .child(icon("icons/chevron-right.svg", 15., t.text_faint))
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.open_panel(Panel::Routine(rid), cx)),
                    )
            })
            .collect::<Vec<_>>();
        let files: Vec<(MsgId, String, String, String)> = bot
            .messages
            .iter()
            .filter_map(|m| match &m.kind {
                MessageKind::File {
                    name,
                    size,
                    content,
                } => Some((m.id, name.clone(), size.clone(), content.clone())),
                _ => None,
            })
            .collect();
        let files = files
            .into_iter()
            .map(|(mid, name, size, content)| {
                div()
                    .id(SharedString::from(format!("file-row-{mid}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(40.))
                    .px(px(10.))
                    .rounded(px(10.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .child(icon("icons/file.svg", 16., t.text_muted))
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(px(15.))
                            .child(name.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(t.text_faint)
                            .child(size),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_modal(
                            Modal::File {
                                name: name.clone(),
                                content: content.clone(),
                            },
                            cx,
                        )
                    }))
            })
            .collect::<Vec<_>>();
        let skills = bot
            .skills
            .iter()
            .map(|s| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .min_h(px(40.))
                    .px(px(10.))
                    .child(icon("icons/book.svg", 16., t.text_muted))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_size(px(15.)).child(format!("/{}", s.name)))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(13.))
                                    .text_color(t.text_muted)
                                    .child(s.description.clone()),
                            ),
                    )
            })
            .collect::<Vec<_>>();

        let section = |title: &str, count: usize| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt(px(22.))
                .pb(px(6.))
                .px(px(10.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_muted)
                        .child(format!("{title} \u{00b7} {count}")),
                )
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.panel_header("Details", false, t, cx))
            .child(
                div()
                    .id("details-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(16.))
                    .pb(px(24.))
                    .child(profile)
                    .when(!editing, |d| {
                        d.child(section("Routines", bot.routines.len()))
                            .children(routines)
                            .child(
                                div()
                                    .id("new-routine")
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .h(px(38.))
                                    .px(px(10.))
                                    .rounded(px(10.))
                                    .text_size(px(14.))
                                    .text_color(t.text_muted)
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(hover))
                                    .child(icon("icons/plus.svg", 15., t.text_muted))
                                    .child("New routine")
                                    .on_click(cx.listener(|this, _, _, cx| this.new_routine(cx))),
                            )
                            .child(section("Skills", bot.skills.len()))
                            .children(skills)
                            .child(
                                div()
                                    .id("teach")
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .h(px(38.))
                                    .px(px(10.))
                                    .rounded(px(10.))
                                    .text_size(px(14.))
                                    .text_color(t.text_muted)
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(hover))
                                    .child(icon("icons/sparkles.svg", 15., t.text_muted))
                                    .child("Teach a task")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_modal(Modal::Teach, cx)
                                    })),
                            )
                            .child(section("Files", files.len()))
                            .children(files)
                            .child(
                                div()
                                    .pt(px(24.))
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(8.))
                                    .px(px(6.))
                                    .children([
                                        kit::secondary_button(
                                            "d-pin",
                                            if bot.pinned { "Unpin" } else { "Pin" },
                                            t,
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                this.toggle_pin(id, cx)
                                            }),
                                        ),
                                        kit::secondary_button("d-dup", "Duplicate", t).on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.duplicate_bot(id, window, cx)
                                            }),
                                        ),
                                        kit::secondary_button(
                                            "d-hide",
                                            if bot.hidden { "Unhide" } else { "Hide" },
                                            t,
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.toggle_hidden(id, window, cx)
                                            }),
                                        ),
                                        kit::secondary_button("d-del", "Delete", t)
                                            .text_color(t.danger)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.open_modal(Modal::ConfirmDelete(id), cx)
                                            })),
                                    ]),
                            )
                    }),
            )
    }

    fn routine_panel(
        &self,
        r: &Routine,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let rid = r.id;
        let running = r
            .history
            .first()
            .is_some_and(|h| h.status == RunStatus::Running);
        let name_focused = self.inputs.routine_name.read(cx).is_focused(window);
        let instr_focused = self.inputs.routine_instruction.read(cx).is_focused(window);
        let hover = t.hover;
        let schedules = r
            .schedules
            .iter()
            .enumerate()
            .map(|(ix, s)| {
                let s = s.clone();
                div()
                    .id(SharedString::from(format!("sched-{ix}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(40.))
                    .px(px(6.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(move |st| st.bg(hover))
                    .child(icon("icons/clock.svg", 17., t.text))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(15.))
                            .child(schedule_text(&s, t)),
                    )
                    .when(r.schedules.len() > 1, |d| {
                        d.child(
                            kit::icon_button(
                                SharedString::from(format!("sched-x-{ix}")),
                                "icons/x.svg",
                                t,
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    if let Some(r) = this.routine_mut(rid) {
                                        r.schedules.remove(ix);
                                    }
                                    this.save(cx);
                                    cx.notify();
                                },
                            )),
                        )
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(r) = this.routine_mut(rid) {
                            let current = SCHEDULES
                                .iter()
                                .position(|p| *p == r.schedules[ix])
                                .map(|p| p + 1)
                                .unwrap_or(0);
                            r.schedules[ix] = SCHEDULES[current % SCHEDULES.len()].to_string();
                        }
                        this.save(cx);
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();
        let history: AnyElement = if r.history.is_empty() {
            div()
                .text_size(px(15.))
                .text_color(t.text_muted)
                .child("No runs yet")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(r.history.iter().enumerate().map(|(i, h)| {
                    let (glyph, color, word) = match h.status {
                        RunStatus::Running => (None, t.text_muted, "Running"),
                        RunStatus::Succeeded => (Some("icons/check.svg"), t.online, "Completed"),
                        RunStatus::Failed => (Some("icons/x.svg"), t.danger, "Failed"),
                    };
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .h(px(30.))
                        .child(match glyph {
                            Some(g) => icon(g, 15., color).into_any_element(),
                            None => {
                                kit::spinner(SharedString::from(format!("run-{i}")), 15., color)
                                    .into_any_element()
                            }
                        })
                        .child(div().flex_1().text_size(px(14.)).child(h.when.clone()))
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(t.text_muted)
                                .child(word),
                        )
                }))
                .into_any_element()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.panel_header("Routine", true, t, cx))
            .child(
                div()
                    .id("routine-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(16.))
                    .pb(px(24.))
                    .child(
                        div()
                            .pt(px(14.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(kit::toggle("routine-active", r.active, t).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    if let Some(r) = this.routine_mut(rid) {
                                        r.active = !r.active;
                                    }
                                    this.save(cx);
                                    cx.notify();
                                }),
                            ))
                            .child(div().flex_1().text_size(px(16.)).child(if r.active {
                                "Active"
                            } else {
                                "Paused"
                            }))
                            .child(
                                kit::secondary_button("routine-delete", "Delete", t).on_click(
                                    cx.listener(move |this, _, _, cx| this.delete_routine(rid, cx)),
                                ),
                            )
                            .child(
                                kit::primary_button(
                                    "routine-test",
                                    if running {
                                        "Running\u{2026}"
                                    } else {
                                        "Test run"
                                    },
                                    t,
                                    !running,
                                )
                                .when(!running, |b| {
                                    b.on_click(
                                        cx.listener(move |this, _, _, cx| this.test_run(rid, cx)),
                                    )
                                }),
                            ),
                    )
                    .child(label("Name", t))
                    .child(field_box(t, name_focused).child(self.inputs.routine_name.clone()))
                    .child(label("Instruction", t))
                    .child(
                        field_box(t, instr_focused)
                            .py(px(12.))
                            .child(self.inputs.routine_instruction.clone()),
                    )
                    .child(label("When to run", t))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .rounded(px(12.))
                            .border_1()
                            .border_color(t.border)
                            .children(schedules)
                            .child(
                                div()
                                    .id("add-schedule")
                                    .flex()
                                    .items_center()
                                    .gap(px(10.))
                                    .h(px(36.))
                                    .px(px(6.))
                                    .rounded(px(8.))
                                    .text_size(px(15.))
                                    .text_color(t.text_muted)
                                    .cursor_pointer()
                                    .hover(move |st| st.bg(hover))
                                    .child(icon("icons/plus.svg", 15., t.text_muted))
                                    .child("Add another")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(r) = this.routine_mut(rid) {
                                            let next = SCHEDULES
                                                .iter()
                                                .find(|s| !r.schedules.iter().any(|x| x == *s))
                                                .unwrap_or(&SCHEDULES[0]);
                                            r.schedules.push(next.to_string());
                                        }
                                        this.save(cx);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .pt(px(6.))
                            .text_size(px(12.))
                            .text_color(t.text_faint)
                            .child("Click a time to change it. Times are in your local time zone."),
                    )
                    .child(label("Run history", t))
                    .child(history),
            )
    }

    fn computer_panel(
        &mut self,
        bot: &Bot,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let cursor = if self.cursor_bot == Some(bot.id) {
            (self.cursor.0.tick(window), self.cursor.1.tick(window))
        } else {
            bot.computer.cursor
        };
        let working = bot.status == BotStatus::Working;
        let status = if working {
            bot.computer.status.clone()
        } else {
            "Idle".into()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.panel_header("Agent Computer", false, t, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .p(px(16.))
                    .child(
                        div()
                            .id("computer-preview")
                            .rounded(px(12.))
                            .overflow_hidden()
                            .border_1()
                            .border_color(t.border)
                            .cursor_pointer()
                            .child(computer::screen("panel", &bot.computer, cursor, PANEL_WIDTH - 34., working, t))
                            .on_click(cx.listener(|this, _, _, cx| this.open_modal(Modal::Takeover, cx))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(14.))
                            .text_color(t.text_muted)
                            .child(if working {
                                kit::spinner("computer-status", 14., t.text_muted).into_any_element()
                            } else {
                                div().size(px(8.)).rounded_full().bg(t.text_faint).into_any_element()
                            })
                            .child(status),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(kit::primary_button("take-over", "Take over", t, true).flex_1().on_click(cx.listener(|this, _, _, cx| this.take_over(cx))))
                            .child(kit::secondary_button("full-screen", "Full screen", t).on_click(cx.listener(|this, _, _, cx| this.open_modal(Modal::Takeover, cx)))),
                    )
                    .child(
                        div()
                            .p(px(12.))
                            .rounded(px(12.))
                            .bg(t.field)
                            .flex()
                            .gap(px(10.))
                            .child(icon("icons/lock.svg", 16., t.text_muted))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_size(px(13.))
                                    .line_height(px(19.))
                                    .text_color(t.text_muted)
                                    .child("Take over for passwords, passkeys, 2FA, CAPTCHAs and payments, then hand back. Never paste secrets in chat."),
                            ),
                    )
                    .child(
                        div().text_size(px(12.)).line_height(px(18.)).text_color(t.text_faint).child(
                            "One cloud computer per account. Each Bot gets its own screen; cookies, files and command-line sign-ins are shared.",
                        ),
                    ),
            )
    }

    pub fn sync_panel_inputs_public(&mut self, cx: &mut Context<Self>) {
        let Some(bot) = self.current_bot().cloned() else {
            return;
        };
        self.inputs
            .profile_name
            .update(cx, |i, cx| i.set_text(bot.name.clone(), cx));
        self.inputs
            .profile_title
            .update(cx, |i, cx| i.set_text(bot.title.clone(), cx));
        self.inputs
            .profile_description
            .update(cx, |i, cx| i.set_text(bot.description.clone(), cx));
    }
}

/// "Every day at 10:01 AM" with the frequency in the text colour and the rest muted.
fn schedule_text(s: &str, t: &Theme) -> impl IntoElement + use<> {
    let (head, tail) = match s.find(" at ") {
        Some(_) => {
            let first_space = s.find(' ').unwrap_or(0);
            (&s[..first_space], &s[first_space..])
        }
        None => (s, ""),
    };
    div()
        .flex()
        .child(div().child(head.to_string()))
        .child(div().text_color(t.text_muted).child(tail.to_string()))
}
