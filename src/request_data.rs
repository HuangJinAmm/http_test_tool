use std::{collections::HashMap, fmt::Debug, str::FromStr};

use crate::{component::header_ui::SelectKeyValueItem, utils::template::rander_template};
use hdrhistogram::Histogram;
use minijinja::value::Value as JValue;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ScriptData {
    pub pre: String,
    pub after: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RequestData {
    pub remark: String,
    pub url: String,
    pub method: Method,
    pub headers: Vec<SelectKeyValueItem>,
    pub body: String,
}
impl Default for RequestData {
    fn default() -> Self {
        Self {
            remark: "备注".to_owned(),
            url: Default::default(),
            method: Method::GET,
            headers: Default::default(),
            body: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ResponseData {
    pub headers: Vec<SelectKeyValueItem>,
    pub body: String,
    pub size: u64,
    pub code: String,
    pub time: i64,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct LoadTestData {
    pub reqs: u32,
    pub round: u32,
    pub process: f32,
    #[serde(skip)]
    pub result: LoadTestResult,
    pub result_list: Vec<i64>,
}

impl LoadTestData {
    #[inline]
    pub fn total(&self) -> u64 {
        (self.reqs as u64) * (self.round as u64)
    }

    pub fn update_process(&mut self) {
        self.process = (self.result_list.len() as f32) / (self.total() as f32);
    }

    pub fn recode_time(&mut self, time: i64) {
        self.result
            .result_hist
            .as_mut()
            .unwrap()
            .record(time as u64)
            .unwrap();
    }

    pub fn add_result(&mut self, index: usize, time: i64) {
        self.result_list.get_mut(index).map(|r| *r = time);
    }
}

#[derive(Clone)]
// #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoadTestResult {
    // total: u16,
    // average: f32,
    // median: f32,
    // min: u16,
    // max: u16,
    // line90: f32,
    // line95: f32,
    // line99: f32,
    pub error: f32,
    pub recived: f32,
    pub send: f32,
    // #[serde(skip)]
    pub result_hist: Option<Histogram<u64>>,
}

impl Debug for LoadTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadTestResult")
            .field("error", &self.error)
            .field("recived", &self.recived)
            .field("send", &self.send)
            .finish()
    }
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

#[cfg(not(target_arch = "wasm32"))]
impl TryInto<Request> for RequestData {
    type Error = String;

    fn try_into(self) -> Result<Request, Self::Error> {
        let mth_bytes = self.method.to_string();
        let mth = reqwest::Method::from_bytes(mth_bytes.as_bytes()).expect("请求方法解析错误");
        let url = reqwest::Url::parse(self.url.as_str()).map_err(|e| e.to_string())?;

        let headers = self.headers.into_iter().filter(|slk| slk.selected).fold(
            HeaderMap::new(),
            |mut headmap, slk| {
                let k = HeaderName::from_str(slk.key.as_str()).unwrap();
                let v: HeaderValue;
                if slk.value.contains("{{") && slk.value.contains("}}") {
                    let parsed_temp =
                        rander_template(&slk.value).unwrap_or_else(|_| slk.value.clone());
                    v = HeaderValue::from_str(&parsed_temp)
                        .unwrap_or_else(|_| HeaderValue::from_str("非法的header值").unwrap());
                } else {
                    v = HeaderValue::from_str(slk.value.as_str())
                        .unwrap_or_else(|_| HeaderValue::from_str("非法的header值").unwrap());
                }
                headmap.append(k, v);
                headmap
            },
        );
        let mut req = Request::new(mth, url);
        *req.headers_mut() = headers;
        if !self.body.is_empty() {
            let deal_temp = match rander_template(&self.body) {
                Ok(parsed_temp) => parsed_temp,
                Err(e) => {
                    let mut msg = "模板语法错误：".to_string();
                    msg.push_str(e.to_string().as_str());
                    self.body.clone()
                }
            };
            let paser_body = match json5::from_str::<Value>(&deal_temp) {
                Ok(json_body) => {
                    serde_json::to_string_pretty(&json_body).unwrap_or(self.body.clone())
                }
                Err(_) => self.body.clone(),
            };
            *req.body_mut() = Some(paser_body.into());
        }
        Ok(req)
    }
}

impl Into<PreRequest> for &RequestData {
    fn into(self) -> PreRequest {
        let mth_bytes = self.method.to_string();
        let url = self.url.clone();

        let headers = self.headers.iter().filter(|slk| slk.selected).fold(
            HashMap::new(),
            |mut headmap, slk| {
                let k = slk.key.clone();
                let v = slk.value.clone();
                headmap.insert(k, v);
                headmap
            },
        );
        let mut parse_querys = url.as_str();
        let mut querys_map: HashMap<String, String> = HashMap::new();
        if let Some(q) = parse_querys.find('?') {
            if q + 1 < parse_querys.len() {
                parse_querys = &parse_querys[q + 1..];
                loop {
                    let querys;
                    if parse_querys.is_empty() {
                        break;
                    }
                    if let Some(g) = parse_querys.find('&') {
                        querys = &parse_querys[..g];
                        parse_querys = &parse_querys[g + 1..];
                    } else {
                        querys = parse_querys;
                        parse_querys = "";
                    }
                    if !querys.ends_with('=') {
                        if let Some(eq_p) = querys.find('=') {
                            let key = &querys[..eq_p];
                            let value = &querys[eq_p + 1..];
                            querys_map.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        let body: JValue;
        if let Ok(json_value) = serde_json::from_str::<JValue>(&self.body) {
            body = json_value;
        } else {
            body = JValue::from_serializable(&self.body);
        }
        PreRequest {
            method: mth_bytes,
            querys: querys_map,
            headers,
            body,
            url,
        }
    }
}

impl Into<PreResponse> for &ResponseData {
    fn into(self) -> PreResponse {
        let headers = self.headers.iter().filter(|slk| slk.selected).fold(
            HashMap::new(),
            |mut headmap, slk| {
                let k = slk.key.clone();
                let v = slk.value.clone();
                headmap.insert(k, v);
                headmap
            },
        );
        let code = self.code.to_string();
        let body: JValue;
        if let Ok(json_value) = serde_json::from_str::<JValue>(&self.body) {
            body = json_value;
        } else {
            body = JValue::from_serializable(&self.body);
        }
        PreResponse {
            headers,
            body,
            code,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn covert_to_ui(value: Response) -> ResponseData {
    let code = value.status().to_string();
    let mut headers: Vec<SelectKeyValueItem> = Vec::new();
    let mut is_json = false;
    for (key, value) in value.headers().into_iter() {
        let mut item = SelectKeyValueItem::new("", "");
        item.key = key.to_string();
        item.value = match value.to_str() {
            Ok(ok) => ok.to_string(),
            Err(er) => er.to_string(),
        };
        if item.key.eq_ignore_ascii_case("content-type") && item.value.contains("application/json")
        {
            is_json = true;
        }
        headers.push(item);
    }

    let size = value.content_length().unwrap_or(0);

    let body: String = match value.text().await {
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

    ResponseData {
        headers,
        body,
        size,
        code,
        time: 0,
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PreHttpTest {
    pub req: PreRequest,
    pub resp: PreResponse,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PreRequest {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub querys: HashMap<String, String>,
    pub body: JValue,
    pub url: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PreResponse {
    pub headers: HashMap<String, String>,
    pub body: JValue,
    pub code: String,
}
