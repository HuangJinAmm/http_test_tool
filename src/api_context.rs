use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;

use crate::app::TOASTS;
use crate::component::tree_ui::{TreeUi, self};
use crate::ui::request_ui::{CollectionUi, LoadTestDiagram, LoadTestUi, ScriptUi};
use crate::utils::rhai_script::SCRIPT_ENGINE;
use crate::utils::template::{add_global_var, TMP_SCOPE_CTX};
use crate::{
    request_data::{LoadTestData, RequestData, ResponseData, ScriptData},
    ui::request_ui::{RequestUi, ResponseUi},
};
use egui::WidgetText;
use egui_dock::TabViewer;
use egui_notify::Toasts;
use log::info;
use minijinja::value::Value;
use rhai::{EvalAltResult, Scope};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ApiContext {
    pub selected: Vec<u64>,
    pub tests: BTreeMap<u64, ApiTester>,
    pub collections: BTreeMap<u64, CollectionsData>,
    tree_ui: TreeUi,
    #[serde(skip)]
    req_ui: RequestUi,
    #[serde(skip)]
    col_ui: CollectionUi,
    #[serde(skip)]
    script_ui: ScriptUi,
}
impl TabViewer for ApiContext {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let toast = TOASTS.get_or_init(|| {
            Arc::new(Mutex::new(
                Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            ))
        });
        let selected = *self.selected.first().unwrap_or(&0);
        match tab.as_str() {
            "请求" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.req_ui.ui(ui, &mut req_data.req, selected);
                }
            }
            "响应" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    ResponseUi::ui(ui, &mut req_data.resp, selected);
                } else {
                    ui.label("没有数据");
                }
            }
            "设置" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    LoadTestUi::ui(ui, &mut req_data.load_test, selected)
                }
            }
            "图表" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    LoadTestDiagram::ui(ui, &req_data.load_test)
                }
            }
            "文档" => {
                if let Some(collect_data) = self.collections.get_mut(&selected) {
                    self.col_ui.ui(ui, &mut collect_data.doc, selected);
                }
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.col_ui.ui(ui, &mut req_data.req.remark, selected);
                }
            }
            "后置脚本" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.script_ui.ui(ui, &mut req_data.script.after, selected);
                }
            }
            "前置脚本" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.script_ui.ui(ui, &mut req_data.script.pre, selected);
                }
                if let Some(collect_data) = self.collections.get_mut(&selected) {
                    self.script_ui.ui(ui, &mut collect_data.script, selected);
                }
            }
            "导航" => {
                egui::ScrollArea::both().show(ui, |ui| {
                    // ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui|{
                    match self.tree_ui.ui_impl(ui) {
                        tree_ui::Action::Keep => {
                            //ignore
                        }
                        tree_ui::Action::Delete(dels) => {
                            for del_id in dels {
                                info!("删除{}", del_id);
                                self.delete_collecton(del_id);
                                self.delete_test(del_id);
                            }
                        }
                        tree_ui::Action::Add((adds, node_type)) => {
                            let add_id = adds.first().unwrap().to_owned();
                            info!("添加{},{:?}", &add_id, &node_type);
                            match node_type {
                                tree_ui::NodeType::Collection => {
                                    self.insert_collecton(add_id, CollectionsData::default());
                                }
                                tree_ui::NodeType::Node => {
                                    self.insert_test(add_id, ApiTester::default());
                                }
                            }
                        }
                        tree_ui::Action::Rename(_adds) => {
                            //基本上不用处理
                            info!("重命名")
                        }
                        tree_ui::Action::Selected((selected_id, selected_title)) => {
                            let selected = *selected_id.first().unwrap_or(&0);
                            self.selected = selected_id;
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
                                if let Some(copyed) = self.tests.get(&sid) {
                                    let parse = copyed.clone();
                                    self.insert_test(did, parse);
                                }
                            }
                        }
                    }
                    ui.add_space(ui.available_height());
                });
                //    });
            }
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        // if let Some(index )= self.tree.find_tab(tab) {
        //     self.tree.remove_tab(index);
        // }
        true
    }

    // fn context_menu(
    //     &mut self,
    //     ui: &mut egui::Ui,
    //     tab: &mut Self::Tab,
    //     surface: egui_dock::SurfaceIndex,
    //     node: egui_dock::NodeIndex,
    // ) {
        
    // }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(self.title(tab).text())
    }

    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    fn on_add(&mut self, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {}

    fn add_popup(&mut self, _ui: &mut egui::Ui, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {}

    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn tab_style_override(&self, _tab: &Self::Tab, _global_style: &egui_dock::TabStyle) -> Option<egui_dock::TabStyle> {
        None
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }

    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }
}

impl ApiContext {
    pub fn new() -> Self {
        Self {
            tests: BTreeMap::new(),
            tree_ui: TreeUi::new(),
            collections: BTreeMap::new(),
            req_ui: RequestUi::default(),
            selected: vec![0],
            col_ui: CollectionUi::default(),
            script_ui: ScriptUi::default(),
        }
    }

    pub fn run_script(&self) -> Result<(), Box<EvalAltResult>> {
        let mut parents = self.selected.clone();
        let script_scope = &mut Scope::new();
        let cid = parents.remove(0);
        while let Some(pid) = parents.pop() {
            if let Some(pdata) = self.collections.get(&pid) {
                let _res = SCRIPT_ENGINE.run_with_scope(script_scope, &pdata.script)?;
            }
        }

        if let Some(aip) = self.tests.get(&cid) {
            SCRIPT_ENGINE.run_with_scope(script_scope, &aip.script.pre)?;
        }
        dbg!(&script_scope);
        let mut script_ctx: HashMap<String, Value> = HashMap::new();

        for (name, _is_constant, value) in script_scope.iter() {
            // let temp_value = serde_json::to_string(&value).unwrap();
            if value.is_array()
                || value.is_bool()
                || value.is_char()
                || value.is_decimal()
                || value.is_float()
                || value.is_int()
                || value.is_map()
                || value.is_string()
            {
                let t = Value::from_serializable(&value);
                script_ctx.insert(name.to_owned(), t);
                // add_global_var(name.to_owned(), t);
            }
        }

        let _ = TMP_SCOPE_CTX.write().and_then(|mut tmp| {
            *tmp = Value::from(script_ctx);
            Ok(())
        });
        Ok(())
    }

    pub fn insert_collecton(
        &mut self,
        key: u64,
        value: CollectionsData,
    ) -> Option<CollectionsData> {
        self.collections.insert(key, value)
    }
    pub fn insert_test(&mut self, key: u64, value: ApiTester) -> Option<ApiTester> {
        self.tests.insert(key, value)
    }

    pub fn delete_test(&mut self, key: u64) -> Option<ApiTester> {
        self.tests.remove(&key)
    }

    pub fn delete_collecton(&mut self, key: u64) -> Option<CollectionsData> {
        self.collections.remove(&key)
    }

    pub fn get_mut_collection(&mut self, key: u64) -> Option<&mut CollectionsData> {
        self.collections.get_mut(&key)
    }
    pub fn get_mut_test(&mut self, key: u64) -> Option<&mut ApiTester> {
        self.tests.get_mut(&key)
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ApiTester {
    pub script: ScriptData,
    pub req: RequestData,
    pub resp: ResponseData,
    #[serde(skip)]
    pub load_test: LoadTestData,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct CollectionsData {
    pub doc: String,
    pub script: String,
}
