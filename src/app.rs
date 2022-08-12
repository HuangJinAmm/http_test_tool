use std::collections::{BTreeMap};
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{Receiver};



use egui::{FontData, FontDefinitions, Id, Label, TextStyle};

// use egui_extras::{Size, StripBuilder, TableBuilder};
use serde::{Deserialize, Serialize};

use crate::component::context_list::Action::Selected;
use crate::component::context_list::Action::Delete;
use crate::component::context_list::Action::Keep;
use crate::component::context_list::ContextTree;
use crate::component::request_ui::{NetTestUi, ResponseUi};

pub const ADD_ID_KEY: &str = "Http_esay_mocker_recodes";
const RESPONSE_UI_KEY: &str = "Http_esay_mocker_response_ui";
const APP_KEY: &str = "Http_easy_tester_xxxx";
pub const ID_COUNT_KEY: &str = "Http_easy_mocker_count_id";
const NOTIFICATION_KEY: &str = "http_easy_mocker_Notice";
lazy_static! {
    static ref NOTIFICATION_ID: Id = Id::new(NOTIFICATION_KEY);
}
const NOTIFICATION_SHOW_TIME: i64 = 5000; //毫秒

/// Represents an HTTP method.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(format!("Invalid HTTP method {}", input)),
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        value.parse().expect("Cannot parse HTTP method")
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum AppTab {
    Mock,
    Req,
    // Test,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    is_exiting: bool,
    can_exit: bool,

    apptab: AppTab,
    label: String,
    is_pop: bool,
    filter: String,
    records_list: ContextTree,
    list_selected: u64,
    list_selected_str: Option<String>,
    records: BTreeMap<u64, NetTestUi>,

    #[serde(skip)]
    reciever: Option<Receiver<(u64, ResponseUi)>>,
    #[serde(skip)]
    add_reciever: Option<Receiver<(u64, u64)>>, // current: Option<ApiRecordDefinition>,
                                                // notifications:Vec<(u64, String)>,
                                                // method: Method,
                                                // left:ContextList,
                                                // this how you opt-out of serialization of a member
                                                // #[serde(skip)]
                                                // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            reciever: None,
            add_reciever: None,
            is_exiting: false,
            can_exit: false,
            apptab: AppTab::Req,
            // netTestUi: Default::default(),
            label: "测试案例1".to_owned(),
            // value: 2.7,
            is_pop: false,
            records_list: unsafe { ContextTree::new(0, "HTTP测试") },
            list_selected_str: None,
            list_selected: 0,
            // notifications:Vec::new(),
            filter: "".into(),
            // current: None,
            // method: Method::GET,
            // notifications: VecDeque::new(),
            records: BTreeMap::new(),
        }
    }
}

pub(crate) fn add_notification(ctx: &egui::Context, notice: &str) {
    let mut egui_data = ctx.data();
    let notice_vec: &mut Vec<(i64, String)> =
        egui_data.get_temp_mut_or_default(NOTIFICATION_ID.clone());
    let now = chrono::Local::now().timestamp_millis();
    notice_vec.push((now, notice.to_string()));
}

impl TemplateApp {
    fn display_notifications(&mut self, ctx: &egui::Context) {
        let mut offset = 0.;
        let notice_vec_clone;
        {
            let mut egui_data = ctx.data();
            let notice_vec: Vec<(i64, String)> =
                egui_data.get_temp(NOTIFICATION_ID.clone()).unwrap();
            notice_vec_clone = notice_vec.clone();
        }
        let now = chrono::Local::now().timestamp_millis();
        // let notice_own_vec = std::mem::take(notice_vec);
        notice_vec_clone
            .iter()
            .filter(|notice| notice.0 + NOTIFICATION_SHOW_TIME > now)
            .for_each(|notice| {
                if let Some(response) = egui::Window::new("通知")
                    .id(egui::Id::new(offset as u32))
                    .anchor(egui::Align2::RIGHT_TOP, (0., offset))
                    .default_size([200., 80.])
                    .fixed_size([200., 80.])
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(notice.1.clone());
                    })
                {
                    offset += response.response.rect.height();
                }
            });
        // *notice_vec = filted_notice_vec;

        // for (_, notification) in filted_notice_vec.iter() {
        //     if let Some(response) = egui::Window::new("通知")
        //         .id(egui::Id::new(offset as u32))
        //         .anchor(egui::Align2::RIGHT_TOP, (0., offset))
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ctx, |ui| {
        //             ui.label(notification);
        //         })
        //     {
        //         offset += dbg!(response.response.rect.height());
        //     }
        // }
        // for (_, error) in &self.errors {
        //     if let Some(response) = egui::Window::new("Error")
        //         .id(egui::Id::new(offset as u32))
        //         .anchor(egui::Align2::RIGHT_TOP, (0., offset))
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ctx, |ui| {
        //             ui.colored_label(egui::Color32::RED, error);
        //         })
        //     {
        //         offset += response.response.rect.height();
        //     }
        // }
    }

    // pub fn reset(&mut self) {
    //     self.records = BTreeMap::new();
    //     self.records_list = ContextTree::new(0, "HTTP测试");
    //     self.filter = "".to_string();
    //     self.list_selected = 0;
    //     self.list_selected_str = None;
    // }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("MI_LanTing_Regular.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);


        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let notice_id = NOTIFICATION_ID.clone();
        let notice_vec: Vec<(i64, String)> = Vec::new();
        cc.egui_ctx.data().insert_temp(notice_id, notice_vec);



        if let Some(storage) = cc.storage {
            let mut app: TemplateApp = eframe::get_value(storage, APP_KEY).unwrap_or_default();
            app.is_exiting = false;
            app.can_exit = false;
            let (sender, reciever) = std::sync::mpsc::sync_channel::<(u64, ResponseUi)>(10);
            let (add_sender, add_reciever) = std::sync::mpsc::sync_channel::<(u64, u64)>(100);
            // let arc_sender = Arc::new(Mutex::new(sender));
            cc.egui_ctx
                .data()
                .insert_temp(Id::new(RESPONSE_UI_KEY), sender);
            cc.egui_ctx
                .data()
                .insert_temp(Id::new(ADD_ID_KEY), add_sender);
            app.reciever = Some(reciever);
            app.add_reciever = Some(add_reciever);
            app.records.iter_mut().for_each(|(_, netUI)| {
                netUI.add_mpsc();
            });
            return app;
        }
        let (sender, reciever) = std::sync::mpsc::sync_channel::<(u64, ResponseUi)>(10);
        let (add_sender, add_reciever) = std::sync::mpsc::sync_channel::<(u64, u64)>(100);
        // let arc_sender = Arc::new(Mutex::new(sender));
        cc.egui_ctx
            .data()
            .insert_temp(Id::new(RESPONSE_UI_KEY), sender);
        cc.egui_ctx
            .data()
            .insert_temp(Id::new(ADD_ID_KEY), add_sender);
        println!("初始化==================");
        let mut app: TemplateApp = Default::default();
        app.reciever = Some(reciever);
        app.add_reciever = Some(add_reciever);
        app
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    // fn on_exit_event(&mut self) -> bool {
    //     self.is_exiting = true;
    //     self.can_exit
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // let Self {
        //     label,
        //     apptab,
        //     // is_login,
        //     list_selected,
        //     list_selected_str,
        //     records_list,
        //     records,
        //     filter,
        // } = self;

        self.display_notifications(ctx);

        if let Ok((sup, sub)) = self.add_reciever.as_ref().unwrap().try_recv() {
            let sup_record = self.records.get(&sup).unwrap();
            let mut sub_record = NetTestUi::clone_from(sup_record);
            sub_record.add_mpsc();
            self.records.insert(sub, sub_record);
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        // if !ctx.input().raw.dropped_files.is_empty() {
        //     let dropped_files = ctx.input().raw.dropped_files.clone();
        //     for file in dropped_files {
        //         if let Some(file_p) = file.path {
        //             if file_p.ends_with("json") {
        //                 if let Some(bytes) = file.bytes {
        //                     let slice_bytes = bytes.as_ref();
        //                     let mut app: TemplateApp = serde_json::from_slice(slice_bytes).unwrap();
        //                     app.records.iter_mut().for_each(|(_, netUI)| {
        //                         netUI.add_mpsc();
        //                     });
        //                     self.records = app.records;
        //                     self.records_list = app.records_list;
        //                     self.list_selected = app.list_selected;
        //                     self.list_selected_str = app.list_selected_str;
        //                 }
        //             }
        //         }
        //     }
        // }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.menu_button("菜单", |ui| {
                    if ui.button("退出").clicked() {
                        frame.quit();
                    }
                    if ui
                        .button("加载json文件")
                        .on_hover_text("将加载与app同路径下的app.json文件")
                        .clicked()
                    {
                        let ok = rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Info)
                            .set_buttons(rfd::MessageButtons::OkCancel)
                            .set_description("将加载与app同路径下的app.json文件")
                            .set_title("加载文件")
                            .show();
                        if ok {
                            if let Ok(file) = std::fs::File::open("app.json") {
                                let reader = BufReader::new(file);
                                let mut app: TemplateApp = serde_json::from_reader(reader).unwrap();
                                app.records.iter_mut().for_each(|(_, netUI)| {
                                    netUI.add_mpsc();
                                });
                                self.records = app.records;
                                self.records_list = app.records_list;
                                self.list_selected = app.list_selected;
                                self.list_selected_str = app.list_selected_str;
                            }
                        }
                    }
                    if ui.button("保存为json文件").clicked() {
                        let app_json = std::fs::File::open("app.json")
                            .unwrap_or_else(|_err| std::fs::File::create("app.json").unwrap());
                        if let Err(err) = serde_json::to_writer_pretty(app_json, self) {
                            rfd::MessageDialog::new()
                                .set_level(rfd::MessageLevel::Error)
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description(format!("app.json读取错误:{}",err.to_string()).as_str())
                                .set_title("读取文件错误")
                                .show();
                        } else {
                            rfd::MessageDialog::new()
                                .set_level(rfd::MessageLevel::Info)
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description("已将app信息保存app同路径下的app.json文件中")
                                .set_title("保存文件")
                                .show();
                        }
                    }

                    if ui.button("选择json文件…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("app存储文件", &["json", "JSON"])
                            .set_file_name("app")
                            .set_directory(".")
                            .pick_file()
                        {
                            let picked_path = Some(path.display().to_string()).unwrap();
                            if let Ok(file) = std::fs::File::open(picked_path) {
                                let reader = BufReader::new(file);
                                let mut app: TemplateApp = serde_json::from_reader(reader).unwrap();
                                app.records.iter_mut().for_each(|(_, netUI)| {
                                    netUI.add_mpsc();
                                });
                                // *self = app;
                                self.records = app.records;
                                self.records_list = app.records_list;
                                self.list_selected = app.list_selected;
                                self.list_selected_str = app.list_selected_str;
                            }
                        }
                    }
                    // if ui.button("清除所有记录").clicked() {
                    //     self.is_pop=true;
                    // }
                    // if self.is_pop {
                    // egui::Window::new("警告")
                    //     .collapsible(false)
                    //     .resizable(false)
                    //     .fixed_size([80.,140.])
                    //     .anchor(egui::Align2::CENTER_CENTER, [0.0,0.0])
                    //     .show(ctx, |ui|{
                    //         ui.add_space(20.);
                    //         ui.label("是否清除所有记录?");
                    //         ui.add_space(20.);
                    //         ui.horizontal(|ui|{
                    //             if ui.button("是").clicked() {
                    //                 self.reset();
                    //                 self.is_pop = false;
                    //             }
                    //             if ui.button("否").clicked() {
                    //                 self.is_pop = false;
                    //             }

                    //         });
                    //     });
                    // }
                });
                // ui.selectable_value(apptab, AppTab::Mock, "模拟");
                // ui.selectable_value(apptab, AppTab::Req, "请求");
                // ui.add(toggle(is_login));
                // ui.toggle_value(is_login, "历史记录");
                // ui.selectable_value(apptab, AppTab::Test, "测试");
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui
                .interact(
                    ui.available_rect_before_wrap(),
                    ui.id(),
                    egui::Sense::drag(),
                )
                .drag_released()
            {}

            ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("筛选条件"));
            let list_resp = self.records_list.ui_impl(
                ui,
                self.list_selected.clone(),
                &mut self.label,
                &mut self.filter,
            );
            match list_resp {
                Selected((id, title)) => {
                    self.list_selected = id;
                    self.list_selected_str = Some(title)
                }
                Delete(subids) => {
                    subids.iter().for_each(|id|{
                        self.records.remove(id);
                    });
                },
                Keep => {},
            }

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     ui.horizontal(|ui| {
            //         ui.spacing_mut().item_spacing.x = 0.0;
            //         ui.label("powered by ");
            //         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            //         ui.label(" and ");
            //         ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
            //     });
            // });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.apptab {
                AppTab::Mock => {
                    ui.add(Label::new(
                        egui::RichText::new("模拟服务器")
                            .color(egui::Color32::RED)
                            .heading(),
                    ));
                }
                AppTab::Req => {
                    // ui.add(Label::new(
                    //     egui::RichText::new("发送请求")
                    //         .color(egui::Color32::RED)
                    //         .heading(),
                    // ));
                    ui.heading(
                        self.list_selected_str
                            .clone()
                            .or(Some("未命名".into()))
                            .unwrap()
                            .as_str(),
                    );
                    // ui.separator();
                    if let Some(netUi) = self.records.get_mut(&self.list_selected) {
                        netUi.ui(ui);
                    } else {
                        let mut netUI = NetTestUi::test();
                        netUI.ui(ui);
                        self.records.insert(self.list_selected.to_owned(), netUI);
                    }
                } // AppTab::Test => {
                  //     ui.add(Label::new(
                  //         egui::RichText::new("压力测试")
                  //             .color(egui::Color32::RED)
                  //             .heading(),
                  //     ));
                  // }
            }
        });

        // if self.is_exiting {
        // let ok = rfd::MessageDialog::new()
        //     .set_level(rfd::MessageLevel::Info)
        //     .set_buttons(rfd::MessageButtons::YesNo)
        //     .set_description("是否退出应用？")
        //     .set_title("退出")
        //     .show();
        // if ok {
        //     self.can_exit = true;
        //     frame.quit();
        // }
        // egui::Window::new("确认退出?")
        //     .collapsible(false)
        //     .resizable(false)
        //     .show(ctx, |ui| {
        //         ui.horizontal(|ui| {
        //             if ui.button("暂不").clicked() {
        //                 self.is_exiting = false;
        //             }

        //             if ui.button("是的").clicked() {
        //                 self.can_exit = true;
        //                 frame.quit();
        //             }
        //         });
        //     });
        // }

        // if *is_login {
        // egui::SidePanel::right("right_panel").show(ctx, |ui| {
        //     ui.label("Windows can be moved by dragging them.");
        // });
        // }

        // if !*is_login {
        //     egui::Window::new("登录")
        //         .collapsible(false)
        //         .resizable(false)
        //         // .open(&mut false)
        //         .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0f32, 0f32))
        //         .show(ctx, |ui| {
        //             egui::Grid::new("login_grid")
        //                 .num_columns(2)
        //                 .spacing([40.0, 4.0])
        //                 // .striped(true)
        //                 .show(ui, |ui| {
        //                     ui.label("用户名：");
        //                     ui.text_edit_singleline(label).on_hover_text("请输入用户名");
        //                     ui.end_row();

        //                     ui.label("密  码：");
        //                     ui.add(password(label));
        //                     ui.end_row();

        //                     ui.label("验证码：");
        //                     ui.text_edit_singleline(&mut "请输入验证码");
        //                     ui.end_row();

        //                     ui.add_visible(false, egui::Label::new("zhanwein"));
        //                     ui.horizontal(|ui| {
        //                         let login_click = ui.button("登    录");
        //                         let _regist_click = ui.button("注    册");
        //                         let _forget_password = ui.button("忘记密码");

        //                         if login_click.clicked() {
        //                             *is_login = true;
        //                         }
        //                     })
        //                 });
        //         });
        // }
    }
}

// fn api_url_ui(method: &str, url: &str, mode: bool, font_size: f32) -> LayoutJob {
//     let mut job = LayoutJob::default();

//     let (default_color, strong_color, bg_color) = if mode {
//         (Color32::LIGHT_GRAY, Color32::WHITE, Color32::DARK_RED)
//     } else {
//         (Color32::DARK_GRAY, Color32::BLACK, Color32::LIGHT_RED)
//     };
//     let font = FontId::new(font_size, egui::FontFamily::Proportional);

//     job.append(
//         method,
//         0.0,
//         TextFormat {
//             color: strong_color,
//             font_id: font.clone(),
//             background: bg_color,
//             ..Default::default()
//         },
//     );

//     job.append(
//         url,
//         0.0,
//         TextFormat {
//             color: default_color,
//             font_id: font,
//             ..Default::default()
//         },
//     );
//     job
// }
