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
    pub fn import(root: &Path) -> Datastore {
        if !root.exists() {
            std::fs::create_dir(root).expect("Unable to create data directory");
        }

        let contacts = ContactBook::import(root);
        let invoices = InvoiceSavedData::import(&DocumentType::Invoice.fname(root));
        let quotes = QuotationSavedData::import(&DocumentType::Quotation.fname(root));
        Datastore {
            contacts,
            invoices,
            quotes,
        }
    }

    pub fn export(&self, root: &Path) -> Result<(), Errcode> {
        DocumentType::Invoice.export_data(root, &self.invoices)?;
        DocumentType::Quotation.export_data(root, &self.quotes)?;
        self.contacts.export(root)?;
        Ok(())
    }
}
