use std::{path::PathBuf, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    codegen::{sanitize, write_page_settings},
    errors::Errcode,
    utils::{ask_user, ask_user_nonempty, map_get_str_or_ask},
};

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

    pub fn get_recipient_data(&mut self) -> (String, String) {
        let recipient_slug = ask_user_nonempty("Enter the slug for the recipient: ");
        if let Some((name, addr, _)) = self.recipients_known.get(&recipient_slug) {
            (name.clone(), addr.clone())
        } else {
            let name = ask_user_nonempty(format!("Enter the name of the recipient {recipient_slug}: "));
            let addr = ask_user_nonempty(format!("Enter the address of the recipient {recipient_slug}: "));
            self.recipients_known.insert(recipient_slug, (name.clone(), addr.clone(), vec![]));
            (name, addr)
        }
    }
}

pub struct InvoiceBuilder {
    data: InvoiceSavedData,
}

impl InvoiceBuilder {
    pub fn generate(datafile: PathBuf) -> Result<String, Errcode> {
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
        let mut builder = InvoiceBuilder { data: history };
        let result = builder.generate_invoice()?;
        std::fs::write(datafile, serde_json::to_string_pretty(&builder.data)?)?;
        Ok(result)
    }

    // TODO    Generate an invoice
    pub fn generate_invoice(&mut self) -> Result<String, Errcode> {
        let (rec_name, rec_addr) = self.data.get_recipient_data();
        let date_sell = ask_user_nonempty("Enter the date where the sell was done: ");
        self.data.invoice_total_count += 1;

        let mut source = "".to_string();
        write_page_settings(&mut source);
        // TODO    Add the logo
        source += "#let sep_par() = 28pt\n";
        source += format!(
            "#grid(
            columns: (1fr, auto),
            align(left, text(company_name_font_size())[{}]),
            align(right)[LOGO HERE]
        )\n",
            sanitize(&self.data.company_name)
        )
        .as_str();

        source += format!(
            "#align(left)[
            {} \\ {} \\ {} \\ SIRET: {}
        ]\n",
            sanitize(&self.data.address),
            sanitize(&self.data.email),
            sanitize(&self.data.legal_status),
            sanitize(&self.data.siret_number),
        )
        .as_str();

        source += "#v(sep_par())\n";

        let current_date = "5 Janvier 2024"; // TODO Auto-generate

        source += format!("#grid(
            columns: (1fr, 1fr),
            column-gutter: 10%,
            align(left)[
                #text(17pt)[*Facturé à*] \\
                {} \\ {} \\
            ],
            align(right)[
                Facture numéro \\#*{}* \\
                Créée le *{}* \\
                Date de la prestation: *{}*
            ],
        )", rec_name, rec_addr, self.data.invoice_total_count, current_date, date_sell).as_str();

        source += "#v(sep_par())\n";

        source += "\n";
        Ok(source)
    }
}
