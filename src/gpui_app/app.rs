use std::{
    io::ErrorKind,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use gpui::{
    ClickEvent, ClipboardItem, Context, Div, Entity, Hsla, SharedString, Window, div, prelude::*,
    px, rgb,
};

use crate::{BLOCK_SIZE, CipherMode, KEY_SIZE, decrypt_message, derive_bytes, encrypt_message};

use super::{
    CONTROL_HEIGHT, MODE_OPTIONS, ModeOption, ScrollTextView, TextInput, TextInputValidator, UiTab,
    io::{
        hex_string, hex_to_bytes, import_text_with_dialog, random_complex_string,
        save_output_with_dialog, validate_key_iv,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CipherEngine {
    Rust,
    C,
}

impl CipherEngine {
    fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::C => "C",
        }
    }
}

pub(super) struct BlockCipherApp {
    active_tab: UiTab,
    selected_engine: CipherEngine,
    selected_mode: CipherMode,
    encrypt_key: Entity<TextInput>,
    encrypt_iv: Entity<TextInput>,
    encrypt_plaintext: Entity<TextInput>,
    decrypt_key: Entity<TextInput>,
    decrypt_iv: Entity<TextInput>,
    decrypt_ciphertext: Entity<TextInput>,
    encrypt_result_view: Entity<ScrollTextView>,
    decrypt_result_view: Entity<ScrollTextView>,
    encrypt_result: String,
    decrypt_result: String,
    encrypt_copy_done_until: Option<Instant>,
    decrypt_copy_done_until: Option<Instant>,
    encrypt_key_was_generated: bool,
    encrypt_iv_was_generated: bool,
    decrypt_key_was_generated: bool,
    decrypt_iv_was_generated: bool,
    backend_message: Option<String>,
}

impl BlockCipherApp {
    pub(super) fn new(window: &Window, cx: &mut Context<Self>) -> Self {
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
        let encrypt_plaintext =
            TextInput::new_multiline(window, cx, "Plaintext", TextInputValidator::None);
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
        let encrypt_result_view = ScrollTextView::new(cx);
        let decrypt_result_view = ScrollTextView::new(cx);

        encrypt_key.update(cx, |input, cx| input.set_value("", cx));
        encrypt_iv.update(cx, |input, cx| input.set_value("", cx));
        encrypt_plaintext.update(cx, |input, cx| input.set_value("hello", cx));
        decrypt_key.update(cx, |input, cx| input.set_value("", cx));
        decrypt_iv.update(cx, |input, cx| input.set_value("", cx));

        Self {
            active_tab: UiTab::Encrypt,
            selected_engine: CipherEngine::Rust,
            selected_mode: CipherMode::Cbc,
            encrypt_key,
            encrypt_iv,
            encrypt_plaintext,
            decrypt_key,
            decrypt_iv,
            decrypt_ciphertext,
            encrypt_result_view,
            decrypt_result_view,
            encrypt_result: String::new(),
            decrypt_result: String::new(),
            encrypt_copy_done_until: None,
            decrypt_copy_done_until: None,
            encrypt_key_was_generated: false,
            encrypt_iv_was_generated: false,
            decrypt_key_was_generated: false,
            decrypt_iv_was_generated: false,
            backend_message: None,
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

    fn set_engine(&mut self, engine: CipherEngine, cx: &mut Context<Self>) {
        self.selected_engine = engine;
        self.backend_message = None;
        cx.notify();
    }

    fn clear_encrypt_result_state(&mut self, cx: &mut Context<Self>) {
        self.encrypt_result.clear();
        self.encrypt_result_view
            .update(cx, |view, cx| view.set_content("", cx));
        self.encrypt_copy_done_until = None;
    }

    fn clear_decrypt_result_state(&mut self, cx: &mut Context<Self>) {
        self.decrypt_result.clear();
        self.decrypt_result_view
            .update(cx, |view, cx| view.set_content("", cx));
        self.decrypt_copy_done_until = None;
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
            self.clear_encrypt_result_state(cx);
            cx.notify();
            return;
        }

        self.encrypt_result = match self.selected_engine {
            CipherEngine::Rust => {
                let key = derive_bytes::<KEY_SIZE>(&key_text);
                let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
                let encrypted =
                    encrypt_message(self.selected_mode, plaintext.as_bytes(), &key, &iv);
                hex_string(&encrypted)
            }
            CipherEngine::C => {
                match run_c_backend("enc", self.selected_mode, &key_text, &iv_text, &plaintext) {
                    Ok(ciphertext) => ciphertext,
                    Err(message) => {
                        self.backend_message = Some(message);
                        self.clear_encrypt_result_state(cx);
                        cx.notify();
                        return;
                    }
                }
            }
        };

        self.backend_message = None;
        self.encrypt_result_view.update(cx, |view, cx| {
            view.set_content(self.encrypt_result.clone(), cx)
        });
        self.encrypt_copy_done_until = None;
        self.decrypt_ciphertext.update(cx, |input, cx| {
            input.set_value(self.encrypt_result.clone(), cx)
        });
        self.decrypt_key
            .update(cx, |input, cx| input.set_value(key_text, cx));
        self.decrypt_iv
            .update(cx, |input, cx| input.set_value(iv_text, cx));
        self.decrypt_key_was_generated = self.encrypt_key_was_generated;
        self.decrypt_iv_was_generated = self.encrypt_iv_was_generated;
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
            self.clear_decrypt_result_state(cx);
            cx.notify();
            return;
        }

        self.decrypt_result = match self.selected_engine {
            CipherEngine::Rust => {
                let Some(ciphertext) = hex_to_bytes(&ciphertext_text) else {
                    self.clear_decrypt_result_state(cx);
                    cx.notify();
                    return;
                };

                let key = derive_bytes::<KEY_SIZE>(&key_text);
                let iv = derive_bytes::<BLOCK_SIZE>(&iv_text);
                let Some(plaintext) = decrypt_message(self.selected_mode, &ciphertext, &key, &iv)
                else {
                    self.clear_decrypt_result_state(cx);
                    cx.notify();
                    return;
                };

                String::from_utf8_lossy(&plaintext).into_owned()
            }
            CipherEngine::C => {
                match run_c_backend(
                    "dec",
                    self.selected_mode,
                    &key_text,
                    &iv_text,
                    &ciphertext_text,
                ) {
                    Ok(plaintext) => plaintext,
                    Err(message) => {
                        self.backend_message = Some(message);
                        self.clear_decrypt_result_state(cx);
                        cx.notify();
                        return;
                    }
                }
            }
        };

        self.backend_message = None;
        self.decrypt_result_view.update(cx, |view, cx| {
            view.set_content(self.decrypt_result.clone(), cx)
        });
        self.decrypt_copy_done_until = None;
        cx.notify();
    }

    fn clear_encrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.encrypt_plaintext
            .update(cx, |input, cx| input.clear(cx));
        self.clear_encrypt_result_state(cx);
        cx.notify();
    }

    fn clear_decrypt(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.decrypt_ciphertext
            .update(cx, |input, cx| input.clear(cx));
        self.clear_decrypt_result_state(cx);
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
            self.encrypt_plaintext
                .update(cx, |input, cx| input.set_value_from_top(content, cx));
        }
        cx.notify();
    }

    fn generate_encrypt_key(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let generated = random_complex_string(KEY_SIZE);
        self.encrypt_key
            .update(cx, |input, cx| input.set_value(generated, cx));
        self.encrypt_key_was_generated = true;
        cx.notify();
    }

    fn generate_encrypt_iv(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let generated = random_complex_string(BLOCK_SIZE);
        self.encrypt_iv
            .update(cx, |input, cx| input.set_value(generated, cx));
        self.encrypt_iv_was_generated = true;
        cx.notify();
    }

    fn generate_decrypt_key(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let generated = random_complex_string(KEY_SIZE);
        self.decrypt_key
            .update(cx, |input, cx| input.set_value(generated, cx));
        self.decrypt_key_was_generated = true;
        cx.notify();
    }

    fn generate_decrypt_iv(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let generated = random_complex_string(BLOCK_SIZE);
        self.decrypt_iv
            .update(cx, |input, cx| input.set_value(generated, cx));
        self.decrypt_iv_was_generated = true;
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
            .flex()
            .items_center()
            .justify_center()
            .h(px(CONTROL_HEIGHT))
            .px_4()
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
            .h(px(CONTROL_HEIGHT))
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

    fn render_engine_chip(&self, engine: CipherEngine, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.selected_engine == engine;
        let bg: Hsla = if active {
            rgb(0xD7E1D2).into()
        } else {
            rgb(0xFFFDF9).into()
        };
        let fg: Hsla = if active {
            rgb(0x45524A).into()
        } else {
            rgb(0x6E7E72).into()
        };
        let border: Hsla = rgb(0xB8C7B9).into();
        let id = match engine {
            CipherEngine::Rust => "engine-rust",
            CipherEngine::C => "engine-c",
        };

        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(120.))
            .h(px(CONTROL_HEIGHT))
            .border_1()
            .border_color(border)
            .bg(bg)
            .text_color(fg)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(engine.label())
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| view.set_engine(engine, cx)))
    }

    fn render_result_card(
        &self,
        title: &str,
        body: &str,
        scroll_view: Entity<ScrollTextView>,
    ) -> impl IntoElement {
        let should_scroll = body.lines().count() > 4 || body.len() > 160;
        let body_element = if should_scroll {
            div().pt_1().w_full().child(scroll_view).into_any_element()
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
        let encrypt_key_button_label = if self.encrypt_key_was_generated {
            "Regenerate"
        } else {
            "Generate"
        };
        let encrypt_iv_button_label = if self.encrypt_iv_was_generated {
            "Regenerate"
        } else {
            "Generate"
        };
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
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_3()
                    .w_full()
                    .child(div().flex_1().child(self.encrypt_key.clone()))
                    .child(
                        action_button(
                            encrypt_key_button_label,
                            rgb(0xFFFDF9).into(),
                            rgb(0x7B7287).into(),
                            rgb(0xD1C2B7).into(),
                        )
                        .id("encrypt-generate-key")
                        .on_click(cx.listener(Self::generate_encrypt_key)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_3()
                    .w_full()
                    .child(div().flex_1().child(self.encrypt_iv.clone()))
                    .child(
                        action_button(
                            encrypt_iv_button_label,
                            rgb(0xFFFDF9).into(),
                            rgb(0x7B7287).into(),
                            rgb(0xD1C2B7).into(),
                        )
                        .id("encrypt-generate-iv")
                        .on_click(cx.listener(Self::generate_encrypt_iv)),
                    ),
            )
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
            .child(self.render_result_card(
                "Ciphertext",
                &self.encrypt_result,
                self.encrypt_result_view.clone(),
            ))
    }

    fn render_decrypt_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let decrypt_key_button_label = if self.decrypt_key_was_generated {
            "Regenerate"
        } else {
            "Generate"
        };
        let decrypt_iv_button_label = if self.decrypt_iv_was_generated {
            "Regenerate"
        } else {
            "Generate"
        };
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
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_3()
                    .w_full()
                    .child(div().flex_1().child(self.decrypt_key.clone()))
                    .child(
                        action_button(
                            decrypt_key_button_label,
                            rgb(0xFFFDF9).into(),
                            rgb(0x7B7287).into(),
                            rgb(0xD1C2B7).into(),
                        )
                        .id("decrypt-generate-key")
                        .on_click(cx.listener(Self::generate_decrypt_key)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap_3()
                    .w_full()
                    .child(div().flex_1().child(self.decrypt_iv.clone()))
                    .child(
                        action_button(
                            decrypt_iv_button_label,
                            rgb(0xFFFDF9).into(),
                            rgb(0x7B7287).into(),
                            rgb(0xD1C2B7).into(),
                        )
                        .id("decrypt-generate-iv")
                        .on_click(cx.listener(Self::generate_decrypt_iv)),
                    ),
            )
            .child(self.decrypt_ciphertext.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .children(action_children),
            )
            .child(self.render_result_card(
                "Plaintext",
                &self.decrypt_result,
                self.decrypt_result_view.clone(),
            ))
    }
}

impl Render for BlockCipherApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = match self.active_tab {
            UiTab::Encrypt => self.render_encrypt_panel(cx).into_any_element(),
            UiTab::Decrypt => self.render_decrypt_panel(cx).into_any_element(),
        };
        let mut header_controls = div().flex().flex_col().gap_3().child(
            div()
                .flex()
                .w_full()
                .items_start()
                .justify_between()
                .gap_8()
                .flex_wrap()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(0x8B8293))
                                .child("Mode"),
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
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_end()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(0x6F7B73))
                                .child("Implementation"),
                        )
                        .child(
                            div().flex().gap_3().children([
                                self.render_engine_chip(CipherEngine::Rust, cx)
                                    .into_any_element(),
                                self.render_engine_chip(CipherEngine::C, cx)
                                    .into_any_element(),
                            ]),
                        ),
                ),
        );

        if let Some(message) = self.backend_message.clone() {
            header_controls =
                header_controls.child(div().text_sm().text_color(rgb(0xB05F63)).child(message));
        }

        div()
            .size_full()
            .bg(rgb(0xFFF9F6))
            .text_color(rgb(0x656072))
            .child(
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
                                                                    .font_weight(
                                                                        gpui::FontWeight::BOLD,
                                                                    )
                                                                    .text_color(rgb(0x5E586C))
                                                                    .child("Block Cipher"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(rgb(0x8B8293))
                                                                    .child(
                                                                        "64-bit block | 128-bit key | 8 rounds",
                                                                    ),
                                                            ),
                                                    ),
                                            )
                                            .child(
                                                header_controls,
                                            )
                                            .child(div().flex().gap_3().flex_wrap().children(
                                                MODE_OPTIONS.iter().map(|option| {
                                                    self.render_mode_chip(option, cx).into_any_element()
                                                }),
                                            )),
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
        .flex()
        .items_center()
        .justify_center()
        .h(px(CONTROL_HEIGHT))
        .px_4()
        .border_1()
        .border_color(border_color)
        .bg(color)
        .text_color(text_color)
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label.into())
}

fn mode_slug(mode: CipherMode) -> &'static str {
    match mode {
        CipherMode::Cbc => "cbc",
        CipherMode::Cfb => "cfb",
        CipherMode::Ofb => "ofb",
    }
}

fn c_backend_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("block_cipher"));
        candidates.push(current_dir.join("c").join("block_cipher"));
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        candidates.push(exe_dir.join("block_cipher"));

        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join("block_cipher"));

            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.join("block_cipher"));
                candidates.push(grandparent.join("c").join("block_cipher"));
            }
        }
    }

    candidates
}

fn run_c_backend(
    operation: &str,
    mode: CipherMode,
    key: &str,
    iv: &str,
    payload: &str,
) -> Result<String, String> {
    let binary = c_backend_candidates()
        .into_iter()
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from("block_cipher"));
    let output = Command::new(&binary)
        .args([operation, mode_slug(mode), key, iv, payload])
        .output()
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => {
                "Backend C tidak ditemukan. Jalankan `make` agar binary `block_cipher` tersedia."
                    .to_string()
            }
            _ => format!("Gagal menjalankan backend C: {error}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            return Err(format!("Backend C keluar dengan status {}.", output.status));
        }
        return Err(stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches(['\r', '\n'])
        .to_string())
}
