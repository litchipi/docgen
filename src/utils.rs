use std::{fmt::Display, io::Write};

use chrono::{DateTime, Utc, Datelike};

pub fn ask_user<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    std::io::stdin()
        .read_line(&mut res)
        .expect("Error in string entered");
    res
}

pub fn ask_user_nonempty<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    let mut init = true;
    while init || res.is_empty() {
        std::io::stdin()
            .read_line(&mut res)
            .expect("Error in string entered");
        if !init {
            println!("Answer is empty");
        }
        init = false;
    }
    res.trim().to_string()
}

pub fn map_get_str_or_ask<T: Display>(map: &serde_json::Map<String, serde_json::Value>, slug: &str, question: T, non_empty: bool) -> String {
    match map.get(slug).map(|n| n.as_str()).flatten() {
        Some(n) => n.to_string(),
        None => if non_empty {
            ask_user_nonempty(question)
        } else {
            ask_user(question)
        }
    }
}

pub fn get_month_name(date: &DateTime<Utc>) -> String {
    match std::env::var("LANG").ok().map(|v| v.split(".").nth(0).map(|s| s.to_string())).flatten() {
        Some(s) => match s.as_str() {
            "fr_FR" => get_month_name_french(date.month()).into(),
            _ => date.format("%b").to_string(),
        },
        _ => date.format("%b").to_string(),
    }
}

fn get_month_name_french(month: u32) -> &'static str {
    match month {
        1 => "Janvier",
        2 => "Février",
        3 => "Mars",
        4 => "Avril",
        5 => "Mai",
        6 => "Juin",
        7 => "Juillet",
        8 => "Août",
        9 => "Septembre",
        10 => "Octobre",
        11 => "Novembre",
        12 => "Décembre",
        _ => unreachable!(),
    }
}
