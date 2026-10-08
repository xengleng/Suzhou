//! Small building blocks shared by every screen: icons, avatars, buttons,
//! switches, chips and spinners.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, BoxShadow, Div, ElementId, FontWeight, Hsla, IntoElement,
    SharedString, Stateful, Styled, Svg, Transformation, div, percentage, point, prelude::*, px,
    svg, white,
};

use crate::{anim, model::Shape, theme::Theme};

pub fn icon(path: &'static str, size: f32, color: Hsla) -> Svg {
    svg()
        .path(path)
        .size(px(size))
        .flex_none()
        .text_color(color)
}

/// A Bot's face: its shape in its colour, with two little eyes.
pub fn avatar(color: Hsla, shape: Shape, size: f32, eyes_closed: bool) -> Div {
    let eyes = if eyes_closed {
        format!("avatars/closed-{}.svg", shape.key())
    } else {
        format!("avatars/eyes-{}.svg", shape.key())
    };
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .child(
            svg()
                .absolute()
                .top_0()
                .left_0()
                .size(px(size))
                .path(SharedString::from(format!("avatars/{}.svg", shape.key())))
                .text_color(color),
        )
        .child(
            svg()
                .absolute()
                .top_0()
                .left_0()
                .size(px(size))
                .path(SharedString::from(eyes))
                .text_color(white()),
        )
}

/// The Bot's avatar hopping gently: it is thinking or working.
pub fn bobbing(id: impl Into<ElementId>, child: Div, height: f32) -> impl IntoElement {
    div().relative().child(child).with_animation(
        id,
        Animation::new(Duration::from_millis(900))
            .repeat()
            .with_easing(gpui::bounce(anim::ease_in_out_cubic)),
        move |this, t| this.top(px(-height * t)),
    )
}

pub fn shadow(t: &Theme, y: f32, blur: f32) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: t.shadow,
            offset: point(px(0.), px(y)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        },
        BoxShadow {
            color: Hsla {
                a: t.shadow.a * 0.5,
                ..t.shadow
            },
            offset: point(px(0.), px(0.)),
            blur_radius: px(1.),
            spread_radius: px(0.),
        },
    ]
}

pub fn primary_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    t: &Theme,
    enabled: bool,
) -> Stateful<Div> {
    let base = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap_1p5()
        .h(px(36.))
        .px(px(16.))
        .rounded(px(10.))
        .text_size(px(14.))
        .font_weight(FontWeight::MEDIUM)
        .child(label.into());
    if enabled {
        let hover = if t.dark {
            t.primary.opacity(0.85)
        } else {
            anim_mix(t.primary, white(), 0.18)
        };
        base.bg(t.primary)
            .text_color(t.primary_text)
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
    } else {
        base.bg(t.disabled).text_color(t.disabled_text)
    }
}

pub fn secondary_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    t: &Theme,
) -> Stateful<Div> {
    let hover = t.sidebar_selected;
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap_1p5()
        .h(px(34.))
        .px(px(14.))
        .rounded(px(10.))
        .bg(t.field)
        .text_color(t.text)
        .text_size(px(14.))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(label.into())
}

pub fn icon_button(id: impl Into<ElementId>, path: &'static str, t: &Theme) -> Stateful<Div> {
    let hover = t.hover;
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(30.))
        .rounded(px(8.))
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(icon(path, 17., t.text_muted))
}

/// An on/off switch whose knob slides when it flips.
pub fn toggle(id: &str, on: bool, t: &Theme) -> Stateful<Div> {
    let (track_on, track_off) = (t.accent, t.border_strong);
    let key = SharedString::from(format!("{id}-{on}"));
    div()
        .id(SharedString::from(id.to_string()))
        .flex_none()
        .w(px(40.))
        .h(px(24.))
        .cursor_pointer()
        .child(
            div()
                .flex()
                .items_center()
                .w(px(40.))
                .h(px(24.))
                .rounded_full()
                .child(
                    div()
                        .size(px(20.))
                        .rounded_full()
                        .bg(white())
                        .shadow(vec![BoxShadow {
                            color: gpui::black().opacity(0.25),
                            offset: point(px(0.), px(1.)),
                            blur_radius: px(2.),
                            spread_radius: px(0.),
                        }]),
                )
                .with_animation(
                    key,
                    Animation::new(Duration::from_millis(260)),
                    move |this, t| {
                        let t = anim::ease_out_back(t);
                        let (from, to) = if on { (0.0, 1.0) } else { (1.0, 0.0) };
                        let x = anim::lerp(from, to, t);
                        this.bg(mix(track_off, track_on, x.clamp(0.0, 1.0)))
                            .pl(px(2. + 16. * x))
                    },
                ),
        )
}

pub fn chip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    t: &Theme,
) -> Stateful<Div> {
    let hover = t.hover;
    let base = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .gap_1p5()
        .h(px(32.))
        .px(px(12.))
        .rounded(px(8.))
        .border_1()
        .text_size(px(14.))
        .cursor_pointer()
        .child(label.into());
    if selected {
        base.bg(t.field).border_color(t.field).text_color(t.text)
    } else {
        base.border_color(t.border)
            .text_color(t.text)
            .hover(move |s| s.bg(hover))
    }
}

pub fn spinner(id: impl Into<ElementId>, size: f32, color: Hsla) -> impl IntoElement {
    icon("icons/spinner.svg", size, color).with_animation(
        id,
        Animation::new(Duration::from_millis(800)).repeat(),
        |svg, t| svg.with_transformation(Transformation::rotate(percentage(t))),
    )
}

/// Three dots pulsing in turn.
pub fn typing_dots(id: &str, color: Hsla) -> impl IntoElement + use<> {
    div().flex().gap(px(4.)).children((0..3).map(|i| {
        div().size(px(6.)).rounded_full().bg(color).with_animation(
            SharedString::from(format!("{id}-{i}")),
            Animation::new(Duration::from_millis(1000)).repeat(),
            move |this, t| {
                let phase = (t - i as f32 * 0.18).rem_euclid(1.0);
                let lift = (phase * std::f32::consts::PI * 2.0).sin().max(0.0);
                this.opacity(0.35 + 0.65 * lift)
            },
        )
    }))
}

pub fn divider(t: &Theme) -> Div {
    div().h(px(1.)).w_full().bg(t.border)
}

pub fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let a = a.to_rgb();
    let b = b.to_rgb();
    gpui::Rgba {
        r: anim::lerp(a.r, b.r, t),
        g: anim::lerp(a.g, b.g, t),
        b: anim::lerp(a.b, b.b, t),
        a: anim::lerp(a.a, b.a, t),
    }
    .into()
}

fn anim_mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    mix(a, b, t)
}

/// A menu row with an icon.
pub fn menu_item(
    id: impl Into<ElementId>,
    icon_path: &'static str,
    label: impl Into<SharedString>,
    t: &Theme,
    danger: bool,
) -> Stateful<Div> {
    let hover = t.hover;
    let color = if danger { t.danger } else { t.text };
    div()
        .id(id)
        .flex()
        .items_center()
        .gap_2p5()
        .h(px(32.))
        .px(px(10.))
        .rounded(px(7.))
        .text_size(px(14.))
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(icon(
            icon_path,
            16.,
            if danger { t.danger } else { t.text_muted },
        ))
        .child(label.into())
}

/// The floating card menus and popovers sit in.
pub fn popover_card(t: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .p(px(5.))
        .rounded(px(12.))
        .bg(t.surface)
        .border_1()
        .border_color(t.border)
        .shadow(shadow(t, 8., 24.))
}
