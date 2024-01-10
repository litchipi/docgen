use crate::errors::Errcode;
use crate::style::Style;
use crate::utils::ask_user;
use std::io::Read;
use std::path::PathBuf;
use typst::foundations::Bytes;
use typst::text::Font;

pub fn import_fonts(fonts_dir: &PathBuf) -> Result<Vec<Font>, Errcode> {
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
    Ok(all_fonts)
}

pub fn get_all_fonts(style: &Style, fonts_dir: &PathBuf) -> Result<(), Errcode> {
    if !fonts_dir.exists() {
        std::fs::create_dir(fonts_dir)?;
    }

    // Do this for every fond defined in style
    let font_name = style.get("font_name").unwrap().as_str().unwrap();
    ensure_font_exist(fonts_dir, font_name)?;
    Ok(())
}

fn dafont_name(name: &str) -> String {
    let name = name
        .chars()
        .filter(|c| c.is_whitespace() || c.is_alphanumeric())
        .collect::<String>();
    let name = name.replace("  ", " ").replace(' ', "_");
    name.to_lowercase()
}

fn download_font(font_name: &str, dir: &PathBuf) -> Result<(), Errcode> {
    std::fs::create_dir_all(dir)?;

    let dafont_name = dafont_name(font_name);
    println!("Donwloading font {dafont_name}");
    let data = reqwest::blocking::get(format!("https://dl.dafont.com/dl/?f={dafont_name}",))?;
    data.error_for_status_ref()?;
    let data = data.bytes()?;
    if data.is_empty() {
        return Err(Errcode::InvalidConfig(
            "font",
            format!("Font {dafont_name:?} doesn't exist"),
        ));
    }

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

fn ensure_font_exist(dir: &PathBuf, name: &str) -> Result<(), Errcode> {
    let mut exists = false;
    let name_variants = vec![name.replace(' ', "")];
    for path in std::fs::read_dir(dir)? {
        let path = path?.path();
        // Pretty bad check of a font name, but should be enough
        let fname = path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if name_variants.iter().any(|n| &fname == n) {
            exists = true;
            break;
        }
    }

    if !exists {
        let dl_from_dafont = ask_user(format!(
            "Font {name} not installed in {dir:?}, do you wish to download it from dafont ? [y/N] "
        ));
        if dl_from_dafont.is_empty() || (dl_from_dafont.to_lowercase() == "y") {
            download_font(name, &dir.join(name))?;
        }
    }
    Ok(())
}

#[test]
fn generate_dafont_names() {
    assert_eq!(dafont_name("A B"), "a_b");
    assert_eq!(
        dafont_name("Champagne & Limousines"),
        "champagne_limousines"
    );
}
