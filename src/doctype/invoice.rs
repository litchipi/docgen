use crate::errors::Errcode;

#[derive(Default)]
pub struct InvoiceBuilder {
    source: String,
}

impl InvoiceBuilder {
    pub fn generate() -> Result<String, Errcode> {
        InvoiceBuilder::default().generate_invoice()
    }

    pub fn generate_invoice(mut self) -> Result<String, Errcode> {
        self.source += "= Some title\n";
        self.source += "Some text\nOther text\n";
        self.source += "#figure(\n\timage(\"invoice/molecular.jpg\", width: 80%),\n";
        self.source += "caption: [ A test ],\n)\n";
        // TODO    Generate the code for the invoice type
        //    Store inside the self.source buffer
        self.source += "\n";
        Ok(self.source)
    }
}
