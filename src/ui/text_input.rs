//! A text field for GPUI: single line or wrapping multi-line, with IME,
//! selection, clipboard and a blinking caret.
//!
//! Adapted from GPUI's `input` example and extended to wrap text over
//! several lines, grow to a maximum height and then scroll.

use std::{ops::Range, time::Duration};

use gpui::{
    App, AvailableSpace, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, GlobalElementId, Hsla, KeyBinding, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, SharedString, Style, Task, TextAlign, TextRun,
    UTF16Selection, UnderlineStyle, Window, WrappedLine, actions, div, fill, point, prelude::*, px,
    relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::theme::{FONT, theme};

actions!(
    text_input,
    [
        Backspace,
        Delete,
        DeleteWordLeft,
        Left,
        Right,
        Up,
        Down,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        Enter,
        Newline,
        Tab,
        Paste,
        Copy,
        Cut,
    ]
);

const CONTEXT: &str = "TextInput";

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some(CONTEXT)),
        KeyBinding::new("delete", Delete, Some(CONTEXT)),
        KeyBinding::new("left", Left, Some(CONTEXT)),
        KeyBinding::new("right", Right, Some(CONTEXT)),
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("down", Down, Some(CONTEXT)),
        KeyBinding::new("shift-left", SelectLeft, Some(CONTEXT)),
        KeyBinding::new("shift-right", SelectRight, Some(CONTEXT)),
        KeyBinding::new("shift-up", SelectUp, Some(CONTEXT)),
        KeyBinding::new("shift-down", SelectDown, Some(CONTEXT)),
        KeyBinding::new("secondary-a", SelectAll, Some(CONTEXT)),
        KeyBinding::new("secondary-v", Paste, Some(CONTEXT)),
        KeyBinding::new("secondary-c", Copy, Some(CONTEXT)),
        KeyBinding::new("secondary-x", Cut, Some(CONTEXT)),
        KeyBinding::new("home", Home, Some(CONTEXT)),
        KeyBinding::new("end", End, Some(CONTEXT)),
        KeyBinding::new("enter", Enter, Some(CONTEXT)),
        KeyBinding::new("shift-enter", Newline, Some(CONTEXT)),
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
    ]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("alt-left", WordLeft, Some(CONTEXT)),
        KeyBinding::new("alt-right", WordRight, Some(CONTEXT)),
        KeyBinding::new("alt-backspace", DeleteWordLeft, Some(CONTEXT)),
        KeyBinding::new("cmd-left", Home, Some(CONTEXT)),
        KeyBinding::new("cmd-right", End, Some(CONTEXT)),
    ]);
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-left", WordLeft, Some(CONTEXT)),
        KeyBinding::new("ctrl-right", WordRight, Some(CONTEXT)),
        KeyBinding::new("ctrl-backspace", DeleteWordLeft, Some(CONTEXT)),
    ]);
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    Changed,
    /// Enter in a field that submits.
    Submit,
    /// Up/Down/Tab/Enter while a suggestion list owns the keyboard.
    NavUp,
    NavDown,
    Accept,
}

struct Layout {
    /// Each logical line: its byte offset, its shaped text, and its top.
    lines: Vec<(usize, WrappedLine, Pixels)>,
    height: Pixels,
}

pub struct TextInput {
    focus_handle: FocusHandle,
    content: String,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    layout: Option<Layout>,
    bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    pub multiline: bool,
    pub submit_on_enter: bool,
    /// While set, Up/Down/Tab/Enter are reported instead of acted on.
    pub intercept_nav: bool,
    pub text_size: Pixels,
    pub line_height: Pixels,
    pub max_lines: usize,
    pub color: Option<Hsla>,
    scroll_y: Pixels,
    cursor_visible: bool,
    _blink: Task<()>,
}

impl EventEmitter<InputEvent> for TextInput {}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TextInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let blink = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(530))
                    .await;
                let keep_going = this
                    .update(cx, |this, cx| {
                        this.cursor_visible = !this.cursor_visible;
                        cx.notify();
                    })
                    .is_ok();
                if !keep_going {
                    break;
                }
            }
        });
        TextInput {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            layout: None,
            bounds: None,
            is_selecting: false,
            multiline: false,
            submit_on_enter: true,
            intercept_nav: false,
            text_size: px(15.),
            line_height: px(22.),
            max_lines: 1,
            color: None,
            scroll_y: px(0.),
            cursor_visible: true,
            _blink: blink,
        }
    }

    pub fn multiline(mut self, max_lines: usize, submit_on_enter: bool) -> Self {
        self.multiline = true;
        self.max_lines = max_lines;
        self.submit_on_enter = submit_on_enter;
        self
    }

    pub fn sized(mut self, text_size: f32, line_height: f32) -> Self {
        self.text_size = px(text_size);
        self.line_height = px(line_height);
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let placeholder = placeholder.into();
        if placeholder != self.placeholder {
            self.placeholder = placeholder;
            cx.notify();
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.content = text.into();
        if !self.multiline {
            self.content = self.content.replace('\n', " ");
        }
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.emit(InputEvent::Changed);
        cx.notify();
    }

    pub fn cursor(&self) -> usize {
        self.cursor_offset()
    }

    /// Replaces `range` (byte offsets) with `text` and puts the caret after it.
    pub fn replace_range(&mut self, range: Range<usize>, text: &str, cx: &mut Context<Self>) {
        let range = range.start.min(self.content.len())..range.end.min(self.content.len());
        self.content.replace_range(range.clone(), text);
        let caret = range.start + text.len();
        self.selected_range = caret..caret;
        self.marked_range = None;
        cx.emit(InputEvent::Changed);
        cx.notify();
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    fn show_caret(&mut self) {
        self.cursor_visible = true;
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if self.intercept_nav {
            cx.emit(InputEvent::NavUp);
            return;
        }
        let target = self.vertical_target(-1).unwrap_or(0);
        self.move_to(target, cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if self.intercept_nav {
            cx.emit(InputEvent::NavDown);
            return;
        }
        let target = self.vertical_target(1).unwrap_or(self.content.len());
        self.move_to(target, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        let target = self.vertical_target(-1).unwrap_or(0);
        self.select_to(target, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        let target = self.vertical_target(1).unwrap_or(self.content.len());
        self.select_to(target, cx);
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.previous_word(self.cursor_offset()), cx);
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.next_word(self.cursor_offset()), cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx)
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let offset = self.cursor_offset();
        let start = self.content[..offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.move_to(start, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let offset = self.cursor_offset();
        let end = self.content[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(self.content.len());
        self.move_to(end, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete_word_left(
        &mut self,
        _: &DeleteWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_word(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if self.intercept_nav {
            cx.emit(InputEvent::Accept);
        } else if self.submit_on_enter || !self.multiline {
            cx.emit(InputEvent::Submit);
        } else {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        } else {
            cx.emit(InputEvent::Submit);
        }
    }

    fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        if self.intercept_nav {
            cx.emit(InputEvent::Accept);
        } else {
            window.focus_next();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle);
        let index = self.index_for_mouse_position(event.position);
        if event.click_count >= 2 {
            let start = self.previous_word(index.min(self.content.len()));
            let end = self.next_word(start);
            self.selected_range = start..end;
            self.selection_reversed = false;
            cx.notify();
            return;
        }
        self.is_selecting = true;
        if event.modifiers.shift {
            self.select_to(index, cx);
        } else {
            self.move_to(index, cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let text = if self.multiline {
                text
            } else {
                text.replace('\n', " ")
            };
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.show_caret();
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.show_caret();
        cx.notify()
    }

    /// Where the caret is, relative to the top-left of the text.
    fn position_for_offset(&self, offset: usize) -> Option<Point<Pixels>> {
        let layout = self.layout.as_ref()?;
        let (start, line, top) = layout
            .lines
            .iter()
            .rev()
            .find(|(start, _, _)| *start <= offset)?;
        let local = (offset - start).min(line.len());
        let pos = line.position_for_index(local, self.line_height)?;
        Some(point(pos.x, *top + pos.y))
    }

    /// The offset nearest a point relative to the top-left of the text.
    fn offset_for_position(&self, position: Point<Pixels>) -> usize {
        let Some(layout) = self.layout.as_ref() else {
            return 0;
        };
        if position.y < px(0.) {
            return 0;
        }
        for (start, line, top) in &layout.lines {
            let height = line.size(self.line_height).height.max(self.line_height);
            if position.y < *top + height {
                let rows = (height / self.line_height).round().max(1.0);
                let y = (position.y - *top)
                    .max(px(0.))
                    .min(self.line_height * (rows - 0.5));
                let local = line
                    .closest_index_for_position(point(position.x.max(px(0.)), y), self.line_height)
                    .unwrap_or_else(|ix| ix);
                return start + local.min(line.len());
            }
        }
        self.content.len()
    }

    fn vertical_target(&self, direction: i32) -> Option<usize> {
        let pos = self.position_for_offset(self.cursor_offset())?;
        let y = pos.y + self.line_height * (direction as f32) + self.line_height / 2.;
        let height = self.layout.as_ref()?.height;
        if y < px(0.) || y >= height {
            return None;
        }
        Some(self.offset_for_position(point(pos.x, y)))
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let Some(bounds) = self.bounds.as_ref() else {
            return 0;
        };
        let local = point(
            position.x - bounds.left(),
            position.y - bounds.top() + self.scroll_y,
        );
        self.offset_for_position(local)
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn previous_word(&self, offset: usize) -> usize {
        self.content
            .split_word_bound_indices()
            .rev()
            .find(|(idx, word)| *idx < offset && !word.trim().is_empty())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn next_word(&self, offset: usize) -> usize {
        self.content
            .split_word_bound_indices()
            .find(|(idx, word)| idx + word.len() > offset && !word.trim().is_empty())
            .map(|(idx, word)| idx + word.len())
            .unwrap_or(self.content.len())
    }

    /// Shapes the text for a given width. Used both to measure and to paint.
    fn shape(&self, width: Pixels, window: &mut Window, cx: &App) -> Layout {
        let t = theme(cx);
        let (text, color) = if self.content.is_empty() {
            (self.placeholder.to_string(), t.text_faint)
        } else {
            (self.content.clone(), self.color.unwrap_or(t.text))
        };
        let font = gpui::font(FONT);
        let wrap = if self.multiline { Some(width) } else { None };
        let mut lines = Vec::new();
        let mut top = px(0.);
        let mut start = 0;
        for piece in text.split('\n') {
            let mut runs = vec![TextRun {
                len: piece.len(),
                font: font.clone(),
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }];
            if let Some(marked) = self
                .marked_range
                .as_ref()
                .filter(|_| !self.content.is_empty())
            {
                let a = marked.start.clamp(start, start + piece.len()) - start;
                let b = marked.end.clamp(start, start + piece.len()) - start;
                if b > a {
                    let base = runs[0].clone();
                    let underline = Some(UnderlineStyle {
                        color: Some(color),
                        thickness: px(1.),
                        wavy: false,
                    });
                    runs = vec![
                        TextRun {
                            len: a,
                            ..base.clone()
                        },
                        TextRun {
                            len: b - a,
                            underline,
                            ..base.clone()
                        },
                        TextRun {
                            len: piece.len() - b,
                            ..base
                        },
                    ]
                    .into_iter()
                    .filter(|r| r.len > 0)
                    .collect();
                }
            }
            let shaped = window
                .text_system()
                .shape_text(
                    SharedString::from(piece.to_string()),
                    self.text_size,
                    &runs,
                    wrap,
                    None,
                )
                .ok()
                .and_then(|mut lines| {
                    if lines.is_empty() {
                        None
                    } else {
                        Some(lines.remove(0))
                    }
                })
                .unwrap_or_default();
            let height = shaped.size(self.line_height).height.max(self.line_height);
            lines.push((start, shaped, top));
            top += height;
            start += piece.len() + 1;
        }
        Layout {
            lines,
            height: top.max(self.line_height),
        }
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let new_text = if self.multiline {
            new_text.to_string()
        } else {
            new_text.replace('\n', " ")
        };
        self.content =
            self.content[0..range.start].to_owned() + &new_text + &self.content[range.end..];
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        self.show_caret();
        cx.emit(InputEvent::Changed);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        self.content =
            self.content[0..range.start].to_owned() + new_text + &self.content[range.end..];
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        cx.emit(InputEvent::Changed);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        let start = self.position_for_offset(range.start)?;
        let end = self.position_for_offset(range.end)?;
        let origin = point(bounds.left(), bounds.top() - self.scroll_y);
        Some(Bounds::from_corners(
            origin + start,
            origin + point(end.x.max(start.x + px(1.)), end.y + self.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.bounds?;
        let local = gpui::point(
            point.x - bounds.left(),
            point.y - bounds.top() + self.scroll_y,
        );
        Some(self.offset_to_utf16(self.offset_for_position(local)))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    selections: Vec<Bounds<Pixels>>,
    cursor: Option<Bounds<Pixels>>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        let input = self.input.clone();
        let layout_id =
            window.request_measured_layout(style, move |known, available, window, cx| {
                let width = known.width.unwrap_or(match available.width {
                    AvailableSpace::Definite(width) => width,
                    _ => px(320.),
                });
                let input = input.read(cx);
                let layout = input.shape(width, window, cx);
                let max = input.line_height * input.max_lines.max(1) as f32;
                size(width, layout.height.min(max))
            });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let layout = self.input.read(cx).shape(bounds.size.width, window, cx);
        self.input.update(cx, |input, _| {
            input.layout = Some(layout);
            input.bounds = Some(bounds);
        });
        let input = self.input.read(cx);
        let lh = input.line_height;

        // Keep the caret in view once the field stops growing and scrolls.
        let caret = input
            .position_for_offset(input.cursor_offset())
            .unwrap_or_default();
        let total = input.layout.as_ref().map(|l| l.height).unwrap_or(lh);
        let visible = bounds.size.height;
        let mut scroll_y = input.scroll_y;
        if caret.y < scroll_y {
            scroll_y = caret.y;
        } else if caret.y + lh > scroll_y + visible {
            scroll_y = caret.y + lh - visible;
        }
        scroll_y = scroll_y.clamp(px(0.), (total - visible).max(px(0.)));
        let origin = point(bounds.left(), bounds.top() - scroll_y);

        let mut selections = Vec::new();
        let range = input.selected_range.clone();
        let cursor = if range.is_empty() || input.content.is_empty() {
            Some(Bounds::new(origin + caret, size(px(1.5), lh)))
        } else {
            let start = input.position_for_offset(range.start).unwrap_or_default();
            let end = input.position_for_offset(range.end).unwrap_or_default();
            if (start.y - end.y).abs() < px(1.) {
                selections.push(Bounds::from_corners(
                    origin + start,
                    origin + point(end.x, end.y + lh),
                ));
            } else {
                selections.push(Bounds::from_corners(
                    origin + start,
                    point(bounds.right(), origin.y + start.y + lh),
                ));
                if end.y - start.y > lh {
                    selections.push(Bounds::from_corners(
                        point(bounds.left(), origin.y + start.y + lh),
                        point(bounds.right(), origin.y + end.y),
                    ));
                }
                selections.push(Bounds::from_corners(
                    point(bounds.left(), origin.y + end.y),
                    origin + point(end.x, end.y + lh),
                ));
            }
            None
        };
        self.input.update(cx, |input, _| input.scroll_y = scroll_y);
        PrepaintState { selections, cursor }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let t = theme(cx);
        let input = self.input.read(cx);
        let lh = input.line_height;
        let scroll_y = input.scroll_y;
        let lines: Vec<(WrappedLine, Pixels)> = input
            .layout
            .as_ref()
            .map(|l| {
                l.lines
                    .iter()
                    .map(|(_, line, top)| (line.clone(), *top))
                    .collect()
            })
            .unwrap_or_default();
        let caret_on = input.cursor_visible;
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for selection in prepaint.selections.drain(..) {
                window.paint_quad(fill(selection, t.selection));
            }
            for (line, top) in lines {
                let origin = point(bounds.left(), bounds.top() + top - scroll_y);
                let _ = line.paint(origin, lh, TextAlign::Left, None, window, cx);
            }
            if focus_handle.is_focused(window)
                && caret_on
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(fill(cursor, t.caret));
            }
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("text-input")
            .flex()
            .w_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_left))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::tab))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .text_size(self.text_size)
            .line_height(self.line_height)
            .child(TextElement { input: cx.entity() })
    }
}
