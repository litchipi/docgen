use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::codegen::{
    generate_header, generate_iban, generate_summary_table, generate_transaction_table,
    write_page_settings,
};
use crate::config::ConfigStore;
use crate::contact::Contact;
use crate::data::{Datastore, Date};
use crate::errors::Errcode;
use crate::interface::ask::{ask_for_transactions, ask_user_nonempty};
use crate::interface::select_from_list;
use crate::lang::LangDict;

use crate::doctype::quotation::QuotationInput;
use crate::doctype::TypstData;

#[derive(Serialize, Deserialize)]
pub struct InvoiceSavedData {
    pub history: Vec<InvoiceInput>,
    pub id_counter: usize,
}

impl InvoiceSavedData {
    pub fn init() -> InvoiceSavedData {
        InvoiceSavedData { id_counter: 1, history: vec![] }
    }

    pub fn import(fname: &Path) -> InvoiceSavedData {
        if !fname.is_file() {
            return InvoiceSavedData::init();
        }

        let json_str =
            std::fs::read_to_string(fname).expect("Unable to read JSON data from {fname}");
        match serde_json::from_str::<Self>(json_str.as_str()) {
            Ok(d) => d,
            Err(_) => {
                println!("Failed to import the invoice data");
                InvoiceSavedData::init()
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InvoiceInput {
    pub id: usize,
    pub recipient: String,
    pub quote_nb: Option<usize>,
    date_sell: Date,
    tx: Vec<(String, f64, f64)>,
    tax_rate: Option<f64>,
    created: String,
}

impl InvoiceInput {
    pub fn from_quote(id: usize, config: &ConfigStore, lang: &LangDict, idx: usize, quote: &QuotationInput) -> InvoiceInput {
        let current_date = Utc::now();
        let created = lang.get_date_fmt(&current_date);
        let tax_rate = if config.get_bool("taxes", "tax_applicable") {
            Some(config.get_float("taxes", "tax_rate"))
        } else {
            None
        };
        InvoiceInput {
            id,
            recipient: quote.recipient.clone(),
            tx: quote.tx.clone(),
            date_sell: ask_user_nonempty("Enter the date where the sell was done: "),
            quote_nb: Some(idx),
            tax_rate,
            created,

        }
    }
    pub fn ask(id: usize, recipient: String, config: &ConfigStore, lang: &LangDict) -> InvoiceInput {
        let current_date = Utc::now();
        let created = lang.get_date_fmt(&current_date);
        let date_sell = ask_user_nonempty("Enter the date where the sell was done: ");

        let tx = ask_for_transactions(lang);
        let tax_rate = if config.get_bool("taxes", "tax_applicable") {
            Some(config.get_float("taxes", "tax_rate"))
        } else {
            None
        };

        InvoiceInput {
            id,
            recipient,
            quote_nb: None,
            date_sell,
            tx,
            tax_rate,
            created,
        }
    }
}

pub struct InvoiceBuilder<'a> {
    cfg: &'a ConfigStore,
    lang: &'a LangDict,
    data: &'a mut Datastore,
    inp: &'a InvoiceInput,
}

impl<'a> InvoiceBuilder<'a> {
    pub fn generate_invoice(&mut self) -> Result<(String, String), Errcode> {
        // Getting necessary data before writing the code
        self.data.invoices.history.push(self.inp.clone());
        let current_date = Utc::now();

        let fname = format!(
            "invoice_{}_{}_{}.pdf",
            self.inp.recipient,
            self.data.invoices.history.len(),
            current_date.format("%d%m%y"),
        );

        let footer = self.cfg.get_str("invoice", "footer");
        let mut source = "".to_string();
        write_page_settings(&mut source, footer);
        generate_header(self.cfg, &mut source);
        source += "#v(sep_par())\n";
        self.generate_metadata(&mut source, &current_date);
        source += "#v(sep_par())\n";
        let total_price = generate_transaction_table(&mut source, &self.inp.tx, self.lang);
        source += "#v(sep_par())\n";
        generate_summary_table(&mut source, total_price, self.lang, self.cfg);
        source += "#v(sep_par())\n";

        if self.cfg.get_bool("invoice", "add_iban") {
            generate_iban(&mut source, self.lang, self.cfg);
        }

        source += "\n";
        Ok((fname, source))
    }

    fn generate_metadata(&self, source: &mut String, current_date: &DateTime<Utc>) {
        let current_date_fmt = self.lang.get_date_fmt(current_date);

        let quotation_md = if let Some(nb) = self.inp.quote_nb {
            format!("\\\n\t{} \\#*{}{:0>5}*",
                self.lang.get_doctype_word("invoice", "quotation_related"),
                self.cfg.get_str("quotation", "id_prefix"),
                nb
            )
        } else {
            "".to_string()
        };

        *source += format!(
            "#grid(
            columns: (1fr, 1fr),
            column-gutter: 10%,
            align(left)[
                #text(17pt)[{}] \\
                {} \\ {} \\
            ],
            align(right)[
                {} \\#*{}{:0>5}* \\
                {} *{}* \\
                {}: *{}* {quotation_md}
            ],
        )",
            self.lang.get_doctype_word("invoice", "recipient_intro"),
            self.data.contacts.get(&self.inp.recipient).name,
            self.data.contacts.get(&self.inp.recipient).address,
            self.lang.get_doctype_word("invoice", "invoice_nb"),
            self.cfg.get_str("invoice", "id_prefix"),
            self.data.invoices.history.len(),
            self.lang.get_doctype_word("general", "creation_date"),
            current_date_fmt,
            self.lang.get_doctype_word("general", "sell_date"),
            self.inp.date_sell,
        )
        .as_str();
    }
}

pub fn generate(
    cfg: &ConfigStore,
    lang: &LangDict,
    data: &mut Datastore,
) -> Result<TypstData, Errcode> {
    let id = data.invoices.id_counter;
    data.invoices.id_counter += 1;

    let slug = Contact::ask_slug();
    data.contacts.get_mut(&slug).invoices.push(id);

    let inp = if let Some(qhist) = data.quotations.history.get(&slug) {
        let qhist = qhist
            .iter()
            .enumerate()
            .filter(|(_, (_, i))| i.is_none())
            .collect::<Vec<(usize, &(QuotationInput, Option<usize>))>>();
        let filtered_idx = select_from_list(&qhist, |(_, (inp, _))| inp.single_line_display());
        let idx = qhist.get(filtered_idx).unwrap().0;
        let quote = &qhist.get(filtered_idx).unwrap().1 .0;
        InvoiceInput::from_quote(id, cfg, lang, idx, quote)
    } else {
        let recipient = data.contacts.get_or_add(&slug);
        InvoiceInput::ask(id, recipient.slug, cfg, lang)
    };

    let mut builder = InvoiceBuilder {
        cfg,
        lang,
        data,
        inp: &inp,
    };
    let (fname, result) = builder.generate_invoice()?;
    // For debug
    std::fs::write("/tmp/.typst_result.typ", &result)?;

    if let Some(quote_nb) = inp.quote_nb {
        let invoice_nb = data.invoices.history.len();
        data.quotations
            .mark_quotation_finished(&inp.recipient, quote_nb, invoice_nb)?;
    }
    Ok(TypstData::new(fname, result))
}
