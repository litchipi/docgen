use std::path::PathBuf;

use comemo::Prehashed;
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Smart};
use typst::layout::{Abs, Margin, PageElem};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};

static LIBRARY: Lazy<Prehashed<Library>> = Lazy::new(|| {
    let mut lib = Library::default();
    lib.styles
        .set(PageElem::set_width(Smart::Custom(Abs::pt(240.0).into())));
    lib.styles.set(PageElem::set_height(Smart::Auto));
    lib.styles
        .set(PageElem::set_margin(Margin::splat(Some(Smart::Custom(
            Abs::pt(15.0).into(),
        )))));
    Prehashed::new(lib)
});

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/");

static FONT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

static FONTS: Lazy<(Prehashed<FontBook>, Vec<Font>)> = Lazy::new(|| {
    let fonts: Vec<_> = FONT_DIR
        .files()
        .flat_map(|file| Font::iter(file.contents().into()))
        .collect();
    let book = FontBook::from_fonts(&fonts);
    (Prehashed::new(book), fonts)
});

// TODO
static INVOICE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/invoices");

pub struct InvoiceWorld(Source);

impl InvoiceWorld {
    pub fn new(inpf: PathBuf) -> InvoiceWorld {
        let id = FileId::new(None, VirtualPath::new(&inpf));
        let text = std::fs::read_to_string(inpf).unwrap();
        println!("{text}");
        InvoiceWorld(Source::new(id, text))
    }
}

impl World for InvoiceWorld {
    fn library(&self) -> &Prehashed<Library> {
        &LIBRARY
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &FONTS.0
    }

    fn main(&self) -> Source {
        self.0.clone()
    }

    fn source(&self, _: FileId) -> FileResult<Source> {
        Ok(self.0.clone())
    }

    // TODO
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        assert!(id.package().is_none());
        println!("Getting file {:?}", id.vpath());
        Ok(ASSETS_DIR
            .get_file(id.vpath().as_rootless_path())
            .unwrap_or_else(|| panic!("failed to load {:?}", id.vpath()))
            .contents()
            .into())
    }

    fn font(&self, index: usize) -> Option<Font> {
        Some(FONTS.1[index].clone())
    }

    // TODO
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::from_ymd(1970, 1, 1).unwrap())
    }
}
