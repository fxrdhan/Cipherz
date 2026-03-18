use std::ops::Range;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use gpui::{
    App, Application, Bounds, ClickEvent, ClipboardItem, Context, CursorStyle, Div, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla,
    KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    Pixels, Point, ShapedLine, SharedString, StatefulInteractiveElement, Style, TextRun,
    UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions, actions, div, fill, point,
    prelude::*, px, relative, rgb, rgba, size,
};
use rfd::FileDialog;
use unicode_segmentation::UnicodeSegmentation;

use crate::{BLOCK_SIZE, CipherMode, KEY_SIZE, decrypt_message, derive_bytes, encrypt_message};

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
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    style: TextInputStyle,
    validator: TextInputValidator,
    reveal_validation_on_empty: bool,
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
            last_bounds: None,
            is_selecting: false,
            style: TextInputStyle::light(),
            validator,
            reveal_validation_on_empty: false,
        })
    }

    fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.content = value.into();
        self.reveal_validation_on_empty = !self.content.is_empty();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn value(&self) -> String {
        self.content.clone()
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_value(String::new(), cx);
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
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
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

        self.content =
            self.content[0..range.start].to_owned() + new_text + &self.content[range.end..];
        self.reveal_validation_on_empty = !self.content.is_empty();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
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

        self.content =
            self.content[0..range.start].to_owned() + new_text + &self.content[range.end..];
        self.reveal_validation_on_empty = !self.content.is_empty();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
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
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct TextInputElement {
    input: Entity<TextInput>,
}

struct TextInputPrepaint {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
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
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
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
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
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
                Some(fill(
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
                )),
                None,
            )
        };

        TextInputPrepaint {
            line: Some(line),
            cursor,
            selection,
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

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }

        let line = prepaint.line.take().expect("line should exist");
        line.paint(bounds.origin, window.line_height(), window, cx)
            .expect("paint text input line");

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
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
            .on_mouse_move(cx.listener(Self::on_mouse_move));

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
                .border_1()
                .border_color(border_color)
                .rounded_none()
                .bg(background_color)
                .line_height(px(22.))
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

struct BlockCipherApp {
    active_tab: UiTab,
    selected_mode: CipherMode,
    encrypt_key: Entity<TextInput>,
    encrypt_iv: Entity<TextInput>,
    encrypt_plaintext: Entity<TextInput>,
    decrypt_key: Entity<TextInput>,
    decrypt_iv: Entity<TextInput>,
    decrypt_ciphertext: Entity<TextInput>,
    encrypt_result: String,
    decrypt_result: String,
    encrypt_copy_done_until: Option<Instant>,
    decrypt_copy_done_until: Option<Instant>,
}

impl BlockCipherApp {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let encrypt_key = TextInput::new(
            window,
            cx,
            "Key (16 chars)",
            TextInputValidator::ExactLength {
                len: KEY_SIZE,
                label: "Key",
            },
        );
        let encrypt_iv = TextInput::new(
            window,
            cx,
            "IV (8 chars)",
            TextInputValidator::ExactLength {
                len: BLOCK_SIZE,
                label: "IV",
            },
        );
        let encrypt_plaintext = TextInput::new(window, cx, "Plaintext", TextInputValidator::None);
        let decrypt_key = TextInput::new(
            window,
            cx,
            "Key (16 chars)",
            TextInputValidator::ExactLength {
                len: KEY_SIZE,
                label: "Key",
            },
        );
        let decrypt_iv = TextInput::new(
            window,
            cx,
            "IV (8 chars)",
            TextInputValidator::ExactLength {
                len: BLOCK_SIZE,
                label: "IV",
            },
        );
        let decrypt_ciphertext = TextInput::new(
            window,
            cx,
            "Ciphertext (hex)",
            TextInputValidator::Hex {
                label: "Ciphertext",
            },
        );

        encrypt_key.update(cx, |input, cx| input.set_value("", cx));
        encrypt_iv.update(cx, |input, cx| input.set_value("", cx));
        encrypt_plaintext.update(cx, |input, cx| input.set_value("hello", cx));
        decrypt_key.update(cx, |input, cx| input.set_value("", cx));
        decrypt_iv.update(cx, |input, cx| input.set_value("", cx));

        Self {
            active_tab: UiTab::Encrypt,
            selected_mode: CipherMode::Cbc,
            encrypt_key,
            encrypt_iv,
            encrypt_plaintext,
            decrypt_key,
            decrypt_iv,
            decrypt_ciphertext,
            encrypt_result: String::new(),
            decrypt_result: String::new(),
            encrypt_copy_done_until: None,
            decrypt_copy_done_until: None,
        }
    }

    fn set_tab(&mut self, tab: UiTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    fn set_mode(&mut self, mode: CipherMode, cx: &mut Context<Self>) {
        self.selected_mode = mode;
        cx.notify();
    }

    fn encrypt_now(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key_text = self.encrypt_key.read(cx).value();
        let iv_text = self.encrypt_iv.read(cx).value();
        let plaintext = self.encrypt_plaintext.read(cx).value();

        if validate_key_iv(&key_text, &iv_text).is_some() {
            self.encrypt_key
                .update(cx, |input, cx| input.reveal_validation(cx));
            self.encrypt_iv
                .update(cx, |input, cx| input.reveal_validation(cx));
            self.encrypt_result.clear();
            self.encrypt_copy_done_until = None;
            cx.notify();
            return;
        }

        let key = derive_bytes::<KEY_SIZE>(&key_text);
        let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
        let encrypted = encrypt_message(self.selected_mode, plaintext.as_bytes(), &key, &iv);

        self.encrypt_result = hex_string(&encrypted);
        self.encrypt_copy_done_until = None;
        self.decrypt_ciphertext.update(cx, |input, cx| {
            input.set_value(self.encrypt_result.clone(), cx)
        });
        self.decrypt_key
            .update(cx, |input, cx| input.set_value(key_text, cx));
        self.decrypt_iv
            .update(cx, |input, cx| input.set_value(iv_text, cx));
        cx.notify();
    }

    fn decrypt_now(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key_text = self.decrypt_key.read(cx).value();
        let iv_text = self.decrypt_iv.read(cx).value();
        let ciphertext_text = self.decrypt_ciphertext.read(cx).value();

        if validate_key_iv(&key_text, &iv_text).is_some() {
            self.decrypt_key
                .update(cx, |input, cx| input.reveal_validation(cx));
            self.decrypt_iv
                .update(cx, |input, cx| input.reveal_validation(cx));
            self.decrypt_result.clear();
            self.decrypt_copy_done_until = None;
            cx.notify();
            return;
        }

        let Some(ciphertext) = hex_to_bytes(&ciphertext_text) else {
            self.decrypt_result.clear();
            self.decrypt_copy_done_until = None;
            cx.notify();
            return;
        };

        let key = derive_bytes::<KEY_SIZE>(&key_text);
        let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
        let Some(plaintext) = decrypt_message(self.selected_mode, &ciphertext, &key, &iv) else {
            self.decrypt_result.clear();
            self.decrypt_copy_done_until = None;
            cx.notify();
            return;
        };

        self.decrypt_result = String::from_utf8_lossy(&plaintext).into_owned();
        self.decrypt_copy_done_until = None;
        cx.notify();
    }

    fn clear_encrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.encrypt_plaintext
            .update(cx, |input, cx| input.clear(cx));
        self.encrypt_result.clear();
        self.encrypt_copy_done_until = None;
        cx.notify();
    }

    fn clear_decrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.decrypt_ciphertext
            .update(cx, |input, cx| input.clear(cx));
        self.decrypt_result.clear();
        self.decrypt_copy_done_until = None;
        cx.notify();
    }

    fn copy_encrypt_result(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.encrypt_result.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.encrypt_result.clone()));
            self.encrypt_copy_done_until = Some(Instant::now() + Duration::from_millis(700));
            cx.notify();
            window
                .spawn(cx, async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(700))
                        .await;
                    cx.update(|window, _cx| window.refresh()).ok();
                })
                .detach();
        }
    }

    fn copy_decrypt_result(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.decrypt_result.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.decrypt_result.clone()));
            self.decrypt_copy_done_until = Some(Instant::now() + Duration::from_millis(700));
            cx.notify();
            window
                .spawn(cx, async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(700))
                        .await;
                    cx.update(|window, _cx| window.refresh()).ok();
                })
                .detach();
        }
    }

    fn export_encrypt_result(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = save_output_with_dialog("ciphertext", &self.encrypt_result);
        cx.notify();
    }

    fn export_decrypt_result(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = save_output_with_dialog("plaintext", &self.decrypt_result);
        cx.notify();
    }

    fn import_plaintext(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Ok(Some(content)) = import_text_with_dialog() {
            self.encrypt_plaintext.update(cx, |input, cx| {
                input.set_value(content.replace("\r\n", "\n"), cx)
            });
        }
        cx.notify();
    }

    fn render_tab_button(
        &self,
        id: &'static str,
        label: &'static str,
        tab: UiTab,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.active_tab == tab;
        let bg: Hsla = if active {
            rgb(0xE1CEC2).into()
        } else {
            rgb(0xFFFDF9).into()
        };
        let fg: Hsla = if active {
            rgb(0x474250).into()
        } else {
            rgb(0x7B7287).into()
        };
        let border: Hsla = rgb(0xD1C2B7).into();

        div()
            .id(id)
            .cursor_pointer()
            .px_4()
            .py_2()
            .border_1()
            .border_color(border)
            .bg(bg)
            .text_color(fg)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(label)
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| view.set_tab(tab, cx)))
    }

    fn render_mode_chip(&self, option: &ModeOption, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = option.mode;
        let active = self.selected_mode == mode;
        let bg: Hsla = if active {
            rgb(0xF0C9C1).into()
        } else {
            rgb(0xFFFDF9).into()
        };
        let fg: Hsla = if active {
            rgb(0x474250).into()
        } else {
            rgb(0xB46D72).into()
        };
        let border: Hsla = rgb(0xDEB6B0).into();
        let id = match mode {
            CipherMode::Cbc => "mode-cbc",
            CipherMode::Cfb => "mode-cfb",
            CipherMode::Ofb => "mode-ofb",
        };

        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .w(px(120.))
            .h(px(44.))
            .p_3()
            .border_1()
            .border_color(border)
            .bg(bg)
            .text_color(fg)
            .child(
                div()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(option.title),
            )
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| view.set_mode(mode, cx)))
    }

    fn render_result_card(&self, title: &str, body: &str) -> impl IntoElement {
        let should_scroll = body.lines().count() > 4 || body.len() > 160;
        let scroll_id = if title == "Ciphertext" {
            "ciphertext-output-scroll"
        } else {
            "plaintext-output-scroll"
        };
        let body_element = if should_scroll {
            div()
                .id(scroll_id)
                .pt_1()
                .w_full()
                .h(px(160.))
                .overflow_y_scroll()
                .pr_2()
                .text_color(rgb(0x4E4A59))
                .text_sm()
                .child(if body.is_empty() {
                    "-".to_string()
                } else {
                    body.to_string()
                })
                .into_any_element()
        } else {
            div()
                .pt_1()
                .text_color(rgb(0x4E4A59))
                .text_sm()
                .min_h(px(28.))
                .child(if body.is_empty() {
                    "-".to_string()
                } else {
                    body.to_string()
                })
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_1()
            .child(
                div()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0xAF6A6F))
                    .child(title.to_string()),
            )
            .child(body_element)
    }

    fn render_encrypt_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut action_children = vec![
            action_button(
                "Encrypt",
                rgb(0xDCCCCF).into(),
                rgb(0x474250).into(),
                rgb(0xC7B6D9).into(),
            )
            .id("encrypt-now")
            .on_click(cx.listener(Self::encrypt_now))
            .into_any_element(),
            action_button(
                "Clear",
                rgb(0xFFFDF9).into(),
                rgb(0x7B7287).into(),
                rgb(0xD1C2B7).into(),
            )
            .id("encrypt-clear")
            .on_click(cx.listener(Self::clear_encrypt))
            .into_any_element(),
        ];

        if !self.encrypt_result.is_empty() {
            let copy_label = if self
                .encrypt_copy_done_until
                .is_some_and(|deadline| deadline > Instant::now())
            {
                "Done"
            } else {
                "Copy"
            };
            action_children.push(
                div()
                    .text_color(rgb(0xB7A79B))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("|")
                    .into_any_element(),
            );
            action_children.push(
                action_button(
                    copy_label,
                    rgb(0xFFFDF9).into(),
                    rgb(0x7B7287).into(),
                    rgb(0xD1C2B7).into(),
                )
                .id("encrypt-copy")
                .on_click(cx.listener(Self::copy_encrypt_result))
                .into_any_element(),
            );
            action_children.push(
                action_button(
                    "Export",
                    rgb(0xFFFDF9).into(),
                    rgb(0x7B7287).into(),
                    rgb(0xD1C2B7).into(),
                )
                .id("encrypt-export")
                .on_click(cx.listener(Self::export_encrypt_result))
                .into_any_element(),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(self.encrypt_key.clone())
            .child(self.encrypt_iv.clone())
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_3()
                    .w_full()
                    .child(div().flex_1().child(self.encrypt_plaintext.clone()))
                    .child(
                        action_button(
                            "Import",
                            rgb(0xFFFDF9).into(),
                            rgb(0x7B7287).into(),
                            rgb(0xD1C2B7).into(),
                        )
                        .id("encrypt-import")
                        .on_click(cx.listener(Self::import_plaintext)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .children(action_children),
            )
            .child(self.render_result_card("Ciphertext", &self.encrypt_result))
    }

    fn render_decrypt_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut action_children = vec![
            action_button(
                "Decrypt",
                rgb(0xDCCCCF).into(),
                rgb(0x474250).into(),
                rgb(0xC7B6D9).into(),
            )
            .id("decrypt-now")
            .on_click(cx.listener(Self::decrypt_now))
            .into_any_element(),
            action_button(
                "Clear",
                rgb(0xFFFDF9).into(),
                rgb(0x7B7287).into(),
                rgb(0xD1C2B7).into(),
            )
            .id("decrypt-clear")
            .on_click(cx.listener(Self::clear_decrypt))
            .into_any_element(),
        ];

        if !self.decrypt_result.is_empty() {
            let copy_label = if self
                .decrypt_copy_done_until
                .is_some_and(|deadline| deadline > Instant::now())
            {
                "Done"
            } else {
                "Copy"
            };
            action_children.push(
                div()
                    .text_color(rgb(0xB7A79B))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("|")
                    .into_any_element(),
            );
            action_children.push(
                action_button(
                    copy_label,
                    rgb(0xFFFDF9).into(),
                    rgb(0x7B7287).into(),
                    rgb(0xD1C2B7).into(),
                )
                .id("decrypt-copy")
                .on_click(cx.listener(Self::copy_decrypt_result))
                .into_any_element(),
            );
            action_children.push(
                action_button(
                    "Export",
                    rgb(0xFFFDF9).into(),
                    rgb(0x7B7287).into(),
                    rgb(0xD1C2B7).into(),
                )
                .id("decrypt-export")
                .on_click(cx.listener(Self::export_decrypt_result))
                .into_any_element(),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(self.decrypt_key.clone())
            .child(self.decrypt_iv.clone())
            .child(self.decrypt_ciphertext.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .children(action_children),
            )
            .child(self.render_result_card("Plaintext", &self.decrypt_result))
    }
}

impl Render for BlockCipherApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = match self.active_tab {
            UiTab::Encrypt => self.render_encrypt_panel(cx).into_any_element(),
            UiTab::Decrypt => self.render_decrypt_panel(cx).into_any_element(),
        };

        div()
            .size_full()
            .bg(rgb(0xFFF9F6))
            .text_color(rgb(0x656072))
            .child(
                // Custom titlebar strip (draggable, with window controls on right)
                div()
                    .id("titlebar")
                    .w_full()
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_3()
                    .gap_2()
                    .bg(rgb(0x4F495C))
                    .border_b_1()
                    .border_color(rgb(0x3F394A))
                    // Minimize button
                    .child(
                        div()
                            .id("btn-minimize")
                            .cursor_pointer()
                            .w(px(16.))
                            .h(px(16.))
                            .rounded_full()
                            .bg(rgb(0xF2D9A0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .on_click(|_, window, _| {
                                window.minimize_window();
                            }),
                    )
                    // Close button
                    .child(
                        div()
                            .id("btn-close")
                            .cursor_pointer()
                            .w(px(16.))
                            .h(px(16.))
                            .rounded_full()
                            .bg(rgb(0xEDB7B1))
                            .flex()
                            .items_center()
                            .justify_center()
                            .on_click(|_, window, _| {
                                window.remove_window();
                            }),
                    ),
            )
            .child(
                div()
                    .id("app-scroll")
                    .flex_1()
                    .w_full()
                    .h(px(828.))
                    .overflow_y_scroll()
                    .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .w_full()
                                .p_6()
                                .bg(rgb(0xFFF9F6))
                                .border_b_1()
                                .border_color(rgb(0xE6D9D1))
                                .child(
                                    div()
                                        .max_w(px(1120.))
                                        .mx_auto()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
                                        .child(
                                            div()
                                                .flex()
                                                .justify_between()
                                                .items_start()
                                                .gap_4()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1()
                                                        .child(
                                                            div()
                                                                .text_size(px(28.))
                                                                .font_weight(gpui::FontWeight::BOLD)
                                                                .text_color(rgb(0x5E586C))
                                                                .child("Block Cipher"),
                                                        )
                                                        .child(div().text_sm().text_color(rgb(0x8B8293)).child(
                                                            "64-bit block | 128-bit key | 8 rounds",
                                                        )),
                                                ),
                                        )
                                        .child(
                                            div().flex().gap_3().children([
                                                self.render_tab_button(
                                                    "tab-encrypt",
                                                    "Encrypt",
                                                    UiTab::Encrypt,
                                                    cx,
                                                )
                                                .into_any_element(),
                                                self.render_tab_button(
                                                    "tab-decrypt",
                                                    "Decrypt",
                                                    UiTab::Decrypt,
                                                    cx,
                                                )
                                                .into_any_element(),
                                            ]),
                                        )
                                        .child(div().flex().gap_3().flex_wrap().children(
                                            MODE_OPTIONS.iter().map(|option| {
                                                self.render_mode_chip(option, cx).into_any_element()
                                            }),
                                        ))
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .p_6()
                                .bg(rgb(0xFFF9F6))
                                .child(div().max_w(px(1120.)).mx_auto().child(panel)),
                        ),
                ),
            )
    }
}

fn action_button(
    label: impl Into<SharedString>,
    color: Hsla,
    text_color: Hsla,
    border_color: Hsla,
) -> Div {
    div()
        .cursor_pointer()
        .px_4()
        .py_3()
        .border_1()
        .border_color(border_color)
        .bg(color)
        .text_color(text_color)
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label.into())
}

fn validate_key_iv(key_text: &str, iv_text: &str) -> Option<String> {
    if key_text.len() != KEY_SIZE {
        return Some(format!("Key must be exactly {} characters.", KEY_SIZE));
    }

    if iv_text.len() != BLOCK_SIZE {
        return Some(format!("IV must be exactly {} characters.", BLOCK_SIZE));
    }

    None
}

fn save_output_with_dialog(prefix: &str, content: &str) -> std::io::Result<()> {
    if content.is_empty() {
        return Ok(());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let default_name = format!("{prefix}_{timestamp}.txt");
    let start_dir = std::env::current_dir().ok();

    let mut dialog = FileDialog::new().add_filter("Text file", &["txt"]);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }

    if let Some(path) = dialog.set_file_name(&default_name).save_file() {
        std::fs::write(path, content)?;
    }

    Ok(())
}

fn import_text_with_dialog() -> std::io::Result<Option<String>> {
    let start_dir = std::env::current_dir().ok();

    let mut dialog = FileDialog::new()
        .add_filter("Text and Markdown", &["txt", "md"])
        .add_filter("Text", &["txt"])
        .add_filter("Markdown", &["md"]);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }

    let Some(path) = dialog.pick_file() else {
        return Ok(None);
    };

    Ok(Some(std::fs::read_to_string(path)?))
}

fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(10 + (c - b'a')),
        b'A'..=b'F' => Some(10 + (c - b'A')),
        _ => None,
    }
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let bytes = hex.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }

    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn hex_string(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out.push_str(&format!("{byte:02X}"));
    }
    out
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
