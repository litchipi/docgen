use std::path::PathBuf;
use chrono::{DateTime, Utc, Datelike};

use crate::errors::Errcode;

pub struct LangDict {
    data: toml::map::Map<String, toml::Value>,
}
impl LangDict {
    pub fn get_date_fmt(&self, date: &DateTime<Utc>) -> String {
        let month_list = self.data.get("months").unwrap().as_array().unwrap();
        let month_list: Vec<&str> = month_list.iter().map(|v| v.as_str().unwrap()).collect();
        let month_idx: usize = date.month().try_into().unwrap();
        format!(
            "{} {} {}",
            date.day(),
            month_list.get(month_idx).unwrap(),
            date.year(),
        )
    }

    pub fn get_doctype_word<D: ToString, W: ToString>(&self, doctype: D, word: W) -> String {
        let doctype = doctype.to_string();
        let word = word.to_string();
        self.data
            .get(&doctype)
            .unwrap()
            .as_table()
            .unwrap()
            .get(&word)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }
}

pub fn import_lang_profile(langf: &PathBuf) -> Result<LangDict, Errcode> {
    let data = if langf.is_file() {
        std::fs::read_to_string(langf)?
    } else {
        let s = include_str!("../default/lang.toml").to_string();
        std::fs::write(langf, &s)?;
        s
    };
    let data: toml::Value = toml::from_str(&data).expect("Error in lang definition");
    Ok(LangDict {
        data: data.as_table().unwrap().to_owned(),
    })
}
