use std::sync::{Arc, Mutex};

use egui::{text::LayoutJob, Key, Modifiers, Pos2};
use crate::component::template_tools::{TemplateHint,TemplateHintInfo};

type EditCursorPair = (usize,usize);
lazy_static! {
    static ref TEMPLATE_HINT: Arc<Mutex<TemplateHint>> = {
        let mut tmh = TemplateHint::new();
        tmh.add(TemplateHintInfo::new("中文名".into(), "随机中文名字".into(),"{{NAME_ZH()}}".into()));
        tmh.add(TemplateHintInfo::new("英文名".into(), "随机英文名字".into(),"{{NAME_EN()}}".into()));
        tmh.add(TemplateHintInfo::new("用户名".into(), "随机用户名字".into(),"{{USERNAME()}}".into()));
        tmh.add(TemplateHintInfo::new("邮件地址".into(), "随机邮件地址".into(),"{{EMAIL()}}".into()));
        tmh.add(TemplateHintInfo::new("IPV4地址".into(), "随机IPV4地址".into(),"{{IPV4()}}".into()));
        tmh.add(TemplateHintInfo::new("IPV6地址".into(), "随机IPV6地址".into(),"{{IPV6()}}".into()));
        tmh.add(TemplateHintInfo::new("MAC地址".into(), "随机MAC地址".into(),"{{MAC()}}".into()));
        tmh.add(TemplateHintInfo::new("UserAgent".into(), "随机UserAgent地址".into(),"{{USERAGENT()}}".into()));
        tmh.add(TemplateHintInfo::new("PassWord".into(), "随机PassWord地址".into(),"{{PASSWORD()}}".into()));
        tmh.add(TemplateHintInfo::new("循环模板".into(), 
            "循环语法".into(),
            "{% for x in range(循环次数) %}\n需要循环的数据\n{% endfor %}".into()));
        tmh.add(TemplateHintInfo::new("条件模板  ".into(), 
            "条件语法".into(),
            "{% if 条件 %}\n 分支1 \n {% elif 条件 %} \n 分支2 \n {% else %} \n 默认分支\n {% endif %}".into()));
        tmh.add(TemplateHintInfo::new("数值".into(), "生成大于等于min,小于max的数值".into(),"{{NUM(min,max)}}".into()));
        tmh.add(TemplateHintInfo::new("数字串".into(), "生成长度大于等于min,小于max的数字字符串".into(),"{{NUM_STR(min,max)}}".into()));
        tmh.add(TemplateHintInfo::new("字符串".into(), "随机字符串,min表示最小长度，max表示最大长度(不包含)".into(),"{{STR(min,max)}}".into()));
        tmh.add(TemplateHintInfo::new("16进制字符串".into(), "随机16进制字符串,min表示最小长度，max表示最大长度(不包含)".into(),"{{HEX(min,max)}}".into()));
        tmh.add(TemplateHintInfo::new("UUID".into(), "随机生成UUID".into(),"{{UUID()}}".into()));
        tmh.add(TemplateHintInfo::new("UUID_SIMPLE".into(), "随机生成UUID(中间不包含-)".into(),"{{UUID_SIMPLE()}}".into()));

        tmh.add(TemplateHintInfo::new("NOW".into(), "生成当前时间,需要送日期格式化字符串".into(),r#"{{NOW("%Y-%m-%dT%H:%M:%S")}}"#.into()));
        tmh.add(TemplateHintInfo::new("DATE".into(), "随机日期时间,需要送日期格式化字符串".into(),r#"{{DATE("%Y-%m-%dT%H:%M:%S")}}"#.into()));
        tmh.add(TemplateHintInfo::new("DATE_BEFORE".into(), "随机生成指定日期前的时间,需要送日期格式化字符串".into(),r#"{{DATE_BEFORE('%Y-%m-%dT%H:%M:%S','2020-01-01T00:00:00')}}"#.into()));
        tmh.add(TemplateHintInfo::new("DATE_AFTER".into(), "随机生成指定日期后的时间,需要送日期格式化字符串".into(),r#"{{DATE_AFTER('%Y-%m-%dT%H:%M:%S','2020-01-01T00:00:00')}}"#.into()));
        tmh.add(TemplateHintInfo::new("DATE_ADD".into(), "日期增减操作".into(),r#"{{DATE_ADD('秒数','2020-01-01T00:00:00','%Y-%m-%dT%H:%M:%S')}}"#.into()));

        tmh.add(TemplateHintInfo::new("BASE64编码".into(), "BASE64编码".into(),r#"{{BASE64_EN(字符串)}}"#.into()));
        tmh.add(TemplateHintInfo::new("BASE64解码".into(), "BASE64解码".into(),r#"{{BASE64_DE(字符串)}}"#.into()));

        tmh.add(TemplateHintInfo::new("转换器模板".into(), "转换器语法模板".into(),"{% filter 过滤器 %}\n需要转换的内容\n{% endfilter %}".into()));

        tmh.add(TemplateHintInfo::new("AES加密ECB模式".into(), "AES加密ECB模式".into(),r#"{{AES_ECB_EN('字符串','密钥')}}"#.into()));
        tmh.add(TemplateHintInfo::new("AES加密CBC模式".into(), "AES加密CBC模式".into(),r#"{{AES_CBC_EN('字符串','密钥','IV')}}"#.into()));
        tmh.add(TemplateHintInfo::new("AES加密ECB模式".into(), "AES加密CTR模式".into(),r#"{{AES_CTR_EN('字符串','密钥','IV')}}"#.into()));
        Arc::new(Mutex::new(tmh))
    };
}

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str, language: &str) {
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };

    // ui.set_height(ui.available_height());
    ui.set_min_height(ui.available_height());
    ui.add(
    // ui.add_sized(
        // ui.available_size(),
        // egui::TextEdit::multiline(&mut code)
        egui::text_edit::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Monospace) // for cursor height
            // .code_editor()
            // .desired_rows(1)
            // .lock_focus(true)
            .layouter(&mut layouter),
    );
}

pub fn code_editor_ui(ui: &mut egui::Ui, code: &mut String, language: &str) {
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Max),|ui|{

    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .id_source("requset_ui_scroller_1")
        .show(ui, |ui| {
            let editor_rsp = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(code)
                    // egui::text_edit::TextEdit::multiline(&mut code)
                    // .desired_width(ui.available_width())
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    // .desired_rows(1)
                    // .lock_focus(true)
                    .layouter(&mut layouter),
            );

        let resp_id = editor_rsp.id;
        let mut edit_pos:Option<EditCursorPair> = None;
        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp_id) {
            if let Some(ccursor) = state.ccursor_range() {
                let end = ccursor.primary.index;
                let start = ccursor.secondary.index;
                edit_pos = Some((start,end));
            }
            // let ccursor = egui::text::CCursor::new(0);
            // state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
            // state.store(ui.ctx(), text_edit_id);
            // ui.ctx().memory().request_focus(text_edit_id); // give focus back to the [`TextEdit`].
        }
        // let avp= ui.available_size();
        // // ui.set_height(avp.y);
        // let edit_output = egui::TextEdit::multiline(code)
        //     .code_editor()
        //     .layouter(&mut layouter)
        //     .desired_width(avp.x)
        //     // .desired_rows(48)
        //     .show(ui);

        let hotkey_state_id = ui.id().with("hotkey_alt_q");
        let (mut hotkey, mut pos) = {
            let mut data = ui.data();
            data.get_temp::<(bool, Pos2)>(hotkey_state_id)
                .unwrap_or((false, Pos2::default()))
        };
        if {
            let mut input = ui.input_mut();
            input.consume_key(Modifiers::CTRL, Key::Q)
        } {
            if let Some(current_pos) = {
                ui.input().pointer.hover_pos()
            } {
                pos = current_pos;
            }
            hotkey = !hotkey;
        }
        if hotkey {
            let _hit = egui::Window::new("提示")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .min_width(300.0)
                .current_pos(pos)
                .show(ui.ctx(), |ui| {
                    if TEMPLATE_HINT.lock().unwrap().ui(ui, edit_pos, code) {
                        hotkey = false;
                    }
                });
        }
        {
            let mut data = ui.data();
            data.insert_temp(hotkey_state_id, (hotkey,pos));
        }
        });

    });
    // }
    // edit_output.response.on_hover_ui(|ui|{
    //     let cu = edit_output.galley.end();
    //     ui.label("Selected text: ");
    //     ui.monospace(cu.ccursor.);
    // });
    // if edit_output.response.changed() {
    //     if let Some(text_cursor_range) = edit_output.cursor_range {
    //         use egui::TextBuffer as _;
    //         let mut selected_chars = text_cursor_range.as_sorted_char_range();
    //         if selected_chars.len()==0 {
    //             selected_chars.start = selected_chars.end - 1;
    //             let index = selected_chars.end;
    //             let input_world =code.char_range(selected_chars);
    //             if input_world == "{" {
    //                 code.insert(index, '}');
    //             } else if input_world == "\"" {
    //                 code.insert(index, '\"');
    //             }

    //         }
    //     }
    // }

    // if let Some(ccursor) = edit_output.state.ccursor_range() {
    //     dbg!(ccursor);
    // }
}

// pub fn highlight_layouter<'a>(
//     ui: &'a mut egui::Ui,
//     language: &'a str,
// ) -> impl FnMut(&Ui, &str, f32) -> Arc<Galley> + 'a {
//     let theme = CodeTheme::from_memory(ui.ctx());

//     let layouter = move |ui: &egui::Ui, string: &str, _wrap_width: f32| {
//         let layout_job = highlight(ui.ctx(), &theme, string, language);
//         // layout_job.wrap.max_width = wrap_width; // no wrapping
//         ui.fonts().layout_job(layout_job)
//     };
//     layouter
// }

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code, language))
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntect"))]
#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize, enum_map::Enum)]
enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}

#[cfg(feature = "syntect")]
#[derive(Clone, Copy, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

#[cfg(feature = "syntect")]
impl SyntectTheme {
    fn all() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Base16EightiesDark,
            Self::Base16MochaDark,
            Self::Base16OceanDark,
            Self::Base16OceanLight,
            Self::InspiredGitHub,
            Self::SolarizedDark,
            Self::SolarizedLight,
        ]
        .iter()
        .copied()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "Base16 Eighties (dark)",
            Self::Base16MochaDark => "Base16 Mocha (dark)",
            Self::Base16OceanDark => "Base16 Ocean (dark)",
            Self::Base16OceanLight => "Base16 Ocean (light)",
            Self::InspiredGitHub => "InspiredGitHub (light)",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::Base16EightiesDark
            | Self::Base16MochaDark
            | Self::Base16OceanDark
            | Self::SolarizedDark => true,

            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
}

#[derive(Clone, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeTheme {
    dark_mode: bool,

    #[cfg(feature = "syntect")]
    syntect_theme: SyntectTheme,

    #[cfg(not(feature = "syntect"))]
    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data()
                .get_persisted(egui::Id::new("dark"))
                .unwrap_or_else(CodeTheme::dark)
        } else {
            ctx.data()
                .get_persisted(egui::Id::new("light"))
                .unwrap_or_else(CodeTheme::light)
        }
    }

    pub fn store_in_memory(self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.data().insert_persisted(egui::Id::new("dark"), self);
        } else {
            ctx.data().insert_persisted(egui::Id::new("light"), self);
        }
    }
}

#[cfg(feature = "syntect")]
impl CodeTheme {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    pub fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_buttons(ui);

        for theme in SyntectTheme::all() {
            if theme.is_dark() == self.dark_mode {
                ui.radio_value(&mut self.syntect_theme, theme, theme.name());
            }
        }
    }
}

#[cfg(not(feature = "syntect"))]
impl CodeTheme {
    pub fn dark() -> Self {
        let font_id = egui::FontId::monospace(12.0);
        use egui::{Color32, TextFormat};
        Self {
            dark_mode: true,
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::from_gray(120)),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 100, 100)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(87, 165, 171)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(109, 147, 226)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::LIGHT_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn light() -> Self {
        let font_id = egui::FontId::monospace(12.0);
        use egui::{Color32, TextFormat};
        Self {
            dark_mode: false,
            #[cfg(not(feature = "syntect"))]
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(235, 0, 0)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(153, 134, 255)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(37, 203, 105)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            let selected_id = egui::Id::null();
            let mut selected_tt: TokenType = *ui
                .data()
                .get_persisted_mut_or(selected_id, TokenType::Comment);

            ui.vertical(|ui| {
                ui.set_width(150.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.scope(|ui| {
                    for (tt, tt_name) in [
                        (TokenType::Comment, "// comment"),
                        (TokenType::Keyword, "keyword"),
                        (TokenType::Literal, "literal"),
                        (TokenType::StringLiteral, "\"string literal\""),
                        (TokenType::Punctuation, "punctuation ;"),
                        (TokenType::Whitespace, "whitespace"),
                    ] {
                        let format = &mut self.formats[tt];
                        ui.style_mut().override_font_id = Some(format.font_id.clone());
                        ui.visuals_mut().override_text_color = Some(format.color);
                        ui.radio_value(&mut selected_tt, tt, tt_name);
                    }
                });

                let reset_value = if self.dark_mode {
                    CodeTheme::dark()
                } else {
                    CodeTheme::light()
                };

                if ui
                    .add_enabled(*self != reset_value, egui::Button::new("Reset theme"))
                    .clicked()
                {
                    *self = reset_value;
                }
            });

            ui.add_space(16.0);

            ui.data().insert_persisted(selected_id, selected_tt);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Vec2::splat(2.0))
                .show(ui, |ui| {
                    // ui.group(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    ui.spacing_mut().slider_width = 128.0; // Controls color picker size
                    egui::widgets::color_picker::color_picker_color32(
                        ui,
                        &mut self.formats[selected_tt].color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });
        });
    }
}

// ----------------------------------------------------------------------------

#[cfg(feature = "syntect")]
struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

#[cfg(feature = "syntect")]
impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

#[cfg(feature = "syntect")]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(20.0),
                if theme.dark_mode {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::DARK_GREEN
                },
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, theme: &CodeTheme, text: &str, language: &str) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight(line, &self.ps) {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::none()
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

#[cfg(feature = "syntect")]
fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntect"))]
#[derive(Default)]
struct Highlighter {}

#[cfg(not(feature = "syntect"))]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, mut text: &str, _language: &str) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect

        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or(text.len());
                if end > 5 {
                    let word = &text[1..(end-1)];
                    let tt = if word.starts_with("{{") && word.ends_with("}}") {
                        TokenType::Keyword
                    } else {
                        TokenType::StringLiteral
                    };
                    let end_quote = &text[(end-1)..end];
                    job.append(&text[..1], 0.0, theme.formats[TokenType::StringLiteral].clone());
                    job.append(word, 0.0, theme.formats[tt].clone());
                    if end_quote == "\"" {
                        job.append(end_quote, 0.0, theme.formats[TokenType::StringLiteral].clone());
                    }
                } else {
                    job.append(&text[..end], 0.0, theme.formats[TokenType::StringLiteral].clone());
                }
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_template_word(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Whitespace].clone(),
                );
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Punctuation].clone(),
                );
                text = &text[end..];
            }
        }

        job
    }
}

#[cfg(not(feature = "syntect"))]
fn is_template_word(word: &str) -> bool {
    // word.starts_with("{{") && word.ends_with("}}")
    matches!(
        word,
        "for"|"in"|"range"|"endfor"|"endif"|"elif"|"else"|
        "NAME_ZH"|
        "NAME_EN"|
        "NUM"|
        "NUM_STR"|
        "HEX"|
        "STR"|
        "EMAIL"|
        "USERNAME"|
        "IPV4"|
        "IPV6"|
        "MAC"|
        "USERAGENT"|
        "PASSWORD"|
        "UUID"|
        "UUID_SIMPLE"|
        "NOW"|
        "DATE_BEFORE"|
        "DATE_AFTER"|
        "DATE"|
        "BASE64_EN"|
        "BASE64_DE"
    )
}

#[cfg(not(feature = "syntect"))]
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}
