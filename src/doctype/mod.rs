use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::ConfigStore;
use crate::data::Datastore;
use crate::errors::Errcode;
use crate::lang::LangDict;

pub mod invoice;
pub mod quotation;

pub struct TypstData {
    pub fname: String,
    pub code: String,
}

impl TypstData {
    pub fn new(fname: String, code: String) -> TypstData {
        TypstData { fname, code }
    }
}

#[derive(Hash, Debug, Clone, Eq, PartialEq, Copy)]
pub enum DocumentType {
    Invoice,
    Quotation,
    // TODO Other document types
    // - contracts
    // - letter
}

impl DocumentType {
    pub fn generate_typst(
        &self,
        cfg: &ConfigStore,
        lang: &LangDict,
        datadir: &Path,
    ) -> Result<TypstData, Errcode> {
        let mut data = Datastore::import(datadir);

        let res = match self {
            DocumentType::Invoice => invoice::generate(cfg, lang, &mut data),
            DocumentType::Quotation => quotation::generate(cfg, lang, &mut data),
        }?;
        data.export(datadir)?;
        Ok(res)
    }

    pub fn fname(&self, root: &Path) -> PathBuf {
        root.join(self.to_string()).with_extension("json")
    }

    pub fn export_data<T: Serialize>(&self, root: &Path, data: &T) -> Result<(), Errcode> {
        let fname = self.fname(root);
        std::fs::write(fname, serde_json::to_string(data)?)?;
        Ok(())
    }
}

impl TryFrom<&String> for DocumentType {
    type Error = Errcode;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "invoice" => Ok(DocumentType::Invoice),
            "quotation" => Ok(DocumentType::Quotation),
            _ => Err(Errcode::DocTypeUnsupported(value.clone())),
        }
    }
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Invoice => write!(f, "invoice"),
            DocumentType::Quotation => write!(f, "quotation"),
        }
    }
}
