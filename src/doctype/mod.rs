use std::collections::HashMap;

use crate::{doc_config::DocumentConfig, errors::Errcode, world::TypstWorld};

pub mod invoice;

#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub enum DocumentType {
    Invoice,
    // TODO Other document types
    // - devis
    // - contracts
    // - letter
}

impl TryFrom<String> for DocumentType {
    type Error = Errcode;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "invoice" => Ok(DocumentType::Invoice),
            _ => Err(Errcode::DocTypeUnsupported(value)),
        }
    }
}

impl<'a> DocumentType {
    pub fn generate_typst(&'a self, config: &'a DocumentConfig) -> Result<TypstWorld, Errcode> {
        match self {
            DocumentType::Invoice => invoice::generate(config),
        }
    }

    pub fn init_empty_store<T: Clone>(init_val: T) -> HashMap<DocumentType, T> {
        let mut store = HashMap::new();
        store.insert(DocumentType::Invoice, init_val.clone());
        store
    }
}
