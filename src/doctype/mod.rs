use std::path::PathBuf;

use crate::errors::Errcode;

pub mod invoice;

#[derive(Hash, Debug, Clone, Eq, PartialEq, Copy)]
pub enum DocumentType {
    Invoice,
    // TODO Other document types
    // - devis
    // - contracts
    // - letter
}

impl DocumentType {
    pub fn generate_typst(&self, history: &PathBuf) -> Result<String, Errcode> {
        if !history.exists() {
            std::fs::create_dir(history)?;
        }
        match self {
            DocumentType::Invoice => invoice::InvoiceBuilder::generate(history.join("invoice.json")),
        }
    }

    pub fn all_variants() -> Vec<(DocumentType, &'static str)> {
        vec![
            (DocumentType::Invoice, "invoice"),
        ]
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
