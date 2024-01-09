use std::{path::PathBuf, rc::Rc};

use toml::map::Map;

use crate::errors::Errcode;

pub type ConfigStore = Rc<ConfigDataStore>;
pub struct ConfigDataStore {
    data: Map<String, toml::Value>,
}

impl ConfigDataStore {
    pub fn get_company(&self, data: &str) -> String {
        self.data
            .get("company")
            .unwrap()
            .get(data)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }

    fn get<'a>(&'a self, key: &str, data: &str) -> &'a toml::Value {
        self.data.get(key).unwrap().get(data).unwrap()
    }

    pub fn get_bool(&self, key: &str, data: &str) -> bool {
        self.get(key, data).as_bool().expect("Unable to convert {key}:{data} to boolean")
    }

    pub fn get_float(&self, key: &str, data: &str) -> f64 {
        self.get(key, data).as_float().expect("Unable to convert {key}:{data} to float")
    }
}

pub fn import_config(config_file: &PathBuf) -> Result<ConfigStore, Errcode> {
    let default_config_str = include_str!("../default/config.toml");
    let default_config: toml::Value = toml::from_str(default_config_str)?;
    let default_config = default_config.as_table().unwrap().to_owned();
    if !config_file.exists() {
        std::fs::write(config_file, default_config_str)?;
        return Ok(Rc::new(ConfigDataStore {
            data: default_config,
        }));
    }
    assert!(config_file.is_file());

    let config: toml::Value = toml::from_str(std::fs::read_to_string(config_file)?.as_str())?;
    let mut config = config.as_table().unwrap().to_owned();
    for (key, val) in default_config.into_iter() {
        if !config.contains_key(&key) {
            config.insert(key, val);
        }
    }
    std::fs::write(config_file, toml::to_string(&config)?)?;
    Ok(Rc::new(ConfigDataStore { data: config }))
}