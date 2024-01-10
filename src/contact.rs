use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::errors::Errcode;
use crate::interface::utils::ask_user_nonempty;

pub struct ContactBook(HashMap<String, Contact>);

impl ContactBook {
    fn fname(root: &Path) -> PathBuf {
        root.join("contacts").with_extension("json")
    }

    pub fn get_or_add(&mut self, slug: String) -> Contact {
        if let Some(contact) = self.0.get(&slug) {
            (*contact).clone()
        } else {
            println!("Enter the informations related to the recipient");
            let recipient = Contact::ask(Some(slug.clone()));
            self.0.insert(slug, recipient.clone());
            recipient
        }
    }

    pub fn import(root: &Path) -> Result<ContactBook, Errcode> {
        let json_str = std::fs::read_to_string(Self::fname(root))?;
        let data = serde_json::from_str::<serde_json::Value>(&json_str)?;
        let data = data.as_object().unwrap().to_owned();
        let mut book = ContactBook(HashMap::new());
        for (k, v) in data.into_iter() {
            book.0.insert(k, serde_json::from_value::<Contact>(v)?);
        }
        Ok(book)
    }

    pub fn export(&self, root: &Path) -> Result<(), Errcode> {
        let mut map = serde_json::Map::new();
        for (k, v) in self.0.iter() {
            map.insert(k.clone(), serde_json::to_value(v)?);
        }
        let json_str = serde_json::to_string_pretty(&map)?;
        std::fs::write(Self::fname(root), json_str)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Contact {
    pub slug: String,
    pub name: String,
    pub address: String,
    invoices: Vec<usize>,
    quotations: Vec<usize>,
}

impl Contact {
    pub fn ask(slug: Option<String>) -> Contact {
        let slug = slug.unwrap_or(Self::ask_slug());
        let name = ask_user_nonempty("Name: ".to_string());
        let address = ask_user_nonempty("Address: ".to_string());
        Contact {
            slug,
            name,
            address,
            invoices: vec![],
            quotations: vec![],
        }
    }

    pub fn ask_slug() -> String {
        ask_user_nonempty("Enter the slug for the recipient: ")
            .to_ascii_lowercase()
            .replace(' ', "_")
    }
}
