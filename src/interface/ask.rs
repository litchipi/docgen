use std::fmt::Display;
use std::io::Write;
use std::str::FromStr;

use crate::data::Transaction;
use crate::lang::LangDict;

// TODO    Use ratatui instead to perform these operations

pub fn ask_user<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    std::io::stdin()
        .read_line(&mut res)
        .expect("Error in string entered");
    res.trim().to_string()
}

pub fn ask_user_parse<T: Display, U: FromStr>(question: T) -> Option<U> {
    loop {
        let res = ask_user(&question);
        if res.is_empty() {
            return None;
        }
        if let Ok(res) = res.parse() {
            return Some(res);
        }
        println!("Unable to parse reply to {:?}", std::any::type_name::<U>());
    }
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

pub fn ask_for_transactions(lang: &LangDict) -> Vec<Transaction> {
    let desc = lang.get_doctype_word("general", "tx_item_description");
    let units = lang.get_doctype_word("general", "tx_units");
    let ppu = lang.get_doctype_word("general", "tx_price_per_unit");

    let mut tx = vec![];
    loop {
        println!("\nEnter data about transaction {}: ", tx.len() + 1);
        let descr = ask_user(format!("{desc}: "));
        if descr.is_empty() {
            break;
        }

        let units: Option<f64> = ask_user_parse(format!("{units}: "));
        if units.is_none() {
            break;
        }
        let units = units.unwrap();

        let ppu: Option<f64> = ask_user_parse(format!("{ppu}: "));
        if ppu.is_none() {
            break;
        }
        let ppu = ppu.unwrap();
        tx.push((descr, units, ppu));
    }
    tx
}
