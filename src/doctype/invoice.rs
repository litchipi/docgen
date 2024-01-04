use typst::syntax::{Source, FileId, VirtualPath};

use crate::doc_config::DocumentConfig;
use crate::errors::Errcode;
use crate::world::TypstWorld;

use super::DocumentType;

pub struct InvoiceBuilder<'a> {
    source: String,
    config: &'a DocumentConfig,
}

impl<'a> Into<TypstWorld<'a>> for InvoiceBuilder<'a> {
    fn into(self) -> TypstWorld<'a> {
        let InvoiceBuilder { source , config } = self;
        TypstWorld {
            config,
            source: Source::new(FileId::new(None, VirtualPath::new("/source")), source),
            doctype: DocumentType::Invoice,
        }
    }
}

impl<'a> InvoiceBuilder<'a> {
    pub fn new(config: &'a DocumentConfig) -> InvoiceBuilder {
        InvoiceBuilder { source: String::new(), config, }
    }

    pub fn generate_invoice(&mut self) -> Result<(), Errcode> {
        // TODO    Generate the code for the invoice type
        //    Store inside the self.source buffer
        Ok(())
    }
}

pub fn generate<'a>(config: &'a DocumentConfig) -> Result<TypstWorld<'a>, Errcode> {
    let mut builder = InvoiceBuilder::new(config);
    builder.generate_invoice()?;
    Ok(builder.into())
}
