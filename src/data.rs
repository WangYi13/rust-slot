#![allow(dead_code)]

use serde_json::{Value, Map};

pub struct UserData {
    pub bets:i64,
    pub selfdata:Map<String, Value>,
    pub actions:Vec<i64>
}
impl UserData {
    pub fn new() -> UserData {
        UserData { bets: 9, selfdata: Map::new(), actions:vec![0] }
    }
}

pub struct SpinResult {
    pub wins:i64,
    pub action:i64,
    pub selfdata:Map<String, Value>,
}
impl SpinResult {
    pub fn new() -> SpinResult {
        SpinResult { wins: 0, action:0, selfdata:Map::new()}
    }
}

pub trait RunnerTrait {
    fn run(&self, flow:&mut FlowData);
}


pub struct PhantomLogic {}
impl RunnerTrait for PhantomLogic {
    fn run(&self, _flow:&mut FlowData) { }
}

pub struct FlowData
{
    pub user_data:UserData,
    pub msg:String,
    pub spin_result:SpinResult,
    pub config:Value,
    pub logic:Box<dyn RunnerTrait>
}
impl FlowData {
    pub fn new() -> FlowData {
        FlowData { 
            user_data: UserData::new(), 
            msg: "".to_string(), 
            spin_result: SpinResult::new(), 
            config: serde_json::json!("{}"), 
            logic: Box::new(PhantomLogic {}),
        }
    }
}