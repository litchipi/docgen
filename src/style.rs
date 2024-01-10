
use std::path::PathBuf;

use toml::{map::Map, Value};

use crate::errors::Errcode;


const NO_QUOTE_LIST: &[&str] = &["pt", "%", "fr"];
const FCT_LIST: &[&str] = &["rgb"];

pub type Style = Map<String, Value>;

fn to_typst(val: &Value) -> String {
    if let Value::String(s) = val {
        let is_fct = FCT_LIST.iter().any(|pref| s.starts_with(pref));
        let is_metric = NO_QUOTE_LIST.iter().any(|suff| s.ends_with(suff));
        if is_fct || is_metric {
            s.clone()
        } else {
            val.to_string()
        }
    } else {
        val.to_string()
    }
}

fn get_default_invoice_style() -> Style {
    let mut style = Map::new();
    style.insert("company_name_font_size".into(), "23pt".into());
    style.insert("table_color".into(), "rgb(110, 140, 180, 205)".into());
    style.insert("tx_descr_width".into(), "3fr".into());
    style.insert("logo_width".into(), "150pt".into());
    style
}

fn get_default_style() -> Style {
    let mut style = Map::new();
    style.insert("paper_type".into(), "a4".into());
    style.insert("font_name".into(), "Roboto".into());
    style.insert("font_size".into(), "15pt".into());
    style.insert("margin_top".into(), "8%".into());
    style.insert("margin_x".into(), "4%".into());
    style.insert("margin_bottom".into(), "2%".into());
    style.insert("footer_font_size".into(), "10pt".into());

    style.insert("invoice".into(), Value::Table(get_default_invoice_style()));
    style
}

fn generate_variable_for_style<F>(style: &Style, filter: F) -> String
where
    F: FnOnce(&String) -> bool + std::marker::Copy,
{
    let mut res = String::new();
    for (key, val) in style.iter() {
        if let Some(subtable) = val.as_table() {
            if !filter(key) {
                continue;
            }
            res += generate_variable_for_style(subtable, filter).as_str();
            res += "\n";
        } else {
            res += format!("#let {key}() = {}\n", to_typst(val)).as_str();
        }
    }
    res
}

pub fn import_style(stylefile: &PathBuf) -> Result<Style, Errcode> {
    if !stylefile.is_file() {
        let default_style = get_default_style();
        let default_style_str =
            toml::to_string(&toml::Value::Table(default_style.clone())).unwrap();
        std::fs::write(stylefile, default_style_str)?;
        Ok(default_style)
    } else {
        let style_str = std::fs::read_to_string(stylefile)?;
        Ok(toml::from_str(&style_str)?)
    }
}

pub fn generate_style_variables(
    style: &Style,
    doctype: String,
) -> String {
    generate_variable_for_style(&style, |key| key == &doctype)
}
