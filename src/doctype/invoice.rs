use std::{path::PathBuf, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use chrono::{Utc, DateTime};

use crate::{
    codegen::{sanitize, write_page_settings},
    errors::Errcode,
    utils::{ask_user, ask_user_nonempty, map_get_str_or_ask, LangDict, ask_user_parse},
};

use super::TypstData;

#[derive(Serialize, Deserialize)]
struct InvoiceSavedData {
    company_name: String,
    address: String,
    email: String,
    legal_status: String,
    siret_number: String,
    invoice_total_count: usize,
    recipients_known: HashMap<String, (String, String, Vec<usize>)>,
}

impl InvoiceSavedData {
    pub fn init() -> InvoiceSavedData {
        let company_name = ask_user_nonempty("Enter the name of your company: ");
        let address = ask_user_nonempty("Enter the postal address of your company: ");
        let email = ask_user("Enter the email of your company: ");
        let legal_status = ask_user_nonempty("Enter the legal status of your company: ");
        let siret_number = ask_user_nonempty("Enter the SIRET number of your company: ");
        println!("");
        InvoiceSavedData {
            company_name,
            address,
            email,
            legal_status,
            siret_number,
            invoice_total_count: 0,
            recipients_known: HashMap::new(),
        }
    }

    pub fn partial_import(map: &Map<String, Value>) -> InvoiceSavedData {
        let company_name = map_get_str_or_ask(map, "company_name", "Enter the name of your company: ", true);
        let address = map_get_str_or_ask(map, "address", "Enter the address of your company: ", true);
        let email = map_get_str_or_ask(map, "email", "Enter the email of your company: ", true);
        let legal_status = map_get_str_or_ask(map, "legal_status", "Enter the legal status of your company: ", true);
        let siret_number = map_get_str_or_ask(map, "siret", "Enter the siret of your company: ", true);
        let mut recipients_known = HashMap::new();
        if let Some(data) = map.get("recipients_known").map(|n| n.as_object()).flatten() {
            for (key, val) in data.iter() {
                let Some(val) = val.as_object() else {
                    println!("WARN Unable to import data for recipient {key}");
                    continue;
                };

                let name = map_get_str_or_ask(val, "name", format!("Enter the name of the recipient {key}: "), true);
                let addr = map_get_str_or_ask(val, "address", format!("Enter the address of the recipient {key}: "), true);
                recipients_known.insert(key.clone(), (name, addr, vec![]));
            }
        }
        InvoiceSavedData {
            company_name,
            address,
            email,
            legal_status,
            siret_number,
            invoice_total_count: 0,
            recipients_known,
        }
    }

    pub fn get_recipient_data(&mut self) -> (String, String, String) {
        let slug = ask_user_nonempty("Enter the slug for the recipient: ").to_ascii_lowercase().replace(" ", "_");
        if let Some((name, addr, _)) = self.recipients_known.get(&slug) {
            (slug, name.clone(), addr.clone())
        } else {
            let name = ask_user_nonempty(format!("Enter the name of the recipient {slug}: "));
            let addr = ask_user_nonempty(format!("Enter the address of the recipient {slug}: "));
            self.recipients_known.insert(slug.clone(), (name.clone(), addr.clone(), vec![]));
            (slug, name, addr)
        }
    }
}

pub struct InvoiceBuilder {
    data: InvoiceSavedData,
    lang: LangDict,
}

impl InvoiceBuilder {
    pub fn generate(lang: LangDict, datafile: PathBuf) -> Result<TypstData, Errcode> {
        let history = if !datafile.is_file() {
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
        let mut builder = InvoiceBuilder { lang, data: history };
        let (fname, result) = builder.generate_invoice()?;
        std::fs::write(datafile, serde_json::to_string_pretty(&builder.data)?)?;
        Ok(TypstData::new(fname, result))
    }

    // TODO    Generate an invoice
    pub fn generate_invoice(&mut self) -> Result<(String, String), Errcode> {
        // Getting necessary data before writing the code
        let (rec_slug, rec_name, rec_addr) = self.data.get_recipient_data();
        self.data.invoice_total_count += 1;
        let current_date = Utc::now();

        let fname = format!("invoice_{rec_slug}_{}_{}.pdf",
            current_date.format("%d%m%y"),
            self.data.invoice_total_count
        );

        let mut source = "".to_string();
        write_page_settings(&mut source);
        self.generate_header(&mut source);
        source += "#v(sep_par())\n";
        self.generate_metadata(&mut source, &current_date, rec_name, rec_addr);
        source += "#v(sep_par())\n";
        self.generate_transaction_table(&mut source);
        source += "#v(sep_par())\n";

        source += "\n";
        Ok((fname, source))
    }

    fn generate_header(&self, source: &mut String) {
        // TODO    Add the logo
        *source += "#let sep_par() = 28pt\n";
        *source += format!(
            "#grid(
            columns: (1fr, auto),
            align(left, text(company_name_font_size())[{}]),
            align(right)[LOGO HERE]
        )\n",
            sanitize(&self.data.company_name)
        )
        .as_str();

        *source += format!(
            "#align(left)[
            {} \\ {} \\ {} \\ SIRET: {}
        ]\n",
            sanitize(&self.data.address),
            sanitize(&self.data.email),
            sanitize(&self.data.legal_status),
            sanitize(&self.data.siret_number),
        )
        .as_str();
    }

    fn generate_metadata(&self, source: &mut String, current_date: &DateTime<Utc>, rec_name: String, rec_addr: String) {
        let current_date_fmt = self.lang.get_date_fmt(current_date);
        let date_sell = ask_user_nonempty("Enter the date where the sell was done: ");

        *source += format!("#grid(
            columns: (1fr, 1fr),
            column-gutter: 10%,
            align(left)[
                #text(17pt)[{}] \\
                {} \\ {} \\
            ],
            align(right)[
                {} \\#*{}* \\
                {} *{}* \\
                {}: *{}*
            ],
        )", self.lang.get_doctype_word("invoice", "recipient_intro"),
            rec_name, rec_addr,
            self.lang.get_doctype_word("invoice", "invoice_nb"), self.data.invoice_total_count,
            self.lang.get_doctype_word("invoice", "creation_date"), current_date_fmt,
            self.lang.get_doctype_word("invoice", "sell_date"), date_sell
        ).as_str();
    }

    fn generate_transaction_table(&self, source: &mut String) {
        let word_desc = self.lang.get_doctype_word("invoice", "tx_item_description");
        let word_units = self.lang.get_doctype_word("invoice", "tx_units");
        let word_ppu = self.lang.get_doctype_word("invoice", "tx_price_per_unit");
        let word_total = self.lang.get_doctype_word("invoice", "total_price");
        let curr_sym = self.lang.get_doctype_word("general", "currency_symbol");
        *source += format!("#table(
            stroke: table_color(),
            columns: (tx_descr_width(), 1fr, 1fr, 1fr),
            [*{word_desc}*], [*{word_units}*], [*{word_ppu}*], [*{word_total}*],
        ").as_str();

        let mut nb_tx = 1;
        loop {
            println!("\nEnter data about transaction {nb_tx}: ");
            let descr = ask_user(format!("{word_desc}: "));
            if descr.is_empty() {
                break;
            }

            let units : Option<f64> = ask_user_parse(format!("{word_units}: "));
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
            *source += format!("
                \"{descr}\", \"{units}\", \"{ppu:.2} {curr_sym}\", \"{total:.2} {curr_sym}\",
            ").as_str();
            nb_tx += 1;
        }

        *source += ")\n";
    }
}
