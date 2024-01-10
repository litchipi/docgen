use std::{collections::HashMap, path::PathBuf, rc::Rc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    codegen::{sanitize, write_page_settings},
    config::ConfigStore,
    contact::{Contact, ContactBook},
    errors::Errcode,
    utils::{ask_user, ask_user_nonempty, ask_user_parse, LangDict},
};

use super::TypstData;

pub struct InvoiceInput {
    recipient: Rc<Contact>,
    date_sell: String,
    tx: Vec<(String, f64, f64)>,
}

impl InvoiceInput {
    pub fn new(
        rslug: String,
        rname: String,
        raddr: String,
        date_sell: String,
        tx: Vec<(String, f64, f64)>,
    ) -> InvoiceInput {
        InvoiceInput {
            recipient: Rc::new(Contact::new(rslug, rname, raddr)),
            date_sell,
            tx,
        }
    }

    pub fn ask(lang: LangDict, known: Option<&ContactBook>) -> InvoiceInput {
        let slug = Contact::ask_slug();
        let recipient = if let Some(contact) = known.map(|k| k.get(&slug)).flatten() {
            (*contact).clone()
        } else {
            println!("Enter the informations related to the recipient");
            Rc::new(Contact::ask(Some(slug)))
        };

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
        InvoiceInput {
            recipient,
            date_sell,
            tx,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct InvoiceSavedData {
    invoice_total_count: usize,

    #[serde(serialize_with = "crate::utils::serialize_hashmap_rc")]
    #[serde(deserialize_with = "crate::utils::deserialize_hashmap_rc")]
    recipients_known: HashMap<String, Rc<Contact>>,
}

impl InvoiceSavedData {
    pub fn init() -> InvoiceSavedData {
        InvoiceSavedData {
            invoice_total_count: 0,
            recipients_known: HashMap::new(),
        }
    }

    pub fn partial_import(map: &Map<String, Value>) -> InvoiceSavedData {
        let mut recipients_known = HashMap::new();
        if let Some(data) = map.get("recipients_known").and_then(|n| n.as_object()) {
            for (key, val) in data.iter() {
                let Some(val) = val.as_object() else {
                    println!("WARN Unable to import data for recipient {key}");
                    continue;
                };

                let contact = Contact::partial_import(key.clone(), val);
                recipients_known.insert(key.clone(), Rc::new(contact));
            }
        }
        InvoiceSavedData {
            invoice_total_count: 0,
            recipients_known,
        }
    }
}

pub struct InvoiceBuilder {
    cfg: ConfigStore,
    data: InvoiceSavedData,
    lang: LangDict,
}

impl InvoiceBuilder {
    pub fn generate(
        cfg: ConfigStore,
        lang: LangDict,
        datafile: PathBuf,
    ) -> Result<TypstData, Errcode> {
        let invdata = if !datafile.is_file() {
            InvoiceSavedData::init()
        } else {
            let json_str = std::fs::read_to_string(&datafile)?;
            match serde_json::from_str::<InvoiceSavedData>(&json_str) {
                Ok(data) => data,
                Err(_) => {
                    let json_map: Value = serde_json::from_str(&json_str)?;
                    let json_map = json_map
                        .as_object()
                        .ok_or(Errcode::InvalidData("invoice"))?;
                    InvoiceSavedData::partial_import(json_map)
                }
            }
        };
        let inp = InvoiceInput::ask(lang.clone(), Some(&invdata.recipients_known));
        let mut builder = InvoiceBuilder {
            cfg,
            lang,
            data: invdata,
        };
        let (fname, result) = builder.generate_invoice(inp)?;
        std::fs::write("/tmp/.typst_result.typ", &result)?;
        std::fs::write(datafile, serde_json::to_string_pretty(&builder.data)?)?;
        Ok(TypstData::new(fname, result))
    }

    // TODO    Generate an invoice
    pub fn generate_invoice(&mut self, inp: InvoiceInput) -> Result<(String, String), Errcode> {
        // Getting necessary data before writing the code
        self.data.invoice_total_count += 1;
        let current_date = Utc::now();

        let fname = format!(
            "invoice_{}_{}_{}.pdf",
            inp.recipient.slug,
            current_date.format("%d%m%y"),
            self.data.invoice_total_count
        );

        let footer = self.cfg.get("invoice", "footer").as_str().unwrap();
        let mut source = "".to_string();
        write_page_settings(&mut source, footer);
        self.generate_header(&mut source);
        source += "#v(sep_par())\n";
        self.generate_metadata(&mut source, &inp, &current_date);
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

    fn generate_metadata(
        &self,
        source: &mut String,
        inp: &InvoiceInput,
        current_date: &DateTime<Utc>,
    ) {
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
            inp.recipient.name,
            inp.recipient.address,
            self.lang.get_doctype_word("invoice", "invoice_nb"),
            self.data.invoice_total_count,
            self.lang.get_doctype_word("invoice", "creation_date"),
            current_date_fmt,
            self.lang.get_doctype_word("invoice", "sell_date"),
            inp.date_sell,
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

        let mut nb_tx = 1;
        let mut total_price = 0.0;
        loop {
            println!("\nEnter data about transaction {nb_tx}: ");
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

            let total = units * ppu;
            *source += format!(
                "
                \"{descr}\", \"{units}\", \"{ppu:.2} {curr_sym}\", \"{total:.2} {curr_sym}\",
            "
            )
            .as_str();
            nb_tx += 1;
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
