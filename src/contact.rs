use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::utils::{ask_user_nonempty, map_get_str_or_ask};

pub type ContactBook = HashMap<String, Rc<Contact>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Contact {
    pub slug: String,
    pub name: String,
    pub address: String,
    invoices: Vec<usize>,
}

impl Contact {
    pub fn new(slug: String, name: String, address: String) -> Contact {
        Contact {
            slug,
            name,
            address,
            invoices: vec![],
        }
    }
    pub fn ask(slug: Option<String>) -> Contact {
        let slug = slug.unwrap_or(Self::ask_slug());
        let name = ask_user_nonempty(format!("Name: "));
        let address = ask_user_nonempty(format!("Address: "));
        Contact {
            slug,
            name,
            address,
            invoices: vec![],
        }
    }

    pub fn ask_slug() -> String {
        ask_user_nonempty("Enter the slug for the recipient: ")
            .to_ascii_lowercase()
            .replace(' ', "_")
    }

    pub fn partial_import(slug: String, map: &Map<String, Value>) -> Contact {
        let name = map_get_str_or_ask(
            map,
            "name",
            format!("Enter the name of the contact {slug:?}: "),
            true,
        );
        let address = map_get_str_or_ask(
            map,
            "address",
            format!("Enter the address of the contact {slug:?}: "),
            true,
        );
        Contact::new(slug, name, address)
    }
}
