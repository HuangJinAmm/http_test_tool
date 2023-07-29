use crate::{
    api_context::{ ApiContext, ApiTester, CollectionsData},
    component::tree_ui::{self, TreeUi},
    request_data::{covert_to_ui, PreHttpTest, PreRequest, PreResponse, RequestData, ResponseData},
    utils::{
        // rhai_script::ScriptEngine,
        template::add_global_var, rhai_script::SCRIPT_ENGINE,
    },
};
use chrono::Local;
use egui::{
    global_dark_light_mode_switch, Color32, FontData, FontDefinitions, Frame, Id,
    Window,
};
use egui_dock::{DockArea, Style, Tree};
use egui_file::{DialogType, FileDialog};
use egui_notify::Toasts;
use futures::StreamExt;
use log::info;
use minijinja::value::Value as JValue;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use reqwest::{Client, Request};
use rhai::Scope;
use std::time::Duration;
use std::thread;
use std::{io::BufReader, sync::Mutex};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};
/**
 * 全局变量
 */
const TEMP_GLOBAL_KEY: &str = "PRE_HTTP";

static TABS: OnceCell<Vec<String>> = OnceCell::new();
pub static REQ_UI_ID: OnceCell<Id> = OnceCell::new();
// id.times
pub static mut TASK_CHANNEL: Lazy<(Sender<(u64, u32, u32)>, Receiver<(u64, u32, u32)>)> =
    Lazy::new(|| tokio::sync::mpsc::channel(100));
// id,time
pub static mut RESULTE_CHANNEL: Lazy<(
    Sender<(u64, i64, ResponseData)>,
    Receiver<(u64, i64, ResponseData)>,
)> = Lazy::new(|| tokio::sync::mpsc::channel(100));
pub static mut M_RESULTE_CHANNEL: Lazy<(
    Sender<(u64, usize, i64, ResponseData)>,
    Receiver<(u64, usize, i64, ResponseData)>,
)> = Lazy::new(|| tokio::sync::mpsc::channel(100));
pub static TOASTS: OnceCell<Arc<Mutex<Toasts>>> = OnceCell::new();
pub static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(16)
        .build()
        .unwrap()
});
pub static mut CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default()
});

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    show_log: bool,

    test: String,
    // text: DockUi
    // pub tabs: HashSet<String>,
    pub tree: Tree<String>,
    tree_ui: TreeUi,
    api_data: ApiContext,
    #[serde(skip)]
    opened_file: Option<PathBuf>,
    #[serde(skip)]
    open_file_dialog: Option<FileDialog>,
    // #[serde(skip)]
    // script_engine: ScriptEngine,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut api_context = ApiContext::new();
        api_context.insert_collecton(0, CollectionsData::default());
        Self {
            show_log: false,
            test: "".to_owned(),
            tree_ui: TreeUi::new(),
            // tabs:vec![],
            tree: Tree::new(vec![]),
            api_data: api_context,
            // script_engine: ScriptEngine::new(),
            open_file_dialog: None,
            opened_file: None,
        }
    }
}

impl TemplateApp {
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

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            return app;
        }
        TemplateApp::default()
    }

    // pub fn load_ui(&mut self,title: String,tab_ui: Box<dyn TabUi<T = ApiContext>>) {
    //     self.tabs.insert(title, tab_ui);
    // }

    // pub fn register_ui(&mut self, title: String, tab_ui: Box<dyn TabUi<T = ApiContext>>) {
    //     let _ = self.tabs.insert(title.clone(), tab_ui);
    //     self.tree.push_to_focused_leaf(title);
    // }

    pub fn trigger_tab(&mut self, tab: &String) {
        if let Some(index) = self.tree.find_tab(tab) {
            self.tree.remove_tab(index);
        } else {
            self.tree.push_to_focused_leaf(tab.clone());
        }
    }

    pub fn open_tab(&mut self, tab: &String) {
        if let None = self.tree.find_tab(tab) {
            self.tree.push_to_focused_leaf(tab.clone());
        }
    }

    pub fn is_open(&self, title: &String) -> bool {
        if let Some(_index) = self.tree.find_tab(title) {
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, title: &String) -> bool {
        if let Some(index) = self.tree.find_tab(title) {
            let _rm = self.tree.remove_tab(index);
        }
        true
    }

    fn send_request(&self, req: &RequestData, id: u64) {
        let req = req.clone();
        TOKIO_RT.spawn(async move {
            let start = Local::now().timestamp_millis();
            let pre_req: PreRequest = (&req).into();
            let mut resp: ResponseData;
            if let Ok(send_req) = req.try_into() {
                resp = match unsafe { CLIENT.execute(send_req) }.await {
                    Ok(rep) => covert_to_ui(rep).await,
                    Err(err) => ResponseData {
                        headers: Default::default(),
                        body: err.to_string(),
                        size: 0,
                        code: "999".to_owned(),
                        time: 0,
                    },
                };
                let pre_resp: PreResponse = (&resp).into();
                let pre_http = PreHttpTest {
                    req: pre_req,
                    resp: pre_resp,
                };
                add_global_var(
                    TEMP_GLOBAL_KEY.to_owned(),
                    JValue::from_serializable(&pre_http),
                );
                add_global_var(format!("REQ_{}", id), JValue::from_serializable(&pre_http));
            } else {
                resp = ResponseData {
                    headers: Default::default(),
                    body: "请求URL解析错误".to_string(),
                    size: 0,
                    code: "999".to_owned(),
                    time: 0,
                };
            }
            let end = Local::now().timestamp_millis();
            let now = end - start;
            resp.time = now;
            let _send_res = unsafe { RESULTE_CHANNEL.0.send((id, now, resp)).await };
        });
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if ctx.style().visuals.dark_mode {
            catppuccin_egui::set_theme(&ctx, catppuccin_egui::MACCHIATO);
        } else {
            // catppuccin_egui::set_theme(&ctx, catppuccin_egui::LATTE);
        }
        let toast = TOASTS.get_or_init(|| {
            Arc::new(Mutex::new(
                Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            ))
        });
        if self.show_log {
            Window::new("Log").title_bar(true).show(ctx, |ui| {
                // draws the logger ui.
                egui_logger::logger_ui(ui);
            });
        }

        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                global_dark_light_mode_switch(ui);

                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                ui.menu_button("File", |ui| {
                    if (ui.button("Import")).clicked() {
                        let mut dialog = FileDialog::open_file(self.opened_file.clone())
                            .show_rename(false)
                            .filter(Box::new(|p| p.to_string_lossy().ends_with("json")));
                        dialog.open();
                        self.open_file_dialog = Some(dialog);
                    }

                    if (ui.button("Export")).clicked() {
                        let mut dialog = FileDialog::save_file(self.opened_file.clone())
                            .default_filename("app.json");
                        dialog.open();
                        self.open_file_dialog = Some(dialog);
                    }

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                if ui.selectable_label(self.show_log, "打开日志").clicked() {
                    self.show_log = !self.show_log;
                }
                if !frame.is_web() {
                    ui.menu_button("Zoom", |ui| {
                        egui::gui_zoom::zoom_menu_buttons(ui, frame.info().native_pixels_per_point);
                    });
                }

                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in TABS
                        .get_or_init(|| {
                            vec![
                                "请求".to_owned(),
                                "响应".to_owned(),
                                "设置".to_owned(),
                                "图表".to_owned(),
                                // "记录".to_owned(),
                                "脚本".to_owned(),
                            ]
                        })
                        .iter()
                    {
                        if ui
                            .selectable_label(self.is_open(tab), tab.clone())
                            .clicked()
                        {
                            self.trigger_tab(tab);
                            ui.close_menu();
                        }
                    }
                });
            });
        });

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.clone());
                    match dialog.dialog_type() {
                        DialogType::OpenFile => {
                            if let Ok(rfile) = std::fs::File::open(file.clone()) {
                                let reader = BufReader::new(rfile);
                                let app: TemplateApp = serde_json::from_reader(reader).unwrap();
                                *self = app;
                                // self.records = app.records;
                                // self.records_list = app.records_list;
                                // self.list_selected = app.list_selected;
                                // self.list_selected_str = app.list_selected_str;
                            }
                        }
                        DialogType::SaveFile => {
                            let app_json =
                                std::fs::File::open(file.clone()).unwrap_or_else(|_err| {
                                    std::fs::File::create(file.clone()).unwrap()
                                });
                            if let Err(err) = serde_json::to_writer_pretty(app_json, self) {
                                if let Ok(mut toast_w) = toast.lock() {
                                    toast_w
                                        .error(format!("save file error:{}", err.to_string()))
                                        .set_duration(Some(Duration::from_secs(5)));
                                }
                            } else {
                                if let Ok(mut toast_w) = toast.lock() {
                                    toast_w
                                        .info(format!(
                                            "file saved success:{}",
                                            file.to_string_lossy()
                                        ))
                                        .set_duration(Some(Duration::from_secs(5)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        egui::SidePanel::left("side_panel")
            .max_width(240.0)
            .show(ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    // ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui|{
                    match self.tree_ui.ui_impl(ui) {
                        tree_ui::Action::Keep => {
                            //ignore
                        }
                        tree_ui::Action::Delete(dels) => {
                            for del_id in dels {
                                info!("删除{}", del_id);
                                self.api_data.delete_collecton(del_id);
                                self.api_data.delete_test(del_id);
                            }
                        }
                        tree_ui::Action::Add((adds, node_type)) => {
                            let add_id = adds.first().unwrap().to_owned();
                            info!("添加{},{:?}", &add_id, &node_type);
                            match node_type {
                                tree_ui::NodeType::Collection => {
                                    self.api_data
                                        .insert_collecton(add_id, CollectionsData::default());
                                }
                                tree_ui::NodeType::Node => {
                                    self.api_data.insert_test(add_id, ApiTester::default());
                                }
                            }
                        }
                        tree_ui::Action::Rename(_adds) => {
                            //基本上不用处理
                            info!("重命名")
                        }
                        tree_ui::Action::Selected((selected_id, selected_title)) => {
                            let selected = *selected_id.first().unwrap_or(&0);
                            self.api_data.selected = selected_id;
                            if let Ok(mut toast_w) = toast.lock() {
                                toast_w
                                    .info(format!("已选中{}-标题{}", selected, selected_title))
                                    .set_duration(Some(Duration::from_secs(5)));
                            }
                        }
                        tree_ui::Action::Copy(cop) => {
                            if let Ok(mut toast_w) = toast.lock() {
                                toast_w
                                    .info(format!("已复制{}", cop.0))
                                    .set_duration(Some(Duration::from_secs(5)));
                            }
                        }
                        tree_ui::Action::Parse(mut parse) => {
                            //复制动作
                            let _ = parse.pop();
                            if let Some((sid, did)) = self.tree_ui.parse_node(parse) {
                                if let Some(copyed) = self.api_data.tests.get(&sid) {
                                    let parse = copyed.clone();
                                    self.api_data.insert_test(did, parse);
                                }
                            }
                        }
                    }
                    ui.add_space(ui.available_height());
                });
                //    });
            });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let mut dst = Style::from_egui(ui.style());

                dst.separator.color_dragged = Color32::RED;
                // dst.tab_text_color_active_focused = Color32::BROWN;
                DockArea::new(&mut self.tree)
                    .style(dst)
                    .show_inside(ui, &mut self.api_data);
            });

        // ...
        if let Ok(send) = unsafe { TASK_CHANNEL.1.try_recv() } {
            // if let Ok(mut toast_w) = toast.lock() {
            //     toast_w
            //         .info(format!("发送{}号{}次", send.0, send.1))
            //         .set_duration(Some(Duration::from_secs(5)));
            // }
            if let Some(req) = self.api_data.tests.get(&send.0) {
                if let Err(e) = self.api_data.run_script() {
                    if let Ok(mut toast_w) = toast.lock() {
                        toast_w
                            .info(format!("脚本执行错误:{}-{}", e.position(), e.to_string()))
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                } else {
                    if send.1 == 0 || send.2 == 0 {
                        //执行脚本,不发请求
                    } else if send.1 == 1 && send.2 == 1 {
                        self.send_request(&req.req, send.0);
                    } else {
                        load_test_sender(&req.req, send.0, send.1, send.2);
                    }
                }
            }
        }

        if let Ok((resp_id, resp_time, resp_data)) = unsafe { RESULTE_CHANNEL.1.try_recv() } {
            if let Ok(mut toast_w) = toast.lock() {
                toast_w
                    .info(format!("{}号响应时间{}", resp_id, resp_time))
                    .set_duration(Some(Duration::from_secs(5)));
            }
            if let Some(resp_dn) = self.api_data.tests.get_mut(&resp_id) {
                resp_dn.resp = resp_data;
                if let Some(req_id) = REQ_UI_ID.get() {
                    let state_id = req_id.with(resp_id);
                    //更新对应的ui状态
                    let _send_state = ctx.data_mut(|d| d.insert_temp(state_id, false));
                }
                let script_scope = &mut Scope::new();
                script_scope.push("_req_url", resp_dn.req.url.clone());
                script_scope.push("_req_body", resp_dn.req.body.clone());
                script_scope.push("_resp", resp_dn.resp.clone());
                if let Err(e) = SCRIPT_ENGINE.run_with_scope(script_scope,&resp_dn.script.after) {
                    if let Ok(mut toast_w) = toast.lock() {
                        toast_w
                            .info(format!("脚本执行错误:{}-{}", e.position(), e.to_string()))
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                }
            }
        }

        if let Ok(resp_rs) = unsafe { M_RESULTE_CHANNEL.1.try_recv() } {
            //结束
            info!("返回响应{:?}", &resp_rs.3);
            if resp_rs.1 == 0 && resp_rs.2 == -1 {
                if let Some(req_id) = REQ_UI_ID.get() {
                    let state_id = req_id.with(resp_rs.0);
                    //更新对应的ui状态
                    let _send_state = ctx.data_mut(|d| d.insert_temp(state_id, false));
                }
            } else {
                if let Some(resp_dn) = self.api_data.tests.get_mut(&resp_rs.0) {
                    if &resp_rs.3.code == "999" {
                        resp_dn.load_test.result.error += 1.0;
                    }
                    resp_dn.load_test.update_process();
                    resp_dn.load_test.add_result(resp_rs.1, resp_rs.2);
                    resp_dn.load_test.recode_time(resp_rs.2);
                }
            }
        }
        if let Ok(mut toast_w) = toast.lock() {
            toast_w.show(ctx);
        }
    }
}

fn load_test_sender(req: &RequestData, id: u64, reqs: u32, round: u32) {
    let capacity: usize = (round as usize) * (reqs as usize);
    let mut sender_requsets: Vec<(usize, Request)> = Vec::with_capacity(capacity);
    for i in 0..capacity {
        let req_clone = req.clone();
        if let Ok(real_req) = req_clone.try_into() {
            sender_requsets.push((i, real_req));
        }
    }
    sender_requsets.reverse();

    let _ = thread::Builder::new()
        .name("send_req_thread".to_string())
        .spawn(move || {
            let _respui = TOKIO_RT.block_on(async move {
                for _ in 0..round {
                    // let client = reqwest::Client::new();
                    let start = std::time::SystemTime::now();
                    let mut f_vec = Vec::new();
                    for _ in 0..reqs {
                        let req = sender_requsets.pop().unwrap();
                        let f = send_load_test_request(req, id);
                        // let _tf = tokio::task::spawn(f);
                        f_vec.push(f);
                    }
                    // println!("生成完成：{}-{}",f_vec.len(),duration);
                    // let result = stream::iter(f_vec)
                    //                         .buffer_unordered(16)
                    //                         .collect::<Vec<_>>().await;
                    // let _tf = tokio::spawn(
                    tokio_stream::iter(f_vec)
                        .buffered(reqs as usize)
                        .collect::<Vec<_>>()
                        .await;
                    // );
                    let duration = start.elapsed().unwrap().as_millis() as u64;
                    if duration < 1000 {
                        tokio::time::sleep(Duration::from_millis(1000 - duration)).await;
                    }
                }
                let _rs = unsafe {
                    M_RESULTE_CHANNEL
                        .0
                        .send((id, 0, -1, ResponseData::default()))
                        .await
                };
            });
        });
    //发送一个完成的数据
}

async fn send_load_test_request(ireq: (usize, Request), id: u64) {
    let start = Local::now().timestamp_millis();
    let (index, req) = ireq;
    match unsafe { CLIENT.execute(req) }.await {
        Ok(rep) => {
            let resp_ui = covert_to_ui(rep).await;
            let end = Local::now().timestamp_millis();
            let _ = unsafe {
                M_RESULTE_CHANNEL
                    .0
                    .send((id, index, end - start, resp_ui))
                    .await
            };
        }
        Err(err) => {
            let resp_ui = ResponseData {
                headers: Default::default(),
                body: err.to_string(),
                size: 0,
                code: "999".to_owned(),
                time: 0,
            };
            let end = Local::now().timestamp_millis();
            let _ = unsafe {
                M_RESULTE_CHANNEL
                    .0
                    .send((id, index, end - start, resp_ui))
                    .await
            };
        }
    }
}
