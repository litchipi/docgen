use chrono::{DateTime, Datelike, Utc};
use std::path::PathBuf;

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

    pub fn get_doctype_word<D: ToString, W: ToString>(&self, subtable: D, word: W) -> String {
        let subtable = subtable.to_string();
        let word = word.to_string();
        self.data
            .get(&subtable)
            .unwrap_or_else(|| panic!("Unable to get lang table {subtable}"))
            .as_table()
            .unwrap_or_else(|| panic!("Unable to convert lang table {subtable} to table"))
            .get(&word)
            .unwrap_or_else(|| panic!("Unable to get word {subtable}:{word}"))
            .as_str()
            .unwrap_or_else(|| panic!("Unable to convert word {subtable}:{word} to String"))
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
