use comemo::Prehashed;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::{Library, World};

use crate::doc_config::DocumentConfig;
use crate::doctype::DocumentType;

pub struct TypstWorld<'a> {
    pub config: &'a DocumentConfig,
    pub doctype: DocumentType,
    pub source: Source,
}

impl<'a> World for TypstWorld<'a> {
    fn library(&self) -> &Prehashed<Library> {
        self.config.get_library(&self.doctype)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.config.get_font_book()
    }

    fn main(&self) -> Source {
        self.source.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        unreachable!("Attempt to load source code {id:?}");
        // TODO    assert that this should never get called ?
        // Ok(self.main())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.config
            .get_asset(&self.doctype, id.vpath().as_rootless_path())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.config.get_font(index)
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        Some(self.config.get_datetime(offset.or(Some(0)).unwrap()))
    }
}
