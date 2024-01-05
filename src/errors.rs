use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    DocTypeUnsupported(String),
    InvalidData(&'static str),

    IoError(#[from] std::io::Error),
    TomlDecode(#[from] toml::de::Error),
    PathPrefixStrip(#[from] std::path::StripPrefixError),
    JsonDecode(#[from] serde_json::Error),
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
                writeln!(f, "Message: {}", e.to_string())?;
            }
            e => write!(f, "{e:?}")?,
        }
        Ok(())
    }
}
