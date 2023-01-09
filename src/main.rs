#![allow(dead_code)]

mod tools;
mod data;
mod logic;

use rand::Rng;
use serde_json::{self, Value, json};
use std::{fs::{read_to_string}, io::Write};
use tools::ValueTool;


fn main() {
    let spin_times = 1_000_000;
    run_log(spin_times);
}

fn read_config() -> Value {
    let config = serde_json::from_str::<Value>(&read_to_string("config.json").unwrap()).unwrap();
    return config;
}

fn judge_full(reels:&Vec<Vec<i64>>, wilds:&Vec<i64>) -> bool {
    let label = reels[0][0];
    let mut normal_label = if wilds.contains(&label) {-1} else {label};
    for i in 0..reels.len() {
        for j in 0..reels[0].len()-1 {
            let curr_sym = reels[i][j];
            if wilds.contains(&curr_sym) {
                continue;
            } else if normal_label != -1 && curr_sym != normal_label {
                return false;
            } else if normal_label == -1 {
                normal_label = curr_sym;
            }
        } 
    }
    return true;
}

fn run_round(user_data: &mut data::UserData, config: &Value) -> data::SpinResult{
    let mut spin_result = data::SpinResult::new();

    // read config
    let base = config.get_obj("base");
    let logics = base.get_obj("logics");
    let (rows, columns) = (config.get_usize("rows"), config.get_usize("columns"));
    let calculator = base.get_obj("calculator");
    let wilds = calculator.get_vec_i64("wilds");

    // classic reel
    let mystery_target = tools::weight_sample_int(logics.get_str("mysteryChange"));
    let mut reels:Vec<Vec<i64>> = vec![];
    for _i in 0..rows {
        reels.push(vec![]);
    }
    for i in 0..columns {
        let mut sym = tools::weight_sample_int(base.get_vec_str("reels")[i]);
        if sym == logics.get_i64("mysterySignal") {
            sym = mystery_target;
        }
        reels[0].push(sym);
    }

    // left effect classic
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() < logics.get_f64("copyLeftPos") {
        let select_index = rng.gen_range(0..3);
        let select_sym = match user_data.selfdata.get_mut("leftReels") {
            Some(v) => {
                v.as_array().unwrap()[select_index].as_array().unwrap()[0].as_i64().unwrap()
            },
            None => {
                user_data.selfdata.insert("leftReels".to_string(), logics.get_obj("initLeftReels").clone());
                user_data.selfdata.get("leftReels").unwrap().as_array().unwrap()[select_index].as_array().unwrap()[0].as_i64().unwrap()
            }
        };
        let right_index = rng.gen_range(0..3);
        reels[0][right_index] = select_sym;
    }

    // collect
    match user_data.selfdata.get("collect") {
        Some(_v) => {},
        None => {user_data.selfdata.insert("collect".to_string(), json!(vec![0, 0, 0]));}
    }
    let collect_data = user_data.selfdata.get_mut("collect").unwrap().as_array_mut().unwrap();
    let mut trigger_marker:i64 = -1;
    let collect_request = logics.get_vec_i64("collectRequest");
    if judge_full(&reels, &wilds) {
        for i in 0..collect_request.len() {
            match collect_data[i].as_i64() {
                Some(v) => {
                    if v < collect_request[i] {
                        collect_data[i] = json!(v + 1);
                        if v + 1 >= collect_request[i] {
                            trigger_marker = i as i64;
                            if i == collect_request.len() - 1 {
                                user_data.selfdata.insert("collect".to_string(), json!(vec![0, 0, 0]));
                            }
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    // change jackpot
    let add_jackpot_num = tools::weight_sample_int(logics.get_str("addJackpotNum"));
    let add_cols = tools::sample(&Vec::<usize>::from_iter(0..columns), add_jackpot_num);
    for col in add_cols {
        reels[0][col] = 94;
    }

    // force avoid
    if judge_full(&reels, &wilds) {
        reels = vec![vec![0, 1, 2]];
    }

    // collect change reels
    if trigger_marker != -1 {
        for i in 0..trigger_marker + 1 {
            reels[0][i as usize] = wilds[0];
        }
    }

    // classic score
    let classic_lines = logic::get_winlines(&reels, calculator, config.get_obj("paylines"), user_data.bets / config.get_i64("lineNum"));
    spin_result.wins += classic_lines.1;
    if classic_lines.1 > 0 {
        spin_result.selfdata.insert("classic_wins".to_string(), json!(classic_lines.1));
    }

    // change left reels
    match user_data.selfdata.get_mut("leftReels") {
        Some(_v) => {},
        None => {
            user_data.selfdata.insert("leftReels".to_string(), logics.get_obj("initLeftReels").clone());
        }
    }
    let left_reels = user_data.selfdata.get_mut("leftReels").unwrap();
    for i in 0..columns {
        left_reels.as_array_mut().unwrap()[i].as_array_mut().unwrap().pop();
        left_reels.as_array_mut().unwrap()[i].as_array_mut().unwrap().insert(0, json!(reels[0][i]));
    }

    // left score
    let (left_rows, left_columns) = (logics.get_usize("leftRows"), logics.get_usize("leftColumns"));
    let mut left_calculator = logics.get_obj("fullCalculator");
    let left_reels = user_data.selfdata.get("leftReels").unwrap();
    for i in 0..left_rows * left_columns {
        if left_reels[i / left_columns][i % left_columns] != left_reels[0][0] {
            left_calculator = logics.get_obj("leftCalculator");
            break;
        }
    }
    let mut left_reels_i64 = Vec::<Vec<i64>>::new();
    for row in left_reels.as_array().unwrap() {
        left_reels_i64.push(row.as_array().unwrap().iter().map(|v|{v.as_i64().unwrap()}).collect());
    }
    let left_lines = logic::get_winlines(&left_reels_i64, left_calculator, config.get_obj("leftPaylines"), user_data.bets / logics.get_i64("leftLineNum"));
    let mut left_wins = 0;
    for line in left_lines.0 {
        if line.1 == 94 {
            // paytable has timed 10, so here devide 10
            let jackpot_wins = user_data.bets * config.get_obj("jackpot").get_i64("jackpotMulti") * line.0 / 10;
            match spin_result.selfdata.get_mut("jackpot_wins") {
                Some(v) => {
                    *v = json!(v.as_i64().unwrap() + jackpot_wins);
                },
                None => {
                    spin_result.selfdata.insert("jackpot_wins".to_string(), json!(jackpot_wins));
                }
            }
            spin_result.wins += jackpot_wins;
            left_wins += jackpot_wins
        } else {
            let line_wins = user_data.bets / logics.get_i64("leftLineNum") * line.0;
            spin_result.wins += line_wins;
            left_wins += line_wins;
        }
    }
    if left_wins > 0 {
        spin_result.selfdata.insert("left_wins".to_string(), json!(left_wins));
    }

    return spin_result;
}

fn run_log(spin_times:i64) {
    use std::time::Instant;

    let start_time = Instant::now();
    let mut user_data = data::UserData::new();
    let config = read_config();
    let mut total_wins = 0;
    let mut base_wins = 0;
    let (mut base_hit_times, mut base_win_times) = (0, 0);
    let (mut jackpot_wins, mut jackpot_times) = (0, 0);
    let (mut classic_wins, mut classic_times) = (0, 0);
    let (mut left_wins, mut left_times) = (0, 0);

    for spin in 0..spin_times {
        let result = run_round(&mut user_data, &config);
        total_wins += result.wins;
        if result.wins > 0 {
            base_wins += result.wins;
            base_hit_times += 1;
            if result.wins >= user_data.bets {
                base_win_times += 1;
            }
        }
        match result.selfdata.get("jackpot_wins") {
            Some(v) => {
                jackpot_wins += v.as_i64().unwrap();
                jackpot_times += 1;
            },
            None => {}
        }
        match result.selfdata.get("classic_wins") {
            Some(v) => {
                classic_wins += v.as_i64().unwrap();
                classic_times += 1;
            },
            None => {}
        }
        match result.selfdata.get("left_wins") {
            Some(v) => {
                left_times += 1;
                left_wins += v.as_i64().unwrap();
            },
            None => {}
        }
        tools::print_process(spin + 1, spin_times);
    }

    let total_bet = user_data.bets * spin_times;
    println!("rtp: {}", fdiv!(total_wins, total_bet));
    println!("====================base==================");
    println!("base rtp: {}", fdiv!(base_wins, total_bet));
    println!("hit rate: {}", fdiv!(base_hit_times, spin_times));
    println!("win rate: {}", fdiv!(base_win_times, spin_times));
    println!("==================jackpot=================");
    println!("jackpot interval: {}", if jackpot_times > 0 {fdiv!(spin_times, jackpot_times)} else {0.0});
    println!("jackpot rtp: {}", fdiv!(jackpot_wins, total_bet));
    println!("==================classic=================");
    println!("classic interval: {}", if classic_times > 0 {fdiv!(spin_times, classic_times)} else {0.0});
    println!("classic rtp: {}", fdiv!(classic_wins, total_bet));
    println!("==================left=================");
    println!("left interval: {}", if left_times > 0 {fdiv!(spin_times, left_times)} else {0.0});
    println!("left rtp: {}", fdiv!(left_wins, total_bet));
    println!("time cost: {:?}", Instant::now() - start_time);
}

fn run_csv(file_path:&str, spin_times:i64) {
    use std::fs::File;
    let mut file = File::create(file_path).unwrap();
    file.write(b"id,action,bets,wins,rtp\n").unwrap();
    let mut user_data = data::UserData::new();
    let config = read_config();

    for spin in 0..spin_times {
        let result = run_round(&mut user_data, &config);
        let curr_line = format!("{},{},{},{},{}", spin + 1, result.action, user_data.bets, result.wins, fdiv!(result.wins, user_data.bets));
        file.write(curr_line.as_bytes()).unwrap();
    }
}

fn run_multi_user(file_path:&str, spin_times:i64, user_num:i64) {
    use std::fs::File;
    let mut file = File::create(file_path).unwrap();
    file.write(b"id,rtp").unwrap();
    let config = read_config();

    for user in 0..user_num {
        let mut user_data = data::UserData::new();
        let (mut total_wins, total_bets) = (0, spin_times * user_data.bets);
        for _spin in 0..spin_times {
            let result = run_round(&mut user_data, &config);
            total_wins += result.wins;
        }
        let curr_line = format!("{},{}", user + 1, fdiv!(total_wins, total_bets));
        file.write(curr_line.as_bytes()).unwrap();

    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test_something() {
        use serde_json::json;
        let a = json!(0.1);
        let b = a.as_f64().unwrap();
        println!("{b}");
    }
}