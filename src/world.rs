use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use comemo::Prehashed;
use toml::map::Map;
use typst::diag::{FileError, FileResult};
use typst::eval::Tracer;
use typst::foundations::{Bytes, Datetime};
use typst::model::Document;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};

use crate::doctype::{DocumentType, TypstData};
use crate::errors::Errcode;
use crate::fonts::import_fonts;
use crate::style::{generate_style_variables, import_style};

type AssetStore = HashMap<PathBuf, Bytes>;

pub struct TypstWorld {
    assets: AssetStore,
    fonts: (Prehashed<FontBook>, Vec<Font>),
    library: Prehashed<Library>,
    style: Map<String, toml::Value>,
    doctype: DocumentType,
    source: Option<Source>,
}

impl TypstWorld {
    pub fn new(root: &Path, doctype: DocumentType) -> Result<TypstWorld, Errcode> {
        let style = import_style(&root.join("style.toml"))?;
        let fonts = import_fonts(&style, &root.join("fonts"))?;
        let font_book = FontBook::from_fonts(&fonts);
        let assets_dir = root.join("assets");
        let assets = import_assets(&assets_dir, &assets_dir)?;

        Ok(TypstWorld {
            fonts: (Prehashed::new(font_book), fonts),
            assets,
            style,
            doctype,
            library: Prehashed::new(Library::default()),
            source: None,
        })
    }

    pub fn compile(&mut self, source: TypstData) -> Result<Document, Errcode> {
        let source_id = FileId::new(None, VirtualPath::new("/source"));
        let style_vars = generate_style_variables(&self.style, self.doctype.to_string());
        println!("{style_vars}\n{}", source.code);
        let source = Source::new(source_id, format!("{style_vars}\n{}\n", source.code));
        self.source = Some(source);
        let mut tracer = Tracer::new();
        match typst::compile(self, &mut tracer) {
            Ok(document) => {
                for warn in tracer.warnings() {
                    println!("WARN {:?}", warn);
                }
                Ok(document)
            }
            Err(e) => {
                println!("Source code:\n{}", self.main().text());
                panic!("Typst compilation error: {e:?}");
            }
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.fonts.0
    }

    fn main(&self) -> Source {
        self.source.clone().unwrap()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        assert_eq!(id.vpath().as_rootless_path().to_str().unwrap(), "source");
        Ok(self.main())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        println!("Getting asset {id:?}");
        let path = id.vpath().as_rootless_path();
        self.assets
            .get(path)
            .ok_or_else(|| FileError::NotFound(path.into()))
            .cloned()
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.1.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        if offset.is_some() {
            // TODO    If ever get this printed, take care of offset
            println!("CALLED TODAY WITH OFFSET {offset:?}");
        }
        let cdate = Utc::now();
        Some(
            Datetime::from_ymd(
                cdate.year(),
                cdate.month().try_into().unwrap(),
                cdate.day().try_into().unwrap(),
            )
            .unwrap(),
        )
    }
}

fn import_assets(root: &PathBuf, assets_dir: &PathBuf) -> Result<AssetStore, Errcode> {
    if !assets_dir.exists() {
        std::fs::create_dir(assets_dir)?;
    }
    assert!(assets_dir.is_dir());
    let mut store = HashMap::new();

    for asset_file in std::fs::read_dir(assets_dir)? {
        let asset_file = asset_file?;
        let ftype = asset_file.file_type()?;

        if ftype.is_dir() {
            for (path, data) in import_assets(root, &asset_file.path())? {
                let key_path = assets_dir
                    .join(path)
                    .as_path()
                    .strip_prefix(root)?
                    .to_path_buf();
                store.insert(key_path, data);
            }
        } else {
            // File or symlink
            let key_path = asset_file.path().strip_prefix(root)?.to_path_buf();
            let data = std::fs::read(asset_file.path())?;
            store.insert(key_path, Bytes::from(data));
        }
    }
    Ok(store)
}
