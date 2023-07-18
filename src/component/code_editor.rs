use egui::{epaint::ahash::HashMap, text_edit::CursorRange, Id, Pos2, TextBuffer};
use log::debug;
use std::ops::Range;
use weighted_trie::WeightedTrie;

use super::syntax_highlight::{highlight, CodeTheme};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TextEdit {
    pub language: String,
    #[serde(skip)]
    suggest: AutoSuggester,
    #[serde(skip)]
    pub sug_pos: Option<Pos2>,
    #[serde(skip)]
    pub sug_str: Option<Vec<String>>,
    #[serde(skip)]
    pub selected_range: Option<Range<usize>>,
    #[serde(skip)]
    pub selected_sug: String,
}
macro_rules! register_shotcut {
    ($text:expr,$ui:expr,$output:expr,$mod:expr,$key:expr,$call:expr) => {
        if $ui.input_mut(|i| i.consume_key($mod, $key)) {
            dbg!($output);
            debug!("{}_{}", stringify!($mod), stringify!($key));
            if let Some(text_cursor_range) = $output {
                let selected_chars = text_cursor_range.as_sorted_char_range();
                Self::do_selected_with($text, selected_chars, $call);
            }
        }
    };
}

macro_rules! insert_suggest {
    ($sug:ident,$s:expr) => {
        $sug.insert($s.to_owned(), Box::new(|_s| $s.to_owned()), 1);
    };
    ($sug:ident,$key:expr,$fun:expr) => {
        $sug.insert($key.to_owned(), Box::new($fun), 1);
    };
}

impl Default for TextEdit {
    fn default() -> Self {
        let mut sug = AutoSuggester::default();
        insert_suggest!(sug, "type_of", |_s| "type_of()".to_owned());
        insert_suggest!(sug, "true");
        insert_suggest!(sug, "false");
        insert_suggest!(sug, "let");
        insert_suggest!(sug, "const");
        insert_suggest!(sug, "curry");
        insert_suggest!(sug, "return");
        insert_suggest!(sug, "throw");

        insert_suggest!(sug, "faker::zh_name", |_s| "faker::zh_name()".to_owned());
        insert_suggest!(sug, "faker::en_name", |_s| "faker::en_name()".to_owned());
        insert_suggest!(sug, "faker::hex_str", |_s| "faker::hex_str(0,10)"
            .to_owned());
        insert_suggest!(sug, "faker::str", |_s| "faker::str(0,10)".to_owned());
        insert_suggest!(sug, "faker::num_str", |_s| "faker::num_str(0,100)"
            .to_owned());
        insert_suggest!(sug, "faker::num", |_s| "faker::num(0,100)".to_owned());

        insert_suggest!(sug, "faker::email", |_s| "faker::email()".to_owned());
        insert_suggest!(sug, "faker::username", |_s| "faker::username()".to_owned());
        insert_suggest!(sug, "faker::ip4", |_s| "faker::ip4()".to_owned());
        insert_suggest!(sug, "faker::ip6", |_s| "faker::ip6()".to_owned());
        insert_suggest!(sug, "faker::useragent", |_s| "faker::useragent()"
            .to_owned());
        insert_suggest!(sug, "faker::mac", |_s| "faker::mac()".to_owned());
        insert_suggest!(sug, "faker::password", |_s| "faker::password()".to_owned());
        insert_suggest!(sug, "faker::uuid", |_s| "faker::uuid()".to_owned());
        insert_suggest!(sug, "faker::uuid_simple", |_s| "faker::uuid_simple()"
            .to_owned());

        insert_suggest!(sug, "faker::now", |_s| "faker::now(\"%Y-%m-%dT%H:%M:%S\")"
            .to_owned());

        insert_suggest!(sug, "faker::datetime", |_s| {
            "faker::datetime(\"%Y-%m-%dT%H:%M:%S\")".to_owned()
        });
        insert_suggest!(sug, "faker::datetime_after", |_s| {
            "faker::datetime_after(\"%Y-%m-%dT%H:%M:%S\",\"2020-05-03T00:00:00\")".to_owned()
        });
        insert_suggest!(sug, "faker::datetime_before", |_s| {
            "faker::datetime_before(\"%Y-%m-%dT%H:%M:%S\",\"2020-05-03T00:00:00\")".to_owned()
        });
        insert_suggest!(sug, "faker::date_add", |_s| {
            "faker::date_add(\"%Y-%m-%dT%H:%M:%S\",\"2020-05-03T00:00:00\")".to_owned()
        });

        insert_suggest!(sug, "log::info", |_s| { r#"log::info("msg")"#.to_owned() });
        insert_suggest!(sug, "log::error", |_s| { "log::error(\"msg\")".to_owned() });
        insert_suggest!(sug, "log::debug", |_s| { "log::debug(\"msg\")".to_owned() });
        // insert_suggest!(sug, "log::info", |_s| { "log::info("msg")".to_owned() });
        insert_suggest!(sug, "log::warn", |_s| { "log::warn(\"msg\")".to_owned() });

        insert_suggest!(sug, "base64::encode", |_s| {
            "base64::encode(msg)".to_owned()
        });
        insert_suggest!(sug, "base64::decode", |_s| {
            "base64::decode(msg)".to_owned()
        });

        insert_suggest!(sug, "crypto::Aes::encode_cbc", |_s| {
            "crypto::Aes::encode_cbc(key,input,iv);".to_owned()
        });
        insert_suggest!(sug, "crypto::Aes::decode_cbc", |_s| {
            "crypto::Aes::decode_cbc(key,input,iv);".to_owned()
        });

        insert_suggest!(sug, "trycatch", |_s| {
            "try { \n } catch ( err) { \n log::error(err)\n}".to_owned()
        });
        insert_suggest!(sug, "ifelse", |_s| { "if {\\n} \\nelse {\n\n}".to_owned() });
        insert_suggest!(sug, "switch", |_s| { "switch EXPR {\\n}\\n".to_owned() });
        Self {
            language: "json".to_owned(),
            suggest: sug,
            sug_pos: None,
            selected_sug: "".to_owned(),
            sug_str: None,
            selected_range: None,
        }
    }
}

impl TextEdit {
    pub fn new(lang: &str) -> Self {
        Self {
            language: lang.to_owned(),
            ..Default::default()
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, text: &mut String, id: u64) {
        let Self {
            language,
            suggest,
            sug_pos,
            selected_sug,
            sug_str,
            selected_range,
        } = self;

        let theme = CodeTheme::from_memory(ui.ctx());

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let layout_job = highlight(ui.ctx(), &theme, string, language);
            // layout_job.wrap.max_width = wrap_width; // no wrapping
            ui.fonts(|f| f.layout_job(layout_job))
        };
        let editor_id = ui.id().with(id).with(language.as_str());
        let output = egui::TextEdit::multiline(text)
            .id(editor_id)
            // .desired_rows(10)
            .min_size(ui.available_size())
            .code_editor()
            // .desired_width(ui.available_width())
            .layouter(&mut layouter)
            .show(ui);

        // let anything_selected = output
        //     .cursor_range
        //     .map_or(false, |cursor| !cursor.is_empty());

        // ui.add_enabled(
        //     anything_selected,
        //     egui::Label::new("Press ctrl+Y to toggle the case of selected text (cmd+Y on Mac)"),
        // );
        if output.response.has_focus() {
            // Ctrl+alt+u 切换大小写
            register_shotcut!(
                text,
                ui,
                output.cursor_range,
                egui::Modifiers::ALT | egui::Modifiers::COMMAND,
                egui::Key::Y,
                toggle_case
            );
            //Ctrl+d 复制选择的内容或者当前行
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::D)) {
                debug!("ctrl+d,复制当前行");
                if let Some(text_cursor_range) = output.cursor_range {
                    let text_range = text_cursor_range.as_sorted_char_range();
                    if text_range.start == text_range.end {
                        let total = text.chars().count();
                        let pretext = text.char_range(0..text_range.start);
                        let pre_count = pretext.chars().count();
                        let start = pretext
                            .chars()
                            .rev()
                            .position(|c| c == '\n')
                            .unwrap_or(pre_count);
                        let start = pre_count - start;
                        let aftertext = text.char_range(text_range.end..total);
                        let end = aftertext.chars().position(|c| c == '\n').unwrap_or(total);
                        Self::do_selected_with(text, start..pre_count + end, |s| {
                            s.to_owned() + "\n" + s
                        });
                    } else {
                        Self::do_selected_with(text, text_range, |s| s.to_owned().repeat(2));
                    }
                }
            }
            //Alt + q
            register_shotcut!(
                text,
                ui,
                output.cursor_range,
                egui::Modifiers::ALT | egui::Modifiers::CTRL,
                egui::Key::Q,
                add_quoter
            );
            //Alt+c
            register_shotcut!(
                text,
                ui,
                output.cursor_range,
                egui::Modifiers::ALT | egui::Modifiers::CTRL,
                egui::Key::C,
                add_parentheses
            );
            //Alt+v
            register_shotcut!(
                text,
                ui,
                output.cursor_range,
                egui::Modifiers::ALT | egui::Modifiers::CTRL,
                egui::Key::V,
                add_brackets
            );
            //Alt + b
            register_shotcut!(
                text,
                ui,
                output.cursor_range,
                egui::Modifiers::ALT | egui::Modifiers::CTRL,
                egui::Key::B,
                add_braces
            );

            //新行 ctrl + n
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) {
                debug!("ctrl+n,新行");
                if let Some(text_cursor_range) = output.cursor_range {
                    let text_range = text_cursor_range.as_sorted_char_range();
                    let enter_pos = text
                        .chars()
                        .skip(text_range.end)
                        .position(|c| c == '\n')
                        .unwrap_or(0)
                        + text_range.end;
                    text.insert_text("\n", enter_pos);
                    let text_edit_id = output.response.id;
                    Self::set_cursor_index(ui, text_edit_id, enter_pos);
                }
            }

            let mut is_delete = false;
            if ui.input(|i| i.key_down(egui::Key::Delete)) {
                is_delete = true;
            }
            if ui.input(|i| i.key_down(egui::Key::Backspace)) {
                is_delete = true;
            }

            if !is_delete && output.response.changed() {
                debug!("编辑器操作");
                if let Some(text_cursor_range) = output.cursor_range {
                    let text_range = text_cursor_range.as_sorted_char_range();
                    //处理自动补全
                    if text_range.start == text_range.end {
                        let input_pos = text_range.start.checked_sub(1).unwrap_or(0);
                        if let Some(char) = text.chars().nth(input_pos) {
                            match char {
                                '"' => {
                                    text.insert_text("\"", text_range.start);
                                }
                                '\'' => {
                                    text.insert_text("'", text_range.start);
                                }
                                '{' => {
                                    text.insert_text("}", text_range.start);
                                }
                                '[' => {
                                    text.insert_text("]", text_range.start);
                                }
                                '(' => {
                                    text.insert_text(")", text_range.start);
                                }
                                '<' => {
                                    text.insert_text(">", text_range.start);
                                }
                                _ => {}
                            }
                        }
                    }

                    //处理建议弹框
                    let preword = Self::get_pre_word(text, text_cursor_range);

                    if let Some((word, word_range)) = preword {
                        *sug_str = Some(suggest.search(&word));
                        if sug_str.as_ref().unwrap().len() > 0 && sug_pos.is_none() {
                            if let Some(pos) = ui.input(|p| p.pointer.hover_pos()) {
                                *sug_pos = Some(pos);
                            }
                        }
                        if sug_str.as_ref().unwrap().len() == 0 {
                            *sug_pos = None;
                        }
                        *selected_range = Some(word_range);
                    }
                }
            }

            if let (Some(pos), Some(sugs), Some(text_range)) =
                (sug_pos.clone(), sug_str.clone(), selected_range.clone())
            {
                let _sug_ui = egui::Window::new("建议")
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(false)
                    .min_width(300.0)
                    .current_pos(pos)
                    .show(ui.ctx(), |ui| {
                        for sug in sugs {
                            if ui.button(sug.clone()).clicked() {
                                if let Some(action) = suggest.get_action(&sug) {
                                    let new_index =
                                        Self::do_selected_with(text, text_range.clone(), action);
                                    let text_edit_id = output.response.id;
                                    Self::set_cursor_index(ui, text_edit_id, new_index);
                                    *sug_pos = None
                                }
                            }
                        }
                    });
            }

            //Shift+Home 开头
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Home)) {
                debug!("shift_home,光标设置到开头");
                let text_edit_id = output.response.id;
                Self::set_cursor_index(ui, text_edit_id, 0)
            }

            //Shift+End 末尾
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::End)) {
                debug!("shift_end,光标设置到结尾");
                let text_edit_id = output.response.id;
                Self::set_cursor_index(ui, text_edit_id, text.chars().count())
            }
        }
    }

    fn set_cursor_index(ui: &mut egui::Ui, text_edit_id: Id, index: usize) {
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
            let ccursor = egui::text::CCursor::new(index);
            state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
            state.store(ui.ctx(), text_edit_id);
            ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id)); // give focus back to the [`TextEdit`].
        }
    }

    fn do_selected_with<F>(text: &mut String, selected_range: Range<usize>, conventer: F) -> usize
    where
        F: for<'a> Fn(&'a str) -> String,
    {
        let selected_str = text.char_range(selected_range.clone());
        let new_str = conventer(selected_str).to_owned();
        text.delete_char_range(selected_range.clone());
        text.insert_text(&new_str, selected_range.start);
        //返回新的index给后续设置光标位置
        selected_range.start + new_str.chars().count()
    }

    fn get_selected(text: &str, cursor: CursorRange) -> Option<String> {
        use egui::TextBuffer as _;
        let selected_chars = cursor.as_sorted_char_range();
        if selected_chars.start == selected_chars.end {
            None
        } else {
            let selected_text = text.char_range(selected_chars);
            Some(selected_text.to_owned())
        }
    }

    fn get_pre_word(text: &str, cursor: CursorRange) -> Option<(String, Range<usize>)> {
        use egui::TextBuffer as _;
        let selected_chars = cursor.as_sorted_char_range();
        let pretext = text.char_range(0..selected_chars.start);
        let start = pretext
            .chars()
            .rev()
            .position(|c| c.is_whitespace())
            .unwrap_or_default();
        let start = pretext.chars().count() - start;
        if start <= selected_chars.end {
            let selected_chars = start..selected_chars.end;
            let selected_text = text.char_range(selected_chars.clone());
            Some((selected_text.to_owned(), selected_chars))
        } else {
            None
        }
    }
}

struct AutoSuggester {
    pub trie: WeightedTrie,
    pub action: HashMap<String, Box<dyn Fn(&str) -> String>>,
}

impl Default for AutoSuggester {
    fn default() -> Self {
        Self {
            trie: WeightedTrie::new(),
            action: Default::default(),
        }
    }
}

impl AutoSuggester {
    pub fn insert(&mut self, key: String, aciton: Box<dyn Fn(&str) -> String>, weight: i32) {
        self.trie.insert(key.clone(), weight);
        self.action.insert(key, aciton);
    }

    pub fn search(&self, key: &str) -> Vec<String> {
        self.trie.search(key)
    }

    pub fn get_action(&self, key: &str) -> Option<&Box<dyn Fn(&str) -> String>> {
        self.action.get(key)
    }
}

fn toggle_case(s: &str) -> String {
    dbg!(s);
    let upper_case = s.to_uppercase();
    let new_text = if s == upper_case {
        s.to_lowercase()
    } else {
        upper_case
    };
    new_text
}

fn add_braces(s: &str) -> String {
    "{".to_owned() + s + "}"
}
fn add_parentheses(s: &str) -> String {
    "(".to_owned() + s + ")"
}
fn add_brackets(s: &str) -> String {
    "<".to_owned() + s + ">"
}

fn add_quoter(s: &str) -> String {
    "\"".to_owned() + s + "\""
}
