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
    pub fn generate_typst(&self) -> Result<String, Errcode> {
        match self {
            DocumentType::Invoice => invoice::InvoiceBuilder::generate(),
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
