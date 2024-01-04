use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    DocTypeUnsupported(String),
    IoError(#[from] std::io::Error),
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
