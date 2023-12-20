use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::{Db, IVec};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::api_context::ApiTester;

const HIST_DB: &str = "history_http_test";
const HIST_LIST_KEY: &[u8] = b"history_list";

type HistoryRecode = (String, u32);

fn get_db_name(id: u64) -> String {
    format!("{}/{}", HIST_DB, id)
}

pub fn get_history_list(id: u64) -> Vec<HistoryRecode> {
    let mut res = Vec::new();
    if let Ok(db) = sled::open(get_db_name(id)) {
        if let Some(db_res) = db.get(HIST_LIST_KEY).unwrap() {
            res = convert_to_obj(db_res);
        }
    }
    res
}

pub fn get_apitest(id: u64, versoin: u32) -> Option<ApiTester> {
    if let Ok(db) = sled::open(get_db_name(id)) {
        if let Some(res_ivec) = db.get(key_to_ivec(versoin)).unwrap() {
            let res_obj: ApiTester = convert_to_obj(res_ivec);
            return Some(res_obj);
        }
    }
    None
}

pub fn add_new_version_mockinfo(id: u64, mock_info: &ApiTester) {
    if let Ok(db) = sled::open(get_db_name(id)) {
        let mut last_id = 0;
        let mut history_list: Vec<HistoryRecode> = Vec::new();
        if let Some(db_res) = db.get(HIST_LIST_KEY).unwrap() {
            history_list = convert_to_obj(db_res);
            if let Some(item) = history_list.last() {
                last_id = item.1 + 1;
            }
        }
        let now = chrono::Local::now();
        let now = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        history_list.push((now, last_id));
        let _res: Result<(), sled::transaction::TransactionError<String>> =
            db.transaction(|tx_db| {
                let val = convert_to_ivec(mock_info);
                tx_db.insert(HIST_LIST_KEY, convert_to_ivec(&history_list))?;
                tx_db.insert(key_to_ivec(last_id), val)?;
                Ok(())
            });
    }
}

fn decompose_key(key: IVec) -> Option<(u64, u32)> {
    if key.len() >= 12 {
        let (id, ver) = key.split_at(8);
        let id: [u8; 8] = id.try_into().unwrap();
        let id = u64::from_be_bytes(id);

        let ver: [u8; 4] = ver.try_into().unwrap();
        let ver = u32::from_be_bytes(ver);
        Some((id, ver))
    } else {
        None
    }
}

fn convert_to_ivec<T>(obj: &T) -> IVec
where
    T: ?Sized + Serialize,
{
    let val = serde_json::to_vec(obj).unwrap();
    IVec::from(val)
}

fn convert_to_obj<T>(v: IVec) -> T
where
    for<'de> T: Deserialize<'de>,
{
    let v = v.to_vec();
    // let val:Value = serde_json::from_slice(v.as_slice()).unwrap();
    // let obj:T = serde_json::from_value(val).unwrap();
    // obj
    let obj: T = serde_json::from_slice(v.as_slice()).unwrap();
    obj
}

fn convert_ivec_to_mock(v: IVec) -> ApiTester {
    let v = v.to_vec();
    let obj = serde_json::from_slice::<ApiTester>(v.as_slice()).unwrap();
    obj
}

fn iver_to_key(v: IVec) -> u32 {
    let vec = v.to_vec();
    let ver: [u8; 4] = vec.try_into().unwrap();
    let ver = u32::from_be_bytes(ver);
    ver
}

fn key_to_ivec(versoin: u32) -> IVec {
    let vb = versoin.to_be_bytes();
    vb.as_slice().into()
}
fn compose_key(id: i64, versoin: i32) -> [u8; 12] {
    let mut key = [0; 12];
    let idb = id.to_be_bytes();
    let vb = versoin.to_be_bytes();
    for i in 0..8 {
        key[i] = idb[i];
    }
    for i in 8..12 {
        key[i] = vb[i - 8];
    }
    key
}

#[cfg(test)]
mod tests {

    use sled::IVec;

    use crate::api_context::ApiTester;

    use super::{
        add_new_version_mockinfo, compose_key, get_history_list, get_apitest, iver_to_key, key_to_ivec,
    };

    #[test]
    fn test_add_mockinfo() {
        let mut v =ApiTester::default();
        for i in 0..100 {
            v.req.remark = i.to_string();
            add_new_version_mockinfo(101, &v);
        }
    }

    #[test]
    fn test_get_mockinfo() {
        // dbg!(get_mock(101, 12));
    }

    #[test]
    fn test_get_list() {
        dbg!(get_history_list(101));
    }

    #[test]
    fn test_insert_db() {
        if let Ok(db) = sled::open("100") {
            let k1 = 101i64;
            let k2 = 1025i32;
            let key = compose_key(k1, k2);
            let v = ApiTester::default();
            let val = serde_json::to_string(&v).unwrap();
            let vv = IVec::from(val.as_str());
            let _ = db.insert(&key, vv);
        }
    }

    #[test]
    fn test_convert() {
        let v = 0u32;
        let v_ivec = dbg!(key_to_ivec(v));
        let vv = dbg!(iver_to_key(v_ivec));
        assert_eq!(v, vv);
    }

    #[test]
    fn test_get() {
        if let Ok(db) = sled::open("100") {
            let k1 = 100i32;
            let k = k1.to_be_bytes();
            let v = db.get(k).unwrap().unwrap();
            let v = v.to_vec();
            let m: ApiTester = serde_json::from_slice(v.as_slice()).unwrap();
        }
    }

    #[test]
    fn test_prefix() {
        let k = 100i64;
        let kb = k.to_be_bytes();
        if let Ok(db) = sled::open("100") {
            let red = db.scan_prefix(kb);
            println!("{}", red.count());
            // for item in red {
            //     let (key_v,val_v) = item.unwrap();
            //     let (id,ver) = decompose_key(key_v).unwrap();
            //     let m = convert_ivec_to_mock(val_v);
            //     println!("{}-{}",id,ver);
            //     println!("{:?}",m);
            // }
        }
    }
}
