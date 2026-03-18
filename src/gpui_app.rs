use std::ops::Range;

use gpui::{
    App, Application, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla,
    KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    Pixels, Point, ScrollWheelEvent, ShapedLine, SharedString, Style, TextAlign, TextRun,
    UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions, WrappedLine, actions, div,
    fill, point, prelude::*, px, relative, rgb, rgba, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::CipherMode;

mod app;
mod io;

use app::BlockCipherApp;

actions!(
    block_cipher_text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum UiTab {
    Encrypt,
    Decrypt,
}

struct ModeOption {
    mode: CipherMode,
    title: &'static str,
}

const MODE_OPTIONS: [ModeOption; 3] = [
    ModeOption {
        mode: CipherMode::Cbc,
        title: "CBC",
    },
    ModeOption {
        mode: CipherMode::Cfb,
        title: "CFB",
    },
    ModeOption {
        mode: CipherMode::Ofb,
        title: "OFB",
    },
];

const CONTROL_HEIGHT: f32 = 48.0;
const TEXT_LINE_HEIGHT: f32 = 22.0;
const TEXTAREA_VISIBLE_LINES: usize = 6;
const TEXTAREA_VERTICAL_PADDING: f32 = 12.0;
const TEXTAREA_MAX_CONTENT_HEIGHT: f32 = TEXT_LINE_HEIGHT * TEXTAREA_VISIBLE_LINES as f32;
const TEXTAREA_MAX_HEIGHT: f32 = TEXTAREA_MAX_CONTENT_HEIGHT + (TEXTAREA_VERTICAL_PADDING * 2.0);
const RESULT_VIEW_HEIGHT: f32 = 160.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextInputDisplayMode {
    SingleLine,
    WrappedTextarea,
}

#[derive(Clone)]
struct TextInputStyle {
    background: Hsla,
    border: Hsla,
    border_focus: Hsla,
    invalid_background: Hsla,
    invalid_border: Hsla,
    invalid_border_focus: Hsla,
    text: Hsla,
    placeholder: Hsla,
    selection: Hsla,
    cursor: Hsla,
}

impl TextInputStyle {
    fn light() -> Self {
        Self {
            background: rgb(0xFFF9F6).into(),
            border: rgb(0xD9CBC2).into(),
            border_focus: rgb(0xCC8F88).into(),
            invalid_background: rgb(0xFCEDEC).into(),
            invalid_border: rgb(0xE19A96).into(),
            invalid_border_focus: rgb(0xD37873).into(),
            text: rgb(0x4A4655).into(),
            placeholder: rgba(0x8B8293CC).into(),
            selection: rgba(0xF0C7C199).into(),
            cursor: rgb(0xB87373).into(),
        }
    }
}

#[derive(Clone, Copy)]
enum TextInputValidator {
    None,
    ExactLength { len: usize, label: &'static str },
    Hex { label: &'static str },
}

struct TextInput {
    focus_handle: FocusHandle,
    content: String,
    placeholder: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_wrapped_lines: Option<Vec<WrappedLine>>,
    last_bounds: Option<Bounds<Pixels>>,
    last_line_height: Pixels,
    content_height: Pixels,
    is_selecting: bool,
    style: TextInputStyle,
    validator: TextInputValidator,
    reveal_validation_on_empty: bool,
    display_mode: TextInputDisplayMode,
    scroll_y: Pixels,
}

impl TextInput {
    fn new(
        window: &Window,
        cx: &mut App,
        placeholder: &str,
        validator: TextInputValidator,
    ) -> Entity<Self> {
        let _ = window;
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.to_string(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_wrapped_lines: None,
            last_bounds: None,
            last_line_height: Pixels::ZERO,
            content_height: Pixels::ZERO,
            is_selecting: false,
            style: TextInputStyle::light(),
            validator,
            reveal_validation_on_empty: false,
            display_mode: TextInputDisplayMode::SingleLine,
            scroll_y: Pixels::ZERO,
        })
    }

    fn new_multiline(
        window: &Window,
        cx: &mut App,
        placeholder: &str,
        validator: TextInputValidator,
    ) -> Entity<Self> {
        let _ = window;
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.to_string(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_wrapped_lines: None,
            last_bounds: None,
            last_line_height: Pixels::ZERO,
            content_height: Pixels::ZERO,
            is_selecting: false,
            style: TextInputStyle::light(),
            validator,
            reveal_validation_on_empty: false,
            display_mode: TextInputDisplayMode::WrappedTextarea,
            scroll_y: Pixels::ZERO,
        })
    }

    fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        let value = value.into();
        self.content = sanitize_single_line_text(&value);
        self.reveal_validation_on_empty = !self.content.is_empty();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn set_value_from_top(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        let value = value.into();
        self.content = sanitize_single_line_text(&value);
        self.reveal_validation_on_empty = !self.content.is_empty();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.scroll_y = Pixels::ZERO;
        cx.notify();
    }

    fn value(&self) -> String {
        self.content.clone()
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_value(String::new(), cx);
    }

    fn is_multiline(&self) -> bool {
        self.display_mode == TextInputDisplayMode::WrappedTextarea
    }

    fn field_height(&self) -> Pixels {
        if self.is_multiline() {
            let content_height = self.content_height.max(px(TEXT_LINE_HEIGHT));
            let padded_height = content_height + px(TEXTAREA_VERTICAL_PADDING * 2.0);
            padded_height.clamp(px(CONTROL_HEIGHT), px(TEXTAREA_MAX_HEIGHT))
        } else {
            px(CONTROL_HEIGHT)
        }
    }

    fn max_scroll(&self, viewport_height: Pixels) -> Pixels {
        (self.content_height - viewport_height).max(Pixels::ZERO)
    }

    fn clamp_scroll(&mut self, viewport_height: Pixels) {
        self.scroll_y = self
            .scroll_y
            .clamp(Pixels::ZERO, self.max_scroll(viewport_height));
    }

    fn reveal_validation(&mut self, cx: &mut Context<Self>) {
        self.reveal_validation_on_empty = true;
        cx.notify();
    }

    fn validation_message(&self) -> Option<String> {
        match self.validator {
            TextInputValidator::None => None,
            TextInputValidator::ExactLength { len, label } => {
                if self.content.is_empty() && !self.reveal_validation_on_empty {
                    None
                } else if self.content.len() == len {
                    None
                } else {
                    Some(format!("{label} must be exactly {len} characters."))
                }
            }
            TextInputValidator::Hex { label } => {
                if self.content.is_empty() {
                    None
                } else if self.content.len() % 2 != 0
                    || !self.content.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    Some(format!("{label} must be valid hex."))
                } else {
                    None
                }
            }
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
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

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_multiline() {
            return;
        }

        let Some(bounds) = self.last_bounds else {
            return;
        };

        self.clamp_scroll(bounds.size.height);
        let delta = event.delta.pixel_delta(px(20.)).y;
        self.scroll_y =
            (self.scroll_y - delta).clamp(Pixels::ZERO, self.max_scroll(bounds.size.height));
        cx.notify();
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
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
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        if self.is_multiline() {
            let (Some(bounds), Some(lines)) =
                (self.last_bounds.as_ref(), self.last_wrapped_lines.as_ref())
            else {
                return 0;
            };

            let local = point(
                (position.x - bounds.left()).max(Pixels::ZERO),
                (position.y - bounds.top() + self.scroll_y).max(Pixels::ZERO),
            );
            return wrapped_lines_index_for_position(lines, local, self.last_line_height)
                .min(self.content.len());
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
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

    fn position_for_index(&self, index: usize) -> Option<Point<Pixels>> {
        if self.is_multiline() {
            return wrapped_lines_position_for_index(
                self.last_wrapped_lines.as_deref()?,
                index,
                self.last_line_height,
            );
        }

        let line = self.last_layout.as_ref()?;
        Some(point(line.x_for_index(index), Pixels::ZERO))
    }
}

fn wrapped_lines_content_height(lines: &[WrappedLine], line_height: Pixels) -> Pixels {
    lines.iter().fold(Pixels::ZERO, |height, line| {
        height + line.size(line_height).height
    })
}

fn wrapped_lines_position_for_index(
    lines: &[WrappedLine],
    index: usize,
    line_height: Pixels,
) -> Option<Point<Pixels>> {
    let mut line_origin_y = Pixels::ZERO;
    let mut line_start_ix = 0;

    for line in lines {
        let line_end_ix = line_start_ix + line.len();
        if index <= line_end_ix {
            let relative_index = index.saturating_sub(line_start_ix);
            let position = line.position_for_index(relative_index, line_height)?;
            return Some(point(position.x, line_origin_y + position.y));
        }
        line_origin_y += line.size(line_height).height;
        line_start_ix = line_end_ix;
    }

    lines.last().and_then(|line| {
        line.position_for_index(line.len(), line_height)
            .map(|position| {
                point(
                    position.x,
                    line_origin_y - line.size(line_height).height + position.y,
                )
            })
    })
}

fn wrapped_lines_index_for_position(
    lines: &[WrappedLine],
    position: Point<Pixels>,
    line_height: Pixels,
) -> usize {
    let mut line_origin_y = Pixels::ZERO;
    let mut line_start_ix = 0;

    for line in lines {
        let line_height_total = line.size(line_height).height;
        let line_end_ix = line_start_ix + line.len();
        if position.y <= line_origin_y + line_height_total {
            let local_position = point(position.x, (position.y - line_origin_y).max(Pixels::ZERO));
            let relative_index = line
                .closest_index_for_position(local_position, line_height)
                .unwrap_or_else(|index| index);
            return line_start_ix + relative_index;
        }
        line_origin_y += line_height_total;
        line_start_ix = line_end_ix;
    }

    line_start_ix
}

fn wrapped_lines_cursor_quad(
    lines: &[WrappedLine],
    cursor: usize,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    scroll_y: Pixels,
    color: Hsla,
) -> Option<PaintQuad> {
    let cursor_position = wrapped_lines_position_for_index(lines, cursor, line_height)?;
    Some(fill(
        Bounds::new(
            point(
                bounds.left() + cursor_position.x,
                bounds.top() + cursor_position.y - scroll_y,
            ),
            size(px(2.), line_height),
        ),
        color,
    ))
}

fn wrapped_lines_selection_quads(
    lines: &[WrappedLine],
    selected_range: &Range<usize>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    scroll_y: Pixels,
    color: Hsla,
) -> Vec<PaintQuad> {
    if selected_range.is_empty() {
        return Vec::new();
    }

    let mut quads = Vec::new();
    let mut line_origin_y = Pixels::ZERO;
    let mut line_start_ix = 0;

    for line in lines {
        let line_end_ix = line_start_ix + line.len();
        if selected_range.end <= line_start_ix || selected_range.start >= line_end_ix {
            line_origin_y += line.size(line_height).height;
            line_start_ix = line_end_ix;
            continue;
        }

        let selection_start = selected_range.start.max(line_start_ix) - line_start_ix;
        let selection_end = selected_range.end.min(line_end_ix) - line_start_ix;
        let Some(start_position) = line.position_for_index(selection_start, line_height) else {
            line_origin_y += line.size(line_height).height;
            line_start_ix = line_end_ix;
            continue;
        };
        let Some(end_position) = line.position_for_index(selection_end, line_height) else {
            line_origin_y += line.size(line_height).height;
            line_start_ix = line_end_ix;
            continue;
        };

        let start_visual_line = (start_position.y / line_height) as usize;
        let end_visual_line = (end_position.y / line_height) as usize;

        for visual_line in start_visual_line..=end_visual_line {
            let y = bounds.top() + line_origin_y + line_height * visual_line - scroll_y;
            let x1 = if visual_line == start_visual_line {
                start_position.x
            } else {
                Pixels::ZERO
            };
            let x2 = if visual_line == end_visual_line {
                end_position.x
            } else {
                line.width()
            };

            if x2 > x1 {
                quads.push(fill(
                    Bounds::new(point(bounds.left() + x1, y), size(x2 - x1, line_height)),
                    color,
                ));
            }
        }

        line_origin_y += line.size(line_height).height;
        line_start_ix = line_end_ix;
    }

    quads
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
        let sanitized_new_text = sanitize_single_line_text(new_text);

        self.content = self.content[0..range.start].to_owned()
            + &sanitized_new_text
            + &self.content[range.end..];
        self.reveal_validation_on_empty = !self.content.is_empty();
        self.selected_range =
            range.start + sanitized_new_text.len()..range.start + sanitized_new_text.len();
        self.marked_range.take();
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
        let sanitized_new_text = sanitize_single_line_text(new_text);

        self.content = self.content[0..range.start].to_owned()
            + &sanitized_new_text
            + &self.content[range.end..];
        self.reveal_validation_on_empty = !self.content.is_empty();
        if !sanitized_new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + sanitized_new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| {
                range.start + sanitized_new_text.len()..range.start + sanitized_new_text.len()
            });

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
        if self.is_multiline() {
            let start = self.position_for_index(range.start)?;
            let end = self.position_for_index(range.end)?;
            let top = bounds.top() + start.y - self.scroll_y;
            let bottom = bounds.top() + end.y + self.last_line_height - self.scroll_y;
            return Some(Bounds::from_corners(
                point(bounds.left() + start.x, top),
                point(bounds.left() + end.x.max(start.x + px(2.)), bottom),
            ));
        }

        let last_layout = self.last_layout.as_ref()?;
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let utf8_index = if self.is_multiline() {
            self.index_for_mouse_position(point)
        } else {
            let line_point = self.last_bounds?.localize(&point)?;
            let last_layout = self.last_layout.as_ref()?;
            last_layout.index_for_x(point.x - line_point.x)?
        };
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct TextInputElement {
    input: Entity<TextInput>,
}

struct TextInputPrepaint {
    line: Option<ShapedLine>,
    wrapped_lines: Option<Vec<WrappedLine>>,
    cursor: Option<PaintQuad>,
    selections: Vec<PaintQuad>,
    content_height: Pixels,
    scroll_y: Pixels,
    line_height: Pixels,
}

impl IntoElement for TextInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = ();
    type PrepaintState = TextInputPrepaint;

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
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let is_multiline = self.input.read(cx).is_multiline();
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = if is_multiline {
            relative(1.).into()
        } else {
            px(TEXT_LINE_HEIGHT).into()
        };
        (window.request_layout(style, [], cx), ())
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
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();
        let line_height = px(TEXT_LINE_HEIGHT);

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), input.style.placeholder)
        } else {
            (content, input.style.text)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let display_text: SharedString = display_text.into();
        if input.is_multiline() {
            let lines = window
                .text_system()
                .shape_text(
                    display_text,
                    font_size,
                    &runs,
                    Some(bounds.size.width),
                    None,
                )
                .map(|lines| lines.into_iter().collect::<Vec<_>>())
                .unwrap_or_default();
            let content_height = wrapped_lines_content_height(&lines, line_height);
            let max_scroll = (content_height - bounds.size.height).max(Pixels::ZERO);
            let mut scroll_y = input.scroll_y.clamp(Pixels::ZERO, max_scroll);

            if let Some(cursor_pos) = wrapped_lines_position_for_index(&lines, cursor, line_height)
            {
                if cursor_pos.y < scroll_y {
                    scroll_y = cursor_pos.y;
                } else if cursor_pos.y + line_height > scroll_y + bounds.size.height {
                    scroll_y = (cursor_pos.y + line_height - bounds.size.height)
                        .clamp(Pixels::ZERO, max_scroll);
                }
            }

            let selections = if selected_range.is_empty() {
                Vec::new()
            } else {
                wrapped_lines_selection_quads(
                    &lines,
                    &selected_range,
                    bounds,
                    line_height,
                    scroll_y,
                    input.style.selection,
                )
            };
            let cursor_quad = if selected_range.is_empty() {
                wrapped_lines_cursor_quad(
                    &lines,
                    cursor,
                    bounds,
                    line_height,
                    scroll_y,
                    input.style.cursor,
                )
            } else {
                None
            };

            TextInputPrepaint {
                line: None,
                wrapped_lines: Some(lines),
                cursor: cursor_quad,
                selections,
                content_height,
                scroll_y,
                line_height,
            }
        } else {
            let line = window
                .text_system()
                .shape_line(display_text, font_size, &runs, None);

            let cursor_pos = line.x_for_index(cursor);
            let (selections, cursor) = if selected_range.is_empty() {
                (
                    Vec::new(),
                    Some(fill(
                        Bounds::new(
                            point(bounds.left() + cursor_pos, bounds.top()),
                            size(px(2.), bounds.bottom() - bounds.top()),
                        ),
                        input.style.cursor,
                    )),
                )
            } else {
                (
                    vec![fill(
                        Bounds::from_corners(
                            point(
                                bounds.left() + line.x_for_index(selected_range.start),
                                bounds.top(),
                            ),
                            point(
                                bounds.left() + line.x_for_index(selected_range.end),
                                bounds.bottom(),
                            ),
                        ),
                        input.style.selection,
                    )],
                    None,
                )
            };

            TextInputPrepaint {
                line: Some(line),
                wrapped_lines: None,
                cursor,
                selections,
                content_height: line_height,
                scroll_y: Pixels::ZERO,
                line_height,
            }
        }
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

        let mut stored_line = None;
        let mut stored_wrapped_lines = None;

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            window.paint_layer(bounds, |window| {
                for selection in prepaint.selections.drain(..) {
                    window.paint_quad(selection);
                }

                if let Some(line) = prepaint.line.take() {
                    line.paint(bounds.origin, prepaint.line_height, window, cx)
                        .expect("paint text input line");
                    stored_line = Some(line);
                } else if let Some(lines) = prepaint.wrapped_lines.take() {
                    let mut line_origin = point(bounds.left(), bounds.top() - prepaint.scroll_y);
                    for line in &lines {
                        line.paint(
                            line_origin,
                            prepaint.line_height,
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        )
                        .expect("paint wrapped text input");
                        line_origin.y += line.size(prepaint.line_height).height;
                    }
                    stored_wrapped_lines = Some(lines);
                }

                if focus_handle.is_focused(window)
                    && let Some(cursor) = prepaint.cursor.take()
                {
                    window.paint_quad(cursor);
                }
            });
        });

        self.input.update(cx, |input, _cx| {
            input.last_layout = stored_line;
            input.last_wrapped_lines = stored_wrapped_lines;
            input.last_bounds = Some(bounds);
            input.last_line_height = prepaint.line_height;
            input.content_height = prepaint.content_height;
            input.scroll_y = prepaint.scroll_y;
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_message = self.validation_message();
        let is_invalid = validation_message.is_some();
        let border_color = if is_invalid && self.focus_handle(cx).is_focused(_window) {
            self.style.invalid_border_focus
        } else if is_invalid {
            self.style.invalid_border
        } else if self.focus_handle(cx).is_focused(_window) {
            self.style.border_focus
        } else {
            self.style.border
        };
        let background_color = if is_invalid {
            self.style.invalid_background
        } else {
            self.style.background
        };

        let mut shell = div()
            .id("text-input-shell")
            .flex()
            .flex_col()
            .key_context("BlockCipherTextInput")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .when(self.is_multiline(), |this| {
                this.on_scroll_wheel(cx.listener(Self::on_scroll))
            });

        if let Some(message) = validation_message {
            shell = shell.child(
                div()
                    .pb_1()
                    .text_sm()
                    .text_color(rgb(0xB46D72))
                    .child(message),
            );
        }

        shell.child(
            div()
                .cursor(CursorStyle::IBeam)
                .h(self.field_height())
                .border_1()
                .border_color(border_color)
                .rounded_none()
                .bg(background_color)
                .line_height(px(TEXT_LINE_HEIGHT))
                .text_size(px(16.))
                .p_3()
                .child(TextInputElement { input: cx.entity() }),
        )
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct ScrollTextView {
    content: String,
    last_bounds: Option<Bounds<Pixels>>,
    last_line_height: Pixels,
    last_wrapped_lines: Vec<WrappedLine>,
    content_height: Pixels,
    scroll_y: Pixels,
}

impl ScrollTextView {
    fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self {
            content: String::new(),
            last_bounds: None,
            last_line_height: px(TEXT_LINE_HEIGHT),
            last_wrapped_lines: Vec::new(),
            content_height: Pixels::ZERO,
            scroll_y: Pixels::ZERO,
        })
    }

    fn set_content(&mut self, content: impl Into<String>, cx: &mut Context<Self>) {
        self.content = content.into();
        self.scroll_y = Pixels::ZERO;
        cx.notify();
    }

    fn max_scroll(&self, viewport_height: Pixels) -> Pixels {
        (self.content_height - viewport_height).max(Pixels::ZERO)
    }

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(bounds) = self.last_bounds else {
            return;
        };

        let delta = event.delta.pixel_delta(px(20.)).y;
        self.scroll_y =
            (self.scroll_y - delta).clamp(Pixels::ZERO, self.max_scroll(bounds.size.height));
        cx.notify();
    }
}

struct ScrollTextViewElement {
    view: Entity<ScrollTextView>,
}

struct ScrollTextViewPrepaint {
    wrapped_lines: Vec<WrappedLine>,
    content_height: Pixels,
    scroll_y: Pixels,
    line_height: Pixels,
}

impl IntoElement for ScrollTextViewElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ScrollTextViewElement {
    type RequestLayoutState = ();
    type PrepaintState = ScrollTextViewPrepaint;

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
        _window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (_window.request_layout(style, [], cx), ())
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
        let view = self.view.read(cx);
        let content = if view.content.is_empty() {
            "-".to_string()
        } else {
            view.content.clone()
        };
        let text_style = window.text_style();
        let line_height = px(TEXT_LINE_HEIGHT);
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let runs = [TextRun {
            len: content.len(),
            font: text_style.font(),
            color: rgb(0x4E4A59).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];
        let wrapped_lines = window
            .text_system()
            .shape_text(
                content.into(),
                font_size,
                &runs,
                Some(bounds.size.width),
                None,
            )
            .map(|lines| lines.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let content_height = wrapped_lines_content_height(&wrapped_lines, line_height);
        let scroll_y = view.scroll_y.clamp(
            Pixels::ZERO,
            (content_height - bounds.size.height).max(Pixels::ZERO),
        );

        ScrollTextViewPrepaint {
            wrapped_lines,
            content_height,
            scroll_y,
            line_height,
        }
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
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            window.paint_layer(bounds, |window| {
                let mut line_origin = point(bounds.left(), bounds.top() - prepaint.scroll_y);
                for line in &prepaint.wrapped_lines {
                    line.paint(
                        line_origin,
                        prepaint.line_height,
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    )
                    .expect("paint wrapped output");
                    line_origin.y += line.size(prepaint.line_height).height;
                }
            });
        });

        let wrapped_lines = std::mem::take(&mut prepaint.wrapped_lines);
        self.view.update(cx, |view, _cx| {
            view.last_bounds = Some(bounds);
            view.last_line_height = prepaint.line_height;
            view.content_height = prepaint.content_height;
            view.scroll_y = prepaint.scroll_y;
            view.last_wrapped_lines = wrapped_lines;
        });
    }
}

impl Render for ScrollTextView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h(px(RESULT_VIEW_HEIGHT))
            .pr_2()
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(ScrollTextViewElement { view: cx.entity() })
    }
}

fn sanitize_single_line_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\\n")
}

pub fn run_gpui() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        ]);

        let bounds = Bounds::centered(None, size(px(1180.), px(860.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                ..Default::default()
            },
            |window, cx| cx.new(|cx| BlockCipherApp::new(window, cx)),
        )
        .expect("open gpui window");

        cx.activate(true);
    });
}
