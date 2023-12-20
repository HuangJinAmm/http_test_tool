use std::f64::consts::E;

use once_cell::sync::Lazy;
use rhai::export_module;
use rhai::module_resolvers::FileModuleResolver;
use rhai::module_resolvers::StaticModuleResolver;
use rhai::plugin::*;
use rhai::Engine;
// use rhai_rand::RandomPackage;
// use rhai_sci::SciPackage;

use crate::app::{TASK_CHANNEL, TOKIO_RT};

pub const SCRIPT_ENGINE: Lazy<Engine> = Lazy::new(|| {
    let mut engine = Engine::new();
    // let mut smr = StaticModuleResolver::new();

    // Create new 'RandomPackage' instance
    // let random = RandomPackage::new();

    // Load the package into the `Engine`
    // random.register_into_engine(&mut engine);

    // Create new 'SciPackage' instance
    // let sci = SciPackage::new();

    // Load the package into the [`Engine`]
    // sci.register_into_engine(&mut engine);

    // Create new 'FilesystemPackage' instance
    // let random = FilesystemPackage::new();

    let fmr = FileModuleResolver::new_with_path("./rhai");

    engine.set_module_resolver(fmr);
    // Load the package into the `Engine`
    // random.register_into_engine(&mut engine);
    let log = exported_module!(slog);
    // engine.register_global_module(log.into());
    engine.register_static_module("log", log.into());

    let base64 = exported_module!(sbase64);
    engine.register_static_module("base64", base64.into());

    let fake = exported_module!(sfaker);
    engine.register_static_module("faker", fake.into());

    let crypto = exported_module!(crypto);
    engine.register_static_module("crypto", crypto.into());

    engine.register_fn("call_req", call_req);
    // engine.register_fn("log_info", log_info);
    // engine.register_fn("log_error", log_error);
    // engine.register_fn("log_debug", log_debug);
    // engine.register_fn("log_warn", log_warn);
    // engine.register_fn("log_trace", log_trace);
    // engine.set_module_resolver(smr);
    engine.set_max_call_levels(255);

    engine
});

// pub struct ScriptEngine {
//     pub engine: Engine,
// }

fn call_req(id: u64) {
    let task_sender = unsafe { TASK_CHANNEL.0.clone() };
    TOKIO_RT.spawn(async move {
        if (task_sender.send((id, 1, 1)).await).is_err() {
            log::info!("receiver dropped");
        }
    });
}
#[export_module]
mod slog {
    use log::{debug, error, info, trace, warn};

    pub fn info(msg: &str) {
        info!("{}", msg);
    }
    pub fn error(msg: &str) {
        error!("{}", msg);
    }
    pub fn debug(msg: &str) {
        debug!("{}", msg);
    }
    pub fn warn(msg: &str) {
        warn!("{}", msg);
    }
    pub fn trace(msg: &str) {
        trace!("{}", msg);
    }
}

#[export_module]
mod crypto {
    use crate::utils::aes_tool::{
        aes_dec_cbc_string, aes_dec_ctr_string, aes_dec_ecb_string, aes_enc_cbc_string,
        aes_enc_ctr_string, aes_enc_ecb_string,
    };
    use base64::{engine::general_purpose::STANDARD, Engine};
    pub mod aes {
        #[rhai_fn(return_raw)]
        pub fn decode_cbc(key: &str, input: &str, iv: &str) -> Result<String, Box<EvalAltResult>> {
            aes_dec_cbc_string(key, input, iv)
                .map(|res| STANDARD.encode(res))
                .map_err(|e| Box::new(e.into()))
        }

        #[rhai_fn(return_raw)]
        pub fn encode_cbc(key: &str, input: &str, iv: &str) -> Result<String, Box<EvalAltResult>> {
            aes_enc_cbc_string(key, input, iv)
                .map(|res| STANDARD.encode(res))
                .map_err(|e| Box::new(e.into()))
        }
    }
}

#[export_module]
mod sbase64 {
    use base64::{engine::general_purpose::STANDARD, Engine};

    pub fn encode(fmt: &str) -> String {
        STANDARD.encode(fmt)
    }

    pub fn decode(fmt: &str) -> String {
        let df = STANDARD.decode(fmt);
        if let Ok(bytes) = df {
            if let Ok(f) = String::from_utf8(bytes) {
                return f;
            }
        }
        "解析错误".to_owned()
    }
}

#[export_module]
mod sfaker {

    use chrono::{DateTime, Duration, Local, TimeZone, Utc};
    use fake::faker::name::en::Name as NameEn;
    use fake::faker::name::zh_cn::Name as NameZh;
    use fake::Fake;
    use fake::StringFaker;
    use rhai::EvalAltResult;
    const ASCII_HEX: &str = "0123456789ABCDEF";
    const ASCII_NUM: &str = "0123456789";

    pub fn zh_name() -> String {
        NameZh().fake()
    }

    pub fn en_name() -> String {
        NameEn().fake()
    }

    // #[rhai_fn(name = "")]
    pub fn hex_str(low: i64, high: i64) -> String {
        let low: usize = low.try_into().unwrap_or_default();
        let mut high: usize = high.try_into().unwrap_or_default();
        if high <= low {
            high = low + 1;
        }
        let f = StringFaker::with(Vec::from(ASCII_HEX), low..high);
        let a: String = f.fake();
        a
    }

    pub fn str(low: i64, high: i64) -> String {
        let low: usize = low.try_into().unwrap_or_default();
        let mut high: usize = high.try_into().unwrap_or_default();
        if high <= low {
            high = low + 1;
        }
        let a: String = (low..high).fake();
        a
    }

    pub fn num_str(low: i64, high: i64) -> String {
        let low: usize = low.try_into().unwrap_or_default();
        let mut high: usize = high.try_into().unwrap_or_default();
        if high <= low {
            high = low + 1;
        }
        let f = StringFaker::with(Vec::from(ASCII_NUM), low..high);
        let a: String = f.fake();
        a
    }

    pub fn num(low: i64, high: i64) -> i64 {
        let low: usize = low.try_into().unwrap_or_default();
        let mut high: usize = high.try_into().unwrap_or_default();
        if high <= low {
            high = low + 1;
        }
        let a: usize = (low..high).fake();
        a as i64
    }

    pub fn email() -> String {
        let f: String = fake::faker::internet::en::FreeEmail().fake();
        f
    }

    pub fn username() -> String {
        let f: String = fake::faker::internet::en::Username().fake();
        f
    }
    pub fn ip4() -> String {
        let f: String = fake::faker::internet::en::IPv4().fake();
        f
    }
    pub fn ip6() -> String {
        let f: String = fake::faker::internet::en::IPv6().fake();
        f
    }
    pub fn useragent() -> String {
        let f: String = fake::faker::internet::en::UserAgent().fake();
        f
    }
    pub fn mac() -> String {
        let f: String = fake::faker::internet::en::MACAddress().fake();
        f
    }

    pub fn password(low: usize, mut high: usize) -> String {
        if high <= low {
            high = low + 1;
        }
        let f: String = fake::faker::internet::en::Password(low..high).fake();
        f
    }

    pub fn uuid() -> String {
        let f = uuid::Uuid::new_v4();
        f.hyphenated().to_string()
    }

    pub fn uuid_simple() -> String {
        let f = uuid::Uuid::new_v4();
        f.simple().to_string()
    }

    pub fn now(fmt: String) -> String {
        let local = chrono::Local::now();
        let fmt_data = local.format(fmt.as_str());
        fmt_data.to_string()
    }

    #[rhai_fn(return_raw)]
    pub fn datetime(fmt: String) -> Result<String, Box<EvalAltResult>> {
        let local = Utc::now();
        let ten_years = Duration::days(3660);
        let start = local.checked_sub_signed(ten_years).unwrap();
        let end = local.checked_add_signed(ten_years).unwrap();
        date_between(fmt.as_str(), start, end)
    }

    #[rhai_fn(return_raw)]
    pub fn datetime_after(fmt: String, date: String) -> Result<String, Box<EvalAltResult>> {
        let local = Utc::now();
        let ten_years = Duration::days(3660);
        let end = local.checked_add_signed(ten_years).unwrap();
        if let Ok(start) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
            date_between(fmt.as_str(), start, end)
        } else {
            Err(format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S").into())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn datetime_before(fmt: String, date: String) -> Result<String, Box<EvalAltResult>> {
        let local = Utc::now();
        let ten_years = Duration::days(3660);
        let start = local.checked_sub_signed(ten_years).unwrap();
        if let Ok(end) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
            date_between(fmt.as_str(), start, end)
        } else {
            Err(format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S").into())
        }
    }

    #[rhai_fn(return_raw)]
    pub fn date_between(
        fmt: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<String, Box<EvalAltResult>> {
        let f: String = fake::faker::chrono::zh_cn::DateTimeBetween(start, end).fake();
        let d = f.parse::<DateTime<Utc>>().unwrap();
        Ok(d.format(fmt).to_string())
    }

    #[rhai_fn(return_raw)]
    pub fn date_add(
        duration: i64,
        date: Option<String>,
        fmt: Option<String>,
    ) -> Result<String, Box<EvalAltResult>> {
        let fmt = match fmt {
            Some(f) => f,
            None => "%Y-%m-%dT%H:%M:%S".to_owned(),
        };
        let dura = Duration::seconds(duration);
        let local = if let Some(date_str) = date {
            if let Ok(start) = Local.datetime_from_str(date_str.as_str(), "%Y-%m-%dT%H:%M:%S") {
                start
            } else {
                Local::now()
            }
        } else {
            Local::now()
        };
        let fake_date = local.checked_add_signed(dura).unwrap();
        let fmt_data = fake_date.format(fmt.as_str());
        Ok(fmt_data.to_string())
    }
}

// impl ScriptEngine {
//     pub fn new() -> Self {
//         let mut engine = Engine::new();
//         engine.register_fn("call_req", call_req);
//         engine.register_fn("log_info", log_info);
//         engine.register_fn("log_error", log_error);
//         engine.register_fn("log_debug", log_debug);
//         engine.register_fn("log_warn", log_warn);
//         engine.register_fn("log_trace", log_trace);
//         engine.set_max_call_levels(255);
//         ScriptEngine { engine }
//     }

//     pub fn run(&self, script: &str) {
//         let res = self.engine.run(script);
//     }
// }

#[cfg(test)]
mod tests {
    use rhai::{Dynamic, Engine};

    use super::*;

    #[test]
    fn test_script() {
        let mut engine = Engine::new();
        engine.set_max_call_levels(150);
        let s = r#"
        //! This script calculates the n-th Fibonacci number using a really dumb algorithm
//! to test the speed of the scripting engine.

const TARGET = 28;
const REPEAT = 5;
const ANSWER = 317_811;

fn fib(n) {
    if n < 2 {
        n
    } else {
        fib(n-1) + fib(n-2)
    }
}

print(`Running Fibonacci(${TARGET}) x ${REPEAT} times...`);
print("Ready... Go!");

let result;
let now = timestamp();

for n in 0..REPEAT {
    result = fib(TARGET);
}

print(`Finished. Run time = ${now.elapsed} seconds.`);

print(`Fibonacci number #${TARGET} = ${result}`);

if result != ANSWER {
    print(`The answer is WRONG! Should be ${ANSWER}!`);
}
return result;
        "#;

        let result = engine.eval::<Dynamic>(s).unwrap();

        println!("Answer: {result}"); // prints 42
    }
}
