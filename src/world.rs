// pub fn get_fonts(fonts_dir: &PathBuf) -> (Prehashed<FontBook>, Vec<Font>) {
//     todo!();
//     // let fonts = fonts_dir
//     //     .files()
//     //     .flat_map(|file| Font::iter(file.contents().into()))
//     //     .collect();
//     // let book = FontBook::from_fonts(&fonts);
//     // (Prehashed::new(book), fonts)
// }

// pub fn get_asset(assets_dir: &PathBuf, path: &Path) -> FileResult<Bytes> {
//     println!("Getting file {:?}", path);
//     todo!();
//     // Ok(assets_dir
//     //     .get_file(path)
//     //     .unwrap_or_else(|| panic!("failed to load {:?}", id.vpath()))
//     //     .contents()
//     //     .into())
// }

// pub fn get_date(offset: Option<i64>) -> Option<Datetime> {
//     Some(Datetime::from_ymd(1970, 1, 1).unwrap())
// }

use comemo::Prehashed;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{Source, FileId};
use typst::text::{FontBook, Font};
use typst::{World, Library};

use crate::doc_config::DocumentConfig;
use crate::doctype::DocumentType;

pub struct TypstWorld<'a> {
    config: &'a DocumentConfig,
    doctype: DocumentType,
    source: Source,
}

// TODO    Use this instead of specific world for specific doctype
impl<'a> World for TypstWorld<'a> {
    fn library(&self) ->  &Prehashed<Library>  {
        self.config.get_library(&self.doctype)
    }

    fn book(&self) ->  &Prehashed<FontBook>  {
        self.config.get_font_book()
    }

    fn main(&self) -> Source {
        self.source.clone()
    }

    fn source(&self, id:FileId) -> FileResult<Source>  {
        // TODO    assert that this should never get called ?
        Ok(self.main())
    }

    fn file(&self,id:FileId) -> FileResult<Bytes>  {
        // TODO    Get an asset
        todo!()
    }

    fn font(&self,index:usize) -> Option<Font>  {
        // TODO    Get a font
        todo!()
    }

    fn today(&self,offset:Option<i64>) -> Option<Datetime>  {
        // TODO    Get the datetime
        todo!()
    }
}
