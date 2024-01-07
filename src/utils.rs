use std::{fmt::Display, io::Write};

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
