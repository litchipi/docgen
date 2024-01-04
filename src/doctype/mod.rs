use typst::World;

use crate::{errors::Errcode, doc_config::DocumentConfig};

pub mod invoice;

#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub enum DocumentType {
    Invoice,
    // TODO
    // - devis
    // - contracts
    // - letter
}

impl TryFrom<String> for DocumentType {
    type Error = Errcode;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            _ => Err(Errcode::DocTypeUnsupported(value)),
        }
    }
}

impl DocumentType {
    pub fn dispatch(&self) -> Result<String, Errcode> {
        match self {
            DocumentType::Invoice => invoice::generate_invoice(),
        }
    }

    pub fn generate_world(&self, doc_config: &DocumentConfig, source: String) -> Box<dyn World> {
        match self {
            DocumentType::Invoice => Box::new(invoice::InvoiceWorld::new(doc_config, source)),
        }
    }
}
