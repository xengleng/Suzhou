//! "New Bot": pick a colour, a shape and a name.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, Context, FontWeight, IntoElement, SharedString, Transformation,
    Window, div, prelude::*, px, size, svg, white,
};

use super::{app::AppView, kit};
use crate::{
    anim, data,
    model::Shape,
    theme::{BOT_COLORS, Theme, bot_color, theme},
};

/// The big avatar, which squashes and springs back when you change it.
fn springy_avatar(
    key: u32,
    color: gpui::Hsla,
    shape: Shape,
    px_size: f32,
) -> impl IntoElement + use<> {
    let body = SharedString::from(format!("avatars/{}.svg", shape.key()));
    let eyes = SharedString::from(format!("avatars/eyes-{}.svg", shape.key()));
    let animation = || Animation::new(Duration::from_millis(560));
    let scale = |t: f32| {
        let s = 0.82 + 0.18 * anim::ease_out_back(t);
        Transformation::scale(size(s, s))
    };
    div()
        .relative()
        .size(px(px_size))
        .child(
            svg()
                .absolute()
                .size(px(px_size))
                .path(body)
                .text_color(color)
                .with_animation(
                    SharedString::from(format!("draft-body-{key}")),
                    animation(),
                    move |s, t| s.with_transformation(scale(t)),
                ),
        )
        .child(
            svg()
                .absolute()
                .size(px(px_size))
                .path(eyes)
                .text_color(white())
                .with_animation(
                    SharedString::from(format!("draft-eyes-{key}")),
                    animation(),
                    move |s, t| s.with_transformation(scale(t)),
                ),
        )
}

impl AppView {
    pub fn render_new_bot(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let t = theme(cx);
        let color = bot_color(self.draft_color);
        let name = self.inputs.bot_name.read(cx).text().trim().to_string();
        let can_start = !name.is_empty();
        let focused = self.inputs.bot_name.read(cx).is_focused(window);

        let swatches = (0..BOT_COLORS.len())
            .map(|ix| {
                let selected = ix == self.draft_color;
                div()
                    .id(SharedString::from(format!("swatch-{ix}")))
                    .size(px(40.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_2()
                    .border_color(if selected {
                        t.accent
                    } else {
                        gpui::transparent_black()
                    })
                    .cursor_pointer()
                    .child(div().size(px(30.)).rounded_full().bg(bot_color(ix)))
                    .hover(|s| s.opacity(0.85))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.draft_color = ix;
                        this.draft_bump += 1;
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();
        let shapes = Shape::ALL
            .iter()
            .map(|shape| {
                let shape = *shape;
                let selected = shape == self.draft_shape;
                let hover = t.hover;
                div()
                    .id(SharedString::from(format!("shape-{}", shape.key())))
                    .size(px(48.))
                    .rounded(px(12.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_2()
                    .border_color(if selected {
                        t.accent
                    } else {
                        gpui::transparent_black()
                    })
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .child(kit::avatar(color, shape, 32., false))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.draft_shape = shape;
                        this.draft_bump += 1;
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();
        let suggestions = data::SUGGESTIONS
            .iter()
            .enumerate()
            .map(|(ix, s)| suggestion_card(ix, s, &t, cx))
            .collect::<Vec<_>>();

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(52.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(24.))
                    .border_b_1()
                    .border_color(t.border)
                    .child(kit::avatar(color, self.draft_shape, 26., false))
                    .child(
                        div()
                            .text_size(px(17.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(if name.is_empty() {
                                "New Bot".to_string()
                            } else {
                                name.clone()
                            }),
                    ),
            )
            .child(
                div()
                    .id("new-bot-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .pt(px(56.))
                            .pb(px(24.))
                            .child(springy_avatar(
                                self.draft_bump,
                                color,
                                self.draft_shape,
                                112.,
                            ))
                            .child(div().pt(px(30.)).flex().gap(px(6.)).children(swatches))
                            .child(div().pt(px(14.)).flex().gap(px(6.)).children(shapes))
                            .child(
                                div()
                                    .pt(px(36.))
                                    .w(px(460.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.))
                                    .child(
                                        div()
                                            .pl(px(14.))
                                            .text_size(px(16.))
                                            .text_color(t.text_muted)
                                            .child("Name"),
                                    )
                                    .child(
                                        div()
                                            .h(px(50.))
                                            .px(px(16.))
                                            .flex()
                                            .items_center()
                                            .rounded(px(12.))
                                            .border_1()
                                            .border_color(if focused {
                                                t.accent
                                            } else {
                                                t.field_border
                                            })
                                            .bg(t.surface)
                                            .child(self.inputs.bot_name.clone()),
                                    ),
                            )
                            .child(
                                div().pt(px(36.)).child(
                                    kit::primary_button(
                                        "get-started",
                                        "Get started",
                                        &t,
                                        can_start,
                                    )
                                    .h(px(48.))
                                    .px(px(20.))
                                    .text_size(px(18.))
                                    .when(can_start, |b| {
                                        b.on_click(cx.listener(|this, _, window, cx| {
                                            this.create_bot(window, cx)
                                        }))
                                    }),
                                ),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .pl(px(24.))
                    .pb(px(22.))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .pl(px(4.))
                            .text_size(px(16.))
                            .text_color(t.text_muted)
                            .child("Suggestions"),
                    )
                    .child(
                        div()
                            .id("suggestions")
                            .overflow_x_scroll()
                            .child(div().flex().gap(px(16.)).pr(px(24.)).children(suggestions)),
                    ),
            )
    }
}

fn suggestion_card(
    ix: usize,
    s: &data::Suggestion,
    t: &Theme,
    cx: &mut Context<AppView>,
) -> impl IntoElement + use<> {
    let hover = t.hover;
    div()
        .id(SharedString::from(format!("suggestion-{ix}")))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(18.))
        .w(px(440.))
        .h(px(140.))
        .px(px(24.))
        .rounded(px(22.))
        .border_1()
        .border_color(t.border)
        .bg(t.surface_raised)
        .cursor_pointer()
        .hover(move |st| st.bg(hover))
        .child(kit::avatar(bot_color(s.color), s.shape, 64., false))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(div().text_size(px(24.)).child(s.name))
                .child(
                    div()
                        .text_size(px(18.))
                        .line_height(px(26.))
                        .text_color(t.text_muted)
                        .child(s.blurb),
                ),
        )
        .on_click(cx.listener(move |this, _, window, cx| this.use_suggestion(ix, window, cx)))
}
