use crate::{errors::Errcode, doc_config::DocumentConfig, world::TypstWorld};

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
        match value.as_str() {
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
}
