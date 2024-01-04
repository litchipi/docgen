use toml::Value;
use typst::Library;

use crate::errors::Errcode;
use crate::world::ConfigStore;

// TODO    Define the default style for any doctype
pub fn setup_default_style(lib: &mut Library, config: &ConfigStore) {
    
}

// TODO    Define the style based on the config keys passed as argument
pub fn setup_style(lib: &mut Library, docstyle: &Value) -> Result<(), Errcode> {
    Ok(())
}
