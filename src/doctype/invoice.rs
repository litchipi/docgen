use std::path::PathBuf;

use comemo::Prehashed;

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};

use crate::doc_config::DocumentConfig;
use crate::errors::Errcode;

pub struct InvoiceWorld<'a> {
    source: String,
    doc_config: &'a DocumentConfig,
}

// TODO    Remove the "World" of here, use the generic TypstWorld instead
//    Keep this source file only for typst code generation
impl<'a> InvoiceWorld<'a> {
    pub fn new(doc_config: &'a DocumentConfig, source: String) -> InvoiceWorld {
        InvoiceWorld {
            source,
            doc_config,
        }
    }
}

impl<'a> World for InvoiceWorld<'a> {
    fn library(&self) -> &Prehashed<Library> {
        self.doc_config.get_library(&super::DocumentType::Invoice)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.doc_config.get_book(&super::DocumentType::Invoice)
    }

    fn font(&self, index: usize) -> Option<Font> {
        Some(self.fonts[index].clone())
    }

    fn main(&self) -> Source {
        let id = FileId::new(None, VirtualPath::new("/source"));
        Source::new(id, self.source.clone())
    }

    fn source(&self, _: FileId) -> FileResult<Source> {
        Ok(self.main())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        assert!(id.package().is_none());
        get_asset(&self.assets_dir, id.vpath().as_rootless_path())
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        get_date(offset)
    }
}

pub fn generate_invoice() -> Result<String, Errcode> {
    Ok("".to_string())
}

// TODO    Set style
fn generate_library() -> Prehashed<Library> {
    let mut lib = Library::default();
    // lib.styles
    //     .set(PageElem::set_width(Smart::Custom(Abs::pt(240.0).into())));
    // lib.styles.set(PageElem::set_height(Smart::Auto));
    // lib.styles
    //     .set(PageElem::set_margin(Margin::splat(Some(Smart::Custom(
    //         Abs::pt(15.0).into(),
    //     )))));
    Prehashed::new(lib)
}
