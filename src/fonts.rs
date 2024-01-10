use crate::errors::Errcode;
use crate::style::Style;
use crate::utils::ask_user;
use std::io::Read;
use std::path::PathBuf;
use typst::foundations::Bytes;
use typst::text::Font;

pub fn import_fonts(style: &Style, fonts_dir: &PathBuf) -> Result<Vec<Font>, Errcode> {
    if !fonts_dir.exists() {
        std::fs::create_dir(fonts_dir)?;
    }

    let font_name = style.get("font_name").unwrap().as_str().unwrap();
    if !check_font_exist(fonts_dir, font_name)? {
        let dl_from_dafont = ask_user(format!("Font {font_name} not installed in {fonts_dir:?}, do you wish to download it from dafont ? [y/N] "));
        if dl_from_dafont.is_empty() || (dl_from_dafont.to_lowercase() == "y") {
            download_font(font_name, &fonts_dir.join(font_name))?;
        }
    }

    let mut all_fonts = vec![];
    for font_file in std::fs::read_dir(fonts_dir)? {
        let font_file = font_file?;
        let ftype = font_file.file_type()?;
        if ftype.is_dir() {
            all_fonts.extend(import_fonts(style, &font_file.path())?);
        } else {
            // File or symlink
            let data = std::fs::read(font_file.path())?;
            all_fonts.extend(Font::iter(Bytes::from(data)));
        }
    }
    Ok(all_fonts)
}

fn download_font(font_name: &str, dir: &PathBuf) -> Result<(), Errcode> {
    std::fs::create_dir_all(dir)?;
    println!("Donwloading font {font_name}");
    let data = reqwest::blocking::get(format!(
        "https://dl.dafont.com/dl/?f={}",
        font_name.to_lowercase()
    ))?
    .bytes()?;
    let data = std::io::Cursor::new(data.as_ref());
    let mut archive = zip::ZipArchive::new(data)?;
    let all_fnames = archive
        .file_names()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    for fname in all_fnames {
        let mut file = archive.by_name(&fname)?;
        println!("Extracting file {}", file.name());
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)?;
        std::fs::write(dir.join(file.name()).with_extension("ttf"), buffer)?;
    }
    Ok(())
}

fn check_font_exist(dir: &PathBuf, name: &str) -> Result<bool, Errcode> {
    for path in std::fs::read_dir(dir)? {
        let path = path?.path();
        // Pretty bad check of a font name, but should be enough
        if path.with_extension("").file_name().unwrap() == name {
            return Ok(true);
        }
    }
    Ok(false)
}
