use std::path::PathBuf;

use clap::Parser;
use typst::model::Document;

mod doctype;
mod errors;
mod world;
mod style;

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

fn export(outfile: &PathBuf, document: &Document) -> Result<(), Errcode> {
    let res = typst_pdf::pdf(document, None, None);
    std::fs::write(&outfile, res)?;
    Ok(())
}

fn main() {
    println!("[*] Getting the configuration");
    let args = Args::parse();
    let doctype: DocumentType = (&args.doctype).try_into().unwrap();

    println!("[*] Generating the source code");
    let source = doctype
        .generate_typst()
        .expect("Unable to generate typst code");

    println!("[*] Initializing Typst compilation context");
    let world = TypstWorld::new(&args, source).expect("Unable to create Typst context");

    println!("[*] Compiling the source code");
    let doc = world.compile().expect("Unable to compile generated typst code");

    println!("[*] Rendering the PDF file");
    export(&args.outfile, &doc).expect("Unable to export to file");
}
