#![allow(dead_code)]

use serde_json::{Value, Map};

pub struct UserData {
    pub bets:i64,
    pub selfdata:Map<String, Value>,
}
impl UserData {
    pub fn new() -> UserData {
        UserData { bets: 9, selfdata: Map::new() }
    }
}

pub struct SpinResult {
    pub wins:i64,
    pub action:String,
    pub selfdata:Map<String, Value>,
}
impl SpinResult {
    pub fn new() -> SpinResult {
        SpinResult { wins: 0, action:String::from("BASE"), selfdata:Map::new()}
    }
}