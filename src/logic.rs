#![allow(dead_code)]

use crate::tools::{self, ValueTool};
use serde_json::Value;


pub fn cut_reel(reel_str:&str, rows:usize) -> Vec<i64> {
    let mut result = Vec::<i64>::new();
    let (mut syms, mut weights) = (Vec::<i64>::new(), Vec::<i64>::new());
    for sub_str in reel_str.split(";") {
        let sub_sub_str:Vec<&str> = sub_str.split("-").collect();
        syms.push(sub_sub_str[0].parse::<i64>().unwrap());
        weights.push(sub_sub_str[1].parse::<i64>().unwrap());
    }
    let start_index = tools::weight_sample_index_from_vec(&weights, 1)[0];
    for i in start_index..start_index + rows {
        result.push(syms[(i + syms.len()) % syms.len()]);
    }
    return result;
}

pub fn count_line(slice:&Vec<i64>, calculator:&Value) -> Option<(i64, i64, i64)> {
    let (mut wild_num, mut is_wild) = (0, false);
    let mut curr_sym = slice[0];
    let wilds = calculator.get_vec_i64("wilds");
    if wilds.contains(&curr_sym) {
        is_wild = true;
    }
    let (mut flag, mut normal_num) = (curr_sym, 0);

    let (mut is_continue, mut curr_index) = (true, 0);
    while is_continue {
        curr_sym = slice[curr_index];
        if wilds.contains(&curr_sym) {
            normal_num += 1;
            if is_wild {
                wild_num += 1;
            }
        } else {
            if is_wild {
                is_wild = false;
                flag = curr_sym;
                normal_num += 1;
            } else if flag == curr_sym {
                normal_num += 1;
            } else {
                is_continue = false;
            }
        }
        curr_index += 1;
        if curr_index >= slice.len() {
            is_continue = false;
        }
    }
    let wild_score = match calculator.get_obj("score").get(wilds[0].to_string()) {
        Some(Value::Array(v)) => {v[wild_num].as_i64().unwrap()},
        _ => 0
    };
    let normal_score = match calculator.get_obj("score").get(flag.to_string()) {
        Some(Value::Array(v)) => {v[normal_num].as_i64().unwrap()},
        _ => 0
    };
    // return none if line flag is scatter and scatter is not count by line
    let scatters:Vec<i64> = match calculator.get("scatters") {
        Some(_v) => {
            calculator.get_vec_i64("scatters")
        },
        _ => {vec![]}
    };
    let scatter_is_online:bool = match calculator.get("scatterIsOnLine") {
        Some(v) => {v.as_bool().unwrap()},
        _ => {false}
    };
    if scatters.contains(&flag) && wild_score <= 0 && !scatter_is_online {
        return None;
    }

    // return wild line or normal line if score > 0
    if normal_score > 0 && normal_score >= wild_score {
        return Some((normal_score, flag, normal_num as i64));
    }
    if wild_score > normal_score {
        return Some((wild_score, wilds[0], wild_num as i64));
    }
    return None;
}

pub fn get_winlines(reels:&Vec<Vec<i64>>, calculator:&Value, paylines:&Value, line_bet:i64)
    -> (Vec<(i64, i64, i64)>, i64) 
{
    let (mut lines, mut wins) = (vec![], 0);
    for line in paylines.as_array().unwrap() {
        let mut slice = vec![];
        let line_array = line.as_array().unwrap();
        for i in 0..line_array.len() {
            slice.push(reels[line_array[i].as_u64().unwrap() as usize][i]);
        }
        match count_line(&slice, calculator) {
            Some(v) => {
                // judge if is not a legal jackpot line
                let mut is_invalid_jackpot = false;
                if v.1 == 94 {
                    for i in 0..v.2 {
                        if reels[line_array[i as usize].as_u64().unwrap() as usize][i as usize] != 94 {
                            is_invalid_jackpot = true;
                            break;
                        }
                    }
                }
                if !is_invalid_jackpot {
                    lines.push(v);
                    wins += v.0 * line_bet;
                }
            },
            _ => {}
        }
    }
    return (lines, wins);
}

pub fn get_allway_winlines(reels:&Vec<Vec<i64>>, calculator:&Value, line_bet:i64)
    -> (Vec<(i64, Vec<usize>)>, i64)
{
    let (mut lines, mut total_wins) = (vec![], 0);

    struct Winline {
        kind:i64,
        num:Vec<usize>,  // icon nums of each col
    }

    let (rows, columns) = (reels.len(), reels[0].len());
    let mut winlines:Vec<Winline> = Vec::new();
    let wilds = match calculator.get("wilds") {
        Some(_v) => calculator.get_vec_i64("wilds"),
        _ => vec![]
    };
    let scatters = match calculator.get("scatters") {
        Some(_v) => calculator.get_vec_i64("scatters"),
        _ => vec![]
    };
    for i in 0..columns {
        if i == 0 {
            for j in 0..rows {
                let curr_sym = reels[j][i];
                let mut handled = false;
                for line in winlines.iter_mut() {
                    if curr_sym ==line.kind {
                        let length = line.num.len();
                        line.num[length - 1] += 1;
                        handled = true;
                        break;
                    }
                }
                if !handled && !scatters.contains(&curr_sym) {
                    let new_line = Winline {kind: curr_sym, num: vec![1]};
                    winlines.push(new_line);
                }
            }
        } else {
            for j in 0..rows {
                let curr_sym = reels[j][i];
                for line in winlines.iter_mut() {
                    if curr_sym == line.kind || wilds.contains(&curr_sym) {
                        let length = line.num.len();
                        if length == i + 1 {
                            line.num[length - 1] += 1;
                        } else {
                            line.num.push(1)
                        }
                        break;
                    }
                }
            }
        }
    }

    // keep lines with score
    let score = calculator.get_obj("score");
    for line in winlines {
        let score = match score.get(line.kind.to_string()) {
            Some(v) => {
                v.as_array().unwrap()[line.num.len()].as_i64().unwrap()
            },
            None => 0
        };
        if score > 0 {
            let mut num_multi:i64 = 1;
            let _:Vec<()> = line.num.iter().map(|v|{num_multi *= *v as i64;}).collect();
            let wins = num_multi * score * line_bet;
            lines.push((wins, line.num));
            total_wins += wins;
        }
    }
    return (lines, total_wins);
}

pub fn get_sc_lines(reels:&Vec<Vec<i64>>, calculator:&Value, bets:i64) -> Option<i64> {
    let scatters = match calculator.get("scatters") {
        Some(_v) => calculator.get_vec_i64("scatters"),
        None => vec![]
    };
    let (rows, columns) = (reels.len(), reels[0].len());
    let mut sc_icons = Vec::<usize>::new();
    for i in 0..rows {
        for j in 0..columns {
            if scatters.contains(&reels[i][j]) {
                sc_icons.push(i * columns + j);
            }
        }
    }
    let sc_score = match calculator.get("score").unwrap().get(scatters[0].to_string()) {
        Some(v) => v.as_array().unwrap()[sc_icons.len()].as_i64().unwrap(),
        None => 0
    };
    if sc_score > 0 {
        return Some(sc_score * bets);
    }
    return None;
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use super::get_winlines;

    #[test]
    fn test_get_winlines() {
        let reels = vec![vec![1,1,1,2,2],vec![1,92,1,2,2],vec![92,3,4,5,6]];
        let calculator = json!({
            "score": {
                "0": [ 0, 0, 0, 50, 100, 500 ],
                "1": [ 0, 0, 0, 30, 60, 300 ],
                "2": [ 0, 0, 0, 20, 40, 200 ],
                "3": [ 0, 0, 0, 12, 24, 120 ],
                "4": [ 0, 0, 0, 8, 16, 80 ],
                "5": [ 0, 0, 0, 5, 10, 50 ],
                "6": [ 0, 0, 0, 3, 6, 30 ],
                "7": [ 0, 0, 0, 2, 4, 20 ],
                "92": [ 0, 0, 0, 50, 100, 1000 ],
                "94": [ 0, 0, 0, 1, 3, 6 ]
            },
            "wilds": [ 92 ],
            "scatters": [ ],
            "scatterIsOnLine": false,
            "commonSignals": [ 0, 1, 2, 3, 4, 5, 6, 7 ]
        });
        let paylines = json!([
            [ 0, 0, 0, 0, 0 ],
            [ 1, 1, 1, 1, 1 ],
            [ 2, 2, 2, 2, 2 ],
            [ 0, 1, 2, 1, 0 ],
            [ 2, 1, 0, 1, 2 ],
            [ 1, 0, 0, 0, 1 ],
            [ 1, 2, 2, 2, 1 ],
            [ 0, 1, 0, 1, 0 ],
            [ 2, 1, 2, 1, 2 ]
        ]);
        let line_bet = 1;
        let r = get_winlines(&reels, &calculator, &paylines, line_bet);
        println!("{:?}", r.0);
    }
}