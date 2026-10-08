//! Just enough Markdown for chat: paragraphs, lists, headings, `code`,
//! **bold** and links that open in the browser.

use std::ops::Range;

use gpui::{
    Div, ElementId, FontWeight, Hsla, InteractiveText, IntoElement, SharedString, StyledText,
    TextRun, div, font, prelude::*, px,
};

use crate::theme::{FONT, MONO, Theme};

#[derive(Clone, Copy, PartialEq)]
enum Span {
    Plain,
    Code,
    Bold,
    Link,
}

/// Splits a line into styled spans, dropping the Markdown markers.
fn parse_inline(line: &str) -> (String, Vec<(Range<usize>, Span)>) {
    let mut out = String::new();
    let mut spans: Vec<(Range<usize>, Span)> = Vec::new();
    let mut rest = line;
    let push = |out: &mut String, spans: &mut Vec<(Range<usize>, Span)>, text: &str, kind: Span| {
        if text.is_empty() {
            return;
        }
        let start = out.len();
        out.push_str(text);
        if let Some(last) = spans.last_mut()
            && last.1 == kind
            && last.0.end == start
        {
            last.0.end = out.len();
            return;
        }
        spans.push((start..out.len(), kind));
    };
    while !rest.is_empty() {
        let next_code = rest.find('`');
        let next_bold = rest.find("**");
        let next_link = ["https://", "http://"]
            .iter()
            .filter_map(|p| rest.find(p))
            .min();
        let next = [next_code, next_bold, next_link]
            .into_iter()
            .flatten()
            .min();
        let Some(at) = next else {
            push(&mut out, &mut spans, rest, Span::Plain);
            break;
        };
        push(&mut out, &mut spans, &rest[..at], Span::Plain);
        rest = &rest[at..];
        if Some(at) == next_code {
            if let Some(end) = rest[1..].find('`') {
                push(&mut out, &mut spans, &rest[1..1 + end], Span::Code);
                rest = &rest[end + 2..];
            } else {
                push(&mut out, &mut spans, "`", Span::Plain);
                rest = &rest[1..];
            }
        } else if Some(at) == next_bold {
            if let Some(end) = rest[2..].find("**") {
                push(&mut out, &mut spans, &rest[2..2 + end], Span::Bold);
                rest = &rest[end + 4..];
            } else {
                push(&mut out, &mut spans, "**", Span::Plain);
                rest = &rest[2..];
            }
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let mut url = &rest[..end];
            while url.ends_with(['.', ',', ')', ';', ':', '!', '?', '\u{201d}']) {
                url = &url[..url.len() - url.chars().last().map(char::len_utf8).unwrap_or(1)];
            }
            push(&mut out, &mut spans, url, Span::Link);
            rest = &rest[url.len()..];
        }
    }
    (out, spans)
}

pub struct Style {
    pub size: f32,
    pub line_height: f32,
    pub color: Hsla,
    pub weight: FontWeight,
}

impl Style {
    pub fn new(size: f32, line_height: f32, color: Hsla) -> Self {
        Style {
            size,
            line_height,
            color,
            weight: FontWeight::NORMAL,
        }
    }
}

/// One line of inline-styled text. Links are clickable.
pub fn inline(id: ElementId, line: &str, style: &Style, t: &Theme) -> impl IntoElement + use<> {
    let (text, spans) = parse_inline(line);
    let mut base = font(FONT);
    base.weight = style.weight;
    let mut links = Vec::new();
    let runs: Vec<TextRun> = spans
        .iter()
        .map(|(range, kind)| {
            let mut run = TextRun {
                len: range.len(),
                font: base.clone(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            match kind {
                Span::Plain => {}
                Span::Code => {
                    run.font = font(MONO);
                    run.color = t.code;
                    run.background_color = Some(t.code_bg);
                }
                Span::Bold => run.font.weight = FontWeight::SEMIBOLD,
                Span::Link => {
                    run.color = t.link;
                    links.push(range.clone());
                }
            }
            run
        })
        .collect();
    let urls: Vec<String> = links.iter().map(|r| text[r.clone()].to_string()).collect();
    let styled = if runs.is_empty() {
        StyledText::new(SharedString::from(" "))
    } else {
        StyledText::new(SharedString::from(text)).with_runs(runs)
    };
    div()
        .text_size(px(style.size))
        .line_height(px(style.line_height))
        .child(
            InteractiveText::new(id, styled).on_click(links, move |ix, _window, cx| {
                if let Some(url) = urls.get(ix) {
                    cx.open_url(url);
                }
            }),
        )
}

/// A block of Markdown-ish text as stacked paragraphs.
pub fn markdown(id: &str, text: &str, style: &Style, t: &Theme) -> Div {
    let mut column = div().flex().flex_col().gap(px(3.));
    for (i, line) in text.lines().enumerate() {
        let key = SharedString::from(format!("{id}-{i}"));
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            column = column.child(div().h(px(style.line_height * 0.4)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            column = column.child(heading(key, rest, style.size + 2., style, t));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            column =
                column.child(
                    div()
                        .pt(px(8.))
                        .child(heading(key, rest, style.size + 6., style, t)),
                );
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            column = column.child(heading(key, rest, style.size + 13., style, t));
        } else if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            column = column.child(
                div()
                    .flex()
                    .gap(px(8.))
                    .pl(px(4.))
                    .child(
                        div()
                            .flex_none()
                            .pt(px(style.line_height / 2. - 2.))
                            .child(div().size(px(4.5)).rounded_full().bg(style.color)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(inline(key.into(), rest, style, t)),
                    ),
            );
        } else if let Some((num, rest)) = numbered(trimmed) {
            column = column.child(
                div()
                    .flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_none()
                            .min_w(px(style.size * 1.1))
                            .text_size(px(style.size))
                            .line_height(px(style.line_height))
                            .text_color(style.color)
                            .child(format!("{num}.")),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(inline(key.into(), rest, style, t)),
                    ),
            );
        } else {
            column = column.child(inline(key.into(), trimmed, style, t));
        }
    }
    column
}

fn heading(
    id: SharedString,
    text: &str,
    size: f32,
    style: &Style,
    t: &Theme,
) -> impl IntoElement + use<> {
    inline(
        id.into(),
        text,
        &Style {
            size,
            line_height: size * 1.3,
            color: style.color,
            weight: FontWeight::SEMIBOLD,
        },
        t,
    )
}

fn numbered(line: &str) -> Option<(&str, &str)> {
    let dot = line.find(". ")?;
    let num = &line[..dot];
    (!num.is_empty() && num.len() <= 3 && num.chars().all(|c| c.is_ascii_digit()))
        .then(|| (num, &line[dot + 2..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_code_links_and_bold() {
        let (text, spans) = parse_inline("See `rss.xml` at https://example.com. **Done**");
        assert_eq!(text, "See rss.xml at https://example.com. Done");
        let kinds: Vec<_> = spans.iter().map(|(r, k)| (&text[r.clone()], *k)).collect();
        assert!(kinds.contains(&("rss.xml", Span::Code)));
        assert!(kinds.contains(&("https://example.com", Span::Link)));
        assert!(kinds.contains(&("Done", Span::Bold)));
    }

    #[test]
    fn unclosed_markers_stay_literal() {
        let (text, _) = parse_inline("a ` b ** c");
        assert_eq!(text, "a ` b ** c");
    }
}
