use std::collections::HashMap;
use std::path::{Path, PathBuf};

use comemo::Prehashed;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::text::{Font, FontBook};
use typst::Library;

use crate::doctype::DocumentType;
use crate::errors::Errcode;
use crate::Args;

pub struct DocumentConfig {
    style: HashMap<DocumentType, Prehashed<Library>>,
    assets: HashMap<DocumentType, HashMap<PathBuf, Bytes>>,
    fonts: (Prehashed<FontBook>, Vec<Font>),
    datetime_offset: i64,
}

impl TryFrom<&Args> for DocumentConfig {
    type Error = Errcode;

    fn try_from(args: &Args) -> Result<Self, Errcode> {
        let fonts = import_fonts(&args.fonts_dir)?;
        let assets = import_assets(&args.assets_dir)?;
        let config = import_config(&args.config_file)?;
        let style = import_style(&args.style_sheet, &config)?;
        let datetime_offset = config.get("datetime_offset").unwrap().as_integer().unwrap();

        Ok(DocumentConfig {
            fonts,
            style,
            assets,
            datetime_offset,
        })
    }
}

impl<'a> DocumentConfig {
    pub fn get_library(&'a self, doctype: &DocumentType) -> &'a Prehashed<Library> {
        self.style.get(&doctype).unwrap()
    }

    pub fn get_font_book(&'a self) -> &'a Prehashed<FontBook> {
        &self.fonts.0
    }

    pub fn get_asset(&'a self, doctype: &DocumentType, path: &Path) -> FileResult<Bytes> {
        let all_assets = self.assets.get(doctype).unwrap();
        all_assets
            .get(path)
            .ok_or_else(|| FileError::NotFound(path.into()))
            .cloned()
    }

    pub fn get_font(&'a self, index: usize) -> Option<Font> {
        self.fonts.1.get(index).cloned()
    }

    // TODO    Get datetime from offset
    pub fn get_datetime(&'a self, _offset: i64) -> Datetime {
        Datetime::from_ymd(1970, 1, 1).unwrap()
    }
}

// TODO    Import fonts from directory
fn import_fonts(fonts_dir: &PathBuf) -> Result<(Prehashed<FontBook>, Vec<Font>), Errcode> {
    // let fonts = fonts_dir
    //     .files()
    //     .flat_map(|file| Font::iter(file.contents().into()))
    //     .collect();
    // let book = FontBook::from_fonts(&fonts);
    // (Prehashed::new(book), fonts)
    todo!();
}

// TODO    Import assets from directory
//    - Each subdir for each doctype
//    - Error if a subdir not know or file not in subdir
fn import_assets(
    assets_dir: &PathBuf,
) -> Result<HashMap<DocumentType, HashMap<PathBuf, Bytes>>, Errcode> {
    // TODO    Must assert that all DocumentType variants are represented in the hashmap
    Ok(HashMap::new())
}

// TODO    Import configuration from file
//    - Default configuration, updated with the one loaded from the file
fn import_config(config_file: &PathBuf) -> Result<HashMap<String, toml::Value>, Errcode> {
    Ok(HashMap::new())
}

// TODO    Import style
//    - A default map
//    - Each style key in a sub-map
//    - Build library from the (default or specified) values, and the general config file
fn import_style(
    stylesheet: &PathBuf,
    config: &HashMap<String, toml::Value>,
) -> Result<HashMap<DocumentType, Prehashed<Library>>, Errcode> {
    Ok(HashMap::new())
}
