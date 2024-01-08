use std::path::PathBuf;

use crate::errors::Errcode;

pub mod invoice;

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
    // TODO Other document types
    // - devis
    // - contracts
    // - letter
}

impl DocumentType {
    pub fn generate_typst(&self, datadir: &PathBuf) -> Result<TypstData, Errcode> {
        if !datadir.exists() {
            std::fs::create_dir(datadir)?;
        }
        let dataf = datadir.join(self.to_string()).with_extension(".json");
        match self {
            DocumentType::Invoice => invoice::InvoiceBuilder::generate(dataf),
        }
    }
}

impl TryFrom<&String> for DocumentType {
    type Error = Errcode;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "invoice" => Ok(DocumentType::Invoice),
            _ => Err(Errcode::DocTypeUnsupported(value.clone())),
        }
    }
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Invoice => write!(f, "invoice"),
        }
    }
}
