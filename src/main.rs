use std::path::PathBuf;

use typst::{eval::Tracer, model::Document, World};

mod world;

fn export(world: &world::InvoiceWorld, document: &Document, path: PathBuf) {
    let res = typst_pdf::pdf(document, Some("test"), world.today(None));
    std::fs::write(path, res).unwrap();
}

fn main() {
    let mut tracer = Tracer::new();
    let world = world::InvoiceWorld::new("./test".into());
    let document = typst::compile(&world, &mut tracer).unwrap();
    println!("{document:?}");
    for warn in tracer.warnings() {
        println!("WARN {:?}", warn);
    }
    export(&world, &document, "test.pdf".into())
}

// TODO    CLI tool with subcommands depending on a type of document to create
// Generates typst code, then convert to PDF
// Types of documents:
// - invoices
// - devis
// - contracts
// - letter
