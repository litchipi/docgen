use std::collections::HashMap;
use std::path::PathBuf;

use comemo::Prehashed;
use toml::map::Map;
use typst::diag::{FileResult, FileError};
use typst::eval::Tracer;
use typst::foundations::{Bytes, Datetime};
use typst::model::Document;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};

use crate::Args;
use crate::doctype::DocumentType;
use crate::errors::Errcode;
use crate::style::{setup_style, setup_default_style};

type AssetStore = HashMap<PathBuf, Bytes>;
type StyleStore = HashMap<DocumentType, Prehashed<Library>>;
pub type ConfigStore = Map<String, toml::Value>;

pub struct TypstWorld {
    // pub config: &'a DocumentConfig,
    doctype: DocumentType,
    source: Source,
    style: StyleStore,
    assets: AssetStore,
    fonts: (Prehashed<FontBook>, Vec<Font>),
    datetime_offset: i64,
}

impl TypstWorld {
    pub fn new(args: &Args, source: String) -> Result<TypstWorld, Errcode> {
        let fonts = import_fonts(&args.fonts_dir)?;
        let font_book = FontBook::from_fonts(&fonts);
        let assets = import_assets(&args.assets_dir, &args.assets_dir)?;
        let config = import_config(&args.config_file)?;
        let style = import_style(&args.style_sheet, &config)?;
        let datetime_offset = 0; //config.get("datetime_offset").unwrap().as_integer().unwrap();
        let source_id = FileId::new(None, VirtualPath::new("/source"));

        Ok(TypstWorld {
            source: Source::new(source_id, source),
            doctype: (&args.doctype).try_into()?,
            fonts: (Prehashed::new(font_book), fonts),
            style,
            assets,
            datetime_offset,
        })
    }

    pub fn compile(&self) -> Result<Document, Errcode> {
        let mut tracer = Tracer::new();
        let document = typst::compile(self, &mut tracer).unwrap();
        for warn in tracer.warnings() {
            println!("WARN {:?}", warn);
        }
        Ok(document)
    }
}

impl World for TypstWorld {
    fn library(&self) -> &Prehashed<Library> {
        self.style.get(&self.doctype).unwrap()
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.fonts.0
    }

    fn main(&self) -> Source {
        self.source.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        assert_eq!(id.vpath().as_rootless_path().to_str().unwrap(), "source");
        Ok(self.main())
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rootless_path();
        self.assets
            .get(path)
            .ok_or_else(|| FileError::NotFound(path.into()))
            .cloned()
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.1.get(index).cloned()
    }

    // TODO    Get datetime from offset
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::from_ymd(1970, 1, 1).unwrap())
    }
}

fn import_fonts(fonts_dir: &PathBuf) -> Result<Vec<Font>, Errcode> {
    assert!(fonts_dir.is_dir());
    let mut all_fonts = vec![];
    for font_file in std::fs::read_dir(fonts_dir)? {
        let font_file = font_file?;
        let ftype = font_file.file_type()?;
        if ftype.is_dir() {
            all_fonts.extend(import_fonts(&font_file.path())?);
        } else {
            // File or symlink
            let data = std::fs::read(font_file.path())?;
            all_fonts.extend(Font::iter(Bytes::from(data)));
        }
    }
    println!("Fonts: {all_fonts:?}");
    Ok(all_fonts)
}

fn import_assets(
    root: &PathBuf,
    assets_dir: &PathBuf,
) -> Result<AssetStore, Errcode> {
    assert!(assets_dir.is_dir());
    let mut store = HashMap::new();

    for asset_file in std::fs::read_dir(assets_dir)? {
        let asset_file = asset_file?;
        let ftype = asset_file.file_type()?;

        if ftype.is_dir() {
            for (path, data) in import_assets(root, &asset_file.path())? {
                let key_path = assets_dir.join(path).as_path().strip_prefix(root)?.to_path_buf();
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

fn import_config(config_file: &PathBuf) -> Result<ConfigStore, Errcode> {
    assert!(config_file.is_file());
    let default_config : toml::Value = toml::from_str(include_str!("../default_config.toml"))?;
    let default_config = default_config.as_table().unwrap().to_owned();
    
    let config : toml::Value = toml::from_str(std::fs::read_to_string(config_file)?.as_str())?;
    let mut config = config.as_table().unwrap().to_owned();
    for (key, val) in default_config.into_iter() {
        if !config.contains_key(&key) {
            config.insert(key, val);
        }
    }
    println!("Config: {config:?}");
    Ok(config)
}

fn import_style(
    stylesheet: &PathBuf,
    config: &ConfigStore,
) -> Result<StyleStore, Errcode> {
    assert!(stylesheet.is_file());
    let mut store = HashMap::new();
    let style : toml::Value = toml::from_str(std::fs::read_to_string(stylesheet)?.as_str())?;
    let style = style.as_table().unwrap();

    for (doctype, dockey) in DocumentType::all_variants().iter() {
        let mut lib = Library::default();
        setup_default_style(&mut lib, config);
        if let Some(docstyle) = style.get(*dockey) {
            setup_style(&mut lib, docstyle)?;
        }
        store.insert(*doctype, Prehashed::new(lib));
    }

    println!("Style: {store:?}");
    Ok(store)
}
