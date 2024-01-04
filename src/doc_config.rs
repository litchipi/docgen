use std::collections::HashMap;

use comemo::Prehashed;
use typst::Library;
use typst::foundations::Bytes;
use typst::text::{Font, FontBook};

use crate::Args;
use crate::doctype::DocumentType;
use crate::errors::Errcode;

pub struct DocumentConfig {
    style: HashMap<DocumentType, Prehashed<Library>>,
    assets: HashMap<DocumentType, HashMap<String, Bytes>>,
    fonts: (Prehashed<FontBook>, Vec<Font>),
    datetime_offset: i64,
}

impl TryFrom<&Args> for DocumentConfig {
    type Error = Errcode;
    fn try_from(value: &Args) -> Result<Self, Errcode> {
        // TODO    IMPORTANT    Load all the necessary data from the args
        // - Load fonts from font dir
        // - Load all assets from assets dir
        //    - Each subdir for each doctype
        //    - Error if a subdir not know or file not in subdir
        // - Load style from style sheet
        //    - A default map
        //    - Each style key in a sub-map
        //    - Build library from the (default or specified) values, and the general config file
        // - Load datetime offset from config file
        todo!()
    }
}

impl<'a> DocumentConfig {
    pub fn get_library(&'a self, doctype: &DocumentType) -> &'a Prehashed<Library> {
        self.style.get(&doctype).unwrap()
    }

    pub fn get_font_book(&'a self) -> &'a Prehashed<FontBook> {
        &self.fonts.0
    }
}
