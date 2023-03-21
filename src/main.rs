#![allow(dead_code)]

mod tools;
mod data;
mod machines;
mod router;

use crate::machines::common_logics;


fn main() {
    
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    #[test]
    fn test() {
        let s_str = "{\"a\":1234}";
        println!("{}", s_str);
        let s = serde_json::from_str::<Value>(&s_str).unwrap();
        // let s_obj = s.as_object().unwrap();
        println!("{}", s.get("a").unwrap())
    }
}