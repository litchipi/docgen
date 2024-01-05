use std::path::PathBuf;

use toml::{map::Map, Value};

use crate::{errors::Errcode, world::ConfigStore};

const NO_QUOTE_LIST: &[&str] = &["pt", "%"];

fn to_typst(val: &Value) -> String {
    if let Value::String(s) = val {
        if NO_QUOTE_LIST.iter().any(|suff| s.ends_with(suff)) {
            s.clone()
        } else {
            val.to_string()
        }
    } else {
        val.to_string()
    }
}

fn get_default_invoice_style() -> Map<String, Value> {
    let mut style = Map::new();
    style.insert("company_name_font_size".into(), "23pt".into());
    style
}

fn get_default_style() -> Map<String, Value> {
    let mut style = Map::new();
    style.insert("paper_type".into(), "a4".into());
    style.insert("margin_size".into(), "10%".into());
    style.insert("font_name".into(), "Roboto".into());
    style.insert("font_size".into(), "15pt".into());

    style.insert("invoice".into(), Value::Table(get_default_invoice_style()));
    style
}

fn generate_variable_for_table<F>(table: &Map<String, Value>, filter: F) -> String
where
    F: FnOnce(&String) -> bool + std::marker::Copy,
{
    let mut res = String::new();
    for (key, val) in table.iter() {
        if let Some(subtable) = val.as_table() {
            if !filter(key) {
                continue;
            }
            res += generate_variable_for_table(subtable, filter).as_str();
            res += "\n";
        } else {
            res += format!("#let {key}() = {}\n", to_typst(val)).as_str();
        }
    }
    res
}

pub fn generate_style_variables(
    config: &ConfigStore,
    stylefile: &PathBuf,
    doctype: String,
) -> Result<String, Errcode> {
    let style = if !stylefile.is_file() {
        let default_style = get_default_style();
        let default_style_str =
            toml::to_string(&toml::Value::Table(default_style.clone())).unwrap();
        std::fs::write(stylefile, default_style_str)?;
        default_style
    } else {
        let style_str = std::fs::read_to_string(stylefile)?;
        toml::from_str(&style_str)?
    };

    let vars = generate_variable_for_table(&style, |key| key == &doctype);
    Ok(vars)
}
