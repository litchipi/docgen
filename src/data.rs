use std::path::Path;

use crate::contact::ContactBook;
use crate::doctype::invoice::InvoiceSavedData;
use crate::doctype::quotation::QuotationSavedData;
use crate::doctype::DocumentType;
use crate::errors::Errcode;

pub struct Datastore {
    pub contacts: ContactBook,
    pub invoices: InvoiceSavedData,
    pub quotes: QuotationSavedData,
}

impl Datastore {
    pub fn import(root: &Path) -> Result<Datastore, Errcode> {
        if !root.exists() {
            std::fs::create_dir(root)?;
        }

        let contacts = ContactBook::import(root)?;
        let invoices = InvoiceSavedData::import(&DocumentType::Invoice.fname(root));
        let quotes = QuotationSavedData::import(&DocumentType::Quotation.fname(root));
        Ok(Datastore {
            contacts,
            invoices,
            quotes,
        })
    }

    pub fn export(&self, root: &Path) -> Result<(), Errcode> {
        self.invoices.export(root)?;
        self.quotes.export(root)?;
        self.contacts.export(root)?;
        Ok(())
    }
}
