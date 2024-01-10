use std::collections::HashMap;
use std::path::Path;

use crate::contact::Contact;
use crate::errors::Errcode;
use crate::Transaction;

pub struct QuotationSavedData {
    pub history: HashMap<String, Vec<(QuotationInput, Option<usize>)>>,
}

impl QuotationSavedData {
    pub fn import(_root: &Path) -> QuotationSavedData {
        QuotationSavedData {
            history: HashMap::new(),
        }
    }

    pub fn export(&self, _root: &Path) -> Result<(), Errcode> {
        Ok(())
    }

    pub fn mark_quotation_finished(
        &mut self,
        slug: &String,
        idx: usize,
        invoice_nb: usize,
    ) -> Result<(), Errcode> {
        let data = self
            .history
            .get_mut(slug)
            .ok_or(Errcode::SlugNotFound(slug.clone()))?;
        let data = data
            .get_mut(idx)
            .ok_or(Errcode::HistoryElementNotFound(idx))?;
        data.1 = Some(invoice_nb);
        Ok(())
    }
}

pub struct QuotationInput {
    pub created: String,
    pub recipient: Contact,
    pub tx: Vec<Transaction>,
}

impl QuotationInput {
    pub fn single_line_display(&self) -> String {
        let total_price: f64 = self.tx.iter().map(|(_, u, p)| u * p).sum();
        let descr = self
            .tx
            .iter()
            .map(|(d, _, _)| d.clone())
            .collect::<Vec<String>>()
            .join(", ");
        let line = format!(
            "{} {} {total_price:.2}€ : {descr}",
            self.recipient.slug, self.created,
        );
        if line.len() > 80 {
            line[..80].to_string() + "..."
        } else {
            line
        }
    }
}
