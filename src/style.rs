use std::io::Read;
use std::path::PathBuf;

use toml::{map::Map, Value};
use typst::foundations::Bytes;
use typst::text::Font;

use crate::errors::Errcode;
use crate::utils::ask_user;

const NO_QUOTE_LIST: &[&str] = &["pt", "%", "fr"];
const FCT_LIST: &[&str] = &["rgb"];

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

fn get_default_invoice_style() -> Map<String, Value> {
    let mut style = Map::new();
    style.insert("company_name_font_size".into(), "23pt".into());
    style.insert("table_color".into(), "rgb(110, 140, 180, 205)".into());
    style.insert("tx_descr_width".into(), "3fr".into());
    style.insert("logo_width".into(), "150pt".into());
    style
}

fn get_default_style() -> Map<String, Value> {
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

pub fn import_style(stylefile: &PathBuf) -> Result<Map<String, Value>, Errcode> {
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
    style: &Map<String, Value>,
    doctype: String,
) -> String {
    generate_variable_for_table(&style, |key| key == &doctype)
}

pub fn import_fonts(style: &Map<String, Value>, fonts_dir: &PathBuf) -> Result<Vec<Font>, Errcode> {
    if !fonts_dir.exists() {
        std::fs::create_dir(fonts_dir)?;
    }

    // TODO    Do this every time we don't have the necessary font set from style
    let nb_fonts = std::fs::read_dir(fonts_dir)?.count();
    if nb_fonts == 0 {
        let font_name = style.get("font_name").unwrap().as_str().unwrap();
        let dl_from_dafont = ask_user(format!("No font installed in {fonts_dir:?}, do you wish to download {} from dafont ? [y/N] ", font_name));
        if dl_from_dafont.is_empty() || (dl_from_dafont.to_lowercase() == "y") {
            download_font(font_name, &fonts_dir.join(font_name))?;
        }
    }

    let mut all_fonts = vec![];
    for font_file in std::fs::read_dir(fonts_dir)? {
        let font_file = font_file?;
        let ftype = font_file.file_type()?;
        if ftype.is_dir() {
            all_fonts.extend(import_fonts(&style, &font_file.path())?);
        } else {
            // File or symlink
            let data = std::fs::read(font_file.path())?;
            all_fonts.extend(Font::iter(Bytes::from(data)));
        }
    }
    Ok(all_fonts)
}

fn download_font(font_name: &str, dir: &PathBuf) -> Result<(), Errcode> {
    std::fs::create_dir_all(dir)?;
    println!("Donwloading font {font_name}");
    let data = reqwest::blocking::get(format!("https://dl.dafont.com/dl/?f={}", font_name.to_lowercase()))?.bytes()?;
    let data = std::io::Cursor::new(data.as_ref());
    let mut archive = zip::ZipArchive::new(data)?;
    let all_fnames = archive.file_names().map(|s| s.to_string()).collect::<Vec<String>>();

    for fname in all_fnames {
        let mut file = archive.by_name(&fname)?;
        println!("Extracting file {}", file.name());
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)?;
        std::fs::write(dir.join(file.name()).with_extension("ttf"), buffer)?;
    }
    Ok(())
}
