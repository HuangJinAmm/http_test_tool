use std::collections::{BTreeMap, HashMap};

use crate::ui::request_ui::{CollectionUi, LoadTestDiagram, LoadTestUi, ScriptUi};
use crate::utils::rhai_script::SCRIPT_ENGINE;
use crate::utils::template::{add_global_var, TMP_SCOPE_CTX};
use crate::{
    request_data::{LoadTestData, RequestData, ResponseData, ScriptData},
    ui::request_ui::{RequestUi, ResponseUi},
};
use egui::WidgetText;
use egui_dock::TabViewer;
use minijinja::value::Value;
use rhai::Scope;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ApiContext {
    pub selected: Vec<u64>,
    pub tests: BTreeMap<u64, ApiTester>,
    pub collections: BTreeMap<u64, CollectionsData>,
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
        let selected = *self.selected.first().unwrap_or(&0);
        match tab.as_str() {
            "请求" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.req_ui.ui(ui, &mut req_data.req, selected);
                }
                if let Some(collect_data) = self.collections.get_mut(&selected) {
                    self.col_ui.ui(ui, collect_data, selected);
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
            "记录" => {}
            "脚本" => {
                if let Some(req_data) = self.tests.get_mut(&selected) {
                    self.script_ui.ui(ui, &mut req_data.script, selected);
                }
            }
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(&mut self, _ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            _ => {}
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
}

impl ApiContext {
    pub fn new() -> Self {
        Self {
            tests: BTreeMap::new(),
            collections: BTreeMap::new(),
            req_ui: RequestUi::default(),
            selected: vec![0],
            col_ui: CollectionUi::default(),
            script_ui: ScriptUi::default(),
        }
    }


    pub fn run_after_script(&self,resp:&str,script_scope:&mut Scope) {
        SCRIPT_ENGINE.run_with_scope(script_scope, resp);
    }

    pub fn run_script(&self) {
        let mut parents = self.selected.clone();
        let script_scope = &mut Scope::new();
        let cid = parents.remove(0);
        while let Some(pid) = parents.pop() {
            if let Some(pdata) = self.collections.get(&pid) {
                SCRIPT_ENGINE.run_with_scope(script_scope, &pdata.script);
            }
        }

        if let Some(aip) = self.tests.get(&cid) {
            SCRIPT_ENGINE.run_with_scope(script_scope, &aip.script.pre);
        }
        let mut script_ctx:HashMap<String, Value> = HashMap::new();

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

        let _ = TMP_SCOPE_CTX.write().and_then(|mut tmp|{
            *tmp = Value::from(script_ctx);
            Ok(())
        });
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
    pub remark: String,
    pub doc: String,
    pub script: String,
}
