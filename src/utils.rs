use std::{
    collections::HashMap, fmt::Display, io::Write, marker::PhantomData, path::PathBuf, rc::Rc,
    str::FromStr,
};

use chrono::{DateTime, Datelike, Utc};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::errors::Errcode;

pub fn ask_user<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    std::io::stdin()
        .read_line(&mut res)
        .expect("Error in string entered");
    res.trim().to_string()
}

pub fn ask_user_parse<T: Display, U: FromStr>(question: T) -> Option<U> {
    loop {
        let res = ask_user(&question);
        if res.is_empty() {
            return None;
        }
        if let Ok(res) = res.parse() {
            return Some(res);
        }
        println!("Unable to parse reply to {:?}", std::any::type_name::<U>());
    }
}

pub fn ask_user_nonempty<T: Display>(question: T) -> String {
    print!("{question}");
    let _ = std::io::stdout().flush();
    let mut res = String::new();
    let mut init = true;
    while init || res.is_empty() {
        std::io::stdin()
            .read_line(&mut res)
            .expect("Error in string entered");
        if !init {
            println!("Answer is empty");
        }
        init = false;
    }
    res.trim().to_string()
}

pub fn map_get_str_or_ask<T: Display>(
    map: &serde_json::Map<String, serde_json::Value>,
    slug: &str,
    question: T,
    non_empty: bool,
) -> String {
    match map.get(slug).and_then(|n| n.as_str()) {
        Some(n) => n.to_string(),
        None => {
            if non_empty {
                ask_user_nonempty(question)
            } else {
                ask_user(question)
            }
        }
    }
}

pub type LangDict = Rc<LangWrapper>;
pub struct LangWrapper {
    data: toml::map::Map<String, toml::Value>,
}
impl LangWrapper {
    pub fn get_date_fmt(&self, date: &DateTime<Utc>) -> String {
        let month_list = self.data.get("months").unwrap().as_array().unwrap();
        let month_list: Vec<&str> = month_list.iter().map(|v| v.as_str().unwrap()).collect();
        let month_idx: usize = date.month().try_into().unwrap();
        format!(
            "{} {} {}",
            date.day(),
            month_list.get(month_idx).unwrap(),
            date.year(),
        )
    }

    pub fn get_doctype_word<D: ToString, W: ToString>(&self, doctype: D, word: W) -> String {
        let doctype = doctype.to_string();
        let word = word.to_string();
        self.data
            .get(&doctype)
            .unwrap()
            .as_table()
            .unwrap()
            .get(&word)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }
}

pub fn import_lang_profile(langf: &PathBuf) -> Result<LangDict, Errcode> {
    let data = if langf.is_file() {
        std::fs::read_to_string(langf)?
    } else {
        let s = include_str!("../default/lang.toml").to_string();
        std::fs::write(langf, &s)?;
        s
    };
    let data: toml::Value = toml::from_str(&data).expect("Error in lang definition");
    Ok(Rc::new(LangWrapper {
        data: data.as_table().unwrap().to_owned(),
    }))
}

pub fn serialize_hashmap_rc<K, T, S>(
    data: &HashMap<K, Rc<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
    K: Serialize,
{
    let mut map = serializer.serialize_map(Some(data.len()))?;
    for (k, v) in data.into_iter() {
        map.serialize_key(&k)?;
        map.serialize_value(v.as_ref())?;
    }
    map.end()
}

struct MapVisitor<K, V> {
    marker_key: PhantomData<K>,
    market_val: PhantomData<V>,
}

impl<K, V> MapVisitor<K, V> {
    fn new() -> MapVisitor<K, V> {
        MapVisitor {
            marker_key: PhantomData,
            market_val: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
where
    K: std::hash::Hash + Eq + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = HashMap<K, Rc<V>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Rc<Val>")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(1));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, Rc::new(value));
        }

        Ok(map)
    }
}

pub fn deserialize_hashmap_rc<'de, D, K, V>(deserialize: D) -> Result<HashMap<K, Rc<V>>, D::Error>
where
    D: Deserializer<'de>,
    K: std::hash::Hash + Eq + Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserialize.deserialize_map(MapVisitor::new())
}
