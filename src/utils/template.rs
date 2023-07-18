use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex,RwLock};
use base64::{Engine,engine::general_purpose::STANDARD};
use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use minijinja::value::Value;
use minijinja::{context, Environment, Syntax};
use minijinja::{Error, ErrorKind, State};

use crate::utils::aes_tool::{
    aes_dec_cbc_string, aes_dec_ctr_string, aes_dec_ecb_string, aes_enc_cbc_string,
    aes_enc_ctr_string, aes_enc_ecb_string,
};
use fake::faker::name::en::Name as NameEn;
use fake::faker::name::zh_cn::Name as NameZh;
use fake::Fake;
use fake::StringFaker;
use once_cell::sync::Lazy;
use uuid::Uuid;

// const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
const ASCII_HEX: &str = "0123456789ABCDEF";
const ASCII_NUM: &str = "0123456789";
pub static TMP_SCOPE_CTX:Lazy<Arc<RwLock<Value>>> = Lazy::new(||{
    Arc::new(RwLock::new(Value::UNDEFINED)) 
});
static TEMP_ENV: Lazy<Arc<Mutex<Environment<'static>>>> = Lazy::new(|| {
    let mut t_env = Environment::new();

    t_env
        .set_syntax(Syntax {
            block_start: "%{".into(),
            block_end: "}".into(),
            variable_start: "${".into(),
            variable_end: "}".into(),
            comment_start: "#{".into(),
            comment_end: "}".into(),
        })
        .unwrap();

    t_env.add_function("NAME_ZH", fake_name_zh);
    t_env.add_function("NAME_EN", fake_name_en);
    t_env.add_function("NUM", fake_num);
    t_env.add_function("NUM_STR", fake_num_str);
    t_env.add_function("HEX", fake_hex);
    t_env.add_function("STR", fake_str);
    t_env.add_function("EMAIL", fake_email);
    t_env.add_function("USERNAME", fake_username);
    t_env.add_function("IPV4", fake_ip4);
    t_env.add_function("IPV6", fake_ip6);
    t_env.add_function("MAC", fake_mac);
    t_env.add_function("USERAGENT", fake_useragent);
    t_env.add_function("PASSWORD", fake_password);

    t_env.add_function("UUID", fake_uuid);
    t_env.add_function("UUID_SIMPLE", fake_uuid_s);

    t_env.add_function("NOW", fake_now);
    t_env.add_function("DATE_BEFORE", fake_datetime_before);
    t_env.add_function("DATE_AFTER", fake_datetime_after);
    t_env.add_function("DATE", fake_datetime);
    t_env.add_function("DATE_ADD", fake_date_add);

    t_env.add_function("AES_ECB_EN", aes_enc_ecb);
    t_env.add_function("AES_ECB_DE", aes_dec_ecb);
    t_env.add_function("AES_CBC_EN", aes_enc_cbc);
    t_env.add_function("AES_CBC_DE", aes_dec_cbc);
    t_env.add_function("AES_CTR_EN", aes_enc_ctr);
    t_env.add_function("AES_CTR_DE", aes_dec_ctr);

    t_env.add_function("BASE64_EN", fake_base64_en);
    t_env.add_function("BASE64_DE", fake_base64_de);

    t_env.add_filter("base64Encode", fake_base64_en);
    t_env.add_filter("AesEcbEnc", aes_enc_ecb);
    t_env.add_filter("AesCbcEnc", aes_enc_cbc);
    t_env.add_filter("AesCtrEnc", aes_enc_ctr);
    t_env.add_filter("INT", to_int);
    Arc::new(Mutex::new(t_env))
});

fn to_int(_state: &State<'_, '_>, value: String) -> Result<i32, Error> {
    value.parse::<i32>().map_err(|_e| {
        Error::new(
            ErrorKind::InvalidOperation,
            format!("{}cant turn to int", value),
        )
    })
}

fn aes_dec_ecb(_state: &State<'_, '_>, value: String, key: String) -> Result<String, Error> {
    aes_dec_ecb_string(key.as_str(), value.as_str())
        .map(|res| STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

fn aes_enc_ecb(_state: &State<'_, '_>, value: String, key: String) -> Result<String, Error> {
    aes_enc_ecb_string(key.as_str(), value.as_str())
        .map(|res| STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

fn aes_dec_cbc(
    _state: &State<'_, '_>,
    value: String,
    key: String,
    iv: String,
) -> Result<String, Error> {
    aes_dec_cbc_string(key.as_str(), value.as_str(), iv.as_str())
        .map(|res| STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

fn aes_enc_cbc(
    _state: &State<'_, '_>,
    value: String,
    key: String,
    iv: String,
) -> Result<String, Error> {
    aes_enc_cbc_string(key.as_str(), value.as_str(), iv.as_str())
        .map(|res| STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

fn aes_dec_ctr(
    _state: &State<'_, '_>,
    value: String,
    key: String,
    iv: String,
) -> Result<String, Error> {
    aes_dec_ctr_string(key.as_str(), value.as_str(), iv.as_str())
        .map(|res| STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

fn aes_enc_ctr(
    _state: &State<'_, '_>,
    value: String,
    key: String,
    iv: String,
) -> Result<String, Error> {
    aes_enc_ctr_string(key.as_str(), value.as_str(), iv.as_str())
        .map(|res|STANDARD.encode(res))
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
}

pub fn rander_template(template: &str) -> Result<String, Error> {
    let mut lock = TEMP_ENV.lock().unwrap();
    let env = lock.borrow_mut();

    let tmp = TMP_SCOPE_CTX.read().unwrap().clone();
    let result = env
        .render_str(template, tmp)
        .unwrap_or_else(|s| s.to_string());
    Ok(result)
}
pub fn remove_global_value(key: &str) {
    if let Ok(mut env) = TEMP_ENV.lock() {
        let _res = env.remove_global(key);
    }
}

pub fn add_global_var(key: String, value: Value) {
    if let Ok(mut env) = TEMP_ENV.lock() {
        env.add_global(key, value);
    }
}

fn fake_name_zh(_state: &State<'_, '_>) -> Result<String, Error> {
    let name = NameZh().fake();
    Ok(name)
}

fn fake_name_en(_state: &State<'_, '_>) -> Result<String, Error> {
    let name = NameEn().fake();
    Ok(name)
}

fn fake_hex(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_HEX), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_str(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: String = (low..high).fake();
    Ok(a)
}

fn fake_num_str(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f = StringFaker::with(Vec::from(ASCII_NUM), low..high);
    let a: String = f.fake();
    Ok(a)
}

fn fake_num(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let a: usize = (low..high).fake();
    Ok(a.to_string())
}

fn fake_email(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::FreeEmail().fake();
    Ok(f)
}

fn fake_username(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::Username().fake();
    Ok(f)
}
fn fake_ip4(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv4().fake();
    Ok(f)
}
fn fake_ip6(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::IPv6().fake();
    Ok(f)
}
fn fake_useragent(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::UserAgent().fake();
    Ok(f)
}
fn fake_mac(_state: &State<'_, '_>) -> Result<String, Error> {
    let f: String = fake::faker::internet::en::MACAddress().fake();
    Ok(f)
}

fn fake_password(_state: &State<'_, '_>, low: usize, mut high: usize) -> Result<String, Error> {
    if high <= low {
        high = low + 1;
    }
    let f: String = fake::faker::internet::en::Password(low..high).fake();
    Ok(f)
}

fn fake_uuid(_state: &State<'_, '_>) -> Result<String, Error> {
    let f = Uuid::new_v4();
    Ok(f.hyphenated().to_string())
}

fn fake_uuid_s(_state: &State<'_, '_>) -> Result<String, Error> {
    let f = Uuid::new_v4();
    Ok(f.simple().to_string())
}

fn fake_base64_en(_state: &State<'_, '_>, fmt: String) -> Result<String, Error> {
    let f = STANDARD.encode(fmt);
    Ok(f)
}

fn fake_base64_de(_state: &State<'_, '_>, fmt: String) -> Result<String, Error> {
    let df =  STANDARD.decode(fmt.clone());
    if let Ok(bytes) = df {
        if let Ok(f) = String::from_utf8(bytes) {
            return Ok(f);
        }
    }
    Ok(fmt)
}
fn fake_now(_state: &State<'_, '_>, fmt: String) -> Result<String, Error> {
    let local = Local::now();
    let fmt_data = local.format(fmt.as_str());
    Ok(fmt_data.to_string())
}

fn fake_datetime(_state: &State<'_, '_>, fmt: String) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    let end = local.checked_add_signed(ten_years).unwrap();
    fake_date_between(_state, fmt.as_str(), start, end)
}

fn fake_datetime_after(_state: &State<'_, '_>, fmt: String, date: String) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let end = local.checked_add_signed(ten_years).unwrap();
    if let Ok(start) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(
            ErrorKind::SyntaxError,
            format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S"),
        ))
    }
}

fn fake_datetime_before(
    _state: &State<'_, '_>,
    fmt: String,
    date: String,
) -> Result<String, Error> {
    let local = Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    if let Ok(end) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(
            ErrorKind::SyntaxError,
            format!("{}与{}格式不匹配", date, "%Y-%m-%dT%H:%M:%S"),
        ))
    }
}

fn fake_date_between(
    _state: &State<'_, '_>,
    fmt: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<String, Error> {
    let f: String = fake::faker::chrono::zh_cn::DateTimeBetween(start, end).fake();
    let d = f.parse::<DateTime<Utc>>().unwrap();
    Ok(d.format(fmt).to_string())
}

fn fake_date_add(
    _state: &State<'_, '_>,
    duration: i64,
    date: Option<String>,
    fmt: Option<String>,
) -> Result<String, Error> {
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

#[cfg(test)]
mod tests {

    use chrono::{DateTime, Local, TimeZone, Utc};
    use rhai::Engine;
    // use nom::bytes::complete::is_not;
    // use nom::{
    //     // see the "streaming/complete" paragraph lower for an explanation of these submodules
    //     character::complete::char,
    //     sequence::delimited,
    //     IResult,
    // };
    use super::*;
    use minijinja::{context, Environment};
    use minijinja::{Error, State};
    use serde_json::Value;

    use fake::faker::name::zh_cn::Name;
    use fake::Fake;

    #[test]
    fn test_render() {
        let str_tmpl = "{% if a %} hell0 {% endif %} 中文名字: {{NAME_ZH()}},英文名字{{NAME_EN()}}";
        let result = super::rander_template(str_tmpl);
        println!("{}", result.unwrap());
    }
    #[test]
    fn test_minijinja() {
        let mut env = Environment::new();
        env.add_template("hello", r#"{"a":"{{NAME()}}","b":"afsdf"}"#)
            .unwrap();
        env.add_function("hello", test_minijinja_fn);
        env.add_function("NAME", test_fake_name_zh);
        let tmpl = env.get_template("hello").unwrap();
        println!("{}", tmpl.render(context!(name => "John")).unwrap());
    }

    fn test_minijinja_fn(_state: &State<'_, '_>, mut name: String) -> Result<String, Error> {
        name.push_str("aaaaa");
        Ok(name)
    }

    fn test_fake_name_zh(_state: &State<'_, '_>) -> Result<String, Error> {
        let name = Name().fake();
        Ok(name)
    }

    #[test]
    fn test_fake_num() {
        let f: String = fake::faker::address::zh_cn::CityName().fake();
        println!("城市{}", f);
        let f: String = fake::faker::lorem::zh_cn::Word().fake();
        println!("{}", f);
        let f: String = fake::faker::phone_number::zh_cn::PhoneNumber().fake();
        println!("{}", f);
        // let f:String = fake::faker::lorem::zh_cn::Sentence(10..20).fake();
        // println!("{}",f);
        // let f:String = fake::faker::lorem::zh_cn::Paragraph(100..200).fake();
        // println!("{}",f);
        let f: String = fake::faker::job::zh_cn::Title().fake();
        println!("{}", f);
        let f: String = fake::faker::job::zh_cn::Field().fake();
        println!("{}", f);
        let f: String = fake::faker::job::zh_cn::Position().fake();
        println!("{}", f);
        let f: String = fake::faker::job::zh_cn::Seniority().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::Username().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::FreeEmail().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::SafeEmail().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::IPv4().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::UserAgent().fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::Password(6..12).fake();
        println!("{}", f);
        let f: String = fake::faker::internet::en::MACAddress().fake();
        println!("{}", f);
        let f: u16 = (1000..10000).fake();
        let f = uuid::Uuid::new_v4();
        println!("{}", f);
        let local = Local::now();
        println!("{}", local);
        println!("{}", local.format("%Y%m%d"));
        println!("{}", local.timestamp_millis());

        let _d = "2014-11-28T00:00:00Z".parse::<DateTime<Local>>().unwrap();
        let d = Utc
            .datetime_from_str("2014-11-28 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        println!("{}", d);

        let f: String = fake::faker::chrono::zh_cn::Date().fake();
        println!("{}", f);
        let f: String = fake::faker::chrono::zh_cn::DateTime().fake();
        println!("{}", f);
        let f: String = fake::faker::chrono::en::DateTime().fake();
        println!("{}", f);
        let f: String = fake::uuid::UUIDv4.fake();
        println!("{}", f);
    }

    // fn parens(input: &str) -> IResult<&str, &str> {
    //     delimited(char('('), is_not(")"), char(')'))(input)
    // }

    // fn pair_parse(input: &str) -> IResult<&str, (char, char)> {
    //     pair(char('('), char(')'))(input)
    // }

    // #[test]
    // fn test_parse_demo1() {
    //     let input = "(AD(DC(SD)DS)SC)";
    //     let x = parens(input);
    //     let (a, b) = x.unwrap();
    //     println!("{}=={}", a, b);
    // }

    #[test]
    fn test_pretty() {
        // Some JSON input data as a &str. Maybe this comes from the user.
        let data = r#"
        { "name": "John Doe", "age": 43,
            "phones": [ "+44 1234567", "+44 2345678" ]
        }"#;

        // Parse the string of data into serde_json::Value.
        let v: Value = serde_json::from_str(data).unwrap();

        let s = serde_json::to_string_pretty(&v).unwrap();
        // Access parts of the data by indexing with square brackets.
        println!("{}", s);
    }

    #[test]
    fn test_script() {
        fn fake_name_test() -> String {
            let name = NameZh().fake();
            name
        }

        let mut engine = Engine::new();

        engine.register_fn("add", fake_name_test);

        let result = engine.eval::<String>("add()").unwrap();

        println!("Answer: {result}"); // prints 42
    }

    // #[test]
    // fn test_python_vm() {
    // use rustpython_vm as vm;

    // vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {

    //     let scope = vm.new_scope_with_builtins();
    //     let source = r#"print("Hello World!")"#;
    //     let code_obj = vm
    //         .compile(source, vm::compiler::Mode::Exec, "<embedded>".to_owned())
    //         .map_err(|err| vm.new_syntax_error(&err, Some(source)))?;

    //     vm.run_code_obj(code_obj, scope)?;

    //     Ok(())
    // })
    // }
}
