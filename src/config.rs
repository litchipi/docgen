use std::path::PathBuf;

use toml::map::Map;

use crate::errors::Errcode;

pub struct ConfigStore {
    data: Map<String, toml::Value>,
}

impl ConfigStore {
    pub fn get_company(&self, data: &str) -> String {
        self.data
            .get("company")
            .expect("Unable to get company data")
            .get(data)
            .unwrap_or_else(|| panic!("Unable to get data {data} from company"))
            .as_str()
            .unwrap_or_else(|| panic!("Unable to convert data {data} to String"))
            .to_string()
    }

    fn get_toml_value<'a>(&'a self, key: &str, data: &str) -> &'a toml::Value {
        let Some(table) = self.data.get(key) else {
            panic!("Unable to get table {key} from config");
        };

        let Some(res) = table.get(data) else {
            panic!("Unable to get data {key}:{data} from config");
        };

        res
    }

    pub fn get_bool(&self, key: &str, data: &str) -> bool {
        self.get_toml_value(key, data)
            .as_bool()
            .unwrap_or_else(|| panic!("Unable to convert {key}:{data} to boolean"))
    }

    pub fn get_float(&self, key: &str, data: &str) -> f64 {
        self.get_toml_value(key, data)
            .as_float()
            .unwrap_or_else(|| panic!("Unable to convert {key}:{data} to float"))
    }

    pub fn get_str<'a>(&'a self, key: &str, data: &str) -> &'a str {
        self.get_toml_value(key, data)
            .as_str()
            .unwrap_or_else(|| panic!("Unable to convert {key}:{data} to str"))
    }
}

pub fn import_config(config_file: &PathBuf) -> Result<ConfigStore, Errcode> {
    let default_config_str = include_str!("../default/config.toml");
    let default_config: toml::Value = toml::from_str(default_config_str)?;
    let default_config = default_config.as_table().unwrap().to_owned();
    if !config_file.exists() {
        std::fs::write(config_file, default_config_str)?;
        return Ok(ConfigStore {
            data: default_config,
        });
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
    Ok(ConfigStore { data: config })
}
