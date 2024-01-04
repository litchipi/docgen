use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::Errcode;

#[derive(Serialize, Deserialize)]
struct InvoiceHistory {
    nb: usize,
}

impl InvoiceHistory {
    pub fn init() -> InvoiceHistory {
        InvoiceHistory { nb: 0 }
    }
}

pub struct InvoiceBuilder {
    history: InvoiceHistory,
}

impl InvoiceBuilder {
    pub fn generate(histfile: PathBuf) -> Result<String, Errcode> {
        let history = if !histfile.is_file() {
            InvoiceHistory::init()
        } else {
            serde_json::from_str(std::fs::read_to_string(&histfile)?.as_str())?
        };
        let mut builder = InvoiceBuilder { 
            history,
        };
        let result = builder.generate_invoice()?;
        std::fs::write(histfile, serde_json::to_string(&builder.history)?)?;
        Ok(result)
    }

    // TODO    Generate an invoice
    pub fn generate_invoice(&mut self) -> Result<String, Errcode> {
        let mut source = "".to_string();
        source += "\n";
        self.history.nb += 1;
        Ok(source)
    }
}
