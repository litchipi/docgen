use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::codegen::{sanitize, write_page_settings};
use crate::config::ConfigStore;
use crate::contact::{Contact, ContactBook};
use crate::data::Datastore;
use crate::errors::Errcode;
use crate::interface::select_from_list;
use crate::interface::utils::{ask_user, ask_user_nonempty, ask_user_parse};
use crate::lang::LangDict;

use super::quotation::{QuotationInput, QuotationSavedData};
use crate::doctype::TypstData;

#[derive(Serialize, Deserialize)]
pub struct InvoiceSavedData {
    pub history: Vec<InvoiceInput>,
}

impl InvoiceSavedData {
    pub fn init() -> InvoiceSavedData {
        InvoiceSavedData { history: vec![] }
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
    pub recipient: String,
    pub quote_nb: Option<usize>,
    date_sell: String,
    tx: Vec<(String, f64, f64)>,
    tax_rate: Option<f64>,
}

impl InvoiceInput {
    pub fn from_quote(config: &ConfigStore, idx: usize, quote: &QuotationInput) -> InvoiceInput {
        let tax_rate = if config.get_bool("invoice", "tax_applicable") {
            Some(config.get_float("invoice", "tax_rate"))
        } else {
            None
        };
        InvoiceInput {
            recipient: quote.recipient.slug.clone(),
            tx: quote.tx.clone(),
            date_sell: ask_user_nonempty("Enter the date where the sell was done: "),
            quote_nb: Some(idx),
            tax_rate,
        }
    }
    pub fn ask(config: &ConfigStore, recipient: Contact, lang: &LangDict) -> InvoiceInput {
        let date_sell = ask_user_nonempty("Enter the date where the sell was done: ");

        let word_desc = lang.get_doctype_word("invoice", "tx_item_description");
        let word_units = lang.get_doctype_word("invoice", "tx_units");
        let word_ppu = lang.get_doctype_word("invoice", "tx_price_per_unit");

        let mut tx = vec![];
        loop {
            println!("\nEnter data about transaction {}: ", tx.len() + 1);
            let descr = ask_user(format!("{word_desc}: "));
            if descr.is_empty() {
                break;
            }

            let units: Option<f64> = ask_user_parse(format!("{word_units}: "));
            if units.is_none() {
                break;
            }
            let units = units.unwrap();

            let ppu: Option<f64> = ask_user_parse(format!("{word_ppu}: "));
            if ppu.is_none() {
                break;
            }
            let ppu = ppu.unwrap();
            tx.push((descr, units, ppu));
        }

        let tax_rate = if config.get_bool("invoice", "tax_applicable") {
            Some(config.get_float("invoice", "tax_rate"))
        } else {
            None
        };

        InvoiceInput {
            recipient: recipient.slug.clone(),
            quote_nb: None,
            date_sell,
            tx,
            tax_rate,
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

        let footer = self.cfg.get("invoice", "footer").as_str().unwrap();
        let mut source = "".to_string();
        write_page_settings(&mut source, footer);
        self.generate_header(&mut source);
        source += "#v(sep_par())\n";
        self.generate_metadata(&mut source, &current_date);
        source += "#v(sep_par())\n";
        let total_price = self.generate_transaction_table(&mut source);
        source += "#v(sep_par())\n";
        self.generate_summary_table(&mut source, total_price);
        source += "#v(sep_par())\n";

        if self.cfg.get_bool("invoice", "add_iban") {
            self.generate_iban(&mut source);
        }

        source += "\n";
        Ok((fname, source))
    }

    fn generate_header(&self, source: &mut String) {
        *source += "#let sep_par() = 28pt\n";
        let logo_path = self.cfg.get_company("logo_path");
        let logo = if logo_path.is_empty() {
            "".to_string()
        } else {
            format!("#image(\"{logo_path}\", width: logo_width())")
        };

        let writing_logo_path = self.cfg.get_company("logo_writing");
        let writing_logo = if writing_logo_path.is_empty() {
            format!(
                "#text(company_name_font_size())[*{}*]",
                self.cfg.get_company("name")
            )
        } else {
            format!("#image(\"{writing_logo_path}\", width: logo_width())")
        };

        *source += format!(
            "#grid(
            columns: (1fr, auto),
            align(left)[{writing_logo}],
            align(right)[{logo}]
        )\n"
        )
        .as_str();

        *source += format!(
            "#align(left)[
            {} \\ {} \\ {} \\ {} \\ SIRET: {}
        ]\n",
            sanitize(&self.cfg.get_company("person_name")),
            sanitize(&self.cfg.get_company("address")),
            sanitize(&self.cfg.get_company("email")),
            sanitize(&self.cfg.get_company("legal_status")),
            sanitize(&self.cfg.get_company("siret_number")),
        )
        .as_str();
    }

    fn generate_metadata(&self, source: &mut String, current_date: &DateTime<Utc>) {
        let current_date_fmt = self.lang.get_date_fmt(current_date);

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
                {}: *{}*
            ],
        )",
            self.lang.get_doctype_word("invoice", "recipient_intro"),
            self.data.contacts.get(&self.inp.recipient).name,
            self.data.contacts.get(&self.inp.recipient).address,
            self.lang.get_doctype_word("invoice", "invoice_nb"),
            self.data.invoices.history.len(),
            self.lang.get_doctype_word("invoice", "creation_date"),
            current_date_fmt,
            self.lang.get_doctype_word("invoice", "sell_date"),
            self.inp.date_sell,
        )
        .as_str();
    }

    fn generate_transaction_table(&self, source: &mut String) -> f64 {
        let word_desc = self.lang.get_doctype_word("invoice", "tx_item_description");
        let word_units = self.lang.get_doctype_word("invoice", "tx_units");
        let word_ppu = self.lang.get_doctype_word("invoice", "tx_price_per_unit");
        let word_total = self.lang.get_doctype_word("invoice", "total_price");
        let curr_sym = self.lang.get_doctype_word("general", "currency_symbol");
        *source += format!(
            "#table(
            stroke: table_color(),
            columns: (tx_descr_width(), 1fr, 1fr, 1fr),
            [*{word_desc}*], [*{word_units}*], [*{word_ppu}*], [*{word_total}*],
        "
        )
        .as_str();

        let mut total_price = 0.0;
        for (descr, units, ppu) in self.inp.tx.iter() {
            let total = units * ppu;
            *source += format!(
                "
                \"{descr}\", \"{units}\", \"{ppu:.2} {curr_sym}\", \"{total:.2} {curr_sym}\",
            "
            )
            .as_str();
            total_price += total;
        }

        *source += ")\n";
        total_price
    }

    fn generate_summary_table(&self, source: &mut String, total_price: f64) {
        let curr_sym = self.lang.get_doctype_word("general", "currency_symbol");
        let (tax_fmt, tax_amnt) = if self.cfg.get_bool("invoice", "tax_applicable") {
            let tax_rate: f64 = self.cfg.get_float("invoice", "tax_rate");
            let amnt = total_price * tax_rate;
            (
                format!(
                    "[*{} {:.2}%*], [{:.2} {curr_sym}]",
                    self.lang.get_doctype_word("invoice", "tax_name"),
                    tax_rate * 100.0,
                    amnt,
                ),
                amnt,
            )
        } else {
            (
                format!(
                    "[*{}*], []",
                    self.lang.get_doctype_word("invoice", "tax_not_applicable")
                ),
                0.0,
            )
        };

        *source += format!(
            "#table(
            stroke: table_color(),
            columns: (auto, auto),
            [*{}*], [{total_price:.2} {curr_sym}],
            {tax_fmt},
            [*{}*], [{:.2} {curr_sym}],
        )",
            self.lang.get_doctype_word("invoice", "total_price_no_tax"),
            self.lang
                .get_doctype_word("invoice", "total_price_with_tax"),
            total_price + tax_amnt
        )
        .as_str();
    }

    fn generate_iban(&self, source: &mut String) {
        *source += format!(
            "
            === {}

            #table(
                stroke: table_color(),
                columns: (auto, auto),
                [*{}*], [{}],
                [*IBAN*], [{}],
                [*BIC*], [{}],
            )",
            self.lang.get_doctype_word("invoice", "iban_title"),
            self.lang.get_doctype_word("invoice", "iban_bank"),
            self.cfg.get("bank", "name").as_str().unwrap(),
            self.cfg.get("bank", "iban").as_str().unwrap(),
            self.cfg.get("bank", "bic").as_str().unwrap(),
        )
        .as_str();
    }
}

fn get_inputs(
    config: &ConfigStore,
    quotedata: &QuotationSavedData,
    lang: &LangDict,
    contacts: &mut ContactBook,
) -> InvoiceInput {
    let slug = Contact::ask_slug();
    if let Some(qhist) = quotedata.history.get(&slug) {
        let qhist = qhist
            .iter()
            .enumerate()
            .filter(|(_, (_, i))| i.is_none())
            .collect::<Vec<(usize, &(QuotationInput, Option<usize>))>>();
        let filtered_idx = select_from_list(&qhist, |(_, (inp, _))| inp.single_line_display());
        let idx = qhist.get(filtered_idx).unwrap().0;
        let quote = &qhist.get(filtered_idx).unwrap().1 .0;
        InvoiceInput::from_quote(config, idx, quote)
    } else {
        let recipient = contacts.get_or_add(slug);
        InvoiceInput::ask(config, recipient, lang)
    }
}

pub fn generate(
    cfg: &ConfigStore,
    lang: &LangDict,
    data: &mut Datastore,
) -> Result<TypstData, Errcode> {
    let inp = get_inputs(cfg, &data.quotes, lang, &mut data.contacts);
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
        data.quotes
            .mark_quotation_finished(&inp.recipient, quote_nb, invoice_nb)?;
    }
    Ok(TypstData::new(fname, result))
}
