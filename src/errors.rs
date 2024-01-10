use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    DocTypeUnsupported(String),
    InvalidConfig(&'static str, String),
    ContactNotFound(String),
    HistoryElementNotFound(usize),

    IoError(#[from] std::io::Error),
    TomlDecode(#[from] toml::de::Error),
    TomlEncode(#[from] toml::ser::Error),
    PathPrefixStrip(#[from] std::path::StripPrefixError),
    JsonDecode(#[from] serde_json::Error),
    ReqwestError(#[from] reqwest::Error),
    ZipArchive(#[from] zip::result::ZipError),
}

impl std::fmt::Display for Errcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errcode::IoError(e) => {
                writeln!(f, "An IO error occured")?;
                writeln!(f, "Type: {}", e.kind())?;
                if let Some(code) = e.raw_os_error() {
                    writeln!(f, "Code: {code}")?;
                }
                writeln!(f, "Message: {}", e)?;
            }
            e => write!(f, "{e:?}")?,
        }
        Ok(())
    }
}
