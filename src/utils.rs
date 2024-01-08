use std::{fmt::Display, io::Write, rc::Rc, path::PathBuf, str::FromStr};

use chrono::{Utc, DateTime, Datelike};

use crate::errors::Errcode;

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
            return None
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

pub type LangDict = Rc<LangWrapper>;
pub struct LangWrapper {
    data: toml::map::Map<String, toml::Value>,
}
impl LangWrapper {
    pub fn get_date_fmt(&self, date: &DateTime<Utc>) -> String {
        let month_list = self.data.get("months").unwrap().as_array().unwrap();
        let month_list: Vec<&str> = month_list.into_iter().map(|v| v.as_str().unwrap()).collect();
        let month_idx: usize = date.month().try_into().unwrap();
        format!("{} {} {}",
            date.day(),
            month_list.get(month_idx).unwrap(),
            date.year(),
        )
    }

    pub fn get_doctype_word<D: ToString, W: ToString>(&self, doctype: D, word: W) -> String {
        let doctype = doctype.to_string();
        let word = word.to_string();
        self.data.get(&doctype).unwrap().as_table().unwrap().get(&word).unwrap().as_str().unwrap().to_string()
    }
}

pub fn import_lang_profile(langf: &PathBuf) -> Result<LangDict, Errcode> {
    let data = if langf.is_file() {
        std::fs::read_to_string(&langf)?
    } else {
        let s = include_str!("../default/lang.toml").to_string();
        std::fs::write(langf, &s)?;
        s
    };
    let data : toml::Value = toml::from_str(&data).expect("Error in lang definition");
    Ok(Rc::new(LangWrapper {
        data: data.as_table().unwrap().to_owned(),
    }))
}
