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
    #[arg()]
    doctype: String,

    #[arg(short, long, default_value="./out.pdf")]
    outfile: PathBuf,

    #[arg(short, long)]
    root_dir: Option<PathBuf>,
}

impl Args {
    fn get_root(&self) -> PathBuf {
        if let Some(ref root) = self.root_dir {
            root.clone()
        } else {
            if let Ok(root) = std::env::var("DOCGEN_ROOT") {
                root.into()
            } else {
                panic!("Root directory must be set using --root-dir or the DOCGEN_ROOT env var");
            }
        }
        
    }
}

fn export(outfile: &PathBuf, document: &Document) -> Result<(), Errcode> {
    let res = typst_pdf::pdf(document, None, None);
    std::fs::write(&outfile, res)?;
    Ok(())
}

fn main() {
    println!("[*] Getting the configuration");
    let args = Args::parse();
    let root = args.get_root();
    if !root.exists() {
        std::fs::create_dir(&root).expect("Unable to create root directory");
    }
    let doctype: DocumentType = (&args.doctype).try_into().unwrap();

    println!("[*] Generating the source code");
    let source = doctype
        .generate_typst(&root.join("history"))
        .expect("Unable to generate typst code");

    println!("[*] Initializing Typst compilation context");
    let world = TypstWorld::new(&root, doctype, source).expect("Unable to create Typst context");

    println!("[*] Compiling the source code");
    let doc = world.compile().expect("Unable to compile generated typst code");

    println!("[*] Rendering the PDF file");
    export(&args.outfile, &doc).expect("Unable to export to file");
}
