use chrono::Local;
use egui::plot::{Bar, BarChart, Legend, Line, Plot, Value as PValue, Values};
use egui::{Color32, Key, RichText, Ui, Vec2};
use hdrhistogram::Histogram;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::hash::Hasher;
use std::str::FromStr;
use futures::stream::{self, StreamExt};
use std::time::Duration;
use std::{
    collections::hash_map::DefaultHasher,
    hash::Hash,
    sync::mpsc::{Receiver, Sender},
    thread,
};
// use egui_extras::{Size, TableBuilder};

use crate::template::rander_template;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{Request, RequestBuilder, Response};
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::{Builder, Runtime};

use crate::app::{Method, add_notification};

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    static ref RT: Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(16)
        .build()
        .unwrap();
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
enum TestType {
    Req,
    Load,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct LoadTest {
    reqs: u32,
    time: u16,
    process: f32,
    #[serde(skip)]
    result: LoadTestResult,
    result_list: Vec<i64>,
}

impl LoadTest {
    #[inline]
    fn total(&self) -> u64 {
        (self.reqs as u64) * (self.time as u64)
    }
}

#[derive(Debug, Clone)]
// #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct LoadTestResult {
    // total: u16,
    // average: f32,
    // median: f32,
    // min: u16,
    // max: u16,
    // line90: f32,
    // line95: f32,
    // line99: f32,
    error: f32,
    recived: f32,
    send: f32,
    // #[serde(skip)]
    result_hist: Option<Histogram<u64>>,
}

impl Default for LoadTestResult {
    fn default() -> Self {
        Self {
            error: Default::default(),
            recived: Default::default(),
            send: Default::default(),
            result_hist: Some(Histogram::<u64>::new_with_bounds(1, 60 * 60 * 1000, 2).unwrap()),
        }
    }
}

impl LoadTestResult {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.with_layout(egui::Layout::left_to_right(), |ui| {
                // ui.horizontal_centered(|ui|{
                let result = self.result_hist.as_ref().unwrap();
                ui.label("总共请求数量：");
                let show_text = RichText::new(result.len().to_string()).color(Color32::GOLD);
                ui.label(show_text);

                ui.label("平均时间：");
                let show_text =
                    RichText::new(result.mean().to_string()).color(Color32::LIGHT_GREEN);
                ui.label(show_text);

                // ui.label("中位数：");
                // let show_text = RichText::new(result.median_equivalent(result.max()).to_string()).color(Color32::GREEN);
                // ui.label(show_text);

                ui.label("最小：");
                let show_text = RichText::new(result.min().to_string()).color(Color32::GRAY);
                ui.label(show_text);

                ui.label("最大：");
                let show_text = RichText::new(result.max().to_string()).color(Color32::RED);
                ui.label(show_text);

                ui.label("90%：");
                let show_text = RichText::new(result.value_at_percentile(0.9).to_string())
                    .color(Color32::BROWN);
                ui.label(show_text);

                ui.label("95%：");
                let show_text = RichText::new(result.value_at_percentile(0.95).to_string())
                    .color(Color32::BROWN);
                ui.label(show_text);

                ui.label("99%：");
                let show_text = RichText::new(result.value_at_percentile(0.99).to_string())
                    .color(Color32::BROWN);
                ui.label(show_text);

                ui.label("错误数：");
                let show_text = RichText::new(self.error.to_string()).color(Color32::RED);
                ui.label(show_text);

                ui.label("发送(KB)：");
                let show_text = RichText::new(self.send.to_string()).color(Color32::BLUE);
                ui.label(show_text);

                ui.label("接收(KB");
                let show_text = RichText::new(self.recived.to_string()).color(Color32::BLUE);
                ui.label(show_text);
            });
        });
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NetTestUi {
    issend: bool,
    isEdit: bool,
    remark: String,
    test_type: TestType,
    req: RequestUi,
    resp: ResponseUi,
    load_test: LoadTest,
    #[serde(skip)]
    sender: Option<Sender<(usize,i64, ResponseUi)>>,
    #[serde(skip)]
    reciver: Option<Receiver<(usize,i64, ResponseUi)>>,
}

impl NetTestUi {
    pub fn clone_from(other: &NetTestUi) -> Self {
        Self {
            issend: false,
            isEdit: false,
            remark: other.remark.clone(),
            test_type: TestType::Req,
            req: other.req.clone(),
            resp: other.resp.clone(),
            sender: None,
            reciver: None,
            load_test: other.load_test.clone(),
        }
    }

    pub fn add_mpsc(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.sender = Some(tx);

        self.reciver = Some(rx);
    }

    pub fn test() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            issend: false,
            test_type: TestType::Req,
            sender: Some(tx.clone()),
            reciver: Some(rx),
            isEdit: false,
            remark: "添加备注信息".to_string(),
            req: RequestUi {
                url: "http://www.baidu.com".to_string(),
                method: Method::GET,
                headers: SelectKeyValueInputs {
                    inputs: vec![SelectKeyValueItem {
                        key: "".to_string(),
                        value: "".into(),
                        selected: true,
                    }],
                },
                body: "".to_string(),
            },
            resp: ResponseUi {
                headers: SelectKeyValueInputs { inputs: vec![] },
                body: "".to_string(),
                size: 0,
                code: 0,
                time: 0,
            },
            load_test: Default::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RequestUi {
    url: String,
    method: Method,
    headers: SelectKeyValueInputs,
    body: String,
}

impl Default for RequestUi {
    fn default() -> Self {
        Self {
            url: Default::default(),
            method: Method::GET,
            headers: Default::default(),
            body: Default::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Into<Request> for RequestUi {
    fn into(self) -> Request {
        let mth_bytes = self.method.to_string();
        let mth = reqwest::Method::from_bytes(mth_bytes.as_bytes()).unwrap();
        let url = reqwest::Url::parse(self.url.as_str()).unwrap();

        let headers = self.headers.inputs.into_iter()
                .filter(|slk|slk.selected)
                .fold(HeaderMap::new(), |mut headmap,slk|{
                    let k = HeaderName::from_str(slk.key.as_str()).unwrap();
                    let v =HeaderValue::from_str(slk.value.as_str()).unwrap();
                    headmap.append(k, v);
                    headmap
        });
        let mut req = Request::new(mth, url);
        *req.headers_mut() =headers;
        if !self.body.is_empty() {
            let paser_body = rander_template(self.body.as_str()).unwrap_or(self.body);
            *req.body_mut() = Some(paser_body.into());
        }
        req
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Into<reqwest::blocking::Request> for RequestUi {
    fn into(self) -> reqwest::blocking::Request {
        let mth_bytes = self.method.to_string();
        let mth = reqwest::Method::from_bytes(mth_bytes.as_bytes()).unwrap();
        let url = reqwest::Url::parse(self.url.as_str()).unwrap();
        let mut req = reqwest::blocking::Request::new(mth, url);
        if !self.body.is_empty() {
            let paser_body = rander_template(self.body.as_str()).unwrap_or(self.body);
            *req.body_mut() = Some(paser_body.into());
        }
        req
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ResponseUi {
    headers: SelectKeyValueInputs,
    body: String,
    size: u64,
    code: u16,
    time: i64,
}

#[cfg(target_arch = "wasm32")]
impl From<ehttp::Response> for ResponseUi {
    fn from(res: ehttp::Response) -> Self {
        let status = res.status;
        let mut headers = SelectKeyValueInputs::default();
        for (key, value) in &res.headers {
            let mut item = SelectKeyValueItem::new();
            item.key = key.clone();
            item.value = value.clone();
            headers.inputs.push(item);
        }

        let body = res.text().unwrap().to_string();
        let size = res.bytes.len() / 1024;
        Self {
            headers: headers,
            body: body,
            size: size as u64,
            code: status,
            time: 0,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::blocking::Response> for ResponseUi {

    fn from(resp: reqwest::blocking::Response) -> Self {

        let status = resp.status().as_u16();
        let mut headers = SelectKeyValueInputs::default();
        let mut is_json = false;
        for (key,value) in resp.headers().into_iter() {
            let mut item = SelectKeyValueItem::new();
            item.key = key.to_string();
            item.value = match value.to_str() {
                Ok(ok) => ok.to_string(),
                Err(er) => er.to_string(),
            };
            if item.key.eq_ignore_ascii_case("content-type") && item.value.contains("application/json") {
                is_json = true;
            }
            headers.inputs.push(item);
        }

        let size = resp.content_length().unwrap_or(0)/1024;

        let body:String= match resp.text(){
            Ok(body) => {
                if is_json {
                    if let Ok(json) = serde_json::from_str::<Value>(body.as_str()) {
                        serde_json::to_string_pretty(&json).unwrap_or(body)
                    } else {
                        body
                    }
                } else {
                    body
                }
            },
            Err(err) => err.to_string(),
        };

        Self{
            headers: headers,
            body: body,
            size: size,
            code: status,
            time: 0,
        }
    }
}

impl NetTestUi {
    pub fn ui(&mut self, ui: &mut Ui) {
        // dbg!(&self);
        editable_label(ui, &mut self.isEdit, &mut self.remark);

        ui.horizontal_wrapped(|ui| {
            let send = ui.button("发送💨");
            if send.clicked() {
                self.issend = !self.issend;
                //发送请求
                match self.test_type {
                    TestType::Req => {
                        let sender_clone = self.sender.clone();
                        let req = self.req.clone();

                        if let Some(sender) = sender_clone {
                            send_request(sender, req);
                        }
                    }
                    TestType::Load => {
                        let times = self.load_test.time;
                        let reqs = self.load_test.reqs;
                        let t = times as usize * reqs as usize;
                        self.load_test.process = 0.0;
                        self.load_test.result_list = vec![0].repeat(t);
                        self.load_test.result = LoadTestResult::default();
                        let requset = self.req.clone();
                        let sender_clone = self.sender.clone();
                        if let Some(sender) = sender_clone {
                            start_load_test_multisender(sender, times, reqs, requset);
                        }
                    }
                }
            }
            //接收请求
            match self.test_type {
                TestType::Req => match self.reciver.as_ref() {
                    Some(rspui) => {
                        match rspui.try_recv() {
                            Ok(s) => {
                                ui.ctx().request_repaint();
                                self.resp = s.2;
                                let time = s.1;
                                self.resp.time = time;
                                self.issend = false;
                            }
                            Err(_) => {}
                        };
                    }
                    None => {}
                },
                TestType::Load => match self.reciver.as_ref() {
                    Some(rspui) => {
                        ui.ctx().request_repaint();
                        let mut rs_iter = rspui.try_iter();
                        while let Some(s) = rs_iter.next() {
                            if s.1 == -1 {
                                self.issend = false;
                                self.load_test.process = 1.0;
                                break;
                            }
                            if s.2.code > 500 {
                                self.load_test.result.error = self.load_test.result.error + 1.0;
                            }
                            // self.load_test.result_list.insert(s.0, s.1);
                            let _addto = self.load_test.result_list.get_mut(s.0).map(|r|*r=s.1);
                            self.load_test.process = self.load_test.result_list.len() as f32
                                / self.load_test.total() as f32;
                            let time = s.1 as u64;
                            self.load_test.result.recived =
                                self.load_test.result.recived + s.2.size as f32;
                            self.load_test
                                .result
                                .result_hist
                                .as_mut()
                                .unwrap()
                                .record(time).unwrap();
                        }
                    }
                    None => {}
                },
            }

            egui::ComboBox::from_label("🌐")
                .selected_text(format!("{:?}", &mut self.req.method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.req.method, Method::GET, "GET");
                    ui.selectable_value(&mut self.req.method, Method::POST, "POST");
                    ui.selectable_value(&mut self.req.method, Method::PUT, "PUT");
                    ui.selectable_value(&mut self.req.method, Method::DELETE, "DELETE");
                    ui.selectable_value(&mut self.req.method, Method::PATCH, "PATCH");
                    ui.selectable_value(&mut self.req.method, Method::OPTIONS, "OPTIONS");
                });
            ui.text_edit_singleline(&mut self.req.url)
                .on_hover_text("请求路径");
            if self.issend {
                ui.spinner();
            }
            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                // if self.resp.size > 0 {
                ui.label("KB");
                ui.label(self.resp.size.to_string());
                ui.label("响应大小：");
                // }
                // if self.resp.time > 0 {
                ui.label("毫秒");
                ui.label(self.resp.time.to_string());
                ui.label("响应时间：");
                // }
                let codeRichText = match self.resp.code {
                    x if x >= 100 && x < 200 => RichText::new(x.to_string()).color(Color32::YELLOW),
                    x if x >= 200 && x < 400 => RichText::new(x.to_string()).color(Color32::GREEN),
                    x if x >= 400 && x < 600 => RichText::new(x.to_string()).color(Color32::RED),
                    _ => RichText::new(""),
                };
                ui.label(codeRichText);
                ui.label("响应状态码：")
            });
            ui.end_row();
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.test_type, TestType::Req, "单个测试");
            ui.selectable_value(&mut self.test_type, TestType::Load, "批量测试");
        });

        if self.test_type == TestType::Req {
            ui.with_layout(egui::Layout::left_to_right(), |ui| {
                self.req.ui(ui);
                self.resp.ui(ui);
            });
        } else {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("并发数(req/s):");
                    ui.add(egui::DragValue::new(&mut self.load_test.reqs).speed(1));
                    ui.label("持续时间(s):");
                    ui.add(egui::DragValue::new(&mut self.load_test.time).speed(1));
                    ui.add(egui::ProgressBar::new(self.load_test.process));
                });
            });

            ui.horizontal(|ui| {
                self.load_test.result.ui(ui);
            });

            let show_plot = self.load_test.result_list.clone();
            ui.vertical(|ui| {
                //运行图表
                ui.group(|ui| {
                    // let all = ui.available_width();
                    // ui.set_max_width(all/2.0);
                    let all_h = ui.available_height();
                    ui.set_max_height(all_h / 2.0);
                    let sin = show_plot
                        .iter()
                        .enumerate()
                        .map(|(x, y)| PValue::new(x as f64, *y as f64));
                    let line = Line::new(Values::from_values_iter(sin));
                    Plot::new("runing")
                        .data_aspect(1.0)
                        .show(ui, |plot_ui| plot_ui.line(line));
                });
                //运行结果图表
                let hdrhist = self.load_test.result.result_hist.clone().unwrap();
                ui.group(|ui| {
                    let max_hdr = hdrhist.max();
                    let min_hdr = hdrhist.min();
                    let mut chart = BarChart::new(
                        (min_hdr..max_hdr)
                            .step_by(10)
                            .map(|x| (x as f64, hdrhist.count_between(x, x + 10) as f64))
                            // The 10 factor here is purely for a nice 1:1 aspect ratio
                            .map(|(x, f)| Bar::new(x, f).width(10.0))
                            .collect(),
                    )
                    .color(Color32::LIGHT_BLUE)
                    .name("Normal Distribution");

                    Plot::new("Normal Distribution Demo")
                        .legend(Legend::default())
                        .data_aspect(1.0)
                        .show(ui, |plot_ui| plot_ui.bar_chart(chart));
                });
            });
        }
    }
}

impl RequestUi {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            let Vec2 { x, y } = ui.available_size();
            ui.set_max_width(x/2.0);
            ui.vertical(|ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .id_source("requset_ui_scroller_1")
                    .show(ui, |ui| {
                        let id_source = ui.make_persistent_id("net_test_requset_ui");
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            id_source,
                            false,
                        )
                        .show_header(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("请求头");
                                let add_header = ui.small_button("➕");
                                let del_header = ui.small_button("➖");
                                if add_header.clicked() {
                                    self.headers.inputs.push(SelectKeyValueItem::new());
                                }
                                if del_header.clicked() {
                                    self.headers.inputs = self
                                        .clone()
                                        .headers
                                        .inputs
                                        .into_iter()
                                        .filter(|item| item.selected)
                                        .collect();
                                }
                            });
                        })
                        .body(|ui| {
                            self.headers.ui_grid_input(ui, "request_body_grid_1");
                        });

                        let mut hasher = DefaultHasher::new();
                        self.body.hash(&mut hasher);
                        self.url.hash(&mut hasher);
                        self.method.to_string().hash(&mut hasher);
                        let state_id = ui.id().with(hasher.finish());
                        let (mut show_plaintext, mut template_str) = {
                            let mut data = ui.data();
                            data.get_temp::<(bool, String)>(state_id)
                                .unwrap_or((false, "".to_string()))
                        };
                        ui.horizontal(|ui| {
                            ui.label("请求体：");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                if show_plaintext {
                                    if ui.button("🔃").clicked() {
                                        if show_plaintext {
                                            match rander_template(self.body.as_str()) {
                                                Ok(parsed_temp) => template_str = parsed_temp,
                                                Err(e) => add_notification(ui.ctx(), e.to_string().as_str()),
                                            }
                                        }
                                    }
                                } else {
                                    ui.add_space(29.0);
                                }
                                if ui.toggle_value(&mut show_plaintext, "预览").clicked() {
                                    if show_plaintext {
                                            match rander_template(self.body.as_str()) {
                                                Ok(parsed_temp) => template_str = parsed_temp,
                                                Err(e) =>{
                                                    dbg!(&e);
                                                    let mut msg = "模板语法错误：".to_string();
                                                    msg.push_str(e.to_string().as_str());
                                                    add_notification(ui.ctx(), msg.as_str());
                                                } ,
                                            }
                                    }
                                }
                                if ui.button("格式化JSON").clicked() {
                                    let unfmt_json = self.body.clone();
                                    if let Ok(json) = serde_json::from_str::<Value>(&unfmt_json) {
                                        self.body = serde_json::to_string_pretty(&json)
                                            .unwrap_or(unfmt_json);
                                    }
                                }
                            });
                        });

                        if show_plaintext {
                            super::highlight::code_view_ui(ui, &template_str, "json");
                        } else {
                            super::highlight::code_editor_ui(ui, &mut self.body, "json");
                        }
                        {
                            let mut data = ui.data();
                            data.insert_temp(state_id, (show_plaintext, template_str));
                        }

                        // let req_body_editor = ui.add_sized(
                        //                             ui.available_size(),
                        //                             egui::text_edit::TextEdit::multiline(&mut self.body)
                        //                             .font(egui::TextStyle::Monospace)
                        //                         );
                    })
            });
        });
    }
}

impl ResponseUi {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            egui::ScrollArea::both()
                .id_source("respone_ui_scroller_1")
                .show(ui, |ui| {
                    // ui.set_min_size(ui.available_size());

                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        ui.collapsing("响应头", |ui| {
                            self.headers.ui_grid(ui, "response_grid_ui_1");
                        });
                        // ui.add_sized(
                        // ui.available_size(),
                        super::highlight::code_view_ui(ui, &mut self.body, "json");
                        // egui::text_edit::TextEdit::multiline(&mut self.body)
                        // .desired_rows(24),
                        // );
                    })
                });
        });
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SelectKeyValueItem {
    selected: bool,
    key: String,
    value: String,
}

impl SelectKeyValueItem {
    fn new() -> Self {
        Self {
            selected: true,
            key: "".into(),
            value: "".into(),
        }
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
struct SelectKeyValueInputs {
    inputs: Vec<SelectKeyValueItem>,
}

impl SelectKeyValueInputs {
    pub fn ui_grid_input(&mut self, ui: &mut Ui, id: &str) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(3)
                .min_col_width(20.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(ui.available_size(), egui::widgets::Label::new(""));
                    ui.add_sized(
                        [120., 20.],
                        egui::widgets::Label::new(egui::RichText::new("Key").strong()),
                    );
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("Value").strong()),
                    );
                    ui.end_row();
                    for SelectKeyValueItem {
                        selected,
                        key,
                        value,
                    } in &mut self.inputs
                    {
                        ui.checkbox(selected, "");
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(key),
                        );
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(value),
                        );
                        ui.end_row();
                    }
                });
        });
    }
    pub fn ui_grid(&mut self, ui: &mut Ui, id: &str) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(2)
                .min_col_width(80.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("键").strong()),
                    );
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("值").strong()),
                    );
                    ui.end_row();
                    for SelectKeyValueItem { key, value, .. } in &mut self.inputs {
                        ui.add_sized(ui.available_size(), egui::widgets::Label::new(key.clone()));
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::Label::new(value.clone()),
                        );
                        ui.end_row();
                    }
                });
        });
    }

    // pub fn ui_table(&mut self, ui: &mut Ui) {
    //     TableBuilder::new(ui)
    //         .column(Size::remainder().at_least(100.0))
    //         .column(Size::exact(40.0))
    //         .header(20.0, |mut header| {
    //             header.col(|ui| {
    //                 ui.heading("选择");
    //             });
    //             header.col(|ui| {
    //                 ui.heading("Key");
    //             });
    //             header.col(|ui| {
    //                 ui.heading("Value");
    //             });
    //         })
    //         .body(|mut body| {
    //             body.row(30.0, |mut row| {
    //                 for SelectKeyValueItem {
    //                     selected,
    //                     key,
    //                     value,
    //                 } in &mut self.inputs
    //                 {
    //                     row.col(|ui| {
    //                         ui.checkbox(selected, "");
    //                     });
    //                     row.col(|ui| {
    //                         ui.text_edit_singleline(key);
    //                     });
    //                     row.col(|ui| {
    //                         ui.text_edit_singleline(value);
    //                     });
    //                 }
    //             });
    //         });
    // }
}

pub fn editable_label(ui: &mut egui::Ui, is_edit: &mut bool, value: &mut String) {
    if *is_edit {
        let rsp = ui.text_edit_multiline(value);
        if rsp.lost_focus() || ui.input().key_pressed(Key::Enter) {
            *is_edit = false;
        }
    } else {
        ui.horizontal(|ui| {
            let resp = ui.label(value.clone());
            let rect = resp.rect.expand2(Vec2::new(20., 10.));
            if ui.rect_contains_pointer(rect) {
                let rsp = ui.button("编辑");
                if rsp.clicked() {
                    *is_edit = !*is_edit;
                }
            }
        });
    }
}

// #[cfg(not(target_arch = "wasm32"))]
fn send_request(sender: Sender<(usize,i64, ResponseUi)>, req: RequestUi) {
    thread::Builder::new()
        .name("send_req_thread".to_string())
        .spawn(move || {
            let start = Local::now().timestamp_millis();
            let respui = RT.block_on(async {
                let client = reqwest::Client::new();

                let resp = match client.execute(req.into()).await {
                    Ok(rep) => covert_to_ui(rep).await,
                    Err(err) => ResponseUi {
                        headers: Default::default(),
                        body: err.to_string(),
                        size: 0,
                        code: 500,
                        time: 0,
                    },
                };
                resp
            });
            let end = Local::now().timestamp_millis();
            let now = end - start;
            sender.send((1,now, respui)).unwrap();
        })
        .unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
async fn covert_to_ui(rep: Response) -> ResponseUi {
    let status = rep.status().as_u16();
    let mut headers = SelectKeyValueInputs::default();
    let mut is_json = false;
    for (key, value) in rep.headers().into_iter() {
        let mut item = SelectKeyValueItem::new();
        item.key = key.to_string();
        item.value = match value.to_str() {
            Ok(ok) => ok.to_string(),
            Err(er) => er.to_string(),
        };
        if item.key.eq_ignore_ascii_case("content-type") && item.value.contains("application/json")
        {
            is_json = true;
        }
        headers.inputs.push(item);
    }

    let size = rep.content_length().unwrap_or(0) / 1024;

    let body: String = match rep.text().await {
        Ok(body) => {
            if is_json {
                if let Ok(json) = serde_json::from_str::<Value>(body.as_str()) {
                    serde_json::to_string_pretty(&json).unwrap_or(body)
                } else {
                    body
                }
            } else {
                body
            }
        }
        Err(err) => err.to_string(),
    };

    ResponseUi {
        headers: headers,
        body: body,
        size: size,
        code: status,
        time: 0,
    }
}

async fn send_load_test_request_noclient(sender: Sender<(i64, ResponseUi)>,client:Client, req: Request) {
    let start = Local::now().timestamp_millis();
    match client.execute(req).await {
        Ok(rep) => {
            let end = Local::now().timestamp_millis();
            let respUi = covert_to_ui(rep).await;
            sender.send((end - start, respUi)).unwrap();
        }
        Err(err) => {
            let respUi = ResponseUi {
                headers: Default::default(),
                body: err.to_string(),
                size: 0,
                code: 555,
                time: 0,
            };
            let end = Local::now().timestamp_millis();
            sender.send((end - start, respUi)).unwrap();
        }
    }
}
async fn send_load_test_request(sender: Sender<(usize,i64, ResponseUi)>,client:Client, ireq: (usize,Request)) {

    let start = Local::now().timestamp_millis();
    let (index,req) = ireq;
    match client.execute(req).await {
        Ok(rep) => {
            let end = Local::now().timestamp_millis();
            let respUi = covert_to_ui(rep).await;
            sender.send((index,end - start, respUi)).unwrap();
        }
        Err(err) => {
            let respUi = ResponseUi {
                headers: Default::default(),
                body: err.to_string(),
                size: 0,
                code: 555,
                time: 0,
            };
            let end = Local::now().timestamp_millis();
            sender.send((index,end - start, respUi)).unwrap();
        }
    }
}

fn send_load_test_request_per_sender(sender: Sender<(i64, ResponseUi)>, req: reqwest::blocking::Request) {
    thread::Builder::new().name("load_test_thread".into()).spawn(move ||{
            let client = reqwest::blocking::Client::new();       
            let start = Local::now().timestamp_millis();
            match client.execute(req) {
                Ok(rep) => {
                    let end = Local::now().timestamp_millis();
                    let resp_ui = rep.into();
                    sender.send((end - start, resp_ui)).unwrap();
                }
                Err(err) => {
                    let resp_ui = ResponseUi {
                        headers: Default::default(),
                        body: err.to_string(),
                        size: 0,
                        code: 555,
                        time: 0,
                    };
                    let end = Local::now().timestamp_millis();
                    sender.send((end - start, resp_ui)).unwrap();
                }
            }
    }).unwrap();
}
fn start_load_test(sender: Sender<(usize,i64, ResponseUi)>, times: u16, reqs: u32, req: RequestUi) {
    thread::Builder::new()
        .name("send_req_thread".to_string())
        .spawn(move || {
            let capacity: usize = (times as usize) * (reqs as usize);
            let mut sender_requset: Vec<(usize,Request)> = Vec::with_capacity(capacity);

            let start = std::time::SystemTime::now();
            for i in 0..capacity {
                let body = req.body.clone();
                let rander_body = rander_template(body.as_str()).unwrap_or(body);
                let mut req_clone = req.clone();
                req_clone.body = rander_body;
                sender_requset.push((i,req_clone.into()));
            }
            let duration = start.elapsed().unwrap().as_millis() as u64;
            println!("生成完成：{}-{}",sender_requset.len(),duration);

            let client = reqwest::Client::new();
            let _respui = RT.block_on(async move {
                for _ in 0..times {
                    let start = std::time::SystemTime::now();
                    let mut f_vec = Vec::new();
                    for _ in 0..reqs {
                        let req = sender_requset.pop().unwrap();
                        let f = send_load_test_request(sender.clone(),client.clone(), req);
                        f_vec.push(f);
                    }
                    let _result = stream::iter(f_vec)
                                            .buffer_unordered(capacity)
                                            .collect::<Vec<_>>().await;
                    let duration = start.elapsed().unwrap().as_millis() as u64;
                    println!("reqs:{},duration:{}",reqs,duration);
                    let duration = start.elapsed().unwrap().as_millis() as u64;
                    if duration<1000 {
                        tokio::time::sleep(Duration::from_millis(1000-duration)).await;
                    }
                }
                //发送一个完成的数据
                sender.send((0,-1, ResponseUi::default()))
            });
        })
        .unwrap();
}

fn start_load_test_thread(sender: Sender<(i64, ResponseUi)>,times: u16,reqs: u32,req: RequestUi) {
    let capacity: usize = (times as usize) * (reqs as usize);
    let mut sender_requset: Vec<reqwest::blocking::Request> = Vec::with_capacity(capacity);

    let start = std::time::SystemTime::now();
    for _i in 0..capacity {
        let body = req.body.clone();
        let rander_body = rander_template(body.as_str()).unwrap_or(body);
        let mut req_clone = req.clone();
        req_clone.body = rander_body;
        sender_requset.push(req_clone.into());
    }
    for _ in 0..times {
        for _ in 0..reqs {
            let req = sender_requset.pop().unwrap();
            let f = send_load_test_request_per_sender(sender.clone(), req);
        }
        let duration = start.elapsed().unwrap().as_millis() as u64;
        if duration<1000 {
            thread::sleep(Duration::from_millis(1000-duration));
        }
    }
    //发送一个完成的数据
    sender.send((-1, ResponseUi::default()));
}

fn start_load_test_multisender(sender:Sender<(usize,i64, ResponseUi)>, times: u16, reqs: u32, req: RequestUi) {
    thread::Builder::new()
        .name("send_req_thread".to_string())
        .spawn(move || {
            let capacity: usize = (times as usize) * (reqs as usize);
            let mut sender_requsets: Vec<(usize,Request)> = Vec::with_capacity(capacity);
            for i in 0..capacity {
                let body = req.body.clone();
                let rander_body = rander_template(body.as_str()).unwrap_or(body);
                let mut req_clone = req.clone();
                req_clone.body = rander_body;
                sender_requsets.push((i ,req_clone.into()));
            }
            sender_requsets.reverse();
            let client = reqwest::Client::new();
            let _respui = RT.block_on(async move {
                for _ in 0..times {
                    // let client = reqwest::Client::new();
                    let start = std::time::SystemTime::now();
                    let mut f_vec = Vec::new();
                    for _ in 0..reqs {
                        let req = sender_requsets.pop().unwrap();
                        let f = send_load_test_request(sender.clone(),client.clone(),req);
                        // let _tf = tokio::task::spawn(f);
                        f_vec.push(f);
                    }
                    // println!("生成完成：{}-{}",f_vec.len(),duration);
                    // let result = stream::iter(f_vec)
                    //                         .buffer_unordered(16)
                    //                         .collect::<Vec<_>>().await;
                    let _tf = tokio::spawn(
                        stream::iter(f_vec)
                                                .buffered(reqs as usize)
                                                .collect::<Vec<_>>()
                    );
                    let duration = start.elapsed().unwrap().as_millis() as u64;
                    if duration<1000 {
                        tokio::time::sleep(Duration::from_millis(1000-duration)).await;
                    }
                }
                //发送一个完成的数据
                sender.send((0,-1, ResponseUi::default()))
            });
        })
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn send_request(sender: Sender<(i64, ResponseUi)>, req: RequestUi) {
    let start = Local::now().timestamp_millis();
    let ehttp_req = ehttp::Request::get(req.url);
    crate::log("开始");
    ehttp::fetch(ehttp_req, move |res| {
        let resp = match res {
            Ok(ref resp) => {
                crate::log("成功");
                let resp: ResponseUi = res.unwrap().into();
                resp
            }
            Err(err) => {
                crate::log("失败");
                ResponseUi::default()
            }
        };
        let end = Local::now().timestamp_millis();
        let now = end - start;
        sender.send((now, resp));
    });
}
