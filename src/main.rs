use std::path::PathBuf;

use clap::Parser;
use doc_config::DocumentConfig;
use typst::eval::Tracer;
use typst::model::Document;

mod doctype;
mod errors;
mod world;
mod doc_config;

use doctype::DocumentType;
use errors::Errcode;
use world::TypstWorld;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    doctype: String,

    #[arg(short, long)]
    outfile: PathBuf,

    #[arg(short, long)]
    config_file: PathBuf,

    #[arg(short, long)]
    fonts_dir: PathBuf,

    #[arg(short, long)]
    assets_dir: PathBuf,

    #[arg(short, long)]
    style_sheet: PathBuf,
}

fn compile_typst(source: TypstWorld) -> Result<Document, Errcode> {
    let mut tracer = Tracer::new();
    let document = typst::compile(&source, &mut tracer).unwrap();
    for warn in tracer.warnings() {
        println!("WARN {:?}", warn);
    }
    Ok(document)
}

fn export(outfile: &PathBuf, document: &Document) -> Result<(), Errcode> {
    let res = typst_pdf::pdf(document, None, None);
    std::fs::write(&outfile, res)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    let doc_config = DocumentConfig::try_from(&args).unwrap();
    let Args { doctype, outfile , ..} = args;
    let doctype: DocumentType = doctype.try_into().unwrap();

    let source = doctype.generate_typst(&doc_config).expect("Unable to generate typst code");
    let doc = compile_typst(source).expect("Unable to compile generated typst code");
    export(&outfile, &doc).expect("Unable to export to file")
}
