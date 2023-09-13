use crate::app::REQ_UI_ID;
use crate::app::TASK_CHANNEL;
use crate::app::TOASTS;
use crate::app::TOKIO_RT;
use crate::component::code_editor::TextEdit;
use crate::component::header_ui::HeaderUi;
use crate::component::header_ui::SelectKeyValueItem;
use crate::component::syntax_highlight::CodeTheme;
use crate::component::syntax_highlight::code_view_ui;
use crate::component::syntax_highlight::highlight;
use crate::component::syntax_highlight::highlight_temp_key;
use crate::request_data::LoadTestData;
use crate::request_data::Method;
use crate::request_data::ResponseData;
use crate::request_data::ScriptData;
use crate::utils::template::rander_template;
use crate::{
    api_context::CollectionsData, component::editeable_label::editable_label,
    request_data::RequestData,
};
use egui::plot::Bar;
use egui::plot::BarChart;
use egui::plot::Legend;
use egui::plot::Line;
use egui::plot::Plot;
use egui::Color32;
use egui::RichText;
use egui_commonmark::CommonMarkCache;
use egui_commonmark::CommonMarkViewer;
use serde_json::Value;
pub struct RequestUi {
    pub editor: TextEdit,
}

impl Default for RequestUi {
    fn default() -> Self {
        Self { editor: TextEdit::new_template() }
    }
}

impl RequestUi {

    pub fn ui(&mut self, ui: &mut egui::Ui, request_data: &mut RequestData, id: u64) {
        let RequestData {
            remark,
            url,
            method,
            headers,
            body,
        } = request_data;
        let ui_id = REQ_UI_ID.get_or_init(|| ui.id());
        let req_id = ui_id.with(id);
        let mut send_state = ui.data_mut(|d| d.get_temp::<bool>(req_id).unwrap_or(false));

        ui.vertical(|ui| {
            ui.add(editable_label(remark));
            ui.horizontal(|ui| {
                let send = ui
                    .add_enabled_ui(!send_state, |ui| {
                        let send = ui.button("发送💨");
                        send
                    })
                    .inner;
                egui::ComboBox::from_label("🌐")
                    .selected_text(format!("{:?}", method))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(method, Method::GET, "GET");
                        ui.selectable_value(method, Method::POST, "POST");
                        ui.selectable_value(method, Method::PUT, "PUT");
                        ui.selectable_value(method, Method::DELETE, "DELETE");
                        ui.selectable_value(method, Method::PATCH, "PATCH");
                        ui.selectable_value(method, Method::OPTIONS, "OPTIONS");
                    });

                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let layout_job = highlight_temp_key(ui.ctx(), string);
                    // layout_job.wrap.max_width = wrap_width; // no wrapping
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::TextEdit::singleline(url)
                    .desired_width(ui.available_width() - 24.0)
                    .hint_text("请求路径").layouter(&mut layouter)
                    .show(ui);
                if send.clicked() {
                    send_state = true;
                    let task_sender = unsafe { TASK_CHANNEL.0.clone() };
                    TOKIO_RT.spawn(async move {
                        if let Err(_) = task_sender.send((id, 1, 1)).await {
                            log::info!("receiver dropped");
                            return;
                        }
                    });
                }
                if send_state {
                    ui.spinner();
                }
            });

            egui::ScrollArea::both()
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
                                headers.push(SelectKeyValueItem::new("", ""));
                            }
                            if del_header.clicked() {
                                let new_headers: Vec<SelectKeyValueItem> = headers
                                    .clone()
                                    .into_iter()
                                    .filter(|item| item.selected)
                                    .collect();
                                *headers = new_headers;
                            }
                        });
                    })
                    .body(|ui| {
                        HeaderUi::ui_grid_input(ui, "request_body_grid_1", headers);
                    });

                    let state_id = ui.id().with(id.to_string() + "body");
                    let (mut show_plaintext, mut template_str) = ui.data(|d| {
                        d.get_temp::<(bool, String)>(state_id)
                            .unwrap_or((false, "".to_string()))
                    });
                    ui.horizontal(|ui| {
                        ui.label("请求体：");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if show_plaintext {
                                if ui.button("🔃").clicked() {
                                    if show_plaintext {
                                        match rander_template(body.as_str()) {
                                            Ok(parsed_temp) => template_str = parsed_temp,
                                            Err(e) => {
                                                if let Ok(mut toast_w) =
                                                    TOASTS.get().unwrap().lock()
                                                {
                                                    toast_w.error(e.to_string().as_str());
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                ui.add_space(29.0);
                            }
                            if ui.toggle_value(&mut show_plaintext, "预览").clicked() {

                                if show_plaintext {

                                    let task_sender = unsafe { TASK_CHANNEL.0.clone() };
                                    TOKIO_RT.spawn(async move {
                                        if let Err(_) = task_sender.send((id, 0, 0)).await {
                                            log::info!("receiver dropped");
                                            return;
                                        }
                                    });

                                    let deal_temp = match rander_template(&body) {
                                        Ok(parsed_temp) => parsed_temp,
                                        Err(e) => {
                                            let mut msg = "模板语法错误：".to_string();
                                            msg.push_str(e.to_string().as_str());
                                            if let Ok(mut toast_w) = TOASTS.get().unwrap().lock() {
                                                toast_w.error(e.to_string().as_str());
                                            }
                                            body.clone()
                                        }
                                    };
                                    template_str = match json5::from_str::<Value>(&deal_temp) {
                                        Ok(json_body) => serde_json::to_string_pretty(&json_body)
                                            .unwrap_or(body.clone()),
                                        Err(_) => body.clone(),
                                    };
                                }
                            }
                            if ui.button("格式化JSON").clicked() {
                                let unfmt_json = body.clone();

                                let f = json5format::Json5Format::new().unwrap();
                                match json5format::ParsedDocument::from_str(&unfmt_json, None) {
                                    Ok(d) => match f.to_string(&d) {
                                        Ok(s) => {
                                            *body = s;
                                        }
                                        Err(se) => {
                                            if let Ok(mut toast_w) = TOASTS.get().unwrap().lock() {
                                                toast_w.error(se.to_string().as_str());
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        if let Ok(mut toast_w) = TOASTS.get().unwrap().lock() {
                                            toast_w.error(e.to_string().as_str());
                                        }
                                    }
                                }
                            }
                        });
                    });

                    if show_plaintext {
                        code_view_ui(ui, &template_str, "json");
                    } else {
                        self.editor.ui(ui, body, id);
                    }
                    ui.data_mut(|data| data.insert_temp(state_id, (show_plaintext, template_str)));

                    // let req_body_editor = ui.add_sized(
                    //                             ui.available_size(),
                    //                             egui::text_edit::TextEdit::multiline(&mut self.body)
                    //                             .font(egui::TextStyle::Monospace)
                    //                         );
                })
        });
        ui.data_mut(|d| d.insert_temp(req_id, send_state));
    }
}

pub struct CollectionUi {
    script_editor: TextEdit,
    md_editor: TextEdit,
    cache: CommonMarkCache,
}

impl Default for CollectionUi {
    fn default() -> Self {
        Self { script_editor: TextEdit::new_rhai(), md_editor: TextEdit::new_md(), cache: Default::default() }
    }
}

impl CollectionUi {
    pub fn ui(&mut self, ui: &mut egui::Ui, data: &mut CollectionsData, id: u64) {
        let CollectionsData {
            remark,
            script,
            doc,
        } = data;
        let req_id = ui.id().with(id);
        let mut preview_state = ui.data_mut(|d| d.get_temp::<bool>(req_id).unwrap_or(false));
        ui.vertical(|ui| {
            ui.add(editable_label(remark));
            ui.collapsing("脚本", |ui| {
                self.script_editor.ui(ui, script, id);
            });

            let id_source = ui.make_persistent_id("net_test_collection_ui");
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id_source,
                false,
            )
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("文档");
                    let add_header = ui.small_button("预览");
                    if add_header.clicked() {
                        preview_state = !preview_state;
                    }
                });
            })
            .body(|ui| {
                if preview_state {
                    CommonMarkViewer::new("viewer").show(ui, &mut self.cache, &doc);
                } else {
                    self.md_editor.ui(ui, doc, id);
                }
            });
        });
        ui.data_mut(|d| d.insert_temp(req_id, preview_state));
    }
}

pub struct ScriptUi {
    pre_script_editor: TextEdit,
}

impl Default for ScriptUi {
    fn default() -> Self {
        ScriptUi {
            pre_script_editor: TextEdit::new_rhai()
        }
    }
}

impl ScriptUi {
    pub fn ui(&mut self, ui: &mut egui::Ui, data: &mut ScriptData, id: u64) {
        let ScriptData { pre, after } = data;
        ui.vertical(|ui| {
            ui.collapsing("前置脚本", |ui| {
                self.pre_script_editor.ui(ui, pre, id);
            });
            ui.collapsing("后置脚本", |ui| {
                self.pre_script_editor.ui(ui, after, id);
            });
        });
    }
}

pub struct ResponseUi {}

impl ResponseUi {
    pub fn ui(ui: &mut egui::Ui, data: &mut ResponseData, _id: u64) {
        let ResponseData {
            headers,
            body,
            size,
            code,
            time,
        } = data;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("响应状态码：");
                let code_rich_text = match code.parse::<usize>() {
                    Err(e) => RichText::new(""),
                    Ok(x) if x >= 100 && x < 200 => {
                        RichText::new(x.to_string()).color(Color32::YELLOW)
                    }
                    Ok(x) if x >= 200 && x < 400 => {
                        RichText::new(x.to_string()).color(Color32::GREEN)
                    }
                    Ok(x) if x >= 400 && x < 600 => {
                        RichText::new(x.to_string()).color(Color32::RED)
                    }
                    Ok(x) => RichText::new(x.to_string()),
                };
                ui.label(code_rich_text);
                ui.label("响应时间：");
                ui.label(time.to_string());
                ui.label("毫秒");
                ui.label("响应大小：");
                ui.label(size.to_string());
                ui.label("B");
            });
            ui.group(|ui| {
                egui::ScrollArea::both()
                    .id_source("respone_ui_scroller_1")
                    .show(ui, |ui| {
                        // ui.set_min_size(ui.available_size());

                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.collapsing("响应头", |ui| {
                                HeaderUi::ui_grid(ui, "response_grid_ui_1", headers);
                            });
                            // ui.add_sized(
                            // ui.available_size(),
                            code_view_ui(ui, body, "json");
                            // egui::text_edit::TextEdit::multiline(&mut self.body)
                            // .desired_rows(24),
                            // );
                        })
                    });
            });
        });
    }
}

pub struct LoadTestUi {}

impl LoadTestUi {
    pub fn ui(ui: &mut egui::Ui, data: &mut LoadTestData, id: u64) {
        let ui_id = REQ_UI_ID.get_or_init(|| ui.id());
        let req_id = ui_id.with(id);
        let mut send_state = ui.data_mut(|d| d.get_temp::<bool>(req_id).unwrap_or(false));
        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled_ui(!send_state, |ui| ui.button("开始"))
                        .inner
                        .clicked()
                    {
                        send_state = true;
                        let reqs = data.reqs;
                        let round = data.round;
                        data.result_list = vec![0].repeat((reqs * round) as usize);
                        data.process = 0.0;
                        let task_sender = unsafe { TASK_CHANNEL.0.clone() };
                        TOKIO_RT.spawn(async move {
                            if let Err(_) = task_sender.send((id, reqs, round)).await {
                                log::info!("receiver dropped");
                                return;
                            }
                        });
                    }
                    ui.label("并发数(req/s):");
                    ui.add(egui::DragValue::new(&mut data.reqs).speed(1));
                    ui.label("循环轮数:");
                    ui.add(egui::DragValue::new(&mut data.round).speed(1));
                    if send_state {
                        ui.spinner();
                    }
                    ui.add(egui::ProgressBar::new(data.process));
                });
            });

            let result = data.result.result_hist.as_ref().unwrap();
            egui::Grid::new("id_loadtestreult")
                .num_columns(3)
                .min_col_width(80.)
                .show(ui, |ui| {
                    ui.label("总共请求数量：");
                    let show_text = RichText::new(result.len().to_string()).color(Color32::GOLD);
                    ui.label(show_text);
                    ui.label("错误数：");
                    let show_text =
                        RichText::new(data.result.error.to_string()).color(Color32::RED);
                    ui.label(show_text);

                    ui.label("错误数占比：");
                    let percent;
                    if result.len() == 0 {
                        percent = 0.00;
                    } else {
                        percent = 100.0 * data.result.error / (result.len() as f32);
                    }
                    let show_text = RichText::new(percent.to_string() + "%").color(Color32::RED);
                    ui.label(show_text);
                    ui.end_row();
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

                    ui.end_row();

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

                    ui.end_row();

                    ui.label("发送(KB)：");
                    let show_text =
                        RichText::new(data.result.send.to_string()).color(Color32::BLUE);
                    ui.label(show_text);

                    ui.label("接收(KB)");
                    let show_text =
                        RichText::new(data.result.recived.to_string()).color(Color32::BLUE);
                    ui.label(show_text);
                });
        });
        ui.data_mut(|d| d.insert_temp(req_id, send_state));
    }
}

pub struct LoadTestDiagram {}

impl LoadTestDiagram {
    pub fn ui(ui: &mut egui::Ui, loaddata: &LoadTestData) {
        let data = &loaddata.result_list;
        ui.vertical(|ui| {
            //运行图表
            ui.group(|ui| {
                // let all = ui.available_width();
                // ui.set_max_width(all/2.0);
                let all_h = ui.available_height();
                ui.set_max_height(all_h / 2.0);
                let sin = data.iter().enumerate().map(|(x, y)| [x as f64, *y as f64]);
                let line = Line::new(egui::plot::PlotPoints::from_iter(sin));
                Plot::new("runing")
                    .data_aspect(1.0)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
            //运行结果图表
            let hdrhist = loaddata.result.result_hist.clone().unwrap();
            ui.group(|ui| {
                let max_hdr = hdrhist.max();
                let min_hdr = hdrhist.min();
                let chart = BarChart::new(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map, Number, Value};

    #[test]
    fn test_json5() {
        let v =
            json5::to_string(&json!({"a": [null, true, 42, 42.42, f64::NAN, "hello"]})).unwrap();
        println!("{v}");
    }

    #[test]
    fn test_json5_str_format() {
        let config = r#" {// A traditional message.  
            message: 'hello world', 
            // A number for some reason.
            n: 42, } "#;

        println!("{config}");
        println!("===================");
        let f = json5format::Json5Format::new().unwrap();
        let d = json5format::ParsedDocument::from_str(config, None).unwrap();
        let s = f.to_string(&d).unwrap();
        println!("{s}");
    }
    #[test]
    fn test_json5_str() {
        let config = "
        {
            // A traditional message.
            message: 'hello world',

            // A number for some reason.
            n: 42,
            }
        ";

        let v = json5::from_str::<Value>(&config).unwrap();
        println!("{v}");
    }
}
