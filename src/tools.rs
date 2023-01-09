#![allow(dead_code)]
use rand::prelude::*;
use serde_json::{Value};


fn bisect_left<T>(a:&Vec<T>, x:T)->usize 
where
    T:PartialEq + PartialOrd
{
    let mut lo:usize = 0;
    let mut hi:usize = a.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if a[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid
        }
    }
    return lo;
}

pub fn weight_sample_index_from_vec(weight:&Vec<i64>, num:u64) -> Vec<usize> {
    let mut result = Vec::<usize>::new();
    let (mut sum_weight, mut real_weight) = (Vec::<i64>::new(), Vec::<i64>::new());
    let mut effect_num = 0;
    for i in 0..weight.len() {
        let mut curr_weight = weight[i];
        if curr_weight <= 0 {
            curr_weight = 0;
        } else {
            effect_num += 1;
        }
        real_weight.push(curr_weight);
        if i == 0 {
            sum_weight.push(curr_weight);
        } else {
            sum_weight.push(curr_weight + sum_weight[sum_weight.len() - 1]);
        }
    }
    let final_num = if effect_num < num {effect_num} else {num};
    let mut rng = thread_rng();
    for _i in 0..final_num {
        let select = rng.gen_range(1..=sum_weight[sum_weight.len() - 1]);
        let select_index = bisect_left(&sum_weight, select);
        result.push(select_index);
        for j in select_index..sum_weight.len() {
            sum_weight[j] -= real_weight[select_index];
        }
    }
    return result;
}

pub fn weight_sample_str(input:&str) -> &str {
    let sub_vec = input.split(";");
    let (mut data, mut weight) = (Vec::<&str>::new(), Vec::<i64>::new());
    for s in sub_vec {
        let sub_sub_vec:Vec<&str> = s.split("-").collect();
        data.push(sub_sub_vec[0]);
        weight.push(sub_sub_vec[1].parse::<i64>().unwrap());
    }
    let select_index = weight_sample_index_from_vec(&weight, 1);
    return data[select_index[0]];
}

pub fn weight_sample_int(input:&str) -> i64 {
    return weight_sample_str(input).parse::<i64>().unwrap();
}

// equal weight
pub fn sample<T:Copy>(data:&Vec<T>, num:i64) -> Vec<T> {
    let mut weight = vec![];
    for _i in 0..data.len() {
        weight.push(1);
    }
    let select_index = weight_sample_index_from_vec(&weight, num as u64);
    let mut result = vec![];
    for index in select_index {
        result.push(data[index]);
    }
    return result;
}

pub fn print_process(curr:i64, total:i64) {
    let curr_percent = curr * 100 / total;
    let last_percent = (curr - 1) * 100 / total;
    if curr_percent != last_percent {
        let mut result = String::from("\r");
        for _i in 0..curr_percent {
            result += "|";
        }
        for _i in 0..100-curr_percent {
            result += " "
        }
        result += &curr_percent.to_string();
        result += "%";
        if curr_percent == 100 {
            result += "\n";
        }
        print!("{}", result);
    }
}


#[macro_export]
macro_rules! fdiv {
    ($a:expr, $b:expr) => {
        $a as f64 / $b as f64
    };
    ($a:expr, $b:expr, $c:expr) => {
        $a as f64 / $b as f64 / $c as f64
    };
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        $a as f64 / $b as f64 / $c as f64 / $d as f64
    };
}

use serde_json::{value::Index};
// directly get value from Value
pub trait ValueTool {
    fn get_obj<I: Index>(&self, index: I) -> &Value;
    fn get_i64<I: Index>(&self, index: I) -> i64;
    fn get_usize<I: Index>(&self, index: I) -> usize;
    fn get_str<I: Index>(&self, index: I) -> &str;
    fn get_f64<I: Index>(&self, index: I) -> f64;
    fn get_bool<I: Index>(&self, index: I) -> bool;
    fn get_vec<I:Index>(&self, index: I) -> &Vec<Value>;
    fn get_vec_str<I: Index>(&self, index: I) -> Vec<&str>;
    fn get_vec_i64<I: Index>(&self, index: I) -> Vec<i64>;
    fn get_vec_usize<I: Index>(&self, index: I) -> Vec<usize>;
    fn get_vec_f64<I: Index>(&self, index: I) -> Vec<f64>;
}
impl ValueTool for Value {
    fn get_obj<I: Index>(&self, index: I) -> &Value {
        index.index_into(self).unwrap()
    }

    fn get_i64<I: Index>(&self, index: I) -> i64 {
        index.index_into(self).unwrap().as_i64().unwrap()
    }

    fn get_usize<I: Index>(&self, index: I) -> usize {
        index.index_into(self).unwrap().as_i64().unwrap() as usize
    }

    fn get_str<I: Index>(&self, index: I) -> &str {
        index.index_into(self).unwrap().as_str().unwrap()
    }

    fn get_f64<I: Index>(&self, index: I) -> f64 {
        index.index_into(self).unwrap().as_f64().unwrap()
    }

    fn get_bool<I: Index>(&self, index: I) -> bool {
        index.index_into(self).unwrap().as_bool().unwrap()
    }

    fn get_vec<I:Index>(&self, index: I) -> &Vec<Value> {
        index.index_into(self).unwrap().as_array().unwrap()
    }

    fn get_vec_str<I: Index>(&self, index: I) -> Vec<&str> {
        let mut result = Vec::<&str>::new();
        for value in index.index_into(self).unwrap().as_array().unwrap() {
            result.push(value.as_str().unwrap());
        }
        return result;
    }

    fn get_vec_i64<I: Index>(&self, index: I) -> Vec<i64> {
        let mut result = Vec::<i64>::new();
        for value in index.index_into(self).unwrap().as_array().unwrap() {
            result.push(value.as_i64().unwrap());
        }
        return result;
    }

    fn get_vec_f64<I: Index>(&self, index: I) -> Vec<f64> {
        let mut result = Vec::<f64>::new();
        for value in index.index_into(self).unwrap().as_array().unwrap() {
            result.push(value.as_f64().unwrap());
        }
        return result;
    }

    fn get_vec_usize<I: Index>(&self, index: I) -> Vec<usize> {
        let mut result = Vec::<usize>::new();
        for value in index.index_into(self).unwrap().as_array().unwrap() {
            result.push(value.as_i64().unwrap() as usize);
        }
        return result;
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Instant};

    #[test]
    fn test_bisect_left() {
        let a = vec![1,2,2,3,3,4];
        let x = 2;
        let result = bisect_left(&a, x);
        println!("{result}");
    }

    #[test]
    fn test_weight_sample_index_from_vec() {
        for _i in 0..10 {
            let weight = vec![0, 0, 2, 0];
            let num = 2;
            let result = weight_sample_index_from_vec(&weight, num);
            println!("{:?}", result);
        }
    }

    #[test]
    fn test_weight_sample_str() {
        let input = "1-0;2-1;3-5";
        let r = weight_sample_str(input);
        println!("{}", r);
    }

    #[test]
    fn test_print_process() {
        let start = Instant::now();
        let total_times = 1_000_000;
        for _i in 0..total_times {
            let input = "1-1;2-0;3-5;4-5";
            let _r = weight_sample_int(input);
            print_process(_i + 1, total_times);
        }
        let cost = Instant::now() - start;
        println!("time used: {:?}", cost);
    }

    #[test]
    fn test_fdiv() {
        let r = fdiv!(3,4,5);
        println!("{}", r);
    }

    #[test]
    fn test_valuetool() {
        let json_value = serde_json::json!({"a":1.2,"b":{"c":1},"d":["x","y","z"]});
        println!("{:?}", json_value.get_vec_str("d"));
    }
}