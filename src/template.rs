use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

use chrono::{Local, DateTime, Utc, Duration, TimeZone};
use minijinja::{context, Environment, Source};
use minijinja::{Error, ErrorKind, State};

use fake::faker::name::en::Name as NameEn;
use fake::faker::name::zh_cn::Name as NameZh;
use fake::{StringFaker };
use fake::{Fake};
use uuid::Uuid;


const REQ_TEMPLATE: &str = "req_template";
// const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
const ASCII_HEX: &str = "0123456789ABCDEF";
const ASCII_NUM: &str = "0123456789";
lazy_static! {
    static ref TEMP_ENV: Arc<Mutex<Environment<'static>>> = {
        let mut t_env = Environment::new();
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
        t_env.add_function("DATE",fake_datetime);


        t_env.add_function("BASE64_EN",fake_base64_en);
        t_env.add_function("BASE64_DE",fake_base64_de);
        let source = Source::new();
        t_env.set_source(source);
        Arc::new(Mutex::new(t_env))
    };
}

pub fn rander_template(template: &str) -> Result<String,Error> {
    let mut lock = TEMP_ENV.lock().unwrap();
    let env = lock.borrow_mut();
    let mut source = env.source().unwrap().clone();
    source.add_template(REQ_TEMPLATE, template)?;
    env.set_source(source);
    let temp = env.get_template(REQ_TEMPLATE).unwrap();
    let result = temp
        .render(context!(aaa=>"aaa"))
        .unwrap_or_else(|_s| template.to_string());
    Ok(result)
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

fn fake_base64_en(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let f = base64::encode(fmt);
    Ok(f)
}

fn fake_base64_de(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let df = base64::decode(fmt.clone());
    if let Ok(bytes) = df {
        if let Ok(f) = String::from_utf8(bytes) {
            return Ok(f);
        }
    }
    Ok(fmt)
}
fn fake_now(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let local = Local::now();
    let fmt_data = local.format(fmt.as_str());
    Ok(fmt_data.to_string())
}

fn fake_datetime(_state: &State<'_, '_>,fmt:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    let end = local.checked_add_signed(ten_years).unwrap();
    fake_date_between(_state, fmt.as_str(), start, end)
}

fn fake_datetime_after(_state: &State<'_, '_>,fmt:String,date:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let end = local.checked_add_signed(ten_years).unwrap();
    if let Ok(start) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(ErrorKind::SyntaxError, format!("{}与{}格式不匹配", date,"%Y-%m-%dT%H:%M:%S")))
    }
}

fn fake_datetime_before(_state: &State<'_, '_>,fmt:String,date:String) -> Result<String, Error> {
    let local =Utc::now();
    let ten_years = Duration::days(3660);
    let start = local.checked_sub_signed(ten_years).unwrap();
    if let Ok(end) = Utc.datetime_from_str(date.as_str(), "%Y-%m-%dT%H:%M:%S") {
        fake_date_between(_state, fmt.as_str(), start, end)
    } else {
        Err(Error::new(ErrorKind::SyntaxError, format!("{}与{}格式不匹配", date,"%Y-%m-%dT%H:%M:%S")))
    }
}

fn fake_date_between(_state: &State<'_, '_>,fmt:&str,start:DateTime<Utc>,end:DateTime<Utc>) -> Result<String, Error> {
    let f:String = fake::faker::chrono::zh_cn::DateTimeBetween(start,end).fake();
    let d = f.parse::<DateTime<Utc>>().unwrap();
    Ok(d.format(fmt).to_string())
}



#[cfg(test)]
mod tests {
    

    use chrono::{Local, DateTime, Utc, TimeZone};
    use nom::bytes::complete::is_not;
    use nom::sequence::pair;
    use nom::{
        // see the "streaming/complete" paragraph lower for an explanation of these submodules
        character::complete::char,
        sequence::delimited,
        IResult,
    };
    use serde_json::Value;

    use minijinja::{context, Environment};
    use minijinja::{Error, State};

    use fake::faker::name::zh_cn::Name;
    use fake::{Fake};

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
        // let f:String = fake::faker::address::zh_cn::CityName().fake();
        // println!("城市{}",f);
        // let f:String = fake::faker::lorem::zh_cn::Word().fake();
        // println!("{}",f);
        // let f:String = fake::faker::lorem::zh_cn::Sentence(10..20).fake();
        // println!("{}",f);
        // let f:String = fake::faker::job::zh_cn::Title().fake();
        // println!("{}",f);
        // let f:String = fake::faker::job::zh_cn::Field().fake();
        // println!("{}",f);
        // let f:String = fake::faker::job::zh_cn::Position().fake();
        // println!("{}",f);
        // let f:String = fake::faker::job::zh_cn::Seniority().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::Username().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::FreeEmail().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::SafeEmail().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::IPv4().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::UserAgent().fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::Password(6..12).fake();
        // println!("{}",f);
        // let f:String = fake::faker::internet::en::MACAddress().fake();
        // println!("{}",f);
        // let f: u16 = (1000..10000).fake();
        // let f = uuid::Uuid::new_v4(); 
        // println!("{}", f);
        let local = Local::now();
        println!("{}", local);
        println!("{}", local.format("%Y%m%d"));
        println!("{}", local.timestamp_millis());

        let _d = "2014-11-28T00:00:00Z".parse::<DateTime<Local>>().unwrap();
        let d = Utc.datetime_from_str("2014-11-28 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        println!("{}", d);


        let f:String = fake::faker::chrono::zh_cn::Date().fake();
        println!("{}", f);
        let f:String = fake::faker::chrono::zh_cn::DateTime().fake();
        println!("{}", f);
        let f:String = fake::faker::chrono::en::DateTime().fake();
        println!("{}", f);
        let f:String = fake::uuid::UUIDv4.fake();
        println!("{}", f);


    }

    fn parens(input: &str) -> IResult<&str, &str> {
        delimited(char('('), is_not(")"), char(')'))(input)
    }

    // fn pair_parse(input: &str) -> IResult<&str, (char, char)> {
    //     pair(char('('), char(')'))(input)
    // }

    #[test]
    fn test_parse_demo1() {
        let input = "(AD(DC(SD)DS)SC)";
        let x = parens(input);
        let (a, b) = x.unwrap();
        println!("{}=={}", a, b);
    }

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
}
