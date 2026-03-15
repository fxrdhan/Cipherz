use std::ops::Range;

use gpui::{
    actions, App, Application, Bounds, ClipboardItem, ClickEvent, Context, CursorStyle,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable,
    GlobalElementId, Hsla, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, Pixels, Point, ShapedLine, SharedString, StatefulInteractiveElement,
    Style, TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions, Div,
    div, fill, point, prelude::*, px, relative, rgb, rgba, size, white,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    decrypt_message, derive_bytes, encrypt_message, CipherMode, BLOCK_SIZE, KEY_SIZE,
};

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
    Demo,
}

struct ModeOption {
    mode: CipherMode,
    title: &'static str,
    detail: &'static str,
}

const MODE_OPTIONS: [ModeOption; 4] = [
    ModeOption {
        mode: CipherMode::Cbc,
        title: "CBC",
        detail: "PKCS#7 padding for block-sized output",
    },
    ModeOption {
        mode: CipherMode::Cfb,
        title: "CFB",
        detail: "Feedback stream mode, no padding",
    },
    ModeOption {
        mode: CipherMode::Ofb,
        title: "OFB",
        detail: "Output feedback stream mode",
    },
    ModeOption {
        mode: CipherMode::Ctr,
        title: "CTR",
        detail: "Counter mode with incrementing nonce",
    },
];

const DEMO_PLAINTEXT: &str = "Tugas block cipher tanpa library kriptografi";
const DEMO_KEY: &str = "KAMSIS-KEY-2026!";
const DEMO_IV: &str = "IV2026!!";

#[derive(Clone)]
struct TextInputStyle {
    background: Hsla,
    border: Hsla,
    border_focus: Hsla,
    text: Hsla,
    placeholder: Hsla,
    selection: Hsla,
    cursor: Hsla,
}

impl TextInputStyle {
    fn dark() -> Self {
        Self {
            background: rgb(0xFBF7F0).into(),
            border: rgb(0xD8CDBA).into(),
            border_focus: rgb(0x0F766E).into(),
            text: rgb(0x1E293B).into(),
            placeholder: rgba(0x1E293B66).into(),
            selection: rgba(0x0F766E33).into(),
            cursor: rgb(0x0F766E).into(),
        }
    }
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
}

impl TextInput {
    fn new(window: &Window, cx: &mut App, placeholder: &str) -> Entity<Self> {
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
            style: TextInputStyle::dark(),
        })
    }

    fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.content = value.into();
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

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref()) else {
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

        if focus_handle.is_focused(window) && let Some(cursor) = prepaint.cursor.take() {
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
        let border_color = if self.focus_handle(cx).is_focused(_window) {
            self.style.border_focus
        } else {
            self.style.border
        };

        div()
            .id("text-input-shell")
            .flex()
            .key_context("BlockCipherTextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
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
            .border_1()
            .border_color(border_color)
            .rounded_lg()
            .bg(self.style.background)
            .line_height(px(22.))
            .text_size(px(16.))
            .p_3()
            .child(TextInputElement { input: cx.entity() })
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
    status: String,
    demo_rows: Vec<(CipherMode, String, String)>,
}

impl BlockCipherApp {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let encrypt_key = TextInput::new(window, cx, "16-char key, contoh: KAMSIS-KEY-2026!");
        let encrypt_iv = TextInput::new(window, cx, "8-char IV / nonce, contoh: IV2026!!");
        let encrypt_plaintext = TextInput::new(window, cx, "Tulis plaintext di sini");
        let decrypt_key = TextInput::new(window, cx, "16-char key for decryption");
        let decrypt_iv = TextInput::new(window, cx, "8-char IV / nonce");
        let decrypt_ciphertext = TextInput::new(window, cx, "Ciphertext hex hasil enkripsi");

        encrypt_key.update(cx, |input, cx| input.set_value(DEMO_KEY, cx));
        encrypt_iv.update(cx, |input, cx| input.set_value(DEMO_IV, cx));
        encrypt_plaintext.update(cx, |input, cx| input.set_value("halo dunia", cx));
        decrypt_key.update(cx, |input, cx| input.set_value(DEMO_KEY, cx));
        decrypt_iv.update(cx, |input, cx| input.set_value(DEMO_IV, cx));

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
            status: "Siap. Pilih mode lalu enkripsi atau dekripsi.".to_string(),
            demo_rows: build_demo_rows(),
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

        let key = derive_bytes::<KEY_SIZE>(&key_text);
        let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
        let encrypted = encrypt_message(self.selected_mode, plaintext.as_bytes(), &key, &iv);

        self.encrypt_result = hex_string(&encrypted);
        self.decrypt_ciphertext
            .update(cx, |input, cx| input.set_value(self.encrypt_result.clone(), cx));
        self.decrypt_key
            .update(cx, |input, cx| input.set_value(key_text, cx));
        self.decrypt_iv
            .update(cx, |input, cx| input.set_value(iv_text, cx));
        self.status = format!(
            "Enkripsi {} berhasil. Ciphertext siap dipakai di tab decrypt.",
            self.selected_mode.name()
        );
        cx.notify();
    }

    fn decrypt_now(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key_text = self.decrypt_key.read(cx).value();
        let iv_text = self.decrypt_iv.read(cx).value();
        let ciphertext_text = self.decrypt_ciphertext.read(cx).value();

        let Some(ciphertext) = hex_to_bytes(&ciphertext_text) else {
            self.decrypt_result.clear();
            self.status = "Ciphertext hex tidak valid.".to_string();
            cx.notify();
            return;
        };

        let key = derive_bytes::<KEY_SIZE>(&key_text);
        let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
        let Some(plaintext) = decrypt_message(self.selected_mode, &ciphertext, &key, &iv) else {
            self.decrypt_result.clear();
            self.status = "Dekripsi gagal. Periksa mode, key, IV, atau padding CBC.".to_string();
            cx.notify();
            return;
        };

        self.decrypt_result = String::from_utf8_lossy(&plaintext).into_owned();
        self.status = format!(
            "Dekripsi {} berhasil. Plaintext dipulihkan.",
            self.selected_mode.name()
        );
        cx.notify();
    }

    fn clear_encrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.encrypt_plaintext.update(cx, |input, cx| input.clear(cx));
        self.encrypt_result.clear();
        self.status = "Form encrypt dibersihkan.".to_string();
        cx.notify();
    }

    fn clear_decrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.decrypt_ciphertext.update(cx, |input, cx| input.clear(cx));
        self.decrypt_result.clear();
        self.status = "Form decrypt dibersihkan.".to_string();
        cx.notify();
    }

    fn load_demo_seed(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.encrypt_key
            .update(cx, |input, cx| input.set_value(DEMO_KEY, cx));
        self.encrypt_iv
            .update(cx, |input, cx| input.set_value(DEMO_IV, cx));
        self.encrypt_plaintext
            .update(cx, |input, cx| input.set_value(DEMO_PLAINTEXT, cx));
        self.decrypt_key
            .update(cx, |input, cx| input.set_value(DEMO_KEY, cx));
        self.decrypt_iv
            .update(cx, |input, cx| input.set_value(DEMO_IV, cx));
        self.status = "Demo seed dimuat ke semua field.".to_string();
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
        let bg: Hsla = if active { rgb(0x0F766E).into() } else { rgba(0xFFFFFF11).into() };
        let fg: Hsla = if active { white() } else { rgba(0xF8FAFCDD).into() };

        div()
            .id(id)
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_full()
            .bg(bg)
            .text_color(fg)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(label)
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| view.set_tab(tab, cx)))
    }

    fn render_mode_chip(
        &self,
        option: &ModeOption,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mode = option.mode;
        let active = self.selected_mode == mode;
        let bg: Hsla = if active { rgb(0xF59E0B).into() } else { rgba(0xF8FAFC14).into() };
        let fg: Hsla = if active { rgb(0x1F2937).into() } else { white() };
        let border: Hsla = if active { rgb(0xFCD34D).into() } else { rgba(0xF8FAFC24).into() };
        let id = match mode {
            CipherMode::Cbc => "mode-cbc",
            CipherMode::Cfb => "mode-cfb",
            CipherMode::Ofb => "mode-ofb",
            CipherMode::Ctr => "mode-ctr",
        };

        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .flex_col()
            .gap_1()
            .w(px(180.))
            .p_3()
            .rounded_xl()
            .border_1()
            .border_color(border)
            .bg(bg)
            .text_color(fg)
            .child(div().font_weight(gpui::FontWeight::BOLD).child(option.title))
            .child(div().text_sm().opacity(0.85).child(option.detail))
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| view.set_mode(mode, cx)))
    }

    fn render_result_card(&self, title: &str, body: &str, accent: Hsla) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(rgba(0xFFFFFF16))
            .bg(rgba(0xFFFFFF0A))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size_3().rounded_full().bg(accent))
                    .child(div().font_weight(gpui::FontWeight::BOLD).child(title.to_string())),
            )
            .child(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(0xF8FAFC))
                    .text_color(rgb(0x0F172A))
                    .text_sm()
                    .min_h(px(88.))
                    .child(if body.is_empty() {
                        "Belum ada hasil".to_string()
                    } else {
                        body.to_string()
                    }),
            )
    }

    fn render_encrypt_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(self.encrypt_key.clone())
            .child(self.encrypt_iv.clone())
            .child(self.encrypt_plaintext.clone())
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        action_button("Encrypt Now", rgb(0x0F766E).into())
                            .id("encrypt-now")
                            .on_click(cx.listener(Self::encrypt_now)),
                    )
                    .child(
                        action_button("Clear", rgb(0x475569).into())
                            .id("encrypt-clear")
                            .on_click(cx.listener(Self::clear_encrypt)),
                    ),
            )
            .child(self.render_result_card(
                "Ciphertext Hex",
                &self.encrypt_result,
                rgb(0xF59E0B).into(),
            ))
    }

    fn render_decrypt_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .gap_3()
                    .child(
                        action_button("Decrypt Now", rgb(0xB45309).into())
                            .id("decrypt-now")
                            .on_click(cx.listener(Self::decrypt_now)),
                    )
                    .child(
                        action_button("Clear", rgb(0x475569).into())
                            .id("decrypt-clear")
                            .on_click(cx.listener(Self::clear_decrypt)),
                    ),
            )
            .child(self.render_result_card(
                "Recovered Plaintext",
                &self.decrypt_result,
                rgb(0x0EA5E9).into(),
            ))
    }

    fn render_demo_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_4()
                    .rounded_xl()
                    .bg(rgba(0xFFFFFF0B))
                    .border_1()
                    .border_color(rgba(0xFFFFFF14))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().font_weight(gpui::FontWeight::BOLD).child("Demo data"))
                            .child(div().text_sm().opacity(0.8).child(DEMO_PLAINTEXT)),
                    )
                    .child(
                        action_button("Load Demo Seed", rgb(0x0F766E).into())
                            .id("load-demo-seed")
                            .on_click(cx.listener(Self::load_demo_seed)),
                    ),
            )
            .children(self.demo_rows.iter().map(|(mode, cipher, plain)| {
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_xl()
                    .bg(rgba(0xFFFFFF08))
                    .border_1()
                    .border_color(rgba(0xFFFFFF12))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(div().font_weight(gpui::FontWeight::BOLD).child(mode.name()))
                            .child(div().text_sm().opacity(0.75).child("Verified against C version")),
                    )
                    .child(result_line("Ciphertext", cipher))
                    .child(result_line("Decrypt", plain))
            }))
    }
}

impl Render for BlockCipherApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = match self.active_tab {
            UiTab::Encrypt => self.render_encrypt_panel(cx).into_any_element(),
            UiTab::Decrypt => self.render_decrypt_panel(cx).into_any_element(),
            UiTab::Demo => self.render_demo_panel(cx).into_any_element(),
        };

        div()
            .size_full()
            .bg(rgb(0x08121B))
            .text_color(white())
            .child(
                div()
                    .id("app-scroll")
                    .size_full()
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
                                    .bg(rgb(0x0F172A))
                                    .child(
                                        div()
                                            .max_w(px(1120.))
                                            .mx_auto()
                                            .flex()
                                            .flex_col()
                                            .gap_5()
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
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(rgb(0xFBBF24))
                                                                    .child("Native GPUI Desktop"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(42.))
                                                                    .font_weight(gpui::FontWeight::BLACK)
                                                                    .child("Block Cipher Workbench"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .max_w(px(620.))
                                                                    .text_lg()
                                                                    .opacity(0.85)
                                                                    .child(
                                                                        "UI native untuk eksplorasi mode CBC, CFB, OFB, dan CTR pada implementasi Rust yang ekuivalen dengan versi C.",
                                                                    ),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_2()
                                                            .p_4()
                                                            .rounded_xl()
                                                            .bg(rgba(0xFFFFFF0B))
                                                            .border_1()
                                                            .border_color(rgba(0xFFFFFF16))
                                                            .child(div().text_sm().opacity(0.7).child("Block size"))
                                                            .child(div().text_2xl().font_weight(gpui::FontWeight::BOLD).child("64-bit"))
                                                            .child(div().text_sm().opacity(0.7).child("Key size 128-bit | 8 rounds")),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_3()
                                                    .children([
                                                        self.render_tab_button("tab-encrypt", "Encrypt", UiTab::Encrypt, cx).into_any_element(),
                                                        self.render_tab_button("tab-decrypt", "Decrypt", UiTab::Decrypt, cx).into_any_element(),
                                                        self.render_tab_button("tab-demo", "Demo", UiTab::Demo, cx).into_any_element(),
                                                    ]),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_3()
                                                    .flex_wrap()
                                                    .children(MODE_OPTIONS.iter().map(|option| {
                                                        self.render_mode_chip(option, cx).into_any_element()
                                                    })),
                                            )
                                            .child(
                                                div()
                                                    .p_4()
                                                    .rounded_xl()
                                                    .bg(rgba(0x0F766E33))
                                                    .border_1()
                                                    .border_color(rgba(0x2DD4BF55))
                                                    .text_sm()
                                                    .child(self.status.clone()),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .p_6()
                                    .child(
                                        div()
                                            .max_w(px(1120.))
                                            .mx_auto()
                                            .child(panel),
                                    ),
                            ),
                    ),
            )
    }
}

fn action_button(label: &'static str, color: Hsla) -> Div {
    div()
        .cursor_pointer()
        .px_4()
        .py_3()
        .rounded_lg()
        .bg(color)
        .text_color(white())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label)
}

fn result_line(label: &'static str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().opacity(0.7).child(label))
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(rgb(0xF8FAFC))
                .text_color(rgb(0x0F172A))
                .text_sm()
                .child(value.to_string()),
        )
}

fn build_demo_rows() -> Vec<(CipherMode, String, String)> {
    let key = derive_bytes::<KEY_SIZE>(DEMO_KEY);
    let iv = derive_bytes::<BLOCK_SIZE>(DEMO_IV);
    MODE_OPTIONS
        .iter()
        .map(|option| {
            let encrypted = encrypt_message(option.mode, DEMO_PLAINTEXT.as_bytes(), &key, &iv);
            let decrypted = decrypt_message(option.mode, &encrypted, &key, &iv)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_else(|| "Gagal dekripsi".to_string());
            (option.mode, hex_string(&encrypted), decrypted)
        })
        .collect()
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
                ..Default::default()
            },
            |window, cx| cx.new(|cx| BlockCipherApp::new(window, cx)),
        )
        .expect("open gpui window");

        cx.activate(true);
    });
}
