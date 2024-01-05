use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    codegen::{sanitize, write_page_settings},
    errors::Errcode,
    utils::{ask_user, ask_user_nonempty},
};

#[derive(Serialize, Deserialize)]
struct InvoiceSavedData {
    company_name: String,
    address: String,
    email: String,
    legal_status: String,
    siret_number: String,
    invoice_total_count: usize,
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
        }
    }

    pub fn partial_import(map: &Map<String, Value>) -> InvoiceSavedData {
        let company_name = match map.get("company_name").map(|n| n.as_str()).flatten() {
            Some(n) => n.to_string(),
            None => ask_user_nonempty("Enter the name of your company: "),
        };
        let address = match map.get("address").map(|n| n.as_str()).flatten() {
            Some(n) => n.to_string(),
            None => ask_user_nonempty("Enter the address of your company: "),
        };
        let email = match map.get("email").map(|n| n.as_str()).flatten() {
            Some(n) => n.to_string(),
            None => ask_user_nonempty("Enter the email of your company: "),
        };
        let legal_status = match map.get("legal_status").map(|n| n.as_str()).flatten() {
            Some(n) => n.to_string(),
            None => ask_user_nonempty("Enter the legal status of your company: "),
        };
        let siret_number = match map.get("siret_number").map(|n| n.as_str()).flatten() {
            Some(n) => n.to_string(),
            None => ask_user_nonempty("Enter the siret number of your company: "),
        };
        InvoiceSavedData {
            company_name,
            address,
            email,
            legal_status,
            siret_number,
            invoice_total_count: 0,
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
        let mut source = "".to_string();
        write_page_settings(&mut source);
        // TODO    Add the logo
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

        source += "\n";
        self.data.invoice_total_count += 1;
        Ok(source)
    }
}
