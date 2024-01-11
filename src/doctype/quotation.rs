use std::collections::HashMap;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::codegen::{
    generate_header, generate_iban, generate_summary_table, generate_transaction_table,
    write_page_settings,
};
use crate::config::ConfigStore;
use crate::contact::Contact;
use crate::data::{Datastore, Date, Transaction};
use crate::errors::Errcode;
use crate::interface::ask::ask_for_transactions;
use crate::lang::LangDict;

use super::TypstData;

#[derive(Serialize, Deserialize)]
pub struct QuotationSavedData {
    pub history: HashMap<String, Vec<(QuotationInput, Option<usize>)>>,
}

impl QuotationSavedData {
    pub fn init() -> QuotationSavedData {
        QuotationSavedData {
            history: HashMap::new(),
        }
    }

    pub fn import(fname: &Path) -> QuotationSavedData {
        if !fname.is_file() {
            return QuotationSavedData::init();
        }

        let json_str =
            std::fs::read_to_string(fname).expect("Unable to read JSON data from {fname}");
        match serde_json::from_str::<Self>(json_str.as_str()) {
            Ok(d) => d,
            Err(_) => {
                println!("Failed to import the quotation data");
                QuotationSavedData::init()
            }
        }
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
            .ok_or(Errcode::ContactNotFound(slug.clone()))?;
        let data = data
            .get_mut(idx)
            .ok_or(Errcode::HistoryElementNotFound(idx))?;
        data.1 = Some(invoice_nb);
        Ok(())
    }

    pub fn add_quote(&mut self, quote: &QuotationInput) {
        if self.history.get(&quote.recipient).is_none() {
            self.history
                .insert(quote.recipient.clone(), vec![(quote.clone(), None)]);
        } else {
            self.history
                .get_mut(&quote.recipient)
                .unwrap()
                .push((quote.clone(), None));
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QuotationInput {
    pub recipient: String,
    pub created: Date,
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
            self.recipient, self.created,
        );
        if line.len() > 80 {
            line[..80].to_string() + "..."
        } else {
            line
        }
    }

    pub fn ask(_config: &ConfigStore, recipient: String, lang: &LangDict) -> QuotationInput {
        let current_date = Utc::now();
        let created = lang.get_date_fmt(&current_date);

        let tx = ask_for_transactions(lang);
        QuotationInput {
            recipient,
            created,
            tx,
        }
    }
}

pub struct QuotationBuilder<'a> {
    cfg: &'a ConfigStore,
    lang: &'a LangDict,
    data: &'a mut Datastore,
    inp: &'a QuotationInput,
}

impl<'a> QuotationBuilder<'a> {
    pub fn generate_quotation(&mut self) -> Result<(String, String), Errcode> {
        // Getting necessary data before writing the code
        self.data.quotes.add_quote(self.inp);
        let current_date = Utc::now();

        let fname = format!(
            "quotation_{}_{}_{}.pdf",
            self.inp.recipient,
            self.data.quotes.history.len(),
            current_date.format("%d%m%y"),
        );

        let footer = self.cfg.get_str("quotation", "footer");
        let mut source = "".to_string();
        write_page_settings(&mut source, footer);
        generate_header(self.cfg, &mut source);
        source += "#v(sep_par())\n";
        self.generate_metadata(&mut source);
        source += "#v(sep_par())\n";
        let total_price = generate_transaction_table(&mut source, &self.inp.tx, self.lang);
        source += "#v(sep_par())\n";
        generate_summary_table(&mut source, total_price, self.lang, self.cfg);
        source += "#v(sep_par())\n";
        source += format!(
            "=== {}\n",
            self.lang
                .get_doctype_word("quotation", "payment_conditions")
        )
        .as_str();
        source += self.cfg.get_str("quotation", "payment_conditions");
        source += "\n";

        if self.cfg.get_bool("quotation", "add_iban") {
            generate_iban(&mut source, self.lang, self.cfg);
        }

        source += "\n";
        Ok((fname, source))
    }

    fn generate_metadata(&self, source: &mut String) {
        *source += format!(
            "#grid(
            columns: (1fr, 1fr),
            column-gutter: 10%,
            align(left)[
                #text(17pt)[{}] \\
                {} \\ {} \\
            ],
            align(right)[
                {} \\#*{:0>5}* \\
                {} *{}* \\
            ],
        )",
            self.lang.get_doctype_word("quotation", "recipient_intro"),
            self.data.contacts.get(&self.inp.recipient).name,
            self.data.contacts.get(&self.inp.recipient).address,
            self.lang.get_doctype_word("quotation", "quotation_nb"),
            self.data.quotes.history.len(),
            self.lang.get_doctype_word("general", "creation_date"),
            self.inp.created,
        )
        .as_str();
    }
}

pub fn generate(
    cfg: &ConfigStore,
    lang: &LangDict,
    data: &mut Datastore,
) -> Result<TypstData, Errcode> {
    let recipient_slug = Contact::ask_slug();
    data.contacts.get_or_add(&recipient_slug);
    let inp = QuotationInput::ask(cfg, recipient_slug, lang);
    let mut builder = QuotationBuilder {
        cfg,
        lang,
        data,
        inp: &inp,
    };
    let (fname, result) = builder.generate_quotation()?;
    // For debug
    std::fs::write("/tmp/.typst_result.typ", &result)?;
    Ok(TypstData::new(fname, result))
}
