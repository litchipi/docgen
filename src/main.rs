use std::path::PathBuf;

use clap::Parser;
use typst::model::Document;

mod codegen;
mod config;
mod doctype;
mod errors;
mod style;
mod utils;
mod world;

use doctype::DocumentType;
use errors::Errcode;
use world::TypstWorld;

use crate::{config::import_config, utils::import_lang_profile};

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    doctype: String,

    #[arg(short, long)]
    outdir: PathBuf,

    #[arg(short, long)]
    root_dir: Option<PathBuf>,
}

impl Args {
    fn get_root(&self) -> PathBuf {
        if let Some(ref root) = self.root_dir {
            root.clone()
        } else if let Ok(root) = std::env::var("DOCGEN_ROOT") {
            root.into()
        } else {
            panic!("Root directory must be set using --root-dir or the DOCGEN_ROOT env var");
        }
    }
}

fn export(outf: &PathBuf, doc: &Document) -> Result<(), Errcode> {
    let res = typst_pdf::pdf(doc, None, None);
    std::fs::write(outf, res)?;
    Ok(())
}

fn main() {
    println!("[*] Getting the configuration");
    let args = Args::parse();
    let root = args.get_root();
    if !root.exists() {
        std::fs::create_dir_all(&root).expect("Unable to create root directory");
    }
    let doctype: DocumentType = (&args.doctype).try_into().unwrap();
    let lang = import_lang_profile(&root.join("lang.toml")).expect("Unable to load lang file");
    let config = import_config(&root.join("config.toml")).expect("Unable to load config");

    println!("[*] Initializing Typst compilation context");
    let mut world =
        TypstWorld::new(&root, doctype).expect("Unable to create Typst context");

    println!("[*] Generating the source code");
    let source = doctype
        .generate_typst(config.clone(), lang.clone(), &root.join("data"))
        .expect("Unable to generate typst code");
    if !args.outdir.exists() {
        std::fs::create_dir_all(&args.outdir).expect("Unable to create output directory");
    }
    let outfile = args.outdir.join(&source.fname);

    println!("[*] Compiling the source code");
    let doc = world
        .compile(source)
        .expect("Unable to compile generated typst code");

    println!("[*] Rendering the PDF file");
    export(&outfile, &doc).expect("Unable to export to file");
}
